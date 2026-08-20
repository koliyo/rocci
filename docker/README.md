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
./docker/run-hybrid.sh DIST ISLANDS

# amd64 containers via Docker’s x86_64 emulation (Rosetta/QEMU)
cargo run -q -p rocci-rocdown-cli -- package SITE --target x64musl
DOCKER_DEFAULT_PLATFORM=linux/amd64 ./docker/run-hybrid.sh DIST ISLANDS
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

Host a pre-built Rocdown tree (`rocdown build --cdn-only`). There is no custom
image and no island proxy.

From the repository root, dogfood `docs/` (`build.output = "../dist/docs"`):

```sh
cargo run -q -p rocci-rocdown-cli -- build docs --cdn-only
./docker/run-static.sh dist/docs
```

The script absolutizes `ROCCI_DIST` and runs Compose. Extra arguments go to
`compose up` (`-d`, …). Then open [http://127.0.0.1:8080/](http://127.0.0.1:8080/).

`ROCCI_DIST` must be absolute. Interpolate it yourself if you call Compose
directly:

```sh
ROCCI_DIST="$(cd dist/docs && pwd)" docker compose -f docker/compose.static.yml up
```

`docker compose -f docker/compose.static.yml build` does not compile Rocci; it
is a no-op aside from pulling `caddy:2-alpine` if needed.

Hashed `/assets/` files are immutable. HTML uses `no-cache`. `/actions/` is
not proxied (Caddy 404). This path rejects `live` pages at build time
(`RD2302`).

## Hybrid live-island hosting (pre-built)

Package on the host, then Caddy plus a slim process image that contains only
the island binary (Debian, SQLite, no `rocci` / `rocdown` / `roc`):

```sh
# Apple Silicon Docker → arm64musl; Intel / amd64 containers → x64musl
cargo run -q -p rocci-rocdown-cli -- package examples/rocdown-counter --target arm64musl
./docker/run-hybrid.sh examples/rocdown-counter/dist examples/rocdown-counter/islands
```

`package` writes `dist/`, `publish.json` (live routes and binary fingerprint),
and a sibling `islands` binary. `--target` is the Linux container process
target (see [Choosing `--target`](#choosing---target)); `--host` remains
apply-only.

`ROCCI_DIST` and `ROCCI_ISLANDS_CONTEXT` must be absolute. The wrapper copies
the binary into a build context. The image creates an empty `assets/` directory
so basic-webserver can start; hashed site files stay on the CDN mount. Then open
[http://127.0.0.1:8080/](http://127.0.0.1:8080/). Override the published port
with `ROCCI_HTTP_PORT` when 8080 is already taken. Smoke:

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
cargo run -q -p rocci-cli -- build --release examples/datastar --target arm64musl
./docker/run-app.sh target/release/rocci-server
```

Override the published port with `ROCCI_HTTP_PORT`. Then open
[http://127.0.0.1:8080/](http://127.0.0.1:8080/).

## Hybrid builder/dev demo (toolchain-heavy)

The original Compose file still builds Ubuntu images with Roc, the Rocci CLIs,
and WebKit, then compiles island `main.roc` at start. Use it only as a
**builder/dev** operator demo, not hosting.

From the repository root:

```sh
ROCCI_SITE="$(pwd)/examples/rocdown-counter" docker compose -f docker/compose.yml build
./docker/run-site.sh examples/rocdown-counter
./docker/run-site.sh examples/rocdown-hybrid
./docker/run-site.sh /path/to/any/hybrid-site
```

`ROCCI_SITE` is required even for `build` because Compose interpolates the
volume path. It must be absolute.

The script absolutizes `SITE_DIR` and runs `docker compose up --build`. Extra
arguments go to `compose up` (`-d`, `--no-build`, …).

Then open [http://127.0.0.1:8080/](http://127.0.0.1:8080/). Island health is
also published at [http://127.0.0.1:8001/health](http://127.0.0.1:8001/health);
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
| [`compose.static.yml`](compose.static.yml) | Official `caddy:2-alpine`; bind-mount `ROCCI_DIST` |
| [`run-static.sh`](run-static.sh) | Absolutize `ROCCI_DIST` and `compose up` |
| [`islands/Dockerfile`](islands/Dockerfile) | Slim island process (`debian:bookworm-slim` + binary) |
| [`compose.hybrid.yml`](compose.hybrid.yml) | Pre-built hybrid: Caddy + island binary |
| [`run-hybrid.sh`](run-hybrid.sh) | Absolutize `ROCCI_DIST` and islands binary, `compose up` |
| [`app/Dockerfile`](app/Dockerfile) | Slim Rocci app process (`debian:bookworm-slim` + `server`) |
| [`compose.app.yml`](compose.app.yml) | Pre-built Rocci app (opt-in Linux OCI) |
| [`run-app.sh`](run-app.sh) | Absolutize server dir and app `compose up` |
| [`runtime/Dockerfile`](runtime/Dockerfile) | **Builder/dev** toolchain plus hybrid `cdn` |
| [`cdn/Caddyfile`](cdn/Caddyfile) | Hybrid same-origin proxy; `root * /src/site/dist` |
| [`compose.yml`](compose.yml) | **Builder/dev** hybrid `site-build`, `islands`, `cdn` |
| [`run-site.sh`](run-site.sh) | Absolutize `ROCCI_SITE` and toolchain `compose up` |

The two-artifact production sketch (upload `dist/`, run `serve-islands`,
reverse-proxy) is in [`examples/rocdown-counter/README.md`](../examples/rocdown-counter/README.md)
and the [hybrid sites guide](../docs/guides/hybrid-sites.rocdown).
