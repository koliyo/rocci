---
type: Implementation Plan
title: Deploy ordinary Rocci applications as Workers
description: Preserve the original Counter.rocci and progressively broader Roc handlers through a custom Worker platform, JSPI effects, generated hosting and ordinary Wrangler workflows.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T14:59:33Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: execution
    resource: ../../status/worker-source-portability.md
    title: Phase microplans, implementation evidence and remaining release gates
  - id: architecture-research
    resource: ../../research/rocci/worker-packaging-and-source-portability.md
    title: Fundamental Rocci architecture alternatives and their invasiveness
  - id: audit
    resource: ../../audits/rocci/worker-packaging-and-source-portability.md
    title: Worker packaging inventory and source-portability boundaries
  - id: celld-research
    resource: ../../research/rocci/celld-runtime-alternatives.md
    title: JSPI, celld and native-host alternatives
  - id: probe
    resource: ../../research/rocci/celld-runtime-probe-results.json
    title: Recorded local C-JSPI and independent synchronous Roc probes
  - id: prior-plan
    resource: portable-actor-backends.md
    title: Earlier pure-core actor POC scope
  - id: prior-status
    resource: ../../status/portable-actor-backends.md
    title: Dated POC results and outstanding acceptance gates
  - id: starter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Original one-file SQLite counter and finite fragments
  - id: starter-readme
    resource: ../../../examples/rocci/standalone/counter/README.md
    title: Native counter run and one-shot update semantics
  - id: original-live
    resource: https://github.com/koliyo/rocci/blob/97227bf514da4b5b08fe092db687c2d53141f77a/examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Live counter before the actor POC source extraction
  - id: live
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Current effectful live counter and explicit transactions
  - id: native-platform
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Native program contract, exports and hosted effects
  - id: native-host
    resource: ../../../crates/rocci-platform/platform/Host.roc
    title: Opaque boxed host resources and effect signatures
  - id: native-sql
    resource: ../../../crates/rocci-platform/platform/Sqlite.roc
    title: Typed queries, prepared statements and linear transaction lifetime
  - id: native-allocator
    resource: ../../../crates/rocci-platform/src/roc_alloc.rs
    title: Native resource finalizers and allocation metadata
  - id: html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Native Html node constructors and rendering contract
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Generated lifecycle, routing, finite responses and live polling
  - id: cli
    resource: ../../../crates/rocci-cli/README.md
    title: Existing commands, private actor staging and orchestration ownership
  - id: config
    resource: ../../../crates/rocci-core/src/config.rs
    title: Strict shared configuration schema and validation
  - id: lsp
    resource: ../../../crates/rocci-lsp/README.md
    title: Compiler selection and generated projection platform
  - id: handlers
    resource: ../../../docs/applications/handlers.rocdown
    title: Public document, fragment, command and live behavior
  - id: state
    resource: ../../decisions/server-owned-state.md
    title: Server owns canonical application state
  - id: pure-components
    resource: ../../decisions/pure-render-components.md
    title: Pure component contract
  - id: poc-build
    resource: ../../../examples/rocci/remote/live-counter/tools/build.mjs
    title: Wasm compiler pin, host build and artifact checks
  - id: poc-arena
    resource: ../../../examples/rocci/remote/live-counter/platform/host.c
    title: Import-free invocation arena with no retained allocations
  - id: poc-host
    resource: ../../../examples/rocci/remote/live-counter/src/index.ts
    title: Existing app-specific actor SQL and retry receipts
  - id: poc-streams
    resource: ../../../examples/rocci/remote/live-counter/src/streams.ts
    title: Bounded stream delivery and replacement fencing
  - id: root-pin
    resource: ../../../.roc-version
    title: Native repository compiler pin
  - id: wasi
    resource: ../../../crates/rocci-wasi-http/README.md
    title: Separate experimental WASI HTTP component and ABI
  - id: jspi
    resource: https://v8.dev/blog/jspi
    title: JSPI sequential Wasm integration and current standardization status
  - id: jspi-spec
    resource: https://webassembly.github.io/js-promise-integration/js-api/#jspi
    title: JSPI promising entry and JavaScript-frame restrictions
  - id: jspi-runtime
    resource: https://blog.pyodide.org/posts/jspi-with-c-runtime/
    title: C-runtime linear-memory stack hazards during overlapping suspension
  - id: asyncify
    resource: https://emscripten.org/docs/porting/asyncify.html
    title: Instrumented asynchronous Wasm and JSPI alternatives
  - id: cf-sql
    resource: https://developers.cloudflare.com/durable-objects/api/sqlite-storage-api/
    title: Synchronous SQL, async and sync callback transactions, cursor and numeric limits
  - id: workerd-transactions
    resource: https://github.com/cloudflare/workerd/blob/main/src/workerd/api/actor-state.c%2B%2B
    title: Async transaction callback, concurrency gate, commit and rollback implementation
  - id: cf-lifecycle
    resource: https://developers.cloudflare.com/durable-objects/concepts/durable-object-lifecycle/
    title: Durable Object activation and lack of reliable shutdown hooks
  - id: cf-build
    resource: https://developers.cloudflare.com/workers/wrangler/custom-builds/
    title: Build hooks for Wrangler development and deployment
  - id: cf-bundle
    resource: https://developers.cloudflare.com/workers/wrangler/bundling/
    title: Wasm modules, normal bundling and prebuilt deployment
  - id: cf-generated-config
    resource: https://developers.cloudflare.com/workers/wrangler/configuration/#generated-wrangler-configuration
    title: Framework-generated provider configuration
  - id: cf-wasm
    resource: https://developers.cloudflare.com/workers/runtime-apis/webassembly/
    title: Workers core Wasm and partial experimental WASI support
  - id: cf-websockets
    resource: https://developers.cloudflare.com/durable-objects/best-practices/websockets/
    title: Separate hibernating WebSocket transport
---

# Deploy ordinary Rocci applications as Workers

## Goal

Make the usual application the deployable application. A developer writes
`Counter.rocci`, runs it locally, adds a small deployment configuration and
deploys through the same Wrangler workflow as a TypeScript Worker. They keep
their Roc handlers, SQL, components, CSS and Datastar actions. They do not
author a Worker host, C shim, second route table, application wire protocol,
pure-core extraction or collection of packaging scripts.

The preferred implementation is a custom Roc core-Wasm platform with ordinary
synchronous host imports where possible and JSPI for genuinely asynchronous
effects. Rocci supplies generated dispatch and assets above that platform.
Cloudflare Workers is the first managed deployment adapter; celld is a later
conformance target. Plain Roc applications must be able to use the underlying
platform without adopting Rocci or Rocdown.

The first golden application is the actual one-file
`examples/rocci/standalone/counter/Counter.rocci`, with **zero edits to its
source**. Its document and two POST fragments are a smaller, more useful
compatibility proof than the existing actor rewrite. The next targets are the
pre-POC live counter and today's transactional live counter.[^starter][^starter-readme][^original-live][^live]

This plan follows the [packaging audit](/audits/rocci/worker-packaging-and-source-portability.md)
and [JSPI/celld research](/research/rocci/celld-runtime-alternatives.md).
It broadens the product ambition beyond the earlier deliberately small
pure-core POC. It does not revise that POC's historical scope or declare its
outstanding acceptance gates complete.[^audit][^celld-research][^prior-plan][^prior-status]

## Status and evidence boundary

Planning baseline: checkout `d8b609d2582a0ca8bde25669f3a8213512c8a8a3`,
2026-10-19, including existing uncommitted audit/research work. Local source
and public primary documentation were reviewed. That was the planning-only baseline. A subsequent implementation request now
has [phase microplans and measured execution evidence](/status/worker-source-portability.md).
The Worker preview implements the commands, strict configuration, compatible
platform, generated hosting and editor profile. Implementation revision
`0e1b5f311a89ba6d8c3bea52e68e5ad914c0f2c8` passed full CI and Knowledge; the
execution record stores run IDs, phase outcomes and measured limits. Release
publication and optional broader-provider/economics work remain separate.[^execution]

Existing evidence is encouraging but narrower: the recorded local probe
successfully ran 1,000 sequential **C-Wasm** timer suspensions on workerd
1.20261012.1 and celld 0.6.2. It separately ran the existing synchronous Roc
artifact. That proves neither a Roc effect ABI crossing JSPI nor hosted
Cloudflare availability, concurrent safety or leak freedom.[^probe]

V8 documents JSPI as effectively standardized at phase 4. Its presence in a
browser/V8 release is useful context; the tested provider runtime, compiler,
artifact and compatibility date remain the actual qualification boundary.
The browser displaying a Rocci app does not need JSPI: suspension occurs in
the server's Wasm host.[^jspi]

## Out of bound

- Replacing the existing private actor POC or promoting a production deployment.
- Universal emulation of an operating system, process execution, native
  dynamic libraries, arbitrary filesystem paths or every native platform
  effect on managed Workers.
- Rewriting the Roc compiler, `.rocci` grammar, Datastar or the application
  into a JavaScript client framework.
- Replacing native `rocci run`, desktop packaging, musl servers, Wasm apply
  or the experimental WASI HTTP component.
- Exactly-once arbitrary external side effects, transparent cross-object SQL
  transactions, or declaring arbitrary native applications portable merely
  because they compile to Wasm.
- Making a celld fork, VM fleet or replication subsystem a dependency of the
  first low-friction managed Worker release.

## Constraints that do not move

1. **Preserve authoring first.** Existing `.rocci` syntax and normal Roc stay
   the source language. Preserve `pf` imports for the supported compatibility
   surface; target selection belongs in build/platform staging. Any necessary
   source adjustment must be small, explicit and enumerated by fixture.
2. **Keep HTML and state ownership.** Components remain pure; SQL and Roc own
   durable state. Datastar carries intent and server-rendered HTML. Do not
   introduce browser count/feed stores to simplify the adapter.[^state][^pure-components]
3. **Preserve response roles.** Starter fragments affect the acting tab.
   Commands return empty SSE to Datastar and 204 to ordinary callers. Live
   GETs remain the shared read channel. Provider hosting must not silently
   turn every POST into a broadcast or an automatically retried command.[^handlers]
4. **Make deployment semantics explicit.** A filesystem DB path does not
   identify a globally distributed actor. Configuration selects singleton or
   keyed identity, storage mapping, refresh and retry policy. Preserve that
   identity across rebuilds and deployments.
5. **Fail precisely.** Unsupported reachable effects, lossy numeric paths and
   unqualified compiler/runtime combinations must produce actionable errors.
   No silent in-memory DB, no-op transaction, rounded I64 or success after a
   failed durability gate.
6. **Own complexity once.** Compiler/platform glue, provider hosting, browser
   transport support and conformance tests belong in maintained packages and
   CLI code. App scaffolds should stay small and inspectable.
7. **Preserve existing layers.** Parser/lowering stays in `rocci-template`;
   dispatch/build in `rocci-cli`; shared config in `rocci-core`. Rocdown is
   not needed for a Worker app. A new workspace member must be classified in
   the workspace dependency checker in the same implementation change.

## What source portability means

Track each promise separately. A source hash match alone is insufficient.

| Level | Required result | Target |
| --- | --- | --- |
| Template portability | Components, escaping, scoped CSS and actions lower normally and render equivalently. | Required for every application. |
| Handler portability | Existing `@init`, context, helpers and routes compile against the Worker platform; no pure-core rewrite. | Required for the starter release. |
| Effect portability | Supported `pf` operations preserve values, errors, ownership and sequencing, including real JSPI suspension. | Explicit capability profile. |
| Deployment portability | Only deployment/storage/identity configuration changes between hosts; no app host or route rewrite. | Cloudflare first; celld later. |
| Operational equivalence | Defined persistence, transaction, retry, refresh and reconstruction guarantees pass conformance. | Advertise only guarantees actually tested. |

Freeze three golden inputs in tests without modifying the working examples:

- **C0, starter:** current `Counter.rocci`; record bytes and hash. Preserve
  `Env.var!("DB_PATH")`, `Path`, `Sqlite.open!/execute!/query!`, `Stderr.line!`,
  `@context`, `@init`, `/` and both action URLs. It needs neither an explicit
  transaction handle nor asynchronous host effects for its SQL path.[^starter]
- **C1, original live:** snapshot the live counter and sibling dependencies at
  `97227bf514da4b5b08fe092db687c2d53141f77a`, before the actor extraction.
  Preserve its handlers and markup; do not make new `CounterCore`, `remote`
  props, actor URLs or `window.rocciCounter` calls prerequisites.[^original-live][^audit]
- **C2, transactional live:** current native live counter, including body
  reads, revision/coherent reads, explicit transaction handles and SQL feed.
  Reuse its existing decomposition if present, while compiling its effectful
  handlers rather than bypassing them with `Actor.roc`.[^live]

C1's original multi-statement mutation does not become a claimed atomic
application contract merely because the Worker serializes a turn. C2 is the
separate test for deliberately transactional behavior. Record improvements
and platform limitations independently of source compatibility.[^audit][^live]

## End-user workflow

### Recommended path: ordinary Wrangler, generated Roc hosting

The Worker preview implements this command family; release distribution is
tracked in the execution record:

```sh
# Existing application; scaffold deployment support without editing Counter.rocci.
rocci worker init examples/rocci/standalone/counter --provider cloudflare
cd examples/rocci/standalone/counter
npm install
npm run dev
npm run deploy
```

The scaffold supplies ordinary package scripts invoking `wrangler dev` and
`wrangler deploy`. Wrangler's custom build hook calls `rocci worker build`.
An experienced user can call Wrangler directly. Authentication, accounts,
domains, secrets and provider bindings retain their normal Wrangler meaning.
The documented provider build hook already runs for development and
deployment; no per-app JavaScript build script is necessary.[^cf-build]

Authored project shape should stay close to:

```text
Counter.rocci              existing application, unchanged
assets/                    optional application assets
rocci.toml                 entry and the small Roc portability profile
wrangler.jsonc             ordinary provider deployment configuration
package.json + lockfile    provider tooling and optional CLI distribution
.gitignore
```

`.rocci/worker/` contains generated Roc, Wasm, the Worker entry, staged assets,
source maps and a manifest. Developers can inspect it; they do not maintain
it. A new app can use the same scaffold with an included starter file. The
CLI should reuse existing directory/entry resolution rather than require a
second app manifest.

Implemented preview configuration:

```toml
[app]
entry = "Counter.rocci"

[worker]
provider = "cloudflare"
execution = "durable-object"
binding = "APP"
object_key = "counter"

[worker.sqlite]
path = "./counter.db"
storage = "object"
```

For C0, all visitors use the named singleton because that matches its shared
count. Treat the configured path as a logical mapping to the object's managed
database. `DB_PATH` can select a configured alias; changing a string must not
open a new filesystem DB or accidentally select a different tenant. Reject
unmapped DB paths and unsupported multiple-database/ATTACH use clearly.

Keyed apps opt into an explicit route or authenticated-identity resolver.
Do not infer tenants from arbitrary user headers. A stateless Roc handler
should deploy without a Durable Object; SQLite/shared coordination opts into
the object profile. No storage or actor binding should be required for a
plain rendered hello page.

Keep provider config authoritative for namespace/class identity, migrations,
routes, compatibility date and secrets. Rocci config owns source discovery
and the application-facing capability/storage mapping. Generate one stable
class name and initial migration when scaffolding; never recreate or rename
them on a normal build. Validate the relationship between the two configs.

The current config schema rejects unknown fields, and its existing validation
serves native/desktop needs. Worker fields and target-specific validation
require implementation; a Worker must not acquire a dummy desktop window or
local listen socket merely to satisfy native validation.[^config]

### Toolchain installation and the escape hatch

- Distribute prebuilt Rocci binaries, a checksum-pinned Roc compiler and
  prebuilt platform host objects/glue for supported development hosts. Normal
  app authors should not need Rust, Cargo, a C compiler or this repository.
- Resolve/download the selected toolchain automatically on first build with
  visible version, size, cache location and progress. Allow a supplied compiler
  path and a preseeded/offline cache. CI gets a noninteractive locked mode.
- An npm launcher can make installation familiar to Worker users; it should
  launch the same native CLI, not introduce an independent compiler path.
  Native installation remains usable without an npm runtime SDK dependency.
- Ordinary deployment should need no handwritten TypeScript. Advanced users
  can compose a published adapter or call a typed host extension, retaining
  normal Wrangler bundling and JavaScript ecosystem integration.
- Keep a full Worker entry template as an advanced escape hatch, with the same
  versioned runtime underneath. Ejecting must be deliberate and must not make
  ordinary upgrades require editing generated files.

Prefer embedding/staging the versioned runtime with the CLI for the first
release, producing a self-contained Worker module graph. A separately
published npm adapter is useful for custom Worker composition; it must share
the same ABI/version checks rather than become a second implementation.

## Architecture and ownership

```text
unchanged .rocci / ordinary .roc
        -> existing lowering + shared route policy
        -> generated Roc app against a compatible Worker platform
        -> core Wasm + small versioned JS host + assets + manifest
        -> Wrangler bundle / local workerd / managed Workers
                  -> object SQL and identity when selected
```

| Layer | Proposed responsibility | Boundary |
| --- | --- | --- |
| Roc Worker platform | Roc entry contract, allocation/resource ABI, host effects, values/errors and JSPI entry discipline. | Usable by plain Roc; no counter or Rocci parser dependency. |
| Rocci compatibility modules | Supported `pf.Server`, `Html`, `Datastar`, `Sqlite`, `Env`, `Path`, logging and time surface. | Reuse existing module source and nominal types where feasible; explicitly test deviations. |
| Rocci CLI | Source/entry resolution, route generation, compilation, assets, maps, target checks and build manifest. | Extend focused owning modules; no second public driver crate. |
| Generic JS host | Wasm invocation admission, handles, cancellation, HTTP/body mapping, tracing and bounded stream primitives. | Versioned ABI; no application state codec or SQL schema. |
| Cloudflare adapter | Provider bindings, object identity, SQL, durability/output gating, activation and migration integration. | No domain decisions or hardcoded counter keys. |
| Optional provider extensions | Access JWT, R2/KV/queues, deployment-specific auth and external service bindings. | Opt-in capability modules; not required for a counter. |

Evaluate a focused Worker platform package alongside `rocci-platform` rather
than adding V8/Node dependencies to the native Rust host. Share Roc module
sources and contract tests first; extract a standalone community repository
only after the ABI stabilizes. A plain Roc hello/fetch/SQL example must work
without `.rocci`, Datastar, a DOM or Rocdown.

The native program currently exports init/respond/SSE/drop/shutdown and hosts
many OS effects. Its target list does not provide a Worker host. Generated
dispatch also mixes native server/file initialization with response policy.
Reuse route semantics, not a fictional TCP server inside Workers.[^native-platform][^dispatch]

Prefer a shared route description and response policy with native and Worker
backends. Preserve `Html.Node`, escaping, attribute behavior, generated action
URLs, component calls and extracted CSS. The POC's string-rendering runtime
does not by itself prove compatibility with native `pf.Html`.[^html][^audit]

Do not repurpose native CPU `--target`, Wasm apply's `--host wasm`, or
`--http-module`. A separate `worker` command family makes the hosting profile
clear. Custom `main.roc` apps can adopt the Worker platform explicitly or use
an explicit build-time platform mapping; do not rewrite arbitrary dependency
pins throughout authored Roc.[^cli][^wasi]

## Custom platform, ABI and JSPI

### Retain real Roc lifetimes

Expose a small host-facing binary ABI for initialization, request execution,
finite response extraction, live-source advance/drop and final cleanup. Use
compiler-generated layout/glue where available, plus a narrow C or Rust shim
inside Wasm for ABI details. JavaScript should exchange handles, lengths,
byte ranges and explicit tagged results rather than knowing every Roc record
layout. No application-specific line/hex protocol is required.

Define ownership for every pointer, string, list, result and opaque resource;
cover small/large strings, empty values, Unicode, alignment and I64 values.
Version the compiler ABI independently from the host capability protocol.
Validate imports/exports and reject unexpected WASI/OS dependencies at build.
Recreate JavaScript typed-array views after any guest memory growth; never
retain a view into a replaced or detached buffer across a suspension.

The POC resets its entire arena per call. Native host resources are boxed and
use allocation/finalizer metadata. A real `@context`, SQL handles, retained
SSE closure or suspended request cannot be implemented safely by resetting
that arena at the next request.[^poc-arena][^native-host][^native-allocator]

Use an ARC-compatible allocator with persistent context/resource lifetimes
and bounded invocation allocations. A bump arena may remain for scratch
work only where no allocation escapes. The registry tracks resource kind,
instance/activation generation and lifecycle; closing a cursor/body/stream
is idempotent. Traps invalidate the affected instance and its handles. Rebuild
context from managed storage instead of reusing suspect memory.

### Suspend at the actual host boundary

Wrap async imports in `new WebAssembly.Suspending(hostFunction)` and invoke
exports through `WebAssembly.promising(wasmExport)`. Async host functions
await provider work, copy a checked result into valid guest-owned memory and
return the ABI result. Roc retains sequential effectful code. SQL/logging
that can finish synchronously stay synchronous.

JSPI preserves the executing Wasm computation; it neither makes synchronous
CPU work preemptible nor permits suspension through an arbitrary JavaScript
stack frame. Test every JS-to-Wasm callback crossing and ensure suspending
work has a valid promising entry.[^jspi][^jspi-spec]

Initially allow **one admitted invocation per Wasm instance**, including init,
requests, live advancement and cleanup exports. A suspended request retains
the invocation slot. No render, finalizer or stream callback may reenter that
instance behind the admission guard. JSPI's native Wasm continuation stack
does not automatically isolate the linear-memory C/Roc spill stack.[^jspi-runtime]

Stateless requests may use a bounded instance pool. An object starts with one
instance and a bounded queue. Do not lock an instance for the lifetime of an
SSE response: advance one live step, return, and release it. Queue overload
must reject predictably. Self-calls and mutually waiting actors need timeouts
and a clear error, not an indefinitely blocked queue.

For each suspension define timeout, cancellation and settlement behavior.
Abort underlying fetch/body work when supported; translate expected failures
to typed Roc errors. A `Promise.race` timeout is not cancellation: do not reset
or reuse a guest while a stale continuation can still resume. Quarantine an
unsettled instance, bound retained work, and discard late results using a
generation fence. Pin down the provider request lifetime; detached background
work is a separate opt-in feature, not implicit continuation survival.

### Capability profile

| Surface | First implementation | Qualification / limit |
| --- | --- | --- |
| `Env.var!`, `Path` construction | Explicit scalar binding/secret environment; pure path values for DB aliases/assets. | No OS environment enumeration or arbitrary filesystem implied. |
| `Stderr` / `Stdout` | Provider console/log sink with size limits and request correlation. | Never enumerate secrets into diagnostics. |
| `Html` / `Datastar` / finite `Server` outcomes | Normal compiled Roc rendering and shared generated role behavior. | Golden response and escaping parity. |
| `Sqlite` | Object-managed SQL with Roc-owned typed encoding/decoding and checked handles. | Transactions and integer fidelity are separate gates below. |
| Request body | Bounded buffering first, then owned incremental reads via JSPI. | Limits, disconnect, invalid UTF-8, already-consumed and cancellation cases. |
| Outbound `Http` | `fetch` adapter with JSPI; preserve supported HTTP package types. | Redirect/header/timeout/body semantics and binding permissions documented. |
| Time / `Sleep` | Supplied provider time and cancellable bounded timers. | No durable cron guarantee from an in-memory timer. |
| Assets / read-only file operations | Declared bundled assets or explicit storage bindings. | No arbitrary writeable local filesystem. |
| Raw TCP, `Cmd`, runtime Rocci compile/parse, native libraries | Unsupported initially; extension proposals only after demand. | Reachable use must fail clearly; no eager bundling of a runtime compiler. |

The compiler should determine reachable imported effects where possible.
Lexical searches for `Sqlite`/`Http` are hints, not a sound capability checker:
helpers and dependencies can introduce effects. Keep ordinary unused modules
from causing false rejections, while trapping undeclared dynamic host access
with a precise error. LSP and CLI consume the same profile.[^native-platform][^lsp]

## SQL and transaction compatibility

### Support ordinary SQL before changing application structure

Implement the existing parameter/row codec and error vocabulary. Object SQL
does not expose the native connection pool, but an opaque `Db` can represent
the configured managed database. Prepared statement handles can represent
validated SQL and binding metadata; they must not pretend a provider-native
prepared cursor survives activation. Test named parameters, missing/extra
bindings, duplicate columns, empty/exactly-one/many rows, blobs/nulls and
query byte/row limits.[^native-sql]

Provider numeric conversion and cursor behavior need a separate boundary.
The public SQL API exposes JS numbers and warns of precision loss. Materialize
bounded cursor rows synchronously into owned data before Roc can suspend;
do not leave a provider cursor in a handle for later async iteration.
`transactionSync` requires a synchronous callback;
raw SQL transaction statements are rejected. The asynchronous `transaction`
API also covers SQL on SQLite-backed objects.[^cf-sql]

Prefer exact I64 storage/decoding. Investigate a provider exact-value API or
a verified SQLite text/blob projection and binding codec, retaining SQL
semantics. Do not apply regex rewrites to arbitrary SQL. If a general exact
path is unavailable, publish a checked safe-integer compatibility profile,
guard both parameters and results, and fail on possible precision loss.
Never validate a rounded number and call it the original I64. An explicitly
restricted preview can ship; unrestricted native-I64 parity cannot be claimed.

Check schema/extension differences, read-only and PRAGMA behavior, transaction
modes, timeout support and DDL. Options with no meaningful provider equivalent
must be rejected or documented as a specific mapping. Do not silently accept
native pool size or busy-timeout options as if they control provider behavior.

### Try to preserve linear transactions before requiring source edits

Native `Sqlite.begin!` returns a transaction pinned to one connection, and
final-reference drop rolls it back. Commit, rollback, error recovery and
multiple sequential transaction scopes are observable behavior.[^native-sql]

Evaluate these options in order; record exact results rather than assuming
that JSPI solves callback scope:

1. **JSPI rendezvous over async provider transactions — preferred experiment
   for exact source compatibility.** A suspending `begin!` import starts a
   provider `storage.transaction(async callback)`. The callback signals that
   its scope is active and awaits a close gate; the Roc continuation resumes
   with a transaction handle. SQL stays within that provider scope. A later
   suspending commit/rollback closes the gate and waits for actual completion
   before reporting success. No JavaScript synchronous frame is held across
   suspension. Current workerd implements async callback execution with a
   concurrency gate and rollback on failure, making this credible to test,
   not proven correct for a cross-callback Roc continuation.[^workerd-transactions]
2. **Portable callback transaction helper — preferred fallback.** Expose one
   helper on native and Worker platforms which invokes Roc transaction work
   within a provider-owned callback. Keep the body synchronous initially;
   reject asynchronous effects in that body. Test an async callback variant
   only if its request/transaction lifetime is fully qualified. This may
   require replacing the few begin/commit boundaries, while retaining SQL,
   business logic, handlers and markup.
3. **Explicit synchronous handler transaction profile.** Wrap a declared
   handler or generated SQL phase in `transactionSync`. Logical handles can
   serve a narrowly defined subset only if boundaries and failure behavior
   match it. Moving commit to handler exit changes semantics when later code
   fails; catching an inner error or performing multiple transactions is also
   observable. Do not advertise this as universal begin/commit compatibility.
4. **Native/provider extension.** If full linear semantics require a host
   extension unavailable on managed Workers, retain native servers and make
   that wider profile a distinct target. Do not quietly switch deployment
   products underneath a Worker app.

The rendezvous proof must cover request/IoContext association, input gating,
SQL membership across microtasks, actual commit timing, repeated transactions,
rollback when an error escapes and drops the scope, recovery from caught
statement errors, traps, callback rejection, final-reference drop,
cancellation, timeouts, read-only transactions and unsupported nesting. An ARC
finalizer cannot simply await a Promise: mark the handle closed immediately,
signal rollback without reentry, then drain cleanup before admitting the next
invocation or acknowledge failure. Reject escaping live transaction handles
until persistent ownership and closure are deliberately supported.

Also test the simpler async callback enclosing a separate promising Roc
transaction entry. It can support an explicitly selected transaction helper
or whole-handler transaction profile without retaining JavaScript frames;
it still does not reproduce an arbitrary mid-handler begin/commit boundary.
A Roc `Err` is a value, not a rejected JavaScript Promise: the bridge must
explicitly translate an aborted transaction scope into rollback. A caught
statement error need not abort a recoverable transaction, and failure after a
completed commit must preserve the already committed rows.

Awaiting outbound work while a transaction gate is held can stall an object
or deadlock a callback into itself. Even if mechanically possible, default to
bounded storage-only transaction bodies; document the rule and test failure
before any external effect. No external side effect becomes rollbackable.

If exact linear compatibility fails, retain C0 as the unchanged-source release
target and publish the smallest portable callback change for C2. Do not force
all users into the POC's JSON aggregate state machine to solve this boundary.

## Lifecycle, delivery and durable behavior

### Application context and identity

Run `@init` once per activation/instance, before admitting requests. Rebuild
`Sqlite.Db` and other handles; retain rows in provider storage. The starter's
`CREATE IF NOT EXISTS` and `INSERT OR IGNORE` already provide a useful
repeat-activation test.[^starter]

An actor does not have the native process's lifetime or reliable shutdown
callback. Explicitly classify context as reconstructible in-memory resources,
not durable application state. Make shutdown best-effort cleanup only; durable
writes must complete on the normal operation path.[^cf-lifecycle]

Separate repeatable initialization from versioned data migrations. Provide a
small native/Worker-compatible migration helper when needed; preserve the
initial counter unchanged. Track schema version in managed storage, fence
incompatible code/data combinations and test upgrade/rollback. DB rollback
must not be assumed safe after an irreversible schema change.

### Finite responses and live streams

Implement C0 as normal generated document and finite fragment responses. Keep
the existing actions and stable fragment IDs. The browser should not import a
counter-specific remote adapter.[^starter][^handlers]

For `@get:live`, preserve the declared path and subscription rules. The native
generator currently advances per connection on a 100ms schedule, with periodic
keepalives. A Worker compatibility mode must cover time-driven views and
external writes before replacing polling with command notifications.[^dispatch]

Start with bounded scheduled advancement and the native SSE/source contract.
Do not keep a single promising Wasm call asleep forever per subscriber.
After parity, offer coalesced event-driven invalidation plus declared timed
refresh. Track dirty generations and snapshot/subscription ordering; take a
fresh render on reconnect. Reconnect-from-snapshot is sufficient unless a
separate ordered event-replay contract is selected.

Bound subscriber count, queued bytes and work per refresh. Drop/close a slow
subscriber predictably and release its retained guest source exactly once.
Test aborted reads, replaced activations, late timers, Wasm traps, stale owner
generations and no-op views. Reuse the POC's generic stream tests and fencing
lessons without importing its counter schema.[^poc-streams]

SSE and active polling have residency/cost consequences. A hibernating
WebSocket adapter is a later explicit transport option: it must still deliver
server-rendered HTML and preserve public semantics. Packaging SSE does not
give it WebSocket hibernation.[^cf-websockets][^cf-lifecycle]

### Retries, acknowledgements and security policy

Default C0 behavior remains one submitted mutation per user action with no
new transparent retry promise. Optional SDK transport can attach operation
IDs and persist receipts atomically with SQL mutation, then publish effects
after commit. Scope IDs to actor/principal and request digest, bound receipt
retention and reject an ID reused with different input. Keep any client state
to request metadata and ephemeral UX; the count/feed remain server-owned.

Durable confirmation precedes response success or stream publication. Test
response loss both before and after commit. Do not retry arbitrary HTTP/RPC
side effects as though receipt storage made them exactly once. The POC's
receipt/state schema is an example, not a general application protocol.[^poc-host]

Provider credentials and runtime secrets stay outside artifacts. Same-origin
actions, CSRF/auth policy and optional Access integration belong in maintained
runtime/configuration. Do not impose the private POC's owner email, hostname
or two counter keys on community apps. Test selected policy on every deployed
entry and asset/stream path, using the same HTTP origin as the browser.

## Packaging, diagnostics and day-to-day DX

Build into a temporary directory, validate the module graph/manifest, then
atomically replace successful output. A failed compile preserves the previous
working tree and served application. Stage Datastar, CSS and assets with the
same source discovery and pinning logic as native apps; no copied
`actor_inputs.rs` lists or application-specific staging code.[^cli][^poc-build]

Wrangler can bundle the generated entry and separate Wasm module. Use its
normal pipeline for the default workflow. Evaluate its documented generated
configuration redirect if a framework-generated effective config is needed;
do not require both that mechanism and an app-local `configure.mjs`.[^cf-bundle][^cf-generated-config]

The build manifest records source/assets hashes, frontend version, compiler
release/hash, platform ABI, runtime adapter, optimization mode, dependency
lock, effective capability profile and provider compatibility date. Keep
compiler/layout pins explicit: the current native and POC builds use different
nightlies, and the POC records a root-pin Wasm trap. Unification is desirable
only after both suites pass on one candidate.[^root-pin][^poc-build]

Offer two release paths through the same build implementation:

- Normal `wrangler deploy`: build, validate and deploy with useful defaults.
- Verified artifact release: build/package once, test that exact graph, deploy
  without rebuilding/rebundling, verify hashes and record deployment/version
  identity. This is shared SDK/CI policy, not five scripts per application.

Expose compilation phases, cache hits, unsupported effects, Wasm size and
selected runtime profile in build output. Map compile/type errors to original
`.rocci` positions; attach original request/route identifiers to runtime traps.
Logs should say which capability and source call failed, rather than only
show an unresolved `hosted_*` symbol.

The editor must select the same platform/compiler/capability profile as the
build. Current LSP projection documentation uses a native basic-webserver
stub; Worker-aware projections and restart/cache invalidation are real work.
Keep hover/tokens available without a compiler or network.[^lsp]

Avoid watch loops over `.rocci/worker`, logs and provider local state. Hot
rebuilds retain local SQL state; offer a deliberate reset command. Show build
failures in the existing terminal/preview style and preserve the last working
app. Local auth fixtures must remain outside production entries.

Measure cold install, cold/warm build, edit-to-visible time, deploy time,
generated app file count and user-authored non-Roc lines. Compare an equivalent
TypeScript Worker on the same machine/runtime. Initial product targets are
zero handwritten host/build code, one dev/deploy command after scaffolding,
no native toolchain beyond managed downloads, and no-op warm builds under one
second on the reference machine. Report compiler-bound rebuild measurements
before setting broader latency promises.

## Alternatives and decision rules

| Option | Source / DX result | Disposition |
| --- | --- | --- |
| Custom Roc platform + sync imports + JSPI | Best route to existing handlers and sequential effects with ordinary managed Worker deployment. | Preferred, gated by real Roc ABI, ownership and hosted tests. |
| Synchronous supported-effect subset | Unchanged C0 is possible without JSPI in its SQL path; narrower reach. | Useful first milestone and honest fallback if hosted JSPI is unavailable. |
| Existing pure-core `Actor.roc` model | Shares domain/render code, but requires orchestration extraction and an app adapter. | Keep as opt-in functional architecture and historical POC; not the default portability story. [^audit] |
| Binaryen Asyncify | Could preserve sequential calls on hosts without JSPI, with transformed Wasm and extra runtime/toolchain work. | Bounded fallback experiment only if the preferred host gate fails; test imports, size, stack safety and semantics before adopting. |
| Compiler CPS/effect continuation export | Could make suspension explicit and portable across engines. | Longer-term compiler collaboration; do not require a compiler fork for the first release. |
| D1 or remote SQL via JSPI | Useful provider binding for stateless/shared remote DB workloads. | Separate storage profile; does not reproduce object SQL transaction/identity behavior automatically. |
| SQLite inside Wasm over a storage layer | Potentially wider SQL/number semantics. | Requires a real durable VFS/commit protocol and ownership fencing; no automatic copying of a DB blob after each request. |
| WASI HTTP component | Useful independent portability path, with a separate host ABI. | Retain existing experiment; current partial Workers WASI support is not evidence it hosts that component unchanged. [^wasi][^cf-wasm] |
| celld Worker adapter | Potential reuse of core Wasm and most host code, with operator-owned infrastructure. | Later same-suite target, not an assumption of identical configuration/guarantees. [^celld-research] |
| Native Roc cell engine / replication integration | Can preserve more native SQL/OS behavior. | Separate upstream/fork and durability project; no managed Cloudflare Worker compatibility claim. [^celld-research] |
| Containers, Kata/VMs or unikernels | Native source compatibility may be higher; deployment and durable local SQLite need another design. | Keep a documented native escape hatch, not a hidden implementation of the Worker command. [^celld-research] |

Do not implement every alternative. Revisit one only when a measured blocking
gate or clear community requirement justifies its cost. Share fixtures,
capability descriptions and package tooling across paths where useful.

For build integration, prefer the installed CLI/custom build hook first and
the npm launcher as a distribution convenience. A Vite plugin can follow for
applications already using a JavaScript asset pipeline; it is unnecessary
for a one-file server-rendered counter. Thin `rocci worker dev/deploy`
wrappers can follow if they remove measured friction while preserving
Wrangler's configuration and authentication semantics. Asyncify would be a
maintainer-owned build profile, not application-authored async machinery;
its instrumentation and indirect-call coverage require qualification against
Roc closures and boxed callbacks.[^asyncify]

## Research notes: fundamental Rocci changes

The paired [architecture research](/research/rocci/worker-packaging-and-source-portability.md)
examines whether a cleaner application/runtime boundary could reduce the
compatibility machinery. It compares a resolved application model, portable
application/role API, logical resources and capabilities, scoped ownership,
explicit lifetimes, opt-in live-view semantics, common pure types and richer
compiler/effect support.[^architecture-research]

A shared internal application descriptor and editor/build profile are
compatible refinements of this plan. Changing the public platform `requires`,
storage API, lifecycle declarations or execution semantics needs separate
prototype evidence and an additive migration proposal. These research options
are not added phases or prerequisites for unchanged Counter deployment.
In particular, existing effectful live handlers must not be assumed read-only
or automatically safe to share across subscribers.[^architecture-research]

## Phased implementation

The phases below define acceptance. Execution and microplans are tracked in
the [status record](/status/worker-source-portability.md). Each exit requires
stored reproducible evidence; compiling a demo or seeing a button increment is not sufficient.

### Phase 0 — Freeze source promises and select the build contract

**Bound:** fixtures, API/ownership design and qualification matrix; no public
compatibility promise or deployment change.

- Freeze C0/C1/C2 with hashes and dependencies; list every supported and
  unsupported effect, route behavior and source adjustment.
- Select the Worker command family, config ownership, stable generated class
  identity and logical DB mapping. Check native flag and schema compatibility.
- Specify ABI ownership/admission/cancellation, transaction experiments and
  capability manifest. Choose a candidate compiler/platform pin without
  globally upgrading the native toolchain.
- Pin primary runtime/source versions used for experiments; distinguish
  local workerd, hosted Cloudflare and celld evidence.

**Exit:** reviewable compatibility table, exact golden inputs, proposed
command/config examples and a minimal experiment checklist. No original app
source has been changed to make the tests easier.

### Phase 1 — Prove a real Roc hosted effect through JSPI

**Bound:** tiny ordinary Roc app and reusable Wasm shim, initially local
workerd; then a bounded hosted probe before claiming managed support.

- Compile Roc calling a synchronous host effect and a genuinely async timer
  or body/fetch effect through `Suspending`/`promising`. Test success and typed
  failure, Unicode/bytes, nested effect calls and effect-after-suspension.
- Prove persistent context, retained strings/closures and opaque resources;
  include ARC finalizers, repeated calls, traps and reconstruction.
- Run closure/boxed-callback, multiple-export, hosted-import and allocation
  fixtures in both development and intended production optimization modes;
  a working development backend is not optimized-backend qualification.
- Enforce single admission; test attempted reentry, cancellation, delayed
  settlement, queue limits and generation fencing.
- Repeat at least 10,000 suspensions plus long cancellation/retention runs.
  Measure guest memory and whole-isolate/continuation memory, not only a
  linear stack pointer. Compare warm memory after quiescence and look for
  growth with invocation count.
- Probe hosted API availability and actual Roc execution with the same
  artifact/pins. Keep any necessary flags explicit and retest on upgrades.

**Exit:** real Roc suspension and stable bounded ownership demonstrated on
local and hosted targets, or a recorded no-go with a declared synchronous-only
preview/alternative. A repeated C-only JSPI probe does not satisfy this exit.

### Phase 2 — Run unchanged C0 with managed object SQL

**Bound:** starter counter, singleton identity, finite responses and a supported
checked SQL profile. Can develop in parallel with Phase 1; general async
support remains gated on Phase 1.

- Implement `Env`, path aliasing, logging, SQL codecs/resources and supported
  `Server`/`Html`/`Datastar` compatibility modules.
- Compile the exact starter source using ordinary lowering and generated
  Worker dispatch; serve staged Datastar/CSS/assets on its actual origin.
- Compare native/Worker documents and patches for count 0/1/reset, escaping,
  scoped CSS, headers, errors, routes and non-Datastar requests.
- Test two callers, restart/reactivation, init failure, SQL failure, invalid
  aliases, resource exhaustion and unsafe numeric values. Confirm other tabs
  do not acquire live updates from the starter's finite fragment route.

**Exit:** C0 hash unchanged; original SQL/handlers execute inside actual
workerd and managed Workers; state survives reconstruction; no `Actor.roc`,
custom app JS host, remote UI props or application codec is required.

### Phase 3 — Preserve transaction boundaries and asynchronous handlers

**Bound:** real SQL transaction compatibility experiment and reusable body/
outbound HTTP effects; no automatic retry or external-effect guarantees.

- Test async transaction rendezvous with the actual Roc continuation and
  provider runtime, including begin/drop/commit/rollback semantics.
- Test rollback after two writes, typed error recovery, trap, timeout,
  cancellation, repeated scopes and post-commit failure. Confirm visibility
  from a second request and no published partial state.
- If rendezvous fails, implement the shared callback helper and publish the
  exact minimal C2 source delta. Qualify any declared handler transaction
  profile separately; never hide its weaker semantics.
- Add bounded body reads, fetch response/error decoding, timers and request
  lifetime tests. Deny unsupported effects inside transaction scopes before
  starting them.

**Exit:** explicit transaction support level with failure/recovery evidence;
C2 SQL orchestration runs unchanged where proven, otherwise the documented
small portable adaptation is tested on both native and Worker platforms.

### Phase 4 — Restore the original live authoring model

**Bound:** C1/C2 documents, command/live roles and bounded SSE; no new client
domain model or mandatory retry envelope.

- Compile actual native live handlers and helpers. Preserve normal URLs,
  subscription behavior, CSS and server HTML; remove the need for a separately
  authored Worker route/renderer adapter.
- Test two browsers, concurrent commands, fresh snapshots, commit-before-
  publish, reconnect, clock-only view changes and supported external writes.
- Test backpressure, stalled readers, stream cancellation, instance traps,
  replacement activations and zero-subscriber cleanup.
- Compare polling compatibility against optional notification/timed refresh
  modes; measure idle CPU/residency and per-object memory/queue bounds.

**Exit:** reproducible two-browser live parity and cleanup/recovery evidence;
all source differences from C1/C2 are listed with reasons and no prototype
transport changes are presented as inherent Rocci requirements.

### Phase 5 — Package the platform as a routine Worker workflow

**Bound:** reusable build/scaffold/runtime distribution and local development;
no new production promotion workflow.

- Ship the Worker commands, strict shared config profile and generated
  Wrangler integration, with atomic output replacement and cache keys.
- Replace hardcoded staging lists and app scripts with generic compilation,
  dependency/asset discovery, one build implementation and generated entry.
- Package host objects/compiler/runtime together with integrity/version
  checks. Test first use from a fresh directory without Rust/Cargo/clang.
- Prove `npm run dev`, direct Wrangler dev/deploy and warm rebuilds from the
  published toolchain, including Windows/path-with-spaces qualification or an
  explicit documented supported-host list.
- Verify no watch loops, stale Wasm graph, fixture auth or secret embedding.

**Exit:** a second unrelated app deploys by source plus configuration, with
zero handwritten host/glue/codec/build scripts; install/build/dev/deploy
measurements and supported-host limitations are published.

### Phase 6 — Make errors, extensions and release verification usable

**Bound:** editor/build parity, ordinary Roc adoption and shared release tests.

- Select the same Worker compiler/platform in LSP projections and CLI; map
  unsupported effects, compile errors and runtime failures to source.
- Demonstrate plain Roc stateless HTTP, async fetch and stateful SQL using
  the underlying platform without Rocci frontend/runtime dependencies.
- Document a typed extension API for optional JS bindings and a custom Worker
  entry; keep provider-specific types outside shared Rocci application code
  unless explicitly imported.
- Publish shared conformance tooling and one verified-artifact CI path;
  normal applications supply fixtures rather than copy the POC's test tree.
- Update owning crate READMEs, CLI/config/runtime compatibility references and
  a short starter-to-deploy guide. Mark limitations beside the capability.
  Include `docs/reference/{cli,configuration,runtime}.rocdown`,
  `docs/applications/handlers.rocdown` and the runtime troubleshooting guide.

**Exit:** editor/build agreement, plain Roc reuse, source-level diagnostics,
fresh-user walkthrough and tested artifact deployment all pass. Run required
hosted CI and Knowledge on the actual revision before logging phase completion.

### Phase 7 — Qualify broader portability and economics

**Bound:** optional follow-on after the managed Worker workflow works.

- Run the same platform/app/conformance suite on a pinned celld release;
  generate its supported config and test actual deployment/recovery before
  claiming provider portability.
- Qualify binding extensions based on demand; compare sync object SQL against
  D1/remote database profiles without conflating their semantics.
- Evaluate multiplexed/stateless instance pools or isolated per-invocation
  stacks only if measured queue latency requires them.
- Evaluate hibernating HTML-over-WebSocket transport only if idle SSE costs
  justify the browser transport integration.
- Propose upstream compiler/platform improvements with minimal regressions
  and a migration path, preserving the current supported release meanwhile.

**Exit:** each added capability/provider has independent measured evidence;
unimplemented alternatives stay explicitly exploratory.

## Validation and release gates

Keep ownership tests small and deterministic. Generated route/config/codec
checks do not require a server. Real Roc/runtime integration is a separate
opt-in lane with pinned toolchain inputs; never replace it with mocks.

| Gate | Required evidence |
| --- | --- |
| Source preservation | Frozen C0/C1/C2 hashes, generated source and explicit source-delta report. |
| ABI and ownership | Actual Wasm in workerd; ARC/resources, retained context, suspension/error/trap/cancel and bounded memory. |
| SQL | Typed values/limits, exact or checked numeric behavior, aliases, query errors and no cursor crossing suspension. |
| Transactions | Multi-write rollback, real commit timing, dropped handles, failed requests and cross-request visibility. |
| HTTP/Datastar | Native parity for document/finite fragment/empty command/live response behavior at the browser's origin. |
| Persistence | Activation/restart/deployment reconstruction, migration compatibility and response-loss outcomes. |
| Delivery | Two browsers, snapshot ordering, refresh sources, slow readers, cancellation and generation fencing. |
| Packaging | Fresh install, no application host scripts, atomic output, source maps, locked dependencies and exact artifact graph. |
| Provider | Local runtime plus hosted tests; compatibility-date/compiler/runtime upgrades rerun the suite. |
| Community DX | Plain Roc reuse, second independent app and fresh-user walkthrough with measured friction. |

During implementation, run focused owning-crate tests first. Rust changes
require `cargo fmt --all -- --check`; cross-crate handoff requires the offline
`cargo test --workspace` suite. Keep the hosted/Roc lane explicit, including
`ROCCI_REQUIRE_ROC=1` where required. Public docs/layout changes require a docs
build and inspection; canonical knowledge edits require base-profile
validation with lifecycle/provenance warnings reported separately.

The publication promise is therefore concrete: **ordinary Roc/Rocci source,
small explicit deployment choices, generated platform machinery, and a
standard Worker development/deployment loop**. The first delivery is unchanged
C0; broader source compatibility expands through the effect, transaction and
live gates instead of requiring each application to repeat the prototype.

## Sources and verification

[^execution]: Dated implementation execution; keeps hosted/local results, source deltas, optional follow-ons and unpublished release gates distinct.

[^architecture-research]: Exploratory deeper Rocci changes, with experiments, migration costs and stop criteria; no contract redesign has been approved or implemented.

[^audit]: Reviewed file-level inventory and implementation boundaries; recommendations remain exploratory.
[^celld-research]: Pinned celld alternatives and qualification concerns; not a deployed adapter.
[^probe]: Stored local runtime inputs, outputs, hashes and explicit limits of C-JSPI versus Roc evidence.
[^prior-plan]: Earlier selected pure-core POC; unchanged effectful-handler hosting was outside that scope.
[^prior-status]: Dated POC execution evidence; outstanding hosted/CI gates remain separate.
[^starter]: Exact original starter application, including SQL initialization, helpers, actions and components.
[^starter-readme]: Counter changes only the acting tab and documents native/WASI run paths.
[^original-live]: Immutable repository snapshot before the actor extraction, selected as the original live fixture.
[^live]: Current native handler implementation, body reads, revision/feed SQL and transaction scopes.
[^native-platform]: Current native program/host contract and target list; no Worker host implementation.
[^native-host]: Boxed opaque effect resources and concrete host effect signatures.
[^native-sql]: Roc SQL parameter/row decoding, query limits and transaction ownership contract.
[^native-allocator]: Native ARC allocation metadata and resource finalizer behavior.
[^html]: Node-based Html representation and escaping/rendering helpers to preserve.
[^dispatch]: Native init/server setup, response role policy and generated live advancement.
[^cli]: Existing CLI boundaries, private actor staging and distinct native/WASI paths.
[^config]: Current strict configuration structures; proposed Worker fields do not exist yet.
[^lsp]: Current compiler selection and projection platform; Worker profile selection is planned.
[^handlers]: Public finite fragment, command, document and live stream behavior.
[^state]: Canonical durable state stays on the server.
[^pure-components]: Components are pure render functions.
[^poc-build]: Current experimental build pins a different compiler and documents a root-pin Wasm trap.
[^poc-arena]: Existing import-free arena explicitly permits no allocation to survive invocation.
[^poc-host]: Application-specific SQL/receipt host; not a generic handler platform.
[^poc-streams]: Existing bounded live-delivery runtime and activation fencing lessons.
[^root-pin]: Repository native compiler pin at planning time.
[^wasi]: Experimental WASI component uses a distinct older HTTP ABI and separate command path.
[^jspi]: Primary V8 description of sequential Wasm suspension and current JSPI availability.
[^jspi-spec]: Normative promising entry and suspension restrictions at JavaScript frames.
[^jspi-runtime]: Runtime implementation account of Wasm continuation versus linear-memory stack ownership.
[^asyncify]: Primary toolchain documentation on instrumented async calls and indirect-call analysis; not a tested Roc build profile.
[^cf-sql]: Provider-documented transaction APIs, cursor lifetime and numeric limits; exact native adaptation remains unproven.
[^workerd-transactions]: Current async transaction implementation motivates a rendezvous experiment; not proof that the proposed Roc bridge works.
[^cf-lifecycle]: Provider lifecycle and shutdown differences requiring reconstructible context.
[^cf-build]: Existing Wrangler build/watch hooks; proposed Rocci commands still require implementation.
[^cf-bundle]: Existing separate Wasm modules, normal bundling and no-bundle release options.
[^cf-generated-config]: Existing framework-generated effective config mechanism; an integration option.
[^cf-wasm]: Provider core-Wasm support does not establish compatibility with the experimental WASI HTTP component.
[^cf-websockets]: Provider hibernation is a separate WebSocket transport capability.
