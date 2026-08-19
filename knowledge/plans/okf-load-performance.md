---
type: Implementation Plan
title: OKF load-performance improvements
description: Phased reduction of `okf::load` latency for `rocci-okf run` and `check`, starting from measured provenance and whole-bundle parse costs. Phases 1–4 implemented; Phase 5 skipped (release concept-path load is sub-second); Phase 6 records the new baseline.
tags: [domain/okf, domain/rocci-okf, concern/performance, concern/tooling, concern/validation, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T12:15:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: results-status
    resource: ../status/okf-load-performance.md
    title: OKF load-performance improvement results
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
    title: OKF bundle load, parse, and graph resolve
    author: process:git
    last_modified: 2026-08-19
  - id: okf-validate
    resource: ../../crates/okf/src/validate.rs
    title: Lifecycle and per-source git provenance validation
    author: process:git
    last_modified: 2026-08-19
  - id: okf-preview
    resource: ../../crates/okf/src/preview.rs
    title: Concept-path preview resolution
    author: process:git
    last_modified: 2026-08-18
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild and load span
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run flags including profile-report
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
  - id: cli-profile
    resource: ../../crates/rocci-cli/src/profile.rs
    title: Shared rebuild ProfileSnapshot and SpanRecorder
    author: process:git
    last_modified: 2026-08-19
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-18
  - id: okf-app-plan
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF application plan
    author: process:cursor
    last_modified: 2026-08-17
  - id: static-okf
    resource: ../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: deps-check
    resource: ../../scripts/check-workspace-deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
    last_modified: 2026-08-18
---

# OKF load-performance improvements

## Goal and scope

Make `rocci-okf run` and `rocci-okf check` fast enough for local knowledge
authoring without weakening Rocci-profile diagnostics on the CI and review
paths. The measured bottleneck is `okf::load`, not Roc template compilation or
HTML write once the native renderer is cached.[^preview-audit][^headless-audit]

This plan covers load observability, git provenance cost, preview versus check
policy, and watch-rebuild parse reuse. It does not redesign the portable OKF
engine, the review HTML chrome, or Roc host caching.

This is an exploratory recommendation. Phases 1–4 are implemented in this
revision. Phase 5 is skipped: release `run path/to/concept.md` `load` is
sub-second on this repository, so bounded concept preview is not started.
Phase 6 records the post-change baseline. Measured before/after timings are in
the [OKF load-performance improvement results](../status/okf-load-performance.md)
status snapshot; those numbers are machine-local, not a latency SLA.[^results-status]

## Established baseline

`rebuild_site` wraps a single `load` span around `okf::load`. That function
discovers every Markdown file, parses YAML and CommonMark bodies, resolves the
graph, and for `Profile::Rocci` runs `validate_lifecycle_and_sources`.[^okf-dev]
[^okf-load][^okf-validate]

`rocci-okf run path/to/concept.md` only changes the browser open path. It still
loads the whole bundle.[^okf-preview][^preview-audit]

Headless `--profile-report {terminal,json}` already emits the combined
`ProfileSnapshot` without a preview window. `okf` must stay UI-neutral and must
not depend on `rocci-cli`. Timing structs that leave `okf` should be ordinary
durations, then mapped into CLI spans in `rocci-okf`.[^okf-main][^engine-readme]
[^cli-profile][^deps-check]

Rocci-profile schema rules (types, tags, authority, owners) are separate from
git provenance (OKF4006/4007/4008). `--profile base` skips both the extra schema
rules and provenance. That is too coarse for default preview.[^okf-load]
[^okf-validate]

## Measured problem

Local 2026-08-19 timings are machine-local, not a portability contract. The
relative breakdown is the durable part.[^preview-audit][^headless-audit]

| Path | Profile | `load` | Total |
| --- | --- | --- | --- |
| Cached `run` concept file (debug) | rocci | 9593ms | 9750ms |
| Cached `run` concept file (debug) | base | 5786ms | 5992ms |
| Cached `run` whole bundle (debug) | rocci | 10158ms | 10324ms |
| Isolated `check knowledge` (release) | rocci | — | 4.77s |
| Isolated `check knowledge` (release) | base | — | 0.24s |

On cached rebuilds, `load` is about 98% of wall time. Template compile, Roc
render, and write together stay under 200ms. Concept-path preview is not
materially cheaper than bundle preview.[^preview-audit]

The Rocci-versus-base gap on release `check` is almost entirely provenance:
`validate_lifecycle_and_sources` shells out to `git log -1 --format=%cI` and
often `git status --porcelain` per relative `sources[].resource`. The current
bundle has on the order of tens of concepts and hundreds of source refs, many
repeating the same crate paths.[^okf-validate][^preview-audit][^headless-audit]

The remaining ~5–6s of debug `run` `load` under `--profile base` is whole-bundle
discover/parse/graph on an unoptimized binary. Release `check --profile base` at
0.24s shows that parse is acceptable in release once git is out of the way.
Watch rebuilds still repeat that work on every file change.[^okf-load][^okf-dev]

## Constraints that do not move

| Keep | Meaning for this plan |
| --- | --- |
| `okf` portable | No `rocci-cli` or desktop dependency; no Roc in the engine |
| Knowledge is inert Markdown | No Rocdown or executable content in `knowledge/**/*.md` |
| Diagnostic codes | OKF4004–OKF4008 stay the same; only the lookup implementation changes |
| `check --profile rocci` | CI and review keep full schema plus provenance |
| Three-CLI split | Load work stays in `okf` / `rocci-okf`, not `rocci` or `rocdown` |
| Cached Roc host | Do not spend phases on renderer compile unless load falls below it |

Those boundaries are the portable engine split, inert knowledge Markdown, and
the three-CLI preview ownership already recorded for OKF review.[^engine-readme]
[^static-okf][^cli-plan][^okf-app-plan]

## Non-goals

- Speeding Roc native/Wasm compile or the generation host cache
- Changing OKF YAML, footnote, or source-id rules
- Making `--profile base` the default for `check`
- Lying about a complete graph or search index on a partial preview
- Parallel Markdown parse until Phase 1 timings prove parse is still large
  after git batching
- CI latency SLAs; local `--profile-report json` remains the measurement tool

## Success targets (local, not a contract)

After Phase 2, release `check knowledge --profile rocci` should be in the same
order of magnitude as `--profile base` on this repository (sub-second to low
hundreds of milliseconds, not multiple seconds of git).[^headless-audit]

After Phase 4, a cached debug `rocci-okf run` rebuild of one dirty concept
should spend most of `load` on the changed file plus graph resolve, not on
re-parsing the whole tree or re-running hundreds of git processes.[^okf-dev]

Phase 5 is optional. Re-measure before starting it.

## Delivery phases

Each phase is one mergeable change. Measure with:

```text
cargo run -q -p rocci-okf -- run knowledge/plans/okf-load-performance.md \
  --no-window --port auto --profile-report json
time cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
time cargo run -q -p rocci-okf -- check knowledge --profile base --format terminal
```

### Phase 1 — Split the opaque `load` span (implemented)

**Bound:** A successful `--profile-report json` rebuild lists sub-spans inside
load, at least `discover`, `parse`, `graph`, and `provenance` (the last omitted
or zero under `--profile base`).[^okf-dev][^cli-profile][^okf-load]

**Owner:** `crates/okf` returns a small timing breakdown next to `Bundle`.
`crates/rocci-okf` maps those durations onto `SpanRecorder` / `ProfileSnapshot`.
Do not import `rocci-cli` from `okf`.[^engine-readme][^deps-check]

**Out of bound:** Changing validation, git, or parse behavior.

**Tests:** `okf` unit test that a load records non-zero parse time on a tiny
fixture; `rocci-okf` report formatting still lists total plus named spans.

**Exit:** One local JSON report on this repository showing which stage owns the
Rocci-versus-base gap. Update the two load audits with that split, still as
machine-local evidence.

### Phase 2 — Batch and memoize git provenance (implemented)

**Bound:** `validate_lifecycle_and_sources` performs a constant number of git
invocations per load (repository root, one dirty-status dump, one last-modified
query over the unique relative paths), plus in-process memoization when many
concepts cite the same resource.[^okf-validate]

Suggested lookup shape, not a required git UI:

1. `git rev-parse --show-toplevel` once.
2. Unique-ify repository-relative source paths.
3. One `git status --porcelain -z` (or equivalent) to classify dirty/untracked.
4. One `git log --format=%cI --name-only -- <unique-paths>` (or equivalent)
   walking newest-first and recording the first timestamp per path.

Keep OKF4004/4005 (stale / verification-vs-generated) on the same metadata
path. Keep OKF4006/4007/4008 messages and codes. External URLs and absolute
paths still skip git.[^okf-validate]

**Owner:** `crates/okf/src/validate.rs`. Tests belong here, using a temporary
git repository fixture; they must not require `rocci-okf` or a desktop host.

**Out of bound:** Skipping provenance on `check --profile rocci`. Changing
preview CLI defaults.

**Exit:** Release `check knowledge --profile rocci` on this repo is no longer
dominated by hundreds of git processes. Diagnostic output on a known dirty
source still matches pre-change codes. Headless `run --profile rocci` `load`
drops by roughly the previously measured Rocci-versus-base provenance gap.

### Phase 3 — Preview can skip provenance without dropping Rocci schema (implemented)

**Bound:** Load grows an explicit provenance switch, independent of
`Profile::Base` versus `Profile::Rocci`. Schema, unique ids, graph, and
footnote/source pairing still follow the selected profile.[^okf-load]

CLI policy:

- `rocci-okf check --profile rocci` remains full provenance.
- `rocci-okf run` defaults to Rocci schema **without** git provenance, and
  accepts `--provenance` (name flexible) to turn it back on.
- `--profile base` still means portable OKF, not “fast Rocci”.

Document the split in `crates/rocci-okf/README.md`. Do not document `--profile
base` as the supported fast-preview workflow once this flag exists.[^okf-readme]
[^okf-main]

**Owner:** `crates/okf` load options; `crates/rocci-okf` CLI and README.

**Out of bound:** Changing CI check defaults. Partial bundle loading.

**Exit:** `run knowledge/plans/okf-load-performance.md --profile-report json`
without `--provenance` has a near-zero provenance span and still rejects Rocci
schema errors. `check --profile rocci` still emits OKF4006/4008 on a
constructed dirty tracked source.

### Phase 4 — Incremental parse cache on watch rebuilds (implemented)

**Bound:** `rebuild_site` (or a helper owned by `okf`) reuses parsed concepts,
indexes, and logs whose bytes or mtime+size have not changed. Graph resolve,
id uniqueness, and optional provenance still run over the assembled
bundle.[^okf-dev][^okf-load]

First rebuild of a process may still parse everything. Subsequent rebuilds
triggered by one Markdown file must not re-read unchanged files.

**Owner:** Prefer a pure cache helper in `okf` (path → parsed document) so
`check` can stay simple while `run` holds the cache across watch ticks.
`rocci-okf` wires the cache into `serve_static_site`.

**Out of bound:** Skipping graph resolve. Caching git results across process
restarts. Partial navigation.

**Tests:** Fixture bundle of several concepts; second load with one file
touched reports a parse-cache hit count or otherwise proves unchanged paths
were not re-parsed. Malformed/unclosed inputs still terminate (existing
scanner monotonicity rules elsewhere are unchanged).

**Exit:** Headless watch-style double rebuild (initial + one file touch) shows
`parse` collapsing on the second snapshot while diagnostics for the touched
file still update.

### Phase 5 — Bounded concept preview, only if still needed (skipped)

**Gate:** After Phases 2–4, re-run the Phase 1 measurement commands. Start this
phase only if first-open `run path/to/concept.md` `load` remains painful in a
**release** binary (multiple seconds), not merely a cold debug build.

**2026-08-19 skip:** Release first-open `run knowledge/plans/okf-load-performance.md`
reported `load` 290ms (`parse` 289ms, `provenance` 0). That is not multiple
seconds, so this phase was not started.

**Bound if started:** `rocci-okf run concept.md` loads the bundle root index,
collection indexes needed for chrome, the target concept, and records required
to resolve its immediate links. The open page is complete. Site-wide search,
review queues, and unrelated concept bodies may be absent or marked
incomplete. `rocci-okf run knowledge` stays whole-bundle.[^okf-preview]
[^okf-readme]

Do not present a partial graph as the full catalog.

**Owner:** `okf` load filtering + `rocci-okf` preview chrome honesty.

**Exit:** Concept-path first open is dominated by the target record and its
indexes. Whole-bundle `run knowledge` and `check` behavior is unchanged.

### Phase 6 — Record the new baseline (implemented)

**Bound:** Refresh the two load audits with post-change `--profile-report json`
and release `check` timings. Record the dated before/after summary as a Status
snapshot. Mention the provenance flag and sub-spans in
`crates/rocci-okf/README.md` if Phase 3 shipped. Do not claim a phase complete
in `knowledge/log.md` until the required GitHub workflows on that revision are
green.[^preview-audit][^headless-audit][^results-status][^okf-readme]

**Out of bound:** New product features.

## Layer map

| Concern | Owner |
| --- | --- |
| Discover / parse / graph | `crates/okf/src/lib.rs` |
| Provenance git batching | `crates/okf/src/validate.rs` |
| Load options (profile vs provenance) | `okf::load` / `okf::check` |
| Span mapping and `--profile-report` | `crates/rocci-okf/src/dev.rs` |
| Preview CLI defaults | `crates/rocci-okf/src/main.rs` |
| Watch parse cache wiring | `crates/rocci-okf/src/dev.rs` using an `okf` helper |
| Public commands | `crates/rocci-okf/README.md` |
| Measurement evidence | `knowledge/audits/*-load-performance.md` |

## Risks

- Batched `git log --name-only` can miss a path that has never been committed
  (treat as untracked, same as empty `git log -1` today).[^okf-validate]
- Skipping provenance on `run` can hide OKF4006/4008 until `check`. Phase 3
  must keep `check` strict and make the flag obvious in README.
- A parse cache keyed only on mtime can serve stale bytes on some filesystems;
  prefer content hash, or mtime+size with a hash if those collide.
- Bounded preview (Phase 5) can break collection indexes and search if chrome
  assumes every concept exists. That is why it stays gated.

## Open questions

1. Should the first `run` process rebuild still do provenance once, then skip
   it on watch, or skip it entirely until `--provenance`?
2. Is a content-addressed parse cache worth sharing with `check`, or only with
   the long-lived `run` server?
3. After Phase 2, is Phase 5 unnecessary on this repository’s current size?

[^results-status]: Dated Status snapshot of machine-local before/after timings, preview-versus-check policy, and skipped bounded preview.
[^preview-audit]: Cached concept-path `run` spent 9593ms of 9750ms in `load`; concept-path was not cheaper than whole-bundle load; release `check` was 4.77s rocci versus 0.24s base.
[^headless-audit]: Headless `--profile-report` made load-dominated rebuilds observable; recommended finer load spans and batched git.
[^okf-load]: `okf::load` discovers and parses the whole bundle, resolves the graph, and runs Rocci-only lifecycle validation.
[^okf-validate]: When git provenance is on, validation batches rev-parse, status, and log over unique source paths.
[^okf-preview]: `resolve_preview_path` returns bundle root plus open path; it does not narrow loading.
[^okf-dev]: `rebuild_site` records one `load` span around `okf::load` on every watch rebuild.
[^okf-main]: `Run` forwards `--profile` and `--profile-report` into the headless rebuild path.
[^okf-readme]: `rocci-okf` documents `run`, `check`, inspect, and build; fast-preview policy belongs here once it exists.
[^engine-readme]: `okf` is UI-neutral and has zero dependencies on other Rocci crates.
[^cli-profile]: CLI snapshots are lists of named millisecond spans; `okf` should supply durations, not import this type.
[^cli-plan]: OKF preview stays on `rocci-okf`, not `rocci` or `rocdown`.
[^okf-app-plan]: Portable engine versus Rocci application split; load performance belongs in `okf` with CLI mapping in `rocci-okf`.
[^static-okf]: Canonical knowledge remains inert Markdown; this plan does not add executable records.
[^deps-check]: Workspace dependency direction is mechanical; `okf` must not grow a `rocci-cli` edge.
