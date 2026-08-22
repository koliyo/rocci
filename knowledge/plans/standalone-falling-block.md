---
type: Implementation Plan
title: Standalone falling-block authoring example
description: "Rebuild Rocci Blocks as a nested standalone Rocci app with server-owned gravity, HTML board, @post:command, and @get:live. Retire the custom arena and main-hostname play mount. Exploratory; Phases 1–6 implemented on standalone-falling-block. Not logged complete until CI and Knowledge succeed."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T13:00:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: prior-plan
    resource: multiplayer-falling-block-demonstrator.md
    title: Historical custom-arena falling-block plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: human:nils
    last_modified: 2026-08-16
  - id: run-rs
    resource: ../../crates/rocci-cli/src/run.rs
    title: App-root nested standalone discovery
    author: process:git
    last_modified: 2026-08-22
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Module-local live data-init injection
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Shipped handler and live-injection contract
    author: process:git
    last_modified: 2026-08-22
  - id: backend
    resource: ../../examples/rocci/standalone/blocks/backend/Blocks.rocci
    title: Handler-only Blocks backend
    author: process:git
    last_modified: 2026-08-22
  - id: game
    resource: ../../examples/rocci/standalone/blocks/backend/Game.roc
    title: Solo falling-block rules
    author: process:git
    last_modified: 2026-08-22
  - id: ui
    resource: ../../examples/rocci/standalone/blocks/ui/BlocksUi.rocci
    title: Play page, board, HUD, and controls
    author: process:git
    last_modified: 2026-08-22
  - id: app-docs
    resource: ../../examples/rocci/standalone/blocks/index.rocdown
    title: Blocks tutorial and Shortcomings
    author: process:git
    last_modified: 2026-08-22
  - id: catalog
    resource: ../../examples/rocci/apps.toml
    title: Example catalog hosting for Blocks
    author: process:git
    last_modified: 2026-08-22
  - id: caddy
    resource: ../../docker/cdn/Caddyfile
    title: Hybrid Caddy routes without a play mount
    author: process:git
    last_modified: 2026-08-22
  - id: audit
    resource: ../audits/standalone-falling-block-postmortem.md
    title: Post-mortem of custom arena versus standalone Blocks
    author: process:cursor
    last_modified: 2026-08-22
---

# Standalone falling-block authoring example

## Purpose and authority

This plan rebuilds **Rocci Blocks** as an authoring-first solo game on generated
standalone dispatch. Public copy says “falling-block arena.” It replaces the
product shape in the [custom-arena plan](multiplayer-falling-block-demonstrator.md);
that record stays historical evidence for the eight-player `/play/blocks/`
design.[^prior-plan]

This is exploratory. Phases 1–6 are implemented on `standalone-falling-block`
in this revision. Do not log the plan complete until CI and Knowledge workflow
run IDs exist. A generic live hostname remains out of bound.[^catalog]

## Goal

Ship `examples/rocci/standalone/blocks/` as a nested standalone app:

- `backend/Blocks.rocci` owns `@context`, `@init`, SQLite, routes.
- `backend/Game.roc` owns pure rules and overlay cells.
- `ui/BlocksUi.rocci` owns the document, `#board`, `#hud`, controls, fixtures, CSS.
- Moves are `@post:command`; gravity and shared morphs are `@get:live("/sse")`.
- One shared SQLite row. No cookies, seats, JSON lock acks, or canvas island.
- Catalog `hosting = "docs"`, `entry = "backend/Blocks.rocci"`.
- Snake remains the custom `main.roc` ceiling.

Durable board state stays on the server.[^server-state] UI modules stay
pure.[^pure-render]

## Out of bound

- Eight-player garbage, spectators, leases, canvas island, gamepad.
- A main-hostname `/play/blocks/` Caddy exception or `/health/blocks`.
- New handler roles, command JSON, or a general base-path feature.
- Copying Snake’s `Sse.unfold!`.
- Language or parser work except app-root nested discovery (Phase 1).
- Declaring the audit `stable` or inventing `human:nils` verification events.
- A live example hostname until a generic live origin exists.

## Constraints that do not move

1. No Datastar signals as board source of truth.[^server-state]
2. No `@component` I/O.[^pure-render]
3. Do not morph `#board` from both a fragment and the live stream.
4. Do not add Tetris branding or the old Python lock-protocol harnesses.
5. Preserve unrelated worktree or main dirty files; do not push.

## Phases

### Phase 1 — App-root nested discovery

**Bound:** `rocci-cli` standalone planning/staging only (`standalone_app_root`,
`discover_standalone_tree`, recursive sibling `.roc` copy). Walk up from the
entry to the nearest `rocci.toml`, but do not treat a repo-root `rocci.toml` as
an app. Stop at `.git` or Cargo `[workspace]`. Tests for a two-directory
fixture and unchanged flat live-counter discovery.[^run-rs]

**Exit:** `cargo test -p rocci-cli` and `cargo fmt --all -- --check`.

**Status:** implemented on `standalone-falling-block`.

### Phase 2 — Domain port

**Bound:** `backend/Game.roc` only. Solo rules plus helpers to paint locked
board and active/ghost overlay. No HTTP. Drop garbage, targeting, and seats.
[^game]

**Exit:** compiled with Phase 3; no separate Roc binary.

**Status:** implemented on `standalone-falling-block`.

### Phase 3 — Backend module

**Bound:** `backend/Blocks.rocci` handlers and SQLite schema (`board`, `piece`,
`rot`, `x`, `y`, `bag`, `seed`, `score`, `status`, `last_tick_ms`). No
`@component`.[^backend]

**Exit:** `cargo run -q -p rocci-cli -- inspect --ast examples/rocci/standalone/blocks/backend/Blocks.rocci`

**Status:** implemented on `standalone-falling-block`.

### Phase 4 — UI module and playable loop

**Bound:** `ui/BlocksUi.rocci`, `index.rocdown`, `README.md`, `rocci.toml`. Wire
`@post` / `@get` primitives. Required Shortcomings: latency, no DAS, quoted
keydown, shared single game, gravity only while streamed, HTML grid, no
multiplayer/gamepad, generated `/health` only. Nested apps need app-root
`rocci.toml`. The document in `ui/` authors `data-init` because injection is
module-local.[^ui][^app-docs][^template-lower][^template-readme]

**Exit:** inspect both `.rocci` files; `rocci run …/backend/Blocks.rocci --no-window`.
`POST /actions/reset` is 204; with `Datastar-Request: true` it is empty SSE.

**Status:** implemented on `standalone-falling-block`.

### Phase 5 — Retire custom arena and origin exception

**Bound:** delete `examples/rocci/custom/blocks/`. Retarget catalog, inventory,
`docs/applications/standalone.rocdown`, `docs/applications/custom.rocdown`,
site UX contract. Remove Caddy
`/play/blocks/*` and `/health/blocks`, Compose `blocks` profile,
`docker/blocks/`, and origin packaging of a Blocks binary.[^catalog][^caddy]

**Exit:** leftover `/play/blocks` live routes gone except the historical custom
plan. Site contract path updated.

**Status:** implemented on `standalone-falling-block`.

### Phase 6 — Skills

**Bound:** `.agents/skills/rocci-author` and `rocci-stack`: nested `backend/` vs
`ui/`, app-root `rocci.toml`, quoted keydown exception, counter / live-counter /
blocks / Snake line. Fix stale `@view` / `@patch` idiom rows.

**Exit:** skills match the shipped example.

**Status:** implemented on `standalone-falling-block`.

### Phase 7 — Knowledge close and audit

**Bound:** this plan record, the [post-mortem audit](../audits/standalone-falling-block-postmortem.md),
indexes, and `knowledge/log.md`. Optional case study if the audit earns it.
[^audit]

**Exit:** `cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal`.
Do not invent verification events or log the plan complete without CI and
Knowledge run IDs.

**Status:** this revision authors the records; not CI-complete.

[^prior-plan]: The custom-arena plan remains historical evidence for eight-player `/play/blocks/` design.
[^server-state]: Durable board state stays in SQLite, not Datastar signals.
[^pure-render]: `@component` modules lower to pure Html functions.
[^run-rs]: Discovery walks up to the nearest app `rocci.toml` and skips repo-root workspace config.
[^template-lower]: Live `data-init` injection is computed from that module's live routes only.
[^template-readme]: Auto-subscribe requires exactly one local live route and no authored `data-init`.
[^backend]: Handler-only module owns SQLite, commands, and `@get:live`.
[^game]: Solo rules and overlay helpers live in ordinary Roc.
[^ui]: Play page, board, HUD, and controls live in the UI module.
[^app-docs]: Shortcomings are required example documentation, not a later leftover.
[^catalog]: Catalog `hosting` is `docs`; no `live_url`.
[^caddy]: Hybrid Caddy no longer proxies a Blocks play mount.
[^audit]: Post-mortem compares custom arena versus shipped standalone against code.
