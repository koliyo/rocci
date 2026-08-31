---
type: Audit
title: Rocci implementation structure review
description: Crate-level product boundaries are sound; F-01 through F-04 mixed-concern files are now private directory modules. The parser was left intact. Draft disposition from source evidence; no human verification.
tags: [domain/rocci, domain/rocdown, concern/architecture, concern/tooling, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-31T11:30:00Z }
stale_after: 2026-11-30
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/implementation-structure.md
    title: Internal module-structure improvement plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: system
    resource: ../../architecture/system-overview.md
    title: Rocci system overview
    author: process:cursor
    last_modified: 2026-08-31
  - id: catalog-decision
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:cursor
    last_modified: 2026-08-31
  - id: boundary-plan
    resource: ../../plans/rocdown/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: boundary-audit
    resource: ../rocdown/rocdown-boundary-refactor-review.md
    title: Rocdown product-boundary refactor completion review
    author: process:codex
    last_modified: 2026-08-31
  - id: generator
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:cursor
    last_modified: 2026-08-31
  - id: workspace
    resource: ../../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-30
  - id: deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace dependency-direction checker
    author: process:cursor
    last_modified: 2026-08-31
  - id: readme
    resource: ../../../README.md
    title: Rocci README
    author: human:nils
    last_modified: 2026-08-31
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan/mod.rs
    title: Rocdown build planner
    author: process:git
    last_modified: 2026-08-31
  - id: docs-rs
    resource: ../../../crates/rocci-rocdown/src/docs/mod.rs
    title: Rocdown article-block pipeline
    author: process:git
    last_modified: 2026-08-23
  - id: lower-rd
    resource: ../../../crates/rocci-rocdown/src/lower/mod.rs
    title: Rocdown document lowering
    author: process:git
    last_modified: 2026-08-25
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build/mod.rs
    title: Rocdown site build
    author: process:git
    last_modified: 2026-08-31
  - id: catalog-rs
    resource: ../../../crates/rocci-rocdown/src/catalog/mod.rs
    title: Rocdown catalog resolver
    author: process:git
    last_modified: 2026-08-31
  - id: rd-lib
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: Rocdown public facade
    author: process:git
    last_modified: 2026-08-23
  - id: rd-cargo
    resource: ../../../crates/rocci-rocdown/Cargo.toml
    title: Rocdown crate manifest
    author: process:git
    last_modified: 2026-08-23
  - id: parser-rs
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Rocci template parser
    author: process:git
    last_modified: 2026-08-25
  - id: lower-tpl
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: Rocci template lowering
    author: process:git
    last_modified: 2026-08-30
  - id: tpl-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: Rocci template crate root
    author: process:git
    last_modified: 2026-08-25
  - id: tpl-cargo
    resource: ../../../crates/rocci-template/Cargo.toml
    title: Rocci template crate manifest
    author: process:git
    last_modified: 2026-08-25
  - id: cli-lib
    resource: ../../../crates/rocci-cli/src/lib.rs
    title: Rocci CLI library surface
    author: process:git
    last_modified: 2026-08-30
  - id: cli-cargo
    resource: ../../../crates/rocci-cli/Cargo.toml
    title: Rocci CLI crate manifest
    author: process:git
    last_modified: 2026-08-23
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run/mod.rs
    title: rocci run orchestration
    author: process:git
    last_modified: 2026-08-30
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server/mod.rs
    title: Shared preview and live-reload server
    author: process:git
    last_modified: 2026-08-25
  - id: browse-rs
    resource: ../../../crates/rocci-cli/src/browse/mod.rs
    title: rocci browse gallery
    author: process:git
    last_modified: 2026-08-25
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Generated HTTP dispatcher
    author: process:git
    last_modified: 2026-08-30
  - id: rd-cli
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown command dispatch
    author: process:git
    last_modified: 2026-08-31
  - id: suite-audit
    resource: ../ops/workspace-test-suite.md
    title: Workspace test-suite review
    author: process:cursor
    last_modified: 2026-08-31
  - id: agents
    resource: ../../../AGENTS.md
    title: Rocci agent instructions
    author: process:git
    last_modified: 2026-08-31
---

# Rocci implementation structure review

## Executive verdict

The **workspace architecture is sound**. Product crates, one-way
dependencies, the Rust catalog / Rocci shell split, and the I/O-free
template crate match the approved contracts and ordinary Rust workspace
practice.[^system][^boundary][^catalog-decision][^tpl-cargo]

The **internal module architecture has not kept up**. After Rocs was
folded into `rocci-rocdown`, the public facade stayed one crate (correct)
but private seams were not preserved. The crate is now about 35k lines of
Rust in a flat `src/*.rs` layout. `plan.rs` is 4365 lines and owns theme
compilation, sidebar-forest projection, hashed assets, playground
payloads, discovery feeds, and `Pages.roc` emission in one module. The
same mixed-concern pattern shows up in `docs.rs`, `lower.rs`, `build.rs`,
and several `rocci-cli` files.[^plan-rs][^docs-rs][^boundary-plan]

This is structural debt, not a missing product. The paired
[implementation-structure plan](/plans/rocci/implementation-structure.md)
splits modules in place. It does not add commands, syntax, diagnostics, or
workspace crates.[^plan]

## Scope and method

Reviewed crate manifests, public facades, and the largest source files
against the system overview, the product-boundary decision, and the Rust
catalog / Rocci shell decision. Line counts are current working-tree
`*.rs` totals under `crates/` (generated `*.generated.rs` included; no
`target/`). Did not re-open the [boundary-refactor audit](/audits/rocdown/rocdown-boundary-refactor-review.md)
or the [workspace test-suite audit](/audits/ops/workspace-test-suite.md);
those remain the records for crate edges and suite budget.[^boundary-audit][^suite-audit]

Did not measure compile times. rustc's per-module codegen is the reason
large mixed files still matter for incremental rebuilds.

## What is solid and should be preserved

- **One-way product graph.** Base Rocci must not depend on Rocdown;
  `workspace_deps.py` fails closed on unclassified packages. Keep that
  checker; do not invent a third product crate to make files
  smaller.[^deps][^workspace][^readme]
- **`rocci-template` is a language crate.** It parses and lowers `.rocci`
  and does not invoke Roc, HTTP, or the desktop host. That is the right
  boundary for a compiler front-end.[^tpl-lib][^tpl-cargo]
- **Rust catalog, Rocci shell.** Discovery, identity, graph, navigation
  *data*, article HTML, and artifact planning stay in Rust. Visible chrome
  stays in `RocdownTheme.rocci` and `rocci-ui`. Do not grow a second
  template language in Rust to "clean up" `plan.rs`.[^catalog-decision][^generator]
- **Error style follows layer.** Language crates use `Diagnostic`. Site
  build, CLI, and preview use `anyhow`. That split is idiomatic. Do not
  force `thiserror` through the planner for its own sake.[^plan-rs][^tpl-lib]
- **Ungram-owned AST files** (`ast.generated.rs`, `pprint.generated.rs`)
  already sit beside hand-written parsers. Further parser splits should
  not fight that codegen.[^tpl-lib][^rd-lib]
- **Tests sit at the owning boundary.** Catalog tests do not need Roc;
  language tests do not need a server. Keep that rule while moving
  files.[^agents][^catalog-rs]

The boundary plan already warned: one product must not become one
oversized crate; keep private module seams for compile time and tests,
expose one facade. The facade landed; the seams did not.[^boundary-plan]

## Findings

### F-01 — `plan.rs` is several planners in one file

**Severity:** P1 maintainability.

4365 lines, of which the `#[cfg(test)]` module starts at line 2588
(~1778 test lines). Production code in the same file compiles theme
modules and block-painter Roc, projects lanes and the sidebar forest,
hashes and rewrites assets, loads playground session JSON, emits Atom and
`pages.json`, and writes `Pages.roc`.[^plan-rs]

`plan()` / `plan_preview()` as the *orchestrator* is the right entry.
The helpers are not one concern. `planned_page` takes fifteen arguments
and is `#[allow(clippy::too_many_arguments)]`. The sidebar forest
(directory peel, fold indexes, example source trees) is a view projection
over catalog `NavSection` data, not artifact I/O, but it lives next to
SHA-256 hashing and `PLAYGROUND_CSP`.[^plan-rs][^generator]

A large recursive-descent `Parser` impl is ordinary. A planner that also
compiles themes and embeds a WASM playground session is not.

### F-02 — `rocci-rocdown` is a flat 35k-line product crate

**Severity:** P1 module architecture.

About 35k Rust lines; 31 workspace files are ≥800 lines, and eight of the
twelve files ≥1500 lines are this crate or its tests. `src/` is a single
directory of 35 files. `lib.rs` is a barrel: it `pub use`s article,
catalog, config, docs, plan, site, standalone, service, and re-exports
`rocci_template` plus `rocci_theme`. Host-only modules are sprinkled with
`#[cfg(not(target_arch = "wasm32"))]` at the `mod` line rather than
grouped as a host subtree.[^rd-lib][^rd-cargo]

That matches the approved *product* facade. It does not match the
boundary plan's request for private seams. New site behavior (peel-by-id
nav, block packs, playground assets) keeps landing in `plan.rs` because
there is no `plan/` directory to put it in.[^plan-rs][^generator]

### F-03 — The same mixed-concern shape repeats next door

**Severity:** P1 in `docs.rs` and `lower.rs`; P2 in `catalog.rs` /
`build.rs`.

| File | Lines | Mixed concerns |
| --- | ---: | --- |
| `docs.rs` | 2851 | Field helpers, article tree, registry validation, HTML render, includes, `rocdown test` runner; tests from 2289 |
| `lower.rs` | 2763 | Island lowering, Markdown emission, `:kind` lowering, TOC / default page, source maps |
| `build.rs` | 1948 | Session, staging, Roc/Wasmtime invoke, output commit; tests from 950 |
| `catalog.rs` | 1817 | Types, route/graph resolve, navigation data, `RD2205`; tests from 1085 |

`catalog.rs` is the healthiest of these: it stays on deterministic
catalog work. `docs.rs` is the next split target after `plan.rs` because
validation, render, and the example runner do not need to share a file.
`build.rs` is mostly orchestration plus a large in-file integration
suite; extracting tests first is enough before any further
cut.[^docs-rs][^lower-rd][^build-rs][^catalog-rs]

`rewrite_urls` in `plan.rs` and `docs.rs` is the same longest-key-first
replace loop. That is leftover coupling, not two algorithms.[^plan-rs][^docs-rs]

### F-04 — `rocci-cli` is a public kitchen-sink library

**Severity:** P1 visibility and file size.

The crate is both the `rocci` binary and the generic driver that Rocdown
is allowed to depend on. `lib.rs` declares every command module `pub`.
Downstream uses a small set: `driver`, `serve`, `dev_server`, `logs`,
`inspect`, `profile`, `path_hint`, `error_page`, and
`playground`.[^cli-lib][^rd-cli][^cli-cargo]

`run.rs` (1813), `dev_server.rs` (2385), `browse.rs` (2152), and
`dispatch.rs` (1353) are the CLI-side god files. `browse` is a gallery
app living in the same compile unit as live-reload and generated
dispatch. Because the modules are `pub`, Rocdown could start calling
them without a new dependency edge the checker would notice.[^run-rs][^dev-server][^browse-rs][^dispatch-rs]

`rocci-rocdown-cli` `main.rs` (1475) is a clap tree plus standalone/site
runners. That is acceptable for a binary once the heavy work stays in
the libraries; it should not grow another planner.[^rd-cli]

### F-05 — A large `Parser` is not the same bug

**Severity:** P2, do not "fix" by line count.

`rocci-template/src/parser.rs` is 2897 lines: one `Parser` with recovery,
plus a short interpolation-scan test module at the end. That is a normal
hand-written recursive-descent file. Splitting it by declaration kind is
optional after the mixed-concern files move, and must keep cursor
monotonicity (`cur.pos > before` or `cur.bump()`).[^parser-rs][^agents]

`rocci-template/src/lower.rs` (1955) is closer to F-03: handler metadata,
HTML emission, CSS artifacts, and tests share one module. A directory
split by emission vs route metadata is reasonable; a rewrite of lowering
is not.[^lower-tpl]

### F-06 — Long argument lists and copied helpers

**Severity:** P2 local style.

`#[allow(clippy::too_many_arguments)]` appears on `planned_page`,
`run`, `dev_server` helpers, `driver`, `playground_html`, and the
Rocdown CLI runners. Context structs (page-plan input, run options)
remove the allow without changing behavior.[^plan-rs][^run-rs][^rd-cli]

`timed_ms` is copied in both crate `lib.rs` files. Harmless, but it
shows the facades grew by paste rather than a shared host-timing
helper.[^rd-lib][^tpl-lib]

### F-07 — Kitchen-sink compile tests are a suite problem, not this plan

**Severity:** already tracked.

`rocci-template/tests/compile.rs` (2114) and
`rocci-rocdown/tests/compile.rs` (1990) are golden/kitchen-sink files.
The [workspace test-suite audit](/audits/ops/workspace-test-suite.md)
owns overlap and budget. Do not split those files in this
structure pass except where a module move requires relocating a unit
test that currently lives *inside* `plan.rs` / `docs.rs`.[^suite-audit]

## Idiomatic reading

Best practice here is not "every file under 300 lines." It is:

1. **Crates own products and layers** — already done.
2. **Modules own one transformation** — catalog resolve, article render,
   artifact plan, theme compile, host invoke, CLI dispatch.
3. **Public API is the product facade; internals are `pub(crate)` or
   private** — `rocci-template` is closer than `rocci-rocdown` or
   `rocci-cli`.
4. **Inline tests are fine until they double the file.** Then
   `mod tests` belongs in `tests.rs` beside the production module.
5. **Do not extract a crate for a second consumer that does not
   exist.** The Phase 7 UI rule still applies to module splits.

The structure is therefore **architecturally right at the workspace
edge and locally overdue for directory modules**.

## Disposition (source evidence, 2026-08-31)

The paired plan's phases 0–8 are in the tree. This record stays
`draft`; the findings above are the review snapshot, not a claim that
a human closed the audit.[^plan]

- **F-01 / F-02 / F-03.** `src/plan/`, `src/docs/`, `src/lower/`,
  `src/catalog/`, and `src/build/` exist. Production files in those
  directories are at or under the 800-line preference except the Rocdown
  `lower/emitter.rs` leftover cluster (~834). `lib.rs` is still the
  barrel.[^plan-rs][^docs-rs][^lower-rd][^catalog-rs][^build-rs][^rd-lib]
- **F-04.** `src/run/`, `src/dev_server/`, `src/browse/`, and
  `src/dispatch/` exist. `dispatch`, `inspector`, `playground_compile`,
  `playground_html`, `roc_module`, and `runtime_assets` are
  `pub(crate)`. Modules the `rocci` binary, Rocdown, or tests import
  stay `pub`. `browse` is not imported by Rocdown.[^cli-lib][^run-rs]
- **F-05.** `parser.rs` is still one `Parser` (~2897 lines). Template
  lowering is `src/lower/`.[^parser-rs][^lower-tpl]
- **F-06.** In-crate `too_many_arguments` allows on touched planner and
  CLI helpers were replaced with context structs. Public `run()` still
  allows the lint because Rocdown calls it across the crate edge.
- **F-07.** Kitchen-sink `tests/compile.rs` files were not split.

Crate README internal-module maps match those directories.

## Recommended sequence

The [implementation-structure](/plans/rocci/implementation-structure.md)
sequence was followed. Further splits are out of this plan unless a
later change re-mixes concerns into a new god file. Stop if a split
would change diagnostics, HTML, or generated Roc.

## Validation record

- Line inventory: Python walk of `crates/**/*.rs` on 2026-08-31 (audit)
  and `wc -l` of the split directories after phases 0–8.
- Public consumers of `rocci_cli::` grepped from the workspace.
- `rewrite_urls` compared by reading both definitions; the shared
  helper now lives in `docs` and `plan/assets` delegates to it.
- Phase Exit commands were run as process evidence on the implementing
  branch. That is not a human verification event.

## Closure criteria

Human closure still requires a reviewer to confirm public crate APIs
are unchanged and the listed crate tests plus
`cargo fmt --all -- --check` are green on the declaring revision.
This draft does not record that review.

[^plan]: Paired no-feature module-split sequence.
[^system]: Workspace product boundaries and crate count.
[^catalog-decision]: Rust owns catalog data; Rocci owns chrome.
[^boundary]: Approved one-way Rocci / Rocdown / OKF ownership.
[^boundary-plan]: Warning that one product crate still needs private seams.
[^boundary-audit]: Completed crate-edge review; not this file-size review.
[^generator]: Planner owns artifacts; catalog owns identity and nav data.
[^workspace]: Current workspace members.
[^deps]: Mechanical classification and forbidden reverse edges.
[^readme]: Documented package roles.
[^plan-rs]: Current planner types, `plan_with_preview`, theme/nav/asset/playground helpers, and in-file tests.
[^docs-rs]: Article-block load, validate, render, include, example runner, and duplicate `rewrite_urls`.
[^lower-rd]: Single emitter for islands, Markdown, docs kinds, and page chrome.
[^build-rs]: Build session, staging, Roc invoke, and in-file integration tests.
[^catalog-rs]: Resolve, graph, navigation sections, and catalog tests.
[^rd-lib]: Barrel `pub use` and per-module `wasm32` gates.
[^rd-cargo]: Host-only dependencies behind `cfg(not(target_arch = "wasm32"))`.
[^parser-rs]: Single `Parser` recursive-descent implementation.
[^lower-tpl]: Template lowering of components, routes, and CSS artifacts.
[^tpl-lib]: Language-only crate root and compile pipeline.
[^tpl-cargo]: No Roc, HTTP, or desktop dependencies.
[^cli-lib]: Every CLI module declared `pub`.
[^cli-cargo]: Combined binary-plus-library crate.
[^run-rs]: Standalone and directory run orchestration.
[^dev-server]: Shared static/live preview server used by Rocdown.
[^browse-rs]: Gallery compiler in the CLI library compile unit.
[^dispatch-rs]: Generated dispatcher for verb-first routes.
[^rd-cli]: `rocdown` clap dispatch and documented `rocci_cli` imports.
[^suite-audit]: Existing record for kitchen-sink and Roc-gated tests.
[^agents]: Lowest-owning-boundary tests and monotonic scanner loops.
