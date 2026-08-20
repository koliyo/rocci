---
type: Implementation Plan
title: Snapshot eval must not compile service-only @roc
description: "Live Rocdown pages compile twice against two Roc platforms. lower_islands copies every @roc helper into the basic-cli snapshot module, so unused pf.Sqlite helpers fail CDN splice. Omit snapshot-unreachable @roc from that compile so authors keep one @roc block."
tags: [domain/rocdown, domain/rocci, integration/roc, concern/architecture, concern/rendering]
status: draft
generated: { by: process:cursor, at: 2026-08-20T13:05:00Z }
stale_after: 2026-11-20
authority: exploratory
owners: [human:nils]
sources:
  - id: islands-rs
    resource: ../../crates/rocci-rocdown/src/islands.rs
    title: CDN island snapshot evaluation against basic-cli
    author: process:git
    last_modified: 2026-08-20
  - id: lower-rs
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: lower_islands filters snapshot-unreachable @roc
    author: process:git
    last_modified: 2026-08-20
  - id: service-rs
    resource: ../../crates/rocci-rocdown/src/service.rs
    title: Island service compile of live modules
    author: process:git
    last_modified: 2026-08-20
  - id: dispatch-rs
    resource: ../../crates/rocci-cli/src/dispatch.rs
    title: Generated basic-webserver main.roc
    author: process:git
    last_modified: 2026-08-20
  - id: page-rs
    resource: ../../crates/rocci-rocdown/src/page.rs
    title: split_roc_body and roc_binding_names
    author: process:git
    last_modified: 2026-08-20
  - id: compile-tests
    resource: ../../crates/rocci-rocdown/tests/compile.rs
    title: compile_islands keeps @roc values used by islands
    author: process:git
    last_modified: 2026-08-20
  - id: counter
    resource: ../../examples/rocdown/counter/index.rocdown
    title: Hybrid counter live page
    author: process:git
    last_modified: 2026-08-20
  - id: rocdown-ref
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown declaration table
    author: process:git
    last_modified: 2026-08-20
  - id: pages-guide
    resource: ../../docs/guides/rocdown-pages.rocdown
    title: Authoring guide for @roc values
    author: process:git
    last_modified: 2026-08-20
  - id: hybrid-plan
    resource: hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: research
    resource: ../research/island-snapshot-roc-reachability.md
    title: Why snapshot eval cannot share pf.Sqlite with handlers
    author: process:cursor
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: lib-rs
    resource: ../../crates/rocci-rocdown/src/lib.rs
    title: BASIC_CLI_PLATFORM constant
    author: process:git
    last_modified: 2026-08-20
---

# Snapshot eval must not compile service-only `@roc`

A live `.rocdown` page is compiled twice. CDN splice evaluates islands as a
`basic-cli` stdout program. The island service compiles the same file as a
`basic-webserver` app. `lower_islands` already drops `@on` / `@init` /
`@context`. It used to copy **every** `@roc` statement into the snapshot
module. Roc typechecks unused helpers. `import pf.Sqlite` therefore means two
different APIs in one authoring file.[^islands-rs][^lower-rs][^service-rs][^dispatch-rs][^lib-rs][^research]

That is a compiler leak. Authors must not learn “put Sqlite helpers in `@on`,
not `@roc`.” `@roc` remains the place for Roc values **and** helpers. Snapshot
eval must keep only what `@render` and island components actually use.

## Why the two platforms exist

| Compile | Platform | Entry | Must include |
| --- | --- | --- | --- |
| CDN splice (`evaluate_page`) | `basic-cli` 0.22 | `rocci_islands({})` then `Html.render` to stdout | Components and values the snapshot HTML needs |
| Island service (`compile_live_modules`) | `basic-webserver` 0.16 | generated `respond!` over `@on` | `@context` / `@init` / `@on` plus the same `@roc` and components |

Snapshot eval cannot be the webserver: it is a one-shot renderer with no HTTP
listen, matching site apply.[^islands-rs][^hybrid-plan] Island handlers cannot
be `basic-cli`: they are HTTP. Both are correct. The bug is feeding
service-only Roc into the snapshot compile.

`pf` is the platform module in both generated `main.roc` files. basic-cli
Sqlite is path/bindings-shaped. basic-webserver Sqlite is connection-shaped
(`{ db, query, params }`). A helper written for `@init` / `@on` is legal in
the service module and illegal in the snapshot module even when
`rocci_islands` never calls it.[^dispatch-rs]

`lower_islands` already omits handlers: rocci items are only `@component` /
`@fixture` / `@css`. Phase 2 then drops snapshot-unreachable `@roc` rest and
imports.[^lower-rs]

Document-root text that looks like Roc (`read_count! = |db| …` between
directives) is Markdown, not a module item. Helpers belong in `@roc` or a
sibling `.roc` imported from `@roc`. Do not teach a third hiding place.[^rocdown-ref][^page-rs]

`@component` stays pure. Snapshot HTML bakes props (`count: 0`). A render
expression that calls a Sqlite helper is asking snapshot eval to do handler
work; that should keep failing at Roc compile until a later diagnostic names
it. Do not run `@init` at splice time.[^pure-render][^counter]

## Goal

A live page may declare `read_count!` / `increment_count!` in `@roc` next to
`import pf.Sqlite`, use those helpers only from `@on` / `@init`, and still
splice CDN HTML. `compile_islands` output does not contain the unused helpers
or the unused `pf.Sqlite` import. `compile` (island service) still does.
`examples/rocdown/counter` uses those named helpers again.

## Out of bound

- Unifying basic-cli and basic-webserver Sqlite, or wrapping them in a Rocci
  facade authors must import.
- Compiling snapshot eval as `basic-webserver`, or running `@init` while
  splicing.
- Making `@component` effectful so it can query Sqlite during render.[^pure-render]
- New syntax (`@service`, “handler-only `@roc`”).
- Changing hydrate-only pages except that they share `lower_islands` (keep
  `@roc` values that islands use, as today).[^compile-tests]
- Teaching authors to inline queries in `@on` as the product contract.

## Constraints that do not move

| Keep | Meaning |
| --- | --- |
| Two platforms | Snapshot = basic-cli stdout; service = basic-webserver. |
| One `@roc` | Values and helpers live in the same block. |
| Pure render | Components take baked props; handlers own IO. |
| Full service module | Island `compile()` still emits all `@roc` rest. |
| Existing island export | `rocci_islands = \|{}\|` stays the snapshot entry.[^compile-tests] |

## Phase 1 — Lock the failure in tests

Add `compile_islands` / `compile` fixtures (no Roc required):

- Live page: `@roc` has `import pf.Sqlite` and `read_count! = …` using the
  webserver query shape; `@on:post` calls `read_count!`; `@render` calls
  `counterCard({ count: 0.I64 })`.
- Today: `compile_islands` Roc **contains** `read_count!` and `import pf.Sqlite`
  (this is the bug). Assert that so the next phase flips the assertion.
- `compile` Roc still contains both after the fix.

Keep the existing hydrate test that `feature_count` survives when
`<FeatureCount count={feature_count} />` is an island.[^compile-tests]

**Exit:** `cargo test -p rocci-rocdown --test compile` covering those fixtures;
`cargo fmt --all -- --check`.

## Phase 2 — Reachability in `lower_islands`

Bound:

- Compute names used by island items (`@render`, document-root template
  tags/directives, not Markdown). Include component bodies those items
  instantiate.
- Fixpoint over `@roc` rest: keep a statement if its binding name (including
  `foo!`) appears in kept text, or if it is a type alias whose name appears
  there. Reuse `roc_binding_names` / `split_roc_body`; do not parse Roc.[^page-rs]
- Drop `@roc` rest that is not kept. Drop imports whose imported name does
  not appear in kept rest or island text (`Sqlite` from `import pf.Sqlite`).
- If a name cannot be classified, keep the statement (fail closed toward
  today’s emit).
- `compile()` / island service lowering is unchanged.

Tests: Phase 1 fixture now asserts snapshot Roc has `rocci_islands` and
`counterCard`, and does **not** contain `read_count!` or `import pf.Sqlite`.
A second fixture keeps `feature_count` and a helper used **from** `@render`.

**Exit:** those tests; `cargo test -p rocci-rocdown --lib`;
`cargo fmt --all -- --check`.

## Phase 3 — Counter uses `@roc` helpers again

Restore `read_count!` / `increment_count!` / `reset_count!` in
`examples/rocdown/counter/index.rocdown` `@roc`, called from `@on`. Snapshot
stays `counterIsland({ count: 0.I64 })`.[^counter]

**Exit:** `cargo test -p rocci-rocdown-cli --test islands` (counter POST still
morphs `#counter`, no `<style`); `cargo fmt --all -- --check`.

## Phase 4 — Authoring docs, no platform footnote

Update the Rocdown reference `@roc` row and the pages / hybrid-sites guides:
helpers belong in `@roc`; live pages may import `pf.Sqlite` there for
handlers. Do not document the two-platform leak. Do not recommend inlining
into `@on`.[^rocdown-ref][^pages-guide]

**Exit:** those docs; `cargo test -p rocci-rocdown --test compile`;
`cargo fmt --all -- --check`.

## Later, not this plan

A dedicated diagnostic when `@render` or a component calls a `!` helper that
needs the webserver platform. Until then Roc’s snapshot compile error (now
also the preview Build error page) is the signal that splice asked for
handler IO.

[^islands-rs]: CDN splice evaluates `rocci_islands({})` as a basic-cli stdout program.
[^lower-rs]: `lower_islands` drops handlers; after Phase 2 it also omits snapshot-unreachable `@roc`.
[^service-rs]: Island service compiles live modules as a basic-webserver app.
[^dispatch-rs]: Generated `main.roc` uses basic-webserver; `pf.Sqlite` is connection-shaped.
[^lib-rs]: `BASIC_CLI_PLATFORM` pins snapshot eval to basic-cli 0.22.
[^hybrid-plan]: Hybrid islands: static CDN HTML plus a separate HTTP island service.
[^research]: Optimal one-`pf.Sqlite` story is upstream or a Rocci facade; neither is available. Reachability is the lowering substitute.
[^rocdown-ref]: `@roc` is the declaration for Roc values and helpers in a page.
[^page-rs]: `split_roc_body` / `roc_binding_names` already exist for `@roc` rest.
[^pure-render]: `@component` stays a pure render; handlers own IO.
[^counter]: Hybrid counter declares Sqlite helpers in `@roc` and calls them from `@on`.
[^compile-tests]: Existing hydrate test keeps `@roc` values used from island markup.
[^pages-guide]: Authoring guide for `@roc` values on pages.
