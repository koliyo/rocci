---
type: Research Report
title: Rocci components inside the content generation pipeline
description: Evidence for replacing Rust-authored HTML chrome with Rocci components, including native subprocess versus Wasmtime hosts, two-tier generated-Roc and compiled-artifact caching, and native glue as a later host.
tags: [domain/rocci, domain/rocdown, domain/rocci-okf, integration/roc, concern/rendering, concern/performance, concern/architecture, concern/caching]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell decision
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: generator
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: theming
    resource: ../../architecture/theming.md
    title: Current Rocci theming surfaces
    author: process:okf-phase-4
    last_modified: 2026-08-18
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: static-okf
    resource: ../../decisions/static-okf-boundary.md
    title: Strict OKF Markdown boundary
    author: process:okf-migration
    last_modified: 2026-08-17
  - id: cli-plan
    resource: ../../plans/shared/cli-entry-points.md
    title: CLI entry points plan
    author: process:cursor
    last_modified: 2026-08-18
  - id: ui-readme
    resource: ../../../crates/rocci-ui/README.md
    title: rocci-ui view records
    author: process:git
    last_modified: 2026-08-18
  - id: ui-view
    resource: ../../../crates/rocci-ui/src/view.rs
    title: Domain-neutral view records
    author: process:git
    last_modified: 2026-08-18
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown Roc invocation and watch hash
    author: process:git
    last_modified: 2026-08-18
  - id: build-runtime
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Rocdown generated-page assembly runtime
    author: process:git
    last_modified: 2026-08-17
  - id: docs-rs
    resource: ../../../crates/rocci-rocdown/src/docs.rs
    title: Rust article and docs HTML renderer
    author: process:git
    last_modified: 2026-08-18
  - id: docs-rocci
    resource: ../../../crates/rocci-rocdown/templates/DocsComponents.rocci
    title: Rocci documentation widgets
    author: process:git
    last_modified: 2026-08-17
  - id: theme-rocci
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Rocdown documentation shell
    author: process:git
    last_modified: 2026-08-18
  - id: site-layouts
    resource: ../../../site/theme/Layouts.rocci
    title: rocci.dev site layouts
    author: process:git
    last_modified: 2026-08-18
  - id: okf-presentation
    resource: ../../../crates/rocci-okf/src/presentation.rs
    title: OKF review HTML renderer
    author: process:git
    last_modified: 2026-08-18
  - id: okf-dev
    resource: ../../../crates/rocci-okf/src/dev.rs
    title: OKF preview CSS and TOC shell
    author: process:git
    last_modified: 2026-08-18
  - id: okf-toc-js
    resource: ../../../crates/rocci-okf/src/toc.js
    title: OKF table-of-contents scroll script
    author: process:git
    last_modified: 2026-08-18
  - id: standalone-lower
    resource: ../../../crates/rocci-rocdown/src/lower.rs
    title: Standalone Rocdown TOC emission
    author: process:git
    last_modified: 2026-08-18
  - id: theme-toc-js
    resource: ../../../crates/rocci-theme/src/themes/toc.js
    title: Standalone Rocdown table-of-contents scroll script
    author: process:git
    last_modified: 2026-08-17
  - id: deps-check
    resource: ../../../rocci-ops/src/rocci_ops/workspace_deps.py
    title: Workspace dependency-direction check
    author: process:git
    last_modified: 2026-08-18
  - id: template-readme
    resource: ../../../crates/rocci-template/README.md
    title: Rocci template crate contract
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-cache
    resource: ../../../crates/rocci-cli/src/datastar_asset.rs
    title: Existing ~/.rocci/cache directory and SHA-256 integrity check
    author: process:git
    last_modified: 2026-08-16
  - id: roc-glue
    resource: https://github.com/roc-lang/roc/blob/main/src/glue/README.md
    title: Roc glue ABI generator
    author: organization:roc-lang
  - id: roc-platform-rs
    resource: https://github.com/lukewilliamboswell/roc-platform-template-rust
    title: Rust Roc platform template
    author: human:luke-willis-boswell
  - id: roc-faq
    resource: https://www.roc-lang.org/faq
    title: Roc platforms FAQ
    author: organization:roc-programming-language-foundation
  - id: roc-wasi
    resource: https://github.com/ostcar/roc-wasi-platform
    title: Roc WASI platform and roc build --target wasm32
    author: human:ostcar
---

# Rocci components inside the content generation pipeline

## Scope

This record is exploratory evidence, not a change to the shipped
Rust-catalog/Rocci-shell contract and not a description of current
architecture. It asks whether more of the HTML that Rust currently concatenates
should be authored as `@component` functions, what that implies for the Roc
compiler and runtime, and whether a first-party component library can be shared
by Rocdown sites, standalone Rocdown, and the OKF viewer.[^catalog-shell][^generator]

Two parallel drafts explored the same question. This record keeps the claims
that match current code and Roc's public compiler model, and records where the
drafts diverged.

## Current generation boundary

Rocdown already splits work along three lines:

1. Rust owns discovery, identity, routing, navigation, validation, Markdown
   article HTML, artifact planning, and host orchestration.[^catalog-shell][^generator]
2. Rocci owns visible site chrome (`RocdownTheme.rocci`) and `:kind` widget
   markup (`DocsComponents.rocci`), compiled once per build and applied to
   structured `PageView` records plus typed segment records.[^theme-rocci][^docs-rocci][^ui-readme]
3. The Roc build runtime re-enters `Html` from trusted fragment files with
   `Html.dangerously_include_unescaped_html`, then composes those fragments
   with documentation components and the theme.[^build-runtime][^generator]

That last step is an internal trusted-artifact bridge, not an author-facing
raw-HTML feature. Its safety depends on escaping in every Rust renderer before
the bridge.[^generator]

OKF preview and knowledge HTML currently stay entirely in Rust. `rocci-okf`
must not depend on Rocdown packages, knowledge records must not execute Roc or
Rocci, and CLI polish is not supposed to reopen a `rocci-okf` → Rocdown
presentation adapter.[^static-okf][^cli-plan][^deps-check]

`rocci-ui` currently exposes domain-neutral view records (`PageView`,
`SiteView`, `LaneView`, `NavItemView`, `OutlineView`, `BreadcrumbView`,
`ResourceView`, `CollectionItemView`) and HTML escaping. It does not own
templates, `ConceptView`, `ReviewView`, or `StatCardView`. OKF stat cards live
in `rocci-okf`.[^ui-readme][^ui-view][^okf-presentation]

## Where Rust still writes HTML

The remaining Rust HTML is not one pile. Four layers behave differently.

### Markdown bodies

`article.rs` and `docs.rs` turn semantic Markdown nodes into escaped HTML
fragments, including optional Tree-sitter highlighting. Site builds then hand
those fragments to Roc as files. This is the part the catalog-shell decision
rejected moving into Roc: prose must not become generated Roc modules, and
catalog checks must not require Roc.[^docs-rs][^catalog-shell]

### Structured article-block wrappers

Site builds already paint asides, steps, tabs, cards, and related widgets in
`DocsComponents.rocci`. A parallel Rust renderer in `docs.rs::render_docs`
still emits the same class names for `article_html`, tests, and the empty-segment
fallback. That is duplicated markup, not a second product surface.[^docs-rs][^docs-rocci]

### Documentation chrome

Multi-page Rocdown sites render sidebar, breadcrumbs, previous/next, and "On
this page" in Rocci (`RocdownTheme.rocci` and the rocci.dev `Layouts.rocci`).
Standalone Rocdown emits a `.rd-toc` navigator from `lower.rs`. The OKF review
viewer concatenates a matching `.rd-toc` / `.rd-shell` in
`presentation.rs`, with copied CSS in `dev.rs` and a copied `toc.js`. The
class names match standalone Rocdown by intent; the site shell uses a
different `.outline` vocabulary.[^theme-rocci][^site-layouts][^standalone-lower][^okf-presentation][^okf-dev][^okf-toc-js][^theme-toc-js]

### Product-owned OKF UI

Concept badges, provenance, sources, the priority-1 queue, and the review
page are OKF governance chrome. The CLI plan keeps that HTML in `rocci-okf`,
not in `RocdownTheme.rocci`. Domain-neutral view records in `rocci-ui` remain
the allowed shared data shape.[^cli-plan][^ui-readme][^okf-presentation]

## Feasibility

Moving more *chrome and widgets* into Rocci is feasible with the current
compiler. `@component` already lowers to ordinary Roc functions from explicit
values to `Html`, so a navigation list or table of contents is the same
abstraction the documentation shell already uses.[^pure-render][^template-readme]

Moving *Markdown bodies* into Rocci is feasible technically and a poor fit for
the project contract. It would reintroduce the rejected Roc-first static-site
shape: Roc compilation scaling with prose, catalog checks requiring Roc, and
Rust reimplementing less of the host work it already owns.[^catalog-shell]

Sharing components between Rocdown and OKF is feasible only if the shared
files live in base Rocci (`rocci-ui` or a sibling crate). `rocci-okf` cannot
import Rocdown templates without reversing the frozen dependency rule. Sharing
does not require the two products to look identical; it requires a shared
view-record and render function with product CSS applied around it.[^deps-check][^cli-plan][^theming]

OKF preview can consume Rocci without executing knowledge records. The
renderer would be a first-party tool artifact, like today's `RocdownBuild.roc`,
not content from `knowledge/**/*.md`.[^static-okf]

Names such as `DocumentationNavigationPanel` overstate the shared surface. OKF
has an "On this page" outline and no documentation sidebar. Shared primitives
should follow `PageView` fields (`PageOutline`, `NavList`, `Breadcrumbs`);
product layouts compose them. A documentation-named panel can exist as a
Rocdown layout, not as the shared library's root type.[^ui-view][^okf-presentation][^theme-rocci]

## Roc compiler versus Roc runtime

These are different embeddings.

### Subprocess compiler (shipped)

Rocdown already shells out to `roc` / `roc build` with `basic-cli` as the
platform. Watch mode hashes generated Roc (runtime, theme modules,
`RocdownPages.roc`, `main.roc`) and skips recompile when that hash is
unchanged, then reruns the applicator binary against staging files.[^build-rs]

`rocci-template` deliberately does not invoke Roc. Template compilation is
Rust; Roc type-checking and codegen stay in the `roc` binary.[^template-readme]

### Glue and a Rust host (future native in-process option)

`roc glue` generates host-language ABI bindings so a platform written in Rust,
Zig, or C can call compiled Roc and exchange `Str`, `List`, records, and tag
unions with compiler-committed layout and refcounting. Rust is a first-class
glue target. Glue does not embed the compiler; it embeds the *compiled
application* inside a host that owns `main`.[^roc-glue][^roc-platform-rs][^roc-faq]

That shape is a later native host (`roc build --lib` plus `dlopen`), not part
of the current delivery. The in-scope in-process host is Wasmtime loading a
`wasm32` module of the same `render : Str, Str -> Str` program.

`Html` from `basic-cli` is the wrong type to share across a host boundary
unless the custom platform *is* the HTML runtime. Prefer returning `Str`.

### Wasmtime and `roc build --target wasm32` (in-scope host)

Roc can emit WASI modules with `roc build --target wasm32`. Those modules
run in Wasmtime today in third-party WASI platforms.[^roc-wasi] That is still
whole-program compilation with a different host: it does not create
per-module linking, and it is not in the current Rocdown builder.

A Rocci wasm host therefore needs a small custom platform. `basic-cli` is a
native CLI platform. Latency numbers from the parallel architecture draft
are not measured in this repository.

### Embedding the Roc compiler in Rust (not available as a library)

The Roc compiler is a Zig program invoked as `roc`. There is no supported
"call the compiler crate from Rust" API, and the project does not plan to
self-host the compiler in Roc. Calling `roc` from Rust is what Rocdown
already does. Linking compiler internals into `rocci-cli` would be a new
unofficial integration, version-locked to those internals.[^roc-faq]

## Caching and incremental compilation

Roc compiles a whole application plus platform, then LLVM-optimizes that
program. There is no public model of compiling `NavList.rocci` and
`PageOutline.rocci` to independently loadable objects and dynamically linking
them at preview time the way a JS bundler links chunks.

What does exist:

- Whole-program `roc build` to an executable, which Rocdown uses.
- `roc build --target wasm32` to a WASI module consumed by Wasmtime.
- `roc build --lib` to a shared library consumed by a glue host (future).
- Prebuilt platform hosts, so application rebuilds do not rebuild the host
  toolchain.
- Content kept *out* of Roc source so watch mode can apply without
  recompile. Fragment HTML files already work this way. `RocdownPages.roc`
  still embeds page view records, so title, sidebar, and outline edits change
  the Roc hash and force a recompile.[^build-rs][^build-runtime]
- `rocci-template` lowering of `.rocci` to Roc in Rust, which does not invoke
  `roc` and can be cached independently of LLVM codegen.[^template-readme]

Rocdown's current hash is session-local: generated Roc bytes in a temp
workspace, no persistence of `.roc` files or of the `apply` binary.[^build-rs]
Datastar already persists downloads under `~/.rocci/cache` with a SHA-256
sidecar; renderer caching should reuse that root and the `ROCCI_CACHE`
override, not invent a third home-directory convention.[^datastar-cache]

A durable cache therefore has two tiers of *whole programs*:

1. Generated Roc (`~/.rocci/cache/roc/<gen-hash>/`), keyed by template crate
   identity plus canonical `.rocci` / runtime bytes.
2. Compiled artifacts (`~/.rocci/cache/renderers/<compile-hash>/`), keyed by
   `gen-hash` plus `roc version`, target (`native:<triple>` or `wasm32`),
   opt level, and platform identity. Native `apply` and `components.wasm`
   are different compile hashes of the same generated Roc.

Identity is content hash. mtime plus file length is only a fingerprint so
unchanged files need not be re-read. Manifest `created_at` / `last_used_at`
timestamps support LRU and diagnostics; they must not decide hits. Artifact
SHA-256 must be verified on load.

It is not a per-module object cache. Shared `rocci-ui` templates do not
produce a single machine-wide cache key by themselves: two applicators that
import those templates but differ in shell, platform, or extracted CSS are
different programs. Cross-project hits occur when the *whole renderer
program* is identical, for example two Rocdown sites using the same theme
modules.

Putting compiled programs in `~/.rocci` only pays off if renderer source
changes rarely and page data is passed in at apply time (files, JSON, or
host-imported strings), not baked into generated Roc.

Product CSS should stay out of a *shared* renderer key. An applicator that
inlines extracted theme CSS into its own program hashes that CSS as part of
*that* program.

## Performance implications

Compile cost dominates. A full `rocdown build` always compiles. Watch mode
already treats Roc codegen as the expensive step and avoids it when generated
Roc is stable. Pushing more widgets into the already-compiled theme module is
nearly free at apply time: one more pure function in the same binary.

Apply cost is running that binary over fragment files, or instantiating the
same program in Wasmtime. In-process wasm removes process spawn; native glue
would later remove WASI and ABI translation. Neither changes the rule that
compilation must not run per page or per request.

The costly mistake is compiling Roc per page or per request. OKF preview today
is Roc-free and cheap to rebuild on file watch. Introducing Rocci there is
acceptable only if compilation happens at `rocci-okf` build time or on first
use of an unchanged renderer cache, not on every concept save. Host B is the
path that keeps run time Roc-free after a wasm artifact exists.

Catalog and parser tests must stay Roc-free. Any new Rocci path needs the same
split Rocdown already has: Rust tests for data, optional Roc tests behind
`ROCCI_REQUIRE_ROC`.[^catalog-shell][^template-readme]

## Duplication that a library would actually remove

Demonstrated copies today:

- "On this page" markup and scroll script: standalone Rocdown, OKF viewer, and
  a third Rocci outline in documentation shells.[^standalone-lower][^okf-presentation][^theme-rocci][^okf-toc-js][^theme-toc-js]
- Documentation site chrome: `RocdownTheme.rocci` versus `site/theme/Layouts.rocci`
  both loop `view.sidebar`, `view.breadcrumbs`, and `view.outline`.[^theme-rocci][^site-layouts]
- `:kind` wrappers: `DocsComponents.rocci` versus `docs.rs::render_docs`.[^docs-rocci][^docs-rs]

Not demonstrated, and easy to over-extract:

- OKF has no documentation sidebar. A `DocumentationNavigationPanel` would be
  shared by two Rocci shells, not by OKF.
- OKF governance cards, trust badges, and review queues have no Rocdown twin.
  `StatCardView` already lives in `rocci-okf` after speculative `rocci-ui`
  renderers were pruned.[^okf-presentation][^cli-plan]
- Full `html` document shells carry product CSP, reload scripts, and metadata
  that should stay product-owned.

A new component library has to start from the copies above, not from a guessed
widget catalog.[^ui-readme]

## Draft comparison

Two exploratory drafts were produced in parallel for the same question.

The first draft (research plus plan, no architecture record) matched current
ownership: keep Markdown and OKF governance in their owners, extract only
demonstrated chrome, treat glue as a compiled-app host, and cache whole
programs rather than modules. It refused to publish a descriptive architecture
record for an unimplemented pipeline.

The second draft (architecture plus migration plan) usefully named a
content-addressed `~/.rocci/cache/renderers/<hash>/` layout and listed
external-process, Wasm, and native-library hosts as a taxonomy. It overreached
in four ways that this record does not adopt:

1. It described the pipeline as current architecture (`authority: descriptive`)
   and copied an older human verification event onto new text.
2. It treated OKF governance (`ConceptMetadataCard`, `ReviewDashboard`,
   `StatGrid`) and invented `rocci-ui` view records as phase-1 library work,
   repeating the Phase 7 over-extraction that was pruned.
3. It assumed shared `rocci-ui` templates yield one cache key across every
   workspace. Whole-program compilation does not work that way.
4. It treated Wasmtime latency as a measured v2 gate. Wasm is a real Roc
   target, but those budgets are not evidence from this repository.

The companion plan follows the first draft's ownership contract, requires
both the native subprocess host and a Wasmtime host, persists generated Roc
and compiled artifacts as two content-addressed tiers, and leaves native
glue documented as future potential.

[^catalog-shell]: Accepted ownership of catalog data in Rust and visible chrome in Rocci.
[^generator]: Current fragment-file bridge, once-compiled shell, and static feature gate.
[^theming]: Product-owned CSS surfaces versus research-only token work.
[^pure-render]: `@component` lowers to a pure Roc function returning `Html`.
[^static-okf]: Knowledge records stay inert Markdown and do not execute Rocci.
[^cli-plan]: Forbidden OKF-to-Rocdown presentation adapter; allowed `rocci-ui` view records.
[^ui-readme]: View records are the present shared primitive.
[^ui-view]: Shipped `PageView` fields; no OKF concept or review records.
[^build-rs]: `Command::new("roc")`, watch-mode hash, and applicator reuse.
[^build-runtime]: Segment forest composed in Roc from fragment files and docs records.
[^docs-rs]: Rust Markdown and article-block HTML, plus `plan_segments` for the Rocci path.
[^docs-rocci]: Rocci-authored `:kind` widgets used by the site applicator.
[^theme-rocci]: Site shell, sidebar, breadcrumbs, and outline in Rocci.
[^site-layouts]: Parallel site-theme layout components over the same `PageView`.
[^okf-presentation]: Rust `html_page`, `render_toc`, and OKF-local `StatCardView`.
[^okf-dev]: OKF-local `.rd-toc` CSS copied from the standalone Rocdown vocabulary.
[^okf-toc-js]: Copied table-of-contents scroll script in `rocci-okf`.
[^standalone-lower]: Standalone document TOC emitted during Rocdown lowering.
[^theme-toc-js]: Canonical standalone `toc.js` in `rocci-theme`.
[^deps-check]: `rocci-okf` must not depend on Rocdown packages.
[^template-readme]: Template crate does not invoke `roc`.
[^datastar-cache]: Existing `~/.rocci/cache` root, `ROCCI_CACHE`, and SHA-256 sidecar.
[^roc-glue]: Glue generates ABI bindings; it is not a compiler-embedding API.
[^roc-platform-rs]: Rust hosts call compiled Roc through regenerated glue.
[^roc-faq]: The platform host owns `main` and when Roc code runs; the compiler stays outside Roc.
[^roc-wasi]: `roc build --target wasm32` produces a WASI module runnable in Wasmtime.
