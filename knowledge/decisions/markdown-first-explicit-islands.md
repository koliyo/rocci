---
type: Decision
title: Keep Rocdown Markdown-first with explicit executable islands
description: Rocdown leaves ordinary prose in Markdown and changes language mode only at visible document-root declarations, with client behavior opt-in.
tags: [domain/rocdown, concern/syntax, concern/security, concern/rendering]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
authority: normative
owners: [human:nils]
sources:
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-15
  - id: rocdown-readme
    resource: ../../crates/rocci-rocdown/README.md
    title: Implemented Rocdown language reference
    author: process:git
    last_modified: 2026-08-16
  - id: parser
    resource: ../../crates/rocci-rocdown/src/parse.rs
    title: Rocdown parser
    author: process:git
    last_modified: 2026-08-16
---

# Keep Rocdown Markdown-first with explicit executable islands

## Context

An MDX-like surface could mix arbitrary expressions and component syntax into prose, but it would make literal content, language transitions, security review, and graceful static rendering harder to identify.[^format-report]

## Decision

Ordinary Rocdown content is Markdown. A language transition occurs only at a reserved, document-root line-start declaration or an explicitly recognized root HTML island; inline `@`, email addresses, fenced examples, lists, and quotations do not switch modes.[^rocdown-readme][^parser]

Roc and Rocci regions are visible and block-oriented. Fenced code is never executable, raw inline HTML is disabled by default, and client JavaScript will be emitted only for an explicitly referenced future island.[^format-report][^rocdown-readme]

## Consequences

Documents remain readable in Markdown-oriented tools, static pages require no client runtime, and reviewers can locate executable or browser-owned regions directly in source. Inline dynamic prose sometimes requires a small component or block render instead of a terse embedded expression.[^format-report]

## Current disposition

The Markdown/declaration boundary is implemented. `@island` remains only a reserved design direction and must not be presented as shipped.[^rocdown-readme]

[^format-report]: Original alternatives, rationale, static default, and explicit-island design.
[^rocdown-readme]: Current declaration rules, Markdown profile, raw-HTML policy, and deferred features.
[^parser]: Executable declaration recognition in the current parser.
