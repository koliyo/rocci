---
type: Research Report
title: Bound Rocci handlers to server-rendered UI
description: "The shipped semantic handlers improved local readability, but negotiated command JSON pulls a partial API contract into .rocci while GET fragments and low-level Datastar signal events belong at different layers."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T09:33:26Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: template-ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci AST specification for semantic handlers
    author: process:git
    last_modified: 2026-08-21
  - id: template-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Rocci semantic-handler parser and removal diagnostics
    author: process:git
    last_modified: 2026-08-21
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Rocci route metadata and handler lowering
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated document, patch, command, and live dispatch
    author: process:git
    last_modified: 2026-08-21
  - id: handler-contract
    resource: ../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen semantic handler contract
    author: process:git
    last_modified: 2026-08-21
  - id: handler-syntax
    resource: ../../crates/rocci-template/tests/handler_syntax.rs
    title: Accepted and rejected handler syntax tests
    author: process:git
    last_modified: 2026-08-21
  - id: server-reference
    resource: ../../docs/reference/language/server.rocdown
    title: Current public server declaration reference
    author: process:git
    last_modified: 2026-08-21
  - id: rendering-doc
    resource: ../../docs/concepts/documents-fragments-commands-streams.rocdown
    title: Current document, fragment, command, and stream model
    author: process:git
    last_modified: 2026-08-21
  - id: live-counter
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Generated live stream with negotiated JSON commands
    author: process:git
    last_modified: 2026-08-21
  - id: custom-main
    resource: ../../examples/rocci/custom/datastar/main.roc
    title: Authored Datastar dispatcher with GET fragments
    author: process:git
    last_modified: 2026-08-20
  - id: search-fragment
    resource: ../../examples/rocci/custom/datastar/Search.rocci
    title: Search UI issuing a GET fragment request
    author: process:git
    last_modified: 2026-08-20
  - id: tabs-fragment
    resource: ../../examples/rocci/custom/datastar/Tabs.rocci
    title: Tabs UI issuing GET fragment requests
    author: process:git
    last_modified: 2026-08-20
  - id: datastar-crate
    resource: ../../crates/rocci-datastar/README.md
    title: Rocci Datastar protocol-layer responsibilities
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-roc
    resource: ../../crates/rocci-datastar/src/codegen/mod.rs
    title: Generated Roc Datastar helper surface
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-sse
    resource: ../../crates/rocci-datastar/src/sse/events.rs
    title: Rust patch-elements, patch-signals, removal, and script event builders
    author: process:git
    last_modified: 2026-08-17
  - id: cqrs-research
    resource: datastar-cqrs-action-responses.md
    title: Datastar per-request SSE and generated CQRS research
    author: process:cursor
    last_modified: 2026-08-21
  - id: server-owned-state
    resource: ../decisions/server-owned-state.md
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
  - id: original-plan
    resource: ../plans/action-handler-syntax.md
    title: Original semantic-handler cutover plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: follow-up-plan
    resource: ../plans/handler-ui-boundary.md
    title: Follow-up plan for a bounded Rocci UI handler surface
    author: process:cursor
    last_modified: 2026-08-22
  - id: verb-first-research
    resource: verb-first-handler-declarations.md
    title: Follow-up research on verb-first handler declarations
    author: process:cursor
    last_modified: 2026-08-22
  - id: verb-first-plan
    resource: ../plans/verb-first-handler-declarations.md
    title: Follow-up implementation plan for verb-first handler declarations
    author: process:cursor
    last_modified: 2026-08-22
---

# Bound Rocci handlers to server-rendered UI

## Question and disposition

The semantic `@view`, `@patch`, `@command`, and `@live` cutover improved the
old transport-shaped `@on:method` syntax: declarations now reveal whether a
body returns a document, a one-shot fragment, command data, or a live
fragment.[^template-ungram][^handler-contract] The shipped design nevertheless
mixes two product boundaries. `.rocci` is presented as a server-rendered UI
language, while `@command` also promises a negotiated ordinary-client JSON API
that the Datastar UI does not consume.[^server-reference][^dispatch]

Do not complete a cross-product of HTTP methods, media types, Datastar event
types, patch modes, and streaming behavior in Rocci declaration syntax. The
recommended follow-up is a bounded UI surface:

- keep a dedicated GET document declaration;
- admit one-shot **GET HTML fragments**, because they are an idiomatic UI read;
- keep mutation commands as no-direct-morph writes, but remove generated JSON
  API representation from their high-level contract;
- keep `@live` as the generated shared HTML read stream;
- expose patch-signals and other advanced Datastar events through low-level Roc
  transport helpers and authored `main.roc`, not new declaration nouns.

The spelling of the fragment declaration remains a deliberate gate:
`@patch:get(path)` is the smallest compatible extension, while
`@fragment:get(path)` separates the response role from HTTP PATCH. Complete
examples and diagnostics should decide that noun before implementation.

This revision is exploratory research and does not approve or ship the
follow-up language change. The phased implementation proposal is the linked
[handler UI boundary plan](../plans/handler-ui-boundary.md).[^follow-up-plan]

Subsequent exploratory [verb-first research](verb-first-handler-declarations.md)
resolves this report's fragment-noun/source-order gate in favor of mandatory
`@method:role` headers and supplies a replacement
[implementation sequence](../plans/verb-first-handler-declarations.md). The UI
boundary in this report remains the foundation; neither follow-up is approved
or shipped.[^verb-first-research][^verb-first-plan]

## Shipped baseline

The current AST has separate `ViewDecl`, `PatchDecl`, `CommandDecl`, and
`LiveDecl` nodes. `@view` is fixed to GET; `@patch` and `@command` default to
POST and accept PUT, PATCH, or DELETE; GET is rejected for both mutation
declarations.[^template-ungram][^template-parser][^handler-syntax]

Lowering normalizes both view and patch declarations into route metadata whose
response kind is `Patch`; generated dispatch distinguishes a document by
checking `method == GET` before checking the response kind. Commands get a
separate generated JSON encoder wrapper.[^template-lower][^dispatch]

| Declaration | Successful body value | Shipped response |
| --- | --- | --- |
| `@view(path)` | `Html` | GET `text/html` document |
| `@patch[:method](path)` | `Html` | one patch-elements SSE event |
| `@command[:method](path)` | JSON-encodable Roc data | empty SSE for Datastar; `application/json` otherwise |
| `@live` | `Html` | generated long-lived GET `/sse` |

This is implemented behavior, not merely the original plan. The live-counter
uses `@command` result records solely for the ordinary-client JSON branch; its
Datastar caller receives empty SSE and waits for `@live` to render the updated
HTML. Public concepts describe the same document, fragment, command, and stream
split.[^live-counter][^dispatch][^cqrs-research][^rendering-doc]

The previous implementation plan remains useful as history for the clean
`@on` removal, but its JSON-command recommendation is superseded by this
follow-up investigation.[^original-plan]

## Permutation completeness is the wrong invariant

Datastar backend actions support GET, POST, PUT, PATCH, and DELETE. A response
may be SSE, HTML, JSON, JavaScript, or empty success; SSE may contain zero or
more element or signal patch events.[^datastar-actions][^datastar-sse-reference]
Those are orthogonal protocol dimensions, not evidence that every combination
deserves a Rocci declaration.

A syntax that attempted completeness would need to represent at least:

```text
HTTP method
× document / fragment / data / no-content response
× SSE event type and count
× selector and patch mode
× short response / long-lived stream
× Datastar / ordinary-client negotiation
```

That surface would make Rocci a declarative wrapper over Datastar and HTTP.
It would also be unstable as Datastar adds or revises transport options. Rocci
instead needs semantic coverage of its intended UI workflows and a lower-level
escape hatch for the protocol ceiling.

## Four different meanings of GET

“GET with JSON” is ambiguous. The useful cases are distinct:

| GET shape | Browser meaning | Recommended owner |
| --- | --- | --- |
| Full HTML document | Navigation or initial load | `@view` |
| One-shot HTML fragment | Search, tabs, lazy region, load more | Rocci fragment declaration |
| JSON response to Datastar | Patch ephemeral frontend signals | Low-level Datastar helper |
| JSON resource for an API client | Decode a data representation | Authored HTTP application |
| Long-lived SSE | Repeated shared HTML updates | `@live` |

Datastar sends current signals on GET in a JSON-encoded `datastar` **query
parameter**, not a GET request body. A JSON response patches frontend signals;
it does not morph server-rendered HTML.[^datastar-backend][^datastar-signals]
Therefore `@command:get` would not complete a coherent command matrix. A
command is a write; a GET JSON resource or GET signal patch has a different
semantic role.

The conspicuous missing high-level case is GET **HTML fragment**, not GET JSON.
The custom Datastar example already uses authored GET handlers for search
results and tab panels, but the generated standalone `.rocci` dispatcher
cannot express the same route because `@patch:get` is rejected.[^custom-main]
[^search-fragment][^tabs-fragment][^handler-syntax]

## Why negotiated command JSON crosses the UI boundary

The current command policy makes one route serve two consumers:

```text
Datastar request  -> run write -> empty SSE -> @live renders HTML
ordinary request -> run write -> encode returned Roc data as JSON
```

This is convenient for `curl` demonstrations, but it silently creates a
partial API contract. A production API still requires request schemas,
authentication, authorization, status semantics, versioning, idempotency, and
stable representations. Datastar mutation bodies normally contain frontend
signals, which need not match the domain command body expected by another
client.[^datastar-backend][^cqrs-research]

The cost appears in the high-level handler itself: authors construct and keep
JSON-encodable result data even though the UI ignores it, generated dispatch
adds an encoder branch, and documentation must explain two success contracts.
Supporting GET JSON, redirects, downloads, custom status codes, and API errors
would then be the logical but undesirable next step.

`@command` still has a clear UI meaning without JSON: perform a mutation and
do not directly morph a region that is owned by `@live`. The successful body
should be constrained to `{}`. The runtime may preserve empty SSE for
Datastar because of the pinned host workaround and return representation-free
success to an ordinary caller, but neither branch should promise API data.
[^dispatch][^cqrs-research]

General JSON resources should remain in authored `main.roc`. If repeated
applications later demonstrate demand for a typed low-level route facility,
evaluate one response ADT or `Server.Outcome` escape hatch separately; do not
grow one declaration per media type.

## Patch-signals belongs below declaration syntax

Patch-signals is a legitimate Datastar UI operation. It adds, updates, or
removes frontend signals and can be encoded as `application/json` or as a
`datastar-patch-signals` SSE event.[^datastar-signals]

Rocci's server-owned-state decision reserves browser signals for ephemeral UI
state rather than durable domain authority.[^server-owned-state] That makes
patch-signals an optimization or interaction primitive, not a peer to
documents and fragments in the default rendering model.

The repository already has two different capability levels:

- the Rust `rocci-datastar` protocol crate models patch-elements,
  patch-signals, removals, and script events;[^datastar-crate][^datastar-sse]
- the generated Roc `Datastar` helper exposes backend action strings and only
  a basic `patch_elements` event builder.[^datastar-roc]

The correct follow-up is to add a typed Roc `patch_signals` event builder for
authored servers, after checking it against the pinned Datastar protocol. Do
not add `@signals`, `@script`, `@json`, or patch-mode declaration variants.
Custom `main.roc` can compose mixed or multi-event streams while ordinary
`.rocci` handlers retain predictable return values.

## Options considered

| Option | Strength | Failure |
| --- | --- | --- |
| Keep shipped syntax unchanged | No migration | Retains partial API promise and lacks GET fragments |
| Complete every method/response permutation | Protocol coverage | Combinatorial syntax and transport-driven language |
| Add `@json:get`, `@signals`, and similar nouns | Locally explicit | Media types and Datastar events become language concepts |
| General response ADT in every handler | Complete extension ceiling | Makes the common UI path ceremonial and weakens inspectability |
| Bounded UI declarations plus low-level Roc transport helpers | Small semantic surface and an escape hatch | Requires a clean-cut command change and a fragment naming decision |

The final option is recommended. A general response ADT remains a possible
future low-level facility, not part of this change.

## Recommended high-level contract

| Role | Method policy | Body result | UI effect |
| --- | --- | --- | --- |
| Document | GET only | complete `Html` document | navigation or initial load |
| Fragment | GET plus mutation methods | stable-ID `Html` fragment | one-shot morph in acting tab |
| Command | POST/PUT/PATCH/DELETE | `{}` | write completes; rendering owned elsewhere |
| Live | generated GET `/sse` | stable-ID `Html` fragment | repeated morph for each subscriber |

The fragment noun is unresolved:

- **Keep `@patch`:** smallest migration; `@patch:get` reads as response effect
  plus method, but `@patch:patch` remains awkward.
- **Rename to `@fragment`:** clearly describes the body value and makes
  `@fragment:get` natural, but causes another clean-cut syntax migration.

Do not choose from isolated snippets. The gate must compare search, tabs,
validation, mutation patches, handler inspection, diagnostics, and the
complete syntax fixture before freezing the noun.

## Consequences

- `.rocci` has an explicit product boundary: server-rendered UI, not a general
  API framework.
- GET fragment reads become expressible without an authored dispatcher.
- `@command` no longer requires unused result records or promises JSON schemas.
- Datastar signal and mixed-event capabilities remain available without
  multiplying language declarations.
- Existing JSON-command examples and ordinary-client smoke tests require a
  clean migration.
- API-only `.rocci` modules cease to be a supported high-level shape; authored
  `main.roc` owns them.
- The parser still cannot prove that a GET body is side-effect free. The
  idempotent-read rule is a documented HTTP contract unless Roc gains a useful
  effect-level enforcement mechanism.

## Recommendation

Proceed only through the linked plan's Phase 0 naming and wire-policy gate.
If that gate confirms the bounded surface, implement response-role metadata
before adding GET fragments, remove command JSON in the same clean cut, and
add patch-signals only to the low-level Roc Datastar helper. Do not start from
a protocol permutation table.[^follow-up-plan]

[^template-ungram]: Current AST nodes for view, patch, command, and live declarations.
[^template-parser]: Current method defaults, accepted overrides, and GET rejection.
[^template-lower]: Current route response metadata and generated command JSON adapter.
[^dispatch]: Shipped document-by-GET branch, patch-elements branch, empty-SSE Datastar command branch, ordinary JSON command branch, and generated live stream.
[^handler-contract]: Frozen four-role syntax and result expectations.
[^handler-syntax]: Accepted mutation suffixes and rejected `@patch:get` / `@command:get` forms.
[^server-reference]: Public current handler matrix and negotiated command behavior.
[^rendering-doc]: Public distinction among documents, fragments, commands, and streams.
[^live-counter]: Command result records coexist with live HTML rendering.
[^custom-main]: Authored route dispatch supports GET search and tab fragments.
[^search-fragment]: Search input invokes a GET action for rendered results.
[^tabs-fragment]: Tab controls invoke GET actions for rendered panels.
[^datastar-crate]: Protocol/tooling crate owns broader Datastar metadata and framing.
[^datastar-roc]: Generated Roc helper has backend action builders and basic patch-elements only.
[^datastar-sse]: Rust event builders cover patch-signals and other protocol events beyond generated handlers.
[^cqrs-research]: One-shot versus live transport, empty-SSE host workaround, and API-input caveats.
[^server-owned-state]: Durable state stays authoritative on the server; signals are ephemeral.
[^datastar-actions]: Backend methods and accepted response content types are orthogonal.
[^datastar-backend]: GET signal query encoding and non-GET signal request bodies.
[^datastar-sse-reference]: Zero-or-more patch-elements and patch-signals SSE events.
[^datastar-signals]: JSON and SSE patch-signals update frontend signals rather than morphing HTML.
[^original-plan]: Historical cutover from `@on` to semantic declarations and generated JSON encoding.
[^follow-up-plan]: Phased naming gate, role-metadata change, command simplification, and low-level signal helper.
[^verb-first-research]: Follow-up comparison of conventional routers, Datastar call-site symmetry, source order, explicit roles, and the closed legal matrix.
[^verb-first-plan]: Detailed clean-cut implementation sequence for mandatory verb-first route declarations.
