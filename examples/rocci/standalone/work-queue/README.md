# Work queue

Local run notes. Published tutorial: https://rocci.dev/examples/work-queue/

Docs-only; not a public live origin.

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/work-queue/WorkQueue.rocci
```

Open two browser tabs. Inspect in one: that well fills, the other stays
waiting, both activity logs update. Enqueue or claim: composer and inspect
stay put; both boards move.

Fragments own `#composer` and `#inspect`. Live owns `#queue` and `#activity`.
Commands return no HTML. Job ids travel in the Datastar JSON body; routes are
literal `@method:role` paths.

```sh
curl -s -X POST http://127.0.0.1:8000/actions/inspect \
  -H 'Content-Type: application/json' \
  --data '{"job_id":"1"}'
curl -s -X POST http://127.0.0.1:8000/actions/enqueue \
  -H 'Content-Type: application/json' \
  --data '{"title":"From curl"}'
# empty body; HTTP 204
```
