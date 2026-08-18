---
type: Implementation Plan
title: rocci.dev site architecture and Rocdown evolution
description: Plan the main rocci.dev site, its route and layout structure, the division between Rocdown content and Rocci presentation, and the product boundary for future site-building capabilities.
tags: [domain/rocci, domain/rocdown, concern/architecture, concern/publication, concern/rendering]
status: draft
generated: { by: process:codex, at: 2026-08-18T10:07:17Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview and current rocci.dev build
    author: human:nils
    last_modified: 2026-08-17
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Shipped Rocdown format and site behavior
    author: process:git
    last_modified: 2026-08-17
  - id: generator
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:codex
    last_modified: 2026-08-17
  - id: format
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-17
  - id: shell
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Current Rocci-authored site shell
    author: process:git
    last_modified: 2026-08-17
  - id: config
    resource: ../../crates/rocci-rocdown/src/config.rs
    title: Current Rocdown site configuration
    author: process:git
    last_modified: 2026-08-17
  - id: current-site
    resource: ../../docs/rocdown.toml
    title: Current rocci.dev site configuration
    author: human:nils
    last_modified: 2026-08-17
  - id: preview-plan
    resource: public-preview-community.md
    title: Rocci public-preview branding and community plan
    author: process:codex
    last_modified: 2026-08-17
---

# rocci.dev site architecture and Rocdown evolution

## Goal and scope

Build `rocci.dev` as the first complete public site made with Rocci and
Rocdown, while using that work to establish a reusable boundary for other
content-first sites. The site should combine a distinct product landing page,
technical documentation, news, FAQ, and project information in one static,
accessible, inspectable build.

This plan covers information architecture, source structure, layouts,
rendering ownership, metadata, collections, generated artifacts, and phased
implementation. It deliberately does not choose final copy, documentation
coverage, news topics, visual identity, or launch messaging. Those remain in a
separate content and public-preview process.[^preview-plan]

## Established baseline

The repository already treats `docs/` plus `docs/rocdown.toml` as the source of
the publishable `rocci.dev` tree. `rocdown build` discovers pages, resolves
routes and links, renders Markdown, hashes assets, compiles the Rocci shell,
and emits the static artifact set.[^root-readme][^current-site]

The shipped ownership boundary is sound and should remain: Rust owns catalog,
navigation, route, graph, validation, rendering-data, and artifact work;
Rocci owns the visible shell; and Rocdown owns the complete document and
static-site product above base Rocci.[^generator][^product-boundary][^shell]

Two current limitations shape the plan. Static site pages reject authored
Roc/Rocci islands and project layouts, and Rocdown does not yet implement
content collections. The existing site configuration also exposes one site
shell and documentation-oriented navigation rather than named project-local
layouts.[^rocdown-readme][^generator][^format][^config]

## Recommendation

1. **Keep one site and one build.** Landing pages, docs, news, FAQ, and project
   pages should share one Rocdown catalog, route graph, asset pipeline,
   canonical metadata policy, and static output tree.
2. **Evolve Rocdown from a documentation generator into an opinionated
   content-first site builder.** Documentation remains its strongest preset,
   but named layouts, typed page metadata, and collections should be general
   enough for product sites, blogs/news, changelogs, reports, and knowledge
   portals.
3. **Use project-local `.rocci` files for presentation.** They define the site
   shell, named layouts, and reusable visual components from normalized view
   records. They do not discover files, query collections, resolve routes, or
   write artifacts.
4. **Use `.rocdown` for every authored content page.** Markdown remains the
   main body format. `@page` supplies statically extractable metadata, while
   bounded Rocdown components express semantic content patterns.
5. **Do not create a separate `rocci-site` engine now.** If that name becomes
   useful, initially use it for a starter, scaffold, or opinionated profile
   powered by Rocdown. Split a product only after multiple real sites require
   capabilities that do not belong in a content-first Rocdown build.

## Site map and route policy

The primary navigation should stay small and task-oriented:

| Area | Canonical route | Layout | Purpose |
| --- | --- | --- | --- |
| Home | `/` | `home` | Product proposition, primary paths, proof, and current calls to action |
| Rocdown | `/rocdown/` | `product` | Focused product landing page for the document and site system |
| Docs | `/docs/` | `section` | Documentation portal and learning-path entry |
| Getting started | `/docs/getting-started/…` | `docs` | Installation, orientation, and first success |
| Guides | `/docs/guides/…` | `docs` | Task-oriented implementation guides |
| Concepts | `/docs/concepts/…` | `docs` | Architecture, rationale, and mental models |
| Reference | `/docs/reference/…` | `docs` | Rocci, Rocdown, CLI, configuration, and generated reference |
| Examples | `/docs/examples/…` | `docs` or `section` | Runnable examples and pattern discovery |
| News | `/news/` and `/news/<slug>/` | `news-index`, `news-post` | Announcements, releases, and project updates |
| FAQ | `/faq/` | `plain` | Short cross-cutting answers with links into canonical docs |
| Project | `/project/…` | `plain` or `section` | Status, roadmap, contribution, governance, and support |

Route rules:

- Prefer stable topic routes over dates in news URLs; keep publication dates in
  metadata and feeds.
- Use page IDs for internal catalog identity and canonical routes for output.
- Preserve existing public documentation routes with `@page.aliases` during a
  move under `/docs/`.
- Keep one canonical page for each fact. FAQ, landing pages, and news may
  summarize and link, but should not become competing reference sources.
- Reserve interactive tools such as a future playground as separate Rocci
  applications mounted or deployed beside the static site, not as an excuse to
  make the whole site dynamic.

## Page and layout model

Rocdown should pass a normalized page view into one project shell. The shell
selects a named layout from a closed, inspectable set:

| Layout | Chrome and slots |
| --- | --- |
| `home` | Global header/footer; full-width hero, proof, feature, code, path, and update slots; no docs sidebar |
| `product` | Global header/footer; product summary, use cases, example, capability, and next-step slots |
| `section` | Global header/footer; section introduction plus generated or authored card groups |
| `docs` | Global header plus docs sidebar, breadcrumbs, article, outline, previous/next, and article footer |
| `news-index` | Global chrome plus generated collection listing and feed discovery |
| `news-post` | Global chrome plus title, summary, publication metadata, article, related links, and feed link |
| `plain` | Global chrome plus a readable single-column article; suitable for FAQ and project pages |
| `not-found` | Global chrome plus recovery links and optional site search |

Layout selection must be static and validated. A page may request a supported
layout, but arbitrary executable layout expressions should not become a
catalog hook. The renderer should pass serializable view data to `.rocci`
rather than asking templates to read the filesystem or infer page types.

The global shell should own document structure, metadata tags, header, footer,
favicon and social resources, responsive behavior, accessibility affordances,
and the layout switch. Each layout owns only its page-shaped chrome. Reusable
components own visual patterns such as buttons, cards, code showcases, badges,
metadata rows, and link groups.

## Proposed source tree

Once project-local shells ship, rename the content root from `docs/` to
`site/` so the filesystem describes the full public site rather than only one
section:

```text
site/
├── rocdown.toml
├── index.rocdown
├── rocdown/
│   └── index.rocdown
├── docs/
│   ├── index.rocdown
│   ├── getting-started/
│   ├── guides/
│   ├── concepts/
│   ├── reference/
│   └── examples/
├── news/
│   ├── index.rocdown
│   └── <slug>.rocdown
├── faq/
│   └── index.rocdown
├── project/
│   ├── index.rocdown
│   ├── status.rocdown
│   ├── roadmap.rocdown
│   └── contributing.rocdown
├── theme/
│   ├── SiteShell.rocci
│   ├── Layouts.rocci
│   └── Components.rocci
└── assets/
    ├── brand/
    ├── images/
    └── fonts/
```

Start with a few cohesive `.rocci` modules. Split components into more files
only when their contracts and compilation boundaries are real; the site should
not become a directory of tiny markup fragments.

## Responsibility of each format

### `.rocdown`

- Authored prose, headings, tables, lists, code, images, footnotes, and links.
- Page identity, canonical route, aliases, draft state, layout name, title,
  description, and collection metadata.
- Semantic documentation components such as notes, steps, figures, examples,
  and link cards.
- Manual section introductions and editorial ordering where generated order is
  not appropriate.

Rocdown pages should not define global navigation, site chrome, filesystem
queries, collection loops, or deployment behavior.

### `.rocci`

- `SiteShell`: document root, head resources, global header/footer, and layout
  dispatch.
- `Layouts`: home, product, docs, news, plain, and error-page composition.
- `Components`: reusable typed visual primitives and site-specific content
  presentation.
- Scoped styles colocated with the component or layout that owns them.

Project `.rocci` files should compile once per build when possible. Content
changes should not turn every Markdown page into a generated Roc module.

### Rust in Rocdown

- Discovery, parsing, static metadata extraction, route and alias resolution,
  navigation, links, headings, assets, and diagnostics.
- Collection indexing, filtering, sorting, pagination, related-page selection,
  and feed data.
- Markdown and semantic-component rendering data.
- Search, sitemap, feed, `llms.txt`, redirect, 404, CSP, and atomic artifact
  planning.
- Construction of the serializable view records passed to the Rocci shell.

### `rocdown.toml`

- Site identity and default metadata.
- Build output, assets, canonical URL, CSP, and selected project shell.
- Global navigation and the docs navigation tree.
- Named layout availability and defaults.
- Collection schemas and generated artifact policy.
- Validation policy, without arbitrary executable hooks.

## Metadata and collections

Keep the existing `@page` record, but standardize a statically extractable site
subset. Conceptually, each catalog page should gain:

```text
id, route, aliases, draft
layout, content_type
title, description, short_title
published, updated, authors, tags
```

The exact syntax should be decided with the Rocdown language owner. The
important contract is that site-critical fields are compile-time literals,
validated without Roc, available to inspection and editor tooling, and not
hidden in arbitrary runtime values.

Add collections after the layout seam is stable. A `news` collection is the
first concrete requirement and should prove:

- required metadata and useful diagnostics;
- deterministic date ordering with an explicit tie-breaker;
- draft exclusion;
- a generated collection view available to `news-index` and `home`;
- RSS or Atom plus feed-discovery metadata;
- pagination only when the real corpus requires it;
- one canonical URL per post and stable aliases on change.

FAQ does not need a collection initially. One well-structured Rocdown page is
simpler until its size or reuse demonstrates a need for item-level metadata.

## Product boundary: Rocdown versus `rocci-site`

The near-term product model should be:

```text
Rocci
├── application/runtime framework and .rocci templates
└── Rocdown
    ├── .rocdown content language
    ├── content catalog and static builder
    ├── layouts, themes, and semantic components
    └── docs/news/report/knowledge-portal profiles
```

This preserves the approved one-way dependency: Rocdown uses Rocci; base Rocci
does not learn Rocdown site semantics.[^product-boundary]

`rocci-site` should not be a second catalog, renderer, CLI, or configuration
language. Reconsider the name only when at least two non-rocci.dev sites show a
repeated need for one of these boundaries:

- a scaffold and upgradeable starter with a strong product-site convention;
- non-content data sources and page generation that would distort Rocdown;
- server-rendered application routes combined with the static content graph;
- deployment adapters or a plugin lifecycle beyond static artifact generation;
- a stable set of general-site concepts that deserves a separately versioned
  user experience.

Even then, prefer a profile or package built on the Rocdown facade. A separate
engine is justified only if the ownership boundary is genuinely different.

## Phased implementation

### Phase 0 — freeze the site contract

- Approve the route map, layout names, ownership rules, and source-root move.
- Define redirects from current routes into `/docs/`.
- Define the minimal normalized view records for global, docs, landing, news,
  plain, and not-found layouts.
- Keep content and visual-identity decisions explicitly outside this phase.

Exit when every planned route has one owner, layout, canonical URL, and source
location, with no implementation required to infer the structure from prose.

### Phase 1 — add the Rocdown project-shell seam

- Let `rocdown.toml` select a project-local `.rocci` shell entry point.
- Add static named layouts without enabling arbitrary filesystem access from
  templates.
- Extend page views with layout/content type and typed layout-specific data.
- Compile shell and shared components once, preserving Rust catalog ownership.
- Add focused tests for layout selection, missing layouts, source maps,
  deterministic output, asset dependencies, and static safety.

Exit when a fixture site renders at least `home`, `docs`, `plain`, and
`not-found` from one shell without recompiling prose as Roc.

### Phase 2 — build the first complete rocci.dev structure

- Move the content root to `site/` and existing documentation under
  `site/docs/`.
- Add aliases for existing public documentation routes.
- Implement the branded shell and the initial layout set in `.rocci`.
- Add structural Rocdown pages for home, Rocdown, docs portal, news index, FAQ,
  and project portal; final copy remains a separate workstream.
- Keep the news index authored manually until collection support ships.

Exit when the full route tree builds, checks, previews, and works without
JavaScript at mobile, wide, keyboard, forced-color, dark/light, and print
baselines appropriate to each layout.

### Phase 3 — add the first typed collection

- Implement static collection metadata and catalog queries in Rocdown.
- Generate the news index data and RSS or Atom feed.
- Supply recent-news view data to the home layout.
- Add inspection output, link/feed validation, and deterministic fixtures.

Exit when adding one valid news `.rocdown` file updates the index, home view,
feed, sitemap, and machine indexes without editing a `.rocci` file.

### Phase 4 — generalize only from measured needs

- Extract a reusable starter or profile if a second site validates it.
- Add search, localization, pagination, additional feeds, or richer layout
  packages only from demonstrated requirements.
- Decide whether the user-facing name remains simply Rocdown or gains a
  `rocci-site` starter/profile; do not change the compiler boundary by naming
  alone.

## Acceptance criteria

- One command checks, builds, and previews the whole site.
- All authored content bodies are `.rocdown`; all site presentation code is
  `.rocci`; all catalog and artifact logic remains in Rocdown's Rust layer.
- A prose-only edit does not require per-page Roc compilation.
- Landing, docs, news, FAQ, project, redirect, and 404 pages share one canonical
  route graph and asset pipeline.
- Layout selection and collection metadata are statically inspectable and
  produce actionable diagnostics.
- The generated site is useful with no client JavaScript; later enhancement is
  explicit and capability-scoped.
- Failed builds preserve the previous output tree.
- `rocdown check`, internal links, aliases, sitemap, feed, `llms.txt`, and any
  future search projection consume the same resolved catalog.

## Decision gates

Human approval is required before treating any of these exploratory choices as
normative:

1. Move public documentation beneath `/docs/` rather than preserving the
   current top-level route families.
2. Rename the source root from `docs/` to `site/`.
3. Adopt the proposed named-layout and project-shell contract.
4. Standardize collection metadata in `@page`.
5. Introduce `rocci-site` as any public name, even if only for a starter.

[^root-readme]: Current workspace overview, Rocdown commands, and configured `rocci.dev` output.
[^rocdown-readme]: Shipped format, site workflow, static capabilities, and deferred project layouts and collections.
[^generator]: Implemented Rust-catalog/Rocci-shell boundary, static feature gate, artifacts, and missing collection/search/island work.
[^format]: Current language boundary and explicit statement that content collections are not implemented.
[^product-boundary]: Approved ownership symmetry and one-way dependency between base Rocci and Rocdown.
[^shell]: Current once-compiled Rocci shell and its documentation-oriented view contract.
[^config]: Current closed TOML schema for site metadata, output, assets, navigation, snippets, and examples.
[^current-site]: Existing rocci.dev metadata, build output, asset root, and curated navigation.
[^preview-plan]: Separate public-preview work for message, identity, launch readiness, and community feedback.
