---
type: Implementation Plan
title: Embedded Roc LSP parity with zed-roc
description: "Give Rocci LSP compiler-backed Roc hover, diagnostics, completion, definition, and references inside .rocci and .rocdown by forwarding one optional roc experimental-lsp child against generated modules and mapping through source-map segments."
tags: [domain/rocci, domain/rocdown, integration/roc, concern/tooling, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: language-server
    resource: language-server.md
    title: Full Rocci and Rocdown language tooling plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: tooling-architecture
    resource: ../../architecture/language-tooling.md
    title: Current Rocci language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-25
  - id: source-map
    resource: ../../../crates/rocci-template/src/source_map.rs
    title: Generated-Roc source-map segment model
    author: process:git
    last_modified: 2026-08-22
  - id: wrap-module
    resource: ../../../crates/rocci-template/src/roc.rs
    title: wrap_type_module indent used by rocci view
    author: process:git
    last_modified: 2026-08-16
  - id: remap
    resource: ../../../crates/rocci-template/src/remap.rs
    title: Line-level roc check stderr remapping
    author: process:git
    last_modified: 2026-08-16
  - id: lsp-analysis
    resource: ../../../crates/rocci-lsp/src/analysis.rs
    title: Host-only Rocci hover, completion, and definition
    author: process:git
    last_modified: 2026-08-22
  - id: lsp-core
    resource: ../../../crates/rocci-lsp/src/lib.rs
    title: Generic language-server request dispatch
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-lsp
    resource: ../../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown analyzer hover including interpolation fallback
    author: process:git
    last_modified: 2026-08-23
  - id: view-projection
    resource: ../../../crates/rocci-cli/src/view.rs
    title: Temp workspace that writes wrap_type_module output
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-roc
    resource: https://github.com/koliyo/vscode-roc
    title: VS Code Roc extension launching roc experimental-lsp
    author: human:nils
    last_modified: 2026-08-25
  - id: zed-roc
    resource: https://github.com/h2000/zed-roc
    title: Zed Roc extension launching roc experimental-lsp
    author: human:alf-richter
    last_modified: 2026-08-25
---

# Embedded Roc LSP parity with zed-roc

## Goal

Rocci LSP matches [zed-roc](https://github.com/h2000/zed-roc) and
[vscode-roc](https://github.com/koliyo/vscode-roc) for **executable Roc**
inside `.rocci` and `.rocdown`: type hover, diagnostics, completion,
go-to-definition, and references, via one optional `roc experimental-lsp`
child on generated modules, with results mapped to authored spans.[^language-server][^vscode-roc][^zed-roc]

This is the executable sequence for language-server Phase 4 (Roc
semantics). It does not replace that umbrella plan.[^language-server]

## Out of bound

- vscode-roc or zed-roc merged into Rocci editor extensions; those stay `.roc`
  editors
- VS Code embedded-language injection of vscode-roc onto raw snippets
- Display-only fences forwarded to Roc
- Roc format-on-save or rename of `.rocci` / `.rocdown`
- Roc-derived semantic tokens (Tree-sitter highlighting stays)
- Workspace-wide Rocci component index (language-server Phase 3)

## Constraints that do not move

- `rocci-lsp` does not import Rocdown AST types.[^tooling-architecture]
- Host hover, tokens, and diagnostics stay available when `roc` is missing or
  the child crashes.[^language-server]
- Default `cargo test -p rocci-lsp` does not require Roc. Live child tests use
  `ROCCI_REQUIRE_ROC=1`.
- Map only reversible authored Roc. Scaffolding, static markup, CSS, and
  ambiguous hits are refused.[^source-map]
- One child process per workspace, not per region.
- Host wins on host constructs (component tag, handler, `@page`, `:kind`).
  Executable Roc regions prefer Roc results.

## Phase 1 — Bidirectional map and projection text

**Bound:** Byte-offset `source ↔ generated` lookup on `CompileOutput.segments`.
Helper that emits `(projection_roc, projection_segments)` after
`wrap_type_module` indent. No process spawn.[^source-map][^wrap-module]

**Out of bound:** LSP child, hover wiring.

**Tests:** interpolations, `@roc` blocks, handler bodies, trimmed exprs,
scaffolding refusal, wrap indent. `cargo test -p rocci-template`;
`cargo fmt --all -- --check`.

**Exit:** Those commands pass. A source offset inside `{title}` maps to the
generated `title` span, not to `Html.text`.

## Phase 2 — Optional child client

**Bound:** Spawn `roc experimental-lsp --stdio`, initialize, sync one
projection file, request/response with timeout. Null backend when `roc` is
absent. Record whether a type module is enough or a stub `main.roc` is
required. Fake backend for default tests.[^view-projection][^lsp-core]

**Out of bound:** Hover composition in analyzers.

**Tests:** `cargo test -p rocci-lsp`. Live child:
`ROCCI_REQUIRE_ROC=1 cargo test -p rocci-lsp --test roc_backend -- --ignored`.

**Exit:** Default suite passes without Roc. Gated test hovers a generated
fixture through the child.

**Projection contract:** A `TypeName := [].{ ... }` file from
`project_type_module` is enough for `roc experimental-lsp` hover. A stub
`main.roc` is not required. The child advertises hover, definition, and
completion; it does not advertise `references` or `signatureHelp`.

## Phase 3 — Hover in `.rocci` and `.rocdown`

**Bound:** Executable Roc region (and Rocdown interp hole) forwards hover and
maps the range back. Existing host hovers unchanged. Rocdown “Markdown
interpolation” hover yields to a Roc type when the child answers.[^lsp-analysis][^rocdown-lsp]

**Tests:** `cargo test -p rocci-lsp`; `cargo test -p rocci-rocdown`;
Roc-gated hover on `{title}` / `@roc` ident.

**Exit:** Those commands pass. Host component hover still documents
`@component`.

## Phase 4 — Mapped diagnostics

**Bound:** Merge child `publishDiagnostics` into the host publish, remap to
authored spans, `source: "roc"`. Drop diagnostics on scaffolding or failed
maps. Replace line-only `remap_roc_output` for this path; keep stderr remap
for CLI.[^remap]

**Tests:** same crates; fixture where a type error in `{ expr }` lands on the
expr span.

**Exit:** Those tests pass.

## Phase 5 — Completion and definition

**Bound:** In Roc regions, forward completion and `textDocument/definition`.
Projection-file locations map back to `.rocci` / `.rocdown`; sibling `.roc`
URIs stay. Strip `additionalTextEdits` that touch unmapped scaffolding.

**Tests:** fake backend plus one Roc-gated case.

**Exit:** `cargo test -p rocci-lsp`; `cargo test -p rocci-rocdown`.

## Phase 6 — References and signature help

**Bound:** `referencesProvider`. `signatureHelpProvider` only if the child
answers. Same mapping and refusal rules.

**Out of bound:** rename, formatting.

**Tests:** `cargo test -p rocci-lsp`; `cargo test -p rocci-rocdown`;
`cargo test -p rocci-rocdown-lsp`; `cargo fmt --all -- --check`.

**Exit:** Those commands pass.

## Phase 7 — Settings, docs, degraded mode

**Bound:** `rocci.roc.path` in the VS Code contribution; PATH/`roc` documented
in `crates/rocci-lsp/README.md` and editor READMEs. Zed uses the same server
via initializationOptions or env. Restart child with **Rocci: Restart LSP
server**.

**Exit:** READMEs updated. `uv run rocci-ops check zed` if the Zed manifest
changes. `cargo run -q -p rocci-okf -- check knowledge --profile base --format terminal`.

[^language-server]: Phase 4 child server, generated projection, mapped semantics, degraded mode.
[^tooling-architecture]: Composed `rocci-language-server`; base Rocci free of Rocdown types.
[^source-map]: `Segment` generated/source/origin; no offset lookup yet.
[^wrap-module]: `rocci view` wraps generated Roc in `TypeName := [].{ … }` with four-space indent.
[^remap]: Current remap is `roc check` line numbers, not hover offsets.
[^lsp-analysis]: Hover is handlers and component names only.
[^lsp-core]: Sync `handle_request`; child client needs interior mutability.
[^rocdown-lsp]: Interpolation hover reprints the expr; no Roc types.
[^view-projection]: Isolated temp dir, sibling `.roc` copies, `wrap_type_module`.
[^vscode-roc]: `roc experimental-lsp --stdio` on `language: roc` file-scheme documents.
[^zed-roc]: Same CLI; type tooltips, errors, completion.
