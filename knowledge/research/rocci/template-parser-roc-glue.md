---
type: Research Report
title: Expose the Rust template parser to Roc via hosted glue
description: "If the parser stays in crates/rocci-template, Roc can still call it through platform hosted functions. A package cannot FFI. Glue cannot interpret {expr}; consume means compile then apply. Distinct from rewriting the parser in Roc."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/tooling, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-09T19:40:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: platform-api
    resource: rocci-platform-template-api.md
    title: "Placement update: parse and compile on rocci-platform"
    author: process:cursor
    last_modified: 2026-09-09
  - id: plan
    resource: ../../plans/rocci/template-parser-roc-glue.md
    title: Hosted glue for the Rust template parser
    author: process:cursor
    last_modified: 2026-09-09
  - id: native-research
    resource: roc-native-template-compiler.md
    title: A Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
  - id: native-plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Rocci-as-platform post-mortem
    author: process:cursor
    last_modified: 2026-09-02
  - id: as-platform
    resource: rocci-as-roc-platform.md
    title: Rocci should be a Roc platform, not a package on basic-webserver
    author: process:cursor
    last_modified: 2026-09-02
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: parse, lower, compile; crate does not invoke roc
    author: process:git
    last_modified: 2026-08-31
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented template language contract
    author: process:git
    last_modified: 2026-08-25
  - id: pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: format_ast S-expression printer
    author: process:git
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Tree spec; scanners stay hand-written
    author: process:git
    last_modified: 2026-08-25
  - id: ungram-research
    resource: ungram-ast.md
    title: Ungrammar as AST spec
    author: process:cursor
    last_modified: 2026-08-31
  - id: codes
    resource: ../../../crates/rocci-template/src/codes.rs
    title: Stable RCxxxx diagnostic IDs
    author: process:git
    last_modified: 2026-08-31
  - id: platform-main
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: platform rocci requires, exposes, hosted
    author: process:git
    last_modified: 2026-09-02
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/README.md
    title: Two-tier cache and native/wasm apply
    author: process:git
    last_modified: 2026-09-01
  - id: roc-host-lib
    resource: ../../../crates/rocci-roc-host/src/lib.rs
    title: NativeHost compile_or_cached and run_apply
    author: process:git
    last_modified: 2026-09-01
  - id: gen-research
    resource: rocci-components-in-generation.md
    title: Glue embeds compiled Roc; does not embed the compiler
    author: process:cursor
    last_modified: 2026-08-31
  - id: hosting-abi
    resource: roc-hosting-lazy-abi.md
    title: hosted_* is Roc calling the host over the C ABI
    author: process:cursor
    last_modified: 2026-08-30
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Components lower to pure Html functions
    author: human:nils
    last_modified: 2026-08-16
  - id: workspace-deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace member classification
    author: process:git
    last_modified: 2026-09-02
  - id: html-runtime
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: Wrapper Html over InternalHtml constructors
    author: process:git
    last_modified: 2026-09-02
---

# Expose the Rust template parser to Roc via hosted glue

Exploratory. Not shipped. Not a rewrite of
[crates/rocci-template](/research/rocci/roc-native-template-compiler.md).
Paired plan:
[hosted glue for the Rust template parser](/plans/rocci/template-parser-roc-glue.md).
[^plan][^native-research]

**Placement update (2026-09-09):** for callers that already pin
Rocci, put `compile!` / `parse!` on **rocci-platform** (`pf.Rocci`),
not a new stdio host. A second platform cannot be called from an app
that pins this one. Apply/render stays a follow-on. Details:
[expose parse and compile on rocci-platform](rocci-platform-template-api.md).
[^platform-api]

## The idea

The [native-compiler vision](roc-native-template-compiler.md) is to
consume **pure** `.rocci` templates inside a normal Roc app with **no
rocci CLI**. Rewriting parse/lower in Roc is one way there. It may
never finish. This record is the other way: keep the Rust parser, and
let Roc **call it** through glue.[^native-research][^template-lib]

That is still "Rocci inside Roc." It is not "Rocci without Rust." The
scanner, recovery, and `RCxxxx` diagnostics stay in
`rocci_template::parse` / `compile`.[^template-lib][^codes]

## Glue vocabulary

Two directions, often both called "glue":

| Direction | What it is | Role here |
| --- | --- | --- |
| **Hosted** (`hosted_*`) | Roc calls the host. The host is Rust. | This is how Roc invokes the parser. |
| **`roc glue`** | Generates Rust ABI bindings so the host can call compiled Roc (`init!`, `respond!`, `render`). | Needed when `platform/main.roc` `hosted` / `provides` change. Does **not** wrap `rocci-template` as a Roc package. |

Roc packages have no FFI. Only the **platform** can run Rust. Exposing
the parser therefore means new `hosted` functions on some platform,
implemented by linking `rocci-template` into that host crate.
[^hosting-abi][^gen-research][^as-platform]

`roc glue` already mis-sizes some aggregates (the rocci-platform
`Exit(I64)` workaround). New hosted result types must be sized
explicitly; do not ship an unresolved type variable in the ABI.
[^platform-main]

## Why a package cannot be the parser

A `package [RocciTemplate]` of pure Roc would be the native-compiler
POC. A package that "calls Rust" is not a Roc package; it is a
platform. Staging `Html.roc` next to the app is what `rocci run`
already does, and it is the thing the consume-in-Roc vision wants to
drop.[^native-research][^postmortem]

`cargo run -p rocci-template -- build` already exposes the parser
**without** the product CLI. That is a Rust binary, not a Roc API.
[^template-readme]

## Three payloads (do not mix them)

`rocci-template` does not type-check interpolations and does not invoke
`roc`. `{expr}` stays opaque Roc **source**. A hosted entry cannot
return a finished `Html.Node` from parse alone. Interpreting markup at
run time is the native-compiler **non-goal D** and would break
[pure render](/decisions/pure-render-components.md).
[^template-lib][^native-research][^pure-render]

| Payload | Host does | Roc sees | Serves |
| --- | --- | --- | --- |
| **Parse** | `parse` (+ optional `validate`) | Diagnostics plus a tree: cheap `format_ast` S-expr, or a closed tag union matching the ungram subset | Inspect, tests, a later Roc lowerer |
| **Compile** | `compile` | Generated Roc `Str` plus `RCxxxx` frames | Proof the ABI works; a build step inside the app |
| **Apply** | `compile` + `roc` via rocci-roc-host (cached) | Rendered HTML `Str` (or `Html` if this platform **is** the HTML runtime) | Consume a pure template without `rocci` |

Parse is "expose the parser." Apply is "consume a template." Compile is
the wedge that proves hosted + `rocci-template` without embedding
`roc`. Do not claim compile-alone is consumption: the app still has to
compile the emitted `.roc` in a second step, or write it and
`import`.[^template-lib][^roc-host][^pprint]

Apply already exists **from Rust**: Rocdown's native host shells out to
`roc build` and runs an apply binary. Glue would let a **Roc**
`main!` ask for that path. The host still needs `roc` on PATH for a
cache miss. Glue does not embed the Roc compiler.
[^roc-host][^roc-host-lib][^gen-research]

Returning `pf.Html` from apply only works if **this** app's platform
exposes Html. A stdio template-host can expose a small Html wrapper, or
return `Str` and keep Html out of the ABI. Prefer `Str` for the first
apply cut so the hosted type stays small.[^html-runtime][^gen-research][^postmortem]

## One platform per app

The consume vision said the host app keeps its own platform. Glue
fights that: the parser lives in **this** platform's host. A
`basic-cli` app cannot call rocci-platform `hosted_*`.
[^native-research][^postmortem][^as-platform]

Placement:

| Host | Who can call parse/apply | Cost |
| --- | --- | --- |
| New **stdio template-host** (basic-cli-shaped `requires` `{ main! }`, plus `hosted` parse/compile/apply) | Pure-template CLI/apps that pin it instead of basic-cli | New crate; matches the consume vision; not HTTP |
| **rocci-platform** extras | HTTP apps that already pin Rocci | Smaller increment; does not help a basic-cli host |
| Upstream basic-cli | Everyone on that pin | Out of bound unless roc-lang takes the crate |

The 2026-09-02 cut recommended the **stdio template-host** first so a
basic-cli-shaped app could consume templates without pinning HTTP
Rocci. That still holds for a **non-HTTP** caller. For Rocci apps,
stdio `hosted_*` are unreachable. Current placement:
[rocci-platform extras](rocci-platform-template-api.md).
Do not invent a second HTTP engine.[^as-platform][^platform-main][^platform-api]

`rocci-template` must not depend on the platform crate. A new stdio
host, if ever added, is `base-rocci` in the same change as the
workspace member.[^workspace-deps]

## AST encoding

The tree spec is Rocci.AST.ungram.[^ungram]
Rust already has owned structs. `format_ast` prints an S-expression.
A Roc tag union that matches the ungram would be the "parser in Roc"
surface, with Rust filling it. That mapping is large (spans,
`RocRegion` opaque source, interpolations as source slices). v1 should
return **S-expr + diagnostics**, then decide whether a typed Roc AST
is worth the glue size. Do not generate scanners. Do not send the
full handler matrix in v1.[^ungram][^ungram-research][^pprint]

## Relation to the native-compiler POC

| | Native rewrite | This glue |
| --- | --- | --- |
| Parser implementation | Roc package | `rocci-template` |
| How Roc sees it | `import` after emit, or a Roc `app.roc` driver | `hosted` on a platform |
| Without rocci CLI | After A+B, if emit is `import`-able | After apply!, or compile! + a second `roc` |
| Without **this** Rust crate | The point of the POC | Never |
| Interpolations | Still compiled Roc | Same; apply compiles them |
| Can run in parallel | Yes | Yes |

They are not substitutes for the same Bound. Glue can ship a consume
API while the rewrite stays a parity experiment. A Roc lowerer on top
of hosted parse is a later hybrid, not this plan's first cut.
[^native-plan][^native-research]

## Recommendation

Superseded for product-platform callers by
[expose parse and compile on rocci-platform](rocci-platform-template-api.md):
hosted effects on **rocci-platform** as `pf.Rocci`; compile! then
parse!; apply! later; no new stdio crate unless a non-HTTP caller
appears.[^platform-api]

The payloads and glue vocabulary in this record still hold. Do not
interpret `{expr}` in Rust. Do not start the native-compiler phases
from this record.

Implementation:
[hosted glue for the Rust template parser](/plans/rocci/template-parser-roc-glue.md).
[^plan]

## For a later agent

- **Authority:** exploratory. Do not start phases unless asked.
- Placement for Rocci apps:
  [rocci-platform template API](rocci-platform-template-api.md).
  Keep this pair distinct from
  [roc-native-template-compiler](roc-native-template-compiler.md)
  (rewrite) and from [rocci-as-roc-platform](rocci-as-roc-platform.md)
  (HTTP `pf` packaging).
- `roc glue` is ABI regen, not "wrap the parser crate as a package."
- Apply is not parse. Parse cannot return `Html`.

[^platform-api]: Product pin is rocci-platform; stdio host only if a non-HTTP caller needs it.
[^plan]: Phased Bound retargeted to rocci-platform compile! / parse!.
[^native-research]: Vision is consume-in-Roc, no rocci CLI; rewrite is unstarted; D is non-goal.
[^native-plan]: Parallel-branch emit parity; handlers and HTTP out of Bound.
[^postmortem]: pf.Html is for apps that pin rocci; one platform per app.
[^as-platform]: Domain platform owns hosted I/O; not a package on a thin host.
[^template-lib]: `parse` / `lower` / `compile`; no `roc`; interpolations not evaluated.
[^template-readme]: `build` / `ast` / `inspect` on the crate binary; same on `rocci`.
[^pprint]: S-expression printer over `Document`.
[^ungram]: Tree spec for owned structs; scanners not generated.
[^ungram-research]: Hand-written parsers; ungram names the tree.
[^codes]: `error[RCxxxx]` frames.
[^platform-main]: `hosted { }` list; `Exit(I64)` glue size workaround.
[^roc-host]: Tier-1 generated Roc, tier-2 apply binary; does not hash whole crates.
[^roc-host-lib]: `NativeHost::compile_or_cached` / `run_apply`.
[^gen-research]: Glue embeds compiled Roc; no compiler-as-library; prefer `Str` across the host boundary.
[^hosting-abi]: Roc → host is `hosted_*` C ABI; `roc glue` emits layouts.
[^pure-render]: `@component` is a pure function to Html, not a runtime interpreter.
[^workspace-deps]: New members classified `base-rocci` in the same change.
[^html-runtime]: Html constructors live on the app's platform, not a foreign `pf`.
