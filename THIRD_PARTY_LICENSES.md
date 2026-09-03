# Third-Party Licenses and Grammar Revisions

This repository incorporates third-party Tree-sitter grammars and query definitions for embedded language highlighting in `rocci-highlight` (used by `rocci-lsp` and static site tooling), and a UPL-1.0 Roc platform host in `rocci-platform`.

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

## 3. Roc platform host (`rocci-platform`)

- **Purpose:** Native HTTP/SSE/SQLite Roc platform used as `pf` by generated and custom Rocci apps.
- **Public origin:** [https://github.com/roc-lang/basic-webserver](https://github.com/roc-lang/basic-webserver)
- **Snapshot:** `50e064cdd1c4562c293598c61f6ce7a895d99bcf` (0.16 line)
- **Copyright:** © 2023 Richard Feldman and subsequent Roc authors
- **License:** Universal Permissive License 1.0
- **License location:** [`crates/rocci-platform/LICENSE-UPL`](crates/rocci-platform/LICENSE-UPL)
- **Rocci-original (Apache-2.0):** `platform/Datastar.roc` and compiler helpers on `platform/Html.roc` (`attribute`, `boolean_attribute`, `empty`, `fragment`). Remaining host Rust and `platform/` Roc in that crate is the UPL snapshot, adapted for `platform "rocci"` and workspace crate versions.

---

## 4. Core Rust Dependencies

| Crate | Version | License | Purpose |
| --- | --- | --- | --- |
| `tree-sitter` | `0.25.10` | MIT | Tree-sitter parsing runtime |
| `lsp-server` | `0.7` | MIT OR Apache-2.0 | Language Server Protocol transport and event loop |
| `lsp-types` | `0.97` | MIT | LSP specification types and serialization |
| `serde` / `serde_json` | `1.0` | MIT OR Apache-2.0 | JSON serialization/deserialization |
