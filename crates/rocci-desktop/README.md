# rocci-desktop

Native windowing and webview host built on [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry).

## Responsibilities

- **Window lifecycle**: Spawns and manages the desktop window, runs the Tao event loop, and coordinates IPC between the host and webview.
- **Embedded webview**: Loads the local HTTP server URL, applies DevTools configurations, and configures security policies.
- **Window state persistence**: Automatically restores and saves window size and position across sessions.
- **Packaging runtime**: Acts as the desktop container in ad-hoc signed macOS application bundles.

## Preview chrome

Preview navigation markup is authored in `templates/PreviewNav.rocci`. Layout CSS lives in `assets/preview-nav.css` and is injected with `textContent` so quoted selectors are not HTML-escaped. `build.rs` hashes the Rocci file against `generated/preview_nav.sha256`. When the hash is stale and `roc` is on PATH, it regenerates `generated/preview_nav.html` and embeds the snapshot from `OUT_DIR`. When `roc` is missing, the committed fragment is used (`ROCCI_REQUIRE_ROC=1` fails the build instead). Webview host scripts live in `assets/reduced-motion.js` and `assets/preview-nav.js`. Manual regeneration:

```sh
cargo run -q -p rocci-cli -- render crates/rocci-desktop/templates/PreviewNav.rocci --fragment -o crates/rocci-desktop/generated/preview_nav.html
```

## Dependencies

- Relies on `tao`, `wry`, and `muda` (native menus).
- Consumes `rocci-core` for configuration types.
- Zero *runtime* dependencies on `rocci-rocdown`, `okf`, `rocci-okf`, or language parsers. Build-time snapshot regeneration uses `rocci-template` / `rocci-roc-host` only in `build.rs`. The library still embeds JS assets and HTML with `include_str!`.
