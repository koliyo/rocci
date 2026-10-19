---
type: Status
title: Worker source portability implementation evidence
description: Phase microplans, frozen source promises and measured qualification of a real Roc Worker ABI; managed compatibility remains gated.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T14:48:19Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/rocci/worker-packaging-and-source-portability.md
    title: Requested phased implementation and acceptance gates
  - id: fixtures
    resource: ../../crates/rocci-cli/tests/fixtures/worker/manifest.json
    title: Frozen source hashes, revisions and dependency manifests
  - id: source-test
    resource: ../../crates/rocci-cli/tests/worker_sources.rs
    title: Hash and ordinary lowering checks
  - id: platform
    resource: ../../crates/rocci-worker-platform/platform/main.roc
    title: Candidate ordinary Roc platform contract
  - id: shim
    resource: ../../crates/rocci-worker-platform/platform/host.c
    title: ARC allocator, resource finalizers and entrypoint ownership
  - id: qualification
    resource: ../../crates/rocci-worker-platform/tools/conformance.mjs
    title: Pinned actual-Roc compiler matrix and measurements
  - id: admission
    resource: ../../crates/rocci-worker-platform/runtime/admission.mjs
    title: Bounded invocation admission and generation invalidation
  - id: runtime
    resource: ../../crates/rocci-worker-platform/README.md
    title: Capability, ownership and distribution profile
  - id: packaging
    resource: ../../crates/rocci-cli/src/worker.rs
    title: Public commands and ordinary generated-dispatch staging
  - id: config
    resource: ../../crates/rocci-core/src/worker.rs
    title: Strict Worker target configuration
  - id: editor
    resource: ../../crates/rocci-lsp/src/projection_workspace.rs
    title: Shared Worker editor platform profile
  - id: evidence
    resource: worker-source-portability-evidence.json
    title: Stored pinned artifact hashes and local/hosted measurements
  - id: guide
    resource: ../../docs/applications/workers.rocdown
    title: Public starter workflow and current limits
---

# Worker source portability implementation evidence

This record tracks execution requested after the original plan was written.
No phase is logged complete until its required local, hosted and CI evidence
exists on the implementation revision. The private actor deployment remains
independent. The native compiler pin and working example sources are preserved.[^plan]

## Phase 0 microplan and source promises

1. Snapshot C0 and C2 from `9b451fc34834af50fcee5d850b7d2253c9cbbd3d`;
   snapshot C1 and all sibling Roc/Rocci dependencies from
   `97227bf514da4b5b08fe092db687c2d53141f77a`.
2. Store bytes, SHA-256, original paths and source deltas in a test manifest;
   verify ordinary lowering without rewriting sources.
3. Keep Worker selection separate from CPU targets, native run and WASI.
   Reserve `rocci worker init/build` for the generated provider workflow.
4. Select a stable generated class (`RocciApp`), singleton object identity from
   explicit configuration, object-managed logical SQLite path aliases and
   exact full-range I64 SQL. Provider config owns migrations, identity,
   compatibility date and secrets. Builds must never rewrite namespace identity.
5. Qualify a separate candidate compiler, pinned runtime and ownership ABI
   before advertising an application-facing profile.[^fixtures][^source-test]

| Fixture | Promised authoring surface | Qualification required |
| --- | --- | --- |
| C0 | Unchanged init/context, Env/Path, typed SQL, logging, document and finite fragments | Real object SQL, reconstruction, native HTML/Datastar parity |
| C1 | Original live handlers, body reads, SQL, command/live URLs and markup | Actual compiled handlers, two browsers, bounded delivery and cleanup |
| C2 | Current effectful handlers, body reads and linear transactions | Rendezvous commit/rollback/drop semantics; explicit source delta if needed |

No unsupported capability may be silently stubbed. OS processes, raw TCP,
arbitrary filesystem and runtime compilation are initially unsupported.
SQL numeric fidelity, transactions, async body/fetch, live delivery and hosted
execution are separate gates; freezing a source does not qualify its runtime.[^plan]

## Phase 1 microplan

1. Compile a small ordinary Roc platform with direct hosted effects and JSPI.
2. Replace invocation-reset allocation with persistent ARC allocation and
   coalescing reclamation; prove retained strings, closures, boxes and finalizers.
3. Exercise success and typed failure, empty/small/large/Unicode/NUL strings,
   effects before and after suspension, and complete context teardown.
4. Enforce single admission, queue bounds, trap/cancellation invalidation and
   delayed-settlement fencing. Never reuse an unsettled Wasm stack.
5. Run at least 10,000 real Roc suspensions in each compiler profile. Record
   exact module hashes, imports and guest/host memory measurements.
6. Run the identical artifact in pinned local workerd, then a bounded managed
   provider probe before making a hosted claim.[^platform][^shim][^qualification][^admission]

Initial local experiment: Roc sequentially called an ordinary synchronous
host effect, suspended through a timer import, then resumed and called another
effect. A retained context contains a heap string, a captured render closure
and an opaque resource. The shim's Wasm32 Box(U64) refcount placement was fixed
during development; final-reference resource drop is checked independently
from the allocation count. Stress and provider results are recorded separately
when available.[^platform][^shim][^qualification]

## Phase 2 microplan: unchanged C0

1. Stage the normal application plan and native generated route policy against
   the Worker platform; keep the frozen starter bytes unchanged.
2. Implement scalar Env, UTF-8 logical paths, logging, typed SQL and native
   Html/Datastar modules. Bind a declared singleton to one managed database.
3. Preserve full I64 values using typed parameter casts and a materialized
   result projection; reject unsupported statements/options and duplicate names.
4. Test documents, reset/increment fragments, ordinary versus Datastar headers,
   route failures and full workerd restart. Repeat the finite path hosted.

C0 passes locally and in the disposable hosted Worker. Its finite fragment
contains exactly one patch event and no live subscription. The same object
key retains count 1 after process restart locally and redeployment hosted.
No app codec, Actor extraction, custom route host or source edit is used.
The SQL profile checks metadata at execution rather than native prepare time;
that timing difference is explicit.[^packaging][^runtime][^evidence]

## Phase 3 microplan: transactions and async effects

1. Rendezvous an actual Roc transaction handle with `storage.transaction`.
   Keep the callback open until explicit finish or final-reference drop.
2. Test two-write rollback, explicit commit, drop, typed SQL error, trap,
   unsupported timer inside the scope, post-commit error and new-request reads.
3. Add bounded body/fetch imports with typed errors and cancellation; materialize
   SQL synchronously so no provider cursor survives suspension.
4. Test binary body limits, large chunks, manual redirects, timeout, oversized
   fetch/body and 100 cancelled actual Roc continuations with reconstruction.

The real workerd and hosted transaction scenarios pass. Teardown of the plain
transaction app returns zero live Roc allocations and zero host resources.
The actual async fixture passes 1 MiB binary body round-trip, checked excess,
200 KiB fetch chunks, redirect/timeout/size errors and 100 cancellations.
Body/fetch/timer effects inside open transactions are denied before starting.
No C2 source delta was needed. A delayed provider rollback test now fences
reconstruction after the Wasm stack settles; the next generation cannot join
the old transaction. Queue saturation rejects 17 of 50 concurrent calls while
33 remain admitted/queued, without leaking body resources. The 1,024-resource
bound and failed initialization quarantine are also checked.[^runtime][^evidence]

## Phase 4 microplan: original live handlers

1. Compile both frozen live versions through normal generated dispatch.
2. Preserve command 204/empty-SSE roles, live GET URLs, HTML, scoped CSS,
   Datastar and server-owned state. Add retry only to declared live GET actions.
3. Compare two subscribers under concurrent commands, idle polling, cancellation
   and a fresh reconnect. Validate two real browser clients on the same origin.
4. Run the same C2 stream against managed object SQL and await durability before
   each published event. Bound sources, queues, buffered events and stalled reads.

C1 and C2 pass two-reader/concurrent-command/reconnect tests locally; C2 passes
managed two-subscriber snapshots and updates. IAB and Safari displayed shared
count 23 with Oslo feed values during local browser validation. A later idle
regression exposed a ReadableStream pull bug: a Wait step must retain the pull
until data/end, rather than return without enqueue. The fixed local test waits
1.5 seconds before mutating; the hosted stream test passes with the fix.
Broad provider memory/residency economics and exhaustive replacement/stalled-reader
load are not inferred from these functional tests.[^runtime][^evidence]

## Phase 5 microplan: generated package

1. Add strict shared target config and stable provider identity validation;
   scaffold ordinary package scripts and Wrangler build hooks.
2. Embed platform/runtime sources, resolve a separately pinned compiler and
   versioned host, record integrity locks, source maps and an exact file graph.
3. Build into a fresh tree; validate imports/exports and atomically replace the
   output only on success. Cache only exact input/toolchain/output matches.
4. Test fresh paths with spaces, locked warm hits, arbitrary binary asset
   changes, Wasm tampering, compile failure preservation, npm install, normal
   dev/watch and a deploy dry run. Retain provider config/migrations unchanged.

The unrelated stateless starter passes that workflow. Fresh testing caught
and fixed invalid scaffold syntax and empty-context initialization (Roc's
zero-sized context is a null pointer, not an uninitialized instance). Root
watching also captured generated provider files and looped; the scaffold now
watches authored files/directories. New root files require a watch-list update
or a manual build/restart; nested watched directories discover new files.
Automatic download hosts are Apple Silicon macOS and x64 Linux. The release
workflow now packages/checksums the host, but installation from a published
release awaits publication. The lock retains compiler fingerprints per host
architecture; a new host needs one reviewed unlocked build. The final fresh
application test includes ordinary npm install/dev, source watch quiescence and
a dry deploy that verifies the exact Wasm bytes.[^config][^packaging][^guide][^evidence]

## Phase 6 microplan: editor, plain Roc and release tests

1. Stage the same compatible platform in LSP projections, use the build's exact
   compiler descriptor and preserve source offsets when qualifying imports.
2. Route mixed native/Worker workspaces to separate compiler children; preserve
   native compiler selection. Honor client workspace trust for semantic features.
3. Provide ordinary Roc hello, bounded fetch and SQL examples, plus typed JS
   adapter declarations and an explicit custom-entry/service-binding escape hatch.
4. Add one maintained application conformance runner and a hosted CI lane that
   tests and retains the exact artifacts; package the platform archive on release.
5. Update crate READMEs, CLI/config/runtime/handler/troubleshooting references and
   a starter guide; build and inspect the generated documentation.

The real Worker compiler child now returns hover using the same pin; file URLs
encode spaces correctly, and VS Code forwards workspace trust. Host analysis
continues without compiler trust. The offline workspace suite, workspace deps,
release artifact tests, full workspace clippy, Wasm-host clippy, Rust formatting,
VS Code lint/compile and docs build have passed locally. The generated guide's
navigation and layout were inspected. Hosted CI and Knowledge run IDs on the
implementation revision must be recorded before any phase-completion log.
The standalone release archive was extracted, compiled with plain Roc and run
with the packaged JS runtime, then torn down with zero live allocations or host
resources. Generated logs/traps identify the authored init/route/live declaration;
compile diagnostics retain source maps. Optimized instruction addresses are not
mapped to individual Roc expressions. The independent C0 native/Worker comparison
passed exact document/finite-SSE bytes after normalizing inline versus asset CSS
delivery; each original scoped block is present in the served asset.
No public release has been cut.[^editor][^packaging][^guide][^evidence]

## Phase 7 disposition and qualification limits

The plan defines this as optional follow-on work. No celld provider, remote DB,
WebSocket transport, pooling scheme or compiler fork is advertised. Those
alternatives remain exploratory and need the same independent conformance
and operating evidence if selected. This implementation does not turn native
filesystem, process or raw TCP effects into managed capabilities.[^plan][^runtime]

The real-Roc ABI matrix passed 10,000 suspensions for each specialized dev,
speed and size artifact on Node, local workerd and a hosted remote preview.
Each retained string/closure/resource survived and teardown reported zero guest
allocations. Local/hosted Wasm linear memory stayed at 9,961,472 bytes during
that qualifier. Experimental unspecialized lowering leaked four guest bytes
per call and is excluded. Node process memory samples are stored; they are not
an exhaustive whole-isolate continuation memory or long idle cost measurement.
The maintained Rust host also passed C0 and async/cancellation application
tests in development and size profiles, reusing the speed lane sources and
runtime harnesses. Cold Linux CI found a compiler installer write-handle leak;
the installer now closes the executable before validating it.
Application artifacts use the maintained Rust host, with its own transaction,
body/fetch, source, cleanup and actual-provider tests.[^qualification][^evidence]

[^plan]: Requested scope, constraints and phased acceptance criteria.
[^fixtures]: Exact source/dependency provenance and zero source edits.
[^source-test]: Offline source-integrity and existing-lowering verification.
[^platform]: Small ordinary Roc effectful qualification contract.
[^shim]: Qualification allocator and ownership implementation.
[^qualification]: Reproducible compiler/runtime inputs and measurement harness.
[^admission]: Single invocation ownership across suspension and cancellation.
[^runtime]: Maintained capability profile, independent from the actor POC.
[^packaging]: Shared ordinary lowering, generated lifecycle/routes and build commands.
[^config]: Explicit target-specific identity/storage validation.
[^editor]: Shared embedded platform sources, encoded paths and projection staging.
[^evidence]: Measured results with exact artifacts, pins, scope and limitations.
[^guide]: Public preview walkthrough and limitations beside the workflow.
