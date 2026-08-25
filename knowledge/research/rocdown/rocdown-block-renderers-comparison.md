---
type: Research Report
title: Landed Rocdown block renderers compared with related systems
description: "Post-landing comparison: Rocdown's KindSpec plus theme block-pack painters versus Markdoc, MDX, Nuxt MDC, MyST/Sphinx, Docusaurus/VitePress, shortcodes, Pandoc/remark, Gutenberg, CMS serializers, Typst show, and Bravo. Includes DX, authorable surface, expressability ceiling, and stack-consistency fractures. Scores are exploratory synthesis."
tags: [domain/rocdown, domain/rocci, concern/rendering, concern/theming, concern/architecture, concern/authoring]
status: draft
generated: { by: process:cursor, at: 2026-08-25T10:45:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: design-research
    resource: ../rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers research
    author: process:cursor
    last_modified: 2026-08-19
  - id: impl-plan
    resource: ../../plans/rocdown/rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: registry
    resource: ../../../crates/rocci-rocdown/src/registry.rs
    title: KindSpec table, child policy, pack-kind inference
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rs
    resource: ../../../crates/rocci-rocdown/src/docs.rs
    title: Generic child validation
    author: process:git
    last_modified: 2026-08-19
  - id: config-rs
    resource: ../../../crates/rocci-rocdown/src/config.rs
    title: Site [blocks] configuration
    author: process:git
    last_modified: 2026-08-19
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Pack overlay, BlockPainters, generated dispatcher
    author: process:git
    last_modified: 2026-08-19
  - id: build-runtime
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Forest walk, typed child records, Html flatten
    author: process:git
    last_modified: 2026-08-19
  - id: docs-rocci
    resource: ../../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Builtin article-block painters
    author: process:git
    last_modified: 2026-08-19
  - id: block-debug
    resource: ../../../crates/rocci-rocdown/templates/BlockDebug.rocci
    title: First-party debug painter
    author: process:git
    last_modified: 2026-08-19
  - id: site-ref
    resource: ../../../docs/reference/rocdown-site.rocdown
    title: Public Rocdown site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: docs-guide
    resource: ../../../docs/guides/docs-components.rocdown
    title: Public documentation-component guide
    author: process:git
    last_modified: 2026-08-19
  - id: lang-ref
    resource: ../../../docs/reference/rocdown.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-19
  - id: pages-guide
    resource: ../../../docs/guides/rocdown-pages.rocdown
    title: Public Rocdown pages guide
    author: process:git
    last_modified: 2026-08-19
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Rocci component calling convention and ?? defaults
    author: process:git
    last_modified: 2026-08-19
  - id: roc-defaults
    resource: ../rocci/roc-nightly-record-defaults.md
    title: Roc nightly 2026-08-23 type-position defaults versus pattern ??
    author: process:cursor
    last_modified: 2026-08-25
  - id: lsp-tests
    resource: ../../../crates/rocci-rocdown/tests/lsp.rs
    title: Kind, field, and accepts-aware completions
    author: process:git
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../../crates/rocci-rocdown/src/lower.rs
    title: Standalone article-block lowering
    author: process:git
    last_modified: 2026-08-19
  - id: imports-rs
    resource: ../../../crates/rocci-rocdown/src/imports.rs
    title: "@use component-to-kind mapping"
    author: process:git
    last_modified: 2026-08-19
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Keep Rocdown Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: theming-arch
    resource: ../../architecture/theming.md
    title: Current Rocci theming surfaces
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: markdoc-tags
    resource: https://markdoc.dev/docs/tags
    title: Markdoc custom tags, attributes, and children
    author: organization:stripe
  - id: mdx-using
    resource: https://mdxjs.com/docs/using-mdx/
    title: MDX components prop and MDXProvider
    author: organization:mdx-js
  - id: mdx-missing
    resource: https://github.com/hashicorp/next-mdx-remote/discussions/470
    title: Missing MDX component fallback discussion
    author: organization:hashicorp
  - id: nuxt-prose
    resource: https://content.nuxt.com/docs/components/prose
    title: Nuxt Content Prose component overrides
    author: organization:nuxt
  - id: docusaurus-swizzle
    resource: https://docusaurus.io/docs/swizzling
    title: Docusaurus theme component swizzling
    author: organization:facebook
  - id: vitepress-theme
    resource: https://vitepress.dev/guide/extending-default-theme
    title: VitePress theme extension and internal aliases
    author: organization:vuejs
  - id: myst-syntax
    resource: https://mystmd.org/guide/syntax-overview
    title: MyST directives as markup functions
    author: organization:executablebooks
  - id: gutenberg-inner
    resource: https://developer.wordpress.org/block-editor/how-to-guides/block-tutorial/nested-blocks-inner-blocks/
    title: Gutenberg InnerBlocks and allowedBlocks
    author: organization:wordpress
  - id: typst-show
    resource: https://typst.app/docs/reference/styling/
    title: Typst set and show rules
    author: organization:typst
---

# Landed Rocdown block renderers compared with related systems

## Scope and authority

This record compares the **landed** Rocdown schema/renderer split with the
systems listed in the [pre-landing design research](rocdown-block-renderers.md).
Crate READMEs and public site docs describe shipped behavior. Composite scores
in this record are **exploratory synthesis** against Rocdown's own product
questions, not a popularity ranking and not an approved architecture
change.[^design-research][^rocdown-readme][^site-ref]

The design research and [implementation plan](/plans/rocdown/rocdown-block-renderers.md)
were written before the overlay existed in the crate. Prefer this record for
"how does landed Rocdown sit next to Markdoc / MDX / …". Prefer the crate
README for the current contract. Prefer the plan for leftover phases
(`@block`, heading-sugar painters).[^impl-plan][^rocdown-readme]

## For a later agent

- **Authority:** exploratory for scores and "steal / reject" judgments.
  Shipped painter search order, `[blocks]` config, pack inference, static vs
  interactive authoring, and LSP completions are descriptive of the crate,
  cited below.
- **Do not** treat the design research's "not shipped" banner as current.
- **Do not** start `@block` grammar or a Prose heading layer unless asked.
- **Keep** Markdown-first islands, pure `@component` paint, Rust catalog /
  Rocci shell, one theme compile per build.[^markdown-first][^pure-render][^catalog-shell]

## Landed contract (descriptive)

Documents still write `:kind[params]`. Rust `KindSpec` validates named params
and child policy (`accepts`, `requires`, `forbids`, `accepts_markdown`, plus
named predicates for steps XOR-list and one-image figures). A site binds
painters in this order:[^registry][^docs-rs][^config-rs][^plan-rs][^site-ref]

1. `[blocks.override]` kind → component name, if present
2. `theme/Blocks.rocci`, else `theme/blocks/*.rocci`, else `[blocks] pack`
3. Builtin `DocsComponents`
4. `BlockDebug` in `rocdown run`; `rocdown build` / `check` error unless
   `[blocks] debug = true`

Unknown `:kind` stays an error. Packs may `import DocsComponents` and wrap.
A pack `@component` whose PascalCase name is not a builtin painter becomes a
site-local kind (`Callout` → `:callout`); schema comes from the component
header (`??` defaults, extra body param). Child policy for those kinds
defaults to fragment plus any children. Helpers must not live in the pack.
Widget match arms are generated from the merged registry; `RocdownBuild.roc`
keeps the forest walk and IO.[^rocdown-readme][^docs-guide][^plan-rs][^block-debug]

The dispatcher builds typed child records for `:tabs` / `:steps` /
`:card-grid`, then `html_from_records` concatenates `item.content` before the
parent painter runs. Default `Tabs` still takes opaque `Html`. Heading sugar
(`h1`–`h6`) is a `BlockCall` but is not in the widget painter table.
`@block` is not in `rocci-template`. Interactive `@use` still auto-exports
every `@component` and cannot override builtins.[^build-runtime][^docs-rocci][^registry][^impl-plan]

## Scoring method

Each system is scored 0 (absent), 1 (partial / workaround), or 2 (first-class)
on ten questions taken from the design research. Equal weight. Totals are out
of 20.[^design-research]

| Key | Question |
| --- | --- |
| Split | Is the block interface separate from the function that returns HTML? |
| Kids | Can a parent require exclusive child kinds, including for site-defined types? |
| Override | Can a site change `:note`'s element tree without editing documents? |
| Check | Can the catalog reject bad markup without compiling the paint language? |
| MD | Do documents stay Markdown plus explicit islands, not JSX in prose? |
| Typed | Does the parent renderer receive child params as data, not scraped HTML? |
| Missing | Distinct policies for unknown kind vs known kind with no painter? |
| Addr | Can a module export helpers without every function becoming a kind? |
| API | Is the swappable widget set small and named, not internal swizzle? |
| Prose | Can a site replace heading / paragraph sugar the same way as `:note`? |

## Composite scores

| System | Split | Kids | Override | Check | MD | Typed | Missing | Addr | API | Prose | Total |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Markdoc | 2 | 2 | 2 | 1 | 2 | 1 | 1 | 2 | 2 | 1 | 16 |
| Rocdown (landed) | 2 | 1 | 2 | 2 | 2 | 1 | 2 | 1 | 2 | 0 | 15 |
| MyST / Sphinx | 2 | 2 | 1 | 0 | 2 | 2 | 1 | 2 | 1 | 1 | 15 |
| Typst show | 2 | 1 | 2 | 1 | 0 | 2 | 1 | 2 | 2 | 2 | 15 |
| Gutenberg | 2 | 2 | 1 | 0 | 0 | 2 | 2 | 2 | 2 | 0 | 14 |
| Sanity / Storyblok / Payload | 2 | 2 | 2 | 0 | 0 | 2 | 1 | 2 | 2 | 0 | 14 |
| Nuxt Content MDC | 1 | 1 | 2 | 0 | 2 | 1 | 1 | 1 | 2 | 2 | 13 |
| Pandoc / remark-directive | 1 | 0 | 1 | 1 | 2 | 2 | 0 | 0 | 1 | 1 | 10 |
| Docusaurus / VitePress | 1 | 0 | 2 | 0 | 1 | 0 | 1 | 1 | 1 | 1 | 9 |
| MDX | 0 | 0 | 2 | 0 | 0 | 1 | 1 | 1 | 1 | 2 | 8 |
| Hugo / 11ty shortcodes | 0 | 0 | 1 | 0 | 2 | 0 | 0 | 0 | 1 | 0 | 6 |
| Bravo decorators | 1 | 0 | 0 | 1 | 1 | 0 | 0 | 1 | 0 | 0 | 5 |

Rocdown's Check and MD scores are the constraints that keep it from copying
MDX, Typst-in-document `show`, or Sphinx Python classes. Markdoc's edge is
custom-tag `children` plus treating nodes and tags as one schema type.
Rocdown's Kids and Addr 1s, Typed 1, and Prose 0 are the landed gaps versus
that bar.[^markdoc-tags][^catalog-shell][^markdown-first]

## Closest analogues

**Markdoc** is the closest compiler. A tag schema declares `render`, typed
`attributes`, `children`, `validate`, and optional `transform`. The site
passes a component map at render time. Steal that split. Do not steal
`{% tag %}` spelling or JS `transform` in user config: Rocci renderers *are*
the transform. Markdoc validation is JS (not React) but still the config
language; `rocdown check` does not need Roc. Missing React targets throw
rather than paint a debug placeholder.[^markdoc-tags][^block-debug]

**Nuxt Content MDC** and **Docusaurus** explain the overlay. Nuxt Prose files
under `components/content/` replace Markdown sugar by filename; custom
`::alert` maps to `Alert`. Docusaurus `src/theme` wrap-or-eject, with
`@theme-original` for wrap. Rocdown stole the directory pack and wrap via
`import DocsComponents`, and rejected swizzling `RocdownBuild.roc`. VitePress
internal Vite aliases are the unsafe version of the same idea.[^nuxt-prose][^docusaurus-swizzle][^vitepress-theme][^docs-guide]

**Gutenberg** is the closest schema/children/placeholder split.
`block.json` plus `InnerBlocks.allowedBlocks` / `parent` is the product
need for `:tabs` → `:tab`. Unknown blocks get an editor placeholder;
Rocdown's `data-rocci-block-debug` is the docs equivalent. The block editor
as authoring UI was rejected (Markdown-first).[^gutenberg-inner][^block-debug][^markdown-first]

**Typst `show`** is the strongest "rebind how a kind paints without changing
source" metaphor. Rules live in the document. Rocdown cannot do that;
the site pack is the batch equivalent.[^typst-show][^markdown-first]

## Other systems, briefly

**MDX** has an excellent site `{ h1, p, Note }` map, including heading sugar,
and no child schema. JSX in prose is incompatible with Markdown-first
islands. Unknown *custom* names typically crash (`Expected component X to be
defined`); HTML names fall back to DOM tags.[^mdx-using][^mdx-missing][^markdown-first]

**MyST / Sphinx** treat directives as markup functions and use docutils node
content models for children. The writer/theme is the renderer. Registration
is Python classes, which Rocdown rejected as the authoring API.[^myst-syntax][^pure-render]

**Hugo / 11ty shortcodes** are name plus params plus inner HTML. Rocdown
already outgrew that with `KindSpec`.[^registry]

**Pandoc filters and remark-directive** keep parse stable and rewrite the
AST. Useful comparison for "swap paint"; not a component interface sites
overlay by filename.[^design-research]

**Sanity / Storyblok / Payload** are CMS cousins: JSON tree plus per-type
React serializer map and parent restrictions. Strong renderer tables;
documents are not Markdown islands.[^design-research]

**Bravo** inspired uniform `BlockCall`, not Rocci export syntax. Inline
decorations stayed out of Rocdown v1.[^design-research]

## Intentional non-copies

| From | Not taken | Why |
| --- | --- | --- |
| MDX | JSX in prose / `<Note>` | Document-root `<Tag>` is already an HTML island; article widgets stay `:note`.[^markdown-first] |
| Markdoc | User JS `transform` | Interpreting templates in Rust is out of scope; Rocci is the transform.[^catalog-shell] |
| Nuxt MDC | Named slots | v1 uses params plus one extra argument.[^impl-plan] |
| Typst | In-document `show` | Would break Markdown-first; site pack is the batch `show`.[^typst-show] |
| Shortcodes | Unschema'd inner HTML | `KindSpec` already exists.[^registry] |
| Docusaurus | Unsafe internal swizzle | Public widget names only (`Note`, `Tabs`, `Figure`).[^docusaurus-swizzle][^docs-guide] |
| Sphinx | Python directive classes | Rocci components are the writer.[^pure-render] |
| Bravo | Inline decorations | Out of Rocdown v1.[^design-research] |

## What Rocdown uniquely keeps

| Constraint | Landed behavior | Contrast |
| --- | --- | --- |
| Rust catalog, Rocci shell | `rocdown check` validates `KindSpec` without Roc. Paint compiles once per theme. | Markdoc / MDX / Sphinx need their paint language at check time.[^catalog-shell][^registry] |
| Markdown-first islands | `:kind` at block boundaries. No mid-paragraph widgets. | MDX inverts this. Typst is a document language.[^markdown-first] |
| Pure `@component` | `|{ params }, children| -> Html`. No widget lifecycle. | Docusaurus swizzle wraps stateful React. Gutenberg `edit` ≠ `save`.[^pure-render] |
| Unknown kind ≠ missing painter | Typos error. Known custom kind with no painter debugs in preview. | MDX crashes on custom names. Markdoc prefers throw.[^block-debug][^mdx-missing] |

## Remaining gaps against the same bar

These are the landed deltas versus the design research and versus systems that
already do that piece. They are not a backlog of every comparator feature.

| Gap | Landed today | Who already has it | Cost if ignored |
| --- | --- | --- | --- |
| Custom-kind child policy | Pack kinds default to fragment plus any children. No `@block` `accepts`. | Markdoc `children: ['tab']`; Gutenberg `allowedBlocks` | A site cannot add a `:tabs`-like widget that rejects stray Markdown.[^registry][^markdoc-tags][^gutenberg-inner] |
| Typed lists into the parent painter | Records exist in the walk; parent still receives concatenated Html. | Gutenberg InnerBlocks; Typst `it` fields; Sphinx node trees | A site `Tabs` cannot build a `tablist` from labels without scraping HTML. Default UI stays stacked sections.[^build-runtime][^docs-guide] |
| Helper-safe exports | Anything in the pack becomes a kind. `@use` auto-exports every `@component`. | Markdoc explicit `render`; planned `@block` | Helpers become `:label` / `:icon` or collide with reserved names (`page`, `use`).[^impl-plan][^rocdown-readme] |
| Heading / Prose painters | `h1`–`h6` are sugar `BlockCall`s but not widget painters. | Nuxt `ProseH1`; MDX `components.h1`; Typst `show heading` | Sites restyle headings with CSS only. Catalog ids stay Rust either way.[^registry][^nuxt-prose] |

The first two gaps are the ones that still block the original acceptance test
of **themeable structure for `:tabs`**, not merely for `:note`.[^design-research][^docs-guide]

## Authoring DX

DX here means: how fast an author learns the spelling, how soon a mistake is
caught, and how many languages or files they must touch to add a kind.

**Feedback is a strength.** `rocdown check` validates `KindSpec` without Roc.
Unknown kinds, exclusive children (stray Markdown in `:tabs`, `:tab` outside
`:tabs`), required fields, and `[blocks]` unknown keys are catalog
diagnostics. LSP completions offer kinds, fields, and enum values, and inside
`:tabs` they prefer `accepts` children. Delimiter XOR (line / `{{ }}` /
`:kind.begin` ... `:kind.end`, not mixed) is a sharp error instead of a
silent parse. Missing painters debug in preview and fail build unless opted
in.[^registry][^docs-rs][^config-rs][^lsp-tests][^lang-ref][^block-debug]

**Path-to-kind is a weakness.** There are two ways to introduce a name, with
different rules:[^imports-rs][^site-ref][^lang-ref]

| Path | When | How a component becomes `:kind` | Override `:note` | Helpers |
| --- | --- | --- | --- | --- |
| `@use "./Callout.rocci"` | `rocdown run` only | Every `@component` | No (collision error) | Become kinds |
| Theme pack | `rocdown build` / `check` | Every `@component` in the pack | Yes, by PascalCase match or `[blocks.override]` | Must not live in the pack |

Markdoc has one schema file plus a render map. MDX has one `components` prop
(and optional per-file JSX imports). Nuxt has one filename convention for both
sugar and tags. Rocdown's split is the cost of "static sites do not execute
per-page `@use`" plus "interactive docs still can." Authors who learn `@use`
first will try it on `docs/` and get RD2301.[^markdoc-tags][^mdx-using][^nuxt-prose][^lang-ref]

**Theme-author DX** is close to Nuxt Prose / Docusaurus wrap: drop
`theme/Blocks.rocci`, export `Note`, optionally `import DocsComponents`.
There is no swizzle CLI and no `@theme-original` alias magic — wrap is an
ordinary import. The trap is putting `Label` next to `Callout` in the same
pack. Docusaurus's famous upgrade pain does not apply to a twelve-widget
public surface; the pack-helper trap does.[^docs-guide][^docusaurus-swizzle]

**Standalone preview vs site paint** is a second DX split. `rocdown run` on a
single file still has a conservative `lower.rs` HTML path beside the theme
painters. Builtin docs look like the site only when the theme path runs.
That is two implementations to keep in sync for the same `:note`.[^lower-rs][^rocdown-readme]

Compared with Gutenberg (in-editor placeholder, attribute UI) and Typst
(show rule next to the heading), Rocdown DX is compiler-shaped: write
Markdown, run `check`, read a diagnostic. That matches Markdown-first; it
is not a visual block editor.[^gutenberg-inner][^typst-show][^markdown-first]

## What can actually be authored

Three actors, three surfaces. Mixing them is the usual authoring mistake.

### Document author, `static` pages

Legal: Markdown, `@page`, and `:kind` article blocks (builtins plus any
pack-inferred site kinds). Illegal on `rocdown build` / `check`: `@use`,
`@render`, `@component`, handlers, file CSS, custom layouts, document-root
`<Tag>` islands, and those same islands *inside* a `:note` body.[^pages-guide][^lang-ref]

Builtin kinds an author can write today:[^lang-ref][^registry]

| Kind | Authorable shape |
| --- | --- |
| `note`, `tip`, `caution`, `danger`, `deprecated` | Optional `title`; Markdown plus nested blocks; cannot contain `tabs` |
| `details` | Required `summary`; optional `open` |
| `steps` / `step` | `:step+` XOR an ordered Markdown list, not both; `step` only inside `steps` |
| `figure` | Exactly one image; optional `caption` / `credit` |
| `definition` | Required `term` |
| `badge` | Status label |
| `compatibility` | Table of combinations |
| `card-grid` / `link-card` | Only `link-card` children; `page` or `href` |
| `file-tree` | Nested unordered list |
| `tabs` / `tab` | Only `:tab` children; `kind` is `language` / `platform` / `tool`; stacked no-JS panels |
| `include` | File or named region |
| `example` | Display sample; optional `test` metadata for `rocdown test` |
| `img` / `h1`–`h6` | Sugar `BlockCall`s; not widget-pack paintable |

`api-operation` is reserved (`RD2406`). Admonition sugar other than `:kind`,
definition lists, math, and automatic TOC tokens are not parsed.[^lang-ref][^rocdown-readme]

A site pack can add fragment kinds (`:callout[tone: "warn"]`) that accept any
children. It cannot add a `:callouts` parent that *rejects* stray Markdown
until `@block` or a sidecar exists.[^registry][^site-ref]

### Document author, interactive / hybrid

`rocdown run` on a file may `@use` extra kinds. `hydrate` pages splice pure
Rocci; `live` pages add Datastar and an island service. Those islands are
not `:kind` widgets. `@use` remains a site-build error. A static `:tabs`
cannot become a JS `tablist` by authoring more Markdown.[^pages-guide][^docs-guide][^lang-ref]

### Theme author

Can replace any builtin widget's HTML tree, wrap `DocsComponents.note`, add
new PascalCase kinds, and remap with `[blocks.override]`. Cannot declare
`accepts` / `parents` on pack kinds, keep private helpers in the pack, or
override heading sugar as `H2`. Optional pack params use Rocci `??` in the
header; Rocci still fills omitted values as empty `Str` / `Bool.false` at
the call site even though Roc nightly-2026-08-23 can type defaulted and
optional record fields. Details: [Roc nightly record defaults](/research/rocci/roc-nightly-record-defaults.md).[^site-ref][^docs-guide][^template-readme][^roc-defaults]

## Expressability

What the *language* can say versus what a *painter* can do with it.

| Capability | Rocdown landed | Closest systems |
| --- | --- | --- |
| Named params with required / optional / one-of | Yes, on builtins; pack headers infer names and `??` | Markdoc attributes; Gutenberg `block.json` |
| Exclusive child kinds | Builtins only (`tabs`→`tab`, `card-grid`→`link-card`) | Markdoc `children`; Gutenberg `allowedBlocks` |
| Sugar exceptions | `steps` XOR list; figure one-image | Sphinx/docutils content models |
| Named slots (`#title`) | No | Markdoc, Nuxt MDC |
| Inline widgets in a paragraph | No (`:note` is block-boundary) | MDX JSX; Markdoc inline tags; Typst functions |
| Parent sees child records | Built then flattened to Html | Gutenberg InnerBlocks; Typst `it` |
| Heading / `p` / `a` painters | No | Nuxt Prose; MDX `components.h1` |
| Site-defined static kinds | Yes, fragment-shaped | Markdoc tags; Nuxt `::alert` |
| Per-document custom kinds on a static site | No (`@use` rejected) | MDX imports; Markdoc per-project schema |
| Interactive structure (tablist, live state) | Not on `:kind`; `live` pages are a different island | MDX/React; Gutenberg viewScript |
| In-document restyle | No | Typst `show` |

Rocdown is **more expressive than shortcodes** (schema, child policy,
overrides) and **less expressive than MDX** (no JSX, no inline custom
components, no heading map). It is **close to Markdoc on documents** and
**behind Markdoc on custom-tag schema**. Theme expressability for `:note` is
on par with Nuxt/Docusaurus (replace the element tree). Theme expressability
for `:tabs` is still shortcode-like: wrap concatenated HTML.[^markdoc-tags][^mdx-using][^nuxt-prose][^build-runtime][^docs-rocci]

Document spelling is uniform (`:kind[params]`). That is better consistency
than MDX (Markdown + JSX) or Docusaurus (Markdown + MDX + swizzled React).
The *payload* behind `:tabs` is not yet as rich as the spelling suggests:
authors write `id` / `label`, painters cannot use them as data.[^lang-ref][^docs-guide]

## Stack consistency

The intended stack is one function interface: documents call it, Rust
validates it, Rocci paints it, one compile per theme.[^catalog-shell][^pure-render]

Landed fractures:

1. **Three document-root call spellings.** `:kind` is an article widget,
   `@component` / `<Tag>` is a Rocci island, `@render` is a Roc `Html`
   splice. Same "invoke a function" idea, three grammars, three feature
   gates. MDX collapses this to JSX; Rocdown keeps the split on
   purpose.[^lang-ref][^markdown-first]
2. **Two kind-introduction paths.** `@use` versus pack, rules above.
   Markdoc/Nuxt have one.[^imports-rs][^site-ref]
3. **Two child-policy tables.** Builtin `KindSpec` is exclusive where
   declared; pack kinds are "any children." Authors cannot tell from the
   `:callout` spelling which table applies.[^registry]
4. **Two paint implementations.** Theme `BlockPainters` versus standalone
   `lower.rs`.[^lower-rs][^plan-rs]
5. **Typed records that do not reach the painter.** The dispatcher is
   internally consistent with Phase 2; the public `Tabs` signature is still
   Phase 1.[^build-runtime][^docs-rocci]
6. **Sugar `BlockCall` that is not a widget.** `#` / `:h2` / `:img` parse as
   the same node family as `:note` but skip the painter table. Nuxt Prose
   treats headings as the *same* overlay as tags. Rocdown does not.[^registry][^nuxt-prose]
7. **Optional fields as empty scalars.** Pack headers have `??`; generated
   Roc does not. Theme authors re-learn `title != ""`.[^template-readme]
8. **Page chrome versus article widgets.** `SiteShell` / layouts stay
   separate from `Blocks.rocci`. That split is consistent with the
   catalog-shell decision and with VitePress layout slots versus Prose.
   Mixing `Note` into `SiteShell` is out of scope.[^site-ref][^catalog-shell]

What *is* consistent: one `:kind` spelling on static and live pages; one
Rust catalog for `check`; one generated dispatcher from the merged
registry; wrap-not-eject via `import DocsComponents`; unknown kind always
errors. Those are the pieces that stay stricter than MDX and shortcodes
even with the fractures.[^plan-rs][^block-debug][^lang-ref]

## Takeaway

The landed design is Markdoc's split, Nuxt's pack convention, Docusaurus wrap
(via `DocsComponents`), and Gutenberg's missing-renderer placeholder —
compiled once in Rocci and checked in Rust. Document DX (check, LSP,
exclusive-child errors) is ahead of MDX and shortcodes. Theme DX for `:note`
is on par with Prose/swizzle wrap. Expressability is a **docs widget
language**, not a JSX document language: no inline widgets, no named slots,
no heading Prose, no JS tablist from `:tabs`. Stack consistency is high on
spelling and validation, and split on how a kind is introduced, how children
are constrained, and whether parent painters see records.[^design-research][^rocdown-readme][^site-ref][^lang-ref]

Theming architecture historically described CSS variables and a Rocci shell
only. Article-block painters are now a third shipped surface; they are still
not a general external theme-package interface.[^theming-arch][^site-ref]

[^design-research]: Pre-landing schema/renderer design, related-technology survey, and recommended architecture.
[^impl-plan]: Phased delivery; Phase 8 `@block` deferred; Phase 2 typed lists specified parent-visible records.
[^rocdown-readme]: Pack overlay, `[blocks]`, debug policy, custom kinds from headers, helpers-not-in-pack.
[^registry]: Builtin `KindSpec` child policy; pack inference; sugar headings excluded from `paints_as_widget`; pack kinds `accepts_markdown: true`.
[^docs-rs]: `validate_children` / `validate_accepted_children` for exclusive accepts and markdown rejection.
[^config-rs]: `BlocksConfig { pack, debug, override_map }` with `deny_unknown_fields`.
[^plan-rs]: Pack discovery, `BlockPainters` merge, generated widget arms, debug binding.
[^build-runtime]: `render_tab_items!` records then `html_from_records`; placeholder `# rocci-widget-kind-arms`.
[^docs-rocci]: Builtin `Tabs` extra argument is `content : Html`.
[^block-debug]: `data-rocci-block-debug` placeholder; unused for unknown kinds.
[^site-ref]: Public `[blocks]` table and pack convention.
[^docs-guide]: Override and custom-kind examples; tabs remain stacked sections without `tablist`.
[^lang-ref]: `:kind` inventory, delimiter XOR, `@use` interactive-only, static-page feature gate.
[^pages-guide]: `static` / `hydrate` / `live`; `@use` is a site-build error.
[^template-readme]: `|{ props }, content|`; Rocci `??` still stripped from patterns.
[^roc-defaults]: Type-position Roc defaults work on 2026-08-23; pattern `??` still rejected.
[^lsp-tests]: Kind, field, enum, and accepts-preferring completions inside `:tabs`.
[^lower-rs]: Standalone conservative HTML for article blocks beside the theme path.
[^imports-rs]: Every `@component` becomes a kind; builtin collision is an error.
[^markdown-first]: Mode changes at visible block boundaries only.
[^pure-render]: `@component` is a pure function to Html.
[^catalog-shell]: Rust owns catalog; Rocci owns visible chrome; no Rocci interpreter in Rust.
[^theming-arch]: Historical "CSS and shell, not presentation renderers" boundary.
[^markdoc-tags]: Schema `render`, `attributes`, `children`, `validate`.
[^mdx-using]: `components` prop including HTML names.
[^mdx-missing]: Default crash on missing custom MDX component.
[^nuxt-prose]: Filename convention overrides Markdown sugar.
[^docusaurus-swizzle]: Site theme files shadow preset components; wrap vs eject; unsafe internals.
[^vitepress-theme]: Extend default theme; alias-replace internals.
[^myst-syntax]: Directives as markup functions with args, options, body.
[^gutenberg-inner]: `allowedBlocks` / `parent` nested-block relationship.
[^typst-show]: Transformational show rules rebind element paint in-document.
