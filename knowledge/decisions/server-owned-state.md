---
type: Decision
title: Keep durable application state server-owned
description: Rocci applications read canonical backend state, render coherent HTML boundaries, and use the browser for transient interaction rather than duplicated domain authority.
tags: [domain/runtime, domain/rocci, integration/datastar, concern/rendering, concern/security]
status: stable
generated: { by: process:okf-migration, at: 2026-08-31T08:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
authority: normative
owners: [human:nils]
sources:
  - id: rendering-doc
    resource: ../../docs/concepts/rendering-model.rocdown
    title: Published rendering model
    author: human:nils
    last_modified: 2026-08-16
    last_modified: 2026-08-15
---

# Keep durable application state server-owned

## Context

Rocci uses HTML and Datastar server-sent events as the update boundary. Mirroring durable state in a client model or retained server-side DOM would add synchronization, replay, and recovery protocols before the examples demonstrate a need for them.

Some interactions, especially high-rate keyboard, canvas, drag, media, or third-party-widget behavior, still require transient browser ownership.

## Decision

The backend remains authoritative for durable application state. An action validates and writes through the application boundary, reads the canonical model again, renders the largest coherent stable-ID region, and emits HTML for the browser to morph.[^rendering-doc]

Browser state is reserved for explicitly ephemeral behavior. A future client island may own a private interaction surface, but it must communicate intent to the server and reconcile with authoritative server output rather than becoming a second domain store.

## Consequences

The normal programming model favors direct request patches or a single versioned update stream, stable DOM identity, and full-snapshot recovery. Narrow fragments and client signals remain targeted optimizations rather than the default state model.

The approach trades some render and transfer work for a simpler correctness model. Performance changes require measurement of render time, compressed events, morph duration, and update frequency before introducing retained diff state.

## Current disposition

The server-owned rendering direction is used by current examples and documentation. Typed client islands are proposed but not implemented, so this record does not claim that boundary is yet enforced by a dedicated island runtime.

[^rendering-doc]: Published contract that Datastar transports intent and server-rendered HTML.
