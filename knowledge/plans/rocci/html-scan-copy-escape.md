---
type: Implementation Plan
title: Replace node Html escaping with scan/copy and a no-escape fast path
description: "Port the measured node_scan_escape kernel into crates/rocci-platform/platform/Html.roc. Keep constructor lowering, CR numeric references in attributes, and the public Html API. Do not claim HTTP, Linux, or theme-Str benefit from the investigation."
tags: [domain/rocci, concern/rendering, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T12:35:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: investigation
    resource: ./compile-time-template-preparation.md
    title: Investigation that selected node_scan_escape
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: Cost table, representative fixtures, and coverage limits
  - id: phase-2-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-2-results.json
    title: Isolated escape kernels and Card timings
  - id: phase-4-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-4-results.json
    title: Hello/Card/Compat/Callout byte equality; HTTP/Linux absent
  - id: platform-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Current fold/concat escape_html_bytes
  - id: cli-html
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Wrapper that re-exports platform Html
  - id: lower
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Constructor lowering must stay unchanged
  - id: prior-html-plan
    resource: ./html-node-lowering.md
    title: Earlier status-quo decision for NavList-scale one-shot work
---

# Replace node Html escaping with scan/copy

Exploratory follow-up from [template preparation and Html runtime costs](/plans/rocci/compile-time-template-preparation.md).
Do not start a phase until the user asks. This is not an approved Decision.
[^investigation][^research]

## Goal

Replace `escape_html_bytes` in platform Node Html with a scan for whether
escaping is needed, then either return the original string or copy into a
pre-sized byte buffer. Preserve attribute `&#13;` / `&#10;` behavior and
every public constructor. Do not change generated Roc goldens.
[^platform-html][^lower][^phase-2-receipt]

## Out of bound

- `.rocci` grammar, a second emit mode, static-chunk fusion, or runtime
  unification.[^prior-html-plan]
- Theme painters (`html_type: Str`) and `crates/rocci-ui/runtime/Html.roc`
  split/join escaping.
- Prepared-template libraries, Templegen, or packaging.
- Claiming HTTP, Linux, or webview-origin speedups. Phase 4 did not obtain
  candidate HTTP or Linux coverage.[^phase-4-receipt]
- Changing `boolean_attribute` true/false branches (separate correctness
  repair).

## Constraints that do not move

1. Constructor lowering remains the product path.[^lower]
2. Attribute CR/LF stay numeric character references; text escaping stays
   `&`, `<`, `>` plus quotes when requested.[^platform-html]
3. Temporary experiment copies in `roc/template-preparation-experiment/`
   are not the product implementation.
4. Revert is restoring the fold/concat `escape_html_bytes` body.

## Phase 0 — Port the kernel and lock escape expects

**Bound**

- Change only `escape_html_bytes` in
  `crates/rocci-platform/platform/Html.roc`. If the CLI wrapper duplicates
  the kernel, change the wrapper the same way; prefer keeping one
  implementation.[^platform-html][^cli-html]
- Match the investigation kernel: no-escape fast path; `List.with_capacity`
  then `append` when escaping; same entity bytes as today.
- Add or extend Roc expects for clean text, `&<>"'`, CR/LF in attributes,
  and a no-escape string that must be returned unchanged.
- Do not regenerate template goldens unless a test proves emit changed
  (it must not).

**Exit:** platform Html expects pass; `cargo test -p rocci-template` still
passes without golden edits; `roc fmt --check` on the edited Roc.

## Phase 1 — Record the product change and its limits

**Bound**

- Platform README: Node text/attribute escaping uses scan/copy; not a new
  language or Html type.
- Knowledge: this plan's outcome; do not rewrite the September 9 NavList
  status-quo decision.[^prior-html-plan]
- Explicit remaining gaps: Linux, HTTP origin, theme `Str`.

**Exit:** docs and knowledge name the algorithm and the unmeasured hosts.

## Validation

- `roc` expects on `Html.roc` if the crate already runs them that way
- `cargo test -p rocci-template`
- `cargo fmt --all -- --check` if any Rust changed (none expected)
- Knowledge: `okmate check knowledge --profile base`

[^investigation]: Investigation selected this kernel; it did not ship it.
[^research]: Phase 2 cost table and Phase 4 fixture equality.
[^phase-2-receipt]: ~34% faster on the 100-row escaped Card versus current node escape.
[^phase-4-receipt]: Representative Node fixtures match; HTTP/Linux/theme coverage absent.
[^platform-html]: Product `escape_html_bytes` fold/concat implementation.
[^cli-html]: Wrapper imports; keep behavior aligned.
[^lower]: Generated Roc stays Html constructor calls.
[^prior-html-plan]: September 9 NavList one-shot status quo remains a historical outcome.
