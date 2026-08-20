---
type: Research Report
title: Handler logging into the Rocci runtime console
description: "@on and @init can already print via basic-webserver Stderr/Stdout; stderr reaches the Dev Console and CLI today as source runtime. Prefer a stderr prefix (and optional log! helper) over a language @log or HTTP ingest."
tags: [domain/rocci, domain/runtime, domain/desktop, concern/architecture, concern/tooling, concern/ui]
status: draft
generated: { by: process:cursor, at: 2026-08-20T08:54:17Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: console-scope
    resource: inspector-console-scope.md
    title: Preview inspector console scope
    author: process:cursor
    last_modified: 2026-08-20
  - id: console-plan
    resource: ../plans/inspector-console-scope.md
    title: Runtime console without a component log API
    author: process:cursor
    last_modified: 2026-08-20
  - id: optional-request
    resource: optional-handler-request.md
    title: Optional request argument on Rocci service handlers
    author: process:cursor
    last_modified: 2026-08-20
  - id: logs-rs
    resource: ../../crates/rocci-cli/src/logs.rs
    title: LogHub and LogLine::runtime
    author: process:git
    last_modified: 2026-08-19
  - id: serve-rs
    resource: ../../crates/rocci-cli/src/serve.rs
    title: StderrTee feeds LogHub; stdout inherits
    author: process:git
    last_modified: 2026-08-20
  - id: driver-rs
    resource: ../../crates/rocci-cli/src/driver.rs
    title: Shared LogHub for spawn_roc_with_logs and InspectorServer
    author: process:git
    last_modified: 2026-08-20
  - id: dispatch-rs
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated main imports Env Path Server Sse, not Stdout
    author: process:git
    last_modified: 2026-08-20
  - id: inspector-rs
    resource: ../../crates/rocci-cli/src/inspector.rs
    title: Sibling inspector log GET/SSE/clear routes
    author: process:git
    last_modified: 2026-08-20
  - id: cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: Console is runtime messages, not an app log API
    author: process:git
    last_modified: 2026-08-20
  - id: lower-rs
    resource: ../../crates/rocci-template/src/lower.rs
    title: "@on lowers to effectful Roc; @component returns Html"
    author: process:git
    last_modified: 2026-08-20
  - id: rocci-ref
    resource: ../../docs/reference/rocci.rocdown
    title: Public @on / @init / @component contract
    author: process:git
    last_modified: 2026-08-20
  - id: counter
    resource: ../../examples/counter/Counter.rocci
    title: Effectful @on helpers versus pure CounterCard
    author: process:git
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: human:nils
    last_modified: 2026-08-16
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: bws-stdout
    resource: https://roc-lang.github.io/basic-webserver/0.16.0/
    title: basic-webserver 0.16 Stdout and Stderr effects
    author: process:web
    last_modified: 2026-08-20
  - id: preview-research
    resource: preview-inspector.md
    title: Extended preview-window inspector research
    author: process:cursor
    last_modified: 2026-08-20
---

# Handler logging into the Rocci runtime console

## Scope and authority

This record is exploratory. It asks how Rocci **`@on` / `@init` handlers**
(not `@component` render functions) could send diagnostic lines into the
same host stream the preview Dev **Console** and `rocci` CLI already show.

It extends option E from [Preview inspector console
scope](inspector-console-scope.md). It does not approve a language change
and does not start implementation.[^console-scope][^console-plan]

## Constraint that does not move

`@component` stays a pure function to `Html`. Effects, including prints,
belong in `@on`, `@init`, and ordinary Roc helpers with `!`. That split is
stable project policy, not a Console UX preference.[^pure-render][^server-state][^lower-rs][^rocci-ref]

## Shipped path today

### Host data plane

`LogHub` stores `{ t, level, source, text }` with a 1000-line ring. The only
constructor is `LogLine::runtime`, which hard-codes `source: "runtime"`.
Console consumes `GET /__rocci/logs` and SSE `GET /__rocci/logs/events`
(plus product aliases). There is a clear POST, not an ingest
POST.[^logs-rs][^inspector-rs][^cli-readme]

For `rocci run` / `view`, the CLI creates one `LogHub`, passes it to
`spawn_roc_with_logs`, tees host serve notes with `logs::tee`, and hands
the same hub to sibling `InspectorServer`. Roc **stderr** is piped: each
line is eprinted to the terminal and pushed to the hub. Roc **stdout**
still uses `Stdio::inherit()` and never enters the hub.[^serve-rs][^driver-rs]

Static `rocdown run` / `rocci-okf run` already tee rebuild and bind lines
into the same Console shape; those products have no Roc `@on`
handlers.[^console-scope]

### Handler authoring surface

`@on` / `@init` bodies are ordinary Roc inside a try block. Generated
`main.roc` imports `pf.Env`, `pf.Path`, `pf.Server`, and `pf.Sse`, not
`pf.Stdout` / `pf.Stderr`. Authors may add those imports themselves; the
platform exposes `Stdout.line!` and `Stderr.line!`.[^dispatch-rs][^lower-rs][^bws-stdout][^optional-request]

Counter already shows the intended split: SQLite and mutation live in
`@on` / `*!` helpers; `CounterCard` only renders `{ count }`.[^counter]

### What an author gets if they print today

| Call site | Terminal (`rocci run`) | Dev Console |
| --- | --- | --- |
| `try Stderr.line! "…"` in `@on` / `@init` | Yes (via stderr tee) | Yes, `source: runtime`, level from line heuristics |
| `try Stdout.line! "…"` in `@on` / `@init` | Yes (inherited) | No |
| Print inside `@component` | Not expressible without breaking pure render | N/A |
| Page `console.*` | Native Web Inspector only | Out of Console v1 |

So **targeting the Rocci runtime stream from handlers already works for
stderr**, with no Rocci syntax. The gaps are product clarity (undocumented
convention), lack of an `app` badge, and stdout never joining the
hub.[^serve-rs][^cli-readme][^console-scope]

## Why the host cannot treat handler logs as in-process calls

The Roc app is a **child process**. The Console hub lives in the **CLI**
(sibling inspector for apps; same-origin `/__rocci` for static DevServer).
Handlers cannot call Rust `LogHub::push` directly. Every app→Console path
must be observable I/O or an explicit out-of-band channel the host
owns.[^driver-rs][^inspector-rs]

Sibling ports also make a naive "POST `/__rocci/logs` from the app" brittle:
the app listens on the product port; the inspector often listens elsewhere.
Any HTTP ingest needs an injected base URL and failure policy.

## Options

### A. Document platform stderr (zero language change) — recommended near-term

Publish that effectful regions may `import pf.Stderr` and
`try Stderr.line! "…"`, and that under `rocci run` those lines appear in
both the CLI stderr stream and the Dev Console as runtime messages.

Pros: already implemented; matches basic-webserver examples that log from
`respond!`; no new syntax; production deploys without an inspector still
see stderr.

Cons: no `source: app` filter; compiler diagnostics and author prints share
one badge; levels stay text heuristics (`warning` → warn, failure phrases →
error).

### B. Stderr line protocol → `source: app` — recommended product step

Host `StderrHubFeed` recognizes a stable prefix, for example:

```text
[rocci:log:info] increment wrote count=3
[rocci:log:error] reset failed: …
```

Unprefixed stderr stays `source: runtime`. Prefixed lines strip the marker,
set `source: "app"` (or `handler`), and take the level from the token.
Terminal tee still prints the raw line (or a cleaned line; pick one and
document it).

Pros: still process I/O; works with sibling inspector; optional later
`LogLine` constructor without changing JSON field names; authors can write
the prefix by hand before any helper ships.

Cons: magic string; must not collide with Roc compiler output (prefix is
unlikely in rustc/roc diagnostics).

### C. Generated `log!` / `Rocci.Log` helper — optional sugar on B

Compile injects a small Roc module (or appends helpers next to
`listen_port!`) so handlers write:

```roc
try log!(Info, "increment wrote $(count.to_str())")
```

The helper formats the B prefix onto `Stderr.line!`. Document that `log!`
is legal only in effectful Roc (`@on`, `@init`, `*!` helpers), never in
`@component`.

Pros: ergonomic; keeps effects explicit with `!`; no markup `@log`.

Cons: codegen / docs surface; must not auto-import into every template
module in a way that tempts component use.

### D. Tee stdout as app (or all) lines — weak alone

basic-webserver samples often use `Stdout.line!` for request logs.
Piping stdout like stderr would show those in Console, but also risks
capturing platform chatter if any lands on stdout, and breaks "inherit
until listen" assumptions unless carefully staged.

Prefer B/C on **stderr** and document "use Stderr (or `log!`) for Console."
If stdout tee is added later, apply the same prefix filter rather than
promoting every stdout byte to `app`.

### E. HTTP ingest `POST /__rocci/logs` — defer

CLI sets e.g. `ROCCI_LOG_URL`, helper POSTs JSON `{level,text}`. Heavy for
v1: DNS/loopback, inspector lifetime, auth none, production no-op policy,
and a second code path beside the stderr tee already required for
diagnostics.

### F. Markup `@log` or render-time dbg — reject

Same rejection as console-scope option D for components. For handlers,
`@log` would only be sugar over C and would blur the directive vocabulary
(`@on` bodies are already Roc). Prefer Roc `log!` over a new
directive.[^console-scope][^rocci-ref]

### G. Per-response debug channel (headers / Datastar events) — out of scope

Useful for request-scoped tracing in the page, not for the session Console
stream. Different product.

## Recommendation

1. **Near-term (docs + examples):** treat `pf.Stderr.line!` in `@on` / `@init`
   as the supported way to target the Rocci runtime Console and CLI. Discard
   `StderrErr` with `match` when the handler also uses other `?` errors
   (SQLite and stderr do not share one error type). Documented in the Rocci
   reference and server-actions guide; demonstrated in `examples/counter` and
   `examples/rocdown-counter`.
2. **Next implementation (if wanted):** option **B**, then optional **C**.
   Extend `LogLine` (or `push_line`) so Console can badge `app` without
   renaming JSON fields the pane already reads.
3. **Keep rejected:** effectful `@component`, markup `@log`, and HTTP ingest
   as the primary path.
4. **Do not reopen** console-scope's runtime-only gate for host/compiler
   lines; handler prints are a **second source class** on the same pane,
   still outside pure render.[^console-scope][^preview-research]

## Relation to other records

| Record | Role |
| --- | --- |
| [inspector-console-scope](inspector-console-scope.md) | Rejects component logging; names handler stdout as later optional |
| [inspector-console-scope plan](../plans/inspector-console-scope.md) | Wires Roc stderr into the hub; out-of-bound for `source: app` |
| This record | Designs how **handlers** intentionally join that pane |

A follow-on implementation plan should cite this research if B/C are
scheduled. Writing this report does not start that plan.

## Disposition

Draft and exploratory on the **product** choices (prefix protocol, `log!`,
`source: app`). **Shipped authoring path:** `pf.Stderr.line!` in `@on` /
`@init` is documented in the Rocci reference and server-actions guide, and
demonstrated in `examples/counter` and `examples/rocdown-counter`.

[^console-scope]: Runtime-only Console; component log rejected; handler Stdout sketched as later optional.
[^console-plan]: Stderr feed and runtime-only freeze; `source: app` explicitly out of bound.
[^optional-request]: Handlers remain two-argument effectful Roc at the dispatch boundary.
[^logs-rs]: LogLine::runtime hard-codes source runtime; tee eprints and pushes.
[^serve-rs]: stderr piped into StderrHubFeed; stdout Stdio::inherit.
[^driver-rs]: One LogHub shared by spawn_roc_with_logs and InspectorServer.
[^dispatch-rs]: Generated program does not import Stdout/Stderr by default.
[^inspector-rs]: Log snapshot, SSE, and clear; no ingest POST.
[^cli-readme]: Console is session runtime messages, not an app-level Rocci log API.
[^lower-rs]: @on emits Roc functions with try bodies; components lower to Html.
[^rocci-ref]: Recognized forms include @on and @init; no @log.
[^counter]: Effectful handlers versus pure CounterCard.
[^pure-render]: Effects stay outside @component lowering.
[^server-state]: Handlers own mutation and request lifecycle.
[^bws-stdout]: Platform documents Stdout.line! and Stderr.line! effects.
[^preview-research]: Three message classes; app/component log out of original Console v1 scope.
