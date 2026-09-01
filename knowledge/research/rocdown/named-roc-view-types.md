---
type: Research Report
title: Generated Roc page views are anonymous structural records
description: >-
  A missing nested nav field reprinted the entire apply page type.
  b3129a9a named Views; recursive nav must be nominal on this nightly.
  Leftover idiom: idiomatic-roc-nightly-types.
tags: [domain/rocdown, integration/roc, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T16:10:00Z }
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
  - id: landed
    resource: https://github.com/koliyo/rocci/commit/b3129a9a355441755fab28f1c446b35b2c677278
    title: typecheck named Views against nightly Roc
    author: process:git
    last_modified: 2026-08-31
  - id: follow-on
    resource: ./idiomatic-roc-nightly-types.md
    title: Apply Roc after named Views still mixes holes and nightly idiom
    author: process:cursor
    last_modified: 2026-08-31
  - id: follow-on-plan
    resource: ../../plans/rocdown/idiomatic-roc-nightly-types.md
    title: Align apply Roc with nightly type idiom
    author: process:cursor
    last_modified: 2026-08-31
---

# Generated Roc page views are anonymous structural records

## Purpose and authority

This record explains why `rocdown view site` printed two near-identical
multi-screen Roc types after the sidebar started mixing folds and
leaves. It is exploratory. It does not change the Rust catalog
boundary.[^catalog-shell] First cut: [named Roc view types](/plans/rocdown/named-roc-view-types.md)
landed in `b3129a9a`. Leftover idiom:
[nightly typing review](/research/rocdown/idiomatic-roc-nightly-types.md).[^plan][^landed][^follow-on]

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

As of `b3129a9a` the apply edge is named. The table below is the
**incident** state that caused the dump; do not treat it as current
HEAD.[^landed]

| Layer (before b3129a9a) | Shape | Named in Roc? |
| --- | --- | --- |
| `rocci-ui` Rust | `PageView`, `NavGroupView`, `NavItemView`, `SiteView`, … | No. These are Rust structs.[^view-rs] |
| Generated `RocdownPages.roc` | Anonymous `{ article_path, output_path, segments, view: { … } }` | No. `pages = [` of literals.[^emit] |
| `RocdownBuild.roc` | `write_page!` / `write_all!` unannotated | No.[^build-roc] |
| Theme / `NavList` | Component props inferred from `| { … } |` | No named `Page` or `NavGroup`.[^theme][^nav-list] |

HEAD now stages `Views.roc` (`Page(a)`, nominal `NavGroupView`),
ascribes `pages : List(Views.Page(_))`, and annotates `write_page!`.
Theme helpers stay inferred.[^landed][^follow-on]

Rust remains the catalog and planner owner. Rocci owns chrome. That
split is implemented.[^catalog-shell][^compiler-arch] The remaining type
question is leftover nightly idiom, not a missing `Views` module.

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

Idiomatic Roc on the product nightly (`fb208ba`):

- Leave local helpers inferred.
- Name types at module edges and on generated values.
- Use an alias (`SiteView : { … }`) when the type is not recursive.
- Use a **nominal** (`NavGroupView := { …, children : List(NavGroupView) }`)
  when the type refers to itself. Recursive aliases are illegal. This is
  not an opaque (`::`).
- Do not start with opaques.
- Type applications need parentheses: `Page(a)`, `Page(_)`, `List(U8)`.
  `Page a` and `Page _` do not parse.
- Do not restructure `SiteShell` or split `write_page!` merely to shorten
  errors.
- Do not annotate every `|item|` inside `NavList`.

A type annotation without a definition still expands. `write_page! : Str,
Page(_) => …` only helps if `Page` exists and both sides use it.
The first-cut “alias first for nav” advice was wrong; the landed commit
had to use `:=`.[^landed][^follow-on]

## Options

| Option | Effect | Verdict |
| --- | --- | --- |
| Keep emitting identical anonymous records | Prevents this class of mismatch; errors stay huge | Necessary hygiene, not a substitute |
| Annotate `write_all!` with an inline record | Duplicates `PageView` in `RocdownBuild.roc` | Reject as the only step |
| Stable `Views.roc` aliases matching `view.rs` | One contract; generated pages ascribe `Page` | Done, with nominal nav |
| Nominal `NavGroupView :=` | Required for recursion on this nightly | Landed; not optional |
| Opaque wrappers | Shorter identity; emit must wrap | Still not needed |
| Change the Roc compiler | Better diffs and alias names | Out of this repo |
| Theme / catalog restructure | Unrelated to the dump | Reject |

`OkfPages.roc` is the same anonymous-record pattern on the OKF apply
path. Do not fold that fix into this work unless a later plan names it.

## Recommendation

That first cut is in tree (`b3129a9a`). A missing `children` now names
`Views.NavGroupView`. Remaining nightly idiom (Try on effects, wasm
ascription, one old tag payload) is
[idiomatic Roc nightly types](/research/rocdown/idiomatic-roc-nightly-types.md).[^view-rs][^emit][^plan][^landed][^follow-on][^follow-on-plan]

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
[^plan]: First-cut phases; landed in b3129a9a.
[^landed]: `Page(a)`, `Page(_)`, `NavGroupView :=`, `class_name` on previous/next.
[^follow-on]: Post-landing review of leftover holes and old tag syntax.
[^follow-on-plan]: Try, wasm apply, inspector `Some(Str)`.
