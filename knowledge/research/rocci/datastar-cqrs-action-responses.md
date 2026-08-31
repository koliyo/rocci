---
type: Research Report
title: Datastar SSE is a per-request transport; generated apps do not fan out
description: "Datastar morphs from each request's response. Generated Rocci one-shot POST patches do not fan out. Multi-client push is Tao CQRS. Generated @live polls with keepalives; Datastar @command success is empty SSE (not 204); ordinary clients get JSON. Platform idle-timeout limits are in basic-webserver-sse-http."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/rendering]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated empty_sse commands and live keepalive Emit
    author: process:git
    last_modified: 2026-08-21
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Datastar.patch_elements emits datastar-patch-elements
    author: process:git
    last_modified: 2026-08-15
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Standalone SQLite counter with @post increment
    author: process:git
    last_modified: 2026-08-20
  - id: counter-readme
    resource: ../../../examples/rocci/standalone/counter/README.md
    title: First-app counter documents one-shot patch and curl SSE
    author: process:git
    last_modified: 2026-08-21
  - id: hybrid-counter
    resource: ../../../examples/rocdown/counter/index.rocdown
    title: Hybrid counter island with @live and json commands
    author: process:git
    last_modified: 2026-08-21
  - id: snake-main
    resource: ../../../examples/rocci/custom/snake/main.roc
    title: Authored CQRS - GET /sse stream and POST /api/direction empty SSE
    author: process:git
    last_modified: 2026-08-20
  - id: snake-rocci
    resource: ../../../examples/rocci/custom/snake/Snake.rocci
    title: Play page opens GET /sse from data-init
    author: process:git
    last_modified: 2026-08-20
  - id: snake-readme
    resource: ../../../examples/rocci/custom/snake/README.md
    title: Snake documents long-lived SSE and JSON direction API
    author: process:git
    last_modified: 2026-08-20
  - id: server-actions
    resource: ../../../docs/guides/server-actions.rocdown
    title: Public server-actions guide covers one-shot and @live
    author: process:git
    last_modified: 2026-08-21
  - id: rendering-doc
    resource: ../../../docs/concepts/rendering-model.rocdown
    title: Rendering model includes generated @live /sse
    author: process:git
    last_modified: 2026-08-21
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Standalone HTTP documents @live and empty SSE commands
    author: process:git
    last_modified: 2026-08-21
  - id: rocci-ref
    resource: ../../../docs/reference/rocci.rocdown
    title: Public @view, @patch, @command, and @live contract
    author: process:git
    last_modified: 2026-08-21
  - id: lower-rs
    resource: ../../../crates/rocci-template/src/lower.rs
    title: RouteInfo respond kind and live data-init injection
    author: process:git
    last_modified: 2026-08-21
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: LiveDecl and handler grammar
    author: process:git
    last_modified: 2026-08-21
  - id: service-rs
    resource: ../../../crates/rocci-rocdown/src/service.rs
    title: Live islands reuse rocci-cli generated dispatch
    author: process:git
    last_modified: 2026-08-20
  - id: server-state
    resource: ../../decisions/server-owned-state.md
    title: Durable application state is server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: author-skill
    resource: ../../../.agents/skills/rocci-author/SKILL.md
    title: Authoring table for handlers and empty SSE commands
    author: process:git
    last_modified: 2026-08-21
  - id: bws-limits
    resource: ../basic-webserver-sse-http.md
    title: basic-webserver 0.16 SSE idle timeout and HTTP/1.1 limits
    author: process:cursor
    last_modified: 2026-08-21
  - id: ds-backend
    resource: https://data-star.dev/guide/backend_requests
    title: Datastar backend requests and SSE events
    author: organization:star-federation
  - id: ds-tao
    resource: https://data-star.dev/guide/the_tao_of_datastar
    title: Tao of Datastar - CQRS, 0-n SSE events, signals sparingly
    author: organization:star-federation
  - id: ds-sse
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event types
    author: organization:star-federation
  - id: ds-actions
    resource: https://data-star.dev/reference/actions
    title: Backend actions, empty body, response content types
    author: organization:star-federation
  - id: ds-docs
    resource: https://data-star.dev/docs.md
    title: Combined Datastar docs including CQRS and Accept handling
    author: organization:star-federation
  - id: plan
    resource: ../../plans/rocci/datastar-cqrs-action-responses.md
    title: Implementation plan for generated CQRS and JSON commands
    author: process:cursor
    last_modified: 2026-08-21
---

# Datastar SSE is a per-request transport; generated apps do not fan out

## Scope and authority

This record explains the counter's observed network shape against Datastar 1.0
and Rocci's generated dispatcher. Generated `@live` and `@command` negotiation
(empty SSE for Datastar, JSON for ordinary clients, keepalives on unchanged
polls) are shipped in dispatch; platform idle-timeout limits remain
documented separately.[^server-state][^plan][^dispatch-rs][^bws-limits]

Exploratory on further fan-out ceilings. Companion
[plan](/plans/rocci/datastar-cqrs-action-responses.md). Platform limits:
[basic-webserver SSE and HTTP](basic-webserver-sse-http.md).

## Observation

Two browsers on the standalone or hybrid counter share SQLite, but an
Increment in one tab does not update the other. DevTools shows the POST to
`/actions/counter/increment` returning `text/event-stream` with a single
`datastar-patch-elements` event. There is no other open Datastar stream.[^counter][^hybrid-counter][^dispatch-rs]

That matches shipped generation, not a Datastar client bug. It also matches
the public guide, which tells authors that mutation handlers return one-shot
patch events and that two windows only prove the **store** is shared.[^server-actions]

## Two different uses of SSE

Datastar uses Server-Sent Events as the **encoding of one HTTP response**,
not as an implicit multi-client bus.[^ds-backend][^ds-sse]

| Pattern | Connection | Who updates | Datastar name |
| --- | --- | --- | --- |
| One-shot action | POST/GET opens, 0–n events, then closes | Only the requester | Direct backend action |
| Sequential events on one client | Same request stays open; later events follow | Only that client | Streaming one response (HAL demo) |
| Shared live view | Long-lived **GET** stays open; writes are other requests | Every subscriber | Tao **CQRS** |

The HAL homepage demo streams two patches down **one** click's GET. That is
not fan-out to a second browser. Fan-out is the CQRS example: `data-init=@get('/cqrs_endpoint')` for reads, `@post('/do_something')` for writes.[^ds-tao][^ds-docs]

Rocci's generated apps only implement the first row. Snake implements the
third, in an authored `main.roc`. The published rendering model already lists
that third row as “authored long-lived SSE,” and the Snake architecture
report already split `GET /sse` from JSON direction POSTs; generated `@on`
never grew the same split.[^snake-main][^snake-rocci][^rendering-doc]

## Current generated contract

`@on` lowering records method, path, and function name. Generated `respond!`
then branches on method only:[^lower-rs][^ungram][^dispatch-rs][^rocci-ref]

- **GET** → `text/html` document (`html_ok(Html.render(html))`).
- **Any other method** → `patch_html!`: one `Datastar.patch_elements` event
  inside `Sse.unfold!` that Emits then Ends.[^datastar-roc]

There is no generated path that keeps a GET open, returns JSON, or returns
204. `GET /sse` authored as `@on:get("/sse")` would be served as a **document**,
not a stream. Live Rocdown islands reuse this dispatcher, so the hybrid
counter has the same ceiling.[^service-rs][^template-readme]

The crate README already names this as a POC skip: handler bodies do not
return `Server.Outcome` or `Sse.Event`; custom long-lived SSE stays in
authored `main.roc`. The authoring skill still lists `/sse` that way.[^template-readme][^author-skill]

## Datastar's actual response rules

A backend action (`@get` / `@post` / `@put` / `@patch` / `@delete`) may
complete in several legal ways:[^ds-actions][^ds-docs]

| `Content-Type` / status | Datastar client effect |
| --- | --- |
| `text/event-stream` | 0–n `datastar-patch-elements` / `datastar-patch-signals` / script events |
| `text/html` | Morph top-level elements by `id` (optional `datastar-*` headers) |
| `application/json` | **Patch frontend signals** (JSON Merge Patch), not HTML |
| `text/javascript` | Execute script |
| **204 No Content** | Success, empty body; no morph |

SSE is allowed to be empty (`0` events). Tao prefers `text/event-stream` for
Datastar clients because one encoding covers short and long responses, but
HTML and 204 are first-class.[^ds-tao][^ds-sse]

Default `@get` **closes when the tab is hidden** and reconnects on visible.
Shared views must pass `openWhenHidden: true` or a background tab drops the
read stream.[^ds-actions]

Requests send `Datastar-Request: true` and
`Accept: text/event-stream, text/html, application/json`. Accept therefore
**cannot** distinguish a Datastar browser from `curl`. The dedicated header
can.[^ds-docs]

## Should the increment POST respond?

Yes, the HTTP request must finish. No, it does not need to carry a DOM
patch.

- Datastar treats **204** or **zero SSE events** as success.[^ds-actions]
- Tao CQRS: the long-lived GET is the read/update channel; short POSTs are
  commands. Loading indicators should hide when the **stream** morphs the
  DOM, not when the POST returns.[^ds-tao]
- Do **not** patch the same `#counter` from both the POST and the stream
  unless events are versioned. A late POST can overwrite a newer streamed
  render.

Snake already does this: `POST /api/direction` writes SQLite and returns
`empty_sse!` (unfold immediately `End`). `GET /sse` polls revision and emits
`patch_elements` to every player and spectator.[^snake-main][^snake-readme]

## JSON for API clients is not a Datastar HTML patch

Returning `application/json` `{ "count": 5 }` from `@post` would make
Datastar merge `$count` into **signals**. The counter UI is server-rendered
`{count.to_str()}` inside `#counter`, not `data-text="$count"`. The acting
tab would not morph the output from that JSON, and domain state would leak
into the client store, against Tao's "use signals sparingly" and Rocci's
server-owned model.[^ds-actions][^ds-tao][^server-state][^counter]

`Accept` is also the wrong switch: Datastar lists JSON in Accept on every
action. Discriminate on `Datastar-Request`:[^ds-docs]

| Client | Header | Recommended command response |
| --- | --- | --- |
| Datastar `@post` | `Datastar-Request: true` | **Empty SSE** (HTTP 200 `text/event-stream`, zero events). Stream morphs HTML. |
| `curl`, API, tests | absent | **200** `application/json` with the new resource (for a counter, `{ "count": N }`) |

Datastar also accepts **204**. Generated Rocci prefers empty SSE so Safari
inspector and basic-webserver stay quiet; Snake already used that shape.[^snake-main][^bws-limits][^dispatch-rs]

Same `@command` path, two encodings. The handler returns data; dispatch chooses
status and content type. That is how action endpoints stay usable as a
normal API without teaching Datastar that JSON is HTML.

Form **validation** that must appear only for the acting user is a different
flow: keep a **direct patch** (or a signal `$error`) on that POST. Do not
broadcast another user's field errors over the shared stream.

## Recommended wire shape for a shared view

Keep durable state in SQLite. Keep `#counter` as the morph boundary. Split
read and write:[^server-state][^ds-tao]

```text
GET  /                         document (initial HTML)
GET  /sse                      generated long-lived patches of the live region (keepalives when unchanged)
POST /actions/counter/increment command: write + empty SSE or JSON (not a second patch)
POST /actions/counter/reset     same
```

The stream belongs on `body` (or a dedicated node), not on the Increment
button, so a POST does not cancel the GET under default
`requestCancellation`. Shared views need `openWhenHidden: true`.[^ds-actions]

Hybrid CDN HTML can keep snapshot `count = 0`. The stream's **first** event
replaces `/actions/counter/sync`.[^hybrid-counter]

Direct one-shot patches remain correct for **single-viewer** forms (gallery
search, click-to-edit, inline validation). Those should not be forced onto
CQRS.

## What authors must write versus what Rocci can generate

Authors must never write `Sse.unfold!`, `Wait`, `After`, or event framing.
That already lives in generated `main.roc` for one-shot patches and is what
Snake copies by hand because it ships an authored `main.roc`.[^dispatch-rs][^snake-main][^template-readme]

The leftover authoring is only **which HTML is live** and **that POSTs are
commands**. Those are product facts Rocci cannot infer without breaking
forms. Inferring CQRS from “there is a POST” would turn gallery validation
into a broadcast bus.

| Layer | Who | Notes |
| --- | --- | --- |
| SSE framing, poll loop, 204 vs JSON encoding | **Rocci** | Always generate. |
| Opt-in “this app has a shared live region” | **Author** | Explicit. Default stays one-shot. |
| Read+render of that region | **Author** | Effectful load, then a pure `@component`. |
| `data-init` + `OpenWhenHidden` | **Rocci** (short/medium) | Inject on the document body when live is on. |
| Command JSON body | **Author** (short); Encode later | Needed if curl should see `{ "count": N }`. |

`@component` stays pure. A live loader is `@on`-shaped (effects allowed),
not a component.[^server-state]

### Short term (this plan)

Ship an opt-in **live render**, not a second handwritten route:

```text
@live = |{ db }| {
    count = read_count!(db)?
    counterCard({ count })
}
```

Rocci then:

1. Generates `GET /sse` as a Snake-style poll of that Html, emitting
   `datastar-patch-elements` when render bytes change and a non-Datastar
   keepalive when unchanged (so the host 30s response idle timeout cannot kill
   a silent `Wait`).[^dispatch-rs][^bws-limits]
2. Injects `data-init=@get("/sse", [OpenWhenHidden(Bool.true)])` on document
   `body` when the module has `@live` and the body has no `data-init`.
3. Leaves unmarked `@patch` POST as today’s one-shot patch (forms keep working).
4. Lets `@command` return Roc data: empty SSE for Datastar-Request, JSON for
   everyone else.

An explicit authored `GET /sse` remains an escape hatch, not the
tutorial. Snake keeps authored `main.roc` (ticks, cookies, custom JSON
API).[^snake-main]

### Medium term

Drop `@live` for the common single-page app: `rocci.toml` `[datastar] live = true`
(or a module flag) re-runs the document `GET /` and fat-morphs `html` /
`#main`. Tao already prefers fat morph over hand-picked fragments. Cost is
re-rendering the whole page each poll. Multi-page apps pass the current
path as a query on `/sse`.[^ds-tao]

Command POSTs in live mode can return `{}` / JSON only; the stream is the
read path. That **reduces** authoring versus today’s increment handler,
which both mutates and returns `counterCard`.[^counter]

### Long term

- Named regions (`@live #counter`, `@live #hud`) so one stream emits only
  changed hashes (Snake already patches three ids from one unfold).
- Process-local revision: POSTs bump an integer; streams skip Html until it
  changes (cheaper than hashing markup every 100 ms). Still a timer `Wait`
  until the platform can wake other connections.
- Roc `Encode` for JSON commands; drop authored JSON strings.
- Platform mailbox so a write wakes parked streams instead of polling.

## Split the examples

The standalone counter is the **first app**: one file, one-shot patches,
and the README teaches `curl` to expect `datastar-patch-elements`. That
story is valid Datastar. Converting it would hide the simple pattern and
break the documented smoke check.[^counter][^counter-readme][^server-actions]

| Example | Role | Live stream |
| --- | --- | --- |
| `examples/rocci/standalone/counter` | Tutorial: `@on`, SQLite, fragment patch | No. Keep one-shot. |
| New `examples/rocci/standalone/live-counter` | Tutorial: two browsers, CQRS, JSON POST | Yes (`@live`). |
| `examples/rocdown/counter` | Public hybrid “shared count” | Yes. Two visitors is the demo. |
| `examples/rocci/custom/datastar` | Forms, search, tabs | No. Direct patches. |
| `examples/rocci/custom/snake` | Authored stream + game loop | Already live; do not generate. |

Hybrid should switch because its product claim is a shared island, not
because every Rocci app is live. The simple counter README should say
explicitly that a second tab will not move until click or refresh; point
at live-counter for fan-out.

## Platform ceiling: poll, not pub/sub

`basic-webserver` 0.16 typed SSE is `Emit` / `Wait` / `End` per connection.
Snake fans out by parking `Wait({ wake: After(125) })` and rereading a
SQLite `revision`. There is no cross-request mailbox to wake other
streams.[^snake-main]

A pure `Wait` with no bytes also hits the host **30s response idle timeout**.
Generated `@live` therefore keepalive-Emits on the unchanged path. Further
host shortcomings (HTTP/1.1 on plaintext `rocci run`, opaque Body-stream
logs, shared SQLite with blocking handlers) are catalogued in
[basic-webserver SSE and HTTP](basic-webserver-sse-http.md).[^bws-limits][^dispatch-rs]

A generated `/sse` handler can use the same loop: render the fragment,
emit when the bytes (or a revision column) change, otherwise wait ~100 ms.
That is push from the browser's point of view and poll inside the host. A
true in-process pub/sub is out of scope until the platform grows one.

## What not to do

- Treat "Datastar architecture" as "every POST is a broadcast SSE bus."
- Return domain JSON to Datastar `@post` expecting `#counter` to morph.
- Bind the visible count to `$count` just to reuse JSON (duplicates
  authority in the browser).[^ds-tao][^server-state]
- Auto-wrap **all** GET handlers as streams (`GET /` must stay a document).
- Infer CQRS from "there is a POST" (breaks gallery-style direct patches).
- Invent a retained server VDOM or per-tab LiveView process for v1.

## Open product questions

1. **`@live` spelling** versus a `rocci.toml` flag for the first ship. The
   companion plan picks module `@live` so the render stays next to the
   card; toml-only live is medium-term fat morph.[^plan][^ungram]
2. **JSON body type**: authored `Str` versus Roc `Encode` of a record.
3. **Poll interval** for generated streams (snake uses 125 ms).
4. Whether **error** responses for JSON commands should stay the HTML
   overlay for Datastar and JSON `{ "error": ... }` for API clients.
5. Whether injecting `data-init` is refused when the author already set one
   (recommended) or merged.

## Disposition

The increment SSE body is working as generated. It is the wrong pattern for
a **shared** view and the **right** pattern for the first-app counter.
Datastar best practice for fan-out is CQRS. The companion plan generates the
stream machinery: authors write `@live` plus `json` commands. On
`datastar-cqrs-action-responses` (not CI-complete), `examples/rocci/standalone/counter`
stays one-shot; `live-counter` and the hybrid island share the stream. Snake
stays the hand-written ceiling.[^dispatch-rs][^snake-main][^counter-readme][^plan]

[^dispatch-rs]: `@patch` arms call `Ok(patch_html!(html))`. `@command` Datastar success is `empty_sse!`; ordinary clients get `json_ok`. `@live` emits patch-elements or keepalive `Sse.Event.data("")`.
[^datastar-roc]: `patch_elements` is `Sse.Event.keyed("datastar-patch-elements", "elements", ...)`.
[^counter]: Increment/reset return `counterCard`; buttons use `@post("/actions/counter/...")`; lede text says one HTML patch.
[^counter-readme]: First-app README documents curl of the POST SSE patch and single-event morph of `#counter`.
[^hybrid-counter]: Converted to `@live` plus `@command` increment/reset; `/actions/counter/sync` removed.
[^snake-main]: `GET /sse` → `stream_game!` revision poll; `POST /api/direction` → `empty_sse!`.
[^snake-rocci]: `<body data-init=@get("/sse")>` on the play page.
[^snake-readme]: Documents long-lived SSE patches and JSON `POST /api/direction`.
[^server-actions]: Live CQRS commands return empty SSE or JSON; `@patch` stays one-shot.
[^rendering-doc]: Table lists `@live` stream and `@command` empty SSE separately from POST fragments.
[^template-readme]: Documents `@live` keepalives and empty SSE for Datastar commands.
[^rocci-ref]: `@command` encodes empty SSE for Datastar and JSON otherwise.
[^lower-rs]: `RouteInfo` respond kind and live `data-init` injection.
[^ungram]: Handler and `LiveDecl` grammar.
[^service-rs]: `IslandServicePlan.into_app_plan` feeds `rocci_cli` generic dispatch.
[^server-state]: Backend remains authoritative; Datastar transports intent and server HTML.
[^author-skill]: Server-app table lists empty SSE for Datastar `@command`.
[^bws-limits]: Silent `Wait` vs 30s idle timeout; HTTP/1.1 on plaintext `rocci run`; opaque Body-stream logs.
[^ds-backend]: Zero or more SSE events per response; SDKs format `PatchElements` / `PatchSignals`.
[^ds-tao]: CQRS long-lived GET plus short writes; 0–n SSE events; signals for interaction not domain.
[^ds-sse]: `datastar-patch-elements` morphs by id; events end with a blank line.
[^ds-actions]: 204 or empty SSE allowed for empty morph; `openWhenHidden`; JSON content type patches signals.
[^ds-docs]: CQRS snippet; Accept lists html, json, and event-stream together.
[^plan]: Phased `@live` generation, empty SSE / JSON commands, live-counter example; first-app counter stays one-shot.
