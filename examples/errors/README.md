# Error-page examples

What `rocci run` shows in the preview when a route is missing or a file does
not compile. The diagnostic HTML is self-contained (no app theme) so it still
renders if CSS or lowering is the thing that broke.

| Path | What it shows |
| --- | --- |
| [`ErrorDemo.rocdown`](ErrorDemo.rocdown) | A working page at `/error-demo/` with links that hit the 404 page |
| [`parse/Broken.rocdown`](parse/Broken.rocdown) | Unterminated `@page`; compile frames in the browser |
| [`roc/BrokenRoc.rocdown`](roc/BrokenRoc.rocdown) | Valid Rocdown that Roc rejects; compiler output in the browser |

The parse and Roc-failure files sit in their own directories so they are not
compiled as siblings of `ErrorDemo.rocdown`.

## 404

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/errors/ErrorDemo.rocdown
```

The window opens on `/error-demo/`. Click **Missing route** (`/missing`) for
the 404 with the registered route table, or **Slash redirect** (`/error-demo`)
to follow 308 to `/error-demo/`. Pass `--no-window` to serve on
[http://127.0.0.1:8000](http://127.0.0.1:8000).

The same 404 appears on any standalone `rocci run` app, including
[`examples/rocdown/Guide.rocdown`](../rocdown/Guide.rocdown). Generated
dispatchers 308 the unregistered slash variant of a GET route by default
(`http.redirect_trailing_slash` in `rocci.toml`).

## Parse error

```sh
cargo run -q -p rocci-cli -- run examples/errors/parse/Broken.rocdown
```

The CLI still prints a rustc-style frame on stderr. The preview stays up and
shows the same diagnostic. `rocci build` on that file exits after printing
frames and does not start a server.

## Roc compile error

```sh
cargo run -q -p rocci-cli -- run examples/errors/roc/BrokenRoc.rocdown
```

The document parses. Roc then rejects the generated program (a `Str` bound to
`1`). If Roc exits, or if it binds the port anyway and then crashes, the
preview still opens on the remapped compiler output instead of a crashed
server.

Handler failures (a route `Err`) render a 500 page or a Datastar overlay on
POST. Those need a `Try` that actually fails at runtime; this folder does not
force one.
