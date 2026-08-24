---
type: Research Report
title: State-of-the-art OKF tools and workflows
description: Emerging OKF tools converge on portable Markdown, agent-native deterministic interfaces, separate conformance and curation, evidence review, rebuildable retrieval, and authenticated machine access.
tags: [domain/okf, domain/rocs-okf, concern/agents, concern/review, concern/retrieval, concern/security]
status: draft
generated: { by: process:codex, at: 2026-08-17T13:24:13Z }
stale_after: 2026-11-15
authority: exploratory
owners: [human:nils]
sources:
  - id: report
    resource: ../../../archive/reports/okf/ROCS_OKF_REPORT.rocdown
    title: rocs-okf research and product direction
    author: process:codex
    last_modified: 2026-08-17
  - id: okf-spec
    resource: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md
    title: Open Knowledge Format v0.2 specification
    author: organization:google-cloud
  - id: okf-reference
    resource: https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/README.md
    title: Google Cloud OKF reference agent and visualizer
    author: organization:google-cloud
  - id: okfcli
    resource: https://github.com/okfcli/okf
    title: Agent-native OKF Go CLI
    author: organization:okfcli
  - id: okf-gem
    resource: https://okfgem.com/docs/
    title: okf-gem toolkit documentation
    author: human:rodrigo-serradura
  - id: openknowledge
    resource: https://github.com/openknowledge-sh/openknowledge
    title: Open Knowledge CLI
    author: organization:openknowledge-sh
  - id: aws-data-wiki
    resource: https://github.com/aws-samples/sample-okf-llm-wiki/blob/main/docs/ARCHITECTURE.md
    title: AWS Data Wiki architecture
    author: organization:aws
  - id: github-review
    resource: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets
    title: GitHub ruleset review controls
    author: organization:github
---

# State-of-the-art OKF tools and workflows

## Scope

OKF is an emerging ecosystem rather than a settled product category. The
detailed comparison, product implications, risks, acceptance scenarios, and
source list live in the accompanying Rocdown report.[^report]

## Portable format and bounded production

OKF v0.2 keeps conformance deliberately small while making provenance, trust,
lifecycle, progressive indexes, ordinary links, and keyed per-claim citations
portable. Consumers must tolerate unknown types, extension fields, missing
optional metadata, broken links, and absent indexes; an organizational policy
may flag those conditions without redefining base conformance.[^okf-spec]

The official reference implementation demonstrates two bounded production
passes: structured metadata first, then optional source enrichment constrained
by explicit seeds, allowed hosts, and a maximum page count. Its self-contained
viewer proves that graph, backlinks, search, filters, and rendered Markdown can
remain derived from the bundle.[^okf-reference]

## Agent-native operations

The independent Go CLI treats machine use as a public contract through a
capability/schema command, JSON-first output, structured error envelopes,
stable rule IDs, explicit exit codes, SARIF, and commands for validation,
linting, indexes, search, backlinks, and graphs.[^okfcli]

`okf-gem` separates legal conformance from advisory curation and exposes graph,
files, catalog, tag, type, statistics, search, backlinks, registry, and hub
views over one bundle model. Its local viewer shows that a graph is useful for
orientation but should coexist with queue, search, and file projections.[^okf-gem]

Open Knowledge extends the same pattern into agent setup, validation, search,
viewer, MCP, registry, HTML export, publication filtering, and runtime
automation. Its breadth supports composable workflows but reinforces the need
to keep a minimal engine distinct from an application and deployment
surface.[^openknowledge]

## Review, serving, and evaluation

The AWS Data Wiki sample keeps versioned Markdown authoritative, treats vector
search as rebuildable, constrains agent writes with middleware, reviews small
link-related concept clusters against live evidence, incrementally re-indexes
changed content, exposes authenticated semantic retrieval over MCP, and
benchmarks answers with the bundle as the solver's only knowledge source.[^aws-data-wiki]

Git-host pull-request workflows already supply diff comments, ownership,
required checks, approve/request-change decisions, and dismissal of approvals
when later commits change the reviewed diff. An OKF application should reuse
those controls where available and add knowledge-specific evidence, lifecycle,
backlink, source-drift, and retrieval-impact views.[^github-review]

## Synthesis

The strongest workflow is a closed evidence loop: agents make small sourced
Markdown changes; deterministic conformance, curation, profile, graph, and
retrieval checks judge the candidate; humans review the semantic diff and
evidence against an exact revision; an approved revision becomes an immutable
query snapshot; and source drift, staleness, and retrieval measurements create
the next maintenance work.[^report]

Query should first return authorized, ranked evidence with concept identity,
heading, lifecycle, trust, staleness, sources, and snapshot revision. Full-text
search should precede optional embeddings, and generative answer composition
should remain an adapter whose retrieval inputs can be evaluated
independently.[^report]

[^report]: Detailed state-of-the-art comparison, Rocci baseline, proposed rocs-okf boundary, review/query contracts, security constraints, phased delivery, and acceptance scenarios.
[^okf-spec]: OKF v0.2 conformance, extension, source attribution, trust, lifecycle, index, link, and versioning rules.
[^okf-reference]: Official proof-of-concept production passes and self-contained bundle visualizer.
[^okfcli]: Machine-discoverable command schema, structured output and errors, stable diagnostics, SARIF, and agent-oriented workflow.
[^okf-gem]: Conformance/curation separation and the multi-view local graph, catalog, search, and registry application.
[^openknowledge]: Broad agent, validation, retrieval, MCP, export, registry, and automation workflow surface.
[^aws-data-wiki]: Guarded multi-agent production, evidence review, versioned source, derived vectors, authenticated MCP, incremental freshness, and evaluation design.
[^github-review]: Required-review, stale-approval, code-owner, and merge-gate behavior available from a git host.
