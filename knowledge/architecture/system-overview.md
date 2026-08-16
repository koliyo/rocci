---
type: Architecture
title: Rocci system overview
description: Rocci separates template compilation, Markdown-first content, static documentation cataloging, runtime hosting, and desktop presentation.
tags: [domain/rocci, domain/rocdown, domain/rocs, concern/rendering]
status: draft
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: readme
    resource: ../../README.md
    title: Rocci README
    author: human:nils
    last_modified: 2026-08-16
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-16
  - id: workspace
    resource: ../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-16
---

# Rocci system overview

## Current contract

Rocci compiles `.rocci` templates to ordinary Roc HTML, keeps prose-first executable documents in `.rocdown`, and uses Rocs for multi-page static documentation builds.[^readme]

The workspace separates compiler, theme, runtime, desktop host, language-server, Rocs library, and Rocs CLI responsibilities across dedicated crates.[^workspace]

## Boundaries

Rocs owns static catalog, route, navigation, graph, artifact, and build planning in Rust. A Rocci theme owns the visible documentation shell and is compiled once per build.[^roadmap]

Knowledge records follow the [static OKF boundary](/decisions/static-okf-boundary.md) and remain inert Markdown.

## Not yet implemented

Dynamic Rocdown islands, production desktop signing and installers, and native capabilities beyond the current window/webview boundary remain planned work.[^roadmap]

[^readme]: Current repository overview and supported workflows.
[^roadmap]: Current architectural direction and remaining limitations.
[^workspace]: Current workspace membership and crate boundaries.
