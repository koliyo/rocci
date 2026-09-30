---
type: Research Report
title: Roc nightly September 18 compatibility and opportunities
description: Local macOS arm64 validation of 1d982dc versus 62fcb65 found one repaired Wasm export break, working empty nominal defaults, indexed iterators, and automatic dev object caching.
tags: [domain/rocci, integration/roc, concern/validation, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-09-30T11:18:19Z }
stale_after: 2026-12-30
authority: exploratory
owners: [human:nils]
sources:
  - id: results
    resource: ./roc-nightly-2026-09-18-results.json
    title: Local compiler probes and validation results
    author: process:cursor
  - id: comparison
    resource: https://github.com/roc-lang/roc/compare/62fcb65bfccb2467604635fda6747786654dab19...1d982dca644aaddf1cc858f8580fadccf025358b
    title: Exact upstream compiler comparison
    author: organization:roc-lang
  - id: wasm-change
    resource: https://github.com/roc-lang/roc/pull/11145
    title: Require explicit Wasm exports and reject unknown symbols
    author: organization:roc-lang
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Embedded platform target and explicit main export
    author: process:git
  - id: wasm-apply
    resource: ../../../crates/rocci-rocdown/src/build/invoke.rs
    title: Wasm stub and subsequent native applicator
    author: process:git
  - id: wasm-test
    resource: ../../../crates/rocci-rocdown/src/build/tests.rs
    title: Wasm and native site parity regression
    author: process:git
  - id: defaults-change
    resource: https://github.com/roc-lang/roc/pull/11279
    title: Nominal defaults through polymorphic calls
    author: organization:roc-lang
  - id: defaults-history
    resource: ./nominal-props-defaults-dx.md
    title: Historical September 3 defaults limitations
    author: process:cursor
  - id: props
    resource: ../../../crates/rocci-template/src/ast.rs
    title: Nominal props and Bool default fallback
    author: process:git
  - id: iterator-change
    resource: https://github.com/roc-lang/roc/pull/11101
    title: Iter.with_index and specialized step_by adapters
    author: organization:roc-lang
  - id: cache-change
    resource: https://github.com/roc-lang/roc/commit/90469cef282018df15d5d8d94cc28f6055a767a4
    title: Enable the object store for cached dev builds
    author: organization:roc-lang
  - id: dev-backend
    resource: ../../../crates/rocci-rocdown/src/islands.rs
    title: Site island build and dev optimization choice
    author: process:git
  - id: pin
    resource: ../../../.roc-version
    title: Product compiler pin
    author: process:git
---

# Roc nightly September 18 compatibility and opportunities

## Scope and conclusion

Compared the installed Apple Silicon macOS binaries
`nightly-2026-09-03-62fcb65` and `nightly-2026-09-18-1d982dc` on September 30.
The new compiler is ahead by 1,026 commits; the comparison was retrieved in
eleven pages rather than relying on GitHub's truncated default response.
The shell still resolved the old compiler, so every new-compiler check used
an explicit `PATH` override. The repository pin and install inventory now
select September 18. No global installation or shell configuration was
changed.[^comparison][^results][^pin]

After repairing one target-header incompatibility, the Roc-gated suite and
offline workspace suite passed. This establishes local macOS compatibility
for their coverage, not Linux packaging, Windows, or native webview interaction.
Two additional example builds fail on both compilers; they are existing
problems, not new regressions.[^results]

## Compatibility repair

Upstream now requires an explicit `exports` list for linked Wasm targets.
The embedded platform omitted that field, so
`build::tests::wasm_host_build` failed with **Missing Wasm Exports**. The
target now declares `exports: ["main"]`, matching the entry function in its
C object. An attempted `_start` export failed because that object does not
define `_start`. The corrected regression passed.[^wasm-change][^wasm-platform][^results]

The existing Rocdown Wasm site path builds and instantiates a no-op app,
then builds and runs the native applicator for the actual page rendering.
The parity test covers this combined path. It does not demonstrate
independent Wasm rendering. The owning README and public site reference
now describe that limitation; this investigation did not expand the Wasm
runtime.[^wasm-apply][^wasm-test]

## Opportunities verified locally

| Candidate | September 3 | September 18 | Consequence |
| --- | --- | --- | --- |
| `get({})` for an all-defaulted nominal record | Type mismatch | Pass in dev and speed | Existing generated `<Hello />` calls can now work without filling fields in Rocci. |
| Annotated `value : Config = id({})` | Type mismatch | Pass in dev and speed | Expected nominal types can propagate through annotated polymorphic constructions. |
| Annotated `List(Config)` using `List.repeat({}, 2)` or `List.map([1, 2], |_| {})` | Type mismatch | Pass in dev and speed | Useful for fixture and view construction with explicit type edges. |
| Inferred `get(id({}))` and `List.map([{}, {}], get)` in the saved probe | Type mismatch | Still type mismatch | Do not assume every inferred construction absorbs defaults. |
| Nominal `Bool ?? True` | Explicit nominal construction worked; empty `{}` failed | Both explicit nominal and empty construction pass in dev and speed | Bool defaults are a possible lowering simplification, not implemented here. |
| `Bool.true` default expression | Does not exist | Does not exist | Rocci's current Bool spelling conversion still needs a deliberate change before removing its fallback. |
| Structural `{ value : I64 ?? 42 }` | Rejected | Rejected | Keep generated nominal Props; this upgrade does not restore structural defaults. |
| `[10.I64, 20, 30].iter().with_index()` | Missing method | Pass in dev and speed | Ordinary Roc helpers can use indexed iterator pipelines. |

The table records executable probe outcomes; exact sources, diagnostics,
and exit codes are in the adjacent results file. In particular, a Roc
summary saying some tests passed was not treated as success when the
process also returned a type error.[^results]

The defaults changes align with the upstream empty-record and polymorphic
construction fixes. A generated `.rocci` probe with `Hello`, an empty
`<Hello />` parent call, and render assertions passes six expects on the new
compiler; the old compiler reports `{}` versus `HelloProps`. This resolves
that historical direct-call limitation in the probe without changing Rocci's
nominal Props design.[^defaults-change][^defaults-history][^results]

Rocci currently skips nominal backing records containing Bool defaults and
fills them at call sites. A later focused change could emit `True` / `False`
directly and remove that fallback after testing mixed props, component calls,
fixtures, Node rendering, and string rendering. This report does not claim
that change is shipped.[^props][^results]

The indexed iterator addition also specializes `step_by`; the saved local
probe verifies `with_index`, while `step_by`'s changes were inspected
upstream only. No new Rocci directive is necessary to use ordinary Roc
iterator helpers.[^iterator-change][^results]

## Compiler performance and remaining workarounds

The dev object store is enabled for builds that allow caching; it no longer
requires setting `ROC_OBJECT_CACHE`. The enabling commit supersedes the
earlier pack-store PR's opt-in description. Local builds emitted `pack hits`
without an opt-in variable. This is an automatic compiler benefit, not a
reason to add configuration to Rocci.[^cache-change][^results]

Successful optimized app builds were sampled once per compiler:

| App | September 3 wall time | September 18 wall time |
| --- | --- | --- |
| Counter | 7.234 s | 2.436 s |
| Live counter | 5.535 s | 4.365 s |
| Custom Datastar | 9.588 s | 8.558 s |

These are end-to-end samples, not controlled benchmarks. Compiler caches,
concurrent checks, asset staging, and build state were not normalized. They
support successful compilation, not a speedup percentage or a runtime
performance claim.[^results]

A small recursive two-error union passes dev and speed on both compilers.
That alone does not establish that the larger optimized-backend failures
behind existing workarounds are fixed. Keep site island packaging on dev
until the Linux target and representative live site paths have been
measured and smoked. No multipart decoder or backend-policy change was made
here.[^results][^dev-backend]

## Validation and pre-existing failures

| Check | Result |
| --- | --- |
| `ROCCI_REQUIRE_ROC=1 cargo test -p rocci-cli -p rocci-rocdown -p rocci-rocdown-cli -p rocci-roc-host` | 747 passed, 0 failed, 1 ignored across 21 test binaries/doc-test groups |
| `cargo test --workspace` with the new compiler on PATH, Roc gate unset | 1,302 passed, 0 failed, 14 ignored across 84 groups |
| Fresh `ROCCI_CACHE` documentation build | 54 static pages published; generated Roc compiled with zero errors/warnings |
| Counter, live-counter, custom Datastar `--release --opt speed` | Compile on both versions |
| Styling `--release --opt speed` | Fails on both: generated `Html` annotation versus `HtmlNode` |
| Blocks `--release --opt speed` | Fails on both: game-state record shape, including missing `last_tick_ms` |

The Roc-gated suite includes real generated-app HTTP and SSE checks,
component snapshots, standalone document and hybrid-site checks, and the
Wasm/native combined-path regression. The workspace counts overlap that
suite and should not be summed as distinct tests. Existing user edits,
including the styling example and component references, were preserved.
The first sandboxed run denied local TCP ports/cache access; the recorded
passing suite was rerun with those operations allowed.[^results][^wasm-test]

Local validation is not hosted CI completion. Linux release artifacts,
WASI HTTP components against the sibling experimental platform, and native
desktop interaction were not exercised by this investigation.[^results]

The final documentation rebuild again published 54 pages and the generated
site-reference page contains the Wasm limitation. `cargo fmt --all -- --check`
and `git diff --check` passed. `okmate check knowledge --profile base` passed
with no errors and 37 existing warnings: 34 unused-source citations
(`OKF4002`) and three broken concept links (`OKF3002`). There were no
diagnostics for this new record and no lifecycle/provenance warnings in
that base-profile output. These final checks are recorded in the results
file.[^results]

[^results]: Saved probes, exact compiler identities, process results, and local suite/build counts.
[^comparison]: Full comparison between the two nightly commit IDs.
[^wasm-change]: Required final function exports for linked Wasm targets.
[^wasm-platform]: Current explicit `main` export and embedded platform contract.
[^wasm-apply]: `main_roc(true)` is a no-op; `apply_html` follows Wasm execution with native compilation and application.
[^wasm-test]: `wasm_host_build` checks final HTML against the native build.
[^defaults-change]: Polymorphic nominal-default propagation, with annotated expected types in the upstream regressions.
[^defaults-history]: Historical September 3 direct empty-record and nominal Props constraints.
[^props]: `component_props_backing_record` skips Bool; `roc_type_default_expr` maps Bool literals to lowercase names.
[^iterator-change]: Indexed iterator adapter and step-by specialization.
[^cache-change]: Compiler object-cache enablement follows `--no-cache`, without `ROC_OBJECT_CACHE`.
[^dev-backend]: Existing site island compile-hash and build choice remain dev.
[^pin]: Product pin now selects the validated candidate.
