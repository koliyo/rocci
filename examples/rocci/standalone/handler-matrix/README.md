# Handler matrix

Local run notes. Published tutorial: https://rocci.dev/examples/handler-matrix/

Docs-only; not a public live origin.

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci
```

Open two browser tabs. A fragment patches one tab's result well and both
server logs. A command leaves the command card unchanged; both logs update.

GET fragment is an in-place Datastar `@get`, not a navigation link.

```sh
curl -s -X POST http://127.0.0.1:8000/actions/post-frag
curl -s -X POST http://127.0.0.1:8000/actions/post-cmd
# empty body; HTTP 204
```
