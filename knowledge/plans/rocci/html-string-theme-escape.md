---
type: Implementation Plan
title: Decide whether theme and string Html split/join escaping deserves a kernel change
description: "string_scan_escape failed the Phase 2 small-case screen. Theme painters and Rocdown/ui string Html still use repeated split/join. Measure that path on its actual consumers, then close it or file a separate implementation plan. Do not switch painters to Html.Node."
tags: [domain/rocci, domain/rocdown, concern/rendering, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T13:53:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: investigation
    resource: ./compile-time-template-preparation.md
    title: Parent investigation; theme Str coverage absent
  - id: scan-copy
    resource: ./html-scan-copy-escape.md
    title: Node kernel follow-up; string Html is out of its bound
  - id: host-coverage
    resource: ./html-scan-copy-host-coverage.md
    title: HTTP/Linux coverage for Node Html, not Str painters
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: string_scan_escape failed the 15%/5% screen
  - id: phase-2-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-2-results.json
    title: 3.7% large-case gain and 34% empty-card regression versus current string
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: Split/join escape used by the string comparison backend
  - id: rocdown-html
    resource: ../../../crates/rocci-rocdown/runtime/Html.roc
    title: Same split/join helper staged for theme painters
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: compile_single_module sets html_type Str
  - id: prior-html-plan
    resource: ./html-node-lowering.md
    title: NavList one-shot status quo; painters already annotate Str
---

# Decide whether theme and string Html split/join escaping deserves a kernel change

Exploratory follow-up from [template preparation and Html runtime costs](/plans/rocci/compile-time-template-preparation.md).
Do not start a phase until the user asks. Not an approved Decision. Closing
with no change is a valid Exit.[^investigation][^research]

## Goal

Determine whether scan/copy (or another single-pass escape) on **string** Html
helps the consumers that actually select `Str`, after Phase 2's Card string
variant failed the small-case screen. Name one owner if a later
implementation is warranted, or record status quo.[^phase-2-receipt][^theme]

## Out of bound

- Porting `node_scan_escape` or changing platform Node Html.[^scan-copy]
- HTTP/Linux Node coverage.[^host-coverage]
- Switching theme painters to `Html.Node`, dual emit modes, fusion, or
  unifying the two Html.roc backends.[^prior-html-plan]
- Prepared-template libraries or `.rocci` grammar.
- Treating the Card string backend as a theme painter measurement.

## Constraints that do not move

1. Theme compilation keeps `html_type: "Str"`.[^theme]
2. String escape must not invent `&#13;` attribute semantics the string
   contract does not already have.[^ui-html]
3. `crates/rocci-ui/runtime/Html.roc` and
   `crates/rocci-rocdown/runtime/Html.roc` are two copies of the same
   split/join helper; a later product change must list both or justify
   touching only one.[^ui-html][^rocdown-html]
4. Experiments copy Html only in a work directory. Preserve earlier receipts.

## Phase 0 — Name the real Str consumers

**Bound**

- Trace who imports the ui copy versus the Rocdown runtime copy versus
  inline string constructors from `html_type: "Str"` lowering.
- One actual painter (DocsComponents or RocdownTheme) and one ui consumer
  if any remain besides the experiment's `rocci-string` backend.
- Record CR/quote behavior on the string contract (LF in attributes after
  parse) so a later kernel cannot silently adopt Node `&#13;`.

**Exit:** owner table (path, import, Html type) and the promised string
escape contract.

## Phase 1 — Remeasure isolated string escape on a painter-shaped workload

**Bound**

- Isolated kernel: current split/join versus scan/copy (no CR encoding),
  same as Phase 2 string kernels, plus one painter-shaped render (nested
  chrome, scoped CSS text, ordinary attributes).
- Timing: process totals are noisy on empty Cards; prefer enough work that
  the delta exceeds a stated noise floor, or separate setup from render if
  the pin allows it.
- Screen (investigation filter, not an SLA): repeatable ≥15% on the
  painter-shaped case, no repeatable >5% regression on a small case after
  noise. Record any departure.

**Exit:** keep status quo, or evidence that a string kernel would pass the
screen on a real Str consumer.

## Phase 2 — Close or file an implementation plan

**Bound**

- If the screen fails or owners are unused: close. Update the parent
  investigation recommendation. Do not add more string variants.
- If it passes: file a small implementation plan naming exact files,
  tests, docs, revert (restore split/join), and that Node Html stays
  independent.

**Exit:** a recorded stop or a new implementation-plan path. This plan
does not itself edit product Html.

## Validation

- Experiment receipts under `knowledge/research/rocci/`
- `okmate check knowledge --profile base`
- `git diff --check`

[^investigation]: Theme Str and string_scan failure were left as coverage gaps.
[^research]: Phase 2 selected Node scan/copy, not the string variant.
[^scan-copy]: Node product port; string Html is out of that bound.
[^host-coverage]: Preview HTTP origin for Node, not painters.
[^phase-2-receipt]: string_scan_escape large gain 3.7%; empty-card regression ~34%.
[^ui-html]: Experiment string backend; split/join `escape`.
[^rocdown-html]: `include_str!` copy staged for Rocdown theme compile.
[^theme]: `html_type: "Str"` in `compile_single_module`.
[^prior-html-plan]: Unification and fusion stay skipped.
