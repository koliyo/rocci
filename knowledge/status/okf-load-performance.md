---
type: Status
title: OKF load-performance improvement results
description: Machine-local before/after timings after load sub-spans, batched git provenance, default `run` without provenance, and a watch parse cache. Phase 5 bounded concept preview was skipped.
tags: [domain/okf, domain/rocci-okf, concern/performance, concern/tooling, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-19T12:15:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/okf-load-performance.md
    title: OKF load-performance improvements plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-audit
    resource: ../audits/hybrid-rocdown-islands-preview-performance.md
    title: hybrid-rocdown-islands preview performance audit
    author: process:cursor
    last_modified: 2026-08-19
  - id: headless-audit
    resource: ../audits/rocci-okf-headless-load-performance.md
    title: rocci-okf headless load-performance audit
    author: process:cursor
    last_modified: 2026-08-19
  - id: okf-load
    resource: ../../crates/okf/src/lib.rs
    title: OKF load timings, LoadOptions, and ParseCache
    author: process:git
    last_modified: 2026-08-19
  - id: okf-options
    resource: ../../crates/okf/src/ast.rs
    title: LoadOptions profile and provenance switch
    author: process:git
    last_modified: 2026-08-19
  - id: okf-validate
    resource: ../../crates/okf/src/validate.rs
    title: Batched git provenance validation
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild spans and watch parse cache
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run --provenance flag
    author: process:git
    last_modified: 2026-08-19
  - id: okf-readme
    resource: ../../crates/rocci-okf/README.md
    title: rocci-okf usage contract
    author: process:git
    last_modified: 2026-08-19
  - id: engine-readme
    resource: ../../crates/okf/README.md
    title: Portable OKF engine boundary
    author: process:git
    last_modified: 2026-08-19
---

# OKF load-performance improvement results

## Snapshot date

2026-08-19.

These timings came from local command execution on this repository and this
machine. They are evidence for the load-performance work, not a latency SLA or
a portability contract.[^preview-audit][^headless-audit][^plan]

## Result

Phases 1–4 of the load-performance plan are implemented in this revision.
Phase 5 (bounded concept-path loading) was skipped: release first-open `load`
is 290ms here, not multiple seconds. Phase 6 recorded the post-change
baseline in the two load audits; this status snapshot is the dated
before/after summary.[^plan][^preview-audit][^headless-audit]

Default local preview is now sub-second on a release first open, and a debug
watch rebuild after one Markdown content change reuses unchanged parses.
`check --profile rocci` remains whole-bundle and still runs git provenance,
now batched instead of one subprocess per source path.[^okf-readme][^okf-validate]
[^okf-dev]

## Preview versus check

`rocci-okf run` defaults to the Rocci schema with git provenance off. Pass
`--provenance` to turn OKF4006/4007/4008 back on during preview.
`rocci-okf check --profile rocci` still runs full provenance. `--profile base`
is portable OKF, not the fast Rocci preview path.[^okf-main][^okf-readme]
[^okf-options]

Rocci schema without provenance still emits lifecycle warnings such as
OKF4004/4005. The `provenance` span is omitted or zero when git checks are
off.[^okf-load][^okf-options]

## Measured before and after

Cached-renderer `run` numbers use `--no-window --profile-report json`. Isolated
`check` numbers use a prebuilt release binary and `time`.[^preview-audit]
[^headless-audit]

| Path | Before | After Phases 1–4 |
| --- | --- | --- |
| Debug cached `run` concept path, `--profile rocci` | load 9593ms / total 9750ms | superseded by default `run` (no git provenance) |
| Release first-open `run` (default, no `--provenance`) | not measured in this form | load 290ms / total 357ms; parse 289ms (`cache_hit=0 miss=53`); provenance 0ms |
| Debug watch rebuild after one Markdown content change | full reparse (~6s parse class) | parse 4ms (`cache_hit=52 miss=1`); load 5ms; total 177ms |
| Release `check --profile rocci` | 4.77s | 0.40s |
| Release `check --profile base` | 0.24s | 0.27s |
| Debug `provenance` span, `--profile rocci` | ~4065ms | 0ms on default `run`; still present on `check --profile rocci` after batching |

The Rocci-versus-base gap on the original debug split was the `provenance`
span. Parse stayed in the same ~6s debug bucket under both profiles.
`discover` and `graph` remain sub-millisecond.[^preview-audit][^headless-audit]
[^okf-load]

A metadata-only `touch` does not trigger a watch rebuild. A Markdown content
change does, and then the parse cache reuses documents whose path, mtime, and
size are unchanged.[^okf-load][^okf-dev]

## What shipped in code

Load reports `discover`, `parse`, `graph`, and `provenance` beside the wall
`load` span. `LoadOptions` selects profile and whether provenance runs.
`ParseCache` is keyed by relative path plus mtime and size, and is cleared if
the profile changes. Graph resolution, unique-id checks, and optional
provenance still run on every load.[^okf-load][^okf-options][^engine-readme]

When git provenance is on, validation uses a constant number of git
invocations per load: `rev-parse --show-toplevel`, one
`status --porcelain -z --untracked-files=no`, and one
`git log --format=%cI --name-only` over unique source paths. Paths missing
from that log are treated as untracked (OKF4007). The public three-argument
`validate_lifecycle_and_sources` still runs git.[^okf-validate]

`rebuild_site` keeps a `ParseCache` across watch ticks and annotates the parse
span with `cache_hit=N miss=M`.[^okf-dev]

## What did not ship

Bounded concept-path loading was not started. `rocci-okf run path/to/concept.md`
still loads the whole bundle; it only changes the browser open path.[^plan]
[^preview-audit]

Debug first-open parse without a warm cache remains a multi-second cost on this
machine. `check` is still whole-bundle. The parse cache is mtime-and-size, not
content-addressed, and is not shared with `check`.[^plan][^okf-load]

These numbers are not logged as CI-complete. Required GitHub workflows have
not been recorded as green on this revision.[^plan]

[^plan]: Phases 1–4 implemented; Phase 5 skipped after a 290ms release first-open load; Phase 6 records the post-change baseline.
[^preview-audit]: Cached concept-path run spent 9593ms of 9750ms in load; release check was 4.77s rocci versus 0.24s base; post-change release first-open load is 290ms and watch parse is 4ms.
[^headless-audit]: Headless profile-report made load-dominated rebuilds observable; post-change check is 0.40s rocci versus 0.27s base; provenance is batched.
[^okf-load]: `load_timed` returns discover/parse/graph/provenance durations; `ParseCache` reuses unchanged documents; whole-bundle parse still runs on first open.
[^okf-options]: `LoadOptions` selects profile independently of whether provenance runs.
[^okf-validate]: `validate_lifecycle_and_sources_with(..., check_git)` batches rev-parse, status, and log over unique paths.
[^okf-dev]: `rebuild_site` maps load sub-spans onto the CLI profile snapshot and holds ParseCache across watch ticks.
[^okf-main]: `run` accepts `--provenance`; the flag is off by default; `check --profile rocci` still runs git provenance.
[^okf-readme]: Documents default run as Rocci schema without git provenance, `--provenance` to turn it on, and check as the strict review path.
[^engine-readme]: `okf` remains UI-neutral; load timings and ParseCache live in the portable engine, not in CLI snapshot types.
