---
type: Status
title: Private Rocci durable-actor backend results
description: Dated implementation, hosted recovery and resource evidence for the private Cloudflare live-counter experiment, with provider cleanup and measurement limits explicit.
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/persistence, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-10-19T12:06:00Z }
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
  - id: ci
    resource: https://github.com/koliyo/rocci/actions/runs/39247815201
    title: Required CI on the final clean-EOF recovery implementation
  - id: knowledge-ci
    resource: https://github.com/koliyo/rocci/actions/runs/39247819873
    title: Required Knowledge checks on the final implementation revision
  - id: provider-metrics
    resource: https://dash.cloudflare.com/4ffee8f8e3c6afde3feabe31262afe5c/workers/durable-objects/view/8409af3c13d347a48124dd4114a11811
    title: Owner-only retained namespace aggregate dashboard, observed October 19
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
| Idle and deployment reconstruction | Hosted idle passed for 600,226 ms; both corrected readers automatically rejoined rollback and final versions and received new increments after each, without a page refresh. |
| Restore / compatible rollback | Hosted separate-namespace import, exact export round trip, independent mutation, additive upgrade and provider rollback passed. |
| Required CI / Knowledge | All six CI jobs and Knowledge passed on final source `d8b609d2`. |

The scenario definitions and limits are executable in the linked ABI,
workerd and owner-browser suites; this table does not substitute local
results for hosted exits.[^abi][^runtime][^browser]

## Inputs and artifacts

The tested source baseline was `97227bf514da4b5b08fe092db687c2d53141f77a`
with an explicitly dirty working tree. Existing plan/research/log changes
were preserved. Implementation commits are `e6467a1b` (core/host), `28ce93d4`
(hosted compiler compatibility and deploy mode), `bbf7b1ef` (first stream fence),
`a33eeef4` (configured-schema diagnostic), `e559f03d` (explicit version
activation), and `c92d4e12` (durable asynchronous stream ownership fence).
`6eca1cba` exposes ownership-write costs; `d8b609d2` fixes clean-EOF client
retry. The final build reports a dirty tree from separately authored knowledge
work; runtime inputs are hashed against `d8b609d2`. That other work was
preserved and excluded from this task's commits.
Actor pins are Roc `nightly-2026-10-09-258ab27`, Clang
`23.1.3`, Node `26.12.0`, Wrangler `4.151.1`, workerd
`1.20261012.1`, Datastar `1.0.2`, compatibility date `2026-10-19`,
ABI 1 and export schema 1. The repository's September 18 native compiler pin
is unchanged. That pin traps on this standalone Wasm `List.concat` workload;
the isolated October 9 compiler is required. An earlier input-string issue
was a corrected shim capacity encoding error, not evidence of compiler data
corruption.[^build][^abi][^guide]

The initial core Wasm was 74,972 bytes, SHA-256
`1173da4b3ab19c2d29eff8c749c3e510ee7a959332a90b3bffd6d57a6ad08bcd`,
with no imports. Generated `LiveCounterUi.roc` hash is
`d0b62f2dcf216d9650d669c73d70888d1009c43e31a7d5f68a6120a3135cdd7a`.
`CounterCore.roc` hash is
`f899e7c77e6c7df84797b9fb88a1460b6a5e85266d348f0f84cbadf6e83fb939`.
The final clean-EOF retry build (`d8b609d2`) is 74,989 bytes, SHA-256
`0266f9244fa6ef5753eb01ee09ac37dad176fc51921cc7784d56e317c81c74cc`;
its generated document hash is
`7516c04d00f975ab4666bf0a5dac50fd3b773725a56209985faef81c0eb984af`.
Build manifests also hash the generated document, Html, Datastar, Origin,
host source, assets and lockfile. Packaging stamps the upload bundle and
conformance checks its fingerprint; deploy refuses subsequent drift and
does not rebuild.[^build][^abi][^guide]

## Local validation and resources

The offline `cargo test --workspace` suite and `cargo fmt --all -- --check`
passed, as did `cargo clippy --workspace --all-targets -- -D warnings`.
Documentation built 54 static pages. The opt-in native HTTP control
passed increment/reset response checks and a SQL-trigger failure showing
count/feed rollback. The actor suite passed all 12 tests, including strict
envelopes, signed JWT claims, retry identity, slow-reader coalescing, actual
SQL rollback, post-commit lost reply, actor eviction/reconstruction, frozen
export, separate-namespace import, additive upgrade, compatible rollback
and receipt-cap retry behavior.[^native][^abi][^runtime][^guide]

Hosted CI initially found a new Rust deprecation in two existing atomic
`fetch_update` calls. Equivalent `compare_exchange_weak` loops preserve the
behavior and pass the existing tests on local and hosted compilers. All six
CI jobs passed for `bbf7b1ef` in run `39244597669`; Knowledge passed that same
revision in `39245470712`. Final diagnostic revision `a33eeef4` uses CI
`39245696314` and Knowledge `39245700556`; their completion is recorded when
observed, separately from local conformance. Durable-fence revision `c92d4e12`
passed CI `39246473147` and Knowledge `39246476572`. Final source `d8b609d2`
passed all six jobs in CI `39247815201` and Knowledge `39247819873`.
Documentation-only evidence updates receive a separate Knowledge run.
These checks remain distinct from local actor conformance.[^ci][^knowledge-ci][^guide]

One 10,000-call full-feed ABI sample took 12,846 ms total (~1.28 ms/call in
Node, not hosted CPU). Both actor memories stayed at 85 pages each, totaling
11,141,120 bytes; peak invocation arena was 995,743 bytes. Node RSS was
113,653,344 bytes and is not a Worker-isolate measurement.[^abi]

The final retry-policy artifact repeated all 10,000 calls in 13,782 ms,
retaining the same 85 pages per actor, 11,141,120 combined linear-memory bytes
and 995,743-byte peak arena. Its Node RSS was 112,719,696 bytes. This confirms
the changed document's ABI bounds, not a new hosted memory profile.[^abi]

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

A later instrumented local sample reported 387 SQL statements, 1,977 rows
read and 186 rows written per active object, plus 410 frames / 11,014,060
bytes over its 20 readers. Zero coalescing in that sample reflects drained
OS buffers, not absence of the bounded queue; the unit test stalls the
writer itself. Runtime manifest counters are per activation and include
initialization and diagnostic reads; they are not account billing totals.[^resource-probe][^host][^streams]

Existing provider dashboards were read without enabling logs at approximately
12:04 UTC. Their **Last 24 hours** window mixes experiment versions and lags
live state; it is not an isolated benchmark of the final deployment:

| Provider scope | CPU p50 / p90 / p99 | Memory p50 / p90 / p99 (displayed units) |
| --- | --- | --- |
| Ingress Worker | 1.93 / 6.36 / 23.27 ms | 2.1 / 2.78 / 3.58 MB |
| Retained Durable Object namespace | 1.88 / 5.2 / 13.79 ms | 9.13 / 11.96 / 13.35 MB |

The object dashboard's p999 memory was also 13.35 MB; a displayed time-bucket
p99 was 14,843,745 bytes. These sampled provider values support memory
headroom for the hosted exercise, but are not exhaustive whole-isolate peaks
for both maximum fixtures plus forty stalled readers. CPU includes host and
Roc work, not separately timed Roc functions. The namespace showed 241 HTTP
requests, about 540 GB-s billable duration, 7k rows read and 410 rows written.
Storage still displayed 0 B despite verified durable state, demonstrating
that these storage totals were not yet current. Seventeen error outcomes
comprised twelve client disconnects and five Worker exceptions across the
iteration window; they were not attributed to final-version requests. No
CPU-limit, memory-limit or internal errors were reported. This is not an
exception-free production claim or a final account bill.[^provider-metrics]

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

The ten-minute idle exercise passed at 600,226 ms with 39 heartbeats and
214,922 received bytes. Signed-out GET/POST probes across 28 document,
asset, command, operator and unknown paths all redirected to Access, including
spoofed email-header requests; both alternate workers.dev surfaces remained
404. Assets are gated before routing.[^browser][^auth][^guide]

## Hosted recovery and compatible rollback

The retained actor was frozen at count 101, revision 106, 50 feed rows and
106 receipts. Its schema-1 export checksum was
`0363cb19b9d431b3c42564b637682ff2dd602cbb2a15225a820f5a067b145bb0`.
The additive schema-2 deployment `640edf87-d5d8-48da-8c25-ff837db636ac`
preserved those values. Actual provider rollback to
`43a129cd-88d1-4bc6-8722-3e3e22181dba` preserved stored schema 2 and all
state/receipts; both schema-2 exports had checksum
`cffa62dc0b2e459cb564ee9c3a0417c182f2b39c85885d8830370388cb7f663d`.
An accepted increment ID from revision 6 still returned 204 with its original
receipt revision, leaving current revision 106 unchanged through upgrade,
rollback and later code versions.[^host][^browser][^guide]

Restore Worker `rocci-counter-restore`, version
`04011b6e-97bb-489e-a4f1-d1be45cb9687`, used the distinct namespace
`302abd9750fb4f80bdba69cdf0bf4311` at
`counter-restore.worker.rocci.dev`. Import into the empty target reproduced
the source state, all receipts and the original schema-1 checksum exactly.
The same old retry returned receipt revision 6 without another mutation.
After unfreezing only the restore target, one browser increment reached
count 102/revision 107/107 receipts; source revision stayed 106. The target
was refrozen and exported before retirement. This demonstrates namespace
isolation and application-level recovery, not a production routing cutover
or an arbitrary provider-node power failure.[^host][^browser][^guide]

## Deployment stream investigation

An early update exposed a real lifecycle bug: old HTTP streams kept showing
count 101 while the replacement actor reported zero viewers. Unchanged HTML
alone was insufficient proof of reconnect. Cloudflare can allow old HTTP
responses to survive actor replacement until storage is accessed. A synchronous
SQLite-read fence did not restore the viewers; that test alone did not isolate
server termination from client retry behavior. The conservative shared
15-second heartbeat now overwrites one fixed KV metadata key
with a fresh lease marker and awaits `storage.sync()` before writing stream
bytes. Failed durability aborts all old viewers and clears both timers; only
one ownership check can be in flight. The asynchronous fake-timer regression
proves one shared check and complete cleanup on failure. This adds at most
four small metadata writes per active object per minute, independent of viewer
count, without changing domain state/receipts or polling them every 100ms.
These KV writes are additional to the manifest's explicit SQL counters.
The metadata key is overwritten, so it does not grow per heartbeat.[^streams][^host][^guide]

The final client-side cause was verified against the pinned Datastar 1.0.2
implementation: default `auto` retries network errors but stops on clean EOF.
Managed replacement can end a stream cleanly. The remote document now explicitly
sets `Retry(Always)`; native document behavior is unchanged. The actual compiled
native/Wasm document parity test also asserts this retry policy. New pages
must load this corrected document once; subsequent actor replacements require
no manual reload. This finding supersedes any inference that zero replacement
viewers alone proved the server-side storage check had failed.[^browser][^streams][^guide]

Guarded bootstrap version `78952e44-45bb-413f-ba7c-9fea9d9b589e` explicitly
reported two active browser profiles at revision 106 before another update.
Configuration-only version `0aa58a63-5348-4b67-8132-79db5b73b627` did not
immediately replace that active actor. The next actual code update is
`2db4be23-e82b-44cb-afa6-c12026619afb`, exposing configured schema separately
from stored schema. Record the returned per-actor version and viewers after
rollout; upload completion or an unchanged count is not acceptance evidence.
Explicit activation through the deployments API advanced the actor to that
version, but viewers did not return until the client EOF policy was corrected.
The deploy tool
now follows the declarative-export upload with `wrangler versions deploy` of
the exact uploaded version at 100% and immediate code-update mode. Both steps
are checked for failure. Durable-fence bootstrap
`c791e0fd-e86b-40da-ad50-2151852385ba` (source `c92d4e12`) then reported two
live readers at revision 106, configured/stored schema 2. The final reconnect
drill uses compatible guarded versions. Global rollout remains eventually
consistent: immediate mode is not a promise of an instantaneous change at
every actor.[^host][^streams][^guide]

Final artifact `0772316a-c97c-4f98-a8fd-c0a562c8d84c` (source `d8b609d2`)
bootstrapped two fresh in-app readers with the explicit retry policy at count
103/revision 108. Without refreshing either page, actual provider rollback to
`c791e0fd-e86b-40da-ad50-2151852385ba` eventually reported those two viewers
on the previous host version with unchanged revision 108. A new command then
rendered count 104/revision 109 in both readers. This final rollout exercise
used two in-app tabs; the earlier independent Safari/in-app profile and
bidirectional update exercise remains separate evidence. Roll-forward to
`0772316a-c97c-4f98-a8fd-c0a562c8d84c` subsequently reported both viewers,
revision 109, configured schema 1 and preserved stored schema 2. A further
increment reached count 105/revision 110 in both existing pages. The original
accepted operation still returned receipt revision 6 without another mutation.
Propagation took minutes, so neither the upload command nor the deployment
dashboard's 100% label was used as proof of actor replacement.[^host][^browser][^guide]

The final source export retained count 105, revision 110, 50 feed rows and
110 operations, checksum
`641e03d56e1ee53995d6b882ae4e3d8866cef2488285ce45ca63fa4ee856ee3b`.
It was saved locally, then writes were explicitly unfrozen again. The final
bundle upload was 138.52 KiB with 2 ms reported startup; core Wasm remains
74,989 bytes with 85 pages per activated actor. No namespace was recreated
for these updates.[^host][^build][^guide]

All test viewers were closed. A subsequent fresh activation reported zero
viewers, frames, heartbeats and ownership writes while retaining revision 110;
the original retry again acknowledged receipt revision 6 without changing it.
This confirms the final stream drain and durable reconstruction. A fresh
signed-out request still redirected to Access with 302.[^host][^browser][^auth]

A separate local browser probe served the real compiled document and pinned
Datastar module, ended three successful SSE responses cleanly, and observed
four subscriptions (the last returned 204 to stop the probe) with rendered
count 3. It verifies actual clean-EOF retry behavior independently of provider
rollout timing. The probe server and tab are disposable.[^browser][^abi]

## Cleanup, disposition and operating limits

After export, an auth-gated tombstone deployment
`59c45232-48aa-4596-9ef4-1b09857b1858` reported `Deleted: Counter` for
the restore Worker. Provider reads verified restore namespace `302abd…` gone
and retained namespace `8409af…` present. The disposable Worker and Custom
Domain were removed; the owner approved removal of its Access destination,
which saved successfully. The main hostname, policy, AUD, namespace and data
were preserved, and the source was unfrozen after the drill.[^host][^guide]

Cloudflare retained the unused restore certificate
`5e8c6a48-f18f-4d1d-bfda-5439c1e92f92`. The owner approved its deletion, but
the dashboard rejected it with **Your certificate could not be deleted** and
a support link. This is the outstanding provider cleanup: in rocci.dev →
SSL/TLS → Edge Certificates, remove only the certificate containing
`counter-restore.worker.rocci.dev`; if it still fails, contact Cloudflare
support with that certificate ID. The retained counter certificate and root
Universal certificate must remain. No paid ACM subscription was enabled.[^guide]

The runbook includes scope-specific manual account steps, all configuration
roles, certificate readiness, tested-artifact deployment, export/restore,
compatible rollback, cleanup and a two-object idle/active duration estimate.
SSE residency is explicitly charged; the small owner's successful demo is
not a scaling or low-cost-hosting claim. The measured disposition is **iterate
the private POC**: functional and recovery evidence supports keeping the demo,
while exhaustive whole hosted-isolate memory below 64 MiB, isolated Roc CPU
and final billing totals remain unproven. Local V8/Wasm and provider aggregate
measurements provide headroom evidence with
explicit coverage limits; no production promotion or general reusable Worker
platform is claimed. celld remains the plan's optional later gate.[^guide][^plan]

A proposed short provider live-log probe was not executed: automatic approval
review rejected attaching it because live logging can transmit request/runtime
data upstream and trigger a runtime update. No workaround or broader token was
used. Normal provider rollback supplied the hosted reconstruction evidence;
provider live-log CPU profiling remains unverified. Existing aggregate CPU,
memory and usage dashboards were subsequently read as the safer alternative
above.[^guide][^provider-metrics]

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
[^ci]: Successful required hosted CI for the ownership-fenced implementation revision.
[^knowledge-ci]: Successful required hosted Knowledge checks on the same revision.
[^provider-metrics]: Dated owner-dashboard snapshot; mixed versions, sampling and reporting lag limit interpretation.
