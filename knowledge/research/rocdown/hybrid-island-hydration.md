---
type: Research Report
title: Hybrid island hydration versus SSR island frameworks
description: "Shipped Rocdown static/hydrate/live is build-time HTML splice plus an optional Datastar island service, not React-style client hydration. v1 of that deploy shape is implemented and experimental; @island JS islands are not. Knowledge status records still describe the pre-ship gate."
tags: [domain/rocdown, domain/rocci, domain/runtime, integration/datastar, concern/rendering, concern/architecture]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-28
authority: exploratory
owners: [human:nils]
sources:
  - id: article-rs
    resource: ../../../crates/rocci-rocdown/src/article.rs
    title: PageKind classification and island placeholders
    author: process:git
    last_modified: 2026-08-23
  - id: build-rs
    resource: ../../../crates/rocci-rocdown/src/build.rs
    title: splice_islands for hydrate and live pages
    author: process:git
    last_modified: 2026-08-22
  - id: islands-rs
    resource: ../../../crates/rocci-rocdown/src/islands.rs
    title: Build-host evaluation of island Html
    author: process:git
    last_modified: 2026-08-22
  - id: service-rs
    resource: ../../../crates/rocci-rocdown/src/service.rs
    title: serve-islands, live CSP, action URL prefix
    author: process:git
    last_modified: 2026-08-24
  - id: site-rs
    resource: ../../../crates/rocci-rocdown/src/site.rs
    title: RD2302 cdn-only live gate
    author: process:git
    last_modified: 2026-08-23
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language and site kinds
    author: process:git
    last_modified: 2026-08-24
  - id: hybrid-guide
    resource: ../../../docs/rocdown/hybrid.rocdown
    title: Public hybrid site publish guide
    author: process:git
    last_modified: 2026-08-23
  - id: counter-readme
    resource: ../../../examples/rocdown/counter/README.md
    title: Hybrid live-counter two-artifact runbook
    author: process:git
    last_modified: 2026-08-22
  - id: hybrid-plan
    resource: ../../plans/rocdown/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands implementation plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: catalog-shell
    resource: ../../decisions/rust-catalog-rocci-shell.md
    title: Rust catalog and Rocci documentation shell
    author: process:okf-migration
    last_modified: 2026-08-18
  - id: hybrid-research
    resource: ../hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands design research
    author: process:cursor
    last_modified: 2026-08-24
  - id: hosting-follow-ons
    resource: ../../plans/rocdown/hybrid-island-hosting-follow-ons.md
    title: Hybrid island hosting follow-ons
    author: process:cursor
    last_modified: 2026-08-24
  - id: snapshot-research
    resource: ../island-snapshot-roc-reachability.md
    title: Snapshot eval must not compile service-only @roc
    author: process:cursor
    last_modified: 2026-08-25
  - id: client-islands
    resource: ../../decisions/client-behavior-islands.md
    title: Explicit islands for browser-owned behavior
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Server-owned durable state
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure render components
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: impl-status
    resource: ../../status/implementation.md
    title: Rocci implementation status
    author: process:cursor
    last_modified: 2026-08-26
  - id: limitations
    resource: ../../status/known-limitations.md
    title: Known Rocci limitations
    author: process:cursor
    last_modified: 2026-08-24
  - id: streams-doc
    resource: ../../../docs/concepts/documents-fragments-commands-streams.rocdown
    title: Documents, fragments, commands, and streams
    author: process:git
    last_modified: 2026-08-22
  - id: astro-islands
    resource: https://docs.astro.build/en/concepts/islands/
    title: Astro islands architecture
    author: process:web
    last_modified: 2026-08-28
  - id: astro-server
    resource: https://docs.astro.build/en/guides/server-islands/
    title: Astro server islands
    author: process:web
    last_modified: 2026-08-28
  - id: fresh-islands
    resource: https://fresh.deno.dev/docs/concepts/islands
    title: Fresh islands
    author: process:web
    last_modified: 2026-08-28
  - id: qwik-resumable
    resource: https://qwik.dev/docs/concepts/resumable/
    title: Qwik resumability
    author: process:web
    last_modified: 2026-08-28
  - id: next-rsc
    resource: https://nextjs.org/docs/app/getting-started/server-and-client-components
    title: Next.js server and client components
    author: process:web
    last_modified: 2026-08-28
  - id: islands-essay
    resource: https://jasonformat.com/islands-architecture/
    title: Islands Architecture
    author: process:web
    last_modified: 2026-08-28
  - id: htmx-hateoas
    resource: https://htmx.org/essays/hateoas/
    title: HATEOAS and hypermedia
    author: process:web
    last_modified: 2026-08-28
---

# Hybrid island hydration versus SSR island frameworks

## Research question

Is Rocci/Rocdown hybrid-island work a solved problem, or are there several unfinished approaches? What is missing relative to other SSR and island frameworks?

This record is exploratory synthesis. Public crate docs and `main` code are the shipped contract. Industry comparisons are not audits of those codebases.[^hybrid-guide][^rocdown-readme]

## Vocabulary (do not collapse)

Three different “island” ideas sit in this repository. They are not one feature.[^hybrid-research][^client-islands][^markdown-first]

| Term | What it is | Shipped? |
| --- | --- | --- |
| **Hybrid Rocdown islands** | CDN-static HTML plus a mutation/live HTTP service for selected pages | Yes, labeled experimental |
| **`hydrate` page kind** | Build-time splice of pure Rocci Html (and Markdown `@{expr}`) into the CDN file; **no Datastar** | Yes |
| **`live` page kind** | Same splice, plus Datastar.js and `serve-islands` / sibling `rocci run` | Yes, experimental |
| **Client-behavior `@island`** | Custom-element JS for canvas, keyboard, maps, third-party widgets | No; reserved / exploratory |

**Hydrate is a misnomer relative to React, Astro, Fresh, and Next.** Those systems “hydrate” by attaching a client component runtime to server HTML. Rocdown `hydrate` means: evaluate Rocci on the **build host**, write ordinary HTML, ship no island JS. There is no client VDOM replay, no `client:load`, and no resumable serialized app state.[^article-rs][^build-rs][^rocdown-readme][^astro-islands]

A document-root `<Tag>` is an **HTML island** in the Markdown-first sense (mode change at a visible line). That is not `@island` and not hydration.[^rocdown-readme][^markdown-first]

## Current shipped state

The [hybrid implementation plan](/plans/rocdown/hybrid-rocdown-islands.md) records Phases 1–10 as shipped on `main` after the `:name[params]` rebase. Public docs and the crate README match that, and mark the product **experimental**.[^hybrid-plan][^hybrid-guide][^rocdown-readme]

What `main` does:

1. **Classify** each page as `static` / `hydrate` / `live` (`classify_document`). Article `:name[params]` stays `static`. `@render` / `@component` / `@roc` / `@css` / document-root templates / `@{expr}` promote at least `hydrate`. Handlers, `@context` / `@init`, or `import Datastar` promote `live`.[^article-rs]
2. **Dual apply.** `static` pages keep the widget forest (`:note`, `:tabs`). `hydrate` / `live` pages put `<!--rocci-island-->` placeholders in Rust article HTML, evaluate island Roc on the build host, and `splice_islands` fills those holes.[^build-rs][^islands-rs][^hybrid-plan]
3. **Two artifacts.** The CDN tree is files. `rocdown serve-islands` (colocated handlers) or `[http].service` (`rocci run`) owns mutations and `@get:live`. Document GET stays on the CDN. Preview (`rocdown view`) proxies `/actions/` on one origin.[^hybrid-guide][^service-rs][^counter-readme]
4. **Per-page Datastar.** Only `live` pages hash Datastar.js and loosen CSP for the service. Neighboring `static` / `hydrate` pages do not get Datastar. `--cdn-only` errors `RD2302` if any published page is `live`.[^site-rs][^rocdown-readme][^hybrid-guide]
5. **Live interactivity is hypermedia**, not client-state hydration. Commands are representation-free (empty SSE vs 204). A spliced island must author `data-init=@get("/sse", …)` itself; a full-page `@get:live` app can inject that on `<body>`.[^hybrid-guide][^streams-doc][^server-owned]

Worked examples: `examples/rocdown/hybrid/widgets.rocdown` is `hydrate` (FeatureCount, no Datastar). `examples/rocdown/counter` is `live` plus a `static` neighbor. rocci.dev home composes a sibling live-counter service.[^hybrid-guide][^counter-readme]

Durable state stays on the island process (SQLite / `@init`). The CDN file is a snapshot that the live stream may replace after connect.[^server-owned][^pure-render][^hybrid-guide]

## Is it “solved”?

**The v1 hybrid deploy shape is implemented, not finished, and not the React/Astro hydration problem.**

Solved enough to use:

- Static catalogs (`docs/`) unchanged.
- Pure Rocci on a CDN page without a process (`hydrate`).
- Live counters / commands / generated `@get:live` beside static neighbors.
- Same-origin reverse proxy, `inspect artifacts`, `package` of CDN plus island binary, local Docker demo.[^hybrid-guide][^rocdown-readme][^hosting-follow-ons]

Explicitly unsolved (product still says experimental):

| Gap | Why it matters |
| --- | --- |
| **`@island` / `*.client.js`** | No first-class browser-owned surface. Canvas, drag, maps, editors still have no typed host + controller contract.[^client-islands][^rocdown-readme] |
| **Cross-origin CORS and cookies** | `service_origin` already prefixes action URLs and `connect-src`; credentialed cross-origin is not shipped. Same-origin proxy is the supported layout.[^hybrid-guide][^hosting-follow-ons] |
| **Live island GET refresh** | Snapshot is build-time; the live stream replaces it if authored. There is no generic “re-fetch this host Html from the service” primitive in v1.[^hybrid-plan] |
| **Islands inside `:note` bodies** | `@render` in article-block bodies is out of v1.[^hybrid-plan][^hybrid-guide] |
| **`@use` kinds on `rocdown build`** | Closed registry on static site builds.[^hybrid-plan] |
| **Snapshot vs handler Roc** | Live pages compile twice (`basic-cli` snapshot vs `basic-webserver`). Unused `@roc` still typechecks; SQLite APIs differ. Lowering reachability is a workaround, not a platform fix.[^snapshot-research] |
| **Hosting polish** | WebKit-free `--no-window` binaries, slim runtime without `roc` on PATH, vendor cache-header adapters remain follow-ons (some packaging exists; the follow-on plan still tracks the rest).[^hosting-follow-ons] |
| **Knowledge drift** | [`implementation.md`](/status/implementation.md) and [`known-limitations.md`](/status/known-limitations.md) still say dynamic splice is incomplete. That is stale versus `article.rs` / `build.rs` / the public hybrid guide. `@island` remaining unimplemented is still accurate.[^impl-status][^limitations][^rocdown-readme] |

There are **not many competing shipped approaches** inside Rocci. The design record already rejected: whole-page-to-Roc site builds, whole-site-as-Datastar-app, edge SSR at the CDN, waiting on article widgets, and `@island` JS first. What shipped is that one recommended shape. Separate tracks still exist (standalone `rocdown run FILE`, sibling `.rocci` service, colocated handlers) but they share the same HTTP/Datastar contract.[^hybrid-research][^hybrid-plan]

## How many “hydration” approaches exist in the industry?

The industry uses the same word for several different machines. Rocci picked one and deferred another.[^islands-essay][^astro-islands]

| Approach | Who | What the browser does | Rocci analog |
| --- | --- | --- | --- |
| **Partial client hydration (islands architecture)** | Astro `client:*`, Fresh islands, Enhance, older Marko | Static HTML; selected framework components boot a client runtime | **`@island` — not shipped**[^astro-islands][^fresh-islands] |
| **Server/client component split** | Next.js App Router RSC, Remix-ish progressive enhancement | Server Components never hydrate; `"use client"` subtrees do | Closest to **pure `@component` vs handlers**, but Next still hydrates client trees[^next-rsc] |
| **Server islands** | Astro server islands | Defer a hole to request-time HTML from the server | **Not Rocci.** Rocci forbids request-time Roc at the CDN; the hole is filled at **build**, then patched later via Datastar if `live`[^hybrid-research][^astro-server] |
| **Resumability** | Qwik | Serialize listeners/closures; resume without replay | **No analog.** Rocci does not serialize a client app[^qwik-resumable] |
| **Full-app hydration** | Classic Next Pages, SvelteKit default, Nuxt | SSR HTML then hydrate the whole app | **Rejected** for Rocdown sites |
| **Hypermedia / morph** | HTMX, Datastar, Phoenix LiveView, Turbo | Server returns HTML; client morphs by id; no component tree in JS | **Shipped `live` path.** This is the actual interactivity model[^htmx-hateoas][^streams-doc] |

Rocci’s hybrid v1 is therefore closer to **Astro static pages + HTMX/Datastar islands** (or Enhance-style progressive HTML) than to Astro/Fresh **client** islands or Next RSC. The missing piece versus those frameworks is not another splice algorithm; it is the **explicit JS island** for work Datastar should not own.[^client-islands]

## Comparison (exploratory scores)

Scores are synthesis, not a bake-off. Higher is closer to that system’s stated goal.

| Concern | Rocci hybrid v1 | Astro / Fresh client islands | Next RSC | Qwik | HTMX / Datastar-only apps |
| --- | --- | --- | --- | --- | --- |
| Zero JS on reading pages | Strong (`static` / `hydrate`; Datastar only on `live`) | Strong if no `client:*` | Weaker (runtime + client boundaries) | Medium (loader, then resume) | Strong if no hyperscript |
| Interactive forms / counters | Strong (server Html + morph) | Strong if you bring a UI framework | Strong | Strong | Strong |
| Canvas / maps / editors | Weak (no `@island`) | Strong (that is the point) | Strong (`"use client"`) | Strong | Weak unless you drop to JS |
| CDN-cheap document GET | Strong (files; no Roc at GET) | Strong (static) or mixed (SSR) | Usually request-time RSC | Usually request-time | Depends; often origin SSR |
| Ownership of durable state | Strong (server / SQLite) | Author-dependent | Author-dependent | Author-dependent | Strong if disciplined |
| Typed server components | Strong (Roc `@component`) | Medium (depends on framework) | Strong (RSC) | Medium | N/A |
| Progressive hydration (`idle` / `visible`) | N/A (no client component runtime) | First-class | Partial (lazy client) | Different (resume) | N/A |
| Streaming SSR of the document | Absent by design | Optional | First-class | First-class | Origin-dependent |
| Cross-origin static + API | Config prefix only; CORS/cookies open | Usually same-origin or well-trodden | Usual Next API/CORS | Usual | Usual |

Rocci is **ahead** of typical island SSGs on: server-owned state as a default, per-page CSP, refusing to advertise live actions on a CDN-only publish, and not compiling Markdown to the component language.[^rocdown-readme][^site-rs][^catalog-shell]

Rocci is **behind** those SSGs on: client-island DX, hydration timing strategies, request-time personalization at the edge, and a mature cross-origin story.[^client-islands][^hosting-follow-ons][^astro-server]

Rocci is **aligned** with HTMX/Datastar and unlike Next/Astro client islands: interactivity is Html in, Html out. Comparing “hydration completeness” to React is the wrong metric.[^streams-doc][^htmx-hateoas]

## Features worth considering (not a plan)

These are gaps, not approved work. Do not start them from this record.

1. **Keep using “hydrate” only as the page kind**, and say “build-time splice” in docs when talking to people who know Astro/React. The collision is already causing status-record confusion.[^impl-status][^limitations]
2. **Ship or reject `@island`.** Until that decision is approved, “hybrid hydration” will keep being read as the missing JS story. The Rocket report still recommends light-DOM behavior islands and not bundling Rocket.[^client-islands]
3. **Hosting follow-ons** (CORS/cookies, WebKit-free CLI, runtime without `roc`) if hybrid leaves laptop Docker.[^hosting-follow-ons]
4. **Snapshot/service Roc reachability** so live pages are not a dual-platform authoring trap.[^snapshot-research]
5. **Live GET refresh and islands-in-blocks** only if authors hit those walls; v1 explicitly deferred them.[^hybrid-plan]
6. **Do not add** `client:idle` / Qwik resumability / edge SSR unless the stack rule changes. Those optimize a client component tree Rocci chose not to have.[^hybrid-research][^server-owned]

## Relationship to other records

- Design-era research: [Hybrid Rocdown islands](hybrid-rocdown-islands.md) (still says “not shipped” in places; treat as historical).[^hybrid-research]
- Implementation: [hybrid Rocdown islands plan](/plans/rocdown/hybrid-rocdown-islands.md).[^hybrid-plan]
- Hosting leftovers: [hybrid island hosting follow-ons](/plans/rocdown/hybrid-island-hosting-follow-ons.md).[^hosting-follow-ons]
- Dual compile: [snapshot Roc reachability](island-snapshot-roc-reachability.md).[^snapshot-research]
- JS islands: [client-behavior islands](/decisions/client-behavior-islands.md); design blockers and recommended sequence: [`@island` design](/research/rocci/client-behavior-islands.md).[^client-islands]

[^article-rs]: `PageKind` and `classify_document` promote interpolation and Rocci items to hydrate; handlers to live.
[^build-rs]: `splice_islands` evaluates hydrate/live pages and fills placeholders.
[^islands-rs]: Build-host island Html evaluation.
[^service-rs]: Island process, CSP, and `service_origin` URL prefix.
[^site-rs]: `RD2302` when `--cdn-only` sees a live page.
[^rocdown-readme]: Shipped kinds, splice, `serve-islands`, package; `@island` deferred.
[^hybrid-guide]: Experimental label; two artifacts; `data-init` on spliced fragments; CORS not shipped.
[^counter-readme]: Live counter plus static neighbor.
[^hybrid-plan]: Phases 1–10 shipped on `main`; follow-ons listed.
[^hybrid-research]: Recommended CDN plus service; rejected alternatives; design-era “not shipped” wording.
[^hosting-follow-ons]: CORS/cookies, WebKit-free CLI, precompiled runtime image.
[^snapshot-research]: Two platforms, unused `@roc`, SQLite API split.
[^client-islands]: Proposed explicit JS islands; unimplemented.
[^markdown-first]: Mode change only at document-root declarations; `@island` reserved.
[^server-owned]: Durable state on the server.
[^pure-render]: `@component` is a Roc function to Html.
[^impl-status]: 2026-08-26 snapshot still lists island splicing as missing.
[^limitations]: Static-docs section still describes the pre-splice `RD2301` gate.
[^streams-doc]: Documents vs fragments vs commands vs live streams.
[^astro-islands]: Astro client islands hydrate framework components.
[^astro-server]: Astro server islands fetch HTML at request time.
[^fresh-islands]: Fresh Preact islands on otherwise static pages.
[^qwik-resumable]: Resume serialized listeners without hydration replay.
[^next-rsc]: Server Components vs `"use client"` hydration.
[^islands-essay]: Islands architecture as partial hydration.
[^htmx-hateoas]: Hypermedia updates without a client domain model.
[^catalog-shell]: Rust catalog and Markdown HTML stay off the Roc path for static regions; island Roc compiles only where used.
