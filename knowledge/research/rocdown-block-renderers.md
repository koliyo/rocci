---
type: Research Report
title: Custom Rocdown block schemas and renderers
description: "Exploratory research: treat :kind as a Rocci function interface with named params and constrained children, then let a site-selected renderer paint Html. Syntax is shipped; schema/renderer split and site overrides are not."
tags: [domain/rocdown, domain/rocci, concern/syntax, concern/rendering, concern/architecture, concern/authoring, concern/theming]
status: draft
generated: { by: process:cursor, at: 2026-08-19T17:20:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: registry
    resource: ../../crates/rocci-rocdown/src/registry.rs
    title: Closed v1 article-block kind schema
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rs
    resource: ../../crates/rocci-rocdown/src/docs.rs
    title: Article-block validation, including tabs and steps
    author: process:git
    last_modified: 2026-08-19
  - id: parse-rs
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Colon-call validation against the registry
    author: process:git
    last_modified: 2026-08-19
  - id: imports-rs
    resource: ../../crates/rocci-rocdown/src/imports.rs
    title: "@use component-to-kind mapping"
    author: process:git
    last_modified: 2026-08-19
  - id: config-rs
    resource: ../../crates/rocci-rocdown/src/config.rs
    title: Rocdown site configuration schema
    author: process:git
    last_modified: 2026-08-19
  - id: plan-rs
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: PlannedNode emission and theme module compile
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Standalone article-block lowering
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rocci
    resource: ../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Builtin article-block painters
    author: process:git
    last_modified: 2026-08-19
  - id: build-runtime
    resource: ../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Flattened-forest dispatcher into DocsComponents
    author: process:git
    last_modified: 2026-08-19
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Rocci component calling convention and ?? defaults
    author: process:git
    last_modified: 2026-08-19
  - id: template-ast
    resource: ../../crates/rocci-template/src/ast.rs
    title: Component param-default extraction
    author: process:git
    last_modified: 2026-08-19
  - id: site-ref
    resource: ../../docs/reference/rocdown-site.rocdown
    title: Public Rocdown site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: docs-guide
    resource: ../../docs/guides/docs-components.rocdown
    title: Public documentation-component guide
    author: process:git
    last_modified: 2026-08-19
  - id: all-syntax
    resource: ../../test/AllSyntax.rocdown
    title: Shipped colon-syntax fixture
    author: process:git
    last_modified: 2026-08-19
  - id: block-research
    resource: generalized-rocdown-block-model.md
    title: Generalized Rocdown block model research
    author: process:cursor
    last_modified: 2026-08-19
  - id: block-plan
    resource: ../plans/generalized-rocdown-block-model.md
    title: Generalized Rocdown block model implementation plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-research
    resource: rocci-components-in-generation.md
    title: Rocci components inside the content generation pipeline
    author: process:cursor
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
  - id: theming-arch
    resource: ../architecture/theming.md
    title: Current Rocci theming surfaces
    author: process:okf-phase-4
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
  - id: callout-fixture
    resource: syntax/Callout.rocci
    title: Research-only custom kind example
    author: process:cursor
    last_modified: 2026-08-19
  - id: markdoc-tags
    resource: https://markdoc.dev/docs/tags
    title: Markdoc custom tags, attributes, and children
    author: organization:stripe
  - id: markdoc-render
    resource: https://markdoc.dev/docs/render
    title: Markdoc parse, transform, and render phases
    author: organization:stripe
  - id: mdx-using
    resource: https://mdxjs.com/docs/using-mdx/
    title: MDX components prop and MDXProvider
    author: organization:mdx-js
  - id: nuxt-prose
    resource: https://content.nuxt.com/docs/components/prose
    title: Nuxt Content Prose component overrides
    author: organization:nuxt
  - id: nuxt-mdc-issue
    resource: https://github.com/nuxt-content/mdc/issues/439
    title: MDCRenderer components prop for theme overrides
    author: organization:nuxt-content
  - id: docusaurus-swizzle
    resource: https://docusaurus.io/docs/swizzling
    title: Docusaurus theme component swizzling
    author: organization:facebook
  - id: docusaurus-admonitions
    resource: https://docusaurus.io/docs/markdown-features/admonitions
    title: Docusaurus custom admonition type mapping
    author: organization:facebook
  - id: vitepress-theme
    resource: https://vitepress.dev/guide/extending-default-theme
    title: VitePress theme extension and component aliases
    author: organization:vuejs
  - id: myst-syntax
    resource: https://mystmd.org/guide/syntax-overview
    title: MyST directives as markup functions
    author: organization:executablebooks
  - id: sphinx-extend
    resource: https://www.sphinx-doc.org/en/master/development/tutorials/extending-syntax.html
    title: Sphinx custom roles and directives
    author: organization:sphinx-doc
  - id: mdx-missing
    resource: https://github.com/hashicorp/next-mdx-remote/discussions/470
    title: Missing MDX component fallback discussion
    author: organization:hashicorp
  - id: astro-markdoc
    resource: https://github.com/withastro/roadmap/issues/496
    title: Astro Markdoc undefined-component policy
    author: organization:withastro
  - id: impl-plan
    resource: ../plans/rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers plan
    author: process:cursor
    last_modified: 2026-08-19
---

# Custom Rocdown block schemas and renderers

## Research question

The generalized block *syntax* is in the tree: authors write `:note`,
`:tabs`, `:img`. The remaining product question is how a `:kind` becomes a
typed Rocci function, how a site paints it, and how that paint can change
without rewriting documents.[^rocdown-readme][^all-syntax][^block-research]

Sub-questions:

1. What is the **block interface** — named params, optionality, defaults,
   child list — versus the **block renderer** that returns `Html`?
2. How do we constrain children (`:tabs` accepts only `:tab`) in a way that
   custom kinds can reuse, not only hardcoded `validate_tabs`?
3. Should a Rocci module need a **decorator or export primitive** before a
   component is addressable as `:kind`?
4. Must every kind ship a default renderer, or can a **generic debug
   renderer** stand in?
5. How does **site configuration** override `:note` (and add new kinds) so
   the change is HTML structure, not only CSS?

This is not shipped behavior. Do not treat sketches as language or site
config. The [implementation plan](../plans/rocdown-block-renderers.md) is
exploratory until a human accepts a scope. Architecture records describe
today's format and generator, not this overlay.[^impl-plan][^format-arch]

## For a later agent

- **Authority:** exploratory. Crate READMEs and architecture records describe
  shipped behavior; this record does not.
- **Shipped already:** `:name[params]` article blocks, the closed Rust
  `KindSpec` registry, per-kind `DocsComponents` painters, `@use` on
  `rocdown run`, parent-kind rules for `tab` / `step`.[^registry][^docs-rocci][^imports-rs]
- **Not shipped:** schema/renderer split, site-level renderer override,
  custom static kinds in the theme, structured child lists into Rocci,
  `@block` / export tables, a debug painter.
- **Do not implement** unless the user asks. Follow
  [the plan](../plans/rocdown-block-renderers.md).
- **Keep:** Markdown-first islands, pure `@component` render, Rust catalog /
  Rocci shell, OKF Markdown-only, `rocci-template` grammar unchanged unless a
  phase explicitly adds `@block`.[^markdown-first][^pure-render][^catalog-shell]
- **Owning crates if implemented later:** `rocci-rocdown` (registry, plan,
  theme compile, site config, catalog diagnostics), `rocci-template` only if
  a decorator is added, public `docs/reference/rocdown.rocdown` and
  `docs/reference/rocdown-site.rocdown`.
- **Skills:** `rocci-language-dev` for any grammar; `rocci-author` for theme
  block modules; `manage-rocci-knowledge` for this record.

## Scope and authority

The [block-model research](generalized-rocdown-block-model.md) decided
source spelling and a uniform `BlockCall` tree. That spelling is now the
public Rocdown contract. This record starts *after* parse: how a `BlockCall`
is a function interface, who implements it, and who may replace the
implementation per site.[^block-research][^block-plan][^rocdown-readme][^docs-guide]

Human authoring DX still matters, but the acceptance test here is
**themeability of structure**: a site must be able to paint `:note` as an
`<aside>`, a `<section>`, or a disclosure widget without forking every
document. CSS variables on `.rd-docs-note` are not that capability.[^docs-rocci][^theming-arch]

## Current contract

### Syntax and registry

A line-start `:kind` is an article `BlockCall`. Params are a bracket record.
Content is line-scope, `{{ }}`, or `:end.kind`. Kinds are data in
`registry.rs`, not parser keywords. Unknown `:foo` is a diagnostic. `@use
"./Callout.rocci"` on interactive `rocdown run` maps each exported
`@component Callout` to `:callout` by PascalCase-to-kebab. Builtin name
collision is an error, so `@use` cannot override `:note`. Static `rocdown
build` / `check` still reject `@use`.[^rocdown-readme][^registry][^imports-rs][^parse-rs]

`KindSpec` already stores most of an interface:

| Field | Role today |
| --- | --- |
| `name` / `component` | Source kind vs Rocci painter name (`note` / `Note`) |
| `required_fields` / `optional_fields` | Authoring params |
| `required_one_of` | `link-card` `page` or `href` |
| `parents` | `tab` only inside `tabs`; `step` only inside `steps` |
| `child_kinds` | Declared (`tabs` → `tab`) but **not enforced as an exclusive set** |
| `required_child_kinds` | `card-grid` must contain `link-card` |
| `forbidden_children` | Asides must not contain `tabs` |
| `paint_fields` / `paint_content` | Which scalars the planner emits |

Exclusive child models (`:tabs` needs `:tab`; `:steps` is `:step` or an
ordered list; `:figure` needs exactly one image) still live as special
cases in `docs.rs`, not as generic `child_kinds` checks. Markdown and other
blocks inside `:tabs` are ignored when collecting tabs rather than rejected
up front by the table.[^docs-rs][^registry]

### Painting

`DocsComponents.rocci` defines one `@component` per widget kind, matching
the usual Rocci shape `|{ props }, content| -> Html`. Optional props are
not Roc optional fields; painters branch on `title != ""` or `open`. Rocci
`??` defaults exist in the template language but these painters do not use
them. There is no magic `children` field.[^docs-rocci][^template-readme][^pure-render]

Static apply does **not** call those components with a typed child list.
`RocdownBuild.roc` matches a flattened `ArticleNode` forest, reads
`child_count`, recursively paints descendants to `Html`, then calls
`DocsComponents.tabs({}, body)` with an already-concatenated fragment.
`Tabs` cannot see tab `id` / `label` as data. It wraps whatever HTML the
child painters produced. That is why the shipped tabs UI is stacked
sections (`h3` + panel), not a `tablist`: the parent renderer never
received structured children.[^build-runtime][^plan-rs][^docs-guide]

Standalone `rocdown run` on a single file still has a conservative
`lower.rs` HTML path beside the theme painters. Two paint implementations
must stay in sync today.[^lower-rs]

### Theme versus blocks

`rocdown.toml` `[build].theme` names a directory or `SiteShell.rocci`. That
module owns **page chrome** (nav, outline, article slot). It does not own a
map from `:note` to a painter. Builtin widgets always come from
`DocsComponents` via `RocdownBuild.roc`. A project theme can restyle
`.rd-docs-note` if the classes leak into the article, but it cannot emit a
different element tree for the same `:note` without replacing the builtin
runtime — and there is no supported hook for that.[^config-rs][^site-ref][^docs-rocci][^theming-arch]

The theming architecture record is explicit: current surfaces are CSS
variables and a Rocci shell, not presentation renderers or an external
theme-package interface.[^theming-arch]

## The interesting idea

Treat a `:kind` as a **function interface**, and a Rocci component as one
**renderer** of that interface.

```text
interface  :tabs[group, kind]  children :tab+     ->  (schema)
renderer   Tabs = |{ group, kind }, tabs| Html    ->  (paint)
document   :tabs[group: "os", kind: "platform"]   ->  (call)
```

Documents call the interface. Sites bind renderers. Validation uses the
interface. HTML comes only from a renderer. Swapping `:note` from an
`<aside>` to a `<details>` is a renderer change, not a document or CSS
change.[^pure-render][^catalog-shell]

That is already how `@docs` *wanted* to work, and how `KindSpec.component`
hints, except three things are fused:

1. Schema lives in Rust; painters live in Rocci; the dispatcher in
   `RocdownBuild.roc` hard-codes both.
2. Children are flattened to `Html` before the parent runs.
3. The site can replace the shell, not the widget table.

The generation-pipeline research already forbids compiling a renderer per
page and forbids interpreting `.rocci` in Rust. A site-level renderer pack
compiled once per theme matches that rule.[^generation-research][^compiler-arch]

## Block interface

An article block is a Rocci-shaped function:

```text
|{ params }, children| -> Html
```

**Params** are the bracket record. They are named, not positional. The
schema says which names exist, which are required, which are optional, and
what defaults apply when omitted. Enumerations (`tabs.kind`, `badge.tone`)
stay schema, not painter branches that silently accept anything.[^registry][^template-readme]

**Defaults** belong on the *interface*, so documents and renderers stay
stable when a site swaps paint. Rocci already has `|{ title ?? "" }|`;
lowering strips `??` for current Roc nightly and fills call sites. The
planner can do the same for `:note` with no title: either emit the default
into generated Roc, or require the renderer to treat empty string as
absent (today's pattern). Prefer schema-level defaults so a custom `Note`
does not have to reimplement "missing title".[^template-readme][^template-ast]

Roc nightly still cannot express optional *record fields*. Optional block
params are therefore "present or defaulted", not `Option Str` in generated
Roc, unless a later Roc change lands. Empty `Str` / `Bool.false` remain the
v1 encoding, matching `paint_fields`.[^template-readme][^registry]

**Children** are not a magic `children` prop. They are the extra parameter
(s), same as paired Rocci tags. The schema must say which of two payload
shapes that parameter has:

| Mode | Extra argument | When |
| --- | --- | --- |
| Fragment | `content : Html` | Nested Markdown plus allowed blocks, already painted (`:note`, `:details`, `:tab` body) |
| Typed list | `items : List { …props, content : Html }` | Parent must *compose* children (`:tabs`, `:steps`, `:card-grid`) |

Fragment mode is today's calling convention and is enough when the parent
only wraps. Typed-list mode is what `:tabs` needs in order to build a real
tablist from `id` / `label` without scraping HTML. The child kinds still
have their own fragment renderers for the panel body; the parent renderer
receives records, not a concatenated blob.[^docs-rocci][^build-runtime][^template-readme]

A third mode, **slots**, appears in Markdoc and MDC (`#title`, named
slots). Rocdown should not add named slots in v1. `:figure` caption/credit
are params; the image is the body. If a later kind needs two document
regions, add a second extra parameter explicitly (`|{ }, caption, body|`)
rather than a slot language.[^markdoc-tags]

**Return** is always `Html`. Renderers stay pure `@component` functions. No
hidden lifecycle on `:note`. Interactive tabs that need JS are a hashed
article script or a `hydrate` / `live` island, not a stateful block
component.[^pure-render][^docs-guide]

## Constraining children

Authors already expect `:tabs` to contain `:tab`. The catalog almost
enforces that, but the mechanism is split and incomplete.[^docs-rs][^registry]

Generalize `KindSpec` child policy into one table the parser, catalog, and
LSP all read:

| Policy | Meaning | Example |
| --- | --- | --- |
| `parents` | Kind may only appear under these parents | `:tab` → `:tabs` |
| `accepts` | Direct block children must be this set; empty means any article block plus Markdown | `:tabs` → `[:tab]` |
| `accepts_markdown` | Whether non-block Markdown is legal beside those children | `:note` yes; `:tabs` no |
| `requires` | At least one of each listed kind | `:tabs` requires `:tab`; `:card-grid` requires `:link-card` |
| `forbids` | Nested kinds banned even if `accepts` is open | asides forbid `:tabs` |
| Sugar exceptions | Ordered list as `:step` equivalent; exactly one image in `:figure` | keep as named predicates on the spec, not one-off `match` |

`:steps` today allows either `:step` children *or* an ordered Markdown
list, but not both. That is an `accepts` union plus a "no mix" flag, not a
reason to keep a private validator forever.[^docs-rs]

Custom kinds must use the same table. If `:callout` is imported, its
`accepts` cannot be a Rust match arm. Extraction from the Rocci interface
(see [Addressability](#addressability)) or an explicit schema sidecar is
required; otherwise custom kinds are "any children" and `:tabs`-like
widgets cannot be added from a theme.

LSP completions already filter by `parent_allowed`. Completions inside
`:tabs` should offer `:tab` first once `accepts` is real data.[^parse-rs]

Do not encode child policy only as Roc types (`List Tab`). Roc cannot see
Markdown sugar nodes, and `rocdown check` must not require Roc. Rust stays
the validator; Rocci types document the renderer argument.[^catalog-shell]

## Addressability

Open question from the brief: decorator, export primitive, or neither?

### What exists

Interactive `@use "./Callout.rocci"` treats **every** `@component` as a
block kind. `Callout` → `:callout`, `LinkCard` → `:link-card`. There is no
opt-in, no rename, and no way to ship helpers (`Label`, `Icon`) beside the
block without them becoming `:label` / `:icon`. Collision with builtins is
fatal, so this path cannot override `:note`.[^imports-rs][^callout-fixture]

Ordinary Roc `module M exposing [note]` exports lowered functions, not
article kinds. Reusing `exposing` would collide with Roc module semantics
and with handlers that must expose camelCase names.[^template-readme]

### Options

| Mechanism | How a component becomes `:kind` | Override `:note` | Helpers in the same file |
| --- | --- | --- | --- |
| A. Convention (current `@use`) | Every `@component` | No (collision error) | No — they become kinds |
| B. Directory convention | Files under `theme/blocks/` | Same name as `KindSpec.component` | Yes, if helpers live elsewhere or are not scanned |
| C. Export table | Roc record `blocks = { note: Note }` | Yes, by key | Yes |
| D. `@block` decorator | `@block Note = \|{ title }, content\|` | `@block Note` in the site pack | Ordinary `@component` stays private |
| E. `@block as` | `@block(as: "note") Warning = …` | Rename without matching PascalCase | Yes |

**Recommendation:** B + D, with C as a config overlay.

- **Directory / pack convention** is enough for *renderer override* of
  builtins and matches Nuxt Prose files, Docusaurus `src/theme`, and
  Rocdown's existing `build.theme` directory. A module named
  `theme/Blocks.rocci` (or each file in `theme/blocks/`) that defines
  `@component Note` replaces the builtin `Note` painter. No new grammar.
  Scan only that pack, so `Button` in `theme/SiteShell.rocci` is not
  `:button`.[^nuxt-prose][^docusaurus-swizzle][^site-ref]
- **`@block` decorator** is the right *opt-in* once custom kinds share a
  file with helpers, or once child constraints must sit on the declaration.
  It is a small `rocci-template` grammar addition: recognize `@block` like
  `@component`, plus optional metadata (`as`, `accepts`). Until that lands,
  custom *static* kinds can be the same pack convention plus a registry row
  inferred from the component name and params.
- **Export table in `rocdown.toml`** (option C) is the site-config override
  list, not the authoring primitive. Authors should not edit TOML to create
  `:callout`; they should write a block module. Operators should edit TOML
  to point `:note` at a different module without renaming components.

Do **not** auto-export every `@component` from an arbitrary `@use` module
once `@block` exists. Keep `@use` as "import this block pack into an
interactive document", and require `@block` (or pack membership) so helpers
stay private. Until `@block` exists, document that `@use` modules must
contain only block components — today's trap.[^imports-rs]

Do **not** use JSX-style `<Note>` as the article spelling. Document-root
`<Tag>` is already a Rocci HTML island; article blocks stay `:note`.[^rocdown-readme][^markdown-first]

## Default renderer versus debug renderer

Related systems split this question in three ways:

1. **HTML fallback** — MDX maps unknown names to DOM tags (`p`, `h1`).
   Missing *custom* components typically crash
   (`Expected component MyCustom to be defined`).[^mdx-using][^mdx-missing]
2. **Hard error** — Markdoc's React renderer leaves a missing `render`
   target undefined; Astro's Markdoc discussion adopted "throw, do not
   invent HTML".[^markdoc-render][^astro-markdoc]
3. **Editor fallback** — Gutenberg shows an "invalid / unknown block"
   placeholder so the document remains visible while the schema is wrong.

Rocdown should use **two different policies**:

| Situation | `rocdown check` / `build` | `rocdown run` / inspect |
| --- | --- | --- |
| Unknown `:kind` (typo, not in schema) | Error today; keep it | Error; do not paint |
| Known kind, **no renderer bound** | Error, unless `build.debug_blocks` | Generic debug painter |
| Known kind, renderer fails to type-match interface | Error | Error |

**Builtin kinds require a default renderer.** `:note` is part of the
language users already write in `docs/`. Shipping the syntax without a
first-party `Note` would make the default theme a broken product. That
default lives in `DocsComponents` (or a successor pack) and is always on
the renderer search path.[^docs-rocci][^docs-guide]

**Custom kinds do not require a first-party renderer.** A site that
declares `:callout` must bind one, or accept the debug painter in preview.
The debug painter should be obviously unfinished: kind name, params as a
definition list, children in a nested box, `data-rocci-block-debug`. It
must not look like a styled aside. Never skip the node (markdown-to-jsx
"drop missing components"): silent omission loses content.

Standalone `lower.rs` conservative HTML can *be* that debug-shaped
fallback for unknown imported kinds during `rocdown run`, instead of a
second hand-written widget set. Builtin kinds should still use real
painters in standalone preview so `test/AllSyntax.rocdown` looks like
docs.[^lower-rs][^all-syntax]

## Site configuration and override

The site is the renderer host. Documents stay renderer-agnostic.

Search order for a kind's painter:

1. `rocdown.toml` explicit map, if present
2. Project `build.theme` block pack (`theme/Blocks.rocci` or
   `theme/blocks/`)
3. Builtin `DocsComponents`
4. Debug painter (preview only) or error (build)

Later entries do not win; this is overlay-from-specific-to-default, the
same idea as Docusaurus swizzle (site `src/theme` beats preset) and MDC's
`components` prop (explicit map beats Prose defaults).[^docusaurus-swizzle][^nuxt-mdc-issue]

Suggested config shape (not shipped):

```toml
[build]
theme = "theme"

[blocks]
pack = "theme/Blocks.rocci"
debug = false

[blocks.override]
note = "Note"
tabs = "Tabs"
```

`pack` is a Rocci module compiled with the theme (once per build, not per
page). `override` remaps kind → component name inside that pack or the
builtin module. Omitting `[blocks]` keeps today's DocsComponents-only
behavior.[^config-rs][^generation-research]

Custom *kinds* (new names) appear when the pack exports a `@block` /
scanned component whose kebab name is not in the builtin table. The
catalog merges those rows into the registry for that site only. Other
sites without the pack still error on `:callout`. That is the static-site
answer to `@use`: custom kinds live in the compiled theme, not in each
page. Interactive `@use` remains for one-off documents under
`rocdown run`.[^rocdown-readme][^imports-rs]

Override must be allowed to change **structure**, not only class names.
The replacement `Note` receives the same interface (`title`, `content :
Html`) and may return any `Html`. CSS in the block module stays scoped
`@css` on that component. Do not require the override to keep
`rd-docs-note`; first-party docs can keep those classes until a paint
pass renames them.[^docs-rocci][^pure-render]

Layout (`@page.layout`, named site layouts) stays **page chrome**. Block
renderers stay **article widgets**. Mixing them (`Note` calling
`SiteShell`) is out of scope.[^site-ref][^catalog-shell]

## Dispatcher

Today's `match ArticleNode` in `RocdownBuild.roc` is the bottleneck: every
new kind is a Roc arm, a `KindSpec` row, and a component. After a renderer
table exists, generated Roc should call through a single walk:

```text
paint(node) =
    renderer = table.get(node.kind) ?? debug
    renderer(node.params, paint_children(node))
```

For typed-list kinds, `paint_children` builds `List { params, content }`
instead of `Html.fragment`. Generating that walk from the registry — or
emitting a Roc `Dict` of functions — removes the closed matcher. Roc has
no first-class dict of functions as a stable pattern; emitting an
exhaustive `match` **from the merged registry** at theme compile time is
enough and stays type-safe. The match is generated, not hand-written, so
a site pack that adds `:callout` extends it.[^build-runtime][^registry][^generation-research]

Do not interpret templates in Rust to avoid compiling the theme.[^catalog-shell]

## Related technologies

The useful split in the wild is almost always **schema / transform /
render**, plus a **component map** the site owns.

### Markdoc (Stripe)

Closest analogue. A tag schema declares `render` (component name),
`attributes` (typed, required, default, `matches` enums), `children`
(allowed node type names), `validate`, and optional `transform`. Parse
produces an AST; transform binds schema; render maps `render` names to
React components (or HTML tags). The same document can render to HTML or
React. Missing React components are undefined / throw, not a styled
fallback. Nodes (headings, fences) use the same schema type as custom
tags, which matches Rocdown sugar `#` → `h2` `BlockCall`.[^markdoc-tags][^markdoc-render][^block-research]

Steal: schema `children: ['tab']`, attribute defaults, site `components`
map, validate vs render phases. Do not steal `{% tag %}` spelling or JS
`transform` in user config — Rocci renderers *are* the transform.

### MDX and `mdx-components`

Markdown + JSX. The site passes `{ h1, p, Note }` into the compiled
module or `MDXProvider`. HTML names fall back to DOM elements. There is
no child schema; nested MDX is just React children. Next.js App Router
uses a conventional `mdx-components.tsx`. Unknown custom components
crash at runtime. Great for override; weak for `:tabs`-only-`:tab`.[^mdx-using][^mdx-missing]

Steal: site-level name map, including heading sugar. Do not steal inline
JSX in prose.

### Nuxt Content MDC

Source spelling is the nearest cousin (`::note`). Rendering is Vue
components. **Prose** components (`ProseH1`, `ProseA`) override Markdown
sugar by filename in `components/content/`. Custom `::alert` maps to
`Alert`. `MDCRenderer`'s `components` prop overlays the map for
theme-switchers — undocumented but used in the wild. Named slots exist
(`#title`).[^nuxt-prose][^nuxt-mdc-issue][^block-research]

Steal: directory convention for overrides; explicit overlay map for
multi-theme. Do not steal Vue `::` closers (already rejected).

### MyST, Sphinx, docutils

Directives are "functions in markup": name, arguments, options, body.
Sphinx `app.add_directive` registers a class whose `run()` returns
docutils nodes. Child constraints are the docutils node content model
(`paragraph` may only contain inlines). Roles are the inline analogue —
out of Rocdown v1 scope. MyST `{note}` is the Markdown spelling of the
same registry.[^myst-syntax][^sphinx-extend]

Steal: directive = interface, writer/theme = renderer; node content
models. Do not steal Python classes as the authoring API; Rocci
components are the writer.

### Docusaurus swizzle and VitePress aliases

Docusaurus lets a site **wrap** or **eject** `@theme/Admonition`. Custom
admonition keywords need an `Admonition/Types` map. Swizzle is powerful
and famous for upgrade pain (unsafe internals). VitePress extends the
default theme and replaces internals with Vite aliases.[^docusaurus-swizzle][^docusaurus-admonitions][^vitepress-theme]

Steal: wrap-vs-replace (a site `Note` may call builtin `Note` if we
expose it under a stable name, like `@theme-original`). Prefer a small
**public** widget surface (`Note`, `Tabs`, `Tab`, `Figure`, …) marked
safe to override. Do not swizzle `RocdownBuild.roc`.

Gatsby **theme shadowing** is the same pattern with files.

### Shortcodes (Hugo, 11ty)

Name + params + inner HTML, no schema, no child kinds. Easy and sloppy.
Rocdown already outgrew this with `KindSpec`.

### Pandoc filters and remark-directive

AST rewrites after parse. Filters can change structure but are not a
component interface. Useful comparison for "keep parse stable, swap
paint"; not an authoring model to copy.

### Gutenberg `block.json` and InnerBlocks

`block.json` is an explicit schema (attributes, supports).
`InnerBlocks.allowedBlocks` is the child constraint API (`core/tabs`
allows `core/tab`). Unknown blocks get a placeholder. Heavy editor
orientation, but the schema/renderer/allowed-children split is exactly
the product need.

Sanity Portable Text **serializers**, Storyblok **bloks**, and Payload
**blocks** are CMS cousins: JSON tree + per-type React renderer map,
parent-type restrictions in the schema.

### Typst `show` rules

`#show heading: it => …` rebinds how a *kind* paints without changing
source. Strongest "renderer override" metaphor outside the JS docs
ecosystem. Rocdown cannot run a script inside the document (Markdown-
first), but a site pack is the batch equivalent of `show`.

### Bravo decorators

Bravo's ungram treats decorated blocks as the same `Block` layer as
headings. Decorators resolve to calls. Inspiration for uniform
`BlockCall`, not for Rocci export syntax. Inline `DecoratedFragment`
stays out of Rocdown v1.[^block-research]

## Compatibility with existing decisions

| Decision | Meaning here |
| --- | --- |
| Markdown-first islands | New `@block` / `@use` stay document-root or theme modules; no mid-paragraph `:note` |
| Pure `@component` | Renderers are functions to `Html`; no lifecycle on widgets |
| Rust catalog / Rocci shell | Schema validation, includes, heading ids stay Rust; paint stays Rocci; do not interpret `.rocci` in Rust |
| Generation pipeline | One renderer compile per theme; no per-page Roc for `:note` |
| Theming surfaces | This research *proposes* presentation renderers; it does not claim they exist |
| OKF Markdown-only | No `:note` in `knowledge/**/*.md` |
| Hybrid islands | `static` pages keep the widget forest; block renderers are not Datastar islands |

Source spelling does not change. `@docs` stays gone.[^markdown-first][^pure-render][^catalog-shell][^generation-research][^theming-arch][^block-research]

## Recommended architecture

Freeze these answers unless a plan revision says otherwise:

1. **Split schema and renderer.** Schema is `KindSpec` (builtins) plus
   extracted rows from the site block pack (custom). Renderer is a Rocci
   `@component` whose props and extra argument match the schema.
2. **Two child modes:** fragment `Html` vs typed `List` of child records.
   `:tabs` / `:steps` / `:card-grid` use typed lists.
3. **Child policy is data** (`accepts`, `requires`, `forbids`,
   `accepts_markdown`). Special predicates remain only for list-as-steps
   and one-image figures.
4. **Addressability:** scan a theme block pack by convention; add
   `@block` only when helpers-in-the-same-file or `as:` renames force it.
   Stop treating every `@use` `@component` as a kind once `@block` exists.
5. **Defaults:** builtin kinds always have `DocsComponents` painters.
   Missing renderer for a *known* custom kind is a debug painter in
   preview and an error in `rocdown build` unless `blocks.debug = true`.
   Unknown kinds stay errors.
6. **Site overlay:** explicit TOML map, then theme pack, then builtins.
   Overrides may emit any Html. Page layouts stay separate from widgets.
7. **Generated dispatcher** from the merged registry, replacing the
   hand-written `RocdownBuild.roc` match as the source of truth.

## Non-goals

- Inline decorations or MDX-in-prose
- Named slots in v1
- Per-page `@component` on static `rocdown build` (hybrid islands own that)
- Interpreting templates in Rust
- Changing OKF authoring
- Requiring Roc traits / typeclass `implements Block`
- Making CSS-variable themes a substitute for renderer override

## Open questions

1. **`@block` in v1 or pack convention only?** Pack-only ships overrides
   faster and avoids `rocci-template` grammar. `@block` is cleaner for
   mixed helper/block modules and `as:` names. Leaning: pack-only in the
   first delivery; `@block` as a follow-on unless custom kinds land in
   the same change.
2. **Typed child lists vs parent-only painters?** If `Tabs` receives
   `List { id, label, content }`, is `@component Tab` still called for
   each panel, or does `Tabs` own all markup? Leaning: still paint each
   `:tab` body through `Tab` (or a fragment renderer), then pass records
   to `Tabs` for chrome. Sites can override `Tab`, `Tabs`, or both.
3. **May an override wrap the builtin?** Docusaurus `@theme-original`.
   Leaning: yes, via an explicit import name (`DocsComponents.note`) so
   a site can add a class without copying markup. That import is a
   stability promise on the public widget surface.
4. **Do heading sugar renderers move to Rocci?** `#` is already `h2`
   `BlockCall`. A site `H2` would let chrome control heading HTML.
   Catalog ids stay Rust. Leaning: allow override, keep default in Rust
   or a trivial `H2` component; not required to prove `:note` override.
5. **Config key bikeshed:** `[blocks]` vs `[build.blocks]` vs files only.
   Leaning: `[blocks]` table, because it is a product surface, not a
   compiler path.
6. **Should `child_kinds` exclusive-reject Markdown in `:tabs`?** Today
   extra Markdown is skipped when collecting tabs. Leaning: yes, error;
   authors should not leave stray paragraphs between `:tab` children.

## Recommended next experiments

Delivery phases live in
[the implementation plan](../plans/rocdown-block-renderers.md). Do not
start them until asked. The first vertical slice that proves the
architecture is: structured `:tabs` children into `Tabs`, plus a site
`theme/Blocks.rocci` that replaces `Note` with a different element tree,
with `rocdown build docs` still green on un-overridden kinds.[^impl-plan]

[^rocdown-readme]: Shipped `:kind[params]`, closed registry plus interactive `@use`, `@use` rejected on static build.
[^registry]: `KindSpec` fields, `child_kinds` unused as exclusive policy, `component` names, paint flags.
[^docs-rs]: Special-case validators for steps, figure, tabs, file-tree, compatibility; `required_child_kinds` used; `child_kinds` not.
[^parse-rs]: Unknown kind error; imported kinds skip builtin lookup; `parent_allowed`.
[^imports-rs]: Every `@component` becomes a kind; builtin collision is an error; PascalCase to kebab.
[^config-rs]: `SiteConfig` / `BuildConfig` have `theme` and no blocks table.
[^plan-rs]: Theme module compile from `build.theme`; `PlannedNode` emits component tags plus `child_count`.
[^lower-rs]: Standalone conservative HTML for article blocks beside the theme path.
[^docs-rocci]: Per-kind `@component` painters; `Tabs` takes opaque `content`; `rd-docs-*` classes.
[^build-runtime]: Hand-written `match` and `render_children!` concatenating Html before parent calls.
[^template-readme]: Props record plus extra body parameter; `??` defaults stripped for Roc nightly; no magic children.
[^template-ast]: `param_defaults` captured from `??` for call-site filling.
[^site-ref]: `build.theme` is chrome; no renderer map.
[^docs-guide]: Public `:name[params]` widgets; tabs currently stacked sections without `tablist`.
[^all-syntax]: Shipped colon examples including `:tabs` / `:tab`.
[^block-research]: Syntax research; uniform BlockCall; registry idea; `@use` sketch.
[^block-plan]: Nine phases for spelling and closed registry; `@use` interactive-only in v1.
[^generation-research]: Do not compile per page; do not interpret Rocci in Rust.
[^format-arch]: Current format boundary; architecture is descriptive of shipped parse, not this proposal.
[^compiler-arch]: Static article tree and Rocci painting from planned nodes.
[^theming-arch]: CSS variables and Rocci shell; no presentation-renderer package interface.
[^markdown-first]: Mode changes at visible block boundaries only.
[^pure-render]: `@component` is a pure function to Html.
[^catalog-shell]: Rust owns catalog; Rocci owns visible chrome.
[^callout-fixture]: Research `Callout.rocci` used as the `@use` example shape.
[^markdoc-tags]: Schema `render`, `attributes`, `children`, `validate`.
[^markdoc-render]: Parse / transform / render; component map at render time.
[^mdx-using]: `components` prop and provider merge; HTML name fallback.
[^nuxt-prose]: Filename convention overrides Markdown sugar and MDC tags.
[^nuxt-mdc-issue]: Overlay `components` prop for theme-specific maps.
[^docusaurus-swizzle]: Site theme files shadow preset components; wrap vs eject.
[^docusaurus-admonitions]: Keyword-to-component map for custom admonitions.
[^vitepress-theme]: Extend default theme; alias-replace internals.
[^myst-syntax]: Directives as markup functions with args, options, body.
[^sphinx-extend]: Register directive classes; docutils child node models.
[^mdx-missing]: Default crash on missing custom MDX component.
[^astro-markdoc]: Prefer throw over inventing HTML for undefined components.
[^impl-plan]: Phased delivery; exploratory; do not start until asked.
