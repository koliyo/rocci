---
type: Decision
title: Keep Rocci render components pure
description: Rocci component declarations lower to ordinary Roc functions from explicit values to Html and do not own persistence, request lifecycle, or client state.
tags: [domain/rocci, integration/roc, concern/rendering]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
authority: normative
owners: [human:nils]
sources:
  - id: template-lowering
    resource: ../../crates/rocci-template/src/lower.rs
    title: Rocci template lowering
    author: process:git
    last_modified: 2026-08-16
  - id: rendering-doc
    resource: ../../docs/concepts/rendering-model.rocdown
    title: Published rendering model
    author: human:nils
    last_modified: 2026-08-16
  - id: runtime-report
    resource: ../../archive/reports/ROC_DATASTAR_COMPONENT_FILETYPE_REPORT.md
    title: Roc and Datastar component architecture report
    author: human:nils
    last_modified: 2026-08-15
---

# Keep Rocci render components pure

## Context

A colocated component file can be mistaken for a runtime component instance that owns state, effects, or lifecycle. Rocci instead compiles markup into ordinary Roc functions and keeps application orchestration separate.[^runtime-report]

## Decision

An `@component` is a render abstraction: explicit parameters and body content become a Roc function returning `Html`. Component calls become ordinary function calls with Roc records and, for paired tags, an explicit body value.[^template-lowering][^rendering-doc]

Persistence, request routing, decoding, effects, sessions, and durable domain state stay outside template lowering. The same pure renderer can be called for a full page or for a server-produced fragment.[^runtime-report]

## Consequences

Components remain composable Roc code with no hidden instance lifecycle. State architecture can evolve independently, and testing can compare rendered values without starting the host. Authors must pass required state explicitly and use application handlers for mutation.[^runtime-report]

Browser components are a distinct future concept; adding an island must not silently change existing `@component` semantics.

## Current disposition

Implemented in template lowering and reflected in published documentation. Human-reviewed and promoted to `stable` on 2026-08-16.

[^template-lowering]: Current lowering of component declarations and calls to Roc functions.
[^rendering-doc]: User-facing statement that authored components are Roc functions returning Html.
[^runtime-report]: Architectural separation between the pure render layer and application/runtime concerns.
