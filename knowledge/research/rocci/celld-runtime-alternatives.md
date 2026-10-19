---
type: Research Report
title: celld alternatives for Roc and Rocci hosting
description: Compare stock celld, shared JSPI effects, custom native cell engines and containers using pinned source and a bounded local runtime probe.
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/persistence]
status: draft
generated: { by: process:cursor, at: 2026-10-19T11:44:31Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: container-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/container.rs
    title: Named OCI runtime forwarded through Docker HostConfig
  - id: kata
    resource: https://katacontainers.io/
    title: OCI containers running inside lightweight hardware-isolated virtual machines
  - id: unikraft
    resource: https://unikraft.org/docs/concepts/compatibility
    title: Unikernel Linux ABI and ELF binary compatibility
  - id: native-host-cargo
    resource: ../../../crates/rocci-platform/Cargo.toml
    title: Native Rust host dependency and operating-system surface
  - id: prior-audit
    resource: ../../audits/rocci/worker-packaging-and-source-portability.md
    title: Worker POC source-portability assessment
  - id: probe
    resource: celld-runtime-probe-results.json
    title: Recorded local inputs, artifact hashes, runtime results and limits
  - id: wasm-doc
    resource: https://celld.dev/docs/wasm/
    title: Imported core Wasm and prebuilt no-bundle deployment
  - id: compat
    resource: https://celld.dev/docs/cloudflare-compat/
    title: Runtime and configuration differences
  - id: objects
    resource: https://celld.dev/docs/services/durable-objects/
    title: Object lifecycle, transactions and durability
  - id: limits
    resource: https://celld.dev/docs/limitations/
    title: Beta deployment and fleet requirements
  - id: security
    resource: https://celld.dev/docs/security/
    title: Application trust and private operator boundary
  - id: containers
    resource: https://celld.dev/docs/services/containers/
    title: Experimental native containers and ephemeral disks
  - id: convergence
    resource: https://blog.cloudflare.com/deno-joins-cloudflare/
    title: October 9 convergence announcement and future work
  - id: jspi
    resource: https://v8.dev/blog/jspi
    title: Runtime suspension of sequential Wasm through Promise-bearing imports
  - id: sql-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/storage.rs
    title: SQL authorizer, exact integer values and internal transaction control
  - id: harness-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/js/harness.js
    title: Callback transactions and storage lifecycle wrapping
  - id: host-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/cell_host.rs
    title: Internal V8 and test cell-host variants
  - id: engine-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/engine_api.rs
    title: Engine-neutral HTTP and durability-position types
  - id: runtime-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/runtime.rs
    title: V8 RuntimeManager and engine-specific execution
  - id: python-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/celld/python/shims.js
    title: Python-specific JSPI workaround and its documented stack leak
  - id: ltx-readme
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/ltx/README.md
    title: Replication capture versus the wider celld durability protocol
  - id: ltx-source
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/ltx/src/lib.rs
    title: Embeddable replication and restore API
  - id: ltx-manifest
    resource: https://github.com/denoland/celld/blob/90b43017241f81189453d326d05948f388b34652/crates/ltx/Cargo.toml
    title: Unpublished library and SQLite dependency coupling
  - id: roc-platform
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Native Rocci hosted-effect and lifecycle surface
  - id: roc-sql
    resource: ../../../crates/rocci-platform/platform/Sqlite.roc
    title: Native database pools and transaction handles
  - id: wasi
    resource: ../../../crates/rocci-wasi-http/README.md
    title: Experimental WASI HTTP path
  - id: current-config
    resource: ../../../examples/rocci/remote/live-counter/tools/configure.mjs
    title: Current Cloudflare-specific deployment configuration
  - id: arena
    resource: ../../../examples/rocci/remote/live-counter/platform/host.c
    title: Current counter ABI invocation arena reset
  - id: cf-sql
    resource: https://developers.cloudflare.com/durable-objects/api/sqlite-storage-api/
    title: Cloudflare callback transaction restrictions
---

# celld alternatives for Roc and Rocci hosting

## Answer and evidence scope

Stock celld does not remove the main effect-compatibility work identified in
the [Worker packaging audit](/audits/rocci/worker-packaging-and-source-portability.md).
It deliberately implements the Workers programming model. Its distinct
opportunity is control over the host implementation: a custom native or
Wasmtime cell engine could preserve more of Rocci's native effects, at the
cost of developing a runtime integration rather than an ordinary deployable
Worker.[^prior-audit][^host-source][^runtime-source]

A concrete additional approach is JavaScript Promise Integration (JSPI).
It suspends a sequential Wasm call at an asynchronous imported function.
The local probe passed on both celld and workerd, so this avenue is shared,
not an advantage exclusive to celld. It may reduce changes to Roc handler
source without requiring an explicit continuation transformation in Roc,
but a real Roc effect ABI remains to be qualified.[^jspi][^probe]

Reviewed celld v0.6.2, commit
`90b43017241f81189453d326d05948f388b34652`, and Rocci
`bbf7b1effb4bbeca6fb1548ac119e82c20435622`. The celld release binary's
published SHA-256 was checked before execution. Existing Rocci knowledge
edits and concurrent actor-POC README/status edits were preserved.
The results below are local loopback execution, not hosted/fleet acceptance,
and no new product target or deployment was implemented.[^probe]

## What remains substantially the same

| Concern | Implication for Rocci |
| --- | --- |
| JavaScript-hosted core Wasm | celld imports a compiled Wasm module through a Worker entry point, including workers-rs shims. This does not load the native Rocci platform or a WASI HTTP component automatically. [^wasm-doc][^wasi] |
| SQL effects | Both hosts provide synchronous object SQL. celld explicitly denies application transaction/savepoint statements in its Rust authorizer. Native pools and linear transaction handles still need a compatibility layer. [^sql-source][^roc-sql][^cf-sql] |
| Actor turns | Synchronous turns serialize; awaiting can admit another event. A compatible host must preserve application invariants across those boundaries. [^objects] |
| Reconstruction | Object identity, persistent rows and migrations survive; context/handles and heap must be recreated. Native process initialization is not once-ever actor creation. [^objects][^roc-platform] |
| Framework infrastructure | Dispatch, request/response mapping, asset staging, retries and live delivery still need reusable tooling. Switching hosts does not generate it. [^prior-audit] |

celld implements transaction control internally using real SQLite BEGIN,
COMMIT and savepoints, behind a callback-managed public surface. Therefore,
a native engine integration could adapt that internal capability differently.
That is a possible extension to the host; the ordinary celld Worker SQL API
does not become a native transaction-handle API.[^sql-source][^harness-source]

## Bounded local experiment

The same Worker class/config and the existing 74,972-byte Roc counter Wasm
were run inside an object under celld 0.6.2 and local workerd 1.20261012.1
through Wrangler 4.151.1. Node was 26.12.0 on Apple Silicon.
The complete small C/JS/config inputs, hashes, observed values and
reproduction steps are in the recorded evidence file
`celld-runtime-probe-results.json`.[^probe]

| Check | celld | Local workerd |
| --- | --- | --- |
| Existing Roc `actor.wasm`, increment and fragment rendering | Count 1, revision 1; counter and feed targets present | Identical result |
| Raw SQL `BEGIN` | Rejected as unauthorized | Rejected with callback-transaction guidance |
| Throw after UPDATE inside `transactionSync` | Rolled back; value remained 0 | Rolled back; value remained 0 |
| `WebAssembly.Suspending` / `promising` | Available | Available |
| 1,000 sequential calls into C Wasm, each awaiting a 1ms host timer | All results correct | All results correct |
| C linear stack pointer before/after JSPI loop | 65,536 / 65,536 | 65,536 / 65,536 |
| C linear memory | 65,536 bytes | 65,536 bytes |
| Roc arena / linear memory for the separate synchronous call | 14,316 / 5,570,560 bytes | Same |

These observations come from the recorded local probe.[^probe]

The JSPI test is compiled C with a volatile stack frame, not an effectful
Roc program. The real Roc artifact is tested separately via its existing
synchronous import-free ABI. An unchanged linear stack pointer is not a
measurement of V8 continuation-stack memory or proof of leak freedom.
The probe does not establish concurrent Roc suspension, cancellation,
outbound-fetch behavior, SSE parity, takeover or hosted runtime support.
Both temporary server process groups were stopped after the final run.[^probe]

## The shared alternative: suspend effects at the Wasm boundary

Instead of extracting every effectful handler into a pure state machine,
a proposed Worker platform could wrap asynchronous hosted imports with
`WebAssembly.Suspending` and enter its Wasm exports through
`WebAssembly.promising`. Roc can keep sequential calls while the JS host
awaits body input, outbound HTTP or a timer. Synchronous SQL imports can
remain ordinary imports.[^jspi][^probe]

This refines the earlier audit's asynchronous-effect discussion: suspension
support is still required, but JSPI supplies a concrete runtime mechanism
worth trying on both targets. No inference about compiler ABI compatibility
follows from the tiny C proof.[^prior-audit][^probe]

The key remaining qualification work is:

- Exercise an actual Roc hosted effect across a suspension, including result/
  error marshalling and cleanup.
- Establish one active invocation per Wasm instance initially, or correctly
  isolate concurrent invocation stacks/arenas. The current POC resets its
  shared arena on every invoke and cannot safely be reused concurrently
  while an earlier invocation owns suspended data.[^arena]
- Test cancellation and repeated suspension with retained memory, traps,
  reentrancy and actual resource ownership. Serializing suspended calls can
  also stall an actor; calls back into the same actor require care.
- Keep transaction work synchronous within `transactionSync`; suspension
  does not extend its callback lifetime. SQL transaction scope and the
  native `Sqlite.begin!` contract still need a separate design.[^cf-sql]

celld's own Python shim hides JSPI only within the generated Pyodide bundle
because that runtime leaks 48 bytes of its C stack per awaited entry.
The global APIs remain available to ordinary Workers, as the probe confirms.
That is evidence for testing stack/lifetime behavior, not proof that Roc has
the same problem or that JSPI is unusable.[^python-source][^probe]

## What celld changes operationally

- `celld deploy` can load prebundled JS/Wasm using `no_bundle: true` without
  esbuild. A future Rocci CLI could produce that artifact, so deployment
  need not require Node/npm/Wrangler on that path.[^wasm-doc]
- It does not accept the POC's current Cloudflare configuration unchanged:
  custom routes/account/preview controls, declarative class exports, version
  metadata and additional-module discovery need provider-specific output.
  celld uses its supported legacy SQLite-class migrations. Its compatibility
  flags are a subset, so accepted-but-ignored flags are not behavior proofs.
  This matters to the POC's request-signal configuration.[^compat][^current-config]
- Remote HTTP stream leases expire after 60 seconds without a successful
  operation, and cancellation/RPC and retry behavior differ. Keep heartbeat,
  operation-ID and reconnect checks in cross-host conformance.[^compat]
- The operator supplies nodes, ingress/TLS, a qualified bucket and supervision.
  A fleet runs one application; v0.6.2 is beta and trusts its application
  code and operators. The switch trades managed infrastructure for control
  over placement, capacity and investigation.[^limits][^security]
- SSE still keeps active application work resident. Owning the node changes
  the cost model, but does not by itself implement hibernation of a running
  SSE response or its timers.[^objects]

## celld-specific alternative: a direct Roc cell engine

The pinned source has useful seams:

- `engine_api.rs` separates engine-neutral request/response values and
  committed/observed storage positions.[^engine-source]
- `CellRuntime` holds generation, replication and other common runtime
  material; the current `RuntimeManager` supplies V8 execution.[^runtime-source]
- `cell_host.rs` is the actor's runtime boundary. It currently has a
  crate-private V8 variant and a test-only scripted variant, not a supported
  third-party engine registration API.[^host-source]

An exploratory integration could add a Roc engine there. It could execute
native compiled Roc or core Wasm under Wasmtime, bind hosted effects directly
to Rust, and render through the existing Rocci dispatch. Removing the JS
value boundary could preserve full SQLite integer values and avoid JS
callback constraints at the Roc-facing API, while the host still enforces
transaction scope. These are design possibilities, not implemented features.

The hard requirement is to use the cell's managed storage and durability
protocol: activation epochs, ownership fencing, restore, observed/committed
positions, output gating, stop/cancellation and migration. Opening an
independent `counter.db` in a native child process does not attach it to those
guarantees. The engine-neutral response fields make that responsibility
visible.[^engine-source][^host-source][^runtime-source]

This could improve source compatibility substantially, but it requires an
upstream extension or maintained fork and a different isolation policy for
native code. It would not run on managed Cloudflare Workers. The announced
celld/workerd convergence is future integration work, so internal seams are
not a stable product commitment.[^host-source][^convergence]

### Reuse replication without adopting the whole runtime

`celld-ltx` is an embeddable Rust WAL capture/restore library. This presents
a second native-host research direction: retain Rocci's native process and
SQLite and add replication at the host layer.[^ltx-source]

It is not equivalent to adopting celld's Durable Objects. Its README
explicitly assigns output gates, epoch fencing, replicated node logs and
takeover recovery to the larger celld protocol. The library is also
`publish = false`, with SQLite dependency coupling. Reusing it alone can
support a deliberately narrower replication/backup product, but cannot
honestly promise celld's distributed acknowledged-write semantics.[^ltx-readme][^ltx-manifest]

## Container alternative

celld can start an experimental native container for an object and forward
HTTP to its port. That can run the existing native Rocci server with little
source change. Container storage is ephemeral: node movement/restart destroys
the container, and native SQLite does not automatically become object SQL.
The documented container snapshot methods are unavailable in celld.
A separate durable-state design is still required.[^containers]

This is appropriate if native application packaging is the priority and its
state backend is independently solved. It does not prove that the Roc app
executes as a Worker or that its local DB has Durable Object recovery.
Cloudflare also has Containers, so native process hosting itself is not
unique to celld.[^containers]

## Recommendation

### Native binary with VM isolation

There is an existing celld hook for this path: each container class can name
an OCI runtime through `runtime`, with a node-wide default in
`CELLD_CONTAINER_RUNTIME`. The pinned source forwards that choice into
Docker's `HostConfig.Runtime`. celld's docs name `kata` as an example;
the operator must configure a compatible runtime in the daemon. This is an
implementation hook, not evidence that a celld/Kata deployment was tested
here.[^container-source][^containers]

Kata runs OCI workloads in lightweight virtual machines. A proposed native
Rocci deployment can therefore package the ordinary Linux server binary
into an OCI image and run it behind that VM boundary, while a small generated
Worker/object wrapper owns routing and container lifecycle. The application
does not need to become Wasm. A default ordinary container shares the host
kernel; gVisor's `runsc` and a hardware-backed Kata VM are distinct isolation
choices.[^kata][^containers]

A true unikernel is another step: Unikraft offers Linux ELF compatibility,
so an application may use a binary-compatibility loader rather than being
rewritten. Rocci's native host uses Rust/Tokio, sockets, timers and SQLite;
the selected unikernel would need qualification against that actual syscall,
threading and filesystem workload. Stock celld accepts OCI images through
its container engine, not a raw unikernel kernel image; an OCI-compatible
unikernel runtime or additional integration would be required.[^unikraft][^native-host-cargo][^container-source]

VM isolation does not attach native SQLite to object durability. The native
process writes its guest filesystem, while the Durable Object's database is
a separate storage capability. Two proposed ways forward are a platform
storage bridge, or managed per-actor disks with ownership fencing, replication,
restore and acknowledged-write guarantees. Neither is supplied merely by
choosing a different OCI runtime. VM memory/disk snapshots alone do not prove
those guarantees.

For preserving existing native applications, investigate Linux OCI plus
VM-backed isolation before a true unikernel; the latter adds another
compatibility boundary. This is a separate, credible native-hosting product
direction. It can keep application source largely intact while moving the
hard work into shared packaging, lifecycle and persistent storage support.

### Target selection

For a reusable SDK that should run on both hosts, retain the common
Worker/core-Wasm target, build the compatible Roc SQL/lifecycle/dispatch
surface, and investigate JSPI with one real Roc hosted-effect proof.
Use provider-specific deployment configuration and authentication.
celld is a useful second execution target, not a shortcut that removes the
platform port.

If preserving the broad native Rocci effect surface matters more than
Cloudflare compatibility, a direct Roc/Wasmtime cell engine is the most
interesting celld-specific option. Seek an upstream engine boundary before
committing to that implementation. Treat containers and replication-only
native hosting as separate products with explicit persistence semantics.

The next small experiment should combine an actual Roc effect suspension,
a SQL mutation/rollback and one repeat activation. It should not yet add a
new parser declaration, port all effects, or claim a complete celld deployment.

[^prior-audit]: Worker POC source-portability assessment.
[^container-source]: Named OCI runtime forwarded through Docker HostConfig.
[^kata]: OCI containers running inside lightweight hardware-isolated virtual machines.
[^unikraft]: Unikernel Linux ABI and ELF binary compatibility.
[^native-host-cargo]: Native Rust host dependency and operating-system surface.
[^probe]: Recorded local inputs, artifact hashes, runtime results and limits.
[^wasm-doc]: Imported core Wasm and prebuilt no-bundle deployment.
[^compat]: Runtime and configuration differences.
[^objects]: Object lifecycle, transactions and durability.
[^limits]: Beta deployment and fleet requirements.
[^security]: Application trust and private operator boundary.
[^containers]: Experimental native containers and ephemeral disks.
[^convergence]: October 9 convergence announcement and future work.
[^jspi]: Runtime suspension of sequential Wasm through Promise-bearing imports.
[^sql-source]: SQL authorizer, exact integer values and internal transaction control.
[^harness-source]: Callback transactions and storage lifecycle wrapping.
[^host-source]: Internal V8 and test cell-host variants.
[^engine-source]: Engine-neutral HTTP and durability-position types.
[^runtime-source]: V8 RuntimeManager and engine-specific execution.
[^python-source]: Python-specific JSPI workaround and its documented stack leak.
[^ltx-readme]: Replication capture versus the wider celld durability protocol.
[^ltx-source]: Embeddable replication and restore API.
[^ltx-manifest]: Unpublished library and SQLite dependency coupling.
[^roc-platform]: Native Rocci hosted-effect and lifecycle surface.
[^roc-sql]: Native database pools and transaction handles.
[^wasi]: Experimental WASI HTTP path.
[^current-config]: Current Cloudflare-specific deployment configuration.
[^arena]: Current counter ABI invocation arena reset.
[^cf-sql]: Cloudflare callback transaction restrictions.
