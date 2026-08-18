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

```sh
# Run with desktop window
cargo run -p rocci-okf -- run knowledge

# Open a concept inside the enclosing bundle
cargo run -p rocci-okf -- run knowledge/plans/cli-entry-points.md

# Run headless (prints server URL)
cargo run -p rocci-okf -- run knowledge --no-window --port 8000
```

### Build Artifacts & Review Site

```sh
cargo run -p rocci-okf -- build knowledge -o dist/knowledge --profile rocci
```
