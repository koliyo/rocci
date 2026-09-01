---
type: Implementation Plan
title: Roc-native template parser and lowerer
description: "Phased proof of concept: a Roc package on a parallel branch that matches crates/rocci-template emit on the template subset. Rust stays the product compiler. Not a replacement or CLI cutover. Exploratory; do not start a phase until asked."
tags: [domain/rocci, integration/roc, concern/syntax, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-31T11:05:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../../research/rocci/roc-native-template-compiler.md
    title: Research for a Roc-native template parser and lowerer
    author: process:cursor
    last_modified: 2026-08-31
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented template language contract
    author: process:git
    last_modified: 2026-08-25
  - id: lexer
    resource: ../../../crates/rocci-template/src/lexer.rs
    title: "Cursor, skip_string with `${`, skip_roc_token"
    author: process:git
    last_modified: 2026-08-25
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: scan_interpolation, scan_roc_expr, declaration dispatch
    author: process:git
    last_modified: 2026-08-23
  - id: lower-html
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Html.element lowering
    author: process:git
    last_modified: 2026-08-31
  - id: lower-emitter
    resource: ../../../crates/rocci-template/src/lower/emitter.rs
    title: Component emission and CSS scope ids
    author: process:git
    last_modified: 2026-08-31
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Tree names to match in Roc tags
    author: process:git
    last_modified: 2026-08-22
  - id: html-runtime
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Html type module the emit must call
    author: process:git
    last_modified: 2026-08-15
  - id: allsyntax
    resource: ../../../test/AllSyntax.rocci
    title: Long-term golden; strip routes in v1
    author: process:git
    last_modified: 2026-08-25
  - id: golden
    resource: ../../../crates/rocci-template/tests/fixtures/all_syntax.roc
    title: Rust-lowered Roc
    author: process:git
    last_modified: 2026-08-25
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Component+CSS example with one route to omit
    author: process:git
    last_modified: 2026-08-23
  - id: roc-defaults
    resource: ../../research/rocci/roc-nightly-record-defaults.md
    title: Pattern ?? still illegal
    author: process:cursor
    last_modified: 2026-08-25
  - id: roc-library
    resource: ../../research/rocci/method-role-handlers-as-roc-library.md
    title: Hybrid keeps @component
    author: process:cursor
    last_modified: 2026-08-24
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Monotonic scanners; no server in parser tests
    author: process:git
    last_modified: 2026-08-22
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: ungram-research
    resource: ../../research/rocci/ungram-ast.md
    title: Do not generate scanners
    author: process:cursor
    last_modified: 2026-08-19
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: var, for, expect, packages
    author: organization:roc-lang
    last_modified: 2026-08-31
  - id: roc-parser
    resource: https://github.com/lukewilliamboswell/roc-parser
    title: Combinators not used as the file walker
    author: human:lukewilliamboswell
    last_modified: 2026-07-10
---

# Roc-native template parser and lowerer

## Purpose and authority

This is the implementation plan for [A Roc-native template parser and
lowerer](/research/rocci/roc-native-template-compiler.md). It is an
exploratory **proof of concept** until a human accepts a scope. Crate
READMEs remain the current language contract. Do not start a phase until
the user asks.[^research][^template-readme]

This plan does **not** replace `crates/rocci-template`. Rust stays the
product compiler (`rocci` CLI, playground, LSP). The Roc package exists to
prove **emit parity** on the template subset and to keep a long-term
vision testable. Implementation commits stay on git branch
`roc-native-template-compiler`, in parallel with `main`.[^research]

Use `rocci-language-dev` for grammar fidelity against Rust. Author new
`.roc` in the dialect this repo already compiles (`Html := [].{`, `|x|`,
`if { } else { }`). Prove `var` / `for` on the pin in Phase 0 before a
large `Parse.roc`.[^language-dev][^html-runtime][^roc-tutorial][^inventory]

## Goal

On branch `roc-native-template-compiler`, a Roc **package** under
`roc/rocci-template/` parses the **template subset** of `.rocci` and emits
ordinary Roc that matches `cargo run -p rocci-template -- build` on the
same fixtures and that `roc check` accepts. A small `app.roc` is the POC
driver only. When emit disagrees, **Rust wins** and the Roc port
changes.[^research][^roc-library]

After the last phase:

- `roc test` covers cursor, interpolation scan, a component parse, and
  lower-to-string on fixtures in `roc/rocci-template/fixtures/`.
- `roc roc/rocci-template/app.roc -- fixtures/hello.rocci` writes Roc
  equivalent to `cargo run -p rocci-template -- build` on the same file
  (whitespace may be normalized in Phase 4 if a formatter is required).
- A host `.roc` can `import` the generated module and call the component;
  that host is a fixture, not a generated `main!`.
- `@context` / `@init` / `@method:role` are diagnostics or skipped, not
  lowered. `test/AllSyntax.rocci` stays a later golden while it still
  contains routes; `Styling.rocci` is in bound after stripping
  `@get:view`.[^allsyntax][^styling][^golden]
- `rocci` still compiles templates with Rust. The POC is unused by
  product commands.

## Out of bound

- Replacing or idling `crates/rocci-template`
- Switching `rocci-cli`, playground WASM, or `rocci view` to the Roc port
- Merging a product cutover onto `main`
- Standalone HTTP: `@context`, `@init`, `@method:role`, generated
  `main.roc`, `rocci run`
- Rocdown, Markdown, document-root islands
- LSP, source maps, inspect S-expressions, highlighter
- Generating scanners from ungram; depending on roc-parser as the file walk[^roc-parser]
- Interpreting `{expr}` at runtime
- Teaching `roc` to import `.rocci`
- Cargo workspace membership for the Roc package
- Changing `.rocci` spelling

## Constraints that do not move

- **Parallel, not successor.** Product parse/lower stays in Rust. The Roc
  package is a second implementation on this plan's branch.
- **Parity means match Rust.** Do not change Rust goldens to accommodate
  the port.
- Opaque Roc stays a **span**. Do not parse Roc expressions.[^parser][^ungram]
- Every scanner loop makes forward progress (`$pos` increases or equivalent
  bump) on malformed input.[^language-dev][^lexer]
- `@component` stays a pure function to `Html`.[^template-readme]
- Pattern `??` is stripped; type-position defaults follow the Rust
  emitter, including the Bool-default restriction.[^roc-defaults][^lower-emitter]
- Generated `Html.*` calls match `crates/rocci-cli/runtime/Html.roc`
  function style, not a new method API.[^html-runtime][^lower-html]
- Parser tests do not start a server. `roc test` is the package suite.
  `ROCCI_REQUIRE_ROC=1` only if workspace CI must fail without `roc`.[^language-dev]
- Ungram names are the AST vocabulary; do not invent parallel node
  types.[^ungram][^ungram-research]
- Do not `git add` unrelated work. Do not push from a phase unless asked.

## Non-goals (all phases)

- Byte-identical source maps
- Dual-running the Roc compiler inside `cargo test -p rocci-template`
- Porting `format_ast`
- `rocci test` / `expect` trailer (collect `TestInfo` only if cheap; do
  not emit `expect` in the lowered body)
- Hash stability for CSS ids across a *different* hash; **do** port the
  existing `file_scope_id` algorithm so goldens can match

## Phase 0: Pin spike for Cursor

**Bound:** A tiny `roc/rocci-template/` module (or headerless file) that
implements byte `pos`, `bump`, `skip_string` (including `"${"`),
`skip_comment`, and `scan_interpolation` with `expect` tests. Record
whether `var $cur` on a record works on `nightly-2026-08-23-fb208ba`. If
not, freeze the immutable-cursor return style for later phases.

**Out of bound:** Full document parse; lowering; app CLI; roc-parser
dependency.

**Tests:** `roc test` on the spike file against the pin.

**Exit:** Written note in this plan or the research (draft revision) of the
chosen cursor style; interpolation `{a + {b}}` and `"${x}"` inside a Roc
string in a scanned expr do not desync depth.

## Phase 1: Package + AST + document walk

**Bound:** `package` header exporting `Ast`, `Cursor`, `Parse`. Tag unions
for `Document`, `ModuleItem` (v1: `RocRegion`, `ComponentDecl`, `FixtureDecl`,
`TestDecl`, `CssDecl`), `Span`, `Ident`. Top-level walk: copy Roc until
line-start `@` at depth 0; parse `@component` / `@css` / `@fixture` /
`@test` headers enough to take a body span; unknown `@` including
`@get:` / `@context` / `@init` → diagnostic + skip that declaration.
`scan_roc_expr` for fixture values and test bools.

**Out of bound:** Template *item* tree (everything inside the component
body can be an opaque `TemplateBlock` span for this phase); lowering.

**Tests:** `roc test` on `module X exposing [x]` plus one `@component Name =
|{ }| <p/>` producing a `ComponentDecl` with name and param span.

**Exit:** `Parse.parse` returns items; a file with a helper function
between two components keeps the helper as `RocRegion`.

## Phase 2: Template grammar

**Bound:** Fill `TemplateBlock` / `TemplateItem`: `Element`, `ComponentCall`,
`Fragment`, `TextNode`, `Interpolation`, `IfDirective`, `ForDirective`,
`MatchDirective`, `LetDirective`, in-body `@css`. Attributes: string,
`{expr}`, boolean, `@get`/`@post`/… actions. Recovery on unclosed tags.
Port `parse_component_params` enough for body params vs props record.

**Out of bound:** Lowering; route decls.

**Tests:** `roc test` fixtures: self-closing `<Hello />`, paired
`<Badge>…</Badge>`, `@if`/`@else`, `@for`, `@match`, `{name}`, void
`<br>`. Malformed: unclosed `{`, missing `</div>`.

**Exit:** AST for `roc/rocci-template/fixtures/hello.rocci` matches the
ungram-shaped tree the Rust parser would build for that file (spot-check
node kinds and spans, not `format_ast`).

## Phase 3: Lower to Roc source

**Bound:** `Lower.lower` / `Compile.compile`: copy regions; emit component
functions, CSS scope + style sibling, fixture bindings; inject `import
Datastar` when actions appear; PascalCase → camelCase; `??` type
annotation + stripped pattern. `file_scope_id` ported.

**Out of bound:** Source maps; wrapping `Type := [].{`; `expect` trailers;
CLI file IO beyond returning `Str`.

**Tests:** `roc test` compares emitted string to a checked-in expected
`.roc` for `fixtures/hello.rocci`. Update the expected file only when Rust
`build` on that fixture matches it (`cargo run -q -p rocci-template --
build roc/rocci-template/fixtures/hello.rocci`).

**Exit:** Hello component golden matches Rust build output.

## Phase 4: Broader goldens and `roc check`

**Bound:** Additional fixtures: directives, qualified `<Design.Button />`,
file + component `@css`, `@fixture`/`@test` markers stripped. Optional
trimmed Styling (no `@get:view`). Document remaining AllSyntax gaps (routes).
`roc check` on each generated `.roc` with a stub `Html` / `Datastar` if
the fixture imports them.

**Out of bound:** Full `test/AllSyntax.rocci` until routes are skipped
identically to Rust (they are not: Rust *lowers* them). Do not change Rust
to skip routes to make a golden easier.

**Tests:** `roc test` goldens; `roc check` on generated files.

**Exit:** At least Hello + one directive file + one CSS file check clean.
A short list of known mismatches vs AllSyntax is recorded in the research
or this plan as a draft revision.

## Phase 5: Compiler app

**Bound:** `roc/rocci-template/app.roc` reads a path (and stdin `-`), writes
Roc to stdout or `-o`. Diagnostics to stderr with span start/end. Uses
`basic-cli` or headerless file IO — whichever the pin supports for reading
files; spike in this phase if headerless cannot read argv files.

**Out of bound:** Watch mode; wrapping for `rocci run`; installing a
`rocci` binary from Roc.

**Tests:** `roc roc/rocci-template/app.roc -- fixtures/hello.rocci` stdout
matches Phase 3 golden.

**Exit:** README in `roc/rocci-template/` on this branch documents the
command as a POC driver. Do not change public docs or the product
`rocci` CLI. Rust remains the only compiler `rocci` invokes.

## Phase 6: Host import smoke

**Bound:** A fixture app that `import`s generated `Hello.roc` (or a
committed generated file under `roc/rocci-template/fixtures/`) and calls
`hello({ name: "Ada" })`. Prove the module is ordinary Roc. Prefer
`basic-cli` printing `Html` only if `Html.render` is available without
basic-webserver; otherwise `roc check` of the host plus a comment that
render needs the web platform is enough.

**Out of bound:** Desktop preview; Datastar; HTTP.

**Tests:** `roc check` (and `roc` if render is in bound) on the host.

**Exit:** Research and this plan note the exact command. Knowledge log is
not marked complete until CI and Knowledge workflows succeed on the
revision that contains the Roc sources.

## Tests (whole plan)

```sh
roc test roc/rocci-template/main.roc
roc roc/rocci-template/app.roc -- roc/rocci-template/fixtures/hello.rocci
cargo run -q -p rocci-template -- build roc/rocci-template/fixtures/hello.rocci
okmate check knowledge --profile base --format terminal
```

`cargo test -p rocci-template` stays the Rust suite and is not replaced.
`cargo fmt` is required only if a phase touches Rust (Phase 5 README in a
crate). Default Cargo tests must not spawn `roc` unless
`ROCCI_REQUIRE_ROC=1`.[^language-dev]

[^research]: Dual implementation; A then B; no `.rocci` import.
[^template-readme]: Template contract; copy-through; `@component` purity.
[^lexer]: Port skip_string and skip_roc_token first.
[^parser]: Opaque expr scan and declaration dispatch.
[^lower-html]: Target emit shape.
[^lower-emitter]: camelCase, defaults, CSS ids, no expect in body.
[^ungram]: Match node names; do not generate the parser.
[^html-runtime]: Function-style Html API.
[^allsyntax]: Not a v1 golden while it contains routes.
[^golden]: Rust emit reference.
[^styling]: Strip `@get:view` for an in-bound fixture.
[^roc-defaults]: strip_param_defaults.
[^roc-library]: Template half of the hybrid.
[^language-dev]: Termination, server-free tests.
[^inventory]: Compiler pin.
[^ungram-research]: Hand-written scanners.
[^roc-tutorial]: New-compiler features for Phase 0.
[^roc-parser]: Not the file walker.
