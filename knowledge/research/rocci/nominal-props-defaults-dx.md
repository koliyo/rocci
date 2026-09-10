---
type: Research Report
title: Roc nominal-only ?? defaults force generated Props types
description: "Nightly 2026-09-03 rejects ?? on structural records. afb4d15e emits HelloProps := { … } so CI typechecks, but that unique nominal per defaulted component plus empty {} not absorbing defaults is a real DX cliff for <Hello /> and ordinary Roc callers."
tags: [domain/rocci, integration/roc, concern/developer-experience, concern/syntax, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-10T09:30:00Z }
stale_after: 2026-12-10
authority: exploratory
owners: [human:nils]
sources:
  - id: commit
    resource: https://github.com/koliyo/rocci/commit/afb4d15ea1203ee66b620dad96c8d718b9680109
    title: emit nominal Props types for Roc ?? defaults
    author: process:git
    last_modified: 2026-09-10
  - id: roc-pr-defaults
    resource: https://github.com/roc-lang/roc/pull/10320
    title: Optional + Default Record Fields
    author: organization:roc-lang
  - id: roc-pr-overhaul
    resource: https://github.com/roc-lang/roc/pull/10834
    title: Record fields unset syntax and defaulted-fields overhaul
    author: organization:roc-lang
  - id: roc-restrict
    resource: https://github.com/roc-lang/roc/commit/917e3717ad730a3f7e995c22368bac383835184f
    title: defaults restrict ?? to nominal backing records
    author: organization:roc-lang
  - id: roc-issue-empty
    resource: https://github.com/roc-lang/roc/issues/11271
    title: Empty record literal {} does not absorb nominal field defaults
    author: organization:roc-lang
  - id: ast
    resource: ../../../crates/rocci-template/src/ast.rs
    title: defaulted_props_type and Bool skip
    author: process:git
    last_modified: 2026-09-10
  - id: emitter
    resource: ../../../crates/rocci-template/src/lower/emitter.rs
    title: Emit HelloProps := before the component function
    author: process:git
    last_modified: 2026-09-10
  - id: lower-skip
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: Skip call-site fill when a props backing record exists
    author: process:git
    last_modified: 2026-09-10
  - id: html-fill
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: lower_props emits {} when no filled defaults remain
    author: process:git
    last_modified: 2026-09-10
  - id: wrap
    resource: ../../../crates/rocci-template/src/roc.rs
    title: wrap_type_module nests the body in Type := [].{ … }
    author: process:git
    last_modified: 2026-08-16
  - id: view-call
    resource: ../../../crates/rocci-cli/src/view.rs
    title: rocci show emits Foo.hello({}) for omitted defaults
    author: process:git
    last_modified: 2026-09-10
  - id: compile-test
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Snapshot expects HelloProps and hello({},)
    author: process:git
    last_modified: 2026-09-10
  - id: all-syntax
    resource: ../../../test/AllSyntax.rocci
    title: Hello with name default and empty <Hello /> call
    author: process:git
    last_modified: 2026-09-10
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Documented Hello default plus <Hello /> in StylePage
    author: process:git
    last_modified: 2026-09-10
  - id: site-theme
    resource: ../../../site/theme/Components.rocci
    title: Card Hero Badge with untyped required fields plus ??
    author: process:git
    last_modified: 2026-09-10
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Generated HelloProps contract
    author: process:git
    last_modified: 2026-09-10
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Public ?? lowering now names HelloProps
    author: process:git
    last_modified: 2026-09-10
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Product Roc nightly-2026-09-03-62fcb65
    author: process:git
    last_modified: 2026-09-10
  - id: old-research
    resource: ./roc-nightly-record-defaults.md
    title: Aug 23 probe of structural type-position defaults
    author: process:cursor
    last_modified: 2026-08-25
  - id: old-plan
    resource: ../../plans/rocci/roc-nightly-defaults.md
    title: Pin 2026-08-23 and emit structural type-position defaults
    author: process:cursor
    last_modified: 2026-08-25
  - id: named-views
    resource: ../rocdown/named-roc-view-types.md
    title: Prefer aliases except when recursion forces nominal
    author: process:cursor
    last_modified: 2026-08-31
  - id: rocci-test
    resource: ./rocci-test-syntax.md
    title: rocci test aliases fixtures and components, not nested types
    author: process:cursor
    last_modified: 2026-08-31
---

# Roc nominal-only ?? defaults force generated Props types

## Claim

Authored `.rocci` `|{ name ?? "World" }|` did **not** get worse. The
**generated Roc declarations** did. On `nightly-2026-09-03-62fcb65`, Roc
rejects `??` on structural records. `afb4d15e` restored typechecking by
emitting a unique nominal `HelloProps := { name : Str ?? "World" }` per
defaulted component. That is the language's required shape, not a Rocci
preference. Combined with empty `{}` not absorbing those defaults, the
HTML-like `<Hello />` path is a type error unless lowering fills fields or
emits `HelloProps.{}`.[^commit][^roc-pr-overhaul][^roc-issue-empty][^inventory]

This is exploratory. It is not a cutover plan.

## Timeline

| When | What |
| --- | --- |
| Merged 2026-08-12 | [roc-lang/roc#10320](https://github.com/roc-lang/roc/pull/10320) added type-position `??` on **structural** records. `hello : { name : Str ?? "Roc" } -> Str` and `hello({})` typechecked.[^roc-pr-defaults] |
| Probed 2026-08-25 | Rocci recorded that form on `nightly-2026-08-23-fb208ba` and planned to emit it instead of call-site fill.[^old-research][^old-plan] |
| Authored 2026-08-14, merged 2026-08-31 | [roc-lang/roc#10834](https://github.com/roc-lang/roc/pull/10834) / [917e371](https://github.com/roc-lang/roc/commit/917e3717ad730a3f7e995c22368bac383835184f): `??` is only legal on **direct fields of a `:=` backing record**. Aliases, inline annotations, and nested records get **Default Not Allowed In Structural Record**. Payoff stated in the PR: a default belongs to one named type; omission sites are explicit nominal constructions.[^roc-pr-overhaul][^roc-restrict] |
| 2026-09-03 | Rocci pin `nightly-2026-09-03-62fcb65` includes that overhaul.[^inventory] |
| 2026-09-10 | `afb4d15e` emits `HelloProps`. Richard Feldman files [#11271](https://github.com/roc-lang/roc/issues/11271): `total({})` does not unify with an all-defaulted nominal; `total(Config.{})` does.[^commit][^roc-issue-empty] |

The August research is still a valid probe of `fb208ba`. It is **not** a
description of the current pin.[^old-research]

## Current Rocci lowering

`.rocci` still authors `|{ name ?? "World" }|`. The compiler still strips
`??` from the pattern because Roc rejects pattern defaults.[^ast][^all-syntax]

When every first-record field is typed (inferred `Str`/`I64`, or authored)
and no default is `Bool`, lowering now emits:[^ast][^emitter][^template-readme][^components-ref]

```roc
HelloProps := { name : Str ?? "World" }
hello : HelloProps -> Html
hello = |{ name }| { … }
```

It then **stops filling omitted fields at call sites**, so `<Hello />`
becomes `hello({})`. `rocci show` with no `--arg` does the same
(`Foo.hello({})`). Tests snapshot both the nominal decl and the empty
call.[^lower-skip][^html-fill][^view-call][^compile-test][^styling]

`wrap_type_module` then nests that inside the file type:[^wrap]

```roc
Styling := [].{
    HelloProps := { name : Str ?? "World" }
    hello : HelloProps -> Html
    hello = |{ name }| { … }
}
```

A local `roc test` on 2026-09-10 against the product nightly showed that
**associated nominals inside `Type := [].{ … }` are legal**. They are not
the block-local `??` rejection from #10834. Callers see
`Styling.HelloProps`.[^wrap][^rocci-test]

## Probe (2026-09-10, `nightly-2026-09-03-62fcb65`)

`roc test` of throwaway type-module files (not checked in):

| Program | Result |
| --- | --- |
| `hello : { name : Str ?? "World" } -> Str` | **Default Not Allowed In Structural Record**. The default is dropped; `name` is required. |
| `Hello : { name : Str ?? "World" }` alias | Same diagnostic. |
| `HelloProps := { name : Str ?? "World" }`; `hello({ name: "Ada" })` | Pass. |
| `hello(HelloProps.{})` | Pass. |
| `hello({})` | Type mismatch: `{}` vs `HelloProps`. Confirms [#11271](https://github.com/roc-lang/roc/issues/11271). |
| `CardProps := { title : Str, href : Str ?? "" }`; `card({ title: "Hi" })` | Pass. Non-empty literals still absorb omitted **sibling** defaults. |
| `hello(greet_val)` where `greet_val : GreetProps` with the same shape | Type mismatch: `GreetProps` vs `HelloProps`. Same-shape defaults are **distinct types**. |
| `helloTest = { name: "Ada" }` then `hello(helloTest)` | Pass. An unannotated structural binding still unifies at a known expected type. |
| `emptyTest : HelloProps = {}` | Type mismatch, same as #11271. |
| `Outer := { inner : { name : Str ?? "World" } }` | Nested structural `??` rejected. |
| `waiting : Bool ?? True` on a nominal, including `SlotProps.{}` | Pass on this pin. `Bool.true` still does not exist. Rocci still skips **all** Bool defaults in type position because an earlier nightly crashed on `Bool.true`.[^ast] |

## Why this is a DX problem

Rocci's component pitch is HTML-like optional attributes on **structural**
props: write `|{ name ?? "World" }|`, call `<Hello />` or
`<Hello name={person.name} />`, and do not invent a type per widget. That
is also how fixtures and `rocci show` work today.[^all-syntax][^styling][^view-call]

Roc #10834 makes the **Roc** declaration of that idea illegal. A default
is now part of a named type's identity. Two widgets with `{ name : Str ??
"World" }` cannot share a value. Error messages and hover talk about
`HelloProps` / `Styling.HelloProps`, not `{ name : Str }`. That is the
opposite of the alias-first rule Rocci already chose for non-recursive
view records.[^roc-pr-overhaul][^named-views]

#11271 then breaks the empty construction that Rocci already emits. The
 nicest authored form (`all fields defaulted`, `<Hello />`) is exactly the
failing case. Mixed records such as `{ title, href ?? "" }` still work
when the call passes `title`, which is why most theme widgets have not
blown up yet.[^roc-issue-empty][^site-theme]

Ordinary Roc callers (custom `main.roc`, `rocci test` aliases, playground
snapshots) now have three spellings:

- `{ name: "Ada" }` — works when the expected type is `HelloProps`.
- `HelloProps.{}` or `Styling.HelloProps.{}` — required for the all-default
  empty case.
- `{}` — illegal for that type.

`rocci test` aliases fixtures and component functions off the type
module. It does **not** alias `HelloProps`, so a test that writes `hello({})`
cannot name the constructor without extra lowering.[^rocci-test]

Name pollution: every defaulted component gets `{Pascal}Props` in the
generated module. A user-written `HelloProps` in the same file collides.
Cross-file, the name is `Module.HelloProps`.

## Split lowering hides the blast radius

`defaulted_props_type` returns `None` (and call-site fill stays on) when
any required prop lacks a type, or any default is `Bool`.[^ast][^lower-skip]

So today:

| Authored params | Generated Roc | Empty call |
| --- | --- | --- |
| `|{ name ?? "World" }|` | `HelloProps := …`; no fill | `hello({})` — **type error** |
| `|{ tone : Tone ?? Neutral }, content|` | `BadgeProps := …` | `{ tone: Positive }` at calls that pass tone; `{}` would fail |
| `|{ title, href ?? "" }|` | No Props type; fill `href: ""` | Structural record; typechecks |
| `|{ waiting ?? True, note ?? "" }|` | No Props type; fill both | Structural record; typechecks |

Most `site/theme` and handler-matrix widgets take the fill path because
required fields are untyped or a Bool is in the defaults. The documented
Hello example and AllSyntax `<Hello />` take the Props path. The DX cliff
is concentrated on the components that were supposed to show defaults at
their best.[^site-theme][^styling][^all-syntax]

Two lowering strategies for one authored `??` is itself a maintenance and
docs problem. Authors cannot predict whether a sibling field's missing
type silently keeps call-site fill.

## What still works

- Authored `.rocci` `??` syntax and stripped patterns.
- `<Hello name={person.name} />` → `hello({ name: person.name })` against
  `HelloProps`.
- `<Card title="Hi" />` when Card never got a Props type (fill) **or**
  when it is `CardProps` and the literal is non-empty.
- Nested `HelloProps` inside `wrap_type_module`.
- Pattern `??` remains illegal in Roc; that is unchanged.[^old-research]

## Options (not a plan)

These are directions, not phases.

1. **Call-site fill for every omitted `??`.** Restore the pre-August-plan
   emit (`hello({ name: "World" })`). Authored `??` stays Rocci sugar. No
   `HelloProps` required. Survives #10834 and #11271. Generated calls get
   verbose again. This is the only Rocci-only fix that keeps structural
   empty calls.
2. **Keep `HelloProps`, emit `HelloProps.{}` (or `Module.HelloProps.{}`)
   at empty sites**, including `rocci show`. Authored tags stay nice.
   Generated Roc and custom-app callers still see a unique nominal per
   widget. #11271 can still surprise humans who write `{}`.
3. **Wait on Roc.** Restore structural `??` (unlikely; #10834 is
   intentional) and/or fix #11271 so `{}` absorbs. Rocci should not
   assume either lands.
4. **Do not put `??` on generated Roc at all.** Keep stripping and filling.
   Treat Roc's type-position defaults as unused until empty construction
   and structural identity match HTML-like props.

Mixing (1) and (2) by Bool/untyped heuristics should not remain the
product contract.

## What this is not

- A change to authored `@component` grammar.
- Proof that `rocci run` of Styling is red in hosted CI (offline tests
  snapshot strings; Roc-gated lanes were the reason for `afb4d15e`).
- A claim that Bool defaults still crash. `Bool ?? True` materialized
  through `SlotProps.{}` on this pin; the skip is leftover from
  `Bool.true`.[^ast]

[^commit]: `afb4d15e` changed emit, README, AllSyntax goldens, and the public components table to `HelloProps :=`.
[^roc-pr-defaults]: Structural type-position `??` shipped 2026-08-12.
[^roc-pr-overhaul]: Merged 2026-08-31. Restricts `??` to nominal backing records; empty `{}` absorption is a later bug, not that PR's stated goal.
[^roc-restrict]: Canonicalize diagnostic: defaults only on direct `:=` fields.
[^roc-issue-empty]: Filed 2026-09-10. `total({})` vs `total(Config.{})` on `nightly-2026-09-04-c125b82`.
[^ast]: `defaulted_props_type` / `component_props_backing_record`; Bool returns `None`.
[^emitter]: `lower_component` emits the `:=` line then annotates the function with the type name.
[^lower-skip]: Components with a backing record are omitted from `field_defaults`.
[^html-fill]: Empty attrs plus no remaining fill → `{}`.
[^wrap]: Every `rocci show` / `run` / playground / apply module is `Type := [].{ … }`.
[^view-call]: `build_component_call` for defaulted `name` and no args is `Foo.hello({})`.
[^compile-test]: `lowers_component_call_to_props_record` asserts `hello(\n                        {},`.
[^all-syntax]: `<Hello />` next to `<Hello name={person.name} />`.
[^styling]: StylePage calls `<Hello />` and `<Hello name="Foo" />`.
[^site-theme]: `Card` / `Hero` / `Badge` keep untyped required fields, so they never take the Props path.
[^template-readme]: Documents the two-line generated shape.
[^components-ref]: Public table: lowering emits `HelloProps := { name : Str ?? "Roc" }`.
[^inventory]: `roc_nightly = "nightly-2026-09-03-62fcb65"`.
[^old-research]: August probe of structural `{ name : Str ?? "Roc" }` on `fb208ba`.
[^old-plan]: Phase 2 exit was `<Hello />` lowers to `hello({})` against a structural annotation.
[^named-views]: Non-recursive view types stay aliases; recursion is the reason to go nominal.
[^rocci-test]: `format_test_aliases` copies fixture and component names only.
