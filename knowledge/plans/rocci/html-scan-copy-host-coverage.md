---
type: Implementation Plan
title: Measure scan/copy Node escape on the preview HTTP origin and Linux
description: "Fill the Phase 4 coverage gap now that product Node Html uses scan/copy: compare fold/concat versus product Html through a kept Rocci workspace on 127.0.0.1, then on Linux if a host exists. Do not claim throughput from this plan."
tags: [domain/rocci, concern/rendering, concern/performance, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-09-12T13:40:00Z }
stale_after: 2026-10-12
authority: exploratory
owners: [human:nils]
sources:
  - id: investigation
    resource: ./compile-time-template-preparation.md
    title: Parent investigation; HTTP and Linux remain unproven
  - id: scan-copy
    resource: ./html-scan-copy-escape.md
    title: Product scan/copy kernel; HTTP/Linux still unmeasured
  - id: research
    resource: ../../research/rocci/compile-time-template-preparation.md
    title: Phase 4 fixture equality and documented host absence
  - id: phase-4-receipt
    resource: ../../research/rocci/compile-time-template-preparation-phase-4-results.json
    title: rocci build --output dropped the staged workspace; listen smoke failed
  - id: host
    resource: ../../../roc/template-preparation-experiment/host.py
    title: Phase 4 host runner and product-origin limitation
  - id: host-page
    resource: ../../../roc/template-preparation-experiment/HostPage.rocci
    title: View plus fragment fixture used for the origin smoke
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: --platform accepts only rocci; Roc rejects absolute pins
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: compile_standalone_input stages a TempDir then writes a process binary
  - id: platform-readme
    resource: ../../../crates/rocci-platform/README.md
    title: In-tree pin and native libhost
  - id: theme
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Theme painters are Str; out of bound here
---

# Measure scan/copy Node escape on the preview HTTP origin and Linux

Exploratory coverage for [scan/copy Node escape](/plans/rocci/html-scan-copy-escape.md).
Product Node Html now uses that kernel. This plan still does not claim HTTP
or Linux benefit, and it is not an approved Decision. Do not start a phase
until the user asks.[^investigation][^scan-copy]

## Goal

Learn whether `node_scan_escape` still matches original Node Html and still
matters once the renderer runs inside the in-tree Rocci platform on
`http://127.0.0.1`, and on Linux if a machine exists. End with a narrowed or
widened recommendation for the product port. Do not label basic-cli process
totals as HTTP performance.[^research][^phase-4-receipt]

## Out of bound

- Editing product `Html.roc` (that is the scan/copy implementation plan).
- Theme `Str` painters, `rocci-ui` / Rocdown string Html, or prepared
  templates.[^theme]
- Adding a user-facing `--keep-workspace` or extra `--platform` name unless
  measurement is otherwise impossible; prefer experiment-only staging.
- Throughput, load tests, fusion, unification, or a second emit mode.
- Claiming cross-platform benefit from macOS-only data.

## Constraints that do not move

1. Keep generated Roc identical between original and candidate; vary only a
   copied `platform/Html.roc`.[^host]
2. Roc platform specs stay relative or URL; no absolute pins.[^dispatch]
3. Compare full-page `GET /` and one fragment on the same origin used by
   preview (`127.0.0.1`). One low-load request path is enough.
4. Missing Linux or a failed listen is a recorded absence, not a zero result.
5. Work stays under `roc/template-preparation-experiment/` plus knowledge
   receipts. Preserve earlier receipts.

## Phase 0 — Keep a staged workspace whose pin can retarget a copied platform

**Bound**

Phase 4 failed because `rocci build --output` writes a process binary against
the in-tree pin and drops the `islands-build` TempDir, so a copied Html never
reaches `roc`.[^driver][^phase-4-receipt][^dispatch]

- Stage HostPage (or an equivalent small view + fragment) to a **kept**
  directory: generated `main.roc`, modules, assets.[^host-page]
- Copy `crates/rocci-platform/platform` (including native `libhost.a`) into
  the work tree; patch only `escape_html_bytes` on the candidate copy.
- Rewrite the staged pin to a relative path at that copy. Build with `roc`.
- Do not change product CLI unless this staging cannot be done from the
  experiment. If a CLI change is required, stop and record that as the
  finding rather than expanding scope.

**Exit:** two binaries or two `roc` invocations, original versus candidate,
from identical generated Roc, or an explicit staging blocker.

## Phase 1 — Low-load HTTP origin on macOS

**Bound**

- Listen on `127.0.0.1` with `ROC_BASIC_WEBSERVER_HOST` /
  `ROC_BASIC_WEBSERVER_PORT`. Kill process groups on timeout.
- `GET /` and `GET /card` (or the fragment route). Compare status and body
  bytes. Do not treat listen `PermissionError` as a renderer mismatch.
- Optional: one bounded timing of the same paths, separated from startup.
  No sweep.

**Exit:** byte-equal origin responses plus a yes/no on whether renderer
gain is visible on this path, or documented listen/build failure.

## Phase 2 — Linux, or narrow the recommendation

**Bound**

- Repeat Phase 1 on a deployment-relevant Linux target if one is available
  (CI runner or existing host). Record allocator/target differences.
- If no Linux host exists, keep the scan/copy product plan narrowed to
  macOS Node Html.[^scan-copy][^platform-readme]

**Exit:** Linux receipt, or an explicit absence that the product plan must
keep repeating.

## Validation

- Experiment: `--host` or a successor flag; harness faults still pass
- `okmate check knowledge --profile base`
- `git diff --check`
- Do not run `cargo test --workspace` unless product CLI staging changed

[^investigation]: Parent investigation closed locally; this fills its host gap.
[^scan-copy]: Product port remains macOS-Node until this plan reports otherwise.
[^research]: Phase 4 classified HTTP/Linux as uncovered, not as a failed renderer.
[^phase-4-receipt]: Binary existed; candidate Html was not in that process.
[^host]: Phase 4 runner copied Html for basic-cli fixtures only.
[^host-page]: Nested list, scoped CSS, view and fragment routes.
[^dispatch]: `resolve_platform_pin` accepts only `rocci`.
[^driver]: `compile_app_plan_with_opt` stages then `build_roc_server`.
[^platform-readme]: Native `libhost.a` is rebuilt, not committed.
[^theme]: Painters select `Str`; a different exploration owns that path.
