---
type: Reference
title: OKF consolidation disposition
description: Phase 6 preserves seven dated reports in an archive, retains two active root plans, records that no concepts are superseded, and fixes a lexical retrieval baseline before any larger retrieval stack.
tags: [domain/rocci, integration/okf, concern/validation, audience/maintainer]
status: draft
generated: { by: process:okf-phase-6, at: 2026-08-16T20:30:00Z }
authority: descriptive
owners: [human:nils]
sources:
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
  - id: archive
    resource: ../../archive/reports/README.md
    title: Archived Rocci reports
    author: process:okf-phase-6
    last_modified: 2026-08-16
  - id: benchmark
    resource: ../retrieval-benchmark.toml
    title: Fixed retrieval benchmark
    author: process:okf-phase-6
    last_modified: 2026-08-16
  - id: publication
    resource: ../decisions/local-knowledge-publication.md
    title: Local knowledge publication decision
    author: process:okf-phase-5
    last_modified: 2026-08-16
---

# OKF consolidation disposition

## Lifecycle audit

No canonical concept is superseded. The current architecture, status, decision,
design, and reference records have distinct subjects or lifecycle roles, so
Phase 6 does not mark any record `deprecated`. Future replacement records must
name the superseded concept and provide a replacement link or explanation.[^okf-plan]

## Root-report disposition

Seven dated reports moved to `archive/reports/` with their prose unchanged and
only relative links repaired. `ROC_TEMPLATE.md`
and `ROCDOWN_DOCUMENTATION_GENERATOR_IMPLEMENTATION_PLAN.md` remain at the
repository root because they are still active detailed plans. No report was
deleted, and the migration matrix retains original basenames as stable source
identifiers.[^archive]

## Public-documentation audit

The public status page remains a concise audience-facing overview because the
canonical knowledge site is intentionally local and has no stable public URL.
Its stale claim that Rocs aliases and watch mode were pending was removed.
No other public status or decision prose was replaced by local-only links.[^publication]

## Retrieval baseline

`retrieval-benchmark.toml` fixes seven questions covering architecture, current
status, known gaps, language behavior, theming, an implemented decision, and an
exploratory decision. Each question names relevant concepts and expected
lifecycle and authority metadata. `rocs knowledge benchmark` measures top-five
hit rate and mean reciprocal rank, fails below the checked-in threshold, and is
run in CI before embeddings or a database service are considered.[^benchmark]

## Review status

This consolidation record and the measured queries are generated evidence and
remain `draft` until a human reviews their dispositions and relevance labels.

[^okf-plan]: Phase 6 deliverables and the rule that report movement is a separately reviewed change.
[^archive]: File-by-file report disposition and authority notes.
[^publication]: Current local-only publication boundary.
[^benchmark]: Versioned lexical retrieval questions, relevance labels, and lifecycle expectations.
