# Rocci design reference

Status: draft contributor reference, last reviewed against the implementation on 2026-08-16. Code remains the evidence for shipped behavior. This document does not establish a token source of truth or approve a theme migration.

## Character and priorities

Rocci should feel calm, direct, and content-first. Documentation must remain readable before decorative polish; application controls and the desktop shell should support the content rather than imitate it. Prefer clear hierarchy, restrained color, durable browser behavior, and a small number of meaningful states. Avoid ornamental motion, low-contrast chrome, and styling that hides ordinary web semantics.

The design surfaces have different jobs:

- Rocdown is the portable article surface. Its stable styling hooks are the `.rd-*` classes emitted by the renderer and the `--rd-*` custom properties consumed by the bundled chrome.
- Rocs is the documentation-site shell. It owns the header, navigation, article column, outline, responsive layout, and its present shell palette.
- Presentation renderers and broader application UI are separate design problems; neither is silently governed by the article theme.

## Current theme model

Standalone Rocdown supports `paper`, `rocci`, `none`, a local theme name, or a CSS path. Page metadata has priority over command or environment defaults, with `paper` as the fallback. `none` disables injected theme CSS. `auto` follows the user agent's light/dark preference; `light` and `dark` force the corresponding color-scheme branch. See the [theme reference](crates/rocci-theme/README.md), [resolver](crates/rocci-theme/src/resolve.rs), and [scheme policy](crates/rocci-theme/src/scheme.rs).

The bundled themes define three font roles, eight foundation color roles, and content aliases under `.rd-document`. [Chrome CSS](crates/rocci-theme/src/themes/chrome.css) maps those aliases to headings, paragraphs, links, blockquotes, code, tables, media, and inline emphasis. Theme authors should preserve these public hooks even when changing values.

Rocs currently has a separate implementation in [RocsTheme.rocci](crates/rocs/templates/RocsTheme.rocci). Its custom properties describe canvas and surface levels, text hierarchy, borders, accents, code colors, and header height. The shell also contains layout rules and a few literal values; those are current implementation details, not a shared token system.

## Content and layout guidance

- Keep article measure readable and preserve fluid sizing and horizontal overflow for code and tables.
- Use body, heading, and code type by role. Do not rely on a particular installed font for meaning.
- Use text hierarchy, spacing, and structure before adding accent color.
- Keep links recognizable without requiring hover. Images and other media must fit their content column.
- Treat header, navigation, outline, breakpoints, positioning, and responsive transitions as layout behavior. They are not color or typography tokens.

## Accessibility baseline

New or changed UI should provide visible keyboard focus, keyboard-operable controls, sufficient text and non-text contrast, meaningful document structure, and usable layouts at browser zoom. Information and state must not depend on color alone. Motion should respect `prefers-reduced-motion`; forced-colors and print modes should remain understandable; loading, empty, error, disabled, hover, focus, and active states should be distinguishable where they exist.

These are review expectations, not a claim that every current surface has been audited. The present code explicitly supports light/dark color schemes and responsive layout, but a repository-wide contrast, focus, forced-colors, zoom, and print audit has not been recorded.

## DTCG research boundary

The Design Tokens Community Group 2025.10 reports offer useful vocabulary for a possible future portable token model: the [Format Module](https://www.w3.org/community/reports/design-tokens/CG-FINAL-format-20251028/), [Resolver Module](https://www.w3.org/community/reports/design-tokens/CG-FINAL-resolver-20251028/), and [Color Module](https://www.w3.org/community/reports/design-tokens/CG-FINAL-color-20251028/). They are research sources only in Rocci today.

Rocci has no DTCG token files, generator, resolver, compatibility map, or token-validation pipeline. Any proposal to introduce them must separately define ownership, naming, light/dark resolution, CSS compatibility, generated-file policy, and migration safeguards. Recording a possible mapping does not approve or implement it.

## Evidence and updates

Use [Rocci theming surfaces](knowledge/architecture/theming.md) for the architectural boundary and [Design-system knowledge](knowledge/design/design-system.md) plus [Design-token research](knowledge/design/design-tokens.md) for evidence and proposals. Update this reference when public hooks, supported themes or schemes, accessibility expectations, or the Rocs shell contract change. Substantive revisions should be checked against code and reviewed by a human before this document is treated as stable design authority.
