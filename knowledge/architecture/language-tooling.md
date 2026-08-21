---
type: Architecture
title: Rocci language-tooling boundary
description: Generic rocci-lsp analyzers are composed by rocci-rocdown-lsp into the shipped rocci-language-server; VS Code and Zed attach that binary to .rocci and .rocdown.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-08-21T10:00:00Z }
stale_after: 2027-02-13
authority: descriptive
owners: [human:nils]
sources:
  - id: lsp-server
    resource: ../../crates/rocci-lsp/src/lib.rs
    title: Generic Rocci language-server core
    author: process:git
    last_modified: 2026-08-17
  - id: lsp-analyzer
    resource: ../../crates/rocci-lsp/src/analyzer.rs
    title: DocumentAnalyzer extension point and RocciAnalyzer
    author: process:git
    last_modified: 2026-08-17
  - id: lsp-tokens
    resource: ../../crates/rocci-lsp/src/tokens.rs
    title: Rocci semantic-token and embedded-range implementation
    author: process:git
    last_modified: 2026-08-15
  - id: composition
    resource: ../../crates/rocci-rocdown-lsp/src/lib.rs
    title: Product composition of Rocci and Rocdown analyzers
    author: process:cursor
    last_modified: 2026-08-17
  - id: rocdown-lsp
    resource: ../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown-owned language analyzer
    author: process:git
    last_modified: 2026-08-17
  - id: lsp-tests
    resource: ../../crates/rocci-lsp/tests/server.rs
    title: Rocci language-server integration tests
    author: process:git
    last_modified: 2026-08-16
  - id: vscode-client
    resource: ../../editors/vscode/src/extension.ts
    title: VS Code Rocci language client
    author: process:git
    last_modified: 2026-08-17
  - id: vscode-manifest
    resource: ../../editors/vscode/package.json
    title: VS Code language and semantic-highlight registration
    author: process:git
    last_modified: 2026-08-17
  - id: zed-manifest
    resource: ../../editors/zed/extension.toml
    title: Zed language-server registration
    author: process:git
    last_modified: 2026-08-17
  - id: zed-readme
    resource: ../../editors/zed/README.md
    title: Zed Rocci extension setup and highlighting instructions
    author: process:git
    last_modified: 2026-08-17
  - id: zed-languages
    resource: https://zed.dev/docs/extensions/languages
    title: Zed language extension documentation
    author: organization:zed-industries
    last_modified: 2026-08-17
---

# Rocci language-tooling boundary

## Current contract

`rocci-lsp` is a generic library. It stores open documents, recompiles after
full-text open or change notifications, and dispatches to the first
`DocumentAnalyzer` that accepts the URI and language id. `LanguageServer::new()`
registers only `RocciAnalyzer`. Product composition lives outside this
crate.[^lsp-server][^lsp-analyzer]

`rocci-rocdown-lsp` owns the shipped `rocci-language-server` binary. It
constructs the generic core with `RocciAnalyzer` then `RocdownAnalyzer`, so
base Rocci stays free of Rocdown types while one binary serves both file
types.[^composition][^rocdown-lsp]

The host AST remains the authority for language boundaries.[^rocdown-lsp] Current
token collection highlights Rocci and Rocdown declarations, components,
HTML-shaped template names and attributes, handlers, and heading markers. It
records Roc and CSS body spans separately and composes embedded Tree-sitter
tokens into the standard semantic-token response.[^lsp-tokens][^lsp-tests]

The custom `rocci/inspectRegions` request exposes those regions for debugging.
The checked-in VS Code and Zed adapters launch the composed server and consume
standard semantic tokens; they do not call the custom method.[^lsp-tokens][^vscode-client][^zed-manifest]

## Editor boundary

VS Code registers `.rocci` and `.rocdown`, enables semantic highlighting for
both, and selects both languages on the composed `rocci-language-server`
without a TextMate grammar.[^vscode-manifest][^vscode-client]

Zed registers both file types and attaches the same server to `Rocci` and
`Rocdown`. Its checked-in language definitions do not name a Tree-sitter
grammar, while current Zed documentation describes `grammar` as required and
disables LSP semantic tokens by default. The adapter README works around the
latter with per-language `semantic_tokens = "full"`.[^zed-manifest][^zed-readme][^zed-languages]

## Current test and build state

As of 2026-08-17, `rocci-lsp` library tests and `rocci-rocdown-lsp` composition
tests cover analyzer dispatch. Default `cargo test -p rocci-lsp` completes in
under two seconds with unit, server, and invariant smoke checks, while deep
mutation fuzzing and release latency benchmarks remain `#[ignore]`. Editor-host
evidence is the VS Code integration suite plus the Zed manifest assertion in
`uv run rocci-ops verify-zed`.[^lsp-tests][^composition][^vscode-client]

## Planned evolution

The proposed [language-server plan](/plans/language-server.md) keeps one common
LSP server, makes embedded regions context- and purpose-aware, composes
embedded results into source coordinates, and treats editor-native grammars as
optional adapter support rather than semantic authority.

[^lsp-server]: Current synchronization, capabilities, document store, request dispatch, and diagnostic publication.
[^lsp-analyzer]: `DocumentAnalyzer` trait, `RocciAnalyzer` language matching, and `with_analyzers` composition.
[^lsp-tokens]: Current token legend, AST token collector, Roc/CSS span collection, and custom request.
[^composition]: Shipped binary construction that registers both analyzers.
[^rocdown-lsp]: Rocdown-owned analyzer implementation behind the generic core.
[^lsp-tests]: Tested host tokens, position encodings, local component features, and executable-versus-fenced Roc boundary.
[^vscode-client]: Thin client registration for `.rocci` and `.rocdown`.
[^vscode-manifest]: File types and semantic-highlighting defaults.
[^zed-manifest]: Server attachment to Rocci and Rocdown with no grammar declaration.
[^zed-readme]: Current per-language semantic-token settings workaround.
[^zed-languages]: Current grammar requirement and semantic-token mode documentation.
