---
type: Implementation Plan
title: Author the rocci-browser picker as a host-owned Rocci origin
description: "Gate 4 follow-on after rocci-browser Phases 1–5: move the Cmd-P two-stage picker off the initialization-script overlay and onto a host-owned HTTP origin so it can be authored in Rocci without owning wry IPC. Exploratory; no phase started."
tags: [domain/rocci, domain/desktop, concern/ui, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T23:15:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: browser-plan
    resource: rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-research
    resource: ../research/rocci-browser.md
    title: Dedicated rocci-browser CLI and desktop host research
    author: process:cursor
    last_modified: 2026-08-19
  - id: chrome-research
    resource: ../research/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: overlay-rs
    resource: ../../crates/rocci-browser/src/overlay.rs
    title: Picker asset embedding into the initialization script
    author: process:cursor
    last_modified: 2026-08-19
  - id: picker-js
    resource: ../../crates/rocci-browser/assets/picker.js
    title: Host picker overlay script
    author: process:cursor
    last_modified: 2026-08-19
  - id: picker-html
    resource: ../../crates/rocci-browser/assets/picker.html
    title: Host picker overlay markup
    author: process:cursor
    last_modified: 2026-08-19
  - id: launcher-rs
    resource: ../../crates/rocci-browser/src/launcher.rs
    title: Host-owned launcher HTTP origin
    author: process:cursor
    last_modified: 2026-08-19
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
---

# Author the rocci-browser picker as a host-owned Rocci origin

## Goal

Keep Cmd-P as the host picker (Enter / Tab, not Cmd-K) while authoring its
visible UI in Rocci. The picker must still exist before any product origin
and survive `load_url` of adapter origins. That is overlay *lifecycle*, not
a `@component` on a child server.[^browser-plan][^browser-research][^chrome-research]

Do not snapshot the picker from a `.rocci` template into the wry
initialization script. Desktop chrome already rejected that: a template can
freeze idle markup, but it cannot own `window.ipc`, survive navigations, or
push live state.[^desktop-readme][^preview-decision][^pure-render]

Human approval of gate 4 is required before any phase.

## Shipped (not these phases)

The host embeds `picker.html` / `picker.css` / `picker.js` in
`initialization_script`, mounts a shadow-tree custom element, and talks JSON
IPC. A separate loopback **launcher** origin serves a static empty page
until the first `open`. Overlay chrome (nav, find, Dev iframe) stays
`rocci-desktop` HTML.[^overlay-rs][^picker-js][^picker-html][^launcher-rs][^desktop-readme]

## Out of bound

- Teaching `@component` wry IPC or browser objects.
- Replacing preview-nav, find, or the Dev iframe with Rocci (still host
  chrome).
- Encoding `site` / `docs` / `knowledge` as builtin picker rows (gate 6).
- Runtime `roc` compile of the picker on every host launch.
- Adding a `rocci-template` edge that would make `cargo test -p rocci-browser`
  compile Rocdown or start product adapters.

## Phase 1 — Picker as its own origin (still HTML)

Move the current picker DOM out of the init-script overlay and into an
iframe (or second webview) whose `src` is the existing launcher origin.
Content `load_url` must not unload that iframe. Cmd-P focuses it; Tab in
the query still `preventDefault`. IPC can stay `window.ipc` from the parent
or `postMessage` into Rust.[^launcher-rs][^picker-js][^chrome-research]

Keep scoring and two-stage keys. Tests: fixture adapter; overlay script no
longer contains the picker HTML string; no product CLIs.

**Exit:** Cmd-P picker works from a host-owned origin that survives hopping
targets. Still HTML/JS, not Rocci.

## Phase 2 — Rocci markup on that origin

Author picker chrome as `.rocci` under the browser crate (or a tiny
sibling app crate classified base Rocci). Generate static HTML at *crate
build* time through the existing Rocci CLI, and serve those files from the
launcher. Live filter/selection stays JavaScript plus host JSON, matching
`goto.js`. Do not interpret formats in host `src/`.[^desktop-readme][^pure-render][^browser-plan]

The generated files are derived artifacts; check in a build step, not a
second scoring algorithm.

**Exit:** Visible picker shell comes from Rocci-generated HTML on the
launcher origin; Enter / Tab / Escape behavior is unchanged; host source
stays product-blind.

## Phase 3 — Drop the overlay picker

Once Phase 2 is the default, delete `assets/picker.*` from the
initialization script. Native Open Target menu still focuses the iframe.
Document that picker UI is a host-owned Rocci origin, while preview chrome
remains HTML.[^overlay-rs][^desktop-readme]

**Exit:** No picker markup in `extra_initialization_script`; public docs
name the split.

## Status

Exploratory; no phase started. Blocked on rocci-browser gate 4. Phase 1 is
the lifecycle proof; Rocci authorship is Phase 2, not a rewrite of overlay
IPC.

[^browser-plan]: Gate 4: author picker UI in Rocci instead of host HTML.
[^browser-research]: Picker must exist before any product origin; v1 is HTML under the browser crate.
[^chrome-research]: Overlay HUD stays HTML; Rocci only when loaded as its own origin (iframe or second webview).
[^preview-decision]: Overlay chrome stays distinct from preview-origin inspector UI.
[^desktop-readme]: Do not author host chrome in .rocci; snapshot cannot own wry IPC.
[^overlay-rs]: Current picker is JSON-embedded into the initialization script.
[^picker-js]: Cmd-P, Tab preventDefault, and ipc.postMessage live in overlay JS.
[^picker-html]: Overlay dialog markup for Open target.
[^launcher-rs]: Host-owned loopback origin that currently serves static launcher HTML.
[^pure-render]: Rocci components are functions from explicit values to Html.
