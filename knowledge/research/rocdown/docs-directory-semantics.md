---
type: Research Report
title: Directory plus index is the Rocdown site section
description: >-
  Catalog lists pages; sidebar infers sections from index.rocdown. Phases
  0-6 shipped peel-by-id, reserved Overview first child, equal sibling
  indent, RD2205, and Contributor/Appendix landings.
tags: [domain/rocdown, concern/publication, concern/developer-experience, concern/navigation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T11:20:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: catalog
    resource: ../../../crates/rocci-rocdown/src/catalog.rs
    title: Rocdown catalog identity, routes, and explicit navigation
    author: process:git
    last_modified: 2026-08-23
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Sidebar forest, index peel, and nested fold
    author: process:git
    last_modified: 2026-08-31
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: Page discovery, derived ids, titles, and default layouts
    author: process:git
    last_modified: 2026-08-31
  - id: config-rs
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: NavConfig label, items, directory, and groups
    author: process:git
    last_modified: 2026-08-22
  - id: nav-list
    resource: ../../../crates/rocci-ui/templates/chrome/NavList.rocci
    title: Shared expandable sidebar renderer
    author: process:git
    last_modified: 2026-08-31
  - id: docs-nav
    resource: ../../../docs/rocdown.toml
    title: Standalone documentation navigation
    author: process:git
    last_modified: 2026-08-31
  - id: site-nav
    resource: ../../../site/rocdown.toml
    title: Unified rocci.dev navigation and docs mount
    author: process:git
    last_modified: 2026-08-31
  - id: docs-index
    resource: ../../../docs/index.rocdown
    title: Docs portal titled Overview
    author: process:git
    last_modified: 2026-08-22
  - id: templates-index
    resource: ../../../docs/templates/index.rocdown
    title: Templates section landing
    author: process:git
    last_modified: 2026-08-22
  - id: language-index
    resource: ../../../docs/reference/language/index.rocdown
    title: Rocci language reference landing
    author: process:git
    last_modified: 2026-08-25
  - id: reference-index
    resource: ../../../docs/reference/index.rocdown
    title: Reference section landing
    author: process:git
    last_modified: 2026-08-25
  - id: contributor-checklist
    resource: ../../../docs/reference/contributor/checklist.rocdown
    title: Documentation contributor checklist
    author: process:git
    last_modified: 2026-08-25
  - id: sites-ref
    resource: ../../../docs/rocdown/sites.rocdown
    title: Published Rocdown site configuration reference
    author: process:git
    last_modified: 2026-08-31
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:cursor
    last_modified: 2026-08-31
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: stack-ia
    resource: ../site/rocci-dev-docs-stack-ia.md
    title: Stack-first docs information architecture
    author: process:cursor
    last_modified: 2026-08-24
  - id: stack-plan
    resource: ../../plans/site/rocci-dev-docs-stack-ia.md
    title: Stack-first docs implementation plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: follow-on
    resource: ../../plans/rocdown/docs-directory-semantics.md
    title: Implementation plan for directory-plus-index sections
    author: process:cursor
    last_modified: 2026-08-31
---

# Directory plus index is the Rocdown site section

## Verdict

The Rocdown catalog already has a clear **page** contract: a `.rocdown` file
gets a stable id from its path, a directory `index.rocdown` owns that
directory's URL, and `[[nav]]` lists which pages appear and in what
order.[^catalog][^site-rs][^sites-ref]

The **section** contract is the planner's second pass: listed ids plus
`index.rocdown` become the sidebar forest. Before Phases 0–4 it peeled a
landing only when the page title equaled the nav label. After those
phases it peels by id (heading-as-landing) and warns `RD2205` on
indexless clusters. Contributor and Appendix now have landings. The
remaining hole is that the landing is not a child row, and `NavList`
extra-indents subsection folds.[^plan-rs][^nav-list]

That is why the same filename did three different things before
peel-by-id shipped:[^docs-nav][^site-nav][^docs-index][^language-index][^contributor-checklist]

| Authored shape | What readers see |
| --- | --- |
| `docs/index.rocdown` titled Overview under group Start | A child row named **Overview** |
| `docs/reference/language/index.rocdown` | A named subsection **Rocci language reference** |
| `docs/reference/contributor/` with no index | No Contributor heading; three long titles sit after Diagnostics |

Recommend one rule: **a directory with `index.rocdown` is a section**. A
nested index is a named subsection. A directory without an index is not a
section; two or more listed pages in that shape warn `RD2205`.

Phases 0–4 of the paired plan shipped peel-by-id **heading-as-landing** (the
index is only the fold heading). After that shipped, the landing is
invisible as a page, and `NavList` extra-indents subsection folds so
Language and Contributor do not line up with Runtime. The remaining
contract is a reserved first child **Overview** (sidebar label only) plus
equal sibling indent. Heading may still link to the same URL.
Implementation: [directory-plus-index plan](/plans/rocdown/docs-directory-semantics.md).[^follow-on][^nav-list]

This does not reopen the stack-first group labels (Start, Templates,
Applications, Rocdown, Reference, Troubleshooting). Those are *which*
sections exist. This record is *how a directory becomes one*.[^stack-ia][^stack-plan]

## Method

Read catalog resolve, planner forest construction and its unit tests, page
discovery, `NavConfig`, `NavList.rocci`, `docs/rocdown.toml`,
`site/rocdown.toml`, the current `docs/` tree, and the published site
reference. Compared those rules with the stack-IA records and with how
VitePress, Starlight, MkDocs, Docusaurus, and this repo's OKF `index.md`
treat directory indexes. Did not treat generated `dist/` HTML as the
contract.[^catalog][^plan-rs][^sites-ref][^stack-ia]

## Two layers

Rust owns the data; Rocci owns the chrome. The missing contract sits in the
planner that builds `PageView.sidebar`, not in the theme.[^catalog-shell][^compiler-arch][^nav-list]

### Layer 1 — catalog (stated)

| Fact | Rule |
| --- | --- |
| Identity | Path relative to the content root, minus `.rocdown`. Mounts prefix the id (`docs/` + `index` → `docs/index`).[^site-rs][^sites-ref] |
| Title | `@page.meta.title`, else the first heading, else the id.[^site-rs] |
| Route | `index` at a directory becomes that directory's URL (`guides/index` → `/guides/`). Other files become `/dir/stem/`.[^catalog][^sites-ref] |
| Inclusion and order | Explicit `[[nav]]` / `[[nav.groups]]` page ids. Empty `items` plus `directory` auto-lists that prefix, indexes first.[^config-rs][^catalog] |
| Unlisted | Published authored pages omitted from nav warn `RD2202`.[^catalog][^sites-ref] |
| Groups vs items | `[[nav.groups]]` is a curated sub-list under a lane (Docs → Start, Templates, …). It is not inferred from the filesystem.[^config-rs][^site-nav] |

`inspect nav` prints this layer: a labeled list of `{id, title, route}`. It
does not print the sidebar forest.

### Layer 2 — sidebar forest

`lanes_and_sidebar` walks each group's items. Shipped (Phases 0–4):[^plan-rs]

1. **Shortest index is the group root.** Among listed `*/index` ids, the
   shortest directory is the section root (`docs/reference` in Reference).
2. **Nested index folds.** An `*/index` whose directory is strictly under
   that root becomes a subsection titled from the **page title**, with later
   listed descendants as its children.
3. **Peel by id.** If the first listed item is the group's root `index`
   and there is more than one item, that item is removed and its route
   becomes the group heading href. Title-equality peel is gone.
4. **Indexless directory stays flat.** Pages under a directory with no
   listed `…/index` stay ordinary leaves. `RD2205` warns when two or more
   listed non-draft pages share that shape.
5. **Depth cap.** `flatten_group_depth` promotes grandchildren to siblings.
   The visible tree is group → optional subsection → pages. A third
   `*/index` does not nest.

`NavList` only renders that view: a fold heading may be a link; a child
with no items is a leaf link. When a group has any fold, remaining rows
go into `children`; nested fold titles then pick up extra `.nav-child`
margin that sibling leaves do not.[^nav-list]

The published site page now documents peel-by-id, nested folds, `RD2205`,
and the depth cap. It still describes heading-as-landing without an
Overview child.[^sites-ref]

## The three observations (before Phases 0–4)

### 1. The docs root becomes Overview

`docs/index.rocdown` is titled **Overview**. It is the first item of the
**Start** group.[^docs-index][^docs-nav]

Peel requires `item.title == label`. `"Overview" != "Start"`, so the landing
stays as a child row. Clicking Start does not go to `/docs/` unless the
theme treats an empty group href as something else; the heading is a
non-link label and Overview is the first link.[^plan-rs][^nav-list]

Templates, Applications, Rocdown, Reference, and Troubleshooting title
their index with the **same word as the group label**, so those landings
disappear into the heading.[^templates-index][^reference-index][^docs-nav]

The Overview row is not a reserved generator label. It is a title mismatch.

### 2. Language becomes a named subsection

`docs/reference/language/index.rocdown` is titled **Rocci language
reference**. It is nested under `docs/reference/index`, so it is a fold
index. Its title becomes the subsection heading; file-structure through
grammar nest inside; Runtime and later siblings stay outside.[^language-index][^plan-rs]

A planner test locks this: on the file-structure page the Language fold is
open, Reference's href is `/docs/reference/`, and contributor pages in that
fixture render as flat siblings after Runtime.[^plan-rs]

### 3. Contributor has no section

`docs/reference/contributor/` has three listed pages and no
`index.rocdown`.[^contributor-checklist][^docs-nav][^site-nav]

The catalog includes them. The forest has nothing to fold, so there is no
**Contributor** heading. Readers see three leaf titles — Rocci tree
appendix, Rocdown tree appendix, Documentation contributor checklist —
after Diagnostics, under a long Language fold. That is "does not show up
in docs nav": the *section* is missing, not the routes.

`appendix/` under Start was the same shape: three primers, no index, no
Appendix heading.[^docs-nav]

Phase 2 added `reference/contributor/index.rocdown` and
`appendix/index.rocdown`. After peel-by-id, those landings are only fold
headings. The original Start Overview child is also gone: Start links to
`/docs/` with no page row.[^docs-nav][^plan-rs]

## What authors are writing today

The mental model implied by the tree and the stack-IA plan:[^stack-plan][^docs-nav]

```text
docs/
  index.rocdown                 # portal (sidebar should show Overview)
  install.rocdown
  appendix/index.rocdown        # shipped landing
  templates/index.rocdown       # layer landing
  templates/components.rocdown
  reference/index.rocdown
  reference/language/index.rocdown
  reference/language/tags.rocdown
  reference/contributor/index.rocdown
  reference/contributor/checklist.rocdown
```

Authors treat `index.rocdown` as "the page for this folder." The
generator now agrees on peel-by-id. Readers still cannot find that page
as a sidebar row.

`[[nav]]` still lists every page. Hierarchy is inferred from ids, not from
nested `[[nav.groups]]`. Language is not a configured group; it is a
filename.[^site-nav][^plan-rs]

## Industry and in-repo pattern

Layered static docs that feel predictable share one move: **the folder
index is the section**.

| System | Directory index | Sidebar |
| --- | --- | --- |
| VitePress / Starlight | `index.md` is the folder page | Autogenerated groups from folders; heading is the category |
| MkDocs | `index.md` is the section home | Explicit nav; index is the section entry |
| Docusaurus | authored index or generated category page | Folder plus `_category_` metadata |
| OKF in this repo | `knowledge/**/index.md` is the collection listing | Collection exists because the index exists |

Rocdown should match that, with MkDocs-style **explicit order** (keep
`[[nav]]`) and Starlight-style **inferred tree** (index files, not a second
TOML tree). Do not add `_category_.json`. Do not make authors repeat
Language as `[[nav.groups]]` when `language/index.rocdown` already names
it.[^sites-ref][^catalog-shell]

Diátaxis remains an authoring lens for page *kind*, not a filesystem
rule.[^stack-ia]

## Recommended contract

### Nouns

| Noun | Meaning |
| --- | --- |
| **Page** | One `.rocdown` file. Id and route from Layer 1. |
| **Section** | A directory that contains `index.rocdown`. |
| **Landing** | That index page. In the sidebar it is the reserved first child **Overview**, not the only click target. |
| **Member** | A non-index page in that directory, or in an indexless child directory. |
| **Nav group** | An authored `[[nav]]` / `[[nav.groups]]` label. Curated name and order. |
| **Lane** | A top-level `[[nav]]` that has groups (Docs, Examples, Project). |

### Rules

1. **Directory + index = section.** Adding `index.rocdown` creates a
   section. Removing it dissolves the section; remaining pages become
   members of the parent.
2. **Heading names the section.** Text is the `[[nav]]` label for a
   configured group, or the index page title for an inferred subsection.
   The heading may link to the index route. It is not the only way to
   open the landing.
3. **First child is Overview.** Every section and subsection with an
   index gets a first sidebar row titled **Overview** pointing at that
   index. `@page.meta.title` and the H1 stay the document title. Do not
   print `INDEX`. Do not repeat the section title as the child
   (`Reference` / `Reference`).
4. **Peel by id, then reinsert Overview.** The group's root `index` still
   sets `group.href`. Title-equality peel stays retired. After peel,
   insert the Overview row; do not leave the landing heading-only.
5. **Nested index = named subsection.** `dir/nested/index.rocdown` folds
   listed descendants under a heading titled from that page. Same
   Overview-first rule inside the fold.
6. **Equal sibling indent.** Leaves and subsection headings under the
   same parent share one indent. Extra indent is only *inside* a nested
   fold. `NavList` must not extra-indent `.nav-child` fold titles relative
   to sibling leaf links (that is why Contributor looks like a child of
   Diagnostics).[^nav-list]
7. **No index = no section.** Pages stay in the parent, in list order.
8. **Warn on an indexless cluster.** `rocdown check` warns `RD2205` when
   two or more *listed* pages share a directory that has no listed index.
   Contributor and Appendix landings shipped in Phase 2; the warning
   remains for other sites.
9. **Nav still owns inclusion and order.** Unlisted pages still `RD2202`.
   `directory =` remains an optional way to fill items, not a second
   hierarchy language.
10. **Visible depth is two.** Group → optional subsection → pages. Deeper
    indexes flatten. v1 does not grow a recursive sidebar.

A single-page group (FAQ: only `faq/index`) stays one leaf, as now.[^plan-rs]

### Authoring cookbook

```text
section/
  index.rocdown          # required to name the section
  a.rocdown
  b.rocdown
  nested/
    index.rocdown        # required to name the subsection
    c.rocdown
```

- Title the index as the **section name** readers should see on nested
  folds (`Rocci language reference`, `Contributor`). The configured
  `[[nav]]` label is the fold heading. The sidebar landing row is always
  **Overview**, not the meta title and not `INDEX`.
- Put the landing first in `[[nav]]` items. The generator inserts the
  Overview row; authors do not add a second `overview.rocdown` unless
  they want a distinct member page.
- List subsection members after that subsection's index, before the next
  sibling.
- Do not use `[[nav.groups]]` merely to recreate a directory. Use groups
  for curated lanes (Start vs Templates).

### How the three cases should read

| Case | After the remaining contract |
| --- | --- |
| `docs/index.rocdown` titled Overview | Start heading (may link to `/docs/`). First child **Overview**. H1 stays Overview. |
| `reference/language/index.rocdown` | Subsection **Rocci language reference**. First child **Overview**, then File structure, … |
| `reference/contributor/` | Subsection **Contributor** (landing shipped). First child **Overview**, then the three appendix pages. |

`appendix/` should get `appendix/index.rocdown` if those primers are a
named subsection; otherwise they stay Start members and the warning is
accepted or the files move up.

### Rejected alternatives

**Heading-as-landing only (Phases 0–4).** Peel the index into the fold
heading and omit a child row. Shipped, then failed as UX: readers do not
treat a disclosure label as a page, even when it is current (blue
Reference on `/docs/reference/`).

**Print `INDEX`.** Filename vocabulary. Not a reader label.

**Repeat the section title** as the first child (`Reference` /
`Reference`). That is the old title-matching peel. It collides when the
document title already differs (`Overview` vs Start).

**Use `@page.meta.title` as the child when it equals the group label.**
Same collision as repeating the section title.

**Filesystem-only nav.** Drop `[[nav]]` and autogenerate from directories.
Rejected: order, omission, and mount visibility are editorial. Keep
explicit lists. `directory =` is enough automation.

**Require `[[nav.groups]]` for every subsection.** Rejected: Language
already works as a file. Groups stay a lane tool.

**Keep title-equality peel.** Rejected: it is the Start/Templates
inconsistency. Authors should not have to remember to name the page
Templates for the landing to attach.

## What shipped (Phases 0–6)

Directory+index sections, peel-by-id, reserved Overview first child,
equal sibling indent, `RD2205`, Contributor and Appendix landings, and
the Overview-first cookbook on the site-configuration page.[^follow-on][^sites-ref]

Heading-as-landing-only (Phases 0–4) hid the index as a page and let
`NavList` extra-indent subsection folds. Phases 5–6 inserted the
Overview row, put same-level leaves in `items`, and taught that
contract.[^plan-rs][^nav-list]

## Risks

- **Two links to the same URL.** Heading and Overview may both go to the
  index. That is accepted. Overview is the discoverable page row.
- **Document title vs sidebar label.** `docs/index.rocdown` titled
  Overview still shows Overview as the child; the H1 does not have to
  change. Templates titled Templates still show Overview in the sidebar.
- **Planner tests.** Phase 1 tests that require no Overview child after
  peel must flip.[^plan-rs]
- **Deep trees.** Authors who nest `a/b/c/index.rocdown` will not get a
  third fold. Document the cap; do not silently invent depth.

Writing this record does not start the plan.

[^catalog]: Page ids, derived routes, explicit `[[nav]]` resolve, `directory` auto-list, `RD2202`.
[^plan-rs]: Peel-by-id, Overview first child, leaves in `items`, folds in `children`, Language and appendix tests.
[^site-rs]: Derived id from relative path; title from `@page` then heading; root `index` defaults to `home` layout.
[^config-rs]: `NavConfig` label, items, optional directory, nested groups.
[^nav-list]: Fold headings as links; empty-item children as leaf anchors; `.nav-fold .nav-fold .nav-child` adds 1.4rem only on nested fold titles.
[^sites-ref]: Id/route table, peel-by-id, `RD2205`; still heading-as-landing without an Overview child.
[^docs-nav]: Start lists `index` then primers and Appendix; Reference lists language then Contributor indexes.
[^site-nav]: Same lists under the Docs lane with `docs/` prefixes.
[^docs-index]: Portal `@page` title and H1 are Overview.
[^templates-index]: Templates landing title equals the Templates group label.
[^language-index]: Nested landing titled Rocci language reference.
[^reference-index]: Reference landing title equals the Reference group label.
[^contributor-checklist]: Contributor pages and the docs-PR checklist; landing added in Phase 2.
[^compiler-arch]: Rust owns navigation data; Rocci owns chrome. Phase 3 named peel-by-id forest derivation.
[^catalog-shell]: Catalog in Rust; theme receives a page view.
[^stack-ia]: Stack-layer groups; Diátaxis as authoring lens only.
[^stack-plan]: Target tree with `templates/index`, `applications/index`, `rocdown/index`; no directory-semantics contract.
[^follow-on]: Phased generator, docs-tree, and public-docs work.
