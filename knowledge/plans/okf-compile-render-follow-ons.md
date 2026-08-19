---
type: Implementation Plan
title: Deferred OKF compile and render follow-ons
description: Future work for the first three non-goals of the OKF compile/render plan. Reject hashing embedded page Roc out of the cache key; keep skip-roc off the default product path; do not embed the Roc compiler or wait on basic-cli wasm32.
tags: [domain/okf, domain/rocci-okf, integration/roc, concern/performance, concern/rendering, concern/caching, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:30:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: parent-plan
    resource: okf-compile-render-cost.md
    title: OKF preview compile and render cost plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../research/okf-compile-render-cost.md
    title: OKF preview compile and render cost after load-performance work
    author: process:cursor
    last_modified: 2026-08-19
  - id: results-status
    resource: ../status/okf-compile-render-cost.md
    title: OKF preview compile and render cost results
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-research
    resource: ../research/rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: generation-plan
    resource: rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: presentation
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF generate, compile hash, native apply writes, wasm main no-op
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run host auto does not force Roc
    author: process:git
    last_modified: 2026-08-19
  - id: okf-readme
    resource: ../../crates/rocci-okf/README.md
    title: rocci-okf usage contract
    author: process:git
    last_modified: 2026-08-19
  - id: roc-host
    resource: ../../crates/rocci-roc-host/src/host.rs
    title: Native apply and wasm32 compile
    author: process:git
    last_modified: 2026-08-18
  - id: roc-host-readme
    resource: ../../crates/rocci-roc-host/README.md
    title: Two-tier cache and embedded wasm platform
    author: process:git
    last_modified: 2026-08-18
  - id: engine-readme
    resource: ../../crates/okf/README.md
    title: Portable OKF engine boundary
    author: process:git
    last_modified: 2026-08-19
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell decision
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: deps-check
    resource: ../../scripts/check-workspace-deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
    last_modified: 2026-08-18
---

# Deferred OKF compile and render follow-ons

## Goal and scope

Record the first three [compile/render non-goals](okf-compile-render-cost.md)
as later work, with reopen gates. Those items were research options B and A,
then compiler/platform work. They stay out of the parent cut because the
durable path (external page data, apply writes Rocci chrome, watch reuses
apply) already landed.[^parent-plan][^research][^results-status]

This plan does not reopen dirty-page apply, Rocdown `RocdownPages.roc`,
parallel parse, or provenance. Exploratory; no phase started.

## Why these were deferred

Parent Phases 1–3 and 6 are in this tree. Page identity is `okf-pages.json`
plus article files; native apply writes `OkfTheme` HTML; default `run` host
auto does not force Roc. Remaining debug cost is cold `roc build` and
new-process first apply, not a Markdown-edit compile miss.[^results-status]
[^presentation][^okf-main]

Research option B (drop `OkfPages.roc` from the hash while it still inlines
`article_html`) was a shortcut that is only safe while apply output is
ignored. Apply now writes that HTML, so B would serve stale Rocci pages.
Option C already replaced it.[^research][^presentation]

Research option A (skip `roc build` on default `run` because apply was
unused) would now unship the Rocci shell on machines that have `roc`. Parent
Phase 6 already skips Roc when `roc` is missing; A as the long-term *default*
is a product inversion, not a cache fix.[^research][^okf-readme][^catalog-shell]

Speeding `roc`, linking compiler internals into Rust, or waiting for
`basic-cli` `wasm32` are compiler or platform projects. Rocci already has a
custom wasm platform; wasm `main.roc` is a no-op and does not write
staging.[^generation-research][^generation-plan][^presentation][^roc-host-readme]

## Constraints that do not move

| Keep | Meaning for this plan |
| --- | --- |
| `okf` portable | No Roc or presentation types in the engine |
| Rocci shell when Roc runs | Default preview with `roc` present still serves apply HTML |
| Whole-program Roc | No per-`.rocci` object cache |
| No compiler embed | `roc` stays a subprocess; no Zig internals in Rocci |
| Custom wasm platform | Do not reuse `basic-cli` as the wasm platform |

Those boundaries are the portable engine split, the catalog/shell decision,
and the generation-host contract already recorded.[^engine-readme]
[^catalog-shell][^generation-plan][^deps-check]

## Non-goals

- Dirty-page apply (parent Phase 4)
- Rocdown `RocdownPages.roc` out of hash (parent Phase 5)
- Parallel Markdown parse, bounded concept-path load, git provenance
- Making `--profile base` the fast preview path
- CI latency SLAs

## Delivery phases

Each phase is one mergeable change. Start only when its gate is met.

### Phase 1 — Keep embedded page Roc in the compile hash

**Gate:** A generate path again emits Roc that inlines bundle `article_html`
(or equivalent page literals), or a change proposes omitting such a module
from `compute_compile_hash`.

**Bound:** If generated workspace `.roc` contains populated page HTML or
per-concept view records, that source participates in the renderer hash.
Dropping it from the key while apply writes disk is forbidden. Prefer
external JSON/files (parent Phase 1) over re-embedding.[^research]
[^presentation][^generation-research]

The current tree already asserts wasm/native `main.roc` has no
`article_html:` field. Keep that class of test. If `OkfPages.roc` returns,
hash it until apply is discarded again.

**Owner:** `crates/rocci-okf/src/presentation.rs` hash and generate.

**Out of bound:** Changing apply writes. Hashing Rocdown pages Roc.

**Tests:** Fixture generate with only a body change still hits
`lookup_renderer`. A generated `.roc` that contains a non-empty article
literal is an input to `renderer_compile_hash`.

**Exit:** The option-B shortcut cannot land without failing a named test.

### Phase 2 — Explicit skip-roc host, not the default product

**Gate:** A product decision that some `run` users with `roc` on PATH still
want the Rust shell (CI without compile, measuring write-only, or machines
that must not invoke `roc`). Do not start in order to make default preview
faster while Rocci chrome remains the supported look.[^okf-readme]
[^catalog-shell][^results-status]

**Bound:** Add an explicit none/skip host (`--host none` or equivalent) that
calls `build_review_site_pure_rust` even when `roc` is available. Default
`run` (host auto, `roc` present) still uses the cached Rocci renderer.
`--host native` still forces Roc. Document the skip path as opt-in, not as
the authoring look.[^okf-main][^okf-readme][^presentation]

**Owner:** `crates/rocci-okf/src/main.rs` plus README.

**Out of bound:** Making skip-roc the default. Changing `--profile` meaning.

**Tests:** With `roc` on PATH, `--host none` never invokes `roc build` and
writes the Rust shell. Host auto with `roc` still uses apply when the
renderer is cached or built.

**Exit:** README states skip-roc is opt-in; default preview with Roc present
is unchanged.

### Phase 3 — Wasm apply-to-disk without embedding `roc`

**Gate:** Native first-apply page-in or the desire to ship a prebuilt
`components.wasm` matters more than another native subprocess, *and* wasm
preview must write the same Rocci tree native apply writes today. Do not
start as a Roc compiler-speed project.[^results-status][^generation-plan]
[^roc-host]

**Bound:** Give `--host wasm` the same staging contract as native: read
`okf-pages.json` and article files, write `OKF_STAGING` / `output_path`.
Do that through the existing embedded wasm platform (WASI file I/O or host
imports), not by waiting for `basic-cli` `wasm32`. Do not link Roc compiler
internals into Rust. Do not add a per-`.rocci` object cache.
[^generation-research][^generation-plan][^roc-host-readme][^presentation]

Speeding `roc` itself stays upstream. Glue/`dlopen` remains the generation
plan's later Host C, not this phase.

**Owner:** `crates/rocci-okf` wasm `main.roc` plus `rocci-roc-host` wasm
platform capabilities. Do not move presentation into `okf`.[^deps-check]

**Out of bound:** Changing native apply. Embedding the compiler. Contributing
`wasm32` to `basic-cli` as a Rocci-owned deliverable.

**Tests:** With wasm host, a built concept page contains the knowledge-shell
markers used by the native apply test. Compile hash still ignores Markdown
body.

**Exit:** `--host wasm` serves Rocci chrome from apply, not the Rust
`html_page` fallback, without `basic-cli` as the wasm platform.

## Layer map

| Concern | Owner |
| --- | --- |
| Hash invariant for embedded pages | `crates/rocci-okf/src/presentation.rs` |
| Skip-roc host flag | `crates/rocci-okf/src/main.rs` |
| Public skip-roc contract | `crates/rocci-okf/README.md` |
| Wasm apply-to-disk | `presentation.rs` wasm `main.roc` + `rocci-roc-host` platform |
| Compiler speed / embed | Upstream Roc; not this repo |

## Risks

- Reintroducing `OkfPages.roc` for wasm "because the platform has no Path"
  recreates option B unless that module stays in the hash *or* the platform
  grows file I/O (Phase 3).[^presentation][^research]
- An undocumented skip-roc default would publish Rust chrome while templates
  keep changing, which parent Phase 6 already called out as unsupported
  authoring look.[^parent-plan]
- Teaching wasm apply to write files through a filesystem API fights the
  generation plan's "string in/out, no filesystem" preference; prefer host
  imports if both work.[^generation-plan]

## Open questions

1. Is `--host none` worth a flag, or is "no `roc` on PATH" enough?
2. Should wasm apply pass page JSON through Wasmtime host functions instead
   of WASI files?
3. Does shipping a prebuilt `components.wasm` in the `rocci-okf` release
   archive remove the need for local `roc` on first open?

[^parent-plan]: Phases 1–3 and 6 in this tree; first three non-goals are option B, option A as default product, and compiler/platform work.
[^research]: Option B is a footgun once apply writes HTML; option A skips unused Roc; wasm32 via basic-cli is not a compile shortcut.
[^results-status]: Debug cold compile 1036ms / first apply 1010ms; warm cached compile 0 / first-apply 758ms; watch reuse is session-local.
[^generation-research]: No supported embed of the Roc compiler in Rust; Roc is whole-program; page data must stay out of generated Roc.
[^generation-plan]: Custom wasm platform required; do not reuse basic-cli as wasm; glue/dlopen is later Host C.
[^presentation]: Native main reads JSON and writes staging; wasm main calls `parse_pages("{}")` and writes nothing.
[^okf-main]: `preview_host(Auto)` is `None`; Native and Wasm still force Roc.
[^okf-readme]: Default run uses cached Rocci renderer when roc is on PATH.
[^roc-host]: Wasm compile is `roc build --target=wasm32` against the embedded platform.
[^roc-host-readme]: Native uses basic-cli; wasm uses the embedded WASI platform and Wasmtime.
[^engine-readme]: `okf` stays UI-neutral.
[^catalog-shell]: Catalog stays in Rust; visible shell is Rocci once apply writes.
[^deps-check]: Presentation stays out of `okf`.
