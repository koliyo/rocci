---
type: Architecture
title: Rocdown format boundary
description: Rocdown is Markdown-first content with explicit document-root Roc and Rocci regions, static defaults, and a separate static knowledge-body profile.
tags: [domain/rocdown, concern/syntax, concern/rendering, concern/security]
status: stable
generated: { by: process:okf-migration, at: 2026-08-16T00:00:00Z }
verified:
  - { by: human:nils, at: 2026-08-16T18:14:13Z }
stale_after: 2027-02-12
authority: descriptive
owners: [human:nils]
sources:
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
  - id: format-report
    resource: ../../archive/reports/ROCDOWN_FORMAT_REPORT.md
    title: Original Rocdown format investigation
    author: human:nils
    last_modified: 2026-08-15
---

# Rocdown format boundary

## Current contract

A `.rocdown` document interleaves ordinary Markdown with reserved declarations recognized only at a document-root line boundary. Reserved declarations include page metadata, Roc blocks, rendered Roc expressions, Rocci components and fixtures, scoped CSS, server lifecycle forms, structural template forms, and document-root HTML islands.[^rocdown-readme][^parser]

Markdown supports CommonMark plus tables, strikethrough, task lists, extended autolinks, heading IDs, and Rocdown page-link forms. Raw inline HTML is disabled by default. Ordinary `.rocdown` compilation keeps footnotes disabled; the body-only OKF adapter enables them separately.[^rocdown-readme]

The standalone compiler lowers documents to ordinary Roc exports for metadata, content, and the page shell. It does not type-check Roc or run the server itself.[^rocdown-readme]

## Language boundary

Markdown owns prose. Roc owns data and computation inside explicit regions. Rocci owns server-rendered templates, and server handlers remain visible declarations. Fenced code is always displayed rather than executed.[^format-report]

This is the implemented form of the [Markdown-first explicit-islands decision](/decisions/markdown-first-explicit-islands.md). The stricter [OKF boundary](/decisions/static-okf-boundary.md) reuses only the Markdown AST and renderer, not Rocdown declarations.

## Not yet implemented

`@island`, a formatter, content collections, near-miss directive warnings, and several proposed Markdown extensions are not part of ordinary Rocdown today. Multi-page static generation exists in Rocs rather than in the single-file Rocdown compiler.[^rocdown-readme]

## Evidence policy

The crate README and parser describe shipped behavior. The 2026-08-15 report supplies design rationale and future proposals only where explicitly labeled; it is not allowed to override the current implementation.[^format-report]

[^rocdown-readme]: Current file shape, declarations, Markdown profile, lowering, and implemented/deferred list.
[^parser]: Executable recognition and parsing behavior in code.
[^format-report]: Original design rationale, with its own warning that current crate documentation has precedence.
