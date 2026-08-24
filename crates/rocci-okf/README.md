# rocci-okf

Rocci Open Knowledge Format (OKF) review and query application.

`rocci-okf` is the specialized application binary and dev server for managing, querying, and reviewing Rocci's knowledge base.

## Usage

### Validation & Check

```sh
cargo run -p rocci-okf -- check knowledge --profile rocci --format terminal
```

### Inspect

```sh
# Inspect normalized concept representation
cargo run -p rocci-okf -- inspect concept architecture/system-overview knowledge

# Inspect whole catalog as JSON
cargo run -p rocci-okf -- inspect catalog knowledge

# Inspect directed link graph
cargo run -p rocci-okf -- inspect graph knowledge
```

### Search

```sh
cargo run -p rocci-okf -- search "system overview" knowledge --profile rocci
```

### Retrieval Benchmarks

```sh
cargo run -p rocci-okf -- benchmark knowledge/retrieval-benchmark.toml knowledge --profile rocci
```

### Live Reload Review Server & Desktop Preview

The review viewer uses a dark One Dark Pro palette. The left chrome has
Dashboard, a Review section, and a collection tree. Pages with H2 or H3
headings get a right “On this page” navigator; below `48rem` both the tree and
that outline hide behind no-JS `<details>` menus. The dashboard lists recent
documents first and a compact review-queue button; `/review/` keeps the full
queue. Wide source and review tables scroll inside `.okf-table-container`.
Authored knowledge links such as `/decisions/foo.md` are bundle-root paths;
the review site publishes them at `/decisions/foo/` and writes collection
indexes such as `/architecture/`. Cmd/Ctrl-K opens a fuzzy page palette
backed by `/pages.json` and `/catalog.json` in the review tree (refetched when
the palette opens); same-origin links swap already-rendered HTML without a
full reload.

`rocci-okf` with no subcommand, and `view` with no path, restore the last
bundle and document from `~/.rocci/state/okf.json` (or `ROCCI_STATE_DIR`).
Home in the preview window always opens the dashboard (`/`). Preview servers
listen on localhost unless you pass `--public` (binds `0.0.0.0`; inspector and
`/__rocci/dev` stay loopback-only).

On macOS, `rocci-ops bundle okf` builds an ad-hoc signed `Rocci Knowledge.app`
under `target/release/bundle/macos/`.

Default `view` uses the cached Rocci renderer when `roc` is on PATH. If `roc`
is missing, preview writes the Rust knowledge shell unless you pass
`--host native` or set `ROCCI_REQUIRE_ROC=1`. `--host native` is the explicit
compile path; `--host wasm` selects the in-process Wasmtime host.
`rocci-okf run` remains a deprecated alias for `view`.

```sh
# Preview with desktop window. Rocci schema is on; git provenance is off.
cargo run -p rocci-okf -- view knowledge

# Open a concept inside the enclosing bundle
cargo run -p rocci-okf -- view knowledge/plans/shared/cli-entry-points.md

# Turn git provenance (OKF4006/4007/4008) back on for preview
cargo run -p rocci-okf -- view knowledge --provenance

# Preview headless (prints server URL). Append `?reload=0` to pause auto-refresh.
cargo run -p rocci-okf -- view knowledge --no-window --port 8000

# Restore the last bundle (or ./knowledge) with no path
cargo run -p rocci-okf -- view

# Listen on every interface (default is localhost only)
cargo run -p rocci-okf -- view knowledge --no-window --public

# Pause automatic page refresh (watch/rebuild still runs)
cargo run -p rocci-okf -- view knowledge --no-live-reload
```

`view` without `--no-window` still opens today's one-shot preview window.

`--profile base` is portable OKF, not the fast Rocci preview path. Use default
`view` (Rocci schema, no git provenance) for local authoring, and
`check --profile rocci` when reviewing or in CI.

`run` still builds the review site when the bundle has validation errors. The
preview shows those diagnostics on the page and in the inspector console, and
renders each document as far as it parsed. `check` and `build` keep failing on
errors.

`run` persists parsed Markdown under `ROCCI_CACHE` (default `~/.rocci/cache`)
in `okf-parse/`, so a new process can reuse unchanged documents. `check` always
parses the bundle fresh.

### Build Artifacts & Review Site

```sh
cargo run -p rocci-okf -- build knowledge -o dist/knowledge --profile rocci
```
