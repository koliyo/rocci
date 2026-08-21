---
type: Implementation Plan
title: rocci.dev UX and authoring improvements
description: Repair the landing and page finder, make section sidebar navigation universal except on Home and FAQ, make breadcrumbs consistent, remove the rocci.dev News surface, improve public copy, and reduce site-authoring friction without changing the Rocdown ownership boundary.
tags: [domain/rocci, domain/rocdown, concern/ux, concern/tooling, concern/publication, concern/navigation, concern/accessibility]
status: draft
generated: { by: process:cursor, at: 2026-08-21T21:53:49Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: audit
    resource: ../audits/rocci-dev-site-ux-dx.md
    title: rocci.dev site UX and authoring DX review
    author: process:cursor
    last_modified: 2026-08-21
  - id: root-readme
    resource: ../../README.md
    title: Current workspace and public-site workflow
    author: human:nils
    last_modified: 2026-08-21
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Shipped Rocdown format, theme, mount, and site behavior
    author: process:git
    last_modified: 2026-08-21
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-17
  - id: catalog-decision
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci shell decision
    author: process:cursor
    last_modified: 2026-08-18
  - id: config
    resource: ../../site/rocdown.toml
    title: Current rocci.dev catalog, mounts, and navigation
    author: human:nils
    last_modified: 2026-08-21
  - id: shell
    resource: ../../site/theme/SiteShell.rocci
    title: Current rocci.dev site shell
    author: process:git
    last_modified: 2026-08-21
  - id: layouts
    resource: ../../site/theme/Layouts.rocci
    title: Current rocci.dev named layouts
    author: process:git
    last_modified: 2026-08-21
  - id: catalog
    resource: ../../crates/rocci-rocdown/src/catalog.rs
    title: Current navigation and breadcrumb resolution
    author: process:git
    last_modified: 2026-08-21
  - id: planner
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Current page planning and normalized view construction
    author: process:git
    last_modified: 2026-08-21
  - id: goto
    resource: ../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette
    author: process:git
    last_modified: 2026-08-21
  - id: docs-plan
    resource: comprehensive-rocci-documentation.md
    title: Comprehensive Rocci documentation plan
    author: process:cursor
    last_modified: 2026-08-21
---

# rocci.dev UX and authoring improvements

## Goal

Make rocci.dev easy to enter, orient within, and author by establishing one
visible navigation contract across the existing catalog:

- Home and FAQ are the only pages without a persistent section sidebar.
- Every non-home page has compact, correctly ordered breadcrumbs.
- Every other page, including section indexes, product indexes, Project
  articles, generated example/source pages, and 404 recovery, has useful
  section navigation.
- Page outlines remain reachable at desktop, tablet, and phone widths.
- The landing page presents a clear path to first success before interactive
  proof or repository implementation detail.
- News is removed from rocci.dev after an explicit URL and content disposition.
- Authors get a smaller layout contract, low-noise checks, and one supported
  command for the complete site pipeline.

The evidence and prioritized defects are recorded separately in the
[site UX/DX audit](../audits/rocci-dev-site-ux-dx.md).[^audit]

## Out of bound

- A visual-identity or logo redesign.
- A wholesale rewrite of the Rocci manual; that remains in the comprehensive
  documentation plan.[^docs-plan]
- Full-text search, localization, analytics, personalization, pagination, or a
  client application router.
- A new `.rocdown` grammar form merely to control site chrome.
- A second `rocci-site` catalog, renderer, configuration format, or CLI.
- Removal of Rocdown's generic collections, deterministic sorting, or Atom
  generation. Only the rocci.dev News product surface is being removed.
- Deployment-provider, VPS, Cloudflare, or release-channel changes.
- Executing any phase as part of writing this plan.

## Constraints that do not move

1. Rust continues to own discovery, routes, aliases, catalog validation,
   navigation, breadcrumbs, headings, collections, and artifacts. Rocci owns
   visible shell and layout composition. Rocdown remains the Markdown-first
   content owner.[^product-boundary][^catalog-decision]
2. `docs/` remains the canonical Rocci manual and is mounted into `site/`;
   generated application docs remain a separate staged mount. Do not duplicate
   their prose into `site/`.[^config][^root-readme]
3. Navigation uses real links and works without client JavaScript. The page
   finder is an enhancement, not the only path to content.
4. Static content and site metadata stay inspectable without evaluating
   arbitrary Roc. Presentation changes belong in the project theme unless a
   demonstrated shared defect is in `rocci-ui` or Rocdown planning.
5. Failed site builds preserve the previous output tree.[^rocdown-readme]
6. The existing user modification to `site/index.rocdown` is preserved and
   reconciled deliberately when the landing-content phase begins.
7. Public behavior claims are verified against current code, tests, or current
   package documentation; exploratory plans do not become public “working
   today” copy.

## Target information architecture

After News removal, global navigation is:

| Lane | Canonical entry | Sidebar contents |
| --- | --- | --- |
| Docs | `/docs/` | Start, Tutorials, How to, Understand, Reference, Troubleshooting, Status |
| Examples | `/examples/` | Example catalog and the current example context |
| Rocdown | `/rocdown/` | Overview, Pages, Article blocks, Hybrid, Language, Site config, CLI, Tree |
| FAQ | `/faq/` | None; inline question index only |
| Project | `/project/` | Overview, Status, Roadmap, Contributing |

Home remains reachable through the brand, skip-link target, breadcrumbs, and
page finder. It is not duplicated as a global lane.

### News URL disposition

Do not blanket-redirect every News URL to Home. Before deletion, assign each
route one of:

- **Move and redirect:** retain durable technical content under its canonical
  Docs, Rocdown, or Project owner and emit a permanent redirect.
- **Fold and redirect:** merge only still-accurate facts into an existing
  canonical page, then redirect to that page.
- **Retire:** emit 410 when the announcement has no durable replacement.

The feed route `/news/feed.xml` is retired with the collection. Remove feed
autodiscovery at the same time. Sitemap and `pages.json` must agree with the
chosen redirects/retirements.

## Target page-chrome contract

| Page class | Left section navigation | Breadcrumbs | On-page outline | Journey |
| --- | --- | --- | --- | --- |
| Home | No | No | No | No |
| FAQ | No | `Rocci / FAQ` | Inline/collapsible question list | No |
| Section/product index | Yes | Root / lane / page | When headings warrant | Optional |
| Article/reference/guide | Yes | Root / lane / group / page | Wide right rail, responsive fallback | Previous/next when ordered |
| Generated example/source | Yes | Root / Examples / example / page | When headings warrant | Within defined example order |
| 404 | Recovery navigation | Recovery context | No | No |

The section sidebar and page outline are different controls. “Only Home and
FAQ without sidebar” refers to the persistent left section navigation; FAQ's
inline question index is still required.

## Breadcrumb contract

Breadcrumbs are catalog data rendered consistently, not handwritten in each
Rocdown page.

1. Use the short site title `Rocci` for the root crumb, never the home page's
   SEO title.
2. Include the active global lane.
3. Include a nested group such as Start only when it adds hierarchy.
4. Remove adjacent crumbs with the same normalized title or destination.
5. Render the current page with `aria-current="page"` as text or a
   non-redundant current element, not an ordinary self-navigation action.
6. Home renders no breadcrumbs. FAQ renders `Rocci / FAQ` despite omitting the
   section sidebar.

Examples:

```text
Rocci / Docs / Start / Install Rocci
Rocci / Rocdown / Rocdown language reference
Rocci / Project / Project status
Rocci / FAQ
```

The existing catalog currently derives home and section crumbs from listed
page titles and group labels, so the root label, lane inclusion, and duplicate
suppression need focused catalog/planner tests.[^catalog][^planner]

## Theme composition

### One navigated frame

Create one `NavigatedFrame` (name provisional) in `site/theme/Layouts.rocci`.
It owns:

- left `NavList`;
- breadcrumb row;
- readable content column;
- optional right `PageOutline`;
- optional previous/next journey;
- responsive substitutions.

`Docs`, `Section`, `Product`, ordinary Project articles, Example pages, and
Not Found reuse that frame with body-specific options. `Home` and `Faq` are
the only independent frames. `plain` must not continue to mean “silently omit
navigation.”[^layouts]

### One responsive decision

`SiteShell` must not repeat an allowlist of layout names to decide whether the
mobile menu gets navigation. The same semantic decision that renders the
desktop sidebar supplies “In this section” in the mobile menu. Prefer a small
Roc helper or a frame-owned component over two drifting `match` blocks.[^shell]

At widths where the right outline is removed but the phone menu has not yet
appeared, render a compact inline `details` outline. On phones, label and order
the panel as global Sections, In this section, and On this page; keep the
current group expanded and targets at least 44 CSS pixels high.

### 404 recovery

The generated 404 currently receives no useful current section. Give it a
bounded recovery sidebar, such as the global lane entries plus Docs Start and
Project Status, without pretending it has a current catalog page. This needs a
focused `not_found_page` view test in the planner rather than filesystem logic
in the theme.[^planner]

## Landing-page direction

The landing page should answer, in this order:

1. What Rocci is and the user outcome it improves.
2. Whether the visitor wants to build a Rocci app, write a Rocdown site,
   inspect examples, or evaluate project maturity.
3. What a minimal authored input and rendered result feel like.
4. Why the architecture is different: pure Roc views, HTML/CSS, server-owned
   state, Datastar as transport, and a small desktop preview.
5. What is experimental and what works now.
6. A live island as proof, after the primary paths.

Restore the authored path cards by fixing the hybrid block/island planning
regression first. Remove the News card and Latest Updates section. Do not
replace them with an unmaintained pseudo-changelog. A small “Current status”
link to Project Status is sufficient until a release source exists.[^audit]

## FAQ direction

- Give FAQ the explicit `faq` layout and keep it single-column.
- Add `Rocci / FAQ` breadcrumbs.
- Generate an inline collapsed question index from h2/h3 outline data.
- Shorten answers and link to canonical Docs/Project owners.
- Replace broad comparisons and performance claims with bounded, sourced
  language.
- Remove the News/feed question when the rocci.dev News surface is retired;
  generic collection support belongs in Rocdown reference documentation.

## Authoring and operator DX

### Layout vocabulary

The target public vocabulary is about body intent, not navigation side
effects:

- `home`: unique landing page, no sidebar.
- `faq`: unique FAQ, no sidebar, inline outline.
- `docs`: navigated article/reference/guide.
- `section`: navigated section index.
- `product`: navigated product overview when its body truly differs.
- `not-found`: navigated recovery page.

Remove `news-index` and `news-post`. Either retire `plain` or redefine it as a
navigated ordinary article; do not leave it as a third sidebar-free escape.
Document each layout with its chrome contract.[^rocdown-readme]

### Discoverability policy

Mounted generated example/source pages are legitimate destinations but produce
54 unlisted-page warnings in the current check. Choose one explicit model:

1. generate an Examples sidebar with nested example/source context; or
2. add a validated “linked detail, intentionally omitted from global nav”
   policy that suppresses `RD2202` only for declared generated detail pages.

Prefer model 1 where the resulting sidebar remains usable. If source trees are
too large, use model 2 with a visible local example navigator. Never globally
disable unlisted-page warnings.[^audit]

### One repository command

Provide one supported command, probably under `rocci-ops`, that:

1. stages application documentation;
2. checks Docs and site catalogs;
3. runs declared documentation tests when requested;
4. builds or packages rocci.dev;
5. reports warnings by source class;
6. leaves deployment as a separate authorized operation.

Direct `rocci-docs` and `rocdown` commands remain documented for focused
iteration. The wrapper is orchestration, not a fourth product CLI.[^root-readme]

## Phased implementation

### Phase 0 — freeze routes, chrome, and evidence fixtures

#### Bound

- Approve the post-News lane map and URL disposition categories.
- Approve the page-chrome and breadcrumb matrices above.
- Record representative fixtures/routes for Home, FAQ, Docs index, Docs
  detail, Rocdown index, Project detail, Example source, and 404.
- Capture DOM assertions for sidebar, breadcrumbs, outline fallback, page
  title, and News absence.
- Do not change site content or delete News in this phase.

#### Exit

Exit when every representative route has one canonical page class, expected
sidebar state, breadcrumb sequence, outline behavior at 1280/900/390 CSS
pixels, and News URL disposition.

### Phase 1 — repair the two navigation regressions

#### Bound

- Fix hybrid page planning so static article blocks and Rocci islands coexist
  in authored order.
- Add the lowest-boundary Rocdown planner/build regression test using a
  `:card-grid` plus island fixture.
- Fix the shared `rocci-goto` host so the opened palette is fixed to the
  viewport and does not extend document flow.[^goto]
- Add focused shared-asset browser/DOM coverage for open, initial focus,
  Escape, backdrop close, result bounds, and phone/desktop geometry.
- Verify the Home path cards render before changing landing copy.

#### Exit

Exit when the built Home contains every authored path card and its live island,
and the page finder is a bounded modal at phone and desktop widths with no
horizontal or document-height regression.

### Phase 2 — unify sidebar, breadcrumbs, outline, and journey chrome

#### Bound

- Add the shared navigated frame and explicit FAQ frame.
- Route `section`, `product`, ordinary articles, generated example pages, and
  404 through the shared navigation policy.
- Render the sidebar on every page except Home and FAQ.
- Correct breadcrumb root/lane/group construction and suppress duplicates.
- Add tablet outline fallback and labeled phone navigation sections.
- Remove the duplicated desktop/mobile layout allowlist.
- Add planner/catalog tests and built-site DOM assertions for the Phase 0
  matrix.

#### Exit

Exit when Home and FAQ are the only sidebar-free routes in the fixture matrix;
all non-home routes have exact expected breadcrumbs; current pages are visible
in desktop and mobile section navigation; and page headings remain reachable
at all three target widths without JavaScript.

### Phase 3 — remove News from rocci.dev

#### Bound

- Apply the approved move/fold/retire disposition to every `/news/` URL.
- Remove News from `site/rocdown.toml`, Home cards, Home layout, header feed
  discovery, named layouts, theme CSS, source pages, site docs, sitemap, and
  page-finder index.
- Remove the site-level News collection query and feed output.
- Preserve and test generic Rocdown collection/feed behavior in the owning
  crate.
- Add redirect/410 and no-stale-link checks.

#### Exit

Exit when the built site has no News lane, home promotion, News layout, feed
autodiscovery, or accidental News page; every former public route has its
approved response; and generic Rocdown collection tests still pass.

### Phase 4 — rewrite first contact and public trust surfaces

#### Bound

- Rework Home in the agreed outcome/path/proof/maturity order.
- Rework FAQ into concise answers with an inline question index and canonical
  links.
- Fix document-title composition so the brand appears once.
- Reconcile Project Status and Roadmap with current code, README, and canonical
  knowledge; remove stale handler and phase language.
- Verify every quantified or cross-platform claim or replace it with bounded
  wording.
- Preserve stable routes and aliases.

#### Exit

Exit when a first-time visitor can choose a task from the first screen, no
generated title repeats `Rocci`, FAQ questions deep-link and point to canonical
owners, and public status terms match current implementation evidence.

### Phase 5 — reduce authoring noise and unify the local workflow

#### Bound

- Implement the chosen generated-page discoverability policy.
- Reduce `check site` to zero expected `RD2202` warnings; real unlisted pages
  must still warn.
- Add the one-command staging/check/build wrapper without hiding focused CLI
  commands.
- Document layout contracts, navigation ownership, News removal, and the full
  local validation matrix in the owning README/public site reference.
- Add failure tests that preserve the previous output tree.

#### Exit

Exit when a clean checkout can produce the complete local site with one
documented command, expected generated pages no longer create warning noise,
and a deliberately unlisted authored page still fails or warns according to
policy.

### Phase 6 — accessibility, responsive, and release verification

#### Bound

- Run keyboard-only navigation through skip link, global lanes, sidebar,
  breadcrumbs, page outline, journey, mobile menu, and Go to page.
- Test 390, 768, 900/1024, and 1280 CSS-pixel widths; light, dark, forced
  colors, reduced motion, and print.
- Check semantic landmarks, one h1, accessible names, `aria-current`, focus
  restoration, no horizontal overflow, and target sizes.
- Run site check/build twice and compare deterministic static artifacts where
  expected; separate the live island binary/hash from static equality.
- Validate redirects/410s, sitemap, pages index, canonical metadata, and 404.

#### Exit

Exit when the matrix passes with recorded evidence, no P0/P1 audit finding is
open, site checks have no unexplained warnings, and required CI/Knowledge/site
workflows are green for the revision. Human first-use feedback remains a
separate launch gate and is not invented from automated tests.

## Validation commands

Use the narrowest checks while iterating, then the integrated gates:

```sh
cargo test -p rocci-ui
cargo test -p rocci-rocdown
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- check site
cargo run -q -p rocci-rocdown-cli -- build site
cargo fmt --all -- --check
cargo test --workspace
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

For theme changes, inspect the built representative routes rather than only
the generated source. Failed static builds must leave the previous output tree
in place.[^rocdown-readme]

## Acceptance criteria

- Only `/` and `/faq/` omit the persistent section sidebar.
- Every non-home page has breadcrumbs matching the approved hierarchy with no
  duplicated site or section title.
- FAQ has an inline question index; detailed pages retain an outline at wide,
  tablet, and phone widths.
- The Home path cards and live island both render in authored order.
- Go to page is a bounded, keyboard-operable modal and does not expand the
  document.
- News content, global navigation, layouts, feed discovery, and site-specific
  collection output are removed with explicit URL dispositions.
- Rocdown's generic collection and Atom behavior remains tested and documented.
- Page titles include the Rocci brand exactly once.
- Public status and comparison claims are current, bounded, and linked to
  canonical owners.
- Generated example/source pages are discoverable without 54 expected
  unlisted warnings.
- One documented repository command stages, checks, and builds the complete
  site; focused product commands remain available.
- The site remains usable without JavaScript; JS enhances Go to page and live
  islands without becoming the only navigation path.

## Decision gates

Human approval is required before implementation treats these exploratory
choices as normative:

1. The exact redirect/410 disposition for each current News URL.
2. Whether `plain` is removed or redefined as a navigated article layout.
3. Whether generated source pages receive nested Example sidebar entries or an
   explicit linked-detail visibility class.
4. Final Home messaging and primary call to action.
5. Whether 404 uses global lane recovery only or also a small curated subset
   of Docs/Project links.

The requested sidebar, breadcrumb, and News-removal direction is already an
input to this plan; the gates above refine execution without reopening that
direction.

[^audit]: Current source/build/browser evidence, route matrix, prioritized findings, and News-removal inventory.
[^root-readme]: Current two-step application-doc staging plus site build/package workflow and public product/platform boundaries.
[^rocdown-readme]: Current Rocdown project themes, mounts, page building, responsive-menu requirement, atomic output, and CLI contract.
[^product-boundary]: Approved one-way ownership between base Rocci and the Rocdown product.
[^catalog-decision]: Approved Rust-catalog/Rocci-shell split and shared chrome boundary.
[^config]: Current source mounts, global lanes, curated navigation, News pages, and live service.
[^shell]: Current global document composition, title rule, feed link, desktop lanes, and mobile layout allowlist.
[^layouts]: Current layout-specific sidebar/breadcrumb/outline/journey behavior and responsive breakpoints.
[^catalog]: Current breadcrumbs and ordered journey derive from the resolved navigation tree.
[^planner]: Current sidebar groups, page views, hybrid plan path, and empty 404 navigation data.
[^goto]: Current shared palette styling, host mounting, focus, results, and History API navigation.
[^docs-plan]: Separate exhaustive Rocci documentation coverage, learning paths, reference, and first-use measurement work.
