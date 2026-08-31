---
type: Implementation Plan
title: Hosted editor preview chrome and unbundled Rocci tools
description: Turn the editor-preview VS Code webview into a host for the Rocci toolbar and Dev inspector, print inspector_ready from --no-window CLIs, and download rocci, rocdown, and rocci-language-server from GitHub releases instead of shipping them in the extension.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/ui, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-25T11:50:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/shared/editor-preview-host.md
    title: Hosted editor preview research
    author: process:cursor
    last_modified: 2026-08-25
  - id: v1-plan
    resource: editor-preview.md
    title: Editor preview for Rocci and Rocdown
    author: process:cursor
    last_modified: 2026-08-25
  - id: v1-research
    resource: ../../research/shared/editor-preview.md
    title: Editor preview research
    author: process:cursor
    last_modified: 2026-08-25
  - id: preview-window
    resource: ../../decisions/preview-window.md
    title: Preview window naming decision
    author: process:cursor
    last_modified: 2026-08-24
  - id: chrome-research
    resource: ../../research/rocci/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: inspector-plan
    resource: ../rocci/preview-inspector.md
    title: Extended preview-window inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: repair-plan
    resource: ../rocci/preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: language-tooling
    resource: ../../architecture/language-tooling.md
    title: Language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-25
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-24
  - id: preview-nav-html
    resource: ../../../crates/rocci-desktop/assets/preview-nav.html
    title: Preview chrome navigation markup
    author: process:git
    last_modified: 2026-08-22
  - id: preview-nav-js
    resource: ../../../crates/rocci-desktop/assets/preview-nav.js
    title: Preview chrome host script
    author: process:git
    last_modified: 2026-08-22
  - id: history-rs
    resource: ../../../crates/rocci-desktop/src/history.rs
    title: Overlay IPC command vocabulary
    author: process:git
    last_modified: 2026-08-22
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: Shared --no-window serve helpers and preview_ready
    author: process:git
    last_modified: 2026-08-25
  - id: inspector-rs
    resource: ../../../crates/rocci-cli/src/inspector.rs
    title: Inspector panel and sibling InspectorServer
    author: process:git
    last_modified: 2026-08-22
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Same-origin Dev panel URL
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-ext
    resource: ../../../editors/vscode/src/extension.ts
    title: VS Code Rocci language client
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-package-ops
    resource: ../../../rocci-ops/src/rocci_ops/package.py
    title: package vscode copies language-server into the VSIX
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-readme
    resource: ../../../editors/vscode/README.md
    title: VS Code extension README
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-tests
    resource: ../../../editors/vscode/src/test/suite/extension.test.ts
    title: VS Code extension integration tests
    author: process:git
    last_modified: 2026-08-18
  - id: zed-ext
    resource: ../../../editors/zed/src/lib.rs
    title: Zed Rocci language-server adapter
    author: process:git
    last_modified: 2026-08-13
  - id: zed-readme
    resource: ../../../editors/zed/README.md
    title: Zed Rocci extension README
    author: process:git
    last_modified: 2026-08-25
  - id: release-py
    resource: ../../../rocci-ops/src/rocci_ops/release.py
    title: Release archive names and bundled binaries
    author: process:git
    last_modified: 2026-08-21
  - id: release-yml
    resource: ../../../.github/workflows/release.yml
    title: Release workflow target matrix
    author: process:git
    last_modified: 2026-08-23
  - id: hylo-download
    resource: https://github.com/koliyo/hylo-vscode-extension/blob/main/src/download-hylo-lsp.ts
    title: Hylo VS Code LSP downloader
    author: human:nils
    last_modified: 2026-08-25
  - id: hylo-ext
    resource: https://github.com/koliyo/hylo-vscode-extension/blob/main/src/extension.ts
    title: Hylo VS Code activate and update command
    author: human:nils
    last_modified: 2026-08-25
  - id: zed-api
    resource: https://docs.rs/zed_extension_api/latest/zed_extension_api/
    title: zed_extension_api GitHub release and download helpers
    author: organization:zed-industries
    last_modified: 2026-08-25
---

# Hosted editor preview chrome and unbundled Rocci tools

## Goal

Give VS Code the same **preview host** the desktop window already has: Rocci toolbar plus the full Dev inspector, sitting beside the source file, still driven by `rocci run --no-window` / `rocdown view --no-window`. Stop shipping Rocci binaries inside the VS Code VSIX or the Zed WASM. Download `rocci`, `rocdown`, and `rocci-language-server` from [koliyo/rocci](https://github.com/koliyo/rocci) GitHub releases the way [hylo-vscode-extension](https://github.com/koliyo/hylo-vscode-extension) downloads Hylo LSP. Zed keeps the native preview window for toolbar and inspector, and only gains the downloader.[^research][^v1-plan][^desktop-readme][^hylo-download]

This is exploratory. Implement on the existing `editor-preview` branch (or a follow-on named from this stem) rather than restarting v1 on `main`. Phase 0 recorded the decision gates below as normative.

## Out of bound

- Re-rendering Rocci or Rocdown inside the extension, a Custom Editor, or a WASM playground.[^v1-research][^language-tooling]
- Opening the Tao/Wry preview window from VS Code. VS Code stays `--no-window`.[^preview-window]
- Injecting `preview-nav.js` into the product origin, or authoring host chrome as `.rocci`.[^chrome-research][^desktop-readme]
- Repairing inspector scroll, overlay dock-button overlap, OKF index snapshots, or `tok-*` highlighting. Those stay on [preview inspector repair](/plans/rocci/preview-inspector-repair.md).[^repair-plan]
- OKF preview (`rocci-okf view`), `rocci view` gallery, `rocci browse`, `rocci playground`.
- A plugin host or a fourth CLI.[^cli-plan]
- Teaching the language server to compile or serve.[^language-tooling]
- A Zed in-editor webview, toolbar, or inspector pane. Native window already has them.[^v1-research]
- Bundling Chromium, WebView2, or Wry into either extension.
- Desktop-only chrome: wry titlebar drag/zoom, native find-in-page, wry Web Inspector, host-owned Cmd-K when the page did not mount `__rocciGoto`.
- Expanding the GitHub release matrix to Intel macOS, Linux aarch64, or Windows in this plan. Missing targets fail with a named error.
- Publishing the extension to the VS Code Marketplace.
- Hylo's `decompress` / `node-fetch` dependencies.

## Constraints that do not move

- Product CLIs own compile, watch, and HTTP. The editor only spawns, hosts, and stops.[^cli-plan][^language-tooling]
- `.rocci` → `rocci run`. `.rocdown` → `rocdown view`.[^v1-plan]
- `--no-window` remains the VS Code serve mode.[^serve-rs][^preview-window]
- Inspector content stays on the preview HTTP origin (`/__rocci/dev` or sibling `InspectorServer`). Overlay / webview chrome may iframe it; it must not snapshot compiler output.[^inspector-plan][^chrome-research]
- Toolbar is host chrome (VS Code webview parent), not product HTML.
- Binaries come from a user path setting, a debug `target/debug` build, a verified GitHub-release extract, or `PATH`. The packaged extension contains none of `rocci`, `rocdown`, `rocci-language-server`, or `rocci-okf`.[^vscode-package-ops][^hylo-download]
- Downloads use only `https://github.com/koliyo/rocci/releases` assets and the matching `.sha256`.
- Default test suites stay sub-second and offline. No `rocci run` child, no live GitHub download in `npm test`.
- Knowledge records stay inert Markdown.

## Architecture

```text
Rocci: Preview
        │
        ├─ resolve tools (setting / debug / release cache / PATH)
        ├─ save dirty file
        ├─ rocci run FILE --no-window --port auto --verbose
        │     or rocdown view FILE --no-window --port auto --verbose
        ├─ parse preview_ready <url>
        ├─ parse inspector_ready <url>
        └─ webview beside
              ├─ toolbar
              ├─ page iframe      = preview URL
              └─ inspector iframe = inspector URL (Dev on)
```

Binary update (VS Code, Hylo-shaped):[^hylo-download][^hylo-ext][^release-py]

```text
activate (non-debug)
        │
        ├─ GET /repos/koliyo/rocci/releases/latest   (or tags/dev)
        ├─ compare globalStorage manifest
        ├─ if newer: GET rocci-{ver}-{target}.tar.gz + .sha256
        ├─ verify sha256, extract, chmod, write manifest
        └─ start LSP + later preview from that extract
```

Zed: `latest_github_release` + `download_file(..., GzipTar)` inside `language_server_command`. Preview tasks still launch the native window and still need `rocci` / `rocdown` on `PATH` or in settings.[^zed-api][^zed-ext]

### Host chrome contract (VS Code)

Port the desktop **behavior**, not the wry injection:[^preview-nav-html][^preview-nav-js][^history-rs][^desktop-readme]

| Control | VS Code host |
| --- | --- |
| Back / Forward | Page-iframe history (extension-owned stack of iframe URLs) |
| Home | Navigate iframe to the session's first `preview_ready` URL |
| Reload | Existing nonce reload of the page iframe |
| Live reload | Existing SSE / save behavior; toolbar toggle writes `?reload=0` / session flag and pauses SSE |
| Path / title | `postMessage` from a small iframe reporter, or the navigated URL + `document.title` via a script the host cannot inject — prefer URL path from iframe `src` plus CLI serving name |
| More → Reveal | `revealFileInOS` on the current origin's source path when `pages.json` / catalog maps it; hide if unknown |
| More → Copy | Read source via the inspect JSON `source` field or `workspace.fs`; clipboard |
| Dev | Toggle inspector iframe; do not cover the page |
| Dock right / bottom | Same CSS vars and mins as desktop (`28rem` / `36vh`, min `20rem` / `8rem`) |
| Splitter | Pointer drag on the host, persist in `workspaceState` |
| Inspector tuple | `(origin, path, tab, route)`; never assign `iframe.src` for a `view`-only change |
| `postMessage` | Listen for `{type:"rocci-inspector",tab,view}` from the inspector iframe |
| Open as page | Navigate the **page** iframe to the inspector URL; hide the dock while that is the main content |
| Native Web Inspector | Out of bound; optional later `Developer: Open Webview Developer Tools` |

Persist `rocci-dev-panel`, dock side/size, tab, and view in `workspaceState` (not `sessionStorage` in a disposable webview). Seed the inspector iframe query on first open.

### Ready-line contract (CLI)

When stdout is not a TTY, after listen:[^serve-rs]

```text
preview_ready http://127.0.0.1:8000/guide/
inspector_ready http://127.0.0.1:8000/__rocci/dev
```

`rocci run` sibling inspector example:

```text
preview_ready http://127.0.0.1:8000/
inspector_ready http://127.0.0.1:8001/__rocci/dev
```

Keep existing human `Serving` / `rocdown: serving` lines. Do not require a new CLI flag. If no inspector exists, omit `inspector_ready` and keep Dev hidden.

### Tool resolve order

1. `rocci.lsp.serverPath` / `rocci.preview.rocciPath` / `rocci.preview.rocdownPath` when non-empty.
2. Debug / F5: workspace or repo `target/debug`.
3. Verified extract under `globalStorageUri/releases/<tag>/`.
4. `PATH`.

Do not look for a packaged `dist/bin` after Phase 5.

### Release asset map

| `process.platform` + `arch` | Asset stem |
| --- | --- |
| `darwin` + `arm64` | `rocci-{version}-aarch64-apple-darwin.tar.gz` |
| `linux` + `x64` | `rocci-{version}-x86_64-unknown-linux-gnu.tar.gz` |

Anything else: toast + output-channel error listing those two triples. Extracted names: `rocci`, `rocdown`, `rocci-language-server`, `rocci-okf` (okf unused by this plan).[^release-py][^release-yml]

Settings:

| Setting | Default | Meaning |
| --- | --- | --- |
| `rocci.tools.channel` | `stable` | `stable` → `/releases/latest`; `dev` → tag `dev` |
| `rocci.tools.autoUpdate` | `true` | Check on activate when not debugging |

Command: `rocci.updateTools` forces a check and may overwrite a `dev` extract (Hylo `overwriteDev`).[^hylo-ext]

## Phases

### Phase 0 — freeze host and installer contracts

Bound: this record only. Answer the [decision gates](#decision-gates). Confirm toolbar lives in the webview parent, inspector stays HTTP, binaries stay out of the VSIX, and Zed preview stays native.

Exit: Gates 1–7 recorded above. No code.

### Phase 1 — `inspector_ready` (CLI)

Bound: `rocci-cli` shared serve helpers and the Rocdown / OKF call sites that already emit `preview_ready`. Add `inspector_ready_line` / `emit_inspector_ready` next to `preview_ready`. Emit immediately after listen when an inspector URL exists (`DevServer.inspector_url` or `InspectorServer.url`). Keep `InspectorServer` alive for the whole `--no-window` wait (already true). Do not change `--no-window` window semantics. Do not start HTML-capture policy from the repair plan.[^serve-rs][^dev-server][^inspector-rs]

Exit: `cargo test -p rocci-cli` and `cargo test -p rocci-rocdown-cli` cover the piped line and its absence on a TTY. `cargo fmt --all -- --check`.

### Phase 2 — VS Code toolbar host

Bound: `editors/vscode` on the preview webview. Replace the full-size-only iframe HTML with a host document: toolbar + page iframe. Implement back, forward, home, reload, live-reload toggle, path, and title from the contract. Wire live-reload to the existing SSE / save session. Keep play / stop / reload commands. Do not add the inspector iframe yet. Do not load desktop `preview-nav.js`. Offline unit tests for host HTML shape, history stack, and live-reload query flag.[^v1-plan]

Exit: `cd editors/vscode && npm test`. Default suite still offline. Integration suite stays the existing LSP host tests; do not boot `rocci run` there.[^vscode-tests]

### Phase 3 — VS Code inspector dock

Bound: `editors/vscode` plus the Phase 1 parse. Parse `inspector_ready`; show Dev; iframe the inspector URL; dock right/bottom; splitter; persist prefs; tuple compare; `postMessage` tab/view; Open as page. Hide Dev when the line is missing. Offline tests for parse fixtures, dock class names, and “do not assign inspector src on view-only”.[^inspector-plan]

Exit: `cd editors/vscode && npm test`. Manual wry-quality checks are not required; HTTP `curl` of `/__rocci/dev` remains the inspector proof.

### Phase 4 — Release download contract (ops + offline client)

Bound: document the asset names and sha256 check in installer module comments / README draft. Implement **pure** functions in `editors/vscode/src/tools/`: target triple, asset name, manifest equality, sha256 verify against a fixture buffer. No network in tests. Optional: a `rocci-ops` helper that prints the expected asset name for the current host. Do not change `release.yml` matrix. Do not delete VSIX bundling yet.

Exit: `cd editors/vscode && npm test` covers triple map, refuse-unknown-target, manifest compare (Hylo-style id/name/date), and sha256 mismatch. `uv run rocci-ops` tests still pass if ops changed.

### Phase 5 — VS Code installer and unbundled VSIX

Bound: `editors/vscode` + `rocci-ops` `package_vscode`. On non-debug activate, if `autoUpdate`, fetch latest (or `dev`) and install into `globalStorageUri`. `rocci.updateTools` forces it. Resolve LSP and preview binaries with the new order. Remove `dist/bin` copy from `package_vscode`. README: no packaged binaries; first launch needs network once; path settings still win; debug uses `target/debug`. Mock GitHub in unit tests (fixture JSON + fixture archive bytes). Do not hit live GitHub in CI.[^vscode-package-ops][^hylo-download][^vscode-readme]

Exit: `cd editors/vscode && npm test`. `uv run rocci-ops` packaging tests assert the VSIX build is **not** preceded by a language-server copy (or that `dist/bin` is absent). `cargo fmt --all -- --check` if Rust unchanged.

### Phase 6 — Zed GitHub download

Bound: `editors/zed`. In `language_server_command`, after settings / `PATH` / worktree debug miss: `latest_github_release("koliyo/rocci")` (or `github_release_by_tag_name` for a pin), pick the current `zed::current_platform()` archive, `download_file` as `GzipTar`, `make_file_executable` on `rocci-language-server`, `set_language_server_installation_status`. Refuse unknown platform with the same two-triple message. Do not add a webview. README: LSP auto-install; preview tasks still need `rocci` / `rocdown` on `PATH`.[^zed-ext][^zed-api][^zed-readme]

Exit: `uv run rocci-ops check zed`. Unit-test the asset-name function if it can live in a small Rust helper without downloading. WASM build still succeeds.

### Phase 7 — Docs

Bound: `editors/vscode/README.md`, `editors/zed/README.md`, and a one-line root README editor blurb only if that section already lists extension features. Describe toolbar, Dev dock, `inspector_ready`, update command, channels, supported triples, and that Zed preview is still the native window. Point inspector UX defects at the repair plan, not at the extension.

Exit: READMEs match shipped commands and settings. `cd editors/vscode && npm test`. `uv run rocci-ops check zed`.

## Acceptance criteria

- VS Code Preview shows the Rocci toolbar and can open the Dev inspector (Performance / Source / Console as the CLI already serves) without a Tao/Wry window.
- Inspector iframe follows the current page route; Source `view` survives host chrome updates; dock right/bottom insets the page iframe.
- `rocci run` and `rocdown view --no-window` print `inspector_ready` when an inspector exists.
- A VSIX built with `package vscode` contains no Rocci executables. First non-debug activate (or `Rocci: Update tools`) installs from GitHub releases after sha256 verify.
- Path settings and debug `target/debug` still override the download. LSP start keeps using the same client as today, only the resolve path changes.[^vscode-ext]
- Zed still previews in the native window and can start `rocci-language-server` from a downloaded release when PATH/debug miss.
- Offline tests stay offline. No inspector-repair work is claimed done.

## Decision gates

Recorded as normative (recommended answers):

1. **Host chrome implementation:** reimplement the contract in the VS Code webview parent. Do not extract a shared host-kit from `rocci-desktop/assets` in this plan. Extraction stays optional later if the two hosts diverge.
2. **Inspector default:** Dev closed on first preview (matches desktop).
3. **Title/path without injecting into the page:** show the iframe URL path and the CLI serving name. No same-origin reporter script.
4. **More menu:** implement Reveal/Copy in Phase 3 with the Dev dock (the host chrome contract already names both).
5. **Tools channel default:** `stable` (`/releases/latest`). `dev` is an explicit setting.
6. **Storage:** verified extracts under `globalStorageUri/releases/<tag>/`, not `extensionPath/dist`.
7. **Zed preview binaries:** document PATH-only (or settings). Do not teach Zed tasks a downloaded path.

Confirmed: toolbar lives in the webview parent; inspector stays HTTP (`/__rocci/dev` or sibling `InspectorServer`); binaries stay out of the VSIX; Zed preview stays the native window.

## Status

Phases 0–7 implemented on branch `editor-preview-host`. Depends on [editor preview](/plans/shared/editor-preview.md) work already on `main`. Evidence: [hosted editor preview research](/research/shared/editor-preview-host.md). Do not log phases complete until CI and Knowledge succeed.

[^research]: Webview host, inspector_ready, Hylo-style release install.
[^v1-plan]: Play / session / preview_ready / Zed tasks already specified.
[^v1-research]: Hosted origin; Simple Browser cannot grow this chrome.
[^preview-window]: `--no-window` skips the preview window.
[^chrome-research]: Host chrome versus preview-origin inspector.
[^inspector-plan]: Tabs, dock, Source, Console on `/__rocci/dev`.
[^repair-plan]: Do not restart inspector UX repair here.
[^language-tooling]: LSP is analysis, not serve.
[^cli-plan]: Three CLIs; editor is not a multiplexer.
[^desktop-readme]: Toolbar and Dev iframe contract to port.
[^preview-nav-html]: Control set to match.
[^preview-nav-js]: Tuple compare and dock classes.
[^history-rs]: IPC names the host can reuse.
[^serve-rs]: `preview_ready` already exists; InspectorServer lives through `--no-window`.
[^inspector-rs]: Sibling inspector URL to print.
[^dev-server]: Same-origin inspector URL for Rocdown.
[^vscode-ext]: Current resolve still expects bundled or PATH binaries.
[^vscode-package-ops]: Today's VSIX copy of the language server.
[^vscode-readme]: User-facing install and preview docs.
[^vscode-tests]: Keep default suite offline.
[^zed-ext]: Current lookup has no GitHub download.
[^zed-readme]: Native-window preview honesty.
[^release-py]: Archive contents and stem.
[^release-yml]: Two published targets.
[^hylo-download]: Latest-release + manifest + extract pattern.
[^hylo-ext]: Activate check and forced update command.
[^zed-api]: `latest_github_release` and gzip-tar download.
