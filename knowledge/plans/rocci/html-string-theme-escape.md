---
type: Implementation Plan
title: Decide whether theme and string Html split/join escaping deserves a kernel change
description: "Phases 0–2 on main: named Str owners; painter screen passed (~21% on RocdownTheme siteShell). Filed html-string-scan-copy-escape.md. Product Html unchanged in this plan. Do not switch painters to Html.Node."
tags: [domain/rocci, domain/rocdown, concern/rendering, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T14:35:00Z }
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
    title: string_scan_escape failed the 15%/5% screen; CR-in-attribute LF after parse
  - id: phase-2-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-2-results.json
    title: 3.7% large-case gain and 34% empty-card regression versus current string
  - id: phase-0-receipt
    resource: ../../research/rocci/html-string-theme-escape-phase-0-results.json
    title: Owner table, identical ui/Rocdown Html.roc hashes, string escape contract
  - id: phase-1-receipt
    resource: ../../research/rocci/html-string-theme-escape-phase-1-results.json
    title: Painter screen passed (~21%); CR bytes equal; no product Html edit
  - id: theme-escape
    resource: ../../../roc/template-preparation-experiment/theme_escape.py
    title: Isolated string kernels plus RocdownTheme siteShell runner
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: Split/join escape used by playground, rocci test, and rocci-string
  - id: rocdown-html
    resource: ../../../crates/rocci-rocdown/runtime/Html.roc
    title: Byte-identical split/join helper staged for theme painters
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: compile_single_module sets html_type Str
  - id: docs-components
    resource: ../../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Actual Str painter; interpolates kind/title/aria
  - id: rocdown-theme
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Actual Str painter; nested chrome and scoped CSS
  - id: stage
    resource: ../../../crates/rocci-rocdown/src/build/mod.rs
    title: Site build calls runtime::stage_into for the Rocdown Html copy
  - id: playground
    resource: ../../../crates/rocci-cli/src/playground_html.rs
    title: Remaining ui consumer; stages rocci_ui::HTML_ROC
  - id: rocci-test
    resource: ../../../crates/rocci-cli/src/rocci_test.rs
    title: Stages ui Html.roc and rewrites Html annotations to Str
  - id: prior-html-plan
    resource: ./html-node-lowering.md
    title: NavList one-shot status quo; painters already annotate Str
  - id: string-kernel-plan
    resource: ./html-string-scan-copy-escape.md
    title: Filed product follow-up; both string Html.roc copies; Node stays independent
---

# Decide whether theme and string Html split/join escaping deserves a kernel change

Exploratory follow-up from [template preparation and Html runtime costs](/plans/rocci/compile-time-template-preparation.md).
Not an approved Decision. Closing with no change is a valid Exit.

**State:** draft; Phases 0–2 completed locally on `main`. Owners are
RocdownTheme and DocsComponents (`html_type: Str` over the Rocdown Html
copy) plus playground/`rocci test` over the identical ui copy. Isolated
string scan/copy (no CR encoding) passed the painter screen. Product
follow-up: [string scan/copy](./html-string-scan-copy-escape.md). This
plan did not edit product Html. Not hosted-CI complete.
[^investigation][^research][^phase-0-receipt][^phase-1-receipt][^string-kernel-plan]

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

**Outcome:** The only `html_type: "Str"` caller is theme
`compile_single_module`. The two string Html.roc files are byte-identical
(SHA-256 `7b083eaa…`). Site `build` stages the Rocdown copy. CLI Html.roc
is a Node wrapper and is not an owner.[^theme][^rocdown-html][^ui-html][^stage][^phase-0-receipt]

| Path | Import | Html type |
| --- | --- | --- |
| `crates/rocci-rocdown/templates/RocdownTheme.rocci` | `import Html`; staged Rocdown `Html.roc` | `Str` |
| `crates/rocci-rocdown/templates/DocsComponents.rocci` | same | `Str` |
| `crates/rocci-ui/templates/chrome/{NavList,Breadcrumbs,PageOutline}.rocci` | compiled as theme modules; same Rocdown copy | `Str` |
| `crates/rocci-rocdown/runtime/RocdownBuild.roc` | `import Html` | `Str` (identity render) |
| `crates/rocci-cli/src/playground_html.rs` | `rocci_ui::HTML_ROC` | default `Html`; string constructors |
| `crates/rocci-cli/src/rocci_test.rs` | `rocci_ui::HTML_ROC`; rewrite `Html` → `Str` | `Str` after rewrite |
| `roc/template-preparation-experiment/` | copies ui `Html.roc` as `rocci-string` | string constructors |

Actual painter for Phase 1: RocdownTheme `siteShell` (nested NavList /
Breadcrumbs / PageOutline, scoped CSS, ordinary attributes). Remaining ui
consumer besides `rocci-string`: playground HTML snapshot.[^rocdown-theme][^docs-components][^playground][^rocci-test]

Promised string escape contract: one helper for text and attributes
(`&` `&amp;`, `<` `&lt;`, `>` `&gt;`, `"` `&quot;`, `'` `&#39;`). No
`&#13;` / `&#10;`. A raw CR in an attribute is emitted raw and parses to
LF (U+000A). False `boolean_attribute` omits the attribute.
`render_document` inserts a newline after the doctype. A later kernel must
not adopt Node attribute CR/LF numeric references.[^ui-html][^research]

Islands stage the Rocdown string copy while annotating `Html.Node`. That
is a signature mismatch, not a second Str owner. Rust `rocci_ui::html::escape`
is crate-test only.

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

**Outcome:** `--theme-escape` copied string Html only into the work
directory. Split versus scan/copy (no `&#13;` / `&#10;`) were
byte-identical for small (61824), painter (165871), and a painter
page whose title is `a` then CR then `b` (165736). Isolated kernels (20k reps): scan ~63% faster on clean text
and ~55% on escaped text. Painter-shaped RocdownTheme `siteShell` (800
reps): split 0.215s, scan 0.170s, 20.6% gain. Small case 0.032s versus
0.031s, inside the stated 5ms noise floor (recorded 0% regression).
Screen passed (`string_kernel`). Stated departure: `rocci-template
build` default `embed_css: true` puts scoped CSS through `Html.text`;
product theme compile sets `embed_css: false`. Darwin arm64, Roc
`nightly-2026-09-03-62fcb65`. Product Html hashes unchanged.
[^theme-escape][^phase-1-receipt][^rocdown-theme]

## Phase 2 — Close or file an implementation plan

**Bound**

- If the screen fails or owners are unused: close. Update the parent
  investigation recommendation. Do not add more string variants.
- If it passes: file a small implementation plan naming exact files,
  tests, docs, revert (restore split/join), and that Node Html stays
  independent.

**Exit:** a recorded stop or a new implementation-plan path. This plan
does not itself edit product Html.

**Outcome:** Screen passed, so this plan files
[replace string Html split/join with scan/copy](./html-string-scan-copy-escape.md)
instead of closing. That follow-up names both string Html.roc files,
expects, crate README, revert (restore split/join), and that Node Html
stays independent. It is not started. Product Html is still split/join.
[^string-kernel-plan][^phase-1-receipt]

## Validation

- Experiment receipts under `knowledge/research/rocci/`
- `okmate check knowledge --profile base`
- `git diff --check`

[^investigation]: Theme Str and string_scan failure were left as coverage gaps.
[^research]: Phase 2 selected Node scan/copy, not the string variant; Phase 1 classified CR-in-attribute as LF after parse for string Html.
[^scan-copy]: Node product port; string Html is out of that bound.
[^host-coverage]: Preview HTTP origin for Node, not painters.
[^phase-2-receipt]: string_scan_escape large gain 3.7%; empty-card regression ~34%.
[^phase-0-receipt]: Local Phase 0 owner table; ui and Rocdown Html.roc hashes match; string contract excludes `&#13;`.
[^ui-html]: Split/join `escape`; playground, rocci test, and experiment string backend.
[^rocdown-html]: Byte-identical `include_str!` copy staged for Rocdown theme compile and site apply.
[^theme]: `html_type: "Str"` in `compile_single_module`.
[^docs-components]: DocsComponents Aside interpolates kind, title, and aria into class/text/attributes.
[^rocdown-theme]: RocdownTheme SiteShell nests chrome and scoped CSS; returns Str.
[^stage]: `build_loaded_with_host` calls `runtime::stage_into`.
[^playground]: `rocci render` stages `rocci_ui::HTML_ROC`.
[^rocci-test]: Rewrites `-> Html` to `-> Str` before `roc test`.
[^prior-html-plan]: Unification and fusion stay skipped.
[^phase-1-receipt]: Local Phase 1; painter gain 20.6%; small inside 5ms floor; CR bytes equal.
[^theme-escape]: `--theme-escape` runner; copies Rocdown string Html in the work dir only.
[^string-kernel-plan]: Product follow-up; both copies; no `&#13;`; Node Html independent; not started.
