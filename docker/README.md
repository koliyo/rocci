# Hybrid Docker images

Content-agnostic images for hybrid Rocdown sites: Ubuntu runs `rocdown`
(`build` and `serve-islands`); official Caddy serves `dist/` and reverse-proxies
`/actions/` plus `/health` on one origin.

The images do not bake a site. Bind-mount any directory with `rocdown.toml` at
`/src/site`.

## Build the base images

From the repository root:

```sh
ROCCI_SITE="$(pwd)/examples/rocdown-counter" docker compose -f docker/compose.yml build
```

`ROCCI_SITE` is required even for `build` because Compose interpolates the
volume path. It must be absolute.

## Run a mounted site

```sh
./docker/run-site.sh examples/rocdown-counter
./docker/run-site.sh examples/rocdown-hybrid
./docker/run-site.sh /path/to/any/hybrid-site
```

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

To run two sites at once, set a distinct project name and publish port:

```sh
COMPOSE_PROJECT_NAME=hybrid-other \
  ROCCI_SITE="$(cd /path/to/other && pwd)" \
  docker compose -f docker/compose.yml up --build
```

Override `8080:80` with a Compose override file if the host port is taken.

## Layout

| Path | Role |
| --- | --- |
| [`runtime/Dockerfile`](runtime/Dockerfile) | `builder`, `runtime` (`rocci` / `rocdown` / `roc`), `cdn` (Caddy) |
| [`cdn/Caddyfile`](cdn/Caddyfile) | Same-origin proxy; `root * /src/site/dist` |
| [`compose.yml`](compose.yml) | `site-build`, `islands`, `cdn` |
| [`run-site.sh`](run-site.sh) | Absolutize `ROCCI_SITE` and `compose up` |

The two-artifact production sketch (upload `dist/`, run `serve-islands`,
reverse-proxy) is in [`examples/rocdown-counter/README.md`](../examples/rocdown-counter/README.md)
and the [hybrid sites guide](../docs/guides/hybrid-sites.rocdown).
