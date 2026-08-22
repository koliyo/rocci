# Hybrid counter

A Rocdown site with one `live` page: Markdown plus a SQLite-backed counter
island, the hybrid analog of [`examples/rocci/standalone/live-counter`](../../rocci/standalone/live-counter). The CDN file is
a snapshot (`count` is `0` at build). On load, generated `GET /sse` replaces
that snapshot and keeps two browsers in sync. Increment and reset are `json`
commands (204 to Datastar; JSON to `curl`).

A neighboring [`about.rocdown`](about.rocdown) page stays `static`: no Datastar,
no island routes.

There is no authored `main.roc`. Handlers are colocated in
[`index.rocdown`](index.rocdown). Do not point `[http].service` at
`examples/rocci/standalone/counter/Counter.rocci`; that file is a full-page app, and GET `/`
is not an island route.

For page-kind coverage without SQLite, see
[`examples/rocdown/hybrid`](../hybrid).

## Preview

Debug this example on the host with `roc` and `cargo` on `PATH`. Do not start
Docker until the steps below are green. Compose is only for Caddy same-origin
proxy, published 8001, `ROC_BASIC_WEBSERVER_HOST=0.0.0.0`, and mounted-site
path stamps.

### Catalog without Roc

```sh
cargo run -q -p rocci-rocdown-cli -- inspect artifacts examples/rocdown/counter
```

Expect `index` as `live` with Datastar, `about` as `static`, and
`POST /actions/counter/increment` and `reset` (no `/actions/counter/sync`).

### Snapshot HTML and CSS

```sh
cargo run -q -p rocci-rocdown-cli -- build examples/rocdown/counter
grep -E 'rd-document|id="counter"|href="/assets/' examples/rocdown/counter/dist/index.html
grep -l 'border-radius: 16px' examples/rocdown/counter/dist/assets/*.css
```

No island process. Home HTML should include `rd-document`, `#counter`, and a
hashed stylesheet. The island CSS file should include the card radius.

### Interactive same-origin

```sh
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/counter --no-window \
  --output /tmp/rocdown-counter-preview
```

Serves the CDN tree and proxies `/actions/` and `/sse` on one origin
([http://127.0.0.1:8000](http://127.0.0.1:8000) by default). Logs
`rocdown: preview files at …`. `--output` keeps that tree after stop; without
it the files live in a temp directory that is deleted when the process exits.
Omit `--no-window` to open an embedded preview. Override the port with
`--port` or `ROC_BASIC_WEBSERVER_PORT`. Generated island `main.roc` binds
`127.0.0.1` unless `ROC_BASIC_WEBSERVER_HOST` is set.

Pass `--log-handlers` to print each proxied / dispatched `@on` route on the
CLI and in the Dev Console.

Preview SQLite state is ephemeral (`islands.db` in the staging workspace).
Handler and content edits reload; durable production state uses
`serve-islands` and `DB_PATH` below.

Smoke on 8000:

```sh
curl -sf http://127.0.0.1:8000/health
curl -sf http://127.0.0.1:8000/ | grep -E 'rd-document|#counter|stylesheet'
curl -sf -X POST http://127.0.0.1:8000/actions/counter/increment
# {"count":1}
```

Open [http://127.0.0.1:8000/](http://127.0.0.1:8000/) in two windows, click Increment once, and
confirm `#counter` morphs in both (the stream, not the POST). Dev inspector:
[http://127.0.0.1:8000/__rocci/dev](http://127.0.0.1:8000/__rocci/dev).
While the server is up, grep `/tmp/rocdown-counter-preview` the same way as
`dist/` above.

## Build the CDN tree

```sh
cargo run -q -p rocci-rocdown-cli -- build examples/rocdown/counter
```

Writes `examples/rocdown/counter/dist/`: page HTML, hashed `/assets/`
(including Datastar.js on the live page), `pages.json`, and `islands.json`.
`--output DIR` overrides `[build].output`.

`rocdown build --cdn-only` errors with `RD2302` when any published page is
`live`. Use that flag only when you are not deploying an island service.

## Run the island service

```sh
cargo run -q -p rocci-rocdown-cli -- serve-islands examples/rocdown/counter --no-window
```

This process serves health and mutation routes only. It does not serve
Markdown or the CDN HTML. Default SQLite path is `index.db` next to the site
(the primary live module). Set `DB_PATH` to a persistent file:

```sh
DB_PATH=/var/lib/rocci/counter.db cargo run -q -p rocci-rocdown-cli -- \
    serve-islands examples/rocdown/counter --no-window --port 8001
```

## Two-artifact deploy

Recommended v1 layout is **same-origin**: the browser loads HTML from the CDN
and POSTs `/actions/...` to that same origin. A reverse proxy forwards those
paths to `serve-islands`. Leave `[http].service_origin` empty so action URLs
stay relative and CSP `connect-src` is `'self'`.

1. Build: `rocdown build examples/rocdown/counter`.
2. Upload `dist/` to the CDN or object store. Hashed `/assets/*` files can be
   cached indefinitely. HTML, `pages.json`, `islands.json`, and other
   discovery files should revalidate; they name the current hashes and
   routes.
3. Run `rocdown serve-islands` on a host with a persistent `DB_PATH`.
4. Reverse-proxy `/actions/`, `/sse`, and `/health` to that process. Serve the rest of
   the URL space from the CDN tree (including `/` and `/about/`).

Sketch (Caddy):

```caddy
example.com {
    handle /actions/* {
        reverse_proxy 127.0.0.1:8001
    }
    handle /sse {
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

Package on the host, then Caddy plus a slim island process image (no `roc` /
`rocdown` at runtime):

```sh
# Match the Linux container CPU (Apple Silicon Docker → arm64musl; amd64 → x64musl)
cargo run -q -p rocci-rocdown-cli -- package examples/rocdown/counter --target arm64musl
uv run rocci-ops serve hybrid examples/rocdown/counter/dist examples/rocdown/counter/islands
```

Then open
[http://127.0.0.1:8080/](http://127.0.0.1:8080/) (live counter) and
[/about/](http://127.0.0.1:8080/about/) (static). Caddy serves the CDN tree and
proxies `/actions/`, `/sse`, plus `/health`. SQLite state is the `islands-db` volume.
See [`docker/README.md`](../../docker/README.md) for choosing `--target`.

```sh
curl -sf http://127.0.0.1:8080/health
curl -sf -X POST http://127.0.0.1:8080/actions/counter/increment
# {"count":1}
docker run --rm --entrypoint /bin/sh rocci-islands:local -c 'which roc'; echo $?
```

`which roc` must fail. `uv run rocci-ops serve site` remains the builder/dev toolchain
demo. Operator notes are in [`docker/README.md`](../../docker/README.md).

## Smoke checks

CDN tree after `build` (no service required):

```sh
grep -E 'id="counter"|/assets/datastar' examples/rocdown/counter/dist/index.html
grep -E '<script>|/assets/datastar' examples/rocdown/counter/dist/about/index.html || true
```

Home page HTML should include `#counter` and hashed Datastar.js. `about/` should
not include a script tag or Datastar asset.

With `serve-islands` on port 8001:

```sh
curl -s http://127.0.0.1:8001/health
# ok

curl -s -X POST http://127.0.0.1:8001/actions/counter/increment
# {"count":1}
```

With `rocdown view --no-window` on port 8000, the same POST works on that
origin, and `GET /` is the CDN snapshot rather than a SQLite read. The first
`GET /sse` event replaces snapshot `0` with the live count.
