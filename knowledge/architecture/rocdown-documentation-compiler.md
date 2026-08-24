---
type: Architecture
title: Rocdown documentation generator
description: Rocdown resolves static documentation in Rust, renders article HTML from the Rocdown AST, applies one compiled Rocci shell, and commits planned artifacts atomically.
tags: [domain/rocdown, concern/rendering, concern/validation, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:40:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: refactor-plan
    resource: ../plans/rocdown/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:codex
    last_modified: 2026-08-17
  - id: catalog
    resource: ../../crates/rocci-rocdown/src/catalog.rs
    title: Rocdown catalog implementation
    author: process:git
    last_modified: 2026-08-17
  - id: site
    resource: ../../crates/rocci-rocdown/src/site.rs
    title: Rocdown discovery and inspection implementation
    author: process:git
    last_modified: 2026-08-17
  - id: article
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Rocdown static article renderer and feature gate
    author: process:git
    last_modified: 2026-08-17
  - id: docs
    resource: ../../crates/rocci-rocdown/src/docs.rs
    title: Rocdown typed documentation-component pipeline
    author: process:git
    last_modified: 2026-08-17
  - id: build-runtime
    resource: ../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Rocdown generated-page assembly runtime
    author: process:git
    last_modified: 2026-08-19
  - id: plan
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Rocdown build planner
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-reference
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Published Rocdown site reference
    author: process:git
    last_modified: 2026-08-19
  - id: okf
    resource: ../../crates/rocci-okf/README.md
    title: OKF knowledge application
    author: process:git
    last_modified: 2026-08-17
---

# Rocdown documentation generator

## Current contract

Rocdown discovers `.rocdown` pages, derives or reads stable identity and routes, resolves links, headings, assets, aliases, drafts, and explicit navigation, and reports catalog diagnostics before rendering.[^catalog][^site]

Static article bodies are rendered in Rust from Rocdown's semantic Markdown nodes. Line-start `:kind` article blocks become a typed article tree: includes and examples are catalog data, Markdown runs become fragment files, and documentation components are Rocci-rendered from scalar segment records. A structured page view plus composed article Html is passed to a Rocci-authored theme compiled once for the build; Roc is not asked to parse or type-check ordinary prose.[^refactor-plan]

Code blocks render as escaped source inside `rd-code-block` and `rd-code` elements with a `language-*` class when a fence language is present. Non-Rocdown `:include` content becomes the same Markdown code-block node, taking its language from an explicit field and then the file extension, while fences inside `:example` bodies use the ordinary Markdown path.[^article][^docs]

The build plan owns output paths, hashed assets and theme CSS, URL rewriting, redirects, `404.html`, CSP, canonical metadata, and the complete artifact set before the output tree is committed.[^plan][^rocdown-reference]

## HTML composition boundary

Rocdown compiles source with raw Markdown HTML disabled and accepts only Markdown, page metadata, and `:kind` article blocks in its static feature gate. A document-root HTML tag has already become a Rocci `Template` item by that point, so Rocdown rejects it with the other Roc/Rocci islands rather than treating it as static raw HTML. The same restriction applies inside article-block bodies and included Rocdown.[^site][^article][^docs]

Rust renders Markdown runs into fragment files with source-derived text and attribute values escaped, while `:kind` structure is carried as typed scalar segment records for Rocci rendering. The Roc build runtime reads those already-rendered fragment files and uses `Html.dangerously_include_unescaped_html` to re-enter the Roc `Html` type before composing them with documentation components and the theme. Widget painters resolve through a generated `BlockPainters` module (site pack, then `DocsComponents`, then a debug placeholder when allowed). Parent structure kinds still receive concatenated Html after child records are built. This is an internal trusted-artifact bridge, not an author-facing raw-HTML feature; its safety depends on preserving escaping in every Rust renderer before the bridge.[^article][^docs][^build-runtime][^plan][^rocdown-reference]

## Ownership boundaries

Rust owns deterministic documentation-data transformations: discovery, identity, graph resolution, navigation, validation, article rendering, artifact planning, and host orchestration (`rocci-roc-host`). Rocci owns visible site chrome (`RocdownTheme.rocci`) composed from shared base primitives in `rocci-ui` (`PageOutline.rocci`, `NavList.rocci`, `Breadcrumbs.rocci`). Authored Roc or Rocci islands will remain on the Roc compilation path when support lands.[^refactor-plan]

This architecture implements the [Rust-catalog/Rocci-shell decision](/decisions/rust-catalog-rocci-shell.md).

## Shipped state

Nested routes, aliases, drafts, link and asset validation, curated navigation, breadcrumbs, previous/next relations, hashed resources, CSP, responsive shell layouts, two-tier persistent renderer caching (`~/.rocci/cache`), native subprocess and Wasmtime execution hosts, inspection, watch/serve, live reload, and bounded `:kind` article blocks (asides, steps, figures, cards, no-JS tabs, includes, opt-in `rocdown test`, and site block-pack painter overlay) are implemented.[^refactor-plan][^rocdown-reference]

The separated OKF path lives in `okf` and `rocci-okf`, validating and reviewing knowledge collections independently from documentation site builds while sharing the base `toc.js` asset and `PageView` domain records.[^okf][^rocdown-reference]

## Not yet implemented

Dynamic island splicing is rejected. Static per-token syntax highlighting, public documentation-site search, clean per-page Markdown artifacts, and remaining machine-output polish remain on the backlog. `:api-operation`, snippet parameter substitution, tab-persistence JavaScript, generated API references, richer themes, locales, and advanced interaction remain planned.[^site][^article][^refactor-plan][^okf]

## Validation

Catalog checks do not require compiling Roc. Full builds additionally verify the Rocci shell/application path. The same resolved catalog must drive every derived artifact.[^refactor-plan]

[^refactor-plan]: Active ownership rule, implementation phases, testing, and remaining work.
[^catalog]: Current page identity, graph, routes, navigation, and diagnostic resolution.
[^site]: Current source discovery, static-feature gate, check, and inspection behavior.
[^article]: Static feature classification plus escaping of Markdown text and attributes.
[^docs]: Typed `:kind` article-block validation, rendering, and fragment planning.
[^build-runtime]: Roc-side composition of generated HTML fragments with typed documentation-component output.
[^plan]: Current deterministic artifact and structured page-view planning.
[^rocdown-reference]: User-facing configuration and generated-artifact contract.
[^okf]: Isolated knowledge validation, generated-output, inspection, search, filtering, and retrieval-benchmark implementation.
