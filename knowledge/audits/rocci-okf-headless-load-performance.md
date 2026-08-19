---
type: Audit
title: rocci-okf headless load-performance audit
description: Measured headless `rocci-okf run --no-window` rebuild timings, CLI profile-report path, and Phase 1 load sub-spans showing the Rocci-versus-base gap is provenance.
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
    title: rocci-okf CLI entry points and run flags
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild path and profile-report emission
    author: process:git
    last_modified: 2026-08-19
  - id: dev-server
    resource: ../../crates/rocci-cli/src/dev_server.rs
    title: Shared static dev server profile snapshot storage and endpoints
    author: process:git
    last_modified: 2026-08-19
  - id: okf-load
    resource: ../../crates/okf/src/lib.rs
    title: OKF bundle load path with discover/parse/graph/provenance timings
    author: process:git
    last_modified: 2026-08-19
  - id: okf-validate
    resource: ../../crates/okf/src/validate.rs
    title: Lifecycle and source-provenance validation, including git subprocess checks
    author: process:git
    last_modified: 2026-08-19
  - id: cli-plan
    resource: ../plans/cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-18
---

# rocci-okf headless load-performance audit

## Scope

This audit measures the current headless rebuild path for `rocci-okf` and
records the smallest code change needed to make that path observable from the
CLI: `rocci-okf run ... --no-window --profile-report {terminal,json}`.[^okf-main][^okf-dev]

The measurements in this record came from local command execution on the
current revision and are machine-local rather than a portability contract.
The code paths and conclusions below are the durable part.[^okf-dev][^okf-load]

## Current headless profiling path

`rocci-okf run` already supported `--no-window`, but before this audit a
headless run only printed the served URL and left the rebuild timings inside
the dev-server inspector endpoint. This revision adds an explicit profile
report mode at the `rocci-okf` CLI boundary and emits each successful rebuild's
`ProfileSnapshot` to stderr without requiring the embedded preview
window.[^okf-main][^okf-dev][^dev-server]

That makes CLI-only performance work possible:

```text
cargo run -q -p rocci-okf -- run knowledge --no-window --port auto --profile-report json
cargo run -q -p rocci-okf -- run knowledge --no-window --port auto --profile base --profile-report json
```

## Measured rebuild timings

The most useful comparison is the cached renderer case, because it removes Roc
compilation noise from the result.

### Cached `--profile rocci`

Observed JSON report:

```json
{"total_ms":8718,"spans":[{"name":"load","duration_ms":8560},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":40},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":36},{"name":"write","duration_ms":78}]}
```

### Cached `--profile base`

Observed JSON report:

```json
{"total_ms":5276,"spans":[{"name":"load","duration_ms":5142},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":39},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":25},{"name":"write","duration_ms":66}]}
```

## Phase 1 load sub-spans (2026-08-19 follow-up)

`rebuild_site` now maps `okf::load_timed` durations onto named snapshot spans.
`--profile rocci` emits `discover`, `parse`, `graph`, and `provenance`;
`--profile base` omits `provenance`. Sub-spans are listed beside wall-clock
`load` and are not added into `total_ms`.[^okf-dev][^okf-load]

Command:

```text
cargo run -q -p rocci-okf -- run knowledge/plans/okf-load-performance.md \
  --no-window --port auto --profile-report json
```

### Debug `--profile rocci`

```json
{"total_ms":11529,"spans":[{"name":"load","duration_ms":9963},{"name":"discover","duration_ms":0},{"name":"parse","duration_ms":5897},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":4065},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":53},{"name":"compile","duration_ms":1003},{"name":"render","duration_ms":397},{"name":"write","duration_ms":109}]}
```

### Debug `--profile base`

```json
{"total_ms":8124,"spans":[{"name":"load","duration_ms":6380},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":6379},{"name":"graph","duration_ms":0},{"name":"compile templates","duration_ms":4},{"name":"generate","duration_ms":46},{"name":"compile","duration_ms":1035},{"name":"render","duration_ms":407},{"name":"write","duration_ms":252}]}
```

The Rocci-versus-base gap is the `provenance` span (~4065ms). Parse remains a
~6s debug cost under both profiles; `discover` and `graph` are sub-millisecond.
Debug `check knowledge` was 13.62s rocci versus 8.48s base on this revision.

## Post-change baseline (Phases 1–4)

The dated before/after summary is
[OKF load-performance improvement results](../status/okf-load-performance.md).[^results-status]

After splitting load spans, batching git, defaulting `run` to Rocci schema
without git provenance, and adding a watch parse cache:[^okf-dev][^okf-load]
[^okf-validate]

### Release first-open `run` (default, no `--provenance`)

```json
{"total_ms":357,"spans":[{"name":"load","duration_ms":290},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":289,"note":"cache_hit=0 miss=53"},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":0},{"name":"compile templates","duration_ms":0},{"name":"generate","duration_ms":7},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":30},{"name":"write","duration_ms":30}]}
```

### Debug watch rebuild after one Markdown change

```json
{"total_ms":177,"spans":[{"name":"load","duration_ms":5},{"name":"discover","duration_ms":1},{"name":"parse","duration_ms":4,"note":"cache_hit=52 miss=1"},{"name":"graph","duration_ms":0},{"name":"provenance","duration_ms":0},{"name":"compile templates","duration_ms":6},{"name":"generate","duration_ms":53},{"name":"compile","duration_ms":0,"note":"cached"},{"name":"render","duration_ms":29},{"name":"write","duration_ms":84}]}
```

### Isolated `check` (release binary)

| Profile | Wall time |
| --- | --- |
| `--profile rocci` | 0.40s |
| `--profile base` | 0.27s |

`run` defaults skip git provenance (`provenance` 0ms) while keeping Rocci
schema. `check --profile rocci` still runs batched provenance. Bounded
concept-path loading was not started: release first-open `load` is 290ms, not
multiple seconds.

## Findings

1. The dominant cost on cached rebuilds is `load`, not template compilation,
   Roc rendering, or writing output. In the Rocci-profile run, `load`
   accounted for 8560ms of 8718ms total. In the base-profile run, `load`
   accounted for 5142ms of 5276ms total.

2. Switching from `--profile rocci` to `--profile base` reduced cached rebuild
   time by about 3442ms, and nearly all of that reduction appears inside
   `load`, not later generator stages.

3. That result matches the code structure: `rebuild_site` records wall-clock
   `load` plus `okf::load_timed` sub-spans, and provenance runs only for the
   Rocci profile. Phase 1 reports confirm the gap is the `provenance`
   span.[^okf-dev][^okf-load]

4. Provenance git work is now a constant number of subprocesses per load
   (repository root, one dirty-status dump, one batched `git log`), not one
   process per source path.[^okf-validate]

## Interpretation

The current rebuild cost is not primarily a Roc host problem once the renderer
is cached. The larger issue is that concept preview and bundle preview still
pay for whole-bundle OKF loading, and the Rocci profile adds repository
provenance work on top of that.[^okf-load][^okf-validate]

The new headless profile-report path changes the debugging workflow more than
the runtime itself: it turns a previously window-bound performance question
into something that can be measured from a plain terminal session and reused by
future audits or CI-oriented perf checks.[^okf-main][^okf-dev][^cli-plan]

## Recommended next steps

The follow-up implementation plan is
[OKF load-performance improvements](../plans/okf-load-performance.md).

1. Add finer profiling inside `okf::load`. Implemented: reports now split
   parse versus provenance, and provenance is the Rocci-versus-base gap.[^okf-load]

2. Reduce provenance overhead in `validate_lifecycle_and_sources`. Implemented:
   batched git status and log, plus in-process reuse of repeated source
   paths.[^okf-validate]

3. Fast local preview is default `rocci-okf run` (Rocci schema, no git
   provenance). Use `--provenance` to turn git checks back on. `--profile base`
   remains portable OKF, not the fast-preview workflow.[^okf-main]

4. Bounded concept-path loading was gated off: release first-open `load` is
   290ms on this repository.[^okf-load]

[^okf-main]: `Run` supports `--no-window` and now also accepts `--profile-report` to surface rebuild timings directly in the CLI path.
[^results-status]: Dated Status snapshot of machine-local before/after timings after Phases 1–4.
[^okf-dev]: `run_knowledge` uses `serve_static_site`, `rebuild_site` records the top-level `load` span, and this revision emits `ProfileSnapshot` values during successful rebuilds.
[^dev-server]: The shared static dev server stores the current `ProfileSnapshot` for the inspector and profile endpoint; headless reporting now reuses that same snapshot data.
[^okf-load]: `okf::load` discovers and parses the whole bundle, then runs extra validation for `Profile::Rocci`.
[^okf-validate]: `validate_lifecycle_and_sources` performs stale/provenance checks and now batches git status and log over unique source paths.
[^cli-plan]: The three-CLI boundary keeps `rocci-okf` as the OKF-specific viewer path, so headless OKF profiling belongs here rather than in `rocci` or `rocdown`.
