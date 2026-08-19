---
type: Implementation Plan
title: OKF preview compile and render cost
description: Phased reduction of `rocci-okf run` Roc compile and apply cost after load-performance work. Stop baking page HTML into the renderer hash, write Rocci chrome from apply, and keep compile off the Markdown-edit path. Phases 1–3 are in this tree.
tags: [domain/okf, domain/rocci-okf, integration/roc, concern/performance, concern/rendering, concern/caching, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:05:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/okf-compile-render-cost.md
    title: OKF preview compile and render cost after load-performance work
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
  - id: load-plan
    resource: okf-load-performance.md
    title: OKF load-performance improvements plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: load-status
    resource: ../status/okf-load-performance.md
    title: OKF load-performance improvement results
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-audit
    resource: ../audits/hybrid-rocdown-islands-preview-performance.md
    title: hybrid-rocdown-islands preview performance audit
    author: process:cursor
    last_modified: 2026-08-19
  - id: presentation
    resource: ../../crates/rocci-okf/src/presentation.rs
    title: OKF review site compile, generate, apply, and Rust write fallback
    author: process:git
    last_modified: 2026-08-19
  - id: okf-build
    resource: ../../crates/rocci-okf/runtime/OkfBuild.roc
    title: OKF apply runtime that maps pages to HTML strings
    author: process:git
    last_modified: 2026-08-18
  - id: okf-theme
    resource: ../../crates/rocci-okf/templates/OkfTheme.rocci
    title: OKF knowledge shell
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild spans and watch parse cache
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run host and profile-report flags
    author: process:git
    last_modified: 2026-08-19
  - id: okf-readme
    resource: ../../crates/rocci-okf/README.md
    title: rocci-okf usage contract
    author: process:git
    last_modified: 2026-08-19
  - id: roc-cache
    resource: ../../crates/rocci-roc-host/src/cache.rs
    title: Two-tier generated-Roc and compiled-renderer cache
    author: process:git
    last_modified: 2026-08-18
  - id: roc-host
    resource: ../../crates/rocci-roc-host/src/host.rs
    title: HostChoice and native apply
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-build
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Rocdown applicator hash includes generated pages Roc
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
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-18
---

# OKF preview compile and render cost

## Goal and scope

Make Markdown-edit `rocci-okf run` rebuilds compile-free, and make the served
review site the Rocci shell that is already compiled, without putting page
HTML or per-concept view records into `roc build`.[^research][^generation-research]

Load-performance Phases 1–4 already moved `okf::load` off the critical path
for default preview. Remaining first-open and save cost is `compile` plus
`render` in `build_review_site_with_host`. This plan owns that presentation
path. It does not reopen provenance, parse caching, or bounded concept-path
loading.[^load-plan][^load-status][^presentation]

Phases 1–3 are in this tree: page identity is outside the renderer hash, native
apply writes `OkfTheme.knowledgeShell` HTML, and watch keeps the cached apply
path across ticks. Phases 4–6 are not started. Exploratory; not CI-complete. Measured numbers in the companion [research
record](../research/okf-compile-render-cost.md) are machine-local, not a
latency SLA.[^research]

## Established baseline

On a content-edit watch rebuild, parse cache hits unchanged Markdown
(`hit=53 miss=1` here) and `write` stays ~40ms. `roc_source_hash` still
includes `OkfPages.roc`, which inlines every `article_html` and governance
view. A body sentence therefore misses the renderer cache, runs
`roc build --opt=dev`, and spawns a new 4.8MB `apply`. Earlier cached-renderer
audits reported `compile` 0 and hid this path.[^presentation][^roc-cache]
[^research][^preview-audit]

`OkfBuild.render_all` builds HTML strings that `main.roc` discards. The
`write` span fills missing `index.html` files with Rust `html_page` chrome.
Served pages currently show the Rust “On this page” navigator, not
`OkfTheme.rocci`.[^okf-build][^okf-theme][^presentation]

`rocci-okf run` always passes `Some(host)`, which sets `force_roc`, so the
pure-Rust write path is unreachable from preview even when `roc` is
missing.[^okf-main][^presentation]

Isolated release timings on this repository (2026-08-19):

| Path | Total | Compile | Render |
| --- | --- | --- | --- |
| Cold first open | 1382ms | 727ms | 334ms |
| Warm process, cached renderer | 348ms | 0 | 31ms |
| Watch, one body sentence | 1006ms | 461ms | 467ms |

A second apply of an unchanged cached binary is ~30ms. First apply of a
freshly built binary is 300–600ms.[^research]

## Constraints that do not move

| Keep | Meaning for this plan |
| --- | --- |
| `okf` portable | No Roc, no `rocci-cli`, no presentation types in the engine |
| Knowledge is inert Markdown | No Rocdown or executable content in `knowledge/**/*.md` |
| Rust catalog / Rocci shell | Governance data stays in `okf`; visible shell is Rocci once apply writes |
| Three-CLI split | Work stays in `rocci-okf` / `rocci-roc-host`, not `rocci` or `rocdown` |
| Whole-program Roc | No per-`.rocci` object cache; identity is the whole renderer program |
| `check --profile rocci` | Unchanged; this plan is preview generate/apply, not validation |

Those boundaries are the portable engine split, inert knowledge Markdown, and
the catalog/shell decision already recorded for OKF review.[^engine-readme]
[^static-okf][^catalog-shell][^cli-plan][^deps-check]

## Non-goals

- Hashing `OkfPages.roc` out of the cache key while it still embeds page data
  (research option B). That is only safe while apply output is ignored.
- Skipping `roc build` on default `run` as the long-term product (research
  option A). Acceptable only as an unshipped local experiment, not a phase
  exit.
- Speeding the Roc compiler, embedding it in Rust, or targeting `wasm32` via
  `basic-cli`
- Parallel Markdown parse, bounded concept-path load, or git provenance work
- Changing Rocdown's `RocdownPages.roc` hash in the required phases (optional
  Phase 5)
- CI latency SLAs; `--profile-report json` remains the measurement tool

## Success targets (local, not a contract)

After Phase 1, a watch rebuild that only changes Markdown body or metadata
must report `compile` 0 (`cached`) on this repository. `roc build` runs when
`.rocci` templates, `OkfBuild.roc`, `Html.roc`, platform, or `roc` version
change, not when a concept sentence changes.[^research][^generation-research]

After Phase 2, a concept page served by `run` must contain the Rocci knowledge
shell (Home / Governance & Review links from `OkfTheme.rocci`), not only the
Rust “On this page” fallback.[^okf-theme][^presentation]

After Phase 3, a cached-renderer watch rebuild should stay in the same order
of magnitude as today's parse-cache + write path (low hundreds of milliseconds
or less here), with `render` near the ~30ms reused-binary class rather than
the 300–600ms first-run class.[^research]

## Delivery phases

Each phase is one mergeable change. Measure with:

```text
cargo run -q -p rocci-okf -- run knowledge/research/okf-compile-render-cost.md \
  --no-window --port auto --profile-report json
```

Use an isolated `ROCCI_CACHE` for cold-compile checks. For watch, keep the
process up and edit one concept body after the first snapshot.

### Phase 1 — Externalize page data from the renderer hash

**Bound:** Generated Roc that participates in the compile hash contains
templates, `Html.roc`, `OkfBuild.roc`, and `main.roc` only. Page identity
(output path, article path, title, outline, concept meta) is written beside
the workspace as files or JSON and read at apply time. `article_html` must
not appear as a Roc string literal.[^presentation][^okf-build][^roc-cache]

Use `compute_gen_hash` / `compute_compile_hash` (template identity plus `roc`
version, target, opt, platform) as the `lookup_renderer` key, or an equivalent
hash that is demonstrably independent of bundle Markdown.[^roc-cache]
[^generation-plan]

**Owner:** `crates/rocci-okf` generate + `OkfBuild.roc`. Cache key wiring may
call `rocci-roc-host` helpers; do not move presentation into `okf`.[^deps-check]

**Out of bound:** Writing Roc HTML to staging (Phase 2). Changing Rocdown.
Skipping apply.

**Tests:** Fixture bundle of two concepts. Generate twice with only the second
concept's body changed; compile hashes match and the second call hits
`lookup_renderer`. A template-source change misses. `ROCCI_REQUIRE_ROC` tests
stay behind that gate.

**Exit:** Headless watch after one Markdown body edit reports `compile` 0
(`cached`). `generate` may still rewrite page JSON and article files.

### Phase 2 — Apply writes the Rocci shell

**Bound:** `render_all` writes each document to `OKF_STAGING` /
`output_path`. After a successful apply, concept, index, and review HTML on
disk come from `OkfTheme.knowledgeShell`, not from `html_page` /
`render_toc`. Rust `if !destination.exists()` remains only as a failure
fallback when apply omitted a path, and tests must not rely on that fallback
when Roc is required.[^okf-build][^okf-theme][^presentation]

**Owner:** `OkfBuild.roc` plus the write span in `presentation.rs`.

**Out of bound:** Changing the compile hash. Dirty-page apply. Wasm host.

**Tests:** With Roc available (or `ROCCI_REQUIRE_ROC`), a built concept page
contains the knowledge-shell TOC label and Home / review links. The Rust-only
path still builds when `host` is absent and `roc` is not forced.

**Exit:** `run` preview of `knowledge/research/okf-compile-render-cost.md`
shows Rocci chrome. Profile still lists `render` greater than zero.

### Phase 3 — Reuse the applicator across watch ticks

**Bound:** `rebuild_site` keeps the cached `apply` path (and, if cheap, a
warmed process or in-process host) across watch rebuilds whose compile hash
did not change. First page-in of the 4.8MB binary is once per process, not
once per save. Spawning a new `apply` for an unchanged hash is the behavior
to remove or justify with numbers.[^okf-dev][^roc-host][^research]

**Owner:** `crates/rocci-okf/src/dev.rs` holding session state, using
`rocci-roc-host` to run apply. Prefer native subprocess reuse first; Wasmtime
is in-scope only if it is already the selected host, not as a new platform
project.[^generation-plan]

**Out of bound:** Compiling wasm with `basic-cli`. Per-page Roc programs.

**Tests:** Two consecutive rebuilds with unchanged renderer hash do not invoke
`roc build`. Profile notes or spans show apply reuse (session-local binary
path, or `render` in the reused-binary class).

**Exit:** Watch body-edit `render` is in the ~30ms class on this machine, not
the 300–600ms first-run class, while Phase 1 `compile` 0 still holds.

### Phase 4 — Dirty-page apply, only if render is still large

**Gate:** After Phase 3, re-run the measurement command on a one-file watch
edit. Start this phase only if `render` remains hundreds of milliseconds on a
warm cached binary, not ~30ms.

**Bound if started:** Apply only output paths whose page JSON, article file,
or template hash changed. Unchanged concept HTML is left in staging. Graph
and unique-id checks still run over the whole loaded bundle during `load`;
this phase does not trim `okf::load`.[^okf-dev][^load-plan]

**Owner:** `OkfBuild` plus generate's page manifest.

**Exit:** Watch `render` scales with dirty pages, not with bundle size, on a
fixture of several concepts with one file touched.

### Phase 5 — Optional Rocdown pages-out-of-hash

**Gate:** Only if Rocdown watch still recompiles on ordinary article edits
after OKF Phases 1–3. Measure `rocdown` / `rocci-rocdown` separately; do not
assume OKF numbers.[^rocdown-build][^generation-research]

**Bound if started:** Same rule as Phase 1: `RocdownPages.roc` page records
must not participate in the compile hash; fragment HTML files already exist
for article bodies.

**Out of bound:** Changing Rocdown grammar or the catalog/shell split.

### Phase 6 — CLI honesty and recorded baseline

**Bound:** `run` must not set `force_roc` merely because `HostArg::Auto` is
always `Some`. Missing `roc` uses `build_review_site_pure_rust` unless
`--host native` or `ROCCI_REQUIRE_ROC=1`. Document that default preview with
Roc present uses the cached Rocci renderer; `--host native` remains the
explicit compile path.[^okf-main][^okf-readme][^presentation]

Refresh the compile/render research or a short Status snapshot with post-change
`--profile-report json` (cold first open, warm process, one-file watch). Do
not claim a phase complete in `knowledge/log.md` until required GitHub
workflows on that revision are green.[^research][^load-plan]

**Out of bound:** New product features. Making `--profile base` the fast
preview path.

## Layer map

| Concern | Owner |
| --- | --- |
| Page JSON / article files | `crates/rocci-okf/src/presentation.rs` generate |
| Apply runtime | `crates/rocci-okf/runtime/OkfBuild.roc` |
| Knowledge shell | `crates/rocci-okf/templates/OkfTheme.rocci` |
| Compile hash | `rocci-roc-host` helpers used by `rocci-okf` |
| Watch session / apply reuse | `crates/rocci-okf/src/dev.rs` |
| Preview CLI host policy | `crates/rocci-okf/src/main.rs` |
| Public commands | `crates/rocci-okf/README.md` |
| Measurement evidence | `knowledge/research/okf-compile-render-cost.md` |

## Risks

- Reading page JSON from Roc can reintroduce a hidden compile dependency if
  the decoder types or a generated type module still hash per-bundle field
  shapes. Keep one stable page record type in the renderer; put values in
  data files.[^okf-build][^generation-research]
- Wiring apply to disk before Phase 1 would serve stale embedded pages if
  someone ships research option B. Phase 2 must not land first.[^research]
- A long-lived apply process can hold a staging directory from a previous
  rebuild; Phase 3 must still write a coherent output tree (replace or
  overlay with a known prefix).
- Phase 6's Rust fallback will diverge from Rocci chrome whenever templates
  change. That is acceptable for no-Roc environments; do not document it as
  the supported authoring look.

## Open questions

1. Should page data be one `pages.json` or per-concept files plus a path
   list? Files already exist under `articles/`; JSON is the smaller apply
   contract for outlines and meta.
2. After Phase 3, is Phase 4 unnecessary on this repository’s current size?
3. Is Rocdown Phase 5 in this plan or a follow-up owned by `rocci-rocdown`?

[^research]: After load-performance work, watch body edits still pay ~0.5s compile and ~0.5s first apply; apply HTML is discarded; served chrome is Rust; durable fix is externalize page data then write Roc HTML.
[^generation-research]: Compiled `~/.rocci` programs pay off only when page data is passed in at apply time; compiling per save is the costly mistake.
[^generation-plan]: Two-tier gen-hash and compile-hash; native subprocess plus Wasmtime; glue later.
[^load-plan]: Phases 1–4 implemented; compile/render were out of scope until load fell below them; Phase 5 skipped.
[^load-status]: Warm-renderer release first-open total 357ms is not a cold-compile or content-edit baseline.
[^preview-audit]: Cached-renderer profiles hid compile (`compile` 0) and are not an edit-path baseline.
[^presentation]: `roc_source_hash` includes `pages_roc`; `force_roc` when `host` is `Some`; `write` fills missing apply outputs with `html_page`.
[^okf-build]: `render_all` maps pages to HTML strings; `main.roc` discards the list.
[^okf-theme]: Rocci shell links Home and Governance & Review; that markup was absent from the measured served page.
[^okf-dev]: Watch `ParseCache` already reuses unchanged Markdown; presentation spans are appended after load.
[^okf-main]: `run` passes `Some(host.into())`.
[^okf-readme]: Documents `run`, `check`, host flags, and preview policy.
[^roc-cache]: `lookup_renderer` keys on the caller-supplied compile hash.
[^roc-host]: Native apply is a subprocess; wasm is a distinct target.
[^rocdown-build]: Rocdown `roc_source_hash` also includes generated pages Roc.
[^engine-readme]: `okf` stays UI-neutral.
[^catalog-shell]: Catalog and governance data stay in Rust; visible shell is Rocci.
[^static-okf]: Canonical knowledge remains inert Markdown.
[^deps-check]: Workspace dependency direction is mechanical; `okf` must not grow a CLI or host edge.
[^cli-plan]: OKF preview stays on `rocci-okf`.
