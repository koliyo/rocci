---
type: Implementation Plan
title: WASI HTTP adapter host for basic-webserver apps
description: "Ship a Rocci-owned WASI 0.3 wasi:http/service adapter that links the 0.16 Roc ABI (init/respond/shutdown/sse_advance) and yields around Roc, not by making Roc async. Hello HTML, then SSE Wait as adapter clocks, then preopens, then sqlite. Do not change --host wasm or replace musl publish."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-29T13:10:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/basic-webserver-wasi.md
    title: Gaps for running basic-webserver as a WASI HTTP module
    author: process:cursor
    last_modified: 2026-08-29
  - id: sse-research
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: Native 0.16 SSE idle timeout and Wait-is-silent
    author: process:cursor
    last_modified: 2026-08-21
  - id: efficient-plan
    resource: ../rocdown/efficient-publishing.md
    title: Phase 6 no-go; musl remains the process story; Wasm remains apply
    author: process:cursor
    last_modified: 2026-08-28
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps pin basic-webserver 0.16 and emit init/respond/SSE
    author: process:git
    last_modified: 2026-08-22
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: rocci run listen host and port from ROC_BASIC_WEBSERVER_* env
    author: process:git
    last_modified: 2026-08-25
  - id: host-rs
    resource: ../../../crates/rocci-roc-host/src/host.rs
    title: Wasmtime preview1 _start apply runner, no wasi:http
    author: process:git
    last_modified: 2026-08-23
  - id: roc-host-cargo
    resource: ../../../crates/rocci-roc-host/Cargo.toml
    title: wasmtime 47 plus wasmtime-wasi only
    author: process:git
    last_modified: 2026-08-23
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Apply wasm32 platform is main!, not HTTP
    author: process:git
    last_modified: 2026-08-18
  - id: workspace-deps
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: New workspace crate must be classified in BASE_ROCCI or ROCDOWN
    author: process:git
    last_modified: 2026-08-26
  - id: cargo-toml
    resource: ../../../Cargo.toml
    title: Workspace members list
    author: process:git
    last_modified: 2026-08-26
  - id: bws-main
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/main.roc
    title: 0.16 platform contract, native targets, hosted table
    author: organization:roc-lang
  - id: bws-http
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http_server.rs
    title: call_roc is sync roc_respond_for_host; SSE Wait parks on Tokio
    author: organization:roc-lang
  - id: bws-exec
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/roc_executor.rs
    title: FixedExecutor OS threads for blocking Roc
    author: organization:roc-lang
  - id: bws-time
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/time.rs
    title: hosted_sleep_millis is std::thread::sleep
    author: organization:roc-lang
  - id: bws-sse-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Sse.roc
    title: unfold Wait returns WaitToHost; host owns the timer
    author: organization:roc-lang
  - id: bws-http-send
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http.rs
    title: hosted_http_send_request block_on of a nested Tokio runtime
    author: organization:roc-lang
  - id: wasi-http-03
    resource: https://wasi.dev/releases/wasi-p3
    title: WASI 0.3 native async; wasi:http/service handle async func
    author: organization:bytecode-alliance
  - id: wasi-async-cm
    resource: https://component-model.bytecodealliance.org/design/async.html
    title: Canonical ABI async func, stream, future; runtime owns wake-up
    author: organization:bytecode-alliance
  - id: wasmtime-serve
    resource: https://component-model.bytecodealliance.org/running-components/wasmtime.html
    title: wasmtime serve runs 0.3 service or 0.2 proxy
    author: organization:bytecode-alliance
  - id: cm-async-rfc
    resource: https://github.com/dicej/rfcs/blob/component-async/accepted/component-model-async.md
    title: Wasmtime fibers for async-to-sync fusion; C ABI stays sync to the guest
    author: organization:bytecode-alliance
  - id: wasmtime-async
    resource: https://docs.wasmtime.dev/api/wasmtime/
    title: Async host imports park the guest fiber without blocking the host thread
    author: organization:bytecode-alliance
---

# WASI HTTP adapter host for basic-webserver apps

## Goal

A Rocci-owned **WASI 0.3** `wasi:http/service` component can run the same
generated Roc contract as native 0.16 (`init!` / `respond!` / `shutdown!` /
`roc_sse_advance_for_host`) under `wasmtime serve`, with overlapping
I/O-bound requests, without changing lowering or `--host wasm`.[^research][^dispatch-rs][^wasi-http-03]

Native `rocci run` stays a basic-webserver process. Musl island publish
stays the default (publishing Phase 6 no-go is unchanged).[^efficient-plan][^serve-rs]

**Implementation (this revision):** Phases 0–6 are on branch
`basic-webserver-wasi`. Experimental crate embedder plus
`rocci build --http-module`. Not logged complete until CI and Knowledge
workflow run IDs succeed.

## Out of bound

- Overloading `--host wasm` (that remains apply `main!`).[^wasm-platform][^host-rs]
- Replacing musl process binaries as the default publish path.[^efficient-plan]
- Compiling the 0.16 Hyper/Tokio host to `wasm32-wasip2` sockets (guest
  bind is not WASI HTTP).[^research]
- Teaching the apply platform HTTP.
- Roc language async, async `respond!`, or a new Roc ABI.
- Cmd, raw TCP, in-guest TLS / `ring` outbound client.
- HTTP/2, Brotli thread pools, or Hyper compression parity inside the guest.
- Shared-memory Wasm threads.
- Changing generated dispatch / `Sse.unfold!` / `empty_sse!` for this host.
- Vendoring a full copy of basic-webserver; silently forking upstream.
- `rocci run` UX, desktop URL, or preview-child replacement before Phase 6.
- Upstream `wasm32` target in `roc-lang/basic-webserver` before hello-web
  plus one SSE route are proven.

## Constraints that do not move

- Portable contract is **WASI HTTP**, not guest sockets. The runtime
  binds; `Server.Config.with_listen` is ignored.[^research][^wasi-http-03]
- Prefer WASI 0.3 `handle: async func` and `stream<u8>` bodies.
  `wasmtime serve` 0.2 `proxy` fallback is a compatibility note, not the
  target ABI.[^wasmtime-serve]
- Keep the **0.16 Roc C-ABI**. Adapter maps WASI request/response onto
  `RequestFromHost` / `OutcomeToHost`. Do not lower Rocci differently.[^bws-main][^dispatch-rs]
- **Yield around Roc**, not inside it. `roc_respond_for_host` and
  `roc_sse_advance_for_host` stay synchronous. Long waits that 0.16
  already parked in the host (SSE `Wait`) stay in the adapter as WASI
  `async func` (clocks). Nested hosted I/O inside `respond!` is a
  measured Phase 0 exception, not a reason to invent Roc async.[^bws-http][^bws-sse-roc]
- Do not retarget 0.16 OS thread pools (`FixedExecutor`, Tokio
  `rt-multi-thread`, Brotli workers) into Wasm.[^bws-exec]
- New workspace crate is classified in `BASE_ROCCI` in the same
  change.[^workspace-deps][^cargo-toml]
- Linking shape is research option 1: **Rust is the wasm component; Roc
  is a linked object.** Option 2 (native Wasmtime embedder calling Roc
  exports) is allowed as a Phase 0/6 preview helper, not the portable
  artifact.[^research]
- Hello-web, then SSE, then one `file_root`, then sqlite. Each phase
  must stay runnable under `wasmtime serve` (or an equivalent
  `wasmtime-wasi-http` embedder in tests).

## WASI async vs the Roc handler call

This is the design invariant the phases implement. It is **not** "Wasm
is single-threaded so one request at a time."

### The call to Roc is blocking. That is the wrong wait to fear.

`handle` is a WASI 0.3 `async func`. The adapter may `await` Canonical
ABI `future` / `stream` points. Roc's wasm backend is **core wasm with a
C ABI**. `roc_respond_for_host` and `roc_sse_advance_for_host` are
`extern "C"` functions: they occupy the guest stack until they return.
WASI 0.3 does not preempt a synchronous C-ABI function.[^bws-http][^wasi-async-cm]

So **yes: the call to the Roc handler is blocking** from the component's
point of view. For generated Rocci apps that is usually **CPU occupancy
measured in milliseconds** (route match, `Html.render`), the same class
as a Node request handler that does not `await`.

### The long wait in Rocci apps already left Roc

Native 0.16 already split two stacks:[^bws-http][^bws-sse-roc][^sse-research]

| Step | Who runs | Blocks `roc_respond_for_host`? |
| --- | --- | --- |
| `respond!` returns `Server.stream(Sse.unfold!(…))` | Roc, once | Only for that short call |
| `roc_sse_advance_for_host` one transition | Roc, briefly | N/A (different export) |
| `WaitToHost { wait_millis }` | Host `tokio::time::Sleep` (`RocSseItemSource::park`) | **No** |

`Sse.unfold!` `Wait` is **not** `hosted_sleep_millis` inside `respond!`.
The Roc transition returns; the host owns the timer. On WASI the adapter
does the same with `clocks.wait-for` (or equivalent) **after**
`roc_sse_advance_for_host` returns. Concurrent `@get:live` streams can
overlap without Roc being async.

### Nested hosted I/O inside `respond!` is the real stall

Effects Roc calls **during** `respond!` are also `extern "C"` hosted
symbols. Native 0.16 parks an OS worker on them (`FixedExecutor`,
`std::thread::sleep`, sqlite C, `block_on` of outbound Hyper).[^bws-exec][^bws-time][^bws-http-send]

On one Wasm instance, while that C stack is live, another `handle`
cannot run **unless** the hosted symbol is actually an async-lowered
WIT import that Wasmtime can park on a **fiber** (`async→sync` fusion).
Fibers park the *guest* without blocking the *host thread*; they do not
make C-ABI look like `async fn` to Roc.[^cm-async-rfc][^wasmtime-async]

Roc hosted names today are linker symbols, not WIT. Do **not** assume
fibers yield `hosted_sleep_millis` until Phase 0 measures it.

Generated Rocci uses nested I/O mainly for sqlite (and file reads) inside
`@post:command` / `@init`. Sleep-in-respond and `Http.send!` are not on
the generated path.[^dispatch-rs]

### Adapter policy (ordered)

1. **Prefer yield-around-Roc.** Buffer the WASI request body, then call
   `roc_respond_for_host`. For SSE, loop `advance` → if Wait, `await`
   clocks → if Emit, write `stream<u8>`.
2. **Keep nested hosted I/O short** (sqlite query, small file). Document
   that those milliseconds serialize other `handle`s on the instance if
   fibers do not apply.
3. **Probe fibers** in Phase 0. If a WIT async import called from
   `extern "C"` yields other `handle`s, hosted sleep/sqlite may use that.
   If not, do not `block_on` WASI async from C: that would freeze the
   instance exactly like `thread::sleep`.
4. **Scale-out** with more instances is for isolation and CPU, not a
   substitute for (1).

## Crate and link shape

New workspace crate `crates/rocci-wasi-http` (`rocci-wasi-http`):

- Compiles to a WASI 0.3 component exporting `wasi:http/service`.
- Implements a **subset** of 0.16 hosted symbols: alloc, env, path, file,
  stdio, clocks/sleep, optional sqlite. Stub or omit Cmd, TCP,
  `hosted_http_send_request`.
- Links Roc app object code (`roc build --target=wasm32 --no-link` or
  equivalent). Rust is the final linker (`cargo component` / `wit-bindgen`).
- Tests: component instantiation and `handle` without Rocci CLI. One
  `wasmtime serve` (or embedder) proof per capability phase.

`rocci-roc-host` stays apply. Do not add `wasmtime-wasi-http` there until
Phase 6 if an in-process preview embedder is wanted; CLI may depend on
`rocci-wasi-http` then, still classified `BASE_ROCCI`.[^roc-host-cargo][^workspace-deps]

## Phase 0: Measure Roc-handler blocking vs adapter waits

**Bound:** In `crates/rocci-wasi-http` (create the crate, classify it,
add to `Cargo.toml` members), ship a **probe** component that exports
0.3 `handle` and three routes or modes:

1. Adapter `await` of WASI clocks for ~200ms, then 200.
2. `extern "C"` busy-loop or CPU-only stub standing in for
   `roc_respond_for_host` (~200ms), then 200.
3. `extern "C"` hosted sleep standing in for `hosted_sleep_millis`
   (~200ms) during a sync handler, then 200.

From a test harness, overlap two `handle` calls (or two `wasmtime serve`
requests) and record whether the second completes during the first wait.
Amend [the research](/research/rocci/basic-webserver-wasi.md) with the
measured table (yields / serializes) and whether Wasmtime fibers applied
to (3). No Roc compiler. No product mapping.

**Out of bound:** hello-web, SSE, sqlite, CLI flags.

**Tests:** `cargo test -p rocci-wasi-http`; `cargo fmt --all -- --check`.
Probe may be `#[ignore]` if it needs `wasmtime` CLI; then run it in the
phase and paste timings into the research.

**Exit:** Research names, with numbers, whether concurrent `handle`
overlaps for adapter-await, CPU-C, and hosted-sleep-C. Plan policy (1)–(3)
above is confirmed or amended. Workspace classify check still passes.

## Phase 1: Request/response mapping, stub Roc

**Bound:** Real WASI 0.3 `handle`: map incoming method, path, headers, and
a **fully buffered** body onto 0.16 host request structs (same field
names as `http_server.rs` `request_to_roc`). A stub `roc_respond_for_host`
returns a 200 `text/html` ordinary body. `init` once per instance (first
request or `_initialize`); ignore listen host/port. Map ordinary
`OutcomeToHost` onto a WASI response. No SSE, no files.

**Out of bound:** linking a Roc app; Hyper.

**Tests:** `cargo test -p rocci-wasi-http` (construct a WASI request,
assert status/headers/body). `cargo fmt --all -- --check`.

**Exit:** Those commands pass. A stub component answers `GET /` with HTML
under `wasmtime serve` or the crate's embedder test.

## Phase 2: Link real Roc hello-web

**Bound:** Build recipe: Roc app using the 0.16 **Roc API** (may use a
local platform `main.roc` that `provides` the same exports and a wasm32
`host` input pointing at this crate's object) compiled `--target=wasm32`
and linked into the component. Fixture is hello-web class: `respond!`
returns HTML, no SSE, no sqlite. `roc_init_for_host` / `roc_respond_for_host`
/ `roc_shutdown_for_host` are the real symbols. Hosted env/stdio/time
enough for that example.

**Out of bound:** changing Rocci `dispatch.rs`; Cmd; TLS.

**Tests:** crate tests plus one documented command:
`wasmtime serve <component.wasm>` returns 200 HTML for `GET /`.
`cargo fmt --all -- --check`.

**Exit:** That serve proof is green. Generated-dispatch **Roc** would not
need edits to run this fixture if it only used `respond!` HTML (later
phases add SSE/sqlite).

## Phase 3: SSE as adapter streams and clocks

**Bound:** When `respond!` returns `Server.stream`, export a WASI
`stream<u8>` body. Loop: `roc_sse_advance_for_host` → `EmitToHost` writes
framed bytes → `WaitToHost` **awaits** WASI clocks for `wait_millis`
(0 means immediately) → `EndToHost` closes the stream. Drop source/step
via the 0.16 drop exports. Idle timeout is the WASI host's, not Hyper's
30s `response_idle_timeout_ms`.[^sse-research][^bws-sse-roc]

Prove with (a) `empty_sse!`-shaped immediate End and (b) Wait then one
Emit (keepalive or a fake Datastar frame). Overlapping two streams must
not serialize on the Wait (Phase 0 adapter-await path).

**Out of bound:** changing generated keepalive policy; HTTP/2.

**Tests:** `cargo test -p rocci-wasi-http`; overlapping-stream test or
documented `wasmtime serve` two connections. `cargo fmt --all -- --check`.

**Exit:** Those proofs pass. Research SSE subsection notes that live
`Wait` overlap does not require async Roc.

## Phase 4: Preopened `file_root`

**Bound:** One preopen directory. Implement hosted file/path subset used
by static mounts / `Server.file_root`. Map a File outcome (if the stub or
hello-static example uses it) or `hosted_file_read_*` onto WASI
filesystem. Grant only that preopen; document that native OS paths are
not available.

**Out of bound:** writing arbitrary host paths; Cmd.

**Tests:** `cargo test -p rocci-wasi-http` with a fixture directory;
`wasmtime serve` with `--dir`. `cargo fmt --all -- --check`.

**Exit:** `GET` of one static file from the preopen returns the bytes.

## Phase 5: SQLite that is honest about yield

**Bound:** One wasm or host-backed sqlite path sufficient for a todos- or
notes-class query. Prefer a build that **yields** (host import or
progress callback that hits a WASI `async func`). If Phase 0 showed
nested C does not yield, ship sync wasm sqlite **and** document that
queries serialize other `handle`s for their duration; do not pretend
otherwise. No `libsqlite3-sys` + `ring` accidental pull of the native
host crate.

**Out of bound:** connection-pool parity with 0.16; WAL-on-network-fs.

**Tests:** `cargo test -p rocci-wasi-http` open/query/close; one example
app or fixture. `cargo fmt --all -- --check`.

**Exit:** A request that reads or writes sqlite returns 200. Research
records whether sqlite nested in `respond!` overlapped or serialized.

## Phase 6: Rocci product wiring (optional embedder)

**Bound:** A **new** flag or `roc build --target` selection that produces
the HTTP component. `--host wasm` unchanged. `rocci run` remains native
0.16. Optional: `rocci-cli` or a small binary uses `wasmtime-wasi-http`
to serve the component and print a URL (desktop may load it later; not
required). Classify remaining deps. README: planned vs shipped.

**Out of bound:** replacing preview child-process UX; publishing musl
images; changing apply cache.

**Tests:** `cargo test -p rocci-cli` for the new flag parse (no Roc
required); `cargo test -p rocci-wasi-http`; `cargo fmt --all -- --check`.

**Exit:** `--help` shows the HTTP-module path as distinct from `--host wasm`.
A documented command builds one example `.rocci` to the component when Roc
is available (`ROCCI_REQUIRE_ROC` or `#[ignore]`).

## Phase 7: Knowledge, docs, upstream offer

**Bound:** Research disposition (measured blocking table, shipped
capability ladder). This plan status. Indexes. Public docs: mark WASI
HTTP as experimental/planned per what actually shipped. Short note on
what would be offered upstream (wasm32 host artifact + WIT adapter),
without opening a fork PR unless the maintainer asks. `knowledge/log.md`
phase-complete only after CI and Knowledge workflow run IDs succeed.

**Exit:** `okmate check knowledge --profile rocci --format terminal`.
Crate READMEs and a docs page (if Phase 6 shipped a flag) agree with
the research.

[^research]: Gap analysis, option A, yield-at-I/O, capability ladder.
[^sse-research]: Native Wait vs 30s idle; adapter must not copy Hyper's timer.
[^efficient-plan]: Musl default; apply wasm is not HTTP.
[^dispatch-rs]: 0.16 URL; file roots; `Sse.unfold!`; `empty_sse!`.
[^serve-rs]: Native listen env; `rocci run` stays this.
[^host-rs]: Preview1 `_start` only.
[^roc-host-cargo]: No `wasmtime-wasi-http` today.
[^wasm-platform]: Apply `main!`.
[^workspace-deps]: Unclassified workspace members fail CI.
[^cargo-toml]: Members list.
[^bws-main]: Native targets; hosted table; provides init/respond/sse/shutdown.
[^bws-http]: `call_roc` → `roc_respond_for_host`; SSE park on Tokio Sleep.
[^bws-exec]: OS-thread Roc workers.
[^bws-time]: Blocking sleep hosted function.
[^bws-sse-roc]: `WaitToHost` after unfold transition.
[^bws-http-send]: Nested `block_on` for outbound HTTP.
[^wasi-http-03]: `handle` is `async func`.
[^wasi-async-cm]: Runtime owns scheduling; no guest pollable.
[^wasmtime-serve]: `wasmtime serve`.
[^cm-async-rfc]: Fibers for async→sync; guest C ABI remains sync.
[^wasmtime-async]: Host async parks guest fiber.
