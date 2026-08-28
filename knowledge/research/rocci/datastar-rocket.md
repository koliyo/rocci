---
type: Research Report
title: Datastar Rocket and Rocci-native islands (current)
description: "2026-08-28 restatement of the archived Rocket investigation against shipped Rocci. Ownership and no-bundle rules still hold. Crate paths, declaration syntax, CSS artifacts, and the Snake input module have moved. Custom-element morph identity is still unproven."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/rendering, concern/architecture, concern/security]
status: draft
generated: { by: process:cursor, at: 2026-08-28T17:55:00Z }
stale_after: 2026-11-28
authority: exploratory
owners: [human:nils]
sources:
  - id: archive
    resource: ../../../archive/reports/DATASTAR_ROCKET_IN_ROCCI_REPORT.md
    title: Historical Rocket-style client components report (2026-08-14)
    author: human:nils
    last_modified: 2026-08-16
  - id: snake-study
    resource: ../../../archive/reports/SNAKE_DATASTAR_ARCHITECTURE_REPORT.md
    title: Snake input and Datastar architecture
    author: human:nils
    last_modified: 2026-08-16
  - id: runtime-report
    resource: ../../../archive/reports/ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md
    title: Roc and Datastar component architecture report
    author: human:nils
    last_modified: 2026-08-23
  - id: decision
    resource: ../../decisions/client-behavior-islands.md
    title: Explicit islands for browser-owned behavior
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: blockers
    resource: ../client-behavior-islands.md
    title: "@island design blockers and recommended approach"
    author: process:cursor
    last_modified: 2026-08-28
  - id: hydration
    resource: ../rocdown/hybrid-island-hydration.md
    title: Hybrid island hydration versus SSR frameworks
    author: process:cursor
    last_modified: 2026-08-28
  - id: parser
    resource: ../../../crates/rocci-template/src/parser.rs
    title: Template parser
    author: process:git
    last_modified: 2026-08-25
  - id: lexer
    resource: ../../../crates/rocci-template/src/lexer.rs
    title: Tag and attribute name scanners
    author: process:git
    last_modified: 2026-08-21
  - id: lower
    resource: ../../../crates/rocci-template/src/lower.rs
    title: Html.element lowering
    author: process:git
    last_modified: 2026-08-25
  - id: compile-rs
    resource: ../../../crates/rocci-template/src/lib.rs
    title: CompileOutput with styles, no JS islands
    author: process:git
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: ModuleItem without IslandDecl
    author: process:git
    last_modified: 2026-08-25
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Shipped @component grammar
    author: process:git
    last_modified: 2026-08-25
  - id: datastar-asset
    resource: ../../../crates/rocci-cli/src/datastar_asset.rs
    title: Pinned Datastar 1.0.2 free bundle
    author: process:git
    last_modified: 2026-08-23
  - id: view-rs
    resource: ../../../crates/rocci-cli/src/view.rs
    title: Preview injects /assets/datastar.js
    author: process:git
    last_modified: 2026-08-25
  - id: dispatch-rs
    resource: ../../../crates/rocci-cli/src/dispatch.rs
    title: Generated Server.static_mount for /assets
    author: process:git
    last_modified: 2026-08-22
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Site CSP, goto.js, Datastar on live only
    author: process:git
    last_modified: 2026-08-24
  - id: snake-js
    resource: ../../../examples/rocci/custom/snake/assets/snake-input.js
    title: Hand-authored Snake input module
    author: process:git
    last_modified: 2026-08-20
  - id: snake-rocci
    resource: ../../../examples/rocci/custom/snake/Snake.rocci
    title: Explicit module script tags for Datastar and snake-input
    author: process:git
    last_modified: 2026-08-24
  - id: handlers
    resource: ../method-role-handlers-datastar-ecosystem.md
    title: Shipped @method:role matrix
    author: process:cursor
    last_modified: 2026-08-23
  - id: cqrs
    resource: ../datastar-cqrs-action-responses.md
    title: One-shot patches versus generated live
    author: process:cursor
    last_modified: 2026-08-21
  - id: rocket-docs
    resource: https://data-star.dev/reference/rocket
    title: Datastar Rocket reference
    author: process:web
    last_modified: 2026-08-28
  - id: rocket-license
    resource: https://data-star.dev/pro
    title: Datastar Pro licensing
    author: process:web
    last_modified: 2026-08-28
---

# Datastar Rocket and Rocci-native islands (current)

## Purpose and authority

This restates the 2026-08-14 Rocket investigation against **shipped** Rocci as of 2026-08-28. The archive file is historical evidence. Syntax here is still illustrative. The exploratory [client-behavior islands](/decisions/client-behavior-islands.md) decision is not approved. Plan-readiness questions live in [`@island` design blockers](client-behavior-islands.md).[^archive][^decision][^blockers]

## What still holds

Rocket is Datastar Pro’s browser custom-element runtime (props/codecs, setup, render, optional shadow DOM, instance-scoped `$$` signals). Rocci `@component` is a pure Roc function to Html. They solve different problems and must stay different constructs.[^archive][^rocket-docs][^runtime-report]

The recommended product is still an opt-in **Rocci-native behavior island**, not Rocket-in-the-box:[^archive][^decision]

- Roc and handlers remain authoritative for durable state and Html/SSE patches.
- An island is a native custom element only where browser ownership is required (canvas, drag, keyboard, maps, editors, media, observers, third-party widgets).
- First cut is **behavior-only and light-DOM-first**: server renders the host and meaningful children; JS attaches behavior, observes server changes, emits events, owns only private DOM.
- Large controllers stay in `*.client.js`. Do not compile Roc to JavaScript.
- **Do not bundle or redistribute Datastar Rocket.** Pro terms forbid publishing it as a toolchain dependency. A later bring-your-own provider would need written permission. Rocci pins the **free** Datastar 1.0.2 bundle, which has no Rocket API.[^archive][^rocket-license][^datastar-asset]

The Flow lesson is unchanged: light-DOM **descriptors** are server-morphable; the island paints a **private** SVG/canvas surface; intent returns as custom events / HTTP; a monotonic `server-revision` reconciles optimistic UI. One owner per DOM node.[^archive]

Full Rocket parity (client render/morph, shadow DOM, `$$` rewriting, local actions, light-DOM slots, `js` codec) is still larger than the demonstrated need. Do not clone it.[^archive]

## What the archive got wrong or left behind

Treat these archive claims as **stale**. Use the current column.[^archive]

| Archive (2026-08-14) | Current (2026-08-28) |
| --- | --- |
| Declaration sketches `flowCanvas = island "tag"` and `component \|{…}\|` | Shipped form is `@component Name = \|params\|`. Any `@island` must rhyme with that, not the old binding.[^components-ref][^ungram] |
| `crates/rocci-http/src/assets.rs` serves `.js` / `.mjs` | **`rocci-http` is gone.** Generated apps mount `assets/` via `Server.static_mount`. Preview injects `/assets/datastar.js`. Sites hash `goto.js` and, on `live` pages, Datastar.[^dispatch-rs][^view-rs][^plan-rs] |
| Compile emits no CSS artifacts | `CompileOutput` now has `styles`. It still has **no** JS island assets, island metadata, or client entry module.[^compile-rs] |
| AST is Roc + `Component` only | Also fixtures, tests, CSS, context, init, verb-first routes. Still **no** `IslandDecl`.[^ungram][^handlers] |
| `@on` / ad-hoc `/api/…` in examples | Closed `@method:role(path)`: view / fragment / command / `@get:live`. Commands are empty SSE vs 204. One-shot POST does not fan out.[^handlers][^cqrs] |
| Site CSP `script-src 'none'` for pages without islands | Rocdown site CSP is `script-src 'self'` because **every** page hashes `goto.js`. Datastar is still **live-only**. Island JS would be a third hashed script class, not the first.[^plan-rs] |
| No in-repo client JS island | Snake ships `assets/snake-input.js` and explicit `<script type="module">` tags. It is a **document-level** input module, not a custom element, and does not prove morph/CE identity.[^snake-js][^snake-rocci][^snake-study] |
| Hybrid / CDN splice absent | Rocdown `static` / `hydrate` / `live` is shipped and experimental. That is build-time Html splice + optional island **HTTP** service, not `@island` JS. Do not conflate the two.[^hydration] |
| Gaps “do not require changes to `rocci-core`, `rocci-http`, or `rocci-wry`” | Drop `rocci-http`. Client artifacts belong in `rocci-template` compile output plus `rocci-cli` / `rocci-rocdown` staging and hashing. Desktop still only needs to serve those files.[^compile-rs][^dispatch-rs] |
| Bundled `examples/datastar/assets/datastar.js` | Staging is `rocci-cli` `datastar_asset` (1.0.2, cache under `~/.rocci`). Still no Rocket.[^datastar-asset] |

Archive Stage 0–3 effort guesses and `ds:*` sketches are not a schedule. `ds:*` / `client { }` attribute literals remain a **separate** Snake ergonomics question.[^archive][^snake-study][^blockers]

## Current fit (what already works)

The **server half** of a Flow-shaped island still works without new grammar:[^lexer][^parser][^lower][^dispatch-rs]

- Lowercase tags, including hyphenated custom-element names (`<rocci-flow>`), scan as HTML elements (`alphanumeric`, `-`, `_`).
- Attribute names accept `-`, `:`, `_`, `.`, so `data-on:flow-node-commit` is representable.
- `@component` lowers to `Html.element` / Roc calls. PascalCase tags are component calls; there is still no generated wrapper for a browser host.
- Datastar morph and `@get:live` exist on standalone apps and hybrid `live` pages.
- Authors can already drop a file in `assets/` and write a module script tag (Snake). Generated dispatch serves that directory.

Illustrative **no-syntax** host (current language, not archive syntax):

```text
@component
FlowView = |{ graph }| {
    <rocci-flow
        id="main-flow"
        grid="32"
        server-revision={graph.revision_str}
        data-on:flow-node-commit="@post('/actions/flow/node')"
    >
        @for node in graph.nodes {
            <rocci-flow-node id={node.id} x={node.x_str} y={node.y_str}>
                <span>{node.label}</span>
            </rocci-flow-node>
        }
    </rocci-flow>
}
```

A hand-written `assets/flow.client.js` that `customElements.define`s those tags is still the right spike. Snake does **not** substitute: it listens on `window`, POSTs JSON to `/api/direction`, and never upgrades a host that Datastar morphs.[^snake-js][^snake-rocci][^archive]

## Current gaps (client artifact model)

Still missing, and still mostly compile/CLI work:[^compile-rs][^ungram][^archive]

- `IslandDecl`, prop/event schemas, generated Roc wrapper, `rocci-islands.js` registration entry.
- JS in `CompileOutput`, content-hashed client modules, automatic or shell-injected module entry **except** where authors write tags by hand.
- LSP JavaScript regions and island prop/event completions.
- Browser tests for the ten morph/lifecycle rules (stable host id, batched attrs, descriptor observation, cleanup, reconnect, revision reconcile, multi-instance, failed decode, no-JS fallback, untrusted payloads).[^archive]

Those ten rules remain the acceptance bar. Rust unit tests cannot replace them.[^archive][^blockers]

## Programming model (unchanged target, current names)

Keep three terms:[^archive][^decision]

1. **Server component** — today’s `@component`.
2. **Island host** — server-rendered custom element, hyphenated tag, stable `id`, typed attribute encoding.
3. **Island controller** — browser JS on that host. Ephemeral only.

v1 codecs stay String / Number / Bool / Json / OneOf. No `js` codec. Large collections as child descriptors. Events are `CustomEvent` with `bubbles` + `composed`; authors wire `data-on`. Tiny runtime: define, observe attributes, batch, cleanup — not a render library. Shadow DOM and Rocket-as-provider stay later.[^archive][^blockers]

## Staging (current recommendation)

Archive stages are still the right **order**. Dates and week-counts are not.[^archive][^blockers]

0. **Spike, no grammar** — custom elements + existing `@component` + today’s `@method:command` / `@get:live`. Kill if morph drops the instance. Snake input is not this spike.
1. **`@island Name = "tag-name"`** + `client_module` only + hashed entry + inspect.
2. Inline client fences and LSP richness.
3. Private `view` / shadow DOM / `apply(root)` / licensed provider only after demand.

Do not start Stage 1 from this record. The decision is unapproved and Stage 0 has no custom-element evidence.[^decision][^blockers]

## Relationship

- Historical dump: archive `DATASTAR_ROCKET_IN_ROCCI_REPORT.md`.[^archive]
- What blocks a plan: [`@island` design](client-behavior-islands.md).[^blockers]
- Hybrid CDN splice (different feature): [hydration comparison](../rocdown/hybrid-island-hydration.md).[^hydration]

[^archive]: 2026-08-14 ownership, Rocket-vs-component split, Stage 0–3, ten rules, no-bundle; syntax and crate map are stale.
[^snake-study]: Keyboard/canvas need a small client module; `client { }` literals are separate.
[^runtime-report]: Pure server components; no second domain store.
[^decision]: Exploratory; unimplemented; not approved.
[^blockers]: Plan-readiness questions and recommended defaults.
[^hydration]: Hybrid `hydrate` is build-time splice, not client hydration or `@island`.
[^parser]: Template items include HTML elements.
[^lexer]: Tag names allow `-`; attribute names allow `-` `:` `.`.
[^lower]: Server components and HTML become Roc `Html`.
[^compile-rs]: Styles exist; JS island artifacts do not.
[^ungram]: No `IslandDecl`.
[^components-ref]: `@component Name = |params|`.
[^datastar-asset]: Free 1.0.2; no Rocket.
[^view-rs]: Preview document injects Datastar as a module script.
[^dispatch-rs]: Generated `basic-webserver` serves `assets/`.
[^plan-rs]: `goto.js` site-wide; Datastar on `live` only.
[^snake-js]: Window keydown/pointer → JSON POST; not a custom element.
[^snake-rocci]: Authored module script tags in the play document.
[^handlers]: Verb-first closed matrix.
[^cqrs]: One-shot vs live; empty-SSE commands.
[^rocket-docs]: Rocket is a client custom-element API.
[^rocket-license]: Pro redistribution limits.
