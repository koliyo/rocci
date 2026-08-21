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
| `releases/<sha>/blocks-context/` | `Dockerfile` plus musl `server` and `assets/` |
| Docker volume `rocci-prod_islands-db` | Persistent SQLite at `/var/lib/rocci/site.db` inside `islands` |
| Docker volume `rocci-prod_blocks-db` | Persistent SQLite at `/var/lib/rocci/blocks.db` inside `blocks` |

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

### Install Python and `uv`

The origin runs the `rocci-ops` operator package, which requires Python 3.12
or later. Install the base Python tooling and `uv` as the sudo-capable
administrator:

```sh
sudo apt install -y python3 python3-pip python3-venv
sudo env UV_INSTALL_DIR="/usr/local/bin" sh -c "$(curl -LsSf https://astral.sh/uv/install.sh)"
```

Debian 12's system `python3` can be older than the required version. If required, install
the managed Python for the account that will run deployment commands:

```sh
sudo -iu deploy uv python install 3.12
sudo -iu deploy uv python find 3.12
```

`uv` is installed system-wide; its managed Python is deliberately stored for
`deploy`, not in `/srv/rocci`. This does not put the Rocci/Roc toolchain on the
VPS.

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

## Deploy from `staging` and `production`

Do not copy artifacts by hand. Land pull requests on `main`. Promote in
two steps: `staging` first (Access-gated `staging.rocci.dev`), then
`production` (public hostname once the Tunnel route exists):

```sh
git fetch origin
git push origin origin/main:staging
git push origin origin/staging:production
```

A push to either branch runs `.github/workflows/site.yml`: it packages on
linux/amd64, then the `deploy` job (GitHub Environment named after the
branch, never `pull_request` and never `main`) probes SSH
(`uv run rocci-ops deploy probe`), bootstraps the origin kit, scps
`site.tgz` / `islands` / `blocks` / `blocks-assets.tgz`, and runs `uv run rocci-ops origin publish SHA` on
the box: unpack to `releases/<sha>/`, `compose up -d --build` with Compose
profile `blocks` when that release has a Blocks server, GET
`http://127.0.0.1:8080/health` and `http://127.0.0.1:8080/health/blocks`, then flip `current`. A failed health check
leaves the previous symlink and restores that release. Spectator admission
defaults to 20 (`BLOCKS_SPECTATOR_CAP`). Both branches
currently publish the same `/srv/rocci` origin; origin deploys are
serialized so they cannot interleave. **Run workflow** on `staging` or
`production` packages and deploys the same way as a push; on any other
ref it packages only.
Laptop one-shot:

```sh
uv run rocci-ops deploy push ARTIFACT_DIR SHA
```

Secrets (names only): `DEPLOY_HOST` (`ssh.rocci.dev`), `DEPLOY_USER`,
`DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`. Fork PRs
cannot read them. Create GitHub Environments `staging` and `production`
with those same names. Restrict each Environment to its matching branch so
a workflow_dispatch on another ref cannot use those secrets. Copy the
values into `staging` even while both lanes share one VPS. The deploy job
runs `cloudflared access ssh --hostname ssh.rocci.dev` as SSH
`ProxyCommand`.

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

`staging.rocci.dev` is the pre-production route for the `staging` branch.
It resolves publicly, but Cloudflare Access must deny every request unless
it matches an Access policy; DNS alone does not make the origin reachable. Configure it in Zero Trust as
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

Add a Cache Rule: bypass cache for URI Path starts with `/actions/`,
`/play/blocks/`, or equals `/health` or `/health/blocks`. Hashed `/assets/` already send
`Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.

## SQLite backup

```sh
cd /srv/rocci && sudo uv run --no-dev rocci-ops origin backup /var/backups/rocci
```

That copies `site.db` out of volume `rocci-prod_islands-db` and `blocks.db`
out of `rocci-prod_blocks-db`. Restore by stopping the matching service,
copying the file back onto `/var/lib/rocci/site.db` or
`/var/lib/rocci/blocks.db`, and starting again.

## Rocci Blocks (experimental)

Same-origin `/play/blocks/` is a first-party arena, not a generic live-example
hostname. Origin publish starts Compose profile `blocks` (512 MiB, 1 CPU,
SQLite volume `blocks-db`) at spectator cap **20**. Public copy says
“falling-block arena.”

Watch on the VPS:

```sh
curl -sf http://127.0.0.1:8080/health
curl -sf http://127.0.0.1:8080/health/blocks
docker compose -f /srv/rocci/docker/compose.hybrid.yml --profile blocks logs --tail=100 blocks
docker stats --no-stream
```

Look for lock 409/429 bursts, stream reconnects, SQLite busy, CPU, memory, and
egress. Lower `BLOCKS_SPECTATOR_CAP` or roll back the previous release rather
than weakening lock validation. A 24-hour public observation is an operator
gate; shipping the route in git is not that observation.

## Origin smoke (on the VPS)

```sh
curl -sf http://127.0.0.1:8080/health
curl -sf http://127.0.0.1:8080/health/blocks
```

Public `https://rocci.dev/` smoke waits until the Tunnel hostname is routed.
