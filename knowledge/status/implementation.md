---
type: Status
title: Rocci implementation status
description: Rocci ships its core template, Rocdown, preview, desktop, packaging, editor, and static Rocs foundations while dynamic islands and broader packaging remain incomplete.
tags: [domain/rocci, domain/rocs, concern/tooling, concern/packaging]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T18:00:00Z }
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
---

# Rocci implementation status

## Snapshot date

2026-08-16.

## Shipped

Template and Rocdown compilation, standalone preview/run workflows, the desktop preview host, ad-hoc macOS application packaging, editor registration, and the Rust-catalog/Rocci-shell Rocs foundation are implemented.[^roadmap]

Rocs currently resolves nested routes, links, assets, navigation, drafts, hashed artifacts, CSP, a generated 404 page, and structured theme input.[^rocs-plan]

## Missing

Dynamic Roc/Rocci island splicing, broader production packaging, cross-platform installers, and native capability APIs remain incomplete.[^roadmap]

## Decided direction

Current implementation and accepted project direction keep render components as ordinary Roc functions, durable application state on the server, Rocdown Markdown-first with visible executable regions, and the Rocs catalog in Rust with its visible shell in Rocci. These choices are recorded separately so their lifecycle does not depend on this status snapshot.

The OKF compatibility boundary, bundle location, metadata vocabulary, ownership convention, local-first publication, and future DTCG authority are approved implementation contracts.[^okf-plan]

## Approved but not shipped

DTCG token sources, checked compatibility CSS, per-theme light/dark resolvers, and the root `DESIGN.md` are approved Phase 4 work but do not exist yet.[^okf-plan]

## Proposed, not approved

Typed client-behavior islands, their syntax, generated JavaScript artifact model, and any licensed Rocket provider remain exploratory. They are not part of the shipped language or the Phase 0 approved decision register.

## Validation

This record must be reviewed when its `stale_after` date is reached or when either cited implementation plan changes.

[^roadmap]: Current shipped focus and deliberate remaining limitations.
[^rocs-plan]: Current Rocs phase state and build architecture.
[^okf-plan]: Approved decision register and phased DTCG work.
