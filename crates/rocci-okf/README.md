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

The review viewer uses a dark One Dark Pro palette. Home and Governance &
Review stay in `.okf-global-nav` at every width. Pages with H2 or H3 headings
also get a left “On this page” navigator; below `48rem` that outline hides and
a no-JS `<details class="okf-outline-menu">` control appears. Wide source and
review tables scroll inside `.okf-table-container`. Authored knowledge links
such as `/decisions/foo.md` are bundle-root paths;
the review site publishes them at `/decisions/foo/` and writes collection
indexes such as `/architecture/`. Cmd/Ctrl-K opens a fuzzy page palette
backed by `/pages.json` and `/catalog.json` in the review tree; same-origin
links swap already-rendered HTML without a full reload.

Default `run` uses the cached Rocci renderer when `roc` is on PATH. If `roc`
is missing, preview writes the Rust knowledge shell unless you pass
`--host native` or set `ROCCI_REQUIRE_ROC=1`. `--host native` is the explicit
compile path; `--host wasm` selects the in-process Wasmtime host.

```sh
# Run with desktop window. Rocci schema is on; git provenance is off.
cargo run -p rocci-okf -- run knowledge

# Open a concept inside the enclosing bundle
cargo run -p rocci-okf -- run knowledge/plans/cli-entry-points.md

# Turn git provenance (OKF4006/4007/4008) back on for preview
cargo run -p rocci-okf -- run knowledge --provenance

# Run headless (prints server URL). Append `?reload=0` to pause auto-refresh.
cargo run -p rocci-okf -- run knowledge --no-window --port 8000

# Pause automatic page refresh (watch/rebuild still runs)
cargo run -p rocci-okf -- run knowledge --no-live-reload
```

`run` without the browser still opens today's one-shot preview window.

```sh
# Speak the rocci-browser adapter protocol on stdio (probe / listDocuments / open)
cargo run -p rocci-okf -- browser-adapter
```

`--profile base` is portable OKF, not the fast Rocci preview path. Use default
`run` (Rocci schema, no git provenance) for local authoring, and
`check --profile rocci` when reviewing or in CI.

`run` persists parsed Markdown under `ROCCI_CACHE` (default `~/.rocci/cache`)
in `okf-parse/`, so a new process can reuse unchanged documents. `check` always
parses the bundle fresh.

### Build Artifacts & Review Site

```sh
cargo run -p rocci-okf -- build knowledge -o dist/knowledge --profile rocci
```
