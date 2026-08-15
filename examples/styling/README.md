# Styling

A standalone page that shows colocated `@css`: file-level document chrome,
component-scoped rules, default component params, and Vue-style isolation.
There is no database and no Datastar handlers. For the app shape (SQLite,
`@on`, patches), see [`examples/counter`](../counter).

v1 injects a `<style>` tag into the document `<head>`. Rules are wrapped in
`@scope ([data-rocci-css~="id"])`. Put document chrome on `body`, not `html`.

Pinned together:

- Roc nightly **2026-08-08** (the platform release was built against 2026-08-10)
- `basic-webserver` **0.16.0** (an implementation detail of `rocci run`)

## Run

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/styling/Styling.rocci
```

This opens an embedded window at [http://127.0.0.1:8000](http://127.0.0.1:8000). Pass `--no-window` to serve only. Override the port with `ROC_BASIC_WEBSERVER_PORT`.

Preview components from fixtures with `rocci view` or `rocci browse examples`.

## Smoke checks

With the server running (`--no-window` if you do not want an embedded window):

```sh
curl -s http://127.0.0.1:8000/health
# ok

curl -s http://127.0.0.1:8000/ | grep -E 'data-rocci-css|hello-list|feature-card|@scope'
```
