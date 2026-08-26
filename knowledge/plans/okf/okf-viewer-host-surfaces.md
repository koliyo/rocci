---
type: Implementation Plan
title: Short-term OKF viewer host surfaces
description: Make settings a live Rust host route with bundle nav, stop compiling unused Settings/ReviewQueue Rocci, and leave document apply and future queue/agent operations alone.
tags: [domain/okf, domain/rocci-okf, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: okmate
    resource: okmate.md
    title: Okmate — extractable Rust OKF mate
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-datastar
    resource: okf-viewer-rust-datastar.md
    title: In-place rocci-okf Askama rewrite (superseded as vehicle)
    author: process:cursor
    last_modified: 2026-08-26
  - id: research
    resource: ../../research/okf/okf-viewer-rust-vs-rocci.md
    title: OKF viewer Rust HTML versus finished Rocci shell
    author: process:cursor
    last_modified: 2026-08-26
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
  - id: settings-rs
    resource: ../../../crates/rocci-okf/src/settings.rs
    title: Settings article HTML and POST actions
    author: process:git
    last_modified: 2026-08-25
  - id: presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: Apply generate, html_page_for, and html_page_at
    author: process:git
    last_modified: 2026-08-25
  - id: okf-main
    resource: ../../../crates/rocci-okf/src/main.rs
    title: extra_http wraps settings with html_page_at
    author: process:git
    last_modified: 2026-08-25
  - id: runtime-rs
    resource: ../../../crates/rocci-okf/src/runtime.rs
    title: Embedded Settings and ReviewQueue sources
    author: process:git
    last_modified: 2026-08-25
  - id: settings-rocci
    resource: ../../../crates/rocci-okf/templates/Settings.rocci
    title: Stub SettingsPage not used as live renderer
    author: process:git
    last_modified: 2026-08-25
  - id: review-rocci
    resource: ../../../crates/rocci-okf/templates/ReviewQueue.rocci
    title: ReviewQueue compiled but unused by OkfBuild
    author: process:git
    last_modified: 2026-08-24
  - id: okf-readme
    resource: ../../../crates/rocci-okf/README.md
    title: rocci-okf view, settings, and host contract
    author: process:git
    last_modified: 2026-08-25
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable state is server-owned
    author: process:okf-migration
    last_modified: 2026-08-16
---

# Short-term OKF viewer host surfaces

## Goal

Stop the settings page from living in two pipelines, and make the crate’s
surface split visible so later queue and agent work has a place to land.
Preview `/settings/` is a live Rust host route with the same sidebar as the
bundle. Unused `.rocci` widgets are not compiled. Document apply stays as
it is.[^research][^settings-ux][^okf-main][^presentation]

## Out of bound

Askama, Maud, or any new Rust HTML DSL. Datastar, live SSE, or `@post` on
settings or the review queue. Review decisions, agent query/tasks, or an
operations workbench. Shipping a prebuilt applicator or wasm apply-to-disk.
Deleting `OkfTheme.rocci` or forcing skip-Roc as the default document path.
Changing the portable `okf` engine. Replacing [settings UX](settings-ux.md)
folder-pick and card copy. Interpreting `.rocci` in Rust to skip Roc.

## Constraints that do not move

- Knowledge records stay inert Markdown. `okf` stays UI-neutral.[^research]
- Durable registry is `okf.toml`. Mutations stay one-shot POST. Tokens are
  never echoed.[^multi-roots][^settings-ux][^server-state]
- Live settings markup is `settings.rs`, not `Settings.rocci`.[^settings-ux][^settings-rs]
- `/__rocci_okf/settings` stays loopback-only.[^okf-readme]
- Catalog `check` / `inspect` / `search` do not require Roc.

## Current structure to fix

Three things already disagree:[^research][^presentation][^okf-main]

1. `view` extra_http GET/POST `/settings/` wraps `settings::render_article`
   with `html_page_at`, whose nav is `render_nav_tree(None, …)` — Dashboard /
   Review / Settings only, no collection tree.
2. `generate_okf_page_data` still bakes a settings article into apply. With
   `roc` present, first paint of `/settings/` can be `OkfTheme`; after POST
   it is the Rust host shell.
3. `Settings.rocci` and `ReviewQueue.rocci` compile into the renderer hash
   and are not called from `OkfBuild.render_page`.[^runtime-rs][^settings-rocci][^review-rocci]

Short-term work is ownership, not a stack migration.

## Surfaces (keep)

| Surface | Short-term owner | Later |
| --- | --- | --- |
| Concept / collection / dashboard / review HTML | Current apply-if-roc, Rust `html_page_for` if not | Unchanged here |
| Settings, session, folder pick | Rust `extra_http` + `settings.rs` | Stay host |
| Queue decisions / agent jobs | Do not start | Rocci+Datastar in a later plan |

## Phases

### Phase 1 — Unplug unused Rocci widgets

**Bound:** `compile_okf_templates` compiles `PageOutline`, `ConceptMeta`, and
`OkfTheme` only. Leave `templates/Settings.rocci` and
`templates/ReviewQueue.rocci` on disk as sketches, or move them under
`templates/sketches/` so they are not `include_str!` into the hash. Update
tests that assert those type names are compiled. README notes they are not
the live renderers.[^runtime-rs][^presentation][^okf-readme]

**Out of bound:** Changing `OkfBuild.render_page`. Rewriting the sketches
into a real UI.

**Tests:** `compile_okf_templates` listing; renderer hash fixtures if they
name ReviewQueue/Settings.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `crates/rocci-okf/src/presentation.rs`, `runtime.rs`.

### Phase 2 — Settings is not an apply page

**Bound:** `generate_okf_page_data` does not emit a settings page record or
`articles/settings.html`. Apply does not write `settings/index.html`.
`build` / pure-Rust write still emit `settings/index.html` through
`html_page_for` so `inspect` and static `build` keep a snapshot.
Preview `view` continues to serve live GET `/settings/` and POST
`/__rocci_okf/settings` from extra_http.[^presentation][^okf-main][^settings-rs]

**Out of bound:** Changing document apply for concepts. Datastar.

**Tests:** generate/apply fixture has no `settings/index.html` from apply;
`build` still writes one; extra_http GET still returns the article.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `presentation.rs` generate and write.

### Phase 3 — Live settings uses bundle nav

**Bound:** extra_http wraps settings with the same nav as other preview
pages (`html_page_for` or equivalent with the loaded `Bundle`), not
`html_page_at` + `render_nav_tree(None)`. Pass the bundle (or pre-rendered
nav HTML) into the preview handler. `--no-window` and missing bundle still
render the three top links.[^okf-main][^presentation]

**Out of bound:** Making settings a Rocci apply page. Changing goto.js.

**Tests:** extra_http GET HTML contains a known collection route from the
test bundle; POST still redacts tokens and stays `text/html`.

**Exit:** `cargo test -p rocci-okf` and `cargo fmt --all -- --check`.

**Owner:** `crates/rocci-okf/src/main.rs`, `presentation.rs`.

### Phase 4 — Name the split in the crate README

**Bound:** `crates/rocci-okf/README.md` section: document pages vs host
routes vs future operations; settings is extra_http; sketches are not
compiled; point at this plan and the research. No public product-docs
change unless the README sentence is already the contract.[^okf-readme][^research]

**Out of bound:** Architecture decision rewrite. Operations plan.

**Exit:** README mentions the three surfaces; `cargo test -p rocci-okf`.

**Owner:** `crates/rocci-okf/README.md`.

## Status

Superseded. The extractable app is [okmate](okmate.md). Do not start this
plan or the in-place [rust+datastar](okf-viewer-rust-datastar.md)
rewrite.[^okmate][^rust-datastar]
Pair:
[viewer rust vs rocci](/research/okf/okf-viewer-rust-vs-rocci.md). Settings
copy and folder pick remain [settings UX](settings-ux.md).

[^okmate]: Extractable Askama + Axum + official Datastar app; this record is not started.
[^rust-datastar]: Superseded in-place vehicle.
[^research]: Avoid whole-app Askama and static-only Rocci apply; settings stay Rust host; operations would be a later Rocci+Datastar app.
[^settings-ux]: Live markup is `settings.rs`; `Settings.rocci` is not the renderer; no live SSE for the registry.
[^multi-roots]: Registry POSTs, token redaction, `/settings/` chrome.
[^settings-rs]: GET `/settings/` and POST `/__rocci_okf/settings` return article HTML.
[^presentation]: generate still includes settings; `html_page_at` has no bundle nav; write fallback uses `html_page_for`.
[^okf-main]: extra_http wraps settings with `html_page_at`.
[^runtime-rs]: Settings and ReviewQueue sources are embedded and compiled.
[^settings-rocci]: Stub forms only.
[^review-rocci]: Not called from `OkfBuild.render_page`.
[^okf-readme]: Settings POSTs are loopback-only; view host auto.
[^server-state]: Durable settings are not a browser store.
