---
type: Research Report
title: Ungrammar as AST spec for Rocci and Rocdown
description: "Exploratory research: keep hand-written scanners and parsers; use ungrammar as the tree spec for both languages and generate owned AST structs. Not shipped."
tags: [domain/rocci, domain/rocdown, concern/syntax, concern/architecture, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-19T16:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: ungram-intro
    resource: https://rust-analyzer.github.io/blog/2020/10/24/introducing-ungrammar.html
    title: Introducing Ungrammar
    author: human:matklad
    last_modified: 2020-10-24
  - id: ungrammar-crate
    resource: https://crates.io/crates/ungrammar
    title: ungrammar crate 1.16.1
    author: organization:rust-analyzer
    last_modified: 2022-03-05
  - id: ra-arch
    resource: https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/architecture.md
    title: rust-analyzer architecture, syntax and codegen
    author: organization:rust-lang
    last_modified: 2026-08-19
  - id: rocdown-ungram
    resource: ../../../crates/rocci-rocdown/Rocdown.AST.ungram
    title: Draft Rocdown document AST ungrammar
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-ast
    resource: ../../../crates/rocci-rocdown/src/ast.rs
    title: Shipped Rocdown document AST
    author: process:git
    last_modified: 2026-08-19
  - id: template-ast
    resource: ../../../crates/rocci-template/src/ast.rs
    title: Shipped Rocci template AST
    author: process:git
    last_modified: 2026-08-17
  - id: template-parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Rocci hand-written recursive-descent parser
    author: process:git
    last_modified: 2026-08-17
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Implemented Rocci template language reference
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-scan
    resource: ../../../crates/rocci-rocdown/src/scan.rs
    title: Rocdown document-root scanner
    author: process:git
    last_modified: 2026-08-19
  - id: rocdown-markdown
    resource: ../../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak conversion and heading-id assignment
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-pprint
    resource: ../../../crates/rocci-rocdown/src/pprint.rs
    title: Rocdown format_ast inspect contract
    author: process:git
    last_modified: 2026-08-19
  - id: template-pprint
    resource: ../../../crates/rocci-template/src/pprint.rs
    title: Rocci format_ast inspect contract
    author: process:git
    last_modified: 2026-08-15
  - id: ast-test
    resource: ../../../crates/rocci-rocdown/tests/ast.rs
    title: Rocdown AST inspect fixture and ungram name drift test
    author: process:git
    last_modified: 2026-08-19
  - id: block-research
    resource: ../generalized-rocdown-block-model.md
    title: Generalized Rocdown block model research
    author: process:cursor
    last_modified: 2026-08-19
  - id: block-plan
    resource: ../../plans/rocdown/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: ungram-plan
    resource: ../../plans/rocci/ungram-ast.md
    title: Ungrammar AST codegen implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: follow-ons
    resource: ../../plans/rocci/ungram-follow-ons.md
    title: Ungram follow-on backends after owned-struct codegen
    author: process:cursor
    last_modified: 2026-08-19
  - id: format-arch
    resource: ../../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: language-tooling
    resource: ../../architecture/language-tooling.md
    title: Rocci language-tooling boundary
    author: process:cursor
    last_modified: 2026-08-18
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-18
  - id: workspace-deps
    resource: ../../../tools/rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace package class and edge checker
    author: process:git
    last_modified: 2026-08-18
  - id: cargo-workspace
    resource: ../../../Cargo.toml
    title: Cargo workspace manifest
    author: process:git
    last_modified: 2026-08-18
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Keep Rocdown Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: product-boundary
    resource: ../../decisions/consolidate-rocdown-product-boundary.md
    title: Consolidate the Rocdown format and documentation generator
    author: process:cursor
    last_modified: 2026-08-17
  - id: template-lib
    resource: ../../../crates/rocci-template/src/lib.rs
    title: rocci-template public AST exports
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-lib
    resource: ../../../crates/rocci-rocdown/src/lib.rs
    title: rocci-rocdown public AST exports
    author: process:git
    last_modified: 2026-08-19
---

# Ungrammar as AST spec for Rocci and Rocdown

## Research question

Should Rocci (`.rocci`) and Rocdown (`.rocdown`) both treat ungrammar as the
canonical description of their parse trees, and generate owned Rust AST
structs from those files, without generating scanners or parsers?

Sub-questions:

1. What problem does ungrammar actually solve, versus EBNF / parser
   generators / a lossless CST library?
2. Does the existing `Rocdown.AST.ungram` already match that role, and what
   would a `Rocci.AST.ungram` contain?
3. Can codegen emit the *owned* structs these crates use today, or does
   ungrammar only pay off if we adopt a rust-analyzer-style CST?
4. Where should a generator live so it does not reverse the Rocci / Rocdown
   product boundary?
5. What stays hand-written: scanners, parsers, helpers, Markdown, inspect
   printers, analysis types?

This is not a language change and not a description of shipped behavior.
Crate READMEs and architecture records remain the current contract.[^format-arch]

## Topic background

Rocci has no generated grammar. The shipped `.rocci` contract is the
`rocci-template` README plus hand-written types in `ast.rs`. The parser is
recursive descent over a cursor; ordinary Roc outside `@` declarations is
copied as an opaque span.[^template-readme][^template-ast][^template-parser]

Rocdown is Markdown-first with document-root `@` islands and line-start
`:kind` article blocks. Comrak still owns Markdown sugar. The document
scanner is hand-written and mode-switching. The shipped tree is a hybrid:
`Item::Markdown(MdNode)` for Comrak blocks, `Item::Block(BlockCall)` for
`:name[params]`, and re-exported `rocci-template` declaration types for
Rocci islands.[^rocdown-readme][^rocdown-ast][^rocdown-scan][^rocdown-markdown][^markdown-first]

The [generalized Rocdown block model](/research/rocdown/generalized-rocdown-block-model.md)
already decided that ungrammar describes **node types, not the scanner**.
It checked in `crates/rocci-rocdown/Rocdown.AST.ungram` and a weak drift
test: production names exist as Rust types. The block-model plan deferred
actual AST codegen to a follow-on. A second ungram for `rocci-template` was
left optional.[^block-research][^block-plan][^rocdown-ungram][^ast-test]

Ungrammar, as rust-analyzer defined it, is a notation for **tree shape**.
It is not a specification of the language of strings. Parser generators
that emit both parser and tree force the tree to follow parser-technical
rewrites (left-recursion, precedence). Ungrammar drops that coupling and
pairs with a hand-written parser. Producing a parser is an explicit
non-goal of the `ungrammar` crate.[^ungram-intro][^ungrammar-crate]

rust-analyzer uses that split: hand-written lexer and parser, generated
`SyntaxKind` plus typed wrappers over a lossless `rowan` CST. Generated
code is committed; `cargo xtask codegen` refreshes it. The syntax crate
does not bootstrap rust-analyzer to generate itself.[^ra-arch]

## For a later agent

- **Authority:** exploratory. Do not treat this as shipped, approved, or as
  a replacement for crate READMEs.
- **Do not implement** codegen unless the user asks. Delivery lives in the
  [ungram AST plan](/plans/rocci/ungram-ast.md).[^ungram-plan]
- Keep scanners and parsers hand-written. Do not generate them from ungram.
- Do not introduce `rowan` / a CST in this workstream.
- Do not put Rocdown-specific nodes into a Rocci ungram, or Rocci template
  internals into a Rocdown ungram beyond opaque / foreign references.
- Do not add ungrammar or Rocdown declarations to `knowledge/**/*.md`.

## Recommendation

**Yes: embrace ungrammar for both languages as the tree spec, and generate
owned AST data types from it.**

Do that with these bounds:

| Do | Do not |
| --- | --- |
| Two ungram files, one per language | One merged grammar |
| Generate owned structs / enums + `span` | Generate scanners, parsers, or `rowan` wrappers |
| Commit generated Rust | `build.rs` that runs ungram during every compile |
| Hand-written `impl` blocks beside generated types | Put `walk`, `format_ast`, or param-parsing into generated files |
| Opaque leaves for Roc, CSS, Markdown inlines | A second Markdown or Roc parser in ungram |
| Shared generator crate classified as base Rocci | Runtime dependency from language crates on the generator |

The payoff is a readable, greppable tree contract that language-dev can
edit first, plus mechanical structs that cannot silently drift. The
scanners stay in Rust because they encode mode changes, recovery, brace
skipping, and Comrak holes that ungrammar cannot and should not
express.[^ungram-intro][^language-dev][^rocdown-scan][^template-parser]

## What ungrammar is for here

Ungrammar is a good fit for Rocci and Rocdown for the same reason it
exists: the **API tree** is the thing we want to stabilize, and it is not
the same object as the **scanner**.

A `.rocci` file is mostly ordinary Roc. The interesting tree is the
bounded template grammar inside `@component` bodies and the handful of
module declarations. An EBNF of the whole file would be dominated by
“Roc until the next top-level `@`”, which is a scanner rule, not a node
shape.[^template-readme][^template-parser]

A `.rocdown` file is mostly Markdown. The interesting tree is module
islands plus article `BlockCall`. Comrak already recognizes headings,
lists, and fences. An ungram that tried to specify CommonMark would fight
the Markdown-first decision and duplicate Comrak.[^rocdown-readme][^markdown-first][^rocdown-markdown]

The existing Rocdown ungram already writes that split down: tokens such as
`'@'` `'page'` document source spelling; `'rocci-component'` and
`'md-inline'` mark foreign or later trees; `BlockCall` /
`BracketRecord` / `BlockContent` are the nodes codegen should
own.[^rocdown-ungram]

That dual use — human grammar reference *and* AST spec — is the
rust-analyzer lesson that the grammars turn out readable enough to cite.
It is also why unlabeled tokens should stay in the ungram even when the
owned struct omits them: `'{{'` / `'}}'` teach wrapping; they are not
Rust fields.[^ungram-intro][^rocdown-ungram]

## What we would not generate

### Scanners and parsers

Rocci’s cursor skips trivia, recognizes top-level declarations, and
`skip_balanced_braces` understands Roc strings and `#` comments, not
Markdown fences. Rocdown’s scanner classifies line-start `@`, `:`, and
`<`, punches Comrak holes, and re-enters `parse_fragment` for nested
article bodies. Those loops must stay terminating on unclosed input.
Ungrammar has no recovery, no “document root versus list item”, and no
precedence. Generating them would be the mistake ungrammar was invented
to avoid.[^template-parser][^rocdown-scan][^language-dev][^ungram-intro]

### A lossless CST

rust-analyzer generates *wrappers* over `rowan` because IDEs need
whitespace, comments, and partial trees. Rocci and Rocdown lower to Roc
and inspect an S-expression. The shipped types are owned structs with
byte spans and extra methods (`span()`, `is_colon()`, `walk`,
`text_content`). Switching to a CST is a different product (lossless
editing, syntax rewriting). It is not required to get a grammar
reference or generated node types.[^ra-arch][^rocdown-ast][^template-ast][^language-tooling]

If an editor later needs a CST, ungrams can feed `SyntaxKind` then. That
is a follow-on, not a prerequisite.

### Inspect printers, analysis types, helpers

`format_ast` is a public debugging contract with fixture tests. Generating
it in v1 would freeze inspect names to ungram labels and churn
`test/AllSyntax` output for no parser gain.[^template-pprint][^rocdown-pprint][^ast-test]

Stay hand-written:

- `ParsedParams`, `parse_component_params`, `strip_param_defaults`
- `PageMeta`, `HeadingInfo`, `LinkInfo`
- `MdNode` walk / text helpers
- `BlockCall::is_colon`, `payload_span`, `BlockContent::scope_name`

Pattern: generated `ast.generated.rs` (data types only) plus `ast.rs`
`impl` blocks and analysis structs, as rust-analyzer splits
`ast/generated` from `node_ext`.[^ra-arch][^template-ast]

## Owned-struct lowering, not CST wrappers

The `ungrammar` crate parses the DSL into nodes, tokens, and rules. It
does not dictate the backend. rust-analyzer lowers to CST accessors.
Biome vendors a related dialect for the same CST purpose. Rocci should
lower to **owned types** that match today’s `#[derive(Clone, Debug,
PartialEq, Eq)]` structs.[^ungrammar-crate][^template-ast][^rocdown-ast]

Restrict the dialect so lowering is deterministic:

| Ungram form | Generated Rust |
| --- | --- |
| `Name = A \| B \| C` (node names only) | `enum Name { A(A), B(B), C(C) }` plus `fn span(&self)` |
| `Name = label:T …` (sequence) | `struct Name { …, pub span: Span }` |
| `T*` | `Vec<T>` |
| `T?` | `Option<T>` |
| unlabeled `'token'` | omitted from the struct; kept in the ungram as spelling |
| `Name = 'leaf-token'` | newtype with payload + `span` (see opaque leaves) |
| recursive `TemplateItem` in a struct | `Box<TemplateItem>` where a field would be infinite |

Reject, or require a sidecar, anything else: nested anonymous
alternatives, unlabeled repeated tokens, mixed token/node enums. The
generator owns those restrictions; do not fork the ungrammar language.

**Always attach `span: Span`.** Named extra spans (`name_span`) should
come from a labeled `Ident` (or a leaf with its own span), not from
ad-hoc generator magic. Prefer Rocci’s `Ident { name, span }` over
Rocdown’s parallel `name: String` + `name_span: Span` once codegen
owns both.[^template-ast][^rocdown-ast]

## Two grammars, shared generator

Keep separate files:

- `crates/rocci-template/Rocci.AST.ungram`
- `crates/rocci-rocdown/Rocdown.AST.ungram` (already drafted)

Rocdown may name Rocci declarations as **foreign** nodes. It must not
re-generate `ComponentDecl`. Today those types are re-exported from
`rocci-template`. Ungrammar has no import statement; a small sidecar
(for example `Rocdown.AST.toml`) should map `ComponentDecl` →
`rocci_template::ComponentDecl` and skip emitting a struct.[^rocdown-ungram][^rocdown-lib][^template-lib]

A merged ungram would blur the product boundary: Rocci owns the template
language; Rocdown owns document scanning and article trees; Rocdown
reuses Rocci parsing for islands rather than cloning the grammar.
The generator may be shared. The grammars must not be.[^product-boundary][^language-dev]

**Generator placement.** Add a workspace crate such as `rocci-ungram`,
classified as **base-rocci** in `tools/rocci-ops/src/rocci_ops/workspace_deps.py`. It
depends on the `ungrammar` crate. Language crates do **not** depend on
it at runtime. `cargo run -p rocci-ungram -- generate` writes committed
`ast.generated.rs` files; `--check` fails CI on drift. No
`build.rs` in `rocci-template` or `rocci-rocdown`. That matches
rust-analyzer’s “avoid bootstrapping” rule and keeps default tests
sub-second.[^workspace-deps][^cargo-workspace][^ra-arch][^language-dev]

`rocci-template` must not depend on `rocci-rocdown`. A base-rocci
generator used only as a CLI does not create that edge. Rocdown may
depend on generated Rocci types the way it already depends on
`rocci-template`.[^product-boundary][^workspace-deps]

OKF’s YAML AST is out of scope. It is not a Rocci or Rocdown source
language.

## Current Rocdown ungram versus shipped AST

The draft ungram is already useful and already slightly ahead of, and
beside, the shipped tree.[^rocdown-ungram][^rocdown-ast]

Aligned today: `Document`, `BlockCall`, `BracketRecord`, `BracketList`,
`ParamField`, `ParamValue`, `BlockContent`, `LineContent`,
`BraceSection`, `EndSection`, `EndMarker`, `UseDecl`. The drift test
locks those names, not their fields.[^ast-test]

Intentional opacity: Rocci declarations as `'rocci-component'` and
friends; `BracePayload`; `Inline = 'md-inline'`;
`TemplateSplice = 'rocci-splice'`; `HtmlIsland = 'html-island'`.

Still a hybrid in Rust: `Item` is a single enum of Markdown, module
decls, template splices, *and* `Block`. The ungram splits
`Item = ModuleItem | Block` and then lists Markdown as `Paragraph` /
`List` / … under `Block`. Shipped code still wraps Comrak output as
`Item::Markdown(MdNode)`, including headings that sugar-lower to
`BlockCall` elsewhere. `MdNode` is a full inline tree (emphasis, links,
footnotes) that the ungram explicitly postpones.[^rocdown-ast][^rocdown-ungram][^rocdown-markdown]

Codegen v1 should generate the **shipped article and module nodes**, not
rewrite Markdown into the ungram’s target `Block` layer. Replacing
`Item::Markdown` with ungram `Paragraph` / `List` is a language-tree
change owned by the block-model workstream, not by AST codegen. Until
that tree is the runtime tree, treat Markdown productions in the ungram
as documentation, or mark them foreign to `MdNode`.[^block-plan][^rocdown-ast]

The name-only drift test would still pass if `BlockCall` lost `params`.
Generated structs, plus `--check`, are the real lock.

`bravo/Bravo.AST.ungram` is cited by the block-model research as
inspiration for line content versus delimited sections. That path is
**not in this repository**; do not treat it as a local source of
truth.[^block-research]

## Sketch: `Rocci.AST.ungram`

No Rocci ungram exists yet. A first draft should describe the shipped
template tree, with Roc payloads as opaque leaves:[^template-ast]

```text
// Draft Rocci template AST.
// Generate node types from this file. Do not generate the scanner.
// Ordinary Roc outside recognized declarations is opaque.
// Spans are attached by codegen.

Document =
  items:ModuleItem*

ModuleItem =
  RocRegion
| ComponentDecl
| FixtureDecl
| CssDecl
| ContextDecl
| InitDecl
| OnDecl

RocRegion = 'roc-region'

ComponentDecl =
  '@' 'component'
  name:Ident
  params:ParamList
  body:TemplateBlock

FixtureDecl =
  '@' 'fixture'
  name:Ident
  target:ComponentPath
  value:RocExpr

CssDecl =
  '@' 'css'
  body:CssPayload

ContextDecl =
  '@' 'context'
  ty:RocType

InitDecl =
  '@' 'init'
  body:RocExpr

OnDecl =
  '@' 'on'
  method:Ident
  path:StringLit
  params:RocExpr?
  body:RocExpr

TemplateBlock =
  items:TemplateItem*

TemplateItem =
  Element
| ComponentCall
| Fragment
| TextNode
| Interpolation
| IfDirective
| ForDirective
| MatchDirective
| LetDirective
| CssDecl

Element =
  name:Ident
  attrs:Attr*
  children:TemplateItem*

ComponentCall =
  path:ComponentPath
  attrs:Attr*
  children:TemplateItem*

Fragment =
  children:TemplateItem*

TextNode = 'text'
Interpolation = expr:RocExpr

Attr =
  name:Ident
  value:AttrValue

AttrValue =
  Static
| Expr
| Action
| Boolean

Static = value:StringLit
Expr = expr:RocExpr
Action =
  name:Ident
  args:RocExpr
Boolean = 'boolean-attr'

IfDirective =
  condition:RocExpr
  then_body:TemplateBlock
  else_ifs:ElseIf*
  else_body:TemplateBlock?

ElseIf =
  condition:RocExpr
  body:TemplateBlock

ForDirective =
  binder:Ident
  collection:RocExpr
  body:TemplateBlock

MatchDirective =
  scrutinee:RocExpr
  arms:MatchArm*

MatchArm =
  pattern:RocPattern
  value:TemplateItem

LetDirective =
  binder:Ident
  expr:RocExpr

ComponentPath = parts:Ident*

ParamList = 'param-list'
RocExpr = 'roc-expr'
RocType = 'roc-type'
RocPattern = 'roc-pattern'
CssPayload = 'css-payload'
Ident = 'ident'
StringLit = 'string'
```

Flags that are not tree children (`Element.self_closing`,
`ComponentCall.children: Option<_>`) can stay in a hand-written `impl`
or be labeled leaves. `ParsedParams` stays out: it is derived from
`ParamList` text, not a node.[^template-ast]

## Risks

- **Ungram mistaken for “the grammar.”** Readers may think scanners are
  specified. Header comments and crate READMEs must keep saying: tree
  spec, not lexer. Public language references stay prose plus examples.
- **Editing generated files.** `--check` in CI is mandatory; generated
  files should say `// @generated`.
- **Foreign nodes.** Generating dummy `ComponentDecl` in Rocdown would
  clash with `rocci_template::ComponentDecl`. Sidecar mappings are part
  of v1, not polish.
- **Markdown hybrid.** Generating ungram `Paragraph` while parsers still
  produce `MdNode` would fork the tree. Generate what the parser
  constructs.
- **Scope creep into pprint / LSP tokens.** Useful later; they are
  separate contracts.
- **Crate classification.** An unclassified `rocci-ungram` workspace
  member fails CI. It belongs in `BASE_ROCCI`.[^workspace-deps]

## Alternatives considered

**Keep hand-written AST only.** Cheap, and the Rocci tree is still
small (~20 node types). Rejected as the long-term contract because
Rocdown already showed the failure mode: ungram and `ast.rs` can disagree
on fields while the name-only test stays green. Two languages make a
shared lowering worth one crate.

**Drift tests without codegen.** A stronger test could parse the ungram
and compare field names. That is a valid Phase-2 checkpoint. It does not
remove the duplicate struct bodies. Once the ungram is canonical,
generating the types is less work than maintaining a schema checker that
almost generates them.

**Adopt `rowan` now.** Pays off for lossless editing and error-tolerant
wrappers. Rocci is not there. It would touch every parser, lowerer, and
inspect path.

**Parser generator (pest, lalrpop, tree-sitter).** Solves a different
problem and fights island scanning. Tree-sitter might still appear later
for editors; it should not own the compiler AST.

**Per-crate copy-pasted generators.** Avoids a workspace member, and
duplicates lowering rules. Worse than one base-rocci CLI.

## Open questions

1. Sidecar format for foreign nodes and opaque leaf Rust types (`Span`
   versus `String` + `Span`): TOML next to the ungram, or flags on the
   generate command?
2. Should generated enums use tuple variants (`Item::Block(BlockCall)`)
   to match today, or named fields?
3. When, if ever, should `Item::Markdown(MdNode)` move to ungram
   `Paragraph` / `List` nodes? Owned by the block-model tree, not this
   codegen.
4. Is `format_ast` generation wanted once inspect tags stabilize, or is
   the S-expression a hand-written facade forever? Delivery answer is in
   the [follow-on plan](/plans/rocci/ungram-follow-ons.md): generate the
   walker; keep tags, atoms, and truncation as a facade.[^follow-ons]
5. Should public reference pages (`docs/reference/rocci.rocdown`,
   `docs/reference/rocdown.rocdown`) cite the ungrams as the tree
   appendix, or keep them developer-only? Delivery answer: generated
   name/tag appendix pages, not a raw ungram paste.[^follow-ons]

## Recommended next work

Phased v1 delivery is in the [ungram AST implementation
plan](/plans/rocci/ungram-ast.md). Follow-ons after owned-struct codegen are
in the [ungram follow-on backends plan](/plans/rocci/ungram-follow-ons.md).
Do not start a phase until asked.[^ungram-plan][^follow-ons]

[^ungram-intro]: Ungrammar describes concrete syntax trees, not the language of strings; pair it with a hand-written parser; do not generate the parser.
[^ungrammar-crate]: `ungrammar` 1.16.1 parses the DSL; producing a parser is an explicit non-goal.
[^ra-arch]: rust-analyzer generates syntax kinds and AST wrappers from ungram via committed codegen; syntax is independent of the rest of the IDE; avoid bootstrapping.
[^rocdown-ungram]: Draft document tree with module islands, `BlockCall`, bracket params, opaque Rocci and Markdown leaves; header says generate node types, not the scanner.
[^rocdown-ast]: Shipped hybrid `Item` with `Markdown(MdNode)`, module decls, `Block(BlockCall)`, and extra analysis types.
[^template-ast]: Shipped owned template nodes, `Ident`, opaque Roc spans, and `ParsedParams` helpers in the same file.
[^template-parser]: Recursive descent; top-level Roc regions are opaque spans between declarations.
[^template-readme]: File shape, reserved `@` declarations, ordinary Roc copied through; `format_ast` is the inspect tree.
[^rocdown-readme]: Markdown-first islands, reserved `@` names, `:kind` article blocks, HTML islands; README is the shipped contract.
[^rocdown-scan]: Document-root `@` / `:` / `<` classification and reserved-name table; leftover experimental tokens are a removal diagnostic.
[^rocdown-markdown]: Comrak conversion, heading ids, and hole punching around declarations.
[^rocdown-pprint]: Hand-written S-expression inspect for Rocdown items and block calls.
[^template-pprint]: Hand-written S-expression inspect for Rocci modules.
[^ast-test]: AllSyntax inspect fixture plus a name-only ungram drift test.
[^block-research]: Ungram as AST spec not scanner; Rocci ungram optional; Bravo cited as inspiration; open question on CI drift.
[^block-plan]: v1 hand-writes matching types; generating node types is a follow-on; do not generate the scanner.
[^ungram-plan]: Phased owned-struct codegen; exploratory until a phase is requested.
[^follow-ons]: Inspect exhaustiveness first; NodeKind not SyntaxKind; Markdown ungram generates MdNode; CST deferred.
[^format-arch]: Descriptive Rocdown format boundary; architecture stays the shipped contract.
[^language-tooling]: LSP consumes compiler trees and token spans; no CST layer today.
[^language-dev]: Parser and lowering tests stay server-free; scanners must terminate; inspect AST is part of the language-dev loop.
[^workspace-deps]: Unclassified workspace members fail; base Rocci must not depend on Rocdown.
[^cargo-workspace]: Current member list has no generator or xtask crate.
[^markdown-first]: Mode changes at visible block boundaries, not mid-sentence.
[^product-boundary]: Rocci owns the template language; Rocdown owns the document format and must reuse Rocci parsing for islands.
[^template-lib]: Public export of template AST types used by Rocdown.
[^rocdown-lib]: Re-exports template types and Rocdown document nodes together.
