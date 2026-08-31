---
type: Research Report
title: Generated Roc page views are anonymous structural records
description: >-
  RocdownBuild and RocdownPages share PageView by inference. A missing
  nested field reprints the entire page type. Name the Rust view records
  in Roc at the module edge.
tags: [domain/rocdown, integration/roc, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T13:25:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: emit
    resource: ../../../crates/rocci-rocdown/src/plan/emit.rs
    title: Generated RocdownPages.roc page and sidebar records
    author: process:git
    last_modified: 2026-08-31
  - id: build-roc
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Shared apply runtime write_page! and write_all!
    author: process:git
    last_modified: 2026-08-19
  - id: view-rs
    resource: ../../../crates/rocci-ui/src/view.rs
    title: Rust PageView, NavGroupView, and related records
    author: process:git
    last_modified: 2026-08-31
  - id: nav-list
    resource: ../../../crates/rocci-ui/templates/chrome/NavList.rocci
    title: Shared expandable sidebar renderer
    author: process:git
    last_modified: 2026-08-31
  - id: theme
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell SiteShell
    author: process:git
    last_modified: 2026-08-31
  - id: nav-rs
    resource: ../../../crates/rocci-rocdown/src/plan/nav.rs
    title: Sidebar forest, Overview peel, mixed children
    author: process:git
    last_modified: 2026-08-31
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator architecture
    author: process:cursor
    last_modified: 2026-08-31
  - id: component-gen
    resource: ../../plans/rocci/rocci-component-generation.md
    title: Rocci component generation plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: block-plan
    resource: ../../plans/rocdown/rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers
    author: process:cursor
    last_modified: 2026-08-30
  - id: plan
    resource: ../../plans/rocdown/named-roc-view-types.md
    title: Name Rocdown page and nav view types
    author: process:cursor
    last_modified: 2026-08-31
---

# Generated Roc page views are anonymous structural records

## Purpose and authority

This record explains why `rocdown view site` printed two near-identical
multi-screen Roc types after the sidebar started mixing folds and
leaves, and what an author should do instead of treating that dump as a
compiler bug. It is exploratory. It does not change the Rust catalog
boundary.[^catalog-shell] Implementation: [named Roc view types](/plans/rocdown/named-roc-view-types.md).[^plan]

## Incident

`NavList` now treats an empty-item child as a leaf only when
`is_leaf_group` is true, which reads `child.children`. Nested sidebar
groups were emitted without a `children` field. Roc records are
structural, so `{ title, href, open, items }` is not
`{ title, href, open, items, children }`.[^nav-list][^emit][^nav-rs]

`write_all!` takes `RocdownPages.pages` with no annotation. The expected
shape is inferred from `write_page!` → `item.view` →
`RocdownTheme.siteShell`. The mismatch therefore appeared as
`List.iter` on the entire page record, including every widget tag in
`segments` and every `PageView` field.[^build-roc][^theme]

The field was added (`children: []` on nested groups). The dump remains
the default error form for this pipeline.

## What is shipped

| Layer | Shape | Named in Roc? |
| --- | --- | --- |
| `rocci-ui` Rust | `PageView`, `NavGroupView`, `NavItemView`, `SiteView`, … | No. These are Rust structs.[^view-rs] |
| Generated `RocdownPages.roc` | Anonymous `{ article_path, output_path, segments, view: { … } }` | No. `pages = [` of literals.[^emit] |
| `RocdownBuild.roc` | `write_page!` / `write_all!` unannotated | No.[^build-roc] |
| Theme / `NavList` | Component props inferred from `| { … } |` | No named `Page` or `NavGroup`.[^theme][^nav-list] |

Rust remains the catalog and planner owner. Rocci owns chrome. That
split is implemented.[^catalog-shell][^compiler-arch] The gap is that the
shared view **values** never become a Roc type at the apply boundary.

Earlier generation research already wanted `PageView` out of
`RocdownPages.roc` so content edits do not recompile chrome. That is a
hash and apply-data question. This record is the **type** question: even
after values move, apply still needs one named page record.[^component-gen]

Block-renderer work treated named Roc aliases as optional documentation
for `:kind` schemas, not as a schema source. That does not apply here.
`PageView` is an apply-time contract between generated data and a
hand-written runtime, not a kind registry.[^block-plan]

## Compiler versus authoring

The checker was correct. Inference found a missing field. The weakness
is **presentation**: Zig Roc expands the full structural type and does
not print a `Page` / `NavGroup` name that does not exist.

Idiomatic Roc in this repo:

- Leave local helpers inferred.
- Name types at module edges and on generated values.
- Use an alias (`NavGroup : { …, children : List(NavGroup) }`) first.
  Recursive nav needs a name; an anonymous nested record cannot refer to
  itself cleanly.
- Use an opaque (`NavGroup := { … }.{}`) only if aliases still expand
  in diagnostics and wrap/unwrap cost is acceptable. Do not start there.
- Do not restructure `SiteShell` or split `write_page!` merely to shorten
  errors.
- Do not annotate every `|item|` inside `NavList`.

A type annotation without a definition still expands. `write_page! : Str,
Page -> …` only helps if `Page` exists and both sides use it.

## Options

| Option | Effect | Verdict |
| --- | --- | --- |
| Keep emitting identical anonymous records | Prevents this class of mismatch; errors stay huge | Necessary hygiene, not a substitute |
| Annotate `write_all!` with an inline record | Duplicates `PageView` in `RocdownBuild.roc` | Reject as the only step |
| Stable `Views.roc` aliases matching `view.rs` | One contract; generated pages ascribe `Page` | Preferred first cut |
| Opaque wrappers | Shorter identity; emit must wrap | Follow-on if aliases still dump |
| Change the Roc compiler | Better diffs and alias names | Out of this repo |
| Theme / catalog restructure | Unrelated to the dump | Reject |

`OkfPages.roc` is the same anonymous-record pattern on the OKF apply
path. Do not fold that fix into this work unless a later plan names it.

## Recommendation

Add a stable runtime module of aliases that mirror `view.rs`, annotate
`RocdownBuild`, and emit `pages : List(Page)` (or constructors) so a
missing `children` is a `NavGroup` field error. Keep Rust field names
as the source of truth. Keep `Views.roc` out of the per-page generated
file so content edits do not rewrite the type module.[^view-rs][^emit][^plan]

[^emit]: `pages_roc` prints anonymous page and sidebar records; nested groups now include `children: []`.
[^build-roc]: `write_all!` iterates `RocdownPages.pages`; `write_page!` passes `item.view` to `siteShell` with no annotation.
[^view-rs]: Rust `PageView` and recursive `NavGroupView.children`.
[^nav-list]: `is_leaf_group` reads `group.children`; empty-item children render as leaf anchors.
[^theme]: `SiteShell` takes the page view record as component props.
[^nav-rs]: Planner forest splits no longer bucket all folds after all leaves.
[^catalog-shell]: Rust catalog; Rocci shell receives normalized view data.
[^compiler-arch]: Generator architecture; chrome is Rocci, data is Rust.
[^component-gen]: Move `PageView` values out of `RocdownPages.roc` is a compile-cache goal, not this type contract.
[^block-plan]: Named Roc aliases are optional for `:kind` schemas; they are not the apply `PageView` contract.
[^plan]: Phased aliases, annotations, ascription, and a missing-field diagnostic.
