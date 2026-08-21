# Live counter

The shared-view sibling of [`../counter`](../counter). Same SQLite card, but
`@live` opts into a generated `GET /sse` poll. Increment and reset are marked
`json`: Datastar receives **204**, and `curl` without `Datastar-Request`
receives `{"count":N}`. There is no authored `main.roc`.

Open the URL in two windows. Click Increment in one; the other updates without
refresh. The first-app counter stays one-shot: a second tab there stays stale
until you click or reload.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0** (an implementation detail of `rocci run`)
- Datastar **1.0.2** (CLI cache)

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/live-counter/LiveCounter.rocci
```

This opens an embedded window on a free local TCP port and prints the URL. Pass `--no-window` to serve on [http://127.0.0.1:8000](http://127.0.0.1:8000) without a window (then open that URL yourself, or curl it). Override the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`. SQLite state lives in `examples/rocci/standalone/live-counter/counter.db` (created on first start). Set `DB_PATH` to use another file.

## Two-window check

1. Open the printed URL in two browsers or two windows.
2. Click **Increment** in one. The `<output>` in both windows should change
   without a refresh (the stream morphs `#counter`).
3. Click **Reset** in the other window. Both return to `0`.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'datastar.js|id="counter"|Increment|/sse'

curl -s -X POST http://127.0.0.1:8000/actions/counter/increment
# {"count":1}

curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/actions/counter/increment \
  -H 'Datastar-Request: true'
# HTTP/1.1 204
```

`curl -X POST` without `Datastar-Request` must print JSON, not
`datastar-patch-elements`. Commands do not morph `#counter`; the stream does.
