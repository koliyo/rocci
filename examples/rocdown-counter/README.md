# Hybrid counter

A Rocdown site with one `live` page: Markdown plus a SQLite-backed counter
island, the hybrid analog of [`examples/counter`](../counter). The CDN file is
a snapshot (`count` is `0` at build). Increment and reset POST to
`rocdown serve-islands`, which owns the database.

A neighboring [`about.rocdown`](about.rocdown) page stays `static`: no Datastar,
no island routes.

There is no authored `main.roc`. Handlers are colocated in
[`index.rocdown`](index.rocdown). Do not point `[http].service` at
`examples/counter/Counter.rocci`; that file is a full-page app, and GET `/`
is not an island route.

For page-kind coverage without SQLite, see
[`examples/rocdown-hybrid`](../rocdown-hybrid).

## Preview

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown-counter --no-window
```

This serves the CDN tree and proxies `/actions/` on one origin
([http://127.0.0.1:8000](http://127.0.0.1:8000) by default). Omit `--no-window`
to open an embedded preview. Override the port with `--port` or
`ROC_BASIC_WEBSERVER_PORT`. Generated island `main.roc` binds `127.0.0.1` unless
`ROC_BASIC_WEBSERVER_HOST` is set (use `0.0.0.0` behind Docker Compose).

Preview SQLite state is ephemeral (`islands.db` in the staging workspace).
Handler and content edits reload; durable production state uses
`serve-islands` and `DB_PATH` below.

Inspect the publish plan without Roc:

```sh
cargo run -q -p rocci-rocdown-cli -- inspect artifacts examples/rocdown-counter
```

Expect `index` as `live` with Datastar, `about` as `static`, and
`POST /actions/counter/increment` plus `POST /actions/counter/reset`.

## Build the CDN tree

```sh
cargo run -q -p rocci-rocdown-cli -- build examples/rocdown-counter
```

Writes `examples/rocdown-counter/dist/`: page HTML, hashed `/assets/`
(including Datastar.js on the live page), `pages.json`, and `islands.json`.
`--output DIR` overrides `[build].output`.

`rocdown build --cdn-only` errors with `RD2302` when any published page is
`live`. Use that flag only when you are not deploying an island service.

## Run the island service

```sh
cargo run -q -p rocci-rocdown-cli -- serve-islands examples/rocdown-counter --no-window
```

This process serves health and mutation routes only. It does not serve
Markdown or the CDN HTML. Default SQLite path is `index.db` next to the site
(the primary live module). Set `DB_PATH` to a persistent file:

```sh
DB_PATH=/var/lib/rocci/counter.db cargo run -q -p rocci-rocdown-cli -- \
    serve-islands examples/rocdown-counter --no-window --port 8001
```

## Two-artifact deploy

Recommended v1 layout is **same-origin**: the browser loads HTML from the CDN
and POSTs `/actions/...` to that same origin. A reverse proxy forwards those
paths to `serve-islands`. Leave `[http].service_origin` empty so action URLs
stay relative and CSP `connect-src` is `'self'`.

1. Build: `rocdown build examples/rocdown-counter`.
2. Upload `dist/` to the CDN or object store. Hashed `/assets/*` files can be
   cached indefinitely. HTML, `pages.json`, `islands.json`, and other
   discovery files should revalidate; they name the current hashes and
   routes.
3. Run `rocdown serve-islands` on a host with a persistent `DB_PATH`.
4. Reverse-proxy `/actions/` and `/health` to that process. Serve the rest of
   the URL space from the CDN tree (including `/` and `/about/`).

Sketch (Caddy):

```caddy
example.com {
    handle /actions/* {
        reverse_proxy 127.0.0.1:8001
    }
    handle /health {
        reverse_proxy 127.0.0.1:8001
    }
    handle {
        root * /var/www/rocdown-counter
        try_files {path} {path}/index.html
        file_server
    }
}
```

Cross-origin: set `[http] service_origin = "https://islands.example.com"` in
`rocdown.toml`, rebuild so `@post` URLs and CSP `connect-src` are absolute,
and host `serve-islands` on that origin. CORS and cookies for that layout
are not shipped yet; prefer the same-origin proxy.

A sibling `[http].service` `.rocci` app is an alternative to colocated `@on`.
`rocdown serve-islands` runs that file with `rocci run` instead of generating
a dispatcher. The app must return island fragments, not a full HTML document,
and must not rely on GET `/` (the CDN owns page GET).

## Local Docker

Same-origin two-image layout: Caddy serves the CDN tree and reverse-proxies
`/actions/` plus `/health` to `serve-islands`. Images are defined in
[`docker/runtime/Dockerfile`](../../docker/runtime/Dockerfile). From the
repository root:

```sh
docker compose -f examples/rocdown-counter/docker-compose.yml up --build
```

Then open [http://127.0.0.1:8080/](http://127.0.0.1:8080/) (live counter) and
[/about/](http://127.0.0.1:8080/about/) (static). SQLite state is the
`counter-db` volume (`DB_PATH=/var/lib/rocci/counter.db`). The islands
container sets `ROC_BASIC_WEBSERVER_HOST=0.0.0.0` so Caddy can reach it.

Smoke through Caddy, not the island port:

```sh
curl -s http://127.0.0.1:8080/health
curl -s -X POST http://127.0.0.1:8080/actions/counter/increment
curl -s http://127.0.0.1:8080/about/ | grep -E '<script>|datastar' || true
```

First boot may spend a minute compiling generated `main.roc`. Image build
needs Docker with BuildKit and network access for the pinned Roc nightly and
crates.io.

## Smoke checks

CDN tree after `build` (no service required):

```sh
grep -E 'id="counter"|/assets/datastar' examples/rocdown-counter/dist/index.html
grep -E '<script>|/assets/datastar' examples/rocdown-counter/dist/about/index.html || true
```

Home page HTML should include `#counter` and hashed Datastar.js. `about/` should
not include a script tag or Datastar asset.

With `serve-islands` on port 8001:

```sh
curl -s http://127.0.0.1:8001/health
# ok

curl -s -X POST http://127.0.0.1:8001/actions/counter/increment
# event: datastar-patch-elements
# data: elements <section id="counter" ...><output>1</output>...
```

With `rocdown run --no-window` on port 8000, the same POST works on that
origin, and `GET /` is the CDN snapshot rather than a SQLite read.
