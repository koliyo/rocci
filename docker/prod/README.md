# rocci.dev origin (hybrid Compose)

The VPS does not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit. CI (or a
toolchain host) packages `site/`; this directory only **serves** those
artifacts behind Cloudflare Tunnel.

## Layout on the box

Default `ROCCI_ORIGIN_ROOT=/srv/rocci`:

| Path | Role |
| --- | --- |
| `current/dist/` | Unpacked CDN tree (`index.html`, hashed `/assets/`, `publish.json`) |
| `current/islands-context/` | `Dockerfile` plus the musl `islands` binary (Compose build context) |
| Docker volume `rocci-prod_islands-db` | Persistent SQLite at `/var/lib/rocci/site.db` inside `islands` |

`ROCCI_DIST` and `ROCCI_ISLANDS_CONTEXT` must be absolute. Copy
[`env.example`](env.example) to `/srv/rocci/.env` if you invoke Compose by
hand.

## First publish

On a machine with the linux/amd64 artifacts (`site.tgz` and `islands` from
the `site.yml` workflow):

```sh
mkdir -p /tmp/rocci-dist
tar -xzf site.tgz -C /tmp/rocci-dist
# site.tgz is rooted at the dist folder; adjust if the archive has a prefix.
sudo ./docker/prod/up.sh /tmp/rocci-dist ./islands
```

`up.sh` copies the tree and binary under `/srv/rocci/current`, builds
`rocci-islands:local` and `rocci-cdn:local`, and `compose up -d` using
[`compose.hybrid.yml`](../compose.hybrid.yml) and
[`cdn/Caddyfile`](../cdn/Caddyfile). Caddy listens on `127.0.0.1:8080`.

Confirm the island image has no Roc:

```sh
docker run --rm --entrypoint /bin/sh rocci-islands:local -c 'which roc'; echo $?
```

`which roc` must fail.

## Tunnel

`cloudflared` is already installed. Point the named Tunnel at Caddy (Zero
Trust Public Hostname UI, or merge [`cloudflared-ingress.yml.example`](cloudflared-ingress.yml.example)
into the service config). Map `rocci.dev` and `www.rocci.dev` to
`http://127.0.0.1:8080`. Do not open provider firewall 80/443.

## Cloudflare cache

Add a Cache Rule: bypass cache for URI Path starts with `/actions/` or
equals `/health`. Hashed `/assets/` already send
`Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.

## SQLite backup

```sh
sudo ./docker/prod/backup-sqlite.sh /var/backups/rocci
```

That copies `site.db` out of volume `rocci-prod_islands-db`. Restore by
stopping `islands`, copying the file back onto the volume path
`/var/lib/rocci/site.db`, and starting again.

## Smoke (from a machine that is not the VPS)

```sh
curl -sfI https://rocci.dev/
curl -sf https://rocci.dev/health
curl -sf -X POST https://rocci.dev/actions/counter/increment \
  -H 'datastar-request: true' -H 'content-type: application/json' \
  -d '{"tz":"Europe/Stockholm","handle":"Coral Lynx"}'
```
