# rocci.dev origin (hybrid Compose)

The VPS does not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit. CI (or a
toolchain host) packages `site/`; this directory only **serves** those
artifacts behind Cloudflare Tunnel.

## Layout on the box

Two lanes on one VPS. `ROCCI_LANE` (from `site.yml` = branch name) selects the
root, port, Compose project, image tag, and whether live-example containers
start. Explicit env vars override any field.

| Lane | Root | Port | Compose project | Live examples |
| --- | --- | --- | --- | --- |
| `production` | `/srv/rocci` | `8080` | `rocci-prod` | no |
| `staging` | `/srv/rocci-staging` | `8081` | `rocci-staging` | yes |

Each root:

| Path | Role |
| --- | --- |
| `current` | Symlink to `releases/<sha>` after a healthy publish |
| `releases/<sha>/dist/` | Unpacked CDN tree |
| `releases/<sha>/islands-context/` | `Dockerfile` plus the musl `islands` binary |
| `releases/<sha>/examples-live/<id>/` | Live app Docker context (staging compose only) |
| Docker volume `<project>_islands-db` | Persistent SQLite at `/var/lib/rocci/site.db` inside `islands` |
| Docker volume `<project>_live-counter-db` | Staging only: live-counter tenant |
| Docker volume `<project>_datastar-db` | Staging only: datastar tenant |

Image tags are `rocci-islands:prod` / `:staging` (not a shared `:local`) so a
staging rebuild cannot retag production. `ROCCI_DIST` and
`ROCCI_ISLANDS_CONTEXT` must be absolute. Copy [`env.example`](env.example) to
`/srv/rocci/.env` or `/srv/rocci-staging/.env` if you invoke Compose by hand.

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
  /srv/rocci/releases \
  /srv/rocci-staging/docker \
  /srv/rocci-staging/incoming \
  /srv/rocci-staging/releases
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
uv run rocci-ops promote staging
uv run rocci-ops promote production
```

`promote production` is `git push origin origin/staging:refs/heads/production`.
It creates `production` on first use. That push runs hosted CI and Knowledge
and then this workflow's package/deploy jobs. Do not run it until staging has
been smoked signed-out. `promote staging` merges `origin/main` into `staging` and pushes (fast-forward only vs `origin/staging`). Do not rebase or force-push `staging`. Equivalent raw git:

```sh
git fetch origin
git switch staging
git merge --ff-only origin/staging
git merge origin/main -m "Promote main into staging"
git push origin staging
git push origin origin/staging:production
```

A push to either branch runs `.github/workflows/site.yml`: it packages on
linux/amd64, then the `deploy` job (GitHub Environment named after the
branch, never `pull_request` and never `main`) sets `ROCCI_LANE` to the
branch name, probes SSH (`uv run rocci-ops deploy probe`), bootstraps that
lane's origin kit, scps `site.tgz` / `islands` / `examples-live/`, and runs
`uv run rocci-ops origin publish SHA` on the box with lane env exported.
Unpack to `releases/<sha>/`, `compose up -d --build --remove-orphans`.
Production is hybrid only (`ROCCI_PUBLISH_LIVE=0`) and probes
`http://127.0.0.1:8080/health`. Staging merges
[`compose.origin.yml`](../compose.origin.yml) and probes `:8081` `/health`,
`/play/<id>/health`, plus `Host: <id>-example-staging.rocci.dev` and
`<id>.examples.localhost`. A failed health check leaves the previous
symlink and restores that release. Origin deploys stay serialized
(`rocci-dev-origin`) so two `compose up --build`s do not fight the VPS.
**Run workflow** on `staging` or `production` packages and deploys the same
way as a push. On any other ref, both package and deploy are no-ops.
Laptop one-shot:

```sh
uv run rocci-ops deploy push ARTIFACT_DIR SHA
```

Secrets (names only): `DEPLOY_HOST` (`ssh.rocci.dev`), `DEPLOY_USER`,
`DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`. Fork PRs
cannot read them. Create GitHub Environments `staging` and `production`
with those same names. Keep those values Environment-only; do not copy them
to repository secrets. Restrict each Environment with a **custom branch
policy** for its matching branch (`staging` only, `production` only). A
`protected_branches` toggle is not enough on a free private repo, and a
workflow_dispatch on another ref must not be able to use those secrets.
Copy the same secret names into both Environments; `ROCCI_LANE` selects
the origin root on the shared VPS. The
deploy job runs on `ubuntu-latest` and uses `cloudflared access ssh`
--hostname ssh.rocci.dev` as SSH `ProxyCommand`. After writing
`$HOME/.ssh/deploy`, the job always `shred`/`rm`s that file.

After the repository is public, enable a `main` / `staging` / `production`
ruleset (available on public repos without Pro). Do not require the CI
check on pull requests. Leave the Actions default token read-only, and do
not grant Actions the right to approve reviews. Require approval for
workflows from all outside collaborators.

Production Caddy listens on `127.0.0.1:8080` via
[`compose.hybrid.yml`](../compose.hybrid.yml) and the site
[`cdn/Caddyfile`](../cdn/Caddyfile) (examples snippet is a stub). Staging
Caddy is `:8081` and remounts [`cdn/examples.caddy`](../cdn/examples.caddy)
through [`compose.origin.yml`](../compose.origin.yml). Do not run
[`compose.examples.yml`](../compose.examples.yml) `edge` on this box.

Confirm the island image has no Roc:

```sh
docker run --rm --entrypoint /bin/sh rocci-islands:local -c 'which roc'; echo $?
```

`which roc` must fail.

## Tunnel

After origin health succeeds, point the named Tunnel at Caddy (Zero
Trust Public Hostname UI, or merge [`cloudflared-ingress.yml.example`](cloudflared-ingress.yml.example)
into the service config). Map `rocci.dev` and `www.rocci.dev` to
`http://127.0.0.1:8080`. Map `staging.rocci.dev`,
`live-counter-example-staging.rocci.dev`,
`datastar-example-staging.rocci.dev`, and `*.examples.staging.rocci.dev`
to `http://127.0.0.1:8081`. Do not publish `<id>-example.rocci.dev` or
`*.examples.rocci.dev` until Launch is advertised. Do not open provider
firewall 80/443.

Cloudflare `*.rocci.dev` does **not** cover `*.examples.rocci.dev`. Add
DNS CNAMEs for both example wildcards to the Tunnel, then issue certificate
coverage for `*.examples.rocci.dev` and `*.examples.staging.rocci.dev`
(Total TLS or an advanced certificate). Do not advertise those hostnames
from the site until a staging deploy has served them with TLS.

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
   `http://127.0.0.1:8081`, and attach the Access application.
5. Repeat Access for `live-counter-example-staging.rocci.dev` and
   `datastar-example-staging.rocci.dev` (same Allow + Service Auth).
   Those first-level names use Universal SSL. Optional later: the same
   for `*.examples.staging.rocci.dev` if ACM is on. Staging example
   hosts stay gated like `staging.rocci.dev`.

Keep the Access application in place whenever this hostname is enabled. Test
in a private browser window: Cloudflare Access should require the maintainer
login before it serves the site. CI must pass the service token as
`CF-Access-Client-Id` and `CF-Access-Client-Secret`; do not put either value
in the repository. `staging.rocci.dev` is a first-level subdomain so it is
covered by Cloudflare's normal `*.rocci.dev` certificate; example hosts are
not.

## Cloudflare cache

Add a Cache Rule: bypass cache for URI Path starts with `/actions/`
or equals `/health`. Hashed `/assets/` already send
`Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.

## SQLite backup

```sh
cd /srv/rocci && sudo uv run --no-dev rocci-ops origin backup /var/backups/rocci
cd /srv/rocci-staging && sudo ROCCI_LANE=staging uv run --no-dev rocci-ops origin backup /var/backups/rocci-staging
```

That copies `site.db` out of `<project>_islands-db`. Restore by
stopping the islands service, copying the file back onto
`/var/lib/rocci/site.db`, and starting again.

## First cutover (existing shared origin)

1. Create `/srv/rocci-staging/{docker,incoming,releases}` as `deploy` (see
   Bootstrap).
2. Promote `staging` so `site.yml` publishes to `:8081`. Leave Cloudflare on
   `:8080` until that stack is healthy.
3. Retarget staging Tunnel hostnames to `http://127.0.0.1:8081`.
4. Promote `production` so `:8080` republishes without live-example
   containers. Keep `rocci-prod_islands-db`. Leave
   `rocci-prod_live-counter-db` / `rocci-prod_datastar-db` until staging
   smoke is done; then prune if unused.

## Origin smoke (on the VPS)

```sh
curl -sf http://127.0.0.1:8080/health
curl -sf http://127.0.0.1:8081/health
curl -sf http://127.0.0.1:8081/play/live-counter/health
curl -sf http://127.0.0.1:8081/play/datastar/health
curl -sf -H 'Host: live-counter-example-staging.rocci.dev' http://127.0.0.1:8081/health
curl -sf -H 'Host: datastar-example-staging.rocci.dev' http://127.0.0.1:8081/health
curl -sf -H 'Host: live-counter.examples.localhost' http://127.0.0.1:8081/health
curl -sf -H 'Host: datastar.examples.localhost' http://127.0.0.1:8081/health
curl -sf -H 'Host: staging.rocci.dev' \
  -X POST http://127.0.0.1:8081/actions/counter/increment \
  -H 'datastar-request: true' -H 'content-type: application/json' -d '{}'
```

Public `https://rocci.dev/` smoke waits until the Tunnel hostname is routed.
