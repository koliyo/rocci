---
type: Research Report
title: A Roc-native template parser and lowerer
description: "Exploratory proof of concept: a parallel Roc port of template parse and lower that aims at emit parity with crates/rocci-template. Rust stays the product compiler. Motivating vision is consuming pure .rocci templates in a normal Roc app with no rocci CLI; this record does not replace the Rust crate. Not shipped."
tags: [domain/rocci, integration/roc, concern/syntax, concern/architecture, concern/language-design, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-03T07:20:00Z }
stale_after: 2026-11-30
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Implementation plan for a Roc-native template compiler
    author: process:cursor
    last_modified: 2026-08-31
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented Rocci template language reference
    author: process:git
    last_modified: 2026-08-25
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: parse, lower, compile entry points; crate does not invoke roc
    author: process:git
    last_modified: 2026-08-25
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Hand-written recursive-descent parser and opaque Roc scanners
    author: process:git
    last_modified: 2026-08-23
  - id: lexer
    resource: ../../../crates/rocci-template/src/lexer.rs
    title: Byte cursor, skip_string including ${ interpolation, skip_roc_token
    author: process:git
    last_modified: 2026-08-25
  - id: lower-mod
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: LowerOptions and LoweredModule
    author: process:git
    last_modified: 2026-08-31
  - id: lower-html
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Template-item lowering to Html.element calls
    author: process:git
    last_modified: 2026-08-31
  - id: lower-emitter
    resource: ../../../crates/rocci-template/src/lower/emitter.rs
    title: Component function emission and ?? type annotations
    author: process:git
    last_modified: 2026-08-31
  - id: validate
    resource: ../../../crates/rocci-template/src/validate.rs
    title: Template-item and declaration validation
    author: process:git
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Tree spec; scanners stay hand-written
    author: process:git
    last_modified: 2026-08-22
  - id: source-map
    resource: ../../../crates/rocci-template/src/source_map.rs
    title: OriginKind segments for generated Roc
    author: process:git
    last_modified: 2026-08-25
  - id: html-runtime
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Type-module Html wrapper over platform constructors
    author: process:git
    last_modified: 2026-08-15
  - id: datastar-runtime
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: Datastar helpers already a Roc library
    author: process:git
    last_modified: 2026-08-22
  - id: allsyntax
    resource: ../../../test/AllSyntax.rocci
    title: Comprehensive syntax fixture including handlers
    author: process:git
    last_modified: 2026-08-25
  - id: allsyntax-golden
    resource: ../../../crates/rocci-template/tests/fixtures/all_syntax.roc
    title: Rust-lowered golden Roc for AllSyntax
    author: process:git
    last_modified: 2026-08-25
  - id: styling
    resource: ../../../examples/rocci/standalone/styling/Styling.rocci
    title: Colocated CSS components plus one GET view
    author: process:git
    last_modified: 2026-08-23
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Parser and lowering tests stay server-free
    author: process:git
    last_modified: 2026-08-22
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Pinned Roc nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: ungram-research
    resource: ungram-ast.md
    title: Keep scanners hand-written; ungram is the tree spec
    author: process:cursor
    last_modified: 2026-08-19
  - id: roc-library
    resource: method-role-handlers-as-roc-library.md
    title: Hybrid that keeps @component and moves only routes into Roc
    author: process:cursor
    last_modified: 2026-08-24
  - id: roc-defaults
    resource: roc-nightly-record-defaults.md
    title: Type-position ?? works; pattern ?? still illegal
    author: process:cursor
    last_modified: 2026-08-25
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: "`@component` lowers to a pure Html function"
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Rocci owns templates; Rocdown owns documents
    author: process:cursor
    last_modified: 2026-08-24
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: New-compiler syntax, var, for, Try, methods, packages
    author: organization:roc-lang
    last_modified: 2026-08-31
  - id: roc-parser
    resource: https://github.com/lukewilliamboswell/roc-parser
    title: Combinator package on List(U8); XML is not Rocci HTML
    author: human:lukewilliamboswell
    last_modified: 2026-07-10
  - id: roc-parser-string
    resource: https://raw.githubusercontent.com/lukewilliamboswell/roc-parser/main/package/String.roc
    title: "String :: {}.{ combinators; parse_str on UTF-8 bytes"
    author: human:lukewilliamboswell
    last_modified: 2026-07-10
  - id: cursor-spike
    resource: ../../../roc/rocci-template/Cursor.roc
    title: Phase 0 cursor spike on the pinned nightly
    author: process:cursor
    last_modified: 2026-09-02
  - id: postmortem
    resource: ./roc-native-template-compiler-postmortem.md
    title: Implementation findings on nightly-2026-08-23-fb208ba
    author: process:cursor
    last_modified: 2026-09-03
  - id: platform-postmortem
    resource: ../../audits/rocci/rocci-as-roc-platform-postmortem.md
    title: Rocci-as-platform post-mortem
    author: process:cursor
    last_modified: 2026-09-02
---

# A Roc-native template parser and lowerer

## Status

Exploratory **proof of concept**. Nothing here is shipped. `crates/rocci-template`
remains the language contract and the **product compiler**. This is not a
cutover, not a replacement for Rust in `rocci` / playground / LSP, and not
permission to delete or idle the Rust crate. Implementation: [Roc-native
template compiler](/plans/rocci/roc-native-template-compiler.md). After
Phases 0–6: [implementation post-mortem](/research/rocci/roc-native-template-compiler-postmortem.md).[^plan][^template-readme][^postmortem]

**Near-term goal:** a Roc package that matches Rust **emit parity** on the
template subset (`@component`, `@css`, `@fixture`, `@test`, copy-through
Roc). Goldens compare Roc output to `cargo run -p rocci-template -- build`.
The two implementations are maintained **in parallel**: Rust on `main`'s
product path; the Roc port on git branch `roc-native-template-compiler`.[^plan]

**Long-term vision** (not this POC's delivery, not assumed soon): use
Rocci **inside ordinary Roc**, with **no rocci CLI or tooling**. A
`.rocci` template is loaded/consumed into a normal Roc application —
especially **pure templates** (`@component` / `@css`, no `@init`, no
routes, no server). The host app keeps its own platform (`basic-cli`,
rocci-platform, or another). That vision motivates the port. It does
not change who compiles production `.rocci` today.[^template-lib][^pure-render][^postmortem]

There is still no `import Hello.rocci`. Honest consumption is: a Roc
package lowers the template to `.roc`, then the host `import`s that
module. End state C (`roc` learns `.rocci`) stays a Roc-lang change.[^roc-tutorial]

It is the template half of the hybrid already named in [handlers as a Roc
library](method-role-handlers-as-roc-library.md): keep `@component` / `@css`
as a grammar. The library paper's route constructors are a different
fork.[^roc-library]

Related: [platform post-mortem](/audits/rocci/rocci-as-roc-platform-postmortem.md).[^platform-postmortem]

## What "run from Roc without Rust" can mean

The Roc compiler still only compiles `.roc`. There is no `import Hello.rocci`,
no user macros, and no compile-time splice of a parsed string into the
surrounding module. Constant folding can evaluate a *pure* parser on a string
literal at compile time; it cannot then typecheck the emitted source in the
same compilation.[^roc-tutorial][^template-lib]

Honest end states, in order of ambition:

| End state | What it is | Role now |
| --- | --- | --- |
| **Parity POC (this work)** | Roc package emit matches Rust `build` on template fixtures | Rust still compiles every product `.rocci` |
| **A. Roc compiler CLI** | `roc roc/rocci-template/app.roc -- Hello.rocci -o Hello.roc` | Demo / golden driver on the parallel branch |
| **B. Pre-lowered modules in a Roc app** | `import Hello exposing [hello]` after A | Vision check; not `rocci view` |
| **C. `roc` learns `.rocci`** | Compiler plugin or custom file type | Out of Rocci's hands |
| **D. Runtime interpreter** | Parse markup and eval `{expr}` at run time | Non-goal |

**Parity on A is the POC exit.** A+B is the long-term vision: a normal
Roc app consumes a lowered template with no rocci CLI, especially pure
templates. Not a product switch. C is a Roc-lang change. D is a
non-goal: interpolations are opaque Roc *source*, not values.[^parser][^pure-render]

"Without Rust" in the vision sense means without this workspace's template
crate. The POC still uses `roc` and still compares against Rust. Product
commands keep calling `rocci-template`.

## Shipped pipeline (Rust)

`.rocci` is a Roc module with a bounded HTML grammar. The crate copies
ordinary Roc through as byte spans, recognizes `@` forms only at the start of
a top-level definition, and lowers `@component` to a camelCase function
returning `Html`. It does not invoke `roc` or type-check Roc regions.[^template-readme][^template-lib][^ungram]

```text
.rocci  →  parse  →  validate  →  lower  →  .roc  →  roc check / roc / wrap_type_module
              ↑                                      ↑
         rocci-template (Rust)                 roc nightly
```

That pipeline stays the product path. The Roc package is a second walk of
parse → validate → lower, compared by golden strings, not by replacing
the first walk.[^lower-mod][^validate]

The hard scanner work is **not** HTML tags. It is delimiter-balanced
skipping of Roc so `{expr}`, `|params|`, strings, comments, and `@` at
top-level stay correct. `scan_roc_expr` walks tokens until it returns to the
starting paren/bracket/brace depth. `skip_string` already understands
new-compiler `"${…}"` interpolations inside Roc strings so a `}` in
`"Hello, ${name}!"` does not close a template interpolation.[^parser][^lexer]

Lowering is string emit: `Html.element`, `Html.void_element`,
`Html.fragment`, `Html.attribute`, `List.map` for `@for`, `if` / `match` for
directives, type-position `{ name : Str ?? "World" }` with a stripped
pattern, `data-rocci-css` + `@scope` for `@css`. Source-map `Segment`s ride
along for the LSP and inspector.[^lower-html][^lower-emitter][^roc-defaults][^source-map]

Ungrammar specifies the *tree*. It does not generate the scanner. A Roc port
should keep that split: hand-written cursor, owned AST tags whose names match
the ungram, no parser generator.[^ungram][^ungram-research][^language-dev]

## Template subset (in bound)

v1 is **pure template modules**, not standalone apps.[^product-boundary]

| Keep | Why |
| --- | --- |
| Module header, imports, types, helpers | Copy-through `RocRegion` |
| `@component` | The render abstraction |
| `@css` file-level and in-body | Scoped CSS preamble |
| `@fixture` / `@test` | Ordinary Roc after strip; `expect` stays a later CLI concern |
| Template body: tags, `<>`, interpolations, `@if` / `@else if` / `@else`, `@for`, `@match`, `@let` | The HTML grammar |
| Attribute `{expr}`, boolean attrs, `name=@post("…")` | Datastar actions are string emit + maybe `import Datastar` |
| PascalCase calls, `Design.Button`, void tags | Same resolve rules |

| Defer (not this POC) | Why |
| --- | --- |
| `@context` / `@init` / `@method:role` | Standalone HTTP; generated `main.roc` wrap lives in the CLI |
| Source maps / LSP | Editor product; not required to `roc check` output |
| Rocdown islands | Different scanner; reuse this package later |
| Switching `rocci` / playground off Rust | Product compiler stays Rust; this is a parallel POC |

A file like `Styling.rocci` is *almost* in bound: `Hello` / `FeatureCard` /
`StylePage` are the target. The leading `@get:view("/")` is out of v1; a
fixture for the Roc compiler should strip or omit it.[^styling][^template-readme]

`test/AllSyntax.rocci` is the long-term golden *after* handlers are ignored
or the fixture is split. v1 should start from a dedicated
`Hello.rocci`-shaped file, then grow toward AllSyntax minus routes.[^allsyntax][^allsyntax-golden]

## Why the 2026 compiler makes a port tractable

Rocci already targets the **new compiler** (`nightly-2026-08-23-fb208ba`).
Authored and runtime Roc in this repo uses `|args|`, `if { } else { }`,
`match`, `?`, type modules `Html := [].{`, and method calls such as
`count.to_str()`. The old-compiler tutorial on roc-lang.org is not the pin
to design against.[^inventory][^html-runtime][^roc-tutorial][^styling]

Load-bearing new-compiler facts for a parser:

| Feature | Use in a Rocci port |
| --- | --- |
| `var $pos` and `for` | A 1:1 `Cursor` (`$pos`, `$paren`, `$bracket`, `$brace`) instead of `List.walk` combinators. This is the reason to do the rewrite *now* rather than on alpha4. |
| `match` on tags | `TemplateItem` / `ModuleItem` as tag unions |
| `expect` / `roc test` | Parser and lowerer tests without Cargo |
| `?` on `Try` | Recoverable parse errors; do not `crash` on bad input |
| `"${name}"` | Diagnostics and generated-Roc scaffolding |
| Type modules `Name := [].{` | Package modules, matching `Html.roc` / `Datastar.roc`. roc-parser's `String :: {}.{` is a later spelling; do not mix until the pin moves. |
| `{ name : Str ?? "Roc" }` in **types** | Same lowering as Rust; never copy `??` into `|…|` patterns |
| Static dispatch `list.map(…)` / `123.to_str()` | Optional in the parser; generated `Html.element(...)` should stay function-style to match the runtime wrapper |
| Compile-time constant folding | Useful for `dbg` and tiny fixture strings; **not** macros |
| Headerless `main!` and `package [A, B] {}` | Parser is a package; the compiler CLI is a `basic-cli` (or headerless) app |

Old → new spellings the tutorial lists (`Result` → `Try`, `List U8` →
`List(U8)`, `Bool.true` → `Bool.True`) are already half-applied in this
repo. **Author the parser in the dialect `roc check` already accepts for
`Html.roc`**, and prove `var` / `for` on the pin in Phase 0. Do not assume
every tutorial snippet typechecks on `fb208ba` without a spike.[^roc-tutorial][^html-runtime][^roc-defaults]

`Bool` defaults still must not appear in generated type annotations; tag
defaults still need an authored type. The Roc lowerer inherits those
nightly bugs from the Rust one.[^roc-defaults][^lower-emitter]

## Parser architecture: cursor, not combinators

[roc-parser](https://github.com/lukewilliamboswell/roc-parser) is a real
new-compiler combinator package (`Parser.build_primitive_parser` on
`List(U8)`, `String.parse_str`, XML/CSV/Markdown helpers). It is the wrong
host for Rocci.[^roc-parser][^roc-parser-string]

1. **Opaque Roc is not a grammar in the combinator.** Success is "consumed
   until depth returns," including strings with `"${"`, `#` comments, and
   `|params|`. That is a cursor with monotonic `bump`, the same invariant
   the language-dev skill requires on every branch.[^lexer][^language-dev]
2. **Recovery.** One bad `@component` must not hide the next. Combinators
   that fail the whole `one_of` fight that. The Rust parser records a
   diagnostic and continues.[^parser]
3. **Spans.** The AST stores byte offsets into the original source. A
   leftover-`List(U8)` combinator either copies the tail or loses the base
   pointer. An index into one `List(U8)` (or `Str` plus byte `pos`) matches
   Rust `Cursor.pos`.
4. **XML in roc-parser is XML**, not HTML-plus-Rocci-directives-plus-opaque
   Roc. Reusing it would still leave the hard scanner to write.

Recommendation: **hand-write `Cursor` in Roc** with `$pos` and UTF-8-width
bumps (`ch` as `U8` for ASCII HTML, full scalar bump for Roc regions). Port
`skip_string`, `skip_comment`, `skip_roc_token`, `scan_interpolation`,
`scan_roc_expr` first. Combinators may wrap tiny closed lexemes later
(`@component`, tag names) but must not own the file walk.

Sketch (not compiled; Phase 0 proves the pin):

```roc
Cursor := {
    src : List(U8),
    pos : U64,
    paren : U64,
    bracket : U64,
    brace : U64,
}

bump! = |$cur| {
    if $cur.pos >= $cur.src.len() {
        return
    }
    $cur.pos = $cur.pos + 1
}

scan_interpolation = |src, open_brace| {
    var $cur = { src: src.to_utf8(), pos: open_brace, paren: 0, bracket: 0, brace: 0 }
    # eat '{'; depth = 1; skip_string / skip_comment / bump until depth 0
    # every branch: $cur.pos > before, else bump
    { expr: { start: 0, end: 0 }, span: { start: 0, end: 0 }, terminated: Bool.False }
}
```

If `var` on a record field is illegal on the pin, Phase 0 falls back to
returning a new `Cursor` from every function (still index-based, still not
combinators). That fallback is slower to port from Rust but still the right
shape.

## AST and lowering

Hand-write tag unions whose variant names match `Rocci.AST.ungram`
(`ComponentDecl`, `TemplateItem`, `Element`, …). Leave `RouteDecl` /
`ContextDecl` / `InitDecl` out of v1; if a `@get:` is seen at top level,
emit a diagnostic and skip the declaration so remaining components still
parse (or treat the rest of that def as a `RocRegion` — pick one in Phase 1
and freeze).[^ungram]

Spans are `{ start : U64, end : U64 }`. Interpolation and param lists stay
**slices of the source string**, not a Roc expression AST. Lowering
interpolates those slices into generated text, exactly as Rust
`emit_mapped` does without a Roc parser.[^parser][^lower-html]

Lowering output for v1:

- Copy `RocRegion` bytes unchanged.
- Emit `name : { … ?? … } -> Html` when the Rust helper would.
- Emit `name = |{ stripped }| { … Html.element … }`.
- `@for` → `List.map`; `@if` → Roc `if`; `@match` → `match`.
- `@css` → style sibling + `data-rocci-css` using the same `file_scope_id`
  hash as Rust (port the hash; golden CSS ids must match).
- `@fixture` strips the marker; `@test` is either omitted from the module
  body (Rust `compile` omits `expect` from `compiled.roc`) or collected for
  a later `rocci test` equivalent. Match Rust: **no `expect` in the
  lowered module body**.[^lower-emitter][^template-readme]

Do not emit source maps in v1. Byte-identical Roc with the Rust golden is
the exit; maps can be a follow-on once `Segment` layout is worth the
string-builder complexity.[^source-map][^allsyntax-golden]

`Html` constructors stay `Html.element("div", attrs, children)` to match
`crates/rocci-cli/runtime/Html.roc`, not a new method-call style that would
fork goldens.[^html-runtime][^lower-html]

Attribute `name=@post("/path")` still lowers to `Datastar.post("/path")` and
injects `import Datastar` when any action appears. That is template
lowering, not CQRS policy.[^datastar-runtime][^lower-html]

## How a Roc app actually runs a template

After A:

```roc
app [main!] { pf: platform "https://github.com/roc-lang/basic-cli/releases/download/…" }

import pf.Stdout
import Hello exposing [hello]

main! = |_| {
    node = hello({ name: "Ada" })
    Stdout.line!(node)?
    Ok({})
}
```

That `Hello.roc` is generated. The app never links `rocci-template` and
never runs `rocci`. This is the motivating consumption shape: a normal
Roc app calls a pure component.[^template-readme][^html-runtime]

Rendering still needs an `Html` module the host can see. One platform
per app: `import pf.Html` works only if **this** app's platform exposes
`Html`. A `basic-cli` host cannot import rocci-platform's `pf.Html`.
For pure templates in a foreign platform, Html has to be a **package**
or a local module, not rocci-platform `exposes`. v1 can still copy
through `import Html` the way Rust does today.[^html-runtime][^postmortem]

A headerless `roc Hello.roc` only works if lowering also emitted `main!`,
which **pure components do not**. Running a template "directly" means a
**host Roc file** that calls the generated functions, or `rocci view` still
in Rust. Do not invent a generated `main!` that prints HTML in v1; that is
a gallery/CLI feature.

## Dual implementation (parallel, not a successor)

Rust is the **canonical** compiler for as long as this plan runs. The Roc
package is a **parallel** implementation whose success metric is **parity**
with that compiler's template emit, not adoption by `rocci`.

| Track | Lives where | Job |
| --- | --- | --- |
| Product | `crates/rocci-template` on `main` | Parse, lower, LSP, CLI, playground |
| POC | `roc/rocci-template/` on branch `roc-native-template-compiler` | Match Rust emit; `roc test` / `app.roc` |

Knowledge records may land on `main`. The Roc sources stay on the plan
branch unless a later human decision copies them as an unused experimental
tree. Copying them onto `main` still does not switch product commands.

When Rust lowering changes, the POC updates to match. Do not change Rust
emit to make the Roc port easier.

| Compare | How |
| --- | --- |
| Generated Roc | `cargo run -p rocci-template -- build fixture.rocci` vs Roc `app.roc` stdout |
| `roc check` | Generated file typechecks on the pin |
| Diagnostics | Message + span start/end on a small malformed set; wording may drift |
| Parse tree | Optional later; inspect S-expressions are ungram follow-on |

Parser/lowering tests in Roc use `expect` and `roc test`. They must not
start a server. Native `roc test` of the package is the analogue of
`cargo test -p rocci-template`; gate on `ROCCI_REQUIRE_ROC=1` in workspace
CI only if the job should fail without `roc`. Default `cargo test` must
not require the Roc package.[^language-dev]

## Package layout (proposed)

Not a Cargo workspace member. Suggested tree:

```text
roc/rocci-template/
  main.roc          # package [Cursor, Parse, Ast, Lower, Compile]
  Cursor.roc
  Ast.roc
  Parse.roc
  Lower.roc
  Compile.roc
  app.roc           # basic-cli: stdin/file → Roc source
  fixtures/         # .rocci in, expected .roc out
```

The Rust crate **keeps** the crate name `rocci-template`. The Roc package
may use the same words on disk (`roc/rocci-template/`) because it is a
parallel host, not a Cargo member. If that confuses docs, `RocciParse` is
an acceptable package name. Do not remove or rename the Rust crate for this
POC.

## Alternatives rejected (for v1)

| Alternative | Why not |
| --- | --- |
| Delete `.rocci`; Html builders only | That is the [library paper](method-role-handlers-as-roc-library.md)'s component half. This work keeps HTML tags. |
| roc-parser XML | Wrong grammar; leftover-list spans |
| Interpret templates at runtime | `{expr}` is Roc source |
| Generate a Roc parser from ungram | Ungram research forbids generating scanners |
| Port handlers in the same phases | User bound: template only |
| Retarget lowering to another host (Mojo, …) | Copy-through only works because non-template regions *are already Roc* |

## Recommendation

1. Treat this as a **parity POC** on branch `roc-native-template-compiler`,
   not a product rewrite.
2. Port the **cursor + opaque Roc skippers + template grammar + Html
   lowering** to a Roc package on the pinned nightly.
3. Prove with goldens against Rust `build` and `roc check`. When they
   diverge, **Rust wins**.
4. Keep a tiny `app.roc` as the POC compiler driver. Do not wire it into
   `rocci`.
5. Leave standalone HTTP, Rocdown, LSP, and any CLI replacement to later
   records that would have to argue a cutover. This one does not.

The 2026 compiler's `var` / `for` / `match` / `expect` stack is what makes
the port tractable. Phase 0 exists to confirm those features on `fb208ba`
before a thousand-line `Parse.roc`.[^roc-tutorial][^inventory]

**Phase 4 (2026-09-02):** On branch `roc-native-template-compiler`, Hello and
`@if` goldens match Rust `build`. CSS scope ids match; `roc check` of stub
`Html` hosts for hello/branch/css is clean. Remaining vs AllSyntax: routes
(Rust lowers them; the POC skips), qualified `Design.button` byte emit,
CSS fragment whitespace, and type-position `??`. `Parse` and `Template`
cannot share a Roc file on this pin.

**Phase 6 host command:** `roc check roc/rocci-template/fixtures/host.roc`.
Render still needs the web `Html` platform.

Compiler surprises on this pin (open-union merge, `foo = foo` recursion,
Parse/Template isolation) are in the [post-mortem](/research/rocci/roc-native-template-compiler-postmortem.md),
not restated here.[^postmortem]

[^plan]: Paired implementation plan; writing it is not executing a phase.
[^template-readme]: `.rocci` is a Roc module; `@component` bodies are HTML; copy-through of non-`@` regions; no typecheck in the crate.
[^template-lib]: Public `parse` / `lower` / `compile`; no Roc compiler, no HTTP.
[^parser]: Recursive descent; `scan_roc_expr` is depth-based; recovery on later declarations.
[^lexer]: `Cursor` byte `pos`; `skip_string` handles `"""` and `"${"`; `skip_roc_token`.
[^lower-mod]: `LoweredModule.roc` is a `String` plus metadata; options choose Html module names and CSS embed.
[^lower-html]: Tags, directives, interpolations, Datastar actions become Roc calls.
[^lower-emitter]: Component camelCase, type-position defaults, CSS scope ids, fixtures without `expect` in the body.
[^validate]: Template-item rules plus fixture/test name checks; route matrix is out of v1.
[^ungram]: Owned structs from ungram; scanners not generated; Roc leaves opaque.
[^source-map]: `OriginKind` segments; needed for LSP, not for `roc check`.
[^html-runtime]: `Html := [].{ element, text, fragment, … }` wrapping the platform.
[^datastar-runtime]: `Datastar.post` etc. already ordinary Roc.
[^allsyntax]: Includes `@css`, components, directives, and standalone routes.
[^allsyntax-golden]: Rust emit is the byte-level target once the fixture is in bound.
[^styling]: Components plus one `@get:view`; strip the route for a v1 fixture.
[^language-dev]: Monotonic scanners; parser tests without servers.
[^inventory]: Product nightly `nightly-2026-08-23-fb208ba`.
[^ungram-research]: Hand-written parsers; ungram is the tree spec.
[^roc-library]: Hybrid (3) keeps `@component`/`@css` if routes become a library.
[^roc-defaults]: Pattern `??` illegal; type-position defaults on the pin.
[^pure-render]: `@component` is a pure function to `Html`.
[^product-boundary]: Rocci templates versus Rocdown documents.
[^roc-tutorial]: New compiler: `var $x`, `for`, `Try`, `?`, methods, `expect`, packages, no `.rocci` imports; constant folding is not macros.
[^roc-parser]: Combinators including XML; not HTML+Roc spans.
[^roc-parser-string]: `String :: {}.{` and `List(U8)` leftover parsing.
[^cursor-spike]: `var $cur` plus `{ ..$cur, pos: n }`; `skip_string` understands `"\${"`; `roc test roc/rocci-template/main.roc`.
[^postmortem]: Open-union merge, `parse = do_parse`, Parse/Template isolation; not product behavior.
[^platform-postmortem]: Platform Html on pf is for apps that pin rocci; pure-template hosts on another platform need a package or local Html.
