# rocci-desktop

Native windowing and webview host built on [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry).

## Responsibilities

- **Window lifecycle**: Spawns and manages the desktop window, runs the Tao event loop, and coordinates IPC between the host and webview.
- **Embedded webview**: Loads the local HTTP server URL, applies DevTools configurations, and configures security policies.
- **Window state persistence**: Automatically restores and saves window size and position across sessions.
- **Packaging runtime**: Acts as the desktop container in ad-hoc signed macOS application bundles.

## Dependencies

- Relies on `tao`, `wry`, and `muda` (native menus).
- Consumes `rocci-core` for configuration types.
- Zero dependencies on `rocci-rocdown`, `okf`, `rocci-okf`, or language parsers.
