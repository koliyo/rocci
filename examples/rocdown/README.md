# Rocdown guide

A single `.rocdown` page: Markdown prose, `@roc` values, a colocated
`@component`, file and component `@css`, and a displayed (never executed)
code fence.

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/rocdown/Guide.rocdown
```

This opens an embedded window on a free local TCP port and prints the URL.
Pass `--no-window` to serve on [http://127.0.0.1:8000](http://127.0.0.1:8000).
The page route is `/guides/rocdown/`; GET `/` serves the same document.

Language and compiler status: [`crates/rocci-rocdown`](../../crates/rocci-rocdown).
