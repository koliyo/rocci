---
type: Audit
title: rocci-okf headless load-performance audit
description: Measured headless `rocci-okf run --no-window` rebuild timings, added a CLI profile-report path, and traced most cached rebuild latency to `okf::load` under the Rocci profile.
tags: [domain/okf, domain/rocci, concern/performance, concern/tooling, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-19T10:31:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
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
    title: OKF bundle load path
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

## Findings

1. The dominant cost on cached rebuilds is `load`, not template compilation,
   Roc rendering, or writing output. In the Rocci-profile run, `load`
   accounted for 8560ms of 8718ms total. In the base-profile run, `load`
   accounted for 5142ms of 5276ms total.

2. Switching from `--profile rocci` to `--profile base` reduced cached rebuild
   time by about 3442ms, and nearly all of that reduction appears inside
   `load`, not later generator stages.

3. That result matches the code structure: `rebuild_site` wraps `okf::load` in
   the top-level `load` span, and `okf::load` conditionally runs
   `validate_lifecycle_and_sources` only for the Rocci profile.[^okf-dev][^okf-load]

4. The likeliest hot path inside Rocci-profile `load` is source provenance
   validation. `validate_lifecycle_and_sources` iterates concepts and sources,
   then shells out to git per source path through `git log -1 --format=%cI`
   and `git status --porcelain` checks.[^okf-validate]

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

1. Add finer profiling inside `okf::load`, especially around Markdown/body
   parsing versus lifecycle/provenance validation, so `load` stops being one
   large opaque bucket.[^okf-load]

2. Reduce provenance overhead in `validate_lifecycle_and_sources`, starting
   with process-local caching or batched git queries for repeated source-path
   checks.[^okf-validate]

3. Consider a fast development mode that makes `--profile base` the explicit
   default for local preview work when provenance warnings are not the thing
   under investigation.[^okf-main]

4. If single-concept preview latency remains important, investigate a bounded
   load path so `rocci-okf run path/to/concept.md` does not always parse the
   full bundle before serving.[^okf-load]

[^okf-main]: `Run` supports `--no-window` and now also accepts `--profile-report` to surface rebuild timings directly in the CLI path.
[^okf-dev]: `run_knowledge` uses `serve_static_site`, `rebuild_site` records the top-level `load` span, and this revision emits `ProfileSnapshot` values during successful rebuilds.
[^dev-server]: The shared static dev server stores the current `ProfileSnapshot` for the inspector and profile endpoint; headless reporting now reuses that same snapshot data.
[^okf-load]: `okf::load` discovers and parses the whole bundle, then runs extra validation for `Profile::Rocci`.
[^okf-validate]: `validate_lifecycle_and_sources` performs stale/provenance checks and shells out to git for tracked/dirty source-path inspection.
[^cli-plan]: The three-CLI boundary keeps `rocci-okf` as the OKF-specific viewer path, so headless OKF profiling belongs here rather than in `rocci` or `rocdown`.
