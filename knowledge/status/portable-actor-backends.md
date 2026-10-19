---
type: Status
title: Private Rocci durable-actor backend results
description: Dated implementation and acceptance evidence for the private Cloudflare live-counter experiment, with remaining hosted gates explicit.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/persistence, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T10:55:00Z }
stale_after: 2026-11-19
authority: descriptive
owners: [human:nils]
sources:
  - id: plan
    resource: ../plans/rocci/portable-actor-backends.md
    title: Portable actor backend implementation plan
  - id: core
    resource: ../../examples/rocci/standalone/live-counter/CounterCore.roc
    title: Shared pure Roc decisions and view values
  - id: native
    resource: ../../examples/rocci/standalone/live-counter/LiveCounter.rocci
    title: Native transactional control
  - id: host
    resource: ../../examples/rocci/remote/live-counter/src/index.ts
    title: Durable actor storage, lifecycle and operator routes
  - id: streams
    resource: ../../examples/rocci/remote/live-counter/src/streams.ts
    title: Bounded snapshot delivery and viewer cleanup
  - id: build
    resource: ../../examples/rocci/remote/live-counter/tools/build.mjs
    title: Pinned independent native and core-Wasm build
  - id: abi
    resource: ../../examples/rocci/remote/live-counter/test/abi.test.mjs
    title: Native/Wasm parity and repeated-call conformance
  - id: runtime
    resource: ../../examples/rocci/remote/live-counter/test/runtime.test.mjs
    title: Actual workerd SQL, faults, streams, restore and upgrade tests
  - id: resource-probe
    resource: ../../examples/rocci/remote/live-counter/tools/resources.mjs
    title: Local workerd V8 resource workload
  - id: browser
    resource: ../../examples/rocci/remote/live-counter/assets/verify.js
    title: Owner-browser hosted conformance and idle scenarios
  - id: guide
    resource: ../../examples/rocci/remote/live-counter/README.md
    title: Manual Cloudflare setup, deployment and recovery runbook
  - id: auth
    resource: ../../examples/rocci/remote/live-counter/src/auth.ts
    title: Verified Access identity and same-origin checks
---

# Private Rocci durable-actor backend results

## Disposition

The small host implementation is present on `codex/portable-actor-backends`.
This is a private example, not a shipped general platform target. It executes
real Roc decisions and lowered Rocci components in core Wasm inside each
Durable Object; SQL, identity and transport remain in the host. The native
example consumes the same core and document/components and now transacts
count, revision and feed together. The larger unchanged-effectful-handler
port and optional celld deployment were not undertaken.[^plan][^core][^native][^host]

| Gate | Observed status on October 19 |
| --- | --- |
| Baseline / independent ABI | Native HTTP control, native/Wasm byte parity, malformed/trap/overflow cases and 10,000 full-feed calls passed locally. |
| Actual remote Roc execution | Authenticated Safari loaded the hosted document and incremented 0 → 1 with an Oslo feed row and live patch. |
| Durable mutations | Local workerd fault/reconstruction suite and hosted 100-concurrent-command exercise passed. |
| Two browser profiles | Safari and the in-app browser authenticated independently; increments propagated in both directions. |
| Idle and deployment reconstruction | Ten-minute hosted idle check running; version-drain/reconstruction drill still pending. |
| Restore / compatible rollback | Passed in separate local workerd namespaces; hosted separate-namespace drill and schema/config rollback pending. |
| Required CI / Knowledge | Local workspace and knowledge validation are separate from required hosted runs; hosted run IDs pending. |

The scenario definitions and limits are executable in the linked ABI,
workerd and owner-browser suites; this table does not substitute local
results for hosted exits.[^abi][^runtime][^browser]

## Inputs and artifacts

The tested source baseline was `97227bf514da4b5b08fe092db687c2d53141f77a`
with an explicitly dirty working tree. Existing plan/research/log changes
were preserved. Actor pins are Roc `nightly-2026-10-09-258ab27`, Clang
`23.1.3`, Node `26.12.0`, Wrangler `4.151.1`, workerd
`1.20261012.1`, Datastar `1.0.2`, compatibility date `2026-10-19`,
ABI 1 and export schema 1. The repository's September 18 native compiler pin
is unchanged. That pin traps on this standalone Wasm `List.concat` workload;
the isolated October 9 compiler is required. An earlier input-string issue
was a corrected shim capacity encoding error, not evidence of compiler data
corruption.[^build][^abi][^guide]

The core Wasm is 74,972 bytes, SHA-256
`1173da4b3ab19c2d29eff8c749c3e510ee7a959332a90b3bffd6d57a6ad08bcd`,
with no imports. Generated `LiveCounterUi.roc` hash is
`d0b62f2dcf216d9650d669c73d70888d1009c43e31a7d5f68a6120a3135cdd7a`.
`CounterCore.roc` hash is
`f899e7c77e6c7df84797b9fb88a1460b6a5e85266d348f0f84cbadf6e83fb939`.
Build manifests also hash the generated document, Html, Datastar, Origin,
host source, assets and lockfile. Packaging stamps the upload bundle and
conformance checks its fingerprint; deploy refuses subsequent drift and
does not rebuild.[^build][^abi][^guide]

## Local validation and resources

The offline `cargo test --workspace` suite and `cargo fmt --all -- --check`
passed. Documentation built 54 static pages. The opt-in native HTTP control
passed increment/reset response checks and a SQL-trigger failure showing
count/feed rollback. The actor suite passed all 11 tests, including strict
envelopes, signed JWT claims, retry identity, slow-reader coalescing, actual
SQL rollback, post-commit lost reply, actor eviction/reconstruction, frozen
export, separate-namespace import, additive upgrade, compatible rollback
and receipt-cap retry behavior.[^native][^abi][^runtime][^guide]

One 10,000-call full-feed ABI sample took 12,846 ms total (~1.28 ms/call in
Node, not hosted CPU). Both actor memories stayed at 85 pages each, totaling
11,141,120 bytes; peak invocation arena was 995,743 bytes. Node RSS was
113,653,344 bytes and is not a Worker-isolate measurement.[^abi]

The local inspector-attached workerd sample at `2026-10-19T10:51:05.555Z`
populated both keys with 50 maximum-field rows, opened 20 stalled clients
per key, and accepted ten more commands per key. Each actor retained 85
Wasm pages, peak arena 839,568 bytes, 20 viewers and revision 60. Warm
HTTP command latency was p50 7.34 ms, p95 19.72 ms, maximum 25.47 ms.
Raw V8 byte counters were:[^resource-probe]

| Workload | Used JS heap | Reserved JS heap | Embedder heap | Backing storage |
| --- | ---: | ---: | ---: | ---: |
| Baseline | 838,700 | 1,572,864 | 515,872 | 232,569 |
| Both full feeds | 4,069,560 | 11,272,192 | 773,792 | 554,033 |
| Forty stalled readers | 19,148,944 | 31,981,568 | 6,964,032 | 2,861,247 |

These counters and the separate Wasm page totals are measured resource
evidence, not a proof of total hosted-isolate memory below 64 MiB. Native
provider/SQL overhead is not fully reported, no forced GC was used, and OS
network buffering can drain a nominally stalled client. Unit conformance
separately proves the bounded one-frame/one-pending-snapshot policy.[^resource-probe][^streams][^runtime]

## Private deployment and Access investigation

Retained Worker: `rocci-counter-private`; hostname:
`counter-private.worker.rocci.dev`; SQLite namespace:
`8409af3c13d347a48124dd4114a11811` (`rocci-counter-private_Counter`).
Initial deployment version was `baf67c66-5885-43ab-973f-6791796f618a`,
startup 2 ms, upload 135.49 KiB (43.25 KiB gzip). Later asset/config
version was `43a129cd-88d1-4bc6-8722-3e3e22181dba`. Generated credentials
were excluded from assets, manifests and Worker vars. The first Custom
Domain certificate was briefly pending, then HTTPS reached Access login.[^guide][^auth]

Read-only provider settings confirmed `enabled:false` and
`previews_enabled:false` for the workers.dev surface. Public probes of
`rocci-counter-private.nils-hjelte.workers.dev` and the initial version's
preview hostname returned 404. The custom hostname redirected signed-out
requests to `yellow-snow-90f1.cloudflareaccess.com`.[^guide][^auth]

The owner's existing Access setup was correct. Dashboard inspection found
self-hosted application `8df868e3-9fcb-4830-aa74-15f399438bbe`, covering
the entire selected hostname, with only the attached **Admin Email** Allow
policy `8f7f8747-3c72-425d-bbdd-900afc060cb8` including exactly
`nils.hjelte@gmail.com`. Its displayed AUD exactly matched the ignored
`.env`. The deployment token returned an empty list but `403 auth.forbidden`
for a direct read of that existing app/policy. Therefore its empty list
could not establish absence; dashboard evidence and successful owner login
resolved the discrepancy. No Access policy or token-scope expansion was
needed for the retained deployment.[^guide][^auth]

## Hosted browser evidence

Safari and the in-app browser saw count 2, then propagated increments to
3 and 4 in both directions. The owner-browser conformance run observed at
`2026-10-19T10:48:55.806Z` reset and accepted 100 concurrent increments,
all with finite 204 responses; both extra SSE readers saw count 100 at
revision 105. A matching retry left revision unchanged, a mismatched reuse
returned 409, and a new increment plus a reopened reader converged to
count 101/revision 106. The second key remained revision 0. Concurrent
HTTP latencies were p50 439.9 ms, p95 451.9 ms and maximum 515.2 ms;
these include client/network/queueing and are not Roc CPU timings.[^browser]

The demo actor's manifest still reported the initial version while the
newly activated isolation actor reported the later version. Active stream
requests had not yet demonstrated a reconstruction onto the newer version.
Close viewers and verify the returned version explicitly during the
upgrade drill; uploading code alone does not prove this gate.[^host][^browser]

## Remaining acceptance and operating limits

Finish the running idle check, signed-out path matrix, actual hosted export
and separate restore, source fencing, schema-2 upgrade/reconnect and
schema-1-compatible rollback. The restore configuration is packaged/tested;
the owner is adding `counter-restore.worker.rocci.dev` to the same existing
Access app before its first deployment. Then collect runtime SQL/stream
counters and provider analytics. Preserve the retained demo and remove
only explicitly disposable resources after exporting and checking their
identities.[^guide][^host][^browser]

The runbook includes scope-specific manual account steps, all configuration
roles, certificate readiness, tested-artifact deployment, export/restore,
compatible rollback, cleanup and a two-object idle/active duration estimate.
SSE residency is explicitly charged; the small owner's successful demo is
not a scaling or low-cost-hosting claim. Required hosted CI and Knowledge
run IDs must be recorded before implementation phases are logged complete.
celld remains an optional later gate.[^guide][^plan]

[^plan]: Requested scope, phased exits and operating budgets.
[^core]: Shared values, checked revision/count decisions and supplied-time view.
[^native]: Native coherent reads and immediate mutation transactions.
[^host]: Provider storage transactions, receipts, durability barriers and operator boundary.
[^streams]: Snapshot ordering, coalescing, cancellation and conditional timers.
[^build]: Exact pins, input hashes and independent artifact generation.
[^abi]: Reproducible decision/HTML parity, bounds, trap and 10,000-call workload.
[^runtime]: Actual workerd SQL and HTTP recovery/conformance scenarios.
[^resource-probe]: Reproducible local inspector workload and telemetry limitations.
[^browser]: Owner-session concurrent commands, coherent readers, reconnect and idle exercise.
[^guide]: Account setup roles, deployment/recovery instructions and recorded investigation.
[^auth]: Runtime signature/issuer/audience/expiry/owner and same-origin validation.
