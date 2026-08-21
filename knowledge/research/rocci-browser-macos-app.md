---
type: Research Report
title: Native macOS app for rocci-browser; drop the TUI
description: "Exploratory research for a Finder-launchable ad-hoc macOS .app around the shipped rocci-browser preview window, and for deleting the terminal picker. Complements cargo run and headless open --no-window. Does not reuse rocci bundle of a Roc app, embed product adapters, or add notarization."
tags: [domain/rocci, domain/desktop, concern/architecture, concern/tooling, concern/packaging, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T05:20:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: original-research
    resource: rocci-browser.md
    title: Dedicated rocci-browser CLI and desktop host research
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-plan
    resource: ../plans/rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: macos-plan
    resource: ../plans/rocci-browser-macos-app.md
    title: rocci-browser macOS app and TUI removal plan
    author: process:cursor
    last_modified: 2026-08-20
  - id: folder-plan
    resource: ../plans/browser-folder-dialogs-and-plugins.md
    title: Native folder dialogs and a later adapter plugin index
    author: process:cursor
    last_modified: 2026-08-19
  - id: run-no-window
    resource: ../plans/browser-run-no-window.md
    title: Product run skips preview when rocci-browser is already open
    author: process:cursor
    last_modified: 2026-08-19
  - id: picker-rocci
    resource: ../plans/browser-picker-in-rocci.md
    title: Author the rocci-browser picker as a host-owned Rocci origin
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: chrome-research
    resource: desktop-host-chrome-and-inspector-ui.md
    title: Desktop host chrome versus Rocci inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: tui-rs
    resource: ../../crates/rocci-browser/src/tui.rs
    title: crossterm two-stage terminal picker
    author: process:cursor
    last_modified: 2026-08-19
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
  - id: overlay-rs
    resource: ../../crates/rocci-browser/src/overlay.rs
    title: Cmd-P picker overlay embedding
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
  - id: run-bundled
    resource: ../../crates/rocci-cli/src/run.rs
    title: Bundled Roc app launch from Contents/Resources
    author: process:git
    last_modified: 2026-08-19
  - id: bundle-script
    resource: ../../tools/rocci-ops/src/rocci_ops/local.py
    title: Maintainer rocci-ops bundle macos command
    author: process:git
    last_modified: 2026-08-13
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
  - id: menu-rs
    resource: ../../crates/rocci-desktop/src/menu.rs
    title: Native menus including Open Target
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
  - id: roadmap
    resource: ../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-17
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
---

# Native macOS app for rocci-browser; drop the TUI

## Scope and authority

This record is exploratory. It does not approve production signing, notarization,
Windows or Linux installers, or embedding `rocci` / `rocdown` / `rocci-okf`
inside an `.app`. It asks how to make **rocci-browser** a Finder-launchable
macOS application around the window that already exists, and records that the
terminal picker should leave the product.[^original-research][^browser-plan][^macos-plan]

The owner request that TUI support does not belong in rocci-browser is treated
as an accepted product constraint for the [implementation
plan](../plans/rocci-browser-macos-app.md). Ad-hoc `.app` assembly is the
delivery that still needs a freeze; notarization stays a later known
limitation.[^known-limitations][^roadmap]

It does not reverse option D (product-blind host plus out-of-process adapters),
the preview-window name, or the rejection of `rocci bundle` as the way to
package this host.[^original-research][^preview-decision][^browser-guide]

## Job to be done

Authors should launch **Rocci Browser** from the Dock or Finder, get the same
persistent preview window and Cmd-P picker they already get from
`cargo run -p rocci-browser`, and keep using `add` / `list` / `open --no-window`
from a terminal or agent. They should not need a crossterm TUI, a Roc `main.roc`,
or a second windowing stack.[^window-rs][^browser-main][^tui-rs][^browser-readme]

## Established baseline

### The graphical host already is a native window

No-argument `rocci-browser` calls `rocci_desktop::preview` with a host launcher
origin, picker overlay, session table, and `picker: true` so the View menu has
**Open Target**. That loop stays until quit, `load_url`s adapter origins, and
applies the embedded PNG as the macOS Dock tile at runtime. Public docs already
call a signed `.app` **planned** and tell people to `cargo run`.[^window-rs][^preview-rs][^menu-rs][^icon-rs][^browser-guide][^root-readme][^cli-docs]

`rocci_desktop::run` is a different shell: multi-window, `RunningBackend`,
authored `rocci.toml` windows. The original browser research guessed a later
`.app` would use `run()`. Phase 2 extended `preview()` so it no longer returns
when the first URL is shown. **The Mac app does not need to switch shells.**
`run()` remains extra product scope (one native window per target).[^desktop-lib][^browser-plan][^original-research]

### `rocci bundle` packages a different product

`rocci bundle` compiles `main.roc`, copies the `rocci` host into
`Contents/MacOS`, writes `Contents/Resources/app/server` plus `rocci.toml`,
and ad-hoc `codesign`s. Opening that `.app` starts the bundled Roc server and
a one-shot preview. The layout detector looks for `rocci.toml` and
`app/server`.[^bundle-rs][^run-bundled][^bundle-script][^desktop-readme]

rocci-browser has no `main.roc`, must not depend on `rocci-cli`, and must not
name product adapters in host source. Reusing that packager would force a fake
Roc app or a forbidden package edge.[^deps-check][^browser-cargo][^original-research]

### TUI is a leftover Phase 1 front

Phase 1 shipped `rocci-browser tui` (crossterm raw mode, Enter / Tab /
Shift-Tab) so the two-stage picker could be exercised before the overlay
existed. The same `Picker` state machine is unit-tested without a terminal.
The overlay plus `open --document --no-window` cover Tab-then-Enter for
agents. The TUI is the only `crossterm` user in the crate.[^tui-rs][^browser-main][^picker-rs][^overlay-rs][^browser-cargo][^browser-plan][^chrome-research]

`tui` without `--no-window` does not open the preview window; it prints a hint
to run with no args. So the TUI is not a graphical fallback. It is a second
picker UI that the Mac app would duplicate.[^browser-main]

### Launch environment is terminal-shaped

`Paths::from_env` uses `current_dir()` as the repo that may contain
`.rocci/browser.toml`. Plugin `bin` names without a slash resolve on `PATH`.
Repo-local rows with a slash resolve against that repo. Finder-launched macOS
apps typically start with cwd `/` and a sanitized PATH
(`/usr/bin:/bin:/usr/sbin:/sbin`), so today's discovery would miss both the
workspace file and Homebrew / `~/.local/bin` adapters.[^paths-rs][^discovery-rs][^browser-readme]

Runtime Dock identity is the PNG via `NSApplication.setApplicationIconImage`.
Finder, Spotlight, and a not-yet-running Dock tile use `CFBundleIconFile`.
`rocci bundle`'s generated plist has no icon key and no `.icns`.[^icon-rs][^bundle-rs][^desktop-readme]

## Why the TUI should go

1. **The product is a desktop host.** Cmd-P overlay and Open Target are the
   picker. A terminal UI is a third surface with different keys, no preview
   chrome, and no session reuse in the same process.[^overlay-rs][^window-rs][^tui-rs]
2. **Headless work already has a contract.** `open --no-window --json` prints
   `{ url, title }` and keeps the origin. Registry CRUD stays. Agents do not
   need raw-mode stdin.[^browser-readme][^browser-plan]
3. **Tests do not need it.** `Picker` unit tests cover Enter / Tab. Host tests
   use the fixture adapter. Phase 1's "Tab-then-Enter via tui or `--document`"
   exit is already satisfied by `--document`.[^picker-rs][^browser-plan]
4. **A `.app` makes it worse.** Double-clicking a bundle should not attach a
   terminal. Keeping `tui` invites a "use the TUI when there is no display"
   fork that this host is not staffed to own.

Keep `Picker` in the library. Delete `src/tui.rs`, the `tui` subcommand, and
`crossterm`. After removal, no-args with no display fails at `preview()`;
document that agents must pass `open --no-window`.

## What "native Mac app" means here

Not a Swift/AppKit rewrite, not Tauri, not `rocci bundle` of a gallery. A
standard bundle:

```text
Rocci Browser.app/
  Contents/
    Info.plist
    PkgInfo
    MacOS/rocci-browser
    Resources/AppIcon.icns
```

Opening it is the same as no-args `rocci-browser`: one preview window, host
launcher, Cmd-P. `CFBundleDisplayName` is **Rocci Browser**;
`CFBundleExecutable` is `rocci-browser`; `CFBundleIdentifier` is
`dev.rocci.browser`. Signature is ad-hoc (`codesign --sign -`), the same local
bar as `rocci bundle`. Production notarization stays out.[^bundle-rs][^roadmap][^known-limitations][^preview-decision]

`cargo run -p rocci-browser` remains the workspace dev loop. The `.app` is how
a Mac user launches the same binary without Cargo.

## Options

### Packaging

| Option | Disposition |
| --- | --- |
| A. Teach `rocci bundle` to wrap rocci-browser | **Reject.** Needs `main.roc` / `app/server`, lives in `rocci-cli`, and would make the Rocci CLI the multiplexer the browser was created to avoid.[^bundle-rs][^deps-check][^original-research] |
| B. cargo-bundle / cargo-packager / Tauri | **Reject for v1.** Extra toolchain; the workspace already knows plist + copy + ad-hoc codesign.[^bundle-rs] |
| C. Switch the host to `rocci_desktop::run` then bundle that | **Defer.** `preview()` already owns the long-lived window the app needs. `run()` is multi-window plus `RunningBackend`.[^preview-rs][^desktop-lib] |
| D. Browser-owned macOS layout plus ad-hoc codesign | **Recommend.** Logic in `rocci-browser` (plist, copy, icon, codesign) with a maintainer script that `cargo build --release` then invokes it. Optional later extract of plist/codesign helpers into `rocci-desktop` if `rocci bundle` wants to share them. No `rocci-cli` dependency.[^deps-check][^bundle-script] |

### Adapters inside the `.app`

| Option | Disposition |
| --- | --- |
| Copy `rocci`, `rocdown`, `rocci-okf` into `Contents/MacOS` | **Reject.** The host would ship named products and a three-binary rebuild matrix. Option D forbids encoding those names in host source.[^original-research][^discovery-rs] |
| PATH-only plugins, same as today | **Recommend**, with GUI PATH repair (below). User `plugins/*.toml` may still use absolute `bin` paths. |
| An in-app plugin folder the host always scans | **Defer.** Discovery already loads `~/.rocci/browser/plugins/*.toml`. Do not add a second, bundle-relative plugin root until a real gap shows. |

### Finder cwd and repo-local `browser.toml`

| Option | Disposition |
| --- | --- |
| Walk ancestors from cwd `/` for `.rocci/browser.toml` | **Reject.** The folder-dialogs plan already forbids ancestor walks.[^folder-plan][^paths-rs] |
| Ignore cwd when it is `/` or when `current_exe` is inside `.app/Contents/MacOS` | **Recommend** as the bundled-launch rule. Use the user registry. `--root` still selects a repo file. |
| Persist last successful `--root` / opened repo under `browser_dir` | **Recommend** for Dock relaunch. CLI `--root` wins. |
| Native Open Folder on first launch | **Defer** to gate 5. Complementary, not required to ship an `.app`.[^folder-plan] |

### GUI PATH

When PATH is the sanitized GUI default, prepend well-known *user bin* directories
(`/opt/homebrew/bin`, `/usr/local/bin`, `$HOME/.local/bin`, `$HOME/.cargo/bin`)
without naming product CLIs. Do not bake user-specific PATH into `LSEnvironment`
in the plist (machine-local, hard to test). Absolute plugin `bin` paths keep
working unchanged.[^discovery-rs]

## Recommended product split

| Surface | Behavior |
| --- | --- |
| `Rocci Browser.app` (Finder / Dock) | Same as no-args graphical host; repaired PATH; no repo-local file unless last-root or `--root` |
| `rocci-browser` (no args, terminal) | Unchanged preview window; cwd may supply `.rocci/browser.toml` |
| `add` / `remove` / `list` / `open` | Registry and headless open; `open --no-window --json` for agents |
| `tui` | **Delete** |
| `rocci bundle` | Still packages authored Roc apps, not this host |

Native window name stays **preview window**. App name is **Rocci Browser**.
Bundle identifier `dev.rocci.browser`. Do not call it Studio, Hub, or
`rocci browse`.[^preview-decision][^original-research]

## Relationship to other browser work

| Track | Interaction |
| --- | --- |
| Gate 3 live-session lock | Graphical `.app` and no-args `preview()` should write the lock; CLI `open --no-window` should not. Drop TUI from that plan's wording.[^run-no-window] |
| Gate 4 picker as Rocci origin | Unchanged. Overlay vs launcher origin is independent of the bundle wrapper.[^picker-rocci] |
| Gate 5 folder dialogs | More important after Dock launch (no typed cwd). Sequence after or beside this plan; do not block the `.app` on `rfd`.[^folder-plan] |
| Gate 6 builtin targets | Still forbidden. Last-root is a path the user already opened, not `site` / `docs` / `knowledge` in Rust.[^browser-plan] |

## Decision gates

Human approval is required before:

1. Shipping a Finder-visible `Rocci Browser.app` (ad-hoc codesign, new
   identifier `dev.rocci.browser`).
2. Production signing, notarization, or Sparkle-style updates.
3. Copying product CLIs into the bundle.
4. Replacing `preview()` with `rocci_desktop::run` for this host.

TUI removal is **not** gated. The owner request plus the overlay and
`--document` coverage is enough to delete it in the first implementation
phase.[^macos-plan]

Until gate 1 opens, do not add packaging files. This record is evidence and a
recommended split. Delivery phases live in the [implementation
plan](../plans/rocci-browser-macos-app.md).[^macos-plan]

## Disposition

Draft and exploratory. Recommend deleting the TUI, keeping the graphical
`preview()` host and headless CLI, and assembling an ad-hoc **Rocci Browser**
`.app` in the browser crate (option D) that does not reuse `rocci bundle`, does
not embed adapters, and repairs GUI PATH plus Finder cwd. Production signing
stays a known limitation. The [implementation plan](../plans/rocci-browser-macos-app.md)
sequences TUI removal, launch-environment repair, bundle assembly, and docs.

[^original-research]: Option D host; TUI listed as a front; desktop packaging deferred; do not rocci-bundle a gallery.
[^browser-plan]: Phases 1–5 shipped a tui subcommand and left signed .app as later work.
[^macos-plan]: Phased TUI deletion, GUI launch rules, ad-hoc .app, docs.
[^folder-plan]: Host-only folder dialog; no ancestor walk for browser.toml; signed .app called a separate row.
[^run-no-window]: Live-session lock for graphical preview(); TUI must not write it.
[^picker-rocci]: Gate 4 moves picker markup off the init script; independent of packaging.
[^preview-decision]: Native Tao/Wry shell stays the preview window.
[^chrome-research]: Overlay chrome stays HTML; not a reason to switch window shells.
[^tui-rs]: crossterm raw-mode picker; only TUI implementation.
[^browser-main]: clap Tui command; no-args calls window::run; tui does not open preview().
[^browser-cargo]: crossterm 0.28; rocci-desktop; no rocci-cli.
[^window-rs]: preview() with launcher, overlay IPC, picker true, state key browser.
[^picker-rs]: Enter/Tab outcomes tested without a terminal.
[^overlay-rs]: Cmd-P picker lives in initialization-script assets.
[^paths-rs]: cwd is the repo-local root; Finder cwd is typically /.
[^discovery-rs]: bin without slash is PATH lookup; slash bins resolve against repo root.
[^browser-readme]: tui example; .app not documented as shipped.
[^browser-guide]: signed .app planned; v1 is cargo run; rocci bundle packages a Roc app.
[^bundle-rs]: main.roc, Resources/app/server, generated plist without icon, ad-hoc codesign.
[^run-bundled]: Bundled launch requires rocci.toml plus compiled server.
[^bundle-script]: Maintainer Darwin wrapper around rocci bundle.
[^desktop-readme]: Packaging runtime for ad-hoc Roc app bundles; Dock PNG at runtime.
[^preview-rs]: Long-lived preview() with menus, load_url, Dock icon on Init.
[^desktop-lib]: run() multi-window shell and Reopen; picker false.
[^icon-rs]: 1024 PNG; macOS Dock via NSApplication, not CFBundleIconFile.
[^menu-rs]: Open Target when picker is true.
[^deps-check]: rocci-browser is base Rocci; cannot depend on rocci-cli.
[^known-limitations]: Production signing, notarization, and installers absent.
[^roadmap]: Ad-hoc Roc-server .app is done; production signing is not.
[^root-readme]: Project browser section; signed .app planned.
[^cli-docs]: rocci-browser fourth binary; signed .app planned.
