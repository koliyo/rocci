---
type: Research Report
title: Mobile chrome for OKF, Rocdown, and rocci.dev
description: Code-backed inventory of narrow-viewport navigation, article overflow, and review-table behavior across the OKF review site, default Rocdown shells, and the project site theme. Recommends no-JS details menus, separating global nav from TOC, and aligning breakpoints without a shared token system.
tags: [domain/okf, domain/rocci-okf, domain/rocdown, domain/rocci, concern/rendering, concern/accessibility, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-19T15:10:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/shared/mobile-chrome.md
    title: Mobile chrome implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: site-plan
    resource: ../../plans/site/rocci-dev-site.md
    title: rocci.dev site architecture and Rocdown evolution
    author: process:codex
    last_modified: 2026-08-18
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell decision
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: theming
    resource: ../../architecture/theming.md
    title: Rocci theming surfaces
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: design-system
    resource: ../../design/design-system.md
    title: Rocci design-system knowledge
    author: process:okf-phase-4
    last_modified: 2026-08-17
  - id: design-ref
    resource: ../../../DESIGN.md
    title: Rocci design reference
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: ui-readme
    resource: ../../../crates/rocci-ui/README.md
    title: rocci-ui view records and chrome templates
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-theme
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Default Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-base
    resource: ../../../crates/rocci-rocdown/templates/RocdownBase.rocci
    title: Shared Rocdown article styles
    author: process:git
    last_modified: 2026-08-18
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: rocci.dev site shell
    author: process:git
    last_modified: 2026-08-18
  - id: site-layouts
    resource: ../../../site/theme/Layouts.rocci
    title: rocci.dev named layouts
    author: process:git
    last_modified: 2026-08-19
  - id: nav-list
    resource: ../../../crates/rocci-ui/templates/chrome/NavList.rocci
    title: Shared navigation list chrome
    author: process:git
    last_modified: 2026-08-19
  - id: page-outline
    resource: ../../../crates/rocci-ui/templates/chrome/PageOutline.rocci
    title: Shared on-this-page outline chrome
    author: process:git
    last_modified: 2026-08-18
  - id: okf-theme
    resource: ../../../crates/rocci-okf/templates/OkfTheme.rocci
    title: OKF knowledge shell
    author: process:git
    last_modified: 2026-08-19
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: OKF review HTML, app.css, and Rust write fallback
    author: process:git
    last_modified: 2026-08-19
  - id: review-queue
    resource: ../../../crates/rocci-okf/templates/ReviewQueue.rocci
    title: OKF review queue tables and filters
    author: process:git
    last_modified: 2026-08-18
  - id: concept-meta
    resource: ../../../crates/rocci-okf/templates/ConceptMeta.rocci
    title: OKF concept metadata and sources table
    author: process:git
    last_modified: 2026-08-18
  - id: theme-chrome
    resource: ../../../crates/rocci-theme/src/themes/chrome.css
    title: Standalone Rocdown article chrome
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Rocdown format and site contract
    author: process:git
    last_modified: 2026-08-19
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: rocci-okf usage contract
    author: process:git
    last_modified: 2026-08-19
  - id: generator-report
    resource: ../../../archive/reports/ROCDOWN_DOCUMENTATION_GENERATOR_REPORT.md
    title: Historical Rocdown documentation-generator research
    author: human:nils
    last_modified: 2026-08-16
  - id: known-limitations
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:okf-phase-6
    last_modified: 2026-08-17
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: compile-research
    resource: ../okf-compile-render-cost.md
    title: OKF preview compile and render cost
    author: process:cursor
    last_modified: 2026-08-19
---

# Mobile chrome for OKF, Rocdown, and rocci.dev

## Scope and authority

This is exploratory synthesis from current templates and CSS, not a visual
audit and not an approved accessibility certification. It asks how the three
public HTML surfaces behave below the existing `48rem` and tablet
breakpoints, and what it would take for a phone-width browser to read and
navigate them without JavaScript.[^plan][^design-ref]

The three surfaces are product-owned shells over shared view records, not one
responsive framework:[^theming][^catalog-shell][^ui-readme]

| Surface | Owner | Shell | Typical URL |
| --- | --- | --- | --- |
| OKF review site | `rocci-okf` | `OkfTheme.rocci` plus `presentation.rs` `app.css` | `rocci-okf run` HTTP origin |
| Default Rocdown site | `rocci-rocdown` | `RocdownTheme.rocci` | `rocdown build` / `run` without a project theme |
| rocci.dev | `site/theme` | `SiteShell.rocci` and `Layouts.rocci` | public static site |

Standalone Rocdown documents (`rocdown run` on one file, `paper` / `rocci`
themes) are a fourth, smaller chrome: left `.rd-toc` over article content.
They share the OKF review site's TOC-hiding rule.[^theme-chrome][^rocdown-readme]

The desktop preview window is not the mobile target. Mobile means the same
HTTP origin a phone can open: public `rocci.dev`, `rocdown` static output, and
`rocci-okf run --no-window`.[^okf-readme]

Implementation plan: [Mobile chrome](/plans/shared/mobile-chrome.md). Not
shipped.[^plan]

## Established constraints

Rust owns catalog, routes, and article HTML. Rocci owns visible chrome.
Layout, breakpoints, sticky headers, and navigation collapse are layout
behavior, not design tokens.[^catalog-shell][^theming][^design-system][^design-ref]

`rocci-ui` may hold domain-neutral chrome (`NavList`, `PageOutline`,
`Breadcrumbs`) after two consumers share a contract. Product shells stay
product-owned. OKF must not depend on Rocdown. Canonical knowledge stays
inert Markdown.[^ui-readme][^static-okf]

Default pages should remain usable with JavaScript disabled. Documentation
tabs already ship as stacked no-JS sections.[^site-plan][^known-limitations]
Historical generator research asked for a 320 CSS-pixel layout without
page-level horizontal scrolling, a keyboard-usable mobile menu and TOC, and
a no-JS site. That report is evidence, not a shipped checklist.[^generator-report]

The rocci.dev plan already names mobile as a Phase 2 exit condition for the
full route tree. That exit is not met for documentation navigation.[^site-plan]

## What the code does today

All three multi-page shells emit
`viewport` `width=device-width, initial-scale=1`.[^rocdown-theme][^site-shell][^presentation]
Article measure, code overflow, and images are mostly in place: RocdownBase
sets `.article { overflow-x: auto }` and code-block overflow; standalone
chrome sets `.rd-image { max-width: 100% }` and code-block overflow; OKF
`pre` scrolls horizontally.[^rocdown-base][^theme-chrome][^presentation]

Shared `NavList` and `PageOutline` are structure-only. They do not implement
a drawer, media queries, or tap-target sizing. Each shell styles and shows
or hides them.[^nav-list][^page-outline]

### Default Rocdown theme

`RocdownTheme.rocci` is the only shell with a replacement for hidden
navigation. At `max-width: 70rem` it drops the outline. At `max-width: 48rem`
it switches the grid to a single column, hides sidebar, lanes, subtitle, and
source link, and reveals a header `<details class="mobile-menu">` whose panel
repeats lanes plus `NavList`. The panel is `position: fixed` under the sticky
header. Native `<details>` keeps this working without JavaScript.[^rocdown-theme]

The mobile panel does not include the page outline. Heading navigation on a
phone therefore depends on in-article headings. The Menu `summary` padding is
`.38rem .7rem` at `.82rem` type, below the 24 CSS-pixel target the historical
report treated as a minimum.[^rocdown-theme][^generator-report]

### rocci.dev site theme

`Layouts.rocci` docs layout is a three-column grid that collapses to two
columns (outline hidden) at `64rem` and to one column (sidebar `display:
none`) at `48rem`. There is no `<details>` menu, no duplicate `NavList` in
the header, and no compact TOC.[^site-layouts]

`SiteShell.rocci` keeps brand, subtitle, section lanes, and Source link in
one non-wrapping flex header. Lanes are not hidden at the docs-layout
breakpoint. On a 320–400 CSS-pixel viewport the header and docs pages
therefore lose sidebar navigation while still showing a crowded top
bar.[^site-shell][^site-layouts]

Project themes compile `RocdownBase` when the theme directory does not define
it, so article overflow still applies. Chrome, not prose styles, is the site
gap.

### OKF review site

`KnowledgeShell` puts Home and Governance & Review links inside the same
`<aside class="rd-toc">` as `PageOutline`, and only renders that aside when
`has_outline` is true. `app.css` then does `.rd-toc { display: none }` at
`max-width: 48rem`, the same rule as standalone Rocdown. On a phone, both
on-this-page links and the only global review navigation disappear together.
A concept without outline headings never gets those links, even on a wide
window.[^okf-theme][^presentation][^theme-chrome]

The review queue and concept source lists are multi-column tables. The
Rocci template wraps them in `.okf-table-container`, but the served OKF
`app.css` in `presentation.rs` does not define overflow on that class.
`RocdownTheme.rocci` still carries leftover `.okf-table-container {
overflow-x: auto }` from the pre-split OKF presentation path; that rule is
not the stylesheet `rocci-okf` serves. The filter bar is a non-wrapping flex
row of buttons plus a search input. Filtering is a `type="button"`
enhancement; the full table remains in the DOM without JavaScript.[^review-queue][^concept-meta][^presentation][^rocdown-theme]

OKF still has a Rust HTML write fallback when Roc apply output is missing.
Chrome structure therefore exists twice: `.rocci` templates and
`presentation.rs` string builders. A mobile markup change that updates only
the Rocci shell will not cover the fallback path.[^presentation][^compile-research]

### Standalone Rocdown

The README documents that the automatic left navigator is hidden on narrow
and print viewports. That is current behavior, not an accidental CSS
omission. A long single document on a phone is readable but has no compact
TOC control.[^rocdown-readme][^theme-chrome]

## Findings

1. **rocci.dev docs are not navigable at the sidebar breakpoint.** Hiding
   `.sidebar` without a menu removes the documentation tree. Breadcrumbs and
   previous/next remain, which is not enough to move across guides.[^site-layouts]
2. **OKF hides unique navigation inside a TOC that mobile CSS discards.**
   Home / Review must not live under `.rd-toc` if that class is the
   “hide on narrow” hook.[^okf-theme][^presentation]
3. **The default Rocdown theme is the existence proof.** A no-JS
   `<details>` menu already ships. The site theme diverged and dropped it
   when named layouts moved into `site/theme`.[^rocdown-theme][^site-shell]
4. **Breakpoints are not a shared contract.** RocdownTheme uses `70rem` then
   `48rem`; the site uses `64rem` then `48rem`; OKF and standalone chrome
   only use `48rem`. Tablet outline collapse and phone nav collapse should
   be named and aligned even if pixel values stay local to each
   shell.[^rocdown-theme][^site-layouts][^presentation]
5. **Article overflow is ahead of chrome.** Code, images, and the article
   column already try to contain width. Wide OKF tables and the site header
   are the likely page-level scroll sources, not ordinary Markdown
   paragraphs.[^rocdown-base][^presentation][^site-shell]
6. **Tap targets and iOS chrome are untested in-repo.** DESIGN.md records
   that no repository-wide contrast, focus, zoom, or print audit exists.
   Safe-area insets, `100dvh` versus `100vh`, and 24px targets are not
   implemented as a policy.[^design-ref][^generator-report]
7. **Shared extraction is not the first patch.** Copying the details pattern
   into the site and splitting OKF nav from TOC does not require a new
   `rocci-ui` widget. Extract `MobileNav` only after RocdownTheme and
   `site/theme` share the same slot contract (lanes, sidebar, optional
   outline).[^ui-readme]

## Recommendation

Keep three shells. Make them obey one narrow-viewport policy:

- At the phone breakpoint, the reading column is full width.
- Every navigation region that `display: none` removes from the layout has a
  labeled `<details>` (or equivalent no-JS) replacement in the header or
  above the article.
- Global links are never descendants of the class that hides the outline.
- Wide tables scroll inside a labeled region; they do not expand the page.
- Header secondary chrome (subtitle, source, extra lanes) yields to brand
  plus menu.
- JavaScript may enhance filters and scroll-spy; it must not be the only
  path to another page.

Apply that policy in product order: OKF structure first (lost unique nav),
rocci.dev menu second (public docs), RocdownTheme hardening third (menu
exists, TOC and targets do not), standalone compact TOC last.

Do not introduce a client drawer library, Datastar menu, or `@island` for
this. Do not fold OKF review chrome into RocdownTheme. Do not encode
breakpoints as portable tokens.[^known-limitations][^theming]

## Open questions

Human approval is required before treating these as normative:

1. Whether phone and tablet breakpoints become one documented pair (`48rem` /
   `64rem`) or stay per-shell with the same hide/replace rules.
2. Whether the OKF review queue on a phone becomes horizontally scrolling
   tables, stacked definition lists, or both (table for wide, list via CSS).
3. Whether standalone Rocdown grows an in-article TOC `<details>` or keeps
   the documented “hidden on narrow” behavior.
4. Whether `MobileNav` moves into `rocci-ui` in the same change that fixes
   the site, or after both shells have matching markup.

[^plan]: Companion implementation plan; exploratory; no phase started.
[^site-plan]: Named layouts, no-JS site, and mobile as a rocci.dev Phase 2 exit.
[^catalog-shell]: Rust catalog versus Rocci shell ownership.
[^theming]: Separate standalone, documentation-shell, and layout concerns.
[^design-system]: Current two styling surfaces and layout-versus-token split.
[^design-ref]: Contributor accessibility expectations and lack of a recorded audit.
[^ui-readme]: Domain-neutral view records and chrome templates; product shells remain product-owned.
[^rocdown-theme]: Default site header, mobile `<details>` menu, and breakpoint CSS.
[^rocdown-base]: Article overflow, code-block scrolling, and table styles compiled into project themes.
[^site-shell]: rocci.dev header without a mobile menu.
[^site-layouts]: Docs grid that hides outline and sidebar with no replacement.
[^nav-list]: Shared nav markup without responsive behavior.
[^page-outline]: Shared outline markup without a compact control.
[^okf-theme]: Home/Review links nested in the outline aside.
[^presentation]: OKF `app.css` TOC hiding, missing table-container overflow, and Rust HTML fallback.
[^review-queue]: Five-column review tables and JS filter buttons.
[^concept-meta]: Concept source tables.
[^theme-chrome]: Standalone `.rd-toc` hidden at `48rem`; image max-width and code overflow.
[^rocdown-readme]: Documented hiding of the left navigator on narrow viewports.
[^okf-readme]: Preview is an HTTP origin; git provenance off by default for `run`.
[^generator-report]: Historical 320px, mobile menu/TOC, and no-JS acceptance language.
[^known-limitations]: No `@island`; tabs are stacked no-JS sections.
[^static-okf]: Canonical knowledge remains inert Markdown; presentation is `rocci-okf`.
[^compile-research]: Apply output versus Rust write fallback duplication.
