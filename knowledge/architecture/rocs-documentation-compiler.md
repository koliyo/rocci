---
type: Architecture
title: Rocs documentation compiler
description: Rocs resolves static documentation in Rust, renders article HTML from the Rocdown AST, applies one compiled Rocci shell, and commits planned artifacts atomically.
tags: [domain/rocs, domain/rocdown, concern/rendering, concern/validation, concern/performance]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: rocs-plan
    resource: ../../ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md
    title: Rocs implementation plan
    author: human:nils
    last_modified: 2026-08-16
  - id: catalog
    resource: ../../crates/rocs/src/catalog.rs
    title: Rocs catalog implementation
    author: process:git
    last_modified: 2026-08-16
  - id: site
    resource: ../../crates/rocs/src/site.rs
    title: Rocs discovery and inspection implementation
    author: process:git
    last_modified: 2026-08-16
  - id: plan
    resource: ../../crates/rocs/src/plan.rs
    title: Rocs build planner
    author: process:git
    last_modified: 2026-08-16
  - id: rocs-reference
    resource: ../../docs/reference/rocs.rocdown
    title: Published Rocs reference
    author: human:nils
    last_modified: 2026-08-16
---

# Rocs documentation compiler

## Current contract

Rocs discovers `.rocdown` pages, derives or reads stable identity and routes, resolves links, headings, assets, aliases, drafts, and explicit navigation, and reports catalog diagnostics before rendering.[^catalog][^site]

Static article bodies are rendered in Rust from Rocdown's semantic Markdown nodes. A structured page view is passed to a Rocci-authored theme compiled once for the build; Roc is not asked to parse or type-check ordinary prose.[^rocs-plan]

The build plan owns output paths, hashed assets and theme CSS, URL rewriting, redirects, `404.html`, CSP, canonical metadata, and the complete artifact set before the output tree is committed.[^plan][^rocs-reference]

## Ownership boundaries

Rust owns deterministic documentation-data transformations: discovery, identity, graph resolution, navigation, validation, article rendering, artifact planning, and host orchestration. Rocci owns visible site chrome. Authored Roc or Rocci islands will remain on the Roc compilation path when support lands.[^rocs-plan]

This architecture implements the [Rust-catalog/Rocci-shell decision](/decisions/rust-catalog-rocci-shell.md).

## Shipped state

Nested routes, aliases, drafts, link and asset validation, curated navigation, breadcrumbs, previous/next relations, hashed resources, CSP, responsive shell layouts, inspection, watch/serve, and live reload are implemented.[^rocs-plan][^rocs-reference]

## Not yet implemented

Dynamic island splicing is rejected. Search, clean Markdown, and remaining machine-output polish are still Phase 4 work; later semantic documentation components, generated references, richer themes, locales, and advanced interaction remain planned.[^site][^rocs-plan]

## Validation

Catalog checks do not require compiling Roc. Full builds additionally verify the Rocci shell/application path. The same resolved catalog must drive every derived artifact.[^rocs-plan]

[^rocs-plan]: Active ownership rule, implementation phases, testing, and remaining work.
[^catalog]: Current page identity, graph, routes, navigation, and diagnostic resolution.
[^site]: Current source discovery, static-feature gate, check, and inspection behavior.
[^plan]: Current deterministic artifact and structured page-view planning.
[^rocs-reference]: User-facing configuration and generated-artifact contract.
