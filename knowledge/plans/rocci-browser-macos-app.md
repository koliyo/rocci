---
type: Implementation Plan
title: rocci-browser macOS app and TUI removal
description: "Follow-on after rocci-browser Phases 1–5: delete the terminal picker, keep graphical preview() plus headless open, and assemble an ad-hoc Finder-launchable Rocci Browser.app. Does not reuse rocci bundle, embed product adapters, or notarize."
tags: [domain/rocci, domain/desktop, concern/architecture, concern/tooling, concern/packaging, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T07:50:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/rocci-browser-macos-app.md
    title: Native macOS app for rocci-browser; drop the TUI
    author: process:cursor
    last_modified: 2026-08-20
  - id: browser-plan
    resource: rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: original-research
    resource: ../research/rocci-browser.md
    title: Dedicated rocci-browser CLI and desktop host research
    author: process:cursor
    last_modified: 2026-08-19
  - id: folder-plan
    resource: browser-folder-dialogs-and-plugins.md
    title: Native folder dialogs and a later adapter plugin index
    author: process:cursor
    last_modified: 2026-08-19
  - id: run-no-window
    resource: browser-run-no-window.md
    title: Product run skips preview when rocci-browser is already open
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: tui-rs
    resource: ../../crates/rocci-browser/src/main.rs
    title: rocci-browser clap surface after TUI removal
    author: process:cursor
    last_modified: 2026-08-20
  - id: browser-main
    resource: ../../crates/rocci-browser/src/main.rs
    title: rocci-browser clap surface including tui
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-cargo
    resource: ../../crates/rocci-browser/Cargo.toml
    title: rocci-browser package dependencies
    author: process:cursor
    last_modified: 2026-08-19
  - id: window-rs
    resource: ../../crates/rocci-browser/src/window.rs
    title: Graphical host preview loop
    author: process:cursor
    last_modified: 2026-08-19
  - id: picker-rs
    resource: ../../crates/rocci-browser/src/picker.rs
    title: Two-stage picker state machine and unit tests
    author: process:cursor
    last_modified: 2026-08-19
  - id: paths-rs
    resource: ../../crates/rocci-browser/src/paths.rs
    title: Browser directory and cwd-based repo-local file
    author: process:cursor
    last_modified: 2026-08-19
  - id: discovery-rs
    resource: ../../crates/rocci-browser/src/discovery.rs
    title: Plugin discovery and PATH bin resolution
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-readme
    resource: ../../crates/rocci-browser/README.md
    title: rocci-browser crate contract
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-guide
    resource: ../../docs/guides/rocci-browser.rocdown
    title: Public project-browser guide
    author: process:cursor
    last_modified: 2026-08-19
  - id: bundle-rs
    resource: ../../crates/rocci-cli/src/bundle.rs
    title: rocci bundle macOS .app assembly for Roc apps
    author: process:git
    last_modified: 2026-08-17
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
    last_modified: 2026-08-19
  - id: preview-rs
    resource: ../../crates/rocci-desktop/src/preview.rs
    title: Persistent preview() event loop
    author: process:git
    last_modified: 2026-08-19
  - id: desktop-lib
    resource: ../../crates/rocci-desktop/src/lib.rs
    title: Persistent multi-window run() shell
    author: process:git
    last_modified: 2026-08-19
  - id: icon-rs
    resource: ../../crates/rocci-desktop/src/icon.rs
    title: Runtime Dock icon from embedded PNG
    author: process:git
    last_modified: 2026-08-19
  - id: deps-check
    resource: ../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Mechanical one-way workspace dependency check
    author: process:cursor
    last_modified: 2026-08-19
  - id: known-limitations
    resource: ../status/known-limitations.md
    title: Known Rocci limitations
    author: process:cursor
    last_modified: 2026-08-19
  - id: root-readme
    resource: ../../README.md
    title: Rocci workspace overview
    author: human:nils
    last_modified: 2026-08-19
  - id: cli-docs
    resource: ../../docs/reference/cli.rocdown
    title: Public CLI reference
    author: process:cursor
    last_modified: 2026-08-19
  - id: agents
    resource: ../../AGENTS.md
    title: Workspace ownership and CLASSES rule
    author: process:git
    last_modified: 2026-08-19
---

# rocci-browser macOS app and TUI removal

## Purpose and authority

This is the implementation plan for the [macOS app
research](../research/rocci-browser-macos-app.md). It follows rocci-browser
Phases 1–5. TUI removal is accepted with this record. Assembling a Finder
`.app` waits on research gate 1 (ad-hoc `Rocci Browser.app`). Production
signing is not in these phases.[^research][^browser-plan]

Do not start a phase until the user asks. Do not implement Phase 3 until gate 1
is accepted.

This plan does **not** reverse option D, does not add plugins on `rocci` or
`rocdown`, and does not teach `rocci bundle` to wrap this host.[^original-research][^bundle-rs][^deps-check]

## Goal

1. Delete the terminal picker so rocci-browser is a graphical host plus a
   headless CLI.
2. Make Dock / Finder launch work: repaired PATH, no bogus repo-local file from
   cwd `/`, optional last-root.
3. Assemble an ad-hoc **Rocci Browser.app** around the existing `preview()`
   binary.
4. Document that `rocci bundle` still packages Roc apps, not this host.

## Frozen architecture

| Choice | Freeze |
| --- | --- |
| Window shell | Keep `rocci_desktop::preview`. Do not migrate to `run()`.[^window-rs][^preview-rs][^desktop-lib] |
| Picker | Cmd-P overlay and Open Target only. Library `Picker` stays for overlay IPC and unit tests. No TUI.[^picker-rs][^tui-rs] |
| CLI | `add` / `remove` / `list` / `open`. Drop `tui`. `open --no-window --json` remains the agent form.[^browser-main] |
| Packager | Browser-owned layout + ad-hoc codesign. Not `rocci bundle`, not cargo-packager.[^bundle-rs][^deps-check] |
| Adapters | Stay out of process on PATH or absolute plugin `bin`. Do not copy product CLIs into the `.app`.[^discovery-rs][^original-research] |
| Identity | Display name `Rocci Browser`; executable `rocci-browser`; identifier `dev.rocci.browser`. Native window still named preview window.[^preview-decision] |
| Signing | `codesign --force --deep --sign -` only. Notarization is later.[^bundle-rs][^known-limitations] |

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Option D | Product-blind host; no Rocdown/OKF package edge.[^deps-check] |
| Three product CLIs | Direct `run` still opens one-shot preview until gate 3.[^run-no-window] |
| Preview window name | Tao/Wry shell is not renamed "browser window".[^preview-decision] |
| Cmd-P vs Cmd-K | Host picker vs in-page `goto.js`. |
| CLASSES | Packaging code stays in `rocci-browser` or `rocci-desktop`, never `rocci-cli`.[^agents][^deps-check] |
| No ancestor walk | Do not search parent directories for `.rocci/browser.toml`.[^folder-plan][^paths-rs] |
| `cargo test -p rocci-browser` | No product CLIs, no display, no `codesign` on Linux CI. |

## Non-goals (all phases)

- Production signing, notarization, stapling, Sparkle, or Developer ID.
- Windows or Linux installers.
- Copying `rocci` / `rocdown` / `rocci-okf` into `Contents/MacOS`.
- Replacing `preview()` with `rocci_desktop::run`.
- Native folder dialogs (gate 5) or live-session lock (gate 3).
- Authoring picker UI in Rocci (gate 4).
- A `rocci browser` subcommand.
- Encoding `site` / `docs` / `knowledge` in host source.
- Changing adapter protocol version.

## Naming

| Surface | Name |
| --- | --- |
| Finder / Dock | Rocci Browser |
| Bundle directory | `Rocci Browser.app` |
| Executable | `rocci-browser` |
| Identifier | `dev.rocci.browser` |
| Native window | preview window |
| Output path | `target/release/bundle/macos/Rocci Browser.app` |

Avoid: wrapping this host with `rocci bundle`, calling the app `rocci browse`.

## Delivery phases

### 0. Freeze the app contract

This record is the freeze:

- TUI is withdrawn.
- Graphical host stays `preview()`.
- `.app` is a wrapper around that binary, ad-hoc signed.
- Adapters are not bundled.
- GUI launch repairs PATH and ignores cwd `/` for repo-local files.

**Exit:** A reviewer treats this plan as the owner for TUI removal and the
ad-hoc Mac app, including research gate 1 for Phase 3.

### 1. Remove the TUI

Bound: crate CLI and docs that name `tui`. Out of bound: packaging, PATH,
window shell.

- Delete `crates/rocci-browser/src/tui.rs`.
- Remove `mod tui`, `Commands::Tui`, and the `tui` match arm.
- Drop the `crossterm` dependency.[^browser-cargo]
- Keep `Picker` / `PickerAction` tests. Host tests still use `--document`, not a
  terminal.
- Update crate README. Public docs do not currently list `tui`; do not add
  it.[^browser-readme][^browser-guide][^cli-docs][^root-readme]

**Exit:** `rocci-browser --help` has no `tui`. `cargo test -p rocci-browser`
and `cargo fmt --all -- --check` pass. `crossterm` is gone from
`Cargo.toml` / `Cargo.lock`.

### 2. Bundled-launch environment

Bound: `Paths`, discovery PATH, optional last-root file. Out of bound: `.app`
layout, codesign, folder dialogs.

Detect bundled launch when `current_exe()` sits under `*.app/Contents/MacOS/`.

- If bundled, or cwd is `/`, do not read `.rocci/browser.toml` from cwd.
- `--root` still sets the repo-local file.
- On graphical quit after a successful session that had a real repo root, write
  `last-root` under `browser_dir`. Bundled launch with no `--root` restores it
  when that path still exists.
- When PATH equals the macOS GUI default (only system dirs), prepend
  `/opt/homebrew/bin`, `/usr/local/bin`, `$HOME/.local/bin`, `$HOME/.cargo/bin`
  if those directories exist. Do not name product binaries. Do not use
  `LSEnvironment` in the plist.
- Terminal `cargo run` with a project cwd is unchanged.

Tests: temp exe layout that looks like a bundle; cwd `/` skipped; `--root`
wins; PATH prepend is unit-tested with a fake env; no display.

**Exit:** A simulated bundled `Paths` uses user `projects.json` plus last-root,
not `/`. Fixture plugin resolution still works with an absolute `bin`.

### 3. Assemble `Rocci Browser.app`

Bound: packaging command/script, plist, icns, ad-hoc codesign. Out of bound:
notarization, embedding adapters, `rocci-cli` changes.

Requires research gate 1.

- Add a macOS-only `package` (or `bundle-macos`) command on `rocci-browser`
  that copies a given executable (default: current release binary) into
  `target/release/bundle/macos/Rocci Browser.app`, writes `Info.plist` /
  `PkgInfo`, installs `AppIcon.icns` generated from
  `rocci-desktop`'s 1024 PNG (`iconutil` on Darwin), and ad-hoc codesigns.[^icon-rs][^desktop-readme]
- Maintainer script `uv run rocci-ops bundle browser-macos`: Darwin-only; `cargo
  build --release -p rocci-browser`; invoke the package command. Mirror
  `uv run rocci-ops bundle macos` without calling `rocci bundle`.
- Plist keys: `CFBundleDisplayName` Rocci Browser, `CFBundleExecutable`
  `rocci-browser`, `CFBundleIdentifier` `dev.rocci.browser`,
  `CFBundleIconFile`, `LSMinimumSystemVersion` 12.0, `NSHighResolutionCapable`.
  No `LSUIElement`. No `LSEnvironment`.
- Tests: generated plist strings and directory layout in a temp dir **without**
  calling `codesign` (Linux CI). Skip or `#[cfg(target_os = "macos")]` the
  codesign invocation. `package` on non-macOS returns a clear error.

Do not extract helpers into `rocci-desktop` in this phase unless duplication
with `rocci-cli` bundle.rs is already being touched. Prefer copy-small over a
new shared packager.

**Exit:** On a Mac, the script produces an `.app` that opens the picker window
without Cargo. `cargo test -p rocci-browser` stays green on Linux.

### 4. Public contract

Bound: crate README, root README, `docs/guides/rocci-browser.rocdown`, CLI
reference, parent browser plan citations already pointing here.

- Document: build/open the `.app`; adapters still on PATH or plugin absolute
  bins; GUI PATH repair; last-root; TUI removed; `open --no-window` for agents
  and for machines without a display; `rocci bundle` is unrelated.
- Mark production signing **planned** / known limitation, not shipped.
- Point gate 3 wording at graphical preview / `.app` only (no TUI lock).

**Exit:** Public docs match the freeze. No remaining `rocci-browser tui`
examples in crate or docs.

## Later, gated work

Not in Phases 0–4:

| Gate | Work |
| --- | --- |
| research 2 | Notarization, stapling, updates |
| research 3 | Bundling product CLIs (rejected unless a later decision reverses it) |
| research 4 | `preview()` → `run()` |
| browser 3 | Live-session lock so product `run` skips a second window |
| browser 5 | Native folder dialog (especially useful after Dock launch) |
| — | Windows / Linux installers |

## Acceptance criteria (through Phase 4)

- No `tui` command and no `crossterm` in `rocci-browser`.
- `Picker` unit tests still cover Enter / Tab.
- No-args and the `.app` both open one preview window with Cmd-P.
- Host source still does not name product formats or copy those binaries into
  the bundle.
- `rocci bundle` still requires `main.roc` and still packages Roc apps.
- `cargo test -p rocci-browser` does not codesign, open a display, or start
  product CLIs.
- Public docs state TUI is gone and the `.app` is ad-hoc, not notarized.

## Decision gates

1. Add a Finder-visible `Rocci Browser.app` (ad-hoc, identifier
   `dev.rocci.browser`). Required before Phase 3.
2. Production signing / notarization.
3. Embed product adapters in the bundle (default: no).
4. Replace `preview()` with `run()` for this host.

TUI removal (Phase 1) is accepted. Phase 2 may proceed without gate 1.

## Status

Exploratory; Phase 0 freeze plus Phases 1–4 in this revision. Gate 1 accepted
with the request to assemble the ad-hoc app. Production notarization stays
later. Not CI-complete.

[^research]: Recommended split: delete TUI; wrap preview() in an ad-hoc .app; repair GUI PATH and Finder cwd.
[^browser-plan]: Original later row for signed .app; Phase 1 shipped tui.
[^original-research]: Option D; do not rocci-bundle a gallery.
[^folder-plan]: No ancestor walk; folder dialogs are a separate gate.
[^run-no-window]: Lock file is for graphical preview(), not CLI open.
[^preview-decision]: Keep the preview window name.
[^tui-rs]: Terminal picker to delete.
[^browser-main]: clap fronts including tui and no-args window.
[^browser-cargo]: crossterm dependency to drop.
[^window-rs]: preview() already long-lived.
[^picker-rs]: Headless Enter/Tab tests.
[^paths-rs]: cwd-based repo-local file.
[^discovery-rs]: PATH plugin bins.
[^browser-readme]: Current tui example and cargo-run v1.
[^browser-guide]: Planned .app; rocci bundle is a different product.
[^bundle-rs]: Roc app bundle layout rocci-browser must not reuse as-is.
[^desktop-readme]: Dock PNG at runtime; packaging runtime for Roc apps.
[^preview-rs]: Event loop to keep.
[^desktop-lib]: run() is the multi-window shell not used here.
[^icon-rs]: PNG source for icns and runtime Dock tile.
[^deps-check]: No rocci-browser → rocci-cli edge.
[^known-limitations]: Production signing absent.
[^root-readme]: Project browser; .app planned.
[^cli-docs]: Fourth binary; .app planned.
[^agents]: Browser host behavior lives in crates/rocci-browser.
