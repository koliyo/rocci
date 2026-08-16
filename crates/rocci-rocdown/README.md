# rocci-rocdown

Parse `.rocdown` documents and lower Markdown plus explicit `@` declarations to
ordinary Roc.

Rocdown is a Markdown-first sibling of `.rocci`. Ordinary document text is
Markdown. Executable regions are line-start `@` declarations. Fenced code is
always displayed and never executed. The page is static unless it defines
`@on` routes.

This crate does not invoke the Roc compiler, type-check expressions, or spawn
servers. `rocci build` / `inspect` / `ast` / `run` on the workspace CLI accept a
`.rocdown` file.

```sh
cargo run -p rocci-cli -- build examples/rocdown/Guide.rocdown
cargo run -p rocci-cli -- inspect --ast examples/rocdown/Guide.rocdown
cargo run -p rocci-cli -- run examples/rocdown/Guide.rocdown
```

Library entry points are `parse`, `compile`, and `format_ast` in
`rocci_rocdown`.

The original language design lives in
[`ROCDOWN_FORMAT_REPORT.md`](../../ROCDOWN_FORMAT_REPORT.md). This README is
what the compiler actually does.

## File shape

A file is a sequence of Markdown blocks, `@` declarations, and document-root
HTML islands:

```text
Document            := (MarkdownBlock | RocdownDeclaration | HtmlIsland)* EOF
RocdownDeclaration  := Indent "@" Reserved ...
HtmlIsland          := Indent "<" Tag ...
Indent              := (" " | "\t")*
```

Reserved names: `page`, `roc`, `render`, `component`, `fixture`, `css`,
`context`, `init`, `on`, `if`, `for`, `match`, `let`. Unknown `@name` stays
Markdown. `\@roc` is escaped prose; the backslash is dropped in rendered text.

Declarations are recognized only when all of these hold:

1. the line is at the document root (not inside a list, block quote, or fence);
2. after optional spaces or tabs it starts with `@` plus a reserved name;
3. the rest of the header matches that declaration's shape.

HTML islands use the same document-root line-start rule, but start with `<`
plus a tag name (or `<>`). `<http:...>`, `<!--`, and `<!DOCTYPE` stay Markdown.

`@` in a paragraph, email, handle, or inline code is never special. Indented
`@roc {` at document root is a real declaration; CommonMark indented-code does
not win for reserved names. Put a literal example in a fence or write `\@roc`.

Text after a declaration's closing `}` on the same line is an error.

````rocdown
@page {
    route: "/guides/rocdown/",
    draft: Bool.false,
    theme: "paper",
    color_scheme: "auto",
    meta: {
        title: "Rocdown",
        description: "Markdown content with explicit Roc and Rocci islands",
    },
}

@roc {
published = "2026-08-15"
feature_count = 3.I64
}

@css {
    body { font-family: system-ui, sans-serif; }
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Rocdown

Email docs@example.com or mention @roclang normally.

<FeatureCount count={feature_count} />

See also [[Interactive]] or [Interactive](Interactive.rocdown).

```roc
answer = 42
```
````

See [`examples/rocdown/Guide.rocdown`](../../examples/rocdown/Guide.rocdown).

## Declarations

| Form | Body | Meaning |
| --- | --- | --- |
| `@page { ... }` | one Roc record | route, layout, draft, meta, theme, color_scheme |
| `@roc { ... }` | Roc module items | imports, types, values; outer braces stripped |
| `@render { ... }` | one Roc expression | spliced as `Html` into the Markdown stream |
| `@component` | Rocci template | same grammar as `.rocci` |
| `@fixture` | Roc binding | preview/test sample; not rendered into the article |
| `@css { ... }` | raw CSS | file-level scoped stylesheet |
| `@context` / `@init` / `@on` | Roc | standalone HTTP, same as `.rocci` |
| `@if` / `@for` / `@match` / `@let` | Rocci template | same constructs as a `@component` body, spliced into the page |
| `<Tag>` / `<Hello />` | Rocci template | document-root HTML island; instantiates elements and components |

`@if`, `@for`, `@match`, and `@let` at document root use Rocci HTML template
bodies, not Markdown. `#` in those bodies is a template comment. `@else` /
`@else if` attach to the preceding `@if` (blank lines in between are fine).
Document-level `@let` bindings are hoisted to the start of `rocci_content`.
Those bodies, and document-root HTML islands, may contain HTML elements,
`<Component />` calls, `{expr}`, and nested control flow. They must not declare
`@component`, `@fixture`, `@page`, `@roc`, `@render`, or other module-level
forms.

A line-start `<Tag>` or `<>...</>` at document root (same list/quote/fence
rules as `@`) is a Rocci HTML island, not CommonMark raw HTML. Use it to
instantiate colocated components next to Markdown. Bare `{expr}` at document
root is still prose; wrap a Roc `Html` value in `@render { ... }`. There is no
`@html { ... }` wrapper.

Inline HTML inside a Markdown paragraph stays disabled raw HTML. See
[`rocci-template`](../rocci-template).

`@component` bodies are the same Rocci HTML grammar: interpolation, `@if`,
`@for`, `@match`, `@let`, and component-local `@css`.

`@island` is reserved in the design and is **not** parsed yet.

### `@page`

At most one per file. Unknown top-level fields are errors. Extracted controls:

| Field | Rule |
| --- | --- |
| `route` | compile-time string literal; no `..`, query, fragment, `%2f`, or NUL |
| `layout` | statically resolvable Roc path, called as `Layout({ meta, content })` |
| `draft` | `Bool.true` or `Bool.false` |
| `theme` | compile-time string; `paper` (default), `rocci`, `none`, a name in `~/.rocci/themes`, or a CSS file path |
| `color_scheme` | `"auto"` (OS light/dark), `"light"`, or `"dark"` |
| `meta` | arbitrary Roc record; `title` is copied onto the default document `<title>` |

Without `layout`, the compiler emits a minimal `<html>` document: charset,
viewport, `color-scheme` meta, title, selected theme CSS, file CSS in `<head>`,
`.rd-document` and `data-rd-theme` on `html`, `data-rd-color-scheme` when
light or dark is forced, `data-rocci-css` on `html` / `body` / `main` when the
file has `@css`, and `rocci_content({})` inside `<main>`. Document `@css`
overrides the theme. Default theme comes from `rocci run --theme`,
`ROCCI_THEME`, then builtin `paper`. `@page.theme` wins for that file.

Without `@page.route`, the synthesized GET path is `/`.

### `@roc`

There may be several blocks. Import lines are collected into the generated
header (and `import Html` is added if missing). Remaining statements are
emitted in source order. Names `rocci_meta`, `rocci_content`, and `rocci_page`
are reserved.

### `@render`

Block-level only. The expression is inserted as a node, not escaped as text.
Roc must type it as `Html`.

## Markdown profile

Parser: [Comrak](https://github.com/kivikakk/comrak) 0.54.

**CommonMark:** ATX/Setext headings, paragraphs, emphasis, strong, lists,
block quotes, thematic breaks, inline and fenced/indented code, links,
reference links, images, autolinks, backslash escapes.

**GFM extensions enabled:** tables, strikethrough, task lists, extended URL
autolinks.

**Rocdown additions:** heading `id` attributes with `-1`, `-2`, … on
duplicates; stable `rd-*` classes on Markdown HTML (`rd-header-1`,
`rd-paragraph`, …); fenced info strings become
`class="rd-code language-…"`; wiki links `[[Foo]]` and `[[Foo|label]]`
(optionally `[[Foo#heading-id]]`).

**Page links:** `[[Foo]]`, `[text](Foo.rocdown)`, `[text](./Foo.rocdown)`,
`[text](Foo)`, and reference links to those destinations resolve to the
target file’s `@page.route` using sibling `.rocdown` files in the same
directory. Same-page `#heading-id` is checked against this file’s heading
ids. Absolute `/path/` destinations are checked against known page routes
when a page index is present. `http(s):`, `mailto:`, and other schemes pass
through. Unknown wiki / `.rocdown` targets are errors. Duplicate
`@page.route` values across siblings are errors.

**Raw HTML** in a Markdown paragraph is an error by default (`raw HTML is
disabled in Rocdown; use Markdown or @render { ... }`). `CompileOptions.raw_html`
preserves that inline/comment HTML through `Html.dangerously_include_unescaped_html`.
It never turns inline tags into Rocci component calls. Document-root `<Hello />`
is an HTML island, not this escape hatch.

**Not parsed yet:** footnotes, admonitions, definition lists, math,
automatic TOC tokens.

## Generated Roc

Every document exports:

```text
rocci_meta    : record from @page.meta, or {}
rocci_content : {} -> Html     # Markdown + HTML islands + @render + @if/@for/@match/@let
rocci_page    : {} -> Html     # layout call, or the default document shell
```

Markdown lowers to `Html.element` / `void_element` / `text` / `fragment`. File
`@css` is wrapped in `@scope ([data-rocci-css~="id"])` using the same file-scope
id as `.rocci`. Component CSS keeps a per-component id.

If the file has no `@on:get` for the page route, lowering synthesizes a GET
handler that returns `rocci_page({})`. When that route is not `/`, GET `/` is
registered to the same handler so `rocci run` can open a preview.

Datastar is imported only when a Rocci region uses a Datastar action.

## CLI

`build`, `inspect`, `ast`, and `run` accept `.rocdown` the same way they accept
`.rocci`. `rocci run --theme paper foo.rocdown` (or `--theme path/to/theme.css`,
or `ROCCI_THEME`) selects the default theme; `@page.theme` overrides it.
`--color-scheme` / `ROCCI_COLOR_SCHEME` force `light`, `dark`, or `auto`.
`rocci run` on a directory compiles sibling `.rocdown` next to `.rocci`.
Preview opens the first GET route that is not `/health`.

`rocci view` / `browse` still target `.rocci` components. The language server
and VS Code / Zed extensions register `.rocdown` next to `.rocci`.

## Implemented vs deferred

**In this crate (compiler core)**

- Scan / parse / lower a single `.rocdown` file to Roc
- Declaration boundary rules (prose `@`, fences, lists, quotes, indent, `\@`)
- `@page`, `@roc`, `@render`, delegated Rocci declarations, document-root
  `@if` / `@for` / `@match` / `@let`, and document-root HTML islands
- CommonMark + GFM tables/strikethrough/task lists/autolink + wiki links
- Sibling page-link resolution (`[[Foo]]`, `.rocdown` Markdown/reference links)
- Heading IDs, scoped CSS, default HTML shell, synthesized GET
- Source-map segments (`MarkdownStructure`, `MarkdownText`, `MarkdownBoilerplate`,
  `PageRoc`, `RocBlock`, `RenderRoc`, plus existing Rocci kinds)
- CLI `build` / `inspect` / `ast` / `run` for one file or sibling modules
- LSP diagnostics, symbols, hover, completion, semantic tokens, and editor
  registration for `.rocdown`

**Not implemented**

- Multi-page SSG, `dist/` output, path-derived routes, draft exclusion, assets
- Project default layouts and layout packages
- `@island` and client JS
- Content collections, feeds, sitemaps
- Formatter
- Footnotes and the other deferred Markdown extensions
- Near-miss warnings for typos such as `@componnent`
