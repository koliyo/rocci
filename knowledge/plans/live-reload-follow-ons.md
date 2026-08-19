---
type: Implementation Plan
title: Live reload follow-ons after the preview chrome toggle
description: "Three live-reload controls after the shipped preview-bar toggle: a native View menu check item, a shared CLI flag, and a --no-window browser pause. Phases 1–3 implemented in this revision; not CI-complete."
tags: [domain/rocci, domain/desktop, domain/runtime, domain/rocdown, domain/rocci-okf, concern/ui, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T20:05:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-html
    resource: ../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview chrome navigation markup
    author: process:git
    last_modified: 2026-08-19
  - id: preview-nav-js
    resource: ../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script
    author: process:git
    last_modified: 2026-08-19
  - id: reload-js
    resource: ../../crates/rocci-cli/src/dev_server.rs
    title: Shared static dev server and reload.js
    author: process:git
    last_modified: 2026-08-19
  - id: menu-rs
    resource: ../../crates/rocci-desktop/src/menu.rs
    title: Native preview window menus
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: Shared ServeOptions including --no-window
    author: process:git
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: deferred
    resource: live-reload-deferred.md
    title: Deferred live-reload controls after the three follow-ons
    author: process:cursor
    last_modified: 2026-08-19
---

# Live reload follow-ons

## Goal

Keep one client source of truth (`sessionStorage` key `rocci-live-reload` plus
`window.__rocciLiveReload`) while adding three controls that the first chrome
toggle left out: a native menu check item, a CLI flag, and a pause that works
in `--no-window` browsers.[^desktop-readme][^reload-js]

Watch/rebuild stays on in every phase. Only the webview `location.reload()` is
gated. Do not author these controls in `.rocci`.[^preview-decision]

## Shipped (this revision, not these phases)

Preview chrome has a Live reload toggle next to Reload. `/__rocci/reload.js`
keeps EventSource connected, skips reload while the key is `"0"`, marks dirty,
and reloads on re-enable if a rebuild arrived while paused. Manual Reload
still works.[^preview-nav-html][^preview-nav-js][^reload-js]

## Out of bound

Stopping the file watcher, per-route reload policy, and inspector-only pause
are planned separately in [deferred live-reload controls](live-reload-deferred.md).
Persisting the pause across preview-window sessions (`localStorage`) stays
unplanned.[^deferred]

## Phase 1 — Native View menu check item

Implemented in this revision. View already had Reload (`view.reload`). It had
no live-reload item and no `CheckMenuItem` usage.[^menu-rs]

- Add a checkable **Live Reload** item next to Reload.
- Toggle by evaluating `window.__rocciLiveReload.set(...)` in the webview.
- Sync the check mark from the overlay (new IPC, or poll `enabled()` after
  page load). Overlay click and menu click must agree.
- Tests: menu id present when `MenuConfig.reload` is true; no preview window.

**Exit:** View menu can pause and resume live reload; chrome button and menu
check stay in sync after a click from either side.

## Phase 2 — Shared CLI flag

Implemented in this revision. `ServeOptions` already shared `--no-window` and
`--port` across `rocci run` / `view` / `browse`. Rocdown and `rocci-okf run`
had parallel flags.[^serve-rs]

- Add `--no-live-reload` (default off) on those same run surfaces.
- Seed `sessionStorage` from the initialization script, or skip injecting
  `reload.js` and still expose `__rocciLiveReload` so the chrome toggle can
  turn it back on.
- Do not stop watch/rebuild. Print that live reload is paused.
- Tests: clap parse; `initialization_script` or inject path honors the flag.

**Exit:** `rocci run`, `rocdown run`, and `rocci-okf run` accept the flag;
preview chrome opens already paused; re-enabling from the bar still works.

## Phase 3 — `--no-window` browser pause

Implemented in this revision. `--no-window` prints a URL and skips the
preview window, so there is no overlay toggle.[^serve-rs][^desktop-readme]

- Honor a same-origin query such as `?reload=0` (or a cookie) inside
  `reload.js` before connecting behavior.
- Keep `sessionStorage` as the runtime flag so a later preview window on the
  same origin sees the pause.
- Document the query on the three CLI `--no-window` help texts.
- Tests: `RELOAD_JS` reads the query; default URL without the query still
  auto-reloads.

**Exit:** Opening the printed URL with `?reload=0` in an ordinary browser
pauses auto-refresh; removing the query (or calling `set(true)`) resumes.

## Status

Phases 1–3 implemented in this revision. Exploratory until CI and Knowledge
workflows succeed. The chrome toggle remains a separate shipped change on the
parent revision.

[^desktop-readme]: Chrome Live reload toggle, sessionStorage key, and `--no-window` gap.
[^preview-nav-html]: Overlay markup for the Live reload button.
[^preview-nav-js]: Overlay click handler and sessionStorage restore.
[^reload-js]: EventSource client, dirty flag, and `__rocciLiveReload` API.
[^menu-rs]: View menu Reload item; no live-reload check item.
[^serve-rs]: Shared `--no-window` / `--port`; no live-reload flag.
[^preview-decision]: Overlay chrome stays host HTML/JS, not Rocci.
[^deferred]: Watcher stop, per-route policy, and inspector-only pause.
