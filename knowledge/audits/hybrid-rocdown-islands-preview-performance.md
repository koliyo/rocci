---
type: Audit
title: hybrid-rocdown-islands preview performance audit
description: Profiled `rocci-okf run knowledge/plans/hybrid-rocdown-islands.md` with the new headless rebuild profiler and traced perceived slowness to full-bundle `okf::load`, especially Rocci-profile source provenance checks.
tags: [domain/okf, domain/rocci, concern/performance, concern/tooling, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-19T11:25:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
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
    title: OKF bundle load path
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

1. **Add sub-spans inside `okf::load`.** Split Markdown parse, graph resolve,
   and provenance validation so future reports do not collapse ~10s into one
   opaque `load` bucket.[^okf-load][^headless-audit]

2. **Batch or cache git provenance lookups.** Reuse repository-root discovery
   and memoize `(repository, relative_path)` git results within a process.
   [^okf-validate][^headless-audit]

3. **Offer a bounded concept preview path.** If single-record latency matters,
   investigate loading only the target concept plus required indexes for
   navigation, rather than the full bundle on every rebuild.[^okf-preview]
   [^okf-load]

4. **Document the fast local workflow.** For draft plan authoring, prefer
   `rocci-okf run … --profile base --profile-report terminal` when provenance
   warnings are not the subject under review.[^okf-main]

[^okf-main]: `Run` accepts `--profile-report` and forwards it to the headless rebuild path.
[^okf-dev]: `rebuild_site` records a top-level `load` span around `okf::load`, then appends generator spans; successful rebuilds emit the combined snapshot.
[^okf-preview]: `resolve_preview_path` returns the bundle root plus a concept open path; it does not narrow bundle loading.
[^okf-load]: `okf::load` discovers every Markdown file, parses concepts, resolves the graph, and runs Rocci-only lifecycle and source validation.
[^okf-validate]: `validate_lifecycle_and_sources` iterates concept sources and invokes git subprocesses for tracked, dirty, and untracked relative paths.
[^headless-audit]: Prior bundle-level audit with the same profiler, showing the same load-dominated breakdown and Rocci-versus-base gap.
[^target-plan]: The hybrid islands plan cites 28 relative sources across crates, docs, decisions, and sibling knowledge records.
