---
type: Research Report
title: Generalized Rocdown block model
description: "Exploratory research for a uniform Rocdown article-block AST. Decision: :name with [params] and {{ }} bodies. Draft AST ungram in crates/rocci-rocdown. Not shipped."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/rendering, concern/architecture, concern/authoring]
status: draft
generated: { by: process:cursor, at: 2026-08-19T08:25:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: ast
    resource: ../../crates/rocci-rocdown/src/ast.rs
    title: Rocdown document AST
    author: process:git
    last_modified: 2026-08-18
  - id: scanner
    resource: ../../crates/rocci-rocdown/src/scan.rs
    title: Rocdown document-root scanner
    author: process:git
    last_modified: 2026-08-17
  - id: parser
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Rocdown parser and fragment re-entry
    author: process:git
    last_modified: 2026-08-17
  - id: docs-rs
    resource: ../../crates/rocci-rocdown/src/docs.rs
    title: Typed @docs projection and field/content split
    author: process:git
    last_modified: 2026-08-18
  - id: article-rs
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Rust Markdown article renderer
    author: process:git
    last_modified: 2026-08-17
  - id: lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Standalone Rocdown lowerer
    author: process:git
    last_modified: 2026-08-18
  - id: docs-rocci
    resource: ../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Rocci documentation widgets
    author: process:git
    last_modified: 2026-08-17
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-17
  - id: lexer
    resource: ../../crates/rocci-template/src/lexer.rs
    title: Balanced-brace skipping
    author: process:git
    last_modified: 2026-08-17
  - id: format-arch
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: markdown-first
    resource: ../decisions/markdown-first-explicit-islands.md
    title: Keep Rocdown Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: generation-research
    resource: rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
    last_modified: 2026-08-18
  - id: compiler-arch
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: rocdown-reference
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: bravo-ungram
    resource: ../../bravo/Bravo.AST.ungram
    title: Bravo document AST ungrammar
    author: human:nils
    last_modified: 2026-08-19
  - id: rocdown-ungram
    resource: ../../crates/rocci-rocdown/Rocdown.AST.ungram
    title: Draft Rocdown document AST ungrammar
    author: process:cursor
    last_modified: 2026-08-19
  - id: syntax-recommended
    resource: syntax/syntax_v2_recommended.rocdown
    title: "Decided v2 spelling: : prefix, bracket params, {{ }} bodies"
    author: process:cursor
    last_modified: 2026-08-19
  - id: syntax-variations
    resource: syntax/syntax_v2_variations.rocdown
    title: "Wrapping, :end, decided : prefix, historical alternatives"
    author: process:cursor
    last_modified: 2026-08-19
  - id: exploration-brief
    resource: ../../generalized-rocdown-input.md
    title: Maintainer brief for generalized Rocdown blocks
    author: human:nils
    last_modified: 2026-08-19
  - id: impl-plan
    resource: ../plans/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
---

# Generalized Rocdown block model

## Research question

Can Rocdown treat every article node as one kind of block — kind, optional
params, optional content — that a Rocci component renders, while remaining a
Markdown-first format that humans like to write?

Sub-questions:

1. What is the common AST for Markdown sugar (`# Heading`) and explicit blocks
   (`:h2`, `:note`)?
2. How should source wrap **params** versus **nested Rocdown content** so they
   are not both `{ }`?
3. Should one-line bodies use **line scope**, like ATX headings?
4. Can block kinds be a **registry** (`:note` as a builtin, `:foo` from an
   import) instead of parser keywords, including dropping `@docs`?
5. Article-block params use **brackets**. `@page` keeps `{ }`. Delimiter
   inconsistency across language modes is accepted.
6. Article blocks do **not** share single `@`. Prefix is **`:`** (not `::`,
   not `!`).

This is not yet a language change. Do not treat sketches as shipped syntax.

## Topic background

Rocdown started as Markdown plus explicit `@` islands. Documentation widgets
were added later as a separate `@docs <kind>` family when the document
language and the documentation generator were different products. Rocdown now
owns both. `@docs note` is leftover namespacing, and the brace body fuses Roc
fields with remainder Markdown. The scanner's brace skip does not understand
fences. Kinds are hardcoded in the parser, in Rust validation, and again in
`DocsComponents.Render`.[^rocdown-readme][^docs-rs][^docs-rocci][^compiler-arch]

The interesting idea is to invert that: parse a document tree, render each
node with a Rocci function `|{ props }, content| -> Html`. Markdown stays
convenient sugar. `:note` is a builtin block type, not a keyword. Custom
types could come from Rocci modules. Bravo's ungram is a related tree
(line content versus delimited sections), not a parser to copy.[^exploration-brief][^bravo-ungram][^pure-render]

Human DX is the acceptance test. There is no compatibility window.

## For a later agent

- **Authority:** exploratory. Architecture records and crate READMEs describe
  shipped behavior; this record does not.
- **Do not implement** the parser unless the user asks. Example files will not
  compile. The draft ungram is for AST node types, not the scanner. The
  implementation plan is
  `knowledge/plans/generalized-rocdown-block-model.md`.[^impl-plan]
- **Read next:** this record, then
  `knowledge/research/syntax/syntax_v2_recommended.rocdown` (`:name[params]`),
  `syntax/syntax_v2_variations.rocdown`,
  `crates/rocci-rocdown/Rocdown.AST.ungram`, and the
  [implementation plan](../plans/generalized-rocdown-block-model.md). The
  maintainer brief is `generalized-rocdown-input.md`. Bravo:
  `bravo/Bravo.AST.ungram`.
- **Keep:** Markdown-first islands, pure `@component` render, Rust catalog /
  Rocci shell, OKF Markdown-only.
- **May break:** `@docs` prefix, mixed `{ fields + markdown }`,
  unknown-`@name`-stays-Markdown, article blocks using `:` and `[params]`
  instead of `@docs`.
- **Owning crates if implemented later:** `rocci-rocdown` (scan, parse, docs,
  lower), `rocci-template` only if Rocci grammar must change, public
  `docs/reference/rocdown.rocdown`.
- **Skill:** `rocci-language-dev` for grammar; `rocci-author` is the wrong
  skill until the syntax exists.

## Scope and authority

This is exploratory language research, not shipped behavior. The current
parser is evidence of today's contract, not a constraint on the next one.
There is no compatibility window: `@docs`, mixed `{ fields + markdown }`, and
other current spellings may disappear if a clearer source form wins. Human
authoring DX is the test, especially line-scoped content, which Markdown
headings already have.[^rocdown-readme][^format-arch][^exploration-brief][^syntax-recommended]

The interesting idea is not a new family of reserved keywords. It is a common
document-block representation: the parser produces a document AST, Markdown is
sugar for some of those nodes, and Rocci component functions render them.[^exploration-brief][^pure-render]

Sample documents for the decided spelling (`:name[params]`) and wrapping
variants live under `knowledge/research/syntax/` and do not parse
yet.[^syntax-recommended][^syntax-variations]

## Current contract

A `.rocdown` file is a sequence of Markdown blocks, reserved `@` declarations,
and document-root HTML islands. Reserved names today are `page`, `roc`,
`render`, `component`, `fixture`, `css`, `context`, `init`, `on`, `if`, `for`,
`match`, `let`, `docs`, and `img`. Unknown `@name` stays Markdown. Recognition
is document-root and header-shaped; inline `@`, email, lists, quotes, and
fences do not switch modes.[^rocdown-readme][^scanner][^markdown-first]

The parse tree is not a uniform block list. `Item` is a sum of Markdown, module
declarations, template splices, `@docs`, and `@img`. Markdown is Comrak-derived
`MdNode`. `@docs` stores a kind string plus one brace body. Static sites then
rebuild a second tree (`ArticleNode`: Markdown, `DocsNode`, or image) and type
`@docs` fields in Rust.[^ast][^docs-rs][^compiler-arch]

`@docs` is one reserved family. Kinds such as `note` or `tabs` are not
top-level `@` names. The body is a single `{ ... }` that mixes leading
`name: value` fields with remainder Markdown. Nested `@docs` are legal;
`@page`, `@roc`, `@render`, `@component`, handlers, and HTML islands inside a
docs body are errors.[^rocdown-reference][^docs-rs]

Brace matching uses `skip_balanced_braces`, which understands Roc strings and
`#` comments but not Markdown fences. A fence containing an unmatched `{` or
`}` can close or swallow a `@docs` body. Field/content splitting is also
heuristic: `split_docs_body` consumes `ident:` values until the remainder no
longer looks like a field.[^scanner][^lexer][^docs-rs]

Rendering is split. Standalone lowering emits `Html.element` for both Markdown
and a conservative `@docs` preview. Static builds render Markdown in Rust and
pass flattened `PlannedSegment` records plus already-rendered body HTML into
one Rocci `Render` matcher keyed on `segment.kind`.[^article-rs][^lowerer][^docs-rocci][^compiler-arch]

Rocci already has the render shape this research wants. A component takes a
props record and optional extra body parameters; paired tags pass nested markup
as that body. There is no magic `children` field.[^template-readme][^pure-render]

`@end` is not part of shipped `@docs`. Include excerpts use comment markers
(`docs-region` / `docs-region-end`) inside the included file. Named `:end.tab`
sketches live in the variations file.[^docs-rs][^syntax-variations]

## The interesting idea

Treat every article node as a block with a kind, optional props, and optional
content, then render it by calling a Rocci component of the same shape:

```text
@component Note = |{ title: string }, content|
    <div class="note">
        <h2>{title}</h2>
        <div>{content}</div>
    </div>
```

Markdown remains the convenient authoring form. `# Heading` and
`:h2 Installing` would lower to the same node. `:note` would not be a
parser keyword; it would be a builtin (or imported) block type that happens
to be in the default registry.[^exploration-brief]

That is already how `@docs` wants to work, except the current pipeline
hard-codes kinds in Rust (`DocsAttrs`, parent/child rules) and again in
`DocsComponents.Render`'s `@match segment.kind`. A generalized model replaces
that closed matcher with a registry from kind to component, and replaces the
flattened segment bag with the component's actual props plus rendered
content.[^docs-rs][^docs-rocci][^generation-research]

Keep module-level declarations out of this protocol. `@page`, `@roc`,
`@component`, `@fixture`, `@css`, `@context`, `@init`, and `@on` declare the
file, not article nodes. The generalization is about what appears in the
article tree: Markdown sugar, `::img` / `!img`, today's `@docs` kinds spelled
as direct names with a distinct prefix, custom blocks, and perhaps
`@render`.[^ast][^rocdown-readme]

## Drop the `@docs` prefix

`@docs` exists because Rocdown-the-language and the documentation generator
were separate products. The family namespaced generator widgets (`note`,
`tabs`, `include`, `example`, `api-operation`) so they would not look like
core language. Rocdown now owns both the document format and the static
generator. The extra prefix is a leftover product boundary, not a language
need.[^rocdown-readme][^compiler-arch][^rocdown-reference]

The intended spelling is the kind as a top-level block name with prefix `:`
(not `@`, not `::`, not `!`):

```text
:note Do not paste raw HTML into Markdown.

:note[title: "Deprecation"] {{
    This API will be removed in 0.4.
}}
```

not `@docs note { ... }` and not `@note`. Nested forms follow the same rule
(`:tabs` / `:tab`). Shipped `@img` already has a top-level name and moves
to the article prefix. `@docs` goes away. No alias, no rewrite
window.[^rocdown-readme][^syntax-recommended]

Kinds are not new reserved keywords. They are names in the builtin block
registry, which is the same mechanism as later `@use` for custom types. Module
names (`page`, `roc`, `component`, …) stay reserved and win when they
collide.[^scanner][^exploration-brief]

The family also mixed general document blocks with generator-only features.
Direct names make that mix visible instead of hiding it under `@docs`:

| Today's kind | Likely role after the prefix drop |
| --- | --- |
| `note`, `tip`, `caution`, `danger`, `deprecated` | General asides |
| `details`, `steps`, `step`, `figure`, `definition`, `tabs`, `tab`, `badge` | General document blocks |
| `include`, `example` | Catalog / authoring tools; keep as `@include` / `@example` if they stay in-document |
| `link-card`, `card-grid`, `file-tree`, `compatibility` | Site chrome widgets; still blocks, not a separate language |
| `api-operation`, `playground` | Generator or playground features; do not pretend they are core syntax |

HTML class names such as `rd-docs-note` and `data-rocci-docs` are renderer
chrome. They can be renamed when painting moves; source spelling does not wait
on CSS.[^docs-rocci]

## Recommended source spelling

A block is `:name`, optional `[params]`, then content in one of three
scopes. The delimiter chooses the scope. Authors should not mix params and
nested document in one `{ }`.[^syntax-recommended][^syntax-variations][^exploration-brief]

| Scope | Form | When |
| --- | --- | --- |
| None | `:img[src: "...", alt: "..."]` | Props only |
| Line | `:h2 Installing` / `:note Don't do this.` | Body fits on the rest of the line |
| Section | `:note[title: "Watch"] {{ ... }}` | Nested Markdown, fences, or child blocks |
| End marker | `:tabs ... :end.tabs` | Long or brace-heavy bodies |

Line scope is the human default for one-line bodies, for the same reason
Markdown headings are `# Title` rather than `# {{ Title }}`. Section wrap is
for nested document. `:end.kind` is the fallback when `{{ }}`
would fight the body (named closer follows the article prefix, not `@end`).
Indentation is not a closer.[^syntax-variations][^bravo-ungram]

`# Heading` remains the usual heading. `:h2 Title` is the same AST node.
An extra line-scope marker is probably unnecessary: if the header has no
`{{` / `:end`, the rest of the line is content.[^syntax-recommended][^syntax-variations]

The decided page is `knowledge/research/syntax/syntax_v2_recommended.rocdown`.
Side-by-side wrapping lives in `syntax/syntax_v2_variations.rocdown`. The draft
AST ungram is `crates/rocci-rocdown/Rocdown.AST.ungram`.

## Bravo AST as inspiration

The Bravo ungrammar describes a document as `Block*`, with a line-based
primitive and a split between line content and delimited sections. That split
is the useful idea for Rocdown wrapping.[^bravo-ungram]

| Bravo node | Role | Rocdown analogue |
| --- | --- | --- |
| `Heading` / `ListItem` | prefix plus `LineContent` | line-scoped `:h2 Title` or Markdown `#` |
| `DecoratedLine` | decorator plus one line | `:note Title on this line` |
| `BlockSection` | delimited `Block*` | `{{ nested Rocdown }}` or `:end` body |
| `Fragment` / `FragmentSection` | inline pieces | Markdown inlines; not the first target |
| `Decorator` | call, template, or ref | block type resolution; later, inline calls |
| `BmlDefinitionBlock` | definition in the document | module `@roc` / imported Rocci, not article content |

Bravo also treats heading, list, quote, and decorated line as the same `Block`
layer rather than as a Markdown AST beside a separate directive AST. Rocdown's
`Item` + `MdNode` + `DocsNode` split is exactly that separation. A uniform
block layer would let sugar and explicit calls share renderers without making
inline Markdown a second language.[^bravo-ungram][^ast]

Inline `DecoratedFragment` is the MDX-like path. It conflicts with the
Markdown-first decision unless it stays opt-in and document-root / block-body
only. This research keeps inline decorations out of the first syntax.[^markdown-first][^format-report]

## Params versus content

Two different payloads currently share one `{ ... }`. They should not.

**Params** are typed values: strings, bools, records. They belong to the
component's props record. They must not be parsed as Rocdown.

**Content** is nested document: Markdown, nested blocks, and on interactive
pages possibly HTML islands. It becomes the extra `content` argument, as in
`|{ title }, content|`, not a string field named `content`.[^template-readme][^exploration-brief]

Today's `@docs note { title: "Watch"\n\n markdown }` is a fusion of both. The
maintainer sketches separate them:[^syntax-recommended][^exploration-brief]

```text
:note[title: "warning"] {{
    this is the content of the note
    <Button>with a button</Button>
}}
```

That maps onto existing Rocci calling convention: paren record for props,
wrapped body for Html. A params-only article block remains valid (`:img`,
`:badge`). A content-only form remains valid for blocks with no props. That
does not by itself decide `@page`.[^syntax-variations][^rocdown-readme]

`<Button>` inside content is not legal in shipped `@docs` bodies. Allowing it
means block content is a Rocdown fragment (`parse_fragment` already re-scans a
span) with a looser gate than today's static `@docs` rule. Static pages can
still forbid document-local `@component` / `@on` while allowing theme-registered
blocks and Markdown.[^parser][^docs-rs][^rocdown-readme]

## What fencing is

**Fencing** means a start marker and an end marker that bound a nested
document region. The parser does not guess the end from indentation or from
counting `{` / `}` inside Markdown. Code fences already work this way:
`` ``` `` opens, a matching `` ``` `` on its own line closes, and the inside
is opaque.[^rocdown-readme]

Article-block bodies need some form of fencing whenever the content can be
more than one line. Nested Rocdown may contain `{`, `}`, `@`, `::`, lists, and
other fences. A closer that is *not* those characters (or that is only special
at line start) is how the scanner finds the end without eating a `}` in a code
sample.[^lexer][^scanner]

Line scope is the exception: the newline *is* the fence. That is why headings
and one-line asides should not require a closer.

These are all fences, with different closer rules:

| Fence | Opens | Closes | Nested `}` in a code sample |
| --- | --- | --- | --- |
| Line | `:note ` | End of line | N/A (no nested lines) |
| Balanced `{{ }}` | `{{` | matching `}}` | Fragile unless fence-aware |
| Named closer | `:tabs` | `:end.tabs` at line start | Safe if scanning skips code fences |
| Markdown ` ``` ` | info line | same-length fence | Opaque (good) |

Prefix and fence are independent. `:` is the sigil. Nested document still
needs one of the fences above:

```text
:note[title: "Watch"] {{
    content of note
}}
```

Vue-family MDC also uses `::` as a *closer* (`::alert` … `::`). That pairing
is not a candidate here: it teaches a second fence, nested homogeneous `::`
is a stack, and Rocdown already has `{{ }}` / named `:end` / line-scope. A
named closer (`:end.tab`) is more verbose and more precise when several
blocks are open. `%` is not a closer either.[^syntax-variations][^syntax-recommended]

Fencing is not optional for nested document. The design choice is *which*
fence, not *whether* to fence. Two colon-fence spellings are out: Vue's
paired closer `::`, and triple-colon `:::note` (MyST, VitePress, Pandoc
fenced divs). `::` as a prefix is still on the table. `%…%` is not.

## Content wrapping alternatives

The scanner must find the end of nested document without treating Markdown,
code, and nested blocks as Roc.

| Delimiter | Strength | Cost |
| --- | --- | --- |
| Single `{ }` (current) | Familiar; matches `@page` / `@roc` / Rocci `@if` | Fence-unaware brace skip; field/content fusion; `}` in prose/code is a close |
| Double `{{ }}` | Visually distinct from params; rarer in Markdown | Still brace-based; `}}` in code can collide; not the Rocci `@if` body |
| `:end.kind` | Line-start, follows the article prefix | Verbose; needs matching kind |
| Vue `::name` … `::` | Line-start pair; familiar from MDC | Not used: prefix and fence would be the same token |
| `@end.kind` / `%…%` | Reuses `@` or a second punctuated fence | Not used: `@` is the language island; `%` is not a fence |
| Fence-aware `{ }` | Keeps one-brace look | More scanner complexity; still mixes params unless fields move out |
| Indentation | No closer token | Hostile to Markdown lists, fences, and copy-paste |

Decided default for **article blocks**: **params in `[]`**, **line content
when the body is one line**, **`{{ }}` or `:end.kind` for nested document**.
Prefix is `:`. Do not use indent as a closer. Do not keep today's mixed
`{ title: "...", markdown }` body. Do not close blocks with `::`, `:::`, or
`%`. Do not use `(params)`.[^syntax-recommended][^syntax-variations][^scanner]

That default is for block *calls* that may also have Rocdown content. It is
not a rule that every `@` form in a `.rocdown` file must use parens or
brackets. See [Delimiter classes](#delimiter-classes).

The variations page compares those forms on headings, asides, and tabs,
including a fence that contains `{` / `}`.[^syntax-variations]

`@if` at document root is a Rocci template splice whose body is HTML in `{ }`,
not Markdown. Double-brace document blocks would not match that. That
inconsistency is acceptable if control flow stays Rocci and article blocks stay
Rocdown. Line-scoped content is the special case that matters for humans, not
making `@if` Markdown.[^rocdown-readme][^exploration-brief]

## Delimiter classes

A `.rocdown` file already mixes languages. Forcing one wrapper for every `@`
form is how `@docs { title, markdown }` went wrong. Three classes are more
honest than one pretty delimiter:[^rocdown-readme][^template-readme]

| Class | Examples | Payload | Wrapper |
| --- | --- | --- | --- |
| Article block | `:note`, `:h2`, `:img`, `:tabs` | Optional params plus optional Rocdown | `[]` params; line / `{{ }}` / `:end` content |
| Module Roc | `@page`, `@roc`, `@css`, `@render` | A Roc record, module body, CSS, or expression | `{ }` as today, like Roc |
| Rocci in Rocdown | `@component`, `@if`, `@for`, `@on` | Rocci template grammar | Rocci's own `{ }` / `|params|` |

`@page(...)` would make article-block parens look consistent, but it is
uglier for a multi-field record, matches Roc record declarations less well
(`meta: { title: "..." }` already lives in braces), and still would not match
`@component Name = |{ name }|`. That remaining inconsistency with Rocci is
acceptable: `@component` is not an article block.[^rocdown-readme][^template-readme]

Leaning: **do not move `@page` / `@roc` / `@css` to parens or brackets.** `{ }`
means "this payload is Roc (or CSS)", not "this is a block call". `[]` means
"these are params to a block that may also have document content."

`:img` is the awkward middle: an article block that is often params-only.

```text
:img[src: "./x.png", alt: "X"]
:img[src: "./x.png", alt: "X", width: "50px"]
@page {
    route: "/guides/syntax-v2/",
    meta: { title: "Rocdown blocks" },
}
```

Leaning for images: keep them in the article-block class. The short form uses
brackets even without a Rocdown body. Do not use a brace record that looks
like `@page`.[^syntax-variations][^syntax-recommended]

## Parens versus brackets for article params

Article-block params need a wrapper that is not `{ }`, so they do not collide
with Roc records or with nested document. Two candidates:

```text
:note[title: "Watch"] {{ Nested Markdown. }}
```

**Brackets** `[title: "Watch"]` are the decided wrapper. They look like markup
attributes, not a Roc call. Cost: Markdown already uses `[text](url)` and
`[[wiki]]`. A line-start `:note[title: "Watch"]` is still distinct from a
paragraph link. Bare `[note]` would not be.

**Parens** `(title: "Watch")` were the other candidate. They look like
`note({ title }, content)` and would make `@page(...)` tempting. They are not
used.

Vue-family MDC puts props in *braces* on a `::` *fence*: `::alert{type="warning"}`.
Do not copy that form. `{ }` is already Roc in this file.

**Decision: `[]` for article params.** `@page` keeps `{ }`. Do not use
`{type="x"}` attribute sugar.[^syntax-recommended][^syntax-variations]

## Prefix versus structure

Module Roc and Rocci already share a single `@` and are consistent with each
other: `@page { }`, `@roc { }`, `@component Name = |params|`, `@if cond { }`.
Article blocks are the third class. They **do not use single `@`**. A distinct
prefix keeps notes out of the language-island namespace. `@docs` tried that
with a family name; a lighter sigil does the job without the leftover
word.[^rocdown-readme][^template-readme][^rocdown-reference][^syntax-variations]

The decided prefix is **`:`**. Nested document uses `{{ }}`, line-scope, or
`:end`. Not Vue's closer `::`, not `:::`, not `%`. `::` and `!` remain
historical alternatives below.

```text
:note Don't do this.
:note[title: "Watch"] {{ Nested Markdown. }}
```

| | `::note` | `!note` |
| --- | --- | --- |
| Distinct from `@page` / `@if` | Yes | Yes |
| Line-scope | `::note Don't.` | `!note Don't.` |
| Nested document | `::note(title: "Watch") {{ ... }}` | `!note(title: "Watch") {{ ... }}` |
| Familiar from | Vue-family MDC | Callouts; GFM `> [!NOTE]` |
| Main collision | Rare line-start `::` in prose | Markdown `![alt](url)` — `!ident` vs `![` |
| Closer | Not `::` | Not `%` |

Historical `::` / `!` prefixes are compared in
`syntax/syntax_v2_variations.rocdown`. The decided page is
`syntax/syntax_v2_recommended.rocdown`.[^syntax-variations][^syntax-recommended]

### Historical: `::`

Vue-family Markdown components (Nuxt Content MDC) spell the *kind* as
`::name`. Rocdown takes that sigil without Vue's closer.

```text
@page { route: "/guides/syntax-v2/" }
@component Hello = |{ name }|
    <p>Hello, {name}</p>

::note Do not paste raw HTML.

::note(title: "Watch") {{
    Nested Markdown, fences, child blocks.
}}

::tabs(group: "os") {{
    ::tab(id: "mac", label: "macOS") {{ Mac panel. }}
}}
```

Strengths: visually off `@`; familiar document-component spelling; wrapping
stays one system. Weaknesses: two characters; looks less like Rocci; line-start
`::` in ordinary Markdown is rare but not impossible.

### Historical: `!`

A single `!` before the kind. Prefix only: no `%` end fence, no `!!` pair.

```text
@page { route: "/guides/syntax-v2/" }
@component Hello = |{ name }|
    <p>Hello, {name}</p>

!note Do not paste raw HTML.

!note(title: "Watch") {{
    Nested Markdown, fences, child blocks.
}}

!tabs(group: "os") {{
    !tab(id: "mac", label: "macOS") {{ Mac panel. }}
}}
```

The scanner treats line-start `!` plus ident as a block. `![alt](url)` stays
Markdown because `[` is not an ident start. GFM alerts (`> [!NOTE]`) stay
blockquote syntax.

Strengths: one character; visually off `@`; line-scope is as short as
`# Title`. Weaknesses: `!` is loud on every aside; some readers will think of
images or shell history; less precedent as a *block* sigil than `::`.

### Also typeable (worth a look)

A prefix has to survive three tests: it looks like document structure, it is
easy to type on a US *and* a German/Nordic keyboard, and a line-start scanner
can take `punct+ident` without stealing Markdown. The rule is the same as for
`::` / `!`: line start, then the sigil, then an ident (`note`, `h2`, `tabs`).
No space between sigil and name. CommonMark lists and thematic breaks need a
space or a run of the same mark; that is the usual escape hatch.

| Form | Look | Type (US / DE-Nordic) | Parse risk |
| --- | --- | --- | --- |
| `::note` | Vue component | Shift `;` twice / Shift `.` twice | Rare `::` in prose |
| `!note` | Callout, loud | Shift `1` / Shift `1` | `![alt](url)` — ident vs `[` |
| `:note` | Label, lighter `::` | Shift `;` once / Shift `.` once | Definition lists use `: ` with a space |
| `.note` | CSS class / apply widget | Unshifted `.` / unshifted `.` | None in CommonMark |
| `/note` | Slash command | Unshifted `/` / Shift `7` | Unix paths; `/h2` looks like a route |
| `+note` | Additive | Shift `=` / often unshifted `+` | Lists are `+ ` with a space |
| `\note` | TeX macro | Unshifted `\` / AltGr | `\` is Markdown escape for punctuation, not letters |

**`:note`** is the obvious sibling of `::`. One colon does the same job. It
looks like a field label turned around (`:note` vs Roc `note:`). Pandoc-style
definition lists are `: definition` with a space after the colon, so `:note`
without a space is free. Cost: still shifted on every common layout; `:h2`
reads a bit like a goto label.

**`.note`** is the strongest *new* look. It is unshifted on US and European
layouts, it reads as "apply the Note class," and CommonMark does not use
line-start `.ident`. Cost: `.img` / `.h2` look a little like a filename or
extension; some SSGs already put `{.class}` on Markdown.

**`/note`** is the Discord/CLI spelling and is one unshifted key on US. On
German and Nordic layouts `/` is Shift-7, so it is worse than `.` for this
repo's likely authors. `/h2 Installing` also looks like a URL path next to
`@page { route: ... }`.

**`+note`** is cheerful where `!` is an alarm. Lists do not trigger (`+ note`
has a space; `+note` does not). Diff headers (`+++`) and "add this" semantics
are the only real noise.

**`\note`** is the LaTeX command: "this is a macro, not prose." CommonMark
only treats `\` as an escape before punctuation, so `\note` is currently
literal text and a scanner can claim it. Cost: AltGr on German keyboards;
the backslash already means "escape" in Markdown, which is the wrong
metaphor for a block you *want* to run.

None of these replace the decided `:note`. They remain in the record as
looks that were considered.[^syntax-variations]

### Other prefixes (not selected)

Kept for the record. Not the comparison to judge next.

Hard no, because the scanner or the eye already owns them:

| Form | Why not |
| --- | --- |
| `@note` | Same sigil as `@if` / `@page` |
| `@@note` | Doubled `@`; looks like a typo; Bravo decorator |
| `@docs note` / `@block note` | Rejected namespace word |
| `#note` | Looks like a broken ATX heading |
| `>note` | CommonMark blockquote (`>` needs no space) |
| `[note]` / `[NOTE]` | Link definitions `[note]: url`; AsciiDoc |
| `[[note]]` | Already a Rocdown wiki link |
| `<Note>` | Document-root HTML island |
| `*note` `_note` | Emphasis and lists |
| `` `note` `` | Code |
| `{note}` / `{{note}}` | Roc records and the content fence |
| `\|note` | GFM tables |
| `note:` | Ordinary English (`Note: don't`) becomes a block |
| `:::note` … `:::` | Extra colon; MyST/VitePress fence |
| Vue `::name` … `::` | `::` as closer |
| `%note` … `%` | `%` as fence; `%note` as prefix is just another punctuated sigil |
| Unicode (`§` `¶` `¡`) | Not typeable |

Quiet leftovers that are parsable but weak: `;note` (easy to miss), `~note`
(shifted; strikethrough `~~`), `=note` (Setext / wiki heading noise).

Do not use `<Note>` for article blocks. Document-root `<Tag>` is already a
Rocci island. Do not revive `@docs` as the separator.

### Decision

**Article blocks are lexically distinct from `@`.** Module and Rocci keep
today's single `@`. `@use` stays `@`.

**Prefix: `:note`.** Not `::`, not `!`, not `@`. Params: `[title: "Watch"]`.
Content: line-scope, `{{ }}`, or `:end.kind`.

Do not put `@page` in parens or brackets, and do not make `@component` look
like a note.[^syntax-variations][^syntax-recommended][^rocdown-ungram]

## Line scope

Some blocks are one line. Bravo encodes that as `Heading` / `DecoratedLine`
rather than as a section. Markdown already does this for ATX headings, and
that is good DX: the marker, then the rest of the line, then a new
block.[^bravo-ungram][^syntax-recommended]

```text
## Installing Rocci
:h2 Installing Rocci
:h2[id: "install"] Installing Rocci
:note Do not paste raw HTML into Markdown.
:caution[title: "Breaking"] This file is research-only.
```

A heading or short aside should not require `{{ }}`. Braces are for nested
document, not for putting a title in a box. Line scope ends at the newline.
There is no indented continuation; a second paragraph uses `{{ }}`.[^syntax-variations]

Markdown `#` / `##` remains the normal heading sugar and lowers to the same
node as `:h2`. Explicit heading blocks are for props (`id`) or for showing the
uniform block renderer.[^syntax-recommended][^article-rs]

## Dynamic block types and imports

Today the parser knows `@docs`, then Rust knows which kinds exist, then Rocci
matches those kinds again. Custom `@foo` cannot appear without a language
change.[^scanner][^docs-rs][^docs-rocci]

A registry inverts that:

1. Builtin / theme modules export Rocci components (`Note`, `Tabs`, `Heading`).
2. The Rocdown parser accepts `:ident` at document root (and inside block
   content) when the header matches a block shape. Today's `@docs note` is
   just `:note` in that registry.
3. Resolution maps the source name to a component (`note` / `link-card` →
   `Note` / `LinkCard`).
4. Unknown shaped `:foo` is a diagnostic, not silent Markdown, once the name is
   in the block namespace. Line-start `@word` without a reserved module/Rocci
   header stays Markdown.

Imports belong at Rocdown file level so documents can extend the registry
without opening every `@ident` as executable:

```text
@use "./Callout.rocci"
:callout[tone: "warn"] Don't copy this command.
```

Interactive `rocdown run` can also see colocated `@component` declarations in
the same file, which already lower to Roc functions. Static `rocdown build`
should keep the current gate: no per-page `@component` / `@roc` islands.
Custom static blocks therefore live in the compiled theme / renderer program,
the same place `DocsComponents.rocci` lives today, and are applied at build
time with page data. That matches the generation-pipeline rule that compilation
must not run per page.[^rocdown-readme][^generation-research][^catalog-shell]

Reuse ordinary Rocci `import` inside those theme modules for composition.
A dedicated `@use` on `.rocdown` files is for *which block types this document
may invoke*, not a second Roc module system. Qualified names
(`:Callout.warn` or `:blocks.note`) are the clash-avoidance valve if two
modules export `Note`.

Builtin kinds (`heading`, `paragraph`, `note`, `img`) are just the default
`@use`. They are not parser keywords beyond the small set of module
declarations and template splices that must remain reserved.

## Compatibility with existing decisions

These are architecture decisions, not a promise to keep current source:

Markdown-first stays: prose is Markdown; language mode changes at visible
block boundaries, not in the middle of a sentence.[^markdown-first]

Pure render stays: a block type is an `@component` from values and body Html to
Html. No hidden lifecycle on `:note`.[^pure-render]

Rust catalog / Rocci shell stays for discovery, routes, links, hashing, and
validation. What can move is article *painting*.[^catalog-shell][^generation-research][^compiler-arch]

OKF stays Markdown-only.

Source compatibility does not. `@docs`, unknown-`@name`-stays-Markdown, and the
mixed brace body may all change.

## Ungrammar as a grammar reference

Rocci has no generated grammar today; the shipped contract is README plus
hand-written AST types. Bravo shows ungrammar as a readable tree spec, not a
parser. The draft Rocdown ungram lives at
`crates/rocci-rocdown/Rocdown.AST.ungram`. Generate **AST node types** (and
later `format_ast` / syntax kinds) from it. Do **not** generate the
scanner.[^bravo-ungram][^rocdown-ungram][^ast]

The tree distinguishes module items (`@page`, `@roc`, `@use`, Rocci decls)
from article `BlockCall` nodes (`:note[title: "Watch"] {{ ... }}`). Markdown
`# Installing` and `:h2 Installing` are the same `BlockCall` (name `h2`).
Params are `BracketRecord` only. Content is `LineContent`, `BraceSection`
(`{{ }}`), or `EndSection` (`:end.kind`). Comrak still owns sugar
recognition; Rocdown still owns document-root scanning.

A second ungram for `rocci-template` is optional and separate. The shipped
`ast.rs` `Item` / `DocsDecl` tree is not this draft.

## Recommended next experiments

Delivery phases live in
[the implementation plan](../plans/generalized-rocdown-block-model.md).
That plan freezes v1 answers (closed registry, bracket param language,
Rust heading ids, no ungram scanner codegen) and sequences registry →
per-kind Rocci components → internal `BlockCall` → dual-parse → sugar →
cutover → typed props → `@use` → LSP.[^impl-plan]

Do not keep `@docs` as an alias. Do not ship `:foo` as an open executable
namespace or inline decorations in the first syntax.

## Open questions

- Prefix and params decided: `:note[title: "x"] {{ ... }}`. Remaining: which
  `@docs` kinds stay in the builtin registry vs theme-only / tooling-only.
- Are bracket props a Roc record literal subset, or a smaller param language?
- May block content on static pages include HTML islands that call
  theme-registered components, or only Markdown plus nested blocks?
- Should unknown shaped `:foo` be an error everywhere, or only when the
  document has at least one `@use` / builtin block invocation?
- Does a heading renderer own `id` generation, or does Rust still compute
  heading ids for links and the outline before render?
- If ungram becomes canonical, who fails CI when it drifts from `ast.rs`?

[^rocdown-readme]: Shipped file shape, reserved names, unknown-`@name` prose rule, `@docs` / `@if` bodies, and static versus deferred features.
[^ast]: `Item`, `DocsDecl`, and `MdNode` as separate article and module shapes.
[^scanner]: Document-root recognition, reserved-name table, `@docs` header requiring `{`, and brace-block skipping.
[^parser]: Fragment re-parse used for `@docs` bodies (`parse_fragment` / `scan_range`).
[^docs-rs]: Field/content split, typed kinds, nested `@docs`, illegal template items, and include region comments.
[^article-rs]: Rust Markdown HTML for headings and other sugar nodes.
[^lowerer]: Standalone `@docs` lowering to `Html.element` after the same field/content split.
[^docs-rocci]: Single `Render` matcher on `segment.kind` plus `|{ kind, title, ... }, body|` widgets.
[^template-readme]: Props record plus extra body parameter; paired tags pass nested Html.
[^lexer]: `skip_balanced_braces` skips strings and `#` comments, not Markdown fences.
[^format-arch]: Current format boundary and unimplemented Markdown extensions, including admonitions.
[^markdown-first]: Mode changes only at visible document-root declarations or root HTML islands.
[^pure-render]: `@component` is a pure function from explicit values and body to Html.
[^catalog-shell]: Rust owns catalog and deterministic data; Rocci owns visible chrome; article HTML is currently Rust.
[^generation-research]: Duplicate `@docs` wrappers and the rule that renderer compilation must not run per page.
[^compiler-arch]: Static article tree, fragment files, and Rocci painting from scalar segment records.
[^rocdown-reference]: Public `@docs` family contract: kinds are not top-level `@` names; mixed brace body.
[^format-report]: Original rationale for Markdown-first islands; not shipped syntax.
[^bravo-ungram]: Line versus section blocks, decorators, and a uniform `Block*` document.
[^syntax-recommended]: Decided sample: `:note`, `[params]`, line-scope, `{{ }}`.
[^syntax-variations]: Wrapping, `:end`, decided `:`, historical prefixes, rejected mixed `{ }`.
[^rocdown-ungram]: Draft document AST ungram; generate node types, not the scanner.
[^exploration-brief]: Maintainer framing: AST then Rocci renderers, params versus content, dynamic types, ungram.
[^impl-plan]: Phased delivery plan for the decided spelling and uniform article tree; exploratory, not shipped.
