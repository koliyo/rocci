---
type: Implementation Plan
title: Native folder dialogs and a later adapter plugin index
description: "Gate 5 follow-on after rocci-browser Phases 1–5: add a host-only native folder dialog for registering projects, then a later local-to-remote plugin index. Does not add a plugin lifecycle on rocci or rocdown, and does not give authored Roc apps dialogs. Exploratory; no phase started."
tags: [domain/rocci, domain/desktop, concern/tooling, concern/architecture, concern/packaging]
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
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points for Rocci, Rocdown, and OKF preview
    author: process:cursor
    last_modified: 2026-08-19
  - id: known-limitations
    resource: ../status/known-limitations.md
    title: Known Rocci limitations
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: rocci-desktop crate contract
    author: process:git
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
  - id: browser-main
    resource: ../../crates/rocci-browser/src/main.rs
    title: rocci-browser add / remove / list CLI
    author: process:cursor
    last_modified: 2026-08-19
  - id: discovery-rs
    resource: ../../crates/rocci-browser/src/discovery.rs
    title: Plugin spec and local discovery order
    author: process:cursor
    last_modified: 2026-08-19
  - id: registry-rs
    resource: ../../crates/rocci-browser/src/registry.rs
    title: User projects.json registry
    author: process:cursor
    last_modified: 2026-08-19
  - id: repo-local
    resource: ../../.rocci/browser.toml
    title: Repo-local plugin and project rows
    author: process:cursor
    last_modified: 2026-08-19
---

# Native folder dialogs and a later adapter plugin index

## Goal

Open rocci-browser gate 5 in two tracks that share one human approval but
not one implementation: (1) a native folder dialog so graphical `add` does
not require typing a path, and (2) a later index for third-party *adapter*
binaries. v1 registration is path-typed CLI plus optional dialogs; plugins
stay local TOML.[^browser-plan][^browser-research][^browser-readme]

Do not add a plugin lifecycle to `rocci` or `rocdown`. Out-of-process
`browser-adapter` stdio is the plugin shape for this host only.[^cli-plan][^browser-plan]

Do not expose dialogs, filesystem pickers, or notifications to authored Roc
apps. Those remain a known desktop limitation.[^known-limitations][^desktop-readme]

Human approval of gate 5 is required before any phase.

## Shipped (not these phases)

`rocci-browser add <path>` canonicalizes a typed path into user
`projects.json`. Repo-local `.rocci/browser.toml` unions `[[plugin]]` and
`[[project]]` rows. Plugin discovery is `plugins/*.toml`, then that file,
then `ROCCI_BROWSER_PLUGINS`. There is no native dialog crate in the
workspace.[^browser-main][^registry-rs][^discovery-rs][^repo-local]

## Out of bound

- Built-in `site` / `docs` / `knowledge` ids in host source (gate 6).
- dlopen / Wasm adapters, Content-Length LSP framing.
- Signed `.app` packaging ([macOS app plan](rocci-browser-macos-app.md); this
  plan is dialogs and a plugin index only).[^macos-plan]
- A marketplace that installs plugins *into* `rocci` or `rocdown`.
- Giving `.rocci` templates a native dialog API.

## Phase 1 — Host-only folder dialog

Add a folder picker on the **browser crate** (for example `rfd`), not as a
Rocci native capability and not as a general `rocci-desktop` app API. CLI
`add` without a path, and a picker-overlay "Add folder" IPC, open the
platform dialog, then write the same `projects.json` row as today's typed
`add`.[^browser-main][^registry-rs][^known-limitations]

Keep `add <path>` for agents and `--no-window` environments. Headless tests
must not open a dialog: pass a path, or stub the picker.

**Exit:** Graphical add can choose a directory through the OS dialog; typed
`add` still works; `cargo test -p rocci-browser` stays headless.

## Phase 2 — Registry affordance

Surface add/remove in the host picker (or launcher origin) with last probe
label and last error, as the research already described. Dialog from Phase
1 is the folder control; path paste remains. Do not walk ancestor
directories to find `.rocci/browser.toml`.[^browser-research][^discovery-rs]

**Exit:** From the persistent window, a user can add and remove user-registry
targets without leaving the graphical host.

## Phase 3 — Adapter index (later)

Only after Phases 1–2 and a separate trust discussion: an optional URL that
lists adapter plugin manifests (`id`, `bin` name, `argv`, checksum). Install
copies a `.toml` into `plugins/` and does not exec from the network on
probe. First-party rows stay repo-local data.[^discovery-rs][^repo-local][^cli-plan]

No auto-update. No in-process native plugins. Missing binaries stay
warnings beside the plugin id.

**Exit:** A documented, fetch-then-local-toml flow exists; product CLIs are
unchanged.

## Status

Exploratory; no phase started. Blocked on rocci-browser gate 5. Prefer Phase
1 dialogs before any marketplace work.

[^browser-plan]: Gate 5: native folder dialogs or a third-party plugin marketplace.
[^browser-research]: v1 add is path-typed plus CLI; native dialogs only if they exist.
[^cli-plan]: Reject a plugin host on rocci and rocdown; exec-sibling adapters are a dispatcher, not that host.
[^known-limitations]: Desktop host has no general native dialogs for authored apps.
[^desktop-readme]: Desktop responsibilities are window, overlay chrome, persistence, packaging — not dialogs.
[^browser-readme]: Plugin discovery is local TOML, repo-local rows, and an env override.
[^browser-main]: add requires a path argument; no dialog flag.
[^discovery-rs]: PluginSpec is id, bin, argv; discovery is files then repo-local then env.
[^registry-rs]: User registry persists projects.json only.
[^repo-local]: Workspace browser.toml lists first-party adapter bins as data.
[^macos-plan]: Ad-hoc Finder .app is a separate follow-on; this plan is folder dialogs and a plugin index.
