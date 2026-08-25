---
type: Research Report
title: Hosted editor preview chrome and unbundled Rocci tools
description: The editor-preview branch already serves a loopback origin in a VS Code webview; full preview means that webview becomes the host for toolbar and inspector iframes, while Hylo-style GitHub-release downloads keep rocci binaries out of the extension package.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/ui, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-25T11:30:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: impl-plan
    resource: ../../plans/shared/editor-preview-host.md
    title: Hosted editor preview implementation plan
    author: process:cursor
    last_modified: 2026-08-25
  - id: v1-plan
    resource: ../../plans/shared/editor-preview.md
    title: Editor preview for Rocci and Rocdown
    author: process:cursor
    last_modified: 2026-08-25
  - id: v1-research
    resource: editor-preview.md
    title: Editor preview research
    author: process:cursor
    last_modified: 2026-08-25
  - id: preview-window
    resource: ../../decisions/preview-window.md
    title: Preview window naming decision
    author: process:cursor
    last_modified: 2026-08-24
  - id: chrome-research
    resource: ../rocci/desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: inspector-plan
    resource: ../../plans/rocci/preview-inspector.md
    title: Extended preview-window inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: repair-plan
    resource: ../../plans/rocci/preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: language-tooling
    resource: ../../architecture/language-tooling.md
    title: Language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-25
  - id: cli-plan
    resource: ../../plans/shared/cli-entry-points.md
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
    title: Shared --no-window serve helpers
    author: process:git
    last_modified: 2026-08-25
  - id: inspector-rs
    resource: ../../../crates/rocci-cli/src/inspector.rs
    title: Preview inspector HTTP panel and sibling InspectorServer
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
    resource: ../../../tools/rocci-ops/src/rocci_ops/local.py
    title: package vscode copies release language-server into the VSIX
    author: process:git
    last_modified: 2026-08-25
  - id: vscode-readme
    resource: ../../../editors/vscode/README.md
    title: VS Code extension README
    author: process:git
    last_modified: 2026-08-25
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
    resource: ../../../tools/rocci-ops/src/rocci_ops/release.py
    title: Release archive names and bundled binaries
    author: process:git
    last_modified: 2026-08-21
  - id: release-yml
    resource: ../../../.github/workflows/release.yml
    title: Release workflow target matrix
    author: process:git
    last_modified: 2026-08-23
  - id: branch-session
    resource: ../../../editors/vscode/src/preview/session.ts
    title: editor-preview branch VS Code preview session
    author: process:git
    last_modified: 2026-08-25
  - id: branch-browser
    resource: ../../../editors/vscode/src/preview/browser.ts
    title: editor-preview branch iframe webview HTML
    author: process:git
    last_modified: 2026-08-25
  - id: branch-binaries
    resource: ../../../editors/vscode/src/preview/binaries.ts
    title: editor-preview branch preview binary lookup
    author: process:git
    last_modified: 2026-08-25
  - id: branch-readme
    resource: ../../../editors/vscode/README.md
    title: editor-preview branch VS Code README
    author: process:git
    last_modified: 2026-08-25
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
  - id: hylo-readme
    resource: https://github.com/koliyo/hylo-vscode-extension/blob/main/README.md
    title: Hylo VS Code extension README
    author: human:nils
    last_modified: 2026-08-25
  - id: zed-api
    resource: https://docs.rs/zed_extension_api/latest/zed_extension_api/
    title: zed_extension_api GitHub release and download helpers
    author: organization:zed-industries
    last_modified: 2026-08-25
---

# Hosted editor preview chrome and unbundled Rocci tools

## Claim

A full in-editor Rocci preview is still a **hosted product origin**, not a second renderer. The missing piece on [editor-preview](/plans/shared/editor-preview.md) is a **host** that can show the same toolbar and Dev inspector the Tao/Wry [preview window](/decisions/preview-window.md) already owns. The VS Code webview on `editor-preview` is that host. The inspector stays an HTTP document on the preview origin. Rocci, Rocdown, and `rocci-language-server` stay GitHub-release artifacts, downloaded the way [Hylo's VS Code extension](https://github.com/koliyo/hylo-vscode-extension) downloads `hylo-lsp`, not files inside the VSIX.[^v1-plan][^v1-research][^preview-window][^chrome-research][^hylo-download][^impl-plan]

This record is exploratory evidence for the [implementation plan](/plans/shared/editor-preview-host.md). It does not change shipped editor, CLI, or release behavior.

## What landed on `editor-preview`

Branch `editor-preview` (tip `6768338` when this was written) completed the v1 plan's five phases and then replaced Simple Browser with a custom webview so reload could be driven from the CLI:[^branch-session][^branch-browser][^branch-binaries][^branch-readme][^v1-plan]

| Surface | Behavior on the branch |
| --- | --- |
| Play / stop / reload | `rocci.preview`, `rocci.stopPreview`, `rocci.reloadPreview` |
| Serve | `rocci run FILE --no-window --port auto --verbose` or `rocdown view …` |
| Ready URL | Prefer `preview_ready <url>`, else the first loopback URL |
| Host | `WebviewPanel` in `ViewColumn.Beside` whose HTML is a full-size iframe |
| Reload | SSE `/__rocci/reload/events` for Rocdown; save restarts `rocci run` |
| Binaries | Setting, bundled `dist/bin`, `PATH`, workspace `target/debug` |
| Zed | Tasks that open the native preview window or `--no-window` serve |

`main` still packages `rocci-language-server` into the VSIX and has no preview session.[^vscode-package-ops][^vscode-ext][^vscode-readme]

The branch README is explicit that the Dev inspector overlay stays on the desktop host. That was the v1 out-of-bound line. It is the gap this follow-on closes for VS Code.[^branch-readme][^v1-research]

## Why Simple Browser cannot grow a toolbar

Desktop preview chrome is a host overlay: back, forward, home, reload, live-reload, path, title, More (reveal / copy), and Dev. Dev toggles a second iframe to `PreviewOptions.inspector_url`, docked right or bottom, with splitter, persisted prefs, and `postMessage` `{type:"rocci-inspector",tab,view}`. Compiler output never enters overlay HTML.[^desktop-readme][^preview-nav-html][^preview-nav-js][^chrome-research][^inspector-plan]

VS Code Simple Browser / Integrated Browser is an address-bar browser. It cannot grow Rocci's toolbar, cannot dock a second origin, and cannot persist inspector tuple state the way `preview-nav.js` does. The branch's custom webview is the correct substrate: the parent document is the host; the product origin stays an iframe.[^branch-browser][^v1-research]

Zed still has no extension webview. Native `rocci run` / `rocdown view` already open the preview window with toolbar and inspector. Zed's remaining gap is binary discovery, not chrome.[^zed-readme][^v1-research]

## Host split (do not inject overlay into the product page)

Desktop injects the overlay **into the webview that loads the product URL**. The VS Code host should **not** copy that. The product origin is a cross-origin iframe; host chrome belongs in the parent webview.

```text
VS Code webview (host)
├─ toolbar (back, forward, home, reload, live-reload, path, title, More, Dev)
├─ page iframe     → preview_ready URL
└─ inspector iframe → inspector_ready URL (hidden until Dev is on)
```

| Concern | Owner |
| --- | --- |
| Compile, watch, HTTP, `/__rocci/dev`, inspect JSON, logs | Product CLI |
| Toolbar, dock, history of the page iframe, reveal/copy via `vscode` APIs | Extension webview |
| Tab / view / source pane / console | Inspector document (already shipped) |
| Analysis | `rocci-language-server` |

`preview-nav.js` talks to `window.ipc.postMessage` (`back`, `reload`, `live-reload:0`, `reveal:`, `inspector-prefs:`, `devtools:1`, drag/zoom). A VS Code host maps the same vocabulary onto iframe history and `acquireVsCodeApi().postMessage`. Do not drop `preview-nav.js` into the extension unchanged: it assumes it lives in the product document, uses wry titlebar drag, and opens native Web Inspector.[^history-rs][^preview-nav-js][^desktop-readme]

Native find-in-page, Cmd-K when the page did not mount `__rocciGoto`, and wry Web Inspector do not transfer. Product-origin Goto already works inside the page iframe when the site shipped it. Those desktop-only tools stay out of the first hosted-preview milestone.

## Inspector is already HTTP; `--no-window` hides the URL

`GET /__rocci/dev?tab=&route=&view=` (aliases `/__rocdown/`, `/__rocci_okf/`) is the inspector. Rocdown and OKF serve it same-origin from `DevServer` (`http://127.0.0.1:{port}/__rocci/dev`). `rocci run` uses a sibling `InspectorServer` on another loopback port. Overlay chrome only supplies `inspector_url` and the iframe.[^inspector-rs][^dev-server][^inspector-plan]

`with_window_and_inspector` already spawns `InspectorServer` when an inspect snapshot exists, including `--no-window`, then waits on the Roc child. The sibling URL is not printed. The branch parser only looks for `preview_ready` / a loopback URL, so the VS Code host cannot open Dev.[^serve-rs][^branch-session]

A second non-TTY line `inspector_ready <url>` next to `preview_ready` is the boring contract. Same-origin products can print `{origin}/__rocci/dev`. `rocci run` prints the sibling listen URL. The human `Serving` lines stay.

Inspector quality (scroll, dock-button overlap, OKF index snapshots, highlighting) is owned by [preview inspector repair](/plans/rocci/preview-inspector-repair.md). Hosting a broken pane is still the right split; do not restart those phases here.[^repair-plan]

## Hylo update mechanism, mapped to Rocci

[hylo-vscode-extension](https://github.com/koliyo/hylo-vscode-extension) ships no LSP binary. On non-debug activate it calls `GET https://api.github.com/repos/koliyo/hylo-lsp/releases/latest`, compares `id` / `name` / `published_at` to `dist/manifest.json`, downloads the OS zip plus stdlib, extracts to `dist/bin`, and writes the release JSON back. A local `dev` install is left alone unless the user runs **Hylo: Make sure LSP server is up-to-date**. Debug mode uses a sibling checkout. Windows is documented as unfinished.[^hylo-download][^hylo-ext][^hylo-readme]

Rocci already publishes a closer artifact than Hylo's per-OS zip: one `rocci-{version}-{target}.tar.gz` that contains `rocci`, `rocdown`, `rocci-language-server`, and `rocci-okf`, plus a `.sha256` sibling. The release matrix today is `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu`. Intel macOS, Linux aarch64, and Windows have no asset.[^release-py][^release-yml]

`package vscode` currently `cargo build`s `rocci-rocdown-lsp --release` and copies it into `editors/vscode/dist/bin` before `vsce package`. That is the opposite of the Hylo model and cannot grow `rocci` / `rocdown` without a huge VSIX.[^vscode-package-ops]

Recommended Rocci installer:

| Rule | Choice |
| --- | --- |
| Source | Only `https://github.com/koliyo/rocci/releases` assets |
| Default channel | Latest non-prerelease (`/releases/latest`) |
| Optional channel | Rolling `dev` tag (workflow_dispatch prerelease) |
| Verify | Download `.sha256` and check before extract |
| Store | `ExtensionContext.globalStorageUri` so extension updates do not wipe tools |
| Manifest | `{ tag, id, published_at, target, sha256 }` |
| Resolve | setting path > debug `target/debug` > downloaded extract > `PATH` |
| Missing target | Error that names the supported triples; do not guess |
| Command | `rocci.updateTools` (overwrite, including a local `dev` install) |

Do not take Hylo's `decompress` / `node-fetch` stack. The VS Code engine is already new enough for built-in `fetch` and a small `tar.gz` extract. Do not `chdir` to the extension path.

Zed already has first-party helpers: `latest_github_release`, `download_file` (including `gzip-tar`), `make_file_executable`, and `set_language_server_installation_status`. The adapter today only searches settings, `PATH`, and worktree `target/debug`. Download belongs in `language_server_command`. Zed tasks cannot see the extension work directory, so native preview still expects `rocci` / `rocdown` on `PATH` after install, or the user points settings at the downloaded files.[^zed-ext][^zed-api][^zed-readme]

## Recommendation

1. Keep the branch's webview session. Turn the parent document into host chrome and add an inspector iframe once `inspector_ready` exists.
2. Port the desktop **contract** (controls, dock sides, inspector tuple, IPC names). Do not inject `preview-nav.js` into the product origin.
3. Leave Zed preview on the native window. Download release archives for the language server (and, if cheap, the same four binaries) through `zed_extension_api`.
4. Stop bundling binaries in the VSIX. Install from Rocci GitHub releases with checksum verification, Hylo's latest-vs-manifest check, and an explicit update command. Keep the three product CLIs; the extension still must not become a multiplexer.[^language-tooling][^cli-plan]

[^impl-plan]: Phased CLI ready line, VS Code host, installer, Zed download.
[^v1-plan]: Play / `--no-window` / Zed tasks; inspector overlay out of bound.
[^v1-research]: Hosted origin, not a second renderer; Simple Browser for v1.
[^preview-window]: `--no-window` skips the Tao/Wry preview window.
[^chrome-research]: Overlay HTML versus preview-origin inspector Rocci.
[^inspector-plan]: Tabs, dock, Source, Console live on the inspector origin.
[^repair-plan]: Remaining inspector UX defects; not this host plan.
[^language-tooling]: Editors consume LSP; they do not compile.
[^cli-plan]: Three product CLIs; no editor multiplexer.
[^desktop-readme]: Overlay toolbar, Dev iframe, dock, no compiler HTML in chrome.
[^preview-nav-html]: Back, forward, home, reload, live-reload, More, Dev.
[^preview-nav-js]: wry `ipc`, inspector tuple compare, dock classes.
[^history-rs]: IPC prefixes the VS Code host can reuse.
[^serve-rs]: InspectorServer spawn then `child.wait()` on `--no-window`.
[^inspector-rs]: Panel HTML, sibling server, `postMessage` notify.
[^dev-server]: Same-origin `inspector_url` for static preview.
[^vscode-ext]: main is still LSP-only binary lookup.
[^vscode-package-ops]: VSIX currently embeds `rocci-language-server`.
[^vscode-readme]: main documents a packaged language-server binary.
[^zed-ext]: Settings, PATH, then worktree debug binary.
[^zed-readme]: No bundled server; no in-editor browser.
[^release-py]: `rocci-{version}-{target}.tar.gz` with four binaries.
[^release-yml]: macOS aarch64 and Linux x86_64 only.
[^branch-session]: Webview session, play/stop/reload, no toolbar.
[^branch-browser]: Full-size iframe HTML only.
[^branch-binaries]: Setting / `dist/bin` / PATH / debug.
[^branch-readme]: Inspector overlay documented as desktop-only.
[^hylo-download]: Latest release, manifest compare, zip extract to `dist/bin`.
[^hylo-ext]: Activate downloads unless debug; `hylo.updateLspServer`.
[^hylo-readme]: Extension downloads Hylo LSP; Windows unfinished.
[^zed-api]: `latest_github_release` and `download_file` gzip-tar.
