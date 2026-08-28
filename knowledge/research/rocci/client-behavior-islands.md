---
type: Research Report
title: "@island design: settled questions, blockers, and recommended approach"
description: "Client-behavior @island is designed, not approved, and not implemented. Do not write an implementation plan until a no-syntax spike proves Datastar morph plus custom-element identity on the current stack, and a short list of grammar/asset questions is answered."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/rendering, concern/architecture, concern/security, concern/language-design]
status: draft
generated: { by: process:cursor, at: 2026-08-28T17:55:00Z }
stale_after: 2026-11-28
authority: exploratory
owners: [human:nils]
sources:
  - id: rocket-current
    resource: ../datastar-rocket.md
    title: Datastar Rocket and Rocci-native islands (current)
    author: process:cursor
    last_modified: 2026-08-28
  - id: rocket-report
    resource: ../../../archive/reports/DATASTAR_ROCKET_IN_ROCCI_REPORT.md
    title: Historical Rocket-style client components report
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
  - id: format-report
    resource: ../../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-16
  - id: decision
    resource: ../../decisions/client-behavior-islands.md
    title: Explicit islands for browser-owned behavior
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: markdown-first
    resource: ../../decisions/markdown-first-explicit-islands.md
    title: Markdown-first with explicit executable islands
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: pure-render
    resource: ../../decisions/pure-render-components.md
    title: Pure render components
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: server-owned
    resource: ../../decisions/server-owned-state.md
    title: Server-owned durable state
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: hydration
    resource: ../rocdown/hybrid-island-hydration.md
    title: Hybrid island hydration versus SSR frameworks
    author: process:cursor
    last_modified: 2026-08-28
  - id: hybrid-plan
    resource: ../../plans/rocdown/hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands implementation plan
    author: process:cursor
    last_modified: 2026-08-24
  - id: compile-rs
    resource: ../../../crates/rocci-template/src/lib.rs
    title: CompileOutput without client artifacts
    author: process:git
    last_modified: 2026-08-25
  - id: ungram
    resource: ../../../crates/rocci-template/Rocci.AST.ungram
    title: Shipped ModuleItem set, no IslandDecl
    author: process:git
    last_modified: 2026-08-25
  - id: components-ref
    resource: ../../../docs/reference/language/components.rocdown
    title: Current @component declaration grammar
    author: process:git
    last_modified: 2026-08-25
  - id: plan-rs
    resource: ../../../crates/rocci-rocdown/src/plan.rs
    title: Site CSP, goto.js, Datastar only on live
    author: process:git
    last_modified: 2026-08-24
  - id: article-rs
    resource: ../../../crates/rocci-rocdown/src/article.rs
    title: PageKind classification
    author: process:git
    last_modified: 2026-08-23
  - id: rocdown-readme
    resource: ../../../crates/rocci-rocdown/README.md
    title: Reserved @island; hybrid uses @component
    author: process:git
    last_modified: 2026-08-24
  - id: datastar-asset
    resource: ../../../crates/rocci-cli/src/datastar_asset.rs
    title: Pinned Datastar 1.0.2 free bundle
    author: process:git
    last_modified: 2026-08-23
  - id: language-dev
    resource: ../../../.agents/skills/rocci-language-dev/SKILL.md
    title: Grammar and lowering ownership
    author: process:git
    last_modified: 2026-08-25
---

# `@island` design: settled questions, blockers, and recommended approach

## Research question

Where is Rocci on **client-behavior `@island`** (custom-element JS, not hybrid CDN splice)? What is already decided, what must be answered before an implementation plan, and what approach should that plan take?

This record is exploratory. It does not approve syntax and is not a plan. Writing a plan is a later step. Current Rocket-in-Rocci map: [datastar-rocket](datastar-rocket.md).[^decision][^hydration][^rocket-current]

## What `@island` is (and is not)

`@island` is the reserved name for an **explicit browser-owned surface**: keyboard, canvas/SVG, drag, media, observers, editors, maps, third-party widgets. It is a different construct from `@component` (pure Roc to Html) and from Rocdown **`hydrate` / `live`** (build-time Html splice plus optional Datastar service).[^decision][^pure-render][^hydration][^rocdown-readme]

It is **not** React/Astro client hydration of the page, not compiling Roc to JavaScript, and not Datastar Rocket bundled into Rocci. Ordinary `@component` remains a pure server renderer; an island must not become a second domain store.[^rocket-report][^snake-study][^runtime-report]

Public docs and the crate README already say the token is reserved and unparsed. Hybrid sites use existing `@component` and handlers.[^rocdown-readme][^markdown-first]

## Design status

| Layer | State |
| --- | --- |
| Architecture reports (2026-08-14/15) | Direction written; syntax illustrative; Stage 0 spike never landed |
| [Client-behavior islands decision](/decisions/client-behavior-islands.md) | Exploratory, draft, unimplemented. Not in the approved register |
| Language (`Rocci.AST.ungram`, `CompileOutput`) | No `IslandDecl`; compile emits Roc, CSS, routes — not JS artifacts |
| Hybrid Rocdown v1 | Explicitly out of bound; follow-on only |
| Motivating in-repo demo | Snake ships document-level `snake-input.js`; custom-element morph spike still missing |

The current Rocket restatement is [datastar-rocket](datastar-rocket.md). The 2026-08-14 archive is historical: pre-`@` syntax, `rocci-http`, and “no client JS” are stale. Reuse **ownership and staging**, not archive crate paths.[^rocket-current][^rocket-report][^compile-rs][^ungram][^components-ref][^plan-rs]

## Already settled (do not reopen in a plan)

These are consistent across the decision, the Rocket report, Snake, and later hybrid work. A plan should treat them as constraints.[^decision][^rocket-report][^server-owned][^hybrid-plan]

1. **`@island` ≠ `@component`.** Adding islands must not give ordinary render functions lifecycle, client state, or JS.
2. **Server owns durable state.** The island owns ephemeral interaction and an explicitly private visual surface. Intent goes back via custom events / HTTP / Datastar; the next server Html is authoritative.
3. **One owner per DOM node.** Datastar may morph host attributes and server-rendered descriptor children. The island must not render-loop the same light-DOM subtree.
4. **Behavior-only, light-DOM-first.** No client morph engine, no shadow DOM, no Rocket `$$` scope rewriting in v1.
5. **Large JS stays in `*.client.js`.** Do not invent a Roc-to-JS compiler. Do not grow a second client language to replace modules.
6. **Do not bundle Datastar Rocket.** Pro license vs open-source redistribution; surface larger than the demonstrated need. Bring-your-own is a later, licensed option only.
7. **Custom-element tags already parse** as lowercase HTML (`<rocci-flow>`). A no-syntax prototype does not need a grammar change.
8. **Hybrid CDN splice is a different track.** `@island` is not required to ship `static` / `hydrate` / `live`.

Snake’s `client { … }` **attribute literal** (multiline Datastar expressions) is a **separate** language question. Do not fold it into the first `@island` plan.[^snake-study]

## What the Rocket report already specified (still useful)

Keep these as the **v1 semantic target** once syntax is chosen:[^rocket-report]

- Island **host**: server-rendered custom element, hyphenated tag, stable `id`.
- Island **controller**: `connect` / attribute-change / disconnect, cleanup registry, emit `CustomEvent` (`bubbles` + `composed`).
- **Prop codecs** narrower than arbitrary Roc: String, Number, Bool, Json, OneOf. No `js` codec. Large collections as **child descriptors**, not one JSON attribute.
- **Generated Roc wrapper** so call sites stay `<FlowCanvas …>` / typed props, lowering to `Html.element` with encoded attributes.
- **Tiny `rocci-islands.js`**: define, observe attributes, batch changes, cleanup. Not a render library.
- **Ten acceptance rules** (stable host identity across morph, batched attrs, descriptor observation, cleanup, reconnect, revision-based optimistic UI, multi-instance isolation, failed decode → default + warning, no-JS fallback, untrusted payloads).

Those rules are the spike’s pass/fail, not a syntax debate.

## What must be answered before writing a plan

A plan that starts at `IslandDecl` in ungram without these answers will guess the expensive parts.

### Blockers (human or spike; do not default silently)

1. **Is the exploratory decision accepted?** Until a human accepts, revises, or rejects [client-behavior islands](/decisions/client-behavior-islands.md), an implementation plan has no approved product intent. This research can recommend; it cannot mint that decision as approved.[^decision]

2. **Does Datastar morph on the current stack preserve custom-element identity?** The Rocket report called this the highest-risk question and required a **Stage 0 spike with no new syntax**. That spike is still missing. Today’s stack also includes hybrid splice, wry/desktop preview, and Datastar 1.0.2 — none of which were proven for CE upgrade + morph + cleanup. **Do not plan grammar until this is yes or explicitly scoped as the first plan phase with a kill criterion.**[^rocket-report][^datastar-asset][^hydration]

3. **Which first example is in this repository?** Snake already has document-level `snake-input.js`. That is not a custom-element morph spike. Candidates for Stage 0: wrap Snake input in a host element that Datastar morphs, or a descriptor-list / canvas island. A copy button alone does **not** answer question 2.[^snake-study][^rocket-current][^rocket-report]

4. **Standalone `.rocci` first, or Rocdown the same week?** Format research wanted one `@island` in both languages and `@render` of the generated host. Hybrid v1 forbade `@island`. A first plan can be `rocci-template` + one standalone example only; Rocdown classification, CDN JS hashing, and `hydrate` vs JS then become a follow-on. That split should be explicit.[^format-report][^hybrid-plan][^article-rs]

### Design questions a plan may answer (with a recommended default)

5. **Declaration grammar.** Shipped components are `@component Name = |params|`. Rocket sketches are `name = island "tag" { props … }`. Rocdown sketched `@island copyButton = "rocci-copy-button" { @client module(…) }`. **Recommendation:** match `@component` visibility: `@island Name = "tag-name"` plus a small body for `module` / props / events. PascalCase Roc name, hyphenated tag. Do not reuse `ComponentDecl`. Language work is `$rocci-language-dev`.[^components-ref][^ungram][^language-dev][^format-report]

6. **v1 controller form.** Inline `client """…"""` vs `module("./X.client.js")` only. **Recommendation:** **external module only** in v1 (Rocket Stage 1). Inline fences need a JS region in the scanner/LSP and invite putting Flow-sized code in `.rocci`.[^rocket-report]

7. **When does JS load?** Pages with no island must not pull `rocci-islands.js`. Site CSP is already `script-src 'self'` because **every** Rocdown page hashes `goto.js` — the old `script-src 'none'` story is stale. **Recommendation:** hash island modules like Datastar; inject `<script type="module">` only on pages that reference an island (catalog-driven, same as live → Datastar). Standalone apps: inject from the document shell when the compile graph has islands, not from inside `@component` Html.[^plan-rs][^rocket-report]

8. **Does `@island` change `PageKind`?** Classification today: Rocci markup → `hydrate`; handlers / Datastar → `live`. An island without handlers still needs JS but not Datastar. **Recommendation:** no fourth kind. `@island` stays **`hydrate`** (CDN files, including hashed JS). Handlers still promote **`live`**. `--cdn-only` remains valid for island-only pages. Do not call this “hydration” in docs.[^article-rs][^hydration][^rocdown-readme]

9. **Artifact model.** `CompileOutput` has no `islands` / JS `assets[]`. **Recommendation:** add that set in the same change as `IslandDecl`; write generated modules into the build/staging tree (not beside sources); content-hash in production. This is CLI/packaging work, not parser-only.[^compile-rs][^rocket-report]

10. **Script tag ownership.** Rocket preferred an explicit `<RocciClient />` helper so CSP and load order stay visible. **Recommendation:** generated shell injection for v1 (one module entry), plus `rocci inspect` listing island assets. Add an authored opt-out / explicit tag only if injection cannot see `<html>`/`<body>` (same class of problem as island-fragment `data-init`).[^rocket-report]

11. **Prop/event surface in v1.** **Recommendation:** String / Number / Bool / Json / OneOf; declared events → `CustomEvent`; authors wire `data-on:…` themselves. Typed `ds:*` helpers wait for a Snake expression plan. No `js` codec.[^rocket-report][^snake-study]

12. **Shadow DOM, private `view`, Rocket provider.** **Recommendation:** out of v1, as in Rocket Stages 2–3. Revisit only after a real island needs encapsulation or Datastar `apply(root)`.[^rocket-report]

13. **Tests.** Parser/lowering/codecs in Rust. Morph, upgrade, cleanup, reconnect, two windows: **browser tests**. A plan that only adds crate unit tests has not implemented the feature.[^rocket-report]

## Recommended approach

Do **not** start with ungram + `@island` on the assumption the 2026-08-14 report is a ready plan. Sequence:

```text
0. Accept or defer the exploratory decision (human).
1. Stage 0 spike, no grammar: *.client.js + lowercase tags + existing @component.
   Kill criterion: morph drops custom-element instance or cleanup fails.
2. Only then: implementation plan for Stage 1
   (@island + client_module + codecs + hashed module entry + inspect).
3. Rocdown site emission and docs as a follow-on or last phases of that plan.
4. Inline client fences, LSP richness, shadow DOM: later plans.
```

**Stage 0 (now, still not a language plan):**

- One example under `examples/` (prefer a **morph-heavy** surface: canvas or descriptor list + SSE/patch; Snake input if that is the product need).
- Hand-written `customElements.define`.
- Server descriptors from `@component`; private surface in the controller.
- `data-on` for intent; `@method:command` / `@get:live` as they exist today.
- Prove the ten acceptance rules that apply (identity, batch, cleanup, reconnect, two instances).
- Record results in an audit or a short addendum here. If the spike fails, stop — do not add syntax to paper over morph races.

**Stage 1 plan (only after 0):**

- `@island Name = "tag-name"` in `rocci-template` (ungram, parse, validate, lower).
- Generated Roc wrapper + `IslandProp` encoders + registration entry that imports the author’s module.
- Light + `client_module` only.
- CLI stages hashed JS; document/shell injects one module script when islands exist.
- `rocci inspect` shows islands and assets.
- Browser lifecycle tests in CI.
- Public language page: reserved → experimental, with the ownership rule in one paragraph.

**Reject in any first plan:**

- Bundling Rocket; compiling Roc to JS; implicit JS on every `@component`.
- Client render/morph, shadow DOM, `js` codec, inline fences, `ds:*` / `client { }` literals.
- Making `@island` a synonym for hybrid `live` pages.
- Astro-style `client:idle` / `client:visible` (no client component tree to schedule).[^hydration]

## Why this order

The missing work is **not** “pick a prettier `@island` spelling.” It is (a) product approval, (b) morph/CE evidence on **this** Datastar and preview host, (c) a compile/serve artifact path that today’s `CompileOutput` does not have. Syntax is the cheap slice and the one agents will over-implement if a plan starts at the parser.[^compile-rs][^rocket-report][^language-dev]

## Relationship to other records

- Decision (exploratory): [Use explicit islands for browser-owned behavior](/decisions/client-behavior-islands.md).[^decision]
- Hybrid splice vs this feature: [hybrid island hydration](../rocdown/hybrid-island-hydration.md).[^hydration]
- Current Rocket map: [Datastar Rocket and Rocci-native islands](datastar-rocket.md).[^rocket-current]
- Historical dump: archive `DATASTAR_ROCKET_IN_ROCCI_REPORT.md`.[^rocket-report]
- Input-island case: archive `SNAKE_DATASTAR_ARCHITECTURE_REPORT.md`.[^snake-study]

[^rocket-current]: 2026-08-28 restatement: stale archive anchors, Snake module, current compile/CSP map.
[^rocket-report]: Ownership split, Stage 0–3, no Rocket bundle, ten acceptance rules; pre-`@` syntax and `rocci-http` anchors are stale.
[^snake-study]: Keyboard/canvas belong in a small client module; `client { }` literals are a different proposal.
[^runtime-report]: Pure server components; islands must not become a second domain store.
[^format-report]: Shared `@island` in Rocdown once Rocci has the declaration; grammar not stabilized.
[^decision]: Proposed, unimplemented, not approved.
[^markdown-first]: `@island` reserved; must not be presented as shipped.
[^pure-render]: `@component` stays a Roc function to Html.
[^server-owned]: Durable state on the server; island is ephemeral.
[^hydration]: Hybrid `hydrate` is build-time splice; `@island` is the missing JS story.
[^hybrid-plan]: `@island` listed as follow-on, not v1.
[^compile-rs]: `CompileOutput` has Roc, styles, routes; no JS island artifacts.
[^ungram]: `ModuleItem` has no `IslandDecl`.
[^components-ref]: Shipped `@component Name = |params|` is the grammar to rhyme with.
[^plan-rs]: `goto.js` on every site page; Datastar only when `live`.
[^article-rs]: `hydrate` vs `live` promotion rules have no island item today.
[^rocdown-readme]: `@island` reserved; hybrid uses `@component` / handlers.
[^datastar-asset]: Free Datastar 1.0.2; no Rocket.
[^language-dev]: Grammar/lowering changes belong to the language-dev skill, not stack policy.
