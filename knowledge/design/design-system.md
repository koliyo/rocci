---
type: Design Standard
title: Rocci design-system knowledge
description: Rocci currently has a portable Rocdown article surface and a separate Rocdown documentation shell; this draft records their design intent, public hooks, and review expectations without claiming a shared token implementation.
tags: [domain/design-system, domain/rocdown, concern/theming, concern/accessibility]
status: draft
generated: { by: process:okf-phase-4, at: 2026-08-31T08:00:00Z }
stale_after: 2026-09-15
authority: descriptive
owners: [human:nils]
sources:
  - id: design-reference
    resource: ../../DESIGN.md
    title: Rocci design reference
    author: process:okf-phase-4
    last_modified: 2026-08-17
  - id: theme-readme
    resource: ../../crates/rocci-theme/README.md
    title: Standalone Rocdown theme reference
    author: process:git
    last_modified: 2026-08-17
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
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-17
    last_modified: 2026-08-16
---

# Rocci design-system knowledge

## Design intent

Rocci's public visual surface is deliberately plain and document-first. The default typographic system uses neutral system fonts, readable measure, explicit hierarchy, visible component states, and dark-scheme support without layout shifts.[^design-reference]

The design standard is intended for human authoring and review. It is not an assertion that a shared cross-crate design token system exists.

## Two current surfaces

Rocci has two distinct styling surfaces today. They do not share a token pipeline or CSS variables.

### Standalone Rocdown article

Standalone Rocdown accepts `paper`, `rocci`, `none`, a local theme name, or a CSS path. Page metadata wins over command or environment defaults, and `paper` is the fallback. `auto`, `light`, and `dark` select the color-scheme policy; `none` returns no injected theme CSS.[^theme-readme][^theme-resolver]

The stable authoring boundary is CSS: a theme assigns `--rd-*` custom properties under `.rd-document`, and shared chrome maps them to emitted `.rd-*` content classes. The chrome covers headings, paragraphs and lists, links, blockquotes, inline and block code, tables, media, and inline emphasis.[^theme-readme][^theme-chrome]

### Rocdown documentation shell

Rocdown owns a separate Rocci-authored shell. It controls header, navigation, article, outline, responsive transitions, a light/dark palette, and documentation-specific presentation. Its current custom properties and literals are not resolved through the standalone Rocdown theme package.[^rocs-theme]

Header dimensions, columns, breakpoints, sticky positioning, responsive navigation, and article measure are layout behavior. They should not be described as though a portable value format could own the whole shell.[^rocs-theme]

## Contributor guidance

- Preserve `.rd-*` classes and the documented `--rd-*` properties when changing portable Rocdown styling.[^theme-readme][^theme-chrome]
- Preserve readable measure, fluid sizing, media containment, and horizontal overflow for code and tables.[^theme-chrome][^design-reference]
- Keep links and component states recognizable without depending on color or hover alone; add explicit focus, disabled, loading, error, and empty treatment where relevant.[^design-reference]
- Treat presentation renderers, app UI, and the documentation shell as distinct surfaces until an approved integration contract says otherwise.

## Research and proposals

The theming report contains useful research about package formats, adapters, and presentation renderers, but those proposals are not shipped contracts.

## Verification triggers

Review this record and `DESIGN.md` when supported theme names, scheme semantics, `.rd-*` hooks, documentation shell structure, or accessibility expectations change. Promote the record only after a human compares its current-behavior claims with the cited code.

[^design-reference]: Root design statement covering visual tone, states, accessibility, and testing.
[^theme-readme]: Supported theme names, resolution precedence, and custom property table.
[^theme-chrome]: Base typography, responsive layout, element classes, and media query definitions.
[^theme-resolver]: Current resolution logic for built-in and external theme assets.
[^rocs-theme]: Current independent Rocdown shell, palette, and responsive layout.
