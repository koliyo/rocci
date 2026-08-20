---
type: Status
title: Rocci implementation status
description: Current shipped status across Rocci templates, Rocdown static sites, the portable OKF engine, and the OKF knowledge path.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-20T07:50:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2026-09-15
authority: descriptive
owners: [human:nils]
sources:
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-17
  - id: refactor-plan
    resource: ../plans/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:codex
    last_modified: 2026-08-17
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
  - id: design-system
    resource: ../design/design-system.md
    title: Rocci design-system knowledge
    author: process:okf-phase-4
    last_modified: 2026-08-16
  - id: design-tokens
    resource: ../design/design-tokens.md
    title: Rocci design-token research
    author: process:okf-phase-4
    last_modified: 2026-08-16
  - id: okf
    resource: ../../crates/okf/README.md
    title: OKF portable engine
    author: process:git
    last_modified: 2026-08-17
  - id: publication
    resource: ../decisions/local-knowledge-publication.md
    title: Local knowledge publication decision
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: consolidation
    resource: ../reference/consolidation.md
    title: OKF consolidation disposition
    author: process:okf-phase-6
    last_modified: 2026-08-16
  - id: lsp-plan
    resource: ../plans/language-server.md
    title: Proposed full Rocci and Rocdown language-server plan
    author: process:codex
    last_modified: 2026-08-17
  - id: rocdown-compiler
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Current Rocdown documentation generator boundary
    author: process:codex
    last_modified: 2026-08-17
  - id: site-ref
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Public Rocdown site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: browser-readme
    resource: ../../crates/rocci-browser/README.md
    title: rocci-browser crate contract
    author: process:cursor
    last_modified: 2026-08-20
  - id: browser-guide
    resource: ../../docs/guides/rocci-browser.rocdown
    title: Public project-browser guide
    author: process:cursor
    last_modified: 2026-08-20
  - id: macos-plan
    resource: ../plans/rocci-browser-macos-app.md
    title: rocci-browser macOS app and TUI removal plan
    author: process:cursor
    last_modified: 2026-08-20
---

# Rocci implementation status

## Snapshot date

2026-08-20.

## Shipped

The shipped implementation across Rocci, Rocdown, and the OKF knowledge bundle includes template and document compilation, standalone preview/run workflows, the `rocci-desktop` preview host, ad-hoc macOS application packaging, editor registration with composed language servers (`rocci-rocdown-lsp`), domain-neutral view records (`rocci-ui`), the portable `okf` engine, and the Rust-catalog/Rocci-shell Rocdown documentation generator.[^roadmap]

Rocdown currently resolves nested routes, links, assets, navigation, drafts, hashed artifacts, CSP, a generated 404 page, and structured theme input. Static pages may include bounded `:kind` article blocks: Rocdown types asides, steps, figures, cards, no-JS tabs, file includes, and example records. Builtin painters live in `DocsComponents.rocci`; a site `theme/Blocks.rocci` (or `[blocks] pack`) can replace those painters or add site-local kinds. `rocdown test` runs declared example commands on demand and is not part of `rocdown build`.[^refactor-plan][^site-ref]

Tree-sitter highlighting library `rocci-highlight` provides token spans for LSP and documentation rendering parity.[^rocdown-compiler]

The shipped OKF knowledge path validates, graphs, renders, previews, inspects, and searches the knowledge bundle using the portable `okf` engine and `rocci-okf` review application. Builds emit deterministic HTML plus catalog, search, agent, and validation indexes; inspection and search expose lifecycle, authority, trust-tier, and stale filters.[^okf]

`rocci-browser` is a fourth binary: a product-blind registry, two-stage Cmd-P picker, and persistent preview window that `load_url`s adapter origins. The terminal `tui` command is removed. macOS can assemble an ad-hoc **Rocci Browser.app**; production signing is not shipped.[^browser-readme][^browser-guide][^macos-plan]

Retrieval benchmarks measure a fixed seven-question lexical retrieval benchmark with JSON hit-rate and mean-reciprocal-rank reporting, with CI threshold enforcement.[^consolidation]

## Missing

Dynamic Roc/Rocci island splicing, broader production packaging, cross-platform installers, and native capability APIs remain incomplete. `:api-operation`, snippet parameter substitution, tab-persistence JavaScript, and generated collection pages are explicitly not shipped.[^roadmap]

The editor adapters and host-language LSP with Rocdown composition exist, but workspace-wide language intelligence and compiler-backed Roc semantics remain proposed work.[^lsp-plan]

## Decided direction

Current implementation and accepted project direction keep render components as ordinary Roc functions, durable application state on the server, Rocdown Markdown-first with visible executable regions, and the Rocdown catalog in Rust with its visible shell in Rocci. These choices are recorded separately so their lifecycle does not depend on this status snapshot.

The OKF compatibility boundary, bundle location, metadata vocabulary, ownership convention, and local-first publication are approved implementation contracts. DTCG is approved only as research evidence for design knowledge, not as implementation authority.[^okf-plan]

## Design-system knowledge phase

The root `DESIGN.md` and two design knowledge records now document the current CSS theme surfaces and DTCG-informed research.[^design-system][^design-tokens] Rocci still has no DTCG token sources, checked compatibility CSS, per-theme token resolvers, generator, or token validation, and Phase 4 does not approve those artifacts.[^okf-plan]

## Publication

Knowledge output remains local and repository-visible. CI validates and compares temporary builds, but no public deployment or verbatim bundle archive is configured pending an explicit source-and-license review.[^publication][^okf-plan]

## Proposed, not approved

Typed client-behavior islands, their syntax, generated JavaScript artifact model, and any licensed Rocket provider remain exploratory. They are not part of the shipped language or the Phase 0 approved decision register.

## Validation

This record must be reviewed when its `stale_after` date is reached or when either cited implementation plan changes.

[^roadmap]: Current shipped focus and deliberate remaining limitations.
[^refactor-plan]: Active ownership rule, implementation phases, testing, and remaining work.
[^okf-plan]: Approved OKF contract and amended knowledge-only DTCG boundary.
[^design-system]: Draft Phase 4 record of current design intent and shipped surfaces.
[^design-tokens]: Draft Phase 4 inventory and external standards research.
[^okf]: Portable OKF engine and rocci-okf review application.
[^publication]: Draft record of the approved local-first publication disposition.
[^consolidation]: Draft Phase 6 lifecycle, report, documentation, and retrieval disposition.
[^lsp-plan]: Proposed embedded-language demonstrator and full language-server phases, explicitly separated from the current tooling contract.
[^rocdown-compiler]: Current static code-block rendering path and token spans.
[^site-ref]: Theme block-pack overlay and `[blocks]` configuration.
[^browser-readme]: Shipped host commands and cargo-run graphical window; TUI removed; ad-hoc .app documented.
[^browser-guide]: Public contract: Cmd-P versus Cmd-K; ad-hoc Rocci Browser.app; production signing planned.
[^macos-plan]: TUI deletion and ad-hoc Finder .app; notarization stays later.
