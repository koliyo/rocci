# Rocdown: a content format for Rocci and Roc

**Investigation date:** 2026-08-15
**Extension:** `.rocdown`
**Status:** compiler core is implemented in [`crates/rocci-rocdown`](../../crates/rocci-rocdown).
This report is the original language design. Where it disagrees with the crate
README, the README is the source of truth for shipped behavior.

**Shipped:** parse/lower a single `.rocdown` file to Roc; `@page`, `@roc`,
`@render`, and Rocci `@component` / `@fixture` / `@css` / `@context` / `@init` /
`@on`; CommonMark plus GFM tables, strikethrough, task lists, and autolink;
heading IDs; scoped CSS; default HTML shell; synthesized GET; CLI
`build` / `inspect` / `ast` / `run`.

**Not shipped:** SSG and path-derived routes, `@island`, collections, formatter,
LSP, footnotes, near-miss `@` warnings. Those remain design in the sections
below.

## Executive decision

Rocdown should be a Markdown-first sibling of `.rocci`, not a second spelling of
Rocci templates and not an MDX clone.

The recommended model is:

- ordinary document text is Markdown;
- executable language regions are explicit, line-start `@...` declarations;
- every Rocdown keyword begins with `@`, matching Rocci;
- `@` anywhere else is ordinary text;
- `@roc { ... }` contains ordinary Roc module declarations;
- `@component`, `@fixture`, `@css`, `@context`, `@init`, and `@on` retain their
  Rocci meaning and grammar;
- `@render { ... }` inserts one Roc expression whose result is `Html`;
- fenced code is always displayed, never executed;
- a page is static by default, with JavaScript emitted only for explicitly used
  client islands;
- server handlers remain explicit for request-time behavior.

This gives Rocdown a small, predictable boundary: Markdown owns content, Rocci
owns server-rendered components, Roc owns data and computation, and an island
controller owns only browser-specific behavior.

Here, “keyword” means a Rocdown-owned directive that changes the document
grammar. Roc identifiers and record fields inside an embedded Roc region,
Markdown markers, HTML names inside a Rocci component, and CSS syntax retain
their native spelling.

The most important syntax choice is recognizing declarations **only at the
start of a document-level line, after optional horizontal whitespace, and
outside literal Markdown blocks**. This makes all of these plain text:

````markdown
Email help@example.com.
Follow @roclang.
Use `@component` in a Rocci file.

```roc
@component is only an example here
```
````

Only an actual top-level declaration switches language mode:

```rocdown
@roc {
# Roc declarations go here.
}
```

## 1. What exists in Rocci today

The current repository already establishes most of the reusable substrate:

- `rocci-template` scans only top-level Rocci declarations and otherwise keeps
  Roc opaque.
- Implemented top-level declarations are `@component`, `@fixture`, `@css`,
  `@context`, `@init`, and `@on`.
- Component bodies use a bounded HTML grammar with Roc interpolation and the
  `@if`, `@for`, `@match`, `@let`, and component-local `@css` directives.
- Lowering produces ordinary Roc, source-map segments, component/fixture
  metadata, extracted CSS, state metadata, initialization metadata, and route
  metadata.
- The LSP already reports embedded Roc and CSS ranges.
- The CLI already knows how to compile `.rocci`, generate a standalone HTTP
  dispatcher, serve assets, render fixtures, and package the result.

That architecture should remain intact. Rocdown needs a new content front end,
but it should lower into the same `Html`, style, route, runtime, source-map, and
island artifact model.

The previous Bravo prototype demonstrated three useful ideas:

1. prose should dominate the file;
2. executable code and rendered values should be insertable between prose
   blocks;
3. a document can introduce domain values and custom views locally.

It also mixed display fences, executable fences, view decorators, `$` strings,
and `$Value` rendering into a syntax that was tied to the old language model.
Rocdown can keep the workflow while removing the ambiguity.

## 2. Goals and non-goals

### Goals

1. Make articles, documentation, blogs, guides, changelogs, and content-heavy
   product pages pleasant to author.
2. Preserve familiar Markdown behavior in ordinary prose.
3. Allow colocated Roc declarations and Rocci components without treating
   braces or angle brackets in prose as executable syntax.
4. Produce zero client JavaScript unless a referenced island needs it.
5. Support static generation first and explicit server behavior second.
6. Reuse Rocci component syntax, CSS scoping, diagnostics, and generated Roc.
7. Preserve accurate source locations across Markdown, Roc, Rocci, and CSS.
8. Permit future content collections and typed metadata without baking YAML or
   JavaScript semantics into the format.

### Non-goals for the first version

- Inline Roc expressions inside Markdown sentences.
- JSX/MDX-style component tags in arbitrary Markdown inline positions.
- Executing fenced code blocks.
- A second client-side Roc dialect.
- Implicit component state, hooks, or whole-page hydration.
- Nested Rocdown declarations inside block quotes or list items.
- A general macro/plugin grammar.
- Automatic conversion of arbitrary Markdown into component children.

These omissions keep prose portable and the parser recoverable. They can be
revisited after real documents reveal a repeated need.

## 3. Proposed source form

````rocdown
@page {
    route: "/guides/rocdown/",
    layout: Docs.article,
    draft: Bool.false,
    meta: {
        title: "Rocdown",
        description: "Markdown content with explicit Roc and Rocci islands",
    },
}

@roc {
import Html
import Docs

published = "2026-08-15"

feature_count = 3
}

@css {
    .feature-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
        gap: 1rem;
    }
}

@component
featureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Rocdown

Rocdown is a content-first format. Email us at docs@example.com or mention
@roclang normally.

@render {
    featureCount({ count: feature_count })
}

## Displayed code

This fence is documentation and is never evaluated:

```roc
answer = 42
```

The page remains static unless it references an island or defines server
routes.
````

The generated module is conceptually equivalent to:

```roc
import Html
import Docs

published = "2026-08-15"
feature_count = 3

featureCount = |{ count }| { ... }

meta = {
    title: "Rocdown",
    description: "Markdown content with explicit Roc and Rocci islands",
}

content = |{}|
    Html.fragment([
        Html.element("h1", [], [Html.text("Rocdown")]),
        Html.element("p", [], [Html.text("Rocdown is ...")]),
        featureCount({ count: feature_count }),
        # More Markdown nodes...
    ])

page = |{}| Docs.article({ meta, content: content({}) })
```

The exact generated names should be collision-resistant internal names rather
than the simple illustrative names above.

## 4. Document grammar

The high-level grammar is deliberately block-oriented:

```text
Document        := (MarkdownBlock | RocdownDeclaration)* EOF

RocdownDeclaration := Indent DeclarationBody
DeclarationBody := PageDecl
                 | RocDecl
                 | ComponentDecl
                 | FixtureDecl
                 | CssDecl
                 | RenderDecl
                 | ContextDecl
                 | InitDecl
                 | OnDecl
                 | IfDecl
                 | ForDecl
                 | MatchDecl
                 | LetDecl
                 | IslandDecl        # once Rocci islands exist

Indent          := (" " | "\t")*

PageDecl        := "@page" RocRecord
RocDecl         := "@roc" RocModuleBlock
RenderDecl      := "@render" RocExpressionBlock

ComponentDecl   := existing Rocci @component grammar
FixtureDecl     := existing Rocci @fixture grammar
CssDecl         := existing Rocci @css grammar
ContextDecl     := existing Rocci @context grammar
InitDecl        := existing Rocci @init grammar
OnDecl          := existing Rocci @on grammar
IfDecl          := existing Rocci @if template directive
ForDecl         := existing Rocci @for template directive
MatchDecl       := existing Rocci @match template directive
LetDecl         := existing Rocci @let template directive
```

`RocRecord`, `RocModuleBlock`, and `RocExpressionBlock` all use braces at the
surface, but have different semantic categories:

- `@page { ... }` is one Roc record expression;
- `@roc { ... }` contains zero or more Roc module declarations and strips the
  outer braces during lowering;
- `@render { ... }` contains one Roc expression returning `Html`.

Roc-aware scanners must ignore braces inside Roc strings, multiline strings,
comments, and nested delimiters. CSS blocks use the existing CSS-aware scanner.
Rocci declarations use the existing Rocci parser.

### 4.1 Declaration recognition

A Rocdown declaration:

1. begins with optional spaces or tabs followed by `@`;
2. uses a reserved Rocdown/Rocci declaration name;
3. occurs at the document root, not inside a list item, block quote, or another
   Markdown container;
4. occurs outside fenced code and raw/literal Markdown blocks;
5. satisfies that declaration's header shape.

An `@` in a paragraph is never special. A reserved declaration written
literally at line start can be escaped, with or without leading whitespace:

```markdown
\@roc { is prose here }
```

The Markdown escape is removed in rendered text. Unknown `@name` lines remain
Markdown so handles and prose are not captured. The LSP should warn for a close
miss such as `@componnent {`, but the compiler should not reserve every possible
word beginning with `@`.

For example, this is a real declaration equivalent to an unindented one:

```rocdown
    @roc {
    answer = 42
    }
```

For a recognized reserved declaration at the document root, directive parsing
takes precedence over CommonMark's indented-code interpretation. Therefore an
indented literal example beginning with `@roc`, `@render`, or another reserved
declaration must be fenced or escape the `@`. Ordinary indented code remains
Markdown. Indentation alone does not make a declaration part of a surrounding
list or block quote; declarations are still document-root constructs.

The formatter should preserve semantically irrelevant leading whitespace or,
by default, normalize declaration introducers to column zero.

### 4.2 Closing braces

The closing brace belongs to the embedded language block and may be indented.
The formatter should align it with the declaration's `@` by default. Text after
the terminating brace on the same line is an error. A declaration must be
separated from surrounding Markdown by a line boundary; blank lines are
strongly recommended and formatter-enforced.

### 4.3 Unknown and misplaced declarations

- A known declaration may have leading spaces or tabs and remains a real
  declaration.
- A declaration-looking line inside a code fence is literal code.
- A declaration-looking line in an active Markdown list or block quote remains
  content of that container; nested declarations are not supported in v1.
- `@render` inside a list or block quote is not supported in v1.
- `@component`, `@roc`, `@page`, and application declarations are document
  level only.
- Rocci body directives `@if`, `@for`, `@match`, and `@let` are also valid at
  the `.rocdown` document root. Their bodies are Rocci HTML templates, not
  Markdown. `#` in those bodies is a template comment.

## 5. Directive semantics

### 5.1 `@page`

`@page` provides one typed Roc record for metadata and layout configuration:

```rocdown
@page {
    route: "/cars/data-views/",
    layout: Site.article,
    draft: Bool.false,
    meta: {
        title: "Building a car",
        description: "Creating data views",
        tags: ["roc", "views"],
    },
}
```

Recommended rules:

- At most one `@page` per file.
- The whole body must parse/type-check as a record.
- `route`, when present, must be a compile-time string literal. Otherwise the
  route derives from the file path.
- `layout`, when present, must be a statically resolvable Roc value path with
  the effective shape `{ meta, content: Html } -> Html`.
- `draft`, when present, must be a literal boolean so discovery can exclude it
  before invoking the renderer.
- `meta` is an arbitrary Roc record, defaults to `{}`, and is type-checked at
  the layout call. Keeping it nested separates author metadata from build
  controls and lets layouts impose their own structural record type.
- Unknown top-level control fields are errors rather than silently ignored
  metadata.
- Project configuration supplies a default layout. Without either layout, the
  compiler emits a minimal HTML document.

Using a Roc record rather than YAML frontmatter keeps metadata values typed and
avoids introducing another scalar/type system. Only the few build-graph fields
above need restricted syntactic extraction before Roc compilation.

### 5.2 `@roc`

`@roc` contributes ordinary declarations to the generated Roc module:

```rocdown
@roc {
import Catalog

Product : { name : Str, price : Dec }

featured = Catalog.featured
}
```

There may be multiple blocks. Import declarations are collected into the
generated module header while preserving their relative order. All non-import
declarations are concatenated in source order; the compiler should not silently
reorder them. The formatter may gather authored imports into the first `@roc`
block for readability.

`@roc` is trusted executable source. A `.rocdown` file from an untrusted author
must not be built merely because Markdown itself is usually treated as data.
The build platform should expose only the capabilities intentionally provided
to content builds.

### 5.3 `@component`

`@component` is exactly the existing Rocci form:

```rocdown
@component
productCard = |{ product }| {
    <article class="product-card">
        <h2>{product.name}</h2>
        <p>{product.price.to_str()}</p>
    </article>
}
```

Its body is Rocci template syntax, not Markdown. Existing Rocci rules for HTML,
component tags, attributes, interpolation, `@if`, `@for`, `@match`, `@let`, and
component-scoped `@css` apply unchanged.

This strict mode switch is useful: authors always know whether a brace means
Markdown text, a Roc expression, CSS, or a Rocci template directive.

### 5.4 `@render`

`@render` inserts one block-level `Html` value into the Markdown node stream:

```rocdown
Before the card.

@render {
    productCard({ product: featured })
}

After the card.
```

The expression is inserted as a node, not escaped as text. Roc type checking
enforces that it produces `Html`. There is no implicit conversion from an
arbitrary record or number and no reflection-based default view in v1.

This replaces Bravo's `$Wheel { ... }`, `$CustomView(...)`, and view decorator
state with an explicit Roc expression:

```rocdown
@render {
    wheelView({ radius: 0.3, color: "blue" })
}
```

An explicit call is easier to type-check, works across modules, and does not
make rendering depend on the nearest decorator declaration.

`@render` is block-only. Inline dynamic text should initially be rendered by a
small component or by constructing the whole paragraph in `@render`. A future
inline form should be added only if this becomes a demonstrated pain point.

### 5.5 `@css`

Top-level `@css` has Rocci's file-level scoping semantics, adapted to the
generated page root:

- all top-level blocks are concatenated in source order;
- selectors are scoped to the page's stable scope identifier;
- the CSS is emitted once as a build artifact or layout style dependency;
- content hashes deduplicate identical output;
- it is not repeated at every `@render` insertion.

Component-local `@css` remains inside the component body and uses the current
component scope behavior.

For SSG, extracted and fingerprinted CSS is preferable to injecting a `<style>`
element into the content fragment. Development may use a stable asset URL.

### 5.6 `@fixture`

`@fixture` remains a Rocci sample binding. It supports previews and tests for
components declared in a Rocdown file but does not render into the article.

### 5.7 `@context`, `@init`, and `@on`

These keep their current standalone Rocci semantics:

```rocdown
@context { db : Sqlite.Db }

@init {
    { db: open_db!()? }
}

@on:post("/api/reactions") = |{ db }| {
    reactions = add_reaction!(db)?
    reactionCount({ count: reactions })
}
```

They do not make the Markdown body request-dependent by implication:

- `@page.route` is the output route for static discovery;
- an `@on:get` route is an explicit runtime handler;
- handlers call generated page/component functions just as they call ordinary
  Rocci views;
- `@init` still requires `@context`;
- a purely static build should reject runtime declarations unless a server
  target is configured.

This preserves Rocci's useful rule that server effects and request boundaries
remain visible.

### 5.8 `@island` (dependent on Rocci island work)

Rocdown should not invent a separate island system. Once Rocci has the proposed
typed island declarations, the same top-level `@island` declaration can be
available in both formats, and `@render` can call its generated Roc host:

```rocdown
@island copyButton = "rocci-copy-button" {
    @props {
        text: String = ""
    }
    @client module("./CopyButton.client.js")
}

@render {
    copyButton({ text: install_command })
}
```

The precise `@island` grammar is not stabilized by this report. The important
contract is:

- the host is rendered to HTML by Roc;
- props use declared encoders;
- client code is an explicit external module initially;
- generated JavaScript is included only when the page references the island;
- server-rendered children and island-owned private DOM never compete for the
  same subtree;
- cleanup and Datastar morph behavior follow the Rocci island contract.

An island is not a synonym for `@render`: most rendered Roc components need no
browser JavaScript at all.

## 6. Markdown profile

Rocdown should name and test a specific profile rather than promise an
unspecified “Markdown.” The recommended v1 base is CommonMark 0.31.2 plus a
small, explicit extension set.

### Core CommonMark

- ATX and Setext headings
- paragraphs and soft/hard breaks
- emphasis and strong emphasis
- ordered and unordered lists
- block quotes
- thematic breaks
- inline code and fenced/indented code blocks
- links, reference links, images, and autolinks
- backslash escapes and character references

### Enabled GFM-style extensions

- tables
- strikethrough
- task lists
- extended URL autolinks

### Recommended Rocdown additions

- deterministic heading IDs with duplicate disambiguation;
- syntax-highlight class names from fenced-code info strings;
- optional footnotes, because technical documentation commonly needs them.

### Deferred extensions

- wiki links such as Bravo's `[[ Integration ]]`;
- directives/admonitions inside Markdown;
- definition lists;
- math syntax;
- automatic table of contents tokens;
- Markdown embedded inside arbitrary HTML.

These can be feature flags or project options later. They should not silently
change the base parser.

### Raw HTML

Raw HTML should be disabled by default and reported with a targeted diagnostic:

```text
raw HTML is disabled in Rocdown; use Markdown or @render { ... }
```

Reasons:

- lowering Markdown nodes through `Html` constructors escapes text safely;
- raw HTML would bypass Rocci's element and attribute handling;
- PascalCase HTML-like tags would invite accidental MDX expectations;
- script/style blocks introduce CSP and injection concerns.

A trusted-project `markdown.raw_html = true` option may preserve CommonMark raw
HTML through `Html.dangerously_include_unescaped_html`, with the risk explicit
in configuration. It must never turn `<Component />` into a Roc call.

### Code fences

Fences are always literal display content. In particular:

````rocdown
```roc
@roc {
    this_is_displayed = Bool.true
}
```
````

does not execute. This is a major improvement over a format where the sigil on
a fence changes whether code is evaluated.

## 7. Layout and generated page contract

A Rocdown document exports three conceptual values:

```text
meta    : record supplied by @page
content : {} -> Html
page    : {} -> Html
```

The layout receives:

```roc
{ meta, content }
```

where `content` is already one `Html` fragment. Layouts own the document shell,
canonical metadata, navigation, asset links, and shared client entry point.
The content compiler owns article nodes and page-scoped CSS.

There should be no implicit global variable containing request state. A dynamic
page can define an ordinary Roc function in `@roc`, use a Rocci component, and
call it from an explicit handler. Static pages remain pure build inputs.

## 8. Static generation and routing

### Route convention

Without `@page.route`, source paths map predictably:

| Source | Public route | Output file |
| --- | --- | --- |
| `content/index.rocdown` | `/` | `dist/index.html` |
| `content/about.rocdown` | `/about/` | `dist/about/index.html` |
| `content/guides/index.rocdown` | `/guides/` | `dist/guides/index.html` |
| `content/guides/rocdown.rocdown` | `/guides/rocdown/` | `dist/guides/rocdown/index.html` |

Projects may select a flat `.html` policy, but one policy must apply to the
whole content root. Explicit routes must be absolute URL paths, must not contain
`..`, query strings, fragments, encoded path separators, or NULs, and must be
collision-checked after normalization.

### Build graph

The SSG build should:

1. discover `.rocdown` files under configured content roots;
2. parse declarations and Markdown into a source-spanned AST;
3. extract route, draft, layout, imports, style, and island dependencies;
4. lower each document to a generated Roc type module;
5. generate a registry of pages and metadata;
6. compile one build renderer rather than invoking Roc separately per page;
7. render pages and write them atomically into a staging output directory;
8. fingerprint CSS, island modules, images, and other copied assets;
9. verify internal links and route collisions;
10. replace the destination only after a successful build.

One build renderer enables shared module compilation and future collection
queries. It also prevents O(number of pages) Roc compiler startup overhead.

### Development server

The development server should cache parse/lower results per document, rebuild
the affected page and reverse dependencies, use stable development asset URLs,
and route generated diagnostics back to `.rocdown` spans. A full browser reload
is sufficient initially; preserving island state can come later.

### Hybrid output

“Hybrid” covers two independent axes:

| Axis | Static choice | Dynamic choice |
| --- | --- | --- |
| Page rendering | Generated at build time | Explicit `@on:get`/server route |
| Browser behavior | No JavaScript | Explicit referenced island module |

A static page can contain client islands. A server-rendered page can contain no
client JavaScript. The compiler should not collapse these into one `dynamic`
flag.

Datastar interactions continue to use explicit HTTP/SSE handlers. A static
shell with a Datastar or custom-element island is therefore a normal and useful
deployment shape.

## 9. Compiler architecture

Add a sibling crate rather than expanding `rocci-template` into a Markdown
parser:

```text
rocci-rocdown
  parse Markdown + line-start declarations
  validate page/content rules
  lower Markdown AST to Html constructors
  delegate Rocci declarations to shared rocci-template machinery
  emit generated Roc + source maps + page metadata + artifacts
```

The current private parser types will need a small refactor so `.rocci` and
`.rocdown` share declaration parsing without copying scanners.

Recommended shared boundary:

```text
rocci-syntax (module or crate)
  Roc-aware cursor and balanced scanning
  Rocci declaration AST and parsing
  component template AST and parsing
  declaration validation/lowering helpers

rocci-template
  .rocci outer document mode

rocci-rocdown
  .rocdown outer document mode + Markdown lowering
```

### Parsing order

CommonMark determines block structure before inline structure. Rocdown should
respect that model:

1. a block scanner identifies literal Markdown regions and valid document-root,
   line-start Rocdown declarations after optional horizontal whitespace;
2. declarations become placeholder/custom nodes with exact source spans;
3. the remaining document is parsed as one Markdown document, preserving link
   reference definitions across declaration boundaries;
4. inline Markdown parsing occurs only in Markdown nodes;
5. language-specific parsers fill each custom node.

Do not split the document into unrelated Markdown strings and concatenate the
rendered HTML. That can change reference-link resolution, list looseness, and
other document-wide semantics.

For a Rust implementation, Comrak is a practical first candidate because it
provides a CommonMark/GFM AST, source positions, formatter support, and the
needed extensions. Its current release advertises CommonMark 0.31.2 compliance.
Pin the selected parser version and run the CommonMark conformance suite plus
Rocdown interaction tests. An event-only parser is attractive for speed but
less convenient for custom nodes, link analysis, heading indexes, and source
maps.

### Output artifact set

Rocdown should return an artifact set, not only a string of Roc:

```text
RocdownCompileOutput {
    document
    roc
    source_map
    diagnostics
    page_meta
    components
    fixtures
    styles
    routes
    islands
    assets
    links
    headings
}
```

This is also the right long-term output shape for Rocci island work.

## 10. Source maps, diagnostics, and editor support

Every generated segment should retain both a source span and an origin kind:

- Markdown structure
- Markdown text
- generated Markdown boilerplate
- `@page` Roc
- `@roc` Roc
- `@render` Roc
- Rocci component signature/template/interpolation
- CSS
- island client module reference

The LSP should expose embedded ranges for Markdown, Roc, Rocci template regions,
CSS, and eventually JavaScript. It should offer:

- declaration completion at document-root line starts, including after leading
  spaces or tabs;
- Markdown completion/preview elsewhere;
- Roc diagnostics and navigation inside `@page`, `@roc`, `@render`, and Rocci
  expressions;
- component completion inside Rocci templates;
- heading/link symbols and broken local-anchor diagnostics;
- route/layout hover information;
- go-to-definition from `layout` and external island modules;
- a rendered preview whose errors map back to source.

High-value targeted diagnostics include:

- reserved declaration escaped accidentally or placed inside a Markdown
  container where it cannot execute;
- unterminated embedded block;
- multiple `@page` blocks;
- `@render` expression does not produce `Html`;
- runtime declarations used in a static-only target;
- duplicate normalized route;
- raw HTML disabled;
- `@component` or `@roc` found inside a fence (informational only when the
  author appears to expect execution);
- a UTF-8 BOM occurs anywhere except the start of the file; one initial BOM is
  ignored before line-start recognition.

The formatter should own the mixed-language boundaries rather than running a
generic Markdown, Roc, or CSS formatter over the whole file.

## 11. Security and correctness boundaries

1. **Executable content:** `@roc`, `@render`, Rocci components, and handlers are
   code. Treat Rocdown repositories as trusted source, not user-submitted
   Markdown.
2. **HTML escaping:** lower Markdown text and attributes through safe `Html`
   constructors. Keep raw HTML opt-in and visibly dangerous.
3. **URL handling:** reject unsafe output routes and define a policy for link
   schemes. Do not rewrite `javascript:` links into active anchors.
4. **Client code:** emit only self-hosted explicit modules under the configured
   CSP. Do not infer hydration from ordinary components.
5. **DOM ownership:** Datastar/server morphing and an island controller must not
   mutate the same private subtree.
6. **Build capabilities:** give the content build renderer the smallest Roc
   platform capability surface practical. Network and arbitrary filesystem
   access should not appear merely because a document is being rendered.
7. **Output writes:** stage output, validate it, then replace atomically. A
   failed build must not leave a partially updated site.
8. **Determinism:** static builds should have explicit inputs for clock, random,
   environment, and remote data. Reproducible pages improve caching and review.

## 12. Bravo-to-Rocdown migration

| Bravo prototype | Rocdown | Reason |
| --- | --- | --- |
| `@@h1` | `# Heading` | Use familiar Markdown headings |
| `$``` ... ```` | `@roc { ... }` | Execution is explicit and cannot be confused with display fences |
| Ordinary fenced code | `````lang` | Always display-only |
| `$"text"` | Markdown paragraph text | Prose is the default language |
| `$Wheel { ... }` | `@render { wheelView(...) }` | Explicit Roc call and `Html` result |
| `$CustomWheelView(value)` | `@render { customWheelView(value) }` | Same call model as ordinary Roc |
| `@@CustomWheelView2` decorator | Explicit view function call | Avoid hidden ambient rendering state |
| `[[ Integration ]]` | `[Integration](...)` initially | Standard Markdown links; wiki links can be an opt-in extension |

Reflection-based default views should be a Roc library facility, if desired,
not a special Rocdown evaluation rule. For example, `@render {
Inspect.view(wheel) }` is explicit and remains type-checked.

## 13. Alternatives considered

### MDX-style inline expressions and component tags

MDX shows the power of combining Markdown, JSX, expressions, and imports. It
also makes ordinary braces and angle-bracket constructs language-sensitive and
requires a much deeper composite parser. Rocdown does not need that cost to
serve its initial block-oriented use cases.

**Decision:** do not adopt inline Roc or JSX-like calls in v1.

### YAML/TOML frontmatter

Frontmatter is familiar and easy for generic SSG tools, but it introduces a
separate value/type model and breaks the “all declarations begin with `@`”
rule.

**Decision:** use a typed `@page` Roc record. Consider import/export adapters
for external CMS data later.

### Executable fenced code

It is concise and notebook-like, but visual differences between executable and
display-only examples become too small, and containing longer nested fences is
awkward.

**Decision:** fences display; `@roc` executes.

### Treat all unknown `@name` lines as errors

That catches directive typos but makes handles and natural prose at the start
of a paragraph fragile.

**Decision:** reserve known names only and add near-miss editor warnings.

### Parse the whole file as Rocci with Markdown sugar

This would inherit Rocci's structural meaning for `<`, `{`, and `@` in text,
exactly where a content format should be forgiving.

**Decision:** Markdown is the outer grammar; Rocci is an explicit declaration
subgrammar.

## 14. Implementation plan

### Stage 0: executable design spike

- Add a small `rocci-rocdown` crate.
- Parse `@roc` and `@render` only, plus headings, paragraphs, links, lists, and
  fences.
- Lower to a generated Roc module and render one static page.
- Prove source mapping for one Markdown error and one Roc error.
- Test `@` in email, handles, inline code, fenced code, ordinary indented code,
  escaped declaration examples, and real declarations with varied indentation.

### Stage 1: useful static MVP

- Add `@page`, route discovery, layouts, `@component`, `@fixture`, and `@css`.
- Enable the documented Markdown/GFM profile.
- Generate one multi-page build renderer and atomic output.
- Add heading IDs, internal link validation, assets, draft handling, and
  incremental development rebuilds.
- Add `.rocdown` LSP registration, embedded ranges, symbols, and preview.

### Stage 2: server/hybrid integration

- Enable `@context`, `@init`, and `@on` through the existing dispatcher model.
- Distinguish static page output, server routes, and client island assets in
  manifests and build commands.
- Add deployment validation for static-only versus server targets.

### Stage 3: Rocci-native islands

- Reuse the island declaration/artifact work proposed for `.rocci`.
- Emit only referenced client modules and one deterministic entry module.
- Add prop/event diagnostics and lifecycle/morph tests.
- Verify a static documentation page containing multiple independently loaded
  islands.

### Stage 4: content collections, only after the page model is stable

- Typed collection schemas and metadata indexes.
- Pagination, feeds, sitemaps, and collection queries.
- Remote content adapters with explicit caching and reproducibility rules.
- Optional footnotes/wiki links/admonitions based on demonstrated needs.

## 15. Acceptance tests for v1

At minimum, the conformance suite should prove:

1. Every supported CommonMark example has the selected parser's expected AST.
2. `@` in email, handles, normal paragraphs, links, code spans, code fences,
   ordinary indented code, HTML-disabled regions, and escaped line-start text is
   literal.
3. Each real declaration is recognized at the document root after zero or more
   leading spaces or tabs, but not inside lists or block quotes.
4. Braces in Roc strings, multiline strings, comments, records, and nested
   expressions do not terminate `@roc` or `@render` early.
5. CSS strings, comments, nested rules, and `@media` work inside `@css`.
6. Link references resolve across intervening Rocdown declarations.
7. Markdown before and after `@render` keeps correct paragraph/list boundaries.
8. Generated Roc errors map to the correct embedded source span.
9. Route normalization detects collisions and traversal attempts.
10. Raw HTML is rejected by default and preserved only under explicit trusted
    configuration.
11. A page with no island emits no client JavaScript.
12. Two uses of one island emit one client module registration.
13. Static output is byte-stable when inputs are unchanged.
14. A failed multi-page build leaves the previous output intact.

## 16. Recommendation

Proceed with `.rocdown` as a distinct Markdown-first front end using the
following minimal language core:

```text
@page       typed metadata and layout
@roc        ordinary Roc module declarations
@component  existing Rocci server component
@fixture    existing Rocci sample binding
@css        existing Rocci scoped CSS
@render     block-level Roc Html insertion
@if @for @match @let  Rocci template constructs at document root
@context    existing Rocci server state type
@init       existing Rocci initialization
@on         existing Rocci HTTP handler
@island     shared with Rocci when that design is implemented
```

Keep Markdown literal by default, keep execution visibly block-delimited, and
keep islands opt-in. This is smaller than MDX, more content-friendly than
embedding Markdown inside Rocci templates, and closely aligned with Rocci's
current bounded-parser and generated-Roc architecture.

## References

Local design and implementation evidence:

- [`README.md`](../../README.md)
- [`crates/rocci-template/README.md`](../../crates/rocci-template/README.md)
- [`ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md`](ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md)
- [`ROCCI_SYNTAX_WEAK_POINTS_REPORT.md`](ROCCI_SYNTAX_WEAK_POINTS_REPORT.md)
- [`DATASTAR_ROCKET_IN_ROCCI_REPORT.md`](DATASTAR_ROCKET_IN_ROCCI_REPORT.md)
- [`crates/rocci-template/src/parser.rs`](../../crates/rocci-template/src/parser.rs)
- [`crates/rocci-template/src/lower.rs`](../../crates/rocci-template/src/lower.rs)
- [`crates/rocci-lsp/src/tokens.rs`](../../crates/rocci-lsp/src/tokens.rs)

External primary references:

- [CommonMark 0.31.2 specification](https://spec.commonmark.org/0.31.2/)
- [MDX documentation](https://mdxjs.com/docs/)
- [Astro islands documentation](https://docs.astro.build/en/concepts/islands/)
- [Astro client and server directives](https://docs.astro.build/en/reference/directives-reference/)
- [Astro content collections API](https://docs.astro.build/en/reference/modules/astro-content/)
- [Comrak repository and documentation](https://github.com/kivikakk/comrak)
