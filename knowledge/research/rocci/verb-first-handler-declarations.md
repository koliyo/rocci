---
type: Research Report
title: Verb-first handler declarations for Rocci's bounded UI surface
description: "Rocci should put the HTTP verb first and require a semantic response-role suffix, including path-addressed live GET routes, without expanding .rocci into a generic API or event-stream language."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T09:52:04Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: template-ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci AST specification for semantic handlers
    author: process:git
    last_modified: 2026-08-21
  - id: template-parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Rocci semantic-handler parser and method diagnostics
    author: process:git
    last_modified: 2026-08-21
  - id: template-lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Rocci route metadata and handler lowering
    author: process:git
    last_modified: 2026-08-21
  - id: template-pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: Rocci handler inspection formatting
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated document, patch, command, and live dispatch
    author: process:git
    last_modified: 2026-08-21
  - id: handler-contract
    resource: ../../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen semantic handler contract
    author: process:git
    last_modified: 2026-08-21
  - id: handler-syntax
    resource: ../../../crates/rocci-template/tests/handler_syntax.rs
    title: Accepted and rejected handler syntax tests
    author: process:git
    last_modified: 2026-08-21
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Current public handler contract in the owning crate
    author: process:git
    last_modified: 2026-08-21
  - id: server-reference
    resource: ../../../docs/reference/language/server.rocdown
    title: Current public server declaration reference
    author: process:git
    last_modified: 2026-08-21
  - id: rendering-doc
    resource: ../../../docs/concepts/documents-fragments-commands-streams.rocdown
    title: Current document, fragment, command, and stream model
    author: process:git
    last_modified: 2026-08-21
  - id: custom-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Authored Datastar dispatcher with GET fragments
    author: process:git
    last_modified: 2026-08-20
  - id: search-fragment
    resource: ../../../examples/rocci/custom/datastar/Search.rocci
    title: Search UI issuing a GET fragment request
    author: process:git
    last_modified: 2026-08-20
  - id: tabs-fragment
    resource: ../../../examples/rocci/custom/datastar/Tabs.rocci
    title: Tabs UI issuing GET fragment requests
    author: process:git
    last_modified: 2026-08-20
  - id: datastar-crate
    resource: ../../../crates/rocci-datastar/README.md
    title: Rocci Datastar protocol-layer responsibilities
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-roc
    resource: ../../../crates/rocci-datastar/src/codegen/mod.rs
    title: Generated Roc Datastar helper surface
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-sse
    resource: ../../../crates/rocci-datastar/src/sse/events.rs
    title: Rust patch-elements, patch-signals, removal, and script event builders
    author: process:git
    last_modified: 2026-08-17
  - id: prior-research
    resource: ../action-handler-syntax.md
    title: Research on bounding Rocci handlers to server-rendered UI
    author: process:cursor
    last_modified: 2026-08-22
  - id: prior-plan
    resource: ../../plans/rocci/handler-ui-boundary.md
    title: Earlier handler UI boundary plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: implementation-plan
    resource: ../../plans/rocci/verb-first-handler-declarations.md
    title: Implementation plan for verb-first bounded handlers
    author: process:cursor
    last_modified: 2026-08-22
  - id: live-path-research
    resource: ../path-addressed-live-streams.md
    title: Follow-up research on path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
  - id: live-path-plan
    resource: ../../plans/rocci/path-addressed-live-streams.md
    title: Follow-up implementation plan for path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
  - id: server-owned-state
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar backend actions and response handling
    author: organization:star-federation
  - id: datastar-backend
    resource: https://data-star.dev/guide/backend_requests
    title: Datastar backend requests guide
    author: organization:star-federation
  - id: datastar-sse-reference
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event reference
    author: organization:star-federation
  - id: datastar-signals
    resource: https://data-star.dev/guide/reactive_signals
    title: Datastar reactive signals guide
    author: organization:star-federation
  - id: fastapi-routing
    resource: https://fastapi.tiangolo.com/tutorial/first-steps/
    title: FastAPI first steps and path operation decorators
    author: organization:tiangolo
  - id: express-routing
    resource: https://expressjs.com/en/guide/routing.html
    title: Express routing guide
    author: organization:openjs-foundation
  - id: spring-routing
    resource: https://docs.spring.io/spring-framework/reference/web/webmvc/mvc-controller/ann-requestmapping.html
    title: Spring MVC annotated request mappings
    author: organization:broadcom
  - id: flask-routing
    resource: https://flask.palletsprojects.com/en/stable/quickstart/
    title: Flask quickstart routing
    author: organization:pallets
  - id: axum-routing
    resource: https://docs.rs/axum/latest/axum/routing/method_routing/fn.get.html
    title: Axum GET method router
    author: organization:tokio
  - id: aspnet-routing
    resource: https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis
    title: ASP.NET Core minimal APIs
    author: organization:microsoft
  - id: phoenix-routing
    resource: https://phoenix.hexdocs.pm/Phoenix.Router.html
    title: Phoenix router reference
    author: organization:phoenix-framework
  - id: actix-routing
    resource: https://actix.rs/docs/getting-started/
    title: Actix Web getting started
    author: organization:actix
  - id: react-router-actions
    resource: https://reactrouter.com/start/framework/actions
    title: React Router framework actions
    author: organization:remix-software
  - id: liveview-bindings
    resource: https://hexdocs.pm/phoenix_live_view/bindings.html
    title: Phoenix LiveView bindings
    author: organization:phoenix-framework
---

# Verb-first handler declarations for Rocci's bounded UI surface

## Question and disposition

The shipped source order is inverted relative to conventional backend routing:
Rocci starts with the generated response role and places an optional HTTP
method after it. `@patch:put(path)` means “PUT this route and turn its `Html`
result into a one-shot element patch”; `@patch:patch(path)` uses the first
`patch` as the response noun and the second as the HTTP method. `@view` hides
GET, while `@patch` and `@command` hide POST.[^template-parser]

That inversion was not arbitrary. Rocci treats ordinary Roc bodies as opaque,
so syntax must select response policy without inferring it from a return
expression. The problem is therefore not that Rocci names a response role. It
is that the route header places the less familiar framework policy before the
wire fact that authors already write in Datastar and recognize from other
backend libraries.

The recommended clean-cut syntax is:

```text
@http-method:response-role(path)
```

with both parts mandatory for ordinary routes:

```rocci
@get:view("/") = |{ db }| { page(...) }
@get:fragment("/search") = |{ db }, request| { results(...) }
@post:fragment("/actions/todo/add") = |{ db }, request| { todoList(...) }
@patch:fragment("/actions/todo/42") = |{ db }| { todoRow(...) }
@post:command("/actions/counter/increment") = |{ db }| { increment!(db)? }

@get:live("/streams/shared") = |{ db }| { sharedCounter(...) }
```

The path-addressed live follow-up adds `live` as a GET-only role because
multiple authored stream paths are route-shaped even though their generated
lifecycle remains special. Keep the accepted method-role matrix closed.
Verb-first syntax does not approve JSON,
patch-signals, scripts, redirects, downloads, arbitrary SSE, or every future
HTTP method as high-level declarations.

This is exploratory research. It resolves the earlier exploratory `@patch`
versus `@fragment` gate in favor of `fragment` under a leading HTTP method and
is amended by the path-addressed live follow-up. It does not change or approve
shipped syntax. The linked
[implementation plan](/plans/rocci/verb-first-handler-declarations.md) has explicit
gates and no phase has started.[^implementation-plan][^live-path-research]

## Shipped baseline

The current AST has separate `ViewDecl`, `PatchDecl`, `CommandDecl`, and
`LiveDecl` nodes. `@view` is fixed to GET; `@patch` and `@command` default to
POST and accept PUT, PATCH, or DELETE; GET is rejected for both mutation
declarations.[^template-ungram][^handler-contract][^handler-syntax]

| Current declaration | Normalized method | Successful body value | Generated success behavior |
| --- | --- | --- | --- |
| `@view(path)` | GET | complete `Html` | HTML document |
| `@patch[:method](path)` | POST/PUT/PATCH/DELETE | stable-ID `Html` | one patch-elements SSE event |
| `@command[:method](path)` | POST/PUT/PATCH/DELETE | JSON-encodable Roc data | empty SSE for Datastar; JSON otherwise |
| `@live` | generated GET `/sse` | stable-ID `Html` | long-lived patch-elements SSE |

Lowering currently records only `Patch` and `Command` response kinds. Generated
dispatch distinguishes a document by checking whether the method is GET before
examining the response kind. That shortcut is safe only while no GET fragment
exists.[^template-lower][^dispatch]

Inspection is already closer to the proposed mental model than source syntax:
it emits kind, normalized method, path, and semantic role. The method is not
hidden from tooling even though source places the noun first.[^template-pprint]

The prior semantic-handler cutover is shipped behavior. The earlier bounded
UI research remains the basis for excluding API JSON and signal-event variants;
this report resolves its source-order and fragment-noun gate.[^prior-research]
[^prior-plan][^template-readme][^server-reference]

## What established routing conventions optimize

Mainstream routers make the HTTP method the entry point:

| Family | Representative form | How response policy is selected |
| --- | --- | --- |
| FastAPI | `@app.get(path)` | return annotation/value and response configuration |
| Express | `app.post(path, handler)` | calls on the response object |
| Spring MVC | `@GetMapping(path)` | return handling plus response annotations/types |
| Flask | `@app.post(path)` | returned value/response object |
| Axum | `.route(path, get(handler))` | handler return type implementing a response |
| ASP.NET minimal API | `MapPost(path, handler)` | handler result and typed result helpers |
| Phoenix router | `get path, Controller, :action` | controller/connection response |
| Actix Web | `#[get(path)]` or `web::get()` | responder return type |

The exact APIs differ, but all make “which request reaches this handler?” the
first visible classification. They can do so without a response-role suffix
because response semantics remain expressible in handler types, return values,
annotations, or response objects.[^fastapi-routing][^express-routing]
[^spring-routing][^flask-routing][^axum-routing][^aspnet-routing]
[^phoenix-routing][^actix-routing]

Rocci should adopt the approachable part of that convention—the leading
method—without copying its implicit response selection. Rocci's parser cannot
typecheck or semantically inspect the opaque Roc body while constructing the
template AST. A required role suffix preserves local inspectability and lets
the compiler choose generated dispatch before Roc compilation.[^template-parser]

## The server-driven UI counterexample

Not every UI-oriented backend leads with HTTP. React Router distinguishes
loaders from actions, and Phoenix LiveView maps named browser events to
`handle_event` callbacks over its live connection. Those systems optimize for
read-versus-mutation or user intent instead of exposing every transport detail.
[^react-router-actions][^liveview-bindings]

This counterexample explains why Rocci's role-first design felt plausible, but
it does not fully fit Rocci:

- Rocci authors explicit HTTP paths and Datastar backend actions rather than a
  framework-private event protocol.
- Datastar exposes `@get`, `@post`, `@put`, `@patch`, and `@delete` at the call
  site.
- The same route is visible to browsers, `curl`, logs, authorization code, and
  generated dispatch.
- Rocci still needs an explicit response role because the body is opaque.

Rocci therefore sits between a conventional router and a semantic live-view
framework. `@method:role` reflects that hybrid more honestly than either a
bare method or a role-only event declaration.

## Datastar symmetry

Datastar's browser actions are method-first. An author writes forms and event
bindings such as `@get('/search')`, `@post('/actions/save')`, and
`@patch('/items/42')`. Datastar then interprets the response independently: it
can consume HTML, SSE, JSON signal patches, JavaScript, or an empty success.
SSE can contain patch-elements or patch-signals events.[^datastar-actions]
[^datastar-backend][^datastar-sse-reference]

The proposed server declaration lets both sides line up:

```text
browser                              server
@get('/search')                ->    @get:fragment('/search')
@post('/actions/save')         ->    @post:fragment('/actions/save')
@post('/actions/increment')    ->    @post:command('/actions/increment')
@patch('/items/42')            ->    @patch:fragment('/items/42')
```

The shared leading word answers the routing question. The suffix adds the
server-only fact Datastar cannot infer from the request: what successful
browser effect Rocci generates.

## Separate method from response role

The language needs two explicit axes:

1. **Method:** the HTTP contract used to reach the route.
2. **Role:** the browser-facing meaning of a successful Roc value.

Neither axis implies the other. GET may return a full document or a fragment.
POST may return a fragment or perform a command whose rendering is owned by a
live stream. HTTP PATCH does not imply a Datastar element patch, and a
Datastar element patch does not imply HTTP PATCH.

The proposed legal matrix is deliberately smaller than the cross-product:

| Leading method | `view` | `fragment` | `command` | `live` |
| --- | --- | --- | --- | --- |
| `get` | accepted | accepted | rejected | accepted |
| `post` | rejected | accepted | accepted | rejected |
| `put` | rejected | accepted | accepted | rejected |
| `patch` | rejected | accepted | accepted | rejected |
| `delete` | rejected | accepted | accepted | rejected |

`live` always requires an authored literal path. Its cardinality and
subscription rules are specified in the path-addressed stream follow-up.
[^live-path-research][^live-path-plan]

The role names mean:

| Role | Successful Roc value | Browser effect |
| --- | --- | --- |
| `view` | complete `Html` document | initial load or navigation |
| `fragment` | stable-ID `Html` fragment | one-shot morph in the acting tab |
| `command` | `{}` | mutation completes without a direct morph |
| `live` | stable-ID `Html` fragment | repeated morphs over a long-lived GET |

The shipped pathless `@live` still owns generated `/sse`, keepalives,
patch-elements framing, and automatic `data-init`. The proposed final language
expresses that lifecycle as path-addressed `@get:live(path)` and permits
multiple streams under explicit injection rules.[^rendering-doc][^dispatch]
[^live-path-research]

## Why `fragment` is the right role noun

Under role-first syntax, `@patch` could be defended as “the browser will
patch.” Under verb-first syntax, `patch` is already the unambiguous HTTP method.
Reusing it as the role would produce either `@get:patch` or `@patch:patch`, and
the latter again requires the reader to distinguish identical words by
position.

`fragment` instead names the authored value. Generated dispatch may encode
that fragment as a Datastar patch-elements SSE event today and could use an
equivalent element-morph transport later without renaming the language role.
It also reads naturally for both idempotent reads and mutations:

```rocci
@get:fragment("/tabs/settings")
@post:fragment("/actions/todo/add")
@patch:fragment("/items/42")
```

The custom Datastar app already demonstrates search and tabs as GET requests
whose useful result is server-rendered HTML. The generated standalone router
cannot currently express those routes.[^custom-main][^search-fragment]
[^tabs-fragment][^handler-syntax]

`view` is retained rather than renamed to `document` because it is shipped,
short, and already documented as returning a complete document. The role table
and diagnostics must say “complete document” so authors do not confuse it with
a partial component render.[^template-readme][^rendering-doc]

## Why both words should be mandatory

A shorthand matrix would save characters:

```rocci
@get("/")                 # default view?
@get:fragment("/search")
@post("/save")            # default fragment?
@post:command("/save")
```

It would also create method-dependent hidden defaults. Authors would need to
remember that bare GET means one role, bare POST another, and decide what bare
PUT, PATCH, and DELETE mean. That reproduces the shipped hidden-POST problem in
a more complex form.

Requiring both words gives every normal route one grammar:

```text
@method:role(path)
```

The repetition is useful contract information. Completion can remain compact:
after `@get:` offer `view` and `fragment`; after a mutation method offer
`fragment` and `command`. Illegal combinations receive a narrow diagnostic
with the accepted roles for that method.

## Expressive power and options

Verb-first ordering does not inherently add or remove semantic power. The
legal pair matrix controls expressiveness. Compared with the shipped language,
the proposed matrix adds one high-level workflow: a one-shot GET HTML fragment.
It deliberately removes the ordinary-client JSON representation from
`command`; that is a product-boundary change, not a consequence of changing
token order.

| Candidate | Expressiveness | DX strength | Principal cost |
| --- | --- | --- | --- |
| Shipped role-first with defaults | Current document, mutation fragment, negotiated command, live | semantic role scans first | hidden methods, inverted convention, `@patch:patch`, no GET fragment |
| Role-first with explicit methods | Same bounded matrix | role remains first | still inverted; `@fragment:get` does not match Datastar call sites |
| Verb-first with mandatory role | Same bounded matrix plus selected GET fragment | explicit wire contract, familiar routing, Datastar symmetry | longer headers; role searches use `:role` |
| Verb-first with method defaults | Potentially same | shortest common cases | context-sensitive hidden semantics |
| Bare verb plus response inference | Theoretically broad | conventional router appearance | impossible at template parse time without inspecting Roc types/body |
| Generic response ADT in every body | Broadest extension ceiling | one low-level mechanism | ceremony on common UI routes; response behavior disappears from the header |

The recommended option is verb-first with a mandatory role. A typed response
ADT remains a possible escape hatch for authored Roc servers, not the default
`.rocci` route contract.

## Developer-experience consequences

### Learnability and call-site matching

Backend users can transfer the strongest common convention: start with the
HTTP method. New users can compare the Datastar action and server declaration
without reversing them. Documentation can teach one sentence: “write the
request method first, then the browser result.”

Users without HTTP experience see more terminology than with `@view`, but
Rocci already exposes paths, requests, headers, methods, and Datastar actions.
Hiding GET and POST does not remove that model; it delays it.

### Scanning and auditing

Verb-first source makes reads and writes visible at the left edge. That helps
route inventories, authorization review, and accidental-GET-mutation audits.
Role-first source makes all commands easier to group visually, but `:command`
remains searchable and structured inspection should expose both fields.

The source order would also match current inspect output, which already
normalizes method separately from role.[^template-pprint]

### Completion and diagnostics

The closed matrix gives deterministic two-stage completion:

```text
@get:        -> view, fragment
@post:       -> fragment, command
@patch:      -> fragment, command
@get:command -> GET cannot be a command; use @get:fragment for a UI read
@post:view   -> views are GET documents; use @get:view
```

Removal diagnostics can rewrite every shipped header mechanically. No alias is
needed, and the old syntax should not lower after the cutover.

### Namespace overlap with Datastar actions

At top level, `@post:command` is a Rocci declaration. Inside an HTML attribute,
`@post(...)` remains a Datastar action value. Parser context already separates
top-level declarations from attribute action expressions. The repeated verb is
an intentional alignment, but syntax highlighting and diagnostics must make
the two contexts visually and semantically distinct.[^template-parser]

## JSON is not an implied role

Datastar's ability to consume JSON does not justify `@get:json` or
`@post:json`. A JSON response to a Datastar request patches frontend signals;
it does not morph the server-rendered HTML fragment. A JSON resource for an API
client has separate representation, schema, status, versioning, authorization,
and compatibility concerns.[^datastar-backend][^datastar-signals]

The shipped `@command` mixes two consumers: Datastar gets empty SSE and ignores
the returned data, while an ordinary caller receives encoded JSON. This is a
partial API promise in a high-level UI file.[^dispatch][^server-reference]

The recommended command role is representation-free:

```rocci
@post:command("/actions/counter/increment") = |{ db }| {
    increment!(db)?
}
```

The generated runtime should return a successful no-morph response acceptable
to Datastar and no representation to ordinary callers. Whether the pinned host
requires the current empty-SSE Datastar response while ordinary callers use
204 is an implementation gate; neither branch should expose command result
data as a general API.

General JSON routes remain ordinary authored Roc in `main.roc`. If repeated
applications later need a typed low-level route facility, research one
response ADT rather than adding MIME-type roles to the high-level matrix.

## Patch-signals is not an implied role

Patch-signals is a valid Datastar transport operation for adding, updating, or
removing frontend signals. It can arrive as JSON or as a
`datastar-patch-signals` SSE event.[^datastar-signals]

Rocci's server-owned-state decision keeps durable application state on the
server and limits signals to ephemeral browser concerns.[^server-owned-state]
Therefore patch-signals is an advanced interaction primitive, not a peer to
documents and HTML fragments in the default rendering language.

The repository already has a layered extension point:

- the Rust `rocci-datastar` protocol crate models patch-elements,
  patch-signals, removal, and script events;[^datastar-crate][^datastar-sse]
- the generated Roc Datastar helper exposes action strings and basic
  patch-elements framing but not the same complete event set.[^datastar-roc]

Add a typed Roc `patch_signals` helper for authored servers after checking the
pinned protocol. Do not add `:signals`, `:json`, `:script`, patch-mode, or
multi-event suffixes to the ordinary handler matrix.

## Why path-addressed live joins the GET role matrix

`@live` currently means more than GET plus an event-stream response. It also
means one generated `/sse` route per module, polling unfold behavior,
keepalives, changed-HTML detection, patch-elements framing, and automatic
`data-init` injection on a root body.[^dispatch][^template-readme]

The earlier version of this report therefore kept `@live` exceptional. The
multi-page follow-up changes that premise: once authors need several explicit
stream paths, each stream is a public GET route even though its `live` role
selects special generated lifecycle. `@get:live(path)` is more regular than a
new `@live(path)` exception, and validation can reject every mutation-live
pair. Generated polling and subscription policy remain runtime/lowering
concerns rather than generic SSE syntax.[^live-path-research]

## Migration shape

The clean rewrite is mechanical:

| Shipped source | Proposed source |
| --- | --- |
| `@view(path)` | `@get:view(path)` |
| `@patch(path)` | `@post:fragment(path)` |
| `@patch:put(path)` | `@put:fragment(path)` |
| `@patch:patch(path)` | `@patch:fragment(path)` |
| `@patch:delete(path)` | `@delete:fragment(path)` |
| `@command(path)` | `@post:command(path)` |
| `@command:put(path)` | `@put:command(path)` |
| `@command:patch(path)` | `@patch:command(path)` |
| `@command:delete(path)` | `@delete:command(path)` |
| `@live` | `@get:live("/sse")` |

Do not retain aliases. A mixed language would force formatters, LSP, examples,
documentation, and readers to understand two orientations. Focused removal
diagnostics can recognize old headers and print the exact new form without
constructing a valid old AST.

## Risks and mitigations

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| Transport becomes too prominent | Authors may treat `.rocci` as a generic backend framework | Keep a closed role matrix and direct APIs/custom responses to authored Roc |
| Headers become longer | More visual syntax in small examples | Require meaningful words; do not add brackets or options objects |
| Commands split across verbs | Role-first visual grouping is lost | `:command` search, structured inspect output, and editor symbols |
| `view` is read as any render | GET document versus fragment confusion | Documentation and diagnostics say “complete document”; keep `fragment` explicit |
| Top-level verbs resemble Datastar actions | Context confusion in highlighting | Separate declaration and attribute-action token kinds/tests |
| GET fragments are effectful in Roc | Accidental non-idempotent reads | Document and test the contract; do not claim parser enforcement |
| Clean cut follows a recent clean cut | Migration churn | One repository-wide change with exact diagnostics; no compatibility window |
| Role matrix grows over time | Return of permutation bloat | Require a demonstrated UI workflow and stack review for every new role/pair |

## Evaluation protocol before implementation approval

Evaluate complete examples rather than isolated headers. Give maintainers and
new users both the shipped and proposed forms and ask them to:

1. Match a Datastar action to its server handler.
2. Identify every GET and every mutation route.
3. Predict whether success navigates, morphs one fragment, performs no direct
   morph, or streams updates.
4. Add an idempotent search endpoint returning HTML.
5. Add an HTTP PATCH endpoint returning a fragment.
6. Find all commands and explain whether they are JSON APIs.
7. Diagnose one illegal pairing and one old-form migration.

Record task completion, incorrect predictions, and questions asked. The
proposal should not be stabilized merely because a single snippet looks more
familiar.

## Recommendation

Adopt verb-first, role-explicit declarations as the proposed high-level
contract:

```text
@get:view
@get:fragment
@post:fragment       @post:command
@put:fragment        @put:command
@patch:fragment      @patch:command
@delete:fragment     @delete:command
@get:live(path)
```

Reject bare-method defaults and illegal method-role pairs. Permit `live` only
with GET and require an authored path; use coarse page/coherence streams rather
than one stream per component. Remove ordinary-client JSON from `command` in
the same boundary change, but treat its exact no-representation wire response
as a runtime gate. Expose patch-signals through low-level Roc transport
helpers, not declaration suffixes.[^live-path-plan]

This gives Rocci the approachability of conventional routing, the call-site
symmetry of Datastar, and the local response-policy clarity required by opaque
Roc bodies—without turning `.rocci` into a complete HTTP or Datastar protocol
DSL.[^implementation-plan]

[^template-ungram]: Current AST nodes for view, patch, command, and live declarations.
[^template-parser]: Current role-first recognition, hidden GET/POST defaults, allowed suffixes, and parser/body-opacity boundary.
[^template-lower]: Current route response metadata and generated command encoder adapter.
[^template-pprint]: Current handler inspection records kind, method, path, and role separately.
[^dispatch]: Shipped document-by-GET branch, patch-elements branch, empty-SSE/JSON command negotiation, and generated live stream.
[^handler-contract]: Frozen four-role syntax, accepted headers, result expectations, and `@patch:patch` naming evidence.
[^handler-syntax]: Accepted mutation suffixes and rejected GET fragment/command forms.
[^template-readme]: Owning crate documentation for the current public handler and live contracts.
[^server-reference]: Public current declaration and negotiated command behavior.
[^rendering-doc]: Public distinction among documents, fragments, commands, and streams.
[^custom-main]: Authored dispatcher already supports GET fragment routes outside generated standalone routing.
[^search-fragment]: Search input invokes a GET action for rendered results.
[^tabs-fragment]: Tab controls invoke GET actions for rendered panels.
[^datastar-crate]: Protocol crate owns broad Datastar request metadata and response framing.
[^datastar-roc]: Generated Roc helper exposes backend action builders and basic patch-elements framing.
[^datastar-sse]: Rust event builders cover patch-signals and other events beyond generated handlers.
[^prior-research]: Earlier research bounded high-level handlers to UI workflows and left the fragment noun unresolved.
[^prior-plan]: Earlier implementation plan left role-first fragment spelling behind a gate.
[^implementation-plan]: Detailed phased proposal for the new source order and bounded response semantics.
[^live-path-research]: Follow-up finding that authored paths plus multiplicity make live a GET route role and require app-level stream binding.
[^live-path-plan]: Detailed plural stream metadata, injection, dispatch, tooling, examples, and validation sequence.
[^server-owned-state]: Durable domain state remains authoritative on the server; browser signals are ephemeral.
[^datastar-actions]: Datastar actions are method-first and accepted response content types are orthogonal to the request method.
[^datastar-backend]: Datastar request encoding and backend response handling for HTML, SSE, and JSON.
[^datastar-sse-reference]: Patch-elements and patch-signals are SSE event types.
[^datastar-signals]: JSON and SSE patch-signals update frontend signals rather than morphing HTML.
[^fastapi-routing]: FastAPI path operation decorators lead with methods such as GET and POST.
[^express-routing]: Express route methods correspond to HTTP methods.
[^spring-routing]: Spring supplies method-specific mapping annotations over its generic request mapping.
[^flask-routing]: Flask exposes method-specific route decorators.
[^axum-routing]: Axum builds method routers with functions such as `get`.
[^aspnet-routing]: ASP.NET minimal APIs expose `MapGet`, `MapPost`, and related method mappings.
[^phoenix-routing]: Phoenix router macros are named for HTTP methods.
[^actix-routing]: Actix supports method-named route attributes and method guards.
[^react-router-actions]: React Router exposes semantic route actions for mutations.
[^liveview-bindings]: Phoenix LiveView sends named UI events to semantic `handle_event` callbacks.
