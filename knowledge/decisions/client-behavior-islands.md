---
type: Decision
title: Use explicit islands for browser-owned behavior
description: Proposed Rocci client islands would isolate keyboard, canvas, drag, media, and third-party behavior without changing server-rendered component semantics.
tags: [domain/rocci, domain/runtime, integration/datastar, concern/rendering, concern/security]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-11-14
authority: exploratory
owners: [human:nils]
sources:
  - id: rocket-now
    resource: ../research/rocci/datastar-rocket.md
    title: Datastar Rocket and Rocci-native islands (current)
    author: process:cursor
    last_modified: 2026-08-28
  - id: blockers
    resource: ../research/rocci/client-behavior-islands.md
    title: "@island design blockers"
    author: process:cursor
    last_modified: 2026-08-28
---

# Use explicit islands for browser-owned behavior

## Context

Rocci components are pure server renderers, while a small class of interfaces needs low-latency browser ownership: keyboard input, canvas or SVG manipulation, drag and drop, media, observers, editors, maps, and third-party widgets.

Turning every component into a browser object would duplicate the server-owned state model and introduce lifecycle, serialization, reconciliation, and asset obligations for ordinary HTML rendering.

## Options

The reports considered keeping all behavior in Datastar expressions, making ordinary components implicitly client-capable, reproducing a broad Rocket-style runtime, using an explicit native island, or allowing an external client module for complex behavior.

## Proposed decision

If client behavior is added, introduce an explicit island construct distinct from `@component`. Start behavior-only and light-DOM-first: the server renders the meaningful host and children, while a generated or referenced custom-element module attaches narrowly scoped browser behavior.

Durable state remains server-owned. Islands own only declared ephemeral or private interaction state, emit intent back to the server, handle cleanup, and reconcile with authoritative server patches. Large client implementations stay in explicit `*.client.js` modules rather than opaque multiline literals.

Do not redistribute Datastar Rocket as a built-in implementation without separate licensing permission. A future provider integration, if any, remains bring-your-own and optional.

## Consequences

Existing render components keep their current semantics and static pages continue to emit no client JavaScript by default. The compiler and runtime would need typed prop serialization, module deduplication, lifecycle and morph rules, CSP-aware asset delivery, diagnostics, and editor support.

The explicit boundary costs more syntax than implicit hydration but makes browser code, ownership, and delivery dependencies reviewable.

## Current disposition

Proposed and unimplemented as of 2026-08-16. The reports support the direction, but neither the syntax nor the runtime contract appears in the approved decision register. Human review may accept, revise, or reject this record without changing the shipped language. Current Rocket restatement: [`datastar-rocket`](/research/rocci/datastar-rocket.md). Open questions and Stage 0 → Stage 1 sequence: [`@island` design](/research/rocci/client-behavior-islands.md).[^rocket-now][^blockers]

[^rocket-now]: 2026-08-28 restatement of the archive against shipped Rocci.
[^blockers]: Plan-readiness questions; Stage 0 spike still required.
