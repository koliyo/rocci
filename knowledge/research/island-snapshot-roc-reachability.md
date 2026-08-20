---
type: Research Report
title: Snapshot eval must not compile service-only @roc
description: "SQLite never runs on the CDN. Live pages compile twice (basic-cli snapshot HTML vs basic-webserver handlers). Roc typechecks unused @roc helpers, so one pf.Sqlite is two APIs. The authoring-optimal fix (one platform Sqlite, or Roc skipping unused bindings) is not available; Rocci ships a lowering reachability subset instead."
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
    last_modified: 2026-08-19
  - id: lower-rs
    resource: ../../crates/rocci-rocdown/src/lower.rs
    title: lower_islands filters snapshot-unreachable @roc
    author: process:git
    last_modified: 2026-08-20
  - id: page-rs
    resource: ../../crates/rocci-rocdown/src/page.rs
    title: split_roc_body, roc_rest_name, indent continuation
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
  - id: lib-rs
    resource: ../../crates/rocci-rocdown/src/lib.rs
    title: BASIC_CLI_PLATFORM constant
    author: process:git
    last_modified: 2026-08-20
  - id: compile-tests
    resource: ../../crates/rocci-rocdown/tests/compile.rs
    title: compile_islands omits unused Sqlite helpers
    author: process:git
    last_modified: 2026-08-20
  - id: counter
    resource: ../../examples/rocdown/counter/index.rocdown
    title: Hybrid counter live page with @roc helpers
    author: process:git
    last_modified: 2026-08-20
  - id: rocdown-ref
    resource: ../../docs/reference/rocdown.rocdown
    title: Public Rocdown @roc declaration
    author: process:git
    last_modified: 2026-08-20
  - id: hybrid-research
    resource: hybrid-rocdown-islands.md
    title: Hybrid Rocdown islands research
    author: process:cursor
    last_modified: 2026-08-20
  - id: efficient-pub
    resource: efficient-publishing.md
    title: Native apply vs island HTTP platforms
    author: process:cursor
    last_modified: 2026-08-20
  - id: pure-render
    resource: ../decisions/pure-render-components.md
    title: Keep Rocci render components pure
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: plan
    resource: ../plans/island-snapshot-roc-reachability.md
    title: Snapshot eval reachability implementation plan
    author: process:cursor
    last_modified: 2026-08-20
---

# Snapshot eval must not compile service-only `@roc`

## Authority

This record is exploratory reasoning for a shipped lowering change. It does
not approve a new architecture decision. The hybrid split (CDN HTML plus a
separate island HTTP process) is existing direction.[^hybrid-research][^plan]

SQLite does not run on the CDN. The failure is a **build-time** compile used
to produce the static HTML that later gets uploaded.[^islands-rs][^counter]

## How we got here

A live `.rocdown` page is one authoring file compiled twice, both times with
the platform module named `pf`:[^islands-rs][^service-rs][^dispatch-rs][^lib-rs]

| Compile | Platform | Job | Must include |
| --- | --- | --- | --- |
| CDN splice (`evaluate_page`) | `basic-cli` 0.22 | One-shot: `rocci_islands({})`, `Html.render` to stdout, exit | Components and values the snapshot HTML needs |
| Island service | `basic-webserver` 0.16 | HTTP `respond!` over `@on` | `@context` / `@init` / `@on` plus the same `@roc` and components |

After splice, the CDN file is baked HTML (`count: 0`). The browser later POSTs
`/actions/…` to the island process. That process is the only place SQLite
should exist.[^hybrid-research][^counter]

`lower_islands` already drops `@on` / `@init` / `@context`. It used to copy
**every** `@roc` statement into the snapshot module. Roc typechecks unused
helpers. `import pf.Sqlite` is then two APIs in one file: webserver Sqlite is
connection-shaped (`{ db, query, params }`); CLI Sqlite is path/bindings-shaped.
A helper written for `@on` is legal in the service compile and illegal in the
snapshot compile even when `rocci_islands` never calls it.[^dispatch-rs][^lib-rs]

That looked like “SQLite vs CDN.” It was service-only Roc fed into the HTML
snapshot compiler.

## Optimal solutions we cannot use

The authoring-optimal world is **one `pf.Sqlite`**. Authors write helpers in
`@roc` once. Both compiles see the same API. That is not available now.

**Unify basic-cli and basic-webserver Sqlite.** Rocci does not own those
packages. Their Sqlite surfaces already differ. Waiting on upstream, or
forking a platform, is outside this product.[^efficient-pub][^dispatch-rs]

**A Rocci Sqlite facade authors must import.** That would be a third API, not
`pf.Sqlite`. It hides the leak by adding a Rocci-owned module. The plan
rejects teaching a new import as the contract.[^plan][^rocdown-ref]

**Compile snapshot eval as `basic-webserver`.** Then `pf.Sqlite` would match
handlers. Snapshot eval is a one-shot stdout renderer with no HTTP listen,
matching site apply. Island handlers cannot be `basic-cli`: they are HTTP.
Both platforms are the right tool for their job. Swapping either one is the
wrong shape, not a missing flag.[^islands-rs][^efficient-pub]

**Have Roc skip typechecking unused bindings.** If unused `read_count!` were
not typechecked, copying all `@roc` into the snapshot module would be fine.
Roc typechecks unused values today. Rocci cannot wait on that compiler
change.[^compile-tests]

**Run `@init` / query Sqlite while splicing, or make `@component` effectful.**
That would make snapshot HTML “live.” It needs a real database at CDN build,
violates baked props, and contradicts pure render. Handlers own IO.
`@render` that calls a `!` helper should keep failing at Roc compile until a
later diagnostic names it.[^pure-render][^plan]

**New syntax (`@service`, handler-only `@roc`).** A third hiding place. Helpers
belong in `@roc` or a sibling `.roc` imported from `@roc`. Document-root text
that looks like Roc is Markdown.[^rocdown-ref][^page-rs]

## What we ship instead

`lower_islands` keeps an `@roc` rest statement when its binding name
(including `foo!`) or type-alias name appears in island text (`@render`,
document-root templates, instantiated component bodies), then drops imports
whose local name is unused. Unclassified statements stay (fail closed toward
today’s emit). `compile()` / the island service still emit all `@roc`
rest.[^lower-rs][^page-rs][^compile-tests]

That is a **lowering heuristic**, not a Roc parser: ident-boundary name match
plus indented lambda continuation in `split_roc_body`. It is good enough so
`examples/rocdown/counter` can keep `read_count!` in `@roc`. It is not the
optimal platform story.[^counter][^page-rs]

Residual (not this change): a dedicated diagnostic when splice asks for
handler IO. Until then Roc’s snapshot compile error is the signal.[^plan]

Implementation plan: [snapshot eval must not compile service-only `@roc`](../plans/island-snapshot-roc-reachability.md).

[^islands-rs]: CDN splice evaluates `rocci_islands({})` as a basic-cli stdout program.
[^lower-rs]: `filter_snapshot_roc` in `lower_islands` keeps reachable `@roc` rest and imports.
[^page-rs]: `split_roc_body` / `roc_rest_name` / indent continuation classify `@roc` without parsing Roc.
[^service-rs]: Island service compiles live modules as a basic-webserver app.
[^dispatch-rs]: Generated `main.roc` uses basic-webserver; `pf.Sqlite` is connection-shaped.
[^lib-rs]: `BASIC_CLI_PLATFORM` pins snapshot eval to basic-cli 0.22.
[^compile-tests]: Snapshot Roc omits unused `read_count!` and `import pf.Sqlite`; service compile keeps both.
[^counter]: Hybrid counter declares Sqlite helpers in `@roc` and calls them from `@on`.
[^rocdown-ref]: `@roc` is the declaration for Roc values and helpers in a page.
[^hybrid-research]: Hybrid islands: static CDN HTML plus a separate HTTP island service.
[^efficient-pub]: Native apply is basic-cli; island/app HTTP is basic-webserver; neither is wasm32.
[^pure-render]: `@component` stays a pure render; handlers own IO.
[^plan]: Plan out of bound: unify Sqlite, snapshot-as-webserver, effectful components, new syntax.
