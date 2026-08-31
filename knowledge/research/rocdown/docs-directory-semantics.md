---
type: Research Report
title: Directory plus index is the Rocdown site section
description: The catalog lists pages; the sidebar infers sections from index.rocdown. Title-matching peel, nested folds, and indexless directories are three presentations of one unstated rule. Make directory+index the section, peel the landing into the heading, and require an index to name a subsection.
tags: [domain/rocdown, concern/publication, concern/developer-experience, concern/navigation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:50:00Z }
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
    last_modified: 2026-08-25
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
    last_modified: 2026-08-30
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:cursor
    last_modified: 2026-08-26
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

The **section** contract is not written down. A second pass in the planner
turns that flat list into the sidebar forest. It peels a landing into the
group heading only when the page title equals the nav label, folds a nested
`*/index` into a named subsection, and leaves an indexless directory as
ungrouped leaves.[^plan-rs]

That is why the same filename does three different things in today's
manual:[^docs-nav][^site-nav][^docs-index][^language-index][^contributor-checklist]

| Authored shape | What readers see |
| --- | --- |
| `docs/index.rocdown` titled Overview under group Start | A child row named **Overview** |
| `docs/reference/language/index.rocdown` | A named subsection **Rocci language reference** |
| `docs/reference/contributor/` with no index | No Contributor heading; three long titles sit after Diagnostics |

Recommend one rule: **a directory with `index.rocdown` is a section**. The
index is the landing (heading is the link; the index is not a second row). A
nested index is a named subsection. A directory without an index is not a
section; two or more listed pages in that shape should warn. Implementation:
[directory-plus-index plan](/plans/rocdown/docs-directory-semantics.md).[^follow-on]

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

### Layer 2 — sidebar forest (unstated)

`lanes_and_sidebar` walks each group's items and applies:[^plan-rs]

1. **Shortest index is the group root.** Among listed `*/index` ids, the
   shortest directory is the section root (`docs/reference` in Reference).
2. **Nested index folds.** An `*/index` whose directory is strictly under
   that root becomes a subsection titled from the **page title**, with later
   listed descendants as its children.
3. **Title-matching peel.** If the first *flat* item's title equals the
   `[[nav]]` label and there is more than one item, that item is removed and
   its route becomes the group heading href.
4. **Indexless directory stays flat.** Pages under `contributor/` or
   `appendix/` are ordinary leaves of the parent group.
5. **Depth cap.** `flatten_group_depth` promotes grandchildren to siblings.
   The visible tree is group → optional subsection → pages. A third
   `*/index` does not nest.

`NavList` only renders that view: a fold heading may be a link; a child
with no items is a leaf link.[^nav-list]

The published site page documents Layer 1 and `[[nav.groups]]`. It does not
document peel, fold, or the depth cap.[^sites-ref]

## The three observations

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

`appendix/` under Start is the same shape: three primers, no index, no
Appendix heading.[^docs-nav]

## What authors are writing today

The mental model implied by the tree and the stack-IA plan:[^stack-plan][^docs-nav]

```text
docs/
  index.rocdown                 # portal
  install.rocdown
  templates/index.rocdown       # layer landing
  templates/components.rocdown
  reference/index.rocdown
  reference/language/index.rocdown
  reference/language/tags.rocdown
  reference/contributor/checklist.rocdown   # no landing
```

Authors already treat `index.rocdown` as "the page for this folder" in
Templates, Applications, Rocdown, Troubleshooting, and Language. Start and
Contributor do not follow that habit. The generator only sometimes agrees,
and only when titles line up.

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
| **Landing** | That index page. It *is* the section in the sidebar. |
| **Member** | A non-index page in that directory, or in an indexless child directory. |
| **Nav group** | An authored `[[nav]]` / `[[nav.groups]]` label. Curated name and order. |
| **Lane** | A top-level `[[nav]]` that has groups (Docs, Examples, Project). |

### Rules

1. **Directory + index = section.** Adding `index.rocdown` creates a
   section. Removing it dissolves the section; remaining pages become
   members of the parent.
2. **Heading is the landing.** The section heading links to the index
   route. The index is not also a child row. The heading text is the
   `[[nav]]` label for a configured group, or the index page title for an
   inferred subsection.
3. **Peel by id, not by title.** If the first listed item of a group is
   that group's root `index`, peel it even when the page title is
   Overview and the label is Start. Title-equality peel is retired.
4. **Nested index = named subsection.** `dir/nested/index.rocdown` folds
   listed descendants under a heading titled from that page. Same
   heading-as-landing rule: no extra Overview row inside the fold.
5. **No index = no section.** Pages stay in the parent, in list order.
6. **Warn on an indexless cluster.** `rocdown check` warns when two or
   more *listed* pages share a directory that has no listed index (today:
   `reference/contributor`, `appendix`). One file in a directory is a
   page, not a forgotten section.
7. **Nav still owns inclusion and order.** Unlisted pages still `RD2202`.
   `directory =` remains an optional way to fill items, not a second
   hierarchy language.
8. **Visible depth is two.** Group → optional subsection → pages. Deeper
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
  folds (`Rocci language reference`, `Contributor`). For a configured
  group, the label in `rocdown.toml` is what the sidebar prints; the
  index title may match (Templates) or describe the landing (Overview on
  `/docs/`) without creating a second row.
- Put the landing first in `[[nav]]` items.
- List subsection members after that subsection's index, before the next
  sibling.
- Do not use `[[nav.groups]]` merely to recreate a directory. Use groups
  for curated lanes (Start vs Templates).
- Do not title a group-root index with a *different* word in order to
  force a child row. If a page must appear beside the landing, it is not
  the index — give it its own stem (`overview.rocdown` is a member).

### How the three cases should read

| Case | After the contract |
| --- | --- |
| `docs/index.rocdown` titled Overview | Start heading links to `/docs/`. No Overview child. The page H1 can stay Overview. |
| `reference/language/index.rocdown` | Subsection **Rocci language reference** linking to `/docs/reference/language/`. Members inside. |
| `reference/contributor/` | Add `contributor/index.rocdown` titled Contributor (or Contributor appendix). Until then, check warns. |

`appendix/` should get `appendix/index.rocdown` if those primers are a
named subsection; otherwise they stay Start members and the warning is
accepted or the files move up.

### Rejected alternatives

**Reserved Overview child.** Always insert a first child labeled Overview
for every section index. Matches today's Start row and some VitePress
skins. Rejected: Templates through Troubleshooting already peel; a second
name for the same URL is noise; Language would grow a nested Overview
that the user did not ask for.

**Filesystem-only nav.** Drop `[[nav]]` and autogenerate from directories.
Rejected: order, omission, and mount visibility are editorial. Keep
explicit lists. `directory =` is enough automation.

**Require `[[nav.groups]]` for every subsection.** Rejected: Language
already works as a file. Groups stay a lane tool.

**Keep title-equality peel.** Rejected: it is the Start/Templates
inconsistency. Authors should not have to remember to name the page
Templates for the landing to attach.

## What should change

| Layer | Change |
| --- | --- |
| Planner | Peel the group-root index by id. Keep nested-index folds. Warn on indexless clusters of two or more listed pages. Keep the depth cap. Update `appendix_without_index_stays_flat` and add Start-Overview and contributor-cluster tests.[^plan-rs] |
| Docs tree | Add `docs/reference/contributor/index.rocdown`. Decide Appendix (index vs flatten). List the new ids in both toml files. |
| Public docs | Teach the cookbook on the site-configuration page and in the contributor checklist. One paragraph on pages: omit `route` on catalog pages; `index.rocdown` is the section landing.[^sites-ref][^contributor-checklist] |
| Architecture | After the planner ships, the generator record should say the sidebar forest is derived from listed ids plus index files, not from title equality.[^compiler-arch] |

Do not change `.rocci` / `.rocdown` grammar, lane labels, or the
Rust-catalog / Rocci-shell split.[^catalog-shell][^stack-plan]

## Risks

- **Start loses the Overview row.** Intended. The portal is still `/docs/`.
- **Indexless clusters warn.** Contributor and appendix will fail a
  newly-strict check until they gain an index or the warning stays
  warning-only (recommended: warning, not error, so existing sites do not
  break).
- **Title-matching tests.** Planner tests that expect Overview to remain
  a child must move to the new peel-by-id rule.[^plan-rs]
- **Deep trees.** Authors who nest `a/b/c/index.rocdown` will not get a
  third fold. Document the cap; do not silently invent depth.
- **Stack-IA scope.** File moves already landed for templates and
  applications. This work is generator semantics plus a few landings, not
  another IA rewrite.[^stack-plan]

Writing this record does not start the plan.

[^catalog]: Page ids, derived routes, explicit `[[nav]]` resolve, `directory` auto-list, `RD2202`.
[^plan-rs]: `peel_matching_index`, `is_fold_index`, `forest_from_items`, `flatten_group_depth`, Language and appendix tests.
[^site-rs]: Derived id from relative path; title from `@page` then heading; root `index` defaults to `home` layout.
[^config-rs]: `NavConfig` label, items, optional directory, nested groups.
[^nav-list]: Fold headings as links; empty-item children as leaf anchors.
[^docs-nav]: Start lists `index` then primers; Reference lists language index then contributor pages; no contributor index id.
[^site-nav]: Same lists under the Docs lane with `docs/` prefixes.
[^docs-index]: Portal `@page` title and H1 are Overview.
[^templates-index]: Templates landing title equals the Templates group label.
[^language-index]: Nested landing titled Rocci language reference.
[^reference-index]: Reference landing title equals the Reference group label.
[^contributor-checklist]: Contributor pages exist; the directory has no index file.
[^sites-ref]: Documents id/route table and `[[nav.groups]]`; does not document peel or fold.
[^compiler-arch]: Rust owns navigation data; Rocci owns chrome. Forest derivation is unnamed.
[^catalog-shell]: Catalog in Rust; theme receives a page view.
[^stack-ia]: Stack-layer groups; Diátaxis as authoring lens only.
[^stack-plan]: Target tree with `templates/index`, `applications/index`, `rocdown/index`; no directory-semantics contract.
[^follow-on]: Phased generator, docs-tree, and public-docs work.
