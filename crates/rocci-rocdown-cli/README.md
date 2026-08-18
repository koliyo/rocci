# rocci-rocdown-cli

Command-line interface for Rocdown standalone documents, multi-page static documentation sites, and documentation verification.

## Binary

Executable name: `rocdown`

## Commands

```sh
# Run a single interactive .rocdown or ordinary .md document with live reload.
# Relative links to other documents are compiled and served as extra routes.
# OKF knowledge records are refused with a pointer to `rocci-okf run`.
cargo run -p rocci-rocdown-cli -- run examples/rocdown/Guide.rocdown

# Run/preview a documentation site directory with watch and live reload
cargo run -p rocci-rocdown-cli -- run docs [--port 8000] [--no-window]

# Build a static documentation site to dist/
cargo run -p rocci-rocdown-cli -- build docs [--output dist]

# Check documentation catalog, routes, links, includes, and assets without compiling Roc
cargo run -p rocci-rocdown-cli -- check docs [--format terminal|json]

# Run tests for documented examples
cargo run -p rocci-rocdown-cli -- test docs [--update]

# Inspect Rocdown AST and planned artifacts
cargo run -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -p rocci-rocdown-cli -- inspect artifacts docs
```

## Architectural Boundary

`rocci-rocdown-cli` provides the public `rocdown` CLI binary for the Rocdown documentation ecosystem. It consumes `rocci-rocdown` for format parsing, catalog resolution, article rendering, and site generation. It does not parse OKF; `rocdown run` and `rocdown build` refuse knowledge records and bundles and point to `rocci-okf run`.
