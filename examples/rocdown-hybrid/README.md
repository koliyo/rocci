# Hybrid islands fixture

Four-page site used by Rocdown CLI tests: `static`, `hydrate`, and `live`
page kinds with colocated `@on` (ephemeral show/hide, no SQLite).

| File | Kind | What it shows |
| --- | --- | --- |
| [`index.rocdown`](index.rocdown) | live | RevealTip + `@on:post` morph |
| [`widgets.rocdown`](widgets.rocdown) | hydrate | Pure `@component` / `@render`, no Datastar |
| [`pair.rocdown`](pair.rocdown) | live | Two hosts with distinct element ids |
| [`about.rocdown`](about.rocdown) | static | Markdown-only neighbor |

For a SQLite-backed counter and the CDN + `serve-islands` deploy runbook, use
[`examples/rocdown-counter`](../rocdown-counter).

```sh
cargo run -q -p rocci-rocdown-cli -- run examples/rocdown-hybrid --no-window
cargo run -q -p rocci-rocdown-cli -- build examples/rocdown-hybrid
cargo run -q -p rocci-rocdown-cli -- serve-islands examples/rocdown-hybrid --no-window
```
