---
type: Research Report
title: Roc-native template compiler post-mortem
description: "After Phases 0–6 on nightly-2026-08-23-fb208ba, Cursor var/while/expect worked as hoped. Unexpected cost was open tag-union merging and cross-module import poisoning, which forced Parse/Template isolation and killed Compile.roc. POC only; Rust stays the product compiler."
tags: [domain/rocci, integration/roc, concern/syntax, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-09-03T07:20:00Z }
stale_after: 2026-12-02
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/roc-native-template-compiler.md
    title: Implementation plan and Phase 0–6 pin notes
    author: process:cursor
    last_modified: 2026-09-03
  - id: research
    resource: ./roc-native-template-compiler.md
    title: Pre-implementation research for a Roc-native template parser
    author: process:cursor
    last_modified: 2026-09-03
  - id: roc-defaults
    resource: ./roc-nightly-record-defaults.md
    title: Same pin; type-position ?? versus pattern ??
    author: process:cursor
    last_modified: 2026-08-25
  - id: inventory
    resource: ../../../docs/inventory.toml
    title: Product Roc pin nightly-2026-08-23-fb208ba
    author: process:git
    last_modified: 2026-08-25
  - id: cursor
    resource: ../../../roc/rocci-template/Cursor.roc
    title: Phase 0 cursor; var $cur plus record update
    author: process:cursor
    last_modified: 2026-09-02
  - id: parse
    resource: ../../../roc/rocci-template/Parse.roc
    title: Document walk; parse = do_parse; Hit/Miss; no Template import
    author: process:cursor
    last_modified: 2026-09-02
  - id: template
    resource: ../../../roc/rocci-template/Template.roc
    title: Template grammar, emit, byte file walk; BodyCss; Forest/NoForest
    author: process:cursor
    last_modified: 2026-09-02
  - id: lower
    resource: ../../../roc/rocci-template/Lower.roc
    title: Parse copy-through; fnv1a32; Html.empty component bodies
    author: process:cursor
    last_modified: 2026-09-02
  - id: app
    resource: ../../../roc/rocci-template/app.roc
    title: basic-cli 0.22.0 POC driver
    author: process:cursor
    last_modified: 2026-09-02
  - id: readme
    resource: ../../../roc/rocci-template/README.md
    title: POC commands; rust remains the product compiler
    author: process:cursor
    last_modified: 2026-09-02
  - id: roc-tutorial
    resource: https://github.com/roc-lang/roc/blob/main/docs/mini-tutorial-new-compiler.md
    title: New-compiler var, for, expect, packages
    author: organization:roc-lang
    last_modified: 2026-08-31
  - id: roc-nightly
    resource: https://github.com/roc-lang/roc/commit/fb208ba17ef1af6254c90a6715f423589a4bcb75
    title: nightly-2026-08-23-fb208ba merge
    author: organization:roc-lang
---

# Roc-native template compiler post-mortem

This is exploratory evidence from executing [Roc-native template parser
and lowerer](/plans/rocci/roc-native-template-compiler.md) on
`nightly-2026-08-23-fb208ba`. It is **not** shipped product behavior.
`crates/rocci-template` remains the compiler `rocci` invokes.[^plan][^research][^readme][^inventory][^roc-nightly]

Work lives on branch `roc-native-template-compiler`. Phases 0–6 are in
that tree. Do not log them complete until hosted CI and Knowledge succeed
on the revision that contains the Roc sources.[^plan]

## Outcome

The pin's `var` / `while` / `match` / `expect` stack was enough to port a
byte cursor, a document walk, a template grammar, and Html-shaped
emit.[^cursor][^roc-tutorial]

On that branch:

- `roc test roc/rocci-template/main.roc` is the package suite.
- Hello and `branch.rocci` (`@if`/`@else`) goldens are byte-identical to
  Rust `build`.
- File + component `@css` stamps match Rust (`css-e7b6899e`,
  `card-98509670`).
- `roc roc/rocci-template/app.roc -- roc/rocci-template/fixtures/hello.rocci`
  stdout is 185 bytes, matching Rust.
- Host smoke is `roc check roc/rocci-template/fixtures/host.roc` (stub
  `Html`; `Html.render` still needs the web platform).[^plan][^app][^readme]

The architecture that actually shipped is **not** the three-module graph
the plan sketched (`Parse` + `Template` + `Compile`). Isolation rules
forced by this nightly dominate the rest of this record.[^parse][^template][^lower]

## Expected on this pin (Phase 0)

These were the point of the cursor spike. They surprised relative to
Rust, not relative to the plan after Phase 0.[^cursor][^plan][^roc-tutorial]

- `var $cur` on a record works. `$cur.pos = n` is illegal. `$cur = {
  ..$cur, pos: n }` works.
- `while` / `break` / `return` exist. There is **no `continue`**.
- Type modules (`Cursor := [].{`) plus top-level `expect` are the package
  test shape.
- Literal `${` in a Roc string is `"\${"`.
- Integer literals are `0.U64`, not `0u64`.
- `Str` has no `.len()`; use `Str.to_utf8(s).len()`.
- Conditionals are `if cond { a } else { b }`, not `then`/`else`
  expressions.
- Function type annotations go on the line **above** the definition.

Same pin already documented that pattern `??` is illegal and that Rocci
emits type-position defaults instead. The POC does not emit those
annotations yet.[^roc-defaults][^plan]

## Unexpected compiler and runtime issues

These were **not** in the Phase 0 pin and were the main cost of Phases
1–5. Several fail at **runtime** (a crashing `expect`, or a sibling
`expect` in the same file going red) with **no type error**.

### Type-module export `foo = foo` is infinite recursion

A type module that exports `parse = parse` (or `compile = compile`)
recurses at runtime. The working form is `parse = do_parse` with a
distinct helper.[^parse][^template][^lower]

Same-file `expect`s that call `Parse.parse` / `Template.parse_body`
through the type-module export can recurse the same way. Call `do_parse`
/ `do_parse_body` from expects in the defining file.

This is a compiler bug relative to ordinary aliasing, not a documented
Roc idiom. It was easy to hit because the type-module tutorial shape
looks like `name = name`.[^roc-tutorial]

### Open tag unions merge across a file (and across imports)

On this nightly, extra tags in the **same file** (and sometimes tags
imported into the same file) collapse into one open union. That produced
the isolation rules recorded in the Phase 4 pin:[^plan][^parse][^template]

- Do not put `Css(...)` on both `ModuleItem` and `TemplateItem`. In-body
  CSS is `BodyCss`.
- Do not reuse `Hit` / `Miss` in both `Parse.roc` and `Template.roc` if
  any file will import both. Template uses `Forest` / `NoForest`.
- Extra tags (for example a leftover `TookComponent`) poison `Forest`
  matching. Sibling `expect`s in the same file then fail even when their
  source did not change.
- A file that `import`s both `Parse` and `Template` cannot safely match
  `Component(_)`. That match crashed at runtime and also failed sibling
  expects.
- Naming a type `Cur` in `Parse.roc` shadows `Cursor.Cur`.

Type aliases also cannot be mutually recursive. `TemplateItem` is a flat
arena (`children: List(U64)` indexes into `nodes`), not a recursive
tree type.[^template]

### Cross-module imports poison Cursor methods

`Template` importing `Parse` broke `Cursor` methods inside
`Template.roc`. A `Compile.roc` that imported both `Template` and
`Cursor` broke `starts_with("@component")`: the walker copied the whole
source as one Roc region. `Compile.roc` was deleted. Emit and the file
walk now live in `Template.roc`.[^template][^plan]

`collect_file_css` that called `Cursor.skip_roc_token` inside
`Template.roc` poisoned `do_parse_body` expects. File CSS collection and
the compile-time file walk are raw `Str.to_utf8` / `U64` index scanners
with **no** `Cur` records and **no** `skip_roc_token`.[^template]

`Lower.roc` imports `Parse` and does not import `Template`. Component
bodies there are stubbed `Html.empty`; real body emit is
`Template.compile`.[^lower][^template]

### Matching payload tags next to Forest parsers

`match List.get(block.nodes, id) { Ok(BodyCss(_)) => ... }` in
`Template.roc` crashed parse expects. Filtering by `item_kind` /
`root_kind` strings (`"Css"`, `"LetDirective"`) does not.[^template][^plan]

`peek` with a **variable** offset failed `U8` versus `I64`. Use
`Cursor.peek(cur) == Ok(123)` with integer literals.

`skip_formatting_ws` skips newlines and the spaces that follow them, not
a leading space after `{`. `{ <p>` needs `skip_spaces_tabs` after eating
`{`. That is scanner semantics, not a crash, but it desynced the first
element parse.[^cursor]

### Stdlib gaps versus a mechanical Rust port

- No `List.walk` / `List.contains`; use `List.fold`.
- `var $x : List(T) = []` is illegal (a bare `var $names = []` compiled
  in some files).
- `Num.div_trunc` and `Num.bitwise_xor` do not exist. Use
  `U64.bitwise_xor` and `U64.bitwise_and`.[^lower][^template]

### Silent `//` is a comment, not integer division

Roc `//` starts a comment. `$v = $v // 16` typechecked and became `$v =
$v`. Hex formatting for `file_scope_id` then emitted the wrong stamp.
The working form is `$v = $v / 16`.[^lower][^template]

This is documented Roc, but it is easy to miss when porting C or Rust
`n // 16` and the compiler gives no unused-expression warning.

### basic-cli 0.22.0 I/O (Phase 5)

Not a language bug, but the 0.22 examples and types did not match the
binary on this pin:[^app][^plan]

- macOS argv is `UnixBytes`, not `Utf8`. Decode with
  `Str.from_utf8_lossy`.
- File IO is `Path.read_utf8!(Path.utf8(path))`, not methods on `Str`.
- `Stdin.line!()` takes **no** argument (`Stdin.line!({})` is too many
  args).
- Process failure is `Err(Exit(1))`, not a two-field `Exit`.

## Architecture that resulted

| Module | Imports | Role |
| --- | --- | --- |
| `Cursor.roc` | none | Byte `pos`, skip string/comment/token |
| `Parse.roc` | `Cursor` | File items; opaque component body spans |
| `Template.roc` | `Cursor` | Body grammar, emit, byte-level file walk |
| `Lower.roc` | `Cursor`, `Parse` | Copy-through + `file_scope_id`; stub bodies |
| `app.roc` | `Template` | POC driver |

Rules that now have to stay true on this pin:[^parse][^template][^lower][^plan]

1. Do not import `Parse` and `Template` into the same Roc file.
2. Do not import `Parse` from `Template.roc`.
3. Do not match `Ok(BodyCss(_))` on `List.get` in `Template.roc`.
4. Do not call `Cursor.skip_roc_token` from file-CSS collection in
   `Template.roc`.
5. Export `name = do_name`, never `name = name`.

`Template.roc` is about 2600 lines because emit could not live in a
third module that also saw `Parse`.[^template]

## Remaining emit gaps (not compiler crashes)

Recorded in the Phase 4 pin; still true after Phase 6:[^plan][^research]

- Routes (`@get:view` and the rest of `@method:role`) are skipped here;
  Rust lowers them. `test/AllSyntax.rocci` stays out of bound.
- Qualified `<Design.Button />` parses as `ComponentCall`; the emitted
  call is not yet byte-identical to `Design.button(...)`. Open-union
  payload merge on `Forest` versus path records is a suspect, not a
  proof.
- CSS `Html.fragment` wrapping can differ in whitespace from Rust even
  when scope ids match.
- Type-position `??` defaults are not emitted.

When emit disagrees, **Rust wins**.[^readme]

## What this means for a later Roc pin

The features the research counted on (`var`, `while`, `match`, `expect`,
packages) are present and usable.[^research][^roc-tutorial]

The unexpected tax is **open-union merging** plus **import-time method
poisoning** that fail at runtime. A later nightly that (a) rejects `parse
= parse` at compile time, (b) does not merge unrelated open unions in
one file, and (c) does not break `Cursor` methods when `Template`
imports `Parse` would let the package look like the original three-module
plan.

Until then, treat this package as a pin-specific POC, not as evidence
that a Roc-native product compiler is a small port.

## Related

- Plan (pins and remaining mismatches): [Roc-native template
  compiler](/plans/rocci/roc-native-template-compiler.md)
- Pre-implementation research: [A Roc-native template parser and
  lowerer](/research/rocci/roc-native-template-compiler.md)
- Same pin, `??` defaults: [Roc nightly record
  defaults](/research/rocci/roc-nightly-record-defaults.md)

[^plan]: Phase 0–6 pins; Parse/Template isolation; basic-cli argv/Path/Stdin/Exit; host `roc check`.
[^research]: Dual implementation; Rust stays product; template subset only.
[^roc-defaults]: Pattern `??` illegal on this pin; type-position defaults exist.
[^inventory]: Product nightly `nightly-2026-08-23-fb208ba`.
[^cursor]: `var $cur` plus `{ ..$cur, field: n }`; `skip_formatting_ws` skips newline then spaces.
[^parse]: `parse = do_parse`; `Hit`/`Miss`; `Component(ComponentDecl)`; no `Template` import.
[^template]: `compile = do_compile_src`; `BodyCss`; `Forest`/`NoForest`; `item_kind` strings; raw-byte file walk; fnv1a `/ 16`.
[^lower]: `compile = do_compile`; imports `Parse` only; `U64.bitwise_xor` / `bitwise_and`; `$v / 16`.
[^app]: `UnixBytes` argv; `Path.read_utf8!`; `Stdin.line!()`; `Err(Exit(1))`.
[^readme]: POC driver only; do not change product CLI docs.
[^roc-tutorial]: New-compiler `var`, `for`, `expect`, packages; no `.rocci` imports.
[^roc-nightly]: `fb208ba` merge used as the product pin.
