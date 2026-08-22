---
type: Audit
title: Standalone falling-block post-mortem
description: Compare the retired custom falling-block arena with the shipped nested standalone Rocci Blocks app against code: play-feel, handler-only modules, gravity-in-live, quoted keydown, origin removal, and what must stay custom.
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T13:00:00Z }
stale_after: 2026-11-22
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/standalone-falling-block.md
    title: Standalone falling-block authoring plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: prior-plan
    resource: ../plans/multiplayer-falling-block-demonstrator.md
    title: Historical custom-arena falling-block plan
    author: process:cursor
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
    title: Blocks catalog row is docs-only
    author: process:git
    last_modified: 2026-08-22
  - id: run-rs
    resource: ../../crates/rocci-cli/src/run.rs
    title: App-root nested standalone discovery
    author: process:git
    last_modified: 2026-08-22
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Module-local singleton live injection
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Shipped live-injection contract
    author: process:git
    last_modified: 2026-08-22
  - id: caddy
    resource: ../../docker/cdn/Caddyfile
    title: Hybrid Caddy without a Blocks play mount
    author: process:git
    last_modified: 2026-08-22
  - id: origin
    resource: ../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin publish health and compose without Blocks
    author: process:git
    last_modified: 2026-08-22
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: snake-main
    resource: ../../examples/rocci/custom/snake/main.roc
    title: Authored Snake server with custom SSE unfold
    author: process:git
    last_modified: 2026-08-20
---

# Standalone falling-block post-mortem

## Scope

This audit compares the retired custom arena (`examples/rocci/custom/blocks/`,
main-hostname `/play/blocks/`) with the shipped nested standalone app
(`examples/rocci/standalone/blocks/`) on branch `standalone-falling-block`.
It is descriptive and draft. It does not approve a live hostname or mark the
implementation plan complete.[^plan][^catalog]

The custom-arena plan remains historical evidence; this audit does not treat
its eight-player protocol as current product behavior.[^prior-plan]

## Custom arena versus shipped standalone

| Concern | Custom arena (retired) | Standalone Blocks (shipped in this revision) |
| --- | --- | --- |
| Runtime | Authored `main.roc`, cookies, JSON lock acks, canvas JS | Generated dispatch; handler-only `.rocci` plus `Game.roc` |
| Piece motion | Browser-owned falling piece | Server-owned gravity via live poll + `last_tick_ms` |
| Play surface | `/play/blocks/` on rocci.dev | Local `rocci run`; docs at `/examples/blocks/` |
| Multiplayer | Seats, garbage, spectators | One shared SQLite row |
| Origin | Compose profile, `/health/blocks`, Blocks artifacts | Island `/health` only |

The catalog row is `hosting = "docs"` with `entry = "backend/Blocks.rocci"`.
There is no `live_url`.[^catalog] Caddy no longer proxies a play mount.
[^caddy] Origin publish waits only on `/health`.[^origin]

## Play-feel

Every key is an HTTP `@post:command`. There is no client DAS/ARR; browser
key-repeat posts instead. Gravity is 800 ms in `Game.roc` and advances only
while a live subscriber polls; commands also apply due steps so a hidden tab
does not permanently desync the next move.[^game][^backend][^app-docs]

The board is a 20×10 HTML grid remorphed over SSE. That is not a 60 fps canvas
paint. Two tabs share one SQLite row, so they play the same game rather than
isolated seats.[^ui][^app-docs][^server-state]

These limits are documented as product Shortcomings, not accidental omissions.
[^app-docs]

## Handler-only module

`backend/Blocks.rocci` declares `@context`, `@init`, `@get:view`, `@get:live`,
and command routes. It has no `@component`. Render lives in
`ui/BlocksUi.rocci`.[^backend][^ui]

Nested discovery walks up from the entry file to the nearest app `rocci.toml`
and recurses; a repository-root `rocci.toml` is not an app.[^run-rs]

## Gravity in live

`load_ticked!` reads `last_tick_ms`, applies `elapsed.div_trunc_by(Game.gravity_ms)`
steps, and saves. Live and commands both call it. Zero subscribers means gravity
does not advance until the next command or a new stream.[^backend][^game]

Generated live polls on a 100 ms `After` loop. That clock is dispatch, not
author-written `Sse.unfold!`.[^template-readme]

## Quoted keydown

Unquoted Rocci actions cannot branch on `evt.key`. The play page uses a quoted
`data-on:keydown__window="… && @post('/actions/…')"` map. Buttons stay unquoted
`@post("/actions/…")`.[^ui][^app-docs]

## Module-local `data-init`

Auto-inject of `data-init=@get(path, [OpenWhenHidden(True)])` runs only when
**that module** has exactly one live route and the `<body>` has no authored
`data-init`.[^template-lower][^template-readme]

`PlayPage` lives in `BlocksUi.rocci`, which has no live route. The live route
is on `Blocks.rocci`. The UI module therefore authors
`data-init=@get("/sse", [OpenWhenHidden(True)])` on the document shell.[^ui]

## Origin removal

The custom binary, `docker/blocks/`, Compose profile `blocks`,
`/play/blocks/*`, `/health/blocks`, and `blocks-assets.tgz` packaging are
gone from this revision. Hybrid origin health is `/health` only.[^caddy][^origin]

## What must stay custom

Snake still owns an authored SSE unfold and a small input island. That ceiling
is appropriate when generated `@get:live` poll/keepalive is not enough.
Standalone Blocks does not copy `Sse.unfold!`.[^snake-main][^plan]

Cookies, JSON lock acknowledgements, seat leases, garbage targeting, and a
main-hostname play mount still require authored `main.roc` (or a future
language feature this plan explicitly refused).[^prior-plan][^plan]

## Residual risks

- **Play-feel vs genre expectation.** HTTP-command gravity will feel slower
  than a canvas island. Shortcomings must stay visible on the example page.
  [^app-docs]
- **Shared single game.** Concurrent browsers fight over one row. That is the
  live-counter pattern, not a bug, but it surprises players who expect seats.
  [^backend]
- **Gravity stall.** An unsubscribed game is frozen until the next command or
  stream. Documented; easy to misread as a hang.[^backend][^app-docs]
- **Injection trap.** Future nested apps that put `<body>` in a UI module will
  silently omit SSE unless they author `data-init`.[^template-lower]
- **No public demo origin.** Docs-only hosting means rocci.dev does not serve
  the playable app until a generic live hostname exists.[^catalog]
- **Operator leftovers.** A previously published origin may still have a
  Blocks volume or Caddy handle until the next origin deploy of this revision.
  This audit does not observe production.[^origin]

[^plan]: Implementation plan for nested standalone Blocks; not logged complete without CI and Knowledge run IDs.
[^prior-plan]: Historical custom-arena design with seats, lock acks, and a main-hostname play mount.
[^backend]: Handler-only module; gravity via `load_ticked!` and `last_tick_ms`.
[^game]: Solo gravity interval is 800 ms; overlay helpers paint active and ghost cells.
[^ui]: `#board` and `#hud` are live morph targets; controls sit outside those IDs.
[^app-docs]: Example Shortcomings document latency, quoted keydown, shared game, and HTML-grid limits.
[^catalog]: `hosting = "docs"` and `entry = "backend/Blocks.rocci"`; no public play URL.
[^run-rs]: Nested staging requires app-root `rocci.toml`.
[^template-lower]: Injection inspects only the module being lowered.
[^template-readme]: Singleton local live route is the auto-subscribe condition.
[^caddy]: No `/play/blocks/*` or `/health/blocks` handle remains.
[^origin]: Origin publish health list is `/health` only.
[^server-state]: Canonical game state is server-owned.
[^snake-main]: Snake still owns a custom SSE unfold.
