# rocci-rocdown-cli

Command-line interface for Rocdown standalone documents, multi-page static documentation sites, and documentation verification.

## Binary

Executable name: `rocdown`

## Commands

```sh
# Run a single interactive .rocdown or ordinary .md document with live reload.
# Relative links to other documents are compiled and served as extra routes.
# Compile failures print rustc-style frames on stderr unless `--quiet`.
# OKF knowledge records are refused with a pointer to `rocci-okf run`.
cargo run -p rocci-rocdown-cli -- run examples/rocdown/Guide.rocdown

# A .rocdown file under a site root (ancestor `rocdown.toml`) previews the
# whole site and opens that page. Includes, aliases, and `/docs/...` links
# resolve the same way as `rocdown run docs`.
cargo run -p rocci-rocdown-cli -- run docs/guides/docs-components.rocdown

# Run/preview a documentation site directory with watch and live reload.
# Hybrid sites serve CDN HTML and proxy island @on actions on the same origin.
cargo run -p rocci-rocdown-cli -- run docs [--port 8000] [--no-window]
cargo run -p rocci-rocdown-cli -- run examples/rocdown-hybrid [--port 8000] [--no-window]

# Build a static documentation site to dist/
# Hybrid sites emit CDN HTML plus islands.json; --cdn-only errors on live pages.
cargo run -p rocci-rocdown-cli -- build docs [--output dist]
cargo run -p rocci-rocdown-cli -- build examples/rocdown-hybrid --cdn-only

# Start the island HTTP service for live pages (colocated @on handlers)
cargo run -p rocci-rocdown-cli -- serve-islands examples/rocdown-hybrid [--port 8000] [--no-window]

# Check documentation catalog, routes, links, includes, and assets without compiling Roc
cargo run -p rocci-rocdown-cli -- check docs [--format terminal|json]

# Run tests for documented examples
cargo run -p rocci-rocdown-cli -- test docs [--update]

# Inspect Rocdown AST and planned artifacts
cargo run -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -p rocci-rocdown-cli -- inspect artifacts docs

# Open the playground with a .rocdown or .rocci file
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/Guide.rocdown
cargo run -p rocci-rocdown-cli -- playground examples/counter/Counter.rocci

# Local mode: native parse/lower; Rocdown HTML snapshots are not available yet
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/Guide.rocdown --mode local
```

## Architectural Boundary

`rocci-rocdown-cli` provides the public `rocdown` CLI binary for the Rocdown documentation ecosystem. It consumes `rocci-rocdown` for format parsing, catalog resolution, article rendering, and site generation. It does not parse OKF; `rocdown run` and `rocdown build` refuse knowledge records and bundles and point to `rocci-okf run`.
