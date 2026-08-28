---
type: Research Report
title: Gaps for running basic-webserver as a WASI HTTP module
description: "Pinned basic-webserver 0.16 is a native Tokio/Hyper process that binds TCP and calls Roc. WASI HTTP inverts that: the runtime owns the listener and the guest exports handle. Compiling the existing host to wasm32 is not enough; a Rust adapter (or a new wasm32 platform target) must replace listen, threads, and several effects."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-28T14:20:00Z }
stale_after: 2026-11-28
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
    last_modified: 2026-08-24
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
    title: WASI 0.3 wasi:http/service world and handle async func
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

This record is gap analysis. It does not approve a platform fork, a Rocci
`--host wasm` meaning change, or an implementation plan.

## Two WASI shapes (do not conflate)

| Shape | Who listens | Guest export | Typical run | Fits basic-webserver today |
| --- | --- | --- | --- | --- |
| WASI **command** + sockets | Guest `TcpListener::bind` | `_start` / `main` | `wasmtime -S inherit-network` | Same control flow as native, if the host compiled |
| WASI **HTTP** | Host (`wasmtime serve`, Spin, wasmCloud) | `wasi:http` `handle` | `wasmtime serve` | Inverted: guest must not bind |

WASI 0.2 names the HTTP world `wasi:http/proxy`. WASI 0.3 replaces it with
`wasi:http/service` (`handle: async func(request) -> result<response,
error-code>`). Wasmtime serves both; 0.3 needs component-model async.
Tokio 1.51+ on `wasm32-wasip2` can use **sockets**. It does **not** implement
the HTTP incoming-handler export.[^wasi-http-03][^wasmtime-serve][^wasmcloud-wasi]

Rocci's apply path is a third shape: a WASI **preview1 command** that writes
files or stdout. That is not a server.[^wasm-platform][^host-rs][^generation-research]

For "basic-webserver as a wasm module with a WASI interface," the portable
contract is WASI HTTP, not guest sockets. Sockets keep a process-shaped
binary that still needs network inheritance and will not run on hosts that
only offer `wasi:http`.

## What exists

### Native basic-webserver 0.16

The Roc platform requires `init!`, `respond!`, `shutdown!` plus SSE
`advance`/`drop` exports. Hosted effects cover files, env, stdio, time,
sleep, SQLite, TCP, outbound HTTP, commands, and request-body streaming.
`targets:` lists `x64mac`, `arm64mac`, `x64musl`, `arm64musl`, `x64win`
only. There is no `wasm32` input.[^bws-main][^bws-readme]

The Rust host is `crate-type = ["staticlib"]`. `main` calls
`http_server::start`. Startup builds a multi-thread Tokio runtime, a
`FixedExecutor` of OS threads for blocking Roc `respond!`, a Brotli worker
pool, and a shutdown watchdog thread. Dependencies include Hyper 1.6
(HTTP/1 and HTTP/2 server and client), `hyper-rustls`/`ring`, bundled
`libsqlite3-sys`, and Tokio 1.45 with `rt-multi-thread` and `net`.[^bws-cargo][^bws-lib][^bws-http][^bws-exec][^bws-sqlite]

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
or a host compiled as `wasm32-wasip2`.[^wit-bindgen][^generation-research][^roc-wasi]

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

### 3. Threading and runtimes

WASI p2/p3 guests are single-threaded. The 0.16 host is not:

- Tokio `new_multi_thread()`
- `FixedExecutor` `std::thread` workers (default 32 handlers)
- Brotli `available_parallelism` workers
- shutdown watchdog `std::thread` + `process::exit`

Those must become `current_thread` (sockets path) or host-driven async
(WASI HTTP path). Blocking Roc `respond!` on a single Wasm thread stalls
the instance; concurrent requests need either host multiplexing across
instances or an explicit single-flight policy.[^bws-http][^bws-exec]

### 4. HTTP stack inside the guest

Hyper's server, h2, HTTP/2 preface detection, native file serving, and
response compression assume a Tokio TCP stream. On WASI HTTP the runtime
already parsed the request. The guest should map `wasi:http` request
fields onto `RequestFromHost`, call `roc_respond_for_host`, and map
`OutcomeToHost` (ordinary body vs SSE stream) onto a WASI response.
Keeping Hyper-in-guest is only for the sockets path.[^bws-http][^bws-cargo]

SSE is the hard mapping: `Sse.Wait` parks on a host timer; WASI HTTP
streams an outgoing body. The adapter must loop `roc_sse_advance_for_host`
and write frames without OS threads. Idle-timeout behavior would follow
the WASI host, not Hyper's 30s `response_idle_timeout_ms`.

### 5. Hosted effects on WASI

| Effect | Native 0.16 | WASI HTTP guest |
| --- | --- | --- |
| Env, stdio, clocks, sleep | OS | `wasi:cli` / clocks (straightforward) |
| Files, Path, static mounts | OS paths | Preopens; `file_root` paths must be granted |
| SQLite | bundled C `libsqlite3` | C-to-wasm sqlite or a host sqlite import; no stock `wasi:sqlite` |
| Outbound `Http.send!` | Hyper + rustls/`ring` | `wasi:http` client, not in-guest TLS |
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
or a host-provided store.

### 6. Component model vs Roc ABI

Roc's wasm backend is C-ABI hosted functions (`roc_respond_for_host`,
`hosted_*`). WASI HTTP is WIT. Missing glue, in Rust:

1. A `wasm32` (or `wasm32-wasip2`) object that implements Roc alloc and
   the hosted table using WASI imports.
2. WIT exports for `wasi:http` `handle` (0.2 incoming-handler or 0.3
   handler).
3. Translation between WASI request/response resources and the 0.16
   `InternalServer` host structs (headers, body streams, SSE steps).
4. If the Roc compiler still emits preview1 core wasm, a
   `wasi_snapshot_preview1.proxy` (or reactor) adapter so `wasmtime serve`
   can instantiate it.[^wit-bindgen][^wasmtime-serve]

`roc glue` can generate the Roc↔Rust ABI. It does not generate WASI HTTP
WIT. That layer is new code.

### 7. Rocci product wiring (after a platform exists)

Even with a wasm server artifact:

- `--host wasm` must stay apply; a different flag or `roc build --target`
  must select the HTTP module.
- `rocci run` / preview assume a child process on `localhost:port`.
  WASI HTTP needs `wasmtime serve` (or an embedder using
  `wasmtime-wasi-http`) and a URL the desktop host can load.
- `rocci-roc-host` would need `wasmtime-wasi-http` and component
  instantiation, not only preview1 `_start`.[^roc-host-cargo][^host-rs][^efficient-plan]

Publishing already chose musl process binaries for islands. This WASI path
is a separate product, not a substitute for that default.[^efficient-plan]

## Solution paths (extra Rust)

### A. WASI-HTTP adapter host (recommended if the goal is a WASI interface)

New Rust crate (Rocci-owned or offered upstream), compiled to
`wasm32-wasip2` **or** run as a native embedder that loads Roc wasm:

- Implement `wasi:http` `handle`.
- On first request (or `_initialize`): `roc_init_for_host`, ignore listen
  host/port, keep context in instance state.
- Translate request → `respond!` → ordinary response or SSE body stream.
- Implement a **subset** of hosted functions: Env, Path, File, Stderr,
  clocks, sleep, optional wasm sqlite.
- Stub or omit Cmd, raw TCP, in-guest TLS client.

Linking options:

1. **Rust is the wasm component; Roc is a linked object.** `roc build
   --target=wasm32 --no-link` (or equivalent) produces app object code;
   `cargo component` / `wit-bindgen` crate links it and exports WIT. This
   matches how native `libhost.a` + app works, with Rust as the final
   linker.
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

## Minimal first slice (if a plan is written later)

Bound for a probe, not this record:

1. Hello `respond!` → 200 HTML through `wasmtime serve` (or a native
   Wasmtime HTTP embedder).
2. Streaming SSE unfold with `Wait` mapped to WASI clocks, enough for
   generated `patch_html!` / `empty_sse!`.
3. Preopened directory for one `file_root` static mount.
4. Optional: sqlite wasm or host-backed db for one example app.

Out of a first slice: Cmd, outbound HTTPS, HTTP/2 inside the guest,
Hyper compression parity, `rocci run` UX, changing `--host wasm`.

## Recommendation

Treat **A** as the only path that matches "WASI interface." The missing
Rust is a small HTTP adapter plus WASI-backed hosted effects, not a
retarget of `http_server.rs`. Keep native 0.16 for `rocci run`. Do not
overload apply `--host wasm`. Upstream a `wasm32` target only after the
adapter ABI is proven against hello-web and one SSE route.

Publishing Phase 6 already recorded a product no-go for replacing musl
islands with Wasm. This research does not reopen that default; it
explains what a later WASI-HTTP platform would have to build.[^efficient-plan]

[^bws-main]: No `wasm32` in `targets:`; hosted table includes sqlite, tcp, http_send, cmd, files.
[^bws-cargo]: `staticlib`; Tokio multi-thread; Hyper server+client; rustls/ring; bundled sqlite.
[^bws-readme]: Documented triples are native only; Hyper/Tokio; apps supply init/respond/shutdown.
[^bws-lib]: `main` → `http_server::start`.
[^bws-http]: Multi-thread runtime, FixedExecutor, Brotli threads, watchdog thread.
[^bws-exec]: Worker threads and condvar admission; not WASI-safe.
[^bws-sqlite]: `libsqlite3_sys` connection pool.
[^dispatch-rs]: Generated platform URL 0.16.0; listen, file roots, `Sse.unfold!`.
[^serve-rs]: `ROC_BASIC_WEBSERVER_HOST` / `PORT` helpers.
[^roc-host-readme]: Wasm host is apply evaluation, not HTTP.
[^wasm-platform]: `main!` only; wasm32 inputs are `host.o` + app.
[^host-c]: Allocator + `roc_main`; no sockets or HTTP.
[^host-rs]: Preview1 `_start`; no `wasi:http` linker.
[^roc-host-cargo]: `wasmtime` + `wasmtime-wasi` only.
[^efficient-research]: Runtime HTTP not available; WASI-HTTP would be a new platform.
[^efficient-plan]: Phase 6 no-go; musl remains the process story.
[^generation-research]: `roc build --target wasm32` for render WASI modules.
[^wasi-http-03]: `wasi:http/service` exports async `handle`.
[^wasmtime-serve]: `wasmtime serve` for 0.3 service or 0.2 proxy.
[^wasmcloud-wasi]: Tokio wasip2 is sockets; HTTP components use `wstd` / incoming-handler.
[^wit-bindgen]: Core wasm + preview1 adapter → component.
[^roc-wasi]: Example WASI command platform, not a webserver.
