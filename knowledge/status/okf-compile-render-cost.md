---
type: Status
title: OKF preview compile and render cost results
description: Machine-local debug `rocci-okf run --profile-report json` after Phases 1–3 and 6. Page data is outside the renderer hash; native apply writes Rocci chrome; watch reuses the apply path. Phases 4–5 skipped.
tags: [domain/okf, domain/rocci-okf, concern/performance, concern/rendering, concern/caching]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:15:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/okf/okf-compile-render-cost.md
    title: OKF preview compile and render cost plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../research/okf/okf-compile-render-cost.md
    title: OKF preview compile and render cost after load-performance work
    author: process:cursor
    last_modified: 2026-08-19
  - id: presentation
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF review site generate, hash, apply session, and write fallback
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Watch apply-session state
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run host policy
    author: process:git
    last_modified: 2026-08-19
  - id: okf-readme
    resource: ../../crates/rocci-okf/README.md
    title: rocci-okf usage contract
    author: process:git
    last_modified: 2026-08-19
---

# OKF preview compile and render cost results

## Snapshot date

2026-08-19.

These timings came from a debug `cargo run -p rocci-okf` on this machine with
an isolated `ROCCI_CACHE`. They are not a latency SLA and are not comparable
to the companion research record's release numbers without a release
remeasure.[^plan][^research]

Command:

```text
cargo run -q -p rocci-okf -- run knowledge/research/okf/okf-compile-render-cost.md \
  --no-window --port auto --profile-report json
```

## What shipped

Phases 1–3 and 6 of the [compile/render plan](/plans/okf/okf-compile-render-cost.md)
are in this tree:[^plan][^presentation][^okf-dev][^okf-main]

- Compile hash is `compute_gen_hash` / `compute_compile_hash` over templates,
  `Html.roc`, `OkfBuild.roc`, and `main.roc`. Page identity is `okf-pages.json`
  plus article files. A Markdown body change does not miss the renderer cache.
- Native apply writes `OkfTheme.knowledgeShell` HTML to `OKF_STAGING`. Rust
  `html_page` remains only when apply omitted a path.
- Watch keeps the cached apply path in session state and marks `render` as
  `reuse` when the compile hash is unchanged.
- Default `run` (host auto) does not force Roc. Missing `roc` uses the Rust
  shell unless `--host native` or `ROCCI_REQUIRE_ROC=1`.[^okf-readme]

## Measured debug profiles

| Path | Total | Compile | Render | Notes |
| --- | --- | --- | --- | --- |
| Cold first open (isolated cache) | 3456ms | 1036ms | 1010ms | `parse` miss=70; first `roc build` |
| Warm process, cached renderer | 1035ms | 0 (`cached`) | 758ms | parse hit=70; first apply of this process |

First apply of a freshly mapped 4.8MB binary is still in the hundreds of
milliseconds in this debug host. That is a new-process page-in cost, not a
compile miss.[^research]

Same-process watch reuse is covered by the session test
(`watch_session_reuses_apply_without_roc_build`): the second rebuild reports
`compile` 0 (`cached`) and `render` note `reuse` without invoking `roc
build`.[^okf-dev][^presentation]

## Gated phases

- **Phase 4 (dirty-page apply)** skipped. The gate is a one-file watch edit on
  a warm cached binary. Same-process session reuse already avoids a second
  `roc build`; dirty-page apply would target remaining whole-bundle apply
  cost if a later watch remeasure stays in the first-run class.[^plan]
- **Phase 5 (Rocdown pages-out-of-hash)** skipped. Optional and owned by
  `rocci-rocdown`; this cut does not change `RocdownPages.roc`.[^plan]

[^plan]: Phases 1–3 and 6 in this tree; 4–5 gated and skipped on this revision.
[^research]: Pre-change release watch still compiled on body edits because `OkfPages.roc` was in the hash; apply HTML was discarded.
[^presentation]: `renderer_compile_hash` ignores bundle Markdown; native apply writes staging; `ApplySession` reuses the apply path.
[^okf-dev]: Watch `rebuild_site` holds `ApplySession` across ticks.
[^okf-main]: `preview_host(Auto)` is `None`, so missing `roc` uses the Rust write path.
[^okf-readme]: Documents cached Rocci renderer vs `--host native` vs no-Roc fallback.
