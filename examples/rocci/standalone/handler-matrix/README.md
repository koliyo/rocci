# Handler matrix

Every accepted handler construct in one standalone app. Patch handlers return
uniquely identified HTML fragments. Command handlers return records. `@live`
owns `#live-tick` so the stream never races a one-shot patch.

There is no authored `main.roc` and no manual JSON string construction.

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci
```

Pass `--no-window` to serve on [http://127.0.0.1:8000](http://127.0.0.1:8000)
without a window. Override the port with `--port` or `ROC_BASIC_WEBSERVER_PORT`.

## Smoke checks

With the server running:

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'frag-post|live-tick|datastar.js'

# One-shot patches: Datastar SSE morphs a unique id
curl -s -X POST http://127.0.0.1:8000/actions/post-frag
# event: datastar-patch-elements
# ... id="frag-post"

curl -s -X PUT http://127.0.0.1:8000/actions/put-frag
curl -s -X PATCH http://127.0.0.1:8000/actions/patch-frag
curl -s -X DELETE http://127.0.0.1:8000/actions/delete-frag

# Commands: ordinary client receives JSON
curl -s -D - -X POST http://127.0.0.1:8000/actions/post-cmd
# HTTP/1.1 200
# Content-Type: application/json
# {"n":1}

curl -s -X PUT http://127.0.0.1:8000/actions/put-cmd
curl -s -X PATCH http://127.0.0.1:8000/actions/patch-cmd
curl -s -X DELETE http://127.0.0.1:8000/actions/delete-cmd

# Commands: Datastar receives 204 and does not morph HTML
curl -s -D - -o /dev/null -X POST http://127.0.0.1:8000/actions/post-cmd \
  -H 'Datastar-Request: true'
# HTTP/1.1 204
```

`curl` without `Datastar-Request` must print JSON, not
`datastar-patch-elements`. Commands do not morph `#live-tick`; the stream does.
