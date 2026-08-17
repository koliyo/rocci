---
type: Architecture
title: Rocci language-tooling boundary
description: Rocci and Rocdown share one parser-backed language server and thin VS Code and Zed adapters, but embedded-language spans are not yet consumed by either editor.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/syntax]
status: draft
generated: { by: process:codex, at: 2026-08-17T05:44:10Z }
stale_after: 2027-02-13
authority: descriptive
owners: [human:nils]
sources:
  - id: lsp-server
    resource: ../../crates/rocci-lsp/src/lib.rs
    title: Rocci language-server protocol implementation
    author: process:git
    last_modified: 2026-08-15
  - id: lsp-tokens
    resource: ../../crates/rocci-lsp/src/tokens.rs
    title: Rocci semantic-token and embedded-range implementation
    author: process:git
    last_modified: 2026-08-15
  - id: lsp-rocdown
    resource: ../../crates/rocci-lsp/src/rocdown.rs
    title: Rocdown language-server analysis
    author: process:git
    last_modified: 2026-08-15
  - id: lsp-tests
    resource: ../../crates/rocci-lsp/tests/server.rs
    title: Rocci language-server integration tests
    author: process:git
    last_modified: 2026-08-16
  - id: rocdown-ast
    resource: ../../crates/rocci-rocdown/src/ast.rs
    title: Current Rocdown AST
    author: process:git
    last_modified: 2026-08-16
  - id: vscode-client
    resource: ../../editors/vscode/src/extension.ts
    title: VS Code Rocci language client
    author: process:git
    last_modified: 2026-08-15
  - id: vscode-manifest
    resource: ../../editors/vscode/package.json
    title: VS Code language and semantic-highlight registration
    author: process:git
    last_modified: 2026-08-15
  - id: zed-manifest
    resource: ../../editors/zed/extension.toml
    title: Zed Rocci extension manifest
    author: process:git
    last_modified: 2026-08-15
  - id: zed-readme
    resource: ../../editors/zed/README.md
    title: Zed Rocci extension setup and highlighting instructions
    author: process:git
    last_modified: 2026-08-15
  - id: zed-languages
    resource: https://zed.dev/docs/extensions/languages
    title: Zed language extension documentation
    author: organization:zed-industries
    last_modified: 2026-08-17
---

# Rocci language-tooling boundary

## Current contract

`rocci-language-server` is the common server for `.rocci` and `.rocdown`. It
stores open documents, recompiles the owning parser/lowerer after full-text
open or change notifications, publishes parser diagnostics, and serves
document symbols, hover, same-file component definition, completion, and full
or range semantic tokens.[^lsp-server][^lsp-tests]

The host AST remains the authority for language boundaries. Current token
collection highlights Rocci/Rocdown declarations, components, HTML-shaped
template names and attributes, handlers, and heading markers. It records Roc
and CSS body spans separately and intentionally leaves their contents out of
the standard semantic-token response.[^lsp-tokens][^lsp-tests]

The custom `rocci/embeddedRanges` request exposes those Roc and CSS spans, but
the checked-in VS Code and Zed adapters only launch the server; neither calls
the custom method. Embedded Roc and CSS highlighting is therefore not shipped
through the current clients.[^lsp-tokens][^vscode-client][^zed-manifest]

## Editor boundary

VS Code registers both file types, enables semantic highlighting for them, and
uses the common server without a TextMate grammar.[^vscode-manifest]

Zed registers both file types and the common server. Its checked-in language
definitions do not name a Tree-sitter grammar, while current Zed documentation
describes `grammar` as required and disables LSP semantic tokens by default.
The Zed adapter README works around the latter with per-language
`semantic_tokens = "full"`; current-version compatibility still requires an
editor smoke test.[^zed-manifest][^zed-readme][^zed-languages]

## Current test and build state

As of 2026-08-17, `rocci-lsp` compiles cleanly and all test suites pass. Tests
are tiered so that default `cargo test -p rocci-lsp` completes in under two
seconds (<2s) with unit, server, and invariant smoke checks, while deep
5,000-iteration mutation fuzzing and release latency benchmarks are gated
behind `#[ignore]` and run on demand.[^lsp-tests]

## Planned evolution

The proposed [language-server plan](/plans/language-server.md) keeps one common
LSP server, makes embedded regions context- and purpose-aware, composes
embedded results into source coordinates, and treats editor-native grammars as
optional adapter support rather than semantic authority.

[^lsp-server]: Current synchronization, capabilities, document store, request dispatch, and diagnostic publication.
[^lsp-tokens]: Current token legend, AST token collector, Roc/CSS span collection, and custom request.
[^lsp-rocdown]: Current Rocdown symbol match without an `Item::Docs` arm.
[^lsp-tests]: Tested host tokens, position encodings, local component features, and executable-versus-fenced Roc boundary.
[^rocdown-ast]: Current `Item::Docs` variant not covered by the cited LSP matches.
[^vscode-client]: Thin client registration with no embedded-range forwarding.
[^vscode-manifest]: File types and semantic-highlighting defaults.
[^zed-manifest]: Thin server registration with no grammar declaration or embedded-range forwarding.
[^zed-readme]: Current per-language semantic-token settings workaround.
[^zed-languages]: Current grammar requirement and semantic-token mode documentation.
