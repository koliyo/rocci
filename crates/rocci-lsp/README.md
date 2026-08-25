# `rocci-lsp`

Generic language-server core and the `RocciAnalyzer` for `.rocci` templates.
This crate is a library. The shipped `rocci-language-server` binary lives in
`rocci-rocdown-lsp`, which composes this analyzer with `RocdownAnalyzer`.

---

## Architecture Overview

`rocci-lsp` stores open documents, recompiles after full-text open or change
notifications, and lets a `DocumentAnalyzer` supply diagnostics, symbols,
hover, completion, definition, and semantic tokens. `RocciAnalyzer` is the
built-in `.rocci` implementation. Rocdown analysis stays in `rocci-rocdown`.

```text
Source Document (.rocci)
        │
        ▼
Host Parser & AST Validation (rocci-template)
        │
        ├── Push Diagnostics & Document Outline / Symbols
        │
        ▼
Typed Region Graph (RegionTree)
        │
        ├──────────────────────┬──────────────────────┬──────────────────────┐
        ▼                      ▼                      ▼                      ▼
  Executable Roc         Embedded CSS          Display Fences         Host Structure
 (Tree-sitter Roc)     (Tree-sitter CSS)     (Tree-sitter HTML)      (Rocci AST Tokens)
        │                      │                      │                      │
        └──────────────────────┼──────────────────────┴──────────────────────┘
                               │
                               ▼
            Semantic Token Compositor & Encoded Stream
          (Priority resolution, bounds clipping, UTF-8/16)
```

---

## Key Capabilities

1. **Boundary Authority**:
   - `rocci-template` is the sole authority for `.rocci` syntax boundaries.
   - Distinct `RegionPurpose` separates `Executable` islands from `DisplayOnly` code fences so display code examples never alter execution semantics.

2. **In-Process Embedded Highlighting**:
   - Embedded Roc: keywords, functions, types, constructors/variants, parameters, strings, numbers, operators, comments.
   - Embedded CSS: selectors, property names, values, units, `@media`/`@keyframes` at-rules, comments.
   - Display-only HTML: element tags, attributes, strings.
   - Markdown: headings, bold, italic, inline code, links, list markers.

3. **Resilient Token Composition**:
   - Monotonic line/column positioning with fast `LineIndex` binary searching.
   - Strict non-overlap guarantee and single-line token span splitting.
   - Full support for both `PositionEncoding::Utf8` and `PositionEncoding::Utf16` negotiation, including non-BMP Unicode characters.

4. **LSP Features**:
   - `textDocument/semanticTokens/full` and `textDocument/semanticTokens/range`
   - `textDocument/documentSymbol`
   - `textDocument/definition` (same-file component navigation; compiler definition in executable Roc)
   - `textDocument/hover` and `textDocument/completion` (compiler-backed in executable Roc when `roc` is available)
   - `textDocument/references` in executable Roc when the child answers
   - `textDocument/publishDiagnostics` with host recovery plus remapped `source: "roc"` compiler diagnostics
   - Custom `rocci/inspectRegions` request for region inspection and debugging

Host hover, tokens, and diagnostics stay available when `roc` is missing or the optional child crashes. Display-only fences are never forwarded.

### Optional `roc experimental-lsp` child

The product binary `rocci-language-server` (`rocci-rocdown-lsp`) may spawn one
`roc experimental-lsp --stdio` process per workspace. It types generated
projection modules and maps results through source-map segments.

Resolution order for the compiler binary:

1. `ROCCI_ROC_PATH`
2. `roc` on `PATH`

VS Code sets `ROCCI_ROC_PATH` from `rocci.roc.path`. Zed can set the same
variable through `lsp.rocci-language-server.binary.env` or `settings.rocPath`.
**Rocci: Restart LSP server** respawns the child. Default `cargo test -p rocci-lsp`
does not require Roc; live child tests use `ROCCI_REQUIRE_ROC=1`.

---

## Performance and Invariants

- **Cold Start**: < 1 ms server initialization and grammar query loading.
- **Small Documents**: < 2 ms for full token generation on typical template files.
- **Large Documents**: < 90 ms for full token generation on a 10,000-line document (35,000+ tokens).
- **Single-Character Updates**: ~2 ms turnaround for incremental changes and re-tokenization.
- **Invariants Verified**:
  - `tests/perf.rs`: Performance latency benchmarks across small, 1k, 5k, and 10k line fixtures.
  - `tests/fuzz_invariants.rs`: Comprehensive property tests covering arbitrary byte slicing, non-BMP characters, truncated constructs, and 5,000-iteration mutation fuzzing.

---

## Testing

```sh
# Run fast unit, server integration, and smoke invariant tests (< 1s)
cargo test -p rocci-lsp

# Run deep invariant property tests and exhaustive byte-slicing stress tests
cargo test -p rocci-lsp --test fuzz_invariants -- --nocapture --ignored

# Run release-mode performance latency benchmarks
cargo test -p rocci-lsp --test perf --release -- --nocapture --ignored
```

---

## Third-Party Licenses

Vendored Tree-sitter grammars and query adaptations are documented in [`THIRD_PARTY_LICENSES.md`](../../THIRD_PARTY_LICENSES.md).
