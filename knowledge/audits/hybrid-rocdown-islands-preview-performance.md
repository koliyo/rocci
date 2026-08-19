---
type: Audit
title: hybrid-rocdown-islands preview performance audit
description: Profiled `rocci-okf run knowledge/plans/hybrid-rocdown-islands.md`; after load-performance Phases 1–4, release first-open `load` is 290ms and watch rebuilds reuse unchanged parses.
tags: [domain/okf, domain/rocci, concern/performance, concern/tooling, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-19T12:15:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: results-status
    resource: ../status/okf-load-performance.md
    title: OKF load-performance improvement results
    author: process:cursor
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf CLI run entry and profile-report flag
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild spans and profile-report emission
    author: process:git
    last_modified: 2026-08-19
  - id: okf-preview
    resource: ../../crates/okf/src/preview.rs
    title: Single-concept preview path resolution
    author: process:git
    last_modified: 2026-08-18
  - id: okf-load
    resource: ../../crates/okf/src/lib.rs
    title: OKF bundle load path with discover/parse/graph/provenance timings
    author: process:git
    last_modified: 2026-08-19
  - id: okf-validate
    resource: ../../crates/okf/src/validate.rs
    title: Lifecycle and git-backed source provenance validation
    author: process:git
    last_modified: 2026-08-19
  - id: headless-audit
    resource: rocci-okf-headless-load-performance.md
    title: rocci-okf headless load-performance audit
    author: process:cursor
    last_modified: 2026-08-19
  - id: target-plan
    resource: ../plans/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands implementation plan
    author: process:cursor
    last_modified: 2026-08-19
---

# hybrid-rocdown-islands preview performance audit

## Scope

This audit measures why `rocci-okf run knowledge/plans/hybrid-rocdown-islands.md`
feels slow on the current revision, using the headless rebuild profiler added in
`rocci-okf run --no-window --profile-report {terminal,json}`.[^okf-main][^okf-dev]

Measurements below were taken locally on 2026-08-19 with a cached native
renderer. They are machine-local timings, not a portability contract. The code
paths and relative breakdown are the durable part.

## Command and profiling setup

```text
cargo run -q -p rocci-okf -- run knowledge/plans/hybrid-rocdown-islands.md \
  --no-window --port auto --profile-report json
```

The `--no-window` path emits a `ProfileSnapshot` to stderr on each successful
rebuild without opening the embedded preview window.[^okf-dev][^headless-audit]

For profile comparison without Roc compilation noise, isolated load timing used
the prebuilt release binary:

```text
time target/release/rocci-okf check knowledge --profile rocci --format terminal
time target/release/rocci-okf check knowledge --profile base --format terminal
```

## Key architectural finding

Passing a concept file to `rocci-okf run` only changes the browser open path.
`resolve_preview_path` maps the plan file to
`/plans/hybrid-rocdown-islands/`, but `rebuild_site` still loads the entire
knowledge bundle before generating the site.[^okf-preview][^okf-dev][^okf-load]

That means a single-record preview pays the same bundle load cost as
`rocci-okf run knowledge`.

## Measured rebuild timings

### Cached renderer, `--profile rocci`, concept path

Observed JSON report:

```json
{"total_ms":9750,"spans":[{"name":"load","duration_ms":9593},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":46},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":29},{"name":"write","duration_ms":78}]}
```

### Cached renderer, `--profile base`, concept path

Observed JSON report:

```json
{"total_ms":5992,"spans":[{"name":"load","duration_ms":5786},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":44},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":68},{"name":"write","duration_ms":90}]}
```

### Cached renderer, `--profile rocci`, whole bundle

Observed JSON report:

```json
{"total_ms":10324,"spans":[{"name":"load","duration_ms":10158},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":47},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":30},{"name":"write","duration_ms":85}]}
```

The concept-specific run is only slightly cheaper on `load` (9593ms versus
10158ms). Almost all of the whole-bundle delta stays inside `load`, with only
small increases in `generate`/`render` and `write`.

### Cold renderer, concept path

When the native renderer cache was cold, one concept-path run reported:

```json
{"total_ms":18932,"spans":[{"name":"load","duration_ms":17647},{"name":"compile templates","duration_ms":6},{"name":"generate","duration_ms":64},{"name":"compile","duration_ms":727},{"name":"render","duration_ms":399},{"name":"write","duration_ms":89}]}
```

First-open latency therefore combines a ~17s load with ~1.1s of Roc compile and
render work on this machine.

### Isolated load (`check`, release binary)

| Profile | Wall time |
| --- | --- |
| `--profile rocci` | 4.77s |
| `--profile base` | 0.24s |

Nearly all of the Rocci-profile overhead sits inside `okf::load`, not in later
preview stages.[^okf-load]

## Phase 1 load sub-spans (2026-08-19 follow-up)

After splitting the opaque `load` span, the same headless profiler lists
`discover`, `parse`, `graph`, and `provenance` beside the wall-clock `load`
span. `--profile base` omits `provenance`.[^okf-dev][^okf-load]

Command:

```text
cargo run -q -p rocci-okf -- run knowledge/plans/okf-load-performance.md \
  --no-window --port auto --profile-report json
```

### Debug `--profile rocci` (cold renderer)

```json
{"total_ms":11529,"spans":[{"name":"load","duration_ms":9963},{"name":"discover","duration_ms":0},{"name":"parse","duration_ms":5897},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":4065},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":53},{"name":"compile","duration_ms":1003},{"name":"render","duration_ms":397},{"name":"write","duration_ms":109}]}
```

### Debug `--profile base` (cold renderer)

```json
{"total_ms":8124,"spans":[{"name":"load","duration_ms":6380},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":6379},{"name":"graph","duration_ms":0},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":46},{"name":"compile","duration_ms":1035},{"name":"render","duration_ms":407},{"name":"write","duration_ms":252}]}
```

The Rocci-versus-base `load` gap is the `provenance` span (~4065ms here). Parse
stays in the same 6s-class bucket under both profiles; `discover` and `graph`
are sub-millisecond and round to 0–1ms in the snapshot. Debug
`check knowledge` wall time on this revision was 13.62s rocci versus 8.48s
base, consistent with a multi-second provenance tax on top of parse.[^okf-load]
[^okf-validate]

## Post-change baseline (Phases 1–4)

The dated before/after summary is
[OKF load-performance improvement results](../status/okf-load-performance.md).[^results-status]

The same headless command on this revision, after load-performance work:[^okf-dev]
[^okf-load]

### Release first-open (default `run`, no `--provenance`)

```json
{"total_ms":357,"spans":[{"name":"load","duration_ms":290},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":289,"note":"cache_hit=0 miss=53"},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":0},{"name":"compile templates","duration_ms":0},{"name":"generate","duration_ms":7},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":30},{"name":"write","duration_ms":30}]}
```

### Debug watch rebuild after one Markdown change

```json
{"total_ms":177,"spans":[{"name":"load","duration_ms":5},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":4,"note":"cache_hit=52 miss=1"},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":0},{"name":"compile templates","duration_ms":6},{"name":"generate","duration_ms":53},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":29},{"name":"write","duration_ms":84}]}
```

Release `check knowledge --profile rocci` is 0.40s versus 0.27s for `--profile
base`. Default `run` keeps Rocci schema and skips git provenance; pass
`--provenance` to turn it back on. Bounded concept-path loading was not
started because release first-open `load` is 290ms.

## Findings

1. **Perceived slowness is dominated by `load`.** On the cached concept-path
   run, `load` was 9593ms of 9750ms total (98.4%). Template compilation and
   downstream render/write together accounted for ~157ms once the renderer
   was cached.

2. **Single-concept `run` does not bound work to one record.** The CLI opens
   the right page, but still parses all concepts, resolves the graph, and runs
   Rocci-profile validation across the bundle.[^okf-preview][^okf-load]

3. **Rocci-profile provenance validation is the main incremental cost.** Switching
   from `--profile rocci` to `--profile base` reduced cached concept-path
   `load` from 9593ms to 5786ms (about 1.7x faster). Isolated `check knowledge`
   still showed a larger provenance-specific delta (4.77s to 0.24s), consistent
   with `validate_lifecycle_and_sources`, which shells out to git per relative
   source path.[^okf-load][^okf-validate]

4. **The bundle amplifies git subprocess overhead.** The current knowledge tree
   has on the order of 37 concepts and 368 relative `sources[].resource`
   entries. Validation can invoke up to two git commands per source
   (`git log -1 --format=%cI` and `git status --porcelain`), so a Rocci-profile
   load can reach hundreds of subprocess calls even when previewing one plan.

5. **The target plan is not special, but it is source-heavy.** The
   hybrid-rocdown-islands plan declares 28 relative sources, including many
   crate paths and sibling knowledge records.[^target-plan] That increases
   provenance work for this record once the bundle is loaded, but it does not
   explain the ~10s baseline by itself; the whole-bundle iteration dominates.

6. **Newly staged hybrid records add provenance warnings, not extra parse work.**
   During this audit the plan and its research sibling were staged but not yet
   committed. Rocci-profile validation emits `OKF4007` for untracked sources on
   those records. That still participates in the same per-source git loop rather
   than introducing a separate code path.[^okf-validate]

## Interpretation

`rocci-okf run knowledge/plans/hybrid-rocdown-islands.md` is slow because OKF
preview is implemented as whole-bundle load plus site generation, and the
default Rocci profile adds repository provenance checks on every rebuild.
Rendering is not the bottleneck once the Roc host cache is warm.[^okf-dev]
[^headless-audit]

The new `--profile-report` flag makes this visible without the preview window.
For local iteration on a single draft plan, `--profile base` is noticeably
faster on `load` in cached runs (about 1.7x for concept-path here), and for
isolated `check` it is dramatically faster. The tradeoff remains skipping Rocci
lifecycle/provenance warnings.[^okf-main][^headless-audit]

## Recommended next steps

The follow-up implementation plan is
[OKF load-performance improvements](../plans/okf-load-performance.md). These
items remain the audit's evidence-backed order of attack:

1. **Add sub-spans inside `okf::load`.** Implemented: JSON reports now split
   `discover`, `parse`, `graph`, and `provenance`. The remaining Rocci-versus-base
   gap is provenance, not parse.[^okf-load][^headless-audit]

2. **Batch or cache git provenance lookups.** Implemented: one dirty-status dump
   and one batched `git log` over unique paths.[^okf-validate][^headless-audit]

3. **Bounded concept preview.** Gated off after a 290ms release first-open
   `load`.[^okf-preview][^okf-load]

4. **Fast local workflow.** Default `rocci-okf run` uses Rocci schema without
   git provenance. `check --profile rocci` stays strict. `--profile base` is
   portable OKF, not the supported fast-preview path.[^okf-main]

[^okf-main]: `Run` accepts `--profile-report` and forwards it to the headless rebuild path.
[^results-status]: Dated Status snapshot of machine-local before/after timings after Phases 1–4.
[^okf-dev]: `rebuild_site` records a top-level `load` span around `okf::load`, then appends generator spans; successful rebuilds emit the combined snapshot.
[^okf-preview]: `resolve_preview_path` returns the bundle root plus a concept open path; it does not narrow bundle loading.
[^okf-load]: `okf::load` discovers every Markdown file, parses concepts, resolves the graph, and runs Rocci-only lifecycle and source validation.
[^okf-validate]: `validate_lifecycle_and_sources` iterates concept sources and now batches git status and log over unique relative paths.
[^headless-audit]: Prior bundle-level audit with the same profiler, showing the same load-dominated breakdown and Rocci-versus-base gap.
[^target-plan]: The hybrid islands plan cites 28 relative sources across crates, docs, decisions, and sibling knowledge records.
