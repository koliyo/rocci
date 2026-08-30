---
type: Status
title: WASI HTTP module PR maturity
description: "Open PR 82 on basic-webserver-wasi is an experimental but capability-complete WASI 0.3 http/service path for compiled .rocci apps. It is not merge-ready: dirty vs main, no CI, fork checkout required. Still omitted by design: Cmd, in-guest TLS, desktop URL."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-08-30T18:55:00Z }
stale_after: 2026-11-30
authority: descriptive
owners: [human:nils]
sources:
  - id: pr-82
    resource: https://github.com/koliyo/rocci/pull/82
    title: feat(rocci-wasi-http) compile a Rocci app into a WASI 0.3 HTTP module
    author: human:nils
    last_modified: 2026-08-30
  - id: pr-head
    resource: https://github.com/koliyo/rocci/commit/3792e3c3ce33bbec27ca77e6be59396865753f56
    title: Serve extracted CSS, request bodies, and sidecar assets
    author: process:git
    last_modified: 2026-08-30
  - id: adapter-plan
    resource: ../plans/rocci/basic-webserver-wasi.md
    title: Embedder-first WASI HTTP adapter; Phases 0–6 on this branch
    author: process:cursor
    last_modified: 2026-08-30
  - id: adapter-research
    resource: ../research/rocci/basic-webserver-wasi.md
    title: Gap analysis; yield around Roc; option 1 plus app-link disposition
    author: process:cursor
    last_modified: 2026-08-30
  - id: app-plan
    resource: ../plans/rocci/wasi-http-03-app.md
    title: App-link plan Phases 0–8 recorded
    author: process:cursor
    last_modified: 2026-08-30
  - id: component-plan
    resource: ../plans/rocci/wasi-http-03-component.md
    title: Portable 0.3 component plan Phases 0–8 recorded
    author: process:cursor
    last_modified: 2026-08-30
  - id: crate-readme
    resource: ../../crates/rocci-wasi-http/README.md
    title: Experimental --http-module; wasmtime serve -Sp3 -Scli
    author: process:git
    last_modified: 2026-08-30
  - id: component-readme
    resource: ../../crates/rocci-wasi-http-component/README.md
    title: wasip3 pins; sqlite-in-component; assets preopen
    author: process:git
    last_modified: 2026-08-30
  - id: cli-ref
    resource: ../../docs/reference/cli.rocdown
    title: Public --http-module copy
    author: process:git
    last_modified: 2026-08-30
  - id: fork-pr
    resource: https://github.com/koliyo/roc-basic-webserver/pull/1
    title: Merged wasm32 target and host.o for Rocci WASI HTTP
    author: human:nils
    last_modified: 2026-08-30
  - id: efficient-plan
    resource: ../plans/rocdown/efficient-publishing.md
    title: Musl remains the process publish path; Wasm remains apply
    author: process:cursor
    last_modified: 2026-08-30
---

# WASI HTTP module PR maturity

## Snapshot date

2026-08-30. Head of [PR 82](https://github.com/koliyo/rocci/pull/82) is
`3792e3c3` on branch `basic-webserver-wasi`. This is **not** shipped on
`main`. Main still has only the original adapter plan and gap
research.[^pr-82][^pr-head][^adapter-plan][^adapter-research]

## Verdict

The **architectural solution is mature** for the capability ladder the
plans named: a Rocci-owned WASI 0.3 `wasi:http/service` component, Rust
as the linker, Roc as the 0.16 C-ABI object, yield-around-Roc for SSE
`Wait`. Counter and live-counter have been served under `wasmtime serve
-Sp3 -Scli`. That is experimental product, not a replacement for
`rocci run` or musl publish.[^app-plan][^component-plan][^crate-readme][^efficient-plan]

The **PR is not merge-ready**. `mergeable_state` is `dirty` against
current `main`, GitHub reports no check runs, the PR test-plan boxes
are unchecked, and there are no reviews. Operator setup still needs a
sibling `../roc-basic-webserver` (or `ROCCI_BASIC_WEBSERVER`) and
Wasmtime CLI 48.[^pr-82][^fork-pr][^component-readme]

Do not read the first wave of `complete Phase N` commits as the current
design. Those (2026-08-29 morning) were the native embedder / stub
path. The second stack on the same branch is the one HEAD implements:
[0.3 component](https://github.com/koliyo/rocci/blob/basic-webserver-wasi/knowledge/plans/rocci/wasi-http-03-component.md)
then [app link](https://github.com/koliyo/rocci/blob/basic-webserver-wasi/knowledge/plans/rocci/wasi-http-03-app.md).[^pr-82][^component-plan][^app-plan]

## What shipped on the PR

| Surface | State on `3792e3c3` |
| --- | --- |
| Native embedder (`rocci-wasi-http` + Wasmtime 47) | Probe overlap table; hello-web; SSE Wait overlap; one preopen; sync rusqlite serializes |
| Portable component (`rocci-wasi-http-component`) | `wasm32-wasip2` + `wasip3` 0.8.0; `wasi:http@0.3.0` world `service`; `handle: async func` |
| `rocci build --http-module App.rocci` | Lowers the app, `roc build --target=wasm32` against the fork, links the object |
| Counter | `GET /` plus increment/reset sqlite morphs under `wasmtime serve` |
| live-counter | Compiled `/sse`; Wait is adapter clocks; two `/sse-wait` connections overlap |
| Sidecar assets (post Phase 8) | `<dest>.assets/` (Datastar, extracted `@css` as `/assets/rocci.css`); missing preopens 404 |
| Request bodies / named sqlite params | Hosted so live-counter increment does not trap |
| `rocci run` / `--host wasm` / musl | Unchanged by design |

Serve shape:[^cli-ref][^crate-readme][^pr-head]

```sh
rocci build --http-module examples/rocci/standalone/counter/Counter.rocci \
  -o http-module.wasm
wasmtime serve -Sp3 -Scli \
  --env DB_PATH=./counter.db \
  --dir=.counter-data::. \
  --dir=./http-module.assets::/assets \
  http-module.wasm
```

Sibling fork `koliyo/roc-basic-webserver` [PR 1](https://github.com/koliyo/roc-basic-webserver/pull/1)
merged a thin `wasm32` target and `scripts/build_wasm32_object.py`. That
is not an upstream `roc-lang/basic-webserver` PR. `--no-link` is absent
on the Roc nightly; the fork captures `roc_app_llvm_wasm32_speed.o` and
relinks `--export=roc_*_for_host`.[^fork-pr][^app-plan]

## How mature the solutions are

**Proven (keep):**

- Portable contract is WASI HTTP, not guest sockets. Runtime binds;
  `Server.Config.with_listen` is ignored.[^adapter-research][^component-plan]
- Option 1 linking: Rust is the component; Roc stays `roc_init_for_host`
  / `roc_respond_for_host` / `roc_sse_advance_for_host`. Do not wait on
  Roc language async or Canonical ABI lazy lowering.[^app-plan][^adapter-research]
- Yield around Roc. SSE `Wait` is adapter clocks. Nested `hosted_*`
  inside `respond!` (sync sqlite, hosted sleep) **serializes** other
  `handle`s. Phase 0 measured that; later sqlite-in-component did not
  overturn it.[^adapter-research][^component-readme]
- 0.3 `service` on `wasm32-wasip2` (no prebuilt `wasm32-wasip3` std).
  `-Scli` is leftover `wasi:cli@0.2.9` from Rust std, not a silent 0.2
  `proxy` retarget.[^component-readme]

**Working but operator-heavy:**

- Product flag requires Roc on PATH, the fork checkout, zig-built
  `sqlite3.o`, and Wasmtime CLI 48 while the native embedder crate still
  pins Wasmtime 47.[^component-readme][^crate-readme]
- File sqlite needs WASI `--dir` plus `DB_PATH`; hosted open forces
  DELETE journal and URI `nolock=1`.[^app-plan]
- Default crate tests must not require Roc; the real `.rocci` proofs are
  `ROCCI_REQUIRE_ROC=1` / `#[ignore]` plus documented curls.[^app-plan]

**Not a solution (and should stay that way unless a new plan says so):**

- Compiling 0.16 Hyper/Tokio/`ring` to wasip2 sockets.
- Overloading `--host wasm` (apply `main!`).
- Replacing musl island publish.[^efficient-plan]

## What is missing

### Merge and verification (blocks landing)

- Rebase onto current `main` (PR base is `97b2e6ea`; `main` has moved,
  including application-structure).[^pr-82]
- Run the PR test plan and tick it: `cargo test -p rocci-wasi-http`,
  `cargo test -p rocci-cli --bin rocci`, overlapping-SSE test,
  `cargo fmt --all -- --check`, `okmate check knowledge --profile base`,
  Counter and live-counter `wasmtime serve` curls.[^pr-82]
- Hosted CI and Knowledge workflow success. Plans on the branch say not
  to log complete until those run IDs exist.[^app-plan][^component-plan]
- No GitHub review yet.[^pr-82]

### Product gaps the plans already named (out of bound unless asked)

| Gap | Why it is still open |
| --- | --- |
| Cmd | Not in `wasi:http/service`; generated apps do not need it |
| In-guest TLS / `Http.send!` | Avoid `ring`; outbound client would be `wasi:http` `send` |
| Desktop preview URL | Would be a `wasmtime-wasi-http` embedder in `rocci run`; not started |
| `roc-lang` wasm32 PR | Fork is enough to learn; upstream only if asked |
| Nested sqlite yield | Sync C; fibers do not park; document serialization |

### Follow-on bugs already closed after Phase 8 knowledge

HEAD after the recorded app-link Phase 8: request body host, named
sqlite params, extracted `@css` as `/assets/rocci.css`, 404 for missing
preopens so a `.map` 500 no longer kills the worker. Those are on
`3792e3c3`, not in the Phase 8 knowledge commit.[^pr-head][^cli-ref]

### Knowledge vs `main`

This snapshot lives on `basic-webserver-wasi` with the three WASI plans.
`main` still has only the original adapter plan (no phase started there).
Landing the PR brings those plans plus this status onto `main`.[^adapter-plan]

## Next steps

Ordered. Do not start a new implementation plan for Cmd/TLS/desktop
unless the maintainer asks.

1. **Rebase** `basic-webserver-wasi` onto `main` and resolve the
   knowledge-index conflicts (PR already rewrote the WASI bullets).
2. **Prove HEAD locally** with the PR test plan, including Counter and
   live-counter under `wasmtime serve` with the assets `--dir`.
3. **CI + Knowledge** on that revision; then log the three plans
   complete with run IDs.
4. **Land as experimental.** Keep `rocci run` native. Do not flip
   publishing off musl.[^efficient-plan]
5. **Operator note** in crate README (already present on the branch):
   sibling fork or `ROCCI_BASIC_WEBSERVER`, Wasmtime 48, `-Sp3 -Scli`.
6. **Optional later:** align embedder Wasmtime 47 with CLI 48; desktop
   URL; upstream `roc-lang` wasm32 target. None of these are required to
   call the HTTP-module path "done" for the original ladder.

[^pr-82]: Open, not draft, 31 commits, 58 files; dirty vs main; empty checks/reviews; test plan unchecked.
[^pr-head]: Post-Phase-8: CSS sidecar, request bodies, named sqlite params, missing-preopen 404.
[^adapter-plan]: This branch recorded embedder Phases 0–6; `main` still has the original plan with no phase started.
[^adapter-research]: Yield-around-Roc; nested hosted I/O serializes; option 1 is the portable path.
[^app-plan]: Phases 0–8 recorded: object emit, fork wasm32, hello-web, env, sqlite, CLI, Counter, live-counter, docs.
[^component-plan]: Option 1 shipped experimental; remaining moved to the app-link plan and recorded there.
[^crate-readme]: `--http-module` compiles the `.rocci`; `--host wasm` stays apply; Cmd/TLS/desktop omitted.
[^component-readme]: Pins Wasmtime CLI 48 / wasip3 0.8.0 / `wasm32-wasip2`; sqlite-in-component serializes.
[^cli-ref]: Public CLI names `--http-module`, assets `--dir`, sqlite `DB_PATH`.
[^fork-pr]: Merged 2026-08-30; wasm32 `host.o` + object capture script; not `roc-lang`.
[^efficient-plan]: Publishing Phase 6 no-go; musl remains the process story.
