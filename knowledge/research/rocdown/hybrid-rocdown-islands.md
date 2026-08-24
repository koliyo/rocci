---
type: Research Report
title: Hybrid Rocdown islands for CDN-static sites
description: Exploratory research for static CDN HTML that embeds dynamic Rocci components backed by a rocci or rocdown HTTP service. Article documentation widgets are out of scope. Not shipped.
tags: [domain/rocdown, domain/rocci, domain/runtime, integration/datastar, integration/roc, concern/rendering, concern/security, concern/packaging, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-19T18:55:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-18
  - id: article-rs
    resource: ../../../crates/rocci-rocdown/src/article.rs
    title: Static-document feature gate and Markdown HTML renderer
    author: process:git
    last_modified: 2026-08-17
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: Site loader, Datastar rejection, and RD2301 island gate
    author: process:git
    last_modified: 2026-08-18
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Build planner, hashed assets, and default CSP
    author: process:git
    last_modified: 2026-08-19
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: Rocdown apply orchestration
    author: process:git
    last_modified: 2026-08-18
  - id: lowerer
    resource: ../../../crates/rocci-rocdown/src/lower.rs
    title: Standalone Rocdown lowerer for mixed Markdown and Rocci
    author: process:git
    last_modified: 2026-08-18
  - id: roc-build-runtime
    resource: ../../../crates/rocci-rocdown/runtime/RocdownBuild.roc
    title: Current Roc apply runtime for site pages
    author: process:git
    last_modified: 2026-08-18
  - id: theme-rocci
    resource: ../../../crates/rocci-rocdown/templates/RocdownTheme.rocci
    title: Site shell, CSP meta, and optional module script
    author: process:git
    last_modified: 2026-08-18
  - id: view-rs
    resource: ../../../crates/rocci-ui/src/view.rs
    title: ResourceView script and CSP fields
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-host
    resource: ../../../crates/rocci-roc-host/src/host.rs
    title: Wasmtime WASI apply with staging preopen
    author: process:git
    last_modified: 2026-08-18
  - id: wasm-platform
    resource: ../../../crates/rocci-roc-host/platform/main.roc
    title: Minimal wasm32 Roc platform without Path
    author: process:git
    last_modified: 2026-08-18
  - id: datastar-asset
    resource: ../../../crates/rocci-cli/src/datastar_asset.rs
    title: Datastar.js pin and cache staging for Rocci apps
    author: process:git
    last_modified: 2026-08-16
  - id: format-report
    resource: ../../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format investigation and hybrid axes
    author: human:nils
    last_modified: 2026-08-16
  - id: rocket-report
    resource: ../../../archive/reports/DATASTAR_ROCKET_IN_ROCCI_REPORT.md
    title: Rocci-native client-behavior island investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: roadmap
    resource: ../../../ROADMAP.md
    title: Implementation roadmap
    author: human:nils
    last_modified: 2026-08-17
  - id: rendering-doc
    resource: ../../../docs/concepts/rendering-model.rocdown
    title: Published rendering model
    author: human:nils
    last_modified: 2026-08-18
  - id: architecture-doc
    resource: ../../../docs/concepts/architecture.rocdown
    title: Published architecture ownership
    author: human:nils
    last_modified: 2026-08-18
  - id: server-actions
    resource: ../../../docs/guides/server-actions.rocdown
    title: Server actions and Datastar guide
    author: human:nils
    last_modified: 2026-08-18
  - id: interactive
    resource: ../../../examples/rocdown/pages/Interactive.rocdown
    title: Standalone dynamic Rocdown document with Datastar patches
    author: process:git
    last_modified: 2026-08-18
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Keep Rocdown Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: client-islands
    resource: ../../decisions/client-behavior-islands.md
    title: Use explicit islands for browser-owned behavior
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Use a Rust catalog and a Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: compiler-arch
    resource: ../../architecture/rocdown-documentation-compiler.md
    title: Rocdown documentation generator
    author: process:codex
    last_modified: 2026-08-18
  - id: format-arch
    resource: ../../architecture/rocdown-format.md
    title: Rocdown format boundary
    author: process:cursor
    last_modified: 2026-08-17
  - id: limitations
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:okf-phase-6
    last_modified: 2026-08-17
  - id: block-plan
    resource: ../../plans/rocdown/rocdown-block-renderers.md
    title: Custom Rocdown block schemas and renderers plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: generation-plan
    resource: ../../plans/rocci/rocci-component-generation.md
    title: First-party Rocci chrome library and generation host
    author: process:cursor
    last_modified: 2026-08-18
  - id: site-plan
    resource: ../../plans/site/rocci-dev-site.md
    title: rocci.dev site architecture and Rocdown evolution
    author: process:codex
    last_modified: 2026-08-18
  - id: impl-plan
    resource: ../../plans/rocdown/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands implementation plan
    author: process:cursor
    last_modified: 2026-08-19
---

# Hybrid Rocdown islands for CDN-static sites

## Research question

How can a Rocdown site publish **ordinary static HTML to a CDN** while
selected **Rocci components stay live**, backed by a **rocci or rocdown
HTTP service** (Datastar patches, `@on` handlers), without compiling
Markdown as Roc and without waiting on article-widget syntax?

Sub-questions:

1. What is the deploy shape: CDN origin versus service origin?
2. How does initial island HTML get into the CDN page without lowering
   prose to Roc?
3. Where do `@component` and `@on` live — colocated in `.rocdown`, a
   sibling `.rocci` app, or both?
4. What CSP and Datastar.js rules keep pages without islands static?
5. What does current code already reject (`RD2301`, no Datastar on site
   builds) versus what standalone `rocdown run FILE` already does?

Article `:kind` widgets (`:note`, `:tabs`, …) are **out of
scope**. They belong to the [block renderer
plan](/plans/rocdown/rocdown-block-renderers.md). This record must not
design around them.[^block-plan]

This is not shipped. Crate READMEs and architecture records remain the
current contract.[^rocdown-readme][^compiler-arch]

## For a later agent

- **Authority:** exploratory. Do not present hybrid splice or a CDN-plus-
  service deploy as implemented.
- **Do not implement** a phase until the user asks. The
  [implementation plan](/plans/rocdown/hybrid-rocdown-islands.md) owns
  slices.[^impl-plan]
- **Do not** wait on or redesign article `:kind` widgets here. That is
  the [block renderer plan](/plans/rocdown/rocdown-block-renderers.md).[^block-plan]
- **Do not** add `@island` grammar unless a phase explicitly consumes the
  [client-behavior island decision](/decisions/client-behavior-islands.md).
  v1 uses existing `@component`, document-root tags, and `@on`.[^client-islands][^rocdown-readme]
- Keep `knowledge/**/*.md` inert. OKF stays Markdown-only.

## Vocabulary

Do not collapse these into one `dynamic` flag.[^format-report][^format-arch]

| Term | Meaning here |
| --- | --- |
| Static region | Markdown (and other non-Rocci article body) rendered to HTML at build and stored on the CDN |
| Rocci component | `@component` / `@render` / document-root `<Tag>`: a pure Roc function to Html |
| Island host | The stable-ID element in the CDN page where that component's HTML sits |
| Island service | A rocci or rocdown HTTP process that owns `@context` / `@init` / `@on` for those hosts |
| Hybrid page | CDN document HTML plus one or more island hosts that may call the service |
| Client-behavior island | Proposed `@island` custom-element JS; **not v1** |

A component is not live merely because it is Rocci. A pure `<FeatureCount />`
can be pre-rendered into CDN HTML with no service. It becomes live when it
declares `@on` (or Datastar actions that post to the service) and durable
state stays on that service.[^pure-render][^server-owned][^rendering-doc]

## Topic background

Rocdown has two working pipelines that do not combine:[^rocdown-readme][^architecture-doc]

1. **`rocdown run FILE`** (and `rocci run App.rocci`) lowers the program,
   stages Datastar.js, and serves HTTP. `examples/rocdown/pages/Interactive.rocdown`
   already has `@component`, `@on:get`, and `@on:post` Datastar morphs.
   The whole document is a Roc app. Prose is generated Roc. There is no
   CDN tree.[^interactive][^datastar-asset][^server-actions]
2. **`rocdown build`** writes hashed static files. Pages with `@component`,
   `@render`, `@roc`, `@css`, handlers, or document-root template tags
   fail `RD2301`. Compiled Roc that `import Datastar` is rejected because
   the site runtime does not stage Datastar.js.[^article-rs][^site-rs][^limitations]

The format report's **hybrid output** is exactly this missing combination:
static page files on a CDN, plus an explicit server for handlers, plus
client JS only where needed. Stage 2 (server/hybrid) did not ship. The
roadmap still lists dynamic island splicing as unchecked.[^format-report][^roadmap]

Rust still owns catalog, routes, and Markdown HTML. Authored Rocci stays
on the Roc path **only where used**. That ownership is unchanged.[^catalog-shell][^compiler-arch]

The public-site plan still wants one static catalog for reading. Hybrid
islands on a page must not turn the whole site into an app.[^site-plan]

## Current pipeline (only what this plan needs)

`is_static_document` rejects every Rocci template item and every handler.
Standalone `lower` already splices those items into `rocci_content` for
`rocdown run FILE`. The site path parses them, then discards the Roc.[^article-rs][^lowerer][^site-rs]

The planner already emits a CDN-shaped tree: directory-index HTML, hashed
`/assets/*`, sitemap and `pages.json`, canonical `base_url`, default CSP
with `script-src 'none'` and `connect-src 'none'`, atomic commit.[^plan-rs][^rocdown-readme]

The theme can inject a module script and per-page CSP when
`ResourceView` fields are non-empty. Site builds do not stage
Datastar.js; `rocci run` does, via `~/.rocci/cache`.[^theme-rocci][^view-rs][^datastar-asset]

Apply currently calls `siteShell` with `Html.empty` and discards
`render_all`, so the content slot is unused. Hybrid splicing needs that
slot to receive **one article Html blob** (Markdown HTML plus island
HTML). Restoring a documentation-widget forest walker is not this
plan's job.[^roc-build-runtime][^build-rs][^wasm-host][^wasm-platform]

## Deploy shape

v1 is two origins that cooperate:

```text
CDN (static)                         Service (dynamic)
------------                         -----------------
/guides/foo/index.html               GET  /islands/...   optional
/assets/theme.<hash>.css             POST /actions/...   Datastar patch
/assets/datastar.<hash>.js           SSE  /stream/...    if authored
  (only on hybrid pages)
```

- The **CDN** serves complete documents. Navigation, Markdown, and the
  initial island HTML work without the service (degraded: buttons do
  nothing). Cache HTML with a short TTL; cache hashed assets long.[^plan-rs][^rendering-doc]
- The **service** is a normal Rocci/Rocdown HTTP app: `@context`,
  `@init`, `@on`. GET handlers for islands return fragments (or a
  document only in standalone preview). Mutation handlers return
  Datastar element patches with stable ids.[^server-actions][^server-owned]
- `site.base_url` is the CDN canonical. A separate **service origin**
  (config) is what hybrid pages put in `connect-src` and in Datastar
  `@post` URLs if the origins differ. Same-origin (CDN in front of both)
  is allowed when the operator routes `/actions/` to the service.

Do not run Roc at CDN GET time. Do not edge-SSR Markdown.

## How a hybrid page is built

Keep Markdown in Rust. Compile **island Roc only** for pages that have
Rocci components or handlers.[^catalog-shell][^architecture-doc]

1. Split the document: Markdown runs versus `@component` / `@render` /
   document-root tags / `@css` / `@roc` values / `@on` / `@context` /
   `@init`.
2. Render Markdown runs to escaped HTML fragments (existing article
   renderer).
3. Evaluate island components at build (via `rocci-roc-host`) to get
   **initial** Html with stable ids.
4. Stitch fragments and island Html in document order into one article
   blob. Pass that blob to the theme `content` slot through the trusted
   unescaped-html bridge. Do not emit Markdown as Roc constructors.[^compiler-arch][^generation-plan]
5. If the page has `@on` / Datastar actions, hash Datastar.js, set
   per-page `script-src` / `connect-src`, and emit the island service
   module for those routes. Pages with no handlers keep the strict
   CSP and no Datastar.js.[^plan-rs][^format-report][^site-rs]
6. Catalog/check of pages with no Rocci still does not require Roc.

`examples/rocdown/pages/Interactive.rocdown` is the live fixture (toggles and
`@on:post` reveal). A page like `Guide.rocdown` (component, no handlers)
is the pre-render-only fixture: CDN HTML, no service.[^interactive][^rocdown-readme]

## Where the service program lives

Both CLIs already know this dispatcher. v1 should accept both, with
colocated `.rocdown` as the default authoring path:[^rocdown-readme][^server-actions]

| Authoring | Service binary | Use |
| --- | --- | --- |
| `@component` + `@on` in the `.rocdown` file | Generated rocdown island service from those handlers | Default; one file |
| Sibling `.rocci` app | `rocci run App.rocci` | Shared components/actions across pages |
| Mix | Rocdown page hosts; `.rocci` owns `@context` / routes | Allowed if routes are explicit |

The CDN page never implies a service. `@on` and Datastar actions are
visible. A component without handlers is CDN-only.[^markdown-first]

## CSP and Datastar

Default site CSP stays `script-src 'none'; connect-src 'none'`. Hybrid
pages that talk to the service need Datastar.js (`unsafe-eval` is
today's Datastar expression cost, same as `rocci run`) and `connect-src`
for the service origin only — not site-wide `'self'`.[^plan-rs][^datastar-asset][^rocket-report]

Pages without islands must not include Datastar.js (format-report
acceptance test 11).[^format-report]

## Options considered

| Option | Advantages | Problems | Recommendation |
| --- | --- | --- | --- |
| Keep binary `rocdown build` vs `rocdown run FILE` | Simple | Interactive documents cannot sit in a CDN site | Reject as the product model |
| Compile the whole page to Roc (standalone lowering) | Reuses today's interactive path | Prose in Roc; not CDN-cheap | Reject for site builds |
| Pre-render components only; no service | Small | Cannot back live `@on` | Ship as a subset, not the goal |
| Whole site as a Datastar app | One origin | Defeats CDN static reading | Reject[^site-plan] |
| Edge SSR at the CDN | Always fresh | Request-time Roc | Out of scope |
| CDN HTML + island service + per-page Datastar | Matches hybrid axes and server-owned state | Origin/CSP/CORS work | **Recommended** |
| Wait for article-widget renderer work | One syntax story | Blocks unrelated live components | Reject; parallel track[^block-plan] |
| `@island` JS first | Browser-only widgets | Not how Rocci actions work | Follow-on[^client-islands] |

## Recommendation

Treat hybrid Rocdown as **CDN documents plus an island service**, not as
a new document syntax and not as an article-widget feature.

1. Pass one Rust-composed article Html blob into the theme content slot
   (Markdown HTML, later plus island HTML). Do not restore documentation-
   widget painting in this plan.
2. Classify pages: static (CDN only) versus hybrid (CDN HTML + optional
   service). No single `dynamic` boolean.
3. Splice pure Rocci components into the CDN blob by compiling island Roc
   only.
4. Compile `@on` / `@context` / `@init` from hybrid pages (or a sibling
   `.rocci` app) into a rocci/rocdown service. Stage Datastar.js only on
   those pages. `connect-src` names the service origin.
5. Keep `@island` custom-element JS and article-widget syntax off this
   track.

Constraints that do not move: Markdown-first declarations, pure
`@component`, server-owned durable state, Rust catalog / Rocci shell,
OKF Markdown-only, no Rocket bundle.[^markdown-first][^pure-render][^server-owned][^catalog-shell][^client-islands]

## Layer map

| Concern | Owner |
| --- | --- |
| Hybrid vs static classification, `RD2301` | `article.rs`, `site.rs` |
| Article blob (Markdown HTML + island HTML) | article renderer + island evaluator; not documentation widgets |
| Theme content slot, per-page CSP, Datastar script | `RocdownTheme.rocci`, `ResourceView`, `plan.rs` |
| Island Roc lowering (no Markdown as Roc) | `lower.rs` plus a site extractor |
| Island service (`@on` dispatcher) | `rocci-rocdown-cli` and/or `rocci-cli` (same HTTP contract) |
| Datastar.js pin | `rocci-cli` datastar cache; site planner hashes a copy for hybrid pages |
| Host evaluation of initial island Html | `rocci-roc-host` |
| `@island` grammar | `rocci-template` (sibling; not v1) |
| Article widgets | Generalized block model plan (parallel; ignore here) |

## Relationship to other work

- [Custom block schemas and renderers](/plans/rocdown/rocdown-block-renderers.md)
  owns article-block paint. This plan must not wait on it.[^block-plan]
- [Client-behavior islands](/decisions/client-behavior-islands.md) owns
  browser custom elements. Not required to back Rocci components with a
  service.[^client-islands]
- [Generation host](/plans/rocci/rocci-component-generation.md) evaluates
  initial island Html.[^generation-plan]
- [rocci.dev site plan](/plans/site/rocci-dev-site.md): reading stays static;
  live islands are opt-in regions, not a site-wide app.[^site-plan]

## Limits

This record does not freeze CORS header policy, whether island GET is
pre-rendered only or also live-refreshable, or whether one service
process covers the whole site versus per-app `rocci run`. Those are
plan v1 answers or later revisions.

[^rocdown-readme]: Standalone vs site CLI; declarations; Datastar only when a region uses an action.
[^article-rs]: `is_static_document` rejects Rocci templates and handlers.
[^site-rs]: `RD2301` and `import Datastar` rejection.
[^plan-rs]: Hashed assets, `DEFAULT_CSP`, per-page resource fields.
[^build-rs]: Apply discards `render_all`; planned outputs omit page HTML writes.
[^lowerer]: Standalone splice into `rocci_content`.
[^roc-build-runtime]: `siteShell(..., Html.empty)`.
[^theme-rocci]: CSP meta and optional module script on the shell.
[^view-rs]: `ResourceView` script and CSP strings.
[^wasm-host]: WASI apply preopens staging.
[^wasm-platform]: wasm32 platform has no Path.
[^datastar-asset]: `rocci run` stages pinned Datastar.js; site builds do not.
[^format-report]: Hybrid axes; JS only when referenced; Stage 2 server/hybrid.
[^rocket-report]: Datastar morph vs private DOM; do not bundle Rocket.
[^roadmap]: Unchecked dynamic island splicing.
[^rendering-doc]: GET documents vs mutation fragments; Datastar transports HTML.
[^architecture-doc]: Compile authored dynamic islands only where used.
[^server-actions]: `@context` / `@init` / `@on` and Datastar `@post` contract.
[^interactive]: Working colocated `@component` + `@on` document, not a site page.
[^markdown-first]: Explicit opt-in executable regions.
[^client-islands]: Exploratory `@island`; not v1 for this plan.
[^catalog-shell]: Rust catalog; Rocci shell; island splice absent.
[^pure-render]: `@component` is a Roc function to Html.
[^server-owned]: Durable state on the server; patches are HTML.
[^compiler-arch]: Trusted HTML bridge; splicing rejected today.
[^format-arch]: Language islands vs unimplemented `@island`.
[^limitations]: Site builds reject Roc/Rocci islands.
[^block-plan]: Article widgets; per-page `@component` on static build is not that plan.
[^generation-plan]: Native and wasm renderer hosts.
[^site-plan]: Public site remains one static catalog for reading.
[^impl-plan]: Phased delivery for this research.
