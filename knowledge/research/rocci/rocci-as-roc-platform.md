---
type: Research Report
title: Rocci should be a Roc platform, not a package on basic-webserver
description: "The app-facing Rocci runtime belongs in a Roc platform that adapts the basic-webserver host. A Datastar package on basic-webserver is the Rails split Richard rejects; the .rocci compiler stays Rust. Luke's rust platform template is the packaging mechanics, not the HTTP engine."
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-02T20:35:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: zulip
    resource: https://roc.zulipchat.com/#narrow/channel/304641-ideas/topic/Platform.20extensibility.20using.20bundle.20of.20effects.20pattern/with/621127036
    title: Platform extensibility using bundle of effects pattern
    author: human:richard-feldman
    last_modified: 2026-09-02
  - id: template
    resource: https://github.com/lukewilliamboswell/roc-platform-template-rust
    title: Roc platform template for Rust
    author: human:luke-boswell
    last_modified: 2026-09-02
  - id: template-main
    resource: https://raw.githubusercontent.com/lukewilliamboswell/roc-platform-template-rust/main/platform/main.roc
    title: Template platform contract (main!, Stdout/Stderr/Stdin)
    author: human:luke-boswell
    last_modified: 2026-09-02
  - id: method-role
    resource: method-role-handlers-as-roc-library.md
    title: Method-role handlers as a pure Roc library or platform
    author: process:cursor
    last_modified: 2026-08-24
  - id: bws-sse
    resource: basic-webserver-sse-http.md
    title: basic-webserver 0.16 SSE and HTTP limits
    author: process:cursor
    last_modified: 2026-08-30
  - id: known-lim
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:cursor
    last_modified: 2026-08-31
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Default pin and platform override
    author: process:git
    last_modified: 2026-09-02
  - id: dispatch-tests
    resource: ../../../crates/rocci-cli/src/dispatch/tests.rs
    title: DispatchOptions.platform override
    author: process:git
    last_modified: 2026-08-31
  - id: html-runtime
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Staged Html wrapper over platform constructors
    author: process:git
    last_modified: 2026-08-30
  - id: datastar-runtime
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Staged Datastar SSE helpers
    author: process:git
    last_modified: 2026-08-30
  - id: readme
    resource: ../../../README.md
    title: Rocci README
    author: process:git
    last_modified: 2026-08-31
  - id: custom-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Authored gallery dispatcher
    author: process:git
    last_modified: 2026-09-02
  - id: bws-main
    resource: ../../../../roc-basic-webserver/platform/main.roc
    title: basic-webserver platform requires and hosted ABI
    author: process:git
    last_modified: 2026-08-30
  - id: bws-design
    resource: ../../../../roc-basic-webserver/design.md
    title: basic-webserver design contract
    author: process:git
    last_modified: 2026-08-06
  - id: bws-cargo
    resource: ../../../../roc-basic-webserver/Cargo.toml
    title: basic-webserver host crate
    author: process:git
    last_modified: 2026-08-05
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Embedded wasm32 apply platform
    author: process:git
    last_modified: 2026-08-18
  - id: roc-host-readme
    resource: ../../../crates/rocci-roc-host/README.md
    title: rocci-roc-host README
    author: process:git
    last_modified: 2026-09-01
  - id: desktop
    resource: ../../../crates/rocci-desktop/README.md
    title: Preview facade over h35-desktop
    author: process:git
    last_modified: 2026-08-26
  - id: workspace
    resource: ../../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-30
  - id: workspace-deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace member classification
    author: process:git
    last_modified: 2026-08-31
  - id: http-module
    resource: ../../../crates/rocci-cli/src/http_module.rs
    title: --http-module asserts the 0.16.0 pin
    author: process:git
    last_modified: 2026-09-02
  - id: playground
    resource: ../../../crates/rocci-cli/src/playground_html.rs
    title: Playground snapshot eval on basic-cli
    author: process:git
    last_modified: 2026-08-30
  - id: island
    resource: ../rocdown/island-snapshot-roc-reachability.md
    title: Snapshot eval vs service compile two platforms
    author: process:cursor
    last_modified: 2026-08-25
  - id: handler-log
    resource: handler-runtime-logging.md
    title: Handler logging into the Rocci runtime console
    author: process:cursor
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: human:nils
    last_modified: 2026-08-31
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-31
  - id: boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Rocci owns the app framework; Rocdown owns documents
    author: process:cursor
    last_modified: 2026-08-31
  - id: plan
    resource: ../../plans/rocci/rocci-as-roc-platform.md
    title: Package Rocci as a Roc platform
    author: process:cursor
    last_modified: 2026-09-02
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Rocci-as-platform post-mortem
    author: process:cursor
    last_modified: 2026-09-02
  - id: native-plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
---

# Rocci should be a Roc platform, not a package on basic-webserver

Exploratory packaging argument. First-cut execution is the paired
[plan](/plans/rocci/rocci-as-roc-platform.md) (Phases 0–6 on
`rocci-as-roc-platform`). Descriptive outcome:
[post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md).
[^plan][^postmortem]

## Scope and authority

The question is **who owns the Roc app's platform pin and public Roc
API**, not whether `.rocci` parse and lower move out of
`rocci-template`. Richard's 2026-09-02 Zulip thread is evidence for that
ownership split. Luke's [Rust platform template](https://github.com/lukewilliamboswell/roc-platform-template-rust)
is evidence for **how** a Rust-hosted platform is built and bundled.
Neither source mentions Rocci.[^zulip][^template]

Prefer the [method-role library counterfactual](method-role-handlers-as-roc-library.md)
for handler DX if the matrix were ordinary Roc constructors. That record
asked a different question and recommended a **package on
basic-webserver**. This record answers packaging and host ownership.
[^method-role]

## For a later agent

- **Authority:** exploratory for the packaging argument. The first-cut
  pin is on the plan branch; the [post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md)
  is descriptive. Do not log the plan complete until CI and Knowledge
  succeed.[^plan][^postmortem]
- Keep three designs distinct: (1) **package on basic-webserver**, (2)
  **compiler that emits basic-webserver apps** (the pre-cutover pin),
  (3) **Rocci domain platform** whose host adapts basic-webserver. This
  record recommended (3); Phases 0–6 are that cutover.
- The [Roc-native compiler](/plans/rocci/roc-native-template-compiler.md)
  vision is consuming **pure templates** in a normal Roc app with **no
  rocci CLI**. `pf.Html` is not that path (one platform per app). That
  plan is an unstarted parity POC; do not assume it lands soon.[^postmortem][^native-plan]
- Do not encode Datastar SSE policy in the `.rocci` parser. If wrap
  helpers move, they move into platform Roc modules, where generated
  `Datastar.roc` already lives.[^datastar-runtime][^dispatch]
- Do not merge the wasm apply host, the WASI HTTP adapter, or
  `h35-desktop` into this platform in the first cut.
  [^roc-host][^desktop][^http-module]

## What Rocci pinned before this cutover

Snapshot of the tree when this research was written. Current pin,
Snake-sized authored diffs, and leftover 0.16.0 paths:
[post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md).[^postmortem]

Generated `main.roc` was an ordinary basic-webserver app:
`app [Context, program]` with `init!`, `respond!`, `shutdown!`, pinned
to the 0.16.0 release bundle. `DispatchOptions.platform` could replace
that URL; `--http-module` still **requires** the 0.16.0 string.
[^dispatch][^dispatch-tests][^http-module]

`rocci run` staged `Html.roc` and `Datastar.roc` next to that app.
Those files wrap `pf.Html` / `pf.Sse`. Custom `main.roc` examples pinned
the same 0.16.0 URL by hand. After the cutover, gallery and snake pin
in-tree `crates/rocci-platform` and `import pf.*`; Notes still pins
0.16.0.[^html-runtime][^datastar-runtime][^readme][^custom-main]

The preview window is not a Roc effect. `rocci-desktop` is a facade over
`h35-desktop`; the generated binary still listens on TCP.
[^desktop][^readme]

Two other Roc platforms already exist in-tree and must stay separate:

| Platform | `requires` | Job |
| --- | --- | --- |
| basic-webserver 0.16 (external pin) | `{ init!, respond!, shutdown! }` | App HTTP |
| `rocci-roc-host` wasm32 | `main! : {} => [Ok({}), Err([Exit(I32)])]` | In-process apply for docs/OKF |
| Playground / island snapshot | basic-cli | One-shot HTML, no `pf.Sqlite` of the server shape |

The wasm apply platform exposes nothing and is not an HTTP host.
Unifying snapshot eval with the server platform is a Rocdown follow-on,
not a prerequisite for packaging the app runtime.
[^roc-host][^roc-host-readme][^playground][^island]

basic-webserver's own design is a dependable HTTP/SSE/SQLite platform,
not a Datastar framework. Pushing Rocci authoring into upstream
`roc-lang/basic-webserver` would violate that contract. Rocci also
already refused to fork 0.16 over idle-timeout defaults.
[^bws-design][^bws-sse][^known-lim]

## What Richard argued

Luke's thread proposal was to thin `roc-ray` / `basic-cli` /
`basic-webserver` toward a lowest-level host so **packages** (terrocotta,
and by implication a Rocci-like framework) own the nice API.[^zulip]

Richard's load-bearing claims:

- An app should ideally have **one dependency besides builtins: the
  platform**. Platforms are batteries-included for a domain.
- Packages should be pure, or declare the minimum effect signatures they
  need. `*-types` packages are an antipattern.
- The unique Roc design is that **the platform is responsible for the
  app-authoring experience**, so **frameworks on top of platforms do not
  make sense**.
- Fitting platform-plus-package (Rails: thin runtime, thick gem) into
  that hole is the friction.
- SwiftUI, Qt, Flutter, React, Solid are **candidate platforms**, not
  libraries on a shared GUI host.
- He chose the name "platform" over "framework" because a platform can
  own the stack down to the OS (unikernel, console SDK) and **reduce
  layers**.

Jasper's follow-up is the same product test: if you want a nice
authoring experience, a platform can do everything a package can, plus
host control, one API/docs surface, and one bug inbox.[^zulip]

Cross-cutting pure algorithms (query builders, a layout **algorithm**)
stay packages. A package coupled to one platform's authoring API is the
smell.

## Three shapes for Rocci

### 1. Package on basic-webserver

The [method-role counterfactual](method-role-handlers-as-roc-library.md):
`rocci: "Rocci"` plus `Rocci.view` / `fragment` / `command` / `live`.
That is the Rails/terrocotta split. It matches official Datastar SDKs
and would help authored `main.roc`. It also gives app authors **two**
dependencies, two docs, and no way to add I/O the host lacks.
[^method-role][^zulip]

Richard's thread is a vote **against** this as the product shape, even
though it is a reasonable SDK analogue.

### 2. Compiler targeting basic-webserver (shipped)

`.rocci` authors never write the platform header. The compiler emits
`respond!` for them. Custom apps still pin basic-webserver.
[^dispatch][^custom-main][^readme]

This is not a Roc package. It is also not a Rocci platform. The
app-facing Roc API is still basic-webserver's `Server` plus staged
helpers. Elm has a compiler **and** is the platform; Rocci currently has
a compiler and **delegates** the platform.

### 3. Domain platform (recommended)

A `rocci-platform` crate in this workspace whose `app` header is:

```roc
app [Context, program] {
    pf: platform "<rocci platform url or path>",
}
```

**Same** `{ init!, respond!, shutdown! }` contract as basic-webserver
0.16 for the first cut, so generated dispatch and custom `main.roc` keep
working. The **host** is an adaptation of the basic-webserver Rust host
(Hyper, Tokio, SQLite, SSE), not a rewrite from the Stdout-only
template. The **Roc surface** grows Rocci modules (`Datastar`, the Html
wrapper, later role constructors, later `Log`) as `exposes`, so they are
not staged copies and not a second package.[^bws-main][^bws-cargo][^template-main]

The `.rocci` compiler stays in `rocci-template`. Rocci still owns the
app framework in the product-boundary sense; that framework's **Roc
runtime** becomes the platform instead of a pin plus two staged files.
[^boundary]

A later plan can change `requires` to `{ init!, routes }` or role
constructors. That is handler-DX work the method-role record already
sketched. It is not required to **be** a platform.[^method-role]

## Why a platform is justified here

The method-role record said a custom platform "buys little DX" over a
package **for the eleven-pair matrix**. That still holds for wrap
selection: constructors in a package and constructors in a platform type
the same way.[^method-role]

Packaging is a different test:

| Job | Package on bws | Rocci platform |
| --- | --- | --- |
| One app dependency | No (bws + rocci + staged files) | Yes, if Datastar/Html are `exposes` |
| Add I/O bws will not grow | No | Yes (`hosted_*`) |
| Single docs / bug inbox for runtime | No | Yes |
| Keep `.rocci` one-file starters | Compiler still required | Compiler still required |
| Idle-timeout / HTTP/1.1 | Unchanged | Unchanged unless a **named** host change is justified |
| Push Datastar into `roc-lang/basic-webserver` | Temptation | Unnecessary; different domain |

Effects that would actually need a host, if ever: write-triggered live
wake (not in 0.16), a `Log.line!` that is not raw stderr, native
capabilities beyond TCP. None of those are the first-cut goal. The first
cut is **ownership and collapsing the staged runtime**.
[^bws-sse][^handler-log]

Durable state stays SQLite in context. Components stay pure Html
functions. The platform does not become a process-local reducer.
[^server-owned][^pure-render][^bws-design]

## What the Rust template is for

[roc-platform-template-rust](https://github.com/lukewilliamboswell/roc-platform-template-rust)
shows the **mechanics** Rocci does not have for an app platform:

- `platform/main.roc` with `requires`, `exposes`, `hosted`, `targets`
- `roc glue` → `src/roc_platform_abi.rs`
- `./build.sh` / `./build.sh --all` writing `platform/targets/<triple>/libhost.a`
- `./bundle.sh` producing `.tar.zst` for `platform "https://…tar.zst"`
- `ci/all_tests.sh` rewriting examples to the local `platform/main.roc` and to a served bundle URL

Its product API is `main! : List(Str) => Try({}, [Exit(I32), ..])` plus
stdio. That is the wrong `requires` for Rocci apps. Use the template's
**layout and release path**; use basic-webserver's **HTTP host and app
contract**.[^template][^template-main][^bws-main]

A new workspace member must land in `Cargo.toml` and in
`workspace_deps.py` `BASE_ROCCI` in the same change.[^workspace][^workspace-deps]

## What not to merge

- **`rocci-roc-host` wasm apply.** Different `requires`, no HTTP, used
  by Rocdown/OKF batch render.[^roc-host-readme]
- **`--http-module` / WASI adapter.** Experimental path that currently
  asserts the 0.16.0 pin. Re-pinning it is a follow-on after native
  `rocci run` works.[^http-module]
- **Desktop window/webview as Roc effects.** Preview chrome stays
  `h35-desktop`. A later platform could expose native APIs; that is not
  the reason to exist.[^desktop]
- **Snapshot eval on basic-cli.** Dual-platform authoring for live
  Rocdown islands stays a Rocdown problem.[^island][^playground]
- **Forking 0.16 to change idle-timeout defaults.** Keep keepalives
  unless a documented host change is the point of a later phase.
  [^bws-sse][^known-lim]

## Recommendation

Package Rocci as a Roc platform in this repository:

1. Adapt the basic-webserver host; do not invent a new HTTP engine and
   do not start the product host from the stdio template.
2. Keep `{ init!, respond!, shutdown! }` until a dedicated handler-DX
   plan moves dispatch into platform constructors.
3. Move staged `Html` / `Datastar` into `exposes`.
4. Point generated and custom apps at that platform.
5. Leave the `.rocci` compiler, Rocdown catalog, wasm apply host, WASI
   HTTP, and desktop shell where they are.

That is Richard's "platform owns the domain authoring runtime" without
pretending the Rust compiler is `platform/main.roc`. Implementation:
[package Rocci as a Roc platform](/plans/rocci/rocci-as-roc-platform.md).
First-cut outcome:
[post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md).
[^zulip][^plan][^postmortem]

[^zulip]: Richard: platform owns app-authoring experience; no framework-on-platform.
[^template]: Template: build, glue, bundle, multi-target, CI rewrite of examples.
[^template-main]: Template `requires` is CLI `main!`, not HTTP `program`.
[^method-role]: Library-on-bws is the SDK analogue; platform was scored as packaging, not DX.
[^bws-sse]: Rocci workarounds keepalives; does not fork 0.16 idle defaults.
[^known-lim]: Shipped pin is basic-webserver 0.16; production packaging absent.
[^dispatch]: Generated apps pin in-tree `crates/rocci-platform` after Phase 4; emit `init!` / `respond!` / `shutdown!`.
[^dispatch-tests]: `DispatchOptions.platform` replaces the default pin.
[^html-runtime]: Staged `Html` wraps `pf.Html` on the 0.16.0 path.
[^datastar-runtime]: Staged `Datastar` wraps `pf.Sse` events on the 0.16.0 path.
[^readme]: `rocci run` stages runtime files on 0.16.0 and opens a preview window on TCP.
[^custom-main]: Gallery `main.roc` now pins in-tree `crates/rocci-platform`.
[^bws-main]: Platform name `webserver`; hosted ABI includes SSE advance.
[^bws-design]: HTTP/SSE/SQLite platform, not a full-stack framework.
[^bws-cargo]: Host is a Rust `staticlib` named `host`.
[^roc-host]: wasm32 apply platform `requires` `main!` only.
[^roc-host-readme]: Native apply uses basic-cli; Wasm uses the embedded platform.
[^desktop]: Preview facade; windowing stays in h35-desktop.
[^workspace]: New members join the workspace manifest.
[^workspace-deps]: Unclassified members fail the lint checker.
[^http-module]: WASI emit currently requires the 0.16.0 URL string.
[^playground]: Playground HTML eval pins basic-cli.
[^island]: Live pages compile twice; unifying Sqlite APIs is not available.
[^handler-log]: Stderr already reaches Console; `log!` is optional later.
[^pure-render]: Components remain pure Html functions.
[^server-owned]: Durable facts stay in SQLite or an external service.
[^boundary]: Rocci owns the app framework; Rocdown owns documents.
[^plan]: Paired implementation plan; Phases 0–6 are on `rocci-as-roc-platform`.
[^postmortem]: First-cut payoff is pf ownership; Snake `respond!` unchanged.
[^native-plan]: Unstarted emit-parity POC; motivating vision is pure templates in a normal Roc app, no rocci CLI.
