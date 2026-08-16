---
type: Decision
title: Use strict OKF Markdown with a static Rocdown rendering profile
description: Canonical knowledge records use OKF YAML and Markdown while reusing Rocdown parsing and Rocs rendering without executable declarations.
tags: [domain/rocs, integration/okf, concern/syntax, concern/security]
status: draft
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
authority: normative
owners: [human:nils]
sources:
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
---

# Use strict OKF Markdown with a static Rocdown rendering profile

## Context

Canonical `.rocdown` records would improve language dogfooding but would not themselves conform to OKF v0.2. A separate Markdown implementation would preserve portability but duplicate Rocdown parsing and Rocs rendering infrastructure.[^okf-plan]

## Decision

Canonical records live in `knowledge/**/*.md` with YAML frontmatter. Their bodies use a static Markdown-only Rocdown path with footnotes enabled and executable Rocdown declarations forbidden.[^okf-plan]

## Consequences

The checked-in bundle is directly portable OKF, unknown metadata remains recoverable, and knowledge builds do not execute Roc or Rocci content. Product documentation continues to use `.rocdown` and `@page` independently.[^okf-plan]

## Current disposition

Approved in Phase 0 and represented by the Phase 1 parser and vertical slice. The record remains `draft` until the Phase 3 human-review workflow verifies migrated concepts.

[^okf-plan]: Approved compatibility-boundary and Phase 0 decision register.
