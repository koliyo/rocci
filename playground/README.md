# Rocci Playground Web Application

The interactive, in-browser editor and workbench for experimenting with `.rocci` templates and `.rocdown` documents in real time.

## Architecture

- **Editor Shell** (`src/editor.ts`): Built on CodeMirror 6 with dynamic syntax decoration layers and live gutter diagnostics.
- **Compiler Worker** (`src/compiler-worker.ts`): Runs `compiler.wasm` inside a Web Worker off the main UI thread.
- **Worker Client** (`src/worker-client.ts`): Handles 120ms debouncing, monotonic revision sequence tracking, stale result dropping, and worker lifecycle recovery.
- **Workbench Application** (`src/app.ts`): Coordinates tab navigation, output mode switching (`roc`, `AST`, `html`), copy-to-clipboard actions, and status updates.
- **Theme & Styles** (`src/styles.css`): Shares color variables and design tokens with `RocdownTheme.rocci`, with full dark/light modes and `@media (forced-colors: active)` support.

## Building the Bundle

```sh
# Full build: compiles WASM and produces standalone dist/ bundle with SHA-256 manifest
./scripts/build-playground.sh
```

## Running the Playground

```sh
# Run standalone playground with a local .rocci template
cargo run -p rocci-cli -- playground examples/counter/Counter.rocci

# Run standalone playground with a local .rocdown document
cargo run -p rocci-rocdown-cli -- playground examples/rocdown/Guide.rocdown
```
