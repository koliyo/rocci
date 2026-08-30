---
type: Implementation Plan
title: Link a real Rocci app into the WASI 0.3 HTTP module
description: "Take rocci build --http-module from the hello-web stub to a compiled .rocci. The sibling koliyo/roc-basic-webserver fork supplies a wasm32 platform target; the Rocci component stays the wasi:http/service linker. Destination: wasmtime serve shows Counter, then a generated SSE app. Do not change --host wasm or rocci run."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-30T10:10:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/basic-webserver-wasi.md
    title: Option 1 shipped; remaining roc build object link and sqlite-in-component
    author: process:cursor
    last_modified: 2026-08-30
  - id: component-plan
    resource: wasi-http-03-component.md
    title: Portable component Phases 0–8; .rocci path is still CLI shape
    author: process:cursor
    last_modified: 2026-08-30
  - id: adapter-plan
    resource: basic-webserver-wasi.md
    title: Native embedder shipped; portable serve is the component plan
    author: process:cursor
    last_modified: 2026-08-30
  - id: cli-main
    resource: ../../../crates/rocci-cli/src/main.rs
    title: --http-module copies rocci-wasi-http-component bytes; ignores app body
    author: process:git
    last_modified: 2026-08-29
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps pin roc-lang basic-webserver 0.16.0
    author: process:git
    last_modified: 2026-08-22
  - id: component-lib
    resource: ../../../crates/rocci-wasi-http-component/src/lib.rs
    title: GET / is LinkedHelloWebGuest; other routes echo
    author: process:git
    last_modified: 2026-08-29
  - id: linked-rs
    resource: ../../../crates/rocci-wasi-http/src/linked.rs
    title: Rust roc_* stubs emit fixed hello-web HTML
    author: process:git
    last_modified: 2026-08-29
  - id: hello-wat
    resource: ../../../crates/rocci-wasi-http/src/hello_web.wat
    title: Fixture export names only; not a compiled app
    author: process:git
    last_modified: 2026-08-29
  - id: crate-readme
    resource: ../../../crates/rocci-wasi-http/README.md
    title: --http-module writes the 0.3 component; rocci run stays native
    author: process:git
    last_modified: 2026-08-29
  - id: cli-ref
    resource: ../../../docs/reference/cli.rocdown
    title: Public copy says .rocci is CLI shape until a Roc object link
    author: process:git
    last_modified: 2026-08-29
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Destination app; Env, Path, Sqlite, Stderr
    author: process:git
    last_modified: 2026-08-30
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Apply wasm32 is main! plus host.o; not HTTP
    author: process:git
    last_modified: 2026-08-18
  - id: roc-pin
    resource: ../../../docs/inventory.toml
    title: Rocci product nightly is nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-23
  - id: efficient-plan
    resource: ../rocdown/efficient-publishing.md
    title: Musl remains the process story; Wasm remains apply
    author: process:cursor
    last_modified: 2026-08-28
  - id: fork
    resource: https://github.com/koliyo/roc-basic-webserver
    title: Configured sibling fork; native targets only; no wasm32
    author: human:nils
  - id: bws-main
    resource: https://raw.githubusercontent.com/roc-lang/basic-webserver/0.16.0/platform/main.roc
    title: 0.16 hosted table and native-only targets
    author: organization:roc-lang
  - id: wasi-http-03
    resource: https://wasi.dev/releases/wasi-p3
    title: WASI 0.3 handle is async func
    author: organization:bytecode-alliance
---

# Link a real Rocci app into the WASI 0.3 HTTP module

## Goal

`rocci build --http-module examples/rocci/standalone/counter/Counter.rocci`
writes a WASI 0.3 `wasi:http/service` component whose `GET /` is the
Counter page and whose increment/reset routes mutate sqlite.
`wasmtime serve -Sp3 -Scli` is the host. A later phase does the same
for a generated SSE app (`live-counter`).[^counter][^cli-main][^wasi-http-03]

Rust stays the component. Roc stays the 0.16 C-ABI object
(`roc_init_for_host` / `roc_respond_for_host` /
`roc_sse_advance_for_host` / `roc_shutdown_for_host`). The sibling
checkout `../roc-basic-webserver` (`koliyo/roc-basic-webserver`) is the
wasm32 platform source. `rocci run` keeps the published 0.16 URL.
`--host wasm` stays apply `main!`.[^research][^component-plan][^dispatch-rs][^fork][^wasm-platform]

## Why a new plan

The [0.3 component plan](wasi-http-03-component.md) shipped the portable
artifact and left two holes: sqlite-in-component, and a real
`roc build --target=wasm32 --no-link` object. `--http-module` still
copies `rocci-wasi-http-component` bytes. `GET /` is
`LinkedHelloWebGuest`, a Rust stub that always emits hello-web HTML.
The `.rocci` path is CLI shape only.[^component-lib][^linked-rs][^cli-ref]

Earlier plans treated an upstream fork as out of bound unless asked.
That checkout is now configured. This plan uses it. It does not open a
`roc-lang/basic-webserver` PR unless the maintainer asks.[^adapter-plan][^fork]

## Out of bound

- Overloading `--host wasm` (apply `main!`).[^wasm-platform]
- Replacing `rocci run` or musl process publish.[^efficient-plan]
- Compiling the fork's Hyper/Tokio/`ring` host to `wasm32-wasip2`.
- Changing generated dispatch / `Sse.unfold!` / `empty_sse!` for native 0.16.
- Changing `dispatch.rs` `PLATFORM` for `rocci run`.[^dispatch-rs]
- Roc language async, async `respond!`, or a new Roc ABI.
- Cmd, raw TCP, in-guest TLS / `Http.send!`.
- Desktop preview URL over `wasmtime-wasi-http`.
- A public `roc-lang/basic-webserver` PR (work stays on the koliyo fork
  until asked).[^fork]
- Silently bumping Rocci's product nightly to the fork pin. Phase 0
  records whether http-module builds must use a different `roc`.[^roc-pin]
- Pretending sync sqlite overlaps other `handle`s.

## Constraints that do not move

- Portable contract is WASI HTTP. The runtime binds;
  `Server.Config.with_listen` is ignored.[^research][^wasi-http-03]
- Keep the 0.16 Roc C-ABI. Do not lower Rocci differently.[^dispatch-rs][^bws-main]
- Yield around Roc. SSE `Wait` stays adapter clocks.
- Linking shape stays option 1: Rust is the component; Roc is a linked
  object. Do not ship preview1+proxy wrapping as the product
  artifact.[^research]
- `--http-module` finds the fork via `ROCCI_BASIC_WEBSERVER` or a
  workspace-relative `../roc-basic-webserver`. Missing fork is a loud
  error, not a silent hello-web copy.[^fork]
- Generated `main.roc` for this flag points at the local fork, not the
  0.16.0 release tarball (that tarball has no wasm32 target).[^dispatch-rs]
- Default tests do not require Roc. `ROCCI_REQUIRE_ROC=1` / `#[ignore]`
  cover the object link.
- New workspace members are classified `BASE_ROCCI` in the same change.
- Do not pull Hyper/`ring` into the component crate.

## Current vs destination

| Surface | Now | Destination |
| --- | --- | --- |
| `rocci build --http-module App.rocci` | Copies prebuilt component; ignores App body[^cli-main] | Lowers App, `roc build --target=wasm32 --no-link` against the fork, links the object |
| `GET /` | Fixed hello-web HTML[^linked-rs][^component-lib] | App's `respond!` |
| Platform URL | Unused for this flag | Local fork for `--http-module` only |
| sqlite | Embedder-only; omitted from the component[^research] | Hosted sqlite Counter uses, sync, serializes |
| `rocci run` | Native 0.16.0 URL[^dispatch-rs] | Unchanged |

## Phase 0: Measure Roc wasm32 emit against the fork

**Bound:** On the maintainer machine, with `../roc-basic-webserver` and
the `roc` on PATH, record whether a 0.16-shaped app can emit
`roc_init_for_host` / `roc_respond_for_host` / `roc_shutdown_for_host`
as a `wasm32` object (`roc build --target=wasm32 --no-link` or the
nightly's equivalent). Add only a stub `targets.wasm32` row plus a
minimal `host.o` if the platform header refuses to compile without
one. Do not implement hosted I/O. Do not change Rocci CLI.[^research][^hello-wat][^fork][^wasm-platform]

Record: Roc nightly vs Rocci pin (`nightly-2026-08-23-fb208ba`) vs fork
pin (`nightly-2026-08-26-b29bef3`); object format; export names;
whether the earlier "header does not emit `roc_respond_for_host`"
finding still holds.[^roc-pin][^research]

**Out of bound:** linking into the component; Counter; sqlite.

**Tests:** none in Rocci. Write the measurement into this plan and the
research record.

**Exit:** A table of command, nightly, exports (present or absent), and
the chosen emit path for Phase 1. If emit is impossible on both
nightlies, stop and do not start Phase 1.

## Phase 1: wasm32 target and thin host.o in the fork

**Bound:** In `../roc-basic-webserver`, add a `wasm32` (or the name
Phase 0 recorded) target: `inputs: ["host.o", app]`. `host.o` is
allocator plus hosted stubs the component will satisfy. Native
`libhost.a` / Hyper / Tokio / `ring` stay off this target. Do not
regenerate glue unless the hosted block must change (prefer existing
names).[^fork][^bws-main][^wasm-platform]

**Out of bound:** Rocci CLI; serving Counter; compiling Hyper to wasm.

**Tests:** fork-local: `roc build --target=wasm32 --no-link
examples/hello-web.roc` writes an object whose exports include the
three `roc_*_for_host` names Phase 0 requires.

**Exit:** That object exists and `wasm-tools` / `llvm-nm` (or
equivalent) shows those exports.

## Phase 2: Link the hello-web Roc object into the component

**Bound:** `rocci-wasi-http-component` (or a build script it owns)
links the Phase 1 object. `hosted_emit_ordinary` and alloc stay in
Rust. Remove the product path that defines `roc_respond_for_host` in
`linked.rs` once the object supplies it (keep WAT as a fixture if
tests still need it). `GET /` under `wasmtime serve` is the Roc
hello-web HTML, not the Rust constant.[^linked-rs][^component-lib][^hello-wat]

**Out of bound:** `--http-module` compiling an arbitrary `.rocci`;
sqlite; Env.

**Tests:** documented serve `GET /` matches hello-web bytes from the
Roc app. Native embedder tests stay green.
`cargo fmt --all -- --check`.

**Exit:** Serve proof uses Roc object export names.

## Phase 3: Hosted Env, Path, and Stderr

**Bound:** Implement the hosted symbols Counter will call before
sqlite: `hosted_env_var`, Path helpers the object imports, and
`hosted_stderr_line` / write. Use WASI env, preopen, and
`wasi:cli` stderr. A Roc fixture that reads an env var and logs still
returns ordinary HTML.[^counter][^bws-main]

**Out of bound:** sqlite; Cmd; changing generated dispatch.

**Tests:** serve a linked fixture: env present → 200 body mentions the
value; stderr line appears on the serve host. `cargo fmt --all -- --check`.

**Exit:** Those proofs pass.

## Phase 4: Sqlite hosted in the component

**Bound:** One compile path that implements the `hosted_sqlite_*`
names Counter uses (`open`, `execute` / prepare-and-step, `query`)
inside the wasm component. Prefer wasi-sdk / `WASI_SYSROOT` so
`libsqlite3-sys` or the fork's sqlite C can target `wasm32-wasip2`.
If that still fails, a wasm-safe sqlite that honors the same C-ABI
names is in bound. Sync queries serialize other `handle`s; say so.
Do not depend on the native `embedder` rusqlite feature from the
component crate. Do not pretend fibers yield sqlite.[^research][^counter]

**Out of bound:** 0.16 connection-pool parity; WAL-on-network-fs;
`Http.send!`.

**Tests:** serve or component test: request that reads sqlite returns
200 with the row. Native serialize test stays green.
`cargo fmt --all -- --check`.

**Exit:** 200 from sqlite-in-component. A skip is not an Exit.

## Phase 5: `--http-module` compiles the input `.rocci`

**Bound:** `rocci build --http-module INPUT.rocci` lowers INPUT,
generates `main.roc` with `pf: platform` pointing at the fork, runs
`roc build --target=wasm32 --no-link`, links the object into the
component, writes `-o`. Missing fork or missing `roc` is an error, not
a hello-web copy. Help, crate README, and `docs/reference/cli.rocdown`
say the `.rocci` is the app. `--host wasm` stays apply.
`rocci run` still uses the 0.16.0 URL.[^cli-main][^cli-ref][^dispatch-rs][^crate-readme]

Use a crate or examples fixture that does **not** need sqlite so this
phase is not blocked on Counter. Default tests parse/help only.
`#[ignore]` + `ROCCI_REQUIRE_ROC=1` builds the fixture and checks two
different `.rocci` inputs produce different `GET /` bodies.

**Out of bound:** replacing preview UX; publishing musl images.

**Tests:** `cargo test -p rocci-cli --bin rocci` (help names compiled
app / `wasmtime serve`). Ignored Roc test as above.
`cargo fmt --all -- --check`.

**Exit:** `wasmtime serve` on CLI output for the fixture is that
fixture's HTML, not hello-web.

## Phase 6: Counter serve proof

**Bound:**
`rocci build --http-module examples/rocci/standalone/counter/Counter.rocci`
produces a component that serves the Counter page. Document
`wasmtime serve -Sp3 -Scli --dir=…` and `DB_PATH` (WASI env). `GET /`
is the counter card. `POST /actions/counter/increment` and
`/actions/counter/reset` change the stored count. Example README
gains the serve commands; `rocci run` line stays.[^counter]

**Out of bound:** live SSE; desktop URL; Cmd.

**Tests:** documented curls (or an ignored Roc test) for `/`,
increment, reset. `cargo fmt --all -- --check`.

**Exit:** Those three routes match native `rocci run` behavior for
status and the count morph.

## Phase 7: Generated SSE app from a real Roc object

**Bound:** `--http-module` on `live-counter` (or the current standalone
live example) links `roc_sse_advance_for_host` from the object. Wait
stays adapter clocks. Overlapping two `/sse` connections must not
serialize on Wait. Idle timeout stays the serve host's.[^component-plan][^research]

**Out of bound:** changing keepalive policy; HTTP/2.

**Tests:** documented two-connection serve, or ignored Roc test.
Native Wait-overlap test stays green. `cargo fmt --all -- --check`.

**Exit:** Live page patches from the compiled app, not `WaitEmitGuest`.

## Phase 8: Knowledge and public docs

**Bound:** Research remaining becomes "done vs still omitted" (Cmd,
TLS, desktop URL). This plan status. Parent adapter and component
plans point here. Indexes. Public CLI page and crate READMEs match.
Upstream offer unchanged: no `roc-lang` PR unless asked.[^research][^component-plan][^adapter-plan]

**Exit:** `okmate check knowledge --profile base --format terminal`.
Crate READMEs and the CLI page agree.

## Disposition (start of plan)

| Item | State |
| --- | --- |
| Portable 0.3 component | Shipped experimental (hello-web stub) |
| Real `roc build` object | Not shipped; this plan |
| `--http-module` uses `.rocci` body | Not shipped; Phase 5 |
| sqlite-in-component | Not shipped; Phase 4 (required for Counter) |
| Counter under `wasmtime serve` | Not shipped; Phase 6 |
| Generated SSE from Roc object | Not shipped; Phase 7 |
| `rocci run` / `--host wasm` / musl | Unchanged |
| Fork wasm32 target | Not shipped; Phase 1 in `../roc-basic-webserver` |
| `roc-lang` PR | Not opened |

## Suggested command surface (after Phase 6)

```sh
# Fork (Phase 1)
roc build --target=wasm32 --no-link examples/hello-web.roc

# Product
rocci build --http-module examples/rocci/standalone/counter/Counter.rocci \
  -o http-module.wasm
wasmtime serve -Sp3 -Scli --dir=. http-module.wasm
# WASI env: DB_PATH=./counter.db
curl -s http://127.0.0.1:8080/
curl -s -X POST http://127.0.0.1:8080/actions/counter/increment
```

## Non-goals that stay with earlier plans

Native embedder tests, the 200ms probe table, and nested-C
serialization honesty stay in `rocci-wasi-http` with `embedder`.
This plan does not delete them.[^adapter-plan]

[^research]: Option 1 shipped; remaining object link and sqlite-in-component.
[^component-plan]: Phases 0–8 recorded; `.rocci` still CLI shape.
[^adapter-plan]: Native embedder; portable serve is the component plan.
[^cli-main]: `--http-module` copies component bytes.
[^dispatch-rs]: Generated apps pin 0.16.0; `rocci run` keeps that URL.
[^component-lib]: `GET /` routes to `LinkedHelloWebGuest`.
[^linked-rs]: Rust `roc_respond_for_host` emits hello-web HTML.
[^hello-wat]: Fixture export names, not a compiled app.
[^crate-readme]: `--http-module` writes the 0.3 component.
[^cli-ref]: Public copy says the bytes are not a compiled app.
[^counter]: Counter needs Env, Path, Sqlite, Stderr.
[^wasm-platform]: Apply wasm32 is `main!`.
[^roc-pin]: Rocci product nightly is `nightly-2026-08-23-fb208ba`.
[^efficient-plan]: Musl stays the process story.
[^fork]: Sibling `../roc-basic-webserver`; native targets only.
[^bws-main]: 0.16 hosted table; no wasm32 on the release platform.
[^wasi-http-03]: `handle` is `async func`.
