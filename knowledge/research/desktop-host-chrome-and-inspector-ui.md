---
type: Research Report
title: Desktop host chrome versus Rocci inspector UI
description: Exploratory split between wry overlay chrome authored in HTML/CSS/JS and richer compiler-derived panels authored as preview-origin Rocci apps that consume host JSON.
tags: [domain/rocci, domain/desktop, domain/runtime, concern/rendering, concern/architecture, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-18T19:48:00Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-18
  - id: chrome-rs
    resource: ../../crates/rocci-desktop/src/chrome.rs
    title: Preview chrome asset embedding and update script
    author: process:git
    last_modified: 2026-08-18
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Preview window initialization script and IPC
    author: process:git
    last_modified: 2026-08-18
  - id: preview-nav-html
    resource: ../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview navigation markup
    author: process:git
    last_modified: 2026-08-18
  - id: preview-nav-js
    resource: ../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview navigation host script
    author: process:git
    last_modified: 2026-08-18
  - id: preview-nav-css
    resource: ../../crates/rocci-desktop/assets/preview-nav.css
    title: Preview navigation styles
    author: process:git
    last_modified: 2026-08-18
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell decision
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: generation-research
    resource: rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: generation-plan
    resource: ../plans/rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: islands
    resource: ../decisions/client-behavior-islands.md
    title: Use explicit islands for browser-owned behavior
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: ui-readme
    resource: ../../crates/rocci-ui/README.md
    title: rocci-ui view records
    author: process:git
    last_modified: 2026-08-18
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-17
  - id: known-limitations
    resource: ../status/known-limitations.md
    title: Known limitations
    author: process:okf-phase-6
    last_modified: 2026-08-18
---

# Desktop host chrome versus Rocci inspector UI

## Scope and authority

This record is exploratory. It does not approve a new language feature or
change the shipped catalog-versus-shell split for documentation sites. It
asks where *desktop preview overlay* UI should be authored, and where
*compiler-derived inspector UI* such as parse timings should live instead.[^desktop-readme][^catalog-shell]

It does not reverse the research or plan that move demonstrated *document and
site* chrome into Rocci. Those surfaces are HTTP-origin pages compiled through
Roc. Host overlay chrome is a different lifecycle.[^generation-research][^generation-plan]

## Problem

Preview navigation must sit above every loaded URL, survive page loads, talk
to wry IPC, and receive title, path, and history flags from Rust. A `.rocci`
module can snapshot markup from props and `@if` / `@for`, but an `@component`
is a pure function from explicit values to `Html`. It does not own wry
`window.ipc`, custom-element lifecycle, or live updates after the snapshot is
taken.[^pure-render][^template-readme][^preview-rs]

Compiling `PreviewNav.rocci` at desktop build time therefore froze the idle
fixture into HTML while back, forward, title, and path updates remained in
JavaScript. Scoped `@css` also failed inside the shadow tree because quoted
`@scope` selectors were HTML-escaped in a `<style>` tag. The overlay is host
chrome, not a Rocci view.[^chrome-rs][^preview-nav-js]

## Current host overlay

`rocci-desktop` embeds `assets/preview-nav.html`, `assets/preview-nav.css`,
`assets/preview-nav.js`, and `assets/reduced-motion.js`. Rust JSON-encodes
those strings into the webview `initialization_script`. The script mounts a
`rocci-preview-nav` custom element, injects CSS with `textContent`, and posts
`back` / `forward` / `home` / `reload` through `window.ipc`. Later
`evaluate_script` calls push title, path, and history flags.[^desktop-readme][^chrome-rs][^preview-nav-html][^preview-nav-css][^preview-nav-js][^preview-rs]

The crate has no runtime or build dependency on `rocci-template`. Native
capabilities beyond the current window and webview boundary remain
absent.[^desktop-readme][^known-limitations]

## Recommended split

Split on lifecycle, not on how detailed the pixels are.

| Layer | Author in | Owns |
| --- | --- | --- |
| Host overlay (navigation bar, always-on HUD that outlives navigation) | HTML, CSS, and JS under `crates/rocci-desktop/assets` | wry initialization script, `window.ipc`, `evaluate_script` |
| Document and site chrome (sidebar, breadcrumbs, theme shell) | `.rocci` on the preview or site HTTP origin | structured view records, Roc `Html`, existing generation hosts |
| Inspector and metrics panels (parse timings, diagnostics, playground tools) | `.rocci` served from the preview HTTP origin | compiler JSON, `@on` / Datastar, lists and charts over view records |

Host overlay stays HTML because it must attach before the document, persist
across navigations, and speak the native IPC contract. Rocci cannot express
that contract without becoming a browser object, which is the unimplemented
island path rather than ordinary `@component` semantics.[^pure-render][^islands][^desktop-readme]

Document and site chrome stay Rocci. That is the accepted catalog-shell
boundary and the generation-pipeline direction: Rust owns data, Rocci owns
visible HTTP-origin chrome compiled through the cached Roc host.[^catalog-shell][^generation-research][^ui-readme]

Inspector UI is a preview-origin Rocci app, not an initialization-script
fragment. Compiler work already happens in Rust. The host should expose
timings and diagnostics as JSON (an HTTP route on the preview origin, or IPC
into a panel that then renders). A Rocci component receives that payload as
ordinary props or a `rocci-ui` view record and renders lists, tables, or
charts with the language's existing `@for` / `@match` and server
handlers.[^ui-readme][^template-readme][^pure-render]

If a panel must overlay *every* page the way the navbar does, it remains host
chrome: HTML and JS that consume the same JSON. Rocci enters that picture only
when the panel is loaded as its own origin (iframe or second webview) so it
gets a normal HTTP lifecycle instead of an init-script overlay.[^preview-rs][^islands]

## Worked example: parse-performance UI

1. `rocci-template` or the CLI records parse, lower, and compile timings in
   Rust while compiling a `.rocci` file.[^template-readme]
2. The preview server exposes that snapshot as JSON, or the desktop host
   forwards it over IPC.[^preview-rs]
3. A Rocci `MetricsPanel` (or playground route) renders the view record on
   the preview origin, composing shared structure from `rocci-ui` when the
   shape is domain-neutral.[^ui-readme][^generation-plan]
4. The host overlay does not snapshot that panel. It may offer a control that
   opens the panel URL.

This keeps parser and host logic in Rust, presentation in Rocci, and wry
chrome in HTML.

## Open questions

- Whether metrics travel over an HTTP route such as `/__rocci/metrics`, wry
  IPC, or both.
- Whether an inspector is a playground route, a sibling iframe, or a second
  webview.
- Whether inspector chrome reuses `rocci-ui` templates (`NavList`,
  `PageOutline`) or stays product-owned in `rocci-cli` / `rocci-okf`.
- Whether a future explicit island construct should ever host overlay
  behavior; until that lands, overlay JS stays ordinary assets.[^islands]

## Disposition

Draft and exploratory. Current desktop preview navigation follows the host
overlay column. Document-shell Rocci and inspector panels remain
recommendations until a reviewer accepts or revises this split.

[^desktop-readme]: Current crate contract that host chrome is HTML/CSS/JS under `assets/` and compiler-derived panels belong on the preview origin.
[^chrome-rs]: `include_str!` embedding of HTML/CSS/JS and `evaluate_script` updates for title, path, and history flags.
[^preview-rs]: wry `initialization_script`, IPC handler for navigation commands, and page-load title sync.
[^preview-nav-html]: Static overlay markup with control ids `back`, `forward`, `home`, `reload`, `path`, and `title`.
[^preview-nav-js]: Custom element, shadow CSS injection, `window.ipc.postMessage`, and `__rocciPreviewNav.update`.
[^preview-nav-css]: Opaque full-width overlay rules injected with `textContent` rather than HTML-escaped `<style>`.
[^pure-render]: `@component` lowers to a Roc function from explicit values to `Html` and does not own persistence, request lifecycle, or client state.
[^catalog-shell]: Accepted split: Rust owns catalog data; Rocci owns visible HTTP-origin documentation chrome.
[^generation-research]: Exploratory evidence for moving demonstrated site and widget HTML into Rocci through cached Roc hosts.
[^generation-plan]: Plan for shared outline, nav, and breadcrumb components in base Rocci, not desktop wry overlays.
[^islands]: Proposed, unimplemented explicit island construct for browser-owned behavior, distinct from `@component`.
[^ui-readme]: Domain-neutral view records as the shared data shape for presentation components.
[^template-readme]: `.rocci` modules lower to Roc; standalone HTTP apps use `@on`, not wry IPC.
[^known-limitations]: Desktop host exposes the current window/webview boundary, not general native capabilities.
