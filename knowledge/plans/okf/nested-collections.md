---
type: Implementation Plan
title: Nested OKF collections
description: Restructure knowledge/ with idiomatic nested collections, first-class engine and viewer support, and a closed product-area vocabulary under plans, research, and audits.
tags: [domain/okf, domain/rocci-okf, concern/architecture, concern/authoring, concern/navigation, concern/validation]
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
  - id: discover
    resource: ../../../crates/okf/src/lib.rs
    title: Recursive Markdown discovery and path-derived concept IDs
    author: process:git
    last_modified: 2026-08-24
  - id: goto-js
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette
    author: process:git
    last_modified: 2026-08-24
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: OKF review HTML, catalog.json, and pages.json
    author: process:git
    last_modified: 2026-08-24
  - id: decision
    resource: ../../decisions/nested-okf-collections.md
    title: Nested collection layout draft
    author: process:cursor
    last_modified: 2026-08-24
  - id: write-knowledge
    resource: ../../../.cursor/rules/write-knowledge.mdc
    title: Knowledge authoring rule
    author: process:git
    last_modified: 2026-08-24
---

# Nested OKF collections

## Goal

Make nested OKF collections a supported Rocci contract: type-first top-level folders, closed product-area subdirectories with `index.md`, stable filename stems, engine diagnostics, Cmd-K and breadcrumbs that use the tree, and a one-time identity-breaking migration of plans, research, and audits.[^okf-spec][^decision][^goto-js][^presentation]

## Out of bound

Per-directory `log.md`; lifecycle or status folders; redirect stub concepts; synthesizing a file-tree sidebar as a replacement for indexes; putting `search.json` into review Cmd-K; renaming tags; nesting architecture, decisions, status, reference, design, or case-studies.

## Constraints that do not move

- Concept ID is the bundle path without `.md`.[^discover]
- Non-root `index.md` has no frontmatter.
- Tags remain cross-cutting; directories are progressive disclosure.[^okf-spec]
- Prefer bundle-root Markdown links (`/plans/okf/nested-collections.md`).
- Plan git branches use the filename stem, which stays unique under `knowledge/plans/`.[^write-knowledge]

## Area vocabulary

| Area | Owns |
| --- | --- |
| `rocci/` | Templates, handlers, runtime, desktop inspector, component generation, falling-block |
| `rocdown/` | Format, blocks, interpolation, islands, app-docs compiler |
| `okf/` | Portable engine, review app, knowledge load and render, this layout |
| `site/` | rocci.dev IA, publish, playground, public launch, branding |
| `ops/` | CI, hosting, python-uv, tangled |
| `shared/` | True multi-product chrome (`cli-entry-points`, `fuzzy-navigation`, `mobile-chrome`) |

Assignment follows the primary owner of the work, not the union of tags. Mirror area names across plans, research, and audits. Deepen in place later; never flatten.

## Phases

### Phase 1 — Contract and engine

Bound: draft this plan and the layout decision; nested load, publish, and preview tests; OKF3005 route collision; collection titles from H1; sanitized nested collection article names; OKF2010 nearest-index membership warning; unique-stem `inspect concept`.

Exit: `cargo test -p okf` and `cargo test -p rocci-okf`.

### Phase 2 — Viewer discoverability

Bound: breadcrumbs from path prefixes; Cmd-K path tiebreaker. No tree sidebar as the primary contract (existing collection tree may group by longest collection prefix).

Exit: nested collection presentation test plus goto.js path sort.

### Phase 3 — Authoring rules

Bound: write-knowledge, manage-rocci-knowledge, phase-runner unique-stem branch rule and area table.

Exit: rules describe type then area; branch name is `file_stem`.

### Phase 4 — Mechanical migration

Bound: `git mv` plans, research, and audits; per-area indexes; rewrite links and `sources[].resource` depth; retrieval benchmark and repo citations; `knowledge/log.md`.

Exit: `cargo test -p okf`, `cargo test -p rocci-okf`, `cargo run -q -p rocci-okf -- check knowledge --profile base`, `cargo fmt --all -- --check`.

## Status

Phases 1–4 land in this revision: nested IDs are a contract, type indexes link areas, and old one-level plan/research/audit IDs are gone. Do not log complete until CI and Knowledge succeed.

[^okf-spec]: Recursive subdirectories and `index.md` progressive disclosure.
[^discover]: Discovery walks any depth; ID is the relative path.
[^decision]: Draft layout contract; not approved.
[^write-knowledge]: Collection-by-intent authoring.
[^goto-js]: Cmd-K scores and displays path; nested layout uses path as a sort tiebreaker.
[^presentation]: Review HTML, collection titles from H1, and breadcrumbs are built from path prefixes.
