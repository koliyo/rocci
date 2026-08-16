---
type: Decision
title: Use a Rust catalog and a Rocci documentation shell
description: Rocs keeps deterministic static-site data work in Rust, visible site chrome in Rocci, and authored dynamic islands on the Roc path.
tags: [domain/rocs, domain/rocdown, integration/roc, concern/rendering, concern/performance]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
authority: normative
owners: [human:nils]
sources:
  - id: rocs-plan
    resource: ../../ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md
    title: Rocs implementation plan
    author: human:nils
    last_modified: 2026-08-16
  - id: site
    resource: ../../crates/rocs/src/site.rs
    title: Rocs site loader and inspector
    author: process:git
    last_modified: 2026-08-16
  - id: rocs-theme
    resource: ../../crates/rocs/templates/RocsTheme.rocci
    title: Rocs documentation shell
    author: process:git
    last_modified: 2026-08-16
---

# Use a Rust catalog and a Rocci documentation shell

## Context

The initial Roc-first static-site spike proved a shared Rocci shell but made Roc compilation scale with prose and reimplemented path, graph, hashing, and output work in a language without the host libraries already available to Rust.[^rocs-plan]

## Decision

Deterministic transformations of parsed documentation data belong in Rust: discovery, identity, routing, graph and navigation resolution, validation, article HTML, search projections, artifact planning, hashing, and output orchestration.[^rocs-plan][^site]

Visible site chrome belongs in a Rocci theme evaluated through Roc. Authored dynamic regions are programs and stay on the Roc/Rocci compilation path, compiled only for pages that require them.[^rocs-plan][^rocs-theme]

## Consequences

Content-only work can be checked without Roc, prose changes do not become generated Roc modules, and all output projections can share one resolved catalog. Rust must not grow an unrelated docs-template language, while the Rocci shell receives normalized view data rather than owning file discovery or routing.[^rocs-plan]

Full builds still depend on the theme compilation/application path, and dynamic-page splicing needs a later explicit integration.

## Current disposition

Implemented for static pages, including the resolved catalog, Rust article renderer, structured page view, once-compiled shell, and planned artifact set. Island splicing remains absent.

[^rocs-plan]: Accepted ownership rule, rejected Roc-first shape, and phased implementation record.
[^site]: Current Rust discovery, validation, catalog inspection, and static feature gate.
[^rocs-theme]: Current Rocci-authored shell and article presentation.
