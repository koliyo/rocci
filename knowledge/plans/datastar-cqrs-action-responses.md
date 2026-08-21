---
type: Implementation Plan
title: Generated CQRS streams and JSON command responses
description: "Generate Datastar CQRS machinery (SSE unfold, data-init, 204 vs JSON) behind an opt-in @live render. Keep the first-app counter as one-shot patches. Add a live-counter example and convert the hybrid island. Do not require authors to write Wait/Emit or a /sse handler."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/architecture, concern/rendering, concern/syntax]
status: draft
generated: { by: process:cursor, at: 2026-08-21T09:02:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/datastar-cqrs-action-responses.md
    title: Why generated POST SSE is not multi-client fan-out
    author: process:cursor
    last_modified: 2026-08-21
  - id: dispatch-rs
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated main.roc html_ok versus patch_html
    author: process:git
    last_modified: 2026-08-20
  - id: datastar-roc
    resource: ../../crates/rocci-cli/runtime/Datastar.roc
    title: OpenWhenHidden and patch_elements helpers
    author: process:git
    last_modified: 2026-08-15
  - id: lower-rs
    resource: ../../crates/rocci-template/src/lower.rs
    title: RouteInfo and @on lowering
    author: process:git
    last_modified: 2026-08-20
  - id: ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: OnDecl grammar
    author: process:git
    last_modified: 2026-08-19
  - id: parser-rs
    resource: ../../crates/rocci-template/src/parser.rs
    title: OnDecl parse and is_http_method
    author: process:git
    last_modified: 2026-08-17
  - id: counter
    resource: ../../examples/rocci/standalone/counter/Counter.rocci
    title: First-app counter stays one-shot
    author: process:git
    last_modified: 2026-08-20
  - id: counter-readme
    resource: ../../examples/rocci/standalone/counter/README.md
    title: Counter README documents curl of POST SSE
    author: process:git
    last_modified: 2026-08-20
  - id: hybrid-counter
    resource: ../../examples/rocdown/counter/index.rocdown
    title: Hybrid counter island becomes the live shared demo
    author: process:git
    last_modified: 2026-08-20
  - id: snake-main
    resource: ../../examples/rocci/custom/snake/main.roc
    title: Reference stream unfold and empty command SSE
    author: process:git
    last_modified: 2026-08-20
  - id: server-actions
    resource: ../../docs/guides/server-actions.rocdown
    title: Public server-actions guide
    author: process:git
    last_modified: 2026-08-20
  - id: rendering-doc
    resource: ../../docs/concepts/rendering-model.rocdown
    title: Published rendering model
    author: process:git
    last_modified: 2026-08-18
  - id: template-readme
    resource: ../../crates/rocci-template/README.md
    title: Standalone HTTP contract
    author: process:git
    last_modified: 2026-08-20
  - id: rocci-ref
    resource: ../../docs/reference/rocci.rocdown
    title: Public @on reference
    author: process:git
    last_modified: 2026-08-20
  - id: author-skill
    resource: ../../.agents/skills/rocci-author/SKILL.md
    title: Authoring server-app table
    author: process:git
    last_modified: 2026-08-21
  - id: stack-skill
    resource: ../../.agents/skills/rocci-stack/SKILL.md
    title: One-shot versus live CQRS composition rules
    author: process:git
    last_modified: 2026-08-21
  - id: language-dev
    resource: ../../.agents/skills/rocci-language-dev/SKILL.md
    title: Grammar skill notes shipped @live and json
    author: process:git
    last_modified: 2026-08-21
  - id: service-rs
    resource: ../../crates/rocci-rocdown/src/service.rs
    title: Islands reuse generated dispatch
    author: process:git
    last_modified: 2026-08-20
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Durable application state is server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: compile-tests
    resource: ../../crates/rocci-template/tests/compile.rs
    title: "Rocci @on lowering and Datastar action tests"
    author: process:git
    last_modified: 2026-08-20
  - id: ds-tao
    resource: https://data-star.dev/guide/the_tao_of_datastar
    title: Tao CQRS and 0-n SSE events
    author: organization:star-federation
  - id: ds-actions
    resource: https://data-star.dev/reference/actions
    title: 204 empty body, openWhenHidden, JSON-as-signals
    author: organization:star-federation
---

# Generated CQRS streams and JSON command responses

## Purpose and authority

The [research](../research/datastar-cqrs-action-responses.md) shows that
Datastar fan-out is Tao CQRS, while generated Rocci only does one-shot POST
patches. Rocci can generate the stream loop; authors should opt in with a
live render rather than copying Snake’s `Wait`/`Emit`. This plan implements
that split, keeps the first-app counter simple, and adds a live example. It
does not mint a new architecture decision; it applies
[server-owned state](../decisions/server-owned-state.md) and Datastar's
published CQRS guidance.[^research][^server-state][^ds-tao]

Exploratory. Phases 1–5 are implemented on `datastar-cqrs-action-responses`;
not CI-complete. Phase 6 aligns composition skills and knowledge indexes with
that branch. Do not start a residual item until the user asks.

## Goal

A generated standalone or island app can opt into a shared live region
without an authored `main.roc`:

1. `@live = |state| { fragment }` is the author-facing opt-in. Rocci
   generates `GET /sse` (poll + `datastar-patch-elements`) and injects
   `data-init=@get("/sse", [OpenWhenHidden(Bool.true)])` on document `body`
   when that attribute is absent.
2. A POST marked `json` returns **204** when `Datastar-Request: true`, and
   **200** `application/json` otherwise. Unmarked POST stays today’s
   one-shot HTML patch (forms).
3. `examples/rocci/standalone/counter` stays one-shot. A new
   `live-counter` sibling and `examples/rocdown/counter` demonstrate two
   browsers updating without refresh. `curl -X POST` on a `json` action
   returns `{"count":N}`.[^counter][^counter-readme][^hybrid-counter][^research]

## Out of bound

- Platform pub/sub or waking other SSE connections except by polling
  (Snake `Wait` + `After`).[^snake-main]
- Binding the visible count to `$count` / `datastar-patch-signals` as the
  shared-state mechanism.[^ds-actions][^server-state]
- Returning `Server.Outcome` from handler bodies.
- Rewriting Snake, the Datastar gallery, or converting the first-app
  counter to CQRS.
- Forcing every POST onto CQRS, or inferring live mode from “there is a
  POST.”
- Requiring authors to write `Sse.unfold!` / `Wait` / `After`.
- Changing the Datastar JS pin or SDK event names.
- Per-tab LiveView processes or a retained server VDOM.
- Content negotiation via `Accept` alone (Datastar sends html, json, and
  event-stream together).[^ds-actions]
- New HTTP methods beyond get/post/put/patch/delete.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Dispatch arity | Generated calls stay `handler!(context, request)`.[^lower-rs] |
| Pure render | `@component` still returns `Html`. Stream handlers call those functions. |
| Fat morph by id | Stream events use `Datastar.patch_elements` / default outer morph.[^datastar-roc] |
| One boundary | Do not patch `#counter` from both the command and the stream. |
| Method defaults | Unmarked `GET` → document; unmarked other method → one-shot patch. `@live` does not change that. |
| First-app counter | `examples/rocci/standalone/counter` remains the one-shot tutorial.[^counter-readme] |
| Islands share dispatch | Rocdown live modules go through the same `generate_bound_main_roc`.[^service-rs] |
| No secrets in git | Counter JSON is `{ "count": N }` only. |

## Phase 1 — Freeze live opt-in and example split

Record (already) two modes and who writes what:

| Mode | Author writes | Rocci generates |
| --- | --- | --- |
| Default (one-shot) | `@on:post` returns Html | Today's `patch_html!` |
| Live | `@live` loader returning Html; commands marked `json` return a JSON `Str` | `GET /sse` poll unfold; body `data-init`; 204 vs JSON |

`@live` is module-level, one per module, same parameter rules as `@on`
(context record, optional request). It is not `@component`. Duplicate
`@live` is a diagnostic.

`json` spelling: optional ident after the `@on` path, before `=`. Unknown
idents are diagnostics. `json` on GET is a diagnostic.

Do **not** require `@on:get("/sse") stream` in examples. That ident is
residual (escape hatch).

Examples:

- Keep `standalone/counter` one-shot; README may note a second tab stays
  stale until click/refresh and point at live-counter.[^counter-readme]
- Add `examples/rocci/standalone/live-counter/` (same SQLite card, `@live`,
  json increment/reset).
- Convert `examples/rocdown/counter` to `@live`; drop `/actions/counter/sync`.

Poll: `After(100)`. Skip emit when rendered fragment bytes are unchanged.

**Exit:** This section. No code.

## Phase 2 — Grammar: `@live` and `json`

Bound:

- `ModuleItem` gains `LiveDecl`: `'@' 'live' params:RocExpr? body:RocExpr`.[^ungram]
- Parser/lowering: `live!` (or `live_patch!`) on the module, wrapping `?`
  like `@on`. Compile tests cover one `@live` and reject two.[^parser-rs][^compile-tests]
- `OnDecl` optional respond ident `json` (and `empty` if still wanted).
  `RouteInfo` grows `respond: Patch | Json` (`Empty` optional).[^lower-rs]
- `AllSyntax.rocci` includes `@live` and a `json` POST.

Out of this phase: dispatcher, injection, examples.

**Tests / Exit:** `cargo test -p rocci-template`. `cargo fmt --all -- --check`.

## Phase 3 — Generated dispatcher and `data-init`

Bound in `crates/rocci-cli/src/dispatch.rs` (islands inherit):[^dispatch-rs]

- If the primary module has `@live`, emit `GET /sse` (409/diagnostic if the
  app already registered that path) as Snake-style unfold calling
  `Type.live!(context, request)` (inject `_request` like `@on`).
- **json** POST: `Datastar-Request: true` (header match case-insensitive) →
  204. Else 200 `application/json` with the handler `Str`. Handler `Err` →
  Datastar HTML overlay; API 500 JSON `{"error":"..."}`.
- **patch** POST: unchanged `patch_html!`.
- Lower document templates in a `@live` module: if the root `<body>` has no
  `data-init`, add
  `data-init=@get("/sse", [OpenWhenHidden(Bool.true)])`. If `data-init`
  exists, leave it (do not merge).[^datastar-roc]
- Collision: authored `@on:get("/sse")` plus `@live` is a diagnostic.

Unit tests on generated `main.roc` strings: `@live` ⇒ `Sse.unfold!` and
`After(100)`; json arm contains `Datastar-Request`; unmarked POST still
`patch_html!`; no `@live` ⇒ no `/sse` arm.

**Exit:** `cargo test -p rocci-cli`. `cargo test -p rocci-template`.
`cargo fmt --all -- --check`.

## Phase 4 — Examples

- New `examples/rocci/standalone/live-counter/` cloned from the simple
  counter: `@live` reads and returns `counterCard`; increment/reset marked
  `json` return `{"count":N}`; page lede says two browsers share the
  stream. README: two-window check; `curl -X POST` without Datastar-Request
  shows JSON, not `datastar-patch-elements`.
- Do **not** change handler behavior of `examples/rocci/standalone/counter`.
  Optional README sentence pointing at live-counter.[^counter][^counter-readme]
- Hybrid `examples/rocdown/counter`: `@live` + json commands; remove sync
  POST; first stream event replaces snapshot `0`.[^hybrid-counter]
- Do not convert gallery or Snake.

**Tests / Exit:** `cargo test -p rocci-cli`. `cargo test -p rocci-template`.
`cargo fmt --all -- --check`. Manual two-browser check stays in the
live-counter README (no new Roc CI job unless one already runs the counter).

## Phase 5 — Public docs and authoring skill

- `docs/guides/server-actions.rocdown`: two patterns; simple counter vs
  live-counter; 204 vs JSON; hybrid is live.[^server-actions]
- `docs/concepts/rendering-model.rocdown`: generated `/sse` from `@live`,
  not only authored `main.roc`.[^rendering-doc]
- `crates/rocci-template/README.md` and `docs/reference/rocci.rocdown`:
  `@live`, `json` respond ident.[^template-readme][^rocci-ref]
- `.agents/skills/rocci-author/SKILL.md`: live row is `@live`, not
  authored `main.roc`.[^author-skill]

**Exit:** Those files agree with Phase 1. `cargo test -p rocci-template`.
`cargo fmt --all -- --check`.

## Phase 6 — Stack skill and knowledge records

Bound:

- `.agents/skills/rocci-stack/SKILL.md`: `@live` and `json` are shipped
  composition, not planned. Hybrid counter is live. Do not tell authors to
  copy Snake’s unfold.[^stack-skill]
- `.agents/skills/rocci-language-dev/SKILL.md`: `@live` / `json` exist; further
  declarations still need `$rocci-stack` and a plan.[^language-dev]
- This plan, the companion research disposition, collection indexes, and
  `knowledge/log.md`: Phases 1–5 executed on this branch, not CI-complete.

Out of this phase: new grammar, dispatcher changes, examples, public Rocdown
rewrites already done in Phase 5.

**Exit:** Those files agree that generated `@live` is on this branch. `cargo run -q -p rocci-okf -- check knowledge --profile rocci --format terminal`.

## Residual (not this plan)

- Explicit `@on:get("/sse") stream` escape hatch.
- `rocci.toml` `[datastar] live = true` fat-morph of document GET (medium).
- Named live regions (`#counter` and `#hud` on one stream).
- Process revision / platform wake (stop hashing Html every tick).
- Roc `Encode` instead of authored JSON strings.
- `empty` respond kind if Phase 1 drops it.
- Gallery remaining on direct patches (intentional).
- Cross-origin cookie/CORS for hybrid `/sse` (hosting follow-ons).

[^research]: Direct POST SSE vs CQRS; Datastar-Request vs Accept; JSON-as-signals.
[^dispatch-rs]: `route_arm` GET vs `patch_html!`; tests assert generated strings.
[^datastar-roc]: `get_with` / `OpenWhenHidden` already exist for authored attributes.
[^lower-rs]: `RouteInfo` method, path, fn_name, span today.
[^ungram]: `OnDecl` has no respond field.
[^parser-rs]: `empty_on` and `is_http_method` are the parse hook.
[^counter]: Current increment returns `counterCard` as a one-shot patch.
[^counter-readme]: Smoke curl expects `datastar-patch-elements` on POST; keep that for the first app.
[^hybrid-counter]: Sync POST plus increment; islands use the same dispatcher.
[^snake-main]: `stream_game!` `After(125)` and `empty_sse!` for commands.
[^server-actions]: Guide to rewrite so two-window sharing is the stream.
[^rendering-doc]: Handler table still says authored long-lived SSE.
[^template-readme]: Documents the POC skip this plan closes for `/sse`.
[^rocci-ref]: `@on` public contract.
[^author-skill]: Server-app kind table.
[^stack-skill]: Stack skill still said generated `@live` was planned after Phase 5.
[^language-dev]: Grammar skill still treated `@live` as a future declaration.
[^service-rs]: `into_app_plan` reuses CLI dispatch.
[^server-state]: Canonical reread and HTML morph; no client domain store.
[^compile-tests]: Lowering tests for `@on` and Datastar `@post`.
[^ds-tao]: CQRS long-lived GET; 0–n events on a response.
[^ds-actions]: 204 empty; JSON patches signals; `openWhenHidden`.
