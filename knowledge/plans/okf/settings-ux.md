---
type: Implementation Plan
title: Settings UX for knowledge roots
description: Native folder picker in rocci-desktop via rfd, and a more informative rocci-okf /settings/ UI, without live SSE or echoed tokens.
tags: [domain/okf, domain/rocci-okf, concern/tooling, concern/security]
status: draft
generated: { by: process:cursor, at: 2026-08-26T08:05:00Z }
stale_after: 2026-11-25
authority: exploratory
owners: [human:nils]
sources:
  - id: okmate
    resource: okmate.md
    title: Okmate — extractable Rust OKF mate
    author: process:cursor
    last_modified: 2026-08-26
  - id: rust-datastar
    resource: okf-viewer-rust-datastar.md
    title: In-place rocci-okf Askama rewrite (superseded as vehicle)
    author: process:cursor
    last_modified: 2026-08-26
  - id: multi-roots
    resource: multi-knowledge-roots.md
    title: Multiple knowledge roots for rocci-okf
    author: process:cursor
    last_modified: 2026-08-25
  - id: settings-rs
    resource: ../../../crates/rocci-okf/src/settings.rs
    title: Settings HTML and POST actions
    author: process:git
    last_modified: 2026-08-25
  - id: preview-rs
    resource: ../../../crates/rocci-desktop/src/preview.rs
    title: Preview IPC and Tao event loop
    author: process:git
    last_modified: 2026-08-25
  - id: history-rs
    resource: ../../../crates/rocci-desktop/src/history.rs
    title: IpcMessage parser
    author: process:git
    last_modified: 2026-08-25
  - id: rfd
    resource: https://crates.io/crates/rfd
    title: Rusty File Dialogs
    author: organization:crates-io
  - id: tauri-dialog
    resource: https://crates.io/crates/tauri-plugin-dialog
    title: Tauri dialog plugin (rfd backend)
    author: organization:tauri
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable state is server-owned
    author: process:git
    last_modified: 2026-08-17
---

# Settings UX for knowledge roots

## Goal

Make `/settings/` usable: pick a local OKF folder with the same native dialog crate Tauri uses (`rfd` / NSOpenPanel), and show resolved path, health, and plain-language help. Durable state stays `okf.toml`. Mutations stay one-shot POSTs.[^multi-roots][^server-state][^rfd][^tauri-dialog]

## Out of bound

Live SSE or Datastar signals for the registry; echoing tokens; `okf.json` as the registry; `Settings.rocci` as the live renderer; HTTP `pick-folder` / osascript from `extra_http`; changing `check` / `inspect` / `search` defaults; App Sandbox bookmarks; `Location` headers on `ExtraHttpHandler`.

## Constraints that do not move

- Tokens never appear in logs, `roots` JSON, or re-rendered settings HTML. Prefer `token_env`.[^multi-roots]
- Folder pick is a **rocci-desktop** IPC verb on the Tao event loop, using `rfd::AsyncFileDialog`, not AppleScript.[^preview-rs][^rfd][^tauri-dialog]
- `--no-window` has no `window.ipc`; keep a path text field.[^preview-rs]
- GET resolve uses `SyncMode::Never` so the page does not fetch git.[^settings-rs]

## Phases

### Phase 1 — Desktop pick-folder IPC

Bound: `IpcMessage::PickFolder`, `PreviewEvent::PickFolder` / `PickFolderResult`, `rfd::AsyncFileDialog` started from the event loop, `evaluate_script` `CustomEvent("rocci-pick-folder")`. Unit test parse and JSON-escaped path script.[^history-rs][^preview-rs]

Exit: `cargo test -p rocci-desktop` and `cargo fmt --all -- --check`.

### Phase 2 — Settings browse and loopback

Bound: Choose folder button when `window.ipc` exists; fill path and suggest id; `/__rocci_okf/settings` loopback-only; `history.replaceState` to `/settings/`.[^settings-rs]

Exit: `cargo test -p rocci-okf` settings tests.

### Phase 3 — Cards, copy, git forms

Bound: root cards (resolved path, last fetch/error, revision); empty state; incoming labels; git field help; `index.md` warning; CSS and READMEs.

Exit: `cargo test -p rocci-desktop -p rocci-okf` and `cargo fmt --all -- --check`.

## Status

Exploratory; implement with the settings UX Cursor plan. Folder-pick and
cards stay this plan for `rocci-okf` if it is still patched. The
extractable app is [okmate](okmate.md) (Askama + official Datastar,
`#okmate-settings`); live SSE for the registry stays out. Do not start
[rust+datastar](okf-viewer-rust-datastar.md) in place.[^okmate][^rust-datastar]

[^okmate]: Settings UI is rewritten in okmate with Askama and official Datastar; this plan’s rfd/copy still apply.
[^rust-datastar]: Superseded in-place vehicle; do not start.
[^multi-roots]: Parent plan: registry, POST not SSE, token redaction, `/settings/` chrome.
[^settings-rs]: Live markup is Rust HTML in `settings.rs`.
[^preview-rs]: wry IPC hops to the Tao loop via `EventLoopProxy`; `Evaluate` runs `evaluate_script`.
[^history-rs]: Prefix IPC verbs (`reveal:`, `home`) live in `IpcMessage::parse`.
[^rfd]: Cross-platform native dialogs; macOS is NSOpenPanel via objc2.
[^tauri-dialog]: `tauri-plugin-dialog` depends on `rfd` and `AsyncFileDialog`.
[^server-state]: Durable settings are not a browser store.
