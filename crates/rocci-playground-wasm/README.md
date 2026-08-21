# rocci-playground-wasm

WebAssembly bridge crate providing C-ABI memory exports and `wasm-bindgen` entrypoints for `rocci-playground`.

This crate compiles to the `wasm32-unknown-unknown` target, packaging the compiler into a lightweight, standalone WebAssembly binary suitable for browser workers.

## Features

- **Raw C-ABI Exports**: `playground_alloc`, `playground_dealloc`, `playground_compile_raw` for zero-overhead host interoperability.
- **Panic Containment**: Catches panics across the WASM boundary and transforms them into structured JSON error diagnostics instead of aborting the Web Worker.
- **Strict Size Budgets**:
  - Uncompressed binary size: **< 1.5 MB** (current: ~994 KB).
  - Gzipped binary size: **< 400 KB** (current: ~310 KB).

## Building WASM

```sh
# Build optimized release WASM binary
cargo build -p rocci-playground-wasm --target wasm32-unknown-unknown --release
```

## Testing & Verification

```sh
# Run browser and Node verification harness
node test/wasm/test-phase2-wasm.mjs
```
