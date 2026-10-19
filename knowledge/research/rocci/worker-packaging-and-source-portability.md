---
type: Research Report
title: Rocci architecture changes for portable applications
description: Explore deeper application, resource, lifecycle, rendering and effect boundaries that could make native and Worker hosting peers without making invasive changes prerequisites for the unchanged counter.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T12:08:17Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/worker-packaging-and-source-portability.md
    title: Unchanged-source Worker implementation plan and qualification gates
  - id: audit
    resource: ../../audits/rocci/worker-packaging-and-source-portability.md
    title: Packaging audit and native-to-Worker semantic gaps
  - id: compile-output
    resource: ../../../crates/rocci-template/src/lib.rs
    title: Compile output separates generated Roc, application metadata and source maps
  - id: metadata
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: Existing route, live, init, renderer and style metadata
  - id: route-lowering
    resource: ../../../crates/rocci-template/src/lower/routes.rs
    title: Ordinary handler body lowering without native server construction
  - id: ast
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci declarations and opaque ordinary Roc regions
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Native entry generation, response policy, imports and live scheduling
  - id: platform
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Native required program contract, host exports and common HTTP package
  - id: server-abi
    resource: ../../../crates/rocci-platform/platform/InternalServer.roc
    title: Native configuration and outcomes include files, stop and process exit
  - id: sql
    resource: ../../../crates/rocci-platform/platform/Sqlite.roc
    title: Native database configuration, typed queries and transaction ownership
  - id: handles
    resource: ../../../crates/rocci-platform/platform/Host.roc
    title: Boxed database, transaction, request and other resource handles
  - id: html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Pure node-based Html and nominal document/fragment values
  - id: datastar
    resource: ../../../crates/rocci-platform/platform/Datastar.roc
    title: Action expressions and event construction over Html and Sse
  - id: sse
    resource: ../../../crates/rocci-platform/platform/Sse.roc
    title: Pure event data and boxed finite source transitions
  - id: projection
    resource: ../../../crates/rocci-lsp/src/projection_workspace.rs
    title: Current hardcoded projection platform and staged runtime modules
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: One-file starter to preserve through every experiment
  - id: live-counter
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Initialization migrations, body effects and explicit transactions
  - id: blocks
    resource: ../../../examples/rocci/standalone/blocks/backend/Blocks.rocci
    title: Effectful page and live reads advance and persist game time
  - id: snake
    resource: ../../../examples/rocci/custom/snake/main.roc
    title: Authored native program and custom effectful live source
  - id: role-research
    resource: method-role-handlers-as-roc-library.md
    title: Earlier exploratory role constructors and authored-main escape hatch
  - id: platform-postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Owning the platform did not redesign the application contract
  - id: parser-research
    resource: roc-native-template-compiler-postmortem.md
    title: Historical template-compiler POC, distinct from runtime portability
  - id: state
    resource: ../../decisions/server-owned-state.md
    title: Durable state remains server-owned
  - id: components
    resource: ../../decisions/pure-render-components.md
    title: Components remain pure functions of explicit values
---

# Rocci architecture changes for portable applications

## Finding and status

There are useful fundamental changes to consider. The strongest direction is
to make **application semantics independent of native server control**, then
give resources, transactions and live work explicit portable contracts. That
could make native and Worker hosts peers rather than requiring a Worker to
imitate every process-oriented API.

This does not require redesigning the template language first. Current
lowering already emits ordinary handler functions and useful metadata; much
of the native coupling appears later, in generated application glue and the
platform's required program contract.[^compile-output][^route-lowering][^dispatch][^platform]

These are **research notes**, following the
[Worker plan](/plans/rocci/worker-packaging-and-source-portability.md) and
[audit](/audits/rocci/worker-packaging-and-source-portability.md). The larger
API and semantic changes below are not additional implementation phases,
approved syntax or release prerequisites. No prototype was executed for this
record. Current-behavior claims were checked against checkout
`d8b609d2582a0ca8bde25669f3a8213512c8a8a3`, including its existing local
knowledge work; older research supplies rationale, not shipped status.[^plan][^audit]

The original `Counter.rocci` remains the compatibility test. A cleaner API
for new applications can coexist with its `pf.Env`, `pf.Path`, `pf.Sqlite`,
`@context`, `@init` and ordinary route declarations. Making users migrate the
starter to demonstrate portability would weaken the product goal.[^counter]

## Evidence: where the coupling actually lives

| Existing boundary | Verified behavior | Implication |
| --- | --- | --- |
| Template frontend | `CompileOutput` carries generated Roc, source-map segments, context/init, routes, live routes, styles, fixtures and tests. | There is already a seed for a shared application model. [^compile-output][^metadata] |
| Handler lowering | Emits named functions, adapts unused request parameters and wraps returned values in `Try`; ordinary Roc bodies remain opaque. | A new backend need not start with new source syntax or a second Roc parser. [^route-lowering][^ast] |
| Application generation | Constructs listen host/port, file roots, native mounts and `{ init!, respond!, shutdown! }`. | Move host control behind an explicit backend boundary. [^dispatch] |
| Platform contract | Requires `Server.Config` during initialization and a process-shaped shutdown; internal outcomes also carry file/process behavior. | A portable application contract could be smaller than the native host contract. [^platform][^server-abi] |
| Storage | `Sqlite.Config` includes a path, pool/cache limits, journal and durability choices; transactions are opaque resources. | Logical data access and native connection policy are currently combined. [^sql] |
| Rendering and events | Html is pure; Datastar constructs actions/events; Sse includes both event bytes and retained effectful source continuations. | Share value APIs without confusing them with stream ownership and scheduling. [^html][^datastar][^sse] |
| Editor | Projection staging uses a fixed basic-webserver pin and sibling Html/Datastar modules. | One resolved build profile should serve editor and build. [^projection] |

The platform post-mortem already distinguishes ownership of `pf` from a new
application API: the first cut intentionally kept authored `respond!`.
Earlier role-constructor research is therefore relevant unfinished design
space, not a missing prerequisite that was secretly implemented.[^platform-postmortem][^role-research]

## Options and invasiveness

All API/module names below are illustrative. None has been type-checked as
a proposed public API on the selected Roc compiler.

| Option | Expected benefit | Invasiveness | Disposition |
| --- | --- | --- | --- |
| R1: resolved application model and backend interface | One source inventory, route policy and source-map model for native/Worker output. | Internal refactor; preserve public syntax and emitted behavior. | Suitable refinement of the existing plan. |
| R2: portable application contract and typed role constructors | Removes native server setup from common application code; gives plain Roc the same route semantics. | Platform API, closure/type and custom-app migration work. | Research before changing `requires` or authored apps. |
| R3: named resources and narrow capabilities | Deployment maps logical storage/services to actual bindings. | Additive library/config first; replacing `pf.Sqlite` would be a larger migration. | Explore additively; keep the counter adapter. |
| R4: scoped transaction/body/stream APIs | Makes ownership align with provider callback scopes and cancellation. | Additive platform API plus nontrivial callback ABI work. | Small helper proof may fit the plan; wider ownership redesign remains research. |
| R5: separate resource, migration, activation and request lifetimes | Avoids treating process initialization as permanent actor creation. | Runtime contract; optional future declarations would affect the language. | Research lifecycle API before new syntax. |
| R6: opt-in read/pure-render live views | Enables safe shared reads, invalidation and host-owned schedules. | New execution semantics, identity model and scheduler ABI. | Research only; preserve existing effectful live behavior. |
| R7: shared pure value modules and nominal type identity | Reusable templates and one Html/event contract across platforms. | Package/type ownership and migration work. | Share source first; prove package identity before extraction. |
| R8: richer compiler capabilities or resumable effect protocol | Could improve effect diagnostics or reduce dependence on JSPI. | Roc compiler/backend or new programming-model work. | Long-term research, never an assumed packaging prerequisite. |

## R1: make the application model explicit

Build a resolved application descriptor from existing `RouteInfo`, `LiveInfo`,
`InitInfo`, generated module references, styles/assets and source maps.
Resolve entry/context ownership and route collisions once. Native and Worker
emitters consume that descriptor instead of independently discovering an app
and duplicating a growing dispatch generator.[^metadata][^dispatch]

Keep two boundaries distinct:

- **Application structure:** known route declarations, roles, handler symbols,
  init/context, assets and original source positions.
- **Roc program behavior:** ordinary function bodies, types, effect sequencing
  and closures, still compiled by Roc.

The descriptor is not an interpreter or universal intermediate language for
Roc. It cannot derive all effects or serialize arbitrary functions merely
because it knows the route names. Initially it should be an internal typed
structure, not a public stable JSON protocol or new driver crate.

Put response-role semantics in one maintained implementation or shared
generation layer: document wrapping, finite patches, empty commands, errors,
and live-source setup. Keep listener setup, native files, Worker bindings and
provider assets in host adapters. Compare a common generated Roc dispatcher
against a shared Roc runtime library before choosing where every helper lives.

**Smallest experiment:** feed unchanged Counter, a multi-module app and a
live app through one descriptor into native and Worker emitters. Compare
route inventory, source maps, action URLs, CSS and response behavior. Include
an authored `main.roc` path; do not pretend arbitrary custom routes are
discoverable from template metadata.

**Stop or narrow:** the abstraction needs a Roc expression parser, causes
unrelated output churn, loses source spans, or puts Cloudflare/Datastar runtime
policy into syntax validation. A focused emitter seam is sufficient if a
larger descriptor does not reduce duplication.

**Counter migration:** none. This is architectural work that can directly
support the current plan without changing its source-preservation promise.

## R2: a portable application contract beneath host launchers

Research a common application API which describes context construction and
request/route behavior, while host adapters describe how it runs. Native
hosting binds a listener and handles process shutdown; Worker hosting receives
requests and reconstructs activation resources. Neither requires the shared
application to manufacture the other's host configuration.

A conceptual split could be:

```text
Application: initialize resources, register typed routes, handle requests
NativeHost: listen, native file service, process signals, drain
WorkerHost: activation, provider bindings, asset service, request lifetime
```

Compare two designs rather than assuming the larger one is cleaner:

1. Keep the existing app-facing handlers and supply target-specific generated
   entry glue. This minimizes source/API migration and may already be enough.
2. Introduce a portable `Application`/route API used by both generated `.rocci`
   applications and authored Roc. Native server control becomes an extension
   around it. This can reduce the custom-`main.roc` cliff but changes a public
   platform contract.

The current native `requires` and the binary exports are not the same
contract. Platforms already wrap application functions before exporting them
to their hosts. A Worker may use different request/allocation/stream exports
without first imposing a new common source-level `program` record.[^platform]

Typed role constructors could express view, fragment, command and live routes
in ordinary Roc. `.rocci` headers could lower to the same constructors, keeping
one-file markup authoring and early illegal-pair diagnostics. Plain Roc users
could compose routes without hand-writing every response wrapper. The earlier
counterfactual explored this shape; its signatures remain design sketches.[^role-research]

Important research questions:

- Can a uniform route representation retain different handler result roles,
  context and error types without awkward erasure or unstable callback code?
- Can generated literal routes keep static duplicate diagnostics while
  dynamically composed Roc routes get honest initialization-time validation?
- Can custom file responses, mixed SSE, prefix routes, stop requests and
  native listener tuning remain clear extensions rather than a generic
  lowest-common-denominator response type?
- Can the domain platform expose this conveniently without forcing every
  beginner to assemble several framework dependencies?

Also make execution/admission a researched part of the contract. Specify
whether contexts may serve overlapping requests, what happens while an effect
is suspended, and whether callbacks can reenter an instance. The Worker
plan's initial serialized admission is a safety policy, not a transaction or
permission to reorder/retry application effects. A portable API should make
its supported concurrency profile explicit rather than imply native and
actor scheduling are identical.[^plan]

**Smallest experiment:** unchanged Counter lowers into the proposed runtime;
a plain Roc app registers equivalent routes; an advanced custom app uses one
extension. Prove closure/error typing, Wasm code size, source diagnostics and
native behavior before considering a `requires` change. Include overlapping
requests, a suspended effect, callback reentry and mixed read/write scopes
under both hosts' declared admission profiles.

**Stop or narrow:** basic apps gain `main.roc` boilerplate, role guarantees
disappear, callback compilation becomes fragile, or custom native features
need a parallel incompatible framework. Snake explicitly authors the native
program/config today, so migration costs must be assessed against that real
app, not only generated Counter.[^snake]

**Counter migration:** none through compatible lowering. New/custom API
adoption is opt-in. Removing `.rocci` route declarations is not required.

## R3: name application resources instead of native locations

A cleaner long-term resource boundary would let source request a logical
database or service while deployment selects the actual storage binding.
For example, an illustrative `Database.open!("counter")` could resolve to a
native SQLite file or the current actor's managed SQL. This identifies a
resource purpose; object/tenant routing remains an explicit application and
deployment decision.

Preserve typed parameter/row handling and useful concrete SQL access. Separate
portable resource requirements from native pool, journal, path and cache
configuration. Those are real current `Sqlite.Config` concerns; moving them
to a host profile could prevent accidental promises that Worker SQL has the
same tuning controls.[^sql]

Do not call two database backends equivalent because they implement an `open`
and `query` function. Specify supported SQL features, exact value ranges,
transaction modes, errors and durability. A portable profile can be explicit
about a subset, while native-specific code can still ask for native features.
Opening two logical databases also does not create two independent SQLite
files inside an object by implication.

For existing Counter, keep the declared path-to-managed-storage mapping from
the plan. A new named-resource API is an optional better vocabulary for future
apps, not a reason to remove the old adapter.[^counter][^plan]

Capability records are another possible seam: pass a helper the database,
clock or HTTP operations it needs rather than letting it assume a broad
platform. That could improve test substitution and reusable Roc libraries.
Prototype ordinary Roc records/modules before adding `@resource` or
`@capability` syntax.

Passing a narrow record is not automatically an enforced effect sandbox. A
helper may still reach other platform modules unless the compiler/host
actually prevents it. Type identity, effectful callbacks, escaping handles
and transitive dependencies need proof. Do not promise that injection makes
arbitrary code portable or statically pure.

**Smallest experiment:** one unchanged Counter through its compatibility
mapping, plus a second app using a named database and one helper supplied a
narrow database/clock interface. Run on native and Worker hosts; compare
types, errors, user-written code and diagnostics.

**Stop or narrow:** every helper needs duplicate adapters, error/type
annotations overwhelm the original application, generic SQL hides essential
features, or storage/tenant identity becomes implicit. Keep logical naming
in configuration if a new source API does not earn its cost.

## R4: scopes as portable ownership APIs

Native transactions have explicit begin/commit/rollback and final-reference
rollback. Host resources also include statements, executions and bodies with
lifetimes. Source portability currently asks the Worker adapter to preserve
those contracts across a different host model.[^sql][^handles]

An additive callback transaction API could make the scope explicit for new
portable code: acquire, run a bounded operation, commit on the selected success
result, otherwise roll back and release. Similar scoped APIs could manage body
readers and live subscriptions without permitting stale handles to escape.

This is more than writing a Roc helper around `begin!` and `commit!`. The
Worker platform must invoke rooted Roc callback state within the provider's
actual transaction scope, preserve values/errors and complete cleanup. A
callback which reenters Wasm also has different stack constraints from a
normal top-level JSPI call. The existing plan deliberately makes that an ABI
experiment, alongside the linear-transaction rendezvous alternative.[^plan]

Research whether a restricted transaction capability can expose only allowed
storage operations. Initially synchronous transaction work is easier to
reason about, but absence of an HTTP field alone does not prove the callback
cannot call an imported HTTP effect. Qualify enforcement and failure behavior
before describing the API as preventing asynchronous side effects.

Keep the semantic differences visible: a caught statement error may permit
recovery; an escaping error can abort the scope; a failure after a completed
commit must not undo that commit. Do not replace every native begin/commit
pair with one whole-handler transaction by textual rewriting.

**Smallest experiment:** actual Roc callback work over native and Worker
transactions, with two writes, success, caught/escaping errors, trap, dropped
values and attempted suspension. Include closure rooting, nested invocation
and cancellation tests.

**Stop or narrow:** callback support needs an unsafe reentrancy workaround,
closures escape scope, rollback is ambiguous, or restrictions cannot be
enforced. Preserve linear compatibility where proven; offer a deliberately
smaller callback profile rather than weakening native promises.

**Counter migration:** none; the starter has no explicit transaction handle.
The current live counter may opt into a small callback-scope change if the
unchanged linear API is impractical.[^counter][^live-counter]

## R5: explicit lifetimes beneath initialization

Research separating five concerns which currently can appear in one `@init`:

| Concern | Candidate contract |
| --- | --- |
| Deployment validation | Check declared bindings/capabilities without mutating application data. |
| Schema migration | Apply a versioned change within the identity and concurrency rules of the selected database. |
| Activation resource setup | Reconstruct DB/service handles and disposable caches for this activation. |
| Request/subscription scope | Own body readers, transaction state, per-user context and stream sources until completion. |
| Scheduled domain work | Explicitly advance time/business state under the target's scheduling guarantees. |

Counter initialization already uses idempotent creation/seeding. Current
live-counter initialization also performs a schema migration. These are useful
fixtures, but arbitrary initialization cannot be split automatically by
recognizing SQL strings or moving statements around.[^counter][^live-counter]

Make the durable state/resource distinction first-class in documentation and
runtime handles: database rows persist; an activation `Context` owns live
resources which must be reconstructed. An activation generation and a request
scope can prevent old handles from touching a replacement instance or a
completed body. Opaque host boxes are not serializable durable state.[^handles]

Start with library/runtime hooks and config. Keep existing `@init` as the
compatibility initializer in the appropriate instance scope. New `@activate`,
`@migrate`, `@request` or `@alarm` declarations would be language/API proposals,
not syntax to add because the provider has similarly named concepts.

Questions include per-object versus shared-database migrations, rolling code
versions, partial init failure, retry safety, native resources which need
cleanup, and whether a logical schedule continues without subscribers. A hook
does not make arbitrary initialization replay-safe or shutdown reliable.

**Smallest experiment:** reconstruct unchanged Counter repeatedly; then use
an opt-in versioned migration and separate resource setup in a second app.
Inject failure after each setup step and test recovery without duplicating
external side effects.

**Stop or narrow:** the lifecycle simply mirrors one provider's class names,
simple native apps gain redundant hooks, or stronger once-ever guarantees
cannot be stated honestly. Explicit configuration plus a small initializer
may be the better default.

## R6: live views with explicit read and execution semantics

Research an opt-in view contract separating snapshot acquisition, pure
rendering and refresh triggers. A host could then share one snapshot/render
for equivalent subscribers, wake on committed changes and refresh explicitly
for time. Authors would still write ordinary SQL and pure Rocci components;
they would not need to turn the whole domain into a reducer or JSON aggregate.

The current contract cannot be optimized that way automatically. Generated
live handlers run effectfully for each connection on a 100ms schedule. Blocks
calls `load_ticked!` from page and live handlers; that function advances gravity
and writes game state. Snake's custom live path also performs timed state
advancement. Sharing or skipping those calls can change application behavior,
even if the final HTML often matches.[^dispatch][^blocks][^snake]

An illustrative new view descriptor might identify:

- A snapshot/query function and a pure render function with explicit props.
- The actor/data identity, authorization context, locale, route/query args and
  other inputs that determine whether results may be shared.
- Commit-scoped invalidation topics or revisions, plus a declared refresh
  schedule for time and external writes.
- Full-snapshot reconnect, subscribe/snapshot ordering, bounded queues and
  cancellation behavior.

Calling a function `read!` is not proof it is read-only. A stronger read
capability, compiler support or enforced host profile is needed before sharing
execution on that basis. Missing identity inputs could leak personalized
content; ordinary “one cached render per route” is insufficient.

Notifications should be published after commit and fenced by activation/
generation. Do not infer complete dependencies by parsing arbitrary SQL.
Explicit invalidation and conservative timed refresh are honest starting
points. Writes from outside the selected runtime need a stated refresh or
notification mechanism.

The existing Sse source already represents finite host-advanced transitions,
which is a useful foundation. Its `Wake` currently offers `Immediately` and
`After`; generic notification waiting would require a runtime/ABI extension,
not merely changing the timer constant.[^sse]

Domain scheduling should be considered separately from delivery scheduling.
Moving Blocks gravity to an actor alarm might be cleaner, but continuing the
game with zero subscribers would be a behavioral choice, not an invisible
performance optimization. Native and Worker hosts must document any different
scheduling guarantees.

**Smallest experiment:** an opt-in live counter with snapshot/pure render,
two equivalent subscribers, a personalized subscriber and time-only updates.
Use Blocks and Snake as adversarial fixtures which retain compatibility
polling until explicitly adapted. Test commit visibility, missed wakeups,
replacement and slow readers.

**Stop or narrow:** execution multiplicity changes under old `@get:live`,
cache keys omit request identity, timers become hidden business logic, or the
API requires a retained server DOM/browser domain store. Pure components and
server-owned state remain established boundaries.[^components][^state]

## R7: share pure types without forcing one renderer representation

Html/Attribute values, action builders and finite event framing are candidates
for one versioned pure implementation used by native and Worker platforms.
That could reduce duplicated runtime code and help ordinary Roc applications
consume compiled templates without adopting the entire HTTP host.

Current Html is a pure node representation with nominal rendered document/
fragment values. Datastar imports Html and Sse; Sse combines pure event bytes
with boxed effectful source transitions. Extraction must preserve this
distinction rather than treating all stream code as pure formatting or all
event construction as host I/O.[^html][^datastar][^sse]

The current CLI rewrites exact runtime imports to `pf.Html`/`pf.Datastar` for
the Rocci platform. That is a concrete compatibility seam, but two copied
definitions with matching names are not proof of nominal type identity.
The platform already pins the common HTTP package explicitly to align HTTP
types; require equally concrete proof for shared Html/event types.[^dispatch][^platform]

Compare three ownership choices: shared source inside platform releases,
a pure package with compatible platform-facing re-exports, and a small common
value package with target-specific implementations around it. Keep one
convenient domain platform for normal Rocci apps. Package extraction need not
make beginners manually assemble unrelated libraries.

Type/module ownership and renderer representation are separate decisions.
Unifying imports does not justify replacing native nodes with the POC's string
renderer, changing escaping, or fusing all Html implementations for speculative
speed. Measure allocations, code size and serialization after semantic parity.

**Smallest experiment:** lower one component once, consume it in native and
Worker apps and an unrelated Roc host, and pass Html/event values through
package/platform interfaces. Test nominal identity, raw versus escaped text,
documents/fragments, multiline events, CSS and ordinary Roc helpers.

**Stop or narrow:** the selected compiler cannot preserve type identity,
crossing APIs requires render/reparse solely to adapt types, imports break
existing apps, or the dependency split costs more than source sharing.

## R8: compiler collaboration and explicit effect protocols

More accurate effect/capability metadata from Roc could improve portability
diagnostics, callback restrictions and binding generation. First collect
precise failing examples: direct, aliased, transitive and indirect effects;
unused code; effectful callbacks; and runtime-selected branches. Compare
declared requirements, compiler errors and final host imports before promising
complete static detection.

Rocci currently treats arbitrary Roc as opaque regions and does not type-check
its expressions. Teaching its parser to recognize a few `pf.*` call spellings
would not establish a sound effect system. A native-only effect is a target
capability issue, not invalid template grammar.[^ast][^route-lowering]

An explicit continuation/effect protocol is a more invasive alternative to
sequential hosted calls through JSPI. It could let the host drive finite
steps, but there are two distinct products to investigate:

- An opt-in ordinary Roc library whose authors construct resumable operations.
  This may suit some workloads but asks users to adopt another programming
  model.
- Compiler-supported transformation of ordinary sequential effectful code.
  This might preserve source, but requires compiler/backend and ownership work
  beyond Rocci's template lowering.

Neither implies that arbitrary live closures or stacks can be persisted and
resumed after eviction. Serializable workflow state, deterministic replay,
external-effect identity and versioning would each need separate contracts.
Do not infer durable continuations from an in-memory JSPI suspension.

Moving the template parser from Rust to Roc is another independent idea.
The earlier Roc-native compiler POC concerns template parsing/lowering and
documents its own compiler-era costs. Reimplementing the parser does not
provide SQL effects, request lifetime, JSPI integration or resource cleanup;
it should not be attached to this deployment goal without an independent
measured benefit.[^parser-research]

**Smallest experiment:** one upstreamable compiler/ABI or capability example
that removes demonstrable shim code or catches an otherwise missed error.
For a continuation approach, preserve a real existing effectful helper and
measure source changes, memory, code size and cancellation semantics.

**Stop or narrow:** a maintained compiler fork becomes necessary before
unchanged Counter works, ordinary handlers become per-app state machines,
or the proposal is merely a second template/Roc interpreter. Prefer the custom
platform/JSPI path while it meets the documented gates.[^plan]

## Promotion criteria and relationship to the plan

Keep the implementation plan focused on source preservation, the actual Roc
ABI and the familiar build/deploy workflow. R1 and shared editor/build profile
resolution are internal refinements compatible with those phases. A scoped
transaction helper is already a conditional compatibility option there; this
record does not approve replacing the wider SQL API.[^plan]

Promote a larger option to a separate implementation proposal only when its
experiment can answer all of the following:

| Question | Evidence needed |
| --- | --- |
| Does it remove a demonstrated portability cost? | Reduced adapter/ABI duplication, fewer source changes or a semantic guarantee impossible with the compatibility layer. |
| Does the starter still work? | Byte-unchanged Counter, same routes/HTML/SQL, plus a documented config mapping. |
| Does it help plain Roc and advanced apps? | A plain Roc example and an authored custom app, not only generated hello HTML. |
| Are the new semantics honest? | Explicit resource/transaction/lifecycle/live guarantees; failure, cancellation and reconstruction tests. |
| Is migration bounded? | Additive API first, compatibility adapter, exact source delta and a versioning/deprecation proposal if later needed. |
| Does it earn its DX cost? | Measured authored code, diagnostics, install/build/edit/deploy friction and ongoing maintenance. |
| Does it preserve the stack? | Pure components, server-owned durability, HTML transport, scoped CSS and no provider policy in parsing. |

The most promising research combination is R1 plus an additive R2/R3/R4
application/resource API, with R5/R6 made explicit only where the application
needs them. R7 can improve reuse independently. R8 should be driven by a real
compiler/host limitation. None requires removing existing one-file Rocci
authoring or making every application adopt every abstraction.

## Sources and verification

[^plan]: Paired exploratory plan; its platform and runtime gates remain unexecuted by this research task.
[^audit]: Reviewed packaging and source-compatibility boundaries, distinct from a product contract.
[^compile-output]: Current frontend result includes generated Roc and separate application/source metadata.
[^metadata]: Current lowering options and `RouteInfo`, `LiveInfo`, `InitInfo` structures.
[^route-lowering]: Handler bodies remain ordinary Roc; lowering supplies names, parameter adaptation and Try wrapping.
[^ast]: AST contract explicitly keeps ordinary Roc opaque.
[^dispatch]: Current native generator, runtime import adaptation, route collision checks and per-connection live loop.
[^platform]: Required native program versus exported host wrappers; shared HTTP package identity rationale.
[^server-abi]: Process/file outcomes and native configuration make host assumptions observable.
[^sql]: Typed query surface, native pool/path policy and explicit transaction lifetime.
[^handles]: Current opaque resource types include database, statement, execution, transaction and request resources.
[^html]: Pure Html node and rendered nominal document/fragment types.
[^datastar]: Ordinary Roc action/event construction, using Html and Sse types.
[^sse]: Pure Event framing alongside boxed effectful sources and timer-based wake vocabulary.
[^projection]: Actual projection staging currently fixes a native basic-webserver platform.
[^counter]: Exact source-preservation fixture; initialization and finite fragment handlers.
[^live-counter]: Actual mixed initialization/resource/migration logic and transactional handlers.
[^blocks]: Concrete counterexample to assuming live handlers are read-only or safely shared.
[^snake]: Authored native program and custom time-advancing live behavior.
[^role-research]: Earlier exploratory role-constructor API; sketches are not compiler-verified public APIs.
[^platform-postmortem]: First-cut ownership/DX distinction; historical mutable details must be checked against current source.
[^parser-research]: Historical template-only parser/lowerer POC; not a Worker runtime implementation.
[^state]: Existing normative server-owned durability boundary.
[^components]: Existing normative pure-component boundary.
