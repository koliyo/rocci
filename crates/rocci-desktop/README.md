# rocci-desktop

Native windowing and webview host built on [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry).

## Responsibilities

- **Window lifecycle**: Spawns and manages the desktop window, runs the Tao event loop, and coordinates IPC between the host and webview.
- **Preview chrome**: Overlay HTML/JS plus optional extra initialization scripts and host IPC (`PreviewOptions.on_ipc`) so a long-lived window can `load_url` a new origin without leaving `preview()`. Cmd-P is an optional **Open Target** picker when a host supplies targets; Cmd-K remains Go to File.
- **Window state persistence**: Automatically restores and saves window size and position across sessions.
- **Packaging runtime**: Acts as the desktop container in ad-hoc signed macOS application bundles.
- **Host icon**: Embeds `assets/rocci-icon.png` (1024px folded-R, rendered from `brand/rocci-app.svg` via `uv run rocci-ops render-brand-icons`) and applies it as the macOS Dock image for `preview()` and `run()`. Windows and Linux use the same PNG as the window icon. macOS ignores Tao window icons; the Dock tile comes from `NSApplication.setApplicationIconImage`.

## Preview chrome

Host chrome is HTML, CSS, and JS under `assets/`. `preview-nav.html` is the markup, `preview-nav.css` is injected with `textContent` into the shadow tree, and `preview-nav.js` plus `reduced-motion.js` mount the custom element and talk to `window.ipc`. Find-in-page (`preview-find.*`, `preview-keys.js`) mounts as a sibling custom element. Go to File embeds the shared `rocci-ui` `goto.js` palette (`window.__rocciGoto`) and aliases it onto `window.__rocciPreviewNav.goto` for native menus. If the loaded site already mounted `__rocciGoto`, the host does not create a second palette. Rust JSON-embeds host assets and pushes title, path, and history flags through `evaluate_script`. Native Edit/View menu items call the same overlay methods via `evaluate_script`. A **Live reload** toggle next to Reload pauses automatic page refresh (`sessionStorage` key `rocci-live-reload`); the View menu has a matching **Live Reload** check item. Overlay clicks post `live-reload:0` / `live-reload:1` so the check mark stays in sync; menu clicks call `window.__rocciLiveReload.set(...)`. Watch/rebuild continues and manual Reload still works. Re-enabling reloads if a rebuild happened while paused. `--no-window` browsers have no chrome; open the printed URL with `?reload=0` to pause automatic refresh (`sessionStorage` key `rocci-live-reload`). Pass `--no-live-reload` on `rocci run` / `view` / `browse` so preview chrome opens paused; the overlay and View menu can turn it back on. When `PreviewOptions.inspector_url` is set, the overlay shows a Dev control that toggles a host-owned iframe to that preview-origin panel. While Dev is open, the overlay docks the iframe right (default `28rem`, inset via `--rocci-chrome-right`) or bottom (default `36vh`, inset via `--rocci-chrome-bottom`) only. Dock side, open/closed, tab, view, and dock sizes persist in `~/.rocci/state/inspector.json` (same directory as window geometry in `windows.json`), keyed by the preview `state_key`, and are seeded into the overlay on launch. Dock controls use DevTools-style icons in an overlay toolbar above the iframe (dock, Open as page, and Web Inspector); the top-nav Dev toggle uses the Rocci mark icon. A visible splitter grip resizes the dock. **Open as page** navigates the host webview to the inspector document for full-page inspect-UI work; the Dev dock is hidden while that page is the main content. **Web Inspector** opens the native wry Web Inspector while keeping the overlay toolbar visible so you can switch back to the Rocci inspector (`devtools:1` / `devtools:0` IPC). The overlay compares inspector URLs as `(origin, path, tab, route)` tuples, and persists tab/view through iframe `postMessage`. It appends `tab` and `route` when the iframe must reload, does not assign `iframe.src` for a Source `view`-only change, and does not embed compiler output. Native Web Inspector remains available from the View menu as well.

Preview keyboard shortcuts (Command on macOS, Control elsewhere):

- **Find** (`F`): open the current-document find overlay; uses the selection when one exists
- **Use Selection for Find** (`E`): set the find query from the selection without forcing the overlay open
- **Find Next** / **Find Previous** (`G` / `Shift-G`): move between matches and wrap at the ends
- **Go to File** (`K`): fuzzy-jump to a document from `/pages.json`, `/catalog.json`, or site nav links. Hosted Rocdown, rocci.dev, and OKF review pages use the same palette and swap already-rendered HTML in-place except for `live` / Datastar pages.
- **Select All** (`A`): select the document article when the page marks a select root (`data-rd-select-root`, `article.rd-article`, or `article.article`); otherwise the whole page. Find and Go to File fields keep field-level Select All. Copy uses the live selection.

When `PreviewOptions.source_root` is set (OKF and Rocdown preview), a trailing **More** (`...`) menu can reveal the original source file and copy its contents. Reveal uses the platform file manager: Finder on macOS, Explorer on Windows, and Files on Linux.

Do not author host chrome in `.rocci`. A template can snapshot markup, but it cannot own wry IPC, survive page loads, or update live state. Compiler-derived panels (parse timings, diagnostics, inspectors) belong in a preview-origin Rocci app that consumes host JSON, not in the initialization script overlay.

## Dependencies

- Relies on `tao`, `wry`, and `muda` (native menus).
- Consumes `rocci-core` for configuration types and `rocci-ui` for the shared go-to-page script.
- Zero dependencies on `rocci-template`, `rocci-rocdown`, `okf`, `rocci-okf`, or language parsers. Chrome assets are embedded with `include_str!`; the host icon is embedded with `include_bytes!`.
