# Roc + Datastar counter POC

A spike: author the counter UI in `Counter.rocci`, compile it to Roc, and serve HTML plus one-shot Datastar patches from [basic-webserver](https://github.com/roc-lang/basic-webserver) 0.16.0. `rocci run` opens the server in an embedded window; use `--no-window` to serve only. There is no live multi-tab SSE stream.

`Counter.rocci` colocates isolated CSS: a file-level `@css` block for the hello list, and component `@css` on `hello` and `counterCard`. Page chrome still comes from `/assets/app.css`. v1 injects a `<style>` tag with the component Html; increment patches `#counter` and may resend that CSS until extract-and-link lands.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** (CLI cache, pinned in `rocci.toml`)

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/counter
```

This opens an embedded window at [http://127.0.0.1:8000](http://127.0.0.1:8000). Pass `--no-window` to serve only (then open that URL yourself, or curl it). Override the port with `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/counter/counter.db` (created on first start). Set `DB_PATH` to use another file.

`rocci run` compiles `Counter.rocci` to a Roc type module (`Counter.roc`, gitignored), copies the pinned Datastar runtime into `examples/counter/assets/`, and executes `main.roc`.

`main.roc` only owns HTTP, SQLite, and SSE; it does not build the page HTML.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'datastar.js|id="counter"|data-rocci-css|hello-list'

curl -s -X POST http://127.0.0.1:8000/api/counter/increment
# event: datastar-patch-elements
# data: elements <section id="counter" ...><output>1</output>...
```

Increment and reset should update `<output>` in the browser via a single `datastar-patch-elements` event that morphs `#counter`.

## Package a desktop app

From the repository root, with `roc` and `cargo` on `PATH` (macOS only):

```sh
./scripts/bundle-macos.sh
open "target/release/bundle/macos/Counter.app"
```

`rocci bundle` compiles `.rocci` modules, `roc build`s `main.roc`, builds the
`rocci` host, and writes an ad-hoc signed `.app`. The bundled app does not need
`roc` on `PATH` at runtime.

This example’s [`rocci.toml`](rocci.toml) uses `bundle.app = "."`. The repository
root [`rocci.toml`](../../rocci.toml) points at this directory so the same
command works from the workspace root.
