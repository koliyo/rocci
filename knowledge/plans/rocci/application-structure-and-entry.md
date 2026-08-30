---
type: Implementation Plan
title: Implement directory-as-app standalone entry
description: "Make a standalone directory an app: at most one process @init, rocci run <dir> when the entry is unique, optional rocci.toml [app].entry, and keep rocci run File.rocci. Do not reuse apps.toml."
tags: [domain/rocci, concern/architecture, concern/developer-experience, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-30T11:00:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/application-structure-and-entry.md
    title: Application structure and run-entry research (lean direction B)
    author: process:cursor
    last_modified: 2026-08-30
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run.rs
    title: File-only standalone run, directory requires main.roc
    author: process:git
    last_modified: 2026-08-26
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: Generated main uses only the primary module's init
    author: process:git
    last_modified: 2026-08-25
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Sibling routes share one Context
    author: process:git
    last_modified: 2026-08-30
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: rocci run default argument is main.roc
    author: process:git
    last_modified: 2026-08-30
  - id: bundle
    resource: ../../../crates/rocci-cli/src/bundle.rs
    title: build --release directory picks one immediate .rocci or main.roc
    author: process:git
    last_modified: 2026-08-26
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: CLI contract for run
    author: process:git
    last_modified: 2026-08-30
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Per-module duplicate @init / @context
    author: process:git
    last_modified: 2026-08-25
  - id: config
    resource: ../../../crates/rocci-core/src/config.rs
    title: AppConfig has no entry field
    author: process:git
    last_modified: 2026-08-20
  - id: island-svc
    resource: ../../../crates/rocci-rocdown/src/service.rs
    title: Island service already allows one @context / @init module
    author: process:git
    last_modified: 2026-08-24
  - id: apps-toml
    resource: ../../../examples/rocci/apps.toml
    title: Site catalog; not a run manifest
    author: process:git
    last_modified: 2026-08-29
  - id: staging-tree
    resource: ../../../examples/rocci/staging-tree.md
    title: Catalog discovery, not every examples directory
    author: process:git
    last_modified: 2026-08-23
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-file standalone with @init
    author: process:git
    last_modified: 2026-08-25
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: View-only standalone, no @init
    author: process:git
    last_modified: 2026-08-25
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Unique @init plus UI sibling
    author: process:git
    last_modified: 2026-08-23
  - id: multi-page
    resource: ../../../examples/rocci/standalone/multi-page-streams/README.md
    title: Several route modules, unique @get:view("/")
    author: process:git
    last_modified: 2026-08-22
  - id: blocks-toml
    resource: ../../../examples/rocci/standalone/blocks/rocci.toml
    title: Nested app-root marker
    author: process:git
    last_modified: 2026-08-22
  - id: blocks-case
    resource: ../../case-studies/standalone-blocks-authoring.md
    title: backend/ui discovery via rocci.toml
    author: process:cursor
    last_modified: 2026-08-24
  - id: tooling-doc
    resource: ../../../docs/applications/tooling.rocdown
    title: Workflow page that already shows rocci run on a standalone directory
    author: process:git
    last_modified: 2026-08-25
  - id: standalone-doc
    resource: ../../../docs/applications/standalone.rocdown
    title: Published standalone contract
    author: process:git
    last_modified: 2026-08-22
  - id: custom-doc
    resource: ../../../docs/applications/custom.rocdown
    title: Directory run for authored main.roc
    author: process:git
    last_modified: 2026-08-24
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: Public rocci run wording
    author: process:git
    last_modified: 2026-08-30
  - id: glossary
    resource: ../../../docs/appendix/glossary.rocdown
    title: Standalone app defined as one module with init and view
    author: process:git
    last_modified: 2026-08-25
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: standalone_app glossary string
    author: process:git
    last_modified: 2026-08-25
  - id: root-readme
    resource: ../../../README.md
    title: File run for standalone, directory for custom
    author: human:nils
    last_modified: 2026-08-30
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Authoring locations and rocci.toml walk-up
    author: process:git
    last_modified: 2026-08-25
  - id: cli-plan
    resource: ../shared/cli-entry-points.md
    title: Three-CLI split; rocci run owns applications
    author: process:cursor
    last_modified: 2026-08-24
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: One process-lifetime State
    author: human:nils
    last_modified: 2026-08-16
---

# Implement directory-as-app standalone entry

## Goal

A standalone **app** is a directory. A `.rocci` file is a **module**. Process
`@init` / `@context` occur at most once per app (or not at all).
`rocci run File.rocci` stays first-class. `rocci run <dir>` runs the app when
the entry is unique. `rocci.toml` may name `[app].entry`. `apps.toml` stays
the site catalog.[^research][^apps-toml][^staging-tree]

## Out of bound

- Requiring `rocci.toml` (or any new manifest) for a one-file app
- Teaching `apps.toml` / `rocci run --app <id>` as the authoring run path
- Composing multiple `@init` into generated `State`
- Treating each `.rocci` file as its own HTTP process
- Changing the no-argument default from `main.roc` to `.`
- An interactive picker as the only resolver
- Changing `rocci view` / `browse` into a second app runner
- Absorbing Rocdown or OKF into `rocci run`[^cli-plan]
- Changing Rocdown island serve (it already refuses multiple init modules)[^island-svc]
- Handler syntax, Datastar transport, or server-owned-state policy[^server-state]

## Constraints that do not move

- One HTTP process, one durable `State`, produced by at most one
  `@init`.[^server-state][^driver][^dispatch]
- Per-module duplicate `@init` / `@context` stay errors.[^validate]
- `rocci run File.rocci` remains a first-class command and still loads
  sibling modules of that file’s app root.[^run-rs][^research]
- Custom depth is unchanged: a directory containing `main.roc` is an
  authored Roc app.[^custom-doc]
- App-root walk-up still stops at `.git` / Cargo `[workspace]` so a
  repository-root `rocci.toml` is not an app.[^run-rs][^blocks-case]
- Parser and lowering tests do not boot HTTP. Directory-resolve tests stay
  offline (`plan_standalone` / a shared resolver). Do not add `ROCCI_REQUIRE_ROC`
  to the default suite.
- `deny_unknown_fields` on `rocci.toml` stays. `entry` is an additive
  known field.[^config]

## Target contract

### Nouns

| Noun | Meaning |
| --- | --- |
| App | Directory: the path passed to `run`, or the walk-up root of a file (first non-workspace `rocci.toml`, else the start directory) |
| Module | One `.rocci` file in that tree |
| Entry | The module that is primary for generated `main.roc` (`Primary.State` / `Primary.init!` when present) |

### Named file

`rocci run File.rocci` sets entry to that file. After the tree is planned,
if any **other** module declares `@init` or `@context`, fail and name that
module (“process `@init` is in `Blocks.rocci`; run that file or the app
directory”). Zero inits remain legal.[^research][^styling]

### Directory without `main.roc`

Treat the argument as the start directory, walk up with the same app-root
rule as a file (so `rocci run blocks/backend` still finds `blocks/rocci.toml`
and stages `ui/`).[^blocks-case][^run-rs]

Then pick entry, first match wins:

1. `[app].entry` when `rocci.toml` is the app-root file and the field is set
2. Exactly one module with `@init` / `@context`
3. Else exactly one module with `@get:view("/")`
4. Else exactly one `.rocci` file in the tree
5. Else fail and list candidates (init modules, `view("/")` modules, other
   routed modules)

Step 3 is the no-init multi-page case: Dashboard owns `/`, Admin owns
`/admin`.[^multi-page] Step 2 covers Counter, live-counter, and
Blocks.[^counter][^live-counter][^blocks-toml]

Two `@init` modules in the tree always fail, even if `[app].entry` names
one of them. Fix by splitting directories or deleting the extra pair.

### `[app].entry`

Optional string, relative to the `rocci.toml` directory, must resolve to a
`.rocci` file inside the app root. Reject empty values and any path that
escapes the root (`..`, absolute). Unknown field names stay errors.[^config]

### `rocci build --release <dir>`

Use the same standalone resolver when `main.roc` is absent, including
recursive discovery and the uniqueness rules. Today it only accepts a
single *immediate* `.rocci` or `main.roc`.[^bundle]

### Bare `rocci run`

Still defaults to `main.roc`. Authors type `rocci run .` for a standalone
directory. Changing the default is a later decision gate.[^cli-main][^research]

## Delivery phases

### 1. App-level unique init on the file-run path

Bound: `crates/rocci-cli/src/run.rs`, `crates/rocci-cli/src/driver.rs` if
the check sits on `GenericAppPlan`. After `plan_standalone`, if more than
one module has `init` or `state_type`, return an error that names both
files. Named-file primary that is not the unique init module also fails
(the extra-init case when the user pointed at a UI file). Existing
examples keep a single init and stay green. Do not change directory
`resolve_entry` yet.

Exit: `cargo test -p rocci-cli` and `cargo fmt --all -- --check`. New
offline tests: two `@init` modules in one tree fail; `rocci run Ui.rocci`
fails when `App.rocci` owns `@init`; a view-only sibling next to one init
still plans with the init file as primary.

### 2. Directory resolve for `rocci run <dir>`

Bound: `run.rs` (and a small helper next to `standalone_app_root` /
`discover_standalone_tree` if that keeps `resolve_entry` readable).
`run()` on a directory without `main.roc` plans the standalone tree and
selects entry by steps 2–5 of the target contract (no `[app].entry`
yet). Extract `app_root_from(start_dir)` so file and directory share
walk-up. Keep the custom `main.roc` path first. Replace
`resolve_entry_directory_suggests_standalone_rocci` with tests that
**run the resolver** (not the server) for Counter, Styling,
live-counter, Blocks, and multi-page-streams. `rocci run examples/rocci/standalone`
fails as multiple inits.

Exit: `cargo test -p rocci-cli` and `cargo fmt --all -- --check`.
`resolve` (or `plan_standalone` on a directory) returns
`counter/Counter.rocci`, `styling/Styling.rocci`,
`live-counter/LiveCounter.rocci`, `blocks/backend/Blocks.rocci`,
`multi-page-streams/Dashboard.rocci`. Ambiguous and empty directories
error with a candidate list, not a `main.roc` hint.

### 3. Same resolver on `rocci build --release <dir>`

Bound: `crates/rocci-cli/src/bundle.rs` `resolve_server_input`. A
standalone directory uses the Phase 2 helper instead of “exactly one
immediate `.rocci`.” `main.roc` and explicit `.rocci` / `rocci.toml`
inputs stay as they are. Offline unit tests for one-file, unique-init
nested, and ambiguous directories.

Exit: `cargo test -p rocci-cli` and `cargo fmt --all -- --check`.
`resolve_server_input(live-counter)` is standalone
`LiveCounter.rocci`, not “pass one .rocci file.”

### 4. Optional `[app].entry`

Bound: `crates/rocci-core/src/config.rs` (`AppConfig.entry: Option<String>`),
core config tests, then the Phase 2 helper reads the field as step 1.
Validate the path stays under the app root. Blocks may set
`entry = "backend/Blocks.rocci"` (optional; uniqueness already picks it).
Do not add `apps.toml` fields. Do not require `entry` on Counter.

Exit: `cargo test -p rocci-core` `cargo test -p rocci-cli` and
`cargo fmt --all -- --check`. Config parses `entry`, rejects
`entry = "../Other.rocci"`, and directory resolve prefers a valid
`entry` when several `view("/")` modules exist.

### 5. Public contract and glossary

Bound: `docs/applications/tooling.rocdown` (the Counter directory
command becomes correct), `docs/applications/standalone.rocdown`,
`docs/applications/custom.rocdown`, `docs/reference/cli.rocdown`,
`docs/appendix/glossary.rocdown`, `docs/inventory.toml`,
`crates/rocci-cli/README.md`, root `README.md`,
`.agents/skills/rocci-author/SKILL.md`, and the research
[Current disposition](/research/rocci/application-structure-and-entry.md).
Glossary: a standalone app is a **directory** of modules with at most one
process `@init`, run by `rocci run <dir>` or `rocci run File.rocci`.
Do not restage `apps.toml` as a run file.

Exit: those pages state directory-as-app and the uniqueness rules.
`okmate check knowledge --profile base --format terminal`.
`cargo fmt --all -- --check` if Rust comments or clap help changed.

## Acceptance criteria

- `rocci run examples/rocci/standalone/counter` and
  `rocci run examples/rocci/standalone/counter/Counter.rocci` plan the
  same primary.
- `rocci run examples/rocci/standalone/live-counter` picks
  `LiveCounter.rocci` and still stages `LiveCounterUi.rocci`.
- `rocci run examples/rocci/standalone/blocks` picks
  `backend/Blocks.rocci` and stages `ui/`.
- `rocci run examples/rocci/standalone/multi-page-streams` picks
  `Dashboard.rocci`.
- Two `@init` modules in one tree fail before Roc typecheck.
- Custom `rocci run examples/rocci/custom/notes` is unchanged.
- `examples/rocci/apps.toml` is not read by `rocci run`.
- One-file apps do not need `rocci.toml`.

## Decision gates

Human review before implementing anything in **Out of bound**, and before
making bare `rocci run` mean the current directory.

Until those gates open, implement this sequence.

[^research]: Lean direction B: directory is the app; at most one process init; optional `rocci.toml` entry; not `apps.toml`.
[^run-rs]: Current file-only standalone run and `main.roc` directory resolve.
[^driver]: Generated `main` calls only `primary.init!`.
[^dispatch]: Merged sibling routes receive one `context`.
[^cli-main]: Clap default for `run` is `main.roc`.
[^bundle]: Release directory resolve is immediate-file uniqueness only.
[^cli-readme]: Documented run examples.
[^validate]: Per-module init/context pair.
[^config]: `AppConfig` is name, identifier, version; unknown keys error.
[^island-svc]: Rocdown islands already refuse multiple init modules.
[^apps-toml]: Site catalog rows with staging `entry`.
[^staging-tree]: Discovery is the catalog, not a walk of `examples/rocci`.
[^counter]: One-file `@init` + `@get:view("/")`.
[^styling]: `@get:view("/")` and no process state.
[^live-counter]: Unique init plus UI module in one directory.
[^multi-page]: Dashboard `/` plus Admin `/admin`; no `@init`.
[^blocks-toml]: Nested app-root `rocci.toml`.
[^blocks-case]: Walk-up from `backend/Blocks.rocci` stages `ui/`.
[^tooling-doc]: Already shows `rocci run examples/rocci/standalone/counter`.
[^standalone-doc]: Generated dispatch; authors do not write `main`.
[^custom-doc]: Directory + `main.roc` is the custom depth.
[^cli-ref]: Current public wording: standalone file or directory/`main.roc`.
[^glossary]: Standalone app defined as one module with context, init, and view.
[^inventory]: Same glossary string in coverage inventory.
[^root-readme]: File command for standalone, directory for custom.
[^author-skill]: App-root `rocci.toml` versus repository-root config.
[^cli-plan]: `rocci run` owns applications only.
[^server-state]: Durable state is server-owned and process-lifetime.
