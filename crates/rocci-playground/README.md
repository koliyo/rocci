# rocci-playground

Target-neutral compiler core and data protocol for the Rocci in-browser WebAssembly playground.

`rocci-playground` provides the platform-independent protocol, serialization types, and compiler dispatch logic that bridges `.rocci` templates and `.rocdown` documents to WebAssembly without requiring filesystem, network, or OS runtime dependencies. Desktop `--mode local` uses the same protocol over `POST /api/compile` from `rocci-cli` / `rocci-rocdown-cli`.

## Architecture & Responsibilities

- **Protocol Types**: Defines `CompileRequest`, `CompileResponse`, `PlaygroundDiagnostic`, `PlaygroundHighlightSpan`, `PlaygroundCapabilities`, and `VirtualWorkspace`.
- **Language Dispatch**: Dispatches compilation for `.rocci` and `.rocdown` files using browser-safe lowering options.
- **UTF-16 Code-Unit Mapping**: Accurately maps byte ranges and error spans to UTF-16 code units (correctly handling non-BMP surrogate pairs like emoji) for CodeMirror 6 text decorations.
- **S-Expression AST Highlighter**: Scans S-expression AST strings and emits non-overlapping, sorted syntax tokens mapped to canonical `tok-*` CSS classes.
- **Virtual Workspace**: Safely manages in-memory multi-file sets with strict memory budgets and path traversal guards (`..` and `/` rejections).

## Data Types

```rust
use rocci_playground::{CompileRequest, CompileResponse, Language, compile};

let req = CompileRequest {
    protocol_version: 1,
    revision: 42,
    filename: "Counter.rocci".to_string(),
    language: Some(Language::Rocci),
    source: "@component Counter = |{ count }| { <button>{count}</button> }".to_string(),
    workspace: None,
};

let resp: CompileResponse = compile(&req);
assert!(!resp.has_errors);
assert!(resp.roc.contains("counter ="));
```

## Testing & Verification

```sh
# Unit and parity tests
cargo test -p rocci-playground

# Invariant fuzzing on token ordering and UTF-16 conversions
cargo test -p rocci-playground --test highlight_invariants
```
