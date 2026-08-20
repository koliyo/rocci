---
type: Research Report
title: Optional request argument on Rocci service handlers
description: "Shipped convenience: one-parameter `@on` handlers inject unused `_request` at lowering. Follow-on shapes (dispatch arity, request as a handler-input field) stay exploratory and are not shipped."
tags: [domain/rocci, concern/syntax, concern/architecture, concern/runtime]
status: draft
generated: { by: process:cursor, at: 2026-08-20T08:00:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: lower-on
    resource: ../../crates/rocci-template/src/lower.rs
    title: "@on lowering, including unused `_request` injection"
    author: process:git
    last_modified: 2026-08-20
  - id: handler-arity
    resource: ../../crates/rocci-template/src/ast.rs
    title: handler_param_arity and ensure_handler_request_param
    author: process:git
    last_modified: 2026-08-20
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated dispatcher always calls handler!(context, request)
    author: process:git
    last_modified: 2026-08-20
  - id: rocci-ref
    resource: ../../docs/reference/rocci.rocdown
    title: Public `@on` handler contract
    author: process:git
    last_modified: 2026-08-20
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Template crate standalone HTTP contract
    author: process:git
    last_modified: 2026-08-20
  - id: route-info
    resource: ../../crates/rocci-template/src/lower.rs
    title: RouteInfo carries method, path, and fn_name only
    author: process:git
    last_modified: 2026-08-20
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Durable application state is server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: "@component is a pure render function"
    author: human:nils
    last_modified: 2026-08-16
---

# Optional request argument on Rocci service handlers

## Scope and authority

This record separates a **shipped lowering convenience** from **follow-on
authoring shapes that are not implemented**. It is exploratory on the
follow-ons. It does not approve a new architecture decision.[^rocci-ref]

## Constraint that does not move

Generated dispatch always calls `handler!(context, request)`. Roc has no
optional trailing arguments, so the generated function must accept two
parameters. An unused `request` binding is illegal unless it is `_`-prefixed,
which is why unused handlers historically wrote `_request`.[^dispatch][^rocci-ref]

`@context` / `@init` `State` is process-lifetime. The HTTP request is
per-call. Follow-ons must not put `request` on `State`.[^server-state][^lower-on]

## Shipped: inject `_request` at lowering

A one-parameter `@on` list (`|{ db }|`, `|state|`, `|_|`) and an empty
`||` list lower with unused `_request` appended. Two-parameter lists pass
through. Omitting the list still yields `|state, _request|`. Arity greater
than two is a diagnostic. `RouteInfo` and dispatcher call sites are
unchanged.[^handler-arity][^lower-on][^route-info][^template-readme]

This is source convenience only. Generated Roc remains two-argument.

## Alternative A: dispatch arity (not shipped)

Record whether the handler takes request on `RouteInfo` and emit
`handler!(context)` versus `handler!(context, request)`.

Pros: generated Roc matches the source list.

Cons: every `RouteInfo` consumer (CLI dispatch, Rocdown standalone, tests)
must branch. Unused two-argument handlers still need `_request` because of
Roc unused-variable rules. Mixed arity in one app is fine for Roc but
splits the generated dispatcher type story.

## Alternative B: request as a handler-input field (not shipped)

Authors would write `|{ db }|` or `|{ db, request }|` against a single
record. Dispatch would pass one value that includes both app state and the
current request.

This is the strongest authoring story **if** Roc record patterns allow
omitting unused fields. Open questions against the pinned nightly:

- Flatten at the call site (`{ db: context.db, request }`) versus nested
  `{ state: context, request }`. Nested form forces
  `|{ state: { db }, request }|` and is worse for the common case.
- Flattening without listing every `State` field needs record spread or a
  generated `HandlerIn` alias.
- Closed versus open records: can `|{ db }|` typecheck against a wider
  `{ db, request }` value without naming unused fields?
- `HandlerIn` must stay distinct from `State`. `@init` still produces
  process-lifetime state; only the handler call site adds `request`.

Do not treat this as “request is a default member of `@context`.” That
would lie about lifetime and mix durable state with a per-call value.[^server-state]

## Alternative C: implicit `request!` (reject)

A module-level or effectful `request!` helper would hide request lifecycle
inside handlers. That is against the explicit-argument model used by
standalone `@on` and is a different kind of hidden lifecycle than the
pure-render rule already forbids on `@component`.[^pure-render][^template-readme]

## Disposition

Keep lowering injection as the shipped default. Revisit alternative B only
after checking pinned Roc record openness and whether a generated
`HandlerIn` alias is cheaper than teaching `RouteInfo` about arity.

[^lower-on]: `@on` lowering wraps the body for `?` and emits a named function whose parameters come from the authored list after default-stripping and `_request` injection.
[^handler-arity]: Top-level `|a, b|` count; a record `{ db }` is one argument; `||` is zero.
[^dispatch]: `route_arm` formats `{handler}(context, request)` for every method.
[^rocci-ref]: Public contract that dispatch calls `handler!(context, request)` and that one-parameter lists inject unused `_request`.
[^template-readme]: Standalone HTTP section documents the same generated call shape.
[^route-info]: `RouteInfo` is method, path, fn_name, and span; it does not record arity.
[^server-state]: Durable application state is server-owned and produced by `@init`, not per HTTP call.
[^pure-render]: `@component` lowers to an ordinary function from explicit values to `Html` and does not own request lifecycle.
