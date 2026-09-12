---
type: Implementation Plan
title: Replace string Html split/join escaping with scan/copy
description: "Port the Phase 1 string scan/copy kernel (no CR encoding) into both ui and Rocdown Html.roc copies. Keep them byte-identical. Node Html, html_type Str, and constructor lowering stay independent. Do not start until asked."
tags: [domain/rocci, domain/rocdown, concern/rendering, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-09-12T14:35:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: investigation
    resource: ./compile-time-template-preparation.md
    title: Parent investigation; Card string_scan_escape failed the small-case screen
  - id: explore
    resource: ./html-string-theme-escape.md
    title: Owners named; painter screen passed; this plan is the product follow-up
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: String contract is raw CR (LF after parse); Node keeps &#13;
  - id: phase-1-receipt
    resource: ../../research/rocci/html-string-theme-escape-phase-1-results.json
    title: ~21% painter gain on RocdownTheme siteShell; CR bytes equal; no product Html edit
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: Split/join escape; playground and rocci test
  - id: rocdown-html
    resource: ../../../crates/rocci-rocdown/runtime/Html.roc
    title: Byte-identical copy staged for theme painters
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: compile_single_module keeps html_type Str
  - id: kernel
    resource: ../../../roc/template-preparation-experiment/theme_escape.py
    title: KERNEL_STRING_SCAN and patch_string_html; quotes always escaped
  - id: node-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Node scan/copy with &#13;/&#10;; must stay independent
  - id: rust-escape
    resource: ../../../crates/rocci-ui/src/html.rs
    title: Rust replace helper; crate-test only; not this kernel
  - id: ui-readme
    resource: ../../../crates/rocci-ui/README.md
    title: Where to name string Roc Html scan/copy
  - id: prior-html-plan
    resource: ./html-node-lowering.md
    title: Unification and fusion stay skipped
---

# Replace string Html split/join escaping with scan/copy

Exploratory product follow-up from [theme/string Html escaping](/plans/rocci/html-string-theme-escape.md).
The painter screen passed locally. This is not an approved Decision.
Do not start a phase until asked.[^explore][^phase-1-receipt][^investigation]

## Goal

Replace the split/join `escape` helper in **both** string `Html.roc` copies
with scan/copy and a no-escape fast path. Keep the string contract: one
helper for text and attributes; no `&#13;` / `&#10;`. Theme compilation
stays `html_type: "Str"`. Node Html stays independent.
[^ui-html][^rocdown-html][^theme][^node-html][^research]

## Out of bound

- Platform or CLI Node Html, including `&#13;` / `&#10;`.[^node-html]
- Changing `html_type`, switching painters to `Html.Node`, dual emit
  modes, fusion, or unifying the two Html backends.[^theme][^prior-html-plan]
- Editing Rust `rocci_ui::html::escape`.[^rust-escape]
- Prepared-template libraries or `.rocci` grammar.
- Treating the Card string backend or `embed_css: true` CLI default as
  the product theme compile path (`embed_css: false`).[^kernel]

## Constraints that do not move

1. List both `crates/rocci-ui/runtime/Html.roc` and
   `crates/rocci-rocdown/runtime/Html.roc`, and keep them byte-identical,
   or justify touching only one.[^ui-html][^rocdown-html]
2. Do not invent Node attribute CR/LF numeric references.[^research]
3. Revert is restoring the split/join `replace` / `escape` bodies in both
   files.
4. Experiments may copy Html only in a work directory.

## Phase 0 — Port the kernel and lock escape expects

**Bound**

- Replace `replace` / `escape` in both string Html.roc files with the
  Phase 1 kernel: scan for `&<>"'`, return the original string when none
  are present, otherwise copy into a pre-sized byte buffer. Quotes are
  always escaped. No CR/LF numeric refs.[^kernel][^ui-html][^rocdown-html]
- Add expects (in those files, or a one-file `roc test` app that imports
  a staged copy if a type module cannot be tested alone): clean-text
  identity, `&<>"'` entities, raw CR/LF in `attribute`, false
  `boolean_attribute` omits the attribute.
- Do not regenerate template goldens unless a test proves emit changed
  (it must not).

**Exit:** both files remain byte-identical; expects pass; `roc fmt
--check` on the edited Roc; `cargo test -p rocci-ui` still passes
without changing the Rust helper.

## Phase 1 — Record the product change and its limits

**Bound**

- `crates/rocci-ui/README.md`: string Roc Html uses scan/copy; not a new
  Html type; Node Html stays independent; CR stays raw.[^ui-readme]
- Name the Rocdown copy in knowledge (and the Rocdown README if it
  already discusses Html staging).
- This plan's outcome; do not rewrite the September 9 NavList status-quo
  decision or the Card `string_scan_escape` failure.[^prior-html-plan][^investigation]

**Exit:** docs and knowledge name the algorithm, both files, revert, and
that Node Html is unchanged.

## Validation

- Expects plus `roc fmt --check` on edited Roc
- `cargo test -p rocci-ui`
- `okmate check knowledge --profile base` after knowledge notes

[^investigation]: Card `string_scan_escape` failed the empty-card screen; this port uses the painter measurement instead.
[^explore]: Exploration closed by filing this path; it did not edit product Html.
[^research]: String CR-in-attribute is raw and parses to LF; Node `&#13;` is a different contract.
[^phase-1-receipt]: Local painter gain 20.6%; small inside 5ms floor; CR bytes equal.
[^ui-html]: Playground and `rocci test` stage this copy.
[^rocdown-html]: Theme compile and site apply stage this copy.
[^theme]: `html_type: "Str"` in `compile_single_module`.
[^kernel]: Isolated `KERNEL_STRING_SCAN`; `costs.patch_string_html` matches it.
[^node-html]: Platform `escape_html_bytes` already scan/copies with quote and CR/LF branches.
[^rust-escape]: Crate-test only; Phase 0 of the exploration named it as not a Str owner.
[^ui-readme]: README today names the Rust helper, not the Roc string kernel.
[^prior-html-plan]: Unification and fusion stay skipped.
