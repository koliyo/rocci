---
type: Decision
title: Use a Rust catalog and a Rocci documentation shell
description: Rocdown keeps deterministic static-site data work in Rust, visible site chrome in Rocci, and authored dynamic islands on the Roc path.
tags: [domain/rocdown, integration/roc, concern/rendering, concern/performance]
status: draft
generated: { by: process:okf-migration, at: 2026-08-17T23:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
authority: normative
owners: [human:nils]
sources:
  - id: refactor-plan
    resource: ../plans/rocdown-boundary-refactor.md
    title: Rocdown refactor plan
    author: process:codex
    last_modified: 2026-08-17
  - id: site
    resource: ../../crates/rocci-rocdown/src/site.rs
    title: Rocdown site loader and inspector
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-theme
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-17
---

# Use a Rust catalog and a Rocci documentation shell

## Context

The initial Roc-first static-site spike proved a shared Rocci shell but made Roc compilation scale with prose and reimplemented path, graph, hashing, and output work in a language without the host libraries already available to Rust.[^refactor-plan]

## Decision

Deterministic transformations of parsed documentation data belong in Rust: discovery, identity, routing, graph and navigation resolution, validation, article HTML, search projections, artifact planning, hashing, and output orchestration.[^refactor-plan][^site]

Visible site chrome belongs in a Rocci theme (`RocdownTheme.rocci`) evaluated through Roc. Shared chrome primitives (`PageOutline.rocci`, `NavList.rocci`, `Breadcrumbs.rocci`) live in base Rocci (`crates/rocci-ui/templates/chrome/`) while product layouts compose them. Authored dynamic regions are programs and stay on the Roc/Rocci compilation path, compiled only for pages that require them.[^refactor-plan][^rocdown-theme]

## Consequences

Content-only work can be checked without Roc, prose changes do not become generated Roc modules, and all output projections can share one resolved catalog. Rust must not grow an unrelated docs-template language, while the Rocci shell receives normalized view data rather than owning file discovery or routing.[^refactor-plan]

Full builds use a two-tier persistent renderer cache (`rocci-roc-host`) for native subprocess (Host A) and in-process Wasmtime (Host B).

## Current disposition

Implemented for static pages, including the resolved catalog, Rust article renderer, structured page view, shared base chrome components, two-tier renderer cache, and planned artifact set. Island splicing remains absent.

[^refactor-plan]: Accepted ownership rule, rejected Roc-first shape, and phased implementation record.
[^site]: Current Rust discovery, validation, catalog inspection, and static feature gate.
[^rocdown-theme]: Current Rocci-authored shell and article presentation.
