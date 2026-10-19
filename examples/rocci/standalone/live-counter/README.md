# Live counter

Local run notes. Published tutorial: https://rocci.dev/examples/live-counter/

This directory is one standalone app, several files:

| File | Role |
| --- | --- |
| `LiveCounter.rocci` | Entry: `@init`, `@get:view`, `@get:live("/sse")`, commands |
| `LiveCounterUi.rocci` | Components (`import LiveCounterUi`) |
| `Origin.roc` | Authored Roc helper (`import Origin`) |
| `CounterCore.roc` | Shared pure decisions, feed retention, and view construction |
| `CounterDocument.rocci` | Shared pure document shell; explicit live subscription |

Pass the entry `.rocci` or this directory (one unique initializer).
`rocci run` and `rocci build --http-module` walk the standalone tree (this
folder; or a `rocci.toml` app root) and compile every sibling `.rocci` plus
authored `.roc` helpers. Same rule as Counter, just more modules.

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/live-counter/LiveCounter.rocci
```

Open two windows on the printed URL. Increment in one; both `<output>` values
and the recent-click feed change. The reserved
`live-counter.examples.rocci.dev` hostname is not serving yet.

The same app powers the rocci.dev home island (`[http].service` +
`@render LiveCounterUi.CounterIsland`).

Commands now read/decide/write count and click history in one immediate
transaction. Reads use one coherent SQLite snapshot. A revision column is
added to existing databases and used for never-reused click IDs; reset keeps
the feed. The shared core rejects count/revision overflow above the exact
integer range `0..9007199254740991`. The native transport remains compatible.

The separate [private Cloudflare experiment](../../remote/live-counter/README.md)
reuses this core and these components. Its optional `remote` UI hook defaults
to false, so the public island retains its existing Datastar actions.

```sh
curl -s -X POST http://127.0.0.1:8000/actions/counter/increment
# empty body; HTTP 204
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/actions/counter/increment \
  -H 'Datastar-Request: true'
# 200 text/event-stream (empty SSE)
```

Experimental WASI 0.3 component (`rocci run` stays native). Live patches come
from the compiled app (`roc_sse_advance_for_host`), not the `/sse-wait`
fixture. Wait is wasmtime clocks; two `/sse` connections should stay near
one 100ms poll, not two.

```sh
cargo run -q -p rocci-cli -- build --http-module \
  examples/rocci/standalone/live-counter/LiveCounter.rocci -o http-module.wasm
mkdir -p .counter-data
wasmtime serve -Sp3 -Scli --env DB_PATH=./counter.db \
  --dir=.counter-data::. --dir=./http-module.assets::/assets \
  --addr 127.0.0.1:8080 http-module.wasm
curl -s http://127.0.0.1:8080/ | head
curl -sN --max-time 1 http://127.0.0.1:8080/sse | head
# overlapping waits (wall ~100ms, not ~200ms)
curl -sN --max-time 1 http://127.0.0.1:8080/sse >/tmp/sse-a &
curl -sN --max-time 1 http://127.0.0.1:8080/sse >/tmp/sse-b &
wait
```
