# rocci-rocdown-cli

Command-line interface for Rocdown standalone documents, multi-page static documentation sites, and documentation verification.

## Binary

Executable name: `rocdown`

## Commands

```sh
# Preview a single interactive .rocdown or ordinary .md document with live reload.
# Relative links to other documents are compiled and served as extra routes.
# Compile failures print rustc-style frames on stderr unless `--quiet`.
# OKF knowledge records are refused with a pointer to `okmate view`.
# `rocdown run` remains a deprecated alias for `view`.
cargo run -p rocci-rocdown-cli -- view examples/rocdown/pages/Guide.rocdown

# A .rocdown file under a site root (ancestor `rocdown.toml`) previews the
# whole site and opens that page. Includes, aliases, and `/docs/...` links
# resolve the same way as `rocdown view docs`.
cargo run -p rocci-rocdown-cli -- view docs/rocdown/blocks.rocdown

# Preview a documentation site directory with watch and live reload.
# Hybrid sites serve CDN HTML and proxy island handler actions on the same origin.
# `--no-window` prints a URL; append `?reload=0` to pause auto-refresh.
# When a rebuild fails, the preview still serves the last HTML on disk and
# opens a native build-error dialog you can dismiss to read the page.
cargo run -p rocci-rocdown-cli -- view docs [--port 8000] [--no-window] [--no-live-reload] [--public]
cargo run -p rocci-rocdown-cli -- view examples/rocdown/hybrid [--port 8000] [--no-window]
cargo run -p rocci-rocdown-cli -- view examples/rocdown/counter [--port 8000] [--no-window]

# Build a static documentation site to dist/
# --host is apply on this machine; --target is Roc's process ISA/OS (see `rocdown package --help`).
# Hybrid sites emit CDN HTML plus islands.json; --cdn-only errors on live pages.
cargo run -p rocci-rocdown-cli -- build docs [--output dist] [--host auto|native|wasm]
cargo run -p rocci-rocdown-cli -- build examples/rocdown/hybrid --cdn-only

# Package: static CDN tree, or hybrid CDN plus sibling islands binary
cargo run -p rocci-rocdown-cli -- package docs [--output dist] [--archive site.tgz]
cargo run -p rocci-rocdown-cli -- package examples/rocdown/counter --target arm64musl

# Serve a previously built dist/ tree without rebuilding (no Roc, no watch)
cargo run -p rocci-rocdown-cli -- serve dist/docs [--port 8000] [--no-window] [--public]

# Start the island HTTP service for live pages (colocated handlers)
cargo run -p rocci-rocdown-cli -- serve-islands examples/rocdown/hybrid [--port 8000] [--no-window]
cargo run -p rocci-rocdown-cli -- serve-islands examples/rocdown/counter [--port 8000] [--no-window]

# Package/check the rocci.dev site after staging generated example docs
cargo run -q -p rocci-docs -- --catalog examples/rocci/apps.toml --output dist/example-docs
cargo run -p rocci-rocdown-cli -- check site
cargo run -p rocci-rocdown-cli -- package site --target x64musl

# Check documentation catalog, routes, links, includes, and assets without compiling Roc
cargo run -p rocci-rocdown-cli -- check docs [--format terminal|json]

# Run tests for documented examples
cargo run -p rocci-rocdown-cli -- test docs [--update]

# Inspect Rocdown AST and planned artifacts
cargo run -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -p rocci-rocdown-cli -- inspect artifacts docs

# Open the playground with a .rocdown or .rocci file
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/pages/Guide.rocdown
cargo run -p rocci-rocdown-cli -- playground examples/rocci/standalone/counter/Counter.rocci

# Local mode: native parse/lower; Rocdown HTML snapshots are not available yet
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/pages/Guide.rocdown --mode local
```

## Architectural Boundary

`rocci-rocdown-cli` provides the public `rocdown` CLI binary for the Rocdown documentation ecosystem. It consumes `rocci-rocdown` for format parsing, catalog resolution, article rendering, and site generation. It does not parse OKF; `rocdown view` and `rocdown build` refuse knowledge records and bundles and point to `okmate view`.
