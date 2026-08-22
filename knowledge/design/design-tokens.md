---
type: Design Standard
title: Rocci design-token research
description: Rocci ships CSS custom-property theme surfaces but no DTCG token system; this draft inventories current roles and records DTCG-informed future options as research only.
tags: [domain/design-system, domain/rocdown, concern/theming, integration/dtcg]
status: draft
generated: { by: process:okf-phase-4, at: 2026-08-17T23:00:00Z }
stale_after: 2026-09-15
authority: exploratory
owners: [human:nils]
sources:
  - id: paper-theme
    resource: ../../crates/rocci-theme/src/themes/paper.css
    title: Paper theme values
    author: process:git
    last_modified: 2026-08-16
  - id: rocci-theme
    resource: ../../crates/rocci-theme/src/themes/rocci.css
    title: Rocci theme values
    author: process:git
    last_modified: 2026-08-16
  - id: theme-chrome
    resource: ../../crates/rocci-theme/src/themes/chrome.css
    title: Rocdown content chrome
    author: process:git
    last_modified: 2026-08-16
  - id: rocs-theme
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-17
  - id: dtcg-format
    resource: https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/
    title: Design Tokens Format Module 2025.10
    author: organization:design-tokens-community-group
    last_modified: 2025-10-28
  - id: dtcg-resolver
    resource: https://www.w3.org/community/reports/design-tokens/CG-FINAL-resolver-20251028/
    title: Design Tokens Resolver Module 2025.10
    author: organization:design-tokens-community-group
    last_modified: 2025-10-28
  - id: dtcg-color
    resource: https://www.w3.org/community/reports/design-tokens/CG-FINAL-color-20251028/
    title: Design Tokens Color Module 2025.10
    author: organization:design-tokens-community-group
    last_modified: 2025-10-28
  - id: okf-plan
    resource: ../../archive/reports/OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
---

# Rocci design-token research

## Current implementation

Rocci does not have DTCG token files, a token generator, a resolver configuration, checked generated CSS, or token validation. Its current value-sharing mechanism is CSS custom properties in two separate surfaces.[^okf-plan][^paper-theme][^rocs-theme]

| Surface | Current roles | Resolution |
| --- | --- | --- |
| Standalone Rocdown | Body, heading, and code fonts; background, surface, text, muted, accent, border, code background, and code text; content-specific aliases | `paper` and `rocci` use CSS `light-dark()` values; `chrome.css` consumes aliases |
| Rocdown shell | Canvas, surface levels, ink hierarchy, border hierarchy, accent levels, code colors, and header height | Light values on the shell root with dark overrides under `prefers-color-scheme` |

The two built-in Rocdown themes share the same variable shape but provide different values. Content aliases decouple emitted `.rd-*` elements from foundation colors, while the shared chrome owns typography, spacing, sizing, borders, and layout.[^paper-theme][^rocci-theme][^theme-chrome]

The Rocdown shell has a similar semantic vocabulary, but different names, values, scope, and resolution. It also contains literal shadows, translucent colors, and layout values. Similarity is evidence for possible analysis, not proof that the surfaces already share tokens.[^rocs-theme]

## DTCG standards findings

The DTCG Format Module 2025.10 is a Final Community Group Report intended as a stable implementation target, but it is not a W3C Standard. It defines a JSON exchange format centered on token `$value`, with metadata such as `$type`, `$description`, `$deprecated`, groups, aliases, and preservable vendor `$extensions`. Types must be declared or inherited rather than guessed from values, and token or group names cannot start with `$` or contain `.`, `{`, or `}`.[^dtcg-format]

The Resolver Module represents ordered token sets and contextual modifiers. Resolution order matters because later applicable sets override earlier ones, so a future light/dark or product-theme model would need explicit precedence and validation rather than an assumed merge.[^dtcg-resolver]

The Color Module represents colors structurally using a color space and components, with optional alpha and fallback data. A future portable source should therefore not treat a CSS hexadecimal string alone as the complete color model.[^dtcg-color]

## Possible future mapping

The following is a research outline, not an approved schema:

- foundation roles could describe typography and colors independently of either CSS namespace;
- semantic roles could map article content and shell UI onto those foundations;
- theme and light/dark variants could be modeled as explicitly ordered contexts;
- compatibility output could preserve existing `--rd-*` names while a migration is evaluated.

Before adoption, a separate proposal must define ownership, naming, types, alias policy, color gamut and fallback rules, modifier precedence, generated-file policy, CSS compatibility checks, and treatment of shell-only layout values.[^dtcg-format][^dtcg-resolver][^dtcg-color][^okf-plan]

## Explicit non-claims

The DTCG reports are external research sources. This record does not make them Rocci's implementation authority, does not claim the present CSS variables conform to DTCG, and does not approve token artifacts or migration work.[^okf-plan]

[^paper-theme]: Current Paper font, foundation-color, and content-alias values.
[^rocci-theme]: Current Rocci font, foundation-color, and content-alias values.
[^theme-chrome]: Current consumer of the `--rd-*` compatibility surface.
[^rocs-theme]: Current independent Rocdown shell variables, literals, and media-query resolution.
[^dtcg-format]: External format, naming, typing, alias, group, metadata, and extension vocabulary.
[^dtcg-resolver]: External ordered-set and contextual-modifier resolution model.
[^dtcg-color]: External structured color representation.
[^okf-plan]: Approved boundary: DTCG is knowledge evidence only and implementation requires a separate proposal.
