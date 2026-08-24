---
type: Decision
title: Nest large OKF collections under a closed product-area vocabulary
description: Keep type-first top-level folders; nest plans, research, and audits into rocci, rocdown, okf, site, ops, and shared. Do not nest by lifecycle. Concept ID remains the path.
tags: [domain/okf, domain/rocci-okf, concern/architecture, concern/authoring, concern/navigation]
status: draft
generated: { by: process:cursor, at: 2026-08-24T11:40:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: okf-spec
    resource: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
    title: Open Knowledge Format v0.2 specification
    author: organization:google-cloud
  - id: static-okf
    resource: static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: plan
    resource: ../plans/okf/nested-collections.md
    title: Nested OKF collections implementation plan
    author: process:cursor
    last_modified: 2026-08-24
---

# Nest large OKF collections under a closed product-area vocabulary

## Context

OKF v0.2 treats a bundle as a recursive directory tree: concept ID is the path without `.md`, and any directory may have `index.md` for progressive disclosure.[^okf-spec] Rocci kept one-level folders (`plans/*.md`) as an authoring habit, not a format limit. The static Markdown boundary does not constrain depth.[^static-okf]

A single `plans/rocci/` dump would recreate the flat index. Lifecycle folders would churn IDs on every status change.

## Decision

- Keep type-first top-level collections (`plans/`, `research/`, `audits/`, and the small flat sets).
- Nest only plans, research, and audits under the closed areas `rocci/`, `rocdown/`, `okf/`, `site/`, `ops/`, and `shared/`.
- Create an area directory only when it has at least one record. Mirror area names across those three type collections.
- Prefer bundle-root links (`/plans/okf/nested-collections.md`). Filename stems stay unique under `knowledge/plans/`.
- Deepen in place later (`plans/rocci/preview/`) if an area index exceeds about twenty records. Do not flatten or rename areas.
- Keep a single bundle-root `log.md`. Do not add redirect stub concepts.

This decision is not approved.[^plan]

## Consequences

Inspect IDs, review routes, Cmd-K paths, and the retrieval benchmark follow the new paths. Moves are identity breaks. Collection indexes and nearest-index membership become the discoverability contract.[^plan]

## Current disposition

Exploratory draft. Implementation: [nested collections](/plans/okf/nested-collections.md).

[^okf-spec]: Bundle tree, concept ID, and per-directory `index.md` (SPEC §§2–3, §8).
[^static-okf]: Inert Markdown plus YAML; no one-level folder requirement.
[^plan]: Phased engine, viewer, authoring, and migration work.
