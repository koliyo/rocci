---
type: Architecture
title: Rocs documentation compiler
description: Rocs resolves static documentation in Rust, renders article HTML from the Rocdown AST, applies one compiled Rocci shell, and commits planned artifacts atomically.
tags: [domain/rocs, domain/rocdown, concern/rendering, concern/validation, concern/performance]
status: draft
generated: { by: process:okf-phase-6, at: 2026-08-16T20:30:00Z }
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
  - id: article
    resource: ../../crates/rocs/src/article.rs
    title: Rocs static article renderer and feature gate
    author: process:git
    last_modified: 2026-08-16
  - id: docs
    resource: ../../crates/rocs/src/docs.rs
    title: Rocs typed documentation-component pipeline
    author: process:git
    last_modified: 2026-08-16
  - id: build-runtime
    resource: ../../crates/rocs/runtime/RocsBuild.roc
    title: Rocs generated-page assembly runtime
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
  - id: okf
    resource: ../../crates/rocs/src/okf.rs
    title: OKF knowledge implementation
    author: process:git
    last_modified: 2026-08-16
---

# Rocs documentation compiler

## Current contract

Rocs discovers `.rocdown` pages, derives or reads stable identity and routes, resolves links, headings, assets, aliases, drafts, and explicit navigation, and reports catalog diagnostics before rendering.[^catalog][^site]

Static article bodies are rendered in Rust from Rocdown's semantic Markdown nodes. `@docs` declarations become a typed article tree: includes and examples are catalog data, Markdown runs become fragment files, and documentation components are Rocci-rendered from scalar segment records. A structured page view plus composed article Html is passed to a Rocci-authored theme compiled once for the build; Roc is not asked to parse or type-check ordinary prose.[^rocs-plan]

The build plan owns output paths, hashed assets and theme CSS, URL rewriting, redirects, `404.html`, CSP, canonical metadata, and the complete artifact set before the output tree is committed.[^plan][^rocs-reference]

## HTML composition boundary

Rocs compiles source with raw Markdown HTML disabled and accepts only Markdown, page metadata, and `@docs` items in its static feature gate. A document-root HTML tag has already become a Rocci `Template` item by that point, so Rocs rejects it with the other Roc/Rocci islands rather than treating it as static raw HTML. The same restriction applies inside `@docs` bodies and included Rocdown.[^site][^article][^docs]

Rust renders Markdown runs into fragment files with source-derived text and attribute values escaped, while `@docs` structure is carried as typed scalar segment records for Rocci rendering. The Roc build runtime reads those already-rendered fragment files and uses `Html.dangerously_include_unescaped_html` to re-enter the Roc `Html` type before composing them with documentation components and the theme. This is an internal trusted-artifact bridge, not an author-facing raw-HTML feature; its safety depends on preserving escaping in every Rust renderer before the bridge.[^article][^docs][^build-runtime]

## Ownership boundaries

Rust owns deterministic documentation-data transformations: discovery, identity, graph resolution, navigation, validation, article rendering, artifact planning, and host orchestration. Rocci owns visible site chrome. Authored Roc or Rocci islands will remain on the Roc compilation path when support lands.[^rocs-plan]

This architecture implements the [Rust-catalog/Rocci-shell decision](/decisions/rust-catalog-rocci-shell.md).

## Shipped state

Nested routes, aliases, drafts, link and asset validation, curated navigation, breadcrumbs, previous/next relations, hashed resources, CSP, responsive shell layouts, inspection, watch/serve, live reload, and bounded `@docs` components (asides, steps, figures, cards, no-JS tabs, includes, and opt-in `rocs test`) are implemented.[^rocs-plan][^rocs-reference]

The isolated OKF path additionally validates and renders knowledge collections, previews them through the Rocs shell, emits catalog, search, agent, and validation indexes, provides filtered catalog inspection and heading-chunk search, and measures fixed lexical retrieval questions with hit-rate and mean-reciprocal-rank output.[^okf][^rocs-reference]

## Not yet implemented

Dynamic island splicing is rejected. Public documentation-site search, clean per-page Markdown artifacts, and remaining machine-output polish are still ordinary Rocs backlog; the shipped OKF search is a separate local knowledge-retrieval path. `@docs api-operation`, snippet parameter substitution, tab-persistence JavaScript, generated API references, richer themes, locales, and advanced interaction remain planned.[^site][^rocs-plan][^okf]

## Validation

Catalog checks do not require compiling Roc. Full builds additionally verify the Rocci shell/application path. The same resolved catalog must drive every derived artifact.[^rocs-plan]

[^rocs-plan]: Active ownership rule, implementation phases, testing, and remaining work.
[^catalog]: Current page identity, graph, routes, navigation, and diagnostic resolution.
[^site]: Current source discovery, static-feature gate, check, and inspection behavior.
[^article]: Static feature classification plus escaping of Markdown text and attributes.
[^docs]: Typed `@docs` validation, rendering, and fragment planning.
[^build-runtime]: Roc-side composition of generated HTML fragments with typed documentation-component output.
[^plan]: Current deterministic artifact and structured page-view planning.
[^rocs-reference]: User-facing configuration and generated-artifact contract.
[^okf]: Current isolated knowledge validation, generated-output, inspection, search, filtering, and retrieval-benchmark implementation.
