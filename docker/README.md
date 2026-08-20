# Host a Rocdown site with Docker

Build the site on a toolchain host. Local Docker then serves only the compiled
artifacts. Official Caddy has no `rocci`, `rocdown`, `roc`, rustc, or WebKit.

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
cargo run -q -p rocci-rocdown-cli -- package examples/rocdown-counter --target x64musl
./docker/run-hybrid.sh examples/rocdown-counter/dist examples/rocdown-counter/islands
```

`package` writes `dist/`, `publish.json` (live routes and binary fingerprint),
and a sibling `islands` binary. `--target x64musl` (or `arm64musl`) is the
Linux container process target; `--host` remains apply-only.

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
| [`runtime/Dockerfile`](runtime/Dockerfile) | **Builder/dev** toolchain plus hybrid `cdn` |
| [`cdn/Caddyfile`](cdn/Caddyfile) | Hybrid same-origin proxy; `root * /src/site/dist` |
| [`compose.yml`](compose.yml) | **Builder/dev** hybrid `site-build`, `islands`, `cdn` |
| [`run-site.sh`](run-site.sh) | Absolutize `ROCCI_SITE` and toolchain `compose up` |

The two-artifact production sketch (upload `dist/`, run `serve-islands`,
reverse-proxy) is in [`examples/rocdown-counter/README.md`](../examples/rocdown-counter/README.md)
and the [hybrid sites guide](../docs/guides/hybrid-sites.rocdown).
