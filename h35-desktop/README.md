# h35-desktop

Native windowing and webview host for hypermedia-driven desktop apps, built on [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry).

The crate owns the preview window and toolbar chrome. The page origin owns document chrome. Datastar, if used, lives in that origin — this crate does not depend on it.

## Responsibilities

- **Window lifecycle**: Spawns the desktop window, runs the Tao event loop, and coordinates IPC between the host and webview.
- **Preview chrome**: Overlay HTML/JS plus optional extra initialization scripts and host IPC (`HostOptions.on_ipc`) so a long-lived window can `load_url` a new origin without leaving `preview()`. Cmd-P is an optional **Open Target** picker; Cmd-K is Go to File when `goto` is on.
- **Window state persistence**: Restores and saves window size, position, and sidebar column widths in `{state_dir}/windows.json`, keyed by `identifier`. `HostOptions.home_url` can keep toolbar Home on a dashboard while the first load is another route. `on_navigate` reports committed page-load URLs.
- **Host icon**: Optional `HostOptions.icon_png` applied as the window icon and, on macOS, the Dock image.

## Preview chrome

Host chrome is HTML, CSS, and JS under `assets/`. `preview-nav.html` is the markup, `preview-nav.css` is injected into the shadow tree, and `preview-nav.js` plus `reduced-motion.js` mount the custom element and talk to `window.ipc`. On macOS, `preview()` uses a Safari-style unified titlebar: a transparent full-size titlebar, hidden window title, and traffic lights vertically centered in the same 52px overlay row. Empty chrome starts a native window drag; double-click zooms. Windows and Linux keep a stacked system titlebar plus the 48px overlay. Find-in-page mounts as a sibling custom element. Go to File embeds `goto.js` (`window.__h35Goto`) and aliases it onto `window.__h35PreviewNav.goto`. If the loaded site already mounted `__h35Goto`, the host does not create a second palette.

A **Live reload** toggle next to Reload pauses automatic page refresh (`sessionStorage` key `h35-live-reload`). When `HostOptions.inspector_url` is set, the overlay shows a Dev control that toggles a host-owned iframe. While Dev is open, the overlay docks the iframe right (default `28rem`) or bottom (default `36vh`) only. Dock side, open/closed, tab, view, and dock sizes persist in `{state_dir}/inspector.json`. Dock controls use DevTools-style icons (dock, Open as page, and Web Inspector). A visible splitter grip resizes the dock. **Open as page** navigates the host webview to the inspector document. **Web Inspector** opens the native wry Web Inspector. The overlay compares inspector URLs as `(origin, path, tab, route)` tuples and does not assign `iframe.src` for a Source `view`-only change.

Pages can post `pick-folder` over `window.ipc`. The host opens a native folder dialog and dispatches `CustomEvent("h35-pick-folder")`.

## Dependencies

tao, wry, muda, rfd. No language parsers and no product crates.
