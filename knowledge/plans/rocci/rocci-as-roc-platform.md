---
type: Implementation Plan
title: Package Rocci as a Roc platform
description: "Add an in-tree Rocci platform that adapts the basic-webserver host, exposes Datastar/Html, and becomes the pin for generated and custom apps. Do not rewrite the template compiler, desktop shell, wasm apply host, or WASI HTTP path."
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-02T20:10:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/rocci-as-roc-platform.md
    title: Rocci should be a Roc platform, not a package on basic-webserver
    author: process:cursor
    last_modified: 2026-09-02
  - id: template
    resource: https://github.com/lukewilliamboswell/roc-platform-template-rust
    title: Roc platform template for Rust
    author: human:luke-boswell
    last_modified: 2026-09-02
  - id: zulip
    resource: https://roc.zulipchat.com/#narrow/channel/304641-ideas/topic/Platform.20extensibility.20using.20bundle.20of.20effects.20pattern/with/621127036
    title: Platform extensibility using bundle of effects pattern
    author: human:richard-feldman
    last_modified: 2026-09-02
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: PLATFORM constant and DispatchOptions.platform
    author: process:git
    last_modified: 2026-09-02
  - id: dispatch-tests
    resource: ../../../crates/rocci-cli/src/dispatch/tests.rs
    title: Platform override and 0.16.0 pin tests
    author: process:git
    last_modified: 2026-08-31
  - id: html-runtime
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Staged Html wrapper
    author: process:git
    last_modified: 2026-08-30
  - id: datastar-runtime
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Staged Datastar helpers
    author: process:git
    last_modified: 2026-08-30
  - id: custom-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Custom dispatcher pin
    author: process:git
    last_modified: 2026-09-02
  - id: bws-main
    resource: ../../../../roc-basic-webserver/platform/main.roc
    title: basic-webserver app contract
    author: process:git
    last_modified: 2026-08-30
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
    title: --http-module asserts 0.16.0
    author: process:git
    last_modified: 2026-09-02
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: wasm32 apply platform
    author: process:git
    last_modified: 2026-08-18
  - id: desktop
    resource: ../../../crates/rocci-desktop/README.md
    title: Preview facade over h35-desktop
    author: process:git
    last_modified: 2026-08-26
  - id: bws-sse
    resource: ../../research/rocci/basic-webserver-sse-http.md
    title: SSE idle-timeout workarounds
    author: process:cursor
    last_modified: 2026-08-30
  - id: method-role
    resource: ../../research/rocci/method-role-handlers-as-roc-library.md
    title: Library versus platform for handlers
    author: process:cursor
    last_modified: 2026-08-24
  - id: boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Workspace class rules
    author: process:cursor
    last_modified: 2026-08-31
  - id: agents
    resource: ../../../AGENTS.md
    title: Classify new workspace members in the same change
    author: process:git
    last_modified: 2026-08-31
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Rocci-as-platform post-mortem
    author: process:cursor
    last_modified: 2026-09-02
---

# Package Rocci as a Roc platform

Exploratory. Phases 0–6 are on `rocci-as-roc-platform`. Descriptive
outcome: [post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md).
Do not log complete until CI and Knowledge succeed.[^postmortem]

## Goal

Generated and custom Rocci apps pin **one** in-tree Roc platform. That
platform adapts the basic-webserver host, exposes the Datastar/Html
helpers that `rocci run` currently stages, and is built/bundled the way
[roc-platform-template-rust](https://github.com/lukewilliamboswell/roc-platform-template-rust)
is. Authors still write `.rocci` or `main.roc`; they do not take a Rocci
**package** on basic-webserver.[^research][^zulip][^template]

## Out of bound

- Rewriting `rocci-template` parse/lower in Roc
- Role constructors as the generated `requires` (`{ init!, routes }` /
  `Rocci.view`) — that is a follow-on from the method-role record
  [^method-role]
- A Roc **package** `Rocci` whose platform remains basic-webserver
- Merging `rocci-roc-host` wasm apply into this platform[^roc-host]
- Cutting over `--http-module` / WASI HTTP (it may keep asserting
  0.16.0 until a later plan)[^http-module]
- Desktop window, webview, or file-picker as Roc effects[^desktop]
- Unifying island snapshot eval (`basic-cli`) with the server platform
- Forking idle-timeout / HTTP/1.1 defaults as the reason this crate
  exists[^bws-sse]
- Pushing Datastar into `roc-lang/basic-webserver`
- Production signing, notarization, Windows/Linux installers
- Async Roc or making `respond!` an async host export

## Constraints that do not move

1. **Compiler stays Rust.** `.rocci` headers still lower in
   `rocci-template`. The platform is the app's `pf`, not a replacement
   parser.[^research]
2. **First-cut `requires` match basic-webserver 0.16:**
   `{ init!, respond!, shutdown! }` with `Context`, `Server.Config`,
   `Server.Request`, `Server.Outcome`. Generated dispatch keeps emitting
   `respond!`.[^bws-main][^dispatch]
3. **Host strategy: adapt basic-webserver, do not start the product
   HTTP engine from the stdio template.** Copy or path-adapt the sibling
   `../roc-basic-webserver` host (Hyper/Tokio/SQLite/SSE). Use the
   template for glue, `targets/`, `build.sh`, and `bundle.sh`.
   [^template][^bws-main]
4. **One platform per app.** Do not add a `rocci:` package dependency
   beside `pf`.[^zulip]
5. **Datastar policy stays in Roc modules**, not in the template
   parser. Moving files from `crates/rocci-cli/runtime/` into
   `platform/` is in bound.[^datastar-runtime]
6. **Classify the crate as `base-rocci`** in `workspace_deps.py` in the
   same change as the `Cargo.toml` member.[^workspace][^workspace-deps][^agents][^boundary]
7. **Keepalives stay** unless a later phase documents a host timeout
   change as its Bound.[^bws-sse]
8. **Parser/lowering tests do not invoke Roc.** Runtime proof uses
   `roc build` / `rocci run` on named examples.
9. **Do not change `--host wasm` apply** or the preview-window crate
   boundary.[^roc-host][^desktop]

## Phase 0 — Freeze the platform contract

Bound: record the crate name, `requires`, first `exposes`, host origin,
and pin story in the tables below. No Rust or Roc required if the tables
are complete enough for Phase 1.

| Item | Frozen first cut |
| --- | --- |
| Crate / dir | `crates/rocci-platform`. Workspace package `rocci-platform`. `[lib] name = "host"` so Roc links `libhost.a`. Classified `base-rocci`. |
| Platform header | `platform "rocci"` |
| `requires` | Same `program` record as basic-webserver 0.16: `{ init!, respond!, shutdown! }` with `Context`, `Server.Config`, `Server.Request`, `Server.Outcome` |
| First `exposes` | `Attribute`, `Cmd`, `Env`, `File`, `Html`, `Http`, `IOErr`, `MultipartFormData`, `OsStr`, `Path`, `Server`, `Sse`, `Sleep`, `Sqlite`, `Stderr`, `Stdout`, `Tcp`, `Url`, `UnixTime` |
| Later `exposes` (Phase 3) | Add `Datastar` (CLI runtime helpers, including style-sibling stripping on `patch_elements`). `Html` becomes the staged wrapper; constructors stay an internal module the wrapper imports. Generated Rocci-pin apps `import pf.Datastar` / `import pf.Html` and are not staged copies.[^html-runtime][^datastar-runtime] |
| Host origin | Vendored snapshot of sibling `../roc-basic-webserver` at `241061577473444a11777abc2f9376cc224e0e5f` (0.16 line). Copy the native Hyper/Tokio/SQLite/SSE host into this crate; do not git-submodule. Record that SHA in the crate README. Keep the UPL notice beside the Apache-2.0 tree. Native triple only in Phase 1. |
| App pin (dev) | `pf: platform "<abs>/crates/rocci-platform/platform/main.roc"`. `hello-web.roc` uses a path relative to that example. rocci-cli resolves the absolute path from the workspace. |
| App pin (release) | `.tar.zst` from `bundle.sh`, not before Phase 6. No GitHub release URL unless a later phase adds the workflow. |
| Opt-in (Phase 2) | `--platform rocci` on `rocci run` / `rocci build` (env `ROCCI_PLATFORM=rocci` accepted too). Default generated apps stay on the 0.16.0 URL until Phase 4. |
| Listen | `Server.default_config` is `127.0.0.1:8000`. Generated apps still honor `ROC_BASIC_WEBSERVER_PORT` / `ROC_BASIC_WEBSERVER_HOST`. Keepalives unchanged. |
| Not in this crate | wasm apply `rocci-roc-host/platform/main.roc`, WASI adapter, `h35-desktop`, `--http-module` (keeps asserting 0.16.0) |

Exit: this phase's tables are the contract Phase 1 implements. No extra
code.

## Phase 1 — Scaffold and hello-web

Bound: add `crates/rocci-platform` with `platform/main.roc` matching
Phase 0 `requires`/`exposes` (basic-webserver set only), a native
`libhost.a` (or equivalent) under `platform/targets/<native>/`, glue
bindings, a `hello-web.roc` example that returns a 200 HTML body, and
workspace classification. Layout may copy `build.sh` / glue paths from
the template. Do not change `rocci-cli` pins yet.

Exit:

```sh
# native host library exists for this checkout
test -f crates/rocci-platform/platform/targets/*/libhost.a \
  || test -f crates/rocci-platform/platform/targets/*/host.lib

roc build crates/rocci-platform/examples/hello-web.roc
# binary serves GET / 200 with the example body (curl against the
# listen port the platform documents, typically ROC_BASIC_WEBSERVER_PORT)

python rocci-ops/src/rocci_ops/workspace_deps.py
cargo fmt --all -- --check
```

`hello-web.roc` uses the local `platform/main.roc` path, not the 0.16.0
URL.

## Phase 2 — Optional pin from rocci-cli

Bound: wire `rocci run` / `rocci build` so generated `main.roc` can pin
the in-tree platform without making it the default. Reuse
`DispatchOptions.platform`. Add a CLI flag or env (one, documented).
Default generated apps still pin 0.16.0. Do not move Datastar/Html yet.
[^dispatch][^dispatch-tests]

Exit:

```sh
cargo test -p rocci-cli --no-default-features dispatch
# existing override test plus a test that the new flag/env writes the
# in-tree platform/main.roc path

cargo run -q -p rocci-cli -- build examples/rocci/standalone/counter --<flag>
# generated main.roc contains crates/rocci-platform/platform/main.roc
# and still typechecks / builds with roc
cargo fmt --all -- --check
```

Exact flag name is this phase's choice (`--platform rocci` or
`ROCCI_PLATFORM=1`). Do not silently switch the default.

## Phase 3 — Expose Datastar and Html from the platform

Bound: `platform/` contains the current CLI runtime `Datastar.roc` and
`Html.roc` behavior (including style-sibling stripping on
`patch_elements`). Generated apps that pin the Rocci platform
`import pf.Datastar` / `import pf.Html` (or the names Phase 0 freezes)
and do not receive staged copies. Apps that still pin 0.16.0 keep being
staged as today. Do not change default pin.

Exit:

```sh
cargo test -p rocci-cli --no-default-features dispatch
# generated main for the Rocci pin has no sibling Datastar.roc copy

roc build crates/rocci-platform/examples/hello-web.roc
# plus a small example that imports pf.Datastar and emits one
# datastar-patch-elements event
cargo fmt --all -- --check
```

## Phase 4 — Default generated apps to the Rocci platform

Bound: `dispatch::PLATFORM` (and other generated-app pins in
`view.rs` / `browse/` if they share the app runtime) become the in-tree
platform path in dev, or the documented local equivalent. Standalone
**counter** and **live-counter** `rocci run` (or `rocci build` + HTTP
GET `/` and a fragment POST) work. Keepalives unchanged. `--http-module`
may still require 0.16.0; do not silently break it — keep its assert or
skip with a clear error. [^dispatch][^http-module][^bws-sse]

Exit:

```sh
cargo test -p rocci-cli --no-default-features dispatch
# default generated main.roc does not contain the 0.16.0 release URL

cargo test -p rocci-cli --no-default-features --test <existing run/http tests that cover counter>
# or the crate's current counter/live-counter compile tests

cargo fmt --all -- --check
```

If a named CLI e2e test already compiles counter, extend it rather than
adding a browser suite.

## Phase 5 — Custom main.roc examples

Bound: `examples/rocci/custom/datastar/main.roc` and
`examples/rocci/custom/snake/main.roc` pin the Rocci platform. They keep
authored `respond!`. Staged Datastar/Html are unnecessary because Phase
3 exposed them. Do not rewrite gallery routing into constructors.
[^custom-main]

Exit:

```sh
cargo run -q -p rocci-cli -- build examples/rocci/custom/datastar
cargo run -q -p rocci-cli -- build examples/rocci/custom/snake
# both main.roc files pin crates/rocci-platform/platform/main.roc
cargo fmt --all -- --check
```

## Phase 6 — Bundle mechanics and public docs

Bound: `build.sh` (native) and `bundle.sh` analog in `crates/rocci-platform`
following the template (`.tar.zst` with Roc sources + prebuilt
`libhost.a`). Glue regen documented (`roc glue … platform/main.roc`).
Crate README, root README pin sentence, and public Rocdown runtime/CLI
pages state that apps use the Rocci platform. Multi-target `--all` may
wait if only the native triple is proven; if `--all` is skipped, the
README says which triples are missing. Do not publish a GitHub release
URL unless this phase also adds the release workflow — default is local
bundle + path pin. [^template][^research]

Exit:

```sh
crates/rocci-platform/build.sh
crates/rocci-platform/bundle.sh
# bundle contains platform/**/*.roc and the native libhost

cargo fmt --all -- --check
okmate check knowledge --profile base --format terminal
```

Public docs in this phase name the pin. `--http-module` and wasm apply
remain documented as **not** this platform.

## Follow-ons (not this plan)

- GitHub release URL for the `.tar.zst` ([platform GitHub release](/plans/ops/rocci-platform-github-release.md))
- Role constructors inside the platform (`Rocci.view` et al.) and
  shrinking generated `respond!`[^method-role]
- Re-pin `--http-module` once the native platform is default
- Snapshot-eval platform unification
- Hosted live-wake or `Log.line!`
- Desktop-native effects

[^research]: Ownership: domain platform, not package-on-bws; compiler stays.
[^template]: Glue, targets, build.sh, bundle.sh, example rewrite tests.
[^zulip]: Platform owns app-authoring runtime; no framework package on a thin host.
[^dispatch]: Generated pin and `DispatchOptions.platform`.
[^dispatch-tests]: Override already replaces the release URL in unit tests.
[^html-runtime]: Wrapper moved into `exposes` in Phase 3.
[^datastar-runtime]: Helpers moved into `exposes` in Phase 3.
[^custom-main]: Custom snake/datastar pin in-tree platform after Phase 5; Notes still 0.16.0.
[^bws-main]: `program` contract to preserve in Phase 0–4.
[^workspace]: New member in root `Cargo.toml`.
[^workspace-deps]: `BASE_ROCCI` classification required.
[^http-module]: WASI path asserts 0.16.0; out of default cutover.
[^roc-host]: Apply wasm platform stays separate.
[^desktop]: Preview window stays h35-desktop.
[^bws-sse]: Keep keepalives; do not fork idle defaults here.
[^method-role]: Constructors are a different plan.
[^boundary]: base-rocci vs rocdown classes.
[^agents]: Classify workspace members in the same change.
[^postmortem]: First-cut payoff is pf ownership; Snake `respond!` unchanged.
