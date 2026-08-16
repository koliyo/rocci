---
type: Design Standard
title: Rocci design-system knowledge
description: Rocci currently has a portable Rocdown article surface and a separate Rocs documentation shell; this draft records their design intent, public hooks, and review expectations without claiming a shared token implementation.
tags: [domain/design-system, domain/rocdown, domain/rocs, concern/theming, concern/accessibility]
status: draft
generated: { by: process:okf-phase-4, at: 2026-08-16T18:32:45Z }
stale_after: 2026-09-15
authority: descriptive
owners: [human:nils]
sources:
  - id: design-reference
    resource: ../../DESIGN.md
    title: Rocci design reference
    author: process:okf-phase-4
    last_modified: 2026-08-16
  - id: theme-readme
    resource: ../../crates/rocci-theme/README.md
    title: Standalone Rocdown theme reference
    author: process:git
    last_modified: 2026-08-16
  - id: theme-chrome
    resource: ../../crates/rocci-theme/src/themes/chrome.css
    title: Rocdown content chrome
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
    resource: ../../archive/reports/ROCDOWN_THEMING_REPORT.md
    title: Rocdown theming investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
---

# Rocci design-system knowledge

## Lifecycle and authority

This is a draft descriptive record generated from current code and project evidence. It makes design knowledge discoverable, but it does not replace code as evidence of shipped behavior and does not create normative token authority.[^design-reference][^okf-plan]

## Design intent

Rocci's contributor reference describes a calm, direct, content-first character: readable documentation comes before decorative polish, ordinary web semantics remain visible, and hierarchy should rely on structure, spacing, and type before accent color.[^design-reference]

Accessibility expectations cover keyboard operation and focus, contrast, zoom, reduced motion, forced colors, print, and distinguishable interaction and system states. These are review expectations. The repository does not currently contain evidence of a complete audit across those dimensions.[^design-reference]

## Shipped surfaces

### Standalone Rocdown

Standalone Rocdown accepts `paper`, `rocci`, `none`, a local theme name, or a CSS path. Page metadata wins over command or environment defaults, and `paper` is the fallback. `auto`, `light`, and `dark` select the color-scheme policy; `none` returns no injected theme CSS.[^theme-readme][^theme-resolver]

The stable authoring boundary is CSS: a theme assigns `--rd-*` custom properties under `.rd-document`, and shared chrome maps them to emitted `.rd-*` content classes. The chrome covers headings, paragraphs and lists, links, blockquotes, inline and block code, tables, media, and inline emphasis.[^theme-readme][^theme-chrome]

### Rocs documentation shell

Rocs owns a separate Rocci-authored shell. It controls header, navigation, article, outline, responsive transitions, a light/dark palette, and documentation-specific presentation. Its current custom properties and literals are not resolved through the standalone Rocdown theme package.[^rocs-theme]

Header dimensions, columns, breakpoints, sticky positioning, responsive navigation, and article measure are layout behavior. They should not be described as though a portable value format could own the whole shell.[^rocs-theme][^theming-report]

## Contributor guidance

- Preserve `.rd-*` classes and the documented `--rd-*` properties when changing portable Rocdown styling.[^theme-readme][^theme-chrome]
- Preserve readable measure, fluid sizing, media containment, and horizontal overflow for code and tables.[^theme-chrome][^design-reference]
- Keep links and component states recognizable without depending on color or hover alone; add explicit focus, disabled, loading, error, and empty treatment where relevant.[^design-reference]
- Treat presentation renderers, app UI, and the Rocs shell as distinct surfaces until an approved integration contract says otherwise.[^theming-report][^okf-plan]

## Research and proposals

The theming report contains useful research about package formats, adapters, and presentation renderers, but those proposals are not shipped contracts.[^theming-report]

DTCG is likewise research vocabulary in the current phase. A future proposal may explore shared semantic roles or generated adapters, but no token files, generator, resolver, theme migration, or new source of truth is approved by documenting that possibility.[^okf-plan]

## Review expectations

Review this record and `DESIGN.md` when supported theme names, scheme semantics, `.rd-*` hooks, Rocs shell structure, or accessibility expectations change. Promote the record only after a human compares its current-behavior claims with the cited code.

[^design-reference]: Draft human-facing intent, scope, accessibility baseline, and update policy.
[^theme-readme]: Current standalone theme selection and authoring interface.
[^theme-chrome]: Current mapping from CSS variables to Rocdown content hooks and layout.
[^theme-resolver]: Current built-in, local, path, disabled, precedence, and scheme behavior.
[^rocs-theme]: Current independent Rocs shell, palette, and responsive layout.
[^theming-report]: Research evidence for separating format, layout, visual theme, and presentation concerns.
[^okf-plan]: Phase 4 knowledge-only boundary and prohibition on implying DTCG adoption.
