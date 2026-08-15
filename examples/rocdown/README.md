# Rocdown examples

Markdown-first pages with colocated Roc and Rocci.

| File | What it shows |
| --- | --- |
| [`Guide.rocdown`](Guide.rocdown) | Static article: `@roc` values, a component, CSS, displayed fences |
| [`Interactive.rocdown`](Interactive.rocdown) | Datastar toggles and a server `@on:post` reveal patch |

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-cli -- run examples/rocdown/Guide.rocdown
cargo run -q -p rocci-cli -- run examples/rocdown/Interactive.rocdown
```

Each command opens an embedded window on a free local TCP port and prints the
URL. Pass `--no-window` to serve on
[http://127.0.0.1:8000](http://127.0.0.1:8000).

| File | Page route | Notes |
| --- | --- | --- |
| Guide | `/guides/rocdown/` | Default HTML shell; no client JS |
| Interactive | `/guides/rocdown-interactive/` | `PageShell` loads Datastar; toggles + POST reveal |

GET `/` serves the same document as the page route.

Language and compiler status: [`crates/rocci-rocdown`](../../crates/rocci-rocdown).
