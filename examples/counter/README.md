# Counter

The first Rocci app: one `.rocci` file with SQLite state, a page, and two
Datastar POST handlers. Increment and reset each return one patch of
`#counter`. There is no authored `main.roc`.

For page chrome, component isolation, and `@scope`, see
[`examples/styling`](../styling).

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0** (an implementation detail of `rocci run`)
- Datastar **1.0.2** (CLI cache)

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/counter/Counter.rocci
```

This opens an embedded window on a free local TCP port and prints the URL. Pass `--no-window` to serve on [http://127.0.0.1:8000](http://127.0.0.1:8000) without a window (then open that URL yourself, or curl it). Override the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/counter/counter.db` (created on first start). Set `DB_PATH` to use another file.

`rocci view` and `rocci browse` render components from fixtures; they do not run `@init` or `@on` handlers.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'datastar.js|id="counter"|Increment'

curl -s -X POST http://127.0.0.1:8000/api/counter/increment
# event: datastar-patch-elements
# data: elements <section id="counter" ...><output>1</output>...
```

Increment and reset should update `<output>` in the browser via a single `datastar-patch-elements` event that morphs `#counter`.
