# Roc + Datastar counter POC

A browser-only spike: author the counter UI in `Counter.rocci`, compile it to Roc, and serve HTML plus one-shot Datastar patches from [basic-webserver](https://github.com/roc-lang/basic-webserver) 0.16.0. There is no tao/wry shell and no live multi-tab SSE stream.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0**
- Datastar **1.0.2** from `assets/datastar.js`

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
./scripts/run-roc-counter.sh
```

Then open [http://127.0.0.1:8000](http://127.0.0.1:8000). Override the port with `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/roc-counter/counter.db` (created on first start). Set `DB_PATH` to use another file.

The script copies shared assets into `examples/roc-counter/assets/` and runs `rocci run`, which compiles `Counter.rocci` to a Roc type module (`Counter.roc`, gitignored) and executes `main.roc`. If assets are already in place:

```sh
cargo run -q -p rocci-cli -- run examples/roc-counter/main.roc
```

`main.roc` only owns HTTP, SQLite, and SSE; it does not build the page HTML.

## Smoke checks

With the server running:

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'datastar.js|id="counter"'

curl -s -X POST http://127.0.0.1:8000/api/counter/increment
# event: datastar-patch-elements
# data: elements <section id="counter" ...><output>1</output>...
```

Increment and reset should update `<output>` in the browser via a single `datastar-patch-elements` event that morphs `#counter`.
