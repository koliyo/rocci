---
type: Research Report
title: OKF preview compile and render cost after load-performance work
description: After Phases 1–4 of load-performance work, first-open and content-edit `rocci-okf run` is dominated by Roc `compile` and `render`. Page HTML is baked into generated Roc, so edits miss the renderer cache; apply output is discarded and the served site is the Rust write fallback.
tags: [domain/okf, domain/rocci-okf, integration/roc, concern/performance, concern/rendering, concern/caching, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T12:40:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: compile-plan
    resource: ../plans/okf-compile-render-cost.md
    title: OKF preview compile and render cost plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: load-plan
    resource: ../plans/okf-load-performance.md
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
  - id: generation-research
    resource: rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: generation-plan
    resource: ../plans/rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
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
    title: OKF knowledge shell with Home/Review links and ConceptMeta
    author: process:git
    last_modified: 2026-08-19
  - id: okf-dev
    resource: ../../crates/rocci-okf/src/dev.rs
    title: Headless rebuild spans and watch parse cache
    author: process:git
    last_modified: 2026-08-19
  - id: okf-main
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf run always passes a host choice into rebuild
    author: process:git
    last_modified: 2026-08-19
  - id: roc-cache
    resource: ../../crates/rocci-roc-host/src/cache.rs
    title: Two-tier generated-Roc and compiled-renderer cache
    author: process:git
    last_modified: 2026-08-18
  - id: roc-host
    resource: ../../crates/rocci-roc-host/src/host.rs
    title: HostChoice Auto resolution and native apply
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-build
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Rocdown applicator hash includes RocdownPages.roc
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
---

# OKF preview compile and render cost after load-performance work

## Scope and authority

This is exploratory measurement and synthesis, not a change to the portable
OKF engine or an approved implementation plan. It starts from the load-performance
plan's explicit gate: do not spend work on Roc compile unless `load` has fallen
below it.[^load-plan][^load-status]

That gate is now met on this repository. After Phases 1–4, default
`rocci-okf run` `load` is a few hundred milliseconds. Remaining first-open and
Markdown-edit cost is `compile` plus `render` in `build_review_site_with_host`.
This record asks what those spans do, why they miss cache on content edits, and
what it would take to cut them. Implementation plan:
[OKF preview compile and render cost](../plans/okf-compile-render-cost.md).
Not shipped.[^compile-plan][^presentation][^okf-dev][^preview-audit]

Timings below are machine-local on 2026-08-19. They are evidence, not a latency
SLA.

## How compile and render are measured

`rebuild_site` records `load` (with discover/parse/graph/provenance sub-spans)
then appends the presentation snapshot: `compile templates`, `generate`,
`compile`, `render`, `write`.[^okf-dev][^presentation]

`compile templates` is Rust lowering of four `.rocci` modules. `generate`
writes `OkfPages.roc` plus article files. `compile` is `roc build --opt=dev`
unless `TwoTierCache` hits. `render` is one native `apply` subprocess.
`write` copies staging plus Rust fallbacks for any missing `index.html`.[^presentation][^roc-cache]

`rocci-okf run` always passes `Some(host)` (default `Auto`). That sets
`force_roc`, so the pure-Rust `build_review_site_pure_rust` path is not used
on the preview CLI even when `roc` is missing.[^okf-main][^presentation]

## Measured problem

Maintainer-supplied first-open profile for
`rocci-okf run knowledge/plans/okf-load-performance.md` (parse cache cold):

| Stage | ms | Note |
| --- | --- | --- |
| total | 2372 | |
| load | 333 | |
| parse | 332 | `cache_hit=0 miss=54` |
| compile templates | 0 | |
| generate | 7 | |
| compile | 1352 | not cached |
| render | 641 | |
| write | 39 | |

Isolated release re-measure on this revision, `ROCCI_CACHE` empty, same command
with `--no-window --profile-report json`:

| Stage | Cold first open | Warm new process | Watch after one body sentence |
| --- | --- | --- | --- |
| total | 1382 | 348 | 1006 |
| load / parse | 272 / 271 (`miss=54`) | 270 / 269 (`miss=54`) | 28 / 26 (`hit=53 miss=1`) |
| compile | 727 | 0 (cached) | 461 |
| render | 334 | 31 | 467 |
| write | 41 | 32 | 39 |

The load-performance status snapshot's 357ms release first-open total is the
**warm renderer** case (`compile` 0). It is not the cold-compile or
content-edit case.[^load-status]

Relative costs that hold across both profiles:

1. Parse cache already does its job on watch (53 hits, one miss).
2. A Markdown body change still forces `roc build` and a slow first `apply`.
3. `write` stays ~30–40ms either way.
4. `compile templates` and `generate` are negligible.

This bundle is 43 catalog concepts, 10 indexes, and 54 Markdown files on the
parse path. Each renderer cache entry is a 4.8MB native `apply` binary.
`~/.rocci/cache/renderers` on this machine was 173MB across 54 hashes before
the isolated run; one body sentence added a second 4.8MB entry.[^roc-cache][^presentation]

## Why compile misses on content edits

`generate_okf_pages_roc` embeds every concept's `article_html`, outline,
governance view, and metadata as Roc string and record literals in
`OkfPages.roc`. `roc_source_hash` hashes that module together with template Roc,
`OkfBuild.roc`, `Html.roc`, and `main.roc`. The hash is the `TwoTierCache`
lookup key.[^presentation][^roc-cache]

A heading or sentence change therefore produces a new compile hash even though
`OkfTheme.rocci`, `ConceptMeta.rocci`, and `OkfBuild.roc` are unchanged.
`roc build` type-checks and codegens a program whose largest input is page
data, not chrome.

Rocdown uses the same shape: `roc_source_hash` includes `RocdownPages.roc`,
which still embeds page view records.[^rocdown-build][^generation-research]

The designed two-tier helpers `compute_gen_hash` / `compute_compile_hash` (template
identity versus `roc` version, target, opt, platform) are not what `rocci-okf`
passes to `lookup_renderer`. The cache is used as a content-addressed blob store
for whole programs that include page HTML.[^roc-cache][^presentation]

Prior generation-pipeline research already stated the rule: compiled programs
in `~/.rocci` only pay off if renderer source changes rarely and page data is
passed in at apply time. OKF preview currently violates that rule.[^generation-research]

## Why render is expensive, then cheap, then wasted

`OkfBuild.render_all` maps every page through `OkfTheme.knowledgeShell` and
builds a full HTML document string. `main.roc` binds that result to `_` and
returns `Ok({})`. There is no `File.write` and no use of `OKF_STAGING`.
`invoke_apply` therefore computes HTML for the whole bundle and throws it
away.[^okf-build][^presentation]

The `write` span then finds each `staging/<id>/index.html` missing and fills it
with `html_page` / `render_concept_meta` / `render_toc`. Served concept HTML on
the cold run contains the Rust “On this page” navigator, not the Rocci shell's
Home / Governance & Review links from `OkfTheme.rocci`.[^presentation][^okf-theme]

First execution of a freshly built `apply` was 334–641ms here. The same binary
on the next rebuild was ~30ms. Spawn plus page-in of a 4.8MB image dominates
the first `render`; later applies are cheap. Watch still starts a new `apply`
process every rebuild; it does not keep a live renderer.

So a content-edit rebuild currently pays ~0.5s of Roc codegen and ~0.5s of
first-run apply in order to discard the strings, then spends 39ms writing the
Rust site the browser actually loads.

## What it would take

Keep `okf` UI-neutral. Compile, cache, and apply stay in `rocci-okf` /
`rocci-roc-host`. Catalog parse and provenance stay in `okf`.[^engine-readme][^catalog-shell]

Options are ordered by expected wall-time drop on this bundle, not by product
completeness. A and B make preview fast while leaving Rocci chrome unshipped.
C is the durable fix if the review site is supposed to be Rocci.

### A. Stop paying for apply output that is not used

If `write` remains the source of truth, skip `roc build` and `invoke_apply` on
the default `run` path (or skip them whenever `host` is only `Auto` and the
cache key would have changed only because of `OkfPages.roc`).

Expected local effect: content-edit total in the same class as today's
parse-cache watch with compile already cached (~100ms here: parse 26 + generate
11 + write 39). Cold first open would be parse plus write (~300ms), matching
the load-performance “sub-second first open” claim for real.

Cost: `OkfTheme.rocci` still would not appear in preview. The force-Roc CLI
default would need an honest policy (`--host native` to opt into Roc, or a
later switch once C ships).[^okf-main][^presentation]

### B. Hash the renderer without page data, still discard apply

Drop `pages_roc` from `roc_source_hash` while `article_html` remains in
`OkfPages.roc`. Content edits would hit the existing 4.8MB binary.

Expected local effect: compile 0 on Markdown edits; first process still pays
one compile; watch `render` ~30ms if apply still runs, or ~0 if combined with
A. The cached binary's embedded pages would be stale, which is harmless only
while apply output is ignored.

This is a footgun the moment someone wires Roc HTML to disk. Prefer C over
shipping B as the long-term key.

### C. Externalize page data and write Roc HTML (durable)

Split the program from the pages, as the generation-pipeline research already
recommends:[^generation-research][^generation-plan]

1. Renderer identity = templates + `OkfBuild` + `Html` + platform + `roc`
   version + target + opt. Use `compute_gen_hash` / `compute_compile_hash`.
2. At apply time, pass page records as files or JSON (article HTML is already
   written under `articles/`; `OkfBuild` should read `article_path` instead of
   `article_html` literals).
3. Have `render_all` write each document to `OKF_STAGING` / `output_path`.
   Drop the Rust `if !destination.exists()` chrome once Roc output is the site.
4. Keep a long-lived apply process or in-process Wasmtime host across watch
   ticks so first-run 4.8MB page-in is paid once per process, not per save.
5. Optionally apply only dirty output paths after parse-cache hits.

Expected local effect after a warm renderer: watch rebuild ≈ parse dirty file
+ generate page list + apply dirty pages + write. Compile returns only when
`.rocci` / runtime / platform changes. First open still pays one `roc build`
(~0.5–1.3s here) unless a previous process stored that renderer.

Wasm `--host wasm` is not a compile shortcut today: `basic-cli` does not target
`wasm32`, and the hint in the apply failure path still calls wasm a later
host.[^presentation][^roc-host] In-process apply would help the ~30ms cached
`render`, not the `roc build`.

### D. Do not do these next

- Parallel Markdown parse. Watch parse is already 26ms with the cache; first-open
  parse (~270ms release) is smaller than cold compile.[^load-plan][^okf-dev]
- Bounded concept-path loading (load-performance Phase 5). Release `load` is
  already sub-second; it would not remove compile.[^load-plan]
- Embedding the Roc compiler in Rust, or per-`.rocci` object files. Roc still
  compiles a whole application.[^generation-research]
- Treating `~/.rocci/cache/renderers` LRU as a substitute for a stable key.
  173MB here is a symptom of hashing page bodies, not of missing eviction.

## Relation to existing records

The [load-performance plan](../plans/okf-load-performance.md) correctly left Roc
host caching out of Phases 1–4. Its non-goal on speeding native/Wasm compile is
still right as a *compiler* project; the remaining OKF work is to stop compiling
page data.[^load-plan]

The [generation-pipeline research](rocci-components-in-generation.md) predicted
this failure mode: OKF preview is acceptable with Rocci chrome only if
compilation happens on first use of an unchanged renderer, not on every concept
save.[^generation-research] That prediction now has numbers on the shipped
`rocci-okf` path.

Rust catalog versus Rocci shell remains the product split. The current preview
pays for a Rocci shell and then publishes the Rust catalog chrome.[^catalog-shell][^okf-theme][^presentation]

## Open questions

1. Should default `rocci-okf run` stay on the Rust write path until C lands
   (option A), or keep compiling unused Roc so the cache is warm when apply is
   wired up?
2. Is one whole-bundle apply with external page files enough, or should watch
   apply a single dirty concept?
3. Should Rocdown's `RocdownPages.roc` hash be fixed in the same change, or is
   OKF preview the only latency that matters for authoring?

[^compile-plan]: Phased delivery: externalize page data from the compile hash, write Rocci chrome from apply, reuse the applicator on watch; dirty-page apply and Rocdown gated. Exploratory; no phase started.
[^load-plan]: Phases 1–4 implemented; compile/render explicitly out of scope until load fell below them; Phase 5 skipped after a 290ms release first-open load.
[^load-status]: Release first-open total 357ms is the warm-renderer case; debug watch parse 4ms after a content change; not a latency contract.
[^preview-audit]: Earlier cached-renderer profiles hid compile by measuring `compile` 0; they are not a first-open or edit-path baseline for Roc.
[^generation-research]: Durable renderer cache requires page data out of generated Roc; compiling per save is the costly mistake.
[^generation-plan]: Two-tier `~/.rocci/cache/roc` and `renderers` keys; native subprocess plus later Wasmtime; glue later.
[^presentation]: `roc_source_hash` includes `pages_roc`; `generate_okf_pages_roc` inlines `article_html`; `force_roc` when `host` is `Some`; `write` fills missing apply outputs with `html_page`.
[^okf-build]: `render_all` maps pages to HTML strings; `main.roc` discards the list.
[^okf-theme]: Rocci shell labels TOC “Knowledge” and links Home and Governance & Review; that markup was absent from the measured served page.
[^okf-dev]: Watch `ParseCache` annotates parse hits/misses; presentation spans are appended after load.
[^okf-main]: `run` passes `Some(host.into())`, so the no-Roc write-only fallback is unreachable from the preview CLI.
[^roc-cache]: `lookup_renderer` / `store_renderer` key on the caller-supplied compile hash; OKF supplies a hash of pages plus templates.
[^roc-host]: `HostChoice::Auto` resolves from `ROCCI_HOST` or stays Auto; wasm is a distinct target, not a faster `basic-cli` build.
[^rocdown-build]: Rocdown `roc_source_hash` also includes generated pages Roc.
[^engine-readme]: `okf` stays UI-neutral; renderer work belongs in `rocci-okf` / `rocci-roc-host`.
[^catalog-shell]: Catalog and governance data stay in Rust; visible shell is the Rocci side of the split.
