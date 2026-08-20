# Rocci Playground Web Application

The interactive, in-browser editor and workbench for experimenting with `.rocci` templates and `.rocdown` documents in real time.

## Architecture

- **Editor Shell** (`src/editor.ts`): Built on CodeMirror 6 with dynamic syntax decoration layers and live gutter diagnostics.
- **Compiler Worker** (`src/compiler-worker.ts`): Runs `compiler.wasm` inside a Web Worker off the main UI thread (WASM mode).
- **HTTP Compiler Client** (`src/http-client.ts`): Posts compile requests to `/api/compile` in desktop `--mode local`.
- **Worker Client** (`src/worker-client.ts`): Handles 120ms debouncing, monotonic revision sequence tracking, stale result dropping, and worker lifecycle recovery.
- **Workbench Application** (`src/app.ts`): Coordinates the source-language dropdown (`rocci` / `rocdown`), example tabs, output mode switching (`roc`, `AST`, `html`), copy-to-clipboard actions, and status updates. Switching language loads that language's example if one was provided, otherwise an empty untitled buffer.
- **Theme & Styles** (`src/styles.css`): Shares color variables and design tokens with `RocdownTheme.rocci`, with full dark/light modes and `@media (forced-colors: active)` support.

## Building the Bundle

```sh
# Full build: compiles WASM and produces standalone dist/ bundle with SHA-256 manifest
./scripts/build-playground.sh
```

## Running the Playground

```sh
# WASM mode (default): parse/lower in compiler.wasm. HTML preview is unavailable.
cargo run -p rocci-cli -- playground examples/rocci/standalone/counter/Counter.rocci

# Local mode: native compile plus a static Html.render snapshot when a fixture or defaultable component exists.
cargo run -p rocci-cli -- playground examples/rocci/standalone/counter/Counter.rocci --mode local

# Run standalone playground with a local .rocdown or .rocci document
cargo run -p rocci-cli -- playground examples/rocdown/pages/Guide.rocdown --mode local
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/pages/Guide.rocdown
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/pages/Guide.rocdown --mode local
```
