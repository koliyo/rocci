---
type: Research Report
title: Apply Roc after named Views still mixes holes and nightly idiom
description: >-
  b3129a9a typechecks on nightly-2026-08-23-fb208ba. Recursive nav must
  be nominal. Remaining gaps are => _, untyped wasm apply, and one old
  tag payload.
tags: [domain/rocdown, integration/roc, concern/developer-experience, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-08-31T16:20:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: commit
    resource: https://github.com/koliyo/rocci/commit/b3129a9a355441755fab28f1c446b35b2c677278
    title: typecheck named Views against nightly Roc
    author: process:git
    last_modified: 2026-08-31
  - id: views
    resource: ../../../crates/rocci-rocdown/runtime/Views.roc
    title: Staged apply aliases and nominal NavGroupView
    author: process:git
    last_modified: 2026-08-31
  - id: build-roc
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: write_page! and write_all! annotations
    author: process:git
    last_modified: 2026-08-31
  - id: build-wasm
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.wasm.roc
    title: Wasm apply render_page without Views
    author: process:git
    last_modified: 2026-08-19
  - id: emit
    resource: ../../../crates/rocci-rocdown/src/plan/emit.rs
    title: Generated RocdownPages.roc ascription and nav literals
    author: process:git
    last_modified: 2026-08-31
  - id: tests
    resource: ../../../crates/rocci-rocdown/src/plan/tests.rs
    title: Missing-children roc check and pages_roc ascription
    author: process:git
    last_modified: 2026-08-31
  - id: view-rs
    resource: ../../../crates/rocci-ui/src/view.rs
    title: Rust PageView and NavGroupView field names
    author: process:git
    last_modified: 2026-08-31
  - id: nav-list
    resource: ../../../crates/rocci-ui/templates/chrome/NavList.rocci
    title: Inferred sidebar props
    author: process:git
    last_modified: 2026-08-31
  - id: metrics
    resource: ../../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: Inspector MetricsSpan note tag
    author: process:git
    last_modified: 2026-08-31
  - id: notes-main
    resource: ../../../examples/rocci/custom/notes/main.roc
    title: App-edge Try({}, [..]) annotations
    author: process:git
    last_modified: 2026-08-31
  - id: allsyntax
    resource: https://raw.githubusercontent.com/roc-lang/examples/main/examples/AllSyntax/main.roc
    title: Official AllSyntax aliases, nominals, opaques, and holes
    author: organization:roc-lang
  - id: nightly
    resource: https://github.com/roc-lang/roc/commit/fb208ba17ef1af6254c90a6715f423589a4bcb75
    title: nightly-2026-08-23-fb208ba
    author: organization:roc-lang
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Product Roc nightly pin
    author: process:git
    last_modified: 2026-08-31
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: Installer pin fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: incident
    resource: ./named-roc-view-types.md
    title: Anonymous page-view incident and first-cut recommendation
    author: process:cursor
    last_modified: 2026-08-31
  - id: plan
    resource: ../../plans/rocdown/idiomatic-roc-nightly-types.md
    title: Align apply Roc with nightly type idiom
    author: process:cursor
    last_modified: 2026-08-31
  - id: named-plan
    resource: ../../plans/rocdown/named-roc-view-types.md
    title: Name Rocdown page and nav view types
    author: process:cursor
    last_modified: 2026-08-31
  - id: defaults
    resource: ../rocci/roc-nightly-record-defaults.md
    title: Type-position defaults versus pattern ??
    author: process:cursor
    last_modified: 2026-08-25
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Rocci author skill Roc-from-Rocci types
    author: process:git
    last_modified: 2026-08-31
  - id: author-idioms
    resource: ../../../.agents/skills/rocci-author/idioms.md
    title: Types on the pinned nightly
    author: process:git
    last_modified: 2026-08-31
---

# Apply Roc after named Views still mixes holes and nightly idiom

## Purpose and authority

This record reviews hand-written and generated apply Roc **after**
[b3129a9a](https://github.com/koliyo/rocci/commit/b3129a9a355441755fab28f1c446b35b2c677278)
against the product nightly. It is exploratory. Implementation:
[align apply Roc with nightly type idiom](/plans/rocdown/idiomatic-roc-nightly-types.md).[^plan][^commit]

The earlier [anonymous page-view research](/research/rocdown/named-roc-view-types.md)
explains the dump. Its “alias first” advice is wrong on this nightly.
First-cut plan: [named Roc view types](/plans/rocdown/named-roc-view-types.md).[^incident][^named-plan]

Probes used `/Users/nils/Projects/roc/roc_nightly-macos_apple_silicon-2026-08-23-fb208ba/roc`
(`roc check`, 2026-08-31) and basic-cli 0.22.[^nightly][^inventory][^install-roc]

## Nightly type forms that matter here

Official AllSyntax and the checker agree:[^allsyntax][^nightly]

| Form | Meaning | This nightly |
| --- | --- | --- |
| `Foo : { … }` | Structural alias | OK if not recursive |
| `Foo := { … }` | Nominal record | Required when the type refers to itself |
| `Foo :: { … }.{ … }` | Opaque | Hide fields; not needed for apply data |
| `Foo(a)` / `List(U8)` | Type application | Parentheses required |
| `Foo a` / `List U8` / `Page _` | Old juxtaposition | Parse error |
| `Type.{ field: v }` | Nominal constructor | Idiomatic; a matching `{ … }` also unifies when the expected type is known |
| `->` | Pure function | Components, wasm render |
| `=>` | Effectful function | `write_page!` |
| `=> Try({}, [..])` | Effect that returns `Ok({})` | Matches custom app `main!` / `init!`[^notes-main] |
| `=> _` | Type hole | Legal; weaker at a module edge |
| `=> {}` | Unit effect | Rejects `Ok({})` |

Recursive aliases are illegal. The diagnostic is explicit: use `:=`.[^nightly]

Record defaults (`field : Type ?? v`, `field ?: Type`) are a separate
contract and still do not allow pattern `??`.[^defaults]

## What b3129a9a shipped

The apply contract now exists and typechecks:[^commit][^views][^emit][^build-roc]

| Piece | Landed form | Nightly fit |
| --- | --- | --- |
| `Views.roc` | Aliases for `SiteView` … `PageView`; `NavGroupView := { …, children : List(NavGroupView) }`; `Page(a)` | Correct. Recursion forces nominal nav. Other views stay aliases. Field names match `view.rs`.[^view-rs] |
| `RocdownBuild.roc` | `write_page! : Str, Views.Page(_) => _` | `Page(_)` is required. `=> _` is a hole, not the app-edge idiom. |
| Generated `pages` | `import Views` and `pages : List(Views.Page(_))` | Correct ascription. Nested nav stays anonymous `{ title, …, children }`. That unifies with nominal `NavGroupView` when the expected type is known. |
| `previous` / `next` | Emit `class_name` | Required so the literal is `NavItemView`. |
| Missing-field smoke | `roc check` on a `NavGroupView` without `children` | Expected side prints `Views.NavGroupView`. It does not name `children`. The test accepts either. |

`NavList` and `SiteShell` stay inferred. That is still the right local
style.[^nav-list]

## What is still not idiomatic

1. **Effect return is a hole.** `=> Try({}, [..])` and `=> Try({}, _)`
   both `roc check` against `Views.Page(_)`. Custom apps already write
   `=> Try({}, [Exit(I64), ..])` / `=> Try({}, [..])`. Prefer that over
   `=> _`.[^build-roc][^notes-main][^nightly]
2. **Wasm apply is still anonymous.** `RocdownBuild.wasm.roc` has no
   `Views` import and no annotations. Pure `render_page` should use
   `->`, not `=>`.[^build-wasm]
3. **Generated nav is structural.** AllSyntax constructs nominals with
   `Type.{ … }`. Anonymous literals work here because `PageView.sidebar`
   is `List(NavGroupView)`. Constructors are polish, not a typecheck
   fix. Do not emit them unless a later phase wants dropped-ascription
   robustness.[^allsyntax][^emit]
4. **Inspector `MetricsSpan`** uses `[Some Str, None]`. Juxtaposed tag
   payloads parse-fail on this nightly (`Some(Str)`). That file is
   rocci-cli chrome, not apply, but it is shipped Roc.[^metrics]
5. **`EmbeddedLanguages.rocci`** still shows old `List Str` / `user ->`
   forms. It is a scanner fixture, not apply Roc. Leave it.

The first-cut goal (name the apply type so a missing nav field does not
reprint `segments`) is met. A missing `children` is now
`Views.NavGroupView` versus a four-field record.[^tests][^incident]

## What not to change

- Do not switch `NavGroupView` back to `:`. Recursion will not parse.
- Do not make `PageView` or `SiteView` nominal. They are not recursive.
- Do not name the segment union in `Views.roc`. `Page(_)` is the
  AllSyntax hole for “inferred from the value”.
- Do not annotate every `|item|` in `NavList`.
- Do not treat `=> {}` as the effect annotation for `Ok({})`.
- Do not fold OKF `OkfPages` or moving `PageView` *values* out of
  `RocdownPages.roc` into this typing pass.

## Recommendation

Keep the landed `Views` module. Tighten only the edges that the nightly
already has words for: `Try` on effectful apply, the same contract on
wasm render, and the one old tag payload in inspector chrome. Implementation:
[align apply Roc with nightly type idiom](/plans/rocdown/idiomatic-roc-nightly-types.md).
The rocci-author skill and idioms now carry the same alias / nominal /
parentheses table for authored Roc.[^plan][^author-skill][^author-idioms]

[^commit]: Commit that switched `Page a` → `Page(a)`, `Page _` → `Page(_)`, and `NavGroupView` to `:=`.
[^views]: `NavGroupView :=` plus `Page(a)`; remaining view types are aliases.
[^build-roc]: `write_page!` / `write_all!` use `Views.Page(_) => _`.
[^build-wasm]: Wasm render has no `Views` ascription.
[^emit]: `pages : List(Views.Page(_))`; sidebar groups are bare records with `children`.
[^tests]: Missing-children fixture must mention `children` or `NavGroup` and must parse.
[^view-rs]: Canonical snake_case fields for the Roc aliases.
[^nav-list]: `is_leaf_group` and `@for child in group.children` stay inferred.
[^metrics]: `note : [Some Str, None]` is juxtaposition.
[^notes-main]: `init!` / `respond!` / `shutdown!` use `=> Try(…, [..])`.
[^allsyntax]: `NominalTypeRecord := { x : U64 }`, `NominalTypeRecord.{ x: 42 }`, `List(a)`, `-> _`, opaques as `::`.
[^nightly]: Product compiler; recursive alias and `Page a` / `Page _` rejected in local `roc check`.
[^inventory]: `roc_nightly = "nightly-2026-08-23-fb208ba"`.
[^install-roc]: Installer defaults `2026-08-23` / `fb208ba`.
[^incident]: First-cut record still useful for the dump; alias-first is stale.
[^plan]: Follow-on phases for Try, wasm, and the inspector tag.
[^named-plan]: First-cut plan; phases landed in b3129a9a.
[^defaults]: Pattern `??` remains illegal; type-position defaults are a different plan.
[^author-skill]: `Roc used from Rocci` now forbids juxtaposition and `=> _` at exports.
[^author-idioms]: `Types on the pinned nightly` table and do/don't examples.
