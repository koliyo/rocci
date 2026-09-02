---
type: Audit
title: Rocci-as-platform post-mortem
description: "First-cut platform cutover bought ownership of Datastar/Html as pf, not a new Snake authoring API. Authored apps changed pin plus imports; respond! stayed. Cost is a vendored host and leftover 0.16.0 paths."
tags: [domain/rocci, domain/runtime, integration/roc, integration/datastar, concern/architecture, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-02T20:25:00Z }
stale_after: 2026-12-02
authority: descriptive
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/rocci-as-roc-platform.md
    title: Rocci should be a Roc platform, not a package on basic-webserver
    author: process:cursor
    last_modified: 2026-09-02
  - id: plan
    resource: ../../plans/rocci/rocci-as-roc-platform.md
    title: Package Rocci as a Roc platform
    author: process:cursor
    last_modified: 2026-09-02
  - id: method-role
    resource: ../../research/rocci/method-role-handlers-as-roc-library.md
    title: Library versus platform for handlers
    author: process:cursor
    last_modified: 2026-09-02
  - id: native-plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
  - id: native-research
    resource: ../../research/rocci/roc-native-template-compiler.md
    title: A Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
  - id: snake
    resource: ../../../examples/rocci/custom/snake/main.roc
    title: Snake custom dispatcher after the pin
    author: process:git
    last_modified: 2026-09-02
  - id: datastar-main
    resource: ../../../examples/rocci/custom/datastar/main.roc
    title: Gallery custom dispatcher after the pin
    author: process:git
    last_modified: 2026-09-02
  - id: notes
    resource: ../../../examples/rocci/custom/notes/main.roc
    title: Notes still pins basic-webserver 0.16.0
    author: process:git
    last_modified: 2026-08-23
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Default pin, pf imports, import rewrite
    author: process:git
    last_modified: 2026-09-02
  - id: runtime-assets
    resource: ../../../crates/rocci-cli/src/runtime_assets.rs
    title: Staged Html/Datastar copies for 0.16.0
    author: process:git
    last_modified: 2026-08-30
  - id: http-module
    resource: ../../../crates/rocci-cli/src/http_module.rs
    title: --http-module still requires 0.16.0
    author: process:git
    last_modified: 2026-09-02
  - id: platform-main
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: platform rocci requires and exposes
    author: process:git
    last_modified: 2026-09-02
  - id: build-sh
    resource: ../../../crates/rocci-platform/build.sh
    title: Native libhost.a copy
    author: process:git
    last_modified: 2026-09-02
  - id: bundle-sh
    resource: ../../../crates/rocci-platform/bundle.sh
    title: roc bundle to tar.zst
    author: process:git
    last_modified: 2026-09-02
  - id: ops-cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: rocci-ops command tree
    author: process:git
    last_modified: 2026-08-31
  - id: template
    resource: https://github.com/lukewilliamboswell/roc-platform-template-rust
    title: Roc platform template for Rust
    author: human:luke-boswell
    last_modified: 2026-09-02
  - id: counter
    resource: ../../../examples/rocci/standalone/counter/Counter.rocci
    title: Generated Counter source
    author: process:git
    last_modified: 2026-08-25
---

# Rocci-as-platform post-mortem

Descriptive of the first-cut cutover on `rocci-as-roc-platform` (plan
Phases 0–6). Not a Decision. Not logged complete until CI and Knowledge
succeed.[^plan]

## What we gained

The payoff is **ownership of the app runtime**, not a new Snake API.
Constraint 2 kept `{ init!, respond!, shutdown! }`. Snake still authors
that record. That is success for this plan, not an incomplete DX
cutover. Role constructors remain a follow-on.[^plan][^method-role][^research]

| Before | After first cut |
| --- | --- |
| Generated apps pin basic-webserver 0.16.0 and receive staged `Html.roc` / `Datastar.roc` | Generated apps pin `crates/rocci-platform`; `import pf.Html` / `import pf.Datastar`; no sibling copies on that pin[^dispatch][^runtime-assets] |
| Custom snake/datastar pin the 0.16.0 URL and `import Html` (CLI-staged) | Path pin plus `import pf.Html` (snake) or `pf.Html` / `pf.Datastar` (gallery). `respond!` unchanged[^snake][^datastar-main] |
| Counter `.rocci` authors `import Html` | Still `import Html` in the template. CLI rewrites to `import pf.Html` when wrapping. Authors of `.rocci` do not change[^counter][^dispatch] |
| Runtime bug inbox is "bws + staged files + CLI" | One `pf` for HTTP, SQLite, SSE, Datastar helpers, and the Html wrapper. Host changes (timeouts, `hosted_*`) can land without a second package[^research][^platform-main] |
| Release pin is an upstream GitHub tarball | Dev pin is an in-tree path. `bundle.sh` can emit a local `.tar.zst`. No GitHub release URL yet[^bundle-sh][^plan] |

Richard's test was one app dependency besides builtins: the platform.
The compiler still exists; `.rocci` authors never wrote the 0.16.0
header. What moved is the **emitted** `pf` and the Datastar/Html
modules from staged copies into `exposes`.[^research]

## Snake is the right small diff

Snake's committed change is the platform path and `import Html` →
`import pf.Html`. Gallery adds `import pf.Datastar`. Game rules, SSE
unfold, SQLite, and `respond!` did not move.[^snake][^datastar-main]

That is the packaging test passing: if Datastar/Html are platform
modules, a custom app stops depending on `rocci run` writing sibling
files. It does **not** shrink authored dispatch. The method-role record
already said a custom platform buys little DX **for wrap selection**;
this plan did not try to buy that.[^method-role]

Generated Counter is even smaller at the source: `Counter.rocci` is
unchanged. The CLI pin and import rewrite are the cutover.[^counter][^dispatch]

## What we paid

- Vendored the 0.16 Hyper/Tokio/SQLite host into `crates/rocci-platform`
  (`[lib] name = "host"`). Glue is large. Host crate pins were loosened
  so one Cargo.lock resolves. Native `libhost.a` is gitignored; `roc
  build` needs `build.sh` first.[^platform-main][^build-sh]
- `build.sh` / `bundle.sh` follow Luke's rust platform template, not
  `rocci-ops`. `rocci-ops build` still builds CLI binaries. CI does not
  yet invoke `build.sh`.[^template][^ops-cli]
- CLI rewrites `import Html` / `import Datastar` to `pf.*` when the
  pin is Rocci. `.rocci` source still says `import Html`. That is a
  staging compatibility hack, not a language change.[^dispatch]
- Split pins remain: `--http-module` asserts 0.16.0; Notes still pins
  0.16.0 and sibling `import Datastar` / `import Html`; wasm apply stays
  `rocci-roc-host`.[^http-module][^notes][^plan]

## What would have been the wrong lesson

Do not read the Snake diff as "the platform was unnecessary." The
alternative first cuts were (1) a Roc **package** on basic-webserver
(two dependencies, still staged or packaged helpers) or (2) leave the
compiler targeting upstream 0.16.0 forever. This plan chose (3): same
`requires`, Rocci owns `pf`.[^research]

Do not fold `roc bundle` into rocci-ops to "simplify." The scripts are
the usual crate-local platform surface. Operator CI should **call**
`build.sh` when hosted jobs need `libhost.a`.[^template][^ops-cli]

Do not treat `build.sh --all` or a GitHub release URL as shipped. Native
triple only; missing triples are in the crate README.[^build-sh][^plan]

## Possible implication for a Roc-native compiler

Not a schedule. The [Roc-native template compiler](/plans/rocci/roc-native-template-compiler.md)
is an unstarted **parity POC**. Rust stays the product compiler. The
long-term vision (not that POC's delivery) is compiling a `.rocci`
without linking `crates/rocci-template`. Do not treat either as
near-term, and do not start that plan from this cutover.[^native-plan][^native-research]

If that vision were ever reached far enough that `roc` typechecks
generated modules **without `rocci run` writing sibling files**, Html
and Datastar have to live where `roc` can see them. CLI-staged copies
exist only because the Rust CLI writes them. This cutover put those
modules in `pf` `exposes`. That is option value for "without the Rust
template crate" also meaning "without the CLI copy step." It is not a
reason to switch product commands or to change goldens until a human
resumes that POC.[^native-research][^platform-main][^dispatch]

If the POC is resumed later:

- Product wrap now rewrites `import Html` → `import pf.Html`. A
  lowerer that matches **wrapped** product emit would follow that;
  matching unwrapped `rocci-template -- build` may still copy
  `import Html` through. Do not change Rust emit to make a Roc port
  easier.[^native-research][^dispatch][^counter]
- The POC driver is `basic-cli`. Generated HTTP apps pin
  `rocci-platform`. Those stay different platforms even in the vision.
  Unifying them is not implied.[^native-research][^plan]
- Handler / `main.roc` lowering stays out of that POC Bound. A native
  compiler would not, by itself, shrink Snake `respond!`.[^native-plan][^method-role]

## Follow-ons that would change Snake

Those are other plans:

- Role constructors / generated `routes` so custom `main.roc` is not
  a hand-written `respond!`[^method-role]
- Re-pin `--http-module` and Notes
- Hosted `build.sh` so CI has `libhost.a` without a local copy
- `Log.line!`, live-wake, desktop effects as `hosted_*`

[^research]: Packaging argument: one pf, Datastar in exposes, compiler stays Rust.
[^plan]: Same requires; Datastar/Html move; default pin; custom examples; bundle.sh.
[^method-role]: Platform vs package for the handler matrix; constructors are a later Bound.
[^native-plan]: Parallel-branch emit-parity POC; Rust stays product compiler; handlers and HTTP out of Bound.
[^native-research]: Vision is roc without the Rust template crate; A+B not a product switch; Html import is copy-through.
[^snake]: Pin path and `import pf.Html` only.
[^datastar-main]: Pin path plus `pf.Datastar` / `pf.Html`.
[^notes]: Still 0.16.0 URL and sibling imports.
[^dispatch]: Default pin, pf imports, rewrite of sibling Html/Datastar imports.
[^runtime-assets]: 0.16.0 path still stages copies.
[^http-module]: WASI compile still forces the 0.16.0 URL then the fork.
[^platform-main]: `platform "rocci"`; exposes include Datastar and Html.
[^build-sh]: Native libhost copy; `--all` exits 1.
[^bundle-sh]: `roc bundle` of platform Roc plus native libhost.
[^ops-cli]: No platform-host subcommand.
[^template]: Crate-local build.sh / bundle.sh convention.
[^counter]: Template `import Html` unchanged.
