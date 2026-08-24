---
type: Research Report
title: Implemented method:role handlers compared with the Datastar ecosystem
description: "Post-landing investigation: Rocci's closed @method:role(path) matrix is shipped and unique as language syntax. Datastar SDKs and app frameworks share CQRS and HTML-patch architecture but leave response roles to handler bodies. The matrix is on the right track for hypermedia DX, with a hard ceiling on Datastar's full SSE and SDK surface."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/developer-experience, concern/language-design]
status: draft
generated: { by: process:cursor, at: 2026-08-23T19:50:00Z }
stale_after: 2026-11-23
authority: exploratory
owners: [human:nils]
sources:
  - id: handler-contract
    resource: ../../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen closed method-role matrix and command wire policy
    author: process:git
    last_modified: 2026-08-22
  - id: handler-syntax
    resource: ../../../crates/rocci-template/tests/handler_syntax.rs
    title: Accepted and rejected handler syntax tests
    author: process:git
    last_modified: 2026-08-22
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Verb-first route parser, recovery, and removal diagnostics
    author: process:git
    last_modified: 2026-08-23
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Closed legal_pair matrix and duplicate path checks
    author: process:git
    last_modified: 2026-08-22
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Route lowering, request-param injection, live data-init injection
    author: process:git
    last_modified: 2026-08-22
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: RouteDecl View, Fragment, Command, and Live productions
    author: process:git
    last_modified: 2026-08-22
  - id: ast-params
    resource: ../../../crates/rocci-template/src/ast.rs
    title: Handler arity and unused _request injection
    author: process:git
    last_modified: 2026-08-22
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated document, fragment, command, and live dispatch
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Public standalone HTTP handler contract
    author: process:git
    last_modified: 2026-08-23
  - id: server-ref
    resource: ../../../docs/reference/language/server.rocdown
    title: Public server declaration reference
    author: process:git
    last_modified: 2026-08-22
  - id: handlers-guide
    resource: ../../../docs/applications/handlers.rocdown
    title: Application-layer handler guide
    author: process:git
    last_modified: 2026-08-22
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-shot GET view plus POST fragment counter
    author: process:git
    last_modified: 2026-08-23
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: GET view, POST command, and GET live counter
    author: process:git
    last_modified: 2026-08-23
  - id: handler-matrix
    resource: ../../../examples/rocci/standalone/handler-matrix/HandlerMatrix.rocci
    title: Complete accepted method-role matrix example
    author: process:git
    last_modified: 2026-08-23
  - id: custom-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Authored dispatcher with prefix-matched GET fragments
    author: process:git
    last_modified: 2026-08-23
  - id: verb-first
    resource: verb-first-handler-declarations.md
    title: Pre-landing verb-first handler research
    author: process:cursor
    last_modified: 2026-08-22
  - id: verb-first-plan
    resource: ../../plans/rocci/verb-first-handler-declarations.md
    title: Verb-first handler implementation plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: live-path
    resource: path-addressed-live-streams.md
    title: Path-addressed live stream research
    author: process:cursor
    last_modified: 2026-08-22
  - id: cqrs-research
    resource: datastar-cqrs-action-responses.md
    title: Datastar SSE versus generated fan-out
    author: process:cursor
    last_modified: 2026-08-21
  - id: bws-sse
    resource: basic-webserver-sse-http.md
    title: basic-webserver SSE idle-timeout and HTTP/1.1 limits
    author: process:cursor
    last_modified: 2026-08-22
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: impl-status
    resource: ../../status/implementation.md
    title: Implementation status snapshot
    author: process:cursor
    last_modified: 2026-08-22
  - id: datastar-docs
    resource: https://data-star.dev/docs.md
    title: Datastar documentation bundle, including CQRS and response handling
    author: organization:star-federation
  - id: datastar-backend
    resource: https://data-star.dev/guide/backend_requests
    title: Datastar backend requests guide
    author: organization:star-federation
  - id: datastar-sse
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event reference
    author: organization:star-federation
  - id: datastar-sdks
    resource: https://data-star.dev/reference/sdks
    title: Official Datastar language SDKs
    author: organization:star-federation
  - id: datastar-adr
    resource: https://github.com/starfederation/datastar/blob/develop/sdk/ADR.md
    title: Datastar SDK architecture decision record
    author: organization:star-federation
  - id: stario-arch
    resource: https://stario.dev/docs/explanation/go-to-architecture
    title: Stario go-to architecture with Datastar CQRS
    author: organization:stario
  - id: lambda-cqrs
    resource: https://lambda-combine.net/guide-cqrs
    title: Lambda Combine Datastar CQRS guide
    author: organization:lambda-combine
  - id: lambda-tao
    resource: https://lambda-combine.net/hyper-guide
    title: Lambda Combine hypermedia guide and Tao of Datastar
    author: organization:lambda-combine
  - id: dstar
    resource: https://github.com/ricotrevisan/dstar
    title: Dstar batteries-included Datastar toolkit for Elixir
    author: human:ricotrevisan
  - id: datastar-go
    resource: https://pkg.go.dev/github.com/starfederation/datastar-go/datastar
    title: Official Datastar Go SDK
    author: organization:star-federation
---

# Implemented `@method:role(path)` handlers compared with the Datastar ecosystem

## Scope and authority

This record investigates the **shipped** Rocci backend-handler mechanism
`@method:role(path)` against current code, tests, and public docs, then
compares that mechanism with Datastar's own architecture and with
Datastar-using projects in any language. Composite DX judgments are
**exploratory synthesis**, not an approved language change.[^handler-contract]
[^server-ref][^datastar-docs]

Prefer this record for "is the landed matrix Datastar-shaped, and what can it
not express?" Prefer the [pre-landing verb-first research](verb-first-handler-declarations.md)
for why method comes before role. Prefer crate READMEs and the public server
declaration reference for the current contract. The verb-first and live-path
records remain labeled historical as design evidence; this record is the
post-landing ecosystem check.[^verb-first][^verb-first-plan][^live-path]
[^impl-status][^server-ref]

## For a later agent

- **Authority:** exploratory for ecosystem scores and "right track" judgments.
  Descriptive for the closed matrix, generated wire policy, and injection
  rules, which are checked against cited tests and dispatch.
- **Do not** treat this as a plan to add `@get:signals`, path parameters, or
  morph-mode options. Those absences are the product ceiling unless a later
  plan is approved.
- **Do not** encode Datastar SSE policy in the `.rocci` parser. Dispatch and
  runtime own empty-SSE versus 204 and the live poll loop.
- Verify claims against `handler_contract.rs`, `handler_syntax.rs`,
  `validate.rs` `legal_pair`, and `dispatch.rs` `route_arm` / `live_sse_arm`
  before repeating them as shipped behavior.

## What shipped

Ordinary `.rocci` HTTP routes are mandatory verb-first headers. There is no
bare `@get(path)`, no role-first `@view(path)`, no `@on`, no trailing `json`,
and no pathless `@live`. The parser recovers those forms with rewrite
diagnostics. Paths are string literals in parentheses.[^parser][^handler-syntax]
[^server-ref]

```text
@method:role("path") = |state, request?| { body }
```

Four roles exist. Eleven method/role pairs are legal:[^handler-contract]
[^validate]

| Method | `view` | `fragment` | `command` | `live` |
| --- | --- | --- | --- | --- |
| GET | document HTML | one-shot morph | rejected | long-lived morph stream |
| POST, PUT, PATCH, DELETE | rejected | one-shot morph | no representation | rejected |

That is the entire high-level surface. HEAD, OPTIONS, and unknown methods are
rejected. Unknown roles are rejected. GET cannot be a command; mutations
cannot be views or live streams.[^validate][^handler-syntax]

Bodies are Roc, not templates. Generated dispatch always calls
`handler!(context, request)`. Omitting parameters, or writing a one-parameter
list, appends unused `_request` at lowering.[^ast-params][^lower]
[^template-readme]

Canonical examples: the standalone counter is `@get:view` plus
`@post:fragment`. The live counter is `@get:view`, `@post:command`, and
`@get:live("/sse")`. The handler-matrix example exercises every accepted
pair.[^counter][^live-counter][^handler-matrix]

## How a header becomes a response

The mechanism is a compile-time classification, not a runtime content-type
negotiation.

1. **Parse.** `@` + HTTP-method ident + `:` + role ident + `("literal-path")`
   plus optional `= |params|` and a Roc block. The AST is four route
   variants, not one generic handler with a response ADT.[^ungram][^parser]
2. **Validate.** `legal_pair` admits only the eleven cells. Duplicate
   method+path pairs fail even when roles differ, so `@get:view("/x")` and
   `@get:fragment("/x")` cannot coexist. Colliding normalized Roc names
   (`/a-b` vs `/a_b`) also fail. App-wide method/path uniqueness extends
   across sibling modules.[^validate][^dispatch]
3. **Lower.** Each route becomes a Roc function named from method and path
   (`on_get_root!`, `on_post_actions_counter_increment!`). Live routes are a
   separate `LiveInfo` list. A module with exactly one live path and no
   authored `data-init` gets `data-init=@get(path, [OpenWhenHidden(True)])`
   on the document `<body>`. Multiple local lives inject nothing.[^lower]
4. **Dispatch.** Generated `respond!` matches `(method, exact-path)` and
   wraps the Roc value:[^dispatch][^template-readme]

| Role | Author returns | Generated success |
| --- | --- | --- |
| `view` | `Html` document | `200` `text/html` |
| `fragment` | `Html` fragment with a stable `id` | one finite `datastar-patch-elements` event, then end |
| `command` | `{}` | Datastar (`Datastar-Request: true`): empty SSE; otherwise `204` |
| `live` | `Html` fragment | long-lived SSE: poll the handler, patch when `Html.render` bytes change, keepalive when unchanged |

Handler `?` failure is HTTP 500. Datastar callers of fragments and commands
receive an HTML error overlay as a patch; ordinary command callers receive
plain text. Live failures patch an overlay and keep the stream open.
[^dispatch][^template-readme]

Live polling is linear: one independent unfold per subscription, waking
`After(100)` (100 ms). Unchanged polls emit `Sse.Event.data("")` so
basic-webserver's response idle timeout cannot kill a silent wait. There is
no in-process pub/sub and no SSE `id` / `Last-Event-ID` resume.[^dispatch]
[^handler-contract][^bws-sse][^cqrs-research]

The browser call site does **not** name the role. Buttons write
`data-on:click=@post("/actions/...")`. Datastar sees only method and URL.
The role is a server-only contract that tells Rocci which wrap to generate.
[^handlers-guide][^handler-matrix][^datastar-docs]

## Datastar's actual architecture

Datastar is a browser runtime plus a wire protocol. Official backend
guidance is: send `@get` / `@post` / `@put` / `@patch` / `@delete` from HTML
attributes; accept `text/html`, `text/event-stream`, `application/json`,
`text/javascript`, or empty/`204`; optionally stream zero or more SSE events
of two types, `datastar-patch-elements` and `datastar-patch-signals`.
Element patches have modes (`outer`, `inner`, `replace`, `prepend`,
`append`, `before`, `after`, `remove`), optional CSS selectors, namespaces,
and view-transition flags. JSON responses patch signals. HTML responses may
set `datastar-selector` and `datastar-mode` headers. CQRS is documented as
one long-lived GET stream plus short writes, not as a required shape for
every app.[^datastar-docs][^datastar-backend][^datastar-sse]

Official SDKs (Go, Python, PHP, Ruby, Rust, .NET, Java, TypeScript,
Clojure, Haskell, and others) implement one imperative generator:
`ServerSentEventGenerator` with `PatchElements`, `PatchSignals`,
`ReadSignals`, plus helpers such as execute-script and redirect. They do
not classify routes by response role. A Go handler is an ordinary
`http.HandlerFunc` that chooses at runtime which events to emit.
[^datastar-adr][^datastar-sdks][^datastar-go]

Community "Tao" writing makes the CQRS split explicit: most state lives on
the backend; the backend patches HTML; signals are ephemeral; trust the
morph; separate reads from writes. Stario (Python) and Lambda Combine
(Common Lisp) encode that as **documentation plus helpers**: a page GET, a
subscribe GET, and POST/PATCH/DELETE actions that return `204` and publish
a relay so the subscribe loop re-renders. Dstar (Elixir) encodes a
LiveView-like `Page` (`mount` / `render` / `handle_event` / `handle_info`)
and can patch **both** signals and elements from one event handler, with a
single dispatcher route per page rather than one HTTP route per action.
[^lambda-tao][^stario-arch][^lambda-cqrs][^dstar]

None of those systems put `@post:command(path)` in a grammar. The closest
shared *architecture* is Stario/Lambda CQRS. The closest shared *authoring
unit* is Dstar's page module. Rocci's closest *call-site* match is the
official Datastar actions: verb first, path in the URL.

## Is this the same mechanism?

**No.** It is the same Datastar product architecture (hypermedia, server-owned
HTML, optional CQRS) expressed as a **closed language matrix**, which the
rest of the ecosystem does not do.

| Layer | Official Datastar / SDKs | Stario, Lambda Combine | Dstar (Elixir) | Rocci `@method:role` |
| --- | --- | --- | --- | --- |
| What you declare | HTTP route in the host framework | HTTP route plus subscribe/action convention | One page module, event names | Method **and** response role |
| Who chooses the wire format | Handler body / SDK calls | Handler body; docs say 204 vs SSE | `patch` / `patch_signals` in `handle_event` | Compiler from the header |
| One-shot HTML patch | `PatchElements` or `text/html` | Allowed, not the live default | `patch(...)` | `:fragment` |
| Representation-free write | Return 204 or empty SSE | Documented default for actions | Possible, not forced | `:command` only `{}` |
| Shared live HTML | Long-lived GET + generator loop | `/subscribe` + relay | `handle_connect` / `handle_info` | `:live` + generated poll |
| Patch signals | First-class SDK method and JSON responses | Used for thin client mirrors | First-class on the page | Low-level Roc / authored `main.roc` only |
| Several events per response | Explicitly recommended | Subscribe loop sends many over time | Common in one `handle_event` | Fragments send exactly one patch-elements |
| Path parameters | Host router (`/items/{id}`) | Host router | Phoenix params | Literal paths only |
| Escape hatch | Always: it is ordinary HTTP | Ordinary Python | Plain Plug | Authored `main.roc` |

Rocci therefore sits **between** a conventional Datastar SDK (open-ended
events in the handler) and a LiveView-style page (hide HTTP). That hybrid
was the verb-first research conclusion; the implementation kept it.
[^verb-first][^server-owned]

Two consequences follow:

1. Authors who learned Datastar from Go/Python examples will not find
   `PatchElements` in `.rocci`. They will find a smaller menu that already
   picked the wrap.
2. Authors who learned Phoenix LiveView from Dstar will not find
   `handle_event("increment", signals)`. They will find explicit HTTP paths
   that match `data-on:click=@post(...)`.

Both mismatches are intentional product boundaries, not incomplete SDK
ports.

## DX: is this on the right track for efficient application development?

**Yes, for server-rendered Datastar apps whose success representation is
HTML or nothing.** The matrix removes the two decisions that dominate
Datastar tutorial confusion: "should this POST return HTML, JSON, or 204?"
and "does this GET load a page, morph a slot, or stay open?" Naming those
decisions in the header is faster than calling an SSE API correctly every
time, and it matches Rocci's constraint that the template parser cannot
inspect Roc types.[^verb-first][^template-readme]

What is efficient about it:

- **One grammar for every route.** No method-dependent defaults. Completion
  after `@get:` is `view | fragment | live`; after a mutation method it is
  `fragment | command`. Illegal pairs fail in the template crate, not at
  Roc compile or in the browser.[^handler-syntax][^validate]
- **Two taught workflows, both shipped.** One-shot fragment (counter,
  search, click-to-edit, validation HTML). CQRS command plus live (shared
  count, blocks, multi-tab logs). The handler matrix makes the difference
  visible: a fragment well changes in one tab; a command only moves the
  live log.[^counter][^live-counter][^handler-matrix][^handlers-guide]
- **Datastar call sites stay ordinary.** Authors do not invent a second
  client API. `@post("/actions/inc")` is the Datastar action; the role lives
  only on the server.[^handlers-guide][^datastar-docs]
- **Representation-free commands prevent a class of bugs.** Generated
  commands cannot accidentally send domain JSON that Datastar would apply
  as signal patches instead of morphing `#counter`. That bug is documented
  in the CQRS research and is now unrepresentable on the high-level
  surface.[^cqrs-research][^handler-contract]
- **Components stay pure.** I/O lives in `@init` and route bodies. That is
  the same split Stario recommends (commands mutate, subscribe re-reads
  and renders), encoded as file-level roles rather than a relay.
  [^server-owned][^stario-arch]
- **The cliff is named.** Mixed SSE events, JSON APIs, prefix routing, and
  custom unfolds belong in authored `main.roc`. The custom Datastar gallery
  already does prefix-matched GET fragments (`/actions/tabs/{id}`) that
  `.rocci` routes cannot spell.[^custom-main][^template-readme]

What is inefficient, or will feel inefficient as apps grow:

- **The browser cannot see the role.** A reader of the `.rocci` file knows
  whether `/actions/inc` is a fragment or a command; a reader of the button
  does not. Path naming conventions (`/actions/...` vs `/fragments/...`)
  become social rather than checked. Dstar inverts this: the template
  `event("increment")` and `handle_event(..., "increment")` share a name.
  [^dstar][^handlers-guide]
- **Two correct ways to increment.** `@post:fragment` updates one tab
  immediately. `@post:command` plus `@get:live` updates every subscriber
  after the next poll. Choosing wrong is a product bug (lost fan-out, or
  double-morph of the same `id`), not a type error. Docs forbid morphing
  the same `id` from both a fragment and a live stream, but the language
  does not enforce it.[^handlers-guide][^cqrs-research]
- **Literal paths only.** There is no `/items/:id`, no wildcard, no query
  routing. Row-specific URLs must be built as Roc strings in attributes and
  then cannot be declared as generated routes unless every id is a separate
  header. The gallery escapes to `Str.starts_with`.[^custom-main][^dispatch]
- **Live is a poll, not a push.** Stario's relay publishes after a command;
  Rocci re-renders every 100 ms per subscription whether or not anything
  changed, plus keepalives. That is simple on basic-webserver and honest
  about the platform, but it is not the Datastar Tao "command publishes,
  subscribe wakes" loop. Cost is `streams × tabs` independent loops. Fine
  for a few coarse page streams; the wrong shape for one connection per
  widget or for fifty low-latency game seats without a custom unfold.
  [^dispatch][^live-path][^bws-sse][^stario-arch]
- **Fragments always SSE.** Datastar can morph from `text/html`. Generated
  fragments always wrap `Datastar.patch_elements` and do not consult
  `Datastar-Request`. `curl` of a fragment is an event stream, not HTML.
  Commands *do* branch on the header. The asymmetry is real.
  [^dispatch][^datastar-docs]
- **Request bodies are unstructured.** Datastar sends signals as a query
  param on GET and JSON on other methods, or as `contentType: 'form'` /
  multipart. High-level handlers receive the platform `request` and parse
  bytes themselves (the live counter reads a Datastar body for `origin` /
  `tz`). There is no `ReadSignals`-shaped helper on the route header.
  [^live-counter][^datastar-docs][^datastar-adr]
- **Vocabulary transfer from Datastar tutorials is incomplete.** Tutorials
  show `sse.PatchElements` then `sse.PatchSignals` in one handler. Rocci
  authors who copy that into a `:fragment` body cannot emit the second
  event. The fix is "use authored `main.roc`", which is a different
  product surface.[^datastar-backend][^template-readme]

Relative to the rest of the Datastar ecosystem, Rocci is **more opinionated
and less complete**, which is the right trade for a language that generates
dispatch. It is closer to "Rails has REST verbs and you pick format" than
to "the Go SDK exposes every SSE field." For CRUD-shaped HTML apps, search
boxes, validation wells, and a small number of shared live regions, the DX
is on the right track. For Datastar's full HOWL palette, it is a subset
with a steep escape hatch.

## Downsides of the structure itself

These follow from making role a closed header, not from missing a later
phase:

1. **Role is not HTTP.** Two GET handlers on the same path with different
   roles are still one resource. The language rejects that, correctly for
   HTTP, but it means you cannot serve a document and a fragment at `/`
   without a second path.
2. **Role is not content negotiation.** A `:fragment` cannot return HTML to
   `curl` and SSE to Datastar. A `:command` cannot return a useful JSON
   body to a non-Datastar client. Ordinary callers of commands get 204,
   which is right for CQRS and wrong for a public JSON API.
3. **Success type is fixed per role.** A fragment cannot return `{}` on
   some branches and HTML on others. A command cannot return validation
   HTML on failure except via the generated 500 overlay (`Err`), so
   user-facing validation belongs on `:fragment`.
4. **Live cannot be a mutation method.** Datastar can stream SSE from POST;
   Rocci will not. Long-lived writes have to be GET live plus separate
   commands.
5. **No composition of events.** The Datastar selling point of "several
   patch-elements and patch-signals in one response" is structurally out of
   bound for generated routes. Coarse live HTML that morphs several stable
   ids in **one** fragment is the supported substitute.[^live-path]
   [^datastar-docs]
6. **Generator names couple path spelling to Roc identifiers.** `/a-b` and
   `/a_b` collide. Path design is constrained by `on_{method}_{path}!`.
   [^validate]
7. **Ungram omits the parentheses** that the parser requires around the
   path. Treat the tests and public docs as the contract, not the ungram
   sketch, until those are aligned.[^ungram][^parser][^server-ref]

## Constructs this structure does not support

Grouped by where they live in Datastar or ordinary HTTP. All of these remain
possible in authored `main.roc` unless a platform limit applies.

### Datastar protocol and SDK surface

- `datastar-patch-signals` as a route role, including JSON responses and
  `datastar-only-if-missing`.
- Multiple SSE events in one finite response (two ids plus a signal patch).
- Element patch modes other than default outer morph: `inner`, `replace`,
  `prepend`, `append`, `before`, `after`, `remove`.
- CSS `selector` patches (elements without ids; remove-by-selector).
- `namespace` (`svg`, `mathml`) and view-transition flags on the event.
- `text/html` morph with `datastar-selector` / `datastar-mode` headers.
- `text/javascript` responses.
- SDK `ExecuteScript`, `Redirect`, `RemoveElement` helpers as roles.
- `filterSignals`, form `contentType`, and multipart file upload as
  declaration options (Datastar client options exist; generated routes do
  not declare them).
- SSE `id` / retry / `Last-Event-ID` resume.
- Keeping a one-shot request open to stream a progress sequence, then
  closing (SDK examples sleep and `PatchElements` twice).

### HTTP and routing

- Path parameters, prefixes, wildcards, optional trailing segments.
- Query-string routing (query data is only available by reading `request`).
- HEAD, OPTIONS, TRACE, CONNECT; CORS preflight as a declared route.
- Status codes other than success 200/204 and failure 500 (no 201, 303,
  401, 404 from the handler value).
- `Set-Cookie`, session middleware, CSRF helpers (Dstar's explicit extra).
- Static vs negotiated content types on one method+path.
- Redirect as a first-class handler result (Datastar documents backend
  redirects separately from morphs).

### Application architecture

- In-process or multi-process pub/sub that wakes live handlers (Stario
  Relay). Generated live always polls.
- Per-connection identity, presence, and targeted patches to one tab.
- One dispatcher route with many named events (Dstar `handle_event`).
- Optimistic UI (Datastar docs argue against it; Rocci has no hook).
- Mixing fragment morph and live morph on the same `id`.
- JSON REST resources, GraphQL, webhooks that need response bodies.
- Authn/authz as route metadata.
- Streaming POST/PUT live; WebSockets.

### Client-side Datastar that looks like a backend feature

- Binding visible domain state to signals (`data-text="$count"` as source
  of truth). Allowed in HTML, contrary to Rocci's server-owned-state
  decision if used for durable data.[^server-owned][^lambda-tao]
- Client-specified swap mode (`hx-swap`-style). Datastar can do this via
  headers on HTML responses; Rocci fragments do not expose it.
- Per-component live connections. Research and the live-path plan reject
  that in favor of coarse page/coherence streams.[^live-path]

## Verdict

The implemented `@method:role(path)` mechanism is **Datastar-architecture
aligned and ecosystem-unique**. It is not a port of the SDK ADR. It is a
deliberate, smaller product: four roles that cover Datastar's two common
update patterns (one-shot patch and Tao CQRS) plus documents, with HTTP
verbs spelled the way Datastar actions spell them.

That is the right track for efficient Rocci application development **as
long as the ceiling stays visible**. Expanding the grammar toward
`PatchElements` options, signal roles, or generic response ADTs would
erase the inspectability that justified putting role in the header. The
supported growth paths already exist: more examples of when to pick
fragment versus command+live; authored `main.roc` for prefix routing and
mixed events; keep live coarse and poll-honest until the platform has a
relay.

Do not treat missing SDK features as incomplete implementation of this
mechanism. The mechanism is complete relative to its closed matrix.
Completeness relative to Datastar is a different, larger product.

[^handler-contract]: Frozen eleven-pair matrix, representation-free commands, empty-SSE versus 204, 100 ms live poll.
[^handler-syntax]: Accepted headers, rejected near-misses, duplicate method+path, removal rewrites.
[^parser]: Verb-first `try_parse_route`, unknown-role recovery, removed `@on` / role-first diagnostics.
[^validate]: `legal_pair` for GET versus mutation methods; duplicate and generated-name collisions.
[^lower]: `RespondKind`, live `LiveInfo`, singleton `inject_live_path`.
[^ungram]: `RouteDecl` of `ViewDecl | FragmentDecl | CommandDecl | LiveDecl`.
[^ast-params]: `ensure_handler_request_param` appends `_request` for arity 0 and 1.
[^dispatch]: `route_arm` document/fragment/command wraps; `live_sse_arm` poll, patch, keepalive; exact path match; `Datastar-Request`.
[^template-readme]: Public matrix, injection rules, empty SSE versus 204, authored `main.roc` escape hatch.
[^server-ref]: Mandatory verb-first headers; closed table; no aliases.
[^handlers-guide]: Choose-a-shape table; `@post` call sites; do not morph the same id from fragment and live.
[^counter]: `@get:view("/")` and `@post:fragment` increment/reset.
[^live-counter]: `@post:command` returns `{}`; `@get:live("/sse")` owns shared morphs; request body parsed in Roc.
[^handler-matrix]: Every accepted pair plus one live log stream.
[^custom-main]: Prefix-matched `/actions/tabs/` GET fragments in authored dispatch.
[^verb-first]: Pre-landing rationale for method-first mandatory roles versus SDK-style inference.
[^verb-first-plan]: Implementation sequence for the landed cutover.
[^live-path]: Plural path-addressed GET live; coarse streams; injection table.
[^cqrs-research]: One-shot POST does not fan out; commands must not send domain JSON to Datastar as a morph.
[^bws-sse]: Idle-timeout workarounds: keepalives and empty SSE.
[^server-owned]: Durable state stays on the server; HTML is the update boundary.
[^impl-status]: Snapshot that ordinary routes are the closed `@method:role` matrix.
[^datastar-docs]: CQRS example, response types, `Datastar-Request`, form/multipart, HTML/JSON/JS handling.
[^datastar-backend]: SDK `PatchElements` plus `PatchSignals` in one handler; multiple events per response.
[^datastar-sse]: `datastar-patch-elements` modes and `datastar-patch-signals`.
[^datastar-sdks]: Official per-language SDKs.
[^datastar-adr]: SDK contract is an SSE generator, not a route-role matrix.
[^stario-arch]: Page GET, subscribe GET, 204 actions, Relay, subscribe re-renders.
[^lambda-cqrs]: POST commands, GET SSE, registry broadcast.
[^lambda-tao]: Tao bullets: backend truth, patch HTML, signals ephemeral, CQRS split.
[^dstar]: `use Dstar.Page`, `handle_event`, combined signal and element patches, one route per page.
[^datastar-go]: Ordinary Go handler plus `NewSSE`, `PatchElements`, `PatchSignals`.
