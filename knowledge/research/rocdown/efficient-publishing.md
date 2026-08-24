---
type: Research Report
title: Efficient publishing of Rocdown sites and Rocci apps
description: "Evidence for a build-once, host-the-artifact publishing workflow. Local Docker should serve pre-built trees, not compile Rocci. Keep native apply and native process binaries; Wasm is optional apply only. Roc --target can emit Linux musl from macOS when the platform ships that host. Exploratory; not an approved product contract."
tags: [domain/rocdown, domain/rocci, concern/packaging, concern/publication, concern/architecture, integration/roc]
status: draft
generated: { by: process:cursor, at: 2026-08-20T05:40:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: docker-readme
    resource: ../../../docker/README.md
    title: Hybrid Docker images
    author: process:cursor
    last_modified: 2026-08-19
  - id: runtime-dockerfile
    resource: ../../../docker/runtime/Dockerfile
    title: Ubuntu Roc and Rocdown runtime image
    author: process:cursor
    last_modified: 2026-08-19
  - id: compose
    resource: ../../../docker/compose.yml
    title: Generic Caddy plus islands Compose file
    author: process:cursor
    last_modified: 2026-08-19
  - id: caddyfile
    resource: ../../../docker/cdn/Caddyfile
    title: Same-origin Caddy reverse proxy for hybrid sites
    author: process:cursor
    last_modified: 2026-08-19
  - id: linux-deps
    resource: ../../../docker/install-linux-deps.sh
    title: Builder and runtime apt packages including WebKitGTK
    author: process:cursor
    last_modified: 2026-08-19
  - id: install-roc
    resource: ../../../docker/install-roc.sh
    title: Pinned Roc nightly installer for Linux images
    author: process:cursor
    last_modified: 2026-08-19
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown site build, apply host, and publish report
    author: process:git
    last_modified: 2026-08-19
  - id: host-rs
    resource: ../../../crates/rocci-roc-host/src/host.rs
    title: HostChoice auto native wasm
    author: process:git
    last_modified: 2026-08-18
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated apps use basic-webserver 0.16.0
    author: process:git
    last_modified: 2026-08-19
  - id: basic-cli-platform
    resource: https://github.com/roc-lang/basic-cli/blob/main/platform/main.roc
    title: basic-cli native targets without wasm32
    author: organization:roc-lang
  - id: basic-webserver-platform
    resource: https://github.com/roc-lang/basic-webserver/blob/main/platform/main.roc
    title: basic-webserver native targets without wasm32
    author: organization:roc-lang
  - id: roc-cross-ci
    resource: https://github.com/roc-lang/roc/blob/main/.github/workflows/ci_cross_compile.yml
    title: Roc compiler CI cross-compiles musl from macOS hosts
    author: organization:roc-lang
  - id: service-rs
    resource: ../../../crates/rocci-rocdown/src/service.rs
    title: serve-islands regenerates and compiles from sources
    author: process:git
    last_modified: 2026-08-19
  - id: islands-rs
    resource: ../../../crates/rocci-rocdown/src/islands.rs
    title: Build-time island HTML evaluation
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-cli
    resource: ../../../crates/rocci-rocdown-cli/src/main.rs
    title: rocdown build, run, serve-islands, inspect artifacts
    author: process:git
    last_modified: 2026-08-19
  - id: bundle-rs
    resource: ../../../crates/rocci-cli/src/bundle.rs
    title: macOS ad-hoc app bundle
    author: process:git
    last_modified: 2026-08-19
  - id: rocci-cli-readme
    resource: ../../../crates/rocci-cli/README.md
    title: Base Rocci CLI contract
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Rocdown format and site behavior
    author: process:git
    last_modified: 2026-08-19
  - id: hybrid-guide
    resource: ../../../docs/guides/hybrid-sites.rocdown
    title: Hybrid CDN plus island-service operator guide
    author: process:cursor
    last_modified: 2026-08-19
  - id: desktop-guide
    resource: ../../../docs/guides/desktop-app.rocdown
    title: Package a desktop app
    author: process:git
    last_modified: 2026-08-19
  - id: roc-host-readme
    resource: ../../../crates/rocci-roc-host/README.md
    title: Two-tier cache and embedded wasm platform
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Minimal wasm32 platform without HTTP
    author: process:git
    last_modified: 2026-08-18
  - id: generation-research
    resource: ../rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: hosting-follow-ons
    resource: ../../plans/rocdown/hybrid-island-hosting-follow-ons.md
    title: Hybrid island hosting follow-ons
    author: process:cursor
    last_modified: 2026-08-19
  - id: hybrid-plan
    resource: ../../plans/rocdown/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands for CDN-static sites
    author: process:cursor
    last_modified: 2026-08-19
  - id: publication
    resource: ../../decisions/local-knowledge-publication.md
    title: Keep generated knowledge publication local
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: rocci-dev-site
    resource: ../../plans/site/rocci-dev-site.md
    title: rocci.dev site architecture and Rocdown evolution
    author: process:codex
    last_modified: 2026-08-18
  - id: release-workflow
    resource: ../../../.github/workflows/release.yml
    title: GitHub Releases for product CLIs
    author: process:git
    last_modified: 2026-08-18
---

# Efficient publishing of Rocdown sites and Rocci apps

## Question

What should a Rocci publishing workflow compile, what should a local Docker
image contain, and which tooling is missing for sites versus apps? The working
hypothesis is that a finished site does not need `rocci` or `rocdown` at
host time, only compiled artifacts, and that Wasm might make those artifacts
host-agnostic.[^docker-readme][^hybrid-guide]

This record is evidence and synthesis. Delivery steps live in the
[efficient publishing plan](/plans/rocdown/efficient-publishing.md). Exploratory;
not an approved hosting contract.

## What a finished build actually is

`rocdown build` already produces a complete CDN tree: page HTML, hashed
`/assets/` (theme CSS, and Datastar.js only on `live` pages), discovery
files, and a publish report. `--cdn-only` refuses `live` pages so a
static publish cannot ship action buttons with no service.[^build-rs][^rocdown-cli][^rocdown-readme][^hybrid-guide]

Page kinds after build:

| Kind | At serve time |
| --- | --- |
| `static` | HTML + hashed assets. No Datastar. |
| `hydrate` | Same, with Rocci HTML spliced at build. No Datastar. |
| `live` | CDN snapshot plus a separate island HTTP process. |

`static` and `hydrate` are ordinary files. Hosting them does not require
Roc, Rocci, Rocdown, or Wasm. `docs/` is this shape today.[^hybrid-plan][^hybrid-guide]

`live` is two artifacts: the same CDN tree, plus an island program.
`serve-islands` still reads `.rocdown` sources, generates `main.roc`, and
invokes `roc` at start. Build-time island evaluation only bakes the
snapshot HTML; it does not emit a durable island binary into `dist/`.[^service-rs][^islands-rs][^hosting-follow-ons]

A Rocci **app** is a third shape. `rocci run` compiles generated
`main.roc` for development. `rocci bundle` on macOS already compiles the
Roc server into the `.app` so runtime `PATH` need not contain `roc`.
Linux, Windows, and OCI packaging are absent.[^bundle-rs][^rocci-cli-readme][^desktop-guide]

## What the Docker images contain today

The local Compose demo is a hybrid operator check, not a publish
path. `site-build` runs `rocdown build` inside the image; `islands` runs
`rocdown serve-islands`; Caddy serves `dist/` and reverse-proxies
`/actions/` and `/health`. The site is bind-mounted as sources, not as a
pre-built tree.[^docker-readme][^compose][^caddyfile]

The `runtime` image is slow because it is a toolchain:

- Ubuntu 24.04, WebKitGTK, GTK, SQLite.[^linux-deps][^runtime-dockerfile]
- A pinned Roc nightly under `/opt/roc`.[^install-roc]
- A builder stage that installs rustup and `cargo build --release` of
  `rocci-cli` and `rocci-rocdown-cli`.[^runtime-dockerfile]
- Those two CLIs, which still link the desktop crate, so Linux needs
  WebKit even for `--no-window`.[^hosting-follow-ons]

Image build therefore pays Rust compile, Roc install, and WebKit. First
container boot then pays `rocdown build` and a second Roc compile of
generated island `main.roc`. None of that is required to **host** a
pre-built static tree.[^docker-readme][^service-rs]

The Caddy stage is already close to a publish image: official
`caddy:2-alpine` plus a Caddyfile. It currently assumes an `islands`
backend and a `dist/` that was just built in the sibling
container.[^caddyfile][^compose]

```mermaid
flowchart TB
  subgraph today [Today local Docker]
    src[".rocdown sources"] --> fat["Ubuntu + rustc + roc + WebKit + rocdown"]
    fat --> rebuild["rocdown build in container"]
    rebuild --> dist["dist/"]
    fat --> rocStart["serve-islands compiles main.roc"]
    dist --> caddy["Caddy :8080"]
    rocStart --> caddy
  end
```

## `--host` versus `--target`

These are different knobs. Conflating them is how Wasm-only publishing gets
oversold.[^host-rs][^rocdown-cli][^build-rs]

**`--host` (apply runtime, shipped).** `rocdown build --host auto|native|wasm`
selects how the *theme applicator* runs on the machine that is writing
`dist/`. Native compiles `apply` with `basic-cli` and execs it. Wasm compiles
`--target=wasm32` against Rocci's embedded WASI platform and runs it in
Wasmtime. `[build].host` in `rocdown.toml` and `ROCCI_HOST` are the same
enum. This flag must stay; a wasm-only apply path cannot replace it while
`basic-cli` refuses `wasm32`.[^rocdown-cli][^host-rs][^build-rs][^basic-cli-platform]

**`--target` (native ISA/OS, not a `rocdown` flag today).** The Roc compiler
accepts `roc build --target=<name>` (`x64musl`, `arm64musl`, `x64glibc`,
and similar). Native `rocdown build` does **not** pass `--target`; it
builds for the host. Island and app `main.roc` use `basic-webserver` the
same way: host-native, no `--target`.[^build-rs][^dispatch-rs][^host-rs]

**Platforms decide which targets exist.** A target works only when that
platform ships link inputs (`libhost.a`, musl `crt1.o`, and so on).
`basic-cli` 0.22 (Rocdown apply) and `basic-webserver` 0.16 (apps and
islands) list `x64mac`, `arm64mac`, `x64win`, `x64musl`, `arm64musl`.
Neither lists `wasm32`. Rocci's custom apply platform is the wasm32
exception, and it has no HTTP.[^basic-cli-platform][^basic-webserver-platform][^wasm-platform][^dispatch-rs]

So native binaries stay first-class. Wasm apply is optional. Wasm cannot
be the island or app runtime until a different HTTP platform exists.

## Can a Mac build Linux binaries?

The compiler can. Roc CI builds `x64musl` and `arm64musl` apps from
`macos-15` and `macos-15-intel` hosts using `roc build --target=…`. That
is a compiler test against a tiny int platform, not a guarantee that
every Rocci-pinned platform tarball contains working musl hosts.[^roc-cross-ci]

For generated Rocci/Rocdown code the practical split is:

| Artifact | Must run where | From a Mac |
| --- | --- | --- |
| Apply (`--host native`) | On the Mac, to write `dist/` | Host-native `apply` only. A musl `apply` would not exec on macOS. |
| Apply (`--host wasm`) | On the Mac, in Wasmtime | `wasm32` via the custom platform. HTML is still portable. |
| Island / app server | On Linux (Docker, VPS) | `roc build --target=x64musl` (or `arm64musl` if that host is in the pinned tarball). Not `--host wasm`. |

A Mac site build that also wants a Linux island binary is therefore two
compiles: native or wasm **apply** for HTML, plus a musl **process**
binary for the service. Do not pass `--target=x64musl` into `--host
native` apply on macOS; that binary cannot generate `dist/` locally.

Whether the pinned `basic-webserver` 0.16.0 tarball actually links
`x64musl` from Darwin is unproven in this repository. Treat Mac→Linux as
the intended Roc mechanism, and make a failed `--target` a clear error
rather than a silent host build.[^dispatch-rs][^roc-cross-ci]

## Wasm versus host-agnostic hosting

Two different embeddings are easy to conflate.[^generation-research][^roc-host-readme]

**Build-time apply (shipped, optional).** `roc build --target wasm32`
against the embedded WASI platform, then Wasmtime, is a renderer host.
`--host wasm` writes the same kind of `dist/` as native apply. The
`.wasm` stays on the build machine. Serving that `dist/` does not load
Wasm.[^build-rs][^roc-host-readme]

**Runtime HTTP (not available).** The wasm platform is `main! : {} =>
Result` with no HTTP. `basic-cli` / `basic-webserver` do not support
`wasm32`. Island and app servers cannot be that module.[^wasm-platform][^build-rs][^generation-research][^basic-cli-platform][^basic-webserver-platform]

So:

- Static and hydrate sites are already host-agnostic: HTML, CSS, and
  hashed JS. Wasm does not make them more portable.
- Native apply must remain selectable because several platforms Rocci
  uses do not compile to wasm.
- Cross-compiling **apply** to wasm does not remove `roc` from the build
  host; it only changes how HTML is generated on that host.
- Cross-compiling an **island or app** to wasm would need a WASI-HTTP
  platform. Until then, Linux process artifacts are native musl
  `--target`, not `--host wasm`.[^build-rs][^hosting-follow-ons]

Playground `--mode wasm` is a browser authoring worker. It is not a
publish format.[^rocci-cli-readme]

## Missing tooling

| Need | Sites | Apps | OKF |
| --- | --- | --- | --- |
| Compile to durable artifacts | `rocdown build` → `dist/` | macOS `.app` only; `run` recompiles | local HTML; no public archive[^publication] |
| Inspect what will publish | `inspect artifacts` | none | inspect concept |
| Package a deployable archive | none | none beyond `.app` | forbidden for now[^publication] |
| Serve a pre-built tree | none (`run` rebuilds) | bundled `.app` only | preview rebuilds |
| Local Docker of artifacts | none (fat hybrid rebuild) | none | none |
| Cross-compile for Linux | not a `build` flag | not a `bundle` flag | n/a |
| OCI / registry | none | none | none |
| Hosting adapters (Pages, Netlify) | none; rocci.dev plan rejects plugin deploy[^rocci-dev-site] | n/a | n/a |
| Product CLI releases | GitHub Releases of `rocdown`, not sites[^release-workflow] | `rocci` binary, not apps | `rocci-okf` binary |

The hybrid public guide already describes the two-artifact production
sketch (upload `dist/`, run `serve-islands`, reverse-proxy). Local Docker
does not follow that sketch: it rebuilds from sources with a toolchain
image.[^hybrid-guide][^docker-readme]

`rocci.dev` (`docs/` → `dist/docs`) is a static tree with no island
service. It is the natural first dogfood for artifact hosting, and it
does not need the current Compose file.[^rocci-dev-site][^hybrid-plan]

## Recommendation

1. **Split build hosts from serve hosts.** Build on the developer machine
   or CI (`rocdown`, `roc`, maybe a fat *builder* image). Serve with
   something that does not contain those tools.
2. **Start with pre-built static Rocdown sites.** Official Caddy (or
   equivalent) mounting a host-built `dist/`. No `rocci`, `rocdown`,
   `roc`, rustc, or WebKit in that image. No image build if the official
   Caddy tag is enough.
3. **Treat `static` and `hydrate` as one publish class.** `--cdn-only` is
   the gate. `live` waits on a precompiled island binary.
4. **Keep `--host native` and `--host wasm` as apply choices.** Do not
   drop native because some Roc platforms (`basic-cli`,
   `basic-webserver`) have no `wasm32`. Wasm apply is optional. Linux
   island/app binaries use `roc build --target=x64musl` (or arm64musl),
   not `--host wasm`. Revisit WASI-HTTP only after native precompiled
   islands exist.[^basic-cli-platform][^basic-webserver-platform]
5. **Add packaging commands, not deploy plugins.** A site archive plus
   `publish.json`, and a way to serve that tree locally, cover the
   missing product surface. CDN and PaaS upload stay operator choice.
6. **Apps follow the same split.** Compile the Roc server once; wrap it
   as `.app`, a Linux binary, or a tiny OCI image. Do not put `rocci` in
   the runtime image. macOS bundle already proves Roc-free run for
   apps.[^desktop-guide][^bundle-rs]
7. **Keep OKF public deploy closed** until the local-first publication
   decision is explicitly reopened.[^publication]

```mermaid
flowchart LR
  subgraph build [Build host]
    src["sources"] --> rocdown["rocdown + roc"]
    rocdown --> dist["dist/"]
    rocdown --> bin["island or app binary later"]
  end
  subgraph serve [Serve host]
    dist --> caddy["caddy:alpine"]
    bin --> slim["distroless or slim, no roc"]
  end
```

## Open questions for a reviewer

- Does the pinned `basic-webserver` 0.16.0 tarball actually link
  `x64musl` when `roc build --target=x64musl` runs on Darwin, or must
  islands be compiled on Linux until a newer platform pin?
- Is official Caddy the only first-party local host, or should `rocdown
  serve --from-dist` exist for machines without Docker?
- Should a fat builder image remain for reproducible Linux `rocdown
  build`, or is host/CI `cargo run -p rocci-rocdown-cli -- build`
  enough?
- When `live` packaging lands, is musl the only Linux target, or also
  glibc?
- Does `rocci bundle` grow Linux/OCI in this plan, or stay a follow-on
  owned by the desktop guide?

[^docker-readme]: Content-agnostic images; `site-build` runs `rocdown build`; first boot compiles `main.roc`; image build needs BuildKit, Roc nightly, crates.io.
[^runtime-dockerfile]: Builder installs rustup and release `rocci`/`rocdown`; runtime copies those binaries, Roc, and WebKit libs.
[^compose]: `site-build`, `islands`, `cdn`; bind-mount `ROCCI_SITE` at `/src/site`.
[^caddyfile]: `/actions/` and `/health` reverse_proxy to `islands`; `root * /src/site/dist`.
[^linux-deps]: Runtime apt list includes `libwebkit2gtk-4.1-0` and GTK.
[^install-roc]: Downloads pinned `roc_nightly-linux_*` into `/opt/roc`.
[^build-rs]: Apply to `dist/`; `--host wasm` uses `wasm32`; native apply omits `--target`; `--cdn-only`; stale basic-cli wasm32 hint names musl targets.
[^service-rs]: `serve_islands` loads the site and `execute_app_plan`; no prebuilt-binary exec.
[^islands-rs]: NativeHost evaluates island HTML for the CDN snapshot during build.
[^rocdown-cli]: `build --host auto|native|wasm`; `run`, `serve-islands`, `inspect artifacts`; no `--target`, `package`, or serve-from-dist.
[^bundle-rs]: macOS `.app` only; `build_roc_server` into Resources; other OS bail.
[^rocci-cli-readme]: `bundle` is ad-hoc macOS; playground wasm is authoring.
[^rocdown-readme]: `build` emits CDN HTML plus `islands.json` for hybrid; `--cdn-only` errors on live.
[^hybrid-guide]: Two-artifact deploy; local Docker is Compose after host preview; Caddy sketch.
[^desktop-guide]: Bundled app does not need `roc` on PATH; Linux/Windows and production signing absent.
[^roc-host-readme]: Native apply vs wasm32 Wasmtime; two-tier renderer cache.
[^wasm-platform]: `main! : {} => Result`; wasm32 inputs `host.o` and app; no HTTP.
[^generation-research]: Wasmtime is an apply host; `basic-cli` is native; glue/HTTP wasm is later.
[^hosting-follow-ons]: Precompiled island binary and WebKit-free CLI are unstarted; runtime image still has `roc` and WebKit.
[^hybrid-plan]: CDN-only publish vs full hybrid; `docs/` stays static.
[^publication]: No public knowledge deploy or verbatim bundle archive.
[^rocci-dev-site]: One static rocci.dev tree; no deployment adapters or plugin lifecycle.
[^release-workflow]: Releases `rocci`, `rocdown`, `rocci-okf` binaries, not site trees.
[^host-rs]: `HostChoice::{Auto,Native,Wasm}`; native cache key is `native:{ARCH}` with no `--target`.
[^dispatch-rs]: Generated HTTP apps pin basic-webserver 0.16.0; no wasm32.
[^basic-cli-platform]: Targets x64mac, arm64mac, x64win, x64musl, arm64musl; no wasm32.
[^basic-webserver-platform]: Same native target names as basic-cli; no wasm32.
[^roc-cross-ci]: Compiler CI matrix: macOS hosts build x64musl and arm64musl test apps.
