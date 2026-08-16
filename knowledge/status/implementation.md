---
type: Status
title: Rocci implementation status
description: Rocci ships its core template, Rocdown, preview, desktop, packaging, editor, and static Rocs foundations while dynamic islands and broader packaging remain incomplete.
tags: [domain/rocci, domain/rocs, concern/tooling, concern/packaging]
status: draft
generated: { by: process:okf-phase-6, at: 2026-08-16T20:30:00Z }
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
    last_modified: 2026-08-16
  - id: rocs-plan
    resource: ../../ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md
    title: Rocs implementation plan
    author: human:nils
    last_modified: 2026-08-16
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
    resource: ../../crates/rocs/src/okf.rs
    title: OKF knowledge implementation
    author: process:git
    last_modified: 2026-08-16
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
---

# Rocci implementation status

## Snapshot date

2026-08-16.

## Shipped

Template and Rocdown compilation, standalone preview/run workflows, the desktop preview host, ad-hoc macOS application packaging, editor registration, and the Rust-catalog/Rocci-shell Rocs foundation are implemented.[^roadmap]

Rocs currently resolves nested routes, links, assets, navigation, drafts, hashed artifacts, CSP, a generated 404 page, and structured theme input.[^rocs-plan]

The isolated OKF path validates, graphs, renders, previews, inspects, and searches the knowledge bundle. Builds emit deterministic HTML plus catalog, search, agent, and validation indexes; inspection and search expose lifecycle, authority, trust-tier, and stale filters.[^okf]

Phase 6 adds a fixed seven-question lexical retrieval benchmark, JSON hit-rate and mean-reciprocal-rank reporting, and CI threshold enforcement. Seven dated research and audit reports are preserved under `archive/reports/`; two active detailed plans remain at the repository root.[^consolidation]

## Missing

Dynamic Roc/Rocci island splicing, broader production packaging, cross-platform installers, and native capability APIs remain incomplete.[^roadmap]

## Decided direction

Current implementation and accepted project direction keep render components as ordinary Roc functions, durable application state on the server, Rocdown Markdown-first with visible executable regions, and the Rocs catalog in Rust with its visible shell in Rocci. These choices are recorded separately so their lifecycle does not depend on this status snapshot.

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
[^rocs-plan]: Current Rocs phase state and build architecture.
[^okf-plan]: Approved OKF contract and amended knowledge-only DTCG boundary.
[^design-system]: Draft Phase 4 record of current design intent and shipped surfaces.
[^design-tokens]: Draft Phase 4 inventory and external standards research.
[^okf]: Current Phase 5 knowledge outputs, retrieval filters, and search implementation.
[^publication]: Draft record of the approved local-first publication disposition.
[^consolidation]: Draft Phase 6 lifecycle, report, documentation, and retrieval disposition.
