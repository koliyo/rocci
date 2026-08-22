---
type: Implementation Plan
title: Bound Rocci handlers to server-rendered UI
description: "Add one-shot GET HTML fragments, remove the ordinary-client JSON representation from high-level commands, and expose patch-signals through low-level Roc helpers without completing a Datastar/HTTP syntax matrix."
tags: [domain/rocci, domain/runtime, integration/datastar, concern/language-design, concern/developer-experience]
status: draft
generated: { by: process:cursor, at: 2026-08-22T09:33:26Z }
stale_after: 2026-11-22
authority: exploratory
owners: [human:nils]
sources:
  - id: research
    resource: ../research/action-handler-syntax.md
    title: Research on a bounded Rocci UI handler surface
    author: process:cursor
    last_modified: 2026-08-22
  - id: original-plan
    resource: action-handler-syntax.md
    title: Original semantic-handler cutover plan
    author: process:cursor
    last_modified: 2026-08-21
  - id: template-ungram
    resource: ../../crates/rocci-template/Rocci.AST.ungram
    title: Current semantic handler AST
    author: process:git
    last_modified: 2026-08-21
  - id: template-parser
    resource: ../../crates/rocci-template/src/parser.rs
    title: Current handler parser and method diagnostics
    author: process:git
    last_modified: 2026-08-21
  - id: template-lower
    resource: ../../crates/rocci-template/src/lower.rs
    title: Current route response metadata and lowering
    author: process:git
    last_modified: 2026-08-21
  - id: dispatch
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Current generated handler dispatch
    author: process:git
    last_modified: 2026-08-21
  - id: handler-contract
    resource: ../../crates/rocci-template/tests/handler_contract.rs
    title: Frozen semantic handler contract
    author: process:git
    last_modified: 2026-08-21
  - id: handler-syntax
    resource: ../../crates/rocci-template/tests/handler_syntax.rs
    title: Handler syntax and diagnostics matrix
    author: process:git
    last_modified: 2026-08-21
  - id: custom-main
    resource: ../../examples/rocci/custom/datastar/main.roc
    title: Authored GET fragment routes
    author: process:git
    last_modified: 2026-08-20
  - id: datastar-roc
    resource: ../../crates/rocci-datastar/src/codegen/mod.rs
    title: Generated Roc Datastar helper
    author: process:git
    last_modified: 2026-08-17
  - id: datastar-sse
    resource: ../../crates/rocci-datastar/src/sse/events.rs
    title: Rust Datastar event builders
    author: process:git
    last_modified: 2026-08-17
  - id: server-state
    resource: ../decisions/server-owned-state.md
    title: Keep durable application state server-owned
    author: human:nils
    last_modified: 2026-08-16
  - id: datastar-actions
    resource: https://data-star.dev/reference/actions
    title: Datastar actions and response handling
    author: organization:star-federation
  - id: datastar-sse-reference
    resource: https://data-star.dev/reference/sse_events
    title: Datastar SSE event reference
    author: organization:star-federation
  - id: verb-first-plan
    resource: verb-first-handler-declarations.md
    title: Replacement implementation sequence for verb-first handlers
    author: process:cursor
    last_modified: 2026-08-22
---

# Bound Rocci handlers to server-rendered UI

## Purpose and authority

The shipped semantic handler cutover replaced `@on` with `@view`, `@patch`,
`@command`, and `@live`. This follow-up narrows their product boundary after
research found that negotiated command JSON is a partial API facility while
GET HTML fragments are an unserved server-rendered UI need.[^research]
The current AST and frozen contract establish those four shipped roles.
[^template-ungram][^handler-contract]

This plan is exploratory. Writing it does not approve the syntax, start a
phase, or change shipped behavior. The original cutover plan remains
historical evidence; this plan owns only the follow-up boundary.[^original-plan]

The later [verb-first handler plan](verb-first-handler-declarations.md)
resolves this plan's source-order and fragment-noun gate and supersedes it as
the proposed implementation sequence. This record remains the earlier boundary
analysis; no phase in either plan has started.[^verb-first-plan]

## Goal

Rocci standalone handlers form a small, predictable UI language:

| Role | Method policy | Successful value | Generated behavior |
| --- | --- | --- | --- |
| Document | GET | complete `Html` | `text/html` document |
| Fragment | GET, POST, PUT, PATCH, DELETE | stable-ID `Html` | one-shot patch-elements response |
| Command | POST, PUT, PATCH, DELETE | `{}` | no direct morph or JSON representation |
| Live | generated GET `/sse` | stable-ID `Html` | long-lived patch-elements stream |

Advanced Datastar operations such as patch-signals are typed Roc transport
helpers used by authored servers. General JSON resources, redirects, downloads,
custom statuses, and mixed response policies remain in authored `main.roc`.
[^datastar-actions][^datastar-sse-reference]

The fragment declaration's final noun is a Phase 0 gate. This plan uses
“fragment” as the semantic role and does not pre-approve either `@patch:get`
or a clean-cut rename to `@fragment:get`.

## Out of bound

- A complete method × content-type × SSE-event declaration matrix
- `@json`, `@signals`, `@script`, `@redirect`, or `@response` declarations
- A general response ADT or direct `Server.Outcome` return from ordinary
  high-level handlers
- Automatic public API schemas, authentication, authorization, versioning, or
  status-code design
- Durable domain state in Datastar signals
- Changes to `@live` polling, keepalive cadence, fan-out model, or generated
  `/sse` path
- Inferring response policy from an opaque Roc body or from the presence of
  `@live`
- Starting compatibility aliases before the naming gate decides whether a
  clean cut is acceptable

## Constraints that do not move

1. **HTML is the high-level UI boundary.** Documents, fragments, and live
   regions return `Html`; Datastar morphs stable IDs.[^server-state]
2. **Transport policy stays out of parsing.** Syntax records semantic role and
   method; `rocci-cli` generated dispatch owns SSE, content type, and request
   negotiation.[^template-lower][^dispatch]
3. **Ordinary Roc bodies stay opaque.** The parser cannot infer Html, unit, or
   JSON from body text; generated Roc must enforce the selected result shape.
   [^template-parser]
4. **GET fragments are documented as idempotent reads.** Rocci cannot prove
   the absence of effects in arbitrary Roc handler bodies.
5. **Commands never race a live owner.** A command does not patch an ID also
   rendered by `@live`.
6. **Public routes stay authored.** Method and path remain inspectable in AST,
   LSP, docs, logs, and generated dispatch.
7. **No hidden compatibility layer.** If Phase 0 chooses `@fragment`, migrate
   current source and keep focused removal diagnostics rather than two active
   nouns.
8. **Parser recovery keeps forward progress.** Every malformed new header path
   advances and preserves later declarations.

## Phase 0 — Freeze the bounded contract and fragment noun

**Bound**

- Add or revise a contract-only test matrix covering:
  - GET document;
  - GET search and tab fragments;
  - POST/PUT/PATCH/DELETE fragments;
  - mutation commands with `{}`;
  - one live fragment;
  - rejected command GET and rejected API/media-type declaration experiments.
- Preserve the current accepted/rejected matrix as the baseline for every
  intentional change.[^handler-syntax]
- Compare `@patch:get` with a clean-cut `@fragment:get` in complete source,
  formatted AST, diagnostics, handler inspection, and documentation prose.
- Freeze command success as no representation. Choose the exact generated
  success pair after an HTTP smoke test: current empty SSE for Datastar plus
  204 for ordinary callers, or one representation-free response accepted by
  both.
- Record whether the migration is a clean cut; do not create aliases in this
  phase.

**Exit**

- A maintainer selects the fragment noun and command wire policy.
- The frozen matrix names every accepted and rejected header without claiming
  JSON resources or patch-signals as declarations.
- No parser, lowerer, dispatcher, example, or public documentation behavior
  has changed yet.

## Phase 1 — Normalize route metadata by semantic response role

**Bound**

- Replace the internal method-based document shortcut with exhaustive semantic
  response metadata such as `Document`, `Fragment`, and `Command`.
- Lower `@view` to `Document`, the selected fragment declaration to
  `Fragment`, and `@command` to `Command`.
- Make generated dispatch branch on response role after method/path matching;
  GET no longer implies document by itself.
- Preserve handler function names, state/request adaptation, source spans,
  duplicate-route validation, and error overlays.
- Update focused lowering, inspect, generated-Roc, and source-map tests.

**Exit**

- A synthetic GET fragment reaches the fragment dispatch arm while GET view
  reaches the document arm.
- No dispatch branch relies on `method == GET` to infer response shape.
- Existing accepted handlers retain their pre-plan behavior.

## Phase 2 — Ship one-shot GET HTML fragments

**Bound**

- Update the ungram and sidecar only if Phase 0 chooses a new declaration node
  or noun; regenerate owned AST code rather than editing generated files.
- Parse and validate the selected GET fragment spelling.
- Keep GET rejected on commands and method suffixes rejected on views.
- Add malformed, unclosed, multiline, duplicate-route, formatter, inspect,
  LSP, and monotonic-recovery coverage.
- Add standalone search and tabs cases based on the existing authored custom
  app, proving that a Datastar `@get` action receives a stable-ID HTML fragment.
  [^custom-main]

**Exit**

- GET fragments compile, format idempotently, inspect with the correct role,
  and morph through generated standalone dispatch.
- GET documents and generated `GET /sse` remain distinct.
- The pinned Roc compiler builds the complete handler matrix.

## Phase 3 — Remove API JSON from high-level commands

**Bound**

- Constrain command success to `{}` in generated Roc and remove generated
  command JSON encoder wrappers.
- Remove JSON success negotiation, command encoder imports/helpers, and the
  promise that a `.rocci` command is an ordinary JSON API.
- Implement the Phase 0 no-representation success policy. Preserve the current
  Datastar developer error overlay; define a small non-Datastar error response
  without presenting it as a versioned API schema.
- Convert live-counter, hybrid counter, handler matrix, fixtures, and HTTP
  smoke tests from result records/lists to `{}`.
- Add a removal diagnostic for obsolete command result guidance where syntax
  can identify it; rely on Roc type errors for arbitrary non-unit body values.

**Exit**

- Successful commands cannot require `Encoding.Json` and emit no JSON body.
- Datastar command writes complete without morphing the live-owned region.
- Ordinary `curl` confirms representation-free success, while custom authored
  applications remain free to serve JSON.

## Phase 4 — Add low-level Roc patch-signals support

**Bound**

- Audit `rocci-datastar` event framing against the pinned Datastar version
  before copying its Rust assumptions into Roc.[^datastar-sse]
- Add a typed `Datastar.patch_signals` Roc event builder to the generated
  runtime helper, which currently exposes only basic patch-elements framing;
  include `onlyIfMissing` only if supported by the pin.[^datastar-roc]
- Keep the helper out of `.rocci` declaration parsing and generated standalone
  response selection.
- Add a custom `main.roc` fixture or focused generated-runtime test that emits
  patch-elements and patch-signals in one SSE response.
- Do not expand this phase into script execution, redirects, every patch mode,
  or a general event-stream DSL.

**Exit**

- An authored Roc server can emit a valid patch-signals event without manual
  SSE string framing.
- No new top-level Rocci declaration or durable signal-backed example exists.
- Rust and Roc event fixtures agree for the supported patch-signals subset.

## Phase 5 — Convert public contracts and examples

**Bound**

- Update the `rocci-template` README, server declaration reference, rendering
  concepts, tutorials, how-to pages, skills, all-syntax fixture, and app docs.
- Teach documents, fragments, commands, and streams as UI roles; explicitly
  send JSON resources and advanced Datastar events to authored `main.roc`.
- Update handler inspection and catalog prose to use the Phase 0 fragment noun.
- Remove ordinary-client JSON claims from live-counter and handler-matrix
  documentation.
- Keep generated output derived and inspect changed public pages after build.

**Exit**

- Repository search finds no active claim that high-level commands return JSON.
- Every accepted handler form appears in the complete matrix and public
  reference; rejected near-misses have one canonical rewrite.
- Public examples demonstrate one GET fragment, one mutation fragment, one
  command/live flow, and one authored patch-signals ceiling.

## Phase 6 — Validate the clean boundary

**Bound**

- Run focused parser/lowering tests, full `rocci-template`, `rocci-cli`, LSP,
  Rocdown-island consumers, ungram checks, all-syntax inspection, formatting,
  and workspace tests.
- Build the docs and inspect the server declaration, update-the-UI, and
  Datastar transport pages.
- Build and HTTP-smoke the standalone counter, live-counter, GET-fragment
  example, handler matrix, hybrid counter, and authored patch-signals fixture.
- Run the Rocci OKF profile and report lifecycle/provenance warnings separately
  from errors.
- Update this plan and the research with implemented phase status only after
  each exit gate has evidence. Do not mark the plan complete until CI and
  Knowledge workflows pass on the landed revision.

**Exit**

- All focused, consumer, workspace, documentation, and OKF checks pass.
- The generated server handles document GET, fragment GET, fragment mutation,
  representation-free command, and live GET without role ambiguity.
- Two-browser testing confirms command/live ownership without using command
  JSON as browser state.
- CI and Knowledge run IDs are recorded before any completion claim.

## Expected ownership

| Change | Owner |
| --- | --- |
| Handler grammar, AST, validation, lowering, diagnostics | `crates/rocci-template` |
| Generated response policy and Roc runtime staging | `crates/rocci-cli` |
| Low-level Datastar metadata and event framing | `crates/rocci-datastar` |
| Rocdown live-handler reuse | `crates/rocci-rocdown` and `crates/rocci-rocdown-cli` |
| Language and rendering documentation | `docs/` and owning crate READMEs |
| Canonical rationale and phase evidence | `knowledge/` |

## Final decision gate

Do not implement Phase 1 merely because this plan exists. The maintainer must
first approve Phase 0's fragment noun, clean-cut migration, and
representation-free command wire policy. If ordinary JSON APIs are declared a
first-class Rocci requirement at that gate, stop this plan and research a
single typed low-level response facility instead of restoring a growing set of
media-type declarations.

[^research]: Revised research separates GET fragments, JSON resources, signal patches, commands, and live streams.
[^original-plan]: Historical four-noun cutover and generated JSON-command design.
[^template-ungram]: Current separate view, patch, command, and live AST nodes.
[^template-parser]: Current method defaults, suffix validation, and body opacity.
[^template-lower]: Current route metadata cannot distinguish GET document from GET fragment.
[^dispatch]: Current method-first document branch and negotiated command JSON response.
[^handler-contract]: Existing frozen matrix and naming gate.
[^handler-syntax]: Current accepted forms and rejected GET mutation suffixes.
[^custom-main]: Existing custom application proves GET HTML fragments for search and tabs.
[^datastar-roc]: Generated Roc helper currently exposes only basic patch-elements framing.
[^datastar-sse]: Rust protocol layer already has patch-signals framing that must be checked against the pin.
[^server-state]: Normative server-owned durable state and coherent HTML boundary.
[^datastar-actions]: HTTP method and response content type are orthogonal Datastar dimensions.
[^datastar-sse-reference]: Patch-elements and patch-signals are SSE event types, not route declaration roles.
[^verb-first-plan]: Mandatory `@method:role` sequence retaining this plan's bounded UI, representation-free command, and low-level signal-helper conclusions.
