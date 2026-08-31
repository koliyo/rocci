---
type: Implementation Plan
title: Name Rocdown page and nav view types
description: >-
  First cut landed in b3129a9a (Views.Page(_), nominal NavGroupView).
  Leftover nightly idiom is idiomatic-roc-nightly-types.
tags: [domain/rocdown, integration/roc, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T16:10:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/named-roc-view-types.md
    title: Generated Roc page views are anonymous structural records
    author: process:cursor
    last_modified: 2026-08-31
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
  - id: landed
    resource: https://github.com/koliyo/rocci/commit/b3129a9a355441755fab28f1c446b35b2c677278
    title: typecheck named Views against nightly Roc
    author: process:git
    last_modified: 2026-08-31
  - id: follow-on
    resource: ./idiomatic-roc-nightly-types.md
    title: Align apply Roc with nightly type idiom
    author: process:cursor
    last_modified: 2026-08-31
---

# Name Rocdown page and nav view types

## Purpose and authority

This plan executed the [anonymous page-view research](/research/rocdown/named-roc-view-types.md).
Rust already owns `PageView` / `NavGroupView`. Before `b3129a9a`,
generated Roc and `RocdownBuild.roc` shared those shapes only by
inference, so a missing nested field reprinted the entire apply page
type.[^research][^view-rs][^build-roc]

The record is exploratory. Phases 1–5 are in `b3129a9a` (with
nightly-required `Page(a)` / `Page(_)` / `NavGroupView :=`, not the
alias-first sketch below). Leftover idiom:
[align apply Roc with nightly type idiom](/plans/rocdown/idiomatic-roc-nightly-types.md).[^landed][^follow-on]

## Goal

Give the static apply path one named Roc contract for chrome data:

1. a stable module lists aliases that match `view.rs` field names;
2. `write_page!` / `write_all!` are annotated with `Page`;
3. generated `RocdownPages.pages` ascribes `List(Page)` (or uses
   constructors) so values unify against that alias;
4. nested `NavGroup` always includes `children` (already required for
   `NavList`);
5. a deliberate missing-field fixture fails with a diagnostic that
   names `NavGroup` or `children`, not only `List.iter` on the full
   page.

## Out of bound

- Changing the Zig Roc compiler or filing that as a Rocci phase.
- Moving catalog, routing, or sidebar forest planning into Roc.
- Renaming or splitting Rust `PageView` fields.
- Restructuring `SiteShell` / `NavList` to shrink errors.
- Opaque wrappers (`NavGroup :: { … }.{ }`). Nominal `:=` for recursive
  nav is required on this nightly and is already landed.
- `OkfPages.roc` / OKF apply hash work.
- Pulling `PageView` values out of `RocdownPages.roc` (apply-data /
  compile-cache plan, not this type contract).
- Annotating every helper inside chrome templates.

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Two layers | Rust catalog and planner own data. Rocci theme renders. Named Roc types describe the hand-off; they do not discover pages.[^catalog-shell][^compiler-arch] |
| Field names | Roc aliases use the same names as `view.rs` (`title`, `href`, `open`, `items`, `children`, …).[^view-rs] |
| Recursive nav | `NavGroup.children` is `List(NavGroup)`. Empty folds still have `children: []`.[^nav-list][^emit] |
| Stable type module | Aliases live in runtime (or equivalent), not in per-site generated `RocdownPages.roc`, so content edits do not rewrite the type file.[^research] |
| Failed builds | A type error keeps the previous output tree. |
| Docs format | Knowledge stays inert Markdown. Public Rocdown pages change only if a published apply contract is named. |

## Target authoring model

```text
crates/rocci-rocdown/runtime/
  Views.roc            # aliases: NavItem, NavGroup, Page, …
  RocdownBuild.roc     # write_page! : Str, Page -> …
generated (per site)
  RocdownPages.roc     # import Views; pages : List(Page)
```

`Views.roc` is ordinary Roc. Emit keeps printing records; it ascribes
the alias or calls a thin constructor. Do not generate a second copy of
the type definitions inside `pages = [`.

## Phases

### Phase 1 — freeze aliases against `view.rs`

**Bound:** Add `Views.roc` (name may be `RocdownViews.roc` if the
runtime import table requires a prefix) with aliases for `SiteView`,
`LaneView`, `NavItemView`, `NavGroupView`, `BreadcrumbView`,
`OutlineView`, `ResourceView`, `CollectionItemView`, and `PageView`.
Use the Rust names or a documented snake_case map, one table in the
module comment. `Page` is the apply item `{ article_path, output_path,
segments, view : PageView }` or a split if `segments` must stay a
separate tag union.

**Out of bound:** Annotating `RocdownBuild`. Changing emit. Opaques.

**Exit:** `cargo test -p rocci-rocdown plan::` and `cargo fmt --all -- --check`.
The module is imported by a compile-smoke the suite already owns, or a
new planner test asserts the file is staged with the runtime.

### Phase 2 — annotate the apply runtime

**Bound:** `RocdownBuild.roc` imports the aliases and annotates
`write_page!` and `write_all!`. `siteShell(item.view, content)` stays
the chrome call; do not change theme markup.

**Out of bound:** Generated `pages` ascription. Theme edits.

**Exit:** Same planner tests. A `rocdown` site load that already
compiles still typechecks (`cargo test -p rocci-rocdown` plan / theme
smokes that invoke Roc if the crate already has one; do not add a new
hosted Roc job).

### Phase 3 — ascribe generated pages

**Bound:** `pages_roc()` emits `import Views` (or the chosen module)
and `pages : List(Page) = [ … ]`. Nested nav records include
`children` (already true after the sidebar-order fix). Every page
literal unifies with `Page`.

**Out of bound:** Moving `view` bytes out of the generated file.
OKF pages.

**Exit:** `cargo test -p rocci-rocdown plan::` including
`pages_roc_is_stable_*`. `cargo fmt --all -- --check`.

### Phase 4 — missing-field diagnostic

**Bound:** A fixture or emit-unit test omits `children` on one nested
group (or builds a tiny `Page` missing that field) and compiles it
with `roc`. The diagnostic must mention `children` or `NavGroup`.
Reject a pass that only shows `List.iter` on the full `segments` union
with no field name.

**Out of bound:** Changing Roc itself. Hosted CI Roc lane unless the
crate already runs this smoke locally behind the existing opt-in.

**Exit:** The new test is green in the same lane as other Rocdown Roc
smokes. Document the command in the crate README if it is opt-in.

### Phase 5 — pointer in crate docs

**Bound:** `crates/rocci-rocdown/README.md` states that apply chrome
data is `Views.Page` / `Views.NavGroup` and that nested groups always
carry `children`. No public language-reference rewrite unless
`docs/rocdown/sites.rocdown` already discusses apply Roc.

**Out of bound:** Author-skill essays. Knowledge architecture
promotion to normative.

**Exit:** README sentence exists. `okmate check knowledge --profile base`
if this plan or the research is edited in the same change.

## Follow-on (not scheduled)

- [Align apply Roc with nightly type idiom](/plans/rocdown/idiomatic-roc-nightly-types.md)
  (`=> Try`, wasm ascription, inspector `Some(Str)`).
- The same aliases for `OkfPages.roc`.
- Moving `PageView` *values* out of `RocdownPages.roc` for compile-cache
  (existing component-generation / OKF cost plans).

[^research]: Anonymous apply records; prefer stable aliases over theme or compiler work.
[^emit]: Generated `RocdownPages.roc` literals; `pages_roc()` is the emit site.
[^build-roc]: Unannotated `write_page!` and `write_all!`.
[^view-rs]: Canonical field names for the aliases.
[^nav-list]: Nested groups must expose `children` for `is_leaf_group`.
[^catalog-shell]: Named types describe the hand-off; they do not discover pages.
[^compiler-arch]: Planner and catalog stay in Rust.
[^landed]: First cut typechecks on nightly-2026-08-23-fb208ba.
[^follow-on]: Remaining Try / wasm / inspector payload work.
