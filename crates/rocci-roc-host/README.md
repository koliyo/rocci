# rocci-roc-host

Host execution, two-tier caching, and WebAssembly evaluation runtime for compiled Roc and Rocci renderers.

`rocci-roc-host` is a base-Rocci crate that manages renderer compilation, fingerprint-based artifact caching in `~/.rocci/cache/`, and renderer execution across both native subprocesses and an in-process Wasmtime WebAssembly host.

## Two-Tier Cache

`rocci-roc-host` implements deterministic two-tier caching:
- **Tier 1 (Generated Roc)**: Caches Lowered `.roc` modules by input SHA-256 hash to skip template lowering.
- **Tier 2 (Compiled Renderers)**: Caches compiled binaries (`apply` executable for native, `components.wasm` for WebAssembly) with SHA-256 integrity verification.

## Host Choices

- **`HostChoice::Native`**: Compiles via `roc build` against `basic-cli` and executes native subprocess.
- **`HostChoice::Wasm`**: Compiles via `roc build --target=wasm32` against the embedded minimal wasm platform and evaluates in-process using Wasmtime.
- **`HostChoice::Auto`**: Automatically chooses the appropriate host execution model.

## Embedded WebAssembly Platform

The crate includes an embedded, relocatable WASI platform (`platform/main.roc`, `platform/host.c`, `platform/targets/wasm32/host.o`) which is staged on demand via `stage_wasm_platform_into` to enable standalone WebAssembly evaluation without requiring an external Roc platform repository.
