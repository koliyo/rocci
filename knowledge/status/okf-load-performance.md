---
type: Status
title: OKF load-performance improvement results
description: Machine-local before/after timings after load sub-spans, batched git provenance, default `run` without provenance, watch and cross-process parse cache, and debug first-open parse. Phase 5 bounded concept preview was skipped.
tags: [domain/okf, domain/rocci-okf, concern/performance, concern/tooling, concern/validation]
status: draft
generated: { by: process:cursor, at: 2026-08-19T16:50:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/okf/okf-load-performance.md
    title: OKF load-performance improvements plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-audit
    resource: ../audits/rocdown/hybrid-rocdown-islands-preview-performance.md
    title: hybrid-rocdown-islands preview performance audit
    author: process:cursor
    last_modified: 2026-08-19
  - id: headless-audit
    resource: ../audits/okf/rocci-okf-headless-load-performance.md
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
    title: Headless rebuild spans and persisted parse cache
    author: process:git
    last_modified: 2026-08-19
  - id: parse-cache
    resource: ../../crates/okf/src/parse_cache.rs
    title: ParseCache memory map and directory persistence
    author: process:git
    last_modified: 2026-08-19
  - id: workspace-cargo
    resource: ../../Cargo.toml
    title: Dev-profile opt-level for comrak, okf, and Markdown parse deps
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

Phases 1–4 of the load-performance plan are implemented in this revision,
plus a Phase 4 follow-on that makes debug first-open parse cheap and persists
`ParseCache` across `run` process restarts. Phase 5 (bounded concept-path
loading) remains skipped: release first-open `load` is 290ms here, not multiple
seconds, and concept-path `run` still loads the whole catalog. Phase 6 recorded
the post-change baseline in the two load audits; this status snapshot is the
dated before/after summary.[^plan][^preview-audit][^headless-audit][^parse-cache]

Default local preview is now sub-second on a release first open. Debug
`cargo run -p rocci-okf -- run` first-open parse is in the hundreds of
milliseconds here when `comrak` and `okf` are opt-level 3 in the dev profile.
A second `run` process reuses the on-disk parse cache. A debug watch rebuild
after one Markdown content change reuses unchanged parses in memory.
`check --profile rocci` remains whole-bundle, still runs git provenance, and
does not read the parse-cache directory.[^okf-readme][^okf-validate][^okf-dev]
[^workspace-cargo]

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

| Path | Before | After Phases 1–4 + debug parse follow-on |
| --- | --- | --- |
| Debug cached `run` concept path, `--profile rocci` | load 9593ms / total 9750ms | superseded by default `run` (no git provenance) |
| Release first-open `run` (default, no `--provenance`) | not measured in this form | load 290ms / total 357ms; parse 289ms (`cache_hit=0 miss=53`); provenance 0ms |
| Debug first-open `run` concept path, empty parse cache | parse ~7100ms (`cache_hit=0 miss=58`) | parse 414ms (`cache_hit=0 miss=58`); load 415ms |
| Debug second `run` process, same bundle | full reparse | parse 1ms (`cache_hit=58 miss=0`); load 2ms; total 203ms with cached renderer |
| Debug watch rebuild after one Markdown content change | full reparse (~6s parse class) | parse 4ms (`cache_hit=52 miss=1`); load 5ms; total 177ms |
| Release `check --profile rocci` | 4.77s | 0.40s |
| Release `check --profile base` | 0.24s | 0.27s |
| Debug `provenance` span, `--profile rocci` | ~4065ms | 0ms on default `run`; still present on `check --profile rocci` after batching |

The Rocci-versus-base gap on the original debug split was the `provenance`
span. Debug first-open parse is no longer in the multi-second bucket once
`comrak` / `okf` are optimized in the dev profile and misses parse in
parallel. `discover` and `graph` remain sub-millisecond.[^preview-audit]
[^headless-audit][^okf-load][^workspace-cargo]

A metadata-only `touch` does not trigger a watch rebuild. A Markdown content
change does, and then the parse cache reuses documents whose path, mtime, and
size are unchanged, including across `run` process restarts.[^okf-load]
[^okf-dev][^parse-cache]

## What shipped in code

Load reports `discover`, `parse`, `graph`, and `provenance` beside the wall
`load` span. `LoadOptions` selects profile and whether provenance runs.
`ParseCache` is keyed by relative path plus mtime and size, and is cleared if
the profile changes. Graph resolution, unique-id checks, and optional
provenance still run on every load. Parse misses run in parallel.
`ParseCache::load_dir` / `save_dir` persist entries to a caller-provided
directory; diagnostic codes are interned from a known list on load.[^okf-load]
[^okf-options][^engine-readme][^parse-cache]

When git provenance is on, validation uses a constant number of git
invocations per load: `rev-parse --show-toplevel`, one
`status --porcelain -z --untracked-files=no`, and one
`git log --format=%cI --name-only` over unique source paths. Paths missing
from that log are treated as untracked (OKF4007). The public three-argument
`validate_lifecycle_and_sources` still runs git.[^okf-validate]

`rebuild_site` keeps a `ParseCache` across watch ticks, loads and saves it
under `ROCCI_CACHE/okf-parse/` (versioned), and annotates the parse span with
`cache_hit=N miss=M`. The workspace dev profile optimizes `comrak`, `okf`, and
their Unicode/YAML helpers so `cargo run` parse is not a debug-cost trap.
[^okf-dev][^workspace-cargo]

## What did not ship

Bounded concept-path loading was not started. `rocci-okf run path/to/concept.md`
still loads the whole bundle; it only changes the browser open path.[^plan]
[^preview-audit]

`check` is still whole-bundle and does not use the on-disk parse cache. The
parse cache is mtime-and-size, not content-addressed.[^plan][^okf-load]
[^parse-cache]

These numbers are not logged as CI-complete. Required GitHub workflows have
not been recorded as green on this revision.[^plan]

[^plan]: Phases 1–4 implemented; Phase 4 follow-on covers debug parse and cross-process cache; Phase 5 skipped after a 290ms release first-open load; Phase 6 records the post-change baseline.
[^preview-audit]: Cached concept-path run spent 9593ms of 9750ms in load; release check was 4.77s rocci versus 0.24s base; post-change release first-open load is 290ms and watch parse is 4ms.
[^headless-audit]: Headless profile-report made load-dominated rebuilds observable; post-change check is 0.40s rocci versus 0.27s base; provenance is batched.
[^okf-load]: `load_timed` returns discover/parse/graph/provenance durations; `load_with_cache` parses misses in parallel; whole-bundle parse still runs on first open when the directory cache is empty.
[^okf-options]: `LoadOptions` selects profile independently of whether provenance runs.
[^okf-validate]: `validate_lifecycle_and_sources_with(..., check_git)` batches rev-parse, status, and log over unique paths.
[^okf-dev]: `rebuild_site` maps load sub-spans onto the CLI profile snapshot, holds ParseCache across watch ticks, and saves it under ROCCI_CACHE for the next process.
[^okf-main]: `run` accepts `--provenance`; the flag is off by default; `check --profile rocci` still runs git provenance.
[^okf-readme]: Documents default run as Rocci schema without git provenance, `--provenance` to turn it on, check as the strict review path, and run-only parse-cache persistence.
[^engine-readme]: `okf` remains UI-neutral; load timings and ParseCache live in the portable engine, not in CLI snapshot types.
[^parse-cache]: `ParseCache::load_dir` / `save_dir` persist entries beside a version stamp; unknown diagnostic codes drop the entry rather than leaking strings.
[^workspace-cargo]: Dev profile sets opt-level 3 for `comrak`, `okf`, `yaml-rust`, and comrak Unicode helpers.
