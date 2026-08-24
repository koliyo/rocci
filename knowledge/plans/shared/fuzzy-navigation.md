---
type: Implementation Plan
title: Cmd-K fuzzy navigation for OKF, Rocdown, and rocci.dev
description: Shared goto.js palette for preview and hosted sites, with History-API swaps of already-rendered HTML. Implemented in this revision; not CI-complete.
tags: [domain/okf, domain/rocci-okf, domain/rocdown, domain/rocci, domain/desktop, concern/rendering, concern/accessibility]
status: draft
generated: { by: process:cursor, at: 2026-08-19T17:45:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: goto-js
    resource: ../../../crates/rocci-ui/assets/goto.js
    title: Shared go-to-page palette
    author: process:cursor
    last_modified: 2026-08-19
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Rocdown CSP, hashed goto.js, chrome_script
    author: process:git
    last_modified: 2026-08-19
  - id: okf-presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: OKF review HTML, catalog.json, and goto.js
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-chrome
    resource: ../../../crates/rocci-desktop/src/chrome.rs
    title: Preview host embeds shared goto.js
    author: process:git
    last_modified: 2026-08-19
  - id: known-limitations
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:okf-phase-6
    last_modified: 2026-08-19
---

# Cmd-K fuzzy navigation

## Goal

One Cmd/Ctrl-K fuzzy page palette in OKF review, default Rocdown docs, and
rocci.dev, working in desktop preview and on hosted static trees. Selecting a
result (and same-origin in-page links) fetches already-rendered HTML and swaps
it via the History API. This is document navigation, not full-text search.[^goto-js][^known-limitations]

## Shipped shape

- Shared client module `crates/rocci-ui/assets/goto.js` exports
  `window.__rocciGoto`. Overlay mounts on `document.documentElement` so it
  survives body swaps.[^goto-js]
- Catalog: `/pages.json`, then `/catalog.json`, then scraped nav.[^goto-js]
- Rocdown hashes `goto.js` onto `ResourceView.chrome_script` and default CSP is
  `script-src 'self'; connect-src 'self'`.[^plan-rs]
- OKF review copies `catalog.json`, `pages.json`, and `/__rocci_okf/goto.js`
  into the HTML tree.[^okf-presentation]
- Desktop preview embeds the same script and aliases native **Go to File** onto
  `__rocciGoto`. A page that already mounted the palette does not get a second
  overlay.[^desktop-chrome]
- `live` / Datastar / extra-script pages keep `location.assign`.[^goto-js]

## Out of bound

Browser Roc WASM, Datastar morph, `@island`, and OKF `search.json` full-text UI.

## Status

Implemented on this revision. Not logged complete until CI and Knowledge
workflows succeed.

[^goto-js]: Shared palette, catalog load, SPA swap, and full-load exceptions.
[^plan-rs]: Hashed chrome script and default CSP.
[^okf-presentation]: Review-tree indexes and script injection.
[^desktop-chrome]: Host embed and native-menu alias.
[^known-limitations]: Full-text documentation search remains a separate gap.
