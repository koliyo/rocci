# rocci-rocdown

Parse `.rocdown` documents, compile multi-page static documentation sites, and lower Markdown plus explicit `@` declarations to ordinary Roc.

Rocdown is a Markdown-first document and static documentation system. Ordinary document text is Markdown. Executable regions are line-start `@` declarations. Fenced code is always displayed and never executed.

```sh
# Run a single interactive .rocdown document
cargo run -p rocci-rocdown-cli -- run examples/rocdown/Guide.rocdown

# Build a documentation site to dist/
cargo run -p rocci-rocdown-cli -- build docs --output dist

# Check documentation catalog, routes, links, and includes
cargo run -p rocci-rocdown-cli -- check docs

# Inspect Rocdown AST
cargo run -p rocci-rocdown-cli -- inspect ast test/AllSyntax.rocdown
```

Library entry points are `parse`, `compile`, and `format_ast` in `rocci_rocdown`.

The original language design lives in
[`ROCDOWN_FORMAT_REPORT.md`](../../archive/reports/ROCDOWN_FORMAT_REPORT.md). This README is
what the compiler actually does.

## File shape

A file is a sequence of Markdown blocks, `@` declarations, line-start `:kind`
article blocks, and document-root HTML islands:

```text
Document            := (MarkdownBlock | RocdownDeclaration | ArticleBlock | HtmlIsland)* EOF
RocdownDeclaration  := Indent "@" Reserved ...
ArticleBlock        := Indent ":" Kind [ "[" Params "]" ] Content
HtmlIsland          := Indent "<" Tag ...
Indent              := (" " | "\t")*
```

Reserved names: `page`, `roc`, `render`, `component`, `fixture`, `css`,
`context`, `init`, `on`, `if`, `for`, `match`, `let`, `use`. Unknown `@name` stays
Markdown. Line-start `@docs` / `@img` is a removal error naming `:note` /
`:img[...]`. `\@roc` is escaped prose; the backslash is dropped in rendered text.

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
| `@use "./Module.rocci"` | path string | interactive only: import `@component` exports as article kinds (`Callout` → `:callout`) |
| `:kind[params]` | line, `{{ }}`, or `:end.kind` | article block; kinds are a closed builtin registry, plus `@use` on `rocdown run` |
| `:img[src: "...", alt: "..."]` | params | native image element (`src`, `alt` or `decorative`, `title`, `width`, `height`, `class`, `loading`, `decoding`) |
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

`@island` is reserved in the design and is **not** parsed yet. v1 hybrid sites
use existing `@component` / `@on` hosts; they do not add `@island` grammar.

### `:img`

Native image declaration. Lowers to `Html.void_element("img", ...)` with standard
`.rd-image` class and compile-time field extraction. Explicit sizing is optional.
`alt` is required unless the image is purely decorative.

```rocdown
:img[src: "./img/yammi_banana.png", alt: "A banana", width: "50px"]
```

| Field | Requirement | Rule |
| --- | --- | --- |
| `src` | **Required** | Compile-time string literal path or URL to image |
| `alt` | Required unless `decorative` | Compile-time string literal accessible description |
| `decorative` | Optional | `Bool.true` emits `alt=""`; a non-empty `alt` with this flag is an error |
| `title` | Optional | Compile-time string literal tooltip / title attribute |
| `width` | Optional | Compile-time string literal (e.g. `"50px"`, `"100%"`) |
| `height` | Optional | Compile-time string literal (e.g. `"50px"`, `"auto"`) |
| `class` | Optional | Compile-time string literal appended to `rd-image` |
| `loading` | Optional | Compile-time string literal (`"lazy"` or `"eager"`) |
| `decoding` | Optional | Compile-time string literal (`"async"`, `"auto"`, `"sync"`) |

Markdown `![](path)` remains the empty-alt decorative shorthand. A paragraph
whose only child is a Markdown image, and ATX headings (`#` through `######`),
parse as the same internal `img` / `h1`–`h6` block kinds as `:img` and `:h2`.
Heading ids still come from the existing slug algorithm unless
`:h2[id: "install"]` sets one. Inline images in mixed paragraphs stay Markdown.
Nested `:img`
inside `:figure` owns accessibility text; figure `caption` and `credit`
do not substitute for `alt`. Local `src` paths, including `./img/photo.png`, resolve against the source file directory. `http(s):`, `mailto:`, and `data:` pass through. `rocdown run` diagnoses missing files, copies them into the preview workspace without hashing, and serves them next to the page route. Static site builds (`rocdown build`) hash files under `build.assets`.

### `@page`

At most one per file. Unknown top-level fields are errors. Extracted controls:

| Field | Rule |
| --- | --- |
| `route` | compile-time string literal; no `..`, query, fragment, `%2f`, or NUL |
| `layout` | statically resolvable Roc path, called as `Layout({ meta, content })` |
| `draft` | `Bool.true` or `Bool.false` |
| `theme` | compile-time string; `paper` (default), `rocci`, `none`, a name in `~/.rocci/themes`, or a CSS file path |
| `color_scheme` | `"auto"` (OS light/dark), `"light"`, or `"dark"`. The default `paper` palette is One Light / One Dark Pro. |
| `meta` | arbitrary Roc record; `title` is copied onto the default document `<title>` |

Without `layout`, the compiler emits a minimal `<html>` document: charset,
viewport, `color-scheme` meta, title, selected theme CSS, file CSS in `<head>`,
`.rd-document` and `data-rd-theme` on `html`, `data-rd-color-scheme` when
light or dark is forced, `data-rocci-css` on `html` / `body` / `main` when the
file has `@css`, an automatic left `<nav class="rd-toc">` for heading levels
2–3, and `rocci_content({})` inside `<main>`. The navigator uses the same
heading IDs as in-page `#` links, is omitted when there are no outline
headings, and is hidden on narrow and print viewports by theme chrome. Clicks
scroll the article quickly with a short animation. Theme
`none` skips chrome, including the navigator. A custom `layout` replaces the
default shell entirely. Document `@css` overrides the theme. Default theme
comes from `rocdown run --theme`, `ROCDOWN_THEME` (or `ROCCI_THEME`), then builtin `paper`.
`@page.theme` wins for that file.

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

**Footnotes:** `[^label]` references and `[^label]:` definitions are parsed in
ordinary Rocdown. Definitions are collected out of flow into a footnotes
section with `data-footnote-ref`, `aria-label="Footnotes"`, and
`data-footnote-backref` (`id="fn-{name}"` / `fnref-{name}`). Missing
definitions and duplicate labels are errors. This is not an OKF `sources[].id`
citation.

**Page links:** `[[Foo]]`, `[text](Foo.rocdown)`, `[text](./Foo.rocdown)`,
`[text](Foo)`, `[text](docs/Foo.md)`, and reference links to those destinations
resolve to the target file’s `@page.route` when that file is in the page index.
Standalone `rocdown run FILE` builds that index from sibling `.rocdown` files and
from relative `.rocdown` / `.md` / `.markdown` links, including nested paths.
Same-page `#heading-id` is checked against this file’s heading ids. Absolute
`/path/` destinations are checked against known page routes when a page index is
present; absolute `*.md` / `*.rocdown` paths suffix-match indexed files when
possible and otherwise pass through. `http(s):`, `mailto:`, and other schemes
pass through. Unknown wiki / `.rocdown` targets are errors. Duplicate
`@page.route` values across siblings are errors.

**Raw HTML** in a Markdown paragraph is an error by default (`raw HTML is
disabled in Rocdown; use Markdown or @render { ... }`). `CompileOptions.raw_html`
preserves that inline/comment HTML through `Html.dangerously_include_unescaped_html`.
It never turns inline tags into Rocci component calls. Document-root `<Hello />`
is an HTML island, not this escape hatch.

**Not parsed yet:** admonitions, definition lists, math, automatic TOC tokens.

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
registered to the same handler so `rocdown run` can open a preview.

Datastar is imported only when a Rocci region uses a Datastar action.

## Project themes

A `theme/` directory, or `build.theme` in `rocdown.toml`, of `.rocci` files owns
site chrome and named layouts. Rocdown still compiles builtin `RocdownBase`
(palette tokens and `.article .rd-*` Markdown styles) and `DocsComponents`
unless the project supplies those modules. Each article kind has a named Rocci
component (`Note`, `Tabs`, `Figure`, …). Static apply data is a tagged union of
per-kind props plus fragment paths; widget bodies stay in HTML files and are
passed as the extra content argument. There is no flattened optional-field bag
and no `Render` kind matcher. Article HTML is
an `Html` body
parameter: write `@component Layout = |{ view }, content|` and `{content}` in
the template body. Putting `content` in the props record wraps it in
`Html.text` and escapes the article as text.

## Composable content mounts

Sites can mount external documentation catalogs using `[[mount]]` in `rocdown.toml`.
For example, `site/` mounts `../docs` at prefix `docs` with `layout = "docs"`,
allowing `docs/` to remain at repository root for standalone `rocdown run docs`
while building as part of `rocdown build site`.

## CLI

`rocdown` is the command package for Rocdown documents and static documentation sites. See [`rocci-rocdown-cli`](../rocci-rocdown-cli).

- `rocdown run FILE.rocdown`: Run a single interactive document, including pages it links to. A file under an ancestor `rocdown.toml` previews that site at the page route.
- `rocdown run DIR`: Run/preview a documentation site with live reload. Hybrid sites serve the CDN tree and proxy the generated island service on the same origin.
- `rocdown serve-islands DIR`: Start the island HTTP service for `live` pages (`@on` / Datastar) by itself (CDN-plus-service deploy, or a sibling `[http].service` app).
- `rocdown build DIR`: Build a static documentation site to `dist/`. Hybrid sites emit CDN HTML plus `islands.json` for the service; `--cdn-only` errors on `live` pages.
- `rocdown check DIR`: Check catalog, routes, and links.
- `rocdown test DIR`: Run documented `:example` tests.
- `rocdown inspect ast FILE.rocdown`: Inspect AST.
- `rocdown inspect artifacts DIR`: Inspect the publish report (page kinds, Datastar, service routes, planned files).

## Tree spec

The owned parse-tree shape lives in [`Rocdown.AST.ungram`](Rocdown.AST.ungram).
`cargo run -q -p rocci-ungram -- generate` writes
[`src/ast.generated.rs`](src/ast.generated.rs). The generator emits article and
module node types only; it does not produce the scanner or parser. Markdown
blocks stay hand-written `MdNode`. `cargo run -q -p rocci-ungram -- check` fails
when the committed generated file is stale. This README remains the language
contract; the ungram is the developer tree spec, not a substitute for the syntax
above.

## Implemented vs deferred

**In this crate**

- Scan / parse / lower a single `.rocdown` file to Roc
- Declaration boundary rules (prose `@`, fences, lists, quotes, indent, `\@`)
- `@page`, `@roc`, `@render`, delegated Rocci declarations, document-root
  `@if` / `@for` / `@match` / `@let`, and document-root HTML islands
- CommonMark + GFM tables/strikethrough/task lists/autolink/footnotes + wiki links
- Sibling page-link resolution (`[[Foo]]`, `.rocdown` Markdown/reference links)
  and standalone preview of nested relative `.md` / `.rocdown` document links
- `:kind[params]` article blocks and `:img` alt/decorative contract and `:figure` caption/credit
- `@use "./Module.rocci"` on interactive `rocdown run` (exported `@component` names become article kinds)
- Heading IDs, scoped CSS, default HTML shell with an automatic H2–H3 navigator, synthesized GET
- Source-map segments (`MarkdownStructure`, `MarkdownText`, `MarkdownBoilerplate`,
  `PageRoc`, `RocBlock`, `RenderRoc`, plus existing Rocci kinds)
- Static site generation (`build`, `check`, `test`, `run`), content catalog,
  curated navigation, and hashed asset pipeline
- Site page kinds `static` / `hydrate` / `live` recorded on the catalog and
  `rocdown inspect catalog`. Classification sits on top of `:name[params]`:
  those article blocks keep a page `static` and apply through the widget forest.
  `hydrate` pages splice pure Rocci components into CDN HTML at build time.
  `live` pages splice initial island Html, hash Datastar.js, and loosen
  per-page CSP. Every site page also hashes `goto.js` (Cmd/Ctrl-K fuzzy
  navigation) with `script-src 'self'; connect-src 'self'`. `rocdown serve-islands DIR` compiles colocated `@on` handlers
  into one island HTTP service. Hybrid builds emit `pages.json` kinds,
  `islands.json` service routes, and a publish report. `--cdn-only` refuses
  `live` pages so a CDN publish cannot ship dead actions. `rocdown run DIR`
  previews both artifacts on one local origin and reloads after content or
  handler edits.

**Not implemented / Deferred**

- `@island` and client JS dynamic island splicing
- Project default layouts and layout packages
- Formatter
- Admonitions, definition lists, math, and in-body automatic TOC tokens
- Near-miss warnings for typos such as `@componnent`
- `@use` on static `rocdown build` / `check` (custom static kinds belong in the compiled theme)
- Qualified `@use` names when two modules export the same kind
