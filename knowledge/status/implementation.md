---
type: Status
title: Rocci implementation status
description: Rocci ships its core template, Rocdown, preview, desktop, packaging, editor, and static Rocs foundations while dynamic islands and broader packaging remain incomplete.
tags: [domain/rocci, domain/rocs, concern/tooling, concern/packaging]
status: draft
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
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
---

# Rocci implementation status

## Snapshot date

2026-08-16.

## Shipped

Template and Rocdown compilation, standalone preview/run workflows, the desktop preview host, ad-hoc macOS application packaging, editor registration, and the Rust-catalog/Rocci-shell Rocs foundation are implemented.[^roadmap]

Rocs currently resolves nested routes, links, assets, navigation, drafts, hashed artifacts, CSP, a generated 404 page, and structured theme input.[^rocs-plan]

## Missing

Dynamic Roc/Rocci island splicing, broader production packaging, cross-platform installers, and native capability APIs remain incomplete.[^roadmap]

## Validation

This record must be reviewed when its `stale_after` date is reached or when either cited implementation plan changes.

[^roadmap]: Current shipped focus and deliberate remaining limitations.
[^rocs-plan]: Current Rocs phase state and build architecture.
