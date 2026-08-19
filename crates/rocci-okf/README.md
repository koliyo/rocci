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

The review viewer uses a dark One Dark Pro palette. Pages with H2 or H3
headings get a left “On this page” navigator, matching standalone Rocdown.
Authored knowledge links such as `/decisions/foo.md` are bundle-root paths;
the review site publishes them at `/decisions/foo/` and writes collection
indexes such as `/architecture/`.

```sh
# Run with desktop window. Rocci schema is on; git provenance is off.
cargo run -p rocci-okf -- run knowledge

# Open a concept inside the enclosing bundle
cargo run -p rocci-okf -- run knowledge/plans/cli-entry-points.md

# Turn git provenance (OKF4006/4007/4008) back on for preview
cargo run -p rocci-okf -- run knowledge --provenance

# Run headless (prints server URL)
cargo run -p rocci-okf -- run knowledge --no-window --port 8000
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
