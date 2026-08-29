---
type: Implementation Plan
title: WASI 0.3 HTTP component for wasmtime serve
description: "Compile the existing rocci-wasi-http adapter as a portable WASI 0.3 wasi:http/service component so wasmtime serve accepts the artifact. Rust is the component; Roc stays a linked C-ABI object. Do not change --host wasm or rocci run."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-29T14:50:00Z }
stale_after: 2026-11-29
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/basic-webserver-wasi.md
    title: Gaps, option 1 (Rust component + Roc object), measured yield table
    author: process:cursor
    last_modified: 2026-08-29
  - id: adapter-plan
    resource: basic-webserver-wasi.md
    title: Embedder-first adapter plan; option 2 shipped, not a component
    author: process:cursor
    last_modified: 2026-08-29
  - id: sse-research
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: Native Wait vs Hyper 30s idle; adapter must not copy that timer
    author: process:cursor
    last_modified: 2026-08-21
  - id: efficient-plan
    resource: ../rocdown/efficient-publishing.md
    title: Phase 6 no-go; musl remains the process story; Wasm remains apply
    author: process:cursor
    last_modified: 2026-08-28
  - id: crate-readme
    resource: ../../../crates/rocci-wasi-http/README.md
    title: --http-module writes core wasm; wasmtime serve refuses it
    author: process:git
    last_modified: 2026-08-29
  - id: hello-wat
    resource: ../../../crates/rocci-wasi-http/src/hello_web.wat
    title: Core module exporting roc_*_for_host, not a WIT world
    author: process:git
    last_modified: 2026-08-29
  - id: handle-rs
    resource: ../../../crates/rocci-wasi-http/src/handle.rs
    title: Native Adapter handle; SSE Wait is tokio::sleep
    author: process:git
    last_modified: 2026-08-29
  - id: abi-rs
    resource: ../../../crates/rocci-wasi-http/src/abi.rs
    title: 0.16 request_to_roc field names and OutcomeToHost
    author: process:git
    last_modified: 2026-08-29
  - id: guest-rs
    resource: ../../../crates/rocci-wasi-http/src/guest.rs
    title: StubGuest, EmptySseGuest, WaitEmitGuest
    author: process:git
    last_modified: 2026-08-29
  - id: files-rs
    resource: ../../../crates/rocci-wasi-http/src/files.rs
    title: One preopen; reject parent escape
    author: process:git
    last_modified: 2026-08-29
  - id: sqlite-rs
    resource: ../../../crates/rocci-wasi-http/src/sqlite.rs
    title: Sync rusqlite; nested respond serializes
    author: process:git
    last_modified: 2026-08-29
  - id: crate-cargo
    resource: ../../../crates/rocci-wasi-http/Cargo.toml
    title: Native wasmtime 47 plus bundled rusqlite; no wit-bindgen
    author: process:git
    last_modified: 2026-08-29
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: --http-module writes hello-web WAT bytes via wat
    author: process:git
    last_modified: 2026-08-29
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: Public --http-module copy; core wasm not a component
    author: process:git
    last_modified: 2026-08-29
  - id: workspace-deps
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: New workspace crate must be classified in BASE_ROCCI
    author: process:git
    last_modified: 2026-08-26
  - id: cargo-toml
    resource: ../../../Cargo.toml
    title: Workspace members; rocci-wasi-http already listed
    author: process:git
    last_modified: 2026-08-29
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps pin 0.16 init/respond/SSE
    author: process:git
    last_modified: 2026-08-22
  - id: serve-rs
    resource: ../../../crates/rocci-cli/src/serve.rs
    title: rocci run is native listen env
    author: process:git
    last_modified: 2026-08-25
  - id: host-rs
    resource: ../../../crates/rocci-roc-host/src/host.rs
    title: Apply wasm is preview1 _start, not wasi:http
    author: process:git
    last_modified: 2026-08-23
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Apply wasm32 platform is main!
    author: process:git
    last_modified: 2026-08-18
  - id: wasi-http-03
    resource: https://wasi.dev/releases/wasi-p3
    title: WASI 0.3 native async; wasi:http/service handle async func
    author: organization:bytecode-alliance
  - id: wasi-async-cm
    resource: https://component-model.bytecodealliance.org/design/async.html
    title: Canonical ABI async func, stream, future
    author: organization:bytecode-alliance
  - id: wasmtime-serve
    resource: https://component-model.bytecodealliance.org/running-components/wasmtime.html
    title: wasmtime serve runs 0.3 service or 0.2 proxy
    author: organization:bytecode-alliance
  - id: wit-bindgen
    resource: https://github.com/bytecodealliance/wit-bindgen/blob/main/README.md
    title: Core wasm plus adapter becomes a WASI component
    author: organization:bytecode-alliance
  - id: cargo-component
    resource: https://github.com/bytecodealliance/cargo-component
    title: cargo component builds a WIT component from a Rust crate
    author: organization:bytecode-alliance
  - id: bws-main
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/main.roc
    title: 0.16 Roc API and hosted table
    author: organization:roc-lang
  - id: bws-http
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/src/http_server.rs
    title: Sync roc_respond_for_host; SSE Wait on Tokio
    author: organization:roc-lang
  - id: bws-sse-roc
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/Sse.roc
    title: WaitToHost after unfold; host owns the timer
    author: organization:roc-lang
---

# WASI 0.3 HTTP component for wasmtime serve

## Goal

`wasmtime serve <artifact.wasm>` accepts a Rocci-built **WASI 0.3**
`wasi:http/service` component and returns 200 HTML for `GET /`. The
component is **Rust**; Roc remains the 0.16 C-ABI object the adapter
already calls (`roc_init_for_host` / `roc_respond_for_host` /
`roc_sse_advance_for_host` / `roc_shutdown_for_host`).[^research][^wasmtime-serve][^wasi-http-03][^hello-wat]

This is research **option 1**. The [embedder plan](basic-webserver-wasi.md)
shipped option 2: a native `Adapter` plus `rocci build --http-module`
core wasm. That file is not a component; `wasmtime serve` refuses
it.[^adapter-plan][^crate-readme][^cli-main]

`rocci run` stays native 0.16. `--host wasm` stays apply. Musl publish
stays the default.[^efficient-plan][^serve-rs][^host-rs][^wasm-platform]

## Why a new crate split

`rocci-wasi-http` today is a **native** library: it depends on Wasmtime
47 (to *load* guests) and bundled rusqlite (to *prove* sqlite
serialize). A portable component **is** the guest. It must not link
Wasmtime as a runtime embedder, and it must compile to `wasm32-wasip2`
(or the 0.3 target Phase 0 records).[^crate-cargo][^research]

Reuse `abi`, `guest`, `handle`, `files` as `no_wasmtime` code. Keep the
native embedder and its tests behind a default `embedder` feature so
Phase 0–5 of the parent plan stay green.

## Out of bound

- Overloading `--host wasm` (apply `main!`).[^wasm-platform][^host-rs]
- Replacing musl process binaries as the default publish path.[^efficient-plan]
- Compiling the 0.16 Hyper/Tokio host to wasip2 sockets (guest bind is
  not WASI HTTP).[^research]
- Teaching the apply platform HTTP.
- Roc language async, async `respond!`, or a new Roc ABI.[^bws-http]
- Cmd, raw TCP, in-guest TLS / `ring` outbound client.
- HTTP/2, Brotli, or Hyper compression parity inside the guest.
- Shared-memory Wasm threads.
- Changing generated dispatch / `Sse.unfold!` / `empty_sse!`.[^dispatch-rs]
- Vendoring basic-webserver or opening an upstream fork PR unless the
  maintainer asks.
- Replacing `rocci run` / preview child-process UX / desktop URL.
- Silently switching the **target ABI** to WASI 0.2 `proxy` because 0.3
  tooling is inconvenient. 0.2 is a Phase 0 compatibility **note**
  only.[^wasmtime-serve]
- Pretending sync sqlite or `thread::sleep` inside `respond!` overlaps
  other `handle`s. Parent Phase 0 measured that they serialize.[^research][^sqlite-rs]

## Constraints that do not move

- Portable contract is **WASI HTTP**. The runtime binds;
  `Server.Config.with_listen` is ignored.[^research][^wasi-http-03]
- Prefer WASI 0.3 `handle: async func` and `stream<u8>` bodies.[^wasi-http-03][^wasi-async-cm]
- Keep the **0.16 Roc C-ABI**. Map WASI request/response onto the
  existing `IncomingRequest` / `OutcomeToHost` types. Do not lower Rocci
  differently.[^abi-rs][^bws-main][^dispatch-rs]
- **Yield around Roc.** `roc_respond_for_host` and
  `roc_sse_advance_for_host` stay synchronous. SSE `Wait` is adapter
  clocks (`clocks.wait-for` or equivalent), not `hosted_sleep_millis`
  inside `respond!`.[^handle-rs][^bws-sse-roc][^sse-research]
- New workspace members are classified `BASE_ROCCI` in the same
  change.[^workspace-deps][^cargo-toml]
- Linking shape is option 1: Rust is the component; Roc is a linked
  object. Do not ship preview1+proxy wrapping as the product
  artifact.[^research][^wit-bindgen]
- Hello-web, then SSE, then one `file_root`, then sqlite. Each of those
  phases must be runnable under **`wasmtime serve`** (not only the
  native embedder).[^adapter-plan]
- Do not pull Hyper/`ring` into the component crate.

## Current vs target artifact

| Artifact | What it is | `wasmtime serve` |
| --- | --- | --- |
| `rocci build --http-module` today | Core wasm from `hello_web.wat` (`roc_*_for_host`) | Refuses: not a component[^crate-readme][^hello-wat][^cli-main] |
| Native `Adapter` tests | Host Tokio + optional inner Wasmtime module | N/A (not `serve`)[^handle-rs] |
| This plan's output | WIT component exporting `wasi:http` `handle` | Required proof |

## Phase 0: Toolchain and empty 0.3 service

**Bound:** Record, with commands that a maintainer can re-run, the exact
path that produces a **WASI 0.3** `wasi:http/service` component
`wasmtime serve` accepts. Prefer `cargo component` / `wit-bindgen` on a
throwaway crate or a new empty `crates/rocci-wasi-http-component`.
Pin: Wasmtime CLI version, WIT package (`wasi:http@…`), world name
(`service` vs preview), Rust target triple, and whether `async func`
lifts compile on that toolchain.[^cargo-component][^wit-bindgen][^wasmtime-serve][^wasi-http-03]

The empty `handle` returns 200 `text/html` with a fixed hello-web body.
No Roc. No `rocci-wasi-http` feature split yet unless it unblocks the
compile.

If 0.3 `service` cannot be produced on the pinned Wasmtime 47 line,
**stop and amend the research**. Do not silently retarget to 0.2
`proxy` as success. A 0.2 note may be recorded as compatibility only.

**Out of bound:** linking Roc; SSE; sqlite; changing `--http-module`.

**Tests:** documented `wasmtime serve` + `curl` (or equivalent) `GET /`
→ 200 HTML. `cargo fmt --all -- --check` if Rust landed.

**Exit:** Research names the WIT world, versions, and the serve command
that succeeded. Classify any new workspace member in
`BASE_ROCCI`.[^workspace-deps]

**Phase 0 recorded (2026-08-29):** `crates/rocci-wasi-http-component`
(`BASE_ROCCI`) exports empty `wasip3::http::service` `handle`. Pins:
Wasmtime CLI **48.0.1**, `wasip3` **0.8.0+wasi-0.3.0** (`wit-bindgen`
0.61.1), WIT **`wasi:http@0.3.0` world `service`**
(`export wasi:http/handler@0.3.0`, `handle: async func`), Rust target
**`wasm32-wasip2`** on rustc 1.97.1 (no `wasm32-wasip3` prebuilt std).
`async func` lifts compile. Serve command that returned 200 HTML:

```sh
cargo build -p rocci-wasi-http-component --target wasm32-wasip2
wasmtime serve -Sp3 -Scli --addr 127.0.0.1:8080 \
  target/wasm32-wasip2/debug/rocci_wasi_http_component.wasm
```

`-Scli` is for Rust `std`'s leftover `wasi:cli@0.2.9` imports, not a
0.2 `proxy` retarget. `wstd` / `#[wstd::http_server]` is 0.2 `proxy`
and was not used. Native embedder crate still depends on Wasmtime 47.

## Phase 1: Component crate and mapping without Wasmtime

**Bound:** Add (or keep) `crates/rocci-wasi-http-component`, classified
`BASE_ROCCI`. Feature-split `rocci-wasi-http`:

- default / `embedder`: current Wasmtime + rusqlite + probe tests
- `map` (always on): `abi`, `guest` stubs, `handle::Adapter`, `files`

The component crate depends on `rocci-wasi-http` with
`default-features = false` and does **not** depend on `wasmtime`. It
exports 0.3 `handle` and calls `Adapter::handle` with `StubGuest`.
Buffer the WASI body before `map_request`. Ignore listen host/port.
`init` once per instance.[^abi-rs][^guest-rs][^handle-rs][^crate-cargo]

**Out of bound:** Roc object link; SSE stream; sqlite in the component.

**Tests:** `cargo test -p rocci-wasi-http` (embedder still green).
Component build command from Phase 0. `wasmtime serve` `GET /` → 200
and the stub HTML bytes. `cargo fmt --all -- --check`.
`uv run --directory tools/rocci-ops rocci-ops check deps`.

**Exit:** Those commands pass. Native embedder tests are unchanged.

## Phase 2: Ordinary WASI request fields

**Bound:** Map incoming WASI method, path, headers, and the **fully
buffered** body onto `IncomingRequest` / `ServerRequest` (same field
names as parent Phase 1). Assert in a component-side test or a small
`wasmtime serve` script that `GET /hello?x=1` and a POST body reach
`StubGuest` / a recording guest. Status, content-type, and body go out
on the WASI response.[^abi-rs]

**Out of bound:** streaming request bodies; Hyper.

**Tests:** crate or documented serve checks for path/query/headers/body.
`cargo fmt --all -- --check`.

**Exit:** Mapping parity with the native embedder tests
(`maps_get_path_query_and_headers`, `buffers_post_body`).

## Phase 3: Link the hello-web Roc guest

**Bound:** The component calls real `roc_*_for_host` symbols. First
source of those symbols may be `hello_web.wat` compiled into the
component (static link or instantiate-an-inner-module **inside** the
component — still one `wasmtime serve` artifact). Prefer a later
`roc build --target=wasm32 --no-link` object when that nightly emits
the 0.16 export names; do not block the phase on an unreliable Roc
wasm32 platform header. Hosted subset: alloc + the emit import the WAT
already uses (`hosted_emit_ordinary`).[^hello-wat][^research][^wit-bindgen]

**Out of bound:** changing `dispatch.rs`; Cmd; TLS.

**Tests:** `wasmtime serve` `GET /` returns the hello-web HTML bytes
(`<!doctype html><html><body>hello-web</body></html>`). Native
`linked_hello_web_get_root` stays green. `cargo fmt --all -- --check`.

**Exit:** Serve proof uses Roc export names, not only `StubGuest`.

## Phase 4: SSE as WASI streams and clocks

**Bound:** When `respond` returns `OutcomeToHost::Stream`, write a WASI
`stream<u8>` body. Loop `sse_advance` → Emit writes framed bytes →
Wait **awaits WASI clocks** for `wait_millis` (0 means immediately) →
End closes the stream. Do not use Hyper's 30s
`response_idle_timeout_ms`. Idle timeout is the `wasmtime serve`
host's.[^handle-rs][^bws-sse-roc][^sse-research][^wasi-async-cm]

Prove (a) `EmptySseGuest`-shaped immediate End and (b) Wait then one
Emit. Overlapping two `wasmtime serve` connections (or two in-component
`handle`s) must not serialize on Wait.

**Out of bound:** changing generated keepalive policy; HTTP/2.

**Tests:** component tests and/or documented two-connection serve.
Native `overlapping_sse_waits_do_not_serialize` stays green.
`cargo fmt --all -- --check`.

**Exit:** Those proofs pass. Research notes that live Wait overlap
still does not require async Roc, now under `wasmtime serve`.

## Phase 5: Preopen `file_root`

**Bound:** One preopen directory granted to the component (`wasmtime
serve --dir` or the 0.3 filesystem equivalent Phase 0 recorded). Map
`OutcomeToHost::File` through existing `resolve_preopen` (reject `..`).
Native OS paths outside the preopen are not available.[^files-rs]

**Out of bound:** writing arbitrary host paths; Cmd.

**Tests:** `GET` of `fixtures/static/hello.txt` via serve `--dir`
returns `preopen-bytes`. Native preopen tests stay green.
`cargo fmt --all -- --check`.

**Exit:** That GET returns the fixture bytes.

## Phase 6: SQLite that stays honest

**Bound:** One path sufficient for a notes-class query **inside the
component**, or a documented skip if `libsqlite3-sys` cannot compile
for the Phase 0 target without pulling `ring` / the native host crate.

Prefer a build that yields. If it is sync C (parent measurement), ship
it and document that queries serialize other `handle`s. Do not pretend
fibers yield sqlite. Do not depend on the native `embedder` rusqlite
feature from the component crate.[^sqlite-rs][^research][^crate-cargo]

**Out of bound:** 0.16 connection-pool parity; WAL-on-network-fs.

**Tests:** serve or component test: request that reads sqlite returns
200, **or** research + crate README record the compile blocker and
that sqlite remains embedder-only. `cargo fmt --all -- --check`.

**Exit:** Either 200 from sqlite-in-component, or an explicit
component-omits-sqlite note with the measured serialize fact unchanged.

## Phase 7: `--http-module` emits the component

**Bound:** `rocci build --http-module` writes the **Phase 3+**
component bytes (at least hello-web `handle`), not `hello_web.wat`
core wasm. `--help`, crate README, and `docs/reference/cli.rocdown`
agree: this is a WASI HTTP **component**; `wasmtime serve` is the
documented command; `--host wasm` is still apply; `rocci run` is still
native 0.16.[^cli-main][^cli-ref][^crate-readme][^serve-rs]

Optional: print the serve command after write. Do not add a desktop
preview URL unless it is a one-line `wasmtime serve` hint.

`#[ignore]` / `ROCCI_REQUIRE_ROC` may still exist for a future Roc
object link; default tests must not require Roc.

**Out of bound:** replacing preview UX; publishing musl images.

**Tests:** `cargo test -p rocci-cli --bin rocci` (parse + help names
component / `wasmtime serve`). A test writes the artifact and checks
the wasm component preamble / `wasm-tools component wit` (or
equivalent) — not only `\0asm`. `cargo test -p rocci-wasi-http`.
`cargo fmt --all -- --check`.

**Exit:** `wasmtime serve` on the CLI output succeeds for `GET /`.
Help no longer describes the file as core-only.

## Phase 8: Knowledge and public docs

**Bound:** Research disposition: option 1 shipped vs remaining
(sqlite-in-component, Roc `roc build` link). This plan status. Parent
adapter plan points here for `serve`. Indexes. Public CLI page matches
the flag. Upstream offer remains: wasm32 host artifact + WIT adapter;
no fork PR unless asked.[^research][^adapter-plan]

Do not log phase-complete with CI/Knowledge run IDs until those
workflows succeed.

**Exit:** `okmate check knowledge --format terminal` (use `--profile
strict` until a `rocci` profile exists on the installed okmate). Crate
READMEs and the CLI page agree with the research.

## Suggested command surface (Phase 0 fills versions)

```sh
# Phase 0 (empty service, no Roc)
cargo build -p rocci-wasi-http-component --target wasm32-wasip2
wasmtime serve -Sp3 -Scli --addr 127.0.0.1:8080 \
  target/wasm32-wasip2/debug/rocci_wasi_http_component.wasm
curl -i http://127.0.0.1:8080/
# After Phase 7
rocci build --http-module App.rocci -o http-module.wasm
wasmtime serve -Sp3 -Scli http-module.wasm
# Phase 5
wasmtime serve -Sp3 -Scli --dir=crates/rocci-wasi-http/fixtures/static http-module.wasm
```

`--http-module` may keep requiring a `.rocci` path for CLI shape even
when the bytes are still hello-web. Say so until a real Roc object
link lands.

## Non-goals that stay with the parent plan

Native embedder tests, the 200ms probe table, and the honesty about
nested C serialization remain in `rocci-wasi-http` with `embedder`.
This plan does not delete them.

[^research]: Option 1 vs 2; yield-around-Roc; capability ladder; no component yet.
[^adapter-plan]: Embedder phases; `--http-module` writes WAT; serve was an unmet original goal.
[^sse-research]: Do not copy Hyper 30s idle onto Wait.
[^efficient-plan]: Musl default; apply wasm is not HTTP.
[^crate-readme]: Serve refused; file is core wasm.
[^hello-wat]: `roc_*_for_host` exports only.
[^handle-rs]: Native SSE Wait is `tokio::sleep`.
[^abi-rs]: 0.16 field names; Stream/File outcomes.
[^guest-rs]: Stub and SSE guests.
[^files-rs]: `resolve_preopen` rejects `..`.
[^sqlite-rs]: Sync rusqlite serializes.
[^crate-cargo]: Wasmtime + rusqlite on the native crate.
[^cli-main]: `--http-module` + `wat::parse_str`.
[^cli-ref]: Public copy that serve will refuse.
[^workspace-deps]: Unclassified members fail CI.
[^cargo-toml]: Members list.
[^dispatch-rs]: Generated 0.16 contract.
[^serve-rs]: Native `rocci run`.
[^host-rs]: Preview1 apply runner.
[^wasm-platform]: Apply `main!`.
[^wasi-http-03]: `handle` is `async func`.
[^wasi-async-cm]: Runtime owns wake-up.
[^wasmtime-serve]: 0.3 service or 0.2 proxy.
[^wit-bindgen]: Component from core + adapter.
[^cargo-component]: Rust → WIT component.
[^bws-main]: 0.16 Roc API.
[^bws-http]: Sync `roc_respond_for_host`.
[^bws-sse-roc]: `WaitToHost` after unfold.
