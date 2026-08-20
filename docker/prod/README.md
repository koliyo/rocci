# rocci.dev origin (hybrid Compose)

The VPS does not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit. CI (or a
toolchain host) packages `site/`; this directory only **serves** those
artifacts behind Cloudflare Tunnel.

## Layout on the box

Default `ROCCI_ORIGIN_ROOT=/srv/rocci`:

| Path | Role |
| --- | --- |
| `current` | Symlink to `releases/<sha>` after a healthy publish |
| `releases/<sha>/dist/` | Unpacked CDN tree |
| `releases/<sha>/islands-context/` | `Dockerfile` plus the musl `islands` binary |
| Docker volume `rocci-prod_islands-db` | Persistent SQLite at `/var/lib/rocci/site.db` inside `islands` |

`ROCCI_DIST` and `ROCCI_ISLANDS_CONTEXT` must be absolute. Copy
[`env.example`](env.example) to `/srv/rocci/.env` if you invoke Compose by
hand.

## Bootstrap (once)

Copy Compose/Caddy/`prod/` (not `site.tgz` / `islands`):

```sh
DEPLOY_HOST=… DEPLOY_USER=deploy ./docker/prod/bootstrap-rsync.sh
```

Default remote dir is `/srv/rocci/docker`. The `deploy` user must be able to
`docker compose` and write `/srv/rocci/{incoming,releases,current}`. SSH from
GitHub-hosted runners must be allowed on the provider firewall (key-only).

## Deploy from `main`

Do not scp artifacts by hand. `.github/workflows/site.yml` packages on
linux/amd64, then the `deploy` job (Environment `production` only, never
`pull_request`) runs [`push-release.sh`](push-release.sh). That rsyncs the
origin kit plus `site.tgz` / `islands` and runs [`publish.sh`](publish.sh)
on the box: unpack to `releases/<sha>/`, `compose up -d --build`, curl
`http://127.0.0.1:8080/health`, then flip `current`. A failed health check
leaves the previous symlink and restores that release.

Secrets (names only): `DEPLOY_HOST`, `DEPLOY_USER`, `DEPLOY_SSH_KEY`. Fork
PRs cannot read them.

Caddy listens on `127.0.0.1:8080` via
[`compose.hybrid.yml`](../compose.hybrid.yml) and
[`cdn/Caddyfile`](../cdn/Caddyfile).

Confirm the island image has no Roc:

```sh
docker run --rm --entrypoint /bin/sh rocci-islands:local -c 'which roc'; echo $?
```

`which roc` must fail.

## Tunnel

After origin health succeeds, point the named Tunnel at Caddy (Zero
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

## Origin smoke (on the VPS)

```sh
curl -sf http://127.0.0.1:8080/health
```

Public `https://rocci.dev/` smoke waits until the Tunnel hostname is routed.
