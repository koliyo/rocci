---
type: Research Report
title: Portable Rocci backends on Durable Objects and actor platforms
description: "Compare Cloudflare Workers, celld, native actor hosts, and durable execution for Rocci, separating portable application contracts from provider execution and storage semantics."
tags: [domain/rocci, domain/runtime, concern/architecture, concern/packaging, concern/persistence]
status: draft
generated: { by: process:cursor, at: 2026-10-19T07:53:39Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: root
    resource: ../../../README.md
    title: Rocci product and deployment overview
  - id: state-decision
    resource: ../../decisions/server-owned-state.md
    title: Server-owned state decision
  - id: render-decision
    resource: ../../decisions/pure-render-components.md
    title: Pure render components decision
  - id: platform
    resource: ../../../crates/rocci-platform/README.md
    title: Current native Rocci platform and shipped target archives
  - id: abi
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Native application lifecycle and hosted effects
  - id: native-host
    resource: ../../../crates/rocci-platform/src/http_server.rs
    title: Server-lifetime context and concurrent request handling
  - id: executor
    resource: ../../../crates/rocci-platform/src/roc_executor.rs
    title: Bounded native worker executor
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Generated live polling and response roles
  - id: runtime-doc
    resource: ../../../docs/reference/runtime.rocdown
    title: Published runtime and HTTP contract
  - id: sse
    resource: ../../../crates/rocci-platform/platform/Sse.roc
    title: Finite SSE transitions, retained sources, IDs, and host responsibilities
  - id: sqlite
    resource: ../../../crates/rocci-platform/platform/Sqlite.roc
    title: Native SQLite pools and pinned transactions
  - id: wasi
    resource: ../../../crates/rocci-wasi-http/README.md
    title: Shipped experimental WASI HTTP adapter
  - id: wasi-component
    resource: ../../../crates/rocci-wasi-http-component/src/lib.rs
    title: Component instance adapter, body buffering, and SSE scheduling
  - id: http-module
    resource: ../../../crates/rocci-cli/src/http_module.rs
    title: Roc object and WASI component build path
  - id: apply
    resource: ../../../crates/rocci-roc-host/README.md
    title: Current combined Wasm and native applicator path
  - id: wasi-history
    resource: ../../status/wasi-http-module.md
    title: Historical August WASI HTTP PR snapshot
  - id: convergence
    resource: https://blog.cloudflare.com/deno-joins-cloudflare/
    title: October 9 announcement of Deno joining Cloudflare and celld-workerd convergence
  - id: cf-storage
    resource: https://developers.cloudflare.com/durable-objects/api/sqlite-storage-api/
    title: Durable Object SQL and transaction APIs
  - id: cf-rules
    resource: https://developers.cloudflare.com/durable-objects/best-practices/rules-of-durable-objects/
    title: Input/output gates and non-storage async interleaving
  - id: cf-state
    resource: https://developers.cloudflare.com/durable-objects/api/state/
    title: Initialization barriers and hibernating WebSockets
  - id: cf-lifecycle
    resource: https://developers.cloudflare.com/durable-objects/concepts/durable-object-lifecycle/
    title: Activation, eviction, and hibernation conditions
  - id: cf-limits
    resource: https://developers.cloudflare.com/durable-objects/platform/limits/
    title: Durable Object storage, execution, and throughput limits
  - id: cf-workers-limits
    resource: https://developers.cloudflare.com/workers/platform/limits/
    title: Worker memory, bundle, and startup limits
  - id: cf-wasm
    resource: https://developers.cloudflare.com/workers/runtime-apis/webassembly/
    title: Core WebAssembly and experimental partial WASI support
  - id: cf-rust
    resource: https://github.com/cloudflare/workers-rs
    title: Rust Workers bindings and SQL Durable Object examples
  - id: cf-streams
    resource: https://developers.cloudflare.com/durable-objects/examples/readable-stream/
    title: Streaming responses through Workers and Durable Objects
  - id: cf-alarms
    resource: https://developers.cloudflare.com/durable-objects/api/alarms/
    title: Single alarm, at-least-once invocation, and bounded retries
  - id: cf-location
    resource: https://developers.cloudflare.com/durable-objects/reference/data-location/
    title: Object placement, location hints, and jurisdictions
  - id: cf-pricing
    resource: https://developers.cloudflare.com/durable-objects/platform/pricing/
    title: Duration allocation, request, and SQLite billing dimensions
  - id: cf-containers
    resource: https://developers.cloudflare.com/containers/concepts/architecture/
    title: Native containers, ephemeral disks, and filesystem snapshots
  - id: celld
    resource: https://github.com/denoland/celld
    title: Self-hosted distributed Workers and Durable Objects
  - id: celld-limitations
    resource: https://celld.dev/docs/limitations/
    title: Beta maturity, fleet, networking, and platform limitations
  - id: celld-objects
    resource: https://celld.dev/docs/services/durable-objects/
    title: Object identity, synchronous turns, durable SQL, and lifecycle
  - id: celld-guarantees
    resource: https://celld.dev/docs/guarantees/
    title: Bucket conditional writes, ownership epochs, and fleet durability proofs
  - id: celld-compat
    resource: https://celld.dev/docs/cloudflare-compat/
    title: API, deployment, retry, and stream compatibility differences
  - id: celld-wasm
    resource: https://celld.dev/docs/wasm/
    title: Imported Wasm and workers-rs deployment
  - id: celld-containers
    resource: https://celld.dev/docs/services/containers/
    title: Experimental native containers and ephemeral state
  - id: celld-security
    resource: https://celld.dev/docs/security/
    title: Trusted fleet assumptions and beta security boundary
  - id: rivet-rust
    resource: https://rivet.dev/actors/docs/quickstart/rust/
    title: Beta native Rust actor SDK and deployment options
  - id: rivet-actions
    resource: https://rivet.dev/actors/docs/actions/
    title: Concurrent actions by default
  - id: rivet-state
    resource: https://rivet.dev/actors/docs/state/
    title: Throttled in-memory state persistence
  - id: rivet-queues
    resource: https://rivet.dev/actors/docs/queues/
    title: Queue removal on receipt and completion semantics
  - id: rivet-sqlite
    resource: https://rivet.dev/actors/docs/sqlite/
    title: Actor-local SQL and callback transactions
  - id: rivet-limits
    resource: https://rivet.dev/actors/docs/limits/
    title: Storage, transaction, and gateway streaming limits
  - id: rivet-streams
    resource: https://docs.rs/rivetkit/latest/rivetkit/struct.StreamingResponse.html
    title: Rust streaming HTTP response API
  - id: rivet-hosting
    resource: https://rivet.dev/pricing/
    title: Apache-2.0 self-hosted core and enterprise distinction
  - id: restate-services
    resource: https://docs.restate.dev/foundations/services
    title: Services, Virtual Objects, and Workflows
  - id: restate-rust
    resource: https://docs.rs/restate-sdk/latest/restate_sdk/
    title: Native Rust durable execution SDK
  - id: restate-db
    resource: https://docs.restate.dev/guides/databases
    title: Application database integration and external idempotency
  - id: restate-streams
    resource: https://docs.restate.dev/ai/patterns/streaming-responses
    title: Pubsub and separate SSE endpoint pattern
  - id: restate-license
    resource: https://github.com/restatedev/restate/blob/main/LICENSE
    title: BSL server license and additional production use grant
  - id: dapr
    resource: https://docs.dapr.io/developing-applications/building-blocks/actors/actors-overview/
    title: Virtual actors and turn-based execution
  - id: dapr-rust
    resource: https://github.com/dapr/rust-sdk
    title: Alpha Rust SDK maturity
  - id: wasmcloud
    resource: https://wasmcloud.com/docs/overview/workloads/services/
    title: Stateful Wasm services with concurrent execution
---

# Portable Rocci backends on Durable Objects and actor platforms

This is exploratory research checked on **19 October 2026**, against Rocci
main at **24ab6a6b plus the existing working tree** and current primary platform
documentation. It approves no platform choice and implements no adapter.
Existing source modifications were preserved. Provider behavior was researched,
not deployment-tested; the experiment criteria below are proposed validation.

Implementation follow-up: [prove a remote Rocci backend on durable actors](/plans/rocci/portable-actor-backends.md), scoped through owner interview to a private Cloudflare deployment of the existing live-counter with real Roc execution, persistence, and two-browser SSE. celld remains a later portability gate; all implementation phases are planned.

## Assessment

Rocci is a good conceptual fit for durable actors: one server-side authority
can own a workspace or document, execute commands, and render coherent HTML to
every authorized viewer. Its pure render components and server-owned state
decisions already support this separation.[^state-decision][^render-decision]

The missing piece is a portable application-host contract. Today's native
server, a Workers Durable Object, and a WASI HTTP component have different
lifecycle, effects, storage, and concurrency contracts. A compiler target
change alone cannot make them equivalent.[^abi][^wasi][^cf-wasm]

The recommended investigation has two tracks: preserve a native deployment
baseline while defining a small actor/application boundary; then test a
Worker-facing Wasm adapter on both managed Cloudflare and self-hosted celld.
Rivet is the most relevant additional native actor candidate. Restate is
especially useful beside that application server for durable jobs.

A material change to the comparison occurred on **9 October 2026**: Deno joined
Cloudflare and the teams announced that celld code and ideas will merge into
workerd. This strengthens the direction toward self-hosted Workers, but it is
an announced convergence, not evidence of a finished unified runtime. celld
offers operational independence through self-hosting; it should no longer be
presented as an independent vendor counterweight.[^convergence]

## What portability should mean

| Dimension | Desired reusable contract | What still varies |
| --- | --- | --- |
| UI | HTML documents, stable-ID fragments, Datastar command/live roles | HTTP transport, assets, CSP, origin configuration |
| Domain | Roc types, validation, command decisions, view construction | Export ABI and effect orchestration |
| Identity | Application-defined tenant/entity key | Provider namespace, object ID, placement |
| State | Versioned schema, atomic command, explicit durability acknowledgement | SQL API, storage engine, backup and recovery |
| Execution | Defined command ordering and duplicate handling | Event-loop interleaving, threads, replay, passivation |
| Deployment | Reproducible bundle and conformance tests | Wrangler, celld fleet, native process, actor SDK, WASI host |
| Exit | Export/import, cutover, rollback, reconnect | Provider snapshots, opaque IDs, runtime metadata |

These are proposed evaluation dimensions. HTTP portability is already useful
without binary portability; source reuse is possible without data migration;
and self-hosting the same API does not establish equivalent failure behavior.
Require an actual data-and-traffic migration before claiming an exit path.

## Current Rocci evidence

The normal application runs as a native server, packaged independently of the
desktop preview; released platform archives currently cover Apple Silicon macOS
and x64 Linux.[^root][^platform] The platform requires init!, respond!, and
shutdown!. It retains one opaque application context for the server lifetime
and invokes handlers using
a bounded native worker executor. This is concurrent request processing,
without a built-in mailbox per document or tenant.[^abi][^native-host][^executor]

The effect surface includes filesystem paths, command execution, SQLite
connections and transactions, TCP, outbound HTTP, environment, clocks, and
sleep. Native SQL exposes a pool and pinned transaction handles. A Durable
Object has a different storage capability; preserving its atomic behavior is
more useful than emulating a process-local connection pool.[^abi][^sqlite]

Generated routes already distinguish full documents, finite fragments,
representation-free commands, and live SSE. Commands return empty SSE for
Datastar and 204 for ordinary callers. These externally visible semantics
should survive a backend change.[^runtime-doc]

Generated live is currently a per-connection timer poll. Each connection calls
its handler, renders HTML, compares with the previous string, and waits
100 ms; 150 quiet iterations produce a keepalive, approximately every
15 seconds. At negligible handler cost, 1,000 clients imply approximately
10,000 handler/render evaluations per second while nothing changes. These
are internal evaluations, not 10,000 new HTTP requests.[^dispatch]

The low-level SSE API already lets the host own scheduling, cancellation,
buffers, backpressure, and finite Emit/Wait/End transitions. It also supports
event IDs and retry metadata. However, the retained Source is a boxed Roc
callable. It is execution state, not a durable serialization format. Recover
an actor from records and versions instead of persisting closures or native
resource handles.[^sse]

There is an existing **shipped experimental** WASI HTTP 0.3 path on current
main. The CLI builds a Roc wasm32 object through a separate basic-webserver
fork and links a Rust component. The component has one instance-local adapter,
synchronous embedded SQLite, and SSE waits that yield outside application
execution. It currently buffers the complete request body before dispatch,
with no bound visible at that adapter boundary. It does not supply keyed
actors, distributed ownership, or provider-managed persistence.[^wasi][^http-module][^wasi-component]

Two evidence traps matter. The August WASI status record says the feature was
absent from main at that time; current code and READMEs supersede that claim.
Separately, the current Rocdown Wasm site path instantiates a no-op module and
uses the native applicator for page rendering. Its parity tests do not prove
that an independent Wasm renderer can run on Workers.[^wasi-history][^wasi][^apply]

## The actor structure that fits Rocci

The proposed unit is an **application aggregate**, not a rendered component:
one document, room, game, project, or workspace whose invariants can be
committed together. A component remains a pure function of explicit values.
The actor owns durable facts and command ordering; the renderer owns HTML;
Datastar carries intent and patches.[^state-decision][^render-decision]

Use a logical key such as (application, tenant, kind, entity ID). Resolve it
through an adapter to the provider object. Authenticate the caller and verify
membership before accepting the key; a user-supplied tenant path is not an
authorization decision. Keep private server-to-server capabilities behind the
same tenant boundary.

An illustrative mutation path is:

1. Receive a typed command with an operation ID, authenticated principal, and
   optional expected entity version.
2. Enter the aggregate's mutation boundary. Recheck authorization and look up
   the operation ID before changing state.
3. Atomically update application rows, increment a monotonic version, and store
   the deduplication result. Add an outbox record if reliable later work is needed.
4. Complete the storage durability barrier. Only then expose success and the
   new state to other systems or viewers.
5. Signal connected read channels, which render the committed canonical state.
   Bound pending delivery per viewer and permit snapshot replacement.

This is a proposed contract, not a claim that every provider supplies those
steps automatically. Ordinary transactional tables are sufficient; event
sourcing is optional. A full event log adds schema evolution and replay costs
and should solve an identified audit or reconstruction need.

Do not create one actor per DOM component. That fragments transactions and
multiplies routing, connections, and scheduling. A workspace actor is simple
when it is small; a busy workspace may need document actors plus asynchronous
indexes. A user actor fits private preferences, but cannot be the sole owner
of a document shared by several users.

## Cloudflare Workers and Durable Objects

Cloudflare's relevant model is stateless ingress routing to an addressable
Durable Object with private SQLite. SQL executes synchronously, and
transactionSync supplies callback-scoped atomicity. Arbitrary BEGIN and
SAVEPOINT statements through sql.exec are unsupported. A native pool-oriented
Sqlite adapter therefore needs semantic translation, not just renamed calls.[^cf-storage]

Single-threaded execution does not serialize an entire async handler. Storage
input gates protect storage operations, while an external fetch can allow
another request to change state. Output gates delay visible output behind
durability. Keep mutations synchronous and transactional where possible;
model external work as a pending operation with a checked version or
idempotent completion.[^cf-rules]

blockConcurrencyWhile is useful for initialization and migrations, but its
30-second timeout makes it unsuitable as a universal lock around long external
work. Reconstruct capabilities at activation and make initialization repeatable.
Do not depend on shutdown to flush required application state.[^cf-state]

Workers can invoke core Wasm and offer Rust bindings through workers-rs,
including SQL Durable Objects. That proves an implementation route for an
adapter; it does not run Rocci's native Rust host unchanged. Workers documents
only partial experimental WASI support, not execution of Rocci's WASI HTTP
0.3 service component. Validate each ABI and required import explicitly.[^cf-wasm][^cf-rust]

Current constraints relevant to Rocci include 10 GB SQLite storage per paid-plan
object, 100 SQL parameters, 2 MB rows/values, and a soft per-object throughput
limit around 1,000 requests per second. Individual objects remain single-threaded.
These are ceilings, not promised application throughput.[^cf-limits]

The Worker memory limit is 128 MB per isolate including Wasm allocations;
multiple requests share it. Current docs list a 64 MiB uncompressed Worker
bundle limit and a one-second global startup budget. Measure compiled Roc
size, memory, and activation time; older remembered size limits are not a
sound basis for this decision.[^cf-workers-limits]

Objects add a state-location hop. Location hints are best effort and apply at
creation; jurisdiction controls are a separate constraint. Globally distributed
ingress does not make one workspace a multi-writer database in every region.
Place the aggregate for its write audience and use derived read projections
where necessary.[^cf-location]

Alarms provide one durable scheduled wakeup per object, with at-least-once
invocation and bounded automatic retries. Store multiple due jobs in application
tables and schedule the earliest deadline. An alarm is not a hard real-time
game clock, nor an unlimited durable workflow retry policy.[^cf-alarms]

## celld and the self-hosted Workers route

celld embeds V8 in a Rust daemon and supplies distributed object ownership and
storage over nodes and an object-store bucket. It is Apache-2.0 licensed.
Unlike running a local Worker engine alone, this includes fleet coordination.
The convergence announcement explicitly identifies single-instance Durable
Objects as a gap in existing workerd self-hosting.[^celld][^convergence]

Current celld limitations identify v0.6.2 as beta. A fleet runs one application,
has no managed ingress, and balances ownership by cell counts/weights rather
than measured CPU or memory. Operators provide nodes, bucket, ingress TLS,
private peer networking, restart supervision, upgrades, and recovery drills.
This buys deployment control while transferring operational responsibility.[^celld-limitations]

Its object surface includes named stubs, SQL, transactions, alarms, and
hibernating WebSockets. Synchronous turns execute one at a time but await can
interleave handlers. Script and class names participate in identity; renaming
a Worker script reaches empty objects, and class rename/transfer migrations
are unsupported. Preserve logical application identity outside opaque IDs.[^celld-objects]

Wasm imports and workers-rs shims are documented. The same constrained Roc
core plus Worker adapter is therefore a credible cross-host experiment,
subject to compatibility testing. This is more promising than inventing a
celld-specific Roc application model.[^celld-wasm]

Durability deserves a precise reading. Ownership uses conditional bucket
updates and fencing epochs. In default fleet mode, acknowledgements can follow
follower fsync before bucket upload; with no follower, acknowledgement waits
for the bucket. Recovery can need surviving follower disks. Therefore a bucket
copy alone may not contain every acknowledged write at that instant. This
distinction matters for full-fleet loss and backup procedures.[^celld-guarantees]

The bucket must implement conditional create/overwrite and consistent reads
and ranges. Its qualified stores include S3, R2, Tigris, GCS, and Azure;
the docs warn that B2, Hetzner Object Storage, and DigitalOcean Spaces lack
the required conditional writes. Treat bucket compatibility as a correctness
requirement and run celld diagnose against the actual service.[^celld-guarantees]

Compatibility is substantial but explicit differences remain: inactive HTTP
streams expire after 60 seconds, with successful stream operations resetting
that inactivity timer; WebSocket transports cannot move with object
ownership, remote calls have limited safe retry opportunities, and Cloudflare
platform products such as Workers AI and Vectorize are unavailable. Deployment
configuration also differs. Budget heartbeats, reconnect, stable operation IDs,
and provider-specific configuration in the adapter.[^celld-compat]

The beta security document excludes hostile multi-tenant operation. A fleet
trusts its application and nodes; private/operator interfaces require network
protection. This does not prevent one trusted application from authenticating
its own end users, but it limits Rocci-as-a-platform hosting arbitrary customer
programs inside the same fleet.[^celld-security]

## Native containers: useful bridge, different durability

Both Cloudflare and celld can put a native program behind an object-managed
container. This is a plausible way to preserve a compiled Rocci server and its
HTTP interface. Cloudflare container disks are ephemeral by default; filesystem
snapshots under the durable_object scheduling policy are point-in-time
artifacts, not continuous transactional SQLite durability.[^cf-containers]

celld's container support is experimental, uses Docker or Podman on the owner
node, and also has ephemeral disk. Movement or restart destroys the container;
snapshot APIs are unsupported. One container per actor is a process-density
and startup tradeoff that needs measurement.[^celld-containers]

Wrapping the current Counter binary and leaving DB_PATH inside either
container is therefore insufficient. Choose an explicit persistence route:
an adapter to object storage APIs, an external transactional database, or a
tested durable-volume/SQLite replication design on a host that supports it.
Never mount a shared writable SQLite file across independent replicas and
assume actor routing makes that safe.

Forwarding HTTP to a container also does not serialize requests within the
Rocci native host. Define the mutation boundary there or in the storage layer.
Keep long-lived SSE out of a lock that blocks subsequent commands.

## Other actor and durable execution platforms

| Candidate | Why it is relevant | Main Rocci cost or mismatch |
| --- | --- | --- |
| Rivet Actors | Native Rust SDK, actor-local SQLite, raw streaming HTTP, self-hosted core | SDK is beta; a Roc FFI/subprocess and storage adapter are still required |
| Restate | Native Rust services, keyed single-writer objects, durable execution | K/V and journal model rather than actor-local application SQL; presentation stream remains separate |
| Dapr Actors | Language-neutral sidecar and turn-based virtual actors | More services to operate; Rust SDK is alpha; state-store adapter replaces native SQLite |
| wasmCloud services | Portable Wasm service execution | Current service model does not itself establish keyed durable actors |

Sources for the comparison are the current SDK and execution-model documents,
not an assumption that every product using the word actor has the same
contract.[^rivet-rust][^rivet-sqlite][^rivet-streams][^restate-rust][^restate-services][^dapr][^dapr-rust][^wasmcloud]

**Rivet** deserves a native prototype. Its Rust SDK avoids a prerequisite
Workers/Wasm port; its raw streaming response API can preserve Datastar instead
of adopting a new browser RPC model. Its self-hosted core is Apache-2.0, with
separate enterprise features.[^rivet-rust][^rivet-streams][^rivet-hosting]

Three qualifications prevent an unsafe abstraction: ordinary actions run
concurrently; in-memory state auto-saves on a throttle, one second by default;
and a queue removes a message when received, so a crash during processing does
not cause automatic redelivery. A successful action, a single actor, and a
persistent queue are not by themselves a durable command protocol.[^rivet-actions][^rivet-state][^rivet-queues]

Use callback SQL transactions for canonical mutations and explicitly persist
command IDs and pending effects. Recoverable submission also needs either a
client retry using the same command ID or a durable inbox persisted before
acknowledging acceptance: deduplication cannot recover a queue message lost
before its transaction began.[^rivet-queues] Current Rivet limits document 10 GiB shared
KV/SQLite per actor and roughly 1.31 MB of dirty SQLite pages per commit.
Chunk imports and verify migration/maintenance operations. The gateway supports
streaming without a fixed total response duration, with deployment-configurable
idle limits.[^rivet-sqlite][^rivet-limits]

**Restate** is strongest for imports, external API coordination, delayed work,
and human approval flows. Virtual Objects serialize writers per key while
allowing explicitly shared, read-only handlers to run concurrently; ordinary
read-only handlers still acquire the exclusive turn. Workflows have one run per
ID. Its journal and
K/V state solve a different problem from keeping a renderable SQLite aggregate
near active connections.[^restate-services]

An external database or API still needs idempotency: its effect can commit
before a durable execution engine records the result. Restate's own database
patterns make that boundary explicit. Keep domain transactions in the chosen
store. Couple reliable dispatch through a transactional outbox and stable
operation IDs, with idempotent or deduplicating receivers. An outbox alone
does not prevent repeated external effects after a lost acknowledgement.[^restate-db]

Restate's streaming guide uses a pubsub Virtual Object and a separate SSE
endpoint; it does not establish native Rust Datastar fan-out, and it currently
excludes direct token-by-token streaming. The server uses BSL with an additional
production grant, rather than an OSI open-source license; a public managed
hosting product exposing Restate and customer-registered services must account
for the license restriction.[^restate-streams][^restate-license]

**Dapr** is reasonable where its placement, sidecars, scheduler, and state store
already exist. Turn-based execution is useful, but the current Rust SDK's alpha
status adds integration uncertainty. **wasmCloud** is a component-hosting
option; its current stateful services handle concurrent tasks and retain heap
between calls, which is not evidence of per-key durable SQL or replay. Neither
is the shortest path for the first Rocci actor experiment.[^dapr][^dapr-rust][^wasmcloud]

## Integration strategies and the effects boundary

The following effort/risk assessments are architectural judgments, not measured
estimates or implementation commitments.

| Strategy | Reuse | New work | Assessment |
| --- | --- | --- | --- |
| Native origin behind edge Worker | Existing compiled server, SQLite, SSE | Routing, auth, deployment, durable origin storage | Fastest deployment baseline; adds no actor semantics by itself |
| Native Roc under actor host | Domain/render code; much native tooling | Per-key lifecycle, FFI or subprocess protocol, SQL adapter, notification delivery | Best way to test actor semantics independently of Wasm |
| Pure Roc core in Worker Wasm | Types, decisions, templates | Small export ABI and JS host orchestration | Strongest managed/self-hosted Worker portability target |
| Full native pf effect surface ported to Workers | Most existing handler source in principle | Every hosted capability and async boundary | Largest risk; avoid making it the first milestone |
| Existing WASI HTTP component on actor supervisor | Current experimental component path | Durable storage, actor routing/lifecycle, host compatibility | Valuable separate target; not a ready Workers artifact |

The cleanest first Worker experiment keeps Roc responsible for deterministic
domain decisions and rendering. A small host supplies an authenticated command,
state/view input, time or randomness where needed, and receives results or
effect requests. The host executes storage and network operations and calls
back with their results. JSON or another explicit value format at this
server boundary does not imply moving domain authority to the browser.

An alternative first slice could bridge synchronous Roc-hosted SQL directly to
Durable Object sql.exec, since that API is synchronous. But row types, parameter
binding, transaction scopes, exceptions, lifetime, and memory copies still need
an ABI. Outbound fetch remains asynchronous. Do not emulate a blocking native
network call by spinning the Worker event loop, and do not promise arbitrary
existing effectful handlers will run unchanged.

The proposed portable capability surface should be small and explicit:

| Capability | Contract to preserve | Provider detail to hide |
| --- | --- | --- |
| Identity | Stable logical aggregate key and authenticated principal | Namespace and object stub |
| Storage | Atomic mutation plus durable acknowledgement | Connection pool, KV journal, SQL callback API |
| Read model | Serializable canonical view input plus version | Query cursor and host handles |
| External effect | Named operation, stable ID, retry/completion policy | fetch, SDK, queue, durable step |
| Wakeup | Persisted due work and repeatable callback | Alarm, scheduler, process timer |
| Live delivery | Ordered versions, full resync, cancellation, bounded backlog | ReadableStream, native SSE body, connection registry |
| Assets | Immutable app assets and content types | CDN binding, local files, object store |

Use native and Worker implementations to discover the common contract before
freezing it. Native-only filesystem, shell, and unrestricted socket capabilities
may remain explicit optional capabilities. A portable subset is more credible
than a universal platform that silently changes their behavior.

Keep these changes in runtime/platform and CLI build/dispatch ownership. Pure
template lowering need not know about actor placement, alarms, or Cloudflare
bindings. Actor lifecycle also differs from process @init: an activation may
repeat, migrate schemas idempotently, and reconstruct capabilities. Do not
silently reinterpret a process initializer as a once-ever actor creation hook.

## Live HTML, hibernation, and economics

Durable Objects can return streaming responses through Workers, so SSE is a
valid transport route.[^cf-streams] However, an outstanding response and pending
work prevent ordinary hibernation. The dedicated WebSocket hibernation API can
keep sockets open after heap state is discarded; SSE has no equivalent
documented capability. A permanent SSE connection therefore changes the cost
and residency model.[^cf-lifecycle]

Do not replace Datastar with browser WebSocket state solely to obtain cheaper
idle actors. First measure the real workload. Plausible alternatives include
short-lived read streams with snapshots, one active read relay for multiple
aggregates, or an optional host notification channel feeding the existing
HTML/SSE presentation server. A relay that holds one stream open to every actor
still pins those upstream actors; it needs durable notifications or another
bounded wakeup protocol to decouple residency.

For an actor-backed live implementation, prefer committed-change notification
over the current polling loop. Subscribe and read a versioned snapshot without
a race: either do both in one serialized turn or register, then verify the
version before releasing the initial result. Otherwise a command between the
snapshot and subscription can disappear from the live view.

Render once per authorization/locale/view-equivalent audience, then fan out.
Never share private HTML merely because viewers watch the same entity. Keep
one authoritative stream per coherent DOM boundary. A version or event ID
alone does not make Datastar reject stale patches: ordering must be enforced
by the host delivery queue, with stale work discarded before emission. Verify
actual reconnect cursor behavior in the deployed Datastar client rather than
assuming that SSE metadata supplies end-to-end replay.

Use a full fresh snapshot after reconnect as the initial design. Retain event
history only if replay is measurably useful. Bound each viewer's queue; for
replaceable HTML, coalesce to the newest committed snapshot rather than
retaining every intermediate render. Detach cancelled/slow consumers and keep
heartbeat traffic independent of render polling.

Cloudflare bills non-hibernating object duration using a 128 MB allocation per
object, shared across that object's simultaneous requests. Request and SQL
row/storage metering are additional. As an illustrative calculation, 100
continuously active objects for 30 days consume approximately **33.18 million
GB-seconds** before included allowances. This is about active objects, not
individual client sockets; many viewers can share an object.[^cf-pricing]

The cost model should measure active-object-seconds, writes, rows scanned,
render frequency, HTML size, and reconnect churn. Polling every 100 ms creates
CPU and query work even without a change. Short timeouts can reduce residency
while causing more activations. Sharding reduces hotspots while increasing
active object count. Measure those coupled effects before choosing granularity.

## Data movement, global queries, and failure handling

These are recommended application responsibilities, rather than additional
guarantees attributed to the platforms:

- **Migration:** export logical keys, schema version, rows/snapshots, current
  entity version, deduplication history, outbox status, and timer intent.
  Rebuild native handles, subscriptions, and provider alarm registrations.
- **Cutover:** stop or fence the old writer, finish/export pending work, import
  and verify checksums/versions, switch routing, and force clients to resync.
  Avoid live dual-writer operation across providers without a separately
  designed replication protocol.
- **Rollback:** distinguish a code rollback from a schema rollback. Preserve
  compatible data formats or an explicit forward repair path, including
  outstanding commands created by the newer version.
- **Global search and reporting:** maintain asynchronous projections outside
  individual object databases. Cross-actor SQL joins and global uniqueness
  constraints do not appear automatically because each actor uses SQLite.
- **Cross-aggregate transactions:** co-locate invariants where possible; use
  explicit sagas/reservations, reconciliation, and idempotent messages where
  they span actors. Do not hold a database transaction over a network call.
- **Hot aggregates:** monitor mailbox depth, processing latency, fan-out cost,
  and database size. Split by a domain boundary when required; autoscaling
  many actors does not parallelize one actor's critical section.
- **Retries:** persist operation identity across a lost reply. A timeout cannot
  tell the caller whether a mutation committed. Deduplicate with payload
  validation and a defined retention window, not just a volatile cache.
- **Timers:** persist deadlines and job identity, define catch-up after downtime,
  and make completion repeatable. Browser ticks and process shutdown hooks
  are insufficient for durable scheduling.

Server-owned state also does not imply offline synchronization. A local
desktop database and a remote actor are two authorities unless one is explicitly
the primary and the other a cache. Offline multi-writer work would require its
own merge/conflict model. Keep that outside the first backend portability claim.

## Recommendation and evidence needed next

Prefer **portable semantics with a constrained execution core**. Retain a
native origin as a control, investigate Rivet for native actor hosting, and
test one Worker adapter against Cloudflare and celld. Use Restate when the
application needs reliable multi-step background execution; do not require it
for an ordinary shared document and HTML stream.

Choose a small collaborative task board or document aggregate, not just a
counter. It exercises typed commands, SQL transactions, two viewers, private
authorization, and a durable delayed action without requiring a new template
language feature. This research recommends the following bounded experiments;
it does not start implementation phases:

1. **Native control:** record current command behavior, rendered bytes, assets,
   startup, memory, and per-connection polling cost through the real HTTP origin.
2. **Roc execution proof:** independently compile and invoke actual domain and
   render functions in the chosen Worker Wasm ABI. Check imports, exports,
   allocation limits, errors, Unicode, and output parity. A stub is not the exit.
3. **Storage proof:** run parallel writes to one key and writes to distinct keys;
   check atomicity, duplicate suppression, expected-version conflict behavior,
   and read-after-acknowledgement after restart.
4. **Live proof:** show two authorized clients receiving only committed versions;
   disconnect one during a write, reconnect, and verify full resync. Exercise
   slow readers, cancelled streams, a snapshot/subscription race, and idle
   periods longer than each proxy/runtime timeout.
5. **Recovery proof:** kill the owner around commit and reply; retry the same
   operation ID; fire the delayed action twice. On celld, also test takeover
   with bucket upload delayed and follower recovery in the supported topology.
6. **Portability proof:** run the same Worker bundle and application tests on
   Cloudflare and celld, with provider configuration isolated. Export a populated
   aggregate and import it into the other backend with pending work intact.
7. **Native actor comparison:** implement the same command/read protocol with
   a native host, preferably testing Rivet's real Rust API. Measure integration
   cost rather than assuming that an SDK eliminates the Roc host boundary.
8. **Operational decision:** compare active/idle cost, cold activation latency,
   per-key throughput, bytes per client update, import limits, recovery time,
   upgrade compatibility, and operator effort. Record measured limits and
   failures before proposing a public backend contract.

The attractive endpoint is one Rocci application model with several explicit
hosts. The evidence so far supports that direction at the domain/HTML level;
execution, storage migration, and failure-equivalent behavior remain to be
demonstrated. No claim of production readiness or unchanged binary portability
is established by this research.

[^root]: Current product separation, native workflows, and server packaging.
[^state-decision]: Accepted direction for authoritative durable state and coherent HTML.
[^render-decision]: Components are ordinary pure render functions, without hidden actors.
[^platform]: Native host origin, effect/platform boundary, and shipped archives.
[^abi]: init/respond/shutdown contract, hosted functions, and native target declarations.
[^native-host]: One server-lifetime retained context; handlers share retained references.
[^executor]: Fixed worker threads and bounded admission, without per-entity mailboxes.
[^dispatch]: live_sse_arm renders each connection at After(100), compares HTML, and emits keepalives.
[^runtime-doc]: Current handler matrix and process lifecycle; generated commands have no representation.
[^sse]: Host-owned finite transitions and framing; Source retains a Roc callable; IDs are supported.
[^sqlite]: Pool and transaction semantics are native capabilities, not generic provider SQL.
[^wasi]: Experimental HTTP service component is present; native run and Wasm apply remain distinct.
[^wasi-component]: Instance adapter mutex, whole-body buffering, and released lock around SSE waits.
[^http-module]: Fork-based object capture and linking establish a separate component build path.
[^apply]: The current site Wasm path still uses a native applicator for actual page rendering.
[^wasi-history]: Dated pre-merge snapshot, preserved as historical evidence.
[^convergence]: Joint announcement dated 2026-10-09; the workerd/celld merge is planned development.
[^cf-storage]: SQL is synchronous; callback transaction APIs replace raw transaction statements.
[^cf-rules]: Input/output gates do not eliminate races across external async operations.
[^cf-state]: Initialization barrier, timeout, and WebSocket hibernation API.
[^cf-lifecycle]: Hibernation discards heap and requires no pending response/event work.
[^cf-limits]: Paid object storage, SQL limits, and soft per-object throughput constraint.
[^cf-workers-limits]: Current per-isolate memory, uncompressed bundle, and startup budgets.
[^cf-wasm]: Core Wasm support is distinct from experimental partial WASI compatibility.
[^cf-rust]: Rust Durable Object and SQL examples use a Workers-specific Wasm/JS binding layer.
[^cf-streams]: ReadableStream response forwarding and cancellation are supported.
[^cf-alarms]: One alarm per object, at-least-once execution, and bounded failure retries.
[^cf-location]: Initial placement, best-effort hints, and jurisdiction are distinct concerns.
[^cf-pricing]: Duration allocation is per object; request and SQL dimensions are separate.
[^cf-containers]: Linux containers run native workloads; ephemeral disks and point-in-time snapshots differ.
[^celld]: Rust/V8 distributed self-hosted runtime, Apache-2.0 licensing, and bucket-based fleet model.
[^celld-limitations]: v0.6.2 beta, one application per fleet, operational and placement limitations.
[^celld-objects]: Supported SQL/alarms, await interleaving, identity derivation, and migration gaps.
[^celld-guarantees]: Conditional storage, fencing, follower fsync, bucket fallback, and recovery requirements.
[^celld-compat]: Stream inactivity, non-movable transports, safe retry limits, and product/API differences.
[^celld-wasm]: Compiled Wasm imports and documented workers-rs shim deployment.
[^celld-containers]: Experimental owner-local Docker/Podman containers with ephemeral disks.
[^celld-security]: Trusted application/fleet assumptions and beta multi-tenant restrictions.
[^rivet-rust]: Native Rust public SDK is beta and needs an actor implementation.
[^rivet-actions]: Actions execute concurrently by default.
[^rivet-state]: Throttled auto-save is not tied to action completion; explicit immediate save exists.
[^rivet-queues]: Receive removes a queue message; crash during processing does not redeliver it.
[^rivet-sqlite]: Callback SQL transactions serialize database work; native SQLite handles are not the API.
[^rivet-limits]: Shared actor storage, dirty-page commit bounds, and configurable gateway idle limits.
[^rivet-streams]: Native Rust streaming response API supports a Datastar-compatible HTTP adapter.
[^rivet-hosting]: Core self-hosting license and enterprise offering are distinguished.
[^restate-services]: Keyed single-writer Virtual Objects, concurrent readers, and journaled workflows.
[^restate-rust]: Rust SDK supplies durable service/object/workflow integration, not the Rocci ABI.
[^restate-db]: External database effects need explicit idempotency despite journaled execution.
[^restate-streams]: Documented pubsub plus separate SSE endpoint; direct token streaming excluded.
[^restate-license]: BSL and additional grant distinguish application use from a competing managed platform.
[^dapr]: Virtual actor turns and state-provider architecture.
[^dapr-rust]: Repository describes the Rust SDK as alpha and incomplete.
[^wasmcloud]: Stateful services retain memory but execute tasks concurrently; this is not a durable-actor specification.
