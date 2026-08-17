---
type: Implementation Plan
title: Standalone Rocci OKF review and query application
description: Extract a portable OKF engine from the current Rocs implementation and build a Rocci application for agent-authored knowledge review, authenticated retrieval, and measured optional semantic search.
tags: [domain/okf, domain/rocci-okf, domain/rocci, concern/architecture, concern/review, concern/retrieval, concern/security]
status: draft
generated: { by: process:cursor, at: 2026-08-17T23:00:00Z }
stale_after: 2026-11-15
authority: exploratory
owners: [human:nils]
sources:
  - id: report
    resource: ../../reports/okf/ROCS_OKF_REPORT.rocdown
    title: rocs-okf research and product direction
    author: process:codex
    last_modified: 2026-08-17
  - id: current-okf
    resource: ../../crates/okf/src/lib.rs
    title: Portable OKF engine implementation
    author: process:git
    last_modified: 2026-08-17
  - id: current-cli
    resource: ../../crates/rocci-okf/src/main.rs
    title: rocci-okf CLI and review application
    author: process:git
    last_modified: 2026-08-17
  - id: static-boundary
    resource: ../decisions/static-okf-boundary.md
    title: Strict OKF Markdown and static Rocdown rendering boundary
    author: process:okf-migration
    last_modified: 2026-08-16
  - id: ecosystem
    resource: ../research/okf-tools-and-workflows.md
    title: State-of-the-art OKF tools and workflows
    author: process:codex
    last_modified: 2026-08-17
  - id: rocdown-boundary
    resource: ../decisions/consolidate-rocdown-product-boundary.md
    title: Approved consolidated Rocdown product direction
    author: process:codex
    last_modified: 2026-08-17
---

# Standalone Rocci OKF review and query application

## Direction and authority

This is an exploratory implementation plan, not an approved authentication
system, deployment, or semantic-search dependency. It records the requested
direction that OKF management should become a standalone Rocci application
while remaining open to third-party consumers. `rocci-okf` is the approved
application and Cargo namespace because this is a Rocci-built OKF product. The
portable engine beneath it is `okf`; the historical `rocs-okf` label is retired
with Rocs. The detailed architecture and tradeoffs are in the Rocdown
report.[^report][^rocdown-boundary]

Canonical records remain strict OKF Markdown. The portable engine and
application must not introduce Rocdown declarations, executable content, or a
required service into the bundle contract.[^static-boundary] Under the approved
product consolidation, Rocdown must not depend on OKF, and the target portable
engine depends on neither Rocdown nor Rocci.[^rocdown-boundary]

## Existing vertical slice

The current Rocs module already parses and validates bundles, derives lifecycle
and trust, checks keyed sources and git provenance, builds graphs and
heading-level chunks, filters and searches concepts, measures a fixed lexical
retrieval benchmark, renders a review queue, and emits site, catalog, search,
agent, and validation artifacts.[^current-okf]

The CLI already exposes local `run`, `check`, `inspect`, `search`, `benchmark`,
and `build` workflows, with JSON for machine inspection and retrieval.[^current-cli]

The gap is product separation: portable domain behavior, the Rocci profile,
Rocci-specific review text, Rocs HTML, and application orchestration are
coupled; review cannot yet record a revision-bound decision; and query has no
authenticated service or token contract.[^current-okf][^report]

During the Rocdown boundary refactor, the current knowledge commands and
rendered review site may remain behind a compatibility adapter. A temporary
application-level dependency on Rocdown's inert Markdown or presentation path
is approved only to preserve behavior while the engine is extracted; it
requires a tracking issue when introduced and must not leak into the canonical
format or portable domain types.[^rocdown-boundary]

## Target boundary

Create three layers:

1. The `okf` crate owns loss-preserving parsing, base conformance,
   curation findings, configurable profile policy, graph/backlinks, semantic
   diff inputs, chunks, filters, scorer interfaces, revision preconditions, and
   stable serializable types.
2. The Rocci OKF application is built with Rocci. It owns the review,
   query, explorer, and health UI; CLI and server orchestration; authenticated
   HTTP and MCP projections; review decision capture; and immutable snapshot
   publication.
3. Replaceable adapters own filesystem/git revisions, SQLite full text,
   optional vectors, identity and token storage, git-host integration, answer
   composition, and events.[^report]

The engine must not depend on Rocdown/Rocs, Rocci, Roc compilation, an HTTP
server, a theme, a git host, a vector database, or an LLM. Rocdown and the Rocci
OKF application may share domain-neutral Rocci view components only after two
working consumers establish a stable common contract; navigation and graph
resolution remain domain-owned.[^report][^rocdown-boundary]

## Workflow contract

Agents orient through indexes, search, concept reads, and backlinks; edit
Markdown directly; run separate conformance, curation/profile, diff, and
retrieval checks; then submit a focused candidate revision. No authoring UI or
write-capable MCP tool is required.[^ecosystem][^report]

The browser review queue orders deterministic work by validation, changed
post-verification content, stale high-impact records, unverified impact, source
drift, and ordinary drafts. A concept review combines semantic diff, rendered
candidate, sources, backlinks, retrieval impact, and an approve,
request-changes, or comment decision.[^report]

Every decision is bound to the reviewed revision. Approval must fail when the
candidate or required evidence changed, append rather than replace verification
history, and apply only an allowed lifecycle transition. Git-host review is an
optional adapter; standalone mode uses an append-only decision log and a narrow
compare-and-swap metadata commit.[^report]

Query returns authorized evidence before optional generated answers. Results
carry snapshot, chunk and concept identity, heading, score components,
lifecycle, authority, trust, staleness, text, and sources. Authorization filters
the candidate set before ranking, graph, backlinks, snippets, or provider
calls.[^ecosystem][^report]

## Delivery phases

### 0. Freeze compatibility

- Serialize golden fixtures for current parsing, validation, inspection,
  chunks, actions, and retrieval results.
- Classify each rule as base conformance, curation, or Rocci profile policy.
- Remove bundle-specific IDs and counts from domain behavior.
- The portable engine crate is `okf`; keep its publication boundary separate
  from Rocci and Rocdown.
- Keep the current knowledge command available through the Rocdown migration
  until the replacement application passes the same fixtures.

### 1. Extract the engine

- Move portable domain behavior out of Rocs.
- Add stable finding/action codes, machine capability description, and
  full-rebuild determinism tests.
- Keep the current knowledge command as a compatibility wrapper, regardless of
  whether its temporary spelling is `rocs knowledge` or `rocdown knowledge`.
- Prove a third-party binary can validate, inspect, graph, and search without a
  Rocdown, Rocs, or Rocci dependency.

### 2. Build the local application

- Add the standalone binary and Rocci shell.
- Port current governance, record, and review views without hardcoded Rocci
  records.
- Add explorer and health projections plus immutable snapshot revisions.
- Match current last-good local preview behavior.

### 3. Add review decisions

- Add metadata- and Markdown-block-aware diffs.
- Show citation/source drift, backlink impact, and affected retrieval cases.
- Record revision-bound approve, request-change, and comment events.
- Implement narrow verification/promotion commits and one optional git-host
  mapping.

### 4. Add authenticated query

- Add a transactional SQLite full-text snapshot.
- Version HTTP errors and resources, add pagination, ETags, OpenAPI, scoped and
  expiring API tokens, and audit events.
- Add query UI and a thin read-only MCP projection.
- Test that authorization is enforced before retrieval.

### 5. Add measured optional semantics

- Expand the versioned question set with negative, stale, lifecycle,
  authority, and permission cases.
- Add optional embedding adapters and deterministic hybrid fusion only if they
  improve measured quality within latency and cost budgets.
- Keep answer composition separate from raw retrieval and always expose its
  cited evidence and snapshot.

### 6. Operational maturity

- Add multi-bundle registry, one-home cross-bundle references, incremental
  freshness jobs, recovery, token rotation, rate limits, observability, and
  explicit filtered public export.
- Verify full rebuilds reproduce every derived graph, chunk, and lexical result.

## First acceptance gate

The extraction is successful when the existing Rocci bundle produces the same
portable normalized concepts, diagnostics by classified layer, graph, chunk
IDs, filters, lexical benchmark, and review actions; the temporary
Rocs/Rocdown compatibility path uses the new engine; and a separate minimal
consumer does the same without linking Rocdown, Rocs, or Rocci.[^current-okf][^report][^rocdown-boundary]

The first application release is successful when the chosen application
command matches current local browse/review capability, replaces hardcoded
queue content with profile data, identifies the served snapshot, preserves the
last good revision after a failed edit, and requires no authoring UI.[^report]

## Deferred decisions

The engine crate is `okf` and the application name is `rocci-okf`. Human review
is still required before choosing the direct-commit versus pull-request
default, profile syntax, local identity model, token verifier, semantic-diff
minimum, answer-provider ownership, or public presentation name. Semantic
search remains optional until measured against the improved full-text
baseline.[^report][^rocdown-boundary]

[^report]: Detailed current-state audit, ecosystem comparison, product architecture, review and query contracts, security model, delivery phases, acceptance scenarios, and open decisions.
[^current-okf]: Current implemented parser, profiles, diagnostics, graph, filters, chunks, review HTML, build artifacts, lexical search, and benchmark behavior.
[^current-cli]: Current implemented local knowledge command and JSON output surface.
[^static-boundary]: Approved canonical strict-Markdown and inert static-rendering boundary.
[^ecosystem]: Emerging workflow evidence for deterministic agent interfaces, conformance/curation separation, multiple views, guarded production, revision history, authenticated MCP, and evaluation.
[^rocdown-boundary]: Approved one-way dependency rules, frozen `okf` engine name, temporary presentation-adapter allowance, and separate-decision requirement for any future Rocdown-backed canonical OKF storage.
