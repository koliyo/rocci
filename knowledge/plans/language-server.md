---
type: Implementation Plan
title: Full Rocci and Rocdown language server
description: Build one region-aware language server for VS Code and Zed, beginning with common semantic-token highlighting for embedded Roc, CSS, HTML-shaped templates, Markdown, and display fences.
tags: [domain/rocci, domain/rocdown, integration/roc, concern/tooling, concern/syntax]
status: draft
generated: { by: process:codex, at: 2026-08-17T05:44:10Z }
stale_after: 2026-10-01
authority: exploratory
owners: [human:nils]
sources:
  - id: detailed-plan
    resource: ../../ROCCI_LANGUAGE_SERVER_IMPLEMENTATION_PLAN.md
    title: Rocci language-server report and detailed implementation plan
    author: process:codex
    last_modified: 2026-08-17
  - id: tooling-architecture
    resource: ../architecture/language-tooling.md
    title: Current Rocci language-tooling boundary
    author: process:codex
    last_modified: 2026-08-17
  - id: rocdown-boundary
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:codex
    last_modified: 2026-08-16
  - id: source-map
    resource: ../../crates/rocci-template/src/source_map.rs
    title: Current generated-Roc source-map segment model
    author: process:git
    last_modified: 2026-08-15
  - id: zed-roc
    resource: https://github.com/h2000/zed-roc/tree/f6a07bfb336549724f9c5694084bfb1869614b5d
    title: Zed Roc extension at inspected revision
    author: human:alf-richter
    last_modified: 2026-06-26
  - id: tree-sitter-roc
    resource: https://github.com/faldor20/tree-sitter-roc/tree/edc18052a9d7382ac9f9f5bf413db3a78d5ea12c
    title: Roc Tree-sitter grammar pinned by zed-roc
    author: human:eli-dowling
    last_modified: 2026-01-27
  - id: zed-languages
    resource: https://zed.dev/docs/extensions/languages
    title: Zed language extension documentation
    author: organization:zed-industries
    last_modified: 2026-08-17
  - id: vscode-embedded
    resource: https://code.visualstudio.com/api/language-extensions/embedded-languages
    title: VS Code embedded-language guidance
    author: organization:microsoft
    last_modified: 2026-08-17
  - id: vscode-semantic
    resource: https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide
    title: VS Code semantic highlighting guide
    author: organization:microsoft
    last_modified: 2026-08-17
  - id: tree-sitter-highlighting
    resource: https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html
    title: Tree-sitter syntax-highlighting documentation
    author: organization:tree-sitter
    last_modified: 2026-08-17
---

# Full Rocci and Rocdown language server

## Goal

Provide a single `rocci-language-server` that owns Rocci/Rocdown analysis and
composes embedded-language results into ordinary LSP responses for VS Code and
Zed. The first milestone is syntax highlighting for embedded Roc and CSS plus
HTML-shaped Rocci templates and Rocdown Markdown; the full target adds
workspace navigation, safe rename and formatting, and compiler-backed Roc
semantics.[^detailed-plan]

This record is a proposed implementation sequence, not an approved or shipped
contract.

## Direction

Use the Rocci and Rocdown parsers as the only authority for language
boundaries. Model each region with a language, syntactic context, purpose, byte
span, parent, and precedence. Purpose must distinguish executable regions from
display-only fences so syntax highlighting never changes execution
semantics.[^rocdown-boundary][^detailed-plan]

Use source-preserving projections for fast lexical features and the compiler's
generated Roc plus an expanded bidirectional source map for later type-aware
Roc features. Compose backend results once in source byte spans before
converting to the client's position encoding.[^source-map][^detailed-plan]

Reuse the Roc Tree-sitter grammar and adapted highlight queries from the
`zed-roc` dependency chain, with pinned revisions and license attribution. Do
not merge the Zed manifest or its direct Roc-server launcher into the common
server. Zed-specific queries and VS Code-specific forwarding are evidence, not
portable LSP APIs.[^zed-roc][^tree-sitter-roc][^zed-languages][^vscode-embedded]

## Prerequisites

- Restore the `rocci-lsp` build after Rocdown `Item::Docs` and define its
  symbols/tokens.
- Add comprehensive embedded-language fixtures and region/token goldens.
- Smoke-test the checked-in VS Code and Zed development extensions on declared
  editor versions.
- Select one compatible Tree-sitter runtime and pinned Roc/CSS/HTML grammar set.
- Record third-party license notices for copied queries or vendored parsers.

## Phases

### 0. Compatibility baseline

Repair the `@docs` LSP regression, freeze current behavior with fixtures, and
verify both clients attach to the same server. Exit when `cargo test -p
rocci-lsp` passes and editor prerequisites are reproducible.[^tooling-architecture]

### 1. Embedded-highlighting demonstrator

Extract a typed region graph, add pinned in-process Tree-sitter backends for
Roc, CSS, and ordinary/display-only HTML, retain Rocci-AST highlighting for
executable HTML-shaped templates, add Markdown and display-fence tokens, and
merge all streams into standard non-overlapping semantic tokens. VS Code and
Zed must render the same fixture through one server
binary.[^tree-sitter-highlighting][^vscode-semantic]

### 2. Structural editing

Add versioned snapshots, incremental text changes, cancellation, folding,
selection ranges, document links, linked tags, richer completion/hover,
syntax code actions, and semantic-token deltas. These features must work
without Roc installed.

### 3. Workspace intelligence

Index modules, components, pages, routes, headings, links, styles, selectors,
and literal class/id uses. Add cross-file definition, references, workspace
symbols, and conservative rename with dependency-based invalidation. Reuse
Rocs catalog logic for page and link semantics.

### 4. Roc semantics

Prototype one optional Roc child server per workspace against generated Roc
modules. Expand source maps for exact bidirectional mapping and add mapped type
diagnostics, hover, completion, signatures, definitions, and references.
Reject ambiguous edits and preserve a useful degraded mode when the Roc
backend is absent or incompatible.[^source-map]

### 5. Formatting and refactoring

Define a lossless host formatter, compose region-owned Roc/CSS/Markdown edits,
and add refactors only where mappings are reversible. Formatting must be
idempotent and preserve the executable/display-only boundary.

### 6. Productization

Ship versioned platform binaries, adapter compatibility policy, grammar and
protocol revision reporting, performance/cancellation budgets, fuzz and crash
hardening, license inventory, and release smoke tests for supported VS Code
and Zed versions.

## Validation and exit criteria

- Every language region and malformed boundary has a byte-span golden test.
- Semantic tokens never overlap, escape a region, or expose synthetic wrapper
  text; UTF-8 and UTF-16 results are equivalent.
- Range and delta tokens reproduce the corresponding full-token state.
- Fenced examples can be highlighted but are never classified as executable.
- Child-backend locations, diagnostics, and edits map only to authored spans;
  stale or ambiguous edits are refused.
- Both editors pass client smoke tests with the same server and fixtures.
- Host functionality remains available when optional embedded backends fail.
- Performance, compatibility, and third-party licenses are recorded before a
  release.

## Open gates

The project must choose the Roc grammar revision/runtime combination, verify
current Zed's grammar requirement against the existing adapter, prove the Roc
child server's generated-workspace behavior, and decide whether display fences
support only bundled lexical backends or editor-specific installed languages.
These gates do not block the common semantic-token demonstrator.[^detailed-plan]

[^detailed-plan]: Detailed baseline, alternatives, region/projection architecture, demonstrator tasks, feature roadmap, risks, and evidence.
[^tooling-architecture]: Current shipped surface, client boundary, embedded-range gap, and `@docs` build regression.
[^rocdown-boundary]: Implemented Markdown-first and explicit executable-region contract.
[^source-map]: Current generated/source span and origin representation that requires richer bidirectional policies.
[^zed-roc]: Editor-specific bundle whose grammar/query assets are reusable but whose manifest and launcher are not.
[^tree-sitter-roc]: Exact grammar revision pinned by the inspected Zed extension.
[^zed-languages]: Current Zed grammar, injection, and semantic-token behavior.
[^vscode-embedded]: Embedded-service alternatives and portability tradeoffs.
[^vscode-semantic]: Standard semantic-token classification and enablement behavior.
[^tree-sitter-highlighting]: Query capture and in-process highlighting model.
