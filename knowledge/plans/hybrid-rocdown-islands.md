---
type: Implementation Plan
title: Hybrid Rocdown islands for CDN-static sites
description: "Phased delivery of CDN-static Rocdown HTML with dynamic Rocci components backed by a rocci or rocdown HTTP service. Phases 1–8 are on the hybrid branch (dual apply: widget forest plus island splice); phases 9–10 remain. Exploratory; not shipped."
tags: [domain/rocdown, domain/rocci, domain/runtime, integration/datastar, integration/roc, concern/rendering, concern/security, concern/packaging, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T14:10:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands for CDN-static sites
    author: process:cursor
    last_modified: 2026-08-19
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: article-rs
    resource: ../../crates/rocci-rocdown/src/article.rs
    title: Static-document feature gate
    author: process:git
    last_modified: 2026-08-17
  - id: site-rs
    resource: ../../crates/rocci-rocdown/src/site.rs
    title: RD2301 island and Datastar rejection
    author: process:git
    last_modified: 2026-08-18
  - id: plan-rs
    resource: ../../crates/rocci-rocdown/src/plan.rs
    title: Build planner, CSP, hashed assets
    author: process:git
    last_modified: 2026-08-19
  - id: build-rs
    resource: ../../crates/rocci-rocdown/src/build.rs
    title: Apply orchestration
    author: process:git
    last_modified: 2026-08-18
  - id: lowerer
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: Standalone mixed-document lowerer
    author: process:git
    last_modified: 2026-08-18
  - id: roc-build-runtime
    resource: ../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Current Roc apply runtime
    author: process:git
    last_modified: 2026-08-18
  - id: theme-rocci
    resource: ../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Site shell and module script injection
    author: process:git
    last_modified: 2026-08-18
  - id: view-rs
    resource: ../../crates/rocci-ui/src/view.rs
    title: ResourceView
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-host
    resource: ../../crates/rocci-roc-host/src/host.rs
    title: Wasmtime WASI apply
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-platform
    resource: ../../crates/rocci-roc-host/platform/main.roc
    title: Minimal wasm32 platform
    author: process:git
    last_modified: 2026-08-18
  - id: datastar-asset
    resource: ../../crates/rocci-cli/src/datastar_asset.rs
    title: Datastar.js staging for Rocci apps
    author: process:git
    last_modified: 2026-08-16
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Hybrid axes and island stages
    author: human:nils
    last_modified: 2026-08-16
  - id: interactive
    resource: ../../examples/rocdown/Interactive.rocdown
    title: Colocated @component and @on document
    author: process:git
    last_modified: 2026-08-18
  - id: server-actions
    resource: ../../docs/guides/server-actions.rocdown
    title: Server actions and Datastar
    author: human:nils
    last_modified: 2026-08-18
  - id: markdown-first
    resource: ../decisions/markdown-first-explicit-islands.md
    title: Markdown-first explicit islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: client-islands
    resource: ../decisions/client-behavior-islands.md
    title: Explicit client-behavior islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Pure render components
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: server-owned
    resource: ../decisions/server-owned-state.md
    title: Server-owned durable state
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: compiler-arch
    resource: ../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: format-arch
    resource: ../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: block-plan
    resource: generalized-rocdown-block-model.md
    title: Generalized Rocdown block model plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-plan
    resource: rocci-component-generation.md
    title: Chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: site-plan
    resource: rocci-dev-site.md
    title: rocci.dev site architecture
    author: process:codex
    last_modified: 2026-08-18
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Rocci and Rocdown language-development skill
    author: process:git
    last_modified: 2026-08-18
  - id: rocdown-reference
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: counter-example
    resource: ../../examples/rocdown-counter/index.rocdown
    title: Hybrid SQLite counter site page
    author: process:cursor
    last_modified: 2026-08-19
  - id: hybrid-guide
    resource: ../../docs/guides/hybrid-sites.rocdown
    title: Hybrid CDN plus island-service operator guide
    author: process:cursor
    last_modified: 2026-08-19
---

# Hybrid Rocdown islands for CDN-static sites

## Purpose and authority

This is the implementation plan for the [hybrid Rocdown islands
research](/research/hybrid-rocdown-islands.md). It is exploratory until a
human reviewer accepts a scope. It does not describe shipped
behavior.[^research][^rocdown-readme][^compiler-arch]

Do not start a phase until the user asks. Phases 1–8 are implemented on
branch `hybrid-rocdown-islands-implementation`. Phase 8 restored dual apply:
`static` pages keep the widget forest; hydrate/live pages splice island Html.
Do not start Phase 9 until the user asks. Do not put islands inside
article-block bodies in v1.[^block-plan][^language-dev]

## Goal

Ship a Rocdown site in which:

- **Static content** (Markdown, site chrome, hashed CSS/images) is
  ordinary files a CDN can serve with no Roc process at GET time.
- **Dynamic Rocci components** on those pages are backed by a **rocci
  or rocdown HTTP service** (`@on`, Datastar patches, server-owned
  state).
- Pages without components or handlers stay `script-src 'none'` and
  `connect-src 'none'`.

v1 uses existing `@component`, document-root tags, `@render`, `@css`,
`@roc` values, `@context`, `@init`, and `@on`. It does not add
`@island` grammar.[^research][^interactive][^server-actions][^format-report]

## Constraints that do not move

| Keep | Meaning for this plan |
| --- | --- |
| Markdown-first | Mode changes at document-root declarations |
| Pure `@component` | Renderer is a Roc function to Html; `@on` is the service |
| Server-owned durable state | The island service is authoritative; the CDN file is a snapshot |
| Rust catalog / Rocci shell | Markdown HTML and routes stay in Rust; chrome stays Rocci |
| Compile islands only where used | Do not lower Markdown to Roc for site pages |
| OKF Markdown-only | No components in `knowledge/**/*.md` |
| Visible handlers | A component without `@on` / Datastar actions is CDN-only |
| Per-page Datastar | Only hybrid pages get Datastar.js and a loosened CSP |
| Consume shipped widgets | Static pages keep main's `:name` / `DocsComponents` forest. Do not flatten `docs/` to a Markdown blob |

The generation host evaluates initial island Html. Block-model **syntax**
no longer blocks hybrid; the **apply architecture** on main does, until
Phase 8 restores the forest for `static` pages.[^generation-plan][^block-plan][^catalog-shell][^pure-render][^server-owned][^markdown-first]

## Non-goals (all phases)

- Reimplementing `:name` article widgets or the `DocsComponents` forest
  (already on `main`)
- Live Rocci hosts inside `:note` / other block bodies (v1)
- `@use` custom kinds on `rocdown build` / `rocdown check`
- Playground widget assets as island Datastar
- `@island` grammar or `*.client.js` custom-element runtime
- Compiling Markdown to Roc for `rocdown build`
- Compiling Roc to JavaScript
- Bundling Datastar Rocket
- Implicit hydration of every component
- Edge SSR / request-time Roc at the CDN
- Making the whole site a Datastar app[^site-plan]
- Vendor `_headers` / S3 metadata as required output
- Island-contributed catalog heading ids (v1)

Silently treating `@on` as static files is forbidden. A CDN-only build
may **omit** the service, but then live actions must not be advertised
as working.

## V1 contract

Changing one of these is a plan revision.[^research]

### Two artifacts

`rocdown build` of a hybrid site emits:

1. **CDN tree** — HTML documents, hashed assets, discovery files. Hybrid
   pages contain initial island Html and, if they have handlers, a
   Datastar script pointing at the **service origin**.
2. **Island service** — a Roc HTTP app (rocdown-generated from colocated
   `@on`, and/or an authored sibling `.rocci` app run with `rocci run`)
   that serves island fragments and patches.

A site with no hybrid pages emits only (1), as today.

### Page kinds

Do not store `dynamic: bool`. Store:

```text
static  — no Rocci components or handlers; CDN only
hydrate — Rocci components, no @on; initial Html on CDN; no Datastar
live    — Rocci components plus @on / Datastar actions; CDN Html + service
```

`rocdown check` accepts all three. A **CDN-only publish** (no service
deploy) may warn or error on `live` pages via an explicit flag; default
full hybrid publish accepts `live` and emits the service.

### Composition

This clause changed after the block-model cutover on `main` (plan
revision).

- **`static` pages:** keep main's `PlannedNode` forest. Apply walks
  fragments plus per-kind `DocsComponents.*` Rocci widgets. Do not
  collapse `docs/` to a single Markdown blob.
- **`hydrate` / `live` pages:** splice build-time island Html into the
  article in document order (placeholder nodes or a single pre-spliced
  `HtmlFile` segment). The theme `content` slot still uses the trusted
  unescaped-html bridge for those fragments. Do not put prose in
  `RocdownPages.roc`.[^roc-build-runtime][^compiler-arch][^block-plan]

Phases 1–6 on the hybrid branch used blob-only apply and stubbed widget
painting. That must not land on current `main`.

### Service program

Default: `@on` / `@context` / `@init` colocated in the `.rocdown` file
compile into one site island service (all `live` pages). Allowed:
handlers in a sibling `.rocci` module; the page only hosts the
component and posts to those routes. Routes stay explicit strings.[^interactive][^server-actions][^rocdown-readme]

Island GET in v1 is optional. Mutation `@on:post` (and friends) is
required for `live`. Initial Html comes from build-time render.

### Origins

- `site.base_url` — CDN canonical URLs.
- `[http] service_origin` (name flexible) — absolute origin for
  Datastar `connect-src` and action URLs when it differs from the page
  origin. Empty means same-origin (operator routes `/actions/` to the
  service).

### Outline

Rust slugifies Markdown headings only. Island Html does not add outline
entries in v1.

## Layer map

| Concern | Owner |
| --- | --- |
| `static` / `hydrate` / `live` | `article.rs`, `site.rs` |
| `Item::Block` / `:name[params]` | `scan.rs`, `parse.rs`, `ast.rs`, `registry.rs` |
| Widget forest (`static`) | `docs.rs` `PlannedNode`, `runtime/RocdownBuild.roc`, `DocsComponents.rocci` |
| Island placeholder / splice | `docs.rs` (`ArticleNode::Island`), `islands.rs`, `build.rs` |
| Theme content, CSP, Datastar script | `RocdownTheme.rocci`, `plan.rs`, `ResourceView` |
| Island Roc extract | beside `lower.rs` |
| Island service binary | `rocci-rocdown-cli` / `rocci-cli` |
| Datastar.js hash | planner + `datastar_asset` cache |
| Initial island Html | `rocci-roc-host` |
| Preview-as-site + island proxy | `dev.rs` |
| Public contract | crate README, `docs/reference/rocdown.rocdown`, `docs/reference/rocdown-site.rocdown`, `docs/guides/hybrid-sites.rocdown` |
| Knowledge architecture | **after** a phase ships |

## Delivery phases

Each phase is one mergeable change. Phases 1–8 below are **done on the
hybrid branch**. Continue at Phase 9.

### Phase 1 — Theme content slot for a single article blob

**Bound:** `rocdown build` writes page HTML whose `<article>` is the
Rust Markdown blob (no Rocci islands yet). Do not restore documentation-
widget painting.

**Does:**

- Apply reads one fragment file per page and passes it to
  `siteShell(view, content)` via `Html.dangerously_include_unescaped_html`.
  Write `${ROCDOWN_STAGING}/${output_path}`.[^roc-build-runtime][^theme-rocci]
- Native: `pf.Path` / `ROCDOWN_STAGING` as needed. Wasm: WASI read of
  that file (preopen workspace or copy the blob into staging). Do not
  embed article HTML in generated Roc.[^wasm-host][^wasm-platform]
- Prove with a Markdown-only fixture (no article widgets required):
  `index.html` contains the Markdown HTML, not an empty article.

**Does not:** relax `RD2301`, stage Datastar, or call documentation
widget components.

**Exit:** `cargo test -p rocci-rocdown` with Roc available shows
Markdown in built `index.html`. `--host wasm` writes the same body.

### Phase 2 — Page kind classification

**Bound:** replace the boolean gate with `static` / `hydrate` / `live`.
Still reject hydrate and live on site build until Phase 3/4.

**Does:**

- Classifier from document items: Rocci templates / `@render` /
  `@component` / `@css` / `@roc` => at least `hydrate`; `@on` /
  `@context` / `@init` or `import Datastar` => `live`.[^article-rs][^site-rs]
- Distinct diagnostics (split `RD2301` if tests allow). Record kind on
  `ResolvedPage` / inspect.
- `static` pages behave as today's allowlist minus any dependence on
  article-widget internals.

**Does not:** splice components or generate a service.

**Exit:** `cargo test -p rocci-rocdown`. `rocdown check docs` still
passes. Fixture with `@render` diagnoses `hydrate`; fixture with `@on`
diagnoses `live`.

### Phase 3 — Build-time splice of Rocci components (`hydrate`)

**Bound:** CDN pages may include pure Rocci. No handlers. No Datastar.js.

**Does:**

- Extract island Roc/Rocci; keep Markdown on the Rust path. Compile
  island modules only for `hydrate`/`live` pages. Evaluate initial Html
  with `rocci-roc-host`. Stitch into the article blob in document
  order.[^lowerer][^catalog-shell][^generation-plan]
- Allow file `@css` / `@roc` values used by those components.
- Site fixture in the spirit of `examples/rocdown/Guide.rocdown`
  (`@component` + `<FeatureCount />` + Markdown).
- `static` catalog/check still does not require Roc.

**Does not:** stage Datastar, compile `@on`, or parse `@island`.

**Exit:**

```text
cargo test -p rocci-rocdown
cargo test -p rocci-rocdown-cli
```

Built HTML contains Markdown and the component output, `script-src
'none'`, no `<script>`. `docs/` remains static.

### Phase 4 — Island service for `live` pages

**Bound:** colocated `@on` compiles to a rocci/rocdown HTTP service.
CDN HTML for those pages includes initial island Html plus Datastar.js
and `connect-src` to the service origin.

**Does:**

- Generate an island service from `@context` / `@init` / `@on` across
  `live` pages (one dispatcher). Reuse the standalone handler contract:
  mutation methods return Datastar patches; stable element ids.[^interactive][^server-actions]
- Hash Datastar.js into `/assets/` (from the existing pin/cache). Inject
  it only on `live` pages. Loosen CSP only there (`script-src` as
  required for Datastar expressions; `connect-src` = service
  origin).[^datastar-asset][^plan-rs][^view-rs]
- Config: `[http] service_origin` (optional). Empty => same-origin
  action URLs.
- CLI: `rocdown serve-islands` or `rocdown run DIR` starts the service
  beside the static tree. Exact flag in the phase micro-plan.
- Fixture derived from `Interactive.rocdown` as a **site page**, not
  only `rocdown run FILE`.
- Allow a sibling `.rocci` app as the service instead of generated
  handlers when configured; do not require it in the first fixture.

**Does not:** serve Markdown from the service; put Datastar on `static`
or `hydrate` pages; implement `@island`.

**Exit:** hybrid fixture: CDN `index.html` shows initial component Html
without the service; with the service, `@post` morphs the host. A
neighboring static page has no Datastar.js. `import Datastar` is no
longer a blanket site-build error for `live` pages.

### Phase 5 — CDN plus service publication contract

**Bound:** document and test the two-artifact deploy. No new runtime.

**Does:**

- README and `docs/reference/rocdown-site.rocdown`: CDN files vs island
  service; cache advice; `service_origin`; `static` / `hydrate` /
  `live`.
- Inspect/build report lists page kind, whether Datastar was emitted,
  service routes.
- Optional `islands.json` (or `pages.json` fields) with service routes
  for operators.
- Failed builds leave the previous CDN tree. Byte-stable rebuild when
  inputs are unchanged.[^build-rs][^format-report]
- Flag for CDN-only publish that errors on `live` pages (no silent
  dead buttons in production without a service).

**Does not:** add `@island` or article widgets.

**Exit:** public site reference describes the hybrid contract.
`rocdown build docs` stays `static`. Knowledge architecture/status
updated in a follow-up after this phase.

### Phase 6 — Preview, morph tests, public language notes

**Bound:** catch-up. No new capabilities.

**Does:**

- `rocdown run DIR` serves the CDN tree and the island service on one
  local origin (or documented pair). Reload on content and handler
  edits.
- Tests: two island instances, patch targets the right id, `hydrate`
  page has no Datastar, `live` page degrades without the service.
- Update crate README deferred list and `docs/reference/rocdown.rocdown`
  (`@island` still reserved).[^rocdown-reference][^format-arch]

**Does not:** start `@island` or block-model syntax.

**Exit:** `cargo test -p rocci-rocdown` and `cargo test -p
rocci-rocdown-cli`. `rocci-okf check knowledge --profile rocci`.

### Phase 7 — Rebase onto `BlockCall` / `:name[params]`

**Bound:** hybrid logic compiles against current `main`'s AST. No new
product behavior.

**Does:**

- Rebase `hybrid-rocdown-islands-implementation` onto `main`.
- Replace `Item::Docs` / `Item::Img` with `Item::Block`. Keep
  `ArticleNode::Island` for root `@render` / templates.
- Port `classify_document` / `PageKind` over main's `is_static_document`
  (`Item::Block` is static; `@use` stays a site-build error).
- Make `cargo test -p rocci-rocdown` compile. Widget and colon-syntax
  tests from `main` must still parse.

**Does not:** change island splice semantics, drop the widget forest, or
allow `@use` on `rocdown build`.

**Exit:** the branch builds. `rocdown check docs` passes. Hybrid
classifier tests compile against `Item::Block`.

### Phase 8 — Dual apply: widget forest plus island splice

**Bound:** `docs/` keeps Rocci widget chrome. Hydrate/live pages still
splice islands at build time.

**Does:**

- Restore main's `PlannedNode` forest and `render_forest!` for `static`
  pages (`RocdownBuild.roc`, `plan.rs`).
- After catalog resolve, splice evaluated island Html into hydrate/live
  articles (placeholder in the forest, or one pre-spliced `HtmlFile`
  segment for those pages only).
- Keep Datastar / `live_csp` / `serve-islands` / `--cdn-only` from
  Phases 4–5.

**Does not:** flatten all pages to a Markdown blob; put `@render` inside
`:note` bodies; compile Markdown to Roc.

**Exit:** `rocdown build docs` still paints `:note` / `:tabs`.
`examples/rocdown-hybrid` and `examples/rocdown-counter` build as
hydrate/live. Neighboring static pages have no Datastar.js.

### Phase 9 — Preview-as-site plus one-origin islands

**Bound:** combine main's file-under-site preview with hybrid's island
proxy. No new island capabilities.

**Does:**

- Keep `run_with_host_at` / persist-HTML-without-prior-build from
  `main`.
- Keep hybrid `dev.rs` island backend on the same origin for `rocdown
  run DIR`.
- Audit `keep_island_route` against preview-as-site routes so GET `/`
  stays CDN-owned.

**Does not:** serve Markdown from the island process.

**Exit:** `rocdown run docs` previews the site. `rocdown run
examples/rocdown-counter` proxies `/actions/` on that origin.

### Phase 10 — Re-prove examples and public contract

**Bound:** catch-up after rebase. No new capabilities.

**Does:**

- Green `cargo test -p rocci-rocdown` and `cargo test -p rocci-rocdown-cli`
  including colon-syntax and island tests.
- `rocdown check docs` and `rocdown build docs`.
- Confirm `examples/rocdown-counter` and
  `docs/guides/hybrid-sites.rocdown` still match the dual-apply
  contract.[^counter-example][^hybrid-guide]
- Align `docs/reference/rocdown.rocdown` so site builds describe
  `static` / `hydrate` / `live` on top of `:name[params]`.

**Does not:** start `@island` or islands-inside-widgets.

**Exit:** the validation commands below. Knowledge architecture/status
remain a follow-up after CI on that revision.

## Suggested merge order

Do **not** merge 1 → 6 onto current `main`. Rebase first: **7 → 8 → 9 →
10**. Phase 8 is the goal after the block-model cutover. Phase 3
(`hydrate`) remains useful without a service once it sits on the forest
apply path.

## Validation

```text
cargo test -p rocci-rocdown
cargo fmt --all -- --check
```

After public-contract or CLI changes:

```text
cargo test -p rocci-rocdown-cli
cargo run -q -p rocci-rocdown-cli -- check docs
cargo run -q -p rocci-rocdown-cli -- build docs
```

After knowledge edits:

```text
cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal
```

Do not log a phase complete until CI and Knowledge workflows succeed on
that revision.

## Follow-ons (not v1)

- First-class `@island` in `rocci-template`.[^client-islands]
- Live island GET refresh (re-fetch host Html from the service).
- Island Html in catalog outlines and in-page links.
- `@render` / Rocci hosts inside `:note` and other article-block
  bodies.[^block-plan]
- `@use` imported components as hydrate-page article kinds (static
  `rocdown build` stays a closed registry).
- CORS/cookie details for cross-origin CDN + service.
- Vendor cache-header adapters.

## Open questions that would still change the plan

1. Default `service_origin` empty (same-origin reverse proxy) versus
   requiring an explicit URL in site config?
2. One island service process for the whole site versus allowing only a
   sibling `rocci run` app in v1?
3. May `live` pages omit build-time island Html and show a no-JS
   fallback until the service responds? (Recommendation: always
   pre-render initial Html.)
4. Cross-origin Datastar: are action URLs absolute to `service_origin`,
   or relative with a `<base>` / meta? Absolute is simpler for CDN HTML.
5. Hydrate/live apply: island nodes inside the `PlannedNode` forest
   versus one pre-spliced `HtmlFile` per hybrid page? Forest-preserving
   placeholders keep `:note` beside `@render` on the same page; a single
   blob is simpler and matches Phases 1–6.

[^research]: CDN plus island service; article widgets out of scope.
[^rocdown-readme]: Standalone vs site; declarations; Datastar when actions exist.
[^article-rs]: Boolean static gate.
[^site-rs]: `RD2301` and Datastar import rejection.
[^plan-rs]: Default CSP and hashed assets.
[^build-rs]: Apply and atomic commit.
[^lowerer]: Standalone splice for `rocdown run FILE`.
[^roc-build-runtime]: Apply runtime; hybrid branch is blob-only, main is widget forest.
[^theme-rocci]: CSP and script injection.
[^view-rs]: `ResourceView` fields.
[^wasm-host]: Staging preopen.
[^wasm-platform]: No Path on wasm32 platform.
[^datastar-asset]: App Datastar staging; site must hash its own copy for `live`.
[^format-report]: Hybrid axes; JS only when referenced.
[^interactive]: Colocated `@component` + `@on` fixture.
[^server-actions]: Handler and patch contract.
[^markdown-first]: Explicit executable regions.
[^client-islands]: `@island` not v1.
[^catalog-shell]: Compile islands only where used.
[^pure-render]: Components stay Roc functions to Html.
[^server-owned]: Service owns durable state.
[^compiler-arch]: Trusted HTML bridge.
[^format-arch]: `@island` unimplemented.
[^block-plan]: Article widgets; `:name[params]` and forest apply on main.
[^generation-plan]: Host for initial island Html.
[^site-plan]: Site reading stays a static catalog.
[^language-dev]: Language-skill boundary.
[^rocdown-reference]: Update after behavior ships.
[^counter-example]: SQLite-backed live page on the hybrid branch.
[^hybrid-guide]: Two-artifact deploy runbook on the hybrid branch.
