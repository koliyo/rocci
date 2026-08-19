---
type: Implementation Plan
title: Ungram follow-on backends after owned-struct codegen
description: "After v1 AST struct codegen: freeze inspect tags and generate exhaustive format_ast walkers, add NodeKind coverage for highlighters, generate MdNode from a second Markdown ungram, and emit a public tree appendix. Do not generate SyntaxKind, highlighters, scanners, or a CST."
tags: [domain/rocci, domain/rocdown, concern/syntax, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T16:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: ungram-plan
    resource: ungram-ast.md
    title: Ungrammar AST codegen for Rocci and Rocdown
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../research/ungram-ast.md
    title: Ungrammar as AST spec for Rocci and Rocdown
    author: process:cursor
    last_modified: 2026-08-19
  - id: pprint-template
    resource: ../../crates/rocci-template/src/pprint.rs
    title: Rocci format_ast inspect printer
    author: process:git
    last_modified: 2026-08-15
  - id: pprint-rocdown
    resource: ../../crates/rocci-rocdown/src/pprint.rs
    title: Rocdown format_ast inspect printer
    author: process:git
    last_modified: 2026-08-19
  - id: ast-fixture
    resource: ../../crates/rocci-rocdown/tests/fixtures/all_syntax.ast
    title: Rocdown AllSyntax inspect fixture
    author: process:git
    last_modified: 2026-08-19
  - id: ast-test
    resource: ../../crates/rocci-rocdown/tests/ast.rs
    title: Rocdown AllSyntax inspect lock
    author: process:git
    last_modified: 2026-08-19
  - id: mdnode
    resource: ../../crates/rocci-rocdown/src/ast.rs
    title: Hand-written MdNode beside generated document nodes
    author: process:git
    last_modified: 2026-08-19
  - id: markdown
    resource: ../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak to MdNode conversion
    author: process:git
    last_modified: 2026-08-17
  - id: highlight-token
    resource: ../../crates/rocci-highlight/src/token.rs
    title: HighlightKind semantic-token vocabulary
    author: process:git
    last_modified: 2026-08-17
  - id: highlight-composite
    resource: ../../crates/rocci-highlight/src/composite.rs
    title: Rocci host-language highlight collector
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-highlight
    resource: ../../crates/rocci-rocdown/src/highlight.rs
    title: Rocdown host-language highlight collector
    author: process:git
    last_modified: 2026-08-19
  - id: lsp-tokens
    resource: ../../crates/rocci-lsp/src/tokens.rs
    title: LSP semantic-token legend and composition
    author: process:git
    last_modified: 2026-08-17
  - id: ungram-lib
    resource: ../../crates/rocci-ungram/src/lib.rs
    title: rocci-ungram generate and check entry points
    author: process:git
    last_modified: 2026-08-19
  - id: ungram-emit
    resource: ../../crates/rocci-ungram/src/emit.rs
    title: Owned-struct Rust emitter
    author: process:git
    last_modified: 2026-08-19
  - id: ungram-readme
    resource: ../../crates/rocci-ungram/README.md
    title: rocci-ungram CLI contract
    author: process:git
    last_modified: 2026-08-19
  - id: rocci-ungram-file
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Rocci template tree spec
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-ungram
    resource: ../../crates/rocci-rocdown/Rocdown.AST.ungram
    title: Rocdown document tree spec
    author: process:git
    last_modified: 2026-08-19
  - id: rocci-sidecar
    resource: ../../crates/rocci-template/Rocci.AST.toml
    title: Rocci ungram sidecar
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-sidecar
    resource: ../../crates/rocci-rocdown/Rocdown.AST.toml
    title: Rocdown ungram sidecar
    author: process:git
    last_modified: 2026-08-19
  - id: language-tooling
    resource: ../architecture/language-tooling.md
    title: Rocci language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-18
  - id: language-server
    resource: language-server.md
    title: Full Rocci and Rocdown language tooling plan
    author: process:codex
    last_modified: 2026-08-18
  - id: block-plan
    resource: generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: inspector-plan
    resource: inspector-source-views.md
    title: Preview inspector source views
    author: process:cursor
    last_modified: 2026-08-19
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-19
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Implemented Rocci template language reference
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rocci
    resource: ../../docs/reference/rocci.rocdown
    title: Public Rocci language reference
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rocdown
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: playground
    resource: ../../crates/rocci-playground/src/compiler.rs
    title: Playground S-expression highlighter
    author: process:git
    last_modified: 2026-08-19
  - id: ra-arch
    resource: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md
    title: rust-analyzer architecture, syntax and codegen
    author: organization:rust-lang
    last_modified: 2026-08-19
  - id: ungram-intro
    resource: https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html
    title: Introducing Ungrammar
    author: human:matklad
    last_modified: 2020-10-24
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Consolidate the Rocdown format and documentation generator
    author: process:cursor
    last_modified: 2026-08-17
---

# Ungram follow-on backends after owned-struct codegen

## Purpose and authority

This is the implementation plan for the [ungram AST plan](ungram-ast.md)
follow-ons that v1 deferred. It is exploratory until a human reviewer
accepts a scope. It does not describe shipped behavior. Crate READMEs
remain the language contract.[^ungram-plan][^research][^template-readme][^rocdown-readme]

Do not start a phase until the user asks to implement it. Prefer the
branch that already has v1 `rocci-ungram --check` in CI. Use
`rocci-language-dev` for inspect-fixture and ungram edits.[^language-dev][^ungram-readme]

## Goal

Reuse the same ungrams as extra **committed backends**, so a new node
cannot appear in the tree spec without also appearing in inspect, and so
`MdNode` can lock to a second ungram the way article nodes already lock
to `Rocdown.AST.ungram` and template nodes lock to
`Rocci.AST.ungram`.[^rocci-ungram-file][^rocdown-ungram]

After the last in-scope phase:

- Inspect tags are a sidecar contract, not tribal knowledge in
  `pprint.rs`.
- `format_ast` walkers are generated and exhaustive; atom layout and
  truncation stay hand-written overlays.
- A crate-private `NodeKind` (not rust-analyzer `SyntaxKind`) exists for
  generated productions, and highlight collectors are tested for
  coverage against it.
- `Rocdown.Markdown.ungram` generates the shipped Comrak projection
  (`MdNode`). Comrak remains the recognizer.
- Public reference pages link a generated tree appendix of production
  names and inspect tags. Knowledge records stay inert Markdown.

## What we should do, and what we should not

The v1 follow-on list mixed five different products that share an input
file. Only some pay off on an **owned AST with omitted unlabeled
tokens**.[^ungram-plan][^research][^ungram-intro]

| Follow-on | Verdict | Why |
| --- | --- | --- |
| Generate `format_ast` | **Do** (Phases 1–2) | Highest drift risk after struct codegen; language-dev, CLI inspect, playground, and the planned preview inspector all consume the dump |
| `SyntaxKind` / highlight from ungrams | **Do NodeKind + coverage only** (Phase 3). Do not generate `SyntaxKind` or highlight collectors | rust-analyzer `SyntaxKind` is a CST token+node enum. Rocci paints `HighlightKind` by walking owned nodes and scanning source for `@component` |
| `Rocdown.Markdown.ungram` | **Do** (Phase 4) as a second tree, generating shipped `MdNode` | Keep Comrak; do not generate doc-only `Paragraph` / `List` as `Item` variants |
| rust-analyzer-style CST | **Defer indefinitely** | No lossless-editing product. Inspect, lowering, and highlight do not need trivia |
| Ungram appendix on public reference pages | **Do** (Phase 5) as a generated node/tag table, not a raw ungram paste | Pointers already exist; the missing piece is a greppable production list |

### Why `format_ast` is the next codegen

v1 stopped at data types. `format_ast` is still a pair of hand-written
walkers whose **heads are not ungram names**: Rocci prints `module` /
`interp` / `call` for `Document` / `Interpolation` / `ComponentCall`;
Rocdown prints `rocdown` / `block` / `h` / `fence` / `md` for `Document`
/ `BlockCall` / `Heading` / `CodeBlock` / leftover `MdNode`. Rocdown
also truncates atoms to 40 characters, reparses nested `:name` bodies
with `parse_fragment`, flattens param fields onto the `block` head, and
collapses template islands to empty tags.[^pprint-template][^pprint-rocdown][^ast-fixture][^ast-test]

A naive generator that used production names as inspect tags would churn
`crates/rocci-rocdown/tests/fixtures/all_syntax.ast`, exact-string Rocci
pprint tests, playground S-expression coloring (words after `(` are
keywords), and the planned inspector AST pane.[^ast-fixture][^playground][^inspector-plan][^language-dev]

The useful generation is **exhaustiveness**, not renaming the dump.
Today a new ungram node can compile and still vanish from inspect. That
is the same failure mode the name-only drift test had for structs.

### Why not rust-analyzer `SyntaxKind`

rust-analyzer generates `SyntaxKind` because the parser emits a lossless
CST: every ungram token and node is a kind the highlighter and typed
wrappers can ask for.[^ra-arch][^ungram-intro]

Rocci and Rocdown do not keep those tokens. Codegen **omits unlabeled
`'@'` / `'component'` / `'{{'`** from the owned structs. Highlight
collectors reconstruct keyword spans from source (`ident_between` for
`@component`, leading `:` for colon blocks, `#` hashes for headings) and
map them onto `HighlightKind` (`Keyword`, `Function`, `Tag`, …), which
already is the LSP semantic-token vocabulary. Embedded Roc, CSS, and
HTML still come from Tree-sitter, composed in `rocci-lsp`.[^highlight-token][^highlight-composite][^rocdown-highlight][^lsp-tokens][^language-tooling]

Generating a CST `SyntaxKind` would invent a layer nothing consumes.
Generating the highlight collectors would re-encode scanner heuristics
the ungram cannot express (recovery, indent, `:` vs definition list).

What *is* worth generating: a **node-kind enum** matching generated
productions, plus a test that every kind is either painted by a
collector or explicitly marked omit in the sidecar (`ParamField`,
`EndMarker`, opaque Roc spans). That catches “new node, no highlight”
without pretending ungram is a highlighter spec. Editor feature work
stays in the [language-server plan](language-server.md).[^language-server]

### Why a second Markdown ungram, not more `Rocdown.AST.ungram`

`Rocdown.AST.ungram` already contains two Markdown stories that must not
be collapsed:[^rocdown-ungram][^rocdown-sidecar][^mdnode][^block-plan]

1. **Shipped runtime:** `Item::Markdown(MdNode)` — Comrak projection,
   including inlines, tables, footnotes, and headings that are still
   `MdNode::Heading`.
2. **Doc-only target article layer:** `Block = BlockCall \| Paragraph \|
   List \| …` — the block-model destination where sugar headings become
   `BlockCall`. Sidecar marks those productions `doc_only`. Generating
   them would invent `Item::Paragraph`.

`MdNode` is still ~20 variants with extra analysis payloads (`HeadingInfo`,
`LinkInfo`, heading ids, footnote numbers). `format_ast` only pretty-prints
a subset and maps the rest to `(md)`. Conversion from Comrak stays
hand-written in `markdown.rs`.[^mdnode][^markdown][^pprint-rocdown]

A `Rocdown.Markdown.ungram` generates that **projection**. It does not
replace Comrak, does not scan Markdown, and does not move
`Paragraph` onto `Item`. If block-model later lowers headings to
`BlockCall`, delete `Heading` from the Markdown ungram in that change.

### Why not a CST

Inspect, lowering, and highlight already operate on owned nodes plus
byte spans. A `rowan` CST pays off for lossless edit, format-on-type,
and syntax rewriting. None of those are a current product goal. If they
become one, the ungrams can grow a CST backend then; that is a new plan,
not a phase here.[^research][^ra-arch][^language-tooling]

## Constraints that do not move

Same table as v1, plus:

| Keep | Meaning for this plan |
| --- | --- |
| Inspect dump stable unless sidecar says otherwise | Unexpected AllSyntax / pprint diffs are bugs |
| `HighlightKind` is the product highlight vocabulary | Do not replace it with ungram token names |
| Comrak owns Markdown sugar | Markdown ungram is a projection spec |
| Doc-only article `Block` / `Paragraph` | Stay documentation until the runtime tree matches |
| Language crates do not depend on `rocci-ungram` | Extra backends are committed files, like `ast.generated.rs` |
| OKF Markdown-only | Do not add ungram or generated appendices under `knowledge/` |

The [block-model plan](generalized-rocdown-block-model.md) owns article
*syntax* and sugar cutover. This plan owns *tree-spec codegen* for
inspect and `MdNode`. The [inspector source-views plan](inspector-source-views.md)
consumes `format_ast` unchanged; it does not own pprint.[^block-plan][^inspector-plan]

## Non-goals

- Generating scanners, lexers, or parsers
- `rowan`, lossless CSTs, or rust-analyzer `SyntaxKind` / `SyntaxToken`
- Generating highlight collectors, Tree-sitter queries, or LSP legends
- Generating `format_ast` tags from raw production names (that would
  rename the dump)
- Generating doc-only `Paragraph` / `List` as Rocdown `Item` variants
- A Roc ungram
- Changing `.rocci` or `.rocdown` source spelling
- Pasting ungrams into `knowledge/**/*.md`

## V1 leftovers this plan freezes

These were open in the research. Delivery answers:[^research]

| Question | Answer here |
| --- | --- |
| Generate `format_ast` or keep it a facade forever? | Generate the **walker**. Keep `Writer`, atoms, truncation, and nested reparse as a facade. |
| Public docs cite ungrams? | Yes, via a **generated appendix** of names and inspect tags. Raw ungram stays in the crates. |
| `Item::Markdown` → ungram `Paragraph`? | Still owned by block-model. Markdown ungram generates `MdNode`, not that article layer. |
| Sidecar vs flags? | Keep TOML sidecars. Inspect tags, NodeKind omit lists, and Markdown extras go there. |

## Layer map

| Concern | Owner |
| --- | --- |
| Extra emit backends (`pprint`, `NodeKind`, Markdown structs, appendix markdown) | `crates/rocci-ungram` (`emit.rs` today writes structs only)[^ungram-emit] |
| Inspect tag / omit / atom-overlay maps | `Rocci.AST.toml`, `Rocdown.AST.toml`, later `Rocdown.Markdown.toml` |
| `Writer` + atom helpers + nested `parse_fragment` | `pprint.rs` in each language crate |
| Generated exhaustive match | `pprint.generated.rs` (committed) |
| Host highlight collectors | stay in `rocci-highlight` / `rocci-rocdown` `highlight.rs` |
| LSP composition / Tree-sitter | `rocci-lsp`, unchanged |
| Comrak conversion | `rocci-rocdown` `markdown.rs` |
| Public tree appendix | `docs/reference/` generated pages, linked from existing pointer sentences |
| Language-dev loop | edit ungram + sidecar inspect tag **before** parser + pprint overlay |

`rocci-template` still must not depend on `rocci-rocdown`. A Markdown
ungram lives only in the Rocdown crate. The generator CLI may read that
path as a file.[^product-boundary][^ungram-lib]

## Delivery phases

Each phase is one mergeable change. Do not start one until asked.

### Phase 1 — Freeze the inspect tag contract

**Bound:** document every inspect head the printers emit today, map it
to an ungram production (or to an explicit omit / fallback), and fail
`--check` when a **generated** production has no inspect mapping. Do not
generate pprint yet.

**Include:**

- `[inspect.tags]` (or equivalent) on both sidecars. Examples that must
  be explicit because they are not the production name:
  Rocci `Document → module`, `Interpolation → interp`,
  `ComponentCall → call`, `TextNode → text`; Rocdown
  `Document → rocdown`, `BlockCall → block`, `MdNode::Heading → h`,
  `Paragraph → p`, `CodeBlock → fence`, leftover `MdNode → md`.[^pprint-template][^pprint-rocdown][^ast-fixture]
- `[inspect.omit]` for nodes that correctly do not print (`SoftBreak`,
  `LineBreak`, empty `RocRegion`).
- `[inspect.fallback]` for open enums (`MdNode` catch-all `(md)`).
- A comment in each `pprint.rs`: tags live in the sidecar; this file
  owns `Writer` and atom policy.
- Tests in `rocci-ungram`: every generated production is tagged, omitted,
  or fallback-covered. Snapshot the mapping.

**Does not:** change AllSyntax output, rename tags to ungram names, or
emit Rust walkers.

**Done when:** a reviewer can walk from inspect head `(block` / `(interp`
to a sidecar entry to a production without guessing, and
`cargo run -q -p rocci-ungram -- check` fails if a new generated node is
added to the ungram without an inspect mapping.

### Phase 2 — Generate exhaustive `format_ast` walkers

**Bound:** emit committed `pprint.generated.rs` (or `mod` inside
`pprint.rs`) with match walkers. Keep `format_ast` as the public
function. Keep `Writer`, quoting, Rocci multiline `(roc …)`, Rocdown
40-character truncation, `nested_items` / `parse_fragment`,
`self_closing`, and param flattening as **hand-written overlays**.

**Include:**

- Default generated behavior for a node with no overlay: print sidecar
  tag, recurse labeled children, skip unlabeled tokens (already absent
  from the IR).
- Overlay hooks per node that today’s printer special-cases
  (`write_block`, `write_md`, `write_element`, `write_on`, …). Moving a
  node to “generated default” is allowed only when the fixture does not
  change.
- Rocdown template islands stay shallow (`(if)`, `(for item)`, …)
  unless an overlay is added; do not start calling into Rocci
  `format_ast` from Rocdown inspect in this phase (that would deepen the
  dump).
- `cargo test -p rocci-template`, `cargo test -p rocci-rocdown`,
  `cargo test -p rocci-ungram`.
- Inspect: `rocci inspect --ast test/AllSyntax.rocci` and
  `rocdown inspect ast test/AllSyntax.rocdown`. Treat unexpected fixture
  diffs as bugs.

**Done when:** deleting a `match` arm in the generated walker is
impossible (exhaustiveness), new ungram nodes fail `--check` until they
have an inspect tag, and AllSyntax inspect text is unchanged.

**Status:** not started.

### Phase 3 — `NodeKind` and highlighter coverage

**Bound:** generate a crate-private `NodeKind` enum for **generated**
productions (and Markdown productions once Phase 4 exists). Add a
coverage test that every kind is handled by the host highlight collector
or listed in `[highlight.omit]`.

**Include:**

- `node_kind.generated.rs` in each language crate, or a single enum per
  ungram next to `ast.generated.rs`.
- Omit list for param-tree internals and opaque leaves that are not
  painted as host tokens (`ParamField`, `BracePayload`, `RocExpr`).
- Do **not** generate `HighlightKind`, LSP `SemanticTokenType`, or
  collector function bodies.
- Run `cargo test -p rocci-highlight` and `cargo test -p rocci-lsp`
  only if a collector match was made exhaustive against `NodeKind`.

**Does not:** change colors, Zed/VS Code legends, or Tree-sitter
queries.

**Done when:** adding a generated node without highlight omit or a
collector branch fails a sub-second test. rust-analyzer `SyntaxKind`
remains out of tree.

### Phase 4 — `Rocdown.Markdown.ungram` generates `MdNode`

**Bound:** add `crates/rocci-rocdown/Rocdown.Markdown.ungram` plus
sidecar. Generate `MdNode` into `md.generated.rs` (keep helpers
`span()`, `children()`, `text_content()`, `walk` in `ast.rs`). Leave
`PageMeta` / `HeadingInfo` / `LinkInfo` hand-written.

**Include:**

- Productions matching shipped variants: `Heading`, `Paragraph`,
  `BlockQuote`, `List`, `Item`, `TaskItem`, `CodeBlock`,
  `ThematicBreak`, `Table`, `TableRow`, `TableCell`, `Text`,
  `SoftBreak`, `LineBreak`, `Code`, `Emph`, `Strong`, `Strikethrough`,
  `FootnoteDefinition`, `FootnoteReference`, `Link`, `Image`,
  `RawHtml`.[^mdnode]
- Extra fields that are not tree children (`Heading.id` / `level`,
  footnote numbers, `Image.alt`, `List.start`) in the sidecar `[extra]`
  / `[add_fields]` style already used for `Element.self_closing`.[^rocci-sidecar]
- Point `Rocdown.AST.ungram` `MdNode = 'md-node'` at the generated type
  (still opaque to the *document* ungram).
- Keep `markdown.rs` conversion hand-written. Keep Comrak.
- Do not emit doc-only `Block` / `Paragraph` from `Rocdown.AST.ungram`.
- Inspect overlays from Phase 1 keep printing `h` / `p` / `fence` /
  `(md)`.
- `cargo test -p rocci-rocdown` and AllSyntax inspect unchanged.

**Sequence with block-model:** generate the **shipped** `MdNode`,
including `Heading`. If a later block-model phase lowers sugar headings
to `BlockCall`, that change removes `Heading` from this ungram. Do not
wait for that cutover; waiting would recreate the v1 “ungram ahead of
the runtime tree” problem.[^block-plan][^rocdown-ungram]

**Done when:** `MdNode` is generated, Comrak conversion still compiles,
and `--check` covers both Rocdown ungrams.

### Phase 5 — Public tree appendix

**Bound:** generate committed appendix pages under `docs/reference/`
that list production name, inspect tag, and classification (generated /
foreign / opaque / doc-only). Link them from the existing pointer
sentences. Do not paste ungram source into knowledge records.

**Include:**

- Something like `docs/reference/rocci-tree.rocdown` and
  `docs/reference/rocdown-tree.rocdown` (names flexible) marked
  generated.
- One-line note: tree spec, not the language of strings; scanners stay
  hand-written.[^docs-rocci][^docs-rocdown][^ungram-intro]
- `--check` fails when the appendix is stale.
- Crate README “Tree spec” sections already point at the ungrams; add
  “inspect tags are in the sidecar / appendix” after Phase 1.

**Does not:** replace `docs/reference/rocci.rocdown` syntax prose, or
add ungram fences to OKF records.

**Done when:** a language-dev change that adds a node updates ungram →
structs → inspect mapping → appendix through one `generate`, and public
docs link the table.

## Explicitly out of this plan (new plan required)

- Lossless CST / `rowan`
- Generating highlight collectors or `SyntaxKind`
- Parser generators
- Promoting doc-only article `Paragraph` / `List` to runtime `Item`s
- Playground or inspector UI (they keep calling `format_ast`)

## Validation

Per phase:

```text
cargo test -p rocci-ungram
cargo fmt --all -- --check
cargo run -q -p rocci-ungram -- check
```

After Phase 2:

```text
cargo test -p rocci-template
cargo test -p rocci-rocdown
cargo run -q -p rocci-cli -- inspect --ast test/AllSyntax.rocci
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
```

After Phase 3: `cargo test -p rocci-highlight` and, if collector
exhaustiveness changed, `cargo test -p rocci-lsp`.

After Phase 4: `cargo test -p rocci-rocdown` again.

After Phase 5: `cargo run -q -p rocci-rocdown-cli -- build docs` and
inspect the new appendix pages.

After knowledge edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Do not set `ROCCI_REQUIRE_ROC=1`. Do not log a phase complete in
`knowledge/log.md` until required GitHub workflows have succeeded on
that revision.

## Open questions that would still change the plan

1. Whether inspect tags should eventually match ungram names (`Interpolation`
   instead of `interp`). That is a **breaking inspect** change and needs
   its own approval; default is keep today’s short heads.
2. Whether Rocdown inspect should recurse into template islands with
   Rocci `format_ast` (deeper dump, fixture churn). Out of scope unless
   asked.
3. Whether `NodeKind` stays crate-private or becomes a playground /
  inspector API. Default private.
4. Appendix as standalone `docs/reference/*-tree.rocdown` pages versus a
   generated section inside the existing language-reference pages.

[^ungram-plan]: v1 follow-ons listed pprint, SyntaxKind, Markdown ungram, CST, and public citation; Phases 1–5 generate owned structs only.
[^research]: Generate owned structs; keep pprint, analysis types, and highlighters hand-written in v1; CST and SyntaxKind are later; inspect generation was an open question.
[^pprint-template]: Hand-written S-expression with heads `module`, `component`, `interp`, `call`, `element`.
[^pprint-rocdown]: Hand-written S-expression with heads `rocdown`, `block`, `h`, `p`, `fence`, `md`; 40-character atoms; nested items reparsed.
[^ast-fixture]: Locked Rocdown inspect dump for AllSyntax.
[^ast-test]: AllSyntax inspect equality plus colon-block assertions.
[^mdnode]: Hand-written `MdNode` enum beside `ast.generated.rs`; analysis types stay out of the parse tree.
[^markdown]: Comrak `NodeValue` conversion into `MdNode`; heading ids and link collection as side tables.
[^highlight-token]: `HighlightKind` is the shared semantic-token vocabulary for LSP and HTML classes.
[^highlight-composite]: Rocci collector reconstructs `@component` / `@css` spans from source, not from stored tokens.
[^rocdown-highlight]: Rocdown collector paints colon markers, block names, and Markdown heading hashes from source.
[^lsp-tokens]: LSP legend is `HighlightKind` indices; Tree-sitter results are composed on top.
[^ungram-lib]: `generate` / `check` write and compare committed `ast.generated.rs` only.
[^ungram-emit]: Emitter writes owned structs and enum `span()` impls; no pprint or kind enum.
[^ungram-readme]: CLI generates node types, not scanners or a CST.
[^rocci-ungram-file]: Template tree spec with opaque Roc leaves and unlabeled spelling tokens.
[^rocdown-ungram]: Document tree plus doc-only article Markdown layer; `MdNode` is opaque.
[^rocci-sidecar]: Extra fields such as `Element.self_closing` already live beside generated types.
[^rocdown-sidecar]: `MdNode` opaque; `Paragraph` / `List` doc-only; foreign Rocci decls.
[^language-tooling]: Host AST is highlight authority; no CST layer.
[^language-server]: Proposed region-aware tooling; does not require ungram SyntaxKind.
[^block-plan]: Owns article syntax and sugar; codegen of `MdNode` was out of its v1.
[^inspector-plan]: Preview Dev pane shows existing `format_ast` text; does not own pprint.
[^language-dev]: Update `format_ast` when the AST changes; edit ungram before new node fields.
[^template-readme]: README is the `.rocci` contract; ungram is the developer tree spec.
[^rocdown-readme]: README is the `.rocdown` contract; Markdown remains `MdNode`.
[^docs-rocci]: Public reference already points at `Rocci.AST.ungram` as tree spec, not language of strings.
[^docs-rocdown]: Public reference already points at `Rocdown.AST.ungram`; Markdown remains `MdNode`.
[^playground]: Playground colors S-expression heads as keywords; tag renames would retint the AST pane.
[^ra-arch]: rust-analyzer generates SyntaxKind and CST wrappers from ungram via committed codegen.
[^ungram-intro]: Ungrammar describes trees and pairs with a hand-written parser; unlabeled tokens teach spelling.
[^product-boundary]: Template language stays in `rocci-template`; Rocdown reuses it.
