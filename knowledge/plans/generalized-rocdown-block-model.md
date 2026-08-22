---
type: Implementation Plan
title: Generalized Rocdown block model
description: "Phased delivery of uniform article BlockCall nodes, :name[params] source spelling, a closed builtin registry, and per-kind Rocci renderers. Phases 1–7 shipped on main; phases 8–9 remain exploratory. Follow-on: custom block schemas and renderers."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/rendering, concern/architecture, concern/authoring]
status: draft
generated: { by: process:cursor, at: 2026-08-22T14:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model research
    author: process:cursor
    last_modified: 2026-08-19
  - id: renderer-plan
    resource: rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: syntax-recommended
    resource: ../research/syntax/syntax_v2_recommended.rocdown
    title: "Decided v2 spelling: : prefix, bracket params, {{ }} bodies"
    author: process:cursor
    last_modified: 2026-08-19
  - id: syntax-variations
    resource: ../research/syntax/syntax_v2_variations.rocdown
    title: "Wrapping, :end, decided : prefix, historical alternatives"
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-ungram
    resource: ../../crates/rocci-rocdown/Rocdown.AST.ungram
    title: Draft Rocdown document AST ungrammar
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: ast
    resource: ../../crates/rocci-rocdown/src/ast.rs
    title: Shipped Rocdown document AST
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
    title: Typed article-block projection, validation, and PlannedSegment
    author: process:git
    last_modified: 2026-08-18
  - id: markdown-rs
    resource: ../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak conversion and heading-id assignment
    author: process:git
    last_modified: 2026-08-17
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
  - id: planner
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Static site planner and PlannedSegment Roc emission
    author: process:git
    last_modified: 2026-08-19
  - id: pprint
    resource: ../../crates/rocci-rocdown/src/pprint.rs
    title: format_ast inspect contract
    author: process:git
    last_modified: 2026-08-17
  - id: lsp
    resource: ../../crates/rocci-rocdown/src/lsp.rs
    title: Rocdown LSP symbols, hover, and completions
    author: process:git
    last_modified: 2026-08-17
  - id: highlight
    resource: ../../crates/rocci-rocdown/src/highlight.rs
    title: Rocdown token spans
    author: process:git
    last_modified: 2026-08-17
  - id: docs-rocci
    resource: ../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Rocci documentation widgets and Render matcher
    author: process:git
    last_modified: 2026-08-17
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-17
  - id: lexer
    resource: ../../crates/rocci-template/src/lexer.rs
    title: Balanced-brace skipping and tag-name scanning
    author: process:git
    last_modified: 2026-08-17
  - id: compile-tests
    resource: ../../crates/rocci-rocdown/tests/compile.rs
    title: Rocdown compiler contract tests
    author: process:git
    last_modified: 2026-08-18
  - id: all-syntax
    resource: ../../test/AllSyntax.rocdown
    title: Rocdown syntax fixture
    author: process:git
    last_modified: 2026-08-17
  - id: rocdown-reference
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: docs-guide
    resource: ../../docs/guides/docs-components.rocdown
    title: Public documentation-component guide
    author: process:git
    last_modified: 2026-08-18
  - id: format-arch
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: compiler-arch
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
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
  - id: generation-plan
    resource: rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-18
  - id: rocci-author
    resource: ../../.agents/skills/rocci-author/SKILL.md
    title: Rocci and Rocdown authoring skill
    author: process:git
    last_modified: 2026-08-18
  - id: ungram-plan
    resource: ungram-ast.md
    title: Ungrammar AST codegen implementation plan
    author: process:cursor
    last_modified: 2026-08-19
---

# Generalized Rocdown block model

## Purpose and authority

This is the implementation plan for the [generalized Rocdown block model
research](/research/generalized-rocdown-block-model.md). Phases 1–7 below
shipped on `main` (the stale
`generalized-rocdown-block-model-implementation` branch was deleted
2026-08-22). Phases 8–9 and follow-ons remain exploratory until a human
reviewer accepts a scope. Architecture records and crate READMEs remain the
current contract for shipped behavior.[^research][^rocdown-readme][^format-arch]

The `:name[params]` spelling and closed registry landed on `main`. Renderer
override, generic child policy, and site `[blocks]` config are a follow-on:
[custom block schemas and renderers](rocdown-block-renderers.md).[^renderer-plan] Do not
start that work from this record. Current article widgets are `:kind` only.
Do not revive the removed experimental family or use it as a design analogy.

Do not start an exploratory phase until the user asks to implement it. Use
the `rocci-language-dev` skill for grammar, scanning, parsing, AST, validation,
lowering, diagnostics, and the public language reference. Use `rocci-author`
only after the syntax exists, when rewriting `.rocdown` pages.[^language-dev][^rocci-author]

## Goal

Ship one article-block representation and the decided source spelling so that
Markdown sugar and explicit calls share renderers:[^research][^syntax-recommended][^rocdown-ungram]

```text
:note Don't do this.
:h2[id: "install"] Installing
:note[title: "Watch"] {{ nested Rocdown }}
:tabs.begin[group: "os"] ...
:tabs.end
```

Each article node has a kind, optional params, and optional content. A Rocci
component of shape `|{ props }, content| -> Html` paints it. Kinds are a
closed builtin registry in v1, not parser keywords. The removed
experimental family and mixed `{ fields + markdown }` go away with no alias
and no rewrite window.[^research][^pure-render][^docs-rs]

## Constraints that do not move

These are existing decisions, not a promise to keep current source
spelling:[^markdown-first][^pure-render][^catalog-shell]

| Keep | Meaning for this plan |
| --- | --- |
| Markdown-first islands | Mode changes at line-start block boundaries, not mid-sentence |
| Pure `@component` render | No hidden lifecycle on `:note` |
| Rust catalog / Rocci shell | Catalog, routes, links, heading ids, and include/example execution stay in Rust; painting may move |
| OKF Markdown-only | Do not put `:note` or `@use` in `knowledge/**/*.md` |
| `@page` / `@roc` / `@css` keep `{ }` | Article params are `[]`; Roc payloads stay braces |
| `@if` stays Rocci HTML | Do not make control-flow bodies Markdown in this plan |
| `rocci-template` grammar | Change it only if Rocci syntax must change; article `{{ }}` skippers live in `rocci-rocdown` |

The [chrome-library plan](/plans/rocci-component-generation.md) owns outline,
nav, breadcrumbs, and Roc hosts. This plan owns article-block kinds and
`DocsComponents.rocci`. Do not extract those widgets into `rocci-ui` without a
second consumer.[^generation-plan][^docs-rocci]

## Non-goals (all phases)

- Family-name aliases, dual public syntax, or a rewrite window
- Inline decorations (`:note` mid-paragraph)
- Vue `::name` … `::`, `:::note`, `%…%`, indentation as a closer, or `(params)`
- Moving `@page` / `@component` to brackets
- Open `:foo` as an executable namespace
- Per-page `@component` / `@roc` on static `rocdown build`
- Generating the **scanner** from the ungram
- A general ungram compiler in v1
- Markdown-conditional blocks (`:if` over Rocdown)
- Changing OKF authoring or the portable `okf` crate

A **removal diagnostic** for leftover experimental tokens is allowed.
Parsing the old mixed brace body is not. See
[Rocdown format](/architecture/rocdown-format.md).

## V1 contract

These answers freeze the research open questions for delivery. Changing one
is a plan revision, not a silent phase tweak.[^research]

### Source spelling

Prefix `:`. Params in `[]`. Content is line-scope, `{{ }}`, or `:kind.begin` ... `:kind.end`. A call uses one delimiter, not both.
No space between `:` and the kind. Kinds are kebab-case tag names (same
`scan_tag_name` rules as today's builtin kinds).[^syntax-recommended][^lexer]

`@use` stays `@`. Module reserved names keep winning on `@`. A line-start
`:page` is not a page declaration; it is an illegal article kind that
collides with a module name.

### Builtin registry

v1 is closed. Unknown shaped `:foo` is always a diagnostic, including in
documents with no `@use`.

| Role | Kinds |
| --- | --- |
| Asides | `note`, `tip`, `caution`, `danger`, `deprecated` |
| Structure | `details`, `steps`, `step`, `figure`, `definition`, `tabs`, `tab`, `badge` |
| Site chrome | `link-card`, `card-grid`, `file-tree`, `compatibility` |
| Tooling (stay article blocks in v1) | `include`, `example` |
| Sugar kinds | `h1`–`h6`, `img` |
| Not authorable | `api-operation` (keep the generator-reserved error), `playground` |

Do not reclassify `include` / `example` as `@include` / `@example` in v1.
That is a follow-on: it would retarget catalog edges and `rocdown test`
without helping the uniform render protocol.[^docs-rs][^rocdown-readme]

### Param language

Bracket params are **not** a Roc record literal. They are the ungram
`ParamValue` set plus one amendment:

- `StringLit`, `BoolLit` (`Bool.true` / `Bool.false`, matching today's field
  parser), `NumberLit`, `Ident` (unquoted atoms such as `platform`)
- Nested `BracketRecord`
- **Amendment:** `BracketList` of those values, so `test: ["unit"]` keeps
  working. After `[`, if the next token is `ident` then `:`, parse a record;
  otherwise parse a list.

No interpolation, no Roc calls, no `#` comments inside `[]` in v1. Newlines
and commas are both field separators. Update `Rocdown.AST.ungram` in the
phase that parses params, not as a drive-by.[^rocdown-ungram][^docs-rs]

### Content on static pages

Block bodies are Markdown plus nested article blocks. Static `rocdown build`
still forbids document-local `@component` / `@on` / `@roc` inside those
bodies. Theme-registered kinds are enough. HTML islands inside `{{ }}` wait
for a follow-on.[^parser][^rocdown-readme]

### Heading ids

Rust still slugifies heading ids for the catalog, outline, and in-page
links, using the existing `markdown.rs` assignment. The heading `BlockCall`
receives `id` as a param (`:h2[id: "install"]` wins; sugar `#` uses the
generated id). Rocci does not own id generation in v1.[^markdown-rs][^compiler-arch]

### Ungram role

`Rocdown.AST.ungram` is the AST spec. Hand-write matching Rust types. Do not
generate the scanner. A later follow-on may generate node types; v1 only
needs a drift test that production names exist on the Rust AST.[^rocdown-ungram][^ast]

## Layer map

| Concern | Owner | Notes |
| --- | --- | --- |
| Line-start `:ident`, `[params]`, `{{ }}`, `:kind.begin` / `:kind.end` | `crates/rocci-rocdown/src/scan.rs` | Reuse existing `fence_open` / `is_fence_close`; do not call `skip_balanced_braces` for article bodies |
| Parse `BlockCall`, fragment re-entry | `parse.rs` | `parse_fragment` already re-scans a span |
| `Item` vs article `Block` | `ast.rs` | Module items stay `@`; article nodes become `Block` / `BlockCall` |
| Kind schema, parent/child rules | new `registry.rs` (or module in `docs.rs`) | Data, not scattered `match` arms |
| Include/example execution, link-card fill | `docs.rs`, `site.rs` | Stay Rust |
| Heading ids, Comrak sugar | `markdown.rs` | Comrak still recognizes `#`, lists, fences |
| Markdown fragment HTML | `article.rs` | Stay Rust in v1 |
| Widget HTML | `templates/DocsComponents.rocci` | Per-kind components |
| Static apply data | `plan.rs` | Replace the flattened `PlannedSegment` bag after components exist |
| Standalone preview | `lower.rs` | Conservative HTML; same registry kinds |
| Inspect | `pprint.rs` | `format_ast` |
| Editor | `lsp.rs`, `highlight.rs`, `rocci-rocdown-lsp` | Must not crash at cutover |
| Public contract | crate README, `docs/reference/rocdown.rocdown`, `docs/guides/docs-components.rocdown` | Same change as behavior |
| Knowledge architecture / status | `knowledge/architecture`, `knowledge/status` | Update **after** a phase ships, not in this plan commit |

`rocci-template`'s `skip_balanced_braces` remains Roc/Rocci `{ }` skipping
(strings and `#` comments, not Markdown fences). Article content must not
use it.[^lexer][^scanner]

## Historical starting state (what phases replaced)

Before the cutover a `.rocdown` file was `Item` = Markdown + module decls +
experimental article widgets + native image decls. The experimental family
used a mixed brace body. `split_docs_body` peeled `ident:` fields until the
remainder looked like Markdown. Kinds were hardcoded in `validate_model`,
again in `DocsAttrs` / `PlannedSegment`, and again in
`DocsComponents.Render`'s `@match segment.kind`. Brace skip did not
understand fences.[^ast][^docs-rs][^docs-rocci][^scanner][^compiler-arch]

The interesting replacement is: parse a document tree, render each
`BlockCall` by calling a registry component with real props plus already
rendered content. Do that in slices so no phase rewrites scanner, AST,
catalog, Roc emission, theme, LSP, and `docs/` in one diff.

## Delivery phases

Each phase is one mergeable change. Later phases may assume earlier ones
have merged. Dual-parse was internal only; the public cutover is Phase 6.
Phases 1–7 are a delivery diary of the `:kind` cutover on `main`. Phases
8–9 remain open. They mention the removed experimental family only as the
starting state they replaced. Do not resume that spelling. Current source
is `:kind`.

### Phase 1 — Closed builtin registry in Rust

**Bound:** extract kind schema from `validate_model` / `DocsAttrs` into an
explicit registry. No source-syntax change. No Rocci change.

**Does:**

- Add `crates/rocci-rocdown/src/registry.rs` (name flexible) with, for each
  v1 kind: source name, component name, required/optional fields, parent
  kinds, child kinds, and whether it is authorable.
- Drive `unknown kind`, parent/child, and required-field diagnostics from
  that table. Keep message text stable where tests snapshot it, except
  where the table makes an existing inconsistency obvious.
- Unit-test the table: every kind `DocsComponents.Render` matches has a
  registry row; `api-operation` stays reserved; unknown `widget` still
  errors.

**Does not:** parse `:note`, split `DocsComponents`, change `PlannedSegment`,
or rewrite fixtures.

**Exit:** `cargo test -p rocci-rocdown` green. Adding a kind is a registry
row plus (still later) a component, not a new parser keyword.

### Phase 2 — Per-kind Rocci components

**Bound:** replace the closed `@match segment.kind` painter with one
component per kind. Authors still used the then-current experimental
spelling.[^docs-rocci][^planner]

**Does:**

- Split `DocsAside` / `Render` into `Note`, `Tip`, `Tabs`, `Tab`, `Figure`,
  and the rest, each `|{ … }, content|` (or props-only for `Badge` /
  `Img`-like widgets).
- Keep CSS in the same module for v1; class names may stay `rd-docs-*`
  until a later paint pass.
- Change generated Roc so the theme calls those components. A thin
  dispatcher is acceptable only as a temporary adapter behind the new
  components, not as the long-term contract.
- Assert through existing `tests/generator.rs` and `rocdown build docs`
  structure, not byte-identical class strings if a rename is required.

**Does not:** change `.rocdown` spelling, heading HTML (still Rust), or move
files into `rocci-ui`.

**Exit:** static docs widgets render without a single kind matcher as the
source of truth. Catalog checks still do not require Roc.

### Phase 3 — Internal `BlockCall` tree

**Bound:** introduce the ungram article nodes beside today's `Item`, and
normalize experimental article decls into `BlockCall` after parse. Authors
still used the then-current experimental spelling.[^rocdown-ungram][^ast][^docs-rs]

**Does:**

- Hand-write `BlockCall`, `BracketRecord`, `BlockContent`, and related
  types to match the ungram (plus `BracketList`).
- After parse, map `Item::Docs` / `Item::Img` into `BlockCall`. Content
  children stay the current fragment tree. Continue using `split_docs_body`
  as the **legacy** param extractor for `{ }` bodies only.
- Point `load_page_docs`, `collect_headings` (docs nodes only),
  `plan_segments`, and standalone docs lowering at `BlockCall` instead of
  `DocsDecl` / `ArticleNode::Docs` where that is a local rename.[^article-rs][^lowerer]
- Keep `MdNode` for Markdown sugar (headings stay `MdNode::Heading` until
  Phase 5).
- Add a drift test: ungram production names used for article calls exist as
  Rust types / `format_ast` tags.
- Extend `format_ast` enough that inspect shows `block note` rather than
  only `docs note`.[^pprint]

**Does not:** scan `:ident`, drop `Item::Docs`, unify headings, or generate
code from the ungram.

**Exit:** existing experimental-family fixtures produce `BlockCall` internally with
unchanged catalog diagnostics and site HTML. `inspect ast` remains useful.

### Phase 4 — New syntax, dual-parse

**Bound:** scan and parse the decided spelling; keep the experimental
family working. This is the first language change.[^syntax-recommended][^scanner][^parser][^language-dev]

**Does:**

- Recognize line-start `:ident` with the same document-root / fragment-root
  rules as `@` (not inside lists, quotes, or fences). Reuse 0–3 space
  indent stripping already used for fences.
- Header shapes:
  - params-only: `:img[src: "...", alt: "..."]`
  - line content: rest of line after optional `[params]`; newline is the
    fence
  - section: `{{` … fence-aware `}}`
  - named closer: body until line-start `:kind.end` (kind must match;
    opener is `:kind.begin`)
    nested same-kind uses a stack)
- `:` plus ident is **not** a block when a space follows the colon
  (`: definition` stays Markdown).
- `:end` is a closer, never a `BlockCall` named `end`. Bare `:end` without
  `.kind` is an error.
- Implement `skip_article_section` in `rocci-rocdown` that:
  - treats Markdown `` ``` `` / `~~~` as opaque (existing `fence_open` /
    `is_fence_close`)
  - counts nested `{{` / `}}` outside fences
  - does not treat a single `}` in prose as a close
  - always advances (`cur.pos > before` or `cur.bump()`) on malformed and
    unclosed input
- Parse `[params]` into `BracketRecord` / `BracketList`. Do not run
  `split_docs_body` on new syntax.
- Re-enter `parse_fragment` / `scan_range` for `{{ }}` and `:end` bodies so
  nested `:note` works.
- Map parsed `BlockCall` through the Phase 1 registry. Unknown kind, bad
  parent/child, and missing fields are diagnostics at the kind span.
- Reserved module-name collision (`:page`, `:roc`, `:component`, …) is a
  dedicated error.
- Tests at the parser boundary (no server, no Roc):
  - copy or slim `syntax_v2_recommended.rocdown` into `test/`
  - stray `{` / `}` inside a fenced body
  - nested tabs with `:tabs.end`
  - unclosed `{{`, mismatched `:end.foo`, `:end` inside a fence
  - line-scope versus following paragraph
  - `: note` and mid-paragraph `:note` remain Markdown
  - the experimental family still parses

**Does not:** delete the experimental family, rewrite `docs/`, change LSP beyond not
crashing on `:note` holes, or open `@use`.

**Exit:**

```text
cargo test -p rocci-rocdown
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -q -p rocci-rocdown-cli -- inspect ast test/<v2-fixture>.rocdown
```

Both old and new fixtures parse. Fence-unaware `skip_balanced_braces` is
not used for article bodies.

### Phase 5 — Heading and image sugar

**Bound:** `# Heading` / `## Heading` and block-level Markdown images lower
to the same `BlockCall` kinds as `:h2` and `:img`. Catalog heading ids stay
Rust-owned.[^markdown-rs][^rocdown-ungram][^syntax-variations]

**Does:**

- Map ATX headings to `BlockCall` name `h1`–`h6` with `id` from existing
  `assign_heading_id`. Explicit `:h2[id: "install"]` uses the authored id
  and still participates in the outline.
- Map block-level `![alt](src)` to `img` `BlockCall`. Keep **inline**
  images as Markdown inlines.
- Walk `BlockCall` headings in `collect_headings` / link checking.
  Nested headings inside `:tab` keep today's "not in outline" or documented
  successor behavior; do not silently change outline policy.
- `:img[...]` uses the same field contract as `@img` (`src` required; `alt`
  or `decorative`). Figure bodies still require exactly one image.

**Does not:** drop `@img` yet, render headings through Rocci, or invent a
second slug algorithm.

**Exit:** in-page `#install` links and outline entries still resolve.
`inspect ast` shows sugar headings as `block h2`. Parser tests cover
`:h2[id]` versus `#`.

### Phase 6 — Public syntax cutover

**Bound:** the breaking language change. One phase so the repo does not
publish two spellings. No alias.[^research][^rocdown-reference][^docs-guide]

**Does:**

- Remove `docs` and `img` from `Reserved`. Stop parsing mixed `{ fields +
  markdown }` bodies.
- If line-start leftover experimental tokens are seen, emit a removal
  error that names the new spelling (`:note`, `:img[...]`). Do not parse
  the old body. The exact leftover tokens are documented in
  [Rocdown format](/architecture/rocdown-format.md).
- Rewrite in-repo documents that use the old forms, including
  `docs/guides/docs-components.rocdown`, `docs/reference/rocdown.rocdown`,
  `test/AllSyntax.rocdown`, `test/EmbeddedLanguages.rocdown`, site pages,
  examples, and report fixtures. Leave `knowledge/**/*.md` inert.
- Update `crates/rocci-rocdown/README.md`, highlight keywords, LSP hover /
  symbols / completions so they speak `:kind` not the experimental family, and
  diagnostic strings in `docs.rs` / `lower.rs` / `site.rs`.[^lsp][^highlight][^lowerer][^all-syntax]
- Update `.agents/skills/rocci-author` idioms for the new spelling.
- Mark planned leftover mentions in historical reports as historical; do
  not rewrite `archive/`.

**Does not:** implement `@use`, HTML islands in block bodies, or CSS class
renames as a required sibling (optional if cheap).

**Exit:**

```text
cargo test -p rocci-rocdown
cargo test -p rocci-rocdown-cli
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

No in-tree authoring `.rocdown` still uses the experimental family. Public
reference describes `:name[params]`. Then update
`knowledge/architecture/rocdown-format.md` and status records in a
knowledge follow-up so architecture stays descriptive of shipped
behavior.[^format-arch]

### Phase 7 — Typed props into Rocci; drop the segment bag

**Bound:** stop flattening every widget through `PlannedSegment`'s parallel
optional fields. Pass the component's props plus rendered `content`.[^planner][^docs-rocci][^template-readme]

**Does:**

- Generate per-kind records (or a small typed union) from the registry
  instead of one struct with `title`, `summary`, `tab_id`, …
- Content argument is already-rendered HTML fragments, same calling
  convention as paired Rocci tags. No magic `children` field.
- Standalone `lower.rs` conservative preview uses the same kind set; it
  may still emit `Html.element` rather than compiling the theme.
- Keep Markdown paragraph/list/table HTML in Rust. Heading painting may
  stay Rust in this phase; a `Heading` Rocci component is optional if Phase
  2 already introduced one.

**Does not:** compile a renderer per page, interpret `.rocci` in Rust, or
encode prose as Roc constructors.

**Exit:** `plan.rs` no longer requires a new `PlannedSegment` field to add
a widget prop. `pages_roc_is_stable_for_docs_body_only_edits` (or
successor) still hashes independently of body text where that invariant
remains intended. `cargo test -p rocci-rocdown` plus a docs build.

### Phase 8 — `@use` for interactive documents

**Bound:** extend the registry from an imported `.rocci` module in
`rocdown run`. Static builds stay closed.[^research][^rocdown-readme][^scanner]

**Does:**

- Add `UseDecl` to scanning/parsing: `@use "./Callout.rocci"` (path string,
  same document-root rules as other `@` decls). Payload is not `{ }`.
- Interactive `rocdown run`: load the module, map exported `@component`
  names to article kinds (`Callout` → `:callout` unless an explicit export
  table is added). Unknown `:callout` without `@use` remains an error.
- Static `rocdown build` / `check`: `@use` is an error (or ignored-with-
  error) that says custom static blocks belong in the compiled theme.
- Qualified names (`:blocks.note`) wait until two modules actually clash.

**Does not:** treat every `@ident` as executable, allow per-page
`@component` on static sites, or add inline calls.

**Exit:** the `@use` example in `syntax_v2_recommended.rocdown` runs under
`rocdown run`. `rocdown check docs` rejects `@use`. Parser tests cover
missing files and non-component exports.

### Phase 9 — Editor and inspect completeness

**Bound:** make the new syntax pleasant in inspect and LSP after cutover.
Phase 6 already prevents crashes and keyword highlighting.

**Does:**

- Completions for builtin kinds and, inside `[ ]`, registry field names.
- Hover and document symbols on `:tabs` / `:tab` (replace leftover
  experimental-family strings).
- `:kind.end` matching highlights / jump if cheap.
- `format_ast` shows params and which content scope was used (`line`,
  `section`, `end`).
- `test/AllSyntax.rocdown` covers line-scope, `[params]`, `{{ }}`,
  `:kind.begin` / `:kind.end`, and `:img`.

**Does not:** Tree-sitter grammar, inline decorations, or a second LSP
architecture.

**Exit:** `cargo test -p rocci-rocdown --test lsp` (and highlight tests)
green. `inspect ast` on AllSyntax is the debugging contract for later
syntax bugs.

## Suggested merge order

Phases 1 → 2 can overlap in two PRs (registry vs theme) if they land
registry first. Phase 3 before 4. Phase 5 can land just before 6 if sugar
tests would otherwise duplicate heading collection. Phase 7 can start after
2 and 3 even before cutover, but merging it after 6 avoids generating two
spelling paths in `plan.rs`. Phase 8 is last among language features.
Phase 9 can trail Phase 6 closely; do not wait on `@use`.

## Validation

While iterating, run the owning crate only:[^language-dev][^compile-tests]

```text
cargo test -p rocci-rocdown
cargo fmt --all -- --check
```

After syntax or public-contract changes:

```text
cargo run -q -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

After knowledge edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Cross-cutting (cutover, planner, theme): `cargo test --workspace`. Do not
set `ROCCI_REQUIRE_ROC=1` unless the phase explicitly needs Roc apply.
Scanner loops must stay terminating on unclosed `{{` and `:end`.

Do not log a phase complete in `knowledge/log.md` until the required GitHub
workflows (CI and Knowledge) have succeeded on that revision.

## Follow-ons (not v1)

- Reclassify `include` / `example` as `@include` / `@example` if the
  tooling-versus-document split is still wanted after the prefix drop.
- HTML islands / theme component calls inside static block bodies.
- Per-page `@component` / `@roc` on static `rocdown build` (owned by the
  [hybrid Rocdown islands plan](hybrid-rocdown-islands.md), not this
  spelling plan).
- Generate AST types (and later syntax kinds) from the ungram; CI fails on
  drift. Owned by the [ungram AST plan](ungram-ast.md), which also covers
  a `Rocci.AST.ungram`.[^ungram-plan]
- Rename `rd-docs-*` / `data-rocci-docs` once painting is fully component-
  owned.
- `Heading` Rocci renderer if Rust heading HTML should move.
- Qualified `@use` names; static theme-registered custom kinds beyond
  builtins.
- Inline decorated fragments (explicitly out of the first syntax).

## Open questions that would still change the plan

1. Should static sites ever honor `@use`, or is the theme always the only
   custom-kind source?
2. Is a `BracketList` versus `BracketRecord` lookahead (`ident` then `:`)
   enough, or do lists need a different wrapper?
3. Should leftover experimental tokens be a hard error (this plan) or silent Markdown
   under the existing unknown-`@name` rule? Hard error is the
   recommendation so old files do not render as prose.
4. Do tab-internal headings belong in the page outline? Preserve current
   behavior until product says otherwise.

[^research]: Exploratory decisions: `:name[params]`, no family-name alias, registry kinds, ungram for nodes not scanner.
[^syntax-recommended]: Decided samples for line-scope, brackets, `{{ }}`, `@use`.
[^syntax-variations]: `:kind.begin` / `:kind.end`, rejected mixed `{ }`, historical prefixes.
[^rocdown-ungram]: Draft `BlockCall` / `BracketRecord` / `LineContent` / `BraceSection` / `EndSection`.
[^rocdown-readme]: Shipped reserved names, unknown-`@name` prose, static versus deferred features, `:kind` article blocks.
[^ast]: Current `Item` sum: Markdown, module decls, `DocsDecl`, `ImgDecl`.
[^scanner]: Document-root `@` / `:` recognition, `skip_balanced_braces`, existing fence helpers.
[^parser]: `parse_fragment` / `scan_range` for nested docs bodies.
[^docs-rs]: `split_docs_body`, `DocsAttrs`, `validate_model`, `PlannedSegment`, include/example execution.
[^markdown-rs]: Comrak conversion and `assign_heading_id` / `slugify`.
[^article-rs]: Rust Markdown HTML for sugar nodes.
[^lowerer]: Standalone article-block lowering after the field/content split.
[^planner]: Flattened segment records emitted into generated Roc.
[^pprint]: `format_ast` inspect tags for `docs` / `img` / Markdown.
[^lsp]: Hover, symbols, and completions keyed on article `:kind`.
[^highlight]: Token spans for article `:kind`.
[^docs-rocci]: Single `Render` matcher on `segment.kind`.
[^template-readme]: Props record plus extra body parameter; no magic `children`.
[^lexer]: `skip_balanced_braces` skips strings and `#` comments, not Markdown fences; `scan_tag_name` allows kebab-case.
[^compile-tests]: Compiler contract tests independent of a server.
[^all-syntax]: Canonical inspect fixture for syntax changes.
[^rocdown-reference]: Public `:kind` article-block contract.
[^docs-guide]: Author-facing `:kind` examples in `docs/`.
[^format-arch]: Descriptive format boundary; update after behavior ships.
[^compiler-arch]: Static article tree, fragments, and Rocci painting from scalar segments.
[^markdown-first]: Mode changes only at visible document-root declarations or root HTML islands.
[^pure-render]: `@component` is a pure function from values and body to Html.
[^catalog-shell]: Rust owns catalog data; Rocci owns visible chrome.
[^generation-plan]: Shared chrome library vs Rocdown-owned `DocsComponents`.
[^language-dev]: Grammar/parser/lowering workflow, monotonic scanners, AllSyntax inspect.
[^rocci-author]: Authoring skill; wrong until the syntax exists, then used to rewrite pages.
[^ungram-plan]: Shared generator of owned AST structs from Rocci and Rocdown ungrams; scanners stay hand-written.
[^renderer-plan]: Follow-on plan for schema/renderer split and site pack overrides.
