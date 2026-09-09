---
type: Implementation Plan
title: Hosted glue for the Rust template parser
description: "Give Roc hosted compile/parse over crates/rocci-template on rocci-platform (pf.Rocci). Do not add a stdio host in v1; do not interpret interpolations; do not replace rocci-cli. Exploratory; do not start a phase until asked."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/tooling, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-09T19:52:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: platform-api
    resource: ../../research/rocci/rocci-platform-template-api.md
    title: Expose parse and compile on rocci-platform via hosted glue
    author: process:cursor
    last_modified: 2026-09-09
  - id: research
    resource: ../../research/rocci/template-parser-roc-glue.md
    title: Glue vocabulary and three payloads
    author: process:cursor
    last_modified: 2026-09-09
  - id: native-research
    resource: ../../research/rocci/roc-native-template-compiler.md
    title: A Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-09-02
  - id: native-plan
    resource: roc-native-template-compiler.md
    title: Roc-native rewrite POC
    author: process:cursor
    last_modified: 2026-09-02
  - id: postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Rocci-as-platform post-mortem
    author: process:cursor
    last_modified: 2026-09-02
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: parse, lower, compile
    author: process:git
    last_modified: 2026-08-31
  - id: pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: format_ast
    author: process:git
    last_modified: 2026-08-25
  - id: codes
    resource: ../../../crates/rocci-template/src/codes.rs
    title: RCxxxx IDs
    author: process:git
    last_modified: 2026-08-31
  - id: platform-main
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Existing hosted block and glue size note
    author: process:git
    last_modified: 2026-09-03
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/README.md
    title: Apply cache
    author: process:git
    last_modified: 2026-09-01
  - id: gen-research
    resource: ../../research/rocci/rocci-components-in-generation.md
    title: Glue vs compiler embed
    author: process:cursor
    last_modified: 2026-08-31
  - id: path-roc
    resource: ../../../crates/rocci-platform/platform/Path.roc
    title: Path.read_utf8! for file wrappers
    author: process:git
    last_modified: 2026-09-03
  - id: hello-web
    resource: ../../../crates/rocci-platform/examples/hello-web.roc
    title: In-tree HTTP example pin
    author: process:git
    last_modified: 2026-09-03
  - id: as-platform
    resource: ../../research/rocci/rocci-as-roc-platform.md
    title: Domain platform vs package
    author: process:cursor
    last_modified: 2026-09-03
  - id: workspace-deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: BASE_ROCCI classification
    author: process:git
    last_modified: 2026-09-03
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure Html functions
    author: human:nils
    last_modified: 2026-08-31
---

# Hosted glue for the Rust template parser

Exploratory. Do not start a phase until the user asks. Placement:
[expose parse and compile on rocci-platform](/research/rocci/rocci-platform-template-api.md).
Glue vocabulary:
[hosted glue research](/research/rocci/template-parser-roc-glue.md).
[^platform-api][^research]

## Goal

A Rocci app that pins **rocci-platform** can call the **existing**
`rocci-template` parser through **`pf.Rocci` hosted functions**, with
**no rocci CLI** and **no second platform crate**. Payloads in order:
**compile!** (generated Roc `Str` plus diagnostics), **parse!**
(`format_ast` S-expr plus diagnostics). File-path wrappers use
`Path.read_utf8!`. Interpolations stay compiled Roc, not a Rust
interpreter. Apply / HTML render is a follow-on.
[^platform-api][^template-lib][^native-research][^pure-render][^path-roc]

## Out of bound

- Rewriting parse/lower in Roc (that is
  [roc-native-template-compiler](roc-native-template-compiler.md))
  [^native-plan]
- Interpreting `{expr}` / `@if` conditions in the host
  [^pure-render][^native-research]
- Returning `pf.Html` / `Html.Node` from parse or compile
  [^platform-api][^gen-research]
- Replacing `rocci` / playground / LSP
- Embedding the Roc compiler as a library [^gen-research]
- Handler / `@init` / `@method:role` apply
- A new `crates/rocci-template-host` stdio platform in this plan
  [^platform-api]
- Adding these `hosted_*` to upstream basic-cli
- WASI `--http-module` / wasm apply as the first backend
- A typed Roc AST for the full ungram in v1 (S-expr first)
- `import Hello.rocci`
- Apply! / nested `roc` during `respond!` (follow-on)

## Constraints that do not move

1. **Parser stays `crates/rocci-template`.** Host calls `parse` /
   `compile`. Do not fork a second grammar. [^template-lib]
2. **Roc → Rust is `hosted`, not a package.** `roc glue` only
   regenerates ABI when `platform/main.roc` changes. Hosted result
   types are fully sized records (no rigid/flex holes; see the
   `Exit(I64)` note). Prefer a compile/parse **record**, not `Try`.
   [^platform-main][^research][^platform-api]
3. **One platform per app.** Callers pin rocci-platform.
   [^postmortem][^as-platform]
4. **`rocci-template` does not depend on rocci-platform.** Feature-gate
   `clap` on the template crate **before** the platform depends on the
   lib, so `libhost.a` does not pull the CLI parser.
   [^workspace-deps][^platform-api]
5. **Parser/lowering unit tests stay Roc-free.** Hosted proofs are
   `roc build` examples on rocci-platform (`hello-web` siblings).
6. **Do not start native-compiler phases from this plan.**
7. **A later basic-webserver vendor snapshot must keep**
   `hosted_rocci_*`, `platform/Rocci.roc`, and the `rocci-template`
   Cargo dep.

## Phase 0 — Freeze the hosted contract

Bound: tables below complete enough that Phase 1 can edit
`rocci-platform` without inventing names. No Rust required if the
tables are filled.

| Item | Frozen first cut |
| --- | --- |
| Crate | Existing `crates/rocci-platform`. No new workspace member. |
| Template Cargo | `rocci-template` `cli` feature = optional `clap`. Package `default = ["cli"]` so `cargo run -p rocci-template` stays. Platform depends `rocci-template` with `default-features = false`. |
| Roc module | `platform/Rocci.roc`, `exposes` name `Rocci` (`import pf.Rocci`) |
| Host types | Named on `Host`: `Diagnostic`, `RocciSource`, `RocciCompile`. Same field shapes as the public aliases. |
| Hosted compile | C symbol `hosted_rocci_compile`. `main.roc` map `"hosted_rocci_compile": Host.rocci_compile!`. `Host.rocci_compile! : RocciSource => RocciCompile`. |
| Public compile | `Rocci.compile! : Source => CompileResult` calls `Host.rocci_compile!`. |
| Source | `{ name : Str, source : Str }` (`name` is `SourceFile` name) |
| CompileResult | `{ roc : Str, diagnostics : List(Diagnostic) }` (record, not `Try`; inspect `diagnostics`) |
| Diagnostic | `{ code : Str, message : Str, start : U64, end : U64 }`. `code` empty when `rocci-template` has `None`. `start`/`end` are `Span` offsets (`U64`); default encoding UTF-16, not line/column. |
| Path IO | `Rocci.compile_file! : Path => Try(CompileResult, [PathErr(IOErr), ..])` = `Path.read_utf8!` then `compile!({ name: Path.display(path), source })`. Not a hosted path. |
| Rust host | `crates/rocci-platform/src/rocci.rs`; `mod rocci` in `src/lib.rs`. Calls `rocci_template::compile(SourceFile::new(name, src), &LowerOptions::default())`. Map every diagnostic; do not panic on `has_errors`. After `roc glue`, alias generated compile/record types in `src/abi/mod.rs`. |
| Glue regen | From `crates/rocci-platform`: `roc glue /path/to/roc/src/glue/src/RustGlue.roc ./src/ platform/main.roc` (overwrites `src/roc_platform_abi.rs`). Needs matching compiler + `RustGlue.roc`. |
| Example | `crates/rocci-platform/examples/hello-compile.roc`, pin `../platform/main.roc`, `init!` / `respond!` / `shutdown!` like `hello-web.roc`. GET `/` inlines success fixture, `Rocci.compile!`, `text/plain` body is `result.roc`. GET `/bad` inlines error fixture; body may be empty Roc plus diagnostics; no panic. |
| Success fixture | name `"Hello.rocci"`; source `@component Hello {\n    <p>ok</p>\n}\n`. Emitted Roc contains `import Html`. |
| Error fixture | name `"Bad.rocci"`; source `@component hello {\n    <p>ok</p>\n}\n` (non-Pascal). Diagnostics non-empty. Phase 2 asserts `code` starts with `"RC"` (`RC1003`). |
| Not hosted yet | `parse!`, `apply!`. No `hosted_rocci_parse` in Phase 1. |

Exit:

```text
# tables in this phase name module, compile! type, diagnostic record
okmate check knowledge --profile base --format terminal
```

## Phase 1 — clap gate and compile! on rocci-platform

Bound: feature-gate `clap` on `rocci-template` (binary `cli` feature
only). `rocci-platform` depends on `rocci-template`. `Host.roc` +
`main.roc` hosted entry + `roc glue` + `src/abi/mod.rs` alias +
`hosted_rocci_compile` calling `rocci_template::compile` with
`LowerOptions::default()`. `platform/Rocci.roc` wraps it. Example
`crates/rocci-platform/examples/hello-compile.roc` is an HTTP app:
GET `/` inlines a tiny `@component` fixture, calls `Rocci.compile!`,
returns generated Roc as `text/plain`. Rebuild native `libhost.a`.
No parse tree, no apply. [^template-lib][^platform-api][^hello-web]

Exit:

```text
crates/rocci-platform/build.sh
roc build crates/rocci-platform/examples/hello-compile.roc
# GET / body contains `import Html` (or the camelCase emit) for the fixture
# a known-bad fixture yields a non-empty diagnostics list and no panic
# rocci-template lib without the cli feature does not link clap
cargo fmt --all -- --check
```

## Phase 2 — Structured diagnostics

Bound: `compile!` diagnostics use `RCxxxx` codes from
`rocci-template` when present. A malformed fixture asserts a specific
code (pick one stable parse error from the catalog).
[^codes]

Exit:

```text
roc build crates/rocci-platform/examples/hello-compile.roc
# bad fixture: diagnostic.code starts with "RC"
```

## Phase 3 — parse! as format_ast

Bound: `Host.rocci_parse!` returns `{ ast : Str, diagnostics :
List(Diagnostic) }` where `ast` is `format_ast`. Example GET `/parse`
(or sibling `hello-parse.roc`) includes a `(component` (or current
inspect head). No typed Roc AST. File wrapper `Rocci.parse_file!`
optional if compile_file! already landed. [^pprint][^path-roc]

Exit:

```text
roc build crates/rocci-platform/examples/hello-parse.roc
# response body is an S-expr; contains the component name from the fixture
```

## Phase 4 — Docs, glue note, vendor warning

Bound: `crates/rocci-platform/README.md` documents `pf.Rocci`, glue
regen, and that this is **not** `rocci run` and **not** the native
rewrite. Point native-compiler research at the distinction.
`--http-module` unchanged. Note that a later basic-webserver vendor
copy must keep the Rocci hosted symbols.

Exit:

```text
okmate check knowledge --profile base --format terminal
cargo fmt --all -- --check
```

## Follow-ons (not this plan)

- **apply!** via rocci-roc-host (HTML `Str`, nested `roc`, cache miss
  needs PATH `roc`) [^roc-host]
- Stdio template-host for non-HTTP callers
- Typed Roc AST matching the ungram subset
- Native-compiler rewrite consuming hosted parse as an oracle
- Apply via wasm host instead of native subprocess

[^platform-api]: Product pin is rocci-platform; clap gate; record ABI; apply is follow-on.
[^research]: Hosted vs roc glue; three payloads; parse cannot return Html.
[^native-research]: Consume-in-Roc vision; rewrite unstarted; D is non-goal.
[^native-plan]: Do not execute that POC from this plan.
[^postmortem]: One platform per app; pf.Html is not a foreign-host import.
[^template-lib]: `compile` returns Roc source; does not run `roc`.
[^pprint]: S-expr is the v1 parse payload.
[^codes]: Stable diagnostic IDs.
[^platform-main]: Hosted list plus glue size caution.
[^roc-host]: Two-tier cache; native apply pins basic-cli; `roc` on miss.
[^gen-research]: No compiler-as-library; prefer Str across the boundary.
[^path-roc]: File wrappers read UTF-8 through existing Path effects.
[^hello-web]: Proof apps stay HTTP `init!` / `respond!` / `shutdown!`.
[^as-platform]: Do not make this a package on basic-webserver.
[^workspace-deps]: Both crates already `BASE_ROCCI`; no new member.
[^pure-render]: Compile emits a function; the host does not interpret markup.
