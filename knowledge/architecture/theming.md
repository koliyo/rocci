---
type: Architecture
title: Rocci theming surfaces
description: Standalone Rocdown uses CSS variable themes while Rocdown sites use a Rocci-authored documentation shell and rocci-ui view records; DTCG is research evidence, not an adopted token authority.
tags: [domain/rocdown, domain/design-system, concern/theming, concern/accessibility]
status: draft
generated: { by: process:okf-phase-4, at: 2026-08-17T23:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
  - id: theme-readme
    resource: ../../crates/rocci-theme/README.md
    title: Standalone Rocdown theme reference
    author: process:git
    last_modified: 2026-08-17
  - id: theme-resolver
    resource: ../../crates/rocci-theme/src/resolve.rs
    title: Theme resolver implementation
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-theme
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-17
  - id: ui-readme
    resource: ../../crates/rocci-ui/README.md
    title: Rocci UI shared primitives and base styles
    author: process:git
    last_modified: 2026-08-17
  - id: theming-report
    resource: ../../archive/reports/ROCDOWN_THEMING_REPORT.md
    title: Rocdown theming investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Approved OKF and DTCG plan
    author: human:nils
    last_modified: 2026-08-16
  - id: design-system
    resource: ../design/design-system.md
    title: Rocci design-system knowledge
    author: process:okf-phase-4
    last_modified: 2026-08-16
  - id: design-tokens
    resource: ../design/design-tokens.md
    title: Rocci design-token research
    author: process:okf-phase-4
    last_modified: 2026-08-16
---

# Rocci theming surfaces

## Current contract

Standalone Rocdown resolves `paper`, `rocci`, `none`, a local theme name, or a CSS path. Page metadata wins over CLI or environment defaults, and the selected `auto`, `light`, or `dark` scheme controls document attributes and color-scheme metadata.[^theme-readme][^theme-resolver]

Standalone themes set `--rd-*` variables under `.rd-document`; shared chrome CSS maps those variables to stable Markdown classes. Built-ins are compiled into `rocci-theme`, while named local files are discovered under the user's theme directory.[^theme-readme][^theme-resolver]

Multi-page documentation sites use `RocdownTheme.rocci`, which owns layout, navigation, article presentation, responsive behavior, and shell palette variables using structured `PageView` records from `rocci-ui`. Its extracted CSS is fingerprinted as a build asset.[^rocdown-theme][^ui-readme]

## Boundaries

Format, layout, visual theme, and code highlighting are separate concerns. The current implementation covers native article themes and a first-party documentation shell, not presentation renderers or a general external theme-package interface.[^theming-report]

## DTCG research boundary

DTCG is source material and vocabulary for design knowledge. It has not been adopted as Rocci's portable value source, and the current phase does not approve token files, generators, compatibility adapters, validation, or a theme migration.[^okf-plan]

Any future token proposal would still need to keep DOM classes, page structure, cascade rules, assets, and layout behavior outside the portable value model unless a separately reviewed design changes those boundaries.[^okf-plan][^theming-report]

## Current gap

No DTCG token files, resolver matrix, generated shared CSS, manifest, or token validation exists. Current variables and literal shell values therefore remain distinct implementation surfaces. The root `DESIGN.md` and design knowledge records describe those surfaces without creating a new machine-readable authority.[^okf-plan][^design-system][^design-tokens]

## Evidence policy

The resolver and shell establish current behavior. The theming report remains useful design research, but its package, adapter, and presentation proposals are not treated as shipped.[^theming-report]

[^theme-readme]: Current standalone selection precedence and CSS-variable authoring contract.
[^theme-resolver]: Built-in, local, path, alias, and color-scheme resolution in code.
[^rocdown-theme]: Current independent Rocdown documentation shell layout and palette implementation.
[^ui-readme]: Shared UI domain-neutral view records in rocci-ui.
[^theming-report]: Research-derived separation of format, layout, theme, and code theme.
[^okf-plan]: Amended Phase 4 DTCG research role and explicit non-adoption boundary.
[^design-system]: Phase 4 description of current design intent and shipped surface boundaries.
[^design-tokens]: Phase 4 inventory of current CSS roles and DTCG research.
