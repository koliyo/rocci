---
type: Audit
title: rocci.dev site UX and authoring DX review
description: Current rocci.dev has a sound catalog and accessible documentation shell, but layout-specific chrome hides navigation on most section pages, breadcrumbs are incomplete, the live home page drops its primary cards, the page finder is visually broken, and the News surface should be removed.
tags: [domain/rocci, domain/rocdown, concern/ux, concern/tooling, concern/publication, concern/navigation, concern/accessibility]
status: draft
generated: { by: process:cursor, at: 2026-08-21T21:53:49Z }
stale_after: 2026-11-21
authority: descriptive
owners: [human:nils]
sources:
  - id: site-config
    resource: ../../../site/rocdown.toml
    title: Current rocci.dev catalog, mounts, and navigation
    author: human:nils
    last_modified: 2026-08-21
  - id: site-shell
    resource: ../../../site/theme/SiteShell.rocci
    title: Current rocci.dev document shell and responsive menu
    author: process:git
    last_modified: 2026-08-21
  - id: layouts
    resource: ../../../site/theme/Layouts.rocci
    title: Current rocci.dev named layouts and layout CSS
    author: process:git
    last_modified: 2026-08-21
  - id: catalog
    resource: ../../../crates/rocci-rocdown/src/catalog.rs
    title: Rocdown navigation, breadcrumb, and journey resolution
    author: process:git
    last_modified: 2026-08-21
  - id: planner
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Rocdown page views, sidebar projection, and hybrid page planning
    author: process:git
    last_modified: 2026-08-21
  - id: page-view
    resource: ../../../crates/rocci-ui/src/view.rs
    title: Shared page view contract
    author: process:git
    last_modified: 2026-08-21
  - id: goto
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette
    author: process:git
    last_modified: 2026-08-21
  - id: landing
    resource: ../../../site/index.rocdown
    title: Current rocci.dev landing page
    author: human:nils
    last_modified: 2026-08-21
  - id: faq
    resource: ../../../site/faq/index.rocdown
    title: Current rocci.dev FAQ
    author: process:git
    last_modified: 2026-08-21
  - id: project-status
    resource: ../../../site/project/status.rocdown
    title: Current public project status page
    author: process:git
    last_modified: 2026-08-21
  - id: root-readme
    resource: ../../../README.md
    title: Current site build and staging workflow
    author: human:nils
    last_modified: 2026-08-21
  - id: prior-plan
    resource: ../../plans/site/rocci-dev-site.md
    title: rocci.dev site UX and authoring improvement plan
    author: process:cursor
    last_modified: 2026-08-21
---

# rocci.dev site UX and authoring DX review

## Executive verdict

The current site has a strong technical foundation: one resolved catalog,
real links, a skip link, semantic landmarks, keyboard focus styling, curated
documentation groups, page outlines, previous/next journeys, responsive
navigation, atomic output, and a shared page view containing sidebar and
breadcrumb data.[^site-config][^site-shell][^layouts][^page-view]

The visible experience does not apply that foundation consistently. Layout
selection, rather than the route hierarchy, decides whether the already-built
sidebar and breadcrumbs are rendered. The result is that only detailed
`docs` pages get the complete reading shell. The Docs index, Rocdown index,
Project index, every Project article, every News page, and the generated 404
drop some or all orientation aids.[^site-shell][^layouts][^planner]

The target contract should be explicit: **only the landing page and FAQ omit
the section sidebar; every other page gets the current section navigation.
Every non-home page gets a compact, deduplicated breadcrumb trail.** FAQ stays
single-column but gains an inline contents control. News is marked for removal
from rocci.dev; removing that product surface must not remove Rocdown's generic
collection and feed capability.[^site-config][^prior-plan]

Two regressions deserve attention before the broader redesign. The hybrid
landing page does not render its authored `:card-grid` primary paths, and the
shared page finder opens as a static, multi-thousand-pixel block below the
page instead of a fixed modal because its shadow `:host { all: initial }`
resets the external host positioning.[^landing][^planner][^goto]

## Scope and method

This review covered source information architecture, page metadata, layout
composition, catalog and view construction, responsive CSS, generated HTML,
and current author/build workflow. It did not treat `dist/` as a source of
truth.

The current tree was checked and built with:

```sh
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- build site
cargo run -q -p rocci-rocdown-cli -- inspect nav site
```

Representative generated routes were then inspected at 1280 CSS pixels and
390 CSS pixels: `/`, `/docs/`, `/docs/start/install/`, `/rocdown/`,
`/rocdown/language/`, `/project/`, `/project/status/`, `/faq/`, `/news/`, and
`/news/introducing-rocci/`. The browser pass checked visible sidebar,
breadcrumbs, outline, page journey, mobile menu, page finder, overflow, and
semantic landmarks. The exact pixels are machine-local observations, not a
performance or rendering benchmark.

## Current route and chrome matrix

| Surface | Current layout | Sidebar | Breadcrumbs | Outline / journey | Disposition |
| --- | --- | --- | --- | --- | --- |
| Home `/` | `home` | No | No | No | Keep as one of two sidebar-free pages |
| Docs `/docs/` | `section` | **No** | Yes | No | Add Docs sidebar; keep section landing body |
| Docs article | `docs` | Yes | Yes | Yes | Keep and refine |
| Examples | mounted `docs` | Yes | Yes | Per page | Keep, but reduce generated-catalog warning noise |
| Rocdown `/rocdown/` | `product` | **No** | **No** | No | Add Rocdown sidebar and breadcrumbs |
| Rocdown article | `docs` | Yes | Yes | Yes | Keep and refine |
| Project `/project/` | `section` | **No** | Yes | No | Add Project sidebar; deduplicate breadcrumb |
| Project article | `plain` | **No** | **No** | No | Move into navigated article frame |
| FAQ `/faq/` | `plain` | No | No | No | Keep sidebar-free; add breadcrumb and inline contents |
| News | `news-index` / `news-post` | No | No | No | Remove from rocci.dev after URL/content disposition |
| Generated 404 | `not-found` | No | No | No | Add recovery sidebar and breadcrumb/recovery context |

The bold cells are direct violations of the requested navigation contract.
The catalog already constructs a per-page `sidebar` and `breadcrumbs`; the
theme discards those fields in most named layouts.[^catalog][^planner][^layouts]

## What is working well

### W-01 — The catalog/theme boundary is correct

Rust resolves routes, navigation, breadcrumbs, previous/next links, headings,
and page data; Rocci owns visible composition. `PageView` already carries the
information needed for a consistent shell. The improvement should reuse this
boundary, not add route discovery to templates or interpret the theme in
Rust.[^catalog][^planner][^page-view]

### W-02 — Detailed documentation pages have a useful reading model

At wide widths, detailed pages combine a collapsible section tree, readable
article column, on-page outline, and previous/next journey. The active page and
active group are visible. Below the phone breakpoint, the no-JavaScript
`details` menu restores the Docs sidebar and outline.[^site-shell][^layouts]

### W-03 — Accessibility foundations are present

The shell uses a skip link, `header`, `nav`, `main`, and `footer` landmarks;
navigation uses real anchors; the mobile menu uses native `details`; and the
base theme supplies visible keyboard focus. Those foundations should survive
the navigation refactor.[^site-shell][^layouts]

### W-04 — One catalog supports authored and mounted content

The site combines `site/`, the canonical `docs/` manual, and generated example
documentation without copying prose into the shell. That is the right authoring
and ownership model even though its orchestration and warning policy need
work.[^site-config][^root-readme]

## Prioritized findings

### F-01 — The landing page loses its primary path cards

**Severity:** P0 functional UX regression.

The landing source places a four-card path selector between the proposition
and the live island. The built live page jumps directly from the introductory
paragraph to “Try a live island”; none of the Getting started, Rocdown, News,
or Project cards are present. Hybrid page planning uses one article HTML file
instead of the static planned widget forest, so the authored `:card-grid` is
not represented in the output.[^landing][^planner]

This removes the landing page's only scannable next-step navigation and leaves
an implementation demo as the dominant action. Add a regression fixture that
combines a static article block before or after an island and asserts both the
block HTML and island HTML survive.

### F-02 — Go to page is not a modal and expands the document

**Severity:** P0 navigation regression.

At 390 by 844, opening “Go to page” produced a `rocci-goto` host with computed
`position: static`, `display: inline`, and a height over 2700 pixels. The page
remained visible and scrollable above it. The shared script installs external
fixed-position host CSS, but its shadow stylesheet begins with
`:host { all: initial; ... }`, which resets the host's position and display.
The result affects the shared Rocdown/rocci.dev navigation asset, not only this
site.[^goto]

The fix needs focused DOM/browser coverage for fixed positioning, viewport
bounds, focus entry, Escape/backdrop close, and body-scroll behavior at phone
and desktop widths.

### F-03 — Sidebar presence is an accidental layout side effect

**Severity:** P0 against the requested UX contract.

`Layouts.Docs` is the only desktop layout that renders `view.sidebar`.
`SiteShell` independently repeats a layout-name match to decide whether the
mobile menu receives the sidebar. `product`, `section`, `plain`, both News
layouts, `not-found`, and `home` all suppress it.[^site-shell][^layouts]

The theme therefore makes “plain article” mean “no navigation,” even when the
catalog has a complete current-section sidebar. Authors must remember a hidden
chrome consequence whenever they select a visual layout. The Docs and Project
indexes visibly fall out of their own section navigation, while mobile Project
pages show only global lanes.

Replace this with one navigated frame used by every layout except explicit
`home` and `faq`. Desktop and mobile must consume the same sidebar-presence
decision.

### F-04 — Breadcrumb data is widespread but presentation is incomplete and noisy

**Severity:** P1 orientation and information-architecture defect.

The catalog constructs breadcrumbs for every listed page, but only `Section`
and `Docs` render them. The home page's long metadata title becomes the first
crumb, top-level lane labels are not consistently represented, and adjacent
section/page labels can duplicate each other. Observed examples included a
long “Rocci · Roc-native interfaces & Markdown-first documents” root and
“Project / Project.” Detailed Docs paths omit the “Docs” lane and read as
Home / Start / Install.[^catalog][^landing][^layouts]

The target is stable and compact: `Rocci / Docs / Start / Install Rocci`,
`Rocci / Rocdown / Rocdown language reference`, and `Rocci / Project`.
Use the site title for the root, include the lane, omit adjacent duplicates,
and render the current page as current text rather than a redundant self-link.

### F-05 — News is cross-cutting site chrome and is now removal work

**Severity:** P1 product-scope and maintenance finding.

News is not isolated to four Rocdown files. It appears in global navigation,
the home path cards, the home “Latest Updates” query, feed autodiscovery, two
named layouts, collection-specific presentation, public layout documentation,
and generated feed behavior.[^site-config][^site-shell][^layouts][^landing]

Mark the rocci.dev News surface for removal. First decide whether each public
URL redirects to a durable Docs/Project owner or deliberately returns 410.
Then remove the lane, landing references, feed discovery, News layouts, News
content, and site-specific collection query together. Preserve Rocdown's
generic typed collections and Atom support; this is a site information-
architecture change, not a compiler feature rollback.

### F-06 — Public metadata and status copy reduce trust

**Severity:** P1 content correctness and discoverability finding.

The shell always appends ` · Rocci` to `view.title`, while the FAQ and News
index metadata already contain that suffix. Generated titles therefore include
“Frequently Asked Questions · Rocci · Rocci” and “News & Releases · Rocci ·
Rocci.” The home title duplicates the brand in a different order.[^site-shell][^faq][^landing]

The Project status page still names `@on`, while current public commands and
source use semantic handlers. The FAQ makes broad unqualified payload,
performance, determinism, and cross-platform statements; some are stronger
than the root README's current packaging boundary. Public status should link
to canonical documentation and use measured or bounded wording.[^faq][^project-status][^root-readme]

Separate `page_title` from `site_title`, define one title composition rule,
and run a content truth pass against current README/code before launch.

### F-07 — Tablet and mobile navigation have an avoidable gap

**Severity:** P1 responsive UX finding.

The right outline disappears at `64rem`, but the mobile menu does not appear
until `48rem`. Between those breakpoints, detailed pages lose “On this page”
without a replacement. On phones, the Docs mobile panel places all global
lanes and the full long Docs tree before the page outline, so local headings
can be several screens away.[^site-shell][^layouts]

Keep the left section navigation at tablet width, add a compact inline or
collapsible page outline when the right rail disappears, and label/reorder the
phone panel so “In this section” and “On this page” are directly reachable.

### F-08 — The landing page speaks to maintainers before users

**Severity:** P1 first-contact UX finding.

The opening paragraph describes `site/`, `site/theme/`, and `rocdown run site`
before explaining which visitor problem Rocci solves. With the path cards
missing, the live counter and implementation details dominate the first
screen. The current home also reserves substantial space for Latest Updates,
which becomes dead product structure when News is removed.[^landing][^layouts]

Reorder the page around outcome, audience paths, a minimal code-to-result
example, explicit maturity, and one primary next action. Keep the live island
as proof after the first-use paths, not as the sole centerpiece. Replace Latest
Updates with durable current-status or release links only if there is a real
maintained source.

### F-09 — Layout and navigation policy is duplicated in author-facing code

**Severity:** P1 authoring DX finding.

The desktop layout dispatch and mobile navigation allowlist repeat the same
layout strings. Page authors choose among `home`, `product`, `section`, `docs`,
`news-index`, `news-post`, `plain`, and `not-found`, but these names mix body
shape, content type, and chrome policy. The same page can have catalog sidebar
data yet silently hide it.[^site-shell][^layouts][^page-view]

Keep specialized body composition where it provides value, but route all
non-home/non-FAQ pages through one `NavigatedFrame`. Give FAQ an explicit
layout rather than overloading `plain`. Add a site fixture that asserts the
sidebar/breadcrumb contract for every named layout.

### F-10 — Site checks are noisy and the complete build is not one step

**Severity:** P2 authoring and operator DX finding.

The current `check site` succeeds but emits 54 `RD2202` warnings for mounted
generated example and source pages that are intentionally not listed in the
single top-level Examples navigation entry. This makes real catalog warnings
harder to notice. The root workflow also requires `rocci-docs` to stage
`dist/example-docs` before the public site build.[^site-config][^root-readme]

Define an explicit discoverability policy for generated detail/source pages
instead of using a warning as normal state. Provide one supported repository
command that stages example docs, checks links/navigation, builds the site,
and reports each phase distinctly.

### F-11 — The FAQ needs navigation even though it should not have a sidebar

**Severity:** P2 findability finding.

FAQ is correctly a sidebar-free reading surface under the requested contract,
but it contains multiple topic groups and question headings with no local
contents. It also has no breadcrumb, so its only orientation is the global
header.[^faq][^layouts]

Keep FAQ single-column, render `Rocci / FAQ`, and add an inline, collapsible
question index derived from the existing heading outline. Do not reintroduce a
left section sidebar.

## Desired navigation contract

| Page class | Section sidebar | Breadcrumbs | Page outline | Previous/next |
| --- | --- | --- | --- | --- |
| Landing | No | No | No | No |
| FAQ | No | Yes | Inline/collapsible | No |
| Section or product index | Yes | Yes | Optional by heading count | Optional |
| Documentation/article/detail | Yes | Yes | Right rail; responsive fallback | Yes when ordered |
| Generated example/source | Yes | Yes | When useful | Within example journey if defined |
| 404 | Recovery navigation | Recovery context | No | No |

“Sidebar” here means the persistent section navigation. An inline FAQ contents
control and the right-side on-page outline are separate orientation aids.

## News removal inventory

Before deleting files, account for:

1. `[[nav]] News` and its page IDs in `site/rocdown.toml`.
2. `site/news/*.rocdown` and their public routes.
3. The landing `:link-card` and `Home` layout's Latest Updates section.
4. Feed autodiscovery in `SiteShell` and `/news/feed.xml` expectations.
5. `NewsIndex` / `NewsPost` components and their CSS.
6. Public Rocdown documentation listing News layouts.
7. Site-level collection query and content metadata.
8. `pages.json`, sitemap, redirects, and any external links after rebuild.

Generic collection sorting, feed generation, registry code, and crate tests
remain unless a separate product decision removes those Rocdown capabilities.

## Recommended order

1. Fix F-01 and F-02 so the current primary navigation mechanisms work.
2. Establish the shared navigated frame and breadcrumb contract (F-03, F-04,
   F-07, F-09, F-11).
3. Remove News with explicit URL dispositions (F-05).
4. Rewrite landing, FAQ, titles, and public status against current evidence
   (F-06, F-08).
5. Reduce warning noise and make the complete site workflow one command
   (F-10).

The implementation sequence and exit gates are in the linked
[rocci.dev UX and authoring improvement plan](/plans/site/rocci-dev-site.md).

## Remediation verification — 2026-08-22

All P0/P1 findings in this audit are closed in the local revision. The finder
is a bounded modal with focus restoration; hybrid planning preserves authored
static widgets and islands; every non-Home/non-FAQ route uses the shared
navigated frame; breadcrumbs follow the approved hierarchy; and the retired
News surface has explicit redirects or terminal `410` responses.

Browser verification covered 390, 768, 900, 1024, and 1280 CSS-pixel widths.
Representative Docs pages had one `h1`, named landmarks, current-state
annotations, no horizontal overflow, the expected mobile/sidebar/outline
transition, and 44 CSS-pixel mobile controls. Home retained all four path cards
before its proof and maturity sections. FAQ exposed six linked questions.
The finder moved focus into its shadow-root search input, closed with Escape,
restored focus to its opener, and restored body scrolling. The theme includes
light/dark color-scheme support plus explicit forced-colors, reduced-motion,
and print rules.

Two consecutive complete site builds produced the same aggregate SHA-256,
`86d3c8b12a2dbc64485db22e58a117fa88878554ad0d839b2697a104b087d28e`.
The built tree contained 140 catalog pages, parseable sitemap XML, canonical
metadata and one `h1` on representative Home/FAQ/Docs/generated-detail routes,
the generated 404, and no News page or feed. Production Caddy policy retains
the three approved `308` redirects and exact `/news/` and `/news/feed.xml`
`410` responses.

Local release gates passed: `uv run rocci-ops site`, `uv run rocci-ops ci
lint`, `cargo test --workspace`, and the OKF profile check. OKF emitted only
pre-existing lifecycle/provenance warnings. Workspace verification also
corrected stale integration expectations for empty SSE command responses and
the current standalone Docs title/route fixture. No remote run exists for the
unpublished `rocci-dev-site` branch; CI, Knowledge, and site workflow status
for the final revision remains a post-push launch gate because the phase runner
forbids pushing.

[^site-config]: Current global lanes, per-lane page lists, mounted Docs/examples catalogs, News entries, and build inputs.
[^site-shell]: Current title composition, feed discovery, header, duplicated mobile layout match, skip link, and responsive menu.
[^layouts]: Current layout-specific rendering of sidebar, breadcrumbs, outline, journey, News, and responsive breakpoints.
[^catalog]: Current breadcrumb construction and ordered journey resolution.
[^planner]: Current per-page sidebar projection, view construction, hybrid article planning, and generated 404 view.
[^page-view]: Shared normalized fields already available to the theme.
[^goto]: Shared page-finder host, shadow stylesheet, catalog loading, focus, and navigation implementation.
[^landing]: Current home metadata, visitor copy, authored path cards, live island, and News link.
[^faq]: Current FAQ structure, metadata title, claims, and next-step links.
[^project-status]: Current public status vocabulary and adoption links.
[^root-readme]: Current public site staging, build, packaging, and platform boundaries.
[^prior-plan]: Exploratory implementation plan revised from this audit and the requested navigation/removal contract.
