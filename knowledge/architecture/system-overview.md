---
type: Architecture
title: Rocci system overview
description: Rocci separates template compilation, Markdown-first content, static documentation cataloging, runtime hosting, and desktop presentation.
tags: [domain/rocci, domain/rocdown, domain/runtime, domain/desktop, concern/rendering]
status: draft
generated: { by: process:okf-migration, at: 2026-08-17T23:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: readme
    resource: ../../README.md
    title: Rocci README
    author: human:nils
    last_modified: 2026-08-17
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-17
  - id: workspace
    resource: ../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-17
---

# Rocci system overview

## Current contract

Rocci compiles `.rocci` templates to ordinary Roc HTML, keeps prose-first executable documents in `.rocdown`, compiles multi-page static documentation sites with `rocdown build`, and hosts applications in an embedded Tao/Wry window.[^readme]

The workspace separates compiler, template, theme, runtime, desktop host, Rocdown document compiler, document CLI, language servers (generic and Rocdown composition), highlighter, portable OKF engine, OKF application, shared UI, and Datastar integration across 14 dedicated crates with enforced one-way dependencies.[^workspace]

## Boundaries

Rocdown owns static catalog, route, navigation, graph, artifact, and build planning in Rust, while `RocdownTheme.rocci` owns the visible documentation shell compiled once per build.[^roadmap]

Knowledge records follow the [static OKF boundary](/decisions/static-okf-boundary.md) and remain inert Markdown managed by the portable `okf` engine and `rocci-okf` application.

Domain-neutral view records and presentation primitives live in `rocci-ui`.

## Not yet implemented

Dynamic Rocdown islands, production desktop signing and installers, and native capabilities beyond the current window/webview boundary remain planned work.[^roadmap]

[^readme]: Current repository overview and supported workflows.
[^roadmap]: Current architectural direction and remaining limitations.
[^workspace]: Current workspace membership and crate boundaries.
