---
type: Decision
title: Call the embedded Tao/Wry shell the preview window
description: Name the native window opened by run, view, and browse commands the preview window, and keep overlay chrome distinct from preview-origin inspector UI.
tags: [domain/rocci, domain/desktop, domain/runtime, concern/architecture, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-18T20:00:00Z }
stale_after: 2026-11-18
authority: descriptive
owners: [human:nils]
sources:
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-18
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Preview window entry point
    author: process:git
    last_modified: 2026-08-18
  - id: preview-nav
    resource: ../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview chrome navigation markup
    author: process:git
    last_modified: 2026-08-18
  - id: chrome-research
    resource: ../research/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: readme
    resource: ../../README.md
    title: Rocci README
    author: human:nils
    last_modified: 2026-08-18
---

# Call the embedded Tao/Wry shell the preview window

## Context

`rocci run`, `rocdown run`, `rocci-okf run`, `rocci view`, and `rocci browse` open the same native Tao/Wry window. Docs and CLI help mixed *embedded window*, *run window*, and *preview window*, while launch research uses *public preview* for a different idea.[^readme][^preview-rs]

Host overlay navigation is HTML/CSS/JS under `rocci-desktop/assets`. Compiler-derived panels are not that overlay.[^desktop-readme][^chrome-research]

## Decision

Call that native window the **preview window**.

| Term | Meaning |
| --- | --- |
| preview window | The Tao/Wry window those commands open |
| preview chrome | The injected `rocci-preview-nav` overlay bar only |
| dev panel | Preview-origin Rocci inspector (profiling first), loaded in a host-owned iframe |
| webview | The Wry content surface inside the preview window |
| embedded | Adjective: `--no-window` skips opening the preview window |

Do not reuse **public preview** (launch) or OKF `resolve_preview_path` (which file or URL to open).

Rejected names: *run window* (misses `view` / `browse`), *host window* (clashes with Roc native/wasm host), *desktop window* (too broad for later multi-window apps).

Where compiler metrics or other inspector UI appear in the preview window, author them as a preview-origin Rocci app that consumes host JSON. Overlay chrome may add a control that opens that panel; it does not snapshot the panel into the initialization script.[^chrome-research][^desktop-readme][^preview-nav]

## Consequences

CLI help, crate READMEs, and public docs can use one noun for the window. `rocci-desktop` stays free of `rocci-template`. Document and site chrome remain Rocci on their HTTP origin; that is a separate catalog-shell decision.

## Current disposition

Draft naming contract aligned with the shipped `preview()` API and overlay assets. Overlay-versus-inspector authorship remains exploratory research until a human accepts this record.

[^desktop-readme]: Host chrome is HTML/CSS/JS under `assets/`; compiler-derived panels belong on the preview origin.
[^preview-rs]: `preview()` and `PreviewOptions` are the desktop entry for the window.
[^preview-nav]: Overlay markup for back, forward, home, reload, path, and title.
[^chrome-research]: Recommended split between wry overlay chrome and preview-origin inspector UI.
[^readme]: Product overview that currently describes the same window as embedded and as a preview window.
