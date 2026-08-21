---
type: Research Report
title: Semantic view, patch, command, and live declarations for Rocci handlers
description: "Prefer four semantic handler nouns: @view returns a GET document, @patch returns one-shot Html, @command returns Roc data for negotiated JSON or 204, and @live renders the shared stream."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-21T10:43:01Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: live-counter
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Live counter with a GET document, live render, and JSON commands
    author: process:git
    last_modified: 2026-08-21
  - id: template-ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci AST specification for LiveDecl and OnDecl
    author: process:git
    last_modified: 2026-08-21
  - id: template-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Rocci parser for @live and @on declarations
    author: process:git
    last_modified: 2026-08-21
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Rocci lowering and response-kind metadata
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated standalone dispatcher response behavior
    author: process:git
    last_modified: 2026-08-21
  - id: compile-tests
    resource: ../../crates/rocci-template/tests/compile.rs
    title: Standalone handler syntax and lowering tests
    author: process:git
    last_modified: 2026-08-21
  - id: server-actions-guide
    resource: ../../docs/guides/server-actions.rocdown
    title: Published Rocci server-actions guide
    author: process:git
    last_modified: 2026-08-21
  - id: rendering-model
    resource: ../../docs/concepts/rendering-model.rocdown
    title: Published Rocci rendering model
    author: process:git
    last_modified: 2026-08-21
  - id: cqrs-research
    resource: datastar-cqrs-action-responses.md
    title: Datastar CQRS and action response research
    author: process:cursor
    last_modified: 2026-08-21
  - id: server-owned-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar actions and response handling reference
    author: organization:star-federation
  - id: datastar-tao
    resource: https://data-star.dev/guide/the_tao_of_datastar
    title: The Tao of Datastar
    author: organization:star-federation
  - id: datastar-backend
    resource: https://data-star.dev/guide/backend_requests
    title: Datastar backend requests guide
    author: organization:star-federation
  - id: htmx-docs
    resource: https://htmx.org/docs/
    title: htmx requests and responses documentation
    author: organization:big-sky-software
  - id: react-router-actions
    resource: https://reactrouter.com/start/data/actions
    title: React Router actions
    author: organization:remix-software
  - id: remix-actions
    resource: https://v2.remix.run/docs/route/action/
    title: Remix route actions
    author: organization:shopify
  - id: sveltekit-actions
    resource: https://svelte.dev/docs/kit/form-actions
    title: SvelteKit form actions
    author: organization:svelte
  - id: next-actions
    resource: https://nextjs.org/docs/app/getting-started/mutating-data
    title: Next.js server actions and mutations
    author: organization:vercel
  - id: astro-actions
    resource: https://docs.astro.build/en/guides/actions/
    title: Astro Actions
    author: organization:withastro
  - id: phoenix-live-view
    resource: https://phoenix-live-view.hexdocs.pm/Phoenix.LiveView.html
    title: Phoenix LiveView lifecycle and callbacks
    author: organization:phoenix-framework
  - id: roc-json
    resource: https://www.roc-lang.org/docs/main/
    title: Roc standard-library JSON encoding API
    author: organization:roc-lang
  - id: implementation-plan
    resource: ../plans/action-handler-syntax.md
    title: Implementation plan for semantic handler declarations
    author: process:cursor
    last_modified: 2026-08-21
---

# Semantic view, patch, command, and live declarations for Rocci handlers

## Question and disposition

Should Rocci replace transport-shaped `@on:get` and `@on:post` declarations
with semantic handler declarations, make POST the default mutation method,
and let JSON-producing handlers return Roc data instead of authored strings?

Yes. The strongest model is four nouns, each naming one rendered or transport
role. Rocci has two intentional mutation flows: a direct patch returns HTML to
the acting tab, while a command returns 204 to Datastar or encoded JSON to an
ordinary API client and lets `@live` render shared UI. Making those different
declarations is clearer than making every author repeatedly decode a response
selector.[^server-actions-guide][^cqrs-research]

The recommended surface is:

```rocci
@view("/") = |{ db }| { ... }

@patch("/actions/todos/add") = |{ db }, request| { ... }

@command("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    { count }
}

@live = |{ db }| { ... }
```

- `@view(path)` means GET and a successful HTML document.
- `@patch(path)` means POST by default and a successful one-shot HTML patch.
- `@command(path)` means POST by default and a successful Roc value that the
  generated host encodes as JSON for ordinary clients; Datastar gets 204.
- `:put`, `:patch`, and `:delete` are explicit method overrides on both
  mutation declarations, for example `@command:delete(path)`.
- `@live` keeps its current meaning.
- The old `@on:method` surface is removed in the same cutover; existing source,
  examples, tests, documentation, and skills migrate together.

The four nouns make the handler's successful value predictable: `@view`,
`@patch`, and `@live` produce `Html`; `@command` produces JSON-encodable data.
Roc now supplies `Json.to_str` and `Json.to_str_try`, so the framework can own
serialization instead of requiring authors to interpolate JSON strings.[^roc-json]

This is exploratory syntax research, not approval to change the language.

## Shipped baseline

The current AST has one `OnDecl` with a method, path, optional response ident,
parameters, and body, plus a separate `LiveDecl`.[^template-ungram] Validation
allows `json` only on mutating methods, lowering classifies routes as `Patch`
or `Json`, and the generated dispatcher treats GET separately from both
mutation response modes.[^template-lower][^dispatch]

| Authored form | Successful handler value | Generated response | UI consequence |
| --- | --- | --- | --- |
| `@on:get(path)` | `Html` | `text/html` | Load a document |
| `@on:post(path)` | `Html` | one-shot patch-elements SSE | Morph the acting tab |
| `@on:post(path) json` | JSON `Str` | 204 for Datastar; JSON otherwise | No Datastar morph; API data |
| `@live` | `Html` | generated long-lived GET `/sse` | Morph every subscribed tab |

The live-counter demonstrates the current split: GET renders the initial
document, `@live` rereads state and renders `#counter`, and increment/reset
handlers return JSON strings while Datastar receives 204.[^live-counter]
Compiler tests enforce the current spelling, generated names, one/two-argument
handler adaptation, duplicate-route checks, response markers, live injection,
and example compilation.[^compile-tests]

The current syntax is technically explicit about HTTP method, but its product
meaning is distributed across three places: the method, the optional trailing
`json`, and the presence or absence of `@live`. A reader must know that a GET
returns a document, an unmarked mutation returns a patch, and a marked
mutation is a command. The public guide has to teach that matrix before the
first handler.[^server-actions-guide]

## Constraints that should not move

1. **HTML remains the rendered UI boundary.** Datastar patches HTML responses
   into the DOM, while `application/json` patches signals. Returning
   `{ "count": 5 }` does not update server-rendered `#counter` unless the UI
   duplicates count into a client signal.[^datastar-actions] Rocci's normative
   direction keeps durable state server-owned and visible state rendered as
   coherent HTML.[^server-owned-state]

2. **Direct patches and live CQRS both remain first-class.** Direct patches are
   the small path for search, edit, and validation. Live CQRS separates a
   long-lived read request from short write requests for shared views. The
   latter is not evidence that every mutation should return JSON.[^datastar-tao][^cqrs-research]

3. **A declaration must reveal its wire contract locally.** The same source
   line should make method and successful response class discoverable to an
   author, formatter, LSP, generated documentation, and code reviewer.

4. **No context-sensitive defaults.** An action must not silently change from
   HTML to JSON because the file later gains `@live`. That would make a distant
   declaration alter the return type and browser behavior of existing
   handlers.

5. **The parser cannot safely infer a response from the body.** Ordinary Roc
   bodies are opaque to the template parser, and Rocci deliberately does not
   type-check them. The response choice must be syntax metadata or a uniform
   runtime response value.[^template-parser][^template-ungram]

6. **Public URLs remain visible.** Rocci examples, `curl` smoke tests, hybrid
   proxying, and ordinary API clients benefit from stable authored routes.
   Opaque action URLs would trade away one of the system's useful HTTP
   properties.

## Datastar JSON is a transport instruction, not a DOM view

“JSON action” is ambiguous unless request direction, response consumer, and UI
ownership are named. Datastar uses JSON in two directions, and ordinary API
clients add a third interpretation:

| Direction and consumer | JSON means | What it does not mean |
| --- | --- | --- |
| Browser → server Datastar mutation | The request body normally carries the current filtered signals; form encoding is an explicit alternative | It is not necessarily a domain command DTO |
| Server → Datastar browser | `application/json` is a merge patch for frontend signals | It does not render or morph HTML |
| Server → ordinary HTTP client | An ordinary JSON representation for that client to decode | It has no inherent Datastar semantics |

Datastar's non-GET backend actions send signals as a JSON body by default. A
form can instead request form encoding, and a backend can decode the incoming
signals using its normal JSON facilities.[^datastar-backend] That is an **input
contract**. It is separate from the handler's successful response.

On the response side, Datastar dispatches by `Content-Type`:

| Response | Datastar behavior |
| --- | --- |
| `text/event-stream` | Process zero or more patch-elements, patch-signals, or script events |
| `text/html` | Morph returned top-level elements into the DOM |
| `application/json` | Merge the object into frontend signals |
| `204 No Content` | Finish successfully without a patch |

Therefore, this response:

```http
HTTP/1.1 200 OK
Content-Type: application/json

{"count": 5}
```

means “set `$count` to `5`” to Datastar. It does not mean “render the
server-owned counter again,” find `#counter`, or replace its `<output>`.
Rendering that JSON would require client-side bindings such as
`data-text="$count"`, which would move a copy of durable count into browser
state and create a second UI model. Rocci instead keeps SQLite authoritative
and sends server-rendered HTML through a direct patch or `@live`.[^datastar-actions][^server-owned-state]

This distinction is why a JSON response is legitimate for an ephemeral signal
such as open/closed or a form-local status, but is not Rocci's general answer
for rendering durable domain state. The media type is not wrong; it simply has
a specific browser meaning.

## How a command also serves an ordinary JSON API

Rocci's shipped `json` mode deliberately does **not** send its successful JSON
body to a Datastar request. Today the handler produces a JSON `Str`. Under the
proposal, a `@command` handler produces ordinary Roc data and generated
dispatch owns its encoding. In both cases dispatch then inspects the dedicated
`Datastar-Request: true` header:

```text
POST /actions/counter/increment
        |
        v
handler mutates SQLite and returns { count: 5 }
        |
        +-- Datastar-Request: true --> 204 No Content
        |                              `@live` renders HTML on GET /sse
        |
        `-- header absent ----------> encode data, then 200 application/json
                                       curl/API/test receives {"count": 5}
```

The `Accept` header is not a safe discriminator because a Datastar action can
accept event-stream, HTML, and JSON responses. The dedicated request header
identifies the Datastar transport path; an ordinary API client normally omits
it.[^cqrs-research] The generated dispatcher currently recognizes lowercase,
title-case, and uppercase spellings of the header, returns 204 on successful
Datastar commands, and returns the handler body with
`Content-Type: application/json` otherwise.[^dispatch]

This lets one stable, authored URL participate in two compatible workflows:

- **Live hypermedia UI:** POST expresses intent, 204 closes the write request,
  and the long-lived GET rereads canonical state and renders HTML to every tab.
- **Ordinary API client:** POST performs the same domain operation and receives
  the resulting JSON representation directly.

The separation avoids racing two render channels. If the command POST also
patched `#counter`, its one-shot render could arrive before or after a newer
render from `@live`. The command response therefore does not own the DOM in a
live flow.[^cqrs-research]

Errors are also consumer-aware in the shipped dispatcher. A failed Datastar
JSON action gets an HTML error-overlay patch so the acting developer sees the
failure in the page; a non-Datastar client gets status 500 and a JSON error
object.[^dispatch] That behavior belongs to response policy, not to the
domain mutation itself.

### What “pure JSON API” does and does not promise

For callers without `Datastar-Request`, a command is a real JSON HTTP
endpoint: it has an explicit path and method, returns
`application/json`, and is usable from `curl`, tests, another service, or a
non-Datastar application. It is not an opaque generated RPC URL.

The selector alone does not make a complete public API contract. A production
API still needs decisions about request schemas, authentication,
authorization, validation, status codes, versioning, idempotency, and typed
encoding. In particular, Datastar's default JSON request body contains
signals; an external client may send a narrower domain command. Sharing one
endpoint is straightforward only when the handler intentionally accepts both
input shapes or they are the same shape.[^datastar-backend]

There is also a meaningful distinction between:

- **negotiated JSON action:** 204 for Datastar, JSON for ordinary clients;
- **unconditional JSON resource:** JSON for every caller, even one that sends
  `Datastar-Request: true`.

The proposed `@command` mode preserves the first. A future unconditional API endpoint
should use an explicit low-level response/route facility rather than weakening
the Datastar command contract. Conversely, the language must not require
`@live` next to every command: an API-only module is valid, and a static
document may call the endpoint without expecting a DOM update.

### Why the command body should be data

Roc's current standard library exposes total and fallible JSON encoders. A
generated dispatcher can call `Json.to_str_try(data)` for ordinary HTTP
clients, return the existing consumer-aware 500 response if encoding fails,
and skip serialization entirely for a successful Datastar command because its
wire response is 204.[^roc-json][^dispatch] The Roc type checker still proves
that the success value has an encoder because the generated non-Datastar branch
is compiled with the handler.

This changes the authored contract from representation construction to data
construction:

```rocci
@command("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    { count, ok: True }
}
```

The generated host, not the domain handler, decides when to serialize that
record, which content type to attach, and whether Datastar should instead see
204. This removes manual escaping bugs, admits records and lists naturally,
and preserves the same pure JSON API. It does not turn JSON into a DOM view:
Datastar still receives no JSON body from this negotiated command policy.

## What related systems establish

The word *action* consistently means an invocation or mutation lifecycle. It
does not consistently mean JSON.

| System | Read/render side | Action side | Lesson for Rocci |
| --- | --- | --- | --- |
| Datastar / htmx | Server sends HTML to morph or swap | HTTP verb triggers a backend request; 204 is also valid | Hypermedia actions normally return HTML, so JSON cannot be the universal meaning of `action`.[^datastar-actions][^htmx-docs] |
| React Router | Loaders supply data and rerun after mutations | Route actions mutate data and are usually invoked with POST | “Action” can default to mutation while a separate read path refreshes UI.[^react-router-actions] |
| Remix | Loaders read; route components render | One action handles a non-GET request and may return data or redirect | Action is separated by request lifecycle, not fixed representation.[^remix-actions] |
| SvelteKit | Page `load` reruns after an action | Form actions always use POST and return serializable form/validation data | POST is a learnable default; automatic read rerun resembles live re-render, but the returned data is framework-owned page state.[^sveltekit-actions] |
| Next.js | Server components render UI | Server Actions are POST-only and can return updated UI and data | POST-only actions are familiar, but Next hides URLs and couples actions to React's transport, unlike Rocci's public endpoints.[^next-actions] |
| Astro | Pages render separately | Typed actions validate form/JSON input and return serialized data through generated callable functions | JSON/data actions work well when they are explicitly a separate RPC facility, not the only hypermedia mutation path.[^astro-actions] |
| Phoenix LiveView | `render/1` produces markup repeatedly | `handle_event` mutates per-connection socket state and may reply with a map | A strong render/event split is usable, but its retained per-client process is not Rocci's stateless server-owned model.[^phoenix-live-view] |

Three transferable findings stand out:

- Default POST for an action is mainstream and removes noise without hiding a
  meaningful choice.
- Read/render and mutation are useful top-level concepts.
- Successful action representation remains architecture-specific. Systems
  either re-render, return HTML, return data, redirect, or combine those. The
  declaration must preserve Rocci's own distinction rather than borrowing the
  name and assuming JSON.

## Alternatives

Scores are directional design heuristics from 1 (weak) to 5 (strong), not
user-test results. “Fit” means compatibility with both shipped Rocci update
patterns; “migration” rewards low language/tooling churn.

| Alternative | Learnability | Rocci fit | Local explicitness | Extension room | Migration | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A. Keep `@on:method`, optional `json` | 3 | 5 | 3 | 4 | 5 | 20 |
| B. `@view`; JSON-only `@action` | 5 | 2 | 5 | 2 | 2 | 16 |
| C. `@view`; POST `@action` requiring `[patch]` or `[json]` | 4 | 5 | 5 | 4 | 3 | 21 |
| D. Four nouns: `@view`, `@patch`, `@command`, `@live` | 5 | 5 | 5 | 5 | 2 | **22** |
| E. `@view`; `@action` returns a typed response ADT | 3 | 5 | 5 | 5 | 2 | 20 |

### A. Keep the current transport-shaped declaration

```rocci
@on:get("/") = |state| { ... }
@on:post("/actions/save") = |state| { ... }
@on:post("/actions/increment") json = |state| { ... }
```

This is the safest option. HTTP expertise transfers directly, all methods fit,
and it already has parser, formatter, lowering, LSP, fixture, documentation,
and generated-dispatch coverage.[^template-parser][^compile-tests] Its main DX
cost is that `on` says when a handler runs but not its UI role, and the bare
trailing `json` reads like an exception bolted onto the route.

### B. Split views from JSON-only actions

```rocci
@view("/") = |state| { page(...) }
@action("/actions/increment") = |state| { { count: 1 } }
```

This is excellent for the live-counter in isolation. It is short, makes POST
implicit, and presents a clean read/write story. Applied as the whole language,
it creates larger usability costs:

- The starter counter and direct forms lose the natural server-rendered patch
  response or need a second escape syntax.
- A Datastar caller does not use the JSON body for DOM rendering; it receives
  204 under the shipped negotiation. Calling the contract “actions return
  JSON” therefore describes the API client but not the interactive browser.
- The name still collapses action lifecycle and JSON representation into one
  concept, leaving no semantic home for direct HTML mutations.
- Validation, redirect, no-content, and downloadable responses do not fit a
  single JSON rule cleanly.

This option is coherent only if Rocci intentionally becomes live-CQRS-only and
withdraws direct HTML actions. Current examples and rendering guidance do not
support that product change.[^rendering-model][^server-actions-guide]

### C. Split by role and require a bracketed response selector

```rocci
@view("/") = |{ db }| { counterPage(...) }

@action[patch]("/actions/search") = |{ db }, request| {
    results(...)
}

@action[json]("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    { count }
}
```

This is a strong compact design. It gives `view` and `action` their useful
semantic meaning, makes POST the terse default, and exposes the choice that
materially changes browser behavior. The bracket is intentionally required;
a novice can inspect one line and know whether the acting request morphs HTML.

For less common verbs:

```rocci
@action:put[patch]("/actions/profile") = |state, request| { ... }
@action:delete[json]("/actions/todos/42") = |state| { ... }
```

Of the three proposed selector spellings, `@action:delete[patch](path)` is the
only one with a stable left-to-right grammar: declaration, optional method,
required response, then route. `@action[patch]:delete(path)` reverses the
existing modifier order, while `@action(path) -> patch` resembles a Roc return
type even though `patch` is generated response policy rather than the complete
handler type.

The remaining cost is conceptual: every action declaration carries a small
two-option type system, and the central difference between direct UI work and
a command is expressed as punctuation inside the same noun. Four top-level
nouns make that architectural distinction easier to search, explain, and
diagnose.

### D. Give every response role its own declaration

```rocci
@view("/") = |state| { ... }
@patch("/actions/search") = |state, request| { ... }
@command("/actions/increment") = |state| { ... }
@live = |state| { ... }
```

This is the clearest architecture diagram in source: documents, one-shot
patches, commands, and streams each have a word. It also maps neatly onto the
four shipped dispatcher branches and makes the expected body value obvious:
`Html` for view/patch/live and encodable data for command. Roc's encoder makes
that command contract practical without a framework response ADT.[^roc-json]

The DX risks are vocabulary and name collision. “Command” introduces a small
amount of CQRS terminology, although it accurately describes a short write
whose render is owned elsewhere. More importantly, `patch` can mean either
the declaration's HTML response effect or the HTTP PATCH method. The grammar
keeps them in separate positions:

```rocci
@patch("/actions/save") = |state| { ... }          # POST, Html patch
@patch:delete("/actions/item") = |state| { ... }   # DELETE, Html patch
@patch:patch("/actions/item") = |state| { ... }    # PATCH, Html patch
```

The doubled form is awkward but honest and uncommon. If usability testing
shows that authors consistently read `@patch` as the HTTP method, rename the
response declaration to `@fragment`; do not hide the distinction by changing
its semantics. With that explicit naming gate, the four-noun design is the
recommended experiment.

### E. Make actions return a typed response value

```rocci
@action("/actions/save") = |state, request| {
    Action.patch(form(...))
}

@action("/actions/increment") = |state| {
    Action.json({ count: increment_count!(state.db)? })
}
```

A response ADT could eventually support `Patch Html`, typed `Json a`,
`NoContent`, `Redirect Str`, and custom status without adding declaration
keywords. Roc would enforce the returned variant. This is the best extension
ceiling, but it makes the common path more ceremonial, moves a crucial wire
choice into an opaque body for tooling, and requires a new runtime response
API even though JSON encoding itself is now available. It should be evaluated
separately from handler naming, not smuggled into a syntax rename.

## Detailed recommendation

### Surface contract

Adopt option D as semantic declarations over the existing route model first:

```text
@view(path)                    = GET, document Html
@patch(path)                   = POST, one-shot patch Html
@patch:METHOD(path)            = explicit PUT/PATCH/DELETE, patch Html
@command(path)                 = POST, encodable data, negotiated 204/JSON
@command:METHOD(path)          = explicit PUT/PATCH/DELETE, negotiated 204/JSON
@live                          = unchanged live Html renderer
```

This is a replacement, not an alias layer. Remove `OnDecl`, the trailing
`json` response marker, and the pre-encoded JSON success path; convert every
active handler, fixture, example, documentation snippet, and skill in the same
change. Old syntax may receive a removal diagnostic, but it does not parse or
lower as a supported declaration.

POST has exactly one spelling: omit it. Reject `@patch:post` and
`@command:post` with a fix-it diagnostic. Reject GET on both mutation forms;
use `@view` for the supported GET document contract or `@live` for the
generated stream. A future non-document GET resource needs its own explicit
contract rather than keeping `@on` as an untyped escape hatch. PUT, PATCH, and
DELETE remain visible overrides.

Do not change error negotiation, handler arity, route naming, or Datastar
client syntax in the same experiment. The semantic declarations normalize to
the existing `RouteInfo`; only a command success value changes from authored
JSON text to generated encoding.[^template-lower][^dispatch]

### Why the response distinction belongs in syntax

The distinction is important in Rocci syntax for reasons beyond readability:

1. **It selects generated dispatch.** Patches and commands produce different
   statuses, content types, error responses, and Datastar effects. This is
   route metadata, not a formatting preference.[^dispatch]
2. **The body is opaque at the owning layer.** `rocci-template` copies ordinary
   Roc and cannot inspect whether a value is `Html` or encodable data.
   Return-type inference is unavailable when route metadata is generated.
   [^template-parser][^template-ungram]
3. **It prevents dangerous inference from `@live`.** Adding or removing a live
   renderer must not silently change an existing mutation's successful response
   from a DOM patch to 204/JSON.
4. **It makes API intent searchable.** Inspect output, LSP symbols,
   documentation generation, security review, and tests can inventory command
   endpoints without interpreting Roc bodies.
5. **It supports useful diagnostics.** The compiler can explain that `@patch`
   requires an `Html`-producing flow and that `@command` returns API data; it
   is not a request to morph returned JSON into the DOM.
6. **It preserves API-only commands.** A command endpoint remains locally
   explicit and valid even when there is no `@live` renderer.

The nouns select successful **response policies**, not literal wire encodings
for every client:

| Declaration | Handler success value | Datastar response | Ordinary-client response |
| --- | --- | --- | --- |
| `@view` | `Html` | GET document HTML | the same document HTML |
| `@patch` | `Html` | one patch-elements SSE response | the same SSE wire response today |
| `@command` | JSON-encodable Roc data | 204, with UI owned elsewhere | encoded 200 `application/json` |
| `@live` | `Html` | long-lived patch-elements SSE | the same SSE stream |

In particular, `@command` means “select the negotiated command/API policy,”
not “always put JSON bytes on the wire.” That fuller meaning must be prominent
in diagnostics and documentation.

### Why `@command` is better than `@json`

`json` names only the ordinary-client representation. Datastar receives 204,
and a live renderer—not returned JSON—owns the shared DOM. `@command` names the
operation consistently for both consumers, while the body can return a typed
record rather than pretending to be wire bytes. Documentation must still say
that commands are encoded as JSON for ordinary clients and do not themselves
morph HTML for Datastar.[^dispatch][^datastar-actions][^roc-json]

### Naming gates: `@view` versus `@page`, `@patch` versus `@fragment`

`@view` describes an effectful handler that loads state and returns HTML.
`@page` is more precise about the current GET
contract and avoids collision with pure component “views,” but it can suggest
file-system routing or static content that Rocci does not have. Before a syntax
freeze, test both in complete examples and diagnostics:

```rocci
@view("/") = |state| { counterPage(...) }
@page("/") = |state| { counterPage(...) }
```

The recommendation uses `@view` because it is the clearest peer to patch,
command, and live. The contract must call it a **view handler**, not a pure
view: effects belong in the handler; reusable rendering remains
`@component`.[^rendering-model]

Test `@fragment` specifically against `@patch`. `@fragment` avoids collision
with HTTP PATCH but describes a value more than the resulting DOM effect and
is less aligned with Datastar vocabulary. Keep `@patch` unless readers
repeatedly mistake it for a method declaration in complete examples.

## Usability test before a language decision

Test the proposal on four complete tasks, not isolated syntax snippets:

1. First one-shot counter: a POST mutates SQLite and patches `#counter`.
2. Live counter: a POST command returns data/204 and `@live` updates two tabs.
3. Validation form: the patch rereads submitted input and returns an HTML
   error region to only the acting user.
4. API-oriented delete: a non-POST override returns JSON to `curl` while the
   Datastar request gets no content.

For each task measure:

- whether a new author predicts which tab or tabs update;
- whether they know the successful handler value (`Html` versus encodable data);
- whether they can find method, route, and handler role from source search;
- whether diagnostics lead them from a missing/wrong mode to a working flow;
- whether changing one-shot to live requires local, reviewable edits;
- whether `@view` is mistaken for `@component` or `@live`;
- whether `@patch` is mistaken for the HTTP PATCH method, especially beside
  `@patch:delete` and `@patch:patch`.

The syntax should not ship from line-count preference alone. Its value is that
authors correctly predict network and rendering behavior.

## Migration and implementation implications

Even when normalized to existing routes, this is a language change. It touches
the AST specification, scanner/parser recovery, formatter, validation,
lowering, inspect output, LSP symbols, syntax fixtures, examples, crate README,
and public reference and server-action guides.
[^template-ungram][^template-parser][^compile-tests]

A companion implementation plan now records the staged work and build gates.
Its essential sequence is:[^implementation-plan]

1. Replace `OnDecl` with `ViewDecl`, `PatchDecl`, and `CommandDecl`; validate
   method overrides and duplicate routes.
2. Make command success values data and serialize them in generated dispatch
   with Roc's fallible encoder for ordinary clients.
3. Add recovery and monotonic-progress cases for every malformed header.
4. Teach formatter, AST inspection, symbols, highlighting, and source maps.
5. Build a complete method/response example plus the counter, live-counter,
   and Rocdown island with the pinned Roc compiler.
6. Decide `@patch` versus `@fragment` from complete examples before freezing
   the surface.

Do not combine that plan with a response ADT, route-name generation, automatic
form decoding, authentication policy, or a pub/sub runtime. Those are separate
product contracts.

## Conclusion

Rocci benefits from being opinionated at the level users reason about:
documents, one-shot patches, commands, and shared live renders. The four-noun
surface—**`@view`, `@patch`, `@command`, and `@live`**—makes those roles and
their expected Roc values locally obvious. POST is implicit for patches and
commands; PUT, PATCH, and DELETE remain explicit suffixes. `@command` returns
data which the generated host encodes for ordinary clients while preserving
204 for Datastar.

Make the cutover repository-wide without aliases or pre-encoded JSON-text handling.
Keep `@fragment` as the named fallback only if complete-example testing shows
that `@patch` is consistently confused with the HTTP method.

[^live-counter]: The file uses `@on:get`, `@live`, and two `@on:post ... json` handlers over the same SQLite count.
[^template-ungram]: `OnDecl` stores method, path, optional response ident, params, and body; `LiveDecl` is separate; ordinary Roc expressions remain opaque.
[^template-parser]: The hand-written parser recognizes and recovers `@live` and `@on:method("path")` headers before opaque Roc bodies.
[^template-lower]: Lowering maps response metadata to `RespondKind::Patch` or `RespondKind::Json` and emits stable generated handler functions.
[^dispatch]: Generated dispatch serves GET as HTML, patch mutations as one-shot patch-elements SSE, and JSON mutations as 204 for `Datastar-Request` or JSON otherwise.
[^compile-tests]: Tests cover handler spelling, arity adaptation, duplicates, JSON restrictions, live conflicts/injection, and both counter examples.
[^server-actions-guide]: The public guide documents direct one-shot Html actions and live `@live` plus `json` commands as separate shipped patterns.
[^rendering-model]: The public model separates effectful GET documents, mutation fragments, JSON commands, live fragments, and pure components.
[^cqrs-research]: The prior exploratory record establishes that one-shot patches and live CQRS solve different flows and must not patch the same target concurrently.
[^server-owned-state]: The normative decision keeps durable state on the server and renders coherent HTML rather than mirroring the domain into browser state.
[^datastar-actions]: Datastar accepts SSE, HTML, JSON, and 204; HTML patches elements while JSON patches signals.
[^datastar-tao]: Datastar recommends backend authority, HTML morphing, sparse signals, and CQRS with a long-lived read request plus short write requests.
[^datastar-backend]: Datastar sends signals in a JSON body for non-GET backend requests by default and supports explicit form encoding as an alternative.
[^htmx-docs]: htmx documents HTML fragments as the typical action response and 204 as a no-swap success response.
[^react-router-actions]: React Router defines actions as data mutations and revalidates loader data when an action completes.
[^remix-actions]: Remix invokes a route action for non-GET methods and allows the action to return data or a redirect.
[^sveltekit-actions]: SvelteKit form actions always use POST, may return serializable form data, and rerun page loads after completion.
[^next-actions]: Next.js Server Actions use POST and may return updated UI and data, while generated invocation hides the explicit route.
[^astro-actions]: Astro Actions are typed backend functions with validated form/JSON input and serialized data/error results, separate from ordinary endpoints.
[^phoenix-live-view]: Phoenix LiveView separates rendering from event handling but retains a stateful socket process for each connected live view.
[^roc-json]: Roc's current standard library exposes `Json.to_str` for total encoders and `Json.to_str_try` for encoders that can fail; a local Rocci build probe confirmed record encoding with the pinned nightly compiler.
[^implementation-plan]: The companion draft plan phases grammar, lowering, encoded command results, editor support, migration, complete examples, and real Roc compilation gates.
