# rocci-desktop

Rocci preview facade over [`h35-desktop`](https://github.com/koliyo/h35-desktop). Product CLIs still call `preview(PreviewOptions)`.

This crate supplies Rocci defaults: `~/.rocci/state` (or `ROCCI_STATE_DIR`), the Rocci icon, and compatibility aliases (`__rocciPreviewNav`, `rocci-pick-folder`, `--rocci-chrome-*` → `--h35-chrome-*`). Windowing, toolbar chrome, and IPC live in `h35-desktop`. Fill `body`; do not size shells with `100vh` / `100dvh`. The host toolbar is a layout row, not an overlay to pad around.
