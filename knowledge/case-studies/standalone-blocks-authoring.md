---
type: Case Study
title: Nested standalone Blocks authoring boundary
description: "Rocci Blocks shows the current advanced standalone boundary: backend handlers and Game.roc versus a pure UI module, app-root rocci.toml discovery, commands plus live HTML, and a quoted keydown exception."
tags: [domain/rocci, integration/datastar, concern/developer-experience, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-22T13:00:00Z }
stale_after: 2026-11-22
authority: descriptive
owners: [human:nils]
sources:
  - id: audit
    resource: ../audits/rocci/standalone-falling-block-postmortem.md
    title: Standalone falling-block post-mortem
    author: process:cursor
    last_modified: 2026-08-22
  - id: plan
    resource: ../plans/rocci/standalone-falling-block.md
    title: Standalone falling-block authoring plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: backend
    resource: ../../examples/rocci/standalone/blocks/backend/Blocks.rocci
    title: Handler-only Blocks backend
    author: process:git
    last_modified: 2026-08-22
  - id: ui
    resource: ../../examples/rocci/standalone/blocks/ui/BlocksUi.rocci
    title: Play page and live slice
    author: process:git
    last_modified: 2026-08-22
  - id: run-rs
    resource: ../../crates/rocci-cli/src/run.rs
    title: App-root nested discovery
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Flat sibling live-counter app
    author: process:git
    last_modified: 2026-08-22
  - id: snake-main
    resource: ../../examples/rocci/custom/snake/main.roc
    title: Custom SSE unfold ceiling
    author: process:git
    last_modified: 2026-08-20
  - id: app-docs
    resource: ../../examples/rocci/standalone/blocks/index.rocdown
    title: Blocks tutorial and Shortcomings
    author: process:git
    last_modified: 2026-08-22
---

# Nested standalone Blocks authoring boundary

## Why this example exists

Rocci Blocks is the advanced **standalone** authoring example. It is not a
custom `main.roc` app and not the eight-player arena the earlier demonstrator
plan described.[^audit][^plan][^app-docs]

Live-counter remains the flat sibling pattern: one directory, handler module
plus UI module.[^live-counter] Snake remains the authored-runtime ceiling.
[^snake-main] Blocks sits between them: generated dispatch, nested modules,
server-owned game state.

## Layout that discovery can see

```text
examples/rocci/standalone/blocks/
  rocci.toml                 # app root; [[windows]].url = "/"
  backend/Blocks.rocci       # @context / @init / routes / SQLite
  backend/Game.roc           # pure rules
  ui/BlocksUi.rocci          # PlayPage, Board, Hud, Controls
```

`rocci run` on the backend entry walks up to this `rocci.toml` and stages both
directories. A repository-root `rocci.toml` is not an app.[^run-rs]

## Boundary rules illustrated

1. **I/O stays in `backend/`.** No `@component` there.[^backend]
2. **Markup stays in `ui/`.** `Board` and `Hud` take a view record; they do not
   open SQLite.[^ui]
3. **Commands vs live.** Moves are `@post:command` (`{}` / empty SSE / 204).
   `#board` and `#hud` morph only from `@get:live("/sse")`.[^backend][^ui]
4. **Document not in the live module.** Author `data-init` on the UI `<body>`.
   [^audit]
5. **Quoted keydown is the exception**, not the default click style.[^ui]

Copy this split when a standalone app outgrows a single directory. Do not copy
Snake’s unfold, and do not restore a canvas island to “fix” gravity feel;
those are documented shortcomings of this boundary, not missing phases.
[^audit][^plan]

[^audit]: Post-mortem records play-feel, injection trap, and custom-runtime remainder.
[^plan]: Standalone falling-block plan; nested layout and out-of-bound custom arena.
[^backend]: `Blocks.rocci` has routes and SQLite, not `@component`.
[^ui]: `BlocksUi.rocci` owns PlayPage, Board, Hud, Controls, and authored `data-init`.
[^run-rs]: App-root walk-up plus recursive staging.
[^live-counter]: Flat sibling modules in one directory remain the smaller live pattern.
[^snake-main]: Custom `main.roc` remains the ceiling for unfold and protocol work.
[^app-docs]: Blocks `index.rocdown` tutorial and Shortcomings.
