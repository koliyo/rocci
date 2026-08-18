---
name: rocci-author
description: >-
  Write idiomatic `.rocci` templates, `.rocdown` pages, and ordinary Roc used
  from those files. Use when authoring or reviewing Rocci components, Rocdown
  documents, documentation-site themes, `@page`/`@docs`/`@img` markup, server
  handlers, Datastar actions, fixtures, scoped CSS, or Roc helpers in template
  modules. Prefer match over chained if/else. Do not use for changing Rocci or
  Rocdown grammar, parsers, lowering, or diagnostics — that is rocci-language-dev.
---

# Rocci Author

Write programs and pages **in** Rocci, Rocdown, and Roc. Do not change the
languages themselves; that is `$rocci-language-dev`.

Keep facts in the public references. This skill is the authoring workflow,
gotchas, and style. For match vs if/else, naming, purity, and worked examples,
read [idioms.md](idioms.md) before writing non-trivial control flow.

## Choose the file

| Need | File | Notes |
| --- | --- | --- |
| HTML components, handlers, fixtures | `.rocci` | Roc module plus `@component` / `@on` |
| Markdown-first page or docs | `.rocdown` | Prose is Markdown; `@` only at document root |
| Shared Roc helpers / opaque modules | `.roc` | Import from `.rocci`; no template grammar |
| Site chrome / layouts | theme `.rocci` | Article is a **body parameter**, not a prop |

Static `rocdown build` currently accepts Markdown, `@page`, and `@docs`. Dynamic
`@roc`, `@render`, Rocci components, file `@css`, handlers, and custom layouts
work with `rocci run` / standalone `rocdown run`, and are rejected by the static
site pipeline until island splicing lands.

## Establish context

1. Inspect `git status --short`. Preserve unrelated work.
2. Read the closest existing file of the same kind (`examples/`, `site/theme/`,
   or the page being edited) before inventing a new shape.
3. Look up exact syntax in:
   - `docs/reference/rocci.rocdown` and `crates/rocci-template/README.md`
   - `docs/reference/rocdown.rocdown` and `crates/rocci-rocdown/README.md`
   - `docs/guides/build-a-component.rocdown`, `docs/guides/rocdown-pages.rocdown`,
     `docs/guides/docs-components.rocdown`, `docs/guides/server-actions.rocdown`
4. Treat `test/AllSyntax.rocci` as a compiler fixture, not as a style guide.
   Prefer `examples/` and current Roc nightly snake_case (`List.is_empty`,
   `to_str()`, `split_on`).

## Rocci essentials

- Ordinary Roc stays Roc. Recognized top-level forms: `@component`, `@fixture`,
  `@css`, `@context`, `@init`, `@on`.
- Component names are PascalCase (`StatusCard`). Lowering emits camelCase Roc
  (`statusCard`). Handlers and `exposing` lists use the lowered name.
- One root tag needs no braces. Directives, `@let`, `@css`, or multiple roots
  use `{ ... }`. Consecutive leading capitals (`HTMLShell`) are rejected; write
  `HtmlShell`.
- There is no implicit `children`. Pass nested markup as a second parameter:
  `@component Card = |{ title }, content|` then `{content}`. Putting `Html` in
  the props record wraps it in `Html.text` and escapes it.
- `{expr}` in text must be `Str` (`{count.to_str()}`). A bare body parameter
  already typed as `Html` is inserted as-is.
- Markup is not allowed inside a Roc interpolation. Use `@if` / `@for` /
  `@match` or call a named render function.
- `@if` / `@for` / `@match` produce markup. Ordinary Roc `if` / `match` belong
  in helpers, interpolations, and attributes.
- Parenthesize records in directive headers: `@match ({ status, items }) {`.
  The body `{` stays on the same logical header line.
- Fixtures are ordinary Roc bindings tagged for `rocci view` / `browse`.

## Rocdown essentials

- Reserved `@` names are recognized only at document root, not in lists,
  quotes, or fences. Unknown `@name` stays Markdown. Write `\@roc` for a
  literal example outside a fence.
- Fenced code is always display-only, even when the language is `roc`,
  `rocci`, or `rocdown`.
- Raw HTML in Markdown is disabled. Use Markdown, `@docs`, `@img`, a
  document-root `<Tag />`, or `@render { htmlExpr }`.
- `@page` is at most once. Docs sites usually omit `route` and let the catalog
  derive it. Use `Bool.true` / `Bool.false` in page metadata.
- `@docs <kind> { ... }` is one family (`note`, `steps`, `tabs`, `include`,
  …). Nested `@docs` are legal; `@page` / `@roc` / handlers inside a docs body
  are errors.
- `@img` requires `alt` or `decorative: Bool.true`, not both with a non-empty
  alt. Local `src` is relative to the source file.
- Names `rocci_meta`, `rocci_content`, and `rocci_page` are reserved.

## Server apps

```rocci
@context { db : Sqlite.Db }
@init { ... }
@on:get("/") = |{ db }| { page({ count }) }
@on:post("/actions/save") = |{ db }| { fragment({ count }) }
```

| Kind | Path | Returns |
| --- | --- | --- |
| Document | `/`, `/todos` | Full `<html>` |
| HTML/SSE patch | `/actions/...` | Fragment with a **stable `id`** |
| JSON / data | `/api/...` | Data |
| Long-lived SSE | `/sse` | Authored `main.roc` |

Use unquoted Rocci actions: `data-on:click=@post("/actions/x")` with Roc
double-quoted strings. A quoted `"@post('/x')"` is opaque Datastar JS. Prefer
file-level `@css` on page modules or shared CSS for patch components; injected
sibling `<style>` should not ride on SSE patches.

## Roc used from Rocci

- Helpers and fields: `snake_case`. Types and tags: `PascalCase`.
- Effectful functions end in `!`. Pure helpers do not. `?` unwraps `Result`.
- Model absence with tags (`[Some a, None]`, `[Ok a, Err e]`), not null.
- Prefer `match` over chained `if` / `else if` whenever the input is a tag
  union, a small closed set of strings, or several discrete cases. Use `if` /
  `@if` only for a true/false condition (empty list, missing string, flag).
- Follow the pinned Roc nightly and nearby `.roc` / `.rocci` files for stdlib
  names. Do not invent JS, Python, or old-Roc camelCase (`isEmpty`, `toStr`).

## Author the change

1. Match the surrounding module: imports, `@css` placement, handler names,
   fixture style.
2. Put types and pure helpers in ordinary Roc. Keep `@component` bodies as
   markup plus structural directives.
3. Reach for `@match` / `match` before adding another `@else if` / `else if`.
4. Colocate isolated CSS. Authors keep writing `class="card"`; lowering scopes
   it. Document chrome belongs on `body` or `:scope`, not `html { ... }`.
5. Add `@fixture` data for new components that `rocci view` should preview.

## Validate

Inspect before running:

```sh
cargo run -q -p rocci-cli -- inspect --ast path/to/File.rocci
cargo run -q -p rocci-rocdown-cli -- inspect ast path/to/File.rocdown
```

Preview or publish with the matching CLI:

```sh
cargo run -q -p rocci-cli -- view File.rocci --component Name
cargo run -q -p rocci-cli -- run File.rocci
cargo run -q -p rocci-rocdown-cli -- run File.rocdown
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

Do not add compiler-crate tests for ordinary authoring. Failed static builds
must leave the previous output tree in place.

## Report

- Name the files authored and whether they are app, page, theme, or helper.
- Call out match vs if choices when a tag union or layout discriminator is
  involved.
- Separate static-site-legal Rocdown from interactive islands.
- List the inspect / run / check / build commands actually used.
