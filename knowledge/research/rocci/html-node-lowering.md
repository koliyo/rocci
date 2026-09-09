---
type: Research Report
title: Html.element lowering is the composition API, not the performance bottleneck
description: "Generated Roc already emits Html.element trees. Compile cost is that emit shape; runtime is whichever Html.roc is linked. Dual lowering modes are the expensive option. Measure, then fuse static chunks or unify the two Html backends."
tags: [domain/rocci, domain/rocdown, domain/runtime, integration/roc, concern/performance, concern/rendering, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-09-09T09:53:00Z }
stale_after: 2026-12-09
authority: exploratory
owners: [human:nils]
sources:
  - id: plan
    resource: ../../plans/rocci/html-node-lowering.md
    title: Measure, then fuse or unify Html emit
    author: process:cursor
    last_modified: 2026-09-09
  - id: lower-html
    resource: ../../../crates/rocci-template/src/lower/html.rs
    title: Constructor-call emit for tags, interpolations, fragment CSS wrap
    author: process:git
    last_modified: 2026-08-31
  - id: lower-mod
    resource: ../../../crates/rocci-template/src/lower/mod.rs
    title: LowerOptions.html_type annotates component signatures
    author: process:git
    last_modified: 2026-09-09
  - id: lower-emitter
    resource: ../../../crates/rocci-template/src/lower/emitter.rs
    title: html_type on props and body parameters
    author: process:git
    last_modified: 2026-09-09
  - id: platform-html
    resource: ../../../crates/rocci-platform/platform/Html.roc
    title: HtmlNode tagged union plus render walk
    author: process:git
    last_modified: 2026-09-03
  - id: cli-html
    resource: ../../../crates/rocci-cli/runtime/Html.roc
    title: Product wrapper; fragment serializes then wraps Raw
    author: process:git
    last_modified: 2026-08-30
  - id: ui-html
    resource: ../../../crates/rocci-ui/runtime/Html.roc
    title: String Html used by theme apply, tests, playground snapshot
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-html
    resource: ../../../crates/rocci-rocdown/runtime/Html.roc
    title: Same string Html as rocci-ui
    author: process:git
    last_modified: 2026-08-17
  - id: theme-plan
    resource: ../../../crates/rocci-rocdown/src/plan/theme.rs
    title: Theme painters lower with html_type Str
    author: process:git
    last_modified: 2026-08-31
  - id: md-lower
    resource: ../../../crates/rocci-rocdown/src/lower/markdown.rs
    title: Hydrate Markdown still emits Html.element per node
    author: process:git
    last_modified: 2026-08-31
  - id: article-render
    resource: ../../../crates/rocci-rocdown/src/docs/render.rs
    title: Static catalog article HTML is Rust strings
    author: process:git
    last_modified: 2026-08-31
  - id: datastar-roc
    resource: ../../../crates/rocci-cli/runtime/Datastar.roc
    title: patch_elements renders then string-strips style tags
    author: process:git
    last_modified: 2026-08-30
  - id: source-map
    resource: ../../../crates/rocci-template/src/source_map.rs
    title: StaticMarkup and TextExpression segments on generated Roc
    author: process:git
    last_modified: 2026-08-25
  - id: golden
    resource: ../../../crates/rocci-template/tests/fixtures/all_syntax.roc
    title: AllSyntax golden is verbose Html.element trees
    author: process:git
    last_modified: 2026-09-09
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Components lower to ordinary Roc Html functions
    author: process:git
    last_modified: 2026-09-09
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: "@component is a pure function to Html"
    author: process:okf-migration
    last_modified: 2026-08-31
  - id: rust-catalog
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Static prose stays Rust; chrome stays Rocci
    author: process:okf-migration
    last_modified: 2026-08-24
  - id: native-research
    resource: roc-native-template-compiler.md
    title: Lowering is string emit of Html.element calls
    author: process:cursor
    last_modified: 2026-09-03
  - id: sse-style
    resource: sse-patch-style-targets.md
    title: embed_css style sibling is a wire-format problem
    author: process:cursor
    last_modified: 2026-08-30
  - id: okf-cost
    resource: ../okf/okf-compile-render-cost.md
    title: Baking page HTML into generated Roc missed the renderer cache
    author: process:cursor
    last_modified: 2026-08-25
  - id: inspector-plan
    resource: ../../plans/rocci/inspector-source-views.md
    title: Inspector already shows source, AST, generated Roc, generated HTML
    author: process:cursor
    last_modified: 2026-08-19
  - id: dispatch
    resource: ../../../crates/rocci-cli/src/dispatch/mod.rs
    title: Handlers Html.render or Datastar.patch_elements
    author: process:git
    last_modified: 2026-09-04
  - id: playground-html
    resource: ../../../crates/rocci-cli/src/playground_html.rs
    title: Local playground snapshots via rocci-ui string Html
    author: process:git
    last_modified: 2026-08-30
---

# Html.element lowering is the composition API, not the performance bottleneck

Exploratory. Implementation:
[measure, then fuse or unify Html emit](/plans/rocci/html-node-lowering.md).[^plan]

## Claim

There are **two layers**, and mixing them produces the wrong plan.

1. **Emit shape** (what the Rust lowerer writes into `.roc`): today this is constructor calls (`Html.element`, `Html.text`, `Html.fragment`, `Html.attribute`). That shape is what Roc must parse and type-check.[^lower-html][^native-research][^golden]
2. **Html runtime** (what those calls do): product apps link a **node tree** (`HtmlNode` then `render`); theme apply, `rocci test`, and local playground snapshots link a **string** Html where `element` concatenates and `render` is identity.[^platform-html][^cli-html][^ui-html][^rocdown-html][^theme-plan][^playground-html]

Switching the runtime from nodes to strings **does not shrink generated Roc**. Dual *lowering* modes (node-shaped Roc for debug, string literals for release) would. That second fork is the expensive option. The inspector already has original source, AST, generated Roc, and generated HTML; it does not need a second compiler emit to “see nodes.”[^inspector-plan]

The wire is already a string: `Html.render` for documents and `Datastar.patch_elements` after `render_without_doc_type`.[^dispatch][^datastar-roc]

## Current emit

A component body becomes nested constructor calls. Adjacent static text is one `Html.text` per source node, not a fused markup literal. Interpolations that are not a body parameter become `Html.text(expr)`. Multi-root bodies and colocated CSS become `Html.fragment([style, content])`. `@for` becomes `List.map` of nodes, wrapped in `fragment` when the parent expects one node.[^lower-html][^golden]

`LowerOptions.html_type` only changes **signatures** (`… -> Html.Node` versus `… -> Str`). Theme painters pass `Str`; standalone, islands, and `rocci run` pass `Html.Node`. The constructor calls stay the same.[^lower-mod][^lower-emitter][^theme-plan]

AllSyntax source is about 3.7kB; the golden lowered Roc is about 10.7kB of `Html.element` trees. That ratio is the compile-time suspect, not the tagged union behind `Html.element`.[^golden]

## Current runtimes

Platform `Html` is a tagged union (`Text`, `Raw`, `Element`, `VoidElement`) plus a recursive `render_without_doc_type`. Escaping walks UTF-8 bytes with `List.fold` and per-entity `concat`. Named helpers (`Html.div`) still call `element`.[^platform-html]

The CLI wrapper keeps that tree **except** `Html.fragment`, which immediately serializes:

```text
fragment = |nodes|
    dangerously_include_unescaped_html(render_fragment(nodes).to_str())
```

Any CSS-wrapped or multi-root component therefore **builds nodes and throws the tree away** on the product path. What HTTP and Datastar see is already a `Raw` string.[^cli-html][^lower-html][^sse-style]

`rocci-ui` / `rocci-rocdown` Html is strings from the first constructor. `element` interpolates tag, joined attributes, and joined children. `fragment` is `Str.join_with`. `render` is identity. Boolean attributes and void tags already **diverge** from the platform module (presence versus empty value; trailing slash on void tags; doctype newline). Dual runtimes are not a clean debug/release pair; they are two implementations of one call shape.[^ui-html][^rocdown-html][^platform-html]

Static documentation does **not** lower prose to Roc `Html.element`. Rust `render_article` owns article HTML; the Rocci theme receives view records. Hydrate / standalone Markdown still emits per-node `Html.element` / `Html.text`. Do not reverse the catalog split to “fix” Html performance.[^rust-catalog][^article-render][^md-lower]

OKF already learned that baking page HTML into generated Roc makes compile scale with prose and misses the renderer cache. That is a content-ownership lesson, not a reason to emit node trees for static pages.[^okf-cost][^rust-catalog]

## Performance split

| Cost | Driven by | Node `Html.roc` | String `Html.roc` | Fused string *emit* |
| --- | --- | --- | --- | --- |
| `roc` parse / typecheck | Size of generated constructor AST | Same as string runtime | Same as node runtime | Smaller Roc AST |
| Runtime alloc | Tree + render, or concat at each call | Extra tagged unions; `fragment` renders early | Intermediate strings at each `element` | Mostly hole escaping + join |
| Generated bytes | Verbose `Html.element("div", …)` | Same | Same | Fewer calls, larger literals |
| Inspect generated Roc | Constructor nesting | Readable tree | Readable tree | Markup soup unless pretty-printed |
| Source maps | Per constructor / literal | Fine-grained `StaticMarkup` | Same | Coarser unless holes keep segments |

Hypotheses, not measurements:

- **Compile time** tracks emit verbosity. AllSyntax-scale goldens and chrome such as `NavList` are the right suspects. Changing `Html.roc` alone will not move `roc check` much.
- **Request runtime** for CSS-injected components is already “build then serialize” on the node path. String Html skips the tree. Neither is a rope; both join strings bottom-up.
- **Live polls** pay render on every tick. `Datastar.patch_elements` then splits the serialized HTML to drop style tags. A fused emit can omit those siblings instead of stripping them later.[^datastar-roc][^sse-style]
- Platform `escape_html_bytes` is a separate micro-cost (`fold` plus per-match `concat`). Worth fixing if a profile shows it; it is not the emit-shape decision.

No wall-clock comparison of the two `Html.roc` backends, or of fused emit, exists in this bundle. Phase 0 of the plan is that measurement.[^plan]

## Options

### A. Keep constructor emit; pick one Html runtime

Leave `html.rs` as `Html.element` trees. Replace platform `HtmlNode` with the string module (or the reverse) so theme, tests, playground, and `rocci run` agree.

**Pays:** one semantics for void tags, booleans, doctype; simpler mental model; possible runtime win if the tree+`fragment` path shows up in profiles.

**Does not pay:** Roc compile time of generated modules.

**Fits** `@component` as a function returning Html, whether Html is `Node` or `Str`.[^pure-render][^template-readme]

### B. Static chunk fusion in the lowerer (one emit)

Rust walks the template AST, escapes static text and trusted tags at compile time, and emits string literals (or `Html.dangerously_include_unescaped_html` of those literals) around dynamic holes (`Html.text(expr)`, attribute expressions, `@if` / `@for` / `@match`, component calls). Component boundaries stay ordinary function calls returning Html.

**Pays:** the compile-time AST and a large fraction of runtime constructor churn. Same approach as compiled HTML templates that still splice expressions.

**Costs:** source-map granularity on fused literals; goldens rewrite once; XSS is a lowerer bug if a hole is not escaped. Inspector “generated Roc” looks less like a DOM tree.

**Does not require** a second product mode. Debug views can pretty-print the template AST or the rendered HTML the inspector already has.[^inspector-plan][^source-map]

### C. Two lowering modes (nodes for debug, strings for release)

Debug emit stays today’s `Html.element` trees. Release emit is fused strings or identity `Str` concat.

**Pays:** pretty generated Roc while iterating.

**Costs:** two goldens, two source-map shapes, CLI/env flags, drift (a bug that exists only in release), and tests that grep `Html.element`. The inspector already separates source / AST / Roc / HTML. This is the option to reject unless Phase 0 shows debug emit is required for a tool that cannot use the AST.

### D. Emit Roc string interpolation only

No `Html.element` in generated Roc. Everything is `"<div>…${escape(x)}…"</div>"` (or `Str.concat`).

**Pays:** smallest Roc AST if fusion is aggressive.

**Costs:** composition with hand-written `Html.element` in `main.roc` / Snake; body parameters that are already Html values; `@for` maps of markup; Datastar action attributes. Reinvents the Html module in the lowerer. Worse inspectability than B unless B is done first.

### E. Status quo

Constructor emit plus two Html backends plus `html_type` annotations. Works. Pays dual-runtime drift and an unmeasured node+`fragment` tax on every CSS-wrapped component.

## Dual-mode specifically

Two *runtimes* already exist (`Html.Node` versus `Str`). That is enough dual-mode for debugging: playground and `rocci test` use string Html; `rocci run` uses nodes. Adding two *emits* stacks a compiler fork on top of a runtime fork.

If a human wants “nodes for debugging,” prefer:

1. Inspector AST and generated-HTML tabs (shipped direction).[^inspector-plan]
2. Optional pretty-printer from the template AST (not a second lowerer).
3. Keep constructor emit as the single product lowerer until fusion is measured.

Do not ship `--html-nodes` / `--html-strings` as the first performance project.

## Exploratory recommendation

1. **Keep constructor-call lowering as the public composition API** until fusion is measured. `@component` stays a pure function to Html. Do not make authors write markup strings in Roc.[^pure-render][^template-readme]
2. **Do not add dual lowering modes** as the default plan. Use inspector views for debug structure.
3. **Measure** compile time and render time on AllSyntax, Counter, and a theme painter, against both Html backends and a hand-fused Hello twin.[^plan]
4. If compile time dominates, implement **B (static fusion)** as one emit, keep `Html.text` / component calls at holes, rewrite goldens once.
5. If only runtime of CSS-wrapped fragments dominates, implement **A**: string Html on the product path (or stop `fragment` from eagerly serializing if a real forest is still wanted). Unify void-tag and boolean-attribute semantics in the same change.
6. Leave static documentation on the Rust article renderer. Do not lower catalog prose to Roc Html to “use nodes.”[^rust-catalog][^okf-cost]

These are hypotheses until Phase 0 numbers exist. Do not treat this record as shipped behavior.

[^plan]: Paired implementation plan; no phase started.
[^lower-html]: Tags, interpolations, `@for` maps, and CSS `fragment` wrap.
[^lower-mod]: `LowerOptions.html_type` default `Html`; consumers override.
[^lower-emitter]: Signature `props, Html, … -> Html` uses `html_type`.
[^platform-html]: `HtmlNode` constructors and `render_without_doc_type`.
[^cli-html]: Wrapper `fragment` renders then `Raw`.
[^ui-html]: String constructors; `render` is identity.
[^rocdown-html]: Copy of the string module for theme apply.
[^theme-plan]: `html_type: "Str"` for painters.
[^md-lower]: Per-Markdown-node `Html.element` / `Html.text` on the Roc path.
[^article-render]: `render_article` returns `String` in Rust.
[^datastar-roc]: Serialize node, then `drop_style_elements` on the string.
[^source-map]: `OriginKind::StaticMarkup` and `TextExpression`.
[^golden]: Nested `Html.element` / `Html.fragment` in the AllSyntax fixture.
[^template-readme]: `@component` lowers to an ordinary Roc Html function.
[^pure-render]: Implemented decision: explicit values in, Html out.
[^rust-catalog]: Prose and catalog stay Rust; chrome stays Rocci.
[^native-research]: Product lowering is string emit of constructor calls.
[^sse-style]: Style siblings are a patch-target issue, not a DOM API.
[^okf-cost]: Page HTML baked into Roc missed cache and scaled with prose.
[^inspector-plan]: Dev panel views for source, AST, Roc, HTML.
[^dispatch]: `html_ok(Html.render(html))` and `Datastar.patch_elements`.
[^playground-html]: Stages `rocci_ui::HTML_ROC` and `Html.render` to stdout.
