---
type: Decision
title: Keep generated knowledge publication local and repository-visible
description: Rocci generates a browsable knowledge site and machine indexes for local and CI use, but does not deploy them publicly or publish a verbatim bundle archive during the bootstrap.
tags: [domain/rocs, integration/okf, concern/security, audience/maintainer]
status: draft
generated: { by: process:okf-phase-5, at: 2026-08-16T19:30:24Z }
authority: normative
owners: [human:nils]
sources:
  - id: okf-plan
    resource: ../../OKF_PLAN.md
    title: Open Knowledge Format plan for Rocci
    author: human:nils
    last_modified: 2026-08-16
  - id: workflow
    resource: ../../.github/workflows/knowledge.yml
    title: Knowledge validation workflow
    author: process:okf-phase-5
    last_modified: 2026-08-16
  - id: builder
    resource: ../../crates/rocs/src/okf.rs
    title: Knowledge artifact builder
    author: process:git
    last_modified: 2026-08-16
---

# Keep generated knowledge publication local and repository-visible

## Context

Phase 5 requires an explicit publication disposition. The canonical bundle links to repository evidence, untracked local research may be present during migration, and the repository does not yet contain a completed source-and-license review for a verbatim distributable archive.[^okf-plan]

## Decision

Keep canonical records in the repository and generate the HTML site, `catalog.json`, `search.json`, `llms.txt`, and `validation.json` for local preview and CI verification. Do not configure a public deployment and do not upload a verbatim bundle archive in this phase.[^okf-plan][^builder][^workflow]

Generated outputs remain derived artifacts rather than canonical inputs. CI may build and compare temporary copies, but it does not retain or publish them.[^workflow]

## Consequences

Contributors and agents can inspect, search, build, and preview the same validated bundle without creating a hosted knowledge service. Public URLs, access policy, source inclusion, and redistribution licensing remain unresolved rather than being inferred from a successful local render.[^okf-plan]

A future public site or archive requires a separately reviewed change that identifies its audience and access level, inventories every included source and license, defines whether repository resources are copied or linked, and adds an explicit deployment or release path.[^okf-plan]

## Current disposition

The Phase 0 local-first contract is implemented by the Phase 5 CLI, generated outputs, preview server, and CI workflow. This record remains `draft` until a human reviews the publication decision and its evidence.

[^okf-plan]: Approved local-first publication baseline and Phase 5 archive safety condition.
[^workflow]: CI validates and compares temporary generated artifacts without uploading them.
[^builder]: Current deterministic local artifact set and canonical/derived boundary.
