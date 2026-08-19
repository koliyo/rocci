# rocci-desktop

Native windowing and webview host built on [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry).

## Responsibilities

- **Window lifecycle**: Spawns and manages the desktop window, runs the Tao event loop, and coordinates IPC between the host and webview.
- **Embedded webview**: Loads the local HTTP server URL, applies DevTools configurations, and configures security policies.
- **Window state persistence**: Automatically restores and saves window size and position across sessions.
- **Packaging runtime**: Acts as the desktop container in ad-hoc signed macOS application bundles.

## Preview chrome

Host chrome is HTML, CSS, and JS under `assets/`. `preview-nav.html` is the markup, `preview-nav.css` is injected with `textContent` into the shadow tree, and `preview-nav.js` plus `reduced-motion.js` mount the custom element and talk to `window.ipc`. Find-in-page and go-to-file overlays (`preview-find.*`, `preview-goto.*`, `preview-keys.js`) mount as sibling custom elements. Rust only JSON-embeds those assets and pushes title, path, and history flags through `evaluate_script`. Native Edit/View menu items call the same overlay methods via `evaluate_script`. When `PreviewOptions.inspector_url` is set, the overlay shows a Dev control that toggles a host-owned iframe to that preview-origin panel.

Preview keyboard shortcuts (Command on macOS, Control elsewhere):

- **Find** (`F`): open the current-document find overlay; uses the selection when one exists
- **Use Selection for Find** (`E`): set the find query from the selection without forcing the overlay open
- **Find Next** / **Find Previous** (`G` / `Shift-G`): move between matches and wrap at the ends
- **Go to File** (`K`): fuzzy-jump to a document from `/pages.json`, `/catalog.json`, or site nav links

Do not author host chrome in `.rocci`. A template can snapshot markup, but it cannot own wry IPC, survive page loads, or update live state. Compiler-derived panels (parse timings, diagnostics, inspectors) belong in a preview-origin Rocci app that consumes host JSON, not in the initialization script overlay.

## Dependencies

- Relies on `tao`, `wry`, and `muda` (native menus).
- Consumes `rocci-core` for configuration types.
- Zero dependencies on `rocci-template`, `rocci-rocdown`, `okf`, `rocci-okf`, or language parsers. Chrome assets are embedded with `include_str!`.
