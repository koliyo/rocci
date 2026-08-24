---
type: Implementation Plan
title: Implement path-addressed live routes
description: "Replace the fixed singleton @live with plural @get:live(path) routes, bind streams from every generated module, and keep subscriptions explicit whenever a page has more than one possible stream."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience, concern/performance]
status: draft
generated: { by: process:cursor, at: 2026-08-22T10:20:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/path-addressed-live-streams.md
    title: Research on path-addressed live streams
    author: process:cursor
    last_modified: 2026-08-22
  - id: verb-first-research
    resource: ../../research/rocci/verb-first-handler-declarations.md
    title: Verb-first handler declaration research
    author: process:cursor
    last_modified: 2026-08-22
  - id: verb-first-plan
    resource: ../verb-first-handler-declarations.md
    title: Verb-first handler implementation plan
    author: process:cursor
    last_modified: 2026-08-22
  - id: cqrs-research
    resource: ../../research/rocci/datastar-cqrs-action-responses.md
    title: Generated CQRS stream research
    author: process:cursor
    last_modified: 2026-08-21
  - id: bws-research
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: basic-webserver SSE and HTTP limits
    author: process:cursor
    last_modified: 2026-08-21
  - id: template-ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Current singleton live AST specification
    author: process:git
    last_modified: 2026-08-22
  - id: template-sidecar
    resource: ../../../crates/rocci-template/Rocci.AST.toml
    title: Rocci AST generation and inspect sidecar
    author: process:git
    last_modified: 2026-08-22
  - id: template-parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Current pathless live parser and recovery
    author: process:git
    last_modified: 2026-08-22
  - id: template-validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Current singleton live and /sse validation
    author: process:git
    last_modified: 2026-08-22
  - id: template-lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Current LiveInfo, live lowering, and fixed injection
    author: process:git
    last_modified: 2026-08-22
  - id: template-pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: Current live formatting and inspection
    author: process:git
    last_modified: 2026-08-22
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Current generated singleton live dispatch and route merge
    author: process:git
    last_modified: 2026-08-22
  - id: driver
    resource: ../../../crates/rocci-cli/src/driver.rs
    title: Current primary and sibling standalone app assembly
    author: process:git
    last_modified: 2026-08-22
  - id: run
    resource: ../../../crates/rocci-cli/src/run.rs
    title: Current sibling module discovery and compile planning
    author: process:git
    last_modified: 2026-08-22
  - id: compile-tests
    resource: ../../../crates/rocci-template/tests/compile.rs
    title: Current live lowering, injection, and conflict tests
    author: process:git
    last_modified: 2026-08-22
  - id: handler-syntax
    resource: ../../../crates/rocci-template/tests/handler_syntax.rs
    title: Current handler syntax and inspect tests
    author: process:git
    last_modified: 2026-08-22
  - id: all-syntax
    resource: ../../../test/AllSyntax.rocci
    title: Comprehensive Rocci syntax fixture
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Owning crate live contract
    author: process:git
    last_modified: 2026-08-22
  - id: server-reference
    resource: ../../../docs/reference/language/server.rocdown
    title: Public server declaration reference
    author: process:git
    last_modified: 2026-08-22
  - id: runtime-reference
    resource: ../../../docs/reference/runtime.rocdown
    title: Public generated runtime reference
    author: process:git
    last_modified: 2026-08-22
  - id: live-tutorial
    resource: ../../../docs/tutorials/commands-and-live.rocdown
    title: Public command and live tutorial
    author: process:git
    last_modified: 2026-08-22
  - id: live-concept
    resource: ../../../docs/concepts/one-shot-versus-live.rocdown
    title: Public one-shot versus live concept
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Current singleton live counter
    author: process:git
    last_modified: 2026-08-22
  - id: live-counter-ui
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounterUi.rocci
    title: Current multi-fragment live render
    author: process:git
    last_modified: 2026-08-22
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar GET lifecycle and response handling
    author: organization:star-federation
  - id: datastar-attributes
    resource: https://data-star.dev/reference/attributes
    title: Datastar data-init lifecycle
    author: organization:star-federation
  - id: datastar-sse
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event reference
    author: organization:star-federation
---

# Implement path-addressed live routes

## Purpose and authority

This plan replaces Rocci's fixed, effectively app-singleton `@live` stream with
plural, path-addressed live GET routes. It implements the companion research's
conclusion that complex generated apps need explicit stream identity and
app-level binding, while keeping one coarse page/coherence stream as the normal
topology.[^research]

The plan amends the exploratory
[verb-first handler plan](verb-first-handler-declarations.md): add `live` to
the legal GET roles and replace its pathless `@live` exception with
`@get:live(path)`. If both plans are approved, their public syntax, parser,
AST, lowering, tooling, examples, and documentation changes land as one clean
cut.[^verb-first-research][^verb-first-plan]

This plan is exploratory. Writing it does not approve the feature, start a
phase, or change shipped behavior. The current fixed `/sse` contract remains
descriptive until implementation and release gates pass.

## Current disposition

Phase 0 was jointly approved by the maintainer on 2026-08-22 with the
verb-first handler contract. The approved direction is plural
`@get:live("literal-path")`, one coarse page/coherence stream by default,
module-local singleton injection, explicit subscription whenever a module has
multiple streams, binding from primary and sibling modules, deterministic
app-wide collision errors, unchanged per-connection polling/keepalive
semantics, and no aliases or temporary `@live(path)` syntax. Phases 0–6 are
implemented with the verb-first cutover on `verb-first-handler-declarations`.
Phase 7 documentation and release gates are in this revision and must not be
marked complete until CI and Knowledge succeed on the landed commit.

## Goal

A generated multi-page app can declare and subscribe to several streams:

```rocci
@get:view("/dashboard") = |state| {
    dashboardPage(...)
}

@get:live("/streams/dashboard") = |state, request| {
    dashboardLive(...)
}

@get:view("/admin") = |state| {
    adminPage(...)
}

@get:live("/streams/admin") = |state, request| {
    authorizeAdmin!(request)?
    adminLive(...)
}

@get:live("/streams/notifications") = |state, request| {
    notificationsLive(...)
}
```

with these stable rules:

| Contract | Required behavior |
| --- | --- |
| Syntax | `@get:live("literal-path")` |
| Method | GET only; mutation `:live` pairs are rejected |
| Cardinality | Many per module and many across the generated app |
| Body | Effectful Roc returning stable-ID `Html` |
| Invocation | Generated `handler!(context, request)` on every poll transition |
| Response | Long-lived SSE with patch-elements on changed HTML and keepalive when unchanged |
| Identity | Authored path appears in inspect, logs, diagnostics, proxy policy, and collisions |
| Subscription | Datastar `@get(path)` from `data-init` or another explicit action |
| Automatic injection | Only when the declaring module has exactly one live route and body has no authored `data-init` |
| Multi-module | Primary and sibling streams are all bound to generated dispatch |

The former singleton migrates mechanically:

```text
@live  ->  @get:live("/sse")
```

## Out of bound

- One SSE connection per `@component` or automatic stream generation from DOM
  IDs
- `@post:live`, `@put:live`, `@patch:live`, or `@delete:live`
- Bare `@live`, `@live(path)`, pathless `@get:live`, or compatibility aliases
- Automatic association of streams to views by path prefix, filename, route
  order, or matching names
- Per-stream poll intervals, debounce, retry policy, keepalive cadence, event
  type, patch mode, selector, replay, or resumability options
- Platform pub/sub, mailboxes, write-triggered wakeups, or replacing the
  generated polling model
- A retained server DOM or per-tab process
- Durable domain state in Datastar signals
- Automatic authentication or authorization policy
- Cross-origin cookie/CORS hosting changes
- General response ADTs or authored arbitrary SSE from high-level handlers
- Reworking sibling-module context/init ownership beyond what stream binding
  strictly requires
- Starting implementation merely because this plan exists

Custom `main.roc` remains the ceiling for custom cadence, mixed events,
pub/sub, replay, or advanced lifecycle. Path-addressed generated streams retain
the current poll-and-render model.[^cqrs-research][^bws-research]

## Constraints that do not move

1. **Streams are GET routes.** Method, role, and literal path are explicit in
   source and normalized metadata.[^research]
2. **One stream may update multiple IDs.** The design does not require a
   connection per region; live `Html` may contain several stable-ID roots.
   [^datastar-sse][^live-counter-ui]
3. **Server state remains authoritative.** Every stream rereads and renders a
   coherent server-owned boundary.[^cqrs-research]
4. **Transport policy stays below grammar.** Parser/validation records the
   GET-live contract; `rocci-cli` owns polling, SSE, keepalive, and response
   framing.[^template-lower][^dispatch]
5. **Bodies remain opaque Roc.** The parser does not infer stream paths,
   subscriptions, authorization, IDs, or result shape from body text.
   [^template-parser]
6. **Explicit subscriptions win.** Lowering never replaces or merges an
   authored `data-init`.
7. **Ambiguous injection is disabled.** Multiple local live routes never cause
   an arbitrary stream to be injected.
8. **All modules participate.** A valid sibling live declaration cannot be
   silently ignored by app assembly.
9. **Collisions are errors.** Distinct declarations cannot be resolved by file
   discovery order or first-wins merging.
10. **Live output does not race direct patches.** A stable ID has one update
    owner unless the application supplies explicit versioning.
11. **Parser recovery advances.** Every malformed path/role/header case keeps
    monotonic cursor progress and preserves later declarations.
12. **No mixed syntax release.** Current `@live` and proposed
    `@get:live(path)` are never both accepted.

## Accepted and rejected contract

Accepted examples:

```text
@get:live("/sse") { ... }
@get:live("/streams/dashboard") = |state| { ... }
@get:live("/streams/notifications") = |state, request| { ... }
```

Rejected examples:

```text
@live
@live("/sse")
@get:live
@post:live("/sse")
@get:live(pathVariable)
@get:stream("/sse")
@get:live[interval: 500]("/sse")
```

The old pathless form receives one exact removal diagnostic recommending
`@get:live("/sse")`; it does not lower.

## Proposed AST and metadata

Within the verb-first route family, `LiveDecl` remains a typed semantic variant
but gains a path:

```text
RouteDecl = ViewDecl | FragmentDecl | CommandDecl | LiveDecl

LiveDecl =
    leading
    '@' method:'get' ':' 'live'
    path:StringLit
    params:RocExpr?
    body:RocExpr
```

The exact ungram representation may use an `Ident` method plus validation if
that is how the verb-first family is generated, but `LiveDecl` must remain a
real typed node rather than an unrelated view or generic string role.
[^template-ungram][^template-sidecar]

Normalized metadata is plural and route-like:

```text
LiveInfo {
    method: "GET",
    path,
    fn_name,
    span,
}

LoweredModule {
    lives: List LiveInfo,
    ...
}
```

The generated function name derives from GET plus the path using the route
naming helper. Add validation for different paths that normalize to the same
Roc identifier; do not fall back to one fixed `live!` binding.

## Injection policy

Lowering can determine only module-local declarations, so keep automatic
association deliberately local:

| Local live count | Root body has authored `data-init` | Result |
| --- | --- | --- |
| 0 | no | no injection |
| 1 | no | inject that route's path with `OpenWhenHidden(True)` |
| any | yes | preserve authored attribute exactly |
| 2+ | no | no injection; author subscribes explicitly |

Do not infer that a sibling stream belongs to a page in another module. Do not
inject the first stream by declaration order. Inspection or a non-fatal lint
may explain why automatic injection is disabled, but absence of a subscription
is legal because some routes may be subscribed conditionally.

Datastar may re-run `data-init` when a patched element or attribute is
initialized again. Documentation and examples place subscriptions on an
unpatched body or shell so a stream does not reconnect itself on every morph.
[^datastar-attributes]

Generated singleton injection retains `OpenWhenHidden(True)`. Explicit
subscriptions must spell whether background tabs stay connected; Datastar's
default GET lifecycle closes hidden-page requests and reopens them when visible.
[^datastar-actions]

## App-level binding and collision model

Replace route-only module binding with route plus live binding:

```text
DispatchSource {
    type_name,
    routes,
    lives,
}
```

App planning collects lives from the primary and every discovered sibling.
Binding preserves the owner module so dispatch calls the correct generated
function. Primary state/init ownership remains unchanged: all generated route
and live handlers receive the app context as today.[^run][^driver]

Before generating Roc, validate the full app route table. Reject duplicate
method/path pairs across all module routes and lives, including live versus
view/fragment collisions. Do not silently discard later declarations as the
current route merge can do.[^dispatch]

Collision diagnostics should identify both owning source files and declaration
spans where the app planner has both maps. If the shared diagnostics type cannot
yet carry a second source frame, fail with a deterministic file/method/path
message and add richer cross-file diagnostics as a bounded follow-up.

## Phase 0 — Approve topology and measure the baseline

**Status:** Approved 2026-08-22 as part of the joint verb-first/live gate.

The approved contract fixture covers one page/one stream, plural streams,
explicit subscriptions, one stream returning several stable-ID fragments, and
the linear cost model. On macOS Apple Silicon with the pinned Roc nightly and
Datastar 1.0.2, two concurrent connections to the current generated `/sse`
route were observed for 0.55 seconds. Each connection received its own initial
multi-ID patch and five idle keepalives from the existing 100 ms render loop.
Thus two subscriptions mean two responses and two independent poll/render
loops; two streams in each of two tabs mean four. This is local baseline
evidence, not a universal latency or capacity claim. Authorization remains an
authored check inside each path-specific live body.

**Bound**

- No production grammar, metadata, dispatch, example, or public-doc changes.
- Coordinate this gate with Phase 0 of the verb-first handler plan.
- Use current generated live-counter and the actual local host for measurements.

**Work**

1. Freeze the accepted/rejected syntax above and the clean migration from
   `@live` to `@get:live("/sse")`.
2. Approve `live` as a legal GET response role rather than a special top-level
   header.
3. Prepare complete source sketches for:
   - one page with one stream and automatic injection;
   - two page modules with one stream each;
   - one page with dashboard and notification streams;
   - one stream returning several stable-ID fragments.
4. Confirm the injection table, especially no implicit selection when local
   multiplicity is greater than one.
5. Measure current live-counter connection count, poll/render calls, emitted
   events, and idle keepalives for one and two browser tabs. Record machine and
   method; do not turn local timings into universal performance claims.
6. Create an expected-cost model for two streams on one page: each adds one
   response and one poll/render loop. Use the result to set an example budget,
   not a hard product limit.[^bws-research]
7. Confirm that path-specific authorization remains authored in the live body.
8. Record maintainer approval or stop without modifying shipped behavior.

**Exit**

- The maintainer approves `@get:live(path)`, multiplicity, the injection table,
  and the no-alias clean cut.
- Complete examples show why one stream can patch several regions and when a
  second stream is justified.
- Baseline measurements make stream multiplication cost visible.
- The verb-first and path-addressed plans describe one compatible final
  handler matrix.

## Phase 1 — Extend the AST, parser, validation, and recovery

**Bound**

- Owner: `crates/rocci-template`.
- Implement with the verb-first route grammar; do not ship an interim
  `@live(path)` spelling.
- Do not change generated dispatch in this phase.

**Work**

1. Add `LiveDecl` as the GET-live typed variant in `Rocci.AST.ungram`; include
   literal path, optional params, body, leading comments, and spans.
   [^template-ungram]
2. Update `Rocci.AST.toml` inspect/generation mappings and regenerate owned AST
   code; never hand-edit generated files.[^template-sidecar]
3. Parse `@get:live("path")` through the common verb-first route header and
   then the existing handler params/body scanner.
4. Remove the pathless `try_parse_live` special case and fixed-live parser.
5. Replace duplicate-live-per-module validation with duplicate GET/path and
   generated-name validation. Allow many distinct live paths.
   [^template-validate]
6. Validate literal non-empty paths according to the same route rules as other
   handlers. Reject mutation-live pairs separately from structural parsing.
7. Preserve handler arity, context requirements, docs attachment, and opaque
   body behavior.
8. Add diagnostics/recovery for pathless live, old `@live`, `@live(path)`,
   mutation live, unknown `stream` role, dynamic path, missing colon, missing
   body, and unclosed path/params/body.
9. Add monotonic-progress inputs with malformed live routes before valid
   routes, components, CSS, Roc, and EOF.
10. Update focused AST, parser, validator, and removal-diagnostic tests.
    [^handler-syntax][^compile-tests]

**Exit**

- Many distinct GET-live declarations parse into typed nodes with exact paths
  and spans.
- Illegal pairs and old forms fail with actionable diagnostics and preserve
  later declarations.
- Distinct-path generated-name collisions are diagnosed before Roc compilation.
- Ungram generation/check and full `rocci-template` tests pass.

## Phase 2 — Lower plural streams and deterministic subscriptions

**Bound**

- Owner: `crates/rocci-template` lowering, formatter, inspect metadata, and
  source maps.
- Do not yet bind sibling streams into generated main.

**Work**

1. Change `LiveInfo` to include method, path, function name, and span; change
   module output from optional live to a collection.[^template-lower]
2. Lower each live route to a unique generated function with ordinary
   `(context, request)` adaptation and `?` handling.
3. Reuse route-name normalization and add focused snapshots for root-like,
   nested, hyphenated, and colliding paths.
4. Implement the module-local injection table:
   - sole local stream injects its authored path;
   - multiple local streams inject none;
   - authored `data-init` always wins.
5. Preserve `OpenWhenHidden(True)` only on generated singleton injection.
6. Format live headers canonically as `@get:live("path")` and print path in
   AST/handler inspection.[^template-pprint]
7. Update source-map segments for method, role, path, generated function, and
   injected path scaffolding.
8. Replace fixed `/sse` lowering tests with singleton custom path, explicit
   data-init, and multiple-stream no-injection cases.[^compile-tests]

**Exit**

- Lowered metadata contains every stream with stable owner-local function name.
- Singleton custom path injection and multiple-stream no-injection are proven.
- Formatter is idempotent and inspection reports each authored path.
- Generated Roc and source maps pass focused and package tests.

## Phase 3 — Bind all module streams and generate route-aware SSE arms

**Bound**

- Owners: `crates/rocci-cli` app planning and dispatch.
- Preserve the current poll interval, changed-byte comparison, keepalive,
  error overlay, and shutdown behavior.
- Do not redesign primary context/init ownership.

**Work**

1. Extend `DispatchSource` and `GenericModule` flow to carry plural live
   metadata from primary and siblings.[^driver]
2. Replace the singleton `live: Option<&LiveInfo>` dispatcher input with a
   bound collection of `(module, live)` pairs.
3. Merge ordinary routes and live routes into one app-level method/path
   collision table. Return an error on duplicates instead of silently keeping
   the first declaration.[^dispatch]
4. Generate one SSE match arm per live path, call its owner module/function,
   and use the authored path in logs and error overlays.
5. Keep each connection's previous-render state independent inside its own
   `Sse.unfold!`.
6. Preserve keepalive output on unchanged polls and patch-elements on changed
   HTML.[^bws-research]
7. Include stream paths in listed routes, slash-redirect decisions, static
   mount policy where applicable, inspect snapshots, and handler logs.
8. Add primary+sibling tests proving both streams dispatch and a sibling live
   is never ignored.
9. Add cross-module collision tests for live/live, live/view, and
   live/fragment paths, with deterministic file/module reporting.
10. Ensure no fixed `/sse` branch remains except examples that authored that
    literal path.

**Exit**

- Primary and sibling live routes all appear once in generated main.
- Each path invokes the correct owner module and logs the authored path.
- Cross-module duplicates fail planning rather than depending on discovery
  order.
- Existing poll, patch, keepalive, error, and context behavior is unchanged per
  connection.
- `cargo test -p rocci-cli` and focused multi-module plan tests pass.

## Phase 4 — Complete LSP and syntax tooling

**Bound**

- No runtime behavior change.
- Coordinate with verb-first method-role completion rather than adding a
  separate live-only completion path.

**Work**

1. Offer `live` after `@get:` and never after mutation methods.
2. Complete a quoted path and display the long-lived Html contract in hover.
3. Emit symbols that include GET, path, and live role.
4. Highlight declaration method/role separately from Datastar `@get()` action
   expressions inside attributes.
5. Surface old `@live` removal diagnostics and multiple-stream injection
   guidance without treating missing subscriptions as a syntax error.
6. Update `test/AllSyntax.rocci` with singleton and multiple live paths and
   regenerate/review the supported AST fixture.[^all-syntax]
7. Add malformed-live recovery fixtures to the LSP invariant suite.

**Exit**

- Completion proposes only legal live syntax.
- Symbols, hover, semantic tokens, diagnostics, and spans agree with parser
  inspection.
- `inspect --ast` and `cargo test -p rocci-lsp` pass.

## Phase 5 — Add multi-page and multi-region examples

**Bound**

- Examples prove generated routing; do not introduce custom main, client
  stores, or durable signals.
- Keep the beginner live-counter at one stream.
- Do not demonstrate one connection per component.

**Work**

1. Convert live-counter from `@live` to `@get:live("/sse")`; preserve its
   singleton automatic injection and two-ID `LiveSlice` result.
   [^live-counter][^live-counter-ui]
2. Convert handler-matrix and Rocdown live island fixtures to explicit paths.
3. Add a generated multi-page example directory with:
   - primary app/context/init module;
   - dashboard page plus `/streams/dashboard`;
   - admin page plus `/streams/admin` and request authorization fixture;
   - shared `/streams/notifications` subscribed explicitly where used.
4. Put each page's subscription on an unpatched shell. A page with two streams
   uses two explicit `data-init` attributes on separate stable elements.
5. Include one stream that returns multiple stable-ID fragments to teach the
   coarse coherence boundary.
6. Add HTTP smoke tests that open each path, assert `text/event-stream`, observe
   first patch and keepalive, and prove an unknown/unauthorized stream does not
   leak another page's HTML.
7. Add browser checks for navigation cleanup, page-specific updates, shared
   notifications, two simultaneous streams, and hidden-tab behavior.
8. Measure connection count and server poll/render calls in the one-stream and
   two-stream pages; record observations in example docs or plan evidence.

**Exit**

- Live-counter behavior is unchanged except explicit source syntax.
- Multi-page generated dispatch serves all primary and sibling streams.
- Each page subscribes only to intended paths and receives only authorized
  HTML boundaries.
- One stream demonstrably patches several IDs; the example does not imply a
  stream per component.
- HTTP and browser tests prove lifecycle and isolation.

## Phase 6 — Update public contract and migration guidance

**Bound**

- Public docs change only after parser, dispatch, real Roc builds, HTTP smoke,
  and browser gates are green on the branch.
- Historical records may retain pathless syntax when labeled historical.

**Work**

1. Update the owning crate README and public server reference with
   `@get:live(path)`, multiplicity, result type, collision rules, and removal
   diagnostic.[^template-readme][^server-reference]
2. Update runtime reference with app-level stream binding, authored paths,
   per-connection polling, keepalive, and no fixed route assumption.
   [^runtime-reference]
3. Rewrite the command/live tutorial using explicit `/sse` migration while
   keeping one-stream beginner ergonomics.[^live-tutorial]
4. Extend one-shot-versus-live guidance with one page/coherence stream as the
   default and measured costs of splitting.[^live-concept]
5. Document the injection table, explicit multi-stream subscriptions,
   `OpenWhenHidden`, and the stable unpatched subscription-shell rule.
   [^datastar-actions][^datastar-attributes]
6. Update verb-first research/plan, language/stack/author skills, inventory,
   coverage, glossary, search queries, examples catalog, and app documentation.
7. Explain that path visibility helps authorization and operations but does not
   implement either.
8. Inventory active `@live`, hardcoded generated `/sse`, `Option<LiveInfo>`,
   fixed `live!`, and primary-only live assumptions. Classify each remaining
   occurrence as migrated, historical, removal test, or defect.
9. Build docs and inspect the server, runtime, tutorial, concept, and example
   pages.

**Exit**

- Active docs teach `@get:live(path)` and never claim one fixed generated
  `/sse` route.
- Beginner docs retain a simple one-stream path; advanced docs explain plural
  paths without promoting per-component streams.
- Every old form has one exact rewrite and no compatibility claim.
- Skills and canonical research agree on app-level stream ownership and custom
  `main.roc` ceilings.

## Phase 7 — Integrated validation and release gate

**Bound**

- Use temporary build/output directories.
- Failed builds preserve previous valid output.
- Do not claim completion before green CI and Knowledge workflows on the exact
  landed revision.

**Work**

1. Language, generated AST, and inspection:

   ```sh
   cargo run -q -p rocci-ungram -- check
   cargo test -p rocci-template
   cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
   ```

2. Dispatcher, tooling, protocol, and consumers:

   ```sh
   cargo test -p rocci-cli
   cargo test -p rocci-lsp
   cargo test -p rocci-datastar
   cargo test -p rocci-rocdown -p rocci-rocdown-cli
   ```

3. Formatting and workspace integration:

   ```sh
   cargo fmt --all -- --check
   cargo test --workspace
   ```

4. Build live-counter, handler matrix, multi-page stream example, and Rocdown
   live island through the pinned Roc compiler. Use `ROCCI_REQUIRE_ROC=1` where
   existing tests support that gate.
5. Run the complete HTTP matrix for each live path: route listing, content
   type, first event, changed event, keepalive, disconnect, unauthorized
   response, and duplicate-route failure.
6. Run browser lifecycle tests: page navigation, hidden/visible tab,
   one-stream multi-ID patch, two simultaneous streams, and two-client fan-out.
7. Compare measured one-stream and two-stream poll/render/connection counts
   with Phase 0; investigate unexpected superlinear work.
8. Build and inspect docs:

   ```sh
   cargo run -q -p rocci-rocdown-cli -- build docs
   ```

9. Validate knowledge and report errors separately from lifecycle/provenance
   warnings:

   ```sh
   cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
   ```

10. Run final repository inventories for pathless `@live`, fixed dispatch
    `/sse`, optional singleton metadata, fixed `live!`, and ignored sibling
    streams.
11. After landing, require green CI and Knowledge workflows and record their
    run IDs before changing phase status.

**Exit**

- Every live path parses, formats, inspects, lowers, compiles through Roc,
  appears once in generated dispatch, and serves expected SSE behavior.
- Singleton automatic injection and multi-stream explicit subscription are
  proven in generated source and browsers.
- Primary and sibling streams share the intended app context and no stream is
  ignored.
- Duplicate app paths and generated-name collisions fail deterministically.
- Performance evidence supports the documented coarse-stream recommendation.
- Focused, consumer, workspace, docs, OKF, CI, and Knowledge gates pass.

## Expected ownership

| Change | Primary owner |
| --- | --- |
| Verb-first live grammar, AST, parser, validation, lowering, injection, diagnostics, source maps | `crates/rocci-template` |
| Plural module binding, app-level collision validation, generated SSE arms, logs | `crates/rocci-cli` |
| Rocdown live island reuse | `crates/rocci-rocdown` and `crates/rocci-rocdown-cli` |
| Completion, symbols, hover, semantic tokens | `crates/rocci-lsp` and `crates/rocci-rocdown-lsp` |
| Multi-page and live-counter examples | `examples/rocci` and `examples/rocdown` |
| Public language, runtime, tutorial, and concept contract | `crates/rocci-template/README.md` and `docs/` |
| Rationale, measurements, phase evidence, lifecycle | `knowledge/` |

## Sequencing with verb-first handlers

Do not implement a temporary `@live(path)` language. Approve the two Phase 0
contracts together, then implement `@get:live(path)` as part of the same
verb-first route family. The path-stream phases can be tracked separately for
review, but the released parser has one orientation.

If the verb-first proposal is rejected, stop this plan and reopen only the
surface spelling. The underlying requirements—plural path metadata, app-level
binding, collision validation, injection policy, and coarse stream guidance—
remain valid, but the syntax would need a new explicit decision.

## Roll-forward and rollback

Before release, rollback the entire live-route cutover rather than retaining a
parser that accepts both pathless and path-addressed forms. After release,
roll forward on explicit GET-live paths. Do not restore a global implicit
`/sse`, silently select the first stream, or hide sibling streams to repair an
isolated consumer.

If app-level stream binding exposes incompatible sibling state assumptions,
keep the route rejected with a clear compile/plan diagnostic and research
shared context declarations separately. Do not expand this plan into a second
initialization model.

## Final approval gate

Implementation may start only after a maintainer approves:

- `@get:live("literal-path")` as the only live spelling;
- many live routes per module and app;
- one page/coherence stream as the recommended default;
- singleton-local automatic injection and explicit multi-stream subscription;
- app-level binding and collision errors across primary and siblings;
- unchanged polling/keepalive semantics per stream;
- no compatibility aliases or interim `@live(path)`; and
- the custom-main ceiling for advanced stream lifecycle.

Approval authorizes a bounded generated multi-stream capability, not a generic
SSE DSL or a stream-per-component architecture.[^research]

[^research]: Companion report establishes the need for authored paths plus multiplicity, compares alternatives, and recommends GET-live route syntax.
[^verb-first-research]: Existing proposal makes method and response role explicit but originally treats singleton live as an exception.
[^verb-first-plan]: Existing phased clean cut must be amended so live joins the GET role matrix.
[^cqrs-research]: Current generated live uses polling CQRS; custom main remains the advanced stream ceiling.
[^bws-research]: Each connection polls, needs keepalives, and uses browser HTTP/1.1 in local preview.
[^template-ungram]: Current AST specifies a separate pathless `LiveDecl`.
[^template-sidecar]: Generated AST/inspect mappings must change with the semantic shape.
[^template-parser]: Current parser has a dedicated pathless live branch and opaque body scanner.
[^template-validate]: Current validation rejects duplicate live declarations and reserves fixed `/sse`.
[^template-lower]: Current lowering emits one `live!`, one span-only `LiveInfo`, and fixed `/sse` injection.
[^template-pprint]: Current inspection hardcodes live GET `/sse`.
[^dispatch]: Current dispatcher accepts one optional live, emits one hardcoded arm, and silently first-wins duplicate merged routes.
[^driver]: Current app plan passes only primary live metadata while merging sibling ordinary routes.
[^run]: Current standalone planner discovers primary and sibling `.rocci` modules.
[^compile-tests]: Current integration tests cover singleton live, duplicate rejection, fixed injection, authored data-init, and `/sse` conflict.
[^handler-syntax]: Current syntax tests expose fixed live inspection and accepted header shape.
[^all-syntax]: Comprehensive fixture must show the final accepted route family.
[^template-readme]: Owning public contract currently documents pathless singleton live.
[^server-reference]: Public language matrix currently fixes live to `/sse` and one per module.
[^runtime-reference]: Public runtime currently assumes one generated `/sse` route.
[^live-tutorial]: Beginner flow relies on automatic injection and must retain its simplicity with an explicit path.
[^live-concept]: Public guidance already recommends live only when shared updates justify its cost.
[^live-counter]: Current canonical generated live example uses one pathless declaration.
[^live-counter-ui]: Current live result contains two stable-ID fragments, proving one stream can update several regions.
[^datastar-actions]: Datastar GET lifecycle closes hidden-page requests by default unless configured otherwise.
[^datastar-attributes]: `data-init` may run again when patched elements or attributes are initialized.
[^datastar-sse]: Patch-elements can morph one or more top-level elements by ID.
