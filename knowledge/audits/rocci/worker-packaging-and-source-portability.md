---
type: Audit
title: Worker packaging and Rocci source portability
description: File-level review of the actor counter POC and the reusable platform work needed for low-effort Worker deployment.
tags: [domain/rocci, integration/roc, integration/datastar, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-10-19T14:42:11Z }
stale_after: 2027-01-19
authority: descriptive
owners: [human:nils]
sources:
  - id: implementation-status
    resource: ../../status/worker-source-portability.md
    title: Subsequent reusable Worker implementation and qualification evidence
  - id: implementation-plan
    resource: ../../plans/rocci/worker-packaging-and-source-portability.md
    title: Detailed custom-platform, JSPI and low-friction Worker implementation plan
  - id: celld-followup
    resource: ../../research/rocci/celld-runtime-alternatives.md
    title: celld alternatives and local JSPI comparison
  - id: plan
    resource: ../../plans/rocci/portable-actor-backends.md
    title: Selected small-host scope and unchanged-handler alternative
  - id: status
    resource: ../../status/portable-actor-backends.md
    title: Dated local and hosted results; remaining gates
  - id: native
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Current native lifecycle, SQL and handlers
  - id: core
    resource: ../../../examples/rocci/standalone/live-counter/CounterCore.roc
    title: Shared pure decisions and view construction
  - id: document
    resource: ../../../examples/rocci/standalone/live-counter/CounterDocument.rocci
    title: Shared document and remote transport selection
  - id: ui
    resource: ../../../examples/rocci/standalone/live-counter/LiveCounterUi.rocci
    title: Shared components and remote command expressions
  - id: actor
    resource: ../../../examples/rocci/remote/live-counter/Actor.roc
    title: Private application-specific value and rendering entry point
  - id: platform
    resource: ../../../examples/rocci/remote/live-counter/platform/main.roc
    title: Private Roc value platform and targets
  - id: shim
    resource: ../../../examples/rocci/remote/live-counter/platform/host.c
    title: Roc ABI, invocation arena and native parity executable
  - id: inputs
    resource: ../../../crates/rocci-cli/examples/actor_inputs.rs
    title: Private hardcoded component lowering and staging
  - id: host
    resource: ../../../examples/rocci/remote/live-counter/src/index.ts
    title: Ingress, actor SQL, receipts, lifecycle and operator routes
  - id: auth
    resource: ../../../examples/rocci/remote/live-counter/src/auth.ts
    title: Access JWT and same-origin checks
  - id: protocol
    resource: ../../../examples/rocci/remote/live-counter/src/protocol.ts
    title: Counter-specific schema, bounds and validation
  - id: roc
    resource: ../../../examples/rocci/remote/live-counter/src/roc.ts
    title: Counter-specific wire codec and Wasm instance management
  - id: streams
    resource: ../../../examples/rocci/remote/live-counter/src/streams.ts
    title: Stream scheduling, ordering, backpressure and ownership fence
  - id: build
    resource: ../../../examples/rocci/remote/live-counter/tools/build.mjs
    title: Independent Wasm/native build and manifest
  - id: configure
    resource: ../../../examples/rocci/remote/live-counter/tools/configure.mjs
    title: Environment parsing and generated provider configuration
  - id: package
    resource: ../../../examples/rocci/remote/live-counter/tools/package.mjs
    title: Wrangler dry-run bundle and hashes
  - id: fingerprint
    resource: ../../../examples/rocci/remote/live-counter/tools/fingerprint.mjs
    title: Input and artifact drift detection
  - id: test
    resource: ../../../examples/rocci/remote/live-counter/tools/test.mjs
    title: Conformance orchestration and tested artifact stamp
  - id: deploy
    resource: ../../../examples/rocci/remote/live-counter/tools/deploy.mjs
    title: Checked no-rebuild Wrangler deployment
  - id: dev
    resource: ../../../examples/rocci/remote/live-counter/tools/dev.mjs
    title: Local Wrangler development wrapper
  - id: native-control
    resource: ../../../examples/rocci/remote/live-counter/tools/native-control.mjs
    title: Opt-in native HTTP and rollback smoke
  - id: resources
    resource: ../../../examples/rocci/remote/live-counter/tools/resources.mjs
    title: Opt-in local workerd resource workload
  - id: abi-test
    resource: ../../../examples/rocci/remote/live-counter/test/abi.test.mjs
    title: Private-platform native/Wasm parity and memory tests
  - id: host-test
    resource: ../../../examples/rocci/remote/live-counter/test/host.test.mjs
    title: Host validation, JWT and stream tests
  - id: browser-test
    resource: ../../../examples/rocci/remote/live-counter/test/browser-adapter.test.mjs
    title: Retry-envelope browser adapter tests
  - id: runtime-test
    resource: ../../../examples/rocci/remote/live-counter/test/runtime.test.mjs
    title: Local workerd SQL, recovery and authorization tests
  - id: fixture
    resource: ../../../examples/rocci/remote/live-counter/test/fixture.ts
    title: Local identity and failure-injection entry point
  - id: support
    resource: ../../../examples/rocci/remote/live-counter/test/support.mjs
    title: Node test bundling and Wasm loader
  - id: remote
    resource: ../../../examples/rocci/remote/live-counter/assets/remote.js
    title: Pending operation IDs and retry transport
  - id: verify
    resource: ../../../examples/rocci/remote/live-counter/assets/verify.js
    title: Authenticated hosted acceptance and operator helpers
  - id: verify-page
    resource: ../../../examples/rocci/remote/live-counter/assets/verify-page.js
    title: Acceptance-page UI wiring
  - id: package-json
    resource: ../../../examples/rocci/remote/live-counter/package.json
    title: Pinned runtime and development dependencies
  - id: native-platform
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Native lifecycle exports and hosted effects
  - id: sqlite
    resource: ../../../crates/rocci-platform/platform/Sqlite.roc
    title: Native pool and pinned transaction API
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Generated native routes and per-connection polling
  - id: wasi
    resource: ../../../crates/rocci-wasi-http/README.md
    title: Experimental WASI HTTP target and limitations
  - id: cf-wasm
    resource: https://developers.cloudflare.com/workers/runtime-apis/webassembly/
    title: Core Wasm support and partial experimental WASI
  - id: cf-wasm-imports
    resource: https://developers.cloudflare.com/workers/runtime-apis/webassembly/javascript/
    title: Worker Wasm calling imported JavaScript functions
  - id: cf-sql
    resource: https://developers.cloudflare.com/durable-objects/api/sqlite-storage-api/
    title: Synchronous SQL, callback transactions and numeric precision
  - id: cf-bundle
    resource: https://developers.cloudflare.com/workers/wrangler/bundling/
    title: Wrangler bundling and separate Wasm modules
  - id: cf-build
    resource: https://developers.cloudflare.com/workers/wrangler/custom-builds/
    title: Custom build hooks for dev and deploy
  - id: cf-lifecycle
    resource: https://developers.cloudflare.com/durable-objects/concepts/durable-object-lifecycle/
    title: Activation, hibernation and absence of shutdown hooks
  - id: cf-websocket
    resource: https://developers.cloudflare.com/durable-objects/best-practices/websockets/
    title: WebSocket hibernation as a different transport option
---

# Worker packaging and Rocci source portability

## Finding and evidence boundary

This audit preserves the pre-implementation findings at the reviewed revision
below. Subsequent reusable platform changes and dated execution evidence are
tracked separately; the POC inventory is not a description of the new Worker
package.[^implementation-status]

Low-effort deployment is a plausible product direction for a declared subset
of Roc/Rocci capabilities. The existing POC proves a smaller claim: shared
pure Roc decisions and compiled Rocci components run inside a Durable Object.
It does not compile the existing native app's effectful routes for Workers.
The selected plan explicitly excluded that platform port.[^plan][^actor][^host]

Reviewed checkout: `bbf7b1effb4bbeca6fb1548ac119e82c20435622`, with a clean
working tree before this audit. Compared the native example against
`e6467a1b^`, before the POC implementation. This audit inspects source and
primary provider documentation; it does not rerun deployment or acceptance.
The dated status record contains earlier execution evidence and remaining
gates; its pending hosted entries must not be treated as current provider
state or silently marked complete.[^status]

The recommendation below is exploratory engineering judgment, not an
approved public platform contract or an implementation plan being executed.

The paired [implementation plan](/plans/rocci/worker-packaging-and-source-portability.md)
now specifies unchanged starter `Counter.rocci` as the first golden target,
followed by original live handlers, a custom Roc Worker platform, real Roc
JSPI qualification, transaction compatibility experiments and ordinary
Wrangler development/deployment. All phases remain planned.[^implementation-plan]

The follow-up [celld comparison](/research/rocci/celld-runtime-alternatives.md)
records successful local JSPI suspension probes on both celld and workerd.
JSPI is a concrete additional route for asynchronous hosted effects; a real
Roc effect ABI and lifetime behavior remain to be proven. It also examines
celld's internal native-engine and replication-library seams.[^celld-followup]

## What every supporting file does

All names in the following tables are relative to
`examples/rocci/remote/live-counter/`, unless another path is shown.
`.mjs` selects JavaScript's ES-module format for Node scripts/tests.
TypeScript is host source; Wrangler produces the deployed JavaScript.
The number of source files does not imply that many deployed services.

### Runtime TypeScript

| File | Current responsibility | Appropriate reusable ownership |
| --- | --- | --- |
| `src/index.ts` | Authenticated ingress, assets, counter-key routing, Durable Object activation, SQL, receipts, durability, command responses, manifest and export/import routes. | Split generic provider runtime from app policy and generated routing. Counter tables, exactly two allowed keys, schema-2 drill and operator format are POC-specific. [^host] |
| `src/auth.ts` | Access JWT validation and same-origin mutation checks. | Reusable optional Access integration; owner identity/hostname are deployment policy, not a universal Roc requirement. [^auth] |
| `src/roc.ts` | Instantiate Wasm, copy bounded bytes, recover from traps, collect arena usage; encode/decode the counter wire format. | Generic Wasm runner plus generated app codec. Current state/command/HTML-target assumptions prevent calling this a general SDK. [^roc] |
| `src/protocol.ts` | Validate counter commands/state, exact integers, UUIDs, body sizes and retry digest. | Generic transport primitives plus generated or Roc-owned application schema; counter/feed types and caps remain app policy. [^protocol] |
| `src/streams.ts` | Datastar framing, subscribe/snapshot ordering, coalescing, cleanup, heartbeat, presentation refresh and replacement ownership checks. | Reusable live-delivery runtime with explicit refresh policy. [^streams] |
| `src/wasm.d.ts` | One ambient type declaration for imported Wasm modules. | SDK/build template; no runtime code. |

### Build, development and deployment JavaScript

| File | Motivation | Packaging judgment |
| --- | --- | --- |
| `tools/build.mjs` | Lower shared templates, stage pure modules/assets, compile the C shim and Roc for Wasm and native parity, check pins/imports, hash inputs, replace successful output safely. | Most is missing Rocci target support; move product build work into the CLI. Native parity build is SDK verification rather than a required app deployment step. [^build] |
| `tools/configure.mjs` | Read local settings and generate hosted/local Wrangler configurations without embedding deployment credentials. | Replace most example-local logic with ordinary provider config and reusable config generation; fixed rocci.dev hostname and two-key policy are POC constraints. [^configure] |
| `tools/package.mjs` | Invoke Wrangler dry-run to capture the upload bundle and hash it. | Thin artifact-verification wrapper; Wrangler already bundles. [^package] |
| `tools/fingerprint.mjs` | Detect source/config/asset/package drift after build or test. | SDK/CI release integrity policy, not per-app authored code. [^fingerprint] |
| `tools/test.mjs` | Run conformance and stamp the exact tested artifact. | Shared SDK test/release workflow. [^test] |
| `tools/deploy.mjs` | Check settings, hashes and test stamp; call Wrangler with `--no-bundle` and immediate actor code update. | Optional SDK release wrapper around Wrangler. Preserving tested-artifact deployment still needs this check somewhere. [^deploy] |
| `tools/dev.mjs` | Generate local config and run loopback Wrangler with the identity fixture. | Mostly ordinary `wrangler dev` plus a shared local-auth policy. [^dev] |
| `tools/native-control.mjs` | Opt-in native HTTP/SQLite smoke and injected rollback check. | POC/SDK conformance evidence; not deployed or required in each app. [^native-control] |
| `tools/resources.mjs` | Opt-in inspector workload with two full objects and stalled subscribers. | POC/SDK measurement tooling; not app runtime. [^resources] |

### Tests and hosted verification

| File | Motivation | Packaging judgment |
| --- | --- | --- |
| `test/abi.test.mjs` | Real decisions/rendering, Unicode/bounds/traps, native/Wasm parity and 10,000 calls. | SDK/compiler conformance plus app fixtures. Its native executable uses the same private platform, not the untouched native HTTP server. [^abi-test] |
| `test/host.test.mjs` | Envelope/JWT checks and stream ordering, backpressure, cleanup and replacement fencing. | Shared runtime tests. [^host-test] |
| `test/browser-adapter.test.mjs` | One operation ID per intent, stable retries and ambiguous reload behavior. | Shared browser transport tests. [^browser-test] |
| `test/runtime.test.mjs` | Actual local workerd SQL, faults, fan-out, reconstruction, restore, upgrades, caps and production authorization denial. | SDK integration suite with application fixtures. [^runtime-test] |
| `test/support.mjs` | Bundle TS helpers for Node tests and load Wasm there. | Test infrastructure only. [^support] |
| `test/fixture.ts` | Explicit loopback identity and response-loss injection; exports the real actor. | Local test entry point, excluded as the production entry point. [^fixture] |
| `assets/verify.js` | Owner-session concurrency, retry, SSE, reconnect, idle and export/restore helpers. | Acceptance/operator tooling; not necessary for the counter UI. [^verify] |
| `assets/verify-page.js`, `assets/verify.html` | Buttons, results and file selection for those helpers. | Optional diagnostic app; currently shipped as authenticated assets. [^verify-page] |

### Other resources that need an explanation

| Resource | Why it exists | Reuse judgment |
| --- | --- | --- |
| `assets/remote.js` | Generate a UUID per intent, retain it across retries and display definite/ambiguous failures. It keeps request metadata, not canonical counter state. | Needed for the chosen retry guarantee, not for basic Workers execution. Generalize as Rocci transport support so UI source need not call `window.rocciCounter`. [^remote] |
| `Actor.roc` | Decode a private line/hex protocol, call shared decisions/view construction and render document or fragment. | App-specific adapter caused by the pure-core boundary. A general platform should generate codecs/dispatch or expose normal handlers instead. [^actor] |
| `platform/main.roc` | Require one `Str -> Str` function and expose callable Wasm/native targets. | Reusable Roc value platform after removing private-counter assumptions. [^platform] |
| `platform/host.c` | Roc allocation/crash/memory ABI, bounded per-call arena, byte exports and native parity executable. | Compiler/platform glue; prebuild or ship once in an SDK. Its fixed arena and string-layout knowledge are private implementation choices requiring compiler-pin coverage. [^shim] |
| `crates/rocci-cli/examples/actor_inputs.rs` | Hardcode the two shared UI modules, lower them and stage pure Html/Datastar/Origin/core plus CSS. | Clear missing CLI build-target functionality. It currently uses the Rocdown pure string Html runtime, not the native Node Html implementation. [^inputs] |
| `package.json`, lockfile | Runtime `jose` JWT library; development Wrangler, TypeScript and Worker types; exact dependency resolution. | Reusable SDK dependencies. A global Wrangler alone does not supply the runtime JWT library or make app-specific host code generic. [^package-json] |
| `tsconfig.json`, `src/wasm.d.ts` | Type-check the host, including imported JSON/Wasm; emit no TS build output through tsc. | Shared SDK configuration. |
| `.env.example`, local `.env`, generated Wrangler config | Account/domain/access policy, stable namespace identity, keys and schema selection. | Some deployment configuration is unavoidable; most private experiment defaults are not universal. [^configure] |
| Datastar JS, extracted CSS | Browser hypermedia transport and styles. | Normal application assets, staged automatically by build tooling. [^build][^inputs] |
| `dist/`, `test-dist/`, `.wrangler/` | Compiled app/bundle, manifests, temporary test modules, local provider state/logs/evidence. | Derived outputs, not additional authored app source. [^build][^support][^resources] |
| README and recorded metrics | Reproducibility, acceptance and operating instructions. | Keep detailed SDK/operator guidance centrally; per-app docs need only specific policy and deployment choices. [^status] |

### What assuming Wrangler changes

The POC already requires a pinned local Wrangler and invokes it for development,
bundling and deployment. Assuming `wrangler` on PATH mostly changes executable
selection and dependency installation. It does not add Roc compilation, a Wasm
ABI, SQL adaptation or live-stream semantics.[^package-json][^dev][^package][^deploy]

A simpler app-facing workflow can use one ordinary Wrangler config and its
custom build hook to invoke a future Rocci Worker build. Wrangler already
supports bundling and separate Wasm modules. This can remove several wrappers
from individual apps. The exact build/test/deploy integrity policy can remain
in shared SDK/CI tooling. Wrangler availability alone does not provide that
policy.[^cf-bundle][^cf-build][^fingerprint]

## Why the counter changed

The main reason was the chosen experiment boundary, not a markup or Roc language
restriction. The Wasm module deliberately has no imports: it cannot call the
native `pf.Sqlite`, request-body or logging effects. The TypeScript host owns
storage and invokes a synchronous pure decision/render function.[^plan][^build][^actor]

Import-free Wasm is a POC simplification, not a Workers requirement. Workers
can supply imported JavaScript functions to Wasm; a compatible SQL host could
use that mechanism rather than requiring pure state extraction in every
application. Implementing its Roc ABI and transaction semantics remains work
to prove.[^cf-wasm-imports][^cf-sql]

Comparing `e6467a1b^` with the reviewed checkout shows five native-example files
changed, with 227 insertions and 225 deletions overall. Much of the visible
movement is extracting the document/CSS from `LiveCounter.rocci` into
`CounterDocument.rocci`. The meaningful changes are:

- Extract decisions, retained click state and supplied-time view construction
  into `CounterCore.roc`, because the Worker does not call native handlers.
  Move rendering out of the effectful entry module so the pure build does not
  import its native platform dependencies.[^core][^document][^inputs]
- Add a monotonic revision, coherent SQL reads and atomic count/feed mutation.
  These are correctness improvements and support the selected ordering/recovery
  guarantees; they are not prerequisites for compiling templates to Wasm.
  The pre-POC increment used separate statements.[^native][^plan]
- Add `remote` selection to the shared document/controls, explicit subscription,
  actor-relative URLs, an extra script and status element. This compensates for
  the separately authored Worker routes and UUID retry transport. A shared
  runtime/generator should own most of it.[^document][^ui][^remote]
- In the Worker, persist a bounded JSON aggregate and receipts rather than
  porting the native counter/click SQL API. That is a storage-model choice;
  Workers does not require the application to become a JSON state machine.
  The POC's safe-integer range and caps are also selected contracts, not Roc
  integer limits.[^host][^protocol][^core]

The original native action URLs and island call shape are retained through
defaults/wrappers. Source reuse has been achieved for domain and rendering,
not for effectful application orchestration.[^native][^ui][^actor]

## What actually limits source compatibility

| Boundary | Assessment |
| --- | --- |
| Roc execution and template language | Core Wasm execution works in this experiment. Existing component lowering is reused; no new syntax was necessary. There is no demonstrated fundamental language blocker for this counter. [^actor][^inputs][^status] |
| Compiler/runtime maturity | The build requires a separate October 9 Roc pin because the root native pin traps on this Wasm workload. The private shim knows Roc string layout and uses an arena. This is real toolchain/ABI maintenance work, not a permanent argument against deployment. [^build][^shim][^status] |
| Native platform | The ordinary platform exposes process init/respond/SSE/shutdown and many OS effects, and has no Workers target. Compiling the app to Wasm does not implement those host functions. [^native-platform] |
| SQLite | Object SQL is synchronous, making imported host calls plausible. Native pools, prepared handles, exact value codecs and linear transaction handles still need adaptation. Cloudflare disallows SQL BEGIN/SAVEPOINT and requires callback transactions; mapping native begin/commit literally is not sufficient. Its SQL numeric boundary also needs explicit precision handling. [^sqlite][^cf-sql] |
| Asynchronous effects | The counter's small storage/render path can remain synchronous after request bytes are buffered. Arbitrary outbound fetch, streamed input, sleep and other async effects cannot return a JS Promise into an ordinary synchronous Wasm import and pretend the Roc stack suspended. A general platform needs explicit suspension/continuations, restructuring, or a declared unsupported capability. [^native-platform][^wasi] |
| Lifecycle and identity | A process DB path is not an actor key. Durable storage survives activation; the Wasm heap/context/handles must be reconstructed. Init must be repeatable and state must not rely on shutdown flush. Object-key selection is an application decision that tooling cannot reliably infer. [^native][^host][^cf-lifecycle] |
| Live delivery | The native generator polls/re-renders per connection every 100ms. A Worker target can keep the declaration but needs scheduling/notification semantics; generic change-on-command is insufficient for external changes or time-driven views unless covered by a refresh policy. [^dispatch][^streams] |
| WASI | The existing experimental WASI 0.3 HTTP component is not a ready Workers artifact. Cloudflare documents only partial experimental WASI. Its local success is useful prior work, not a shortcut around a Worker host. [^wasi][^cf-wasm] |

SSE with the current active timers does not gain WebSocket hibernation by being
packaged differently. This is a provider lifecycle/transport tradeoff relevant
to large numbers of idle objects. Cloudflare has a distinct hibernating
WebSocket API; adopting it would require a deliberate browser transport
adaptation and can still carry server-rendered HTML.[^streams][^cf-lifecycle][^cf-websocket]

## Recommended reusable product boundary

For the stated goal of writing the existing counter largely as-is, package a
supported Worker platform and generated dispatcher. Only moving the current
TypeScript files into a library would leave applications authoring
`CounterCore`-style state machines, custom codecs and a second set of routes.

Suggested responsibilities, all proposed:

1. A small Roc-level Worker ABI/runtime independent of Rocci: allocation,
   strings/bytes, errors, lifecycle and host-capability binding. Start with
   bounded synchronous calls; do not claim arbitrary native effect portability.
2. A Cloudflare adapter for identity-to-object routing, object SQL, request/
   response mapping, asset serving and stream lifecycle. An ordinary stateless
   Worker should not require an actor; opt into Durable Objects for shared
   state/coordination.
3. A Rocci target reusing the current parser/lowering, handler roles,
   components, Datastar expressions and CSS extraction. Compile handlers
   against a compatible supported `pf` surface. Generate entry points and
   codecs; keep provider logic outside template parsing.
4. Generic command IDs/receipts/retry UI and live notification where those
   guarantees are selected. Put limits, access policy, refresh behavior,
   schema/migration policy and actor-key selection in explicit configuration
   or application code. Do not hardcode the POC's counter assumptions.
5. Use Wrangler for provider packaging/dev/deploy; let Rocci own Roc compilation
   and asset staging. SDK tests and release fingerprints should be shared,
   with application-specific fixtures kept small.

The first compatibility proof should compile the pre-POC counter's actual
`@init`, SQL helpers and routes against that subset. Preserve normal URLs,
ordinary `@post` controls and `@get:live` authoring. Explicitly prove a multi-
statement SQL rollback and repeat activation before promising compatibility.
Transaction scope may require a small shared callback-style helper or a
documented command transaction boundary; hiding incompatible linear
begin/commit semantics would be misleading.

The success criterion is an application with Roc/Rocci source, assets and a
small deployment/object-binding configuration, requiring no handwritten
Worker host, C shim, wire codec or per-app build/test/deploy scripts.
The SDK implementation is substantial platform work once; deploying another
supported app should be routine. Preserving every existing native effect and
its exact behavior remains a much larger, sometimes impossible compatibility
promise for this host.

## Sources and verification

[^implementation-plan]: Exploratory implementation follow-up; no phase or deployment was executed to write it.

[^celld-followup]: celld alternatives and local JSPI comparison.

[^plan]: Selected small-host scope and unchanged-handler alternative.
[^status]: Dated local and hosted results; remaining gates.
[^native]: Current native lifecycle, SQL and handlers.
[^core]: Shared pure decisions and view construction.
[^document]: Shared document and remote transport selection.
[^ui]: Shared components and remote command expressions.
[^actor]: Private application-specific value and rendering entry point.
[^platform]: Private Roc value platform and targets.
[^shim]: Roc ABI, invocation arena and native parity executable.
[^inputs]: Private hardcoded component lowering and staging.
[^host]: Ingress, actor SQL, receipts, lifecycle and operator routes.
[^auth]: Access JWT and same-origin checks.
[^protocol]: Counter-specific schema, bounds and validation.
[^roc]: Counter-specific wire codec and Wasm instance management.
[^streams]: Stream scheduling, ordering, backpressure and ownership fence.
[^build]: Independent Wasm/native build and manifest.
[^configure]: Environment parsing and generated provider configuration.
[^package]: Wrangler dry-run bundle and hashes.
[^fingerprint]: Input and artifact drift detection.
[^test]: Conformance orchestration and tested artifact stamp.
[^deploy]: Checked no-rebuild Wrangler deployment.
[^dev]: Local Wrangler development wrapper.
[^native-control]: Opt-in native HTTP and rollback smoke.
[^resources]: Opt-in local workerd resource workload.
[^abi-test]: Private-platform native/Wasm parity and memory tests.
[^host-test]: Host validation, JWT and stream tests.
[^browser-test]: Retry-envelope browser adapter tests.
[^runtime-test]: Local workerd SQL, recovery and authorization tests.
[^fixture]: Local identity and failure-injection entry point.
[^support]: Node test bundling and Wasm loader.
[^remote]: Pending operation IDs and retry transport.
[^verify]: Authenticated hosted acceptance and operator helpers.
[^verify-page]: Acceptance-page UI wiring.
[^package-json]: Pinned runtime and development dependencies.
[^native-platform]: Native lifecycle exports and hosted effects.
[^sqlite]: Native pool and pinned transaction API.
[^dispatch]: Generated native routes and per-connection polling.
[^wasi]: Experimental WASI HTTP target and limitations.
[^cf-wasm]: Core Wasm support and partial experimental WASI.
[^cf-wasm-imports]: Worker Wasm calling imported JavaScript functions.
[^cf-sql]: Synchronous SQL, callback transactions and numeric precision.
[^cf-bundle]: Wrangler bundling and separate Wasm modules.
[^cf-build]: Custom build hooks for dev and deploy.
[^cf-lifecycle]: Activation, hibernation and absence of shutdown hooks.
[^cf-websocket]: WebSocket hibernation as a different transport option.


[^implementation-status]: Later implementation microplans, artifacts and qualification limits.
