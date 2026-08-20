---
type: Implementation Plan
title: Efficient publishing workflow for pre-built Rocdown sites and Rocci apps
description: "Build once on a toolchain host, then host artifacts. Phases 1–5 implemented on this branch (static Caddy, package/serve, musl --target, slim hybrid and app images). Phase 6 is a no-go: musl remains the portable process story; Wasm remains apply. Exploratory until CI and Knowledge workflows succeed."
tags: [domain/rocdown, domain/rocci, concern/packaging, concern/publication, concern/architecture, integration/roc, concern/ci]
status: draft
generated: { by: process:cursor, at: 2026-08-20T07:22:49Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/efficient-publishing.md
    title: Efficient publishing of Rocdown sites and Rocci apps
    author: process:cursor
    last_modified: 2026-08-20
  - id: host-rs
    resource: ../../crates/rocci-roc-host/src/host.rs
    title: HostChoice auto native wasm
    author: process:git
    last_modified: 2026-08-18
  - id: dispatch-rs
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps use basic-webserver 0.16.0
    author: process:git
    last_modified: 2026-08-19
  - id: basic-cli-platform
    resource: https://github.com/roc-lang/basic-cli/blob/main/platform/main.roc
    title: basic-cli native targets without wasm32
    author: organization:roc-lang
  - id: roc-cross-ci
    resource: https://github.com/roc-lang/roc/blob/main/.github/workflows/ci_cross_compile.yml
    title: Roc compiler CI cross-compiles musl from macOS hosts
    author: organization:roc-lang
  - id: docker-readme
    resource: ../../docker/README.md
    title: Hybrid Docker images
    author: process:cursor
    last_modified: 2026-08-19
  - id: runtime-dockerfile
    resource: ../../docker/runtime/Dockerfile
    title: Ubuntu Roc and Rocdown runtime image
    author: process:cursor
    last_modified: 2026-08-19
  - id: compose
    resource: ../../docker/compose.yml
    title: Generic Caddy plus islands Compose file
    author: process:cursor
    last_modified: 2026-08-19
  - id: caddyfile
    resource: ../../docker/cdn/Caddyfile
    title: Same-origin Caddy reverse proxy for hybrid sites
    author: process:cursor
    last_modified: 2026-08-19
  - id: build-rs
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Rocdown site build, apply host, and publish report
    author: process:git
    last_modified: 2026-08-19
  - id: plan-rs
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Build planner, islands.json, publish pages
    author: process:git
    last_modified: 2026-08-19
  - id: service-rs
    resource: ../../crates/rocci-rocdown/src/service.rs
    title: serve-islands from sources
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-cli
    resource: ../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown command surface
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown site behavior
    author: process:git
    last_modified: 2026-08-19
  - id: bundle-rs
    resource: ../../crates/rocci-cli/src/bundle.rs
    title: macOS ad-hoc app bundle
    author: process:git
    last_modified: 2026-08-19
  - id: rocci-cli-readme
    resource: ../../crates/rocci-cli/README.md
    title: Base Rocci CLI contract
    author: process:git
    last_modified: 2026-08-19
  - id: hybrid-guide
    resource: ../../docs/guides/hybrid-sites.rocdown
    title: Hybrid CDN plus island-service operator guide
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-guide
    resource: ../../docs/guides/desktop-app.rocdown
    title: Package a desktop app
    author: process:git
    last_modified: 2026-08-19
  - id: cli-docs
    resource: ../../docs/reference/cli.rocdown
    title: Published CLI reference
    author: process:git
    last_modified: 2026-08-19
  - id: site-docs
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Published Rocdown site reference
    author: process:git
    last_modified: 2026-08-19
  - id: hosting-follow-ons
    resource: hybrid-island-hosting-follow-ons.md
    title: Hybrid island hosting follow-ons
    author: process:cursor
    last_modified: 2026-08-19
  - id: hybrid-plan
    resource: hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands for CDN-static sites
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-plan
    resource: rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: wasm-platform
    resource: ../../crates/rocci-roc-host/platform/main.roc
    title: Minimal wasm32 apply platform
    author: process:git
    last_modified: 2026-08-18
  - id: native-target-rs
    resource: ../../crates/rocci-cli/src/native_target.rs
    title: Native process --target for musl island and app binaries
    author: process:git
    last_modified: 2026-08-20
  - id: islands-dockerfile
    resource: ../../docker/islands/Dockerfile
    title: Slim hybrid island process image
    author: process:cursor
    last_modified: 2026-08-20
  - id: roc-host-readme
    resource: ../../crates/rocci-roc-host/README.md
    title: Native and wasm apply hosts
    author: process:git
    last_modified: 2026-08-18
  - id: publication
    resource: ../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: rocci-dev-site
    resource: rocci-dev-site.md
    title: rocci.dev site architecture
    author: process:codex
    last_modified: 2026-08-18
  - id: cli-plan
    resource: cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: docs-config
    resource: ../../docs/rocdown.toml
    title: rocci.dev site configuration
    author: process:git
    last_modified: 2026-08-19
---

# Efficient publishing workflow for pre-built Rocdown sites and Rocci apps

## Purpose and authority

This plan turns the [publishing research](../research/efficient-publishing.md)
into bounded delivery. It is exploratory until a human reviewer accepts a
scope. It does not describe shipped production packaging.[^research][^docker-readme]

Do not start a phase until the user asks.

The [hybrid island hosting follow-ons](hybrid-island-hosting-follow-ons.md)
keep WebKit-free CLIs, precompiled island binaries, and CORS. This plan owns
the **artifact hosting** workflow, starting with static trees, and the
packaging CLI that those later images consume.[^hosting-follow-ons][^research]

## Goal

Authors and CI compile Rocdown (and later Rocci apps) on a toolchain host.
Local Docker, and any later production host, run only the compiled artifacts.
Phase 1 supports **pre-built static Rocdown sites only**.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Build ≠ serve | Hosting images do not contain `rocci`, `rocdown`, `roc`, rustc, or WebKit. |
| Static first | Phase 1 does not run `serve-islands` or mount `.rocdown` sources as the serve input. |
| Two artifacts stay two | When `live` packaging lands, CDN tree and island process stay separate. Do not fold Caddy into the process image as the default.[^hybrid-guide][^hosting-follow-ons] |
| `--cdn-only` gate | A static publish errors on `live` pages (`RD2302`). No silent dead buttons.[^build-rs][^hybrid-plan] |
| Three CLIs | No plugin host, no `rocci deploy` adapter lifecycle. Packaging emits files; operators upload them.[^cli-plan][^rocci-dev-site] |
| `--host` stays dual | `rocdown build --host auto|native|wasm` is shipped and must remain. Native apply is not optional: `basic-cli` and `basic-webserver` have no `wasm32`.[^rocdown-cli][^host-rs][^basic-cli-platform] |
| `--host` ≠ `--target` | `--host` is how apply runs on the build machine. `--target` (not a flag today) is the native ISA/OS for process binaries that will run elsewhere. Do not pass musl `--target` into Mac apply.[^research][^host-rs] |
| Wasm is apply, not HTTP | `--host wasm` remains a renderer. Do not claim Wasm site hosting until a WASI-HTTP (or equivalent) platform exists.[^wasm-platform][^roc-host-readme][^generation-plan] |
| OKF stays local-first | Do not add a public knowledge deploy or verbatim bundle archive here.[^publication] |
| Hybrid Compose remains | Do not delete today's toolchain Compose until a replacement hybrid path exists. It is the live-island operator demo, not the static publish path.[^compose][^docker-readme] |

## Current evidence

`rocdown build --host auto|native|wasm` already selects the apply runtime.
Native uses `basic-cli` without `--target`. Wasm uses the custom wasm32
platform. `[build].host` and `ROCCI_HOST` are the same enum. The public
CLI reference does not yet document `--host`.[^rocdown-cli][^host-rs][^cli-docs][^build-rs]

`rocdown build` already writes a complete `dist/`. `inspect artifacts`
prints the publish report without Roc. `run` always rebuilds. There is no
command that serves an existing tree, and no archive of that tree.[^rocdown-cli][^build-rs][^rocdown-readme]

Local Docker rebuilds the site inside an Ubuntu image that installs Roc,
compiles the CLIs, and links WebKit, then compiles island `main.roc` at
start. That is why image builds are slow relative to hosting HTML.[^runtime-dockerfile][^compose][^docker-readme]

macOS `rocci bundle` already proves the app-side split: compile the Roc
server into the `.app`, run without `roc` on `PATH`. Linux, Windows, and
OCI are missing.[^bundle-rs][^rocci-cli-readme][^desktop-guide]

## Artifact model

```text
static / hydrate site  →  dist/          →  any static file server
live site              →  dist/ + island binary  →  Caddy + process
rocci app              →  server binary + assets →  .app / binary / OCI
OKF HTML               →  local preview only until publication reopens
```

`hydrate` is static after build. Wasm apply, when used, is a build-host
detail: the served bytes are still HTML.[^research][^build-rs]

## Delivery phases

Each phase is one mergeable change. Start only when asked.

### Phase 1 — Host a pre-built static Rocdown tree in Docker

**Bound:** local Compose (or equivalent) serves a host-built `dist/` with
official Caddy. No Rocci toolchain in that image. No island proxy.

**Does:**

- Add a static Caddyfile: `file_server` of `/srv` (or similar), hashed
  `/assets/` immutable cache, HTML `no-cache`, `try_files` for directory
  indexes. No `reverse_proxy` to `islands`.[^caddyfile][^hybrid-guide]
- Add `docker/compose.static.yml` (name bikesheddable) that uses
  `caddy:2-alpine`, bind-mounts `ROCCI_DIST`, and publishes 8080. Prefer
  **no custom image build**.
- Wrapper script analogous to `docker-serve-site.sh` that absolutizes `ROCCI_DIST`
  and runs Compose. Document: build on the host first.
- Dogfood `docs/` (`build.output = "../dist/docs"`): `rocdown build docs
  --cdn-only` then compose up.[^docs-config][^hybrid-plan]
- Rewrite `docker/README.md` so the static path is the default local
  **hosting** story. Keep the existing hybrid Compose as the live-island
  operator demo, labeled toolchain-heavy.[^docker-readme]
- Public hybrid guide: one paragraph that static sites use the new
  compose; hybrid still uses host preview plus the old file until Phase
  4.[^hybrid-guide]

**Does not:** compile in Docker; mount site sources; run `serve-islands`;
drop WebKit from the hybrid runtime image (that is hosting-follow-ons
Phase 1); publish to a registry; serve `live` pages.

**Owner:** `docker/` plus the hybrid guide and `docker/README.md`.

**Out of bound:** `rocdown` CLI changes; island binaries; app packaging.

**Tests / Exit:**

- `docker compose -f docker/compose.static.yml config` interpolates an
  absolute `ROCCI_DIST`.
- After a host `rocdown build docs --cdn-only`, compose up serves
  `http://127.0.0.1:8080/` with `rd-document` (or equivalent) in the
  homepage HTML. `/actions/` is **not** proxied (404 or Caddy's own
  response, not an island).
- `docker compose … build` is a no-op or only pulls `caddy`; it does not
  run `cargo` or install `roc`.
- `cargo test -p rocci-rocdown-cli` still passes (no CLI change required).

### Phase 2 — Package command and serve a built tree

**Bound:** first-class CLI for a static publish directory/archive and for
serving that tree without rebuilding.

**Does:**

- `rocdown package [ROOT]` (name bikesheddable) implies `--cdn-only`,
  writes `dist/` if needed, and emits `publish.json` beside it: page
  kinds and counts, Datastar false, `service_origin` empty, output hash
  or file list, `rocdown` / `roc` versions when known. Reuse
  `BuildReport` / `publish_report` rather than a second catalog
  walk.[^build-rs][^plan-rs][^rocdown-cli]
- Optional tarball or directory layout `site.tgz` whose root is the CDN
  tree plus `publish.json`. Failed package leaves the previous archive.
- `rocdown serve [DIST]` (or `rocdown run --from-dist`) serves an
  existing tree on loopback, no Roc, no watch rebuild. Preview window
  optional via existing `--no-window` rules. This is the no-Docker smoke
  path and a check that the archive is self-contained.[^rocdown-cli][^cli-plan]
- Update crate README, `docs/reference/cli.rocdown`, and
  `docs/reference/rocdown-site.rocdown`.[^cli-docs][^site-docs][^rocdown-readme]

**Does not:** upload to S3/Pages/Netlify; package `live` sites; change
apply hosts; add a fourth product CLI.

**Owner:** `crates/rocci-rocdown` and `crates/rocci-rocdown-cli`.

**Out of bound:** Docker; `rocci bundle`; OKF archives.

**Tests / Exit:**

- Package of a static fixture (or `docs/` reduced fixture) produces
  `publish.json` with zero `live` pages and Datastar false.
- Package of a `live` fixture fails with `RD2302`.
- `serve` of that `dist/` returns homepage HTML without invoking `roc`
  (PATH without `roc`, or a test spy).
- `cargo test -p rocci-rocdown` and `cargo test -p rocci-rocdown-cli`.
- `cargo fmt --all -- --check`.

### Phase 3 — Keep `--host`, add native `--target` for process binaries

**Gate:** Phase 1 is the documented static host path, and either (a) a
reviewer wants Linux island/app binaries produced on macOS, or (b)
hosting-follow-ons Phase 2 is about to land and needs a target triple.

**Bound:** `rocdown build` keeps `--host auto|native|wasm` for apply.
A separate native target flag (name bikesheddable: `--target`,
`--native-target`) is passed through to `roc build --target=` for
**island and app** `main.roc` only. Apply `--host native` always builds
a binary the current OS can exec. HTML output stays host-agnostic
regardless of process target.[^build-rs][^research][^host-rs][^roc-cross-ci]

**Does:**

- Document `--host` on the public CLI and site references. State that
  `--host wasm` does not produce a hosted Wasm server and that some
  platforms cannot compile to wasm, so native remains required.[^cli-docs][^basic-cli-platform]
- Pass `roc build --target=x64musl` (and `arm64musl` when the pinned
  platform tarball actually ships that host) for island/app process
  artifacts. On failure, error with the Roc output; do not fall back to
  a Mac binary silently.[^dispatch-rs][^roc-cross-ci]
- Record the triple in `publish.json` when a binary is emitted (Phase 4
  will emit it; this phase may land the flag on `rocci bundle`'s
  `build_roc_server` first).[^bundle-rs]
- Do not wait on `basic-cli` or `basic-webserver` `wasm32`.[^wasm-platform]

**Does not:** add WASI-HTTP; put `components.wasm` in `dist/` as a
runtime; change default local `run` to wasm; remove `--host native`.

**Owner:** `rocci-cli` driver / `rocci-rocdown` island compile, plus
docs.

**Out of bound:** Docker slim hybrid (Phase 4); desktop notarization.

**Tests / Exit:** `--host native` and `--host wasm` both still write
`dist/` on the build OS. A documented `--target x64musl` either produces
a Linux binary or fails clearly. README states musl is the Linux
container process target; `--host` is apply-only.

### Phase 4 — Slim Docker for precompiled hybrid sites

**Gate:** [hosting-follow-ons](hybrid-island-hosting-follow-ons.md) Phase 2
(precompiled island binary, no `roc` at runtime) has landed, or lands in
the same change set by explicit user request.[^hosting-follow-ons][^service-rs]

**Bound:** Compose for a **pre-built** hybrid: Caddy + a process image
that contains only the island binary, libc/SQLite as needed, and
`DB_PATH`. Site sources and `rocdown` are not in the process image.

**Does:**

- Process image from `scratch`, distroless, or `debian:bookworm-slim` —
  not Ubuntu+Roc+WebKit. Copy the musl (or Linux) island binary.
- Caddyfile can stay today's hybrid file; `dist/` is mounted read-only
  from the host build, same as Phase 1's static mount.[^caddyfile]
- `publish.json` records `live` routes and the binary fingerprint.
  Package of a hybrid site is allowed only when the binary is present.
- Document the two Compose files: static vs hybrid-prebuilt. Keep or
  retire the toolchain Compose; if kept, mark it **builder/dev**, not
  hosting.[^docker-readme][^compose]

**Does not:** compile Roc inside the hosting images; CORS (hosting-follow-ons
Phase 3); registry publish.

**Owner:** `docker/` plus package CLI; island exec stays in
`rocci-rocdown` as hosting-follow-ons specifies.

**Tests / Exit:** Counter (or equivalent) built on the host, compose up
without `roc` in the islands image, `GET /health` and increment POST
through 8080. Image history / `docker run … which roc` fails.

### Phase 5 — Rocci app Linux binary and OCI

**Bound:** `rocci` can emit a Linux server binary plus assets, and an
optional Dockerfile/Compose that runs that binary, without shipping the
`rocci` CLI.

**Does:**

- `rocci build --release` (or extend `bundle`) writes `server` + staged
  assets, not only a macOS `.app`. Reuse `build_roc_server` and asset
  copy.[^bundle-rs][^desktop-guide]
- Document a tiny image: copy `server` and `app/` resources, listen on
  `0.0.0.0` only when the operator sets the existing
  `ROC_BASIC_WEBSERVER_HOST` (do not weaken `rocci.toml` loopback
  validation for desktop configs).[^hosting-follow-ons]
- Keep macOS `.app` as the desktop path; this phase is **server
  packaging**, not notarization or extra windows.[^desktop-guide]

**Does not:** Windows MSI; auto-update; putting wry in the Linux server
image; a plugin deploy command.

**Owner:** `crates/rocci-cli` plus `docs/guides/desktop-app.rocdown` and
CLI reference.

**Tests / Exit:** `rocci build` of `examples/rocci/standalone/counter` (or datastar) on
Linux CI produces a binary that answers `GET /` without `roc` on PATH.
`cargo test -p rocci-cli`. Docs state Linux OCI is opt-in.

### Phase 6 — Wasm hosting research gate (not default work)

**Gate:** Native precompiled islands (Phase 4) are in use, **and** musl
cross-compile is insufficient (for example a desire to run the same
island artifact on macOS, Linux, and a Wasm edge host without rebuild).

**Bound:** decide whether to add a WASI-HTTP (or similar) Roc platform for
islands/apps. If no, close this phase with a short status note. If yes,
a **new** plan owns the platform; this phase only records the go/no-go.

**Does:**

- Compare: one `components.wasm`-class HTTP module vs musl binaries per
  arch; Wasmtime vs Wasm edge; SQLite/WASI capabilities.
- Do not extend the apply platform with fake HTTP.[^wasm-platform][^generation-plan]

**Does not:** implement the platform in this phase; change `--host wasm`
meaning for `rocdown build`.

**Exit:** A written go/no-go in this record or a follow-on plan. Default
if unstarted: musl remains the portable **process** story; Wasm remains
apply.

**Status (2026-08-20): no-go.** The Phase 6 gate is native precompiled
islands **and** musl cross-compile insufficient. Phase 4 packages a
colocated musl island binary into a Debian process image with no `roc`
at runtime.[^islands-dockerfile][^native-target-rs] That path is enough
for local hybrid hosting. There is no current need for one
`components.wasm`-class HTTP module on macOS, Linux, and a Wasm edge
host without rebuild.

Comparison recorded here, not as a new platform plan:

- **musl per arch** — `roc build --target=x64musl|arm64musl` is the Linux
  container process target. Failed musl builds do not fall back to a
  host-native binary.[^native-target-rs][^roc-cross-ci]
- **one Wasm HTTP module** — would need WASI-HTTP (or equivalent) plus
  SQLite/WASI capabilities. The apply wasm32 platform has no HTTP; do
  not extend it with fake HTTP.[^wasm-platform][^generation-plan]
- **Wasmtime vs Wasm edge** — Wasmtime stays the apply host. Edge Wasm
  hosting is not required while musl containers work.

Wasm remains apply (`--host wasm` is a renderer, not a hosted
server).[^host-rs][^wasm-platform] A new plan would own any later
WASI-HTTP platform; this phase does not implement it and does not
change `--host wasm`.

## Suggested order

1 → 2 → (3 as needed) → 4 (after hosting-follow-ons Phase 2) → 5.
Phase 6 only behind its gate.

Phase 1 is the entire first slice. Do not pull package CLI, hybrid
images, or app OCI into it.

Hosting-follow-ons Phase 1 (headless CLI, drop WebKit) still helps the
**toolchain** image if that image is kept as a builder. It is not
required for Phase 1 static hosting, because that path never uses those
binaries.[^hosting-follow-ons]

## Validation

Per phase as listed. Knowledge after record edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Do not log a phase complete until CI and Knowledge workflows succeed on
that revision.

## Out of scope

- Publishing images or site tarballs to a registry or CDN from Rocci.
- Deployment adapters (GitHub Pages, Netlify, Cloudflare, Tangled Sites).
- Reopening OKF public deployment.[^publication]
- `@island` grammar.
- Embedding the Roc compiler in Rust.
- Making `--host wasm` the default site build, or removing `--host native`.
- One kitchen-sink image that is both builder and production server.
- Production signing, notarization, and installers (desktop guide
  already names these as absent).[^desktop-guide]

## Non-goals of Phase 1

- Faster `cargo build` of `rocci-rocdown-cli` (layer caching of the fat
  image). The fix is not using that image to host static sites.
- Replacing `rocdown run` preview. Preview still rebuilds; publish
  serving does not.

[^research]: Artifact split; `--host` vs `--target`; native required; Mac apply cannot be musl.
[^docker-readme]: Current images build and serve hybrid sites from mounted sources.
[^runtime-dockerfile]: rustup, release CLIs, Roc nightly, WebKit.
[^compose]: `site-build` / `islands` / `cdn`; `ROCCI_SITE` sources.
[^caddyfile]: Hybrid reverse_proxy plus `file_server` of `dist/`.
[^build-rs]: `dist/` apply, `--cdn-only`, `--host wasm|native`, native apply omits `--target`.
[^plan-rs]: `publish_pages`, `islands.json`.
[^service-rs]: Islands still compile from site sources at start.
[^rocdown-cli]: `build --host auto|native|wasm`; no process `--target` or serve-from-dist today.
[^rocdown-readme]: Build and inspect artifacts contracts.
[^bundle-rs]: macOS-only bundle; compiled `server` inside the app.
[^rocci-cli-readme]: Bundle and playground wasm.
[^hybrid-guide]: Two-artifact production sketch; Compose is an operator check.
[^desktop-guide]: Roc-free macOS runtime; other OS packaging absent.
[^cli-docs]: Public `rocdown` command list omits `--host`.
[^site-docs]: `--cdn-only`, `inspect artifacts`, `islands.json`.
[^hosting-follow-ons]: Headless CLI, precompiled islands, CORS; not static-first Docker.
[^hybrid-plan]: CDN-only vs live; `docs/` static.
[^generation-plan]: Wasmtime apply host; later glue; no compiler embed.
[^wasm-platform]: No HTTP on the embedded wasm32 platform.
[^native-target-rs]: `x64musl` / `arm64musl` for island and app `roc build`; no host-native fallback.
[^islands-dockerfile]: Slim `debian:bookworm-slim` image copies the precompiled `islands` binary only.
[^roc-host-readme]: Native vs wasm apply; cached `apply` / `components.wasm`.
[^publication]: Knowledge stays local; no public archive.
[^rocci-dev-site]: No deploy-plugin product.
[^cli-plan]: Three CLIs; no plugin host.
[^docs-config]: `docs/` output `../dist/docs`.
[^host-rs]: `HostChoice::{Auto,Native,Wasm}`; native cache key is host ARCH.
[^dispatch-rs]: Generated HTTP apps pin basic-webserver 0.16.0.
[^basic-cli-platform]: Native targets only; no wasm32.
[^roc-cross-ci]: Compiler CI builds musl apps from macOS hosts.
