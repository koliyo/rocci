---
type: Research Report
title: Path-addressed live streams for multi-page Rocci applications
description: "Complex generated apps need multiple explicit live GET routes, but streams should follow page or coherence boundaries rather than one connection per component; in verb-first syntax the regular form is @get:live(path)."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-08-22T09:52:04Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: template-ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Current singleton LiveDecl grammar
    author: process:git
    last_modified: 2026-08-22
  - id: template-parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Current pathless live parser
    author: process:git
    last_modified: 2026-08-22
  - id: template-validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Current duplicate-live and /sse collision validation
    author: process:git
    last_modified: 2026-08-22
  - id: template-lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Current LiveInfo lowering and fixed data-init injection
    author: process:git
    last_modified: 2026-08-22
  - id: template-pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: Current fixed /sse live inspection
    author: process:git
    last_modified: 2026-08-22
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Current singleton generated /sse dispatch and route merging
    author: process:git
    last_modified: 2026-08-22
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: Multi-module standalone app assembly
    author: process:git
    last_modified: 2026-08-22
  - id: run
    resource: ../../../crates/rocci-cli/src/run.rs
    title: Sibling Rocci module discovery and primary-module selection
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Current public singleton live contract
    author: process:git
    last_modified: 2026-08-22
  - id: server-reference
    resource: ../../../docs/reference/language/server.rocdown
    title: Current public server declaration reference
    author: process:git
    last_modified: 2026-08-22
  - id: runtime-reference
    resource: ../../../docs/reference/runtime.rocdown
    title: Current generated runtime and /sse contract
    author: process:git
    last_modified: 2026-08-22
  - id: live-concept
    resource: ../../../docs/concepts/one-shot-versus-live.rocdown
    title: Public one-shot versus live guidance
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Current singleton live counter declaration
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter-ui
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounterUi.rocci
    title: One live stream rendering multiple stable-ID fragments
    author: process:git
    last_modified: 2026-08-22
  - id: cqrs-research
    resource: ../datastar-cqrs-action-responses.md
    title: Generated CQRS streams and future named-region research
    author: process:cursor
    last_modified: 2026-08-21
  - id: bws-research
    resource: ../basic-webserver-sse-http.md
    title: basic-webserver SSE, polling, keepalive, and HTTP limits
    author: process:cursor
    last_modified: 2026-08-21
  - id: verb-first-research
    resource: ../verb-first-handler-declarations.md
    title: Verb-first handler declaration research
    author: process:cursor
    last_modified: 2026-08-22
  - id: implementation-plan
    resource: ../../plans/rocci/path-addressed-live-streams.md
    title: Implementation plan for path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
  - id: server-owned-state
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar backend actions and GET lifecycle
    author: organization:star-federation
  - id: datastar-sse
    resource: https://data-star.dev/reference/sse_events
    title: Datastar patch-elements and patch-signals SSE events
    author: organization:star-federation
  - id: datastar-attributes
    resource: https://data-star.dev/reference/attributes
    title: Datastar data-init lifecycle
    author: organization:star-federation
  - id: datastar-backend
    resource: https://data-star.dev/guide/backend_requests
    title: Datastar backend requests and multi-event SSE
    author: organization:star-federation
---

# Path-addressed live streams for multi-page Rocci applications

## Question and disposition

Rocci needs explicit stream paths if generated `.rocci` applications are to
support first-class multi-page and independently subscribed live regions. The
current fixed singleton `GET /sse` remains a good beginner default, but it
cannot identify which page or coherence boundary a stream serves, and current
app assembly cannot dispatch live declarations from sibling modules.

A path argument alone is not enough. If Rocci merely changed `@live` to
`@live(path)` while retaining one effective stream, authors could rename the
singleton but could not declare separate dashboard, notifications, and admin
streams. The required capability is:

```text
multiple live declarations
+ authored literal paths
+ app-level module binding and collision validation
+ deterministic subscription/injection rules
```

Because the companion language proposal already makes ordinary routes
verb-first, path-addressed live streams should use the same route grammar:

```rocci
@get:live("/streams/dashboard") = |state, request| {
    dashboardLive(...)
}

@get:live("/streams/notifications") = |state, request| {
    notificationsLive(...)
}
```

This revises the earlier exploratory recommendation to leave pathless `@live`
as a syntax exception. Once live declarations have authored paths and
multiplicity, they are route-shaped: GET is the transport method, `live` is the
generated response/lifecycle role, and the path is part of the public HTTP
contract.[^verb-first-research]

Do not interpret this as “one SSE stream per component.” Datastar can patch
multiple top-level elements from one event, and Rocci's live counter already
returns `#counter` and `#counter-feed` from one live render. Prefer one stream
per page or independently authorized/coherent update boundary. Split streams
only when subscriptions, authorization, cadence, visibility, or failure
isolation genuinely differ.[^datastar-sse][^datastar-backend]
[^live-counter][^live-counter-ui]

This is exploratory research. It does not approve or ship the syntax. The
linked [implementation plan](/plans/rocci/path-addressed-live-streams.md) defines
the clean cut and integration with the verb-first handler plan; no phase has
started.[^implementation-plan]

## Shipped baseline

The current `LiveDecl` has params and a body but no path. Parsing recognizes
only pathless `@live`, validation rejects a second declaration in the same
module, and lowering emits one function named `live!` plus a `LiveInfo` that
contains only the source span.[^template-ungram][^template-parser]
[^template-validate][^template-lower]

Generated behavior is hardcoded in several places:

- dispatch registers `GET /sse` and calls the primary module's `live!`;
- the poll loop renders every 100 ms, emits patch-elements on changed bytes,
  and emits a keepalive when unchanged;
- lowering injects `data-init=@get("/sse", [OpenWhenHidden(True)])` into a root
  body without an authored `data-init`;
- inspection reports every live declaration as method GET, path `/sse`, and
  role `live`.[^dispatch][^template-lower][^template-pprint]

The public contract accurately describes one live render per module and a
generated fixed `/sse` route.[^template-readme][^server-reference]
[^runtime-reference]

Current conceptual guidance also treats one-shot handlers as the default and
reserves live streams for updates that genuinely need a long-lived
subscription.[^live-concept]

### The multi-module limitation is stronger than the public wording

Standalone planning discovers the primary `.rocci` file and all sibling
`.rocci` files in the same directory. It merges ordinary routes from those
modules, skips sibling root routes, and uses the primary module for state,
initialization, and live metadata.[^run][^driver][^dispatch]

Consequently, sibling page routes can participate in generated dispatch, but
only `modules[0].live` is passed to the dispatcher. A sibling module may parse
and lower its own `@live`; that metadata is not bound as a stream route. This
is not merely a missing path spelling. The app-level live collection and
binding model is absent.[^driver]

Route merging also silently retains the first duplicate method/path pair. A
multi-stream design needs app-level duplicate diagnostics rather than relying
on first-wins behavior for public stream paths.[^dispatch]

## What multi-page applications actually need

“Different parts of the app” covers several distinct cases:

| Case | Recommended stream boundary |
| --- | --- |
| Several counters/cards on one dashboard update from the same state revision | One dashboard stream returning several stable-ID fragments |
| `/dashboard` and `/admin` expose different data and authorization | Separate page-scoped paths |
| A notification tray persists across several pages | One shell/notification stream subscribed by those pages |
| A costly chart should connect only while visible | Separate explicitly mounted stream or a custom server when lifecycle control exceeds generated behavior |
| Two regions need different poll cadence or custom events | Authored `main.roc` until generated stream options are deliberately designed |
| One page wants a query-selected variant of the same renderer | One path plus request/query discrimination may be sufficient |

The design unit is an independently subscribed **coherence boundary**, not an
HTML component. The backend remains authoritative, and each stream renders a
coherent snapshot of its boundary.[^server-owned-state]

## Why one global stream is insufficient as the only model

A single stream can technically return a fragment containing every live ID in
the application. Datastar patch-elements matches top-level elements by ID and
can patch more than one element.[^datastar-sse]

Using that as the only multi-page strategy has poor consequences:

- every subscriber causes the server to read and render unrelated page state;
- page-specific authorization is hidden inside one large renderer;
- stream logs and failures identify only `/sse`, not the affected page;
- reverse-proxy, rate-limit, and observability policy cannot distinguish
  stream purposes;
- the response may contain HTML for boundaries absent from the current page;
- changing one unrelated region changes the whole rendered-byte hash and emits
  the combined patch.

One global stream remains reasonable for a small app shell. It should not be
the only generated topology.

## The current query-multiplexing escape hatch

The existing singleton can already inspect its `request`. An author can set
explicit subscriptions such as:

```rocci
<body data-init=@get("/sse?page=dashboard", [OpenWhenHidden(True)])>
```

and branch inside the one live body based on the query. Multiple elements can
also open separate GET requests to variants of the same endpoint.

That means a new path is not required for raw expressive power. Query
multiplexing is nevertheless a weak first-class contract:

- inspection and route inventories still expose only `/sse`;
- every variant shares one generated function and one collision identity;
- page/region dispatch becomes hand-written inside an opaque Roc body;
- automatic injection cannot choose the query;
- authorization and operational policy remain concentrated on one endpoint.

Keep query discrimination as a useful local optimization, not the primary
multi-page abstraction.

## Syntax options

| Option | Strength | Failure |
| --- | --- | --- |
| Keep pathless singleton `@live` | Smallest beginner surface | No first-class page/region identity or sibling streams |
| Add one configurable `@live(path)` | Makes the singleton route explicit | Renames rather than solves multiplicity |
| Allow repeated `@live(path)` | Multiple explicit streams with compact syntax | Becomes an exception beside verb-first routes |
| Add repeated `@get:live(path)` | Aligns Datastar, HTTP inspection, and verb-first roles | Revises the earlier decision to keep live syntax separate |
| Add named regions under one `/sse` | Can optimize several IDs inside one connection | Names DOM outputs, not independently subscribed routes |
| Generic GET returning an SSE response ADT | Maximum flexibility | Makes generated polling and the common Html contract ceremonial |
| Configuration-only stream table | Keeps grammar smaller | Separates renderer body from route identity and weakens local readability |

Repeated `@get:live(path)` is recommended. The new requirement changes the
premise behind the prior exception: a pathless singleton was a module
capability, while multiple authored paths are normal public routes with a
special response lifecycle.

## Recommended language contract

Extend the proposed legal method-role matrix by one pair:

| Method | Role | Cardinality | Successful body value | Generated behavior |
| --- | --- | --- | --- | --- |
| GET | `view` | many | complete `Html` document | finite `text/html` |
| GET | `fragment` | many | stable-ID `Html` | finite one-shot morph |
| GET | `live` | many | stable-ID `Html` | long-lived polling SSE |
| POST/PUT/PATCH/DELETE | `fragment` | many | stable-ID `Html` | finite one-shot morph |
| POST/PUT/PATCH/DELETE | `command` | many | `{}` | representation-free success |

Only GET may pair with `live`. Reject `@post:live`, bare `@live`,
`@live(path)`, and pathless `@get:live`. The clean migration is:

```text
@live  ->  @get:live("/sse")
```

The path is a literal and participates in the same method/path collision
rules as documents and fragments. A live handler keeps the ordinary generated
arity `handler!(context, request)` so it can authorize and discriminate each
subscription.

## Recommended subscription and injection rules

Stream declaration and stream subscription are separate facts:

- `@get:live(path)` declares what a GET to that path streams.
- `data-init=@get(path, options)` declares which page or region subscribes.

Automatic injection remains useful only when association is unambiguous:

1. If a module declares exactly one live route, a root `<body>` without
   `data-init` may receive that route's authored path plus
   `OpenWhenHidden(True)`, preserving beginner ergonomics.
2. If a module declares more than one live route, lowering performs no
   automatic subscription. Authors place explicit `data-init` attributes on
   the relevant document shell or stable region.
3. If an author already supplied `data-init`, lowering never merges or
   replaces it.
4. A sibling module's stream is never injected into another module's view by
   filename or path convention; cross-module subscriptions are explicit.

Datastar runs `data-init` on initial load and can re-run it when an element or
the attribute is patched. A subscription should therefore live on a stable
outer element that its own stream does not replace, usually the document body
or an unpatched shell.[^datastar-attributes]

Datastar GET requests close by default while a page is hidden and reopen when
visible. Generated injection currently opts into `OpenWhenHidden(True)` so
background tabs remain shared-view subscribers. Explicit subscriptions should
make that lifecycle choice visible rather than receiving a hidden global
default.[^datastar-actions]

## Multiple streams on one page

A page can subscribe to more than one route by placing `data-init` on separate
stable elements:

```rocci
<body>
    <main
        id="dashboard-shell"
        data-init=@get("/streams/dashboard", [OpenWhenHidden(True)])
    >
        ...
    </main>
    <aside
        id="notification-shell"
        data-init=@get("/streams/notifications", [OpenWhenHidden(True)])
    >
        ...
    </aside>
</body>
```

This should be used deliberately. Each route opens another long-lived HTTP
response and runs another generated poll loop. With the current host, each loop
renders every 100 ms and needs unchanged-path keepalives because silent waits
hit the response idle timeout.[^dispatch][^bws-research]

The generated local server is plaintext HTTP/1.1 in browsers. Multiple
long-lived streams therefore consume more connection and server resources than
one page-level stream; the implementation should measure rather than encourage
fine-grained stream proliferation.[^bws-research]

## App-level metadata and dispatch requirements

The normalized stream metadata must become route-like and plural:

```text
LiveInfo {
    method: GET,
    path,
    fn_name,
    span,
}
```

`LoweredModule.live: Option<LiveInfo>` becomes a collection. App assembly must
bind streams from the primary and sibling modules with the module type that
owns each generated function. Generated dispatch then emits one live arm per
unique path and calls `Module.function!(context, request)`.[^template-lower]
[^driver][^dispatch]

App-level validation must reject collisions among:

- live versus live at the same GET path;
- live versus document GET;
- live versus finite GET fragment;
- generated infrastructure routes such as `/health` where applicable.

Do not silently discard a sibling stream or ordinary route because another
module was discovered first. The first-wins merge behavior is especially
dangerous for authorization-sensitive streams.[^dispatch]

## Naming and inspection

Generated function names should derive from method and path using the same
deterministic route-name machinery as other handlers, with explicit detection
of distinct paths that normalize to the same Roc identifier. A fixed `live!`
cannot represent multiplicity.

Inspection, handler logs, diagnostics, and source maps should report:

```text
live GET "/streams/dashboard" live
live GET "/streams/notifications" live
```

The leading method and authored path make stream traffic identifiable in
browser tools, server logs, route inventories, and reverse-proxy policy.

## Authorization and data boundaries

Separate paths make authorization boundaries visible; they do not implement
authorization. Each live body receives the request and must authenticate and
authorize before reading or rendering protected state. A stream may remain
open for a long time, so custom applications with revocation or per-event auth
needs may still require authored stream machinery.

Do not send page-specific or privileged HTML through one global stream and
assume absent DOM IDs provide a security boundary. DOM targeting is rendering
behavior, not authorization.[^server-owned-state]

## Relationship to named regions

Named regions and path-addressed streams solve different problems:

- a **path** identifies one subscription and operational/auth boundary;
- a **stable DOM ID** identifies one morph target within events on that
  subscription;
- a future named-region optimization may hash or emit several targets
  independently while retaining one path.

The live counter already proves that one live result can contain several IDs.
Implement path-addressed streams first. Do not add region-selector syntax in
the same grammar change.[^live-counter-ui][^cqrs-research]

## Consequences

- Multi-page generated apps can own page-scoped streams without an authored
  dispatcher.
- A shared shell can subscribe to one stream across several pages.
- Independent regions can use separate connections when their lifecycle or
  policy actually differs.
- The handler matrix becomes fully regular: `@get:live(path)` is a GET route
  with a live response role.
- The former pathless singleton becomes a clean-cut removal with an exact
  `/sse` rewrite.
- Automatic injection remains simple for one local stream and becomes explicit
  for multiple streams.
- App assembly must stop ignoring sibling live metadata and stop silently
  resolving path collisions by discovery order.
- More streams multiply polling, rendering, keepalive, connection, and SQLite
  work. Documentation must recommend coarse coherence boundaries.
- Custom `main.roc` remains the ceiling for pub/sub wakeups, custom cadence,
  mixed event types, replay, resumability, or sophisticated lifecycle policy.

## Recommendation

Add path-addressed live support, but define the feature as **multiple explicit
live GET routes**, not as a configurable singleton:

```rocci
@get:live("/streams/dashboard") = |state, request| {
    dashboardLive(...)
}
```

Allow many across the generated app, bind sibling-module streams, and validate
collisions app-wide. Auto-inject only a module's sole stream; require explicit
subscriptions when association is ambiguous. Teach one page/coherence stream
as the default because one SSE event can patch multiple stable-ID elements.

This is the smallest design that genuinely supports multi-page applications
while preserving Rocci's server-rendered HTML boundary and avoiding a
connection-per-component architecture.[^implementation-plan]

[^template-ungram]: `LiveDecl` currently has no path and appears as a separate top-level node.
[^template-parser]: Current parser accepts only pathless `@live` plus optional params/body.
[^template-validate]: Current validation rejects duplicate live declarations per module and reserves fixed `/sse`.
[^template-lower]: Current `LiveInfo` stores only a span, lowers one `live!`, and injects fixed `/sse`.
[^template-pprint]: Handler inspection hardcodes live GET `/sse`.
[^dispatch]: Generated dispatch accepts one optional live, hardcodes one `/sse` arm, polls every 100 ms, and first-wins route merge deduplicates method/path.
[^driver]: App assembly merges sibling routes but passes only the primary module's optional live metadata to dispatch.
[^run]: Standalone planning discovers sibling `.rocci` files in the primary file's directory and moves the primary module first.
[^template-readme]: Owning crate documents pathless, one-per-module `@live` and fixed injection.
[^server-reference]: Public declaration matrix says one pathless live per module.
[^runtime-reference]: Runtime reference exposes fixed generated `/sse`.
[^live-concept]: Public guidance treats live as one extra polling route and recommends one-shot by default.
[^live-counter]: Current example declares one live renderer for generated `/sse`.
[^live-counter-ui]: `LiveSlice` returns both counter and feed stable-ID fragments from one live render.
[^cqrs-research]: Earlier research proposes named regions as a later optimization and records generated polling limits.
[^bws-research]: Current host needs keepalives, polls each connection, and serves browser preview over HTTP/1.1.
[^verb-first-research]: Companion syntax research uses method plus response role but originally kept singleton live as an exception.
[^implementation-plan]: Phased grammar, metadata, app assembly, injection, tooling, example, and validation proposal.
[^server-owned-state]: Durable state stays server-owned and streams emit coherent server-rendered HTML boundaries.
[^datastar-actions]: GET uses Fetch; hidden pages close/reopen by default unless `openWhenHidden` is enabled.
[^datastar-sse]: Patch-elements events can patch one or more top-level elements by ID.
[^datastar-attributes]: `data-init` runs on initialization and may re-run when patched attributes/elements change.
[^datastar-backend]: One SSE response can contain multiple element and signal events.
