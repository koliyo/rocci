# Error-page examples

What `rocci run` shows in the preview when a route is missing or a file does
not compile. The diagnostic HTML is self-contained (no app theme) so it still
renders if CSS or lowering is the thing that broke.

| Path | What it shows |
| --- | --- |
| [`Dx.rocdown`](Dx.rocdown) | A working page at `/dx/` with links that hit the 404 page |
| [`parse/Broken.rocdown`](parse/Broken.rocdown) | Unterminated `@page`; compile frames in the browser |

The parse file sits in its own directory so it is not compiled as a sibling of
`Dx.rocdown`.

## 404

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/errors/Dx.rocdown
```

The window opens on `/dx/`. Click **Missing route** (`/missing`) for the 404
with the registered route table, or **Without trailing slash** (`/dx`) for the
`/dx/` hint. Pass `--no-window` to serve on
[http://127.0.0.1:8000](http://127.0.0.1:8000).

The same 404 appears on any standalone `rocci run` app, including
[`examples/rocdown/Guide.rocdown`](../rocdown/Guide.rocdown).

## Parse error

```sh
cargo run -q -p rocci-cli -- run examples/errors/parse/Broken.rocdown
```

The CLI still prints a rustc-style frame on stderr. The preview stays up and
shows the same diagnostic. `rocci build` on that file exits after printing
frames and does not start a server.

Handler failures (a route `Err`) render a 500 page or a Datastar overlay on
POST. Those need a `Try` that actually fails at runtime; this folder does not
force one.
