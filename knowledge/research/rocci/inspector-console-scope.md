---
type: Research Report
title: Preview inspector console scope
description: "The Dev Console tab should stay a host runtime stream (CLI, watch, Roc stderr). Do not add logging from Rocci @component render functions; that would need a language effect and would violate pure render."
tags: [domain/rocci, domain/desktop, domain/runtime, concern/architecture, concern/tooling, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T08:54:17Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: preview-research
    resource: ../preview-inspector.md
    title: Extended preview-window inspector research
    author: process:cursor
    last_modified: 2026-08-20
  - id: inspector-plan
    resource: ../../plans/rocci/preview-inspector.md
    title: Original dock/tabs/console specification
    author: process:cursor
    last_modified: 2026-08-20
  - id: repair-plan
    resource: ../../plans/rocci/preview-inspector-repair.md
    title: Investigate and repair the preview inspector
    author: process:cursor
    last_modified: 2026-08-20
  - id: console-plan
    resource: ../../plans/rocci/inspector-console-scope.md
    title: Runtime console wiring without a component log API
    author: process:cursor
    last_modified: 2026-08-20
  - id: logs-rs
    resource: ../../../crates/rocci-cli/src/logs.rs
    title: LogHub, LogLine, runtime-only source field
    author: process:git
    last_modified: 2026-08-19
  - id: inspector-rs
    resource: ../../../crates/rocci-cli/src/inspector.rs
    title: Console pane HTML, sibling InspectorServer log routes
    author: process:git
    last_modified: 2026-08-20
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: StderrTee and InspectorServer spawn after Roc listen
    author: process:git
    last_modified: 2026-08-20
  - id: dev-server
    resource: ../../../crates/rocci-cli/src/dev_server.rs
    title: Static DevServer tees rebuild and bind lines into LogHub
    author: process:git
    last_modified: 2026-08-20
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated app main imports Env Path Server Sse, not Stdout
    author: process:git
    last_modified: 2026-08-20
  - id: metrics-panel
    resource: ../../../crates/rocci-cli/templates/dev/MetricsPanel.rocci
    title: ConsoleBody template used for CSS extraction
    author: process:git
    last_modified: 2026-08-20
  - id: cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: Console is runtime messages, not an app log API
    author: process:git
    last_modified: 2026-08-20
  - id: window-rs
    resource: ../../../crates/rocci-desktop/src/window.rs
    title: WebViewBuilder without a console handler
    author: process:git
    last_modified: 2026-08-19
  - id: history-rs
    resource: ../../../crates/rocci-desktop/src/history.rs
    title: Overlay IPC command vocabulary
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../../crates/rocci-template/src/lower.rs
    title: "@component lowers to an ordinary Roc function returning Html"
    author: process:git
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: human:nils
    last_modified: 2026-08-16
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: chrome-research
    resource: ../desktop-host-chrome-and-inspector-ui.md
    title: Overlay chrome versus preview-origin inspector UI
    author: process:cursor
    last_modified: 2026-08-18
  - id: preview-decision
    resource: ../../decisions/preview-window.md
    title: Preview window versus preview chrome versus Dev panel
    author: process:cursor
    last_modified: 2026-08-18
  - id: rendering-doc
    resource: ../../../docs/concepts/rendering-model.rocdown
    title: Published rendering model
    author: human:nils
    last_modified: 2026-08-18
  - id: rocci-ref
    resource: ../../../docs/reference/rocci.rocdown
    title: Public Rocci language reference
    author: process:git
    last_modified: 2026-08-20
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Counter app handlers and pure CounterCard
    author: process:git
    last_modified: 2026-08-20
---

# Preview inspector console scope

## Scope and authority

This record is exploratory. It answers whether the preview Dev **Console**
tab should list only host runtime messages, or also logs originating from
Rocci `@component` markup. It does not change a language contract and does
not start implementation.[^preview-research][^inspector-plan]

Implementation plan: [Runtime console without a component log
API](/plans/rocci/inspector-console-scope.md).[^console-plan]

## Question

The shipped Console tab and the original inspector research already name
three message classes (runtime, page JavaScript, app/component `log`) and
put app logging out of scope. That left two gaps:[^preview-research][^cli-readme]

1. **Why** `@component` logging is a language change, not a host tee.
2. **Whether** `rocci run` Console is empty because Roc process stderr was
   never wired, which can be mistaken for a missing component logger.

## Shipped Console

The data plane is `LogHub`: a 1000-line ring of `{ t, level, source, text }`
with `source` hard-coded to `"runtime"` on `LogLine::runtime`. Routes are
`GET /__rocci/logs`, SSE `GET /__rocci/logs/events`, and `POST
/__rocci/logs/clear` (aliases `/__rocdown/` and `/__rocci_okf/`). The
Console body is a no-JS snapshot table plus a small script that appends SSE
rows, filters by level, and clears via POST.[^logs-rs][^inspector-rs][^cli-readme]

`ConsoleBody` in `MetricsPanel.rocci` is the CSS/fixture source. Rust in
`inspector.rs` still emits the HTML. Using Rocci for the **pane chrome** is
already the inspector-UI split; it is not a log API for app
components.[^metrics-panel][^inspector-rs][^chrome-research]

Two hosts populate that pane differently:

| Product | Log hub | What authors see in Console today |
| --- | --- | --- |
| `rocdown run` / `rocci-okf run` (static `DevServer`) | Same `ReloadHub.logs` the watcher tees into | Bind URL, rebuild start/finish, rebuild errors |
| `rocci run` / `view` (Roc process + sibling `InspectorServer`) | Shared `LogHub` from `spawn_roc_with_logs` into `InspectorServer` | Host serve notes plus Roc **stderr** lines (`source: runtime`) |

Static preview tees `logs::tee` on serve and rebuild. `rocci run` pipes Roc
stderr through `StderrTee` into the same hub the Console reads (terminal
still gets a copy). Handler intentional logging beyond that stream is
[handler-runtime-logging](handler-runtime-logging.md).[^dev-server][^serve-rs][^repair-plan]

Wry 0.55 still has no `with_console_handler`. Native Web Inspector remains
a separate View-menu command and already shows page `console.*`.[^window-rs][^preview-decision]

## Four things "logging from Rocci components" could mean

### 1. Side effects inside `@component` (reject)

An `@component` lowers to an ordinary Roc function that returns `Html`.
The published rendering model and the stable pure-render decision keep
persistence, request lifecycle, and effects **outside** that function.
There is no `@log`, `dbg`, or Datastar `data-log` in the language
reference.[^lower-rs][^pure-render][^rendering-doc][^rocci-ref]

A render-time log would have to be an effect. That would:

- Make the generated function effectful, so it could no longer be the same
  `Html` value used for fixtures, static apply, and handler patches.
- Fire on every GET and every Datastar morph, flooding the Console.
- Invent a product log language the original inspector plan explicitly
  refused.[^inspector-plan][^server-state]

`examples/rocci/standalone/counter/Counter.rocci` shows the split already: `CounterCard` is
a pure view of `{ count }`; SQLite lives in `@on` / helpers with `!`.
Putting `Stdout.line!` inside `CounterCard` is not expressible without
changing lowering.[^counter][^pure-render]

### 2. Prints from `@on` handlers (not component logging; designed separately)

`@on` bodies are Roc and already effectful (`Sqlite.execute!` in Counter).
Generated `main.roc` imports `pf.Env`, `pf.Path`, `pf.Server`, and
`pf.Sse`, not `pf.Stdout` / `pf.Stderr`. Authors may import those
platform modules. **`Stderr.line!` already reaches** the terminal and the
Dev Console (`source: runtime`) via `StderrTee`. **`Stdout.line!` still
inherits** to the terminal only and is not teed into `LogHub`. That is
application I/O, not a template API.[^dispatch-rs][^serve-rs][^counter]

Follow-on design (prefix protocol, optional `log!`, `source: app`) lives
in [Handler logging into the Rocci runtime
console](handler-runtime-logging.md). Do not treat that work as a reason
to add `@log` in markup or to make `@component` effectful.

### 3. Page JavaScript `console.*` (separate class; optional later)

Datastar, `goto.js`, and any inline script log in the inspected document.
The original plan's Phase 5 was an overlay wrap plus IPC prefix `log:`
because wry has no console callback. That is **page** source, not Rocci
render. Native Web Inspector already shows it. Mixing it into v1 without
badges would blur host rebuild errors with client noise.[^preview-research][^window-rs][^history-rs]

### 4. Authoring the Console pane in Rocci (already the CSS source)

Compiler-derived **content** belongs on the preview origin. Overlay docks
the iframe; it must not snapshot logs into the initialization script. The
pane consuming `/__rocci/logs` JSON is that split. Expanding `ConsoleBody`
into a live Datastar app is a chrome-authoring follow-on, not a log-source
decision.[^chrome-research][^metrics-panel][^preview-decision]

## Recommendation

**Keep Console as a runtime stream only. Do not add logging from Rocci
components.**

Define **runtime** as host-originated diagnostic lines the terminal already
prints for the session:

- CLI / watch / static rebuild / bind / inspect-server errors (`logs::tee`).
- Roc process **stderr** after `spawn_roc` (compile diagnostics, runtime
  panics, platform messages currently in `StderrTee`).

Do **not** define runtime as:

- `@component` render traces.
- A Roc `Log` effect or template `dbg`.
- Datastar `data-log`.
- Inspected-page `console.*` (keep on native Web Inspector until a later
  optional wrap).
- Roc process **stdout** (still inherited; may carry listen or HTTP noise).

Close original inspector gate 3 as **runtime-only for this milestone**.
Page JS stays optional and later. App/component `log` stays a non-goal,
not a deferred phase of this Console.[^preview-research][^inspector-plan]

The useful missing piece is not a component logger. It is feeding `rocci
run`'s existing stderr tee into the sibling hub so app preview Console
matches static preview Console.

## Options considered

| Option | Verdict |
| --- | --- |
| A. Runtime host lines only (today's static path) | Incomplete: `rocci run` Console stays empty |
| B. Runtime = host lines **plus** Roc stderr (recommended) | Same class of message; no language change |
| C. B plus page `console.*` in the same milestone | Optional later; wry wrap; native inspector already has it |
| D. `@log` / effectful components | Reject; contradicts pure render |
| E. Handler prints → Console (`stderr` today; `source: app` later) | See [handler-runtime-logging](handler-runtime-logging.md); not component logging |

## Disposition

Draft and exploratory. The Console **shell** and `rocci run` stderr feed
are in tree. The **scope** should stay runtime for host/compiler lines.
Handler prints that want an `app` badge are a separate follow-on, not a
component log API.

[^preview-research]: Three message classes; app log out of scope; gate 3 left open.
[^inspector-plan]: Console v1 runtime-only; no Rocci app log API; Phase 5 page JS optional.
[^repair-plan]: Leftover: wire sibling InspectorServer to the runtime LogHub, not a new log API.
[^console-plan]: Follow-on implementation: feed Roc stderr into the hub; do not add @log.
[^logs-rs]: LogLine::runtime sets source to runtime; tee eprints and pushes.
[^inspector-rs]: Console pane, empty-state copy, InspectorServer owns a fresh LogHub.
[^serve-rs]: StderrTee eprints Roc stderr; InspectorServer spawns after listen with no hub feed.
[^dev-server]: Static serve and watch call logs::tee into the same hub the Console reads.
[^dispatch-rs]: Generated program imports Server/Sse, not Stdout.
[^metrics-panel]: ConsoleBody markup and CSS; Rust still fills rows from LogHub.
[^cli-readme]: Documents Console as session runtime messages, not an app-level Rocci log API.
[^window-rs]: No wry console-message builder.
[^history-rs]: IPC verbs a future page-log wrap must not collide with.
[^lower-rs]: Component declaration emits a Roc function whose body lowers to Html.
[^pure-render]: @component is a pure render abstraction; effects stay outside template lowering.
[^server-state]: Handlers own mutation; components render Html from explicit values.
[^chrome-research]: Inspector content is preview-origin UI over host JSON, not overlay HTML.
[^preview-decision]: Dev panel versus native Web Inspector naming.
[^rendering-doc]: Authored components are Roc functions returning Html.
[^rocci-ref]: Recognized forms are @component, @fixture, @css, @context, @init, @on; no @log.
[^counter]: CounterCard is a pure count view; I/O is in @on helpers.
