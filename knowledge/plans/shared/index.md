# Shared

Chrome that spans OKF, Rocdown, and Rocci products.

* [CLI entry points for Rocci, Rocdown, and OKF preview](cli-entry-points.md) - Keep the three product CLIs, reject a plugin host, and make `rocci-okf run` the file-aware OKF viewer.
* [Editor preview for Rocci and Rocdown](editor-preview.md) - VS Code play command serves `--no-window` and opens a beside-file webview; Zed gets native-window tasks until it has a webview. Phases 1–5 landed on branch `editor-preview`; not merged to `main`. Research: [editor preview](/research/shared/editor-preview.md).
* [Hosted editor preview chrome and unbundled tools](editor-preview-host.md) - Follow-on: VS Code webview hosts the Rocci toolbar and Dev inspector; GitHub-release downloads replace packaged binaries. Exploratory; no phase started. Research: [hosted editor preview](/research/shared/editor-preview-host.md).
* [Cmd-K fuzzy navigation for OKF, Rocdown, and rocci.dev](fuzzy-navigation.md) - Shared `goto.js` palette in preview and hosted trees, History-API HTML swap. Implemented in this revision; not CI-complete.
* [Mobile chrome for OKF, Rocdown, and rocci.dev](mobile-chrome.md) - No-JS details menus, OKF nav split from TOC, rocci.dev menu restore, table overflow. Exploratory; Phases 1–3 and 5–6 implemented; Phase 4 skipped; not CI-complete.
