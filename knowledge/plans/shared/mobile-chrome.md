---
type: Implementation Plan
title: Mobile chrome for OKF, Rocdown, and rocci.dev
description: "Phased no-JS narrow-viewport work: split OKF global nav from TOC, restore a details menu on rocci.dev, harden the default Rocdown mobile menu, and contain wide tables. Layout stays in product shells; rocci-ui extraction is gated on matching markup."
tags: [domain/okf, domain/rocci-okf, domain/rocdown, domain/rocci, concern/rendering, concern/accessibility, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/shared/mobile-chrome.md
    title: Mobile chrome for OKF, Rocdown, and rocci.dev
    author: process:cursor
    last_modified: 2026-08-19
  - id: site-plan
    resource: ../rocci-dev-site.md
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
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: rocci.dev site shell
    author: process:git
    last_modified: 2026-08-19
  - id: site-layouts
    resource: ../../../site/theme/Layouts.rocci
    title: rocci.dev named layouts
    author: process:git
    last_modified: 2026-08-19
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
    last_modified: 2026-08-19
  - id: theme-chrome
    resource: ../../../crates/rocci-theme/src/themes/chrome.css
    title: Standalone Rocdown article chrome
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-base
    resource: ../../../crates/rocci-rocdown/templates/RocdownBase.rocci
    title: Shared Rocdown article styles
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
  - id: design-ref
    resource: ../../../DESIGN.md
    title: Rocci design reference
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: known-limitations
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:okf-phase-6
    last_modified: 2026-08-17
  - id: compile-research
    resource: ../../research/okf/okf-compile-render-cost.md
    title: OKF preview compile and render cost
    author: process:cursor
    last_modified: 2026-08-19
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
---

# Mobile chrome for OKF, Rocdown, and rocci.dev

## Goal and scope

Make the OKF review site, default Rocdown documentation chrome, and rocci.dev
usable on a phone-width browser: reachable navigation, a readable article,
no page-level horizontal scrolling from chrome or tables, and no JavaScript
requirement for moving between pages.[^research][^site-plan]

This plan covers HTML structure and CSS in the three product shells, the
standalone Rocdown TOC rule, tests at the owning crate boundary, and a
gated `rocci-ui` extraction. It does not choose visual identity, add
public-site search, implement `@island`, or change the portable `okf`
engine.[^known-limitations][^static-okf]

Research: [Mobile chrome for OKF, Rocdown, and rocci.dev](/research/shared/mobile-chrome.md).
Exploratory. Phases 1–3 and 5–6 are implemented; Phase 4
was skipped because the site Menu is layout-gated and the default theme always
emits `NavList`. Not logged complete until CI and Knowledge workflows succeed
on that revision.[^research]

## Established baseline

Visible chrome belongs in Rocci. Rust keeps catalog and article HTML.
`rocci-ui` already supplies `NavList`, `PageOutline`, and `Breadcrumbs`
without media queries. Layout and breakpoints stay in product
shells.[^catalog-shell][^ui-readme][^theming][^nav-list][^page-outline]

`RocdownTheme.rocci` already ships a no-JS header `<details class="mobile-menu">`
at `max-width: 48rem`. The site theme hides `.sidebar` at the same width
without a replacement. OKF nests Home / Review inside `.rd-toc`, which
`app.css` hides at `48rem`. Standalone Rocdown documents the TOC as hidden
on narrow viewports.[^rocdown-theme][^site-layouts][^okf-theme][^presentation][^rocdown-readme]

OKF chrome exists in `.rocci` templates and in the Rust write fallback.
Until apply reliably writes every page, both paths must change
together.[^presentation][^compile-research]

## Narrow-viewport policy

All three shells should obey the same hide/replace rules even if exact
breakpoint pixels stay local until Phase 0 names them:

1. **Phone column.** Below the phone breakpoint, sidebar and outline are not
   in the layout grid. The article uses the full content width.
2. **Replace what you hide.** Any `nav` or complementary region removed with
   `display: none` has a labeled `<details>` (or equivalent disclosure)
   elsewhere, typically in the header or directly above the article.
3. **Global nav is not TOC.** Home, Review, section lanes, and the docs tree
   must not be descendants of the class that hides on-this-page links.
4. **No-JS first.** Native `<details>` / `<summary>` is the menu. Datastar,
   `toc.js`, and review-queue filters may enhance; they must not be the only
   way to reach another route.[^site-plan][^known-limitations]
5. **Contain width.** Code blocks and images already overflow or shrink
   inside the article. Wide tables scroll inside a labeled wrapper. The
   header must not force page-level overflow.[^rocdown-base][^theme-chrome]
6. **Targets.** New or changed isolated controls meet 24 CSS-pixel minimum
   targets; prefer ~44px for the Menu summary. Sticky chrome must not cover
   heading targets (`scroll-margin-top` already exists in several
   sheets).[^design-ref][^theme-chrome]

Ownership:

| Change | Owner |
| --- | --- |
| OKF shell, `app.css`, Rust fallback HTML | `rocci-okf` |
| Default documentation menu and article chrome | `rocci-rocdown` (`RocdownTheme.rocci`, `RocdownBase.rocci`) |
| rocci.dev header and docs layout | `site/theme` |
| Standalone TOC compact control | `rocci-theme` `chrome.css` plus default document shell if markup changes |
| Shared `MobileNav` only after two Rocdown shells match | `rocci-ui` |

Do not interpret templates in Rust to avoid compiling a theme. Do not add
Rocdown declarations to knowledge records.

## Phased implementation

### Phase 0 — freeze the contract

- Name phone and tablet breakpoints for the three shells (recommended
  starting pair: `48rem` phone, `64rem` tablet; RocdownTheme’s `70rem` may
  move to `64rem` or stay with a comment that tablet is “outline hidden”).
- Write the hide/replace table: lanes, sidebar, outline, OKF Home/Review,
  standalone TOC.
- Decide the OKF review-queue narrow treatment (scroll vs stacked list) and
  whether standalone Rocdown gets a compact TOC in this plan or stays
  documented-hidden.
- List fixture pages: a docs page with sidebar + outline, site home, OKF
  concept with sources, OKF `/review/`, a standalone article with H2/H3.

**Exit:** The table exists in this plan or a short follow-up note in
research; no pixel hunting during later phases. Human answers to decision
gates 1–3 are recorded.

Accepted freeze for this implementation:

| Surface | Phone (`max-width: 48rem`) | Tablet | Hide/replace |
| --- | --- | --- | --- |
| Default `RocdownTheme` | Header `<details class="mobile-menu">`; sidebar and outline out of the grid | Outline collapse stays at `70rem` | Menu replaces lanes and on-this-page links |
| rocci.dev `SiteShell` | Brand plus Menu; subtitle, source, and inline lanes hidden | Docs outline at `64rem`, sidebar at `48rem` | Menu always has section lanes; `NavList` and `PageOutline` only on the docs layout |
| OKF review shell | Home / Review always visible in `.okf-global-nav` | Same `48rem` phone rule; no separate tablet breakpoint | Outline stays in `.rd-toc`; phone gets `<details class="okf-outline-menu">` when non-empty |
| Standalone `chrome.css` | Hide `.rd-toc`; show `<details class="rd-toc-menu">` | No tablet rule | Compact TOC in scope |
| OKF review queue | `.okf-table-container { overflow-x: auto }`; filter bar wraps | — | Full table stays in the DOM |

Decision gates: (1) keep `70rem` on the default theme; (2) overflow-x table plus wrapping filters; (3) compact standalone TOC in this plan; (4) extract `MobileNav` only after both Rocdown shells match — they did not, so Phase 4 is skipped.

### Phase 1 — OKF navigation structure

- Render Home and Governance & Review outside `.rd-toc`, visible at every
  width, including concepts with no outline.
- Keep `PageOutline` in `.rd-toc` (or equivalent) and hide only that region
  at the phone breakpoint.
- Add a no-JS `<details>` “On this page” control for phone widths when the
  outline is non-empty.
- Define `.okf-table-container { overflow-x: auto }` in served `app.css`.
- Let the filter bar wrap; keep the full table in the DOM without JS.
- Apply the same markup to the Rust write fallback and add tests that Home /
  Review appear even when `has_outline` is false and that they are not
  inside `.rd-toc`.[^okf-theme][^presentation][^review-queue][^concept-meta]

**Exit:** `cargo test -p rocci-okf` covers the structure. A phone-width
browser can reach `/` and `/review/` from a concept page without JavaScript.

Status: implemented. Home / Review live in
`.okf-global-nav` outside `.rd-toc`; phone outline is `okf-outline-menu`.

### Phase 2 — rocci.dev menu

- Add a header `<details class="mobile-menu">` on `SiteShell` that, at the
  phone breakpoint, contains section lanes, docs `NavList` (docs layout),
  and `PageOutline` when present.
- Hide subtitle, source link, and inline lanes at that breakpoint so the
  header is brand plus Menu.
- Keep the docs grid collapse in `Layouts.rocci`; do not leave `.sidebar`
  visible in a one-column layout.
- Size the Menu summary to the target policy.
- Prefer copying `RocdownTheme`’s details pattern over a new JS drawer.
  Do not extract to `rocci-ui` in this phase unless Phase 4 is done in the
  same change by explicit decision.[^site-shell][^site-layouts][^rocdown-theme]

**Exit:** `rocdown build site` (or the repo’s configured site build) emits
the menu markup. A 320 CSS-pixel view of a docs page can open every sidebar
target. Home, product, news, FAQ, and plain layouts do not grow an empty
docs tree.

Status: implemented. `SiteShell` Menu is layout-gated so non-docs pages do
not copy an empty docs `NavList`.

### Phase 3 — default Rocdown theme hardening

- Put the page outline in the mobile panel (or a sibling details control).
- Raise Menu and lane tap targets to the policy.
- Prefer `100dvh` / `dvh` with a `100vh` fallback for the fixed panel; add
  `env(safe-area-inset-*)` padding on the header and panel.
- Wrap `.article .rd-table` so overflow is on the wrapper, not
  `overflow: hidden` on the table.
- Add a compile or fixture test that `mobile-menu` remains in the builtin
  shell HTML and that phone CSS still reveals it.[^rocdown-theme][^rocdown-base]

**Exit:** Default-theme sites keep today’s menu and gain TOC plus targets.
No regression in the `70rem`/`48rem` (or Phase 0) collapse.

Status: implemented. Outline is in the mobile panel; Menu/lane targets and
`100dvh` / safe-area padding landed; tables wrap in `.rd-table-wrap`.

### Phase 4 — optional shared `MobileNav`

Only if Phases 2 and 3 leave matching slots:

- Move the disclosure markup into `rocci-ui/templates/chrome/` with props
  for lanes, sidebar items, optional outline, and labels.
- Keep CSS for placement in the product shells (header height and z-index
  differ).
- OKF may reuse `PageOutline` only; do not force Home/Review through the
  Rocdown lane record.[^ui-readme]

**Exit:** Two Rocdown shells import the same component. OKF still compiles
without `rocci-rocdown`. Skip this phase if markup diverged on purpose.

Status: skipped. Site Menu omits docs `NavList` on non-docs layouts; the
default theme always emits it. Do not extract `MobileNav` into `rocci-ui`.

### Phase 5 — standalone Rocdown compact TOC

If Phase 0 kept this in scope:

- Replace “hide `.rd-toc` on narrow” with a compact disclosure that still
  uses heading IDs.
- Preserve print hiding.
- Update the Rocdown README sentence that currently documents hiding as
  the contract.[^theme-chrome][^rocdown-readme]

**Exit:** `rocdown run` on a headed document shows an On this page control
below `48rem`. Theme `none` still has no chrome.

Status: implemented. Default lowering emits a sibling `<details class="rd-toc-menu">`;
`chrome.css` reveals it at `48rem` and still hides both in print.

### Phase 6 — content fixtures and docs

- Add or extend a fixture page with a wide table, a long code fence, a
  `:tabs` block, and a deep sidebar.
- Check 320 / 768 / 1280 CSS-pixel layouts by resizing the HTTP origin (site
  build, `rocdown run`, `rocci-okf run --no-window`). Record that this is a
  maintainer check, not an automated pixel audit, unless a later decision
  adds screenshot tests.[^okf-readme]
- Update crate READMEs only where the public contract changed (standalone
  TOC, site theme behavior).
- Do not claim WCAG conformance; keep DESIGN.md’s “no recorded audit”
  language unless a human audit lands.[^design-ref]

**Exit:** README and this plan’s acceptance list match shipped chrome.

Status: implemented at the contract layer. Fixture:
`examples/rocdown/pages/Blocks.rocdown` (wide table, long fence, `:tabs`, nested
outline). Deep docs-tree sidebar coverage is a published `layout: "docs"`
page such as `docs/reference/rocdown.rocdown`. 320 / 768 / 1280 CSS-pixel
layout is a maintainer resize of the HTTP origin, not an automated pixel
audit. Public READMEs document `rd-toc-menu`, site `mobile-menu`, and OKF
global nav.

## Acceptance criteria

- Phone-width OKF concept, review, and home pages expose Home and Review
  without JavaScript.
- Phone-width rocci.dev docs pages expose lanes and the docs tree without
  JavaScript.
- Default Rocdown theme menu still works and includes on-this-page links
  when the outline is non-empty.
- No product shell relies on JavaScript to follow an in-site link.
- Wide OKF tables and Rocdown tables scroll inside their region at 320 CSS
  pixels in the fixture, without the page growing to the table’s min-width.
- Rust OKF fallback HTML matches the Rocci shell’s navigation structure.
- `cargo test -p rocci-okf`, `cargo test -p rocci-rocdown`, and
  `cargo test -p rocci-theme` cover the new structure or CSS contracts at
  the lowest owner. Site theme changes are checked by building `site/` and
  inspecting generated HTML for `mobile-menu`.
- Failed site or OKF builds still preserve the previous output tree.

## Decision gates

Human approval is required before treating these exploratory choices as
normative:

1. Align tablet outline collapse on `64rem` for both Rocdown shells, or keep
   `70rem` on the default theme.
2. OKF review queue on a phone: overflow-x table, stacked CSS list, or both.
3. Standalone Rocdown: compact TOC in this plan, or keep hidden-on-narrow.
4. Extract `MobileNav` into `rocci-ui` during Phase 2–3, or only after both
   Rocdown shells already match.

[^research]: Code-backed inventory of the three shells and hide-without-replace gaps.
[^site-plan]: One static site, Rocci-owned chrome, no-JS plus mobile as a Phase 2 exit.
[^catalog-shell]: Rust catalog versus Rocci shell.
[^theming]: Layout is not a token pipeline.
[^ui-readme]: Shared chrome only for demonstrated domain-neutral contracts.
[^rocdown-theme]: Existing details menu and leftover OKF table-container CSS.
[^site-shell]: Public header without a phone menu.
[^site-layouts]: Docs sidebar and outline hidden without replacement.
[^okf-theme]: Home/Review nested in the outline aside.
[^presentation]: TOC hiding, served CSS, and Rust fallback duplication.
[^review-queue]: Review tables and filter bar.
[^concept-meta]: Concept source tables.
[^theme-chrome]: Standalone TOC hide rule and article overflow.
[^rocdown-base]: Article and table overflow compiled into project themes.
[^nav-list]: Shared nav without media queries.
[^page-outline]: Shared outline without a compact control.
[^rocdown-readme]: Documented narrow TOC hiding.
[^okf-readme]: HTTP preview origin.
[^design-ref]: Accessibility review expectations; no recorded repository-wide audit.
[^known-limitations]: No `@island`; stacked no-JS tabs.
[^compile-research]: Apply versus Rust write fallback.
[^static-okf]: Knowledge records stay inert Markdown.
