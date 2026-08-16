---
type: Status
title: Known Rocci limitations
description: Rocci deliberately lacks dynamic Rocs islands, complete machine-readable documentation outputs, production packaging, broad native APIs, and full cross-platform validation.
tags: [domain/rocci, domain/rocs, domain/desktop, concern/validation, concern/packaging]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
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
  - id: status-doc
    resource: ../../docs/project/status.rocdown
    title: Published project status
    author: human:nils
    last_modified: 2026-08-16
  - id: rocs-site
    resource: ../../crates/rocs/src/site.rs
    title: Current Rocs site loader
    author: process:git
    last_modified: 2026-08-16
  - id: roadmap-plan
    resource: ../../ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md
    title: Rocs implementation plan
    author: human:nils
    last_modified: 2026-08-16
---

# Known Rocci limitations

## Snapshot date

2026-08-16.

## Static documentation

Rocs rejects pages containing `@render`, Roc blocks, Rocci templates, handlers, file CSS, or custom layouts; the dynamic-island splice path is not implemented.[^rocs-site]

Search, clean per-page Markdown, and some machine-output polish remain in the active Rocs Phase 4 backlog. Watch/serve and live reload are already implemented, so older prose that lists development watch mode as pending is stale on that point.[^roadmap-plan][^status-doc]

## Runtime and desktop delivery

Packaging is limited to a local, ad-hoc-signed macOS application. Production signing, notarization, update delivery, Windows and Linux installers, tray and deep-link integration, and full platform CI remain absent.[^roadmap]

The desktop host exposes the current window/webview boundary but not general native capabilities such as dialogs, filesystem access, or notifications. Multi-window application lifecycle is also not connected to authored Roc apps.[^roadmap]

## Language and client behavior

There is no implemented `@island` construct. Rich browser-owned behavior therefore remains an explicit future boundary rather than a capability authors can rely on today.[^roadmap]

## Validation

Review this record when a cited source changes or on its `stale_after` date. The published status page is supporting evidence, not final authority where current code or the active implementation plan differs.

[^roadmap]: Current deliberate limitations and unchecked roadmap items.
[^status-doc]: Published limitations, including one superseded statement about watch mode.
[^rocs-site]: Static-page feature rejection in the current site loader.
[^roadmap-plan]: Current Rocs Phase 4 status and remaining outputs.
