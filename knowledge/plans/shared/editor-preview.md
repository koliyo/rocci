---
type: Implementation Plan
title: Editor preview for Rocci and Rocdown
description: Add a VS Code play command that serves the active .rocci or .rocdown file with --no-window and opens Simple Browser beside the editor; give Zed tasks that open the native preview window until it grows a webview API.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/ui, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-24T21:30:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/shared/editor-preview.md
    title: Editor preview research
    author: process:cursor
    last_modified: 2026-08-24
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
  - id: vscode-readme
    resource: ../../../editors/vscode/README.md
    title: VS Code extension README
    author: process:git
    last_modified: 2026-08-23
  - id: vscode-tests
    resource: ../../../editors/vscode/src/test/suite/extension.test.ts
    title: VS Code extension integration tests
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
    title: Zed extension README
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
    resource: cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: lsp-plan
    resource: ../rocci/language-server.md
    title: Language-server plan
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
  - id: zed-tasks
    resource: https://zed.dev/docs/tasks
    title: Zed tasks documentation
    author: organization:zed-industries
    last_modified: 2026-08-24
---

# Editor preview for Rocci and Rocdown

## Goal

Let an author hit play on a `.rocci` or `.rocdown` buffer in VS Code and see the running product origin in VS Code’s embedded browser **to the right of that buffer**, using the same CLI serve path as `rocci run --no-window` and `rocdown view --no-window`. Give Zed the best preview it can host today (a task that opens the native preview window), and document the missing in-editor browser.[^research][^rocci-cli-readme][^rocdown-readme][^preview-window]

This is exploratory. Do not start a phase until asked.

## Out of bound

- Re-rendering Rocci/Rocdown to HTML inside the extension, WASM playground, or a Custom Editor that replaces the source tab.[^research][^lsp-plan]
- Opening the Tao/Wry preview window from VS Code (VS Code uses `--no-window` plus Simple Browser).[^preview-window]
- OKF preview (`rocci-okf view`), `rocci view` component gallery, `rocci browse`, `rocci playground`.
- Hosting Datastar, live reload, or the Dev inspector overlay inside extension chrome. Those stay on the product origin / desktop host.
- Previewing unsaved untitled buffers, or injecting editor buffer text into the CLI without writing the file.
- A plugin host or a fourth CLI that multiplexes Rocci, Rocdown, and OKF.[^cli-plan]
- Teaching the language server to compile or serve.[^language-tooling]
- Implementing a Zed webview or GPUI HTML pane. That is a Zed-host feature.
- Bundling Chromium, WebView2, or Wry into either editor extension.

## Constraints that do not move

- Product CLIs own compile, watch, and HTTP. The editor only spawns, shows, and stops.[^cli-plan][^language-tooling]
- `.rocci` → `rocci run`. `.rocdown` → `rocdown view`. Do not send Rocdown through `rocci run`.[^rocci-run][^rocdown-cli]
- `--no-window` is the VS Code serve mode so two preview surfaces do not fight.[^serve-rs][^preview-window]
- Binary lookup for `rocci` / `rocdown` follows the existing LSP pattern: setting, bundled `dist/bin`, `PATH`, workspace `target/debug`.[^vscode-client]
- Default test suites stay sub-second. Do not boot `rocci run` inside `npm test`. Parse, dispatch, and contribution tests stay offline; one optional ignored smoke may spawn a fixture later.
- Knowledge records stay inert Markdown.

## Architecture

VS Code:

```text
Play / Rocci: Preview
        │
        ├─ save dirty file
        ├─ rocci run FILE --no-window --port auto
        │     or rocdown view FILE --no-window --port auto
        ├─ parse first loopback URL on stdout
        └─ simpleBrowser.api.open(url, { viewColumn: Beside, preserveFocus: true })
```

Fallback if `simpleBrowser.api.open` is missing: a `WebviewPanel` in `ViewColumn.Beside` whose HTML is a full-size iframe of that URL. Last resort: `vscode.env.openExternal`.[^simple-browser][^research]

One session per VS Code window in v1. A second play on the same origin navigates Simple Browser; a different product or app root kills and respawns. Stop on `Rocci: Stop Preview` and on deactivate. Pipe CLI stdio to the existing Rocci output channel.

Zed: static tasks, not a WASM browser. `rocci run $ZED_FILE` / `rocdown view $ZED_FILE` open the native preview window. Optional `--no-window` tasks for a system browser are documentation, not a fake editor pane.[^zed-tasks][^research]

## Phases

### Phase 1 — VS Code play command and beside-browser

Bound: `editors/vscode` only. Commands `rocci.preview` and `rocci.stopPreview`. `package.json` contributes both to the Command Palette; `rocci.preview` also to `editor/title` (play/open-preview icon) and `editor/title/run` when `editorLangId == rocci || editorLangId == rocdown`. Session helper: save, spawn `--no-window --port auto`, regex-parse `http://127.0.0.1:<port>` (allow `/` path), open Simple Browser beside, kill the process group on stop. Settings `rocci.preview.rocciPath` and `rocci.preview.rocdownPath` (empty = auto). No CLI crate changes.[^vscode-manifest][^vscode-contrib][^simple-browser][^serve-rs]

Exit: `cd editors/vscode && npm test` still passes. New tests cover command registration, language `when` clauses, URL parse fixtures (`Serving Counter at http://127.0.0.1:8000/` and `rocdown: serving Guide at http://127.0.0.1:8000/guide/`), and dispatch of `.rocci` vs `.rocdown` to argv. Keep the default suite offline (no `rocci run` child). `cargo fmt --all -- --check` if no Rust changed.[^vscode-tests]

### Phase 2 — Session lifecycle

Bound: still `editors/vscode`. Dirty-file save before spawn; refuse untitled. Reuse the process when previewing another file of the same app/site origin (navigate Simple Browser to the new URL or route); restart when the product or app root changes. Status bar item while running. Stop command enabled only with an active session. Deactivate kills the child. Surface CLI failures in the output channel and a toast; do not leave a blank browser. If Simple Browser’s command is missing, use the iframe webview fallback from the research record.[^vscode-client][^research]

Exit: `cd editors/vscode && npm test` with unit tests for reuse vs restart, untitled refusal, and fallback choice. No Roc compile in the default suite.

### Phase 3 — Hardened ready URL (CLI)

Bound: when stdout is not a TTY, both `rocci` and `rocdown` print one additional stable line `preview_ready <url>` (no ANSI) immediately after listen, keeping the existing human `Serving` / `rocdown: serving` lines. VS Code prefers that line, then falls back to the Phase 1 regex. No new flags required for v1; `--port auto` stays. Do not change `--no-window` semantics.[^serve-rs][^rocdown-cli]

Exit: `cargo test -p rocci-cli` and `cargo test -p rocci-rocdown-cli` cover the ready line (pipe / non-TTY). `cargo fmt --all -- --check`. VS Code parse tests accept the new line.

### Phase 4 — VS Code docs and packaging notes

Bound: `editors/vscode/README.md` (play button, commands, binary settings, `--no-window` requirement, inspector overlay not in Simple Browser). Mention preview in the root README editor blurb only if that section already lists extension features. No public Rocdown how-to unless a matching editor page already exists.[^vscode-readme]

Exit: README describes Preview and Stop. `cd editors/vscode && npm test`.

### Phase 5 — Zed tasks and honesty

Bound: `editors/zed`. Add a static `tasks.json` the extension can ship (or document project `.zed/tasks.json` if extension-shipped tasks are not available on the current schema): **Preview Rocci file** → `rocci run $ZED_FILE`; **Preview Rocdown file** → `rocdown view $ZED_FILE`. These open the native preview window. README states there is no beside-buffer embedded browser until Zed provides one; do not add a WASM HTML view. The current Zed manifest stays an LSP adapter. `uv run rocci-ops verify-zed` still passes.[^zed-ext][^zed-manifest][^zed-readme][^zed-tasks][^research]

Exit: `uv run rocci-ops verify-zed`. README names the tasks and the limitation.

## Status

No phase started. Evidence: [editor preview research](/research/shared/editor-preview.md).

[^research]: VS Code Simple Browser host; Zed has no extension webview.
[^vscode-client]: Current client is LSP-only; binary resolution pattern to reuse.
[^vscode-manifest]: Commands and menus land here.
[^vscode-readme]: User-facing preview docs.
[^vscode-tests]: Keep default suite offline.
[^zed-ext]: WASM LSP adapter; no UI surface.
[^zed-manifest]: No preview contribution today.
[^zed-readme]: Document tasks and the gap.
[^language-tooling]: LSP is analysis, not serve.
[^preview-window]: `--no-window` skips the Tao/Wry window.
[^cli-plan]: Three CLIs; editor must not become a product multiplexer.
[^lsp-plan]: Language-server work stays separate.
[^rocci-cli-readme]: `rocci run --no-window` is the Rocci serve path.
[^serve-rs]: `--port auto`, `--no-window`, `Serving … at <url>`.
[^rocci-run]: File-type dispatch already owned by `rocci run`.
[^rocdown-readme]: `rocdown view --no-window`; site file opens the page route.
[^rocdown-cli]: Site-root + `rocdown: serving … at …`.
[^vscode-contrib]: `editor/title` and `editor/title/run`.
[^simple-browser]: `simpleBrowser.api.open` with `viewColumn`.
[^zed-tasks]: Language or project tasks.json.
