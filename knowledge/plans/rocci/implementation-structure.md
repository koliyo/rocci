---
type: Implementation Plan
title: Split oversized Rocci modules without adding features
description: Move mixed-concern code in rocci-rocdown, rocci-template, and rocci-cli into private directory modules with unchanged public APIs, diagnostics, and generated output.
tags: [domain/rocci, domain/rocdown, concern/architecture, concern/tooling, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-31T09:00:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: audit
    resource: ../../audits/rocci/implementation-structure.md
    title: Rocci implementation structure review
    author: process:cursor
    last_modified: 2026-08-31
  - id: boundary-plan
    resource: ../rocdown/rocdown-boundary-refactor.md
    title: Rocdown product-boundary refactor plan
    author: process:cursor
    last_modified: 2026-08-31
  - id: catalog-decision
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: generator
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:cursor
    last_modified: 2026-08-31
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Rocdown build planner
    author: process:git
    last_modified: 2026-08-31
  - id: docs-rs
    resource: ../../../crates/rocci-rocdown/src/docs.rs
    title: Rocdown article-block pipeline
    author: process:git
    last_modified: 2026-08-23
  - id: lower-rd
    resource: ../../../crates/rocci-rocdown/src/lower.rs
    title: Rocdown document lowering
    author: process:git
    last_modified: 2026-08-25
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown site build
    author: process:git
    last_modified: 2026-08-31
  - id: catalog-rs
    resource: ../../../crates/rocci-rocdown/src/catalog.rs
    title: Rocdown catalog resolver
    author: process:git
    last_modified: 2026-08-31
  - id: rd-lib
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: Rocdown public facade
    author: process:git
    last_modified: 2026-08-23
  - id: parser-rs
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Rocci template parser
    author: process:git
    last_modified: 2026-08-25
  - id: lower-tpl
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Rocci template lowering
    author: process:git
    last_modified: 2026-08-30
  - id: cli-lib
    resource: ../../../crates/rocci-cli/src/lib.rs
    title: Rocci CLI library surface
    author: process:git
    last_modified: 2026-08-30
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run.rs
    title: rocci run orchestration
    author: process:git
    last_modified: 2026-08-30
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Shared preview and live-reload server
    author: process:git
    last_modified: 2026-08-25
  - id: browse-rs
    resource: ../../../crates/rocci-cli/src/browse.rs
    title: rocci browse gallery
    author: process:git
    last_modified: 2026-08-25
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated HTTP dispatcher
    author: process:git
    last_modified: 2026-08-30
  - id: rd-cli
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown command dispatch
    author: process:git
    last_modified: 2026-08-31
  - id: rd-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Rocdown crate README
    author: process:git
    last_modified: 2026-08-31
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: Rocci CLI README
    author: process:git
    last_modified: 2026-08-30
  - id: suite-audit
    resource: ../../audits/ops/workspace-test-suite.md
    title: Workspace test-suite review
    author: process:cursor
    last_modified: 2026-08-31
  - id: agents
    resource: ../../../AGENTS.md
    title: Rocci agent instructions
    author: process:git
    last_modified: 2026-08-31
---

# Split oversized Rocci modules without adding features

## Goal

Restore private module seams inside the existing product crates so a
change to sidebar projection, theme painters, or live-reload does not
require editing a multi-thousand-line file, while generated Roc, HTML,
diagnostics, and public crate APIs stay the same.[^audit][^boundary-plan]

## Out of bound

- New commands, flags, diagnostic codes, syntax, site features, or
  playground behavior.
- New workspace crates, or moving types across the Rocci / Rocdown /
  OKF package edge.
- Replacing the Rust catalog / Rocci shell split, or rendering ordinary
  prose in Roc.[^catalog-decision][^generator]
- Rewriting the hand-written parsers to generated walkers (that is
  [ungram-ast](ungram-ast.md) / [ungram-follow-ons](ungram-follow-ons.md)).
- Splitting `tests/compile.rs` kitchen sinks or changing Roc-gated
  suite policy (that is the [workspace test-suite](/audits/ops/workspace-test-suite.md)
  plan).[^suite-audit]
- Extracting a `rocci-driver` crate. Tighten `pub` vs `pub(crate)`
  inside `rocci-cli` instead.[^cli-lib]
- Drive-by rustfmt/clippy cleanups outside the files a phase touches.

## Constraints that do not move

1. **Behavior freeze.** Same diagnostics, routes, hashed names, CSP
   strings, `Pages.roc` shape, and article HTML. Prefer moving functions
   over rewriting them.[^plan-rs]
2. **One product facade.** `rocci_rocdown::plan`, `BuildPlan`, and the
   current `pub use` names remain. New files are `mod` / `pub(crate)`,
   not a second public planner API.[^rd-lib]
3. **No reverse crate edges.** Do not teach `rocci-template` about
   sites, or base Rocci about Rocdown types.[^boundary-plan]
4. **Scanner loops stay monotonic** on every parse/lower path.[^agents]
5. **Tests stay at the owning boundary.** Planner unit tests do not
   start Roc; catalog tests do not compile the theme unless they
   already do.[^agents][^catalog-decision]
6. **Size budget after a split:** prefer ≤800 lines of *production*
   code per file. A cohesive `Parser` impl may exceed that. A file that
   is still mostly tests after extraction should move `mod tests` to
   `tests.rs` in the same directory.
7. **One phase per commit.**

## Phase 0 — Freeze the map

Bound:

- Add an "Internal modules" subsection to the Rocdown and Rocci CLI
  READMEs: current `src/*.rs` roles, the target `plan/` / `docs/` /
  `lower/` directories, and the rule that new site-planning code goes
  in the owning subdirectory.[^rd-readme][^cli-readme]
- Do not change Rust APIs or commands.
- Point both READMEs at this plan and the
  [structure audit](/audits/rocci/implementation-structure.md).

**Exit:** README subsections exist; `cargo fmt --all -- --check`.

## Phase 1 — Extract `plan.rs` tests

Bound:

- Replace `crates/rocci-rocdown/src/plan.rs` with `src/plan/mod.rs`
  (move the current file body) and `src/plan/tests.rs` for the existing
  `#[cfg(test)]` module. Keep test names and fixtures.[^plan-rs]
- Update `lib.rs` `mod plan` only if the path changes to a directory.
  Public `plan`, `plan_preview`, `BuildPlan`, `PublishReport`,
  `DEFAULT_CSP` stay on the crate facade.[^rd-lib]
- No theme/nav/asset splits yet.

**Exit:** `cargo test -p rocci-rocdown plan` (and any integration tests
that construct a `BuildPlan`); `cargo fmt --all -- --check`.
`src/plan/mod.rs` production code is unchanged aside from the moved
`mod tests`.

## Phase 2 — Split the planner by concern

Bound:

- Inside `src/plan/`, move existing helpers without new behavior:

  | Module | Takes from today's `plan.rs` |
  | --- | --- |
  | `mod.rs` | `plan` / `plan_preview` / `plan_with_preview`, `BuildPlan` |
  | `theme.rs` | `compile_*`, block-painter Roc, `CompiledThemeModule` |
  | `nav.rs` | lanes, forest, breadcrumbs, example source tree |
  | `assets.rs` | hash, rewrite, `PlannedAsset` |
  | `playground.rs` | playground session/manifest/CSP extras |
  | `emit.rs` | `pages_roc`, discovery JSON/Atom, `not_found_*` |

- Introduce a private page-plan context struct so `planned_page` drops
  `#[allow(clippy::too_many_arguments)]` without changing field
  values.[^plan-rs]
- Keep catalog `NavSection` as the data input; do not move forest
  construction into `catalog.rs` in this phase. Catalog still owns
  identity and nav *data*; the planner still owns `NavGroupView`
  projection.[^catalog-decision][^catalog-rs][^generator]
- Do not add kinds, layouts, or CSP features.

**Exit:** `cargo test -p rocci-rocdown`; `cargo fmt --all -- --check`.
No production file under `src/plan/` exceeds ~800 lines except a
documented leftover that is still one function cluster.

## Phase 3 — Split `docs.rs`

Bound:

- `src/docs/mod.rs` plus private modules for fields, article tree,
  registry validation, HTML/search render, includes, and the example
  runner. Public functions listed in `lib.rs` keep their paths via
  `pub use`.[^docs-rs][^rd-lib]
- Dedup `rewrite_urls` by calling the planner asset helper or a small
  `pub(crate)` function in one place. Same longest-key-first
  replace.[^docs-rs][^plan-rs]
- Move the in-file `#[cfg(test)]` module to `src/docs/tests.rs`.

**Exit:** `cargo test -p rocci-rocdown docs`;
`cargo fmt --all -- --check`.

## Phase 4 — Extract catalog and build tests

Bound:

- `src/catalog/mod.rs` + `tests.rs` if the production file is still
  over budget after the test move. Split types vs resolve vs graph vs
  nav *only* if a file remains mixed after extraction. Do not change
  diagnostic codes or `resolve` signatures.[^catalog-rs]
- `src/build/mod.rs` + `tests.rs` for the in-file integration suite.
  Leave session/staging/Roc invoke together unless a second natural
  file appears (for example `invoke.rs`).[^build-rs]
- Do not touch `tests/compile.rs`.[^suite-audit]

**Exit:** `cargo test -p rocci-rocdown catalog build`;
`cargo fmt --all -- --check`.

## Phase 5 — Split Rocdown `lower.rs`

Bound:

- Directory `src/lower/` with the existing `lower` / `lower_islands`
  entries plus private emitter, Markdown, docs-kind, and island
  modules. Keep one `Emitter` type or a thin wrapper; do not change
  source-map origins or generated Roc text.[^lower-rd]
- Preserve monotonic emit/scan progress.

**Exit:** `cargo test -p rocci-rocdown`;
`cargo test -p rocci-rocdown --test compile`;
`cargo fmt --all -- --check`.

## Phase 6 — Split template lowering; leave the parser

Bound:

- `rocci-template/src/lower/` for route metadata vs HTML/CSS emission
  if both sides of `lower.rs` remain large after moving tests. Public
  `lower`, `LoweredModule`, and `LowerOptions` stay.[^lower-tpl]
- Do **not** split `parser.rs` unless a declaration-vs-template-item
  seam is already obvious and tests stay green. A single `Parser` impl
  is allowed over the line budget.[^parser-rs][^audit]
- Do not change handler lowering or CSS artifact bytes.

**Exit:** `cargo test -p rocci-template`; `cargo fmt --all -- --check`.

## Phase 7 — CLI visibility and file splits

Bound:

- Inventory `rocci_cli::` imports. Keep `pub` only for modules Rocdown
  or tests already use (`driver`, `serve`, `dev_server`, `logs`,
  `inspect`, `profile`, `path_hint`, `error_page`, `playground`, and
  any other current external import). Mark the rest `pub(crate)`.[^cli-lib][^rd-cli]
- Split `run.rs`, `dev_server.rs`, `browse.rs`, and `dispatch.rs` into
  directories only after visibility is tight. `browse` must not become
  a dependency of Rocdown.[^run-rs][^dev-server][^browse-rs][^dispatch-rs]
- Replace `too_many_arguments` allows on touched functions with a
  context struct when the call sites are in-crate.
- Do not add CLI flags or change `rocci run` / `rocdown run` help.

**Exit:** `cargo test -p rocci-cli`; `cargo test -p rocci-rocdown-cli`;
`cargo fmt --all -- --check`. `rg 'use rocci_cli::'` still compiles.

## Phase 8 — Facade note and knowledge

Bound:

- Refresh the README module maps from Phases 2–7. Mention that
  `lib.rs` remains the product barrel.[^rd-lib][^rd-readme]
- Revise this plan's status section and the
  [structure audit](/audits/rocci/implementation-structure.md)
  disposition from source evidence. Keep both `draft`; do not add a
  human verification event.
- Do not change public Rocdown reference pages unless a README sentence
  is already user-facing and now names a moved *internal* path (prefer
  crate README only).

**Exit:** `okmate check knowledge --profile base --format terminal`
(report lifecycle warnings separately);
`cargo fmt --all -- --check`.

## Cross-phase validation

Every code phase:

- `cargo fmt --all -- --check`
- Focused crate tests listed in that phase Exit
- No new `RD*` / template diagnostic codes
- `git diff` shows moves and `mod` declarations, not rewritten
  algorithms

After Phase 2 or later planner work, if docs layout or navigation was
in the touched functions:
`cargo run -q -p rocci-rocdown-cli -- check docs`.

## Principal risks

- **Silent golden drift.** Keep existing test names; if a golden must
  change, stop and treat it as a behavior change.
- **Circular `plan` / `docs` / `catalog` modules.** Only `pub(crate)`
  helpers that already called each other; do not introduce new
  re-exports.
- **Parser split breaks recovery.** That is why Phase 6 leaves
  `parser.rs` intact by default.

[^audit]: Findings F-01 through F-06 and the no-feature disposition.
[^boundary-plan]: Keep one facade; preserve private seams.
[^catalog-decision]: Catalog data vs shell vs planner projection.
[^generator]: Planner owns artifacts; do not invent a Rust theme language.
[^plan-rs]: Current mixed planner and in-file tests.
[^docs-rs]: Article pipeline and duplicate URL rewrite.
[^lower-rd]: Single Rocdown emitter.
[^build-rs]: Build session and in-file integration tests.
[^catalog-rs]: Resolve and navigation data.
[^rd-lib]: Public barrel that must keep names.
[^parser-rs]: Cohesive recursive-descent parser.
[^lower-tpl]: Template lowering to split only by existing seams.
[^cli-lib]: All-public CLI modules.
[^run-rs]: Run orchestration to split after visibility.
[^dev-server]: Shared preview server still used by Rocdown.
[^browse-rs]: Gallery module that must stay CLI-private.
[^dispatch-rs]: Generated dispatcher.
[^rd-cli]: Current `rocci_cli` imports from the Rocdown binary.
[^rd-readme]: Rocdown crate contract; internal map only.
[^cli-readme]: CLI crate contract; internal map only.
[^suite-audit]: Kitchen-sink compile tests stay on the suite plan.
[^agents]: Owning-boundary tests and monotonic scanners.
