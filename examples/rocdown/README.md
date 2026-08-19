# Rocdown examples

Markdown-first pages with colocated Roc and Rocci.

| File | What it shows |
| --- | --- |
| [`Guide.rocdown`](Guide.rocdown) | Static article: `@roc` values, a document-root component tag, wiki links, `rocci` theme |
| [`Blocks.rocdown`](Blocks.rocdown) | `:note`, `:steps`, `:tabs`, `:figure`, and other article blocks |
| [`Interactive.rocdown`](Interactive.rocdown) | Datastar toggles and a server `@on:post` reveal patch |

From the repository root, with `roc` and `cargo` on `PATH`:

```sh
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/Guide.rocdown
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/Blocks.rocdown
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown/Interactive.rocdown
```

Each command opens an embedded window on a free local TCP port and prints the
URL. Pass `--no-window` to serve on
[http://127.0.0.1:8000](http://127.0.0.1:8000). Running one `.rocdown` file also
serves sibling pages, so Guide's `[[Interactive]]` link reaches
`/guides/rocdown-interactive/`. GET `/` still opens the file you passed.

| File | Page route | Notes |
| --- | --- | --- |
| Guide | `/guides/rocdown/` | Default HTML shell; no client JS |
| Blocks | `/guides/rocdown-blocks/` | Article blocks; conservative standalone preview |
| Interactive | `/guides/rocdown-interactive/` | `PageShell` loads Datastar; toggles + POST reveal |

GET `/` serves the same document as the page route.

While the server is up, a missing path such as `/missing` is an HTML 404 that
lists these routes. Dedicated error-page examples live in
[`examples/errors`](../errors).

Language and compiler status: [`crates/rocci-rocdown`](../../crates/rocci-rocdown).
