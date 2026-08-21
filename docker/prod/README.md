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

### Provision the `deploy` user

Run these commands on the Debian VPS while logged in as the existing
sudo-capable administrator. `authorized_keys` receives the **public** half of
the dedicated GitHub deployment key; never paste the private key onto the
server.

```sh
sudo adduser --disabled-password --gecos "" deploy
sudo install -d -o deploy -g deploy -m 0700 /home/deploy/.ssh
sudo vi /home/deploy/.ssh/authorized_keys
sudo chown deploy:deploy /home/deploy/.ssh/authorized_keys
sudo chmod 600 /home/deploy/.ssh/authorized_keys

sudo install -d -o deploy -g deploy -m 0750 \
  /srv/rocci/docker \
  /srv/rocci/incoming \
  /srv/rocci/releases
sudo usermod -aG docker deploy
```

Open a new SSH session after `usermod`; supplementary group membership is set
at login. Confirm Docker access before giving CI the deployment key:

```sh
sudo -iu deploy docker compose version
```

The current deployment scripts run `docker compose` as `deploy`, so Docker
group membership is intentional here. The Docker group is effectively
root-equivalent: protect the GitHub Environment, the deployment private key,
and the `deploy` account accordingly. A later hardening pass can replace this
with a root-owned, narrowly scoped deployment service.

Copy Compose/Caddy/`prod/` docs plus the `tools/rocci-ops` uv project (not
`site.tgz` / `islands`):

```sh
DEPLOY_HOST=ssh.rocci.dev DEPLOY_USER=deploy \
  uv run rocci-ops deploy bootstrap
```

The origin needs Python 3.12 and `uv` on `PATH`. Bootstrap copies the root
`pyproject.toml` / `uv.lock` plus `tools/rocci-ops` to `/srv/rocci`. Default
remote docker dir is `/srv/rocci/docker`. The `deploy`
user must write `/srv/rocci/{incoming,releases,current}` and call `docker compose`
without sudo. Provider firewall should keep 22 and 80/443 closed; CI SSHs through
Cloudflare Access (`ssh.rocci.dev`). From a laptop with Access, export
`CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`, and `CF_SSH_HOSTNAME=ssh.rocci.dev`
so `scp`/`ssh` use
[`access-ssh-proxy.sh`](access-ssh-proxy.sh) as `ProxyCommand`.

## Deploy from `main`

Do not copy artifacts by hand. `.github/workflows/site.yml` packages on
linux/amd64, then the `deploy` job (Environment `production` only, never
`pull_request`) probes SSH (`uv run rocci-ops deploy probe`),
bootstraps the origin kit, scps `site.tgz` / `islands`, and runs
`uv run rocci-ops origin publish SHA` on the box: unpack to `releases/<sha>/`,
`compose up -d --build`, GET `http://127.0.0.1:8080/health`, then flip `current`.
A failed health check leaves the previous symlink and restores that release.
Laptop one-shot:

```sh
uv run rocci-ops deploy push ARTIFACT_DIR SHA
```

Secrets (names only): `DEPLOY_HOST` (`ssh.rocci.dev`), `DEPLOY_USER`,
`DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`. Fork PRs
cannot read them. The deploy job runs `cloudflared access ssh --hostname
ssh.rocci.dev` as SSH `ProxyCommand`.

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

### Private staging

`staging.rocci.dev` is the pre-production route. It resolves publicly, but
Cloudflare Access must deny every request unless it matches an Access policy;
DNS alone does not make the origin reachable. Configure it in Zero Trust as
follows:

1. Create a **Self-hosted** Access application for `staging.rocci.dev`.
2. Add an **Allow** policy for the maintainer's personal email address, so a
   Mac browser can sign in normally.
3. Add a **Service Auth** policy for the existing GitHub deployment service
   token. This permits non-interactive CI smoke tests without giving CI an
   SSH login or making the site anonymous.
4. Add the Tunnel published application `staging.rocci.dev` ->
   `http://127.0.0.1:8080`, and attach the Access application.

Keep the Access application in place whenever this hostname is enabled. Test
in a private browser window: Cloudflare Access should require the maintainer
login before it serves the site. CI must pass the service token as
`CF-Access-Client-Id` and `CF-Access-Client-Secret`; do not put either value
in the repository. This hostname is a first-level subdomain so it is covered
by Cloudflare's normal `*.rocci.dev` certificate; a per-build
`<uuid>.staging.rocci.dev` name is not.

## Cloudflare cache

Add a Cache Rule: bypass cache for URI Path starts with `/actions/` or
equals `/health`. Hashed `/assets/` already send
`Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.

## SQLite backup

```sh
cd /srv/rocci && sudo uv run --no-dev rocci-ops origin backup /var/backups/rocci
```

That copies `site.db` out of volume `rocci-prod_islands-db`. Restore by
stopping `islands`, copying the file back onto the volume path
`/var/lib/rocci/site.db`, and starting again.

## Origin smoke (on the VPS)

```sh
curl -sf http://127.0.0.1:8080/health
```

Public `https://rocci.dev/` smoke waits until the Tunnel hostname is routed.
