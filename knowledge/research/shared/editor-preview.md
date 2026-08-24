---
type: Research Report
title: Editor preview for Rocci and Rocdown
description: VS Code can host the existing CLI --no-window origin in Simple Browser beside the source file; Zed has no extension webview, so the feasible fallback is a task that opens the native preview window or a system browser.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/ui, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-24T21:30:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: vscode-client
    resource: ../../../editors/vscode/src/extension.ts
    title: VS Code Rocci language client
    author: process:git
    last_modified: 2026-08-18
  - id: vscode-manifest
    resource: ../../../editors/vscode/package.json
    title: VS Code extension manifest
    author: process:git
    last_modified: 2026-08-23
  - id: vscode-channels
    resource: ../../../editors/vscode/src/output-channels.ts
    title: VS Code Rocci output channel
    author: process:git
    last_modified: 2026-08-18
  - id: zed-ext
    resource: ../../../editors/zed/src/lib.rs
    title: Zed Rocci language-server adapter
    author: process:git
    last_modified: 2026-08-14
  - id: zed-manifest
    resource: ../../../editors/zed/extension.toml
    title: Zed extension manifest
    author: process:git
    last_modified: 2026-08-18
  - id: zed-readme
    resource: ../../../editors/zed/README.md
    title: Zed Rocci extension README
    author: process:git
    last_modified: 2026-08-18
  - id: language-tooling
    resource: ../../architecture/language-tooling.md
    title: Language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-24
  - id: preview-window
    resource: ../../decisions/preview-window.md
    title: Preview window naming decision
    author: process:cursor
    last_modified: 2026-08-24
  - id: cli-plan
    resource: ../../plans/shared/cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: rocci-cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: rocci CLI contract
    author: process:git
    last_modified: 2026-08-24
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: Shared --no-window serve helpers
    author: process:git
    last_modified: 2026-08-24
  - id: rocci-run
    resource: ../../../crates/rocci-cli/src/run.rs
    title: rocci run entry resolution
    author: process:git
    last_modified: 2026-08-24
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown-cli/README.md
    title: rocdown CLI contract
    author: process:git
    last_modified: 2026-08-24
  - id: rocdown-cli
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown view dispatch and --no-window
    author: process:git
    last_modified: 2026-08-24
  - id: impl-plan
    resource: ../../plans/shared/editor-preview.md
    title: Editor preview implementation plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: vscode-webview
    resource: https://code.visualstudio.com/api/extension-guides/webview
    title: VS Code webview extension guide
    author: organization:microsoft
    last_modified: 2026-08-19
  - id: vscode-contrib
    resource: https://code.visualstudio.com/api/references/contribution-points
    title: VS Code contribution points
    author: organization:microsoft
    last_modified: 2026-08-19
  - id: simple-browser
    resource: https://github.com/microsoft/vscode/blob/main/extensions/simple-browser/src/extension.ts
    title: Built-in Simple Browser command API
    author: organization:microsoft
    last_modified: 2026-08-24
  - id: integrated-browser
    resource: https://github.com/microsoft/vscode/pull/303312
    title: Desktop Simple Browser routes to Integrated Browser
    author: organization:microsoft
    last_modified: 2026-08-24
  - id: zed-webview-issue
    resource: https://github.com/zed-industries/zed/issues/21208
    title: Webview via extensions issue
    author: organization:zed-industries
    last_modified: 2026-08-24
  - id: zed-html-preview
    resource: https://github.com/zed-industries/zed/discussions/27163
    title: General HTML preview discussion
    author: organization:zed-industries
    last_modified: 2026-08-24
  - id: zed-preview-api
    resource: https://github.com/zed-industries/zed/discussions/59598
    title: Custom read-only file preview API discussion
    author: organization:zed-industries
    last_modified: 2026-08-24
  - id: zed-tasks
    resource: https://zed.dev/docs/tasks
    title: Zed tasks documentation
    author: organization:zed-industries
    last_modified: 2026-08-24
  - id: zed-ext-api
    resource: https://docs.rs/zed_extension_api/latest/zed_extension_api/
    title: zed_extension_api crate
    author: organization:zed-industries
    last_modified: 2026-08-24
---

# Editor preview for Rocci and Rocdown

## Claim

A useful in-editor preview is a **hosted product origin**, not a second renderer. VS Code already has an embedded browser that can sit beside the active document; Rocci and Rocdown already print a loopback URL when `--no-window` skips the Tao/Wry [preview window](/decisions/preview-window.md). Zed extensions cannot open an equivalent pane today, so Zed should launch the existing native preview window (or a system browser) from a task, and wait for a host preview API.[^vscode-client][^serve-rs][^rocdown-cli][^zed-webview-issue][^impl-plan]

This record is exploratory evidence for the [implementation plan](/plans/shared/editor-preview.md). It does not change shipped editor or CLI behavior.

## What the editors ship today

The VS Code and Zed adapters are thin LSP clients. They register `.rocci` and `.rocdown`, start `rocci-language-server`, and expose one command (`rocci.restartLspServer` in VS Code). There is no preview command, no editor-title button, and no process manager for `rocci` or `rocdown`.[^vscode-manifest][^vscode-client][^zed-manifest][^language-tooling]

Compile, watch, live reload, Datastar, hybrid islands, and the Dev inspector already belong to the product CLIs and `rocci-desktop`. `--no-window` is the documented headless mode: serve, print the URL, skip the preview window. Rocdown `view` on a file under a `rocdown.toml` ancestor previews the site and opens that page's route.[^rocci-cli-readme][^rocdown-readme][^rocdown-cli][^preview-window]

Re-rendering templates inside the extension would fork that stack and violate the three-CLI split.[^cli-plan]

## Why the preview must be an HTTP origin

Markdown-style HTML preview is the wrong model. A `.rocci` app and a live `.rocdown` page are same-origin HTTP:

- Datastar patches and `data-init` talk to the preview origin.
- Live reload uses EventSource on that origin (`?reload=0` pauses it).
- Hybrid Rocdown sites proxy island POSTs on the same host.
- Failed rebuilds still serve the last HTML plus a dialog.

Those behaviors already work in any browser that loads the printed URL. The editor's job is to start the right CLI, bind the URL beside the file, and stop the process. It should not snapshot HTML into a webview.[^rocci-cli-readme][^rocdown-readme][^serve-rs]

## VS Code embedding options

| Option | Fit | Cost |
| --- | --- | --- |
| `simpleBrowser.api.open` with `viewColumn: Beside` | Matches “embedded browser to the right,” same pattern as Live Preview and Vite helpers; current VS Code desktop may route this to Integrated Browser | Depends on the built-in Simple Browser / Integrated Browser contribution |
| Custom `WebviewPanel` that iframes `http://127.0.0.1` | Full control of chrome; works if Simple Browser is missing | Duplicates address/reload UI; webview CSP and iframe focus quirks |
| Custom editor / `CustomTextEditor` | Replaces or splits the source tab | Wrong for authors who need the template beside the running app; does not remove the HTTP origin requirement |
| `vscode.env.openExternal` | Always works | Leaves the editor; not the requested beside-preview |

Recommendation: **call `simpleBrowser.api.open` first**, with `ViewColumn.Beside` and `preserveFocus: true`, which is the public API the built-in Simple Browser exposes and the path desktop VS Code is folding into Integrated Browser. If that command is absent, fall back to a minimal webview iframe (Live Preview’s pattern), then to the system browser. Do not ship a Custom Editor for v1.[^simple-browser][^integrated-browser][^vscode-webview]

Place the play control on `editor/title` (Markdown preview style) and also register a Command Palette command. `editor/title/run` is the Run submenu used by language “play” buttons; contributing there as well matches Code/Java/Python without hiding the preview icon. Show the button when `editorLangId` is `rocci` or `rocdown`.[^vscode-contrib]

## Session shape

One preview session per VS Code window is enough for v1:

1. Save the active file if dirty (disk is what the CLI watches).
2. Resolve `rocci` or `rocdown` the same way the LSP binary is resolved: setting, bundled `dist/bin`, `PATH`, then workspace `target/debug`.
3. Spawn with `--no-window --port auto` so the desktop preview window does not also open, and so a busy default port does not fail the session.
4. Parse the first `http://127.0.0.1:<port>/…` (or `http://127.0.0.1`) from piped stdout. Piped stdout is not a TTY, so the existing `Serving` / `rocdown: serving` lines are uncolored.[^serve-rs][^rocdown-cli]
5. Open that URL beside the editor. Tee CLI output into the existing Rocci output channel.[^vscode-channels]
6. Stop on command, window deactivate, or a new session that is a different app or product.

Dispatch:

| Active file | Command |
| --- | --- |
| `.rocci` | `rocci run <file> --no-window --port auto` |
| `.rocdown` (and ordinary `.md` only if already accepted by `rocdown view`) | `rocdown view <file> --no-window --port auto` |

`rocci run` already refuses `.rocdown` and points at `rocdown view`. `rocdown view` already refuses OKF records. The extension should follow that split, not add a fourth dispatcher.[^rocci-run][^rocdown-cli][^cli-plan]

Untitled or unsaved buffers cannot be previewed honestly; require a saved path. The Dev inspector overlay is host chrome on the Tao/Wry window and will not appear inside Simple Browser. The sibling inspector still starts under `--no-window`; opening it in a second browser tab is a later extra, not v1.[^preview-window][^rocci-cli-readme]

A later CLI `preview_ready <url>` line would make parsing boring. It is hardening, not a prerequisite: the human serve lines already contain the URL.

## Zed feasibility

Not feasible as an in-editor browser in 2026.

The Rocci Zed extension is WASI that returns an LSP `Command`. `zed_extension_api` exposes language servers, language configs, and related headless hooks. It does not expose webviews, editor items, or an HTML preview surface. Zed’s own Markdown preview is a native GPUI renderer, not a webview, and is not available to extensions. Maintainers have repeatedly treated a general webview as a large extra dependency that would push UI into HTML/JS.[^zed-ext][^zed-ext-api][^zed-html-preview][^zed-webview-issue]

Open discussions propose a **read-only preview provider** (Markdown/table/tree first, maybe sandboxed HTML later). That still would not load `http://127.0.0.1` with Datastar and EventSource. A localhost iframe is exactly the “full browser” layer those threads defer.[^zed-preview-api]

What is feasible now:

- Language or project **tasks** that run `rocci run $ZED_FILE` / `rocdown view $ZED_FILE` and open the **native preview window** (best available UX).
- Optional tasks with `--no-window --port 8000` plus a documented `open http://127.0.0.1:8000/` for users who want a system browser.
- README stating that a beside-the-buffer embedded browser waits on a Zed host API.

Do not emulate Simple Browser in WASM. Revisit only if Zed ships an extension-owned webview or a constrained HTML preview that can load a loopback origin.[^zed-tasks][^zed-readme]

## Recommendation

Implement VS Code preview as a thin session host over `--no-window` plus Simple Browser beside the file, with a play/stop command. Treat Zed as a task-plus-native-window fallback until the editor grows a preview surface. Keep product CLIs in charge of compile and serve.[^impl-plan]

[^vscode-client]: Thin client; LSP start and `rocci.restartLspServer` only.
[^vscode-manifest]: Languages, one command, no preview contribution.
[^vscode-channels]: Existing `Rocci` output channel.
[^zed-ext]: WASI LSP command resolution only.
[^zed-manifest]: Server attachment, no preview hooks.
[^zed-readme]: Features are highlighting, diagnostics, icons.
[^language-tooling]: Editors consume standard LSP; they do not own compile.
[^preview-window]: `--no-window` means skip the Tao/Wry preview window.
[^cli-plan]: Three product CLIs; no plugin host.
[^rocci-cli-readme]: `rocci run --no-window` prints a URL and keeps serving.
[^serve-rs]: Shared `--no-window` and `--port auto`; `Serving … at <url>`.
[^rocci-run]: `.rocci` standalone vs Roc app; `.rocdown` hinted to `rocdown view`.
[^rocdown-readme]: `rocdown view` / `--no-window`; site file opens the page route.
[^rocdown-cli]: Site-root detection, `rocdown: serving … at …`, wait when `--no-window`.
[^impl-plan]: Phased VS Code host, then Zed tasks.
[^vscode-webview]: `createWebviewPanel` and `ViewColumn`.
[^vscode-contrib]: `editor/title` and `editor/title/run`.
[^simple-browser]: `simpleBrowser.api.open(uri, { viewColumn, preserveFocus })`.
[^integrated-browser]: Desktop Simple Browser defers to Integrated Browser when present.
[^zed-webview-issue]: Extension webview still open and far from a stable API.
[^zed-html-preview]: Markdown preview is native GPUI; not an extension HTML surface.
[^zed-preview-api]: Proposed preview providers; localhost browser explicitly later.
[^zed-tasks]: Tasks from global, project, or language extension JSON.
[^zed-ext-api]: Current crate has no webview or custom editor type.
