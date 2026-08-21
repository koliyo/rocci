# Host a Rocdown site with Docker

Build the site on a toolchain host. Local Docker then serves only the compiled
artifacts. Official Caddy has no `rocci`, `rocdown`, `roc`, rustc, or WebKit.

## Choosing `--target`

`--target` is Roc’s process ISA/OS for the **Linux container** binary, not the
machine that runs `package` / `build --release`. Match the container CPU:

| Where the binary runs | Use |
| --- | --- |
| Docker Desktop on Apple Silicon (default `linux/arm64`) | `arm64musl` |
| Docker on Intel Mac, most CI, typical `amd64` Linux VMs | `x64musl` |
| Native macOS process (`.app`, local `rocci run`) | omit `--target` (host-native; not musl) |

Compose does **not** choose the image architecture from `--target`. The slim
Dockerfiles pull `debian:bookworm-slim` for Docker’s **default platform**
(usually `linux/arm64` on Apple Silicon, `linux/amd64` on Intel). You must
build a musl binary that matches that platform—or pin both sides yourself.

On Apple Silicon you can run either path:

```sh
# Native arm64 containers (default, fastest)
cargo run -q -p rocci-rocdown-cli -- package SITE --target arm64musl
uv run rocci-ops serve hybrid DIST ISLANDS

# amd64 containers via Docker’s x86_64 emulation (Rosetta/QEMU)
cargo run -q -p rocci-rocdown-cli -- package SITE --target x64musl
DOCKER_DEFAULT_PLATFORM=linux/amd64 uv run rocci-ops serve hybrid DIST ISLANDS
```

A mismatch (`x64musl` binary in an arm64 image, or the reverse) fails at
container start with an exec-format error. Check the engine default:

```sh
docker info --format '{{.Architecture}}'
# aarch64 → default images are arm64 → prefer arm64musl
# x86_64  → default images are amd64 → prefer x64musl
```

Apple Silicon does **not** mean `arm64mac` for Docker. `arm64mac` / `x64mac`
are macOS process targets. Musl names (`*musl`) are Linux and are what the
slim hybrid/app images expect. Prefer musl over glibc for portable static
Linux binaries; see `rocci build --help` / `rocdown package --help` for the
full Roc list.

## Static site (default hosting)

Host a pre-built Rocdown tree (`rocdown build --cdn-only`). There is no island
proxy.

From the repository root, dogfood `docs/` (`build.output = "../dist/docs"`):

```sh
cargo run -q -p rocci-rocdown-cli -- build docs --cdn-only
uv run rocci-ops serve static dist/docs
```

The script absolutizes `ROCCI_DIST` and runs Compose. Extra arguments go to
`compose up` (`-d`, …). The CDN container prints
`Open http://127.0.0.1:8080/` (or `ROCCI_HTTP_PORT`) when it starts.

`ROCCI_DIST` must be absolute. Interpolate it yourself if you call Compose
directly:

```sh
ROCCI_DIST="$(cd dist/docs && pwd)" docker compose -f docker/compose.static.yml up
```

`docker compose -f docker/compose.static.yml build` does not compile Rocci; it
only builds the thin `rocci-cdn` image (Caddy plus startup banner).

Hashed `/assets/` files are immutable. HTML uses `no-cache`. `/actions/` is
not proxied (Caddy 404). This path rejects `live` pages at build time
(`RD2302`).

## Hybrid live-island hosting (pre-built)

Package on the host, then Caddy plus a slim process image that contains only
the island binary (Debian, SQLite, no `rocci` / `rocdown` / `roc`):

```sh
# Apple Silicon Docker → arm64musl; Intel / amd64 containers → x64musl
cargo run -q -p rocci-rocdown-cli -- package examples/rocdown/counter --target arm64musl
uv run rocci-ops serve hybrid examples/rocdown/counter/dist examples/rocdown/counter/islands
```

`package` writes `dist/`, `publish.json` (live routes and binary fingerprint),
and a sibling `islands` binary. `--target` is the Linux container process
target (see [Choosing `--target`](#choosing---target)); `--host` remains
apply-only.

`ROCCI_DIST` and `ROCCI_ISLANDS_CONTEXT` must be absolute. The wrapper copies
the binary into a build context. The image creates an empty `assets/` directory
so basic-webserver can start; hashed site files stay on the CDN mount. When the
CDN container starts it prints `Open http://127.0.0.1:8080/` (honors
`ROCCI_HTTP_PORT`). Smoke:

```sh
curl -sf http://127.0.0.1:8080/health
curl -sf -X POST http://127.0.0.1:8080/actions/counter/increment \
  -H 'datastar-request: true' -H 'content-type: application/json' -d '{}'
docker run --rm --entrypoint /bin/sh rocci-islands:local -c 'which roc'; echo $?
```

`which roc` must fail.

## Rocci app Linux hosting (opt-in)

Package a Roc server binary plus assets on the host, then a slim process image
(no `rocci` CLI, `roc`, or WebKit). macOS `.app` remains `rocci bundle`.
Linux OCI is opt-in: set `ROC_BASIC_WEBSERVER_HOST=0.0.0.0` in Compose (do not
weaken `rocci.toml` loopback validation).

```sh
# Match the container CPU (Apple Silicon Docker → arm64musl)
cargo run -q -p rocci-cli -- build --release examples/rocci/custom/datastar --target arm64musl
uv run rocci-ops serve app target/release/rocci-server
```

Override the published port with `ROCCI_HTTP_PORT`. The app container prints
`Open http://127.0.0.1:8080/` (or that override) on start.

## Live example origins (planned hostnames)

Catalog rows with `hosting = "live"` (`live-counter`, `datastar`) are separate
processes and hostnames. They do **not** share the rocci.dev hybrid island
`/actions/` or `/sse`. Docs-only apps (counter, styling, snake) are absent
from this Compose file.

```sh
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --print-live
# live-counter	standalone/live-counter	LiveCounter.rocci
# datastar	custom/datastar	.
```

Package each live app with `rocci build --release --target x64musl` (or
`arm64musl` for Apple Silicon Docker), then point
`ROCCI_LIVE_COUNTER_CONTEXT` and `ROCCI_DATASTAR_CONTEXT` at directories that
contain `server`, `assets/`, and `docker/app/Dockerfile`.

```sh
docker compose -f docker/compose.examples.yml up
```

Caddy matches `Host` to `<id>.examples.rocci.dev`,
`<id>.examples.staging.rocci.dev`, and `<id>.examples.localhost`. Cloudflare
DNS/Tunnel for those names is operator work. Until a staging deploy has served
them, treat the live demo links as planned.

The hybrid site Caddy (`docker/cdn/Caddyfile`) still sends `/actions/*` to the
home-page island. Example origins never steal that path.

## Hybrid builder/dev demo (toolchain-heavy)

The original Compose file still builds Ubuntu images with Roc, the Rocci CLIs,
and WebKit, then compiles island `main.roc` at start. Use it only as a
**builder/dev** operator demo, not hosting.

From the repository root:

```sh
ROCCI_SITE="$(pwd)/examples/rocdown/counter" docker compose -f docker/compose.yml build
uv run rocci-ops serve site examples/rocdown/counter
uv run rocci-ops serve site examples/rocdown/hybrid
uv run rocci-ops serve site /path/to/any/hybrid-site
```

`ROCCI_SITE` is required even for `build` because Compose interpolates the
volume path. It must be absolute.

The script absolutizes `SITE_DIR` and runs `docker compose up --build`. Extra
arguments go to `compose up` (`-d`, `--no-build`, …).

Then open [http://127.0.0.1:8080/](http://127.0.0.1:8080/) (the CDN container
also prints this URL; override with `ROCCI_HTTP_PORT`). Island health is also
published at [http://127.0.0.1:8001/health](http://127.0.0.1:8001/health);
the browser should stay on 8080.

`site-build` runs `rocdown build /src/site` into the mounted `dist/`. `islands`
runs `rocdown serve-islands`. SQLite is the `islands-db` volume
(`DB_PATH=/var/lib/rocci/site.db`). Datastar and Roc caches live in
`rocci-cache` (`/var/cache/rocci`).

`ROC_BASIC_WEBSERVER_HOST=0.0.0.0` so Caddy can reach the island process.

First boot may spend a minute compiling generated `main.roc`. Image build needs
Docker with BuildKit and network access for the pinned Roc nightly and
crates.io.

To run two hybrid sites at once, set a distinct project name and publish port:

```sh
COMPOSE_PROJECT_NAME=hybrid-other \
  ROCCI_SITE="$(cd /path/to/other && pwd)" \
  docker compose -f docker/compose.yml up --build
```

Override `8080:80` with a Compose override file if the host port is taken
(static hosting uses the same published port).

## Layout

| Path | Role |
| --- | --- |
| [`static/Caddyfile`](static/Caddyfile) | Static `file_server` of `/srv`; no island proxy |
| [`compose.static.yml`](compose.static.yml) | `rocci-cdn` + bind-mount `ROCCI_DIST` |
| `rocci-ops serve static` | Absolutize `ROCCI_DIST` and `compose up` |
| [`islands/Dockerfile`](islands/Dockerfile) | Slim island process (`debian:bookworm-slim` + binary) |
| [`compose.hybrid.yml`](compose.hybrid.yml) | Pre-built hybrid: Caddy + island binary |
| [`prod/`](prod/) | Origin docs, Access SSH proxy, env examples |
| `rocci-ops serve hybrid` | Absolutize `ROCCI_DIST` and islands binary, `compose up` |
| [`app/Dockerfile`](app/Dockerfile) | Slim Rocci app process (`debian:bookworm-slim` + `server`) |
| [`compose.app.yml`](compose.app.yml) | Pre-built Rocci app (opt-in Linux OCI) |
| [`compose.examples.yml`](compose.examples.yml) | Live example origins (`live-counter`, `datastar`) |
| [`examples/Caddyfile`](examples/Caddyfile) | Host routing for `<id>.examples.rocci.dev`; no `/actions/` |
| `rocci-ops serve app` | Absolutize server dir and app `compose up` |
| [`runtime/Dockerfile`](runtime/Dockerfile) | **Builder/dev** toolchain (no CDN stage) |
| [`cdn/Dockerfile`](cdn/Dockerfile) | Thin Caddy image; prints host URL on start |
| [`cdn/Caddyfile`](cdn/Caddyfile) | Hybrid same-origin proxy; `root * /src/site/dist` |
| [`compose.yml`](compose.yml) | **Builder/dev** hybrid `site-build`, `islands`, `cdn` |
| `rocci-ops serve site` | Absolutize `ROCCI_SITE` and toolchain `compose up` |

The two-artifact production sketch (upload `dist/`, run `serve-islands`,
reverse-proxy) is in [`examples/rocdown/counter/README.md`](../examples/rocdown/counter/README.md)
and the [hybrid sites guide](../docs/guides/hybrid-sites.rocdown).

`rocci.dev` origin deploys from GitHub Actions `site.yml` on `staging` or
`production` via the matching Environment (`DEPLOY_HOST`, `DEPLOY_USER`,
`DEPLOY_SSH_KEY`, `CF_ACCESS_CLIENT_ID`, `CF_ACCESS_CLIENT_SECRET`). `main`
is the PR landing branch and does not package or publish the site. SSH goes
through Cloudflare Access (`ssh.rocci.dev`), not port 22. Fork pull requests
cannot read those secrets and do not run the deploy job. See
[`prod/README.md`](prod/README.md).
