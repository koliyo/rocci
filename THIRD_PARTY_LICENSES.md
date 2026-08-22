# Third-Party Licenses and Grammar Revisions

This repository incorporates third-party Tree-sitter grammars and query definitions for embedded language highlighting in `rocci-highlight` (used by `rocci-lsp` and static site tooling).

---

## 1. Pinned Tree-sitter Grammars

### `tree-sitter-roc`
- **Purpose:** Syntax analysis and semantic highlighting for embedded Roc regions in `.rocci` and `.rocdown` files.
- **Repository:** [https://github.com/faldor20/tree-sitter-roc](https://github.com/faldor20/tree-sitter-roc)
- **Pinned Revision:** `d91c4e8c972ad8aac03fa45414a11f564c2274e3` (2026-08-21)
- **Author:** Eli Dowling
- **License:** MIT
- **License Location:** [`crates/rocci-highlight/grammars/roc/LICENSE`](crates/rocci-highlight/grammars/roc/LICENSE)

### `tree-sitter-css`
- **Purpose:** Syntax analysis and semantic highlighting for `@css` stylesheets in `.rocci` and CSS display code fences in `.rocdown`.
- **Repository:** [https://github.com/tree-sitter/tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)
- **Authors:** Max Brunsfeld and Tree-sitter authors
- **License:** MIT
- **License Location:** [`crates/rocci-highlight/grammars/css/LICENSE`](crates/rocci-highlight/grammars/css/LICENSE)

### `tree-sitter-html`
- **Purpose:** Syntax analysis and semantic highlighting for display-only HTML code fences in `.rocdown`. (Note: Executable `.rocci` HTML-shaped templates are parsed by `rocci-template`, not generic HTML).
- **Repository:** [https://github.com/tree-sitter/tree-sitter-html](https://github.com/tree-sitter/tree-sitter-html)
- **Authors:** Max Brunsfeld and Tree-sitter authors
- **License:** MIT
- **License Location:** [`crates/rocci-highlight/grammars/html/LICENSE`](crates/rocci-highlight/grammars/html/LICENSE)

---

## 2. Query Adaptations

### `zed-roc` Highlight Queries
- **Purpose:** Roc highlight query patterns (`highlights.scm`) adapted to map captures into standard Language Server Protocol (LSP) semantic token types.
- **Repository:** [https://github.com/h2000/zed-roc](https://github.com/h2000/zed-roc)
- **Pinned Revision:** `f6a07bfb336549724f9c5694084bfb1869614b5d` (2026-06-26)
- **Author:** Alf Richter
- **License:** MIT

---

## 3. Core Rust Dependencies

| Crate | Version | License | Purpose |
| --- | --- | --- | --- |
| `tree-sitter` | `0.25.10` | MIT | Tree-sitter parsing runtime |
| `lsp-server` | `0.7` | MIT OR Apache-2.0 | Language Server Protocol transport and event loop |
| `lsp-types` | `0.97` | MIT | LSP specification types and serialization |
| `serde` / `serde_json` | `1.0` | MIT OR Apache-2.0 | JSON serialization/deserialization |
