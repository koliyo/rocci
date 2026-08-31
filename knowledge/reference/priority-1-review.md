---
type: Reference
title: Priority-1 knowledge review checklist
description: This checklist defines the evidence-based human gate for verifying and stabilizing the ten priority-1 Rocci knowledge records.
tags: [domain/rocci, integration/okf, concern/validation, audience/maintainer]
status: draft
generated: { by: process:okf-phase-6, at: 2026-08-31T08:00:00Z }
stale_after: 2026-09-15
authority: descriptive
owners: [human:nils]
sources: []
    last_modified: 2026-08-16
---

# Priority-1 knowledge review checklist

## Review gate

A reviewer must compare each current-behavior claim with code, tests, current crate documentation, or published docs; confirm that report-derived rationale is not presented as shipped behavior; check authority and lifecycle; and verify that keyed citations support the claims they follow.

Substantive corrections update `generated.at` and leave the record `draft`. An accepted current revision receives a real `verified` event with the reviewer's actor ID and timestamp. Only records ready for consumption at their declared authority become `stable`.

## Priority-1 queue

| Record | Review focus | State |
| --- | --- | --- |
| [System overview](/architecture/system-overview.md) | Workspace and product boundaries | Verified by `human:nils`; stable |
| [Implementation status](/status/implementation.md) | Snapshot accuracy; shipped, approved, proposed separation | Revised through Phase 6 consolidation and retrieval measurement; draft; re-verification required |
| [Known limitations](/status/known-limitations.md) | Current absences; ordinary-site versus OKF search boundary | Revised through the Phase 6 public-status correction; draft; re-verification required |
| [Rocdown format](/architecture/rocdown-format.md) | Parser/README precedence over the original report; root HTML template islands | Revised for document-root HTML template island boundary; draft; re-verification required |
| [Rocdown documentation generator](/architecture/rocdown-documentation-compiler.md) | Rocdown generator plus isolated OKF preview/retrieval path | Revised for the Phase 6 retrieval benchmark; draft; re-verification required |
| [Theming](/architecture/theming.md) | Two current surfaces versus DTCG research-only boundary | Revised for amended Phase 4 contract; draft; re-verification required |
| [Pure render components](/decisions/pure-render-components.md) | Implemented render semantics versus application architecture | Verified by `human:nils`; stable |
| [Server-owned state](/decisions/server-owned-state.md) | Current direction versus optional browser state | Verified by `human:nils`; stable |
| [Markdown-first explicit islands](/decisions/markdown-first-explicit-islands.md) | Implemented syntax boundary versus unimplemented `@island` | Verified by `human:nils`; stable |
| [Rust catalog and Rocci shell](/decisions/rust-catalog-rocci-shell.md) | Implemented ownership boundary and remaining splice path | Verified by `human:nils`; stable |

## Contradictions already surfaced

- The published project-status page says Rocs watch mode and aliases are pending; current code, reference docs, and the active Rocs plan show both are implemented.
- The original Rocdown format report lists SSG and LSP support as absent; those statements are historical, while the crate README and current Rocs/LSP code describe shipped behavior.
- The theming report proposes packages, adapters, and presentation renderers beyond the current CSS resolver and first-party Rocs shell.
- Client-island reports recommend a direction, but `@island` is neither implemented nor approved as stable syntax.

## Mechanical checks

Run `rocci-okf check knowledge --profile base`. `OKF4004` reports stale records, `OKF4005` reports verification older than generation, `OKF4006` reports a tracked source committed after human verification, `OKF4007` reports an untracked local source with no git provenance, and `OKF4008` reports tracked evidence with uncommitted changes that cannot be matched to its verification.

## Current review state

All ten priority-1 records were verified by `human:nils` at `2026-08-16T18:14:13Z` and promoted to `stable`. Phase 4 substantively corrected implementation status and theming; Phase 5 updated the Rocs compiler and known-limitations records; Phase 6 further updated implementation status, known limitations, and the Rocs compiler for consolidation and measured retrieval; subsequent work updated the Rocdown format record for document-root HTML template islands. Historical verification events are retained, but those five revised records are now `draft` and must be reviewed again. The other five priority-1 records remain stable. New design, publication, and consolidation records, the exploratory client-behavior-island decision, this checklist, and the static-OKF-boundary seed record are also `draft`.

