# rocci-desktop

Rocci preview facade over [`h35-desktop`](../../h35-desktop). Product CLIs still call `preview(PreviewOptions)`.

This crate supplies Rocci defaults: `~/.rocci/state` (or `ROCCI_STATE_DIR`), the Rocci icon, and compatibility aliases (`__rocciPreviewNav`, `rocci-pick-folder`). Windowing, toolbar chrome, and IPC live in `h35-desktop`.
