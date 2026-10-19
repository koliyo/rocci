---
type: Status
title: Worker packaging implementation
description: Experimental Cloudflare Worker packaging, supported runtime contracts and maintained verification using canonical applications.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T15:34:06Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/rocci/worker-packaging-and-source-portability.md
    title: Requested implementation scope and phase boundaries
  - id: cli
    resource: ../../crates/rocci-cli/src/worker.rs
    title: Public commands and ordinary application staging
  - id: config
    resource: ../../crates/rocci-core/src/worker.rs
    title: Explicit deployment profile and identity configuration
  - id: runtime
    resource: ../../crates/rocci-worker-platform/README.md
    title: Supported capabilities, bounds and distribution
  - id: host
    resource: ../../crates/rocci-worker-platform/src/host.rs
    title: Maintained Roc Wasm ABI and persistent ownership
  - id: examples
    resource: ../../crates/rocci-cli/tests/worker_examples.rs
    title: Canonical example staging and source preservation regression
  - id: applications
    resource: ../../crates/rocci-worker-platform/tools/applications.mjs
    title: Maintained real Roc/workerd application suite
  - id: editor
    resource: ../../crates/rocci-lsp/src/projection_workspace.rs
    title: Shared Worker compiler/platform editor profile
  - id: ci
    resource: https://github.com/koliyo/rocci/actions/runs/39272312080
    title: Original runtime implementation passed all seven hosted CI jobs
  - id: knowledge-ci
    resource: https://github.com/koliyo/rocci/actions/runs/39274065403
    title: Original qualification receipt passed Knowledge validation
  - id: qualification
    resource: https://github.com/koliyo/rocci/blob/db3a64faa1d04eb649224a6da5897d18202d811a/knowledge/status/worker-source-portability-evidence.json
    title: Immutable original local, hosted and CI measurements
  - id: guide
    resource: ../../docs/applications/workers.rocdown
    title: Public application workflow and limitations
---

# Worker packaging implementation

`rocci worker init` and `rocci worker build` package ordinary Roc programs and
Rocci applications for Cloudflare Workers. Generated hosting supplies the
platform, dispatch, lifecycle handling, runtime, Datastar/CSS/assets, source
maps and an integrity manifest. Authored application code uses the ordinary
Roc/Rocci APIs. Provider identity, migrations and secret bindings remain
explicit deployment configuration.[^cli][^config][^guide]

The maintained implementation has one Wasm host in Rust. Its application
suite compiles the existing standalone counter and live-counter examples,
the plain Roc SQL example and a focused async regression program. There are
no copied counter versions or a second prototype ABI in the product tree.
An offline regression checks generated lifecycle/routes and verifies that
staging preserves the canonical example sources.[^host][^examples][^applications]

## Supported runtime profile

The qualified compiler is `nightly-2026-10-09-258ab27`, independently of the
native compiler pin. Builds require specialization and support development,
speed and size optimization. Automatic compiler downloads support Apple
Silicon macOS and x64 Linux. Provider inputs are Wrangler 4.151.1, workerd
1.20261012.1 and compatibility date 2026-10-19.[^runtime]

The host preserves context, closures and resources with persistent ARC.
Single-invocation admission bounds queues; traps, cancellation and timeout
quarantine the generation. Replacement waits for both the suspended stack
and any provider transaction cleanup to settle. SQL preserves full-range I64,
REAL, text, BLOB and NULL distinctions. Transaction commit awaits the provider
callback; rollback and final-reference drop discard uncommitted writes.
Body, fetch and timer effects inside an open transaction fail before starting.
Responses and published events await managed storage durability.[^host][^runtime]

Bodies, outbound responses, SQL materialization, resources, invocation queues,
SSE sources, buffered events and async deadlines have explicit bounds. Native
filesystem/process/socket facilities, nested or multi-object transactions,
Windows toolchain downloads and named Wrangler environments are unsupported.
Provider lifecycle does not promise a shutdown callback on eviction. Broad
provider residency, continuation memory and idle cost qualification remain
outside the current experimental claim.[^runtime][^guide]

## Phase microplans and delivered boundaries

Each phase retains its own implementation and acceptance boundary from the
requested plan. Optional additional providers and economics work remain
follow-on scope.[^plan]

| Phase | Microplan | Delivered boundary |
| --- | --- | --- |
| 0 | Establish source compatibility, commands, capabilities and ownership. | Canonical application sources, strict target configuration and explicit provider identity/storage ownership. |
| 1 | Qualify real Roc suspension and persistent ownership before general async hosting. | One maintained Wasm host, ARC/finalizers, bounded admission, generation fencing and three specialized optimization profiles. |
| 2 | Execute ordinary counter initialization, SQL and finite handlers. | Existing counter runs through generated dispatch, with native response parity and durable reconstruction. |
| 3 | Preserve transaction boundaries and request lifetimes across async effects. | Provider transaction rendezvous, failure visibility, body/fetch bounds, cancellation and delayed rollback fencing. |
| 4 | Preserve ordinary live handlers and browser behavior. | Existing live-counter, concurrent commands, two readers/browsers, idle polling, reconnect and bounded delivery. |
| 5 | Turn hosting into a routine generated package. | npm/Wrangler build/dev/deploy integration, integrity locks, atomic output, exact artifact graph and independent starter. |
| 6 | Align editors, source diagnostics, plain Roc adoption and release tests. | Shared compiler/platform projection, declaration locations, independent platform/runtime archives, public docs and artifact conformance CI. |

## Maintained verification

`tools/applications.mjs` stages and compiles each canonical application once,
then exercises those exact modules on local workerd. Focused tests cover typed
SQL, multi-write commit/rollback/drop, typed errors, traps, initialization
failure, resource/queue saturation, body/fetch bounds and cancellation. The
live test covers ordinary command/empty-SSE roles, two subscribers, concurrent
writes, idle refresh and reconnect. `tools/profiles.mjs` reuses the counter
and async programs for development and size optimization.[^applications][^runtime]

The public packaging test checks an unrelated starter, path spaces, locked
warm builds, binary assets, tampered output, failure preservation, normal npm
dev/watch and a deploy dry run. Standalone archive verification extracts the
platform, builds plain Roc directly and runs the packaged JS adapter. Editor
verification exercises the exact Worker compiler child; native compiler
selection and workspace trust are preserved.[^runtime][^editor]

Original qualification passed 10,000 real Roc suspensions for each specialized
optimization mode on Node, local workerd and an authenticated hosted preview.
Disposable hosted application acceptance passed exact SQL/transactions,
live subscribers and durable reconstruction; its temporary guarded Worker
and three test namespaces were deleted. Guest allocation and linear-memory
measurements, Node memory samples, module hashes and scope limits are archived
at an immutable revision. The temporary compiler probe, historical application
snapshots and generated measurement dump are not maintained product files.
Those measurements do not establish exhaustive provider memory/economics.[^qualification]

Runtime implementation `0e1b5f311a89ba6d8c3bea52e68e5ad914c0f2c8` passed all
seven hosted CI jobs: Linux/macOS workspace tests, native Roc tests, editors,
lints, docs/fixtures and Worker artifact conformance. Its knowledge receipt
also passed. These are the original qualification runs; current tests use the
canonical applications described above. No public release or production
promotion is claimed.[^ci][^knowledge-ci]

[^plan]: Requested phase goals, constraints and optional follow-on boundary.
[^cli]: Public commands, normal lowering and deterministic staging.
[^config]: Target-specific configuration and stable deployment identity.
[^runtime]: Capability limits, ownership and maintained verification commands.
[^host]: Single maintained application ABI and resource implementation.
[^examples]: Offline staging regression using existing applications directly.
[^applications]: Actual application compilation and workerd execution.
[^editor]: Matching compiler/platform projection and source offsets.
[^ci]: Successful original full implementation CI on the named revision.
[^knowledge-ci]: Successful original qualification receipt validation.
[^qualification]: Historical measured evidence retained in Git, with explicit limits.
[^guide]: Public first-use workflow, supported hosts and current restrictions.
