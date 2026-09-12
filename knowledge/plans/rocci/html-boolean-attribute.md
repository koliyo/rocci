---
type: Implementation Plan
title: Omit boolean_attribute when the flag is false
description: "Make platform Html.boolean_attribute omit the attribute on False so presence matches HTML boolean semantics. Unreachable from current .rocci valueless attributes; independent of scan/copy escape work."
tags: [domain/rocci, concern/rendering, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-09-12T12:35:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: investigation
    resource: ./compile-time-template-preparation.md
    title: Phase 1 classified the helper drift
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: String omits false; nodes emit disabled="" for both branches
  - id: phase-1-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-1-results.json
    title: Hand-written False probe; .rocci only lowers True
  - id: platform-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: boolean_attribute constructs the same empty attribute in both branches
  - id: cli-html
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Wrapper that must stay consistent with platform Html
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: String helper already omits the attribute when false
  - id: lower
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Valueless .rocci attributes lower only to boolean_attribute(name, True)
---

# Omit boolean_attribute when the flag is false

Exploratory correctness repair from [template preparation and Html runtime costs](/plans/rocci/compile-time-template-preparation.md)
Phase 1. Do not start a phase until the user asks. Independent of
scan/copy escaping. Not an approved Decision.[^investigation][^research]

## Goal

`Html.boolean_attribute(name, False)` omits the attribute.
`Html.boolean_attribute(name, True)` still emits `name=""`. That matches
the string helper and HTML boolean presence semantics.
[^platform-html][^ui-html][^phase-1-receipt]

## Out of bound

- Changing `.rocci` valueless-attribute syntax or lowering, which already
  passes `True` only.[^lower]
- Scan/copy escape, fusion, unification, or theme `Str` Html.
- Treating this as a `.rocci` authoring bug.

## Constraints that do not move

1. True remains an empty attribute, not `name="true"` or `name="name"`.
2. Revert is restoring identical true/false constructor branches.

## Phase 0 — Repair the helper and test both branches

**Bound**

- Edit `boolean_attribute` in `crates/rocci-platform/platform/Html.roc`
  so the false branch returns no attribute. Mirror the CLI wrapper if it
  inlines the same helper.[^platform-html][^cli-html]
- Expects: True emits `disabled=""` (or the probed name); False omits it.
- Do not change generated `.rocci` goldens unless they call False
  (they should not).

**Exit:** helper expects pass; `cargo test -p rocci-template` unchanged
goldens; `roc fmt --check` on edited Roc.

## Validation

- Helper expects plus `cargo test -p rocci-template`
- `okmate check knowledge --profile base` after knowledge notes

[^investigation]: Phase 1 classified the drift; this plan is the repair scope.
[^research]: String omits false; nodes emit `disabled=""` for both branches.
[^phase-1-receipt]: Hand-written False probe; `.rocci` only lowers True.
[^platform-html]: Identical true/false constructor branches today.
[^cli-html]: Wrapper must not reintroduce the drift.
[^ui-html]: String helper already omits the attribute when false.
[^lower]: Valueless attributes lower only to `boolean_attribute(name, True)`.
