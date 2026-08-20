---
type: Implementation Plan
title: Product run skips preview when rocci-browser is already open
description: "Gate 3 follow-on after rocci-browser Phases 1–5: when a graphical browser session exists, rocci / rocdown / rocci-okf run (and view / browse) default to --no-window instead of opening a second one-shot preview. Exploratory; no phase started."
tags: [domain/rocci, domain/desktop, domain/rocdown, domain/rocci-okf, concern/tooling, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-20T05:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: browser-plan
    resource: rocci-browser.md
    title: Dedicated rocci-browser implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-research
    resource: ../research/rocci-browser.md
    title: Dedicated rocci-browser CLI and desktop host research
    author: process:cursor
    last_modified: 2026-08-19
  - id: browser-readme
    resource: ../../crates/rocci-browser/README.md
    title: rocci-browser crate contract
    author: process:cursor
    last_modified: 2026-08-19
  - id: macos-plan
    resource: rocci-browser-macos-app.md
    title: rocci-browser macOS app and TUI removal plan
    author: process:cursor
    last_modified: 2026-08-20
  - id: browser-guide
    resource: ../../docs/guides/rocci-browser.rocdown
    title: Public project-browser guide
    author: process:cursor
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: Shared ServeOptions and one-shot preview open
    author: process:git
    last_modified: 2026-08-19
  - id: cli-main
    resource: ../../crates/rocci-cli/src/main.rs
    title: rocci run / view / browse clap surface
    author: process:git
    last_modified: 2026-08-19
  - id: paths-rs
    resource: ../../crates/rocci-browser/src/paths.rs
    title: Browser directory and registry paths
    author: process:cursor
    last_modified: 2026-08-19
  - id: window-rs
    resource: ../../crates/rocci-browser/src/window.rs
    title: Graphical host preview loop
    author: process:cursor
    last_modified: 2026-08-19
  - id: preview-decision
    resource: ../decisions/preview-window.md
    title: Call the embedded Tao/Wry shell the preview window
    author: process:cursor
    last_modified: 2026-08-18
  - id: core-readme
    resource: ../../crates/rocci-core/README.md
    title: rocci-core shared contracts
    author: process:git
    last_modified: 2026-08-18
---

# Product run skips preview when rocci-browser is already open

## Goal

Give the persistent `rocci-browser` window ownership of preview chrome while
it is running. Direct `rocci run`, `rocdown run`, `rocci-okf run`, plus
`rocci view` and `rocci browse`, should then behave as `--no-window`: print
the origin and keep serving, instead of calling `preview()`.[^browser-plan][^serve-rs][^browser-research]

Adapters launched from the host already pass `--no-window`. This plan is only
for the *user-invoked* product CLIs that still open today's one-shot
window.[^browser-readme][^browser-guide]

Human approval of gate 3 is required before any phase. Until then, product
`run` keeps opening a preview window when invoked directly.[^browser-plan]

## Shipped (not these phases)

`rocci-browser` with no arguments owns one long-lived preview window,
`load_url`s adapter origins, and reuses warm sessions. Product CLIs still
call `rocci_desktop::preview` unless the user passed `--no-window`. There is
no lock file or other live-session signal in `browser_dir`.[^window-rs][^paths-rs][^serve-rs]

## Out of bound

- Auto-navigating the host webview to a URL the user started outside the
  picker (would need a host control socket).
- Changing adapter `open` (already `--no-window`).
- Making product crates depend on `rocci-browser` (put the lock helper in
  `rocci-core` so base, Rocdown, and OKF CLIs share one check).[^core-readme]
- Defaulting to `--no-window` when the host is *not* running.
- Signed `.app` packaging (see [macOS app plan](rocci-browser-macos-app.md)).[^macos-plan]

## Phase 1 — Live-session lock in rocci-core

`Paths` already resolve `ROCCI_BROWSER_DIR` / `$ROCCI_HOME/.rocci/browser` /
`~/.rocci/browser`. Duplicate that directory rule in `rocci-core` (or export
a tiny shared path helper) and add `session.pid` next to `projects.json`.[^paths-rs][^core-readme]

- Graphical `rocci-browser` writes its pid on entering `preview()` and
  removes the file on clean exit. Stale file plus dead pid is not live.
- `is_browser_session_live()` returns true only for that case. CLI commands
  including `open --no-window` do not write the lock. There is no TUI.
- Tests: live pid, missing file, stale pid; no preview window.

**Exit:** Host writes and clears the lock; core tests cover the three
states without spawning product CLIs.

## Phase 2 — Honor the lock on product serve

`ServeOptions.no_window` stays a clap flag. After parse, if the flag is
false and the lock is live, treat the request as `--no-window` (print URL,
skip `open_preview`). Add `--window` to force a second preview
anyway.[^serve-rs][^cli-main]

Wire the same check on `rocdown run` / `playground` and `rocci-okf run`,
not only `rocci`. Port defaulting still follows the effective no-window
bit (`8000` vs free port).

Tests: clap `--window` wins; lock live plus no flags skips `preview` in a
unit that stubs the lock; no live Roc compile.

**Exit:** With a graphical host running, `rocci run`, `rocdown run`, and
`rocci-okf run` print a URL and do not open a second native window;
`--window` still can.

## Phase 3 — Public contract

Update the browser guide, crate README, and the three CLI `--no-window`
help texts: a live browser session implies no-window unless `--window`.
Keep calling the native window the preview window.[^browser-guide][^preview-decision][^browser-readme]

**Exit:** Docs state the default without promising auto-navigation into the
host.

## Status

Exploratory; no phase started. Blocked on rocci-browser gate 3.

[^browser-plan]: Gate 3: product run defaults to --no-window when a browser session exists.
[^browser-research]: Window ownership table: change product one-shot preview only if the browser owns the window.
[^browser-readme]: Direct product run still opens a one-shot preview.
[^browser-guide]: Same one-shot contract on the public docs page.
[^serve-rs]: Shared --no-window flag and open_preview when the flag is false.
[^cli-main]: rocci run / view / browse pass serve.no_window through.
[^paths-rs]: Browser directory overrides; no session lock file today.
[^window-rs]: Graphical host enters rocci_desktop::preview and stays there.
[^preview-decision]: The native window stays named the preview window.
[^core-readme]: Shared contracts with no GUI or product-format dependencies.
[^macos-plan]: Ad-hoc Finder .app is a separate follow-on; this plan is the live-session lock.
