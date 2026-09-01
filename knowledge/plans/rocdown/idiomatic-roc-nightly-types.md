---
type: Implementation Plan
title: Align apply Roc with nightly type idiom
description: >-
  After b3129a9a, keep Views.Page(_) and nominal NavGroupView. Replace
  => _ with Try, annotate wasm apply, and fix the inspector Some(Str)
  payload.
tags: [domain/rocdown, integration/roc, concern/developer-experience, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-08-31T16:10:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocdown/idiomatic-roc-nightly-types.md
    title: Apply Roc after named Views still mixes holes and nightly idiom
    author: process:cursor
    last_modified: 2026-08-31
  - id: incident
    resource: ../../research/rocdown/named-roc-view-types.md
    title: Anonymous page-view incident
    author: process:cursor
    last_modified: 2026-08-31
  - id: named-plan
    resource: ./named-roc-view-types.md
    title: Name Rocdown page and nav view types
    author: process:cursor
    last_modified: 2026-08-31
  - id: build-roc
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: write_page! and write_all! annotations
    author: process:git
    last_modified: 2026-08-31
  - id: build-wasm
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.wasm.roc
    title: Wasm apply render_page
    author: process:git
    last_modified: 2026-08-19
  - id: views
    resource: ../../../crates/rocci-rocdown/runtime/Views.roc
    title: "Page(a) and nominal NavGroupView"
    author: process:git
    last_modified: 2026-08-31
  - id: emit
    resource: ../../../crates/rocci-rocdown/src/plan/emit.rs
    title: "pages : List(Views.Page(_))"
    author: process:git
    last_modified: 2026-08-31
  - id: metrics
    resource: ../../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: MetricsSpan note tag
    author: process:git
    last_modified: 2026-08-31
  - id: notes-main
    resource: ../../../examples/rocci/custom/notes/main.roc
    title: App-edge Try({}, [..]) annotations
    author: process:git
    last_modified: 2026-08-31
  - id: tests
    resource: ../../../crates/rocci-rocdown/src/plan/tests.rs
    title: Views staging and missing-children smoke
    author: process:git
    last_modified: 2026-08-31
---

# Align apply Roc with nightly type idiom

## Purpose and authority

This plan executes the [post-landing nightly typing review](/research/rocdown/idiomatic-roc-nightly-types.md).
[Named view types](/plans/rocdown/named-roc-view-types.md) already
landed in `b3129a9a`. This record is the leftover idiom work. It is
exploratory. Writing it does not start a phase.[^research][^named-plan][^incident]

## Goal

Generated and hand-written apply Roc uses the same type forms as
`nightly-2026-08-23-fb208ba` and the custom app mains:

1. effectful apply is `=> Try({}, [..])` (or `Try({}, _)`), not `=> _`;
2. wasm apply ascribes `Views.Page(_)` with a pure `->`;
3. inspector `MetricsSpan.note` is `[Some(Str), None]`.

Keep `Page(a)` / `Page(_)` and `NavGroupView :=`. Generated pages
already ascribe `List(Views.Page(_))`.[^views][^build-roc][^emit][^tests]

## Out of bound

- Changing the Zig Roc compiler.
- Switching `NavGroupView` back to an alias.
- Making non-recursive view types nominal.
- Emitting `Views.NavGroupView.{ … }` constructors (anonymous records
  already unify).
- Naming the segment union in `Views.roc`.
- Annotating `NavList` / `SiteShell` helpers.
- `OkfPages.roc`, apply-data hash, or moving `PageView` values out of
  `RocdownPages.roc`.
- Rewriting `test/EmbeddedLanguages.rocci` (old-syntax scanner fixture).
- Pattern `??` / type-position defaults (existing defaults plan).

## Constraints that do not move

| Constraint | Required behavior |
| --- | --- |
| Nightly pin | Type applications use parentheses. Recursive types use `:=`.[^research] |
| Two layers | Rust catalog owns data. Named Roc types describe the hand-off.[^views] |
| Field names | Same as `view.rs`. |
| Failed builds | A type error keeps the previous output tree. |
| Local style | Leave chrome helpers inferred. |
| Effect vs pure | `write_page!` stays `=>`. Wasm `render_page` stays `->`.[^build-roc][^build-wasm] |

## Phases

### Phase 1 — apply-edge Try

**Bound:** In `RocdownBuild.roc`, change

`write_page! : Str, Views.Page(_) => _`

and `write_all!` to `=> Try({}, [..])`, matching
`examples/rocci/custom/notes/main.roc`. Keep `Page(_)`. Do not change
function bodies.[^build-roc][^notes-main]

**Out of bound:** Wasm file. Emit. Inspector.

**Exit:** `cargo test -p rocci-rocdown plan::` and
`cargo fmt --all -- --check`. If `ROCCI_REQUIRE_ROC=1` is already used
for `missing_nav_group_children_names_the_field`, that test stays green.

### Phase 2 — wasm apply ascription

**Bound:** `RocdownBuild.wasm.roc` imports `Views` and annotates
`render_page : Views.Page(_) -> _` (or the concrete `Str` if
`Html.render_document` is already `Str`) and
`render_all : List(Views.Page(_)) -> _`. No `=>`. Do not add Path I/O.[^build-wasm]

**Out of bound:** Changing wasm host or `--host wasm` flags.

**Exit:** Same planner tests. Runtime staging still writes `Views.roc`
next to the wasm build file when that path already stages it; if wasm
staging does not copy `Views.roc`, add only the copy required for the
import.

### Phase 3 — inspector tag payload

**Bound:** In `MetricsPanel.rocci`, replace `[Some Str, None]` with
`[Some(Str), None]`. Update any construction/match arms in that file
that assume juxtaposition.[^metrics]

**Out of bound:** Inspector UX. Other chrome.

**Exit:** `cargo fmt --all -- --check`. The narrowest existing
rocci-cli / inspector test that compiles that module, or
`cargo test -p rocci-cli` if no narrower target is documented.

### Phase 4 — pointer

**Bound:** One sentence in `crates/rocci-rocdown/README.md` that
`write_page!` is `Views.Page(_) => Try({}, [..])` and that recursive
nav is nominal `NavGroupView`. Point the [named-view plan](/plans/rocdown/named-roc-view-types.md)
at this record as the leftover. No public language-reference rewrite.

**Out of bound:** Author-skill essays.

**Exit:** README sentence exists. `okmate check knowledge --profile base`
if this plan or the research is edited in the same change.

## Follow-on (not scheduled)

- `Views.NavGroupView.{ … }` in `pages_roc()` if ascription is ever
  dropped.
- The same Try / `Page(_)` forms on a future `OkfPages.roc`.

[^research]: Nightly probes; `=> _` and wasm are the leftover apply gaps.
[^incident]: Why apply needed a named contract.
[^named-plan]: First cut landed in b3129a9a.
[^build-roc]: Current `=> _` annotations.
[^build-wasm]: Untyped wasm render.
[^views]: Keep `Page(a)` and `NavGroupView :=`.
[^emit]: Generated ascription already uses `Page(_)`.
[^metrics]: `[Some Str, None]` is illegal juxtaposition.
[^notes-main]: App-edge `Try` style to copy.
[^tests]: Existing Views and missing-children coverage.
