---
type: Implementation Plan
title: Rust-templated OKF viewer with Datastar, then a Rocci reference
description: Make rocci-okf a single Rust+Askama+Datastar application for short-term usability and public distribution, and freeze that viewer as the behavioral reference for a later Roc+Rocci port that does not own HTML in Rust.
tags: [domain/okf, domain/rocci-okf, domain/rocci, integration/datastar, concern/architecture, concern/rendering, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-26
authority: exploratory
owners: [human:nils]
sources:
  - id: okmate
    resource: okmate.md
    title: Okmate — extractable Rust OKF mate
    author: process:cursor
    last_modified: 2026-08-26
  - id: research
    resource: ../../research/okf/okf-viewer-rust-vs-rocci.md
    title: OKF viewer Rust HTML versus finished Rocci shell
    author: process:cursor
    last_modified: 2026-08-25
  - id: host-surfaces
    resource: okf-viewer-host-surfaces.md
    title: Short-term OKF viewer host surfaces
    author: process:cursor
    last_modified: 2026-08-25
  - id: settings-ux
    resource: settings-ux.md
    title: Settings UX for knowledge roots
    author: process:cursor
    last_modified: 2026-08-25
  - id: multi-roots
    resource: multi-knowledge-roots.md
    title: Multiple knowledge roots for rocci-okf
    author: process:cursor
    last_modified: 2026-08-25
  - id: site-lane
    resource: ../site/okf-viewer-site-lane.md
    title: Mount the OKF knowledge viewer on rocci.dev
    author: process:cursor
    last_modified: 2026-08-25
  - id: compile-follow-ons
    resource: okf-compile-render-follow-ons.md
    title: Deferred OKF compile and render follow-ons
    author: process:cursor
    last_modified: 2026-08-19
  - id: compile-research
    resource: ../../research/okf/okf-compile-render-cost.md
    title: OKF preview compile and render cost
    author: process:cursor
    last_modified: 2026-08-25
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable application state stays server-owned
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: publication
    resource: ../../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: okf-app
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF review and query application
    author: process:cursor
    last_modified: 2026-08-17
  - id: tools-research
    resource: ../../research/okf/okf-tools-and-workflows.md
    title: Agent-native OKF operations and review workflows
    author: process:codex
    last_modified: 2026-08-17
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Dual Rust html_page_for and Rocci apply review site
    author: process:git
    last_modified: 2026-08-25
  - id: settings-rs
    resource: ../../../crates/rocci-okf/src/settings.rs
    title: Settings article HTML and POST actions
    author: process:git
    last_modified: 2026-08-25
  - id: okf-main
    resource: ../../../crates/rocci-okf/src/main.rs
    title: extra_http wraps settings with html_page_at
    author: process:git
    last_modified: 2026-08-25
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: view host auto, Rust shell when roc is missing
    author: process:git
    last_modified: 2026-08-25
  - id: datastar-crate
    resource: ../../../crates/rocci-datastar/README.md
    title: Rust Datastar spec, SSE framing, signals, and assets
    author: process:git
    last_modified: 2026-08-24
  - id: askama
    resource: https://crates.io/crates/askama
    title: Askama type-safe Rust HTML templates
    author: organization:crates-io
  - id: agents
    resource: ../../../AGENTS.md
    title: Layer owners and no Rust interpretation of Rocci
    author: process:git
    last_modified: 2026-08-25
  - id: ui-goto
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared chrome script that swaps #okf-main
    author: process:git
    last_modified: 2026-08-24
  - id: deps-check
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace package class edges
    author: process:git
    last_modified: 2026-08-18
---

# Rust-templated OKF viewer with Datastar, then a Rocci reference

## Goal

Ship `rocci-okf` as one Rust application: Askama owns every page of HTML,
`rocci-datastar` owns settings and later widget interactivity, and default
`view` / `build` never compile Rocci. That app is the short-term product
(usable `Knowledge.app`, no Roc install, a static tree that rocci.dev can
copy). The same routes, stable IDs, POST/SSE shapes, and view records are
the reference a later Roc+Rocci viewer must match.[^research][^okf-readme][^site-lane]

## Two horizons

| Horizon | What ships | Who authors HTML | Who speaks Datastar | Roc required |
| --- | --- | --- | --- | --- |
| **A — this plan** | Distributable Rust viewer | Askama in `rocci-okf` | `extra_http` + `rocci-datastar` | Never for default `view` / `build` |
| **B — later plan** | Same viewer, Rocci-authored | `.rocci` + Roc handlers | `@method:role` on the same wire | Applicator or Roc host for the *app*, not for `check` |

Horizon A is the product inversion the compile/render follow-ons named and
refused as a cache trick: skip-Roc becomes the *default look*.[^compile-follow-ons][^catalog-shell]
Horizon B is not “finish `OkfTheme` apply.” It is a real Rocci application
that reimplements Horizon A’s contract. Do not start B until A’s contract
is frozen and A is the only shipped shell.[^research][^okf-app]

“Without Rust” in Horizon B means **the viewer UI**. It does not mean
rewriting `okf`, `rocci-desktop`, the CLI, git, or `okf.toml` in Roc. Those
stay Rust unless a third horizon (Roc HTTP platform + engine FFI) is
planned separately.[^static-okf][^agents]

## Out of bound

Porting the portable `okf` engine to Roc. Askama, Maud, or any Rust HTML
DSL in `rocci-rocdown` or `rocci-template`. Interpreting `.rocci` in Rust
to skip a compiler. Live SSE for the settings registry. Review
approve / request-changes / comment, in-UI agent jobs, or a hosted public
origin. Shipping a prebuilt applicator or wasm apply-to-disk. Amending
[local-first publication](/decisions/local-knowledge-publication.md)
(that remains [site-lane](../site/okf-viewer-site-lane.md) Phase 0).
Replacing [settings UX](settings-ux.md) folder-pick and card copy.
Replacing `goto.js` / Cmd-K. Minting an approved Decision in this
authoring. Implementing Horizon B.

## Constraints that do not move

- Knowledge records stay inert Markdown. `okf` stays UI-neutral. `check` /
  `inspect` / `search` do not require Roc or Askama.[^static-okf][^agents]
- Durable registry is `okf.toml`. Mutations validate, write, re-read, and
  render a stable-id region. Tokens are never echoed.[^multi-roots][^settings-ux][^server-state]
- `/__rocci_okf/settings` stays loopback-only.[^okf-readme]
- Rocdown keeps the [catalog-shell](/decisions/rust-catalog-rocci-shell.md)
  split. This exception is `rocci-okf` only.[^catalog-shell]
- Askama lives only in `rocci-okf`. Do not grow a second docs-template
  language for documentation sites.[^catalog-shell][^agents]
- `rocci-okf` may depend on `rocci-datastar` (base-rocci). It must not
  depend on Rocdown.[^deps-check]
- Horizon B must match Horizon A’s routes, IDs, and POST/SSE map. It must
  not invent a second information architecture.

## Why this, given the staged research

The viewer is already a Rust host that emits documents. Settings, review
tables, and badges change with the same classifiers that own the data.
Rocci apply is a second compiler for chrome the host already authored in
`format!`. The hybrid trains the team to edit the wrong file.[^research][^presentation][^settings-rs]

Unbound, Askama-class templates plus Datastar from Rust are the better
*application* design for today’s binary. Finished Rocci apply remains the
better design for Rocdown. Queue operations and agent jobs are a different
surface; they need Datastar either way, and Rust already has the
protocol.[^research][^datastar-crate][^server-state]

This plan takes that unbound pick as the **short-term product**, and treats
the resulting app as the **spec** for a later Rocci port — instead of
investing in static-only `OkfTheme` now and discovering the interaction
contract twice.[^research][^host-surfaces]

## Surfaces

| Surface | Horizon A owner | Public `build` / rocci.dev | Horizon B |
| --- | --- | --- | --- |
| Concept / collection / dashboard HTML | Askama + `article_html` from `okf` | Yes (snapshot) | Same view records, Rocci chrome |
| Settings, session, folder pick | Askama + `extra_http` + desktop IPC | Omit or link to local app | Same POSTs, Rocci `@post` |
| Review queue display | Askama, `#okf-queue` | Yes (read-only) | Same region id |
| Queue decisions / agent jobs | Do not start | No (`file_server` cannot) | Later, same wire |
| Cmd-K / `pages.json` | `goto.js` + host | Yes, prefixed | Unchanged |
| `check` / `inspect` / `search` | Rust CLI | N/A | Unchanged |

## Viewer contract (freeze in Phase 6)

These names are the reference Horizon B must keep. Changing one later is a
breaking change to the port.[^ui-goto][^settings-rs][^research]

| Kind | Horizon A value |
| --- | --- |
| Shell root | `.rd-shell` / `.rd-document` (keep current CSS) |
| Sidebar | `#okf-nav` |
| Article column | `#okf-main` |
| Outline | `#okf-toc` |
| Settings region | `#okf-settings` |
| Review region | `#okf-queue` |
| Settings GET | `/settings/` |
| Settings POST | `/__rocci_okf/settings` (loopback) |
| Datastar branch | `Datastar-Request: true` → fragment or one `patch-elements`; else full article / 204 |
| Registry interactivity | One-shot morph only; no live SSE |
| Trusted splice | Markdown `article_html` is pre-escaped by the engine; templates mark it safe once |
| Machine API | CLI / inspect / search remain the write and query path; the UI is another client |

## Consequences

### Product

`Rocci Knowledge.app` and agent `view` become one look on every machine.
First open is parse + write, not cold `roc build`. That is the usability
win and the reason a public desktop binary is plausible.[^okf-readme][^compile-research]

Readers do not perceive Askama versus Rocci. They perceive whether the app
opened and whether the shell matches last time. Deleting the hybrid is the
UX change; the template dialect is not.[^research]

rocci.dev `/knowledge/` stays a **prefixed static copy** of `rocci-okf
build`. It does not gain settings POST, queue writes, or agent jobs.
Those need local `view` or a separately reviewed live origin. Shipping the
app and publishing the snapshot are different deploys. Local-first
publication still blocks the snapshot until site-lane Phase 0
amends it.[^site-lane][^publication]

### Architecture and decisions

This is a **local exception** to catalog-shell for `rocci-okf` chrome only.
Rocdown’s `RocdownTheme` path does not move. When Phase 1 ships, draft a
Decision that records the exception; do not treat this plan as that
Decision.[^catalog-shell]

Compile/render follow-on Phase 2 (skip-Roc as opt-in, not default) is
**inverted**. Wasm apply-to-disk stops being necessary for this
product.[^compile-follow-ons]

[Host surfaces](okf-viewer-host-surfaces.md) is absorbed: its ownership
fixes land as Phase 1. Do not run that plan separately.[^host-surfaces]

[Settings UX](settings-ux.md) still owns `rfd` and card copy. Its “no
Datastar” bound applies to *that* plan’s phases. This plan later wraps
the same article in a one-shot morph; live SSE for the registry stays
out.[^settings-ux]

[rocci-okf-app](rocci-okf-app.md) “built with Rocci” becomes Horizon B.
Horizon A delivers the local browse/review shell that plan’s application
phase wanted, in Rust. Review decisions and authenticated query stay on
that plan.[^okf-app]

### Authoring

Askama is a second markup language in the repo, scoped to one crate.
`rustc` is the feedback loop for a settings field or queue column. That
matches how this viewer actually changes.[^askama][^research]

The cost is **two Datastar authoring paths** until Horizon B: Rocci
`@method:role` in example apps, and `extra_http` here. The wire must stay
the same (`datastar.js`, morph by id, optional SSE) so the port is a
handler rewrite, not a new interaction model.[^datastar-crate][^research]

Dogfood of `OkfTheme` as a static theme stops. Counter, RocdownTheme, and
Horizon B remain the language-product surfaces. Do not keep `--host
native` as a second supported look; an escape hatch that renders
different pixels recreates the hybrid.[^research][^presentation]

### Distribution

| Artifact | Horizon A | Still needs Roc / applicator |
| --- | --- | --- |
| `rocci-okf` CLI `view` / `build` | One binary | No |
| `Rocci Knowledge.app` | Bundled host + this writer | No |
| `dist/knowledge` / `/knowledge/` | Static HTML + `pages.json` | No |
| Live settings / future writes | Same binary, loopback or hosted origin | No |
| Horizon B viewer process | Later | Yes (compile or shipped applicator) |

Default `rocci-okf` can drop `rocci-template` / `rocci-roc-host` from the
always-on path once apply is gone. That shrinks the desktop archive and
removes Wasmtime from the common case.[^okf-readme][^compile-follow-ons]

### Horizon B (detailed, not implemented here)

A later plan reimplements this viewer as a Rocci application:

1. Rust still owns `okf` load, validation, search, `okf.toml`, git cache,
   desktop IPC, and the process that binds HTTP.
2. Rocci owns chrome and widgets. Handlers declare the same paths and
   roles. View records are the contract Askama already consumed.
3. Golden fixtures from Phase 6 (settings fragment, review region, shell
   landmarks) are the acceptance tests.
4. `file_server` snapshots can still be produced by a Rocci apply *or* by
   keeping a Rust static writer; pick one in that plan. Do not revive a
   dual shell.

True “no Rust in the process” would require a Roc HTTP platform and
either FFI into `okf` or a Roc engine. That is a third horizon. Naming it
now prevents Horizon B from being scoped as “delete the CLI.”[^static-okf][^okf-app]

Risk if B never starts: Askama is the permanent owner. That is an
acceptable product if A is honest. Risk if B starts early: two UIs.
Gate: B does not start until A is the only shipped shell and the contract
record exists.[^research]

### What we give up

- Language dogfood of a knowledge *theme* (not of a knowledge *app*).
- Shared chrome evolution with `RocdownTheme` unless CSS classes stay
  aligned by hand (they already are `.rd-*`).
- The compile/render investment in apply-writes as the supported preview
  look.

### What we do not give up

- Portable engine, inert records, CLI for agents.[^tools-research]
- Server-owned state and one-shot-then-live Datastar rules.[^server-state]
- A path back to Rocci that is a port, not a guess.

## Relationship to other plans

| Record | After this plan |
| --- | --- |
| [Host surfaces](okf-viewer-host-surfaces.md) | Superseded; phases absorbed into Phase 1 |
| [Settings UX](settings-ux.md) | Still implement folder-pick and cards; Datastar morph is this plan |
| [Multi-roots](multi-knowledge-roots.md) | Unchanged domain; UI owner becomes Askama |
| [Compile/render follow-ons](okf-compile-render-follow-ons.md) | Phase 2 inverted; Phase 3 (wasm apply) not needed for this product |
| [Site knowledge lane](../site/okf-viewer-site-lane.md) | Still copies `build`; chrome comes from Askama; live ops still a different deploy |
| [rocci-okf-app](rocci-okf-app.md) | Engine/CLI phases unchanged; “built with Rocci” is Horizon B |

## Phases (Horizon A)

### Phase 1 — One Rust writer is the default product

**Bound:** Default `view` and `build` write only the existing Rust shell
(`html_page_for` / extra_http). Do not compile `Settings.rocci` or
`ReviewQueue.rocci` on that path. `generate_okf_page_data` does not emit a
settings apply page. Preview GET `/settings/` uses bundle nav, not
`html_page_at` + `render_nav_tree(None)`. `--host native` and
`ROCCI_REQUIRE_ROC=1` remain an opt-in compare path, not the documented
look. README states one shell.[^presentation][^okf-main][^host-surfaces][^okf-readme]

**Out of bound:** Askama. Datastar. Deleting `--host native`. Changing
document CSS.

**Tests:** `view` / `build` fixtures have no apply `settings/index.html`;
extra_http GET includes a known collection route; with `roc` on PATH,
default `view` does not invoke `roc build`.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `presentation.rs`, `runtime.rs`, `main.rs`, crate README.

**When this ships:** Draft a Decision that `rocci-okf` chrome is a
catalog-shell exception. Do not mark it approved in the same change.

### Phase 2 — Askama owns the HTML

**Bound:** Pin Askama on the workspace and depend on it from `rocci-okf`
only.[^askama] Replace `html_page_for`, nav, toc, outline, concept-meta,
home, review, and settings article `format!` builders with templates under
`crates/rocci-okf/templates/html/` (or `askama/`). Rust keeps view structs
and handlers. Trusted `article_html` is the one unescaped splice. Keep
current CSS classes and the contract IDs. Move unused `.rocci` files to
`templates/sketches/` so they are not `include_str!` into a hash.

**Out of bound:** Datastar. Changing settings POST semantics. Maud unless
Askama cannot express a page without fighting the type system (record the
reason in the commit if that happens).

**Tests:** Marker tests for `#okf-nav`, `#okf-main`, `#okf-settings`,
review table container; settings still redacts tokens; `build` still
writes `pages.json`.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** new Askama files; `presentation.rs` and `settings.rs` become
thin.

### Phase 3 — Cut apply from the default binary path

**Bound:** Default `view` / `build` do not call `compile_okf_templates` or
native/wasm apply. Optional `--host native` may remain behind a feature
or go away in this phase if tests allow. Drop `rocci-template` /
`rocci-roc-host` from the default dependency set if nothing else in the
crate needs them. README and `--help` no longer describe host-auto Rocci
as the product.

**Out of bound:** Deleting `rocci-roc-host` from the workspace. Changing
Rocdown apply.

**Tests:** Default test suite never spawns `roc`. Feature-gated native
tests, if any remain, stay `#[ignore]` or behind `ROCCI_REQUIRE_ROC`.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `Cargo.toml`, `presentation.rs`, `main.rs`.

### Phase 4 — Datastar one-shot on settings

**Bound:** Depend on `rocci-datastar`. Stage `datastar.js` next to
`goto.js` (reuse the CLI asset helper or the crate’s asset API). Shell
templates include the script. Settings forms use `@post` to
`/__rocci_okf/settings`. extra_http branches on `Datastar-Request`: return
the `#okf-settings` node (HTML morph or one `patch-elements` event).
Ordinary form POST / `curl` still get a full article. No live SSE. Tokens
still redacted. Loopback unchanged.[^datastar-crate][^settings-rs][^server-state]

**Out of bound:** Live GET for the registry. Replacing `goto.js`. Queue
mutations.

**Tests:** Datastar-Request POST returns a fragment that contains
`id="okf-settings"` and no `<html>`; ordinary POST still returns a full
document or a wrapped article as today; token values do not appear.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `settings.rs`, `main.rs`, Askama shell, asset staging.

### Phase 5 — Review region is morph-ready

**Bound:** Review page template has a stable `#okf-queue`. Extract a
fragment renderer used by the full page (and later by a POST). Do not
implement approve, comment, or agent query. Optional: one-shot GET
fragment for a filter query if it is already a server-side table
parameter.

**Out of bound:** Revision-bound decisions. Live SSE. New write path that
the CLI cannot see.[^okf-app][^tools-research]

**Tests:** Built `/review/` contains `#okf-queue`; fragment helper returns
that node.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** review Askama + `presentation.rs`.

### Phase 6 — Freeze the reference and name the public split

**Bound:** Crate README section: document vs host vs future operations;
Askama + Datastar; sketches are not compiled; point at this plan and the
research. Add golden fixtures (or snapshot tests) for shell landmarks,
settings fragment, and `#okf-queue` that Horizon B must match. Document
that `build` is the public snapshot, `view` is the live app, and
site-lane Phase 0 still gates rocci.dev. Knowledge.app / `rocci-ops
bundle okf` notes: no Roc. No public product-docs change unless the
README sentence is already the contract.

**Out of bound:** Implementing Horizon B. Amending local-first
publication. A hosted `rocci-okf` origin.

**Exit:** README names the three surfaces and the two deploys; fixtures
exist; `cargo test -p rocci-okf`.

**Owner:** `crates/rocci-okf/README.md`, fixtures, this plan’s Status
line.

## Horizon B (follow-on, do not start)

Write a new plan when Phase 6 has exited. That plan’s first phase is
“Rocci app matches Phase 6 fixtures,” not “design a new knowledge UI.”

Minimum B shape:

- `okf` and CLI unchanged.
- A Rocci module (theme + settings + queue widgets + `@post` / later
  `@get:live` only where a job runs) consumes the same view structs
  serialized across the apply/host boundary.
- `rocci-okf` becomes a thin host: load bundle, run extra_http by
  forwarding to the Rocci app or serving its apply output for documents
  and its handlers for mutations.
- Delete Askama only after fixtures pass on the Rocci path. One shell
  again.

## Status

Superseded as the implementation vehicle by [okmate](okmate.md). Do not
start this in-place `rocci-okf` rewrite. Pair:
[viewer rust vs rocci](/research/okf/okf-viewer-rust-vs-rocci.md).
Absorbs [host surfaces](okf-viewer-host-surfaces.md) only if someone
still patches `rocci-okf` before okmate exists. Settings copy remains
[settings UX](settings-ux.md). Public snapshot remains
[site lane](../site/okf-viewer-site-lane.md).[^okmate]

[^okmate]: Extractable root crate with Askama, Axum, and the official Datastar SDK; depends only on `okf`.
[^research]: Unbound pick is Rust HTML + Datastar from Rust; operations need the protocol not a static theme; public lane is a snapshot; do not keep the hybrid.
[^host-surfaces]: Settings extra_http, unused Rocci unplugged, no Askama/Datastar in that smaller plan.
[^settings-ux]: Live markup is `settings.rs`; one-shot POST; no live SSE for the registry; `rfd` folder pick.
[^multi-roots]: Registry POSTs, token redaction, `/settings/` chrome.
[^site-lane]: Foreign static app under `/knowledge/`; not a Rocdown mount; live ops need a host.
[^compile-follow-ons]: Skip-Roc as default was a product inversion; wasm apply is a gated follow-on.
[^compile-research]: First-open cost is Roc compile when the apply path runs.
[^catalog-shell]: Approved for Rocdown: Rust catalog, Rocci chrome, no second docs-template language.
[^static-okf]: Inert Markdown; knowledge builds do not execute Roc or Rocci content.
[^server-state]: Write, re-read, render a stable-id region; browser is not the domain store.
[^publication]: Public deploy of generated knowledge is a separate review.
[^okf-app]: Review decisions and query are a later application; CLI stays the agent contract.
[^tools-research]: Viewer is a projection of CLI/schema operations.
[^presentation]: Dual writer, Rust nav/review/home, generate still bakes settings.
[^settings-rs]: GET/POST return a Rust article; `#okf-settings` already exists.
[^okf-main]: extra_http wraps settings with `html_page_at`.
[^okf-readme]: Host auto uses Rocci when `roc` exists; Knowledge.app is a bundled preview host.
[^datastar-crate]: Rust SSE builders, signal extractors, `datastar.js` staging.
[^askama]: HTML-file templates with typed Rust context structs.
[^agents]: Do not interpret Rocci in Rust to skip a theme; catalog tests must not require Roc.
[^ui-goto]: `#okf-main` / `#okf-toc` swap; not Datastar.
[^deps-check]: okf-app may depend on base-rocci; must not depend on Rocdown.
