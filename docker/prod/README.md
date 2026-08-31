# rocci.dev origin (hybrid Compose)

The VPS does not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit. CI
packages `site/`; this directory only **serves** those artifacts behind
Cloudflare Tunnel.

## Layout

`/srv/rocci` is a **parent** owned by `deploy`. Each GitHub Environment
(`staging` / `production`) is a complete origin tree under it. `site.yml`
sets `ROCCI_LANE` to the branch name. [`lanes.py`](../../rocci-ops/src/rocci_ops/lanes.py)
maps that to root, port, Compose project, image tag, and whether live
examples start. Explicit env vars override any field.

```text
/srv/rocci/                      deploy-owned parent (not an origin)
  prod/                          production lane  (:8080, no live examples)
    current -> releases/<sha>
    incoming/  releases/  docker/  tools/  pyproject.toml  uv.lock
  staging/                       staging lane     (:8081, live examples)
    current -> releases/<sha>
    incoming/  releases/  docker/  tools/  pyproject.toml  uv.lock
```

| Lane | Branch / `ROCCI_LANE` | Root | Host port | Compose project | Image tag | Live examples |
| --- | --- | --- | --- | --- | --- | --- |
| production | `production` | `/srv/rocci/prod` | `8080` | `rocci-prod` | `prod` | no |
| staging | `staging` | `/srv/rocci/staging` | `8081` | `rocci-staging` | `staging` | yes |

Inside each lane root:

| Path | Role |
| --- | --- |
| `current` | Symlink to `releases/<sha>` after a healthy publish |
| `releases/<sha>/dist/` | Unpacked CDN tree |
| `releases/<sha>/islands-context/` | `Dockerfile` plus the musl `islands` binary |
| `releases/<sha>/examples-live/<id>/` | Live app context (staging compose only) |
| `docker/` | Hybrid + origin Compose, Caddy, image Dockerfiles |
| `rocci-ops/` | `uv run rocci-ops origin publish` |

Docker **volumes** stay project-prefixed (`rocci-prod_islands-db`,
`rocci-staging_live-counter-db`, …). Image tags are `rocci-islands:prod` /
`:staging` (not a shared `:local`) so a staging rebuild cannot retag
production.

Copy [`env.example`](env.example) to `/srv/rocci/prod/.env` or
`/srv/rocci/staging/.env` only if you invoke Compose by hand.

### Leftovers (do not treat as origins)

A pre-lane checkout put `current`, `releases`, `incoming`, `docker`, and
`tools` **directly under** `/srv/rocci`. A first cutover attempt also
created `/srv/rocci-staging`. Those are not lane roots.

- CI writes **only** `/srv/rocci/prod` and `/srv/rocci/staging`.
- Do not run `origin publish` from `/srv/rocci` itself.
- After migration, delete `/srv/rocci-staging` and move the old siblings
  into `prod/` (see [Migrate the shared origin](#migrate-the-shared-origin)).

## Bootstrap (once)

### Provision the `deploy` user

Run these on the Debian VPS as the sudo-capable administrator.
`authorized_keys` receives the **public** half of the dedicated GitHub
deployment key; never paste the private key onto the server.

```sh
sudo adduser --disabled-password --gecos "" deploy
sudo install -d -o deploy -g deploy -m 0700 /home/deploy/.ssh
sudo vi /home/deploy/.ssh/authorized_keys
sudo chown deploy:deploy /home/deploy/.ssh/authorized_keys
sudo chmod 600 /home/deploy/.ssh/authorized_keys

sudo install -d -o deploy -g deploy -m 0750 /srv/rocci
sudo -iu deploy install -d -m 0750 \
  /srv/rocci/prod/docker \
  /srv/rocci/prod/incoming \
  /srv/rocci/prod/releases \
  /srv/rocci/prod/tools \
  /srv/rocci/staging/docker \
  /srv/rocci/staging/incoming \
  /srv/rocci/staging/releases \
  /srv/rocci/staging/tools
sudo usermod -aG docker deploy
```

`deploy` must own `/srv/rocci` itself so it can create lane children and
write `tools/` / `pyproject.toml`. Owning only `docker/` / `incoming/`
under a root-owned parent causes `mkdir: Permission denied`.

Open a new SSH session after `usermod`. Confirm Docker access before
giving CI the deployment key:

```sh
sudo -iu deploy docker compose version
```

The current scripts run `docker compose` as `deploy`. The Docker group is
effectively root-equivalent: protect the GitHub Environment, the
deployment private key, and the `deploy` account.

### Install Python and `uv`

```sh
sudo apt install -y python3 python3-pip python3-venv
sudo env UV_INSTALL_DIR="/usr/local/bin" sh -c "$(curl -LsSf https://astral.sh/uv/install.sh)"
```

Debian 12's system `python3` can be older than 3.12. If needed:

```sh
sudo -iu deploy uv python install 3.12
sudo -iu deploy uv python find 3.12
```

`uv` is system-wide; its managed Python is stored for `deploy`, not under
`/srv/rocci`. This does not put the Rocci/Roc toolchain on the VPS.

Laptop bootstrap (kit only; CI already does this as part of `deploy push`):

```sh
DEPLOY_HOST=ssh.rocci.dev DEPLOY_USER=deploy ROCCI_LANE=staging \
  uv run rocci-ops deploy bootstrap
```

That streams **one** gzip tar over one SSH connection into the lane root
(Compose, Caddy, `rocci-ops`, `pyproject.toml` / `uv.lock`).
`deploy push` adds `incoming/<sha>/` to the same tar and runs
`origin publish` in that same session.

Provider firewall should keep 22 and 80/443 closed. CI SSHs through
Cloudflare Access (`ssh.rocci.dev`). From a laptop with Access, export
`CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`, and
`CF_SSH_HOSTNAME=ssh.rocci.dev` so `scp`/`ssh` use
[`access-ssh-proxy.sh`](access-ssh-proxy.sh) as `ProxyCommand`.

## Deploy from `staging` and `production`

Do not copy artifacts by hand. Land pull requests on `main`. Promote
`staging` first (Access-gated), smoke it, retarget the Tunnel, then
promote `production`:

```sh
uv run rocci-ops promote staging
# smoke :8081 and Access, then retarget Tunnel staging hosts to :8081
uv run rocci-ops promote production
```

`promote production` is `git push origin origin/staging:refs/heads/production`.
It creates `production` on first use. Do not run it while staging Tunnel
hostnames still point at `:8080` — those names would start serving the
production stack. `promote staging` merges `origin/main` into `staging`
(fast-forward only vs `origin/staging`). Do not rebase or force-push
`staging`. Equivalent raw git:

```sh
git fetch origin
git switch staging
git merge --ff-only origin/staging
git merge origin/main -m "Promote main into staging"
git push origin staging
git push origin origin/staging:production
```

A push to `staging` or `production` runs `.github/workflows/site.yml`:
package on linux/amd64, then the `deploy` job (Environment = branch name,
never `pull_request`, never `main`) probes SSH and streams one tar +
`origin publish`. Unpack to `releases/<sha>/`,
`compose up -d --build --remove-orphans`.

- Production: hybrid only (`ROCCI_PUBLISH_LIVE=0`). Health is
  `http://127.0.0.1:8080/health`. Caddy uses the site
  [`cdn/Caddyfile`](../cdn/Caddyfile) with an empty examples stub.
- Staging: hybrid plus [`compose.origin.yml`](../compose.origin.yml).
  Health is `:8081` `/health`, `/play/<id>/health`, and
  `Host: <id>-example-staging.rocci.dev` plus `<id>.examples.localhost`.
  Caddy remounts [`cdn/examples.caddy`](../cdn/examples.caddy).

A failed health check leaves the previous `current` symlink and restores
that release. Origin deploys stay serialized (`rocci-dev-origin`).
**Run workflow** on those branches matches a push. Any other ref is a
no-op. Laptop one-shot:

```sh
uv run rocci-ops deploy push ARTIFACT_DIR SHA
```

Do not run [`compose.examples.yml`](../compose.examples.yml) `edge` on
this box (`:8080` / `:8081` are hybrid Caddy).

Secrets (names only): `DEPLOY_HOST` (`ssh.rocci.dev`), `DEPLOY_USER`,
`DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`. Fork
PRs cannot read them. Create GitHub Environments `staging` and
`production` with those names. Environment-only; custom branch policy per
matching branch. Copy the same secret names into both Environments;
`ROCCI_LANE` selects the root. The job uses `cloudflared access ssh
--hostname ssh.rocci.dev` as `ProxyCommand` and always `shred`/`rm`s
`$HOME/.ssh/deploy`.

After the repository is public, enable a `main` / `staging` / `production`
ruleset. Do not require the CI check on pull requests. Leave the Actions
default token read-only. Require approval for workflows from outside
collaborators.

Confirm the island image has no Roc (tag matches the lane):

```sh
docker run --rm --entrypoint /bin/sh rocci-islands:prod -c 'which roc'; echo $?
docker run --rm --entrypoint /bin/sh rocci-islands:staging -c 'which roc'; echo $?
```

`which roc` must fail.

## Tunnel

Zero Trust Public Hostname UI, or merge
[`cloudflared-ingress.yml.example`](cloudflared-ingress.yml.example).
Do not open provider firewall 80/443.

| Public hostname | Loopback |
| --- | --- |
| `rocci.dev`, `www.rocci.dev` | `http://127.0.0.1:8080` |
| `staging.rocci.dev` | `http://127.0.0.1:8081` |
| `live-counter-example-staging.rocci.dev` | `http://127.0.0.1:8081` |
| `datastar-example-staging.rocci.dev` | `http://127.0.0.1:8081` |
| `snake-example-staging.rocci.dev` | `http://127.0.0.1:8081` |
| `*.examples.staging.rocci.dev` | `http://127.0.0.1:8081` |

Do not publish `<id>-example.rocci.dev` or `*.examples.rocci.dev` until
Launch is advertised.

Cloudflare `*.rocci.dev` does **not** cover `*.examples.rocci.dev`. Add
DNS CNAMEs for example wildcards to the Tunnel, then issue certificate
coverage (Total TLS or Advanced). Do not advertise those names until a
staging deploy has served them with TLS.

### Private staging

`staging.rocci.dev` resolves publicly; Cloudflare Access must deny every
request that does not match a policy.

1. Create a **Self-hosted** Access application for `staging.rocci.dev`.
2. **Allow** the maintainer's personal email.
3. **Service Auth** for the GitHub deployment token (CI smoke without SSH).
4. Tunnel published application `staging.rocci.dev` →
   `http://127.0.0.1:8081`, attach Access.
5. Repeat Access for `live-counter-example-staging.rocci.dev`,
   `datastar-example-staging.rocci.dev`, and
   `snake-example-staging.rocci.dev`. Optional later:
   `*.examples.staging.rocci.dev` if ACM is on.

Keep Access on whenever the hostname is enabled. Signed-out browser:
Access login. CI sends `CF-Access-Client-Id` and
`CF-Access-Client-Secret`; do not put those in the repository.
`staging.rocci.dev` is covered by `*.rocci.dev`; example hosts are not.

## Cloudflare cache

Cache Rule: bypass URI Path starts with `/actions/` or equals `/health`.
Hashed `/assets/` already send
`Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.

## SQLite backup

```sh
cd /srv/rocci/prod && sudo uv run --no-dev rocci-ops origin backup /var/backups/rocci
cd /srv/rocci/staging && sudo ROCCI_LANE=staging uv run --no-dev rocci-ops origin backup /var/backups/rocci-staging
```

That copies `site.db` out of `<project>_islands-db`. Restore by stopping
islands, copying the file back to `/var/lib/rocci/site.db`, and starting
again.

## Migrate the shared origin

Use this when `/srv/rocci` still has `current`, `releases`, `incoming`,
`docker`, or `tools` as **siblings** of `prod/` and `staging/` (the
2026-08 layout before lanes). Staging can already be live on `:8081`
while production still serves from the old tree on `:8080`.

As `deploy` (or `sudo -iu deploy`):

```sh
# 1. Lane dirs exist (Bootstrap).
# 2. Move the old origin into prod/ if those names are not already there.
for name in current incoming releases docker tools pyproject.toml uv.lock .python-version .env; do
  if [ -e "/srv/rocci/$name" ] && [ ! -e "/srv/rocci/prod/$name" ]; then
    mv "/srv/rocci/$name" "/srv/rocci/prod/"
  fi
done

# 3. Drop the unused sibling from the first cutover attempt.
# sudo rm -rf /srv/rocci-staging
```

Do not move `prod` or `staging` into themselves. An old `/srv/rocci/.venv`
can stay or be removed; publish runs `uv` from the lane root.

Then:

1. Confirm `curl -sf http://127.0.0.1:8081/health` (staging already
   published).
2. Retarget Tunnel staging hostnames to `:8081`. Smoke through Access.
3. `uv run rocci-ops promote production` — republishes `/srv/rocci/prod`
   on `:8080` without live-example containers. Keep
   `rocci-prod_islands-db`. Leave    `rocci-prod_live-counter-db` /
   `rocci-prod_datastar-db` / `rocci-prod_snake-db` until staging smoke is
   done; then prune if
   unused.

## Origin smoke (on the VPS)

Production (`:8080`):

```sh
curl -sf http://127.0.0.1:8080/health
```

Staging (`:8081`):

```sh
curl -sf http://127.0.0.1:8081/health
curl -sf http://127.0.0.1:8081/play/live-counter/health
curl -sf http://127.0.0.1:8081/play/datastar/health
curl -sf http://127.0.0.1:8081/play/snake/health
curl -sf -H 'Host: live-counter-example-staging.rocci.dev' http://127.0.0.1:8081/health
curl -sf -H 'Host: datastar-example-staging.rocci.dev' http://127.0.0.1:8081/health
curl -sf -H 'Host: snake-example-staging.rocci.dev' http://127.0.0.1:8081/health
curl -sf -H 'Host: live-counter.examples.localhost' http://127.0.0.1:8081/health
curl -sf -H 'Host: datastar.examples.localhost' http://127.0.0.1:8081/health
curl -sf -H 'Host: staging.rocci.dev' \
  -X POST http://127.0.0.1:8081/actions/counter/increment \
  -H 'datastar-request: true' -H 'content-type: application/json' -d '{}'
```

Public `https://rocci.dev/` smoke waits until the Tunnel hostname is
routed to `:8080`.
