---
type: Implementation Plan
title: Hosted glue for the Rust template parser
description: "Give Roc a hosted compile/parse/apply API over crates/rocci-template without rewriting the parser. Stdio template-host first; do not interpret interpolations; do not replace rocci-cli. Exploratory; do not start a phase until asked."
tags: [domain/rocci, domain/runtime, integration/roc, concern/architecture, concern/tooling, concern/packaging]
status: draft
generated: { by: process:cursor, at: 2026-09-02T20:45:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/template-parser-roc-glue.md
    title: Expose the Rust template parser to Roc via hosted glue
    author: process:cursor
    last_modified: 2026-09-02
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
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Crate CLI and language contract
    author: process:git
    last_modified: 2026-08-25
  - id: platform-main
    resource: ../../../crates/rocci-platform/platform/main.roc
    title: Existing hosted block and glue size note
    author: process:git
    last_modified: 2026-09-02
  - id: roc-host
    resource: ../../../crates/rocci-roc-host/README.md
    title: Apply cache
    author: process:git
    last_modified: 2026-09-01
  - id: roc-host-lib
    resource: ../../../crates/rocci-roc-host/src/lib.rs
    title: NativeHost
    author: process:git
    last_modified: 2026-09-01
  - id: gen-research
    resource: ../../research/rocci/rocci-components-in-generation.md
    title: Glue vs compiler embed
    author: process:cursor
    last_modified: 2026-08-31
  - id: as-platform
    resource: ../../research/rocci/rocci-as-roc-platform.md
    title: Domain platform vs package
    author: process:cursor
    last_modified: 2026-09-02
  - id: workspace
    resource: ../../../Cargo.toml
    title: Workspace members
    author: process:git
    last_modified: 2026-09-02
  - id: workspace-deps
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: BASE_ROCCI classification
    author: process:git
    last_modified: 2026-09-02
  - id: agents
    resource: ../../../AGENTS.md
    title: Classify new workspace members in the same change
    author: process:git
    last_modified: 2026-08-31
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure Html functions
    author: human:nils
    last_modified: 2026-08-16
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Pure-template-ish fixture after stripping the route
    author: process:git
    last_modified: 2026-08-25
---

# Hosted glue for the Rust template parser

Exploratory. Do not start a phase until the user asks. Analysis:
[expose the Rust template parser to Roc via hosted glue](/research/rocci/template-parser-roc-glue.md).
[^research]

## Goal

A Roc application can call the **existing** `rocci-template` parser
through **platform hosted functions**, with **no rocci CLI**. First
host is a small **stdio template-host** (`requires` `{ main! }`), not
a second HTTP engine. Payloads in order: **compile!** (generated Roc),
**parse!** (S-expr + diagnostics), **apply!** (HTML `Str` for one
pure component via rocci-roc-host). Interpolations stay compiled Roc,
not a Rust interpreter.[^research][^template-lib][^native-research][^pure-render]

## Out of bound

- Rewriting parse/lower in Roc (that is
  [roc-native-template-compiler](roc-native-template-compiler.md))
  [^native-plan]
- Interpreting `{expr}` / `@if` conditions in the host
  [^pure-render][^native-research]
- Replacing `rocci` / playground / LSP
- Embedding the Roc compiler as a library [^gen-research]
- Handler / `@init` / `@method:role` apply
- Adding these `hosted_*` to upstream basic-cli
- Merging this host into rocci-platform in v1 (may copy the same Rust
  helpers later) [^as-platform][^postmortem]
- WASI `--http-module` / wasm apply as the first apply backend
- A typed Roc AST for the full ungram in v1 (S-expr first)
- `import Hello.rocci`

## Constraints that do not move

1. **Parser stays `crates/rocci-template`.** Host calls `parse` /
   `compile`. Do not fork a second grammar. [^template-lib]
2. **Roc → Rust is `hosted`, not a package.** `roc glue` only
   regenerates ABI when `platform/main.roc` changes. Hosted result
   types are fully sized (no rigid/flex holes; see the `Exit(I64)`
   note). [^platform-main][^research]
3. **One platform per app.** Apps that want glue pin the template-host
   (or, later, a platform that copies these hosted names).
   [^postmortem][^as-platform]
4. **`rocci-template` does not depend on the new platform crate.**
   Classify the host `base-rocci` in the same change as the workspace
   member. [^workspace][^workspace-deps][^agents]
5. **Apply uses rocci-roc-host caching** and may require `roc` on a
   miss. Do not hash the whole Rust crate. [^roc-host][^roc-host-lib]
6. **Parser/lowering unit tests stay Roc-free.** Hosted proofs are
   named `roc build` examples.
7. **Do not start native-compiler phases from this plan.**

## Phase 0 — Freeze the hosted contract

Bound: tables below complete enough that Phase 1 can add a crate
without inventing names. No Rust required if the tables are filled.

| Item | Frozen first cut |
| --- | --- |
| Crate | `crates/rocci-template-host`. `[lib] name = "host"`. `base-rocci`. |
| Platform header | `platform "rocci-template"` (or `rocci-templates`; pick one in this phase) |
| App `requires` | `{ main! : {} => Try({}, [Exit(I64), ..]) }` (stdio, not HTTP) |
| Hosted compile | `compile! : Str => Try({ roc : Str }, List(Diagnostic))` — source is file contents, not a path, so the host need not open files in v1 |
| Diagnostic | `{ code : Str, message : Str, start : U64, end : U64 }` using `RCxxxx` |
| Path IO | Out of compile! v1; the Roc app reads the file with `pf.File` if the platform exposes it, or inlines a fixture |
| Not hosted yet | `parse!`, `apply!` |

Exit:

```text
# tables in this phase name crate, platform string, compile! type, diagnostic record
okmate check knowledge --profile base --format terminal
```

## Phase 1 — compile! on the stdio host

Bound: crate + `platform/main.roc` + `build.sh` (native `libhost.a`).
Host implements `hosted_rocci_compile` by
`rocci_template::compile`. Example `examples/hello-compile.roc`
inlines a tiny `@component` fixture, calls `compile!`, prints
`roc` on stdout. `roc glue` documented. No apply, no parse tree.
[^template-lib][^template-readme][^workspace-deps]

Exit:

```text
crates/rocci-template-host/build.sh
roc build crates/rocci-template-host/examples/hello-compile.roc
# stdout contains `hello =` (or the camelCase emit) for the fixture
# a known-bad fixture yields a non-empty diagnostics list and no panic
cargo fmt --all -- --check
```

## Phase 2 — Structured diagnostics

Bound: `compile!` errors use `RCxxxx` codes from `rocci-template`,
not a single `Str`. A malformed fixture asserts a specific code
(pick one stable parse error from the catalog).
[^codes]

Exit:

```text
roc build crates/rocci-template-host/examples/hello-compile.roc
# bad fixture: diagnostic.code starts with "RC"
```

## Phase 3 — parse! as format_ast

Bound: `parse! : Str => Try({ ast : Str }, List(Diagnostic))` where
`ast` is `format_ast` S-expression. Example prints the tree for the
hello fixture and includes a `(component` (or current inspect head).
No typed Roc AST. [^pprint]

Exit:

```text
roc build crates/rocci-template-host/examples/hello-parse.roc
# stdout is an S-expr; contains the component name from the fixture
```

## Phase 4 — apply! for one pure component

Bound: `apply! : { source : Str, component : Str, args_json : Str } => Try(Str, [CompileFailed, ApplyFailed(Str)])`.
Host: `compile`, wrap a `main!` that calls the named camelCase function
with JSON-decoded args, `NativeHost::compile_or_cached` + `run_apply`
(or equivalent), return stdout HTML. Fixture is a single `@component`
with a `Str` field (Styling-like card with the `@get:view` omitted).
Cache miss may require `roc` on PATH; document that. Do not apply
routes. [^roc-host][^roc-host-lib][^styling][^pure-render]

Exit:

```text
roc build crates/rocci-template-host/examples/hello-apply.roc
# stdout is HTML containing the interpolated name from args_json
# second run with unchanged source hits cache (log or test)
```

## Phase 5 — Docs and knowledge

Bound: crate README, a short public note that this is **not**
`rocci run` and **not** the native rewrite. Point native-compiler
research at the distinction. `--http-module` unchanged.

Exit:

```text
okmate check knowledge --profile base --format terminal
cargo fmt --all -- --check
```

## Follow-ons (not this plan)

- Copy the same hosted helpers onto rocci-platform for HTTP apps
- Typed Roc AST matching the ungram subset
- File-path compile! (`File.read!` vs passing `Str`)
- Native-compiler rewrite consuming hosted parse as an oracle
- Apply via wasm host instead of native subprocess

[^research]: Hosted vs roc glue; three payloads; stdio template-host first.
[^native-research]: Consume-in-Roc vision; rewrite unstarted; D is non-goal.
[^native-plan]: Do not execute that POC from this plan.
[^postmortem]: One platform per app; pf.Html is not a foreign-host import.
[^template-lib]: `compile` returns Roc source; does not run `roc`.
[^pprint]: S-expr is the v1 parse payload.
[^codes]: Stable diagnostic IDs.
[^template-readme]: Crate `build` already exists as a Rust binary.
[^platform-main]: Hosted list plus glue size caution.
[^roc-host]: Two-tier cache; `roc` on miss.
[^roc-host-lib]: `compile_or_cached` / `run_apply`.
[^gen-research]: No compiler-as-library; prefer Str across the boundary.
[^as-platform]: Do not make this a package on basic-webserver.
[^workspace]: New member in root `Cargo.toml`.
[^workspace-deps]: `BASE_ROCCI` in the same change.
[^agents]: Classify workspace members with the crate.
[^pure-render]: Apply compiles a function; it does not interpret markup.
[^styling]: Strip `@get:view` for an in-bound apply fixture.
