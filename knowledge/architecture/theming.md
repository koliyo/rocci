---
type: Architecture
title: Rocci theming surfaces
description: Standalone Rocdown uses CSS variable themes while Rocs uses a separate Rocci-authored documentation shell; shared DTCG tokens are approved but not implemented.
tags: [domain/rocdown, domain/rocs, domain/design-system, concern/theming, concern/accessibility]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
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
    last_modified: 2026-08-16
  - id: theme-resolver
    resource: ../../crates/rocci-theme/src/resolve.rs
    title: Theme resolver implementation
    author: process:git
    last_modified: 2026-08-16
  - id: rocs-theme
    resource: ../../crates/rocs/templates/RocsTheme.rocci
    title: Rocs documentation shell
    author: process:git
    last_modified: 2026-08-16
  - id: theming-report
    resource: ../../ROCDOWN_THEMING_REPORT.md
    title: Rocdown theming investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Approved OKF and DTCG plan
    author: human:nils
    last_modified: 2026-08-16
---

# Rocci theming surfaces

## Current contract

Standalone Rocdown resolves `paper`, `rocci`, `none`, a local theme name, or a CSS path. Page metadata wins over CLI or environment defaults, and the selected `auto`, `light`, or `dark` scheme controls document attributes and color-scheme metadata.[^theme-readme][^theme-resolver]

Standalone themes set `--rd-*` variables under `.rd-document`; shared chrome CSS maps those variables to stable Markdown classes. Built-ins are compiled into `rocci-theme`, while named local files are discovered under the user's theme directory.[^theme-readme][^theme-resolver]

Rocs does not consume that theme resolver for its full site shell. `RocsTheme.rocci` owns layout, navigation, article presentation, responsive behavior, and its own palette variables, and its extracted CSS is fingerprinted as a build asset.[^rocs-theme]

## Boundaries

Format, layout, visual theme, and code highlighting are separate concerns. The current implementation covers native article themes and a first-party Rocs shell, not presentation renderers or a general external theme-package interface.[^theming-report]

## Approved target

Phase 4 will introduce DTCG JSON as the source for portable design values, with generated compatibility adapters for existing `--rd-*` consumers and the Rocs shell. DTCG will not own DOM classes, page structure, cascade rules, assets, or layout behavior.[^okf-plan]

## Not yet implemented

No DTCG token files, resolver matrix, generated shared CSS, manifest, or root `DESIGN.md` exists yet. Current variables and literal shell values therefore remain two distinct source surfaces.[^okf-plan]

## Evidence policy

The resolver and shell establish current behavior. The theming report remains useful design research, but its package, adapter, and presentation proposals are not treated as shipped.[^theming-report]

[^theme-readme]: Current standalone selection precedence and CSS-variable authoring contract.
[^theme-resolver]: Built-in, local, path, alias, and color-scheme resolution in code.
[^rocs-theme]: Current independent Rocs shell layout and palette implementation.
[^theming-report]: Research-derived separation of format, layout, theme, and code theme.
[^okf-plan]: Approved Phase 4 DTCG role and explicit unimplemented status.
