---
type: Implementation Plan
title: Deferred live-reload controls after the three follow-ons
description: "Three leftover live-reload controls after the native menu, --no-live-reload flag, and ?reload=0 pause: stop the file watcher, per-route reload policy, and an inspector-only pause. Exploratory; no phase started."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:05:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: follow-ons
    resource: ../live-reload-follow-ons.md
    title: Live reload follow-ons after the preview chrome toggle
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: reload-js
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Shared static dev server, watcher, and reload.js
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-js
    resource: ../../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script
    author: process:git
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: inspector-plan
    resource: ../preview-inspector.md
    title: Extended preview-window inspector
    author: process:cursor
    last_modified: 2026-08-19
---

# Deferred live-reload controls

## Purpose and authority

The [live-reload follow-ons](live-reload-follow-ons.md) plan shipped three
client controls that still leave watch/rebuild running: a View menu check
item, `--no-live-reload`, and `?reload=0`. This plan covers the first three
items that plan left out of bound. It is exploratory until a human reviewer
accepts a scope.[^follow-ons]

Do not start a phase until the user asks.

## Goal

Keep `sessionStorage` key `rocci-live-reload` plus `window.__rocciLiveReload`
as the client flag, then add three stronger controls:[^desktop-readme][^reload-js]

1. Optionally stop the file watcher (not only `location.reload()`).
2. Skip auto-reload on some routes while others still refresh.
3. Pause the preview document independently of the Dev inspector (or the
   reverse).

Do not author these controls in `.rocci`.[^preview-decision]

## Current evidence

`reload.js` always opens EventSource `/__rocci/events`. A `reload` event
calls `location.reload()` only when `enabled()` is true; otherwise it sets
`dirty`. The notify watcher and rebuild loop are not gated by that
flag.[^reload-js]

Pause is origin-wide: one `sessionStorage` key, no path matcher. Overlay
chrome and the View menu both call `set()`.[^preview-nav-js][^desktop-readme]

`GET /__rocci/dev` (and the Rocdown/OKF aliases) renders inspector HTML
without injecting `reload.js`. A parent-page reload still remounts the
host-owned iframe.[^reload-js][^inspector-plan][^desktop-readme]

## Out of bound

Persisting the pause across preview-window sessions (`localStorage`). The
follow-ons plan also deferred that; it stays out of this plan too.[^follow-ons]

Per-tab pause, stopping the HTTP server, and app-level Rocci logging.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Client flag | `sessionStorage` + `__rocciLiveReload` stay the runtime switch. Do not add a second parallel key. |
| Host chrome | Overlay and menus stay HTML/JS, not Rocci. |
| Serve continues | Pausing watch must not unbind the TCP listener or drop last-good HTML. |
| Dev exemption | Inspector HTML stays free of `reload.js` unless a phase explicitly opts it in and preserves `tab` / `route` / `view`. |

## Phase 1 — Stop the file watcher

Today pause skips `location.reload()` only. The watcher thread still
debounces notify events and rebuilds.[^reload-js]

- Add a host-side pause that stops or parks the watcher (and skips rebuild)
  while client live reload is off.
- Re-arm watch on resume. Do not require a process restart.
- Keep serving last-good output. Manual Reload still works.
- Tests: with watcher paused, a source edit does not rebuild; resume
  rebuilds. No preview window required if the watcher is unit-testable.

**Exit:** `--no-live-reload` / overlay pause can optionally stop rebuilds;
turning live reload back on starts watching again.

## Phase 2 — Per-route reload policy

One origin-wide flag reloads every HTML page that injected `reload.js`,
including live/Datastar routes the author may want to keep.[^reload-js]

- Honor a small same-origin policy (prefix list or page-kind) inside
  `reload.js` before `location.reload()`.
- EventSource may stay connected. Policy only gates the reload.
- Default remains “reload every injected page.”
- Tests: a denied route records dirty / skips reload; an allowed route still
  reloads; the default URL with no policy still auto-reloads.

**Exit:** a documented route can decline auto-refresh while siblings refresh;
the chrome toggle still pauses everything.

## Phase 3 — Inspector-only pause

Dev HTML is already exempt from `reload.js`. Parent-page auto-reload still
destroys the iframe and its query. The inspector plan left “exempt vs reload
preserving query” open.[^reload-js][^inspector-plan]

- When Dev is open, either skip parent `location.reload()` or reload the
  iframe in place while preserving `(tab, route, view)`.
- Do not inject `reload.js` into Dev unless that path preserves the query.
- Overlay chrome stays host-owned. No `.rocci` inspector shell.
- Tests: parent pause while the panel is open; iframe query survives a
  rebuild when the chosen policy reloads it.

**Exit:** working in the Dev panel does not lose tab/route/view because the
parent auto-refreshed; closing Dev restores ordinary live reload.

## Status

Exploratory; no phase started. Depends on the shipped follow-ons client
flag.[^follow-ons]

[^follow-ons]: Three shipped controls; watcher stop, per-route policy, inspector-only pause, and localStorage were out of bound.
[^desktop-readme]: Chrome toggle, View menu check, sessionStorage key, Dev iframe.
[^reload-js]: EventSource client, dirty flag, watcher loop, Dev HTML without reload.js.
[^preview-nav-js]: Overlay click handler and sessionStorage restore.
[^preview-decision]: Overlay chrome stays host HTML/JS, not Rocci.
[^inspector-plan]: Inspector live-reload choice (exempt vs preserve query) was listed, not closed.
