---
type: Implementation Plan
title: Rocdown Markdown @{expr} interpolation
description: "Implement mid-sentence Roc Str holes in Rocdown as `@{expr}`: Comrak Text split, source-aware `\\@{` escape, hydrate promotion, Html.text lowering. Do not add `{@expr}`, `{{expr}}`, MDX, or a Rust evaluator."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/authoring, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-23T17:20:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/rocdown-inline-interpolation.md
    title: Inline interpolation in Rocdown Markdown
    author: process:cursor
    last_modified: 2026-08-22
  - id: markdown-first
    resource: ../decisions/markdown-first-explicit-islands.md
    title: Keep Rocdown Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: format-arch
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-20
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: scanner
    resource: ../../crates/rocci-rocdown/src/scan.rs
    title: Document-root scanner and line-start \\@ skip
    author: process:git
    last_modified: 2026-08-22
  - id: markdown-rs
    resource: ../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak to MdNode conversion and heading slugs
    author: process:git
    last_modified: 2026-08-21
  - id: parse-rs
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: parse, parse_fragment, parse_markdown_body, nested_items
    author: process:git
    last_modified: 2026-08-22
  - id: article-rs
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Static/hydrate/live classification and Rust article HTML
    author: process:git
    last_modified: 2026-08-22
  - id: lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Rocdown to Roc lowerer
    author: process:git
    last_modified: 2026-08-22
  - id: docs-rs
    resource: ../../crates/rocci-rocdown/src/docs.rs
    title: Typed article forest for static widgets
    author: process:git
    last_modified: 2026-08-22
  - id: pprint
    resource: ../../crates/rocci-rocdown/src/pprint.rs
    title: Rocdown inspect printer
    author: process:git
    last_modified: 2026-08-22
  - id: highlight
    resource: ../../crates/rocci-rocdown/src/highlight.rs
    title: Rocdown highlight collector
    author: process:git
    last_modified: 2026-08-22
  - id: md-ungram
    resource: ../../crates/rocci-rocdown/Rocdown.Markdown.ungram
    title: Markdown projection AST
    author: process:git
    last_modified: 2026-08-19
  - id: md-toml
    resource: ../../crates/rocci-rocdown/Rocdown.Markdown.toml
    title: Markdown ungram sidecar and inspect tags
    author: process:git
    last_modified: 2026-08-19
  - id: rocci-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Rocci interpolation scan
    author: process:git
    last_modified: 2026-08-22
  - id: rocci-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Interpolation lowering to Html.text
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci {expr} contract
    author: process:git
    last_modified: 2026-08-22
  - id: text-ref
    resource: ../../docs/reference/language/text.rocdown
    title: Rocci text and interpolation reference
    author: process:git
    last_modified: 2026-08-21
  - id: lang-ref
    resource: ../../docs/rocdown/language.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: all-syntax
    resource: ../../test/AllSyntax.rocdown
    title: Comprehensive Rocdown syntax fixture
    author: process:git
    last_modified: 2026-08-22
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-22
  - id: compile-tests
    resource: ../../crates/rocci-rocdown/tests/compile.rs
    title: Rocdown compiler contract tests
    author: process:git
    last_modified: 2026-08-22
  - id: follow-ons
    resource: rocdown-inline-interpolation-follow-ons.md
    title: Rocdown @{expr} follow-ons after v1
    author: process:cursor
    last_modified: 2026-08-23
---

# Rocdown Markdown `@{expr}` interpolation

## Purpose and authority

This is the implementation plan for the settled Markdown hole
`@{expr}`. Research and alternatives live in [inline interpolation in
Rocdown Markdown](../research/rocdown-inline-interpolation.md). This
record does not describe shipped behavior. Crate READMEs remain the
language contract until a phase lands.[^research][^rocdown-readme]

Do not start a phase until the user asks. Use `rocci-language-dev` for
ungram, parser, lowering, and AllSyntax work. Branch name:
`rocdown-inline-interpolation`.[^language-dev]

The spelling is **settled for this plan**: `@{expr}` in Markdown Text;
Rocci template mode keeps `{expr}`. `{@expr}` and `{{expr}}` are not
implemented and are not aliases.[^research]

v1 gap-closing after Phases 1–5 is
[Rocdown `@{expr}` follow-ons after v1](rocdown-inline-interpolation-follow-ons.md).
This record stays exploratory and is not marked stable or CI-complete.[^follow-ons]

## Goal

Authors can splice a Roc `Str` expression into Markdown prose without a
document-root island:

```text
Published @{date}. There are @{count.to_str()} ideas.
```

After the last in-scope phase:

- `@{expr}` in Markdown **Text** (after Comrak) is an interpolation node.
- The payload is the same as Rocci `{expr}`: balanced Roc, `Str`, no
  markup, `Html.text`, `OriginKind::TextExpression`.[^template-readme][^text-ref][^rocci-lower]
- Fences, indented code, and inline code never interpolate.[^rocdown-readme][^markdown-rs]
- `\@{…}` in source is literal `@{…}`; `` `@{upstream}` `` stays code.[^scanner][^research]
- A hole promotes the page to **hydrate**. The Rust article renderer
  never evaluates Roc.[^article-rs][^catalog-shell]
- Public Rocdown reference and the crate README document the form.

## Out of bound

- `{@expr}`, `{{expr}}`, bare `{expr}` in Markdown, `${expr}`, MDX tags.
- Changing Rocci `{expr}` in templates, attributes, or HTML islands.
- A Rust-evaluated static subset sharing the `@{` spelling.
- Interpolation in fenced/indented/inline code, raw HTML, `:kind`
  params, link destinations, image `src`, or headings (v1).
- Executing fences; `@island`; Datastar signals as the hole.
- Identifier-only holes (that would fork Rocci `{expr}`).
- Interpolating `Html` (no body-parameter exception in Markdown).
- Enabling Markdown raw HTML so `{expr}` can hide in a paragraph
  island.[^format-arch]
- Using `@@{upstream}` as the Markdown git escape (in Rocci, `@@` emits
  `@` and `{upstream}` still opens an interpolation).[^rocci-parser]

## Constraints that do not move

1. **Markdown owns prose.** `@{` is an explicit inline island, not a
   line-start keyword. Emails and `@handles` stay Markdown.[^markdown-first][^rocdown-readme]
2. **Comrak first, Text only.** Do not scan the raw file for `@{` before
   block/inline parse. That would see fences.[^markdown-rs][^research]
3. **Source-aware escape.** CommonMark drops `\@` before Text exists. The
   splitter must look at the original span: a `@{` preceded by `\` is
   literal, matching line-start `\@roc`.[^scanner][^research]
4. **Reuse Rocci interpolation scan.** Export or share the balanced `{`
   skip (strings, `#` comments, nested `{ }`). Do not write a second
   expr grammar in `rocci-rocdown`.[^rocci-parser][^language-dev]
5. **Rust does not run Roc.** `classify_document` / `is_static_document`
   must **walk Markdown trees** (and `:kind` body Markdown). Today's
   walk skips `Item::Markdown` and `Item::Block`, so a hole-only page
   would stay `static` unless this walk is added.[^article-rs][^catalog-shell]
6. **Monotonic scan.** Unterminated `@{` diagnoses and still advances
   (`cur.pos > before`).[^rocci-parser]
7. **Document-root scanner unchanged.** `{` and `@{` are not reserved
   line-start declarations.[^scanner]

## Settled contract

| Topic | Rule |
| --- | --- |
| Opener | `@{` then Rocci `{expr}` payload (the `{` after `@`) |
| Close | Matching `}` at Rocci interpolation depth |
| Type | `Str`; `{count.to_str()}` at the hole |
| Where | Paragraphs, emphasis, strong, strike, link **text**, lists, quotes, table cells, footnote **bodies**, Markdown inside `:kind` `{{ }}` / line-scope / begin-end |
| Not where (v1) | `Code`, `CodeBlock`, `RawHtml`, headings, link/image URLs, `:kind` `[params]`, footnote **labels** |
| Escape | Source `\@{`; prefer `` `@{upstream}` `` for git/regex/jsDelivr |
| Forgotten escape | Usually a Roc diagnostic (`upstream` unbound, `@{2,}` not an expr), not silent HTML |
| Page class | Any hole → hydrate (reason `@{`); `docs/` static catalogs cannot use it |
| Bindings | `@roc` and document `@let`; not `@context` in render |
| Copy-paste | Prefix `@` onto an existing Rocci `{expr}` |

`@{if x { "a" } else { "b" }}` is a Roc `if` that returns `Str`. It is
not the markup directive `@if`.[^text-ref][^research]

## Current code anchors

These are the shipped functions a phase must touch. Do not invent a
second Markdown interpolator in the document-root scanner.[^scanner][^markdown-rs]

| Location | Today |
| --- | --- |
| `rocci-template` `Parser::parse_interpolation` | Private. Cursor on `{`; depth scan skips strings/`#` comments; returns `Interpolation { expr, span }` where `span` is `{`…`}` and `expr` is the trimmed inner. Unterminated still advances and diagnoses `"unterminated interpolation; expected \`}\`"`.[^rocci-parser] |
| `rocci-template` `Interpolation` | Already public (`expr: Span`, `span: Span`). Reuse this type from Rocdown; do not mint a second expr wrapper.[^rocci-parser] |
| `rocci-template` `lib.rs` `pub use parser` | Export the new scan helper next to `parse_declaration_from`. |
| `markdown.rs` `convert_document` | Comrak first. `NodeValue::Text` becomes `MdNode::Text { value, span }` with no split. `span` is remapped through `OffsetMap` onto the original file.[^markdown-rs] |
| `markdown.rs` heading convert | Slug from `children.iter().map(MdNode::text_content)`. A hole must not contribute an evaluated value to that string.[^markdown-rs] |
| `parse.rs` `parse` / `parse_fragment` / `parse_markdown_body` | All three call `convert_document`. Split once, after convert, so document-root Markdown, `:kind` bodies (`nested_items` / `parse_fragment`), and OKF `parse_markdown_body` share the hole.[^parse-rs] |
| `article.rs` `classify_document` / `is_static_document` | Skip `Item::Markdown`, `Item::Block`, `Item::Page`. A hole-only page stays `static` / `Ok(())` until these walks are added.[^article-rs] |
| `article.rs` `render_md` | `MdNode::Text` → escaped HTML text. Must not evaluate Roc. New arm is a gate bug if reached.[^article-rs] |
| `lower.rs` `MdNode::Text` | `Html.text("…")` with `OriginKind::MarkdownText`. Holes become `Html.text(expr)` with `OriginKind::TextExpression`.[^lowerer][^rocci-lower] |
| `ast.rs` `MdNode::children` / `text_content` / `children_mut` | Exhaustive. `Interpolation` is a leaf like `Text`. `text_content` on a hole returns `""` so heading slugs stay literal-only.[^markdown-rs] |
| `pprint.rs` / `Rocdown.Markdown.toml` | Overlay `write_md`; no `interp` tag yet.[^pprint][^md-toml] |
| `highlight.rs` | Template interpolations already paint `TextExpression`. Markdown `Text` is plain.[^highlight] |

`parse::nested_items` re-parses a `BlockCall` content span via
`parse_fragment`. Classification must walk **document items** and, for
each `Item::Block`, those nested items. Do not only look at top-level
`Item::Markdown`.[^parse-rs][^article-rs]

## Shared scan API

Add in `rocci-template` (names can move; keep the contract):

```text
pub struct InterpolationScan {
    pub expr: Span,      // trimmed inner, same as Interpolation.expr
    pub span: Span,      // from `{` through matching `}`
    pub terminated: bool
}

pub fn scan_interpolation(src: &str, open_brace: usize) -> InterpolationScan
```

Rules:

- `open_brace` is the `{` of either Rocci `{expr}` or Rocdown `@{expr}`.
- Depth starts at 1. Skip `"…"` via `Cursor::skip_string` and `#…`
  via `Cursor::skip_comment`. Nested `{` / `}` change depth.
- Every loop iteration advances (`pos > before`) so an unclosed hole
  terminates.[^rocci-parser]
- If unterminated, `terminated == false`, `span` ends at EOF or the
  scan stop, and `expr` is the trimmed remainder. Callers diagnose.
- Rocci `parse_interpolation` becomes: record start, `bump` `{`, call
  this, emit the existing unterminated diagnostic when needed, return
  `Interpolation { expr, span }`. Existing Rocci tests must stay green.

Rocdown wraps the result:

- Node `span` is `@` through the scan's closing `}`.
- Node `expr` is the scan `expr` unchanged.
- Inspect tag `interp`; overlay prints the expr atom (same family as
  Rocci `write_interp`).[^md-toml][^pprint]

Do not share `@@` handling. That is a Rocci template-mode text escape.
Markdown uses `\@{` (source backslash) or a code span.[^rocci-parser][^research]

## Source-aware Text split

CommonMark unescapes `\@` to `@` **before** `Text` exists. After convert,
`Text.value` for `\@{upstream}` is `@{upstream}`. A value-only scan would
still interpolate. The splitter must use the **original file** and the
node `span` (already remapped by `OffsetMap`).[^scanner][^research][^markdown-rs]

Suggested helper in `rocci-rocdown` (owned by `markdown.rs` or a small
sibling module):

```text
pub fn split_text_interpolations(
    src: &str,
    value: &str,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MdNode>
```

Algorithm:

1. Walk `src[span.start..span.end]` (or the full `src` with an offset
   cursor). Find the next `@{`.
2. If the byte immediately before that `@` is `\`, emit a `Text` whose
   value is the literal `@{…}` (CommonMark already dropped `\`) and
   continue after the scan close. Do not diagnose.
3. Otherwise call `scan_interpolation(src, brace_pos)`.
4. Emit leading `Text` (from the current offset to `@`), then
   `MdNode::Interpolation { expr, span: at_span }`.
5. Unterminated: diagnose with the Rocci message (or a Rocdown twin),
   still emit a recovery `Interpolation` or keep the remainder as
   `Text`, and **advance** so the walk cannot stall.
6. If the remaining slice has no `@{`, emit a trailing `Text`.
7. Never scan `Code`, `CodeBlock`, or `RawHtml` literals.

Map value slices back to `span` by using source offsets, not
`value.find("@{")` alone. `Text.value` can differ from source (entity
decoding, escaped punctuation). Prefer slicing `src` for hole bounds
and using `value` only when reconstructing inert `Text` runs that
Comrak already unescaped.

Walk after `convert_document` (and after `convert_children` for each
subtree): replace each `Text` in `children` vectors with the split
sequence. Do this in one post-pass over the converted tree so heading
slug assignment either runs **after** split (and `text_content` on
`Interpolation` is `""`) or **before** split while the heading is still
all `Text`. Prefer split-then-slug so a later heading diagnostic does
not rewrite ids. Phase 3 may leave heading glyphs as text and diagnose;
Phase 1 may still split them so tests can see the node.[^markdown-rs]

## Exhaustive match inventory

Adding `MdNode::Interpolation` breaks every `match node` in:

- `crates/rocci-rocdown/src/ast.rs` (`children`, `text_content`,
  `children_mut`)
- `crates/rocci-rocdown/src/article.rs` (`render_md`)
- `crates/rocci-rocdown/src/lower.rs` (Markdown lower)
- `crates/rocci-rocdown/src/pprint.rs` (`write_md`)
- `crates/rocci-rocdown/src/highlight.rs`
- `crates/rocci-rocdown/src/docs.rs` (static forest)
- `crates/rocci-rocdown/src/img.rs`, `links.rs`, `lsp.rs` if they match
  `MdNode` exhaustively

Phase 1 may compile with placeholder arms (`Html.empty`, skip paint,
`render_md` treats a hole as a bug string or empty). Phase 2 replaces
those with the real lower / static-gate behavior. Do not leave a
placeholder that silently prints `@{expr}` as HTML on the Roc path.

## Classification walk

```text
fn md_has_interpolation(node: &MdNode) -> bool
fn items_have_interpolation(src: &str, items: &[Item]) -> bool
```

- `md_has_interpolation`: `node.walk`, true on `Interpolation`.
- `items_have_interpolation`: for `Item::Markdown(n)` check the node;
  for `Item::Block(call)` check `parse::nested_items(src, call)`
  recursively. Ignore other items here (they already promote).
- `classify_document` needs `src` **or** the walk happens on the
  already-parsed `Document` plus nested parse. Today's signature is
  `classify_document(document, uses_datastar)` with no `src`. Either
  add `src: &str` or store nested Markdown on `BlockCall` at parse
  time. Prefer adding `src` at the existing call sites (site check,
  compile) over caching a second tree.[^parse-rs][^article-rs]
- Reason string: `"@{"`.
- `is_static_document` returns `Err("@{")` when the walk finds a hole.
- A page that is only Markdown + `@{x}` becomes **hydrate** even with
  no `@roc`. Missing bindings are a later Roc diagnostic, not a reason
  to stay static.[^article-rs][^catalog-shell]

Static `docs/` catalogs cannot ship `@{`. Phase 2 site check must fail
those files rather than render empty text.

## Implementation sketch

1. Export `scan_interpolation`; Rocci `parse_interpolation` calls it.
2. After Comrak → `MdNode`, split `Text` with the source-aware helper.
3. Lower `Interpolation` like Rocci: `Html.text(expr)` mapped
   `TextExpression`.[^rocci-lower][^lowerer]
4. Classify / static-gate via the walk above.
5. `render_md` / `docs.rs`: matching `Interpolation` is a gate bug
   (error, not evaluate).[^docs-rs]
6. Phase 3: diagnose holes in headings; lock destinations as inert
   strings.

## Phase 1: Tree spec and Text split

**Bound.** Add `Interpolation` to `Rocdown.Markdown.ungram` (leaf,
`expr:RocExpr` like Rocci) and the sidecar `[inline]` payload
`MdNode::Interpolation { expr: Span, span: Span }`, inspect tag
`interp`. Generate. Walk converted `MdNode` and split `Text` on
source-aware `@{`. Export `scan_interpolation` from `rocci-template`
and switch `parse_interpolation` to it with **no Rocci behavior
change**. Diagnose unterminated `@{`. `\@{` and code/fences stay
inert. Exhaustive arms compile; lowering may still be a placeholder.
Prefer AST inspect tests over generated-Roc correctness.[^md-ungram][^md-toml][^language-dev]

**Files.**

- Modify: `crates/rocci-template/src/parser.rs`,
  `crates/rocci-template/src/lib.rs`
- Modify: `crates/rocci-rocdown/Rocdown.Markdown.ungram`,
  `crates/rocci-rocdown/Rocdown.Markdown.toml`
- Generate: `crates/rocci-rocdown/src/md.generated.rs` (and any
  generated walkers)
- Modify: `crates/rocci-rocdown/src/markdown.rs` (post-convert split)
- Modify: `crates/rocci-rocdown/src/ast.rs`, `pprint.rs`, plus
  compile-only arms in `article.rs`, `lower.rs`, `highlight.rs`,
  `docs.rs`
- Test: `crates/rocci-rocdown/tests/compile.rs` (or a focused new
  test file). Optional unit tests next to `scan_interpolation`.[^compile-tests]

**Out of this phase.** Hydrate classification, public docs, AllSyntax
feature rows, LSP, heading placement policy (splitting heading Text is
allowed so nodes exist).

**Tests.**

- `Published @{date}.` → Text + interp(`date`) + Text
- `@{count.to_str()}` and `@{ if x { "a" } else { "b" } }`
- Nested braces: `@{List.len(items)}`, strings inside the expr
- `\@{upstream}` and `` `@{upstream}` `` stay inert
- Fenced `@{date}` and indented code stay `CodeBlock`
- Bare `{date}` stays Text
- Email `docs@example.com` and `@roclang` unchanged
- Unterminated `@{date` diagnoses and terminates
- Rocci `{expr}` tests still pass (extract did not change spans)

**Exit.** `cargo test -p rocci-template`. `cargo test -p rocci-rocdown
--test compile` (or the new file). `cargo run -q -p rocci-ungram --
check`. `cargo fmt --all -- --check`. Inspect dump shows `(interp
date)` (or the agreed atom).

## Phase 2: Lowering, hydrate, static gate

**Bound.** Lower Markdown interpolations to `Html.text(expr)` with
`OriginKind::TextExpression` (same emit path as Rocci template
interpolations). Implement `md_has_interpolation` /
`items_have_interpolation` and thread `src` into
`classify_document` / `is_static_document` (reason / err `"@{`). Site
`check` / `build` reject a static-catalog page that contains a hole.
`render_md` and `docs.rs` static forest: `Interpolation` is forbidden
on the Rust path (diagnostic or `Err`, not evaluate).[^lowerer][^article-rs][^docs-rs][^catalog-shell]

**Files.**

- Modify: `crates/rocci-rocdown/src/lower.rs` (replace placeholder)
- Modify: `crates/rocci-rocdown/src/article.rs` (classify +
  `render_md` arm)
- Modify: `crates/rocci-rocdown/src/docs.rs`, `site.rs` / catalog
  static gate call sites
- Tests: compile + any existing `classify_document` unit tests in
  `article.rs` / `site.rs`

**Out of this phase.** Heading/URL placement diagnostics if Phase 1
left heading holes as nodes (do them in Phase 3). Public docs.

**Tests.**

- Lowered Roc contains `Html.text(date)` (or the expr), not a string
  `"@{date}"`; source map origin is `text_expression`
- Page with only Markdown + `@{x}` + `@roc { x = "hi" }` classifies
  hydrate, reason `"@{"` or `"@roc"` (either is fine if kind is
  hydrate; prefer `"@{"` when the walk runs first, or keep max-kind
  and first reason — document the chosen order)
- Same page without `@roc` still classifies hydrate
- `is_static_document` returns `Err("@{")`
- `:note {{ Hello @{name}. }}` on a hydrate page lowers the hole; a
  static-only site check fails that file
- `render_document` / docs forest never emits the unevaluated glyphs
  as if they were prose

**Exit.** `cargo test -p rocci-rocdown`. `cargo fmt --all -- --check`.

## Phase 3: Placement rules

**Bound.** v1 diagnostics (not successful interpolation nodes):

- `@{` inside a heading (ATX `#` or `:h2` line-scope text). Diagnose
  at the `@` span; keep glyphs as `Text` for slug/`text_content` so
  ids stay source-literal. Do not evaluate into the heading id.
- `@{` that would sit in a link/image **destination**. Destinations
  are `Link.url` / `Image.url` strings today — do not invent URL
  interpolation. If CommonMark puts `@{` in the URL string, leave it
  literal and add a test that locks that.
- Footnote **labels** cannot contain holes; footnote **bodies** may.

Slug generation stays on literal heading text only.[^markdown-rs]

**Files.** `markdown.rs` (heading convert / post-split policy),
`compile.rs` tests, any heading-sugar path in `parse.rs` / `lower.rs`.

**Out of this phase.** URL interpolation follow-on. Heading holes as
a feature.

**Tests.** `# Hello @{ver}` errors at the hole and slug ignores the
expr. `[t](@{url})` does not become a Roc href (document actual
CommonMark behavior and lock it). `[see @{title}](/x/)` interpolates
**text** only. `![alt @{x}](./a.png)` : alt is Image payload — lock
whatever Comrak does; v1 should not interpolate `url`.

**Exit.** Focused compile tests plus `cargo test -p rocci-rocdown`.

## Phase 4: Docs, AllSyntax, authoring surface

**Bound.** Update `crates/rocci-rocdown/README.md`,
`docs/rocdown/language.rocdown`, `docs/rocdown/pages.rocdown` (or the
settled stack-first path), `.agents/skills/rocci-author/SKILL.md`
Rocdown essentials. Add a **hydrate** example (extend
`examples/rocdown/pages/Guide.rocdown` or a small sibling) with `@roc`
+ `@{…}` in a paragraph. Add rows to `test/AllSyntax.rocdown` and
refresh the inspect fixture via the language-dev path. Do not put
`@{` into static `docs/` catalog pages. Mark heading/URL holes
**planned**. Cross-link the research and this plan from the crate
README only as "implemented" after the tests exist.[^rocdown-readme][^lang-ref][^all-syntax][^language-dev]

**Files.** README, public Rocdown pages, `test/AllSyntax.rocdown`,
inspect fixture, example page, `rocci-author` skill, optional
knowledge README one-liner (not a new record).

**Out of this phase.** LSP unless highlight is blocking inspect.

**Exit.** `cargo test -p rocci-rocdown`. `cargo run -q -p
rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown` reviewed.
`cargo run -q -p rocci-okf -- check knowledge --profile rocci --format
terminal` after knowledge/README cross-links. Public pages state
**current** for `@{expr}` and **planned** for heading/URL holes.

## Phase 5: Highlight and LSP

**Bound.** Paint `@{` / expr / `}` in Rocdown highlight like Rocci
interpolations (`OriginKind::TextExpression` / existing template
interp path). LSP semantic tokens and go-to/diagnostics use the
interp span (`@` through `}`). No formatter work unless an existing
Rocdown formatter already walks `MdNode`.[^highlight]

**Files.** `crates/rocci-rocdown/src/highlight.rs`,
`crates/rocci-rocdown/src/lsp.rs`, tests in `crates/rocci-rocdown/tests/lsp.rs`.

**Exit.** `cargo test -p rocci-rocdown`. `cargo test -p rocci-lsp` if
tokens change.

## Validation (every phase)

```sh
cargo test -p rocci-template
cargo test -p rocci-rocdown
cargo run -q -p rocci-ungram -- check
cargo fmt --all -- --check
```

After Phase 4, also inspect AllSyntax. After a cross-crate export,
`cargo test --workspace` before hand-off. Do not set `ROCCI_REQUIRE_ROC=1`
unless a phase explicitly proves generated Roc against the pinned
toolchain.[^language-dev]

## Follow-ons (not this plan)

Renderer gate, hydrate-matrix drift, splitter tests, and LSP go-to without a
hole target:
[Rocdown `@{expr}` follow-ons after v1](rocdown-inline-interpolation-follow-ons.md).[^follow-ons]

Still deferred:

- Heading and destination interpolation.
- `{@` / `{{` aliases (do not add).
- Static Rust substitution of `@page.meta` under a different spelling.

[^research]: Settled `@{expr}`; collisions, escape, and expression table.
[^markdown-first]: Transitions stay explicit; inline `@` in emails is not a declaration.
[^catalog-shell]: Rust owns static article HTML; Roc owns authored dynamic regions.
[^format-arch]: Document-root HTML islands versus disabled Markdown raw HTML.
[^rocdown-readme]: Line-start `@` / `:kind` / `<Tag>`; fences inert; `\@roc`; bare `{expr}` is prose.
[^scanner]: `\@` at line start is not a declaration; no inline interpolator yet.
[^markdown-rs]: Comrak `Text` / `Code` / `CodeBlock`; heading ids from child text.
[^parse-rs]: `parse` / `parse_fragment` / `parse_markdown_body` all convert Comrak; `nested_items` re-parses a `BlockCall` body.
[^article-rs]: `classify_document` skips `Item::Markdown` today; `render_md` is Rust `Html.text`.
[^lowerer]: Markdown text currently lowers as a string literal.
[^docs-rs]: Static widget forest must not evaluate Roc exprs.
[^pprint]: Inspect overlay `write_md` must grow an `interp` leaf.
[^highlight]: Template interpolations already have a highlight path to reuse.
[^md-ungram]: Markdown projection has no interpolation production.
[^md-toml]: Inspect tags live in the Markdown sidecar.
[^rocci-parser]: Balanced `{` scan; `@@` is literal `@` inside templates.
[^rocci-lower]: `Html.text(expr)` and `OriginKind::TextExpression`.
[^template-readme]: Rocci `{expr}` must be `Str`; no markup inside.
[^text-ref]: `{if active { <Icon /> }}` is rejected; use `@if` for markup.
[^lang-ref]: Public Rocdown declaration and static/hydrate/live matrix.
[^all-syntax]: Comprehensive fixture and inspect lock.
[^language-dev]: Ungram then parser; AllSyntax; crate tests and fmt.
[^compile-tests]: Existing compile contract tests to extend, not replace.
[^follow-ons]: Post-v1 gate, docs matrix, tests, and LSP go-to; not heading/URL interpolation.
