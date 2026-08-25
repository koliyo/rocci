---
type: Research Report
title: Method-role handlers as a pure Roc library or platform
description: "Counterfactual: the shipped @method:role matrix can be encoded as Roc constructors on basic-webserver, with a custom platform buying little DX. That matches Datastar SDKs and eases the authored-main.roc cliff, but loses header inspectability, pre-Roc illegal-pair errors, and one-file HTML apps unless @component stays."
tags: [domain/rocci, domain/runtime, integration/datastar, integration/roc, concern/architecture, concern/developer-experience, concern/language-design]
status: draft
generated: { by: process:cursor, at: 2026-08-25T17:20:00Z }
stale_after: 2026-11-24
authority: exploratory
owners: [human:nils]
sources:
  - id: ecosystem
    resource: https://github.com/koliyo/rocci/blob/main/knowledge/research/method-role-handlers-datastar-ecosystem.md
    title: Post-landing method-role matrix compared with the Datastar ecosystem
    author: process:cursor
    last_modified: 2026-08-24
  - id: verb-first
    resource: verb-first-handler-declarations.md
    title: Pre-landing rationale for mandatory roles because the template parser cannot inspect Roc types
    author: process:cursor
    last_modified: 2026-08-24
  - id: handler-contract
    resource: ../../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen eleven-pair matrix and command wire policy
    author: process:git
    last_modified: 2026-08-22
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: legal_pair and duplicate method-plus-path checks
    author: process:git
    last_modified: 2026-08-22
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Route lowering and singleton live data-init injection
    author: process:git
    last_modified: 2026-08-22
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated document, fragment, command, and live wraps on basic-webserver
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Public handler contract and authored main.roc escape hatch
    author: process:git
    last_modified: 2026-08-23
  - id: datastar-runtime
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Generated Roc Datastar helpers already used as a library
    author: process:git
    last_modified: 2026-08-22
  - id: html-runtime
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Thin Html wrapper over platform element constructors
    author: process:git
    last_modified: 2026-08-15
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-file GET view plus POST fragment counter
    author: process:git
    last_modified: 2026-08-23
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: GET view, POST command, GET live
    author: process:git
    last_modified: 2026-08-23
  - id: custom-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Authored dispatcher with prefix matches and mixed SSE events
    author: process:git
    last_modified: 2026-08-23
  - id: snake
    resource: ../../../examples/rocci/custom/snake/main.roc
    title: Custom Sse.unfold live ceiling
    author: process:git
    last_modified: 2026-08-23
  - id: server-ref
    resource: ../../../docs/reference/language/server.rocdown
    title: Public server declaration reference
    author: process:git
    last_modified: 2026-08-22
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Components lower to pure Roc Html functions
    author: human:nils
    last_modified: 2026-08-16
  - id: datastar-adr
    resource: https://github.com/starfederation/datastar/blob/develop/sdk/ADR.md
    title: Datastar SDK architecture decision record
    author: organization:star-federation
  - id: datastar-sdks
    resource: https://data-star.dev/reference/sdks
    title: Official Datastar language SDKs
    author: organization:star-federation
  - id: bws-sse
    resource: basic-webserver-sse-http.md
    title: basic-webserver SSE idle-timeout limits
    author: process:cursor
    last_modified: 2026-08-24
---

# Method-role handlers as a pure Roc library or platform

## Scope and authority

This record is a **counterfactual design**, not a description of shipped
behavior and not an implementation plan. It asks: if the landed
`@method:role(path)` product were expressed as ordinary Roc — a package on
[basic-webserver](https://github.com/roc-lang/basic-webserver), or a Roc
platform that replaced that host — how would it be shaped, and how would
authoring DX compare with the `.rocci` DSL?[^ecosystem][^template-readme]

Prefer the post-landing Datastar ecosystem comparison for what the matrix
is relative to Datastar SDKs. Prefer the
[verb-first research](verb-first-handler-declarations.md) for why role is in
the header. This record only answers the Roc-library fork those papers leave
open.

Composite DX judgments are **exploratory synthesis**.

## For a later agent

- **Authority:** exploratory. Do not treat sketched Roc signatures as a
  public API, and do not start a cutover from this file.
- **Do not** encode Datastar SSE policy in the `.rocci` parser. A Roc
  library would own that policy in dispatch functions, which is where it
  already lives for generated apps.[^dispatch][^ecosystem]
- Keep three designs distinct: (1) **library on basic-webserver**, (2)
  **custom Roc platform**, (3) **hybrid** that keeps `@component` / `@css`
  and moves only routes into Roc. (1) is the realistic SDK analogue. (2)
  is mostly who owns `respond!`. (3) is the DX-relevant product fork.
- Verify any "shipped" claim against `handler_contract.rs`, `legal_pair`,
  and `dispatch.rs` before repeating it.

## What the DSL actually compiles to

The language feature is a **compile-time classifier**. Headers become Roc
functions; generated `main.roc` supplies `program = { init!, respond!, shutdown! }`
for basic-webserver and wraps each return value.[^dispatch][^template-readme][^handler-contract]

| Role | Author returns | Wrap already written in Roc |
| --- | --- | --- |
| `view` | `Html` | `200` `text/html` |
| `fragment` | `Html` | one `Datastar.patch_elements` event, then end |
| `command` | `{}` | empty SSE if `Datastar-Request`, else `204` |
| `live` | `Html` | `Sse.unfold!` poll, patch on `Html.render` change, keepalive otherwise |

`Datastar.roc` and `Html.roc` in the CLI runtime are already a Roc **library
surface**: `patch_elements`, `patch_signals`, `@post(...)` string builders,
`Html.element`.[^datastar-runtime][^html-runtime] The DSL does not replace
that library. It replaces **route registration and wrap selection**, because
the template parser cannot inspect Roc types or the handler body when it
builds the AST.[^verb-first]

Authored `main.roc` is the existence proof of the counterfactual. The
Datastar gallery hand-writes `match (method, path)`, prefix `Str.starts_with`
for `/actions/tabs/{id}`, `patch!` / `events!` helpers, and mixed
`patch_elements` plus `patch_signals` on one GET. Snake hand-writes a custom
`Sse.unfold!`. Those files are the SDK style the closed grammar refuses to
be.[^custom-main][^snake][^ecosystem]

## Library versus platform

Roc splits host effects (platform) from reusable code (package). Only a
platform can add I/O. basic-webserver already exposes HTTP, SSE unfold,
SQLite, files, and Html. Rocci generated apps and custom apps both use it.
[^dispatch][^custom-main][^bws-sse]

### Library on basic-webserver (recommended counterfactual)

A package `Rocci` imported by an ordinary app:

```roc
app [Context, program] {
    pf: platform "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/…",
    rocci: "Rocci",
}

import pf.Server
import Rocci

Context : { db : Sqlite.Db }

program = Rocci.program({
    init!,
    routes: [
        Rocci.view("/", home!),
        Rocci.fragment(Post, "/actions/counter/increment", increment!),
        Rocci.command(Post, "/actions/counter/reset", reset!),
        Rocci.live("/sse", live_slice!),
    ],
})
```

`Rocci.program` would return `{ init!, respond!, shutdown! }`. Authors never
write `Server.Outcome` on the happy path. The package owns exact-path match,
`Datastar-Request` branching, fragment SSE wrap, command empty-SSE versus
204, live poll plus keepalive, error overlays, and optional singleton
`data-init` injection by wrapping view Html at runtime.[^lower]

This is the Datastar SDK shape: host HTTP framework plus a helper that
chooses events. The difference is that **constructors, not handler bodies,
choose the wrap**, so the closed matrix can survive without a grammar.
[^datastar-adr][^datastar-sdks][^ecosystem]

### Custom platform (usually not worth it)

A `rocci-web` platform whose app header exported `{ init!, routes }` instead
of `{ init!, respond! }` would hide basic-webserver's `Server` module. That
is a packaging change, not a new authoring model. The DX would match the
library unless the platform added effects the host lacks (in-process pub/sub
that wakes live unfolds; Datastar-aware request decoding as a primitive).

Forking the platform to chase those effects is the same decision the live
research already refused: work around idle timeout with keepalives rather
than fork basic-webserver.[^bws-sse] A custom platform is justified only if
Rocci needs effects the host will not grow. It does not, by itself, improve
handler DX over a package.

### Why constructors beat a response ADT

Verb-first research rejected "return `Server.Outcome`" and "branch on `Html`
versus `Sse.Event`" as the high-level surface, because then the wire
contract disappears from the declaration.[^verb-first][^template-readme] A
library that used one generic handler plus a result tag would regress to
the Go SDK: every body chooses events.

Roc also has no macros and no Axum-style `impl IntoResponse` coherence
across a heterogeneous list. A `List Route` must be uniform. The honest
encoding is **role constructors that close over handlers**:

```roc
Mutation : [Post, Put, Patch, Delete]

view : Str, (Context, Request => Try(Html, err)) -> Route Context err
get_fragment : Str, (Context, Request => Try(Html, err)) -> Route Context err
fragment : Mutation, Str, (Context, Request => Try(Html, err)) -> Route Context err
command : Mutation, Str, (Context, Request => Try({}, err)) -> Route Context err
live : Str, (Context, Request => Try(Html, err)) -> Route Context err
```

That is the same eleven-cell matrix as `legal_pair`: GET admits view,
fragment, live; mutations admit fragment and command; GET command and POST
view are unrepresentable.[^validate][^handler-contract] The DSL enforces
the matrix in the template crate. The library enforces it in Roc types.
Both beat a single `Route : { method, path, handle! }` record, which would
make illegal pairs representable again.

Escape hatches belong on **additional constructors**, not inside the four
roles:

```roc
events : Mutation, Str, (Context, Request => Try(List Sse.Event, err)) -> Route Context err
prefix_fragment : Method, Str, (Str, Context, Request => Try(Html, err)) -> Route Context err
unfold : Str, (Context, Request => Sse.Stream) -> Route Context err
```

Those are today's authored `main.roc` patterns, named. They keep the closed
matrix as the default and flatten the cliff the ecosystem paper calls
steep.[^ecosystem][^custom-main][^snake]

## Sketch: the same apps

DSL counter (one file today):[^counter]

```rocci
@get:view("/") = |{ db }| {
    count = read_count!(db)?
    counterPage({ count })
}

@post:fragment("/actions/counter/increment") = |{ db }| {
    count = increment_count!(db)?
    counterCard({ count })
}
```

Library equivalent of the **routes only** (components still hypothetical
Roc or still `.rocci`):

```roc
home! = |{ db }, _request| {
    count = read_count!(db)?
    Ok(counter_page({ count }))
}

increment! = |{ db }, _request| {
    count = increment_count!(db)?
    Ok(counter_card({ count }))
}

routes = [
    Rocci.view("/", home!),
    Rocci.fragment(Post, "/actions/counter/increment", increment!),
    Rocci.fragment(Post, "/actions/counter/reset", reset!),
]
```

Live counter: `Rocci.command(Post, "/actions/counter/increment", increment!)`
returning `Ok({})`, plus `Rocci.live("/sse", live_slice!)`. The wrap
(empty SSE versus 204; poll loop) stays in `Rocci.program`, not in the
author body — same split the DSL uses, different registration site.
[^live-counter][^dispatch]

Gallery prefix tabs, which `.rocci` cannot spell:[^custom-main]

```roc
Rocci.prefix_fragment(Get, "/actions/tabs/", |id, _ctx, _req| {
    Ok(Tabs.patch(id))
})
```

## DX comparison

Scores are exploratory. "Better" means faster correct authoring for that
job, not a product recommendation.

| Job | `.rocci` DSL | Roc library / thin platform |
| --- | --- | --- |
| Pick wrap (HTML vs empty vs stream) | Header; illegal pair fails in `rocci-template` before Roc | Constructor; illegal pair fails as a Roc type error |
| See the contract without reading the body | Yes: `@post:command("/x")` greps | Yes if authors use constructors; no if they fall back to `match` + `Server.respond` |
| First counter (markup + two POSTs) | One file: routes, `@component`, `@css`, fixtures | Two languages unless Html is also Roc: `main.roc` plus templates, **or** verbose `Html.element` trees |
| Datastar call site | Unchanged: `data-on:click=@post("/actions/…")` | Unchanged if attributes still compile to the same strings |
| Path parameters / prefixes | Out of bound; escape to authored `main.roc` | Natural extra constructors; gallery becomes ordinary |
| Mixed SSE events | Out of bound on generated routes | Extra `events` constructor; still not the default fragment |
| Duplicate method+path | App-wide diagnostic, including sibling modules | Init-time `Dict` insert unless a preprocessor exists; **not** a Roc type error |
| Generated Roc name collisions (`/a-b` vs `/a_b`) | Language constraint | Gone: no `on_{method}_{path}!` names |
| Live `data-init` injection | Lowering when exactly one local live and no authored attribute | Runtime wrap of view Html; same policy, later in the pipeline, harder for LSP to preview |
| Command JSON accidentally patching signals | Unrepresentable on `:command` | Unrepresentable on `Rocci.command` if that constructor only accepts `{}` |
| Completions | `rocci-lsp` after `@get:` | Roc LSP on `Rocci.view` / `Mutation` tags; no second grammar |
| Error quality | Template diagnostics name the rewrite (`@on` → `@get:view`) | Roc type errors; no mechanical "you meant command" rewrite unless the package docs it |
| Escape hatch | Different product surface (`main.roc`) | Same surface, more constructors |
| Ecosystem transfer | Unique grammar; Go examples do not map | Same shape as official SDKs: import helpers, register HTTP |
| Multi-module composition | Sibling `.rocci` plus generated merge | `List.concat(counter_routes, settings_routes)` |
| Tests | Fixtures for components; HTTP tests through `rocci run` | Handler functions are ordinary Roc; dispatch tests can call `Rocci.respond!` without generating `main.roc` |
| Agent / LLM authoring | Closed headers reduce wire mistakes; second language to learn | One language; easier to emit illegal wraps unless constructors stay the only public API |

### Where the DSL is stronger

**Inspectability without types.** The verb-first paper's constraint is
real: `.rocci` parse cannot see that a body returns `Html` versus `{}`.
Putting role in the header is how a template language gets a closed matrix
at all.[^verb-first] A Roc library deletes that constraint by moving
registration into Roc, where types exist. That is not free: you give up a
surface that `rocci-lsp`, highlighters, and grammars can see **without** a
Roc typechecker. Header grep, rewrite diagnostics, and "completion after
`@get:` is `view | fragment | live`" are language features a package does
not have.[^server-ref][^ecosystem]

**One-file HTML apps.** The counter's DX is not the `:fragment` suffix. It
is colocated markup, scoped `@css`, `@fixture`, and `rocci run File.rocci`
with no app header, no `Server.file_root`, no `program` record.[^counter]
[^template-readme] Pure Roc Html is `Html.element("section", [Html.attribute("id", "counter")], …)`
plus unscoped strings. That is the gallery's `Html.roc` wrapper, not the
standalone starter.[^html-runtime] Replacing handlers with a library while
keeping `@component` is a different product than "pure Roc."

**App-wide uniqueness before runtime.** Duplicate GET `/` across modules is
a template diagnostic today.[^validate] A `List Route` cannot prove
uniqueness at compile time. You check at `init!` or you generate code
again. The library wins names (no `on_get_root!` coupling) and loses static
collision errors unless Rocci still scans source.

**Mechanical rewrites.** Removed `@on` / role-first forms become precise
diagnostics. A library has no equivalent for "you wrote the old SDK-style
`respond!`." Culture and examples have to carry that.

### Where the library is stronger

**The cliff is the same language.** Today's ceiling (prefix paths, mixed
events, custom unfolds, JSON resources) is a second product:
authored `main.roc` that reimplements dispatch.[^custom-main][^snake]
[^template-readme] A library makes those **named extensions** of the same
`routes` list. Authors who outgrow `:fragment` do not leave the helper
vocabulary. That is the usual SDK growth path, and it is how Stario /
Lambda Combine stay in Python and Common Lisp.[^ecosystem]

**Path parameters stop being a grammar issue.** Literal paths are a
classifier convenience (the parser stores a string). They are a product
bug for `/actions/tabs/{id}` and `/actions/todos/{id}`. A prefix
constructor is a function. The DSL cannot add that without becoming a
router, which the bounded-UI research refused to do in `.rocci`.
[^verb-first][^custom-main]

**Composition and tests.** Routes are values. Apps can concatenate module
lists, property-test `legal_pair` equivalents as type inhabitation, and
unit-test `increment!` without generating a server. The DSL's tests sit in
Rust (`handler_syntax.rs`) plus end-to-end `rocci run`.

**Ecosystem literacy.** Official Datastar SDKs are `PatchElements` in an
ordinary handler.[^datastar-adr] Rocci authors who learned Go will find
`Rocci.fragment` closer than `@post:fragment`. The matrix can remain
closed; it just looks like a typed SDK instead of a grammar. The ecosystem
paper's "unique as language syntax" advantage disappears — deliberately.
[^ecosystem]

**No generated-name physics.** Path spelling would no longer have to be a
valid Roc identifier after `on_{method}_{path}!`.[^validate]

### Neutral or mixed

**Illegal wraps.** Both can make GET+command unrepresentable. The DSL does
it earlier (template crate) with friendlier messages. The library does it
in the type checker the author already needs. Neither beats the other on
the matrix itself.

**Call sites.** Roles stay server-only. Buttons still say `@post("/x")`.
The "reader of the button cannot see fragment versus command" problem is
unchanged.[^ecosystem]

**Live is still a poll.** Moving dispatch into Roc does not create a
relay. Keepalives and `After(100)` stay host-honest.[^dispatch][^bws-sse]
A platform fork could add pub/sub; that is a separate, larger product.

**Representation-free commands.** A `command` constructor that returns
`{}` preserves the bug-prevention the CQRS work wanted. A sloppy library
that accepted `Html` or JSON on the same helper would reintroduce signal
patches. The DX win depends on keeping that constructor strict.
[^handler-contract][^ecosystem]

**Components stay pure.** Whether the handler is a header or a constructor,
`@component` should remain a function `props -> Html` with no I/O.
[^pure-render][^server-owned] The library must not grow
`Rocci.component` that talks to SQLite.

## Three product forks (not one)

1. **Replace `.rocci` routes with a Roc package; keep `@component` / `@css`.**
   Handlers become Roc; markup stays the template language. This is the
   only fork that can beat DSL handler DX **without** destroying starter-app
   HTML DX. Custom apps already live here, minus shared constructors.
   [^custom-main][^pure-render]
2. **Pure Roc, including Html.** Matches "Roc library/platform" literally.
   Handler DX can be excellent; page DX collapses to element trees unless
   Roc itself grows JSX-like syntax (it has not). Unsuitable as the
   default Rocci authoring path.
3. **Custom platform with `{ init!, routes }`.** Cosmetic unless new
   effects exist. Prefer a package that fills `respond!` on basic-webserver.

Fork 1 still needs a small compiler if `.rocci` files exist: lower
components, stamp CSS, maybe inject `data-init`. It no longer needs
`legal_pair` in Rust. Route diagnostics move to Roc.

## Verdict

The landed matrix is a **classifier**, not a Datastar SDK. As Roc, the
faithful design is a package of **role constructors plus one `program`
builder** on basic-webserver, with extra constructors for the current
escape hatch. A custom platform is almost the same API with a different
app header.

On **handler-only DX**, that library would likely beat the DSL as apps
grow: same closed matrix, ordinary Roc tooling, prefix routes and mixed
events without changing product surface, no generated-name collisions.
It would match how the rest of the Datastar ecosystem already works.

On **Rocci-app DX**, the DSL still wins the starter and the inspectable
header. The costly part of `@get:view` is not `:view`; it is everything
around it — HTML, CSS, fixtures, `rocci run`, rewrite diagnostics, and
collision checks that do not need a Roc typechecker.

The hybrid (components in `.rocci`, routes in Roc) is the interesting
middle. It is not shipped, not approved, and not a reason to reopen the
grammar toward `PatchElements` options. An overlay probe on
`method-role-lib` compiled `pf.Rocci` wraps with app `match`; a `List` of
constructors linked then crashed at runtime. See
[opinionated BWS overlay](/plans/rocci/opinionated-bws-host.md). If a later
plan continues, keep the four default constructors as strict as today's
roles, and put SDK completeness on named escapes — not on dissolving
`command` back into "return whatever."

[^ecosystem]: Landed matrix is compile-time classification, unique as language syntax; SDKs leave roles in handler bodies; authored `main.roc` is the steep hatch.
[^verb-first]: Template parser cannot inspect Roc types or bodies; role must be in the header for a closed matrix at parse time.
[^handler-contract]: Frozen eleven pairs, representation-free commands, empty-SSE versus 204, poll live wrap.
[^validate]: `legal_pair` admits GET view/fragment/live and mutation fragment/command; duplicate method+path fails, including generated-name collisions.
[^lower]: Singleton live path injects `data-init=@get(path, [OpenWhenHidden(True)])` when the document has no authored `data-init`.
[^dispatch]: Generated `respond!` wraps Html/{} into document, one-shot patch-elements, empty SSE or 204, and poll unfold on basic-webserver.
[^template-readme]: Public matrix, authors never write `respond!` in `.rocci` apps, mixed events belong in authored `main.roc`.
[^datastar-runtime]: `Datastar.patch_elements`, `patch_signals`, and `@post` string helpers are ordinary Roc.
[^html-runtime]: `Html.element` / `Html.attribute` wrap platform constructors; no JSX.
[^counter]: `@get:view("/")` plus `@post:fragment` increment/reset in one `.rocci` file with `@component` and `@css`.
[^live-counter]: `@post:command` returns `{}`; `@get:live("/sse")` owns shared morphs.
[^custom-main]: Prefix-matched `/actions/tabs/` GET fragments, `patch!` / `events!`, mixed patch-elements and patch-signals.
[^snake]: Custom `Sse.unfold!` with revision compare and `After(125)`.
[^server-ref]: Mandatory verb-first headers; closed table; no aliases.
[^server-owned]: Durable state stays on the server; HTML is the update boundary.
[^pure-render]: `@component` lowers to a pure Roc function returning Html.
[^datastar-adr]: SDK contract is an SSE generator used from ordinary host handlers, not a route-role grammar.
[^datastar-sdks]: Official per-language SDKs.
[^bws-sse]: Idle-timeout workarounds: keepalives and empty SSE; do not fork the platform for live.
)
