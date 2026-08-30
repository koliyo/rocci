---
type: Research Report
title: Rocci application structure and run entry
description: "Shipped standalone assembly treats a directory as a bag of modules with a CLI-chosen primary; rocci run on a directory still requires main.roc. Recommend directory-as-app, at most one process @init, and optional rocci.toml entry—not apps.toml and not a new manifest species."
tags: [domain/rocci, concern/architecture, concern/developer-experience, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-30T11:00:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: run-rs
    resource: ../../../crates/rocci-cli/src/run.rs
    title: Standalone discovery, app-root walk-up, and directory resolve_entry
    author: process:git
    last_modified: 2026-08-26
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: GenericAppPlan uses only the primary module's @init
    author: process:git
    last_modified: 2026-08-25
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Merged sibling routes share one generated Context
    author: process:git
    last_modified: 2026-08-30
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: rocci run default entry is main.roc
    author: process:git
    last_modified: 2026-08-30
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: Documented run of a file versus a custom directory
    author: process:git
    last_modified: 2026-08-30
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Per-module duplicate @context / @init diagnostics
    author: process:git
    last_modified: 2026-08-25
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Public standalone HTTP contract for one module
    author: process:git
    last_modified: 2026-08-30
  - id: config
    resource: ../../../crates/rocci-core/src/config.rs
    title: rocci.toml app/window/http/assets fields; no entry
    author: process:git
    last_modified: 2026-08-20
  - id: island-svc
    resource: ../../../crates/rocci-rocdown/src/service.rs
    title: Island service allows one @context / @init module
    author: process:git
    last_modified: 2026-08-24
  - id: apps-toml
    resource: ../../../examples/rocci/apps.toml
    title: Site catalog of published example apps
    author: process:git
    last_modified: 2026-08-29
  - id: staging-tree
    resource: ../../../examples/rocci/staging-tree.md
    title: Discovery is apps.toml, not every examples directory
    author: process:git
    last_modified: 2026-08-23
  - id: docs-catalog
    resource: ../../../crates/rocci-docs/src/catalog.rs
    title: Catalog AppEntry.entry is a staging path, not a run contract
    author: process:git
    last_modified: 2026-08-29
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-file stateful standalone app
    author: process:git
    last_modified: 2026-08-25
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Standalone view with no @context / @init
    author: process:git
    last_modified: 2026-08-25
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Flat sibling handler plus UI modules
    author: process:git
    last_modified: 2026-08-23
  - id: multi-page
    resource: ../../../examples/rocci/standalone/multi-page-streams/README.md
    title: One app, several route modules, no @init
    author: process:git
    last_modified: 2026-08-22
  - id: blocks-toml
    resource: ../../../examples/rocci/standalone/blocks/rocci.toml
    title: Nested standalone app-root marker
    author: process:git
    last_modified: 2026-08-22
  - id: apps-index
    resource: ../../../docs/applications/index.rocdown
    title: Standalone versus custom depth
    author: process:git
    last_modified: 2026-08-22
  - id: standalone-doc
    resource: ../../../docs/applications/standalone.rocdown
    title: Published standalone authoring contract
    author: process:git
    last_modified: 2026-08-22
  - id: custom-doc
    resource: ../../../docs/applications/custom.rocdown
    title: Directory run for authored main.roc
    author: process:git
    last_modified: 2026-08-24
  - id: tooling-doc
    resource: ../../../docs/applications/tooling.rocdown
    title: Workflow page that shows rocci run on a standalone directory
    author: process:git
    last_modified: 2026-08-25
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: Public rocci run entry wording
    author: process:git
    last_modified: 2026-08-30
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Glossary definition of a standalone app
    author: process:git
    last_modified: 2026-08-25
  - id: root-readme
    resource: ../../../README.md
    title: File run for standalone, directory run for custom
    author: human:nils
    last_modified: 2026-08-30
  - id: cli-plan
    resource: ../../plans/shared/cli-entry-points.md
    title: Three-CLI product split; rocci run owns applications
    author: process:cursor
    last_modified: 2026-08-24
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable state is server-owned and process-lifetime
    author: human:nils
    last_modified: 2026-08-16
  - id: blocks-case
    resource: ../../case-studies/standalone-blocks-authoring.md
    title: Nested backend/ui discovery via app-root rocci.toml
    author: process:cursor
    last_modified: 2026-08-24
  - id: live-research
    resource: ./path-addressed-live-streams.md
    title: Multi-module route merge is one generated app
    author: process:cursor
    last_modified: 2026-08-24
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Authoring locations for pages versus backend/ui
    author: process:git
    last_modified: 2026-08-25
  - id: entry-plan
    resource: ../../plans/rocci/application-structure-and-entry.md
    title: Implementation plan for directory-as-app standalone entry
    author: process:cursor
    last_modified: 2026-08-30
---

# Rocci application structure and run entry

## Scope and authority

This record investigates how a standalone Rocci program is identified: what
counts as an app, where `@init` lives, whether sibling `.rocci` files are
modules or other apps, and what `rocci run <dir>` should mean.

It is **exploratory**. Shipped behavior below is checked against current code
and docs. The recommendations are not an approved decision and not an
implementation plan.

It does not reopen the three-CLI product split (`rocci` / `rocdown` /
`okmate`).[^cli-plan] It does not change handler syntax, Datastar transport, or
server-owned state.[^server-state]

## Established nouns

The published layering already names two **depths**, not two products:

| Depth | Authoring | Process entry |
| --- | --- | --- |
| Standalone | `@context` / `@init` / `@method:role` | Generated `main.roc` |
| Custom | authored `main.roc` plus `.rocci` modules | That `main.roc` |

Both produce one HTTP origin.[^apps-index][^standalone-doc][^custom-doc]

A **standalone app** is documented as a `.rocci` module with
`@context` / `@init` / `@get:view`.[^inventory] That sentence is true of
[Counter](/examples/rocci/standalone/counter/Counter.rocci) and false of
[Styling](/examples/rocci/standalone/styling/Styling.rocci) (view, no init)
and of [multi-page streams](/examples/rocci/standalone/multi-page-streams/README.md)
(several route modules, no init). The glossary is describing the first
tutorial, not the assembly rule.[^counter][^styling][^multi-page]

`rocci view` / `browse` are a different product of the same files: a
component gallery that ignores HTTP declarations. They must stay distinct
from `run`.[^cli-readme][^tooling-doc]

`examples/rocci/apps.toml` is a **site catalog**: id, path, title, hosting
class, and an `entry` path used by `rocci-docs` staging. Discovery for
rocci.dev is that file, not “every directory under `examples/rocci`.”
[^apps-toml][^staging-tree][^docs-catalog]

`rocci.toml` is **runtime and desktop config** (name, windows, HTTP, assets,
bundle). It has no `entry` field. For nested standalone apps it is also the
walk-up **app-root marker**. A repository-root `rocci.toml` is not an
app.[^config][^blocks-toml][^blocks-case][^run-rs]

## Shipped assembly

### Per-module `@init` is already unique

Template validation rejects a second `@context` or `@init` **in the same
file**, and requires the pair together. A module may have neither.
Handlers that destructure a record require `@context` in that same
module.[^validate][^template-readme]

So “we allow multiple `@init` sections” is not true inside one file. It is
true **across files in one run**, and that is the gap.

### Across files, the CLI picks a primary and forgets the rest

`rocci run File.rocci` walks up from the file to the first non-workspace
`rocci.toml`, or else stays in the file’s parent. It then compiles **every**
`.rocci` under that root (skipping `generated/`, `target/`,
`node_modules/`, dot-dirs) and puts the named file first.[^run-rs]

Generated dispatch:

- merges **routes and live streams** from every module, rejecting duplicate
  method+path pairs;[^dispatch][^live-research]
- calls **only** `Primary.init!()` and types `Context` as
  `Primary.State`;[^driver][^dispatch]
- passes that one `context` into every sibling handler.[^dispatch]

A sibling `@init` still lowers to `init!` on that module. It is never
called. If the sibling also declared a different `State`, Roc typechecking
fails at the shared call site. If the shapes match, the extra `@init` is
dead code. There is no app-level diagnostic either way.

Rocdown’s island service is stricter: more than one module with
`@context` / `@init` is a hard error (“v1 allows one”).[^island-svc]
Standalone `rocci run` has not grown that check.

### Directory run is the custom-app path

`rocci run` on a directory looks for `main.roc`. If it is missing and the
folder contains `.rocci` files, the error **hints** at `rocci run
First.rocci` (sorted by filename) and does not run. Bare `rocci run`
defaults to `main.roc`.[^run-rs][^cli-main]

Custom examples (`notes`, `datastar`, `snake`) therefore accept
`rocci run <dir>`. Standalone examples do not, even when the directory
contains exactly one obvious program.[^custom-doc][^root-readme][^cli-readme]

The workflow page still shows
`rocci run examples/rocci/standalone/counter`. That command fails with the
hint above. The CLI reference is closer: “standalone `.rocci` or
directory/`main.roc`.”[^tooling-doc][^cli-ref]

Custom compile is also **shallower**: it compiles immediate-directory
`.rocci` siblings only. Standalone discovery is recursive once an app root
exists. Those are two different “bag of files” rules.[^run-rs]

### There is no file-level app boundary

A `.rocci` file is a Roc **module** (stem → type name). Nothing in the
language says “this file is an application” versus “this file is a UI
library.” Routes on any sibling join the process. Two complete programs in
one folder become one origin if their paths do not collide; if both declare
`@get:view("/")`, dispatch fails.[^dispatch][^run-rs]

The examples tree already treats **directory** as the isolation unit:
`standalone/counter`, `standalone/live-counter`, `custom/notes`. Flat
siblings inside one of those dirs are modules of one app
(`LiveCounter.rocci` + `LiveCounterUi.rocci`). Nested Blocks uses
`rocci.toml` so `backend/` and `ui/` stay one app.[^live-counter][^blocks-case][^author-skill]

`live_counter_stays_flat_and_does_not_absorb_sibling_apps` encodes the
current boundary: without a local `rocci.toml`, discovery must not walk
into `standalone/counter`.[^run-rs]

## The actual problem

Not “missing `apps.toml`.” Three mismatches:

1. **Identity.** Authors think in apps. The runner thinks in “the file you
   pointed at, plus every `.rocci` nearby.”
2. **Directory run.** The intuitive `rocci run .` / `rocci run counter`
   works only after you write `main.roc` — the *later* depth.
3. **Silent extra inits.** A second `@init` in the tree is not a second
   app and not a composed store. It is unused.

That makes prototyping awkward in both directions: a scratch folder with
`App.rocci` cannot be run as a directory, and a scratch folder with two
toy programs silently tries to become one process.

## Analogies (comparison, not a port)

These are familiar shapes, not proposals to copy APIs.

| Ecosystem | Unit you run | Extra files | Manifest |
| --- | --- | --- | --- |
| Roc | `main.roc` in a directory | Imported modules | None for a single app |
| Cargo | package directory | `src/lib.rs`, extra `[[bin]]` | `Cargo.toml` |
| Go | `package main` directory | Same-package files | `go.mod` at module root |
| Vite | project directory | Imported modules | Optional until config |
| Next | project directory | Routes are files of one app | Optional `next.config` |
| Deno / Python script | A file, then a project | Imports | Later |

The common successful DX is **file-first for the first minutes**, then
**directory-as-app** once there is more than one file. Manifests appear
when identity, windows, or a non-default entry need a name. Two
`func main` in one Go package is an error, not a merge.

Rocci today is file-first for standalone, directory-first for custom, and
merge-siblings in both — closest to “Roc modules” without Roc’s “there is
one `main`.”

## Directions

### A. Keep today’s split (file for standalone, directory for custom)

Pros: no new contract; `rocci run File.rocci` stays the documented
happy path.[^root-readme][^standalone-doc]

Cons: `rocci run <dir>` stays unintuitive for the apps we tell beginners
to write. Extra `@init` stays silent. Putting two toys in one folder
keeps surprising.

Reject as the end state. Keep the file command.

### B. Directory is the app; file is a module (lean recommendation)

Name the unit:

- An **app** is a directory (the file’s parent, or the tree under a
  non-workspace `rocci.toml`).
- A **module** is one `.rocci` (or `.roc`) file in that app.
- An **entry** is the module that owns process `@init` / `@context` when
  present, and that names the generated app.

Rules that match shipped examples:

- **At most one** `@init` / `@context` pair per app. Zero is legal
  (Styling, multi-page streams). A second pair is an error, as island
  serve already does.[^island-svc][^styling][^multi-page]
- Sibling files may contribute `@component`, `@css`, `@fixture`, `@test`,
  and **additional routes** (Admin, Notifications). They must not declare
  their own process state.
- Two complete programs (each with `@init`, or two `@get:view("/")`) in
  one directory is an error that says “split directories,” not a merge.
- `rocci run File.rocci` still works: that file is the entry; siblings
  load as modules of its app directory.
- `rocci run <dir>` works when the entry is **unique**: the single `@init`
  module, else the single module that has routes, else
  `[app].entry` in `rocci.toml`. Ambiguous: print candidates and exit.
- Nested `backend/` + `ui/` keeps using app-root `rocci.toml` as the
  boundary, optionally with `entry = "backend/Blocks.rocci"`.[^blocks-case]

This is convention plus diagnostics, not a new file type.

### C. Require `rocci.toml` for every app

Pros: explicit root; `rocci run .` is unambiguous; matches Blocks and
custom desktop config.[^blocks-toml][^config]

Cons: a one-file Counter would need a manifest before it runs. That is
the opposite of prototype DX. Reject as a **requirement**. Allow
`rocci.toml` to *name* the entry when convention is not enough.

### D. Reuse `apps.toml` as the run manifest

Pros: the examples repo already writes `entry = "Counter.rocci"` per
app.[^apps-toml][^docs-catalog]

Cons: that file is a **many-app site inventory** (hosting class, audience,
`site = false`). Teaching every prototype to have an `apps.toml` conflates
“what is this directory” with “what do we publish on rocci.dev.”
`rocci run` would have to invent catalog lookup outside `examples/`.
The staging rule is explicitly “not every directory.”[^staging-tree]

Reject as the authoring contract. A later **workspace convenience**
(`rocci run --app counter` from the repo catalog) can stay an examples-only
tool.

### E. Multiple `@init` compose into one `State`

Each module owns a slice; generated `init!` builds `{ counter: …, notes: … }`.

Pros: sounds like “mini-apps” in one process.

Cons: fights one process-lifetime `State`, generated handler records, and
the closed “context is the entry module’s type” story.[^server-state][^dispatch]
Sibling handlers already receive **one** context. Composition belongs in
that record (`{ db, cache }`), authored in **one** `@init`, not in
N process boots.

Reject for v1. Revisit only if a real app cannot express shared resources
in one record.

### F. Each `.rocci` file is an isolated process

`rocci run <dir>` would be undefined, or would start N servers.

Pros: no merge surprises.

Cons: deletes live-counter, Blocks, and multi-page streams — the intended
composition. Routes on Notifications exist to join Dashboard’s
origin.[^multi-page][^live-counter][^live-research]

Reject.

### G. Interactive picker only

`rocci run <dir>` lists files and waits.

Pros: fine in a TTY once.

Cons: scripts, `--no-window`, editor preview, and CI need a
deterministic entry. A picker can sit **on top of** B when several
candidates exist and stdin is a TTY. It must not be the only rule.

### H. New manifest species (`rocci-app.toml`, `[package]`, etc.)

A fourth file next to `rocci.toml` and `apps.toml`.

Pros: clean name.

Cons: authors already meet `rocci.toml` for windows and nested roots.
A second manifest for “which file is main” is cargo-cult until B and an
optional `entry` field fail.

Defer. Prefer one optional field on the file that already marks an app
root.

## Lean recommendation

**Directory is the app. File is a module. At most one process `@init`, or
none. `rocci run <dir>` when unique; `rocci run File.rocci` always.
`rocci.toml` may name `entry`. Do not reuse `apps.toml`.**

Prototype path that should work without ceremony:

```text
# scratch file, no init
rocci run App.rocci

# same folder, now a directory app
rocci run .

# later, only if windows / nested layout / ambiguous entry
# rocci.toml: [app] entry = "backend/App.rocci"
rocci run .
```

One directory, one origin. Two toys → two directories (how
`examples/rocci/standalone/` is already laid out). Shared UI stays extra
modules, not extra inits.

This matches Roc’s “one main, many modules,” Go’s one `package main` per
directory, and the island-service “v1 allows one” check — without forcing
a manifest on Counter.

## Constraints that should not move

- One HTTP process, one durable `State`, produced by at most one
  `@init`.[^server-state]
- Components stay pure; I/O stays in `@init` and handlers.[^template-readme]
- `rocci run File.rocci` remains a first-class command.
- `apps.toml` remains the published-example catalog, not local app
  identity.[^staging-tree]
- `rocci` does not absorb Rocdown or OKF run.[^cli-plan]
- `browse` / `view` stay the component gallery, not a second app runner.

## Decision gates

Human review before:

- requiring `rocci.toml` (or any manifest) for a one-file app;
- teaching `apps.toml` as the way to run an app;
- composing multiple `@init` into generated `State`;
- treating each `.rocci` file as its own server;
- making `rocci run` without a path mean something other than today’s
  `main.roc` default without a migration note.

## Current disposition

No code change in this revision. Implementation of lean direction B is
[directory-as-app standalone entry](/plans/rocci/application-structure-and-entry.md):
app-level unique init, `rocci run <dir>` when the entry is unique,
aligned `build --release <dir>`, optional `[app].entry`, then docs.
The workflow page’s Counter directory command stays a doc bug until that
plan’s public-contract phase.[^tooling-doc][^run-rs][^entry-plan]

[^run-rs]: Walk-up app root, recursive standalone discovery, directory requires `main.roc` and hints the first `.rocci`.
[^driver]: `main_roc` passes `primary.init` only.
[^dispatch]: Sibling routes merge; generated `Context` is `{Primary}.State`; every arm calls `handler!(context, request)`.
[^cli-main]: `Run.file` defaults to `main.roc`.
[^cli-readme]: Documented examples run a `.rocci` file or a custom app directory.
[^validate]: Duplicate `@init` / `@context` are per-module errors; the pair is required together.
[^template-readme]: `@context` / `@init` / routes are the standalone HTTP surface for `rocci run File.rocci`.
[^config]: `AppConfig` is name, identifier, version.
[^island-svc]: Multiple live-page `@context` / `@init` modules are refused.
[^apps-toml]: `[[app]]` rows with `entry` for staging and site hosting.
[^staging-tree]: Catalog discovery, not a directory walk of `examples/rocci`.
[^docs-catalog]: `AppEntry.entry` is a catalog field for `rocci-docs`.
[^counter]: Counter is one file with `@context`, `@init`, and `@get:view("/")`.
[^styling]: Styling serves `@get:view("/")` with no process state.
[^live-counter]: Live-counter imports a sibling UI module from the same directory.
[^multi-page]: Run `Dashboard.rocci`; siblings provide `/admin` and shared streams.
[^blocks-toml]: App-root `rocci.toml` for Blocks windows and identity.
[^apps-index]: Standalone versus custom as depths of one layer.
[^standalone-doc]: Generated dispatch; authors do not write `main`.
[^custom-doc]: `rocci run` on a directory compiles siblings and starts authored Roc.
[^tooling-doc]: Shows `rocci run examples/rocci/standalone/counter`.
[^cli-ref]: Entry is a standalone `.rocci` or a directory / `main.roc`.
[^inventory]: Glossary row defines a standalone app as one module with context, init, and view.
[^root-readme]: File command for standalone; directory command for custom `main.roc`.
[^cli-plan]: Approved three-CLI split; `rocci run` owns applications.
[^server-state]: Durable state is server-owned and process-lifetime.
[^blocks-case]: Nested `backend/` + `ui/` behind app-root `rocci.toml`.
[^live-research]: Multi-module standalone assembly is one generated app.
[^author-skill]: Pages at app root or `backend/` + `ui/`; repository-root `rocci.toml` is not an app.
[^entry-plan]: Phased CLI resolver, unique-init diagnostic, optional `[app].entry`; not `apps.toml`.
