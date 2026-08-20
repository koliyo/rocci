---
type: Implementation Plan
title: Runtime console without a component log API
description: "Keep the Dev Console as a host runtime stream. Feed rocci run Roc stderr into InspectorServer LogHub so app preview matches static preview. Do not add logging from Rocci @component functions."
tags: [domain/rocci, domain/desktop, domain/runtime, concern/architecture, concern/tooling, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T10:45:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/inspector-console-scope.md
    title: Preview inspector console scope
    author: process:cursor
    last_modified: 2026-08-20
  - id: preview-research
    resource: ../research/preview-inspector.md
    title: Extended preview-window inspector research
    author: process:cursor
    last_modified: 2026-08-20
  - id: inspector-plan
    resource: preview-inspector.md
    title: Original dock/tabs/console specification
    author: process:cursor
    last_modified: 2026-08-20
  - id: repair-plan
    resource: preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: logs-rs
    resource: ../../crates/rocci-cli/src/logs.rs
    title: LogHub and logs::tee
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../crates/rocci-cli/src/inspector.rs
    title: Sibling InspectorServer and Console pane
    author: process:git
    last_modified: 2026-08-20
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: StderrTee and with_window_and_inspector
    author: process:git
    last_modified: 2026-08-20
  - id: dev-server
    resource: ../../crates/rocci-cli/src/dev_server.rs
    title: Static DevServer already tees into LogHub
    author: process:git
    last_modified: 2026-08-20
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: rocci-cli Dev Console contract
    author: process:git
    last_modified: 2026-08-20
  - id: desktop-readme
    resource: ../../crates/rocci-desktop/README.md
    title: Overlay docks the inspector iframe
    author: process:git
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: human:nils
    last_modified: 2026-08-16
  - id: window-rs
    resource: ../../crates/rocci-desktop/src/window.rs
    title: WebViewBuilder without a console handler
    author: process:git
    last_modified: 2026-08-19
---

# Runtime console without a component log API

## Purpose and authority

The [console-scope research](../research/inspector-console-scope.md)
recommends keeping the preview Dev Console as a **runtime** stream and
rejecting a Rocci `@component` log API. This plan implements that split:
wire `rocci run` Roc stderr into the existing hub, and leave language
logging undesigned.[^research][^preview-research][^inspector-plan]

Exploratory. Do not start a phase until the user asks.

This plan **does not restart** dock, tabs, Source DX, or the static
`DevServer` log hub. Those already exist. It **does** own the leftover from
the [repair plan](preview-inspector-repair.md) Phase 0 item 4: sibling
`InspectorServer` must share runtime lines, not gain a new log
API.[^repair-plan][^inspector-rs]

## Goal

After `rocci run` (and `view`) with Dev open, the Console tab lists the
same class of lines the terminal already printed for that Roc process
(compile/runtime stderr plus host serve notes), with `source: runtime`.
Static `rocdown run` / `rocci-okf run` Console behavior stays as
shipped.[^logs-rs][^dev-server][^serve-rs]

## Out of bound

- `@log`, template `dbg`, Datastar `data-log`, or any Roc `Log` effect.
- Making `@component` functions effectful so they can print.[^pure-render]
- Page JavaScript `console.*` capture (original inspector Phase 5). Native
  Web Inspector remains the page console. Do not start the overlay wrap
  here.[^inspector-plan][^window-rs]
- Piping Roc **stdout** into the hub (still inherit).
- A `source: app` badge for handler `Stdout.line!`.
- Network waterfall, DOM picker, preserve-log checkbox.
- Rewriting Console HTML as a live Datastar Rocci app.
- Changing `LogLine` JSON field names consumed by the pane script.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Runtime-only `source` | `LogLine::runtime` stays `source: "runtime"`. Do not add `component` / `app` in this plan. |
| Tee, not redirect | Terminal stderr remains. Hub is a copy. |
| Pure render | `@component` stays a function from values to `Html`. |
| Overlay split | `rocci-desktop` still has no language-crate dependency. Overlay does not embed log text in the initialization script.[^desktop-readme] |
| Existing routes | `/__rocci/logs`, `/logs/events`, `POST /logs/clear` and product aliases stay. |
| Native inspector | View-menu wry DevTools stays a separate page-JS console. |
| No preview window in tests | Hub, SSE content-type, and `--no-window` curl prove the feed. |

## Phase 1 — Freeze the message-class contract

Record in this section (already) that Console v1 is:

| In | Out |
| --- | --- |
| Host `logs::tee` lines (bind, rebuild, CLI errors) | `@component` render traces |
| Roc process stderr (`StderrTee` bytes, split into lines) | Page `console.*` |
| `source: runtime`, levels guessed from text or default `info`/`error` | New product log syntax |

Close original inspector gate 3 as runtime-only for this milestone.
That gate now records this freeze: page `console.*` stays later
(original inspector Phase 5 / native Web Inspector), not Console v1.

**Exit:** This section. No code.

## Phase 2 — Feed `InspectorServer` from Roc stderr

`InspectorServer::spawn` creates an empty `LogHub`. `spawn_roc` pipes
stderr into `StderrTee`, which eprints and buffers for listen failure,
then keeps reading for the process lifetime. Nothing copies those bytes
into the inspector hub.[^serve-rs][^inspector-rs]

Bound:

- Create the `LogHub` (or `InspectorServer`) **before** or **with** Roc
  spawn so listen-time diagnostics are not dropped.
- After listen, flush `tee.snapshot()` into the hub as runtime lines
  (skip duplicates if the live thread already pushed).
- Give the stderr reader a hub: each chunk splits on newlines, pushes
  `LogLevel::Error` when the existing `roc_output_is_failure` heuristic
  matches, else `Info` (or `Warn` for lines that already look like
  warnings). Do not invent a Roc diagnostic parser.
- Host notes around serve (`serving … at http://127.0.0.1:…`) should
  `logs::tee` into the same hub, matching static `DevServer`.
- `InspectorServer` already serves `/__rocci/logs`. Keep that.
- Do not change static `DevServer` tee sites except to share a helper if
  that is smaller than duplicating line-split logic.

Tests (`cargo test -p rocci-cli`):

- Helper: stderr bytes → `LogLine`s with `source: runtime`.
- `InspectorServer` after pushing a line: `GET /__rocci/logs` JSON includes
  it (the crate already has a similar test; extend so spawn + push
  simulates tee).
- Optional `--no-window` only if an existing serve test can curl logs
  without waiting on a real Roc compile. Prefer unit tests.

**Exit:** `rocci run` sibling Console is no longer structurally empty:
hub JSON contains flushed stderr and at least one host serve line after a
successful listen in tests that can spawn the inspector without a window.
`cargo test -p rocci-cli`. `cargo fmt --all -- --check`.

## Phase 3 — Docs

- `rocci-cli` README: Console lists runtime messages from the session,
  including Roc stderr for `rocci run`; it is still not an app-level Rocci
  log API.[^cli-readme]
- Public CLI reference only if it already describes Console (keep the
  same sentence).
- Do not claim page-JS parity or Chrome Console parity.
- Point remaining page-JS wrap at the original inspector plan Phase 5;
  do not describe it as shipped.

**Exit:** README sentences match the runtime-only contract and the `rocci
run` stderr feed. `cargo test -p rocci-cli`.

## Status

Exploratory; Phases 1–3 implemented in this revision, not CI-complete.
Depends on the shipped `LogHub` and Console pane. Does not depend on
further inspector-repair phases.

[^research]: Runtime-only recommendation; component log API rejected; rocci run hub empty.
[^preview-research]: Original three-class table and open gate 3.
[^inspector-plan]: Phase 4 hub shipped; Phase 5 page JS optional; app log out of bound.
[^repair-plan]: Leftover sibling LogHub wire-up named, not delivered in repair Phase 7.
[^logs-rs]: Ring buffer, runtime source, tee helper.
[^inspector-rs]: Fresh LogHub per InspectorServer; Console empty-state copy.
[^serve-rs]: StderrTee eprints; with_window_and_inspector does not take a log feed.
[^dev-server]: Static products already tee into the Console hub.
[^cli-readme]: Current public wording: runtime messages, not an app log API.
[^desktop-readme]: Overlay iframe to inspector_url; no compiler output in host assets.
[^pure-render]: Components remain pure Html functions.
[^window-rs]: Page JS capture still needs an overlay wrap, not this plan.
