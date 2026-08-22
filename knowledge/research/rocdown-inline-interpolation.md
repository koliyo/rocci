---
type: Research Report
title: Inline interpolation in Rocdown Markdown
description: "Settled Markdown hole is `@{expr}` (prefix around Rocci `{expr}`). `{@expr}` and `{{expr}}` remain rejected alternatives. Implementation: knowledge/plans/rocdown-inline-interpolation.md. Not shipped."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/authoring, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-22T20:30:00Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: scanner
    resource: ../../crates/rocci-rocdown/src/scan.rs
    title: Document-root line-start scanner
    author: process:git
    last_modified: 2026-08-22
  - id: markdown-rs
    resource: ../../crates/rocci-rocdown/src/markdown.rs
    title: Comrak Markdown to MdNode conversion
    author: process:git
    last_modified: 2026-08-21
  - id: lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Rocdown to Roc lowerer
    author: process:git
    last_modified: 2026-08-22
  - id: article-rs
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Static/hydrate/live classification and Rust article HTML
    author: process:git
    last_modified: 2026-08-22
  - id: rocci-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Rocci interpolation and @@ escape
    author: process:git
    last_modified: 2026-08-22
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Implemented Rocci template contract
    author: process:git
    last_modified: 2026-08-22
  - id: rocci-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Interpolation lowering to Html.text
    author: process:git
    last_modified: 2026-08-22
  - id: format-arch
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-20
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
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: lang-ref
    resource: ../../docs/rocdown/language.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-22
  - id: text-ref
    resource: ../../docs/reference/language/text.rocdown
    title: Rocci text and interpolation reference
    author: process:git
    last_modified: 2026-08-21
  - id: markup-guide
    resource: ../../docs/templates/markup.rocdown
    title: Rocci markup guide
    author: process:git
    last_modified: 2026-08-22
  - id: md-ungram
    resource: ../../crates/rocci-rocdown/Rocdown.Markdown.ungram
    title: Markdown projection AST
    author: process:git
    last_modified: 2026-08-19
  - id: html-roc
    resource: ../../crates/rocci-rocdown/runtime/Html.roc
    title: Roc string interpolation in Html helpers
    author: process:git
    last_modified: 2026-08-17
  - id: pages-guide
    resource: ../../docs/rocdown/pages.rocdown
    title: Write Rocdown pages
    author: process:git
    last_modified: 2026-08-22
  - id: why-roc
    resource: ../../docs/appendix/why-roc.rocdown
    title: Roc string interpolation example
    author: process:git
    last_modified: 2026-08-22
  - id: skip-section
    resource: ../../crates/rocci-rocdown/src/scan.rs
    title: Article-block {{ }} skipper, including fence opacity
    author: process:git
    last_modified: 2026-08-22
  - id: colon-test
    resource: ../../crates/rocci-rocdown/tests/colon_syntax.rs
    title: Fenced braces inside {{ }} bodies stay opaque
    author: process:git
    last_modified: 2026-08-19
  - id: git-upstream
    resource: ../../tools/rocci-ops/src/rocci_ops/local.py
    title: Git @{upstream} ref in operator tooling
    author: process:git
    last_modified: 2026-08-22
  - id: datastar-cdn
    resource: ../../crates/rocci-cli/src/datastar_asset.rs
    title: jsDelivr datastar@{tag} URL template
    author: process:git
    last_modified: 2026-08-16
  - id: regex-quantifiers
    resource: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Regular_expressions/Quantifiers
    title: Regex quantifiers x{n} and x{n,}
    author: organization:mdn
    last_modified: 2026-08-22
  - id: directives-ref
    resource: ../../docs/reference/language/directives.rocdown
    title: Rocci @if @for @match @let contract
    author: process:git
    last_modified: 2026-08-22
  - id: impl-plan
    resource: ../plans/rocdown-inline-interpolation.md
    title: Rocdown Markdown @{expr} interpolation plan
    author: process:cursor
    last_modified: 2026-08-22
---

# Inline interpolation in Rocdown Markdown

## Scope and authority

This record is exploratory research. The Markdown spelling is **settled
as `@{expr}`** for implementation. The plan is [Rocdown Markdown
`@{expr}` interpolation](../plans/rocdown-inline-interpolation.md). This
record does not describe shipped behavior.[^markdown-first][^impl-plan]

The question is how to splice a **Roc or Rocci `Str` value into Markdown
prose** in positions where Rocdown's line-start keywords cannot appear:
mid-paragraph, emphasis, link text, list items, table cells.

It is **not** a proposal for inline component tags, MDX, raw HTML in
paragraphs, or a second expression language in the Rust article
renderer.[^format-report][^catalog-shell]

## The gap

Rocdown recognizes executable syntax only at a document-root line start:
reserved `@name`, `:kind` article blocks, and standalone `<Tag>` HTML
islands. `@` in a paragraph, email, handle, or fence is Markdown. A colon
with a following space stays Markdown. Bare `{expr}` at document root is
still prose.[^rocdown-readme][^scanner][^lang-ref]

Rocci already has interpolation, but only inside template mode (component
bodies, document-root HTML islands, `@if` / `@for` / `@match` / `@let`
bodies): `{expr}` is a Roc expression that must be `Str` in text position
and lowers to `Html.text`. Markup inside `{…}` is rejected. `@@` is a
literal `@`.[^template-readme][^text-ref][^markup-guide][^rocci-lower][^rocci-parser]

So this is legal in a template and illegal as Markdown prose:

```text
Published {date}.
```

The format investigation deferred inline Roc in sentences for v1 and said
to add a form only after demonstrated pain. The Markdown-first decision
records the consequence: inline dynamic prose currently needs a small
component or a whole paragraph built in `@render` / HTML.[^format-report][^markdown-first]

## Shipped workarounds

These are implemented. They are block- or island-shaped, not inline.

| Workaround | Example | Cost |
| --- | --- | --- |
| Colocated component plus root tag | `@component Byline = \|{ date }\| { <p>Published {date}</p> }` then `<Byline date={published} />` | Extra declaration; the sentence is HTML, not Markdown.[^pages-guide] |
| `@render Name({ … })` | Prefix call at line start | Same; produces `Html`, not a text hole in a paragraph.[^rocdown-readme] |
| Document-root HTML island | `<p>Published {date}</p>` | `{date}` works because the line is a Rocci template, not Markdown. Inline HTML inside a Markdown paragraph stays disabled raw HTML.[^rocdown-readme][^format-arch] |
| Document-root `@if` / `@for` / `@let` | Template bodies, not Markdown | `#` in those bodies is a template comment.[^rocdown-readme] |

The typical authoring need this does not cover: keep Markdown emphasis,
links, and wrapping, and insert one `Str` in the middle.

```text
Rocci **@{version}** is the current toolchain. See [the install guide](/docs/install/).
```

Today that sentence must become an HTML island or a component, which drops
Markdown inlining for the whole run.

## Constraints that do not move

1. **Markdown owns prose.** A language transition must stay visible. Inline
   `@`, emails, fences, lists, and quotations do not switch block
   mode.[^markdown-first][^rocdown-readme]
2. **Do not adopt MDX.** Inline Roc plus JSX-like tags in arbitrary
   Markdown positions was considered and rejected for v1 because braces and
   angle brackets in prose would become language-sensitive.[^format-report]
3. **Rust does not evaluate Roc.** Static catalog HTML is a Rust article
   renderer. Roc expressions belong on the hydrate/live Roc path, compiled
   only for pages that need them.[^catalog-shell][^article-rs]
4. **Reuse Rocci interpolation semantics** if a Markdown form exists: opaque
   balanced Roc expr (not an identifier-only hole), `Str` only, no markup
   inside, `Html.text` lowering, existing `OriginKind::TextExpression`.
   A spelling that is only comfortable for `{@date}` but ugly for `{@1 + 1}`
   or `{@if …}` is a different, narrower language.[^rocci-lower][^text-ref][^directives-ref]
5. **Literal Markdown stays literal.** Fenced code (triple backticks or tildes),
   indented code, and inline code spans never execute today and must never
   interpolate. Comrak already projects those to `CodeBlock` / `Code`, not
   `Text`. Any interpolator must run **after** that split and walk `Text`
   only.[^rocdown-readme][^markdown-rs][^md-ungram]
6. **Article-block `{{ }}` is a body wrapper**, recognized only after a
   line-start `:kind`. It is not by itself a reason to forbid inline
   `{{expr}}`. The skipper already nests `{{` / `}}` and treats fences as
   opaque.[^rocdown-readme][^skip-section][^colon-test]

## Collision inventory

Any delimiter has to survive text that already appears in Rocdown source.

| Existing spelling | Owner | Why it collides |
| --- | --- | --- |
| `{expr}` in tags and attributes | Rocci templates | Correct inside template mode; **not** a Markdown signal today.[^template-readme] |
| Bare `{ident}` in Markdown | Prose and docs | Shipped pages write `{id}#heading`, `{primary}.db`, `fn-{name}` as documentation placeholders, not code.[^lang-ref] |
| `{ name: "Ada" }` | Roc records in unfenced primer prose | Would look like an interpolation if `{` opened an expr.[^why-roc] |
| `{{ … }}` | `:kind` wrapped bodies | Same glyphs as a possible inline `{{expr}}`. Not fatal: block `{{` is only after line-start `:kind`; inner `{{` / `}}` already nest; fences inside the body are opaque.[^rocdown-readme][^skip-section][^colon-test] |
| `"${expr}"` | Ordinary Roc strings | Html helpers and docs already use this.[^html-roc][^why-roc] |
| `${item.id}` in actions | Datastar URL templates inside Rocci attributes | Different language, same glyphs.[^template-readme] |
| `@name` at line start | Rocdown/Rocci declarations | Reserved names only; unknown `@name` stays Markdown.[^scanner] |
| `@roclang`, `docs@example.com` | Markdown | `@` then a name or domain; no `{` immediately after `@`.[^rocdown-readme] |
| `@{n}`, `@{n,}`, `@{n,m}` | Regex quantifier on `@` | `@{2,}` means two or more at-signs. Common in email-validation write-ups that quantify `@`. Unfenced, this is a real `@{expr}` false positive.[^regex-quantifiers] |
| `user@{domain}` | Prose / glob placeholder | Documents a metavariable domain. Usually belongs in a code span; unfenced it matches `@{expr}`. |
| `@{upstream}`, `@{u}`, `@{1}` | Git reflog / upstream shorthand | Already appears in this repo's operator tooling. Docs about git would collide unfenced.[^git-upstream] |
| `package@{tag}` | jsDelivr-style URLs | Same glyph run; this repo writes `datastar@{tag}`.[^datastar-cdn] |
| `{user,admin}@example.com` | Shell brace-expansion glob | `{` then a name, then `}@`. Does **not** match `@{` or `{@`. |
| `[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}` | Typical email regex | `{2,}` quantifies the TLD class, not `@`. The `@` sits before `[`, not `{`. |
| `\@roc` | Escape for a literal line-start example | Backslash is dropped in rendered text.[^rocdown-readme] |
| `@@` | Literal `@` in a template body | Template-mode only.[^template-readme] |
| `:kind[params]` | Article blocks at line start | Inline `:name` is not recognized; `: definition` stays Markdown.[^lang-ref] |
| `<Tag>` at line start | HTML island | Inline `<span>` in a paragraph is raw HTML and errors by default.[^format-arch] |
| `` `{expr}` `` and fences | Display | Must remain inert. This is the main mitigation for regex, git, and glob examples.[^rocdown-readme][^markdown-rs] |

The important empirical point: **bare `{expr}` is already documentation
prose.** Teaching Markdown that a single `{` starts Roc would silently rewrite
pages that discuss routes, archives, and syntax. Double braces and `@`-marked
braces are much rarer in unfenced prose.

## The payload is a Roc expression

Earlier rounds scored spellings on **identifiers** (`date`, `version`) and
on **foreign collisions** (email regex, Handlebars, git). That is too
narrow. Rocci `{expr}` is already a full Roc expression: field access,
calls, `++`, `if` / `match` that return `Str`, pipelines. Markdown must
not invent a second, identifier-only hole if constraint 4 stands.[^text-ref][^template-readme]

Rocci also already owns `@` as a **directive prefix** (`@if`, `@for`,
`@match`, `@let`). Those produce markup. A Roc `if` / `match` inside an
interpolation produces `Str` and is a different construct. Authors already
confuse the two: `{if active { <Icon /> }}` is a documented error (markup
in an interpolation). A Markdown spelling that starts the payload with
`@if` or `@match` makes that confusion worse.[^text-ref][^directives-ref]

Humans group `{@1 + 1}` as `{` + `@1` + ` + 1}`, not as opener `{@` plus
`1 + 1`. `@` is a prefix in this stack (directives, mentions, annotations).
Gluing it to the first token of the expr is the `{@expr}` cost. `@{1 + 1}`
groups as `@` + `{1 + 1}`, which matches both the parse and Rocci `{1 +
1}` (still needs `.to_str()` for text). `{{1 + 1}}` groups as a wrapped
expr with no `@` on a token.

Worked forms. `Str` conversions are shown where Rocci already requires
them.

| Rocci template (legal today) | `{@…}` | `@{…}` | `{{…}}` |
| --- | --- | --- | --- |
| `{date}` | `{@date}` | `@{date}` | `{{date}}` |
| `{count.to_str()}` | `{@count.to_str()}` | `@{count.to_str()}` | `{{count.to_str()}}` |
| `{List.len(items).to_str()}` | `{@List.len(items).to_str()}` | `@{List.len(items).to_str()}` | `{{List.len(items).to_str()}}` |
| `{(1 + 1).to_str()}` | `{@(1 + 1).to_str()}` / `{@1 + 1}` | `@{(1 + 1).to_str()}` | `{{(1 + 1).to_str()}}` |
| `{name ++ " (" ++ ver ++ ")"}` | `{@name ++ " (" ++ ver ++ ")"}` | `@{name ++ " (" ++ ver ++ ")"}` | `{{name ++ " (" ++ ver ++ ")"}}` |
| `{if x { "a" } else { "b" }}` | `{@if x { "a" } else { "b" }}` | `@{if x { "a" } else { "b" }}` | `{{if x { "a" } else { "b" }}}` |
| `{match s { A => "a", B => "b" }}` | `{@match s { A => "a", B => "b" }}` | `@{match s { A => "a", B => "b" }}` | `{{match s { A => "a", B => "b" }}}` |

Consequences:

**`{@` affixes `@` to the first token.** `{@date}` is tolerable.
`{@count.to_str()}` reads as a mention ` @count ` then a call.
`{@List.len(…)}` reads as `@List`. `{@1 + 1}` reads as `@1`.
`{@name ++ "x"}` reads as mention-plus-string. `{@if …}` and `{@match …}`
look like Rocci directives stuffed into braces. That is not a cosmetic
nit: `@if` / `@match` are the words authors already use for markup
control flow.[^directives-ref]

Requiring a space (`{@ 1 + 1}`, `{@ if x { "a" } else { "b" }}`) unglues
the first token but keeps `{@` looking like an annotation, splits the
common case (`{@date}` vs `{@ date}`), and still looks like `@if` with an
extra brace. Parenthesizing (`{@(1 + 1)}`) is a tax Rocci `{1 + 1}` does
not charge.

**`@{` is a prefix around existing Rocci `{expr}`.** Copy from a component
`<p>{count.to_str()}</p>` into Markdown by putting `@` in front:
`@{count.to_str()}`. The inner `{…}` is the same balanced-brace payload
the Rocci parser already scans, including `if` / `match` that end in `}}`
(inner block plus interpolation closer). Authors already write that in
templates.[^rocci-parser][^text-ref]

**`{{` reads well until the expr contains `{ }`.** Identifiers and calls
are fine: `{{date}}`, `{{count.to_str()}}`. A Roc `if` / `match` uses
single braces, so the interpolation closer `}}` sits after the last `}`
of the expr: `{{if x { "a" } else { "b" }}}` ends in **three** braces.
Compact `{{if x {"a"} else {"b"}}}` is easy to close one `}` too early if
the scanner treats `}}` as the hole closer at Roc depth 0. That is a
new closer rule Rocci `{expr}` does not have (Rocci counts single `{` /
`}` only).[^rocci-parser]

**Adjacent `@` in prose.** Building an address `{{user}}@example.com` is
readable. `@{user}@example.com` is a run of `@` and braces. `{@user}@example.com`
looks like a mention in braces then a domain. Rare, but it shows `{@` /
`@{` are the wrong tool when the next character is also `@`.

**Silent narrowing.** If `{@name}` feels fine and `{@1 + 1}` / `{@if …}`
feel wrong, authors will only interpolate identifiers and hide arithmetic
in helpers. That forks Markdown holes from Rocci `{expr}` without saying
so in the grammar.

## Alternatives

Scores are relative to Rocci + Rocdown as they exist, not to MDX popularity.
**Fit** means visible mode switch, delimiter uniqueness, and reuse of
shipped interpolation semantics.

### A. Status quo (block islands only)

Keep interpolation inside Rocci regions. Mid-sentence values stay a
component, `@render`, or a root HTML paragraph.

**Fit:** highest with the Markdown-first decision and the v1 format
report.[^markdown-first][^format-report]

**Cost:** a one-word hole still costs a block island and loses Markdown
inlining for that paragraph.

**Keep** until a plan is approved. This research exists so that plan has a
spelling.

### B. Bare `{expr}` in Markdown inlines (reject)

Copy Rocci text interpolation into Markdown Text nodes.

**Fit with Rocci:** spelling match.

**Fit with Rocdown:** poor. `{` is not a language-transition sigil in
Markdown; the format report rejected this class of change so ordinary
braces stay content.[^format-report] Shipped docs would become accidental
programs. Unterminated `{` and nested records are expensive to diagnose.

Reject as a Markdown feature. Keep `{expr}` as the **template-mode** form.

### C. `{{expr}}` (live candidate)

```text
Published {{date}}. There are {{count.to_str()}} ideas.
```

Earlier this record treated Handlebars / Liquid / Mustache wrapping as a
reason to reject. That overweights an outer template language. If Rocdown
already interpolates, wrapping the file in Handlebars is the odd layering;
an external value pass can be added later as a distinct, skipped spelling
if a host ever needs one.

The remaining tension is **Rocdown's own** `:kind {{ … }}` body wrapper,
not a third-party engine.[^rocdown-readme]

Those two uses are different scan phases:

1. Document-root `:note {{` opens a **block body**. The skipper counts
   `{{` / `}}` depth and ignores fences.[^skip-section][^colon-test]
2. After Markdown parse, `{{expr}}` in a **Text** node is an inline hole.

So `:note {{ There are {{count.to_str()}} ideas. }}` already nests: inner
`{{count.to_str()}}` increments then decrements depth and does not close
the note early. Line-scope `:note Hello {{count}}` has no body wrapper.

The awkward line is `:note {{count}}`, which today is a brace **body**
containing `count`, not an interpolation. Document-root `:kind` still
wins. Write `:note Hello {{count.to_str()}}` (line-scope) or put the hole
inside a wrapped body.

**Fit:** high visibility; familiar to Mustache readers; extra brace
distances it from Roc records `{ name: 1 }` and from Rocci `{expr}`;
binary `{{1 + 1}}` and `{{count.to_str()}}` read as wrapped expressions,
not as `@` stuck to a token.

**Cost:** two meanings of `{{` in one language (block wrapper vs inline
hole). Authors must remember `:kind {{` is not interpolation. Roc `if` /
`match` inside the hole end in `}}}` (see expression table). Inline code
and fences still required for Mustache examples. This is **not** a
Handlebars-wrapping reject.

### D. `${expr}` (reject)

Collides with Roc string interpolation and Datastar path templates.[^html-roc][^template-readme]
Also reads as JS, which Rocdown is not.

### E. Inline HTML / `<span>{expr}</span>` in paragraphs (reject for this need)

Would make raw HTML the interpolator. The default parser rejects Markdown
raw HTML; document-root tags are a different, block-level path.[^format-arch]
This is a wider MDX-shaped hole than a dedicated text interpolator.

### F. Inline `:kind`, e.g. `:val[count]` or generic-directive `:name[]{}` (reject for variables)

Article blocks are structure (note, tabs, img), recognized at line start.
Overloading `:` for a `Str` hole mixes block kinds with inlines and still
needs a new inline scanner. Micromark-style inline directives are a
possible later block feature, not a variable interpolator.

### G. Inline `@render` of a `Str` (reject as the primary form)

`@render` is a PascalCase **Html** call at line start, not a text
hole.[^rocdown-readme] Teaching `@render(count.to_str())` mid-sentence
overloads Html vs `Str` and still wants an inline `@` scanner.

### H. `{@expr}` — braces like Rocci, `@` as Markdown disambiguator (weak)

```text
Published {@date}. There are {@count.to_str()} ideas.
```

Scan Text nodes for `{@`, then reuse Rocci's balanced-brace skip.[^rocci-parser]
A record `{ name: 1 }` and a placeholder `{id}` stay prose because `{` is
not followed by `@`.

**Email / glob / regex:** still the spelling that does **not** collide
with a quantifier on `@`. `{@date}` is not `{n}`. Ordinary emails and
`*@host` never produce `{@`.[^regex-quantifiers]

**That regex win does not survive full expressions.** `{@` inserts `@`
*inside* the Rocci `{expr}` form, glued to whatever token comes first.
`{@1 + 1}`, `{@List.len(items).to_str()}`, `{@if x { "a" } else { "b" }}`,
and `{@match s { … }}` are the same construct as `{@date}` in the parser
and a different construct in the reader's eye. `{@if` / `{@match` collide
with the directive vocabulary.[^directives-ref] This is **not** the closest
spelling to Rocci braces: Rocci is `{count.to_str()}`, not `{@count.to_str()}`.

A required space (`{@ 1 + 1}`) is a mitigation, not a fix. It makes the
common identifier case inconsistent and still reads as an annotation.

**Fit:** good only if Markdown holes are *de facto* identifiers. That
violates reuse of Rocci `{expr}` semantics.

**Cost:** first-token `@` binding; directive-word clash; copy-paste from
templates is an edit inside the braces, not a wrapper.

### I. `@{expr}` — `@` as mode switch, `{expr}` as Rocci payload (live)

```text
Published @{date}. There are @{count.to_str()} ideas.
```

`@` is already the executable-transition sigil in both languages. In a
`.rocci` body you are already in template mode, so `{expr}` is enough. In
Markdown you are in prose mode, so the same payload needs a visible switch
that works **off the line start**. `@{` is that switch: **prefix `@` plus
an unchanged Rocci `{expr}`**.

Ordinary emails and handles do **not** collide: `docs@example.com` and
`@roclang` have `@` then a name, not `@` then `{`. Line-start recognition
looks for `@` plus a reserved **name**; `@{` is not a reserved name, so
today's scanner leaves it as Markdown, which is the right hand-off to an
inline pass.[^scanner][^rocdown-readme]

Binary and control-flow exprs keep Rocci shape: `@{1 + 1}`, `@{if x { "a" }
else { "b" }}` (same trailing `}}` authors already type in templates).
`@{if` is `@` + `{if …}`, not the directive `@if`.[^rocci-parser][^directives-ref]

**Email regex / glob (unfenced collision):** `{n}` after an atom repeats
that atom. `@{2,}` means two or more at-signs; `@{1}` means exactly one
`@`. Those appear in email-validation notes. A glob or doc placeholder
`user@{domain}` is the same glyph run. Git `@{upstream}` / `@{1}` and
jsDelivr `package@{tag}` are the same shape.[^regex-quantifiers][^git-upstream][^datastar-cdn]

Fence and inline-code skipping removes almost all of those from
execution. Residual risk is **unfenced** `@{…}` in a sentence.
`@{user}@example.com` is also ugly (two `@` roles in one token run).

**Escape is enough for git `@{upstream}` if we pick `@{expr}`.** Write
`\@{upstream}` in prose, or (better for tokens) a code span
`` `@{upstream}` ``. That is the same family as `\@roc` today: the
backslash is not part of the rendered text. Documentation should say git
reflog / regex `@{n}` / `pkg@{tag}` belong in code or `\@{…}`.[^rocdown-readme][^scanner]

That is **not** automatic from CommonMark. `\@` is a CommonMark escape;
after Comrak the Text node is already `@{upstream}` with the backslash
gone. Today's `\@roc` works because the **document-root scanner** refuses
a line that starts with `\@` before Markdown runs.[^scanner] An inline
interpolator must do the same: when splitting Text, look at the source
span and skip a `@{` that is preceded by `\`. Walking unescaped Text
alone would still interpolate `\@{upstream}`.

A forgotten escape is usually a **diagnostic**, not silent wrong HTML:
`@{upstream}` with no `upstream` binding fails Roc; `@{2,}` is not a Roc
expr. It is only silent if `@roc` actually binds that name as `Str`.

`@@{` in a template body is already `@{` as text because `@@` emits `@`.
Do not reuse `@@{upstream}` as the Markdown git escape: in Rocci that
emits `@` and then still opens `{upstream}`.[^rocci-parser][^rocdown-readme]

**Fit:** only spelling that is composition around shipped `{expr}`;
binaries and `if` / `match` stay readable.

**Cost:** unfenced regex/git/`pkg@{tag}` unless escaped or in code; constructing
`local@domain` with an interpolated local part.

### J. Restricted static substitution in Rust (separate feature)

A catalog-only spelling such as substituting `@page.meta.title` from data
Rust already extracts, with **no general Roc expr**.

That could serve docs version strings on `static` pages without hydrate.
It is not Rocci interpolation and must not pretend to be. Do not mix it
into the same syntax as the Markdown hole.

## Literal contexts (mandatory for every spelling)

Do not interpolate in:

| Context | Why | How the parser already helps |
| --- | --- | --- |
| Fenced code (` ``` ` / `~~~`) | Examples of Roc, regex, git, Handlebars, Rocci | `MdNode::CodeBlock`[^markdown-rs] |
| Indented code blocks | Same | `CodeBlock` |
| Inline code spans | Same for short tokens | `MdNode::Code` |
| Raw HTML | Disabled / dangerous | `RawHtml` error path[^format-arch] |
| `:kind` params `[…]` | No Roc calls in v1 | Not Markdown Text[^lang-ref] |

Walk **only** `MdNode::Text` after Comrak. Do not scan the raw file for
holes before Markdown block/inline parse; that would see fences.

Article-block **bodies** are Markdown. `@{` has no body-skipper
interaction; allowing holes in `:note {{ … }}` is the same Text-node
pass. Still forbid holes in `[params]`.

## Recommendation

Keep **`{expr}`** unchanged in Rocci template mode. Score Markdown holes as
**full Roc expressions**, not as `{@date}` mascots. Fence/code skipping
stays mandatory for every spelling.

| Spelling | Strength | Structural cost |
| --- | --- | --- |
| `@{expr}` | Prefix around shipped `{expr}`; `{@1 + 1}`-class exprs stay `{1 + 1}` | Unfenced `@{2,}`, git `@{upstream}`, `pkg@{tag}` — handled by code spans, `\@{…}` (source-aware, like `\@roc`), and docs; forgotten escape is usually a Roc error |
| `{{expr}}` | `{@`-free; binaries and calls read as a wrapped expr | `:kind {{` already means a body; Roc `if`/`match` closer is `}}}` |
| `{@expr}` | Avoids regex `@{n}` | `@` binds to the first token; `{@if` / `{@match` look like directives; not a wrapper around `{expr}` |

`{@expr}` is the wrong default if the hole is really Rocci `{expr}`. The
implementation plan **settles `@{expr}`**. `{@expr}` and `{{expr}}` stay
documented alternatives only; they are not aliases and are not
shipped.[^impl-plan]

Do not ship MDX, bare `{expr}`, or `${expr}`.

This does not override the Markdown-first decision: the hole is still an
**explicit** island, just an inline one, analogous to how Markdown already
has inlines (`*`, `` ` ``, `[ ]`) that are not line-start keywords.[^markdown-first]

## Semantics if a Markdown hole were adopted

Exploratory contract for the settled spelling `@{expr}`, not implemented.

**Payload.** Same as Rocci interpolation: balanced Roc expr, strings and
`#` comments skipped inside, no markup, result `Str`, lower with
`Html.text` and `OriginKind::TextExpression`. Convert numbers at the hole
(`{@count.to_str()}`, `{{count.to_str()}}`, or `@{count.to_str()}`). No
Html-as-is exception (Markdown has no body parameter).[^rocci-lower][^text-ref]

**Where.** Markdown **Text** descendants after Comrak: paragraphs, emphasis,
strong, strikethrough, link **text**, list items, block quotes, table cells,
and Markdown inside `:kind` wrapped bodies. Not: code spans, fences,
indented code, raw HTML, link destinations, image `src`, or `:kind`
params. Today those Text nodes lower as string literals, not Roc
expressions.[^lang-ref][^markdown-rs][^md-ungram][^lowerer]

**Where not, for v1.** Headings (slug / outline stability), wiki-link
targets, footnote labels.

**Page class.** A hole is a Roc expression. It promotes the page to
**hydrate** the same way `@roc` / `@render` / a root `<Tag>` already
do. `docs/` as a static catalog cannot use it without leaving `static`.
Do not evaluate Roc in the Rust article renderer.[^article-rs][^catalog-shell]

**Bindings.** `@roc` values and document-level `@let` (already hoisted into
`rocci_content`). Not `@context` inside render.[^rocdown-readme]

**Escape.** Backslash before the opener in **source** (`\@{`, `\{@`, `\{{`).
CommonMark drops the `\` from Text, so the interpolator must look at the
byte before `@{` in the original span, the same way line-start `\@roc`
skips declaration scan. Prefer `` `@{upstream}` `` for git/regex tokens.
Fences for longer samples.[^scanner][^rocdown-readme]

**Recovery.** Unterminated hole is a diagnostic at that span; the scanner
must still advance (`cur.pos > before`) on every path.[^rocci-parser]

**Parser sketch.** Do not make `{` special in the document-root scanner.
After Comrak builds `MdNode`, split `MdNode::Text` on the chosen opener
with the existing balanced skip, emitting a new Markdown interpolation
node (ungram on `Rocdown.Markdown.ungram`, not a second template grammar).
Lowering reuses `lower_interpolation`. Static `render_article` never sees
these nodes because hydrate pages are not that path.[^md-ungram][^markdown-rs][^article-rs]

## Consistency with the stack

| Layer | Owns | Inline interpolation |
| --- | --- | --- |
| Markdown | Sentence structure, emphasis, links | Surrounding text; fences and code stay inert |
| Rocdown | When a hole is recognized | Mode switch (`@{`) |
| Rocci | `{expr}` grammar and `Str` rule | Payload inside the braces |
| Roc | The expression, type `Str` | Evaluation on hydrate/live |
| HTML | Escaped text node | `Html.text` |
| Datastar | Morph transport | Out of scope; this is not a signal |

A live count in a paragraph is still a **server-rendered `Str` hole**, not
a client store. If the value changes per request, the page is already
hydrate/live; the hole does not create a new runtime.

## Non-goals

- Inline `<Component />` or JSX in Markdown (MDX).
- Executing fences or code spans.
- Roc calls inside `:kind` params in v1.
- A Rust-evaluated subset sharing the Markdown hole spelling.
- Changing Rocci `{expr}` in templates.
- Treating wrapping Handlebars as a Rocdown language constraint.

## Disposition

The Markdown hole is **`@{expr}`**. Rocci keeps `{expr}`. `{@expr}` fails
full expressions; `{{expr}}` shares `:kind` bodies and needs `}}}` on Roc
blocks. Implementation: [Rocdown Markdown `@{expr}` interpolation](../plans/rocdown-inline-interpolation.md).
Not shipped. No phase started.[^impl-plan]

[^rocdown-readme]: File shape, line-start recognition, reserved names, HTML islands, bare `{expr}` as prose, `\@` escape, fences, `@if` bodies, `@render` prefix calls, `{{ }}` block bodies.
[^scanner]: Document-root scan of `@` reserved names, `:kind`, and `<Tag>`; a line-start `\@` is not a declaration.
[^markdown-rs]: Comrak projects fences and indented code to `CodeBlock`, inline code to `Code`, and prose to `Text`. No interpolation node exists.
[^lowerer]: Markdown text lowers through `Html.text` of a string literal, not a Roc expr.
[^article-rs]: `classify_document` promotes `@roc` / `@render` / template items to hydrate; static article HTML is Rust.
[^rocci-parser]: `{` opens interpolation with balanced depth, string, and comment skipping; `@@` is literal `@`.
[^template-readme]: `{expr}` in text/attributes; Datastar `@post` and `${…}` inside some action URLs; `@@` escape.
[^rocci-lower]: Non-body-param interpolations emit `Html.text(expr)` with `OriginKind::TextExpression`.
[^format-arch]: Document-root HTML islands versus disabled Markdown raw HTML.
[^markdown-first]: Markdown-first; transitions only at reserved document-root declarations or root HTML islands; inline dynamic prose may need a component.
[^catalog-shell]: Rust owns static article HTML; authored dynamic regions stay on the Roc path.
[^format-report]: v1 non-goal: inline Roc expressions in Markdown sentences; MDX-style tags rejected; `@render` block-only; future inline form only after pain.
[^lang-ref]: Public declaration table, `{id}#heading` rewrite example, static/hydrate/live matrix, block-body island errors.
[^text-ref]: Rocci `{expr}` must be `Str`; no markup inside.
[^markup-guide]: Same interpolation contract in the template-layer guide.
[^md-ungram]: Markdown projection has `Text`, `Code`, and `CodeBlock`, not interpolation.
[^html-roc]: Roc `"${name}"` string interpolation in generated Html helpers.
[^pages-guide]: Documented workaround: `@roc` plus `@component` plus `<Byline date={published} />`.
[^why-roc]: Primer shows Roc records and `"${staging}/…"` strings as ordinary prose/code.
[^skip-section]: `skip_article_section` counts `{{` / `}}` depth and skips fenced lines while the fence is open.
[^colon-test]: `fenced_braces_inside_section_are_opaque` keeps Roc `{ a: 1 }` inside a fenced body from closing `:note {{`.
[^git-upstream]: Operator git helpers resolve `@{upstream}`.
[^datastar-cdn]: jsDelivr URL template uses `datastar@{tag}`.
[^regex-quantifiers]: `{n}`, `{n,}`, and `{n,m}` repeat the preceding atom, so `@{2,}` is two or more `@` characters.
[^directives-ref]: `@if` / `@for` / `@match` / `@let` are markup directives; a Roc `if` / `match` inside `{expr}` is a `Str` expression and must not contain tags.
[^impl-plan]: Phased implementation of `@{expr}` only; `{@` / `{{` are not aliases.
