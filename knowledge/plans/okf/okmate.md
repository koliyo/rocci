---
type: Implementation Plan
title: Okmate — extractable Rust OKF mate
description: Create okmate/ as a new workspace crate that rewrites the knowledge viewer in Askama 0.16, Axum, and the official Datastar Rust SDK, depending only on the portable okf engine so the directory can become its own repository.
tags: [domain/okf, domain/okmate, integration/datastar, concern/architecture, concern/rendering, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-26
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/okf/okf-viewer-rust-vs-rocci.md
    title: OKF viewer Rust HTML versus finished Rocci shell
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-datastar
    resource: okf-viewer-rust-datastar.md
    title: In-place rocci-okf Askama rewrite (superseded as vehicle)
    author: process:cursor
    last_modified: 2026-08-26
  - id: okf-app
    resource: rocci-okf-app.md
    title: Standalone Rocci OKF review and query application
    author: process:cursor
    last_modified: 2026-08-26
  - id: settings-ux
    resource: settings-ux.md
    title: Settings UX for knowledge roots
    author: process:cursor
    last_modified: 2026-08-26
  - id: multi-roots
    resource: multi-knowledge-roots.md
    title: Multiple knowledge roots for rocci-okf
    author: process:cursor
    last_modified: 2026-08-25
  - id: site-lane
    resource: ../site/okf-viewer-site-lane.md
    title: Mount the OKF knowledge viewer on rocci.dev
    author: process:cursor
    last_modified: 2026-08-26
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
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: publication
    resource: ../../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: okf-readme
    resource: ../../../crates/okf/README.md
    title: Portable UI-neutral OKF engine
    author: process:git
    last_modified: 2026-08-25
  - id: okf-lib
    resource: ../../../crates/okf/src/lib.rs
    title: okf check, load, inspect, search, build
    author: process:git
    last_modified: 2026-08-25
  - id: rocci-okf-main
    resource: ../../../crates/rocci-okf/src/main.rs
    title: Current rocci-okf CLI surface
    author: process:git
    last_modified: 2026-08-25
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Dual Rust and Rocci review HTML
    author: process:git
    last_modified: 2026-08-25
  - id: settings-rs
    resource: ../../../crates/rocci-okf/src/settings.rs
    title: Settings article and POST actions
    author: process:git
    last_modified: 2026-08-25
  - id: askama
    resource: https://crates.io/crates/askama
    title: Askama 0.16 type-safe compiled HTML templates
    author: organization:crates-io
  - id: datastar-sdk
    resource: https://crates.io/crates/datastar
    title: Official Datastar Rust SDK 0.4 with Axum
    author: organization:starfederation
  - id: datastar-gh
    resource: https://github.com/starfederation/datastar-rust
    title: Official Datastar Rust SDK repository
    author: organization:starfederation
  - id: deps-check
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace package class edges
    author: process:git
    last_modified: 2026-08-18
  - id: workspace
    resource: ../../../Cargo.toml
    title: Root Cargo workspace members
    author: process:git
    last_modified: 2026-08-25
  - id: agents
    resource: ../../../AGENTS.md
    title: Layer owners and knowledge CLI
    author: process:git
    last_modified: 2026-08-25
---

# Okmate — extractable Rust OKF mate

## Goal

Add a new repository-root crate `okmate/` that is the knowledge application:
Askama 0.16 owns HTML, Axum owns HTTP, the official Datastar Rust SDK owns
morph and SSE, and the only in-repo Rust dependency is the portable `okf`
engine. The binary is `okmate` (“open knowledge mate”). The directory is
shaped so it can become its own git repository without dragging Rocci
template, desktop, or CLI crates.[^research][^okf-readme][^askama][^datastar-sdk][^workspace]

## Why a new root, not an in-place `rocci-okf` rewrite

[rust+datastar](okf-viewer-rust-datastar.md) rewrote `rocci-okf` in place
and still sat on `rocci-cli` `extra_http`, `rocci-datastar`, and
`rocci-ui` `goto.js`. That is a Rocci preview host with nicer templates,
not a standalone app.[^rust-datastar][^rocci-okf-main]

Okmate is a **greenfield extract**. Port behavior and contracts from
`rocci-okf`; do not move that crate, do not depend on it, and do not keep
`--host native`. `rocci-okf` stays the Rocci knowledge CLI until an
explicit cutover (out of this plan).[^okf-app][^agents]

Catalog-shell continues to bind **Rocdown**. Okmate is a third product
next to Rocci apps and Rocdown sites, not a second docs-template language
inside `rocci-rocdown`.[^catalog-shell][^agents]

## Out of bound

Deleting or renaming `rocci-okf`. Changing the portable `okf` engine
except to consume its existing public API. Depending on any `rocci-*`
crate. Interpreting `.rocci` in Rust. Live SSE for the settings registry.
Review approve / request-changes / comment, in-UI agent jobs, or a hosted
public origin. Amending [local-first publication](/decisions/local-knowledge-publication.md).
Switching CI / `manage-rocci-knowledge` off `rocci-okf`. Creating the
standalone git remote. Horizon B (a later Rocci-authored viewer). Minting
an approved Decision.

## Constraints that do not move

- Knowledge records stay inert Markdown. `okf` stays UI-neutral.
  `check` / `inspect` / `search` do not require a browser, Askama, or
  Roc.[^static-okf][^okf-lib]
- Okmate’s only workspace Rust dependency is `okf`. All other deps are
  crates.io (Askama, Axum, `datastar`, clap, tokio, notify, tao / wry /
  rfd when the window lands).[^okf-readme][^deps-check]
- Durable settings live under `~/.okmate/` (or `OKMATE_CONFIG`). Do not
  write `~/.rocci/` as the long-term path. An optional one-shot import
  from `~/.rocci/okf.toml` is allowed.[^multi-roots][^settings-ux]
- Mutations validate, write, re-read, and render a stable-id region.
  Tokens are never echoed. Settings POST is loopback-only.[^server-state][^settings-rs]
- Server-owned state. No client domain store. One-shot Datastar morph
  first; live SSE only when a later plan has a real job.[^server-state][^datastar-gh]
- Askama 0.16 with `{% extends %}` / `{% include %}` is the HTML owner.
  Maud is not the default. `format!` HTML is not the default.[^askama][^research]
- Official `datastar` 0.4 (`axum` feature) is the protocol crate, not
  `rocci-datastar`.[^datastar-sdk]
- `okf::build` still emits machine artifacts. Okmate adds the HTML
  review tree on top; it does not reimplement catalog JSON.[^okf-lib]

## Product name and paths

| Role | Value |
| --- | --- |
| Crate / binary | `okmate` |
| Human name | Okmate (open knowledge mate) |
| Repo path now | `/okmate` at the Rocci workspace root |
| Config | `~/.okmate/config.toml` (`OKMATE_CONFIG`) |
| Cache / state | `~/.okmate/cache`, `~/.okmate/state` (`OKMATE_CACHE`, `OKMATE_STATE`) |
| Asset prefix | `/__okmate/` |
| Settings POST | `/__okmate/settings` (loopback) |
| macOS bundle (later) | `Okmate.app` |

Do not keep `/__rocci_okf/` or `Rocci Knowledge.app` as this product’s
names. `rocci-okf` may keep those until cutover.[^rocci-okf-main]

## Idiomatic stack

This is a normal Rust hypermedia app, not a Rocci preview adapter.

```
okmate/
  Cargo.toml
  README.md
  assets/
    app.css
    goto.js          # only if Cmd-K stays a small script; prefer Datastar
  templates/
    base.html        # {% extends %} shell, datastar.js, app.css
    nav.html
    toc.html
    page.html        # concept / collection
    home.html
    review.html
    settings.html
    fragments/
      settings.html  # #okmate-settings
      queue.html     # #okmate-queue
      main.html      # #okmate-main (+ toc) for in-app GET
  src/
    main.rs          # clap, process edge (anyhow)
    lib.rs           # modules for tests
    cli.rs
    config.rs
    views/           # Askama context structs only
    http/            # Axum router, extractors, Datastar responses
    preview.rs       # watch, rebuild, bind
    desktop.rs       # tao/wry/rfd; empty until Phase 7
```

**Templating.** Askama 0.16: one context struct per template, inheritance
for the document, includes for nav/toc/meta, a single `|safe` splice for
engine `article_html` (already HTML from `okf`). Tests render templates
to strings and assert landmarks. No 3k-line `format!` file.[^askama][^presentation][^okf-lib]

**HTTP.** Axum 0.8 on tokio. First paint and `build` are full documents.
In-app navigation is `@get` that returns `PatchElements` for
`#okmate-main` and `#okmate-toc` (Datastar, not a column-swap script as
the primary model). Ordinary browsers and `curl` still get full HTML on
plain GET.[^datastar-sdk][^datastar-gh]

**Datastar.** Use the official SDK the way its Axum examples do:
`ReadSignals` / form extractors on POST, `PatchElements` (and later
`PatchSignals` only if a widget needs it), long-lived SSE via
`tokio::sync::watch` or `broadcast` when a job exists. Branch on
`Datastar-Request`: fragment or SSE for the client, full article or 204
for `curl`. Stage a pinned `datastar.js` in `assets/` (commit or
build-time fetch with integrity). Do not generate Roc
`Datastar.roc`.[^datastar-sdk][^server-state]

**Desktop.** When the window lands, `tao` + `wry` + `rfd` live in
`okmate` itself. That is extractable. Do not take `rocci-desktop`.
Folder pick is the same UX [settings UX](settings-ux.md) specified
(`rfd::AsyncFileDialog`, path field without IPC).[^settings-ux]

**CLI.** clap derive, same verbs as today’s knowledge tool where they
are engine-shaped: `check`, `inspect`, `search`, `benchmark`, `build`,
`view`, `roots`, `sync`. Drop `--host` / `ROCCI_REQUIRE_ROC`.
`--profile rocci` remains an **engine** profile, not a UI
stack.[^okf-lib][^rocci-okf-main]

## Viewer contract (okmate IDs)

New product, new IDs. Do not pretend this is a drop-in `#okf-*` clone.

| Kind | Value |
| --- | --- |
| Shell | `.okmate-shell` (CSS may reuse visual tokens from the current dark review theme) |
| Sidebar | `#okmate-nav` |
| Article | `#okmate-main` |
| Outline | `#okmate-toc` |
| Settings | `#okmate-settings` |
| Review | `#okmate-queue` |
| Settings GET | `/settings/` |
| Settings POST | `/__okmate/settings` |
| Page GET (live) | same published routes as `okf` (`/`, `/review/`, `/{id}/`) |
| Datastar | `Datastar-Request: true` → patch; else full document / 204 |

Machine JSON (`catalog.json`, inspect, search) stays the `okf` contract
so agents do not learn a second schema.[^okf-lib][^okf-app]

## Workspace and extract

While inside Rocci:

1. Add `"okmate"` to the root workspace `members`.[^workspace]
2. Classify it in `workspace_deps.py` as a new class `okmate` that may
   depend on `okf-engine` only. Forbid `okmate` → `base-rocci`,
   `rocdown`, or `okf-app`. Forbid every other class → `okmate`.[^deps-check]
3. Pin Askama, Axum, and `datastar` on the **okmate** package (or a
   narrow workspace.dependencies entry used only by okmate). Do not add
   them to Rocci crates.

When the directory becomes its own repo: keep the crate layout, change
`okf` from a path dep to a git / crates.io dep, and drop the workspace
member line here. No Rocci types should be in the tree.

`cargo test -p okmate` must not require `roc` or Rocci templates.

## Consequences

**Product.** Okmate is the distributable knowledge app: one binary, one
shell, no Roc. `rocci-okf` remains for Rocci CI and the knowledge skill
until cutover. Two CLIs will coexist for a while; document that
`okmate check` is the new app and `rocci-okf check` is the Rocci
tool.[^agents][^okf-app]

**Architecture.** This is the catalog-shell exception as a **separate
binary**, not a second markup language in Rocdown. Rocci apps keep
`.rocci` + `rocci-datastar` codegen. Okmate keeps Askama + official
Datastar. Two Datastar *authoring* paths, one *wire*.[^catalog-shell][^datastar-sdk][^research]

**Extract.** Root `okmate/` plus “only `okf` from this repo” is the
extract story. An in-place `rocci-okf` rewrite would have to peel off
`rocci-cli` later.

**Site.** rocci.dev `/knowledge/` can copy `okmate build` the same way it
would copy `rocci-okf build`. Live settings still need a host. Local-first
publication still gates the public snapshot.[^site-lane][^publication]

**Settings UX / multi-roots.** Domain rules stay; implementation is
rewritten in okmate (new config dir, Axum, Askama, `rfd` in-process).
Those plans’ Rocci-desktop IPC phases do not apply to okmate.[^settings-ux][^multi-roots]

**Previous viewer plans.** [rust+datastar](okf-viewer-rust-datastar.md)
and [host surfaces](okf-viewer-host-surfaces.md) are not the
implementation vehicle. Their research still stands. Do not start them.

**What we give up.** Dogfood of `OkfTheme`. Shared `goto.js` / preview
chrome with Rocci. A later Rocci-authored knowledge UI (Horizon B) is a
different repo/app problem; okmate is not that reference-in-the-same-crate.

## Phases

### Phase 1 — Scaffold `okmate/`

**Bound:** Create `/okmate` with `Cargo.toml` (`name = "okmate"`, edition
2024), `src/main.rs` + `src/lib.rs`, clap `--help` and a `check`
subcommand that calls `okf::check` and prints terminal or JSON.
Workspace member + `okmate` class in `workspace_deps.py`. README: name,
stack, “depends on `okf` only,” extract intent. No templates, no Axum,
no `rocci-*` deps.

**Out of bound:** HTML, Datastar, desktop, config registry.

**Tests:** `okmate check knowledge --profile rocci --format json` matches
`okf::check` error/ok; `uv run` workspace-deps (or the lint job’s
checker) accepts the new class.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `okmate/`, root `Cargo.toml`, `workspace_deps.py`.

### Phase 2 — Engine CLI parity

**Bound:** `inspect` (concept / catalog / graph), `search`, `benchmark`
as thin wrappers over `okf`. Flags and JSON shapes stay aligned with
today’s `rocci-okf` engine commands so agents can switch the binary name
without a new schema. No `--host`.[^okf-lib][^rocci-okf-main]

**Out of bound:** HTML, HTTP, roots registry.

**Tests:** Fixture or `knowledge/` inspect/search snapshots via `okf`
types, not stringly HTML.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `okmate/src/cli.rs` and command modules.

### Phase 3 — Askama documents and `build`

**Bound:** Templates listed above (documents only; settings can be a
static empty state). View structs from `okf::Bundle`. `okmate build
<root> -o <dir>` calls `okf::build` then writes HTML pages, `pages.json`
for Cmd-K, and copies `assets/app.css`. Landmarks: `#okmate-nav`,
`#okmate-main`, `#okmate-toc` when headings exist. No Roc, no apply.

**Out of bound:** Axum, Datastar, desktop.

**Tests:** Build a tiny fixture bundle; assert routes, `catalog.json`
from the engine, and landmark IDs.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `okmate/templates/`, `okmate/src/views/`, `okmate/assets/app.css`.

### Phase 4 — Axum `view` (headless)

**Bound:** `okmate view` starts an Axum server on localhost (or
`--public`). Serves the live tree: full-document GET for every review
route, static `/__okmate/*`. Watch + rebuild with `notify` (reuse the
idea of `rocci-okf` watch, not its types). `--no-window` is the only
preview mode in this phase. Persist last bundle path under
`~/.okmate/state`. No `rocci-cli` serve.

**Out of bound:** Datastar morph, desktop window, settings mutations.

**Tests:** Hyper/Axum test of GET `/` and a concept route; bind
localhost only by default.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `okmate/src/http/`, `preview.rs`.

### Phase 5 — Official Datastar and settings

**Bound:** Depend on `datastar` with `axum`. Stage `datastar.js`.
`base.html` loads it. Settings: load/save `~/.okmate/config.toml`, cards
and help copy from [settings UX](settings-ux.md), path field (no IPC
yet). Forms `@post('/__okmate/settings')`. Handler uses the SDK
extractor, applies the action, re-reads, renders
`fragments/settings.html`, returns `PatchElements`. Plain POST returns
the full settings page. Loopback-only. No live SSE. Optional import from
`~/.rocci/okf.toml` if present and okmate config is missing.[^settings-ux][^datastar-sdk][^server-state]

**Out of bound:** `rfd`, queue writes, `rocci-datastar`.

**Tests:** Datastar-Request POST returns a patch whose HTML contains
`id="okmate-settings"` and no `<html>`; ordinary POST is a full
document; tokens do not appear.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `http/`, `config.rs`, settings templates.

### Phase 6 — Datastar navigation and review region

**Bound:** In-app same-origin navigation via `@get` + `PatchElements` on
`#okmate-main` / `#okmate-toc` (keep `#okmate-nav` in the DOM). Cmd-K:
either a small local script fetching `/pages.json` or a Datastar-driven
palette; do not depend on `rocci-ui`. Review page includes
`#okmate-queue` and a fragment renderer. No approve/comment.

**Out of bound:** Revision-bound decisions. Live SSE.

**Tests:** GET with Datastar-Request on a concept returns a main
fragment; `/review/` contains `#okmate-queue`.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `http/`, `fragments/main.html`, `review.html`.

### Phase 7 — Desktop window and folder pick

**Bound:** `view` without `--no-window` opens wry/tao. `rfd` folder
dialog via a tiny IPC or wry custom protocol, matching [settings
UX](settings-ux.md) (not osascript, not HTTP pick-folder). Home opens
`/`.

**Out of bound:** App Sandbox bookmarks. `rocci-ops bundle okf`. Signed
notarized release.

**Tests:** Unit-test IPC/parse; window smoke is optional / ignored.

**Exit:** `cargo test -p okmate` and `cargo fmt --all -- --check`.

**Owner:** `okmate/src/desktop.rs`.

### Phase 8 — Roots, sync, and extract freeze

**Bound:** `roots` and `sync` against `~/.okmate/config.toml` (directory
and git roots, token_env, no echoed secrets), ported from the
[multi-roots](multi-knowledge-roots.md) contract without taking
`rocci-okf` modules. README: how to extract (`okf` path → git dep),
dependency list, CLI table, Datastar/Askama stack, coexistence with
`rocci-okf`. Golden fixtures for shell landmarks, settings patch, and
queue region.

**Out of bound:** Publishing a second GitHub repo. Switching Knowledge
CI to `okmate`. Cross-root `OKF3010` / `OKF3011` if that stays
application-side — include them here if they are required for `roots
--workspace` parity; otherwise a follow-on.

**Exit:** README names extract steps; fixtures exist; `cargo test -p
okmate`.

**Owner:** `config.rs`, README, fixtures.

## Status

Exploratory; no phase started. Evidence:
[viewer rust vs rocci](/research/okf/okf-viewer-rust-vs-rocci.md).
Supersedes [rust+datastar in rocci-okf](okf-viewer-rust-datastar.md) as
the implementation vehicle. Settings copy:
[settings UX](settings-ux.md) (desktop IPC rewritten here). Engine and
review-decision product:
[rocci-okf-app](rocci-okf-app.md).

[^research]: Unbound pick is Rust HTML + Datastar; hybrid two-shell is not a destination; public lane is a snapshot.
[^rust-datastar]: In-place rocci-okf Askama plan; superseded as vehicle because it stays on Rocci preview crates.
[^okf-app]: Engine/CLI product; review decisions stay later; UI authoring is this extract.
[^settings-ux]: rfd folder pick, cards, no live SSE, tokens redacted.
[^multi-roots]: User-level roots registry, git cache, edge policy, `roots` listing.
[^site-lane]: Foreign static app under `/knowledge/`; live ops need a host.
[^static-okf]: Inert Markdown; knowledge builds do not execute Roc or Rocci.
[^server-state]: Write, re-read, stable-id region; browser is not the domain store.
[^catalog-shell]: Binds Rocdown, not this binary.
[^publication]: Public deploy of generated knowledge is a separate review.
[^okf-readme]: Engine has zero Rocci crate deps.
[^okf-lib]: `check`, `load`, `inspect`, `search`, `build` artifacts are the consume surface.
[^rocci-okf-main]: Current verbs include view/check/inspect/search/build/roots/sync and `--host`.
[^presentation]: Dual writer and `format!` HTML to leave behind.
[^settings-rs]: Live settings are Rust POST today.
[^askama]: Compiled Jinja-like HTML, typed context structs, inheritance.
[^datastar-sdk]: Official SDK 0.4, Axum extractors and PatchElements, MSRV 1.89.
[^datastar-gh]: Framework-native SSE; long-lived streams via watch/broadcast.
[^deps-check]: Unclassified workspace members fail; okmate needs its own class.
[^workspace]: Members today live under `crates/`; okmate is a root member on purpose.
[^agents]: Knowledge CLI is `rocci-okf` until cutover; do not interpret Rocci in Rust.
