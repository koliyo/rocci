# rocci-roc-host

Host execution, two-tier caching, and WebAssembly evaluation runtime for compiled Roc and Rocci renderers.

`rocci-roc-host` is a base-Rocci crate that manages renderer compilation, fingerprint-based artifact caching in `~/.rocci/cache/` (or `ROCCI_CACHE`), and renderer execution across both native subprocesses and an in-process Wasmtime WebAssembly host.

## Two-tier cache

Deterministic two-tier caching:

- **Tier 1 (generated Roc):** caches lowered `.roc` modules by input SHA-256 so template lowering can be skipped.
- **Tier 2 (compiled renderers):** caches compiled binaries (`apply` for native, `components.wasm` for WebAssembly) under `renderers/<hash>/`.

Git dirty state is never consulted. A local `Cargo.toml` pin or an unrelated edit does not miss the cache unless it changes one of the hashed inputs below.

### Lookup

`inspect_renderer` / `lookup_renderer` require a hit to satisfy all of:

1. A directory exists for the compile hash and target (`apply` or `components.wasm`).
2. The artifact SHA-256 matches `artifact.sha256` and the manifest (corrupt entries are deleted).
3. Stored `fingerprints.json` matches the caller’s current input fingerprints by **path + sha256** (mtime is ignored). A missing file, an added or removed path, or a changed digest is `Stale`.

Fingerprints were always written on store. Lookup now compares them, so a hash collision or an incomplete hash cannot reuse a binary whose inputs drifted.

### What Rocdown hashes

Site `view` / `build` (`rocci-rocdown` `roc_source_hash`) keys the renderer on:

- `CARGO_PKG_VERSION` of `rocci-rocdown`
- `Html.roc`, `Views.roc`, chrome `NavList` / `Breadcrumbs` / `PageOutline` sources
- generated `RocdownBuild.roc`, `RocdownPages.roc`, theme module Roc, and `main.roc`

Fingerprints stored with that binary also include each theme module’s **source** `.rocci` and the same generated files. Article HTML is staged and applied later; a body-only edit can reuse the binary (`content changed, applying without recompile`) when the hash is unchanged in the same process.

### What is left out on purpose

Do not hash the whole Rust crate, `git status`, or `Cargo.lock`. That would force a multi-second `roc build` on every planner or dependency edit even when generated Roc is identical. A Rust change that actually changes generated Roc or chrome `include_str` already misses.

`goto.js` and other hashed site assets are not renderer inputs; they do not need a Roc recompile.

### Logs

`rocdown view` / `build` print one of:

- `using cached native renderer for <8 hex> (N inputs)` — hash hit and fingerprints match
- `cached native renderer <8 hex> stale (<path> changed|added|removed|missing fingerprints.json)` — same hash, inputs drifted; recompile
- `cached native renderer <8 hex> corrupt` — artifact digest failed; recompile
- `generated … of Roc, compiling (native) with roc` — no usable entry

The eight hex digits are the compile-hash prefix, not a git SHA.

### Cost

A hit already SHA-256s the apply binary for integrity. Fingerprint compare reads a small JSON and compares hex digests already computed while staging. That is cheap next to `roc build` (seconds, ~1 MiB of generated Roc). A stricter “hash every `.rs` file” check would make the common path slow; this check does not.

Old cache entries without `fingerprints.json` miss once (`missing fingerprints.json`) and are rewritten.

## Host choices

- **`HostChoice::Native`**: Compiles via `roc build` against `basic-cli` and executes native subprocess.
- **`HostChoice::Wasm`**: Compiles via `roc build --target=wasm32` against the embedded minimal wasm platform and evaluates in-process using Wasmtime.
- **`HostChoice::Auto`**: Automatically chooses the appropriate host execution model.

## Embedded WebAssembly platform

The crate includes an embedded, relocatable WASI platform (`platform/main.roc`, `platform/host.c`, `platform/targets/wasm32/host.o`) which is staged on demand via `stage_wasm_platform_into` to enable standalone WebAssembly evaluation without requiring an external Roc platform repository.
