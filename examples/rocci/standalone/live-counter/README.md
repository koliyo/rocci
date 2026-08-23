# Live counter

Local run notes. Published tutorial: https://rocci.dev/examples/live-counter/

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/live-counter/LiveCounter.rocci
```

Open two windows on the printed URL. Increment in one; both `<output>` values
and the recent-click feed change. The reserved
`live-counter.examples.rocci.dev` hostname is not serving yet.

The same app powers the rocci.dev home island (`[http].service` +
`@render LiveCounterUi.CounterIsland`).

```sh
curl -s -X POST http://127.0.0.1:8000/actions/counter/increment
# empty body; HTTP 204
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/actions/counter/increment \
  -H 'Datastar-Request: true'
# 200 text/event-stream (empty SSE)
```
