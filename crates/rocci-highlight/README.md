# rocci-highlight

Shared lexical and semantic syntax highlighting library for Roc, HTML, CSS,
Rocci templates, and Rocdown documents.

## Architecture

- **Zero dependencies on LSP or Rocs**: Operates as a pure library with no wire protocols, no server process, and no HTML formatting constraints.
- **Pinned Tree-sitter backends**: Compiles embedded C grammars for `roc`, `css`, and `html` offline.
- **Canonical token spans**: Emits sorted, non-overlapping `HighlightSpan` structures that drive both LSP semantic tokens and static documentation HTML generators.
