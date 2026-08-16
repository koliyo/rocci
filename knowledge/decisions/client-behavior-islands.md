---
type: Decision
title: Use explicit islands for browser-owned behavior
description: Proposed Rocci client islands would isolate keyboard, canvas, drag, media, and third-party behavior without changing server-rendered component semantics.
tags: [domain/rocci, domain/runtime, integration/datastar, concern/rendering, concern/security]
status: draft
generated: { by: process:okf-migration, at: 2026-08-16T18:00:00Z }
stale_after: 2026-11-14
authority: exploratory
owners: [human:nils]
sources:
  - id: rocket-report
    resource: ../../archive/reports/DATASTAR_ROCKET_IN_ROCCI_REPORT.md
    title: Rocket-style client components inside Rocci
    author: human:nils
    last_modified: 2026-08-14
  - id: snake-study
    resource: ../../archive/reports/SNAKE_DATASTAR_ARCHITECTURE_REPORT.md
    title: Snake input and Datastar architecture
    author: human:nils
    last_modified: 2026-08-15
  - id: runtime-report
    resource: ../../archive/reports/ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md
    title: Roc and Datastar component architecture report
    author: human:nils
    last_modified: 2026-08-15
---

# Use explicit islands for browser-owned behavior

## Context

Rocci components are pure server renderers, while a small class of interfaces needs low-latency browser ownership: keyboard input, canvas or SVG manipulation, drag and drop, media, observers, editors, maps, and third-party widgets.[^rocket-report][^snake-study]

Turning every component into a browser object would duplicate the server-owned state model and introduce lifecycle, serialization, reconciliation, and asset obligations for ordinary HTML rendering.[^runtime-report][^rocket-report]

## Options

The reports considered keeping all behavior in Datastar expressions, making ordinary components implicitly client-capable, reproducing a broad Rocket-style runtime, using an explicit native island, or allowing an external client module for complex behavior.[^rocket-report]

## Proposed decision

If client behavior is added, introduce an explicit island construct distinct from `@component`. Start behavior-only and light-DOM-first: the server renders the meaningful host and children, while a generated or referenced custom-element module attaches narrowly scoped browser behavior.[^rocket-report]

Durable state remains server-owned. Islands own only declared ephemeral or private interaction state, emit intent back to the server, handle cleanup, and reconcile with authoritative server patches. Large client implementations stay in explicit `*.client.js` modules rather than opaque multiline literals.[^rocket-report][^snake-study]

Do not redistribute Datastar Rocket as a built-in implementation without separate licensing permission. A future provider integration, if any, remains bring-your-own and optional.[^rocket-report]

## Consequences

Existing render components keep their current semantics and static pages continue to emit no client JavaScript by default. The compiler and runtime would need typed prop serialization, module deduplication, lifecycle and morph rules, CSP-aware asset delivery, diagnostics, and editor support.[^rocket-report]

The explicit boundary costs more syntax than implicit hydration but makes browser code, ownership, and delivery dependencies reviewable.

## Current disposition

Proposed and unimplemented as of 2026-08-16. The reports support the direction, but neither the syntax nor the runtime contract appears in the approved decision register. Human review may accept, revise, or reject this record without changing the shipped language.

[^rocket-report]: Proposed island architecture, lifecycle, artifact model, alternatives, and licensing constraint from the untracked 2026-08-14 investigation.
[^snake-study]: Concrete keyboard-input case and the boundary between authoritative server state and ephemeral browser behavior.
[^runtime-report]: Current pure-render and server-owned application architecture that an island must preserve.
