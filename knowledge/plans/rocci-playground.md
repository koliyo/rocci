---
type: Implementation Plan
title: Client-side Rocci and Rocdown playground
description: Shared playground with a Rust/WASM compiler, TypeScript browser shell, and desktop plus rocci.dev hosts. Lowers to Roc and AST; HTML preview needs a future Roc WASM compiler.
tags: [domain/rocci, domain/rocdown, concern/tooling, concern/publication, concern/ux]
status: draft
generated: { by: process:cursor, at: 2026-08-23T00:00:00Z }
stale_after: 2026-11-23
authority: exploratory
owners: [human:nils]
sources:
  - id: detailed-plan
    resource: ../../archive/reports/ROCCI_PLAYGROUND_IMPLEMENTATION_PLAN.md
    title: Detailed playground implementation plan
    author: process:codex
    last_modified: 2026-08-21
  - id: product-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved Rocdown product-boundary decision
    author: process:codex
    last_modified: 2026-08-17
  - id: playground-crate
    resource: ../../crates/rocci-playground/README.md
    title: Playground compiler crate
    author: process:git
    last_modified: 2026-08-22
  - id: site-playground
    resource: ../../site/playground/index.rocdown
    title: Public lower-only playground page
    author: process:git
    last_modified: 2026-08-22
---

# Client-side Rocci and Rocdown playground

## Goal

One reusable playground: `rocci-playground` compiles `.rocci` and `.rocdown` to
generated Roc, formatted AST, diagnostics, and highlight spans. `playground/`
is the TypeScript shell. Desktop can use `--mode local` for native HTML
snapshots; the public site stays WASM-only and does not run `roc
build`.[^detailed-plan][^site-playground]

This record is the knowledge-facing plan. Phase bounds and exit checks stay in
the archived detailed plan. The compiler crate lives in
`crates/rocci-playground`.[^detailed-plan][^playground-crate]

## Out of bound

- Running or type-checking generated Roc in the browser.
- A general IDE, LSP client, formatter, or debugger.
- Replacing `rocci-template`, `rocci-rocdown`, or `rocci-highlight` with a
  JavaScript parser.
- A Roc-in-WASM HTML renderer (blocked until Roc ships that target).

## Constraints that do not move

- Base Rocci tooling must not import Rocdown AST types. A Rocdown-owned host
  composes the shared compiler.[^product-boundary]
- Parser and lowerer stay the authority; WASM must match native library
  entry points for Roc and AST output.[^detailed-plan]
- Public playground copy must say HTML preview is unavailable without a Roc
  WASM compiler.[^site-playground]

## Phases

Phase 0 is complete (WASM feasibility, `comrak` default-features off, highlight
sidecar decision). Phases 1–18 are the delivery sequence in the detailed plan:
facade, WASM adapter, highlight bridge, worker protocol, editor shell,
diagnostics, output selector, asset build, loopback host, CLI delivery, parity
suite, Rocdown component, CSP, site examples, virtual workspace,
accessibility, performance, and release documentation. Phase 19 remains
blocked on Roc-in-WASM HTML.[^detailed-plan]

### Current bound

Desktop `--mode local` HTML snapshots exist. Continue remaining phases from the
detailed plan; do not start Phase 19.

### Exit

The detailed plan's definition of done: local `rocci playground` /
`rocdown playground`, WASM site host, honest unavailable-HTML state, and
named size/latency budgets.[^detailed-plan]

[^detailed-plan]: Shared three-host playground, phase list, and first-delivery non-goals.
[^product-boundary]: Crate and CLI ownership after the Rocdown split.
[^site-playground]: Public page states lower-only; no HTML without Roc WASM.
[^playground-crate]: Compiler package and parity tests.
