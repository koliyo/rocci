---
type: Research Report
title: Gaps for running basic-webserver as a WASI HTTP module
description: "Pinned basic-webserver 0.16 is a native Tokio/Hyper process that binds TCP and calls Roc. Portable WASI HTTP inverts that: the runtime owns the listener and the guest exports async handle. The Roc handler call is blocking C-ABI (CPU occupancy); generated SSE Wait already lives in the host. Nested hosted I/O inside respond! is the stall. Missing work is a new adapter, not a Rocci flag."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-29T14:50:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: bws-main
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/main.roc
    title: basic-webserver 0.16 platform contract and native-only targets
    author: organization:roc-lang
  - id: bws-cargo
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/Cargo.toml
    title: Host crate is a native staticlib on Tokio 1.45, Hyper 1.6, bundled SQLite, rustls
    author: organization:roc-lang
  - id: bws-readme
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/README.md
    title: Supported native triples; Hyper/Tokio; no wasm32
    author: organization:roc-lang
  - id: bws-lib
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/lib.rs
    title: Host main calls http_server::start
    author: organization:roc-lang
  - id: bws-http
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http_server.rs
    title: Multi-thread Tokio runtime, OS-thread Roc executor, Brotli workers
    author: organization:roc-lang
  - id: bws-exec
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/roc_executor.rs
    title: FixedExecutor is std::thread workers plus a condvar ring
    author: organization:roc-lang
  - id: bws-sqlite
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/sqlite.rs
    title: Hosted SQLite via libsqlite3-sys C connections
    author: organization:roc-lang
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps pin basic-webserver 0.16 and emit init/respond/shutdown plus SSE
    author: process:git
    last_modified: 2026-08-22
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: Listen host and port come from ROC_BASIC_WEBSERVER_* env
    author: process:git
    last_modified: 2026-08-25
  - id: roc-host-readme
    resource: ../../../crates/rocci-roc-host/README.md
    title: Embedded wasm32 apply platform and Wasmtime evaluation
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Minimal wasm32 platform with main! and no HTTP
    author: process:git
    last_modified: 2026-08-18
  - id: host-c
    resource: ../../../crates/rocci-roc-host/platform/host.c
    title: Apply host.o is malloc plus roc_main, not a server
    author: process:git
    last_modified: 2026-08-18
  - id: host-rs
    resource: ../../../crates/rocci-roc-host/src/host.rs
    title: Wasmtime WASI preview1 _start runner, no wasi:http
    author: process:git
    last_modified: 2026-08-23
  - id: roc-host-cargo
    resource: ../../../crates/rocci-roc-host/Cargo.toml
    title: Optional wasmtime 47 plus wasmtime-wasi only
    author: process:git
    last_modified: 2026-08-23
  - id: efficient-research
    resource: ../rocdown/efficient-publishing.md
    title: Wasm apply is not HTTP; WASI-HTTP would be a new platform
    author: process:cursor
    last_modified: 2026-08-28
  - id: efficient-plan
    resource: ../../plans/rocdown/efficient-publishing.md
    title: Phase 6 WASI-HTTP gate closed as no-go for musl process hosting
    author: process:cursor
    last_modified: 2026-08-24
  - id: generation-research
    resource: rocci-components-in-generation.md
    title: roc build --target wasm32 emits a WASI module for apply, not a server
    author: process:cursor
    last_modified: 2026-08-24
  - id: wasi-http-03
    resource: https://wasi.dev/releases/wasi-p3
    title: WASI 0.3 native async, wasi:http/service handle async func, Wasmtime 46+
    author: organization:bytecode-alliance
  - id: wasi-async-cm
    resource: https://component-model.bytecodealliance.org/design/async.html
    title: Canonical ABI async func, stream, and future; runtime owns wake-up
    author: organization:bytecode-alliance
  - id: wasmtime-serve
    resource: https://component-model.bytecodealliance.org/running-components/wasmtime.html
    title: wasmtime serve runs wasi:http service or 0.2 proxy worlds
    author: organization:bytecode-alliance
  - id: wasmcloud-wasi
    resource: https://wasmcloud.com/docs/wash/developer-guide/language-support/rust/
    title: Tokio wasip2 covers sockets, not wasi:http incoming-handler
    author: organization:wasmcloud
  - id: wit-bindgen
    resource: https://github.com/bytecodealliance/wit-bindgen/blob/main/README.md
    title: Core wasm plus adapter becomes a WASI component
    author: organization:bytecode-alliance
  - id: roc-wasi
    resource: https://github.com/ostcar/roc-wasi-platform
    title: Third-party Roc WASI command platform, not HTTP
    author: organization:ostcar
  - id: bws-time
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/time.rs
    title: hosted_sleep_millis is std::thread::sleep on the Roc worker
    author: organization:roc-lang
  - id: bws-sse-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Sse.roc
    title: unfold Wait returns WaitToHost; the host owns the timer
    author: organization:roc-lang
  - id: bws-http-send
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http.rs
    title: hosted_http_send_request block_on of a nested Tokio runtime
    author: organization:roc-lang
  - id: cm-async-rfc
    resource: https://github.com/dicej/rfcs/blob/component-async/accepted/component-model-async.md
    title: Wasmtime fibers for async-to-sync fusion; guest C ABI stays sync
    author: organization:bytecode-alliance
  - id: wasmtime-async
    resource: https://docs.wasmtime.dev/api/wasmtime/
    title: Async host imports park the guest fiber without blocking the host thread
    author: organization:bytecode-alliance
  - id: wasi-plan
    resource: ../../plans/rocci/basic-webserver-wasi.md
    title: WASI HTTP adapter implementation plan; yield around Roc
    author: process:cursor
    last_modified: 2026-08-29
  - id: probe-crate
    resource: ../../../crates/rocci-wasi-http/src/probe.rs
    title: Phase 0 overlap probe (native current-thread plus Wasmtime guest imports)
    author: process:git
    last_modified: 2026-08-29
  - id: sse-research
    resource: basic-webserver-sse-http.md
    title: Native 0.16 SSE idle timeout and Wait-is-silent
    author: process:cursor
    last_modified: 2026-08-21
  - id: adapter-handle
    resource: ../../../crates/rocci-wasi-http/src/handle.rs
    title: SSE Wait as adapter clocks; overlapping-stream test
    author: process:git
    last_modified: 2026-08-29
  - id: adapter-sqlite
    resource: ../../../crates/rocci-wasi-http/src/sqlite.rs
    title: Sync rusqlite guest; nested respond serializes
    author: process:git
    last_modified: 2026-08-29
  - id: http-module-flag
    resource: ../../../crates/rocci-cli/src/main.rs
    title: rocci build --http-module, distinct from --host wasm
    author: process:git
    last_modified: 2026-08-29
  - id: component-plan
    resource: ../../plans/rocci/wasi-http-03-component.md
    title: Follow-on plan for a wasmtime serve 0.3 component
    author: process:cursor
    last_modified: 2026-08-29
  - id: wasip3-crate
    resource: https://docs.rs/wasip3/0.8.0+wasi-0.3.0/wasip3/
    title: wasip3 0.8.0+wasi-0.3.0 exports wasi:http/service on wasm32-wasip2
    author: organization:bytecode-alliance
  - id: component-crate
    resource: ../../../crates/rocci-wasi-http-component/README.md
    title: Phase 0 empty service; wasmtime serve -Sp3 -Scli
    author: process:git
    last_modified: 2026-08-29
---

# Gaps for running basic-webserver as a WASI HTTP module

## Claim

Rocci already compiles **renderers** to `wasm32` and runs them in Wasmtime.
It does **not** compile `basic-webserver` apps to a WASI HTTP module, and
upstream 0.16 cannot do that either. The missing work is a new host, not a
Rocci flag: the pinned platform is a native process that binds TCP, while
portable WASI HTTP is a **reactor** whose runtime calls `handle`. Extra Rust
(an adapter crate, or a `wasm32` target inside a platform fork) is the
realistic path. Do not extend the apply platform with fake HTTP.[^bws-main][^bws-readme][^roc-host-readme][^efficient-research][^efficient-plan]

WASI 0.3 already supplies **native async** (`async func`, `stream<T>`,
`future<T>`) so an I/O-bound HTTP guest does not need OS-style
multi-threading. The 0.16 thread pools exist because Roc `respond!` is a
**blocking** C-ABI call on a native process, not because WASI HTTP cannot
overlap waits. The handler call itself occupies the instance (CPU).
Generated SSE `Wait` already returns from Roc first; nested `hosted_*`
inside `respond!` is the stall that fibers may or may not park.[^wasi-http-03][^wasi-async-cm][^bws-exec][^bws-http][^wasi-plan]

This record is gap analysis. Implementation sequence:
[WASI HTTP adapter plan](/plans/rocci/basic-webserver-wasi.md). It does
not approve a platform fork or a `--host wasm` meaning change.

## What WASI HTTP would bring

Today `--host wasm` evaluates a `main!` renderer. Native islands and apps
are processes. A WASI HTTP **service** module would be a third artifact:
the same generated Roc (`init!` / `respond!` / `shutdown!`, later SSE and
Sqlite) packaged so other runtimes can call `handle` without Rocci owning
TCP.[^dispatch-rs][^wasi-http-03]

That would allow:

| Capability | Versus today |
| --- | --- |
| One portable server module | Musl still needs `x64musl` vs `arm64musl`; macOS vs Linux natives stay different |
| Runtime-owned listen | `wasmtime serve`, Spin, wasmCloud, or any `wasi:http` host binds; guest does not |
| Capability sandbox | Preopens for `file_root`, clocks, stdio; no `Cmd` or raw TCP unless imported |
| Preview without a Roc child | Desktop or `rocci run` could load a Wasmtime HTTP URL instead of `localhost` from a native binary |
| Edge / multi-tenant later | Same module on a WASI HTTP edge, without a Debian+musl image per island |

It would **not** replace musl as the default publish path (Phase 6 already
closed that product gate), and it must not overload `--host wasm`.[^efficient-plan]

Generated dispatch would not need a lowering change if the adapter maps
WASI requests onto the 0.16 host structs. Hello-web, then SSE, then
sqlite, is the capability ladder; Cmd and in-guest TLS are out of
scope.[^dispatch-rs]

## Two WASI shapes (do not conflate)

| Shape | Who listens | Guest export | Typical run | Fits basic-webserver today |
| --- | --- | --- | --- | --- |
| WASI **command** + sockets | Guest `TcpListener::bind` | `_start` / `main` | `wasmtime -S inherit-network` | Same control flow as native, if the host compiled |
| WASI **HTTP** | Host (`wasmtime serve`, Spin, wasmCloud) | `wasi:http` `handle` | `wasmtime serve` | Inverted: guest must not bind |

WASI 0.2 names the HTTP world `wasi:http/proxy`. WASI 0.3 replaces it with
`wasi:http/service` (`handle: async func(request) -> result<response,
error-code>`). Wasmtime 46+ enables 0.3 and component-model async by
default; `wasmtime serve` runs 0.3 `service` or falls back to 0.2 `proxy`.
Rocci's apply crate already depends on Wasmtime 47, but only links
preview1 `_start`, not `wasmtime-wasi-http`.[^wasi-http-03][^wasmtime-serve][^roc-host-cargo]

Tokio 1.51+ on `wasm32-wasip2` can use **sockets**. It does **not**
implement the HTTP incoming-handler export. That sockets path is still a
command that binds TCP, not portable WASI HTTP.[^wasmcloud-wasi]

Rocci's apply path is a third shape: a WASI **preview1 command** that writes
files or stdout. That is not a server.[^wasm-platform][^host-rs][^generation-research]

For "basic-webserver as a wasm module with a WASI interface," the portable
contract is WASI HTTP, not guest sockets. Sockets keep a process-shaped
binary that still needs network inheritance and will not run on hosts that
only offer `wasi:http`.

## Concurrency: native 0.16 threads vs WASI 0.3 async

Do not read "Wasm is single-threaded" as "one request at a time."

### What 0.16 actually uses threads for

The native host is a process that **accepts on its own sockets**. It
builds Tokio `rt-multi-thread`, a `FixedExecutor` of OS threads (default
32) for blocking Roc `respond!`, Brotli workers, and a shutdown watchdog.
Those threads keep Hyper's accept loop alive while Roc runs
synchronously on another stack.[^bws-http][^bws-exec]

That is a reasonable native design. It is **not** the WASI HTTP design.

### WASI 0.3 native async

WASI 0.2 modeled async as `wasi:io` resources (`pollable`,
`input-stream`, `output-stream`) scoped to one component. Wake-ups did
not compose across a chain of components (the sandwich problem); a middle
component had to poll just to relay readiness.[^wasi-http-03][^wasi-async-cm]

WASI 0.3 (released 2026-06-11; 0.3.1 2026-08-11) moves async into the
Component Model Canonical ABI. Three primitives:[^wasi-http-03][^wasi-async-cm]

| Primitive | Role |
| --- | --- |
| `async func` | Call may suspend; the runtime schedules resume. Guest does not see a `pollable`. |
| `stream<T>` | Typed async channel; a **value**, not a 0.2 stream resource |
| `future<T>` | Single completion; replaces `pollable` |

HTTP `handle` and client `send` are `async func`. Bodies are `stream<u8>`
with `future` trailers/completion. Clocks expose `wait-for` /
`wait-until` as `async func`. Filesystem methods are largely `async
func`. `wasi:io` is removed.[^wasi-http-03]

The **runtime** owns scheduling and wake-up propagation. Concurrent
in-flight `handle` calls can overlap while each is waiting on I/O,
without a guest `std::thread` pool and without each component running an
event loop.

### Typical Rocci servers do not need OS threads for wait

For the work generated apps actually do — SQLite, outbound HTTP,
`Sse.Wait`, reading a preopened static file — the useful concurrency is
**overlapping waits**, not parallel Roc on many cores. That is the same
reason Node, Tokio `current_thread`, and Go's netpoller serve many
connections without one OS thread per request.

A WASI 0.3 adapter should therefore:

1. Export `handle` as `async func`.
2. Map wait points onto WASI `future` / `stream` (clocks for SSE wait,
   `wasi:http` client `send`, async filesystem, host or wasm sqlite that
   yields).
3. Stream SSE as an outgoing `stream<u8>` instead of parking an OS thread
   on Hyper's idle timer.

Full OS-style multi-threading (0.16's 32 Roc workers, Tokio
`rt-multi-thread`, Brotli thread pool) is **often unnecessary** for that
shape. Shared-memory Wasm threads remain a separate story for
CPU-parallel compression or compute, not for request multiplexing.

### What still stalls (adapter constraint, not a WASI 0.3 gap)

**Yes: the call to the Roc handler is blocking.**
`roc_respond_for_host` and `roc_sse_advance_for_host` are core-wasm
`extern "C"` functions. WASI 0.3 `async func` can suspend only at
Canonical ABI await points. A C-ABI call has none; that stack is
occupied until return. WASI does not preempt it.[^bws-http][^wasi-async-cm]

That is **CPU occupancy**, not the long wait in a typical Rocci app.
Do not read "blocking handler" as "one live SSE at a time."

Native 0.16 already split the stacks. `call_roc` is a sync
`roc_respond_for_host`. When `respond!` returns `Server.stream`, the
host loops `roc_sse_advance_for_host` and **parks on Tokio
`Sleep`** for `WaitToHost.wait_millis`. `Sse.unfold!` `Wait` is not
`hosted_sleep_millis` inside `respond!`.[^bws-http][^bws-sse-roc]

| Wait | Where 0.16 puts it | Inside `roc_respond_for_host`? | WASI overlap |
| --- | --- | --- | --- |
| Route + `Html.render` | Roc in `respond!` | Yes (usually ms) | No, on this instance, for that duration |
| SSE `Wait` / idle between frames | Host timer after `advance` returns | No | Adapter `await` clocks; other `handle`s run |
| `Sleep.millis!`, sqlite, file, `Http.send!`, body read during `respond!` | Nested `hosted_*` on the Roc worker | Yes | No, unless that symbol is an async-lowered WIT import Wasmtime can **fiber-park** |

Nested hosted I/O is the real stall. Native implementations are
explicitly blocking: `hosted_sleep_millis` is `thread::sleep`;
outbound HTTP `block_on`s a nested Tokio runtime; sqlite is C on that
same worker. The `FixedExecutor` exists so Hyper's accept loop is not
that worker.[^bws-time][^bws-http-send][^bws-exec]

Wasmtime **can** park a guest fiber when a sync-lifted export calls an
async-lowered host import (`async→sync` fusion). That yields the
*instance* without blocking the *host thread*. It does not make Roc
`async`. Roc hosted names today are linker symbols, not WIT. Phase 0
measured that a **sync** `thread::sleep` import (hosted-sleep C) does
**not** park: other `handle`s serialize. An async host import (adapter
await) does park. Do not `block_on` WASI async from C: it freezes the
instance the same way `thread::sleep` does.[^cm-async-rfc][^wasmtime-async][^wasi-plan][^probe-crate]

Adapter policy (the plan implements this; Phase 0 confirmed it):

1. **Yield around Roc** (preferred). Buffer the WASI body, then call
   `roc_respond_for_host`. For SSE, `advance` then `await` clocks then
   write `stream<u8>`. Generated `@get:live` is this shape.
2. Treat nested sqlite/file inside `respond!` as short critical
   sections. Fibers do **not** apply to sync C hosted sleep.
3. Extra instances are for isolation and CPU, not because "Wasm has
   one thread."

### Phase 0 measured overlap (200ms wait)

Two concurrent probe `handle`s on a current-thread Tokio runtime (one OS
thread). Native analog plus a core-wasm guest whose exports call host
imports (`func_wrap_async` vs sync busy / `thread::sleep`). Command:
`cargo test -p rocci-wasi-http measure_200ms_table -- --ignored --nocapture`.[^probe-crate]

| Mode | Native wall | Native | Wasmtime wall | Wasmtime | Fibers on (3)? |
| --- | --- | --- | --- | --- | --- |
| Adapter await (clocks) | 202ms | overlaps | 203ms | overlaps | Yes (async host import parks) |
| CPU-only C (`roc_respond` stub) | 400ms | serializes | 400ms | serializes | N/A (no await point) |
| Hosted-sleep C (`hosted_sleep_millis`) | 409ms | serializes | 410ms | serializes | **No** |

Making Roc itself async is out of scope. The 0.16 SSE ABI already
pulled the long wait out of `respond!`.[^wasi-plan]

### Adapter SSE Wait (crate embedder)

Two concurrent `WaitEmitGuest` `handle`s on current-thread Tokio, 40ms
`WaitToHost` then one Emit: wall time stays under one-and-a-half waits
(`overlapping_sse_waits_do_not_serialize`). Live `Wait` overlap does
**not** require async Roc. Idle timeout stays the WASI host's, not Hyper's
30s `response_idle_timeout_ms`.[^adapter-handle][^sse-research]

### SQLite nested in `respond!`

Sync rusqlite (bundled `libsqlite3-sys`, no native Hyper/`ring` host crate)
inside `respond!` **serializes** other `handle`s for that duration
(`sqlite_in_respond_serializes`, 40ms stall plus a query). Same class as
hosted-sleep-C. Fibers do not park this path. Do not document it as
yielding.[^adapter-sqlite]

### Capability ladder (experimental crate)

| Capability | In `rocci-wasi-http` | Product |
| --- | --- | --- |
| Hello-web `handle` (ordinary HTML) | Yes (embedder + WAT guest with 0.16 export names) | Experimental |
| SSE `stream` + Wait as adapter clocks | Yes | Experimental |
| One `file_root` preopen | Yes (reject `..`; native OS paths not granted) | Experimental |
| sqlite in `respond!` | Yes, **sync**, serializes | Experimental |
| Portable 0.3 `wasmtime serve` component | Phase 0–5 empty/`Stub`/`roc_*`/`SSE`/`file_root` in `rocci-wasi-http-component`; sqlite omitted from the component | Follow-on: [0.3 component plan](/plans/rocci/wasi-http-03-component.md)[^http-module-flag][^component-plan][^component-crate] |
| `rocci run` / musl publish / `--host wasm` | Unchanged | Native 0.16 / apply |

This nightly's Roc wasm32 platform header did not reliably emit
`roc_respond_for_host` from a 0.16-shaped `platform/main.roc`; tests link
`src/hello_web.wat` instead.[^probe-crate]

### Phase 0 toolchain (`wasmtime serve` empty service)

Recorded 2026-08-29 on the maintainer machine. `async func` lifts
compile. The empty `handle` returns 200 `text/html` hello-web HTML.
No Roc. Not a 0.2 `proxy` retarget.[^wasip3-crate][^component-crate][^wasi-http-03]

| Item | Pin |
| --- | --- |
| Wasmtime CLI | 48.0.1 (`wasmtime serve -Sp3 -Scli`) |
| Native embedder crate | still `wasmtime` 47 |
| WIT | `wasi:http@0.3.0` world `service` (`export wasi:http/handler@0.3.0`, `handle: async func`) |
| Bindings | `wasip3` 0.8.0+wasi-0.3.0 (`wit-bindgen` 0.61.1, `async-spawn`) |
| Rust | rustc 1.97.1, target `wasm32-wasip2` (`wasm32-wasip3` is Tier 3, no prebuilt std) |
| Body write | `spawn_local` after returning `Response`; write-first deadlocks |

Rust `std` on `wasm32-wasip2` still imports `wasi:cli@0.2.9`. That is
why serve needs `-Scli`. `wasm-tools component wit` still shows the 0.3
handler export. `wstd` / `#[wstd::http_server]` exports 0.2
`incoming-handler` and was not used.

SQLite is omitted from the component crate. Bundled `libsqlite3-sys`
does not compile for `wasm32-wasip2` with host `clang` (no
`wasm32-unknown-wasip2` target / no `WASI_SYSROOT`). Sync rusqlite in
the native embedder still serializes other `handle`s; that measurement
is unchanged.[^adapter-sqlite][^component-crate]

## What exists

### Native basic-webserver 0.16

The Roc platform requires `init!`, `respond!`, `shutdown!` plus SSE
`advance`/`drop` exports. Hosted effects cover files, env, stdio, time,
sleep, SQLite, TCP, outbound HTTP, commands, and request-body streaming.
`targets:` lists `x64mac`, `arm64mac`, `x64musl`, `arm64musl`, `x64win`
only. There is no `wasm32` input.[^bws-main][^bws-readme]

The Rust host is `crate-type = ["staticlib"]`. `main` calls
`http_server::start`. Dependencies include Hyper 1.6 (HTTP/1 and HTTP/2
server and client), `hyper-rustls`/`ring`, bundled `libsqlite3-sys`, and
Tokio 1.45 with `rt-multi-thread` and `net`.[^bws-cargo][^bws-lib][^bws-http][^bws-sqlite]

CI-supported triples are those five native targets. Wasm is not listed.[^bws-readme]

### Rocci generated apps

Generated `main.roc` pins that 0.16 tarball, calls
`Server.default_config.with_listen({ host, port })`, mounts native static
files, and uses `Sse.unfold!` for Datastar patches, empty command SSE, and
`@get:live`. Listen address comes from `ROC_BASIC_WEBSERVER_HOST` /
`ROC_BASIC_WEBSERVER_PORT`. Apps and islands also open `pf.Sqlite`.[^dispatch-rs][^serve-rs]

Those apps are ordinary basic-webserver programs. A WASI host that can run
hello-web plus streaming SSE plus SQLite plus static mounts would run them
without changing lowering.

### Rocci wasm today

`rocci-roc-host` stages `platform/main.roc` + `targets/wasm32/host.o`. The
app contract is `main! : {} => [Ok({}), Err([Exit(I32)])]`. `host.c`
provides allocators and `main` → `roc_main`. Wasmtime 47 loads preview1
(`build_p1`, `p1::add_to_linker_sync`) and calls `_start`. There is no
`wasmtime-wasi-http`. `--host wasm` is apply, not `rocci run`.[^roc-host-readme][^wasm-platform][^host-c][^host-rs][^roc-host-cargo][^efficient-research]

Roc `roc build --target=wasm32` emits a **core** module (typically preview1
imports if the host uses libc). It does not emit a WIT component. Turning
that into `wasi:http` needs an adapter (`wasm-tools component new --adapt`)
or a host compiled as `wasm32-wasip2` / a 0.3 component.[^wit-bindgen][^generation-research][^roc-wasi]

## What is missing

### 1. Platform target and prebuilt wasm host

`roc build --target=wasm32` against the 0.16 URL fails because the package
has no `wasm32` `targets:` entry and no `host.o` / wasm `libhost`. Adding
the key is not enough: the native `libhost.a` is Mach-O/ELF/COFF, not Wasm.
Someone must produce a wasm32 host artifact and teach the platform to link
it.[^bws-main]

### 2. Control inversion for WASI HTTP

Native `start` owns the accept loop. WASI HTTP requires exporting `handle`
and **not** binding. `Server.Config.with_listen` becomes advisory or
ignored; the runtime binds. `init!` can still return context and file
roots. `shutdown!` must run when the instance stops, without POSIX signal
threads.[^bws-http][^wasi-http-03]

A sockets-in-guest port would keep listen, but it would not be the WASI
HTTP interface and would not run under `wasmtime serve`.[^wasmcloud-wasi]

### 3. Hosted effects must await, not `spawn_blocking`

Drop 0.16's OS thread pools. Do **not** replace them with a
single-flight policy as the default concurrency story. Implement
`handle` as WASI 0.3 `async func` and make I/O wait on `future` /
`stream`. Blocking Roc `respond!` is the thing to wrap or rewrite at the
adapter boundary, not a reason to abandon overlapping requests.[^wasi-http-03][^bws-exec]

### 4. HTTP stack inside the guest

Hyper's server, h2, HTTP/2 preface detection, native file serving, and
response compression assume a Tokio TCP stream. On WASI HTTP the runtime
already parsed the request. The guest should map `wasi:http` request
fields onto `RequestFromHost`, call into Roc, and map `OutcomeToHost`
(ordinary body vs SSE stream) onto a WASI `response` with a `stream<u8>`
body. Keeping Hyper-in-guest is only for the sockets path.[^bws-http][^bws-cargo][^wasi-http-03]

SSE is a stream mapping, not a thread mapping: `Sse.Wait` becomes
`clocks.wait-for` (or equivalent) between `roc_sse_advance_for_host`
steps; frames go on the outgoing body stream. Idle-timeout behavior
follows the WASI host, not Hyper's 30s `response_idle_timeout_ms`.

### 5. Hosted effects on WASI

| Effect | Native 0.16 | WASI HTTP guest |
| --- | --- | --- |
| Env, stdio, clocks, sleep | OS | `wasi:cli` / clocks (`wait-for` is `async func`) |
| Files, Path, static mounts | OS paths | Preopens; `file_root` paths must be granted; 0.3 filesystem is largely async |
| SQLite | bundled C `libsqlite3` | C-to-wasm sqlite or a host sqlite import that yields; no stock `wasi:sqlite` |
| Outbound `Http.send!` | Hyper + rustls/`ring` | `wasi:http` client `send: async func`, not in-guest TLS |
| TCP | OS sockets | `wasi:sockets` or drop |
| `Cmd` | `std::process` | Not in `wasi:http/service`; drop or a custom import |
| Signals / job objects | OS | Runtime cancels the instance |

Rocci's generated dispatch needs Env, Path, Server, Sse, and often File
roots. Sqlite is required for real apps (todos, notes, live-counter). Cmd
and raw TCP are not on the generated path. Outbound HTTPS is unused in
generated `main.roc` but exists on the platform.[^dispatch-rs][^bws-main][^bws-sqlite]

`ring` and bundled sqlite are the usual wasm compile blockers if the
**whole** host crate is retargeted. A thin adapter that does not link
Hyper-client TLS avoids `ring`; sqlite still needs a wasm build of SQLite
or a host-provided store. Sync sqlite that never yields would serialize
handlers even on WASI 0.3.

### 6. Component model vs Roc ABI

Roc's wasm backend is C-ABI hosted functions (`roc_respond_for_host`,
`hosted_*`). WASI HTTP is WIT. Missing glue, in Rust:

1. A `wasm32` (or `wasm32-wasip2` / 0.3) object that implements Roc alloc
   and the hosted table using WASI imports, preferably async.
2. WIT exports for `wasi:http` `handle` (prefer 0.3 `async func`; 0.2
   incoming-handler still runs under `wasmtime serve`).
3. Translation between WASI request/response (`stream` bodies) and the
   0.16 `InternalServer` host structs (headers, body streams, SSE steps).
4. If the Roc compiler still emits preview1 core wasm, a
   `wasi_snapshot_preview1.proxy` (or reactor) adapter so `wasmtime serve`
   can instantiate it.[^wit-bindgen][^wasmtime-serve][^wasi-http-03]

`roc glue` can generate the Roc↔Rust ABI. It does not generate WASI HTTP
WIT or Canonical ABI async lifts. That layer is new code.

### 7. Rocci product wiring (after a platform exists)

`rocci build --http-module` is an experimental flag that writes the
hello-web guest WAT bytes. It is **not** `--host wasm` (apply) and does
not change `rocci run`. A portable 0.3 `wasmtime serve` component and a
desktop URL over `wasmtime-wasi-http` are still missing; that work is
the [0.3 component plan](/plans/rocci/wasi-http-03-component.md).[^http-module-flag][^component-plan]

## Solution paths (extra Rust)

### A. WASI-HTTP adapter host (recommended if the goal is a WASI interface)

New Rust crate (Rocci-owned or offered upstream), compiled to a WASI 0.3
`wasi:http/service` component **or** run as a native embedder that loads
Roc wasm:

- Implement `handle: async func`.
- On first request (or `_initialize`): `roc_init_for_host`, ignore listen
  host/port, keep context in instance state.
- Translate request → `respond!` → ordinary response or SSE body stream,
  awaiting WASI futures at I/O.
- Implement a **subset** of hosted functions: Env, Path, File, Stderr,
  clocks, sleep, optional wasm sqlite that yields.
- Stub or omit Cmd, raw TCP, in-guest TLS client.

Linking options:

1. **Rust is the wasm component; Roc is a linked object.** `roc build
   --target=wasm32 --no-link` (or equivalent) produces app object code;
   `cargo component` / `wit-bindgen` crate links it and exports WIT. This
   matches how native `libhost.a` + app works, with Rust as the final
   linker. Prefer 0.3 async lifts here.
2. **Roc is the core module; native Rust embedder calls exports.**
   Wasmtime in `rocci-cli` instantiates the module, implements WASI HTTP
   on the outside, and calls `roc_respond_for_host`. The `.wasm` is not a
   portable `wasmtime serve` component unless step (1) or an adapter is
   added.
3. **Preview1 core + proxy adapter.** Keep Roc's current wasm output;
   wrap with `wasm-tools component new --adapt wasi_snapshot_preview1.proxy.wasm`.
   Still need a host.o that exports HTTP handle in a form the adapter
   understands — usually harder than (1).

Option 1 is the one that yields a module other WASI HTTP runtimes can
run. Option 2 is smaller for a Rocci-only preview but fails the "WASI
interface" portability test.

Hello-web (one `respond!` HTML body, no SSE, no sqlite) is the first
proof. Generated Rocci dispatch needs SSE next; apps need sqlite after
that.

### B. Compile the 0.16 host to `wasm32-wasip2` + sockets

Patch Tokio to 1.51+, `current_thread`, drop OS thread pools, `inherit-network`.
Hyper server might run on WASI sockets (community Axum experiments exist).
SQLite C and `ring` remain. The result is still a command that binds TCP,
not `wasi:http`. Reject as the primary "WASI interface" story; keep as a
curiosity if someone only wants one Wasmtime CLI with `--inherit-network`.

### C. Fork basic-webserver and add an official `wasm32` target

Same adapter as A, but living in `roc-lang/basic-webserver` so Rocci keeps
the 0.16 Roc API (`Server`, `Sse`, `Sqlite`). Better long-term if upstream
wants it. Rocci should not silently vendor a full copy; a thin adapter
that **calls** the same Roc ABI is enough to learn.

### D. Do not pretend apply wasm is a server

Already decided for publishing: do not teach `rocci-roc-host`'s `main!`
platform HTTP. A new platform crate/target is required.[^efficient-research][^efficient-plan][^wasm-platform]

## Minimal first slice

The [adapter plan](/plans/rocci/basic-webserver-wasi.md) owns Bound/Exit.
Steps 1–4 of that ladder now exist as crate embedder tests (hello-web,
SSE Wait clocks, one preopen, sync sqlite). A yield-on-sqlite path did
not appear; document serialization instead. A portable `wasmtime serve`
0.3 component is still open.

Out of a first slice: Cmd, outbound HTTPS, HTTP/2 inside the guest,
Hyper compression parity, `rocci run` UX, changing `--host wasm`, Wasm
shared-memory threads.

## Recommendation

Treat **A** as the only path that matches "WASI interface." Target WASI
0.3 `wasi:http/service` so overlapping I/O uses Canonical ABI async, not
a retarget of `http_server.rs` thread pools. Keep native 0.16 for
`rocci run`. Do not overload apply `--host wasm`. The crate embedder has
proven hello-web plus SSE Wait overlap; sqlite nested in `respond!`
serializes. Upstream offer (no fork PR unless the maintainer asks): a
`wasm32` host artifact that exports the 0.16 C-ABI plus a thin WIT
`wasi:http` adapter, not a vendored copy of Hyper/Tokio.[^wasi-plan]

Publishing Phase 6 already recorded a product no-go for replacing musl
islands with Wasm. That no-go is ops (musl containers suffice), not a
finding that WASI HTTP cannot multiplex typical web-server waits. This
research does not reopen that default.[^efficient-plan]

[^bws-main]: No `wasm32` in `targets:`; hosted table includes sqlite, tcp, http_send, cmd, files.
[^bws-cargo]: `staticlib`; Tokio multi-thread; Hyper server+client; rustls/ring; bundled sqlite.
[^bws-readme]: Documented triples are native only; Hyper/Tokio; apps supply init/respond/shutdown.
[^bws-lib]: `main` → `http_server::start`.
[^bws-http]: Multi-thread runtime, FixedExecutor, Brotli threads, watchdog thread.
[^bws-exec]: Worker threads and condvar admission; native workaround for blocking Roc.
[^bws-sqlite]: `libsqlite3_sys` connection pool.
[^dispatch-rs]: Generated platform URL 0.16.0; listen, file roots, `Sse.unfold!`.
[^serve-rs]: `ROC_BASIC_WEBSERVER_HOST` / `PORT` helpers.
[^roc-host-readme]: Wasm host is apply evaluation, not HTTP.
[^wasm-platform]: `main!` only; wasm32 inputs are `host.o` + app.
[^host-c]: Allocator + `roc_main`; no sockets or HTTP.
[^host-rs]: Preview1 `_start`; no `wasi:http` linker.
[^roc-host-cargo]: `wasmtime` 47 + `wasmtime-wasi` only; no wasi-http.
[^efficient-research]: Runtime HTTP not available; WASI-HTTP would be a new platform.
[^efficient-plan]: Phase 6 no-go; musl remains the process story.
[^generation-research]: `roc build --target wasm32` for render WASI modules.
[^wasi-http-03]: Native async in Canonical ABI; `handle` is `async func`; `wasi:io` removed; Wasmtime 46+ default.
[^wasi-async-cm]: Runtime owns scheduling; `stream` and `future` are ABI values, not 0.2 resources.
[^wasmtime-serve]: `wasmtime serve` for 0.3 service or 0.2 proxy.
[^wasmcloud-wasi]: Tokio wasip2 is sockets; HTTP components use `wstd` / incoming-handler.
[^wit-bindgen]: Core wasm + preview1 adapter → component.
[^roc-wasi]: Example WASI command platform, not a webserver.
[^bws-time]: `hosted_sleep_millis` is OS-thread sleep on the Roc worker.
[^bws-sse-roc]: `WaitToHost` after unfold; host owns `wait_millis`.
[^bws-http-send]: Nested `block_on` for outbound Hyper client.
[^cm-async-rfc]: Fibers for async→sync; guest C ABI remains sync.
[^wasmtime-async]: Host async parks guest fiber.
[^wasi-plan]: Yield-around-Roc; Phase 0 measures nested hosted I/O.
[^probe-crate]: `rocci-wasi-http` probe: adapter-await overlaps; CPU-C and hosted-sleep-C serialize; Wasmtime fibers do not apply to sync sleep.
[^sse-research]: Native Wait vs 30s idle; adapter must not copy Hyper's timer.
[^adapter-handle]: SSE Wait as `tokio::sleep` after `advance`; overlapping streams do not serialize.
[^adapter-sqlite]: Sync rusqlite in `respond!` serializes other `handle`s.
[^http-module-flag]: `rocci build --http-module` writes hello-web WAT bytes; `--host wasm` stays apply.
[^component-plan]: Follow-on to emit a `wasmtime serve` 0.3 component (option 1).
[^wasip3-crate]: `wasip3` 0.8.0+wasi-0.3.0; `http::service::export!`; target `wasm32-wasip2`.
[^component-crate]: Empty `handle` crate; `wasmtime serve -Sp3 -Scli` returned 200 hello-web HTML.
