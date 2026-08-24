---
type: Decision
title: Publish generated knowledge HTML on rocci.dev; forbid verbatim archives
description: Signed-out visitors may browse generated HTML of the committed knowledge bundle under /knowledge/ on rocci.dev. Canonical Markdown stays in git. A verbatim bundle archive remains forbidden.
tags: [domain/rocs, domain/site, integration/okf, concern/security, concern/publication, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-24T21:50:00Z }
authority: normative
owners: [human:nils]
sources:
  - id: okf-plan
    resource: ../../archive/reports/OKF_PLAN.md
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
  - id: site-lane
    resource: ../plans/site/okf-viewer-site-lane.md
    title: Mount the OKF knowledge viewer on rocci.dev
    author: process:cursor
    last_modified: 2026-08-24
---

# Publish generated knowledge HTML on rocci.dev; forbid verbatim archives

## Context

Phase 5 required an explicit publication disposition. The canonical bundle links to repository evidence, untracked local research may be present in a working tree, and the repository does not contain a completed source-and-license review for a verbatim distributable archive.[^okf-plan]

The original local-first rule allowed generated HTML, `catalog.json`, `search.json`, `llms.txt`, and `validation.json` for local preview and CI verification, and forbade public deployment and archive upload. rocci.dev packaging later needed a reviewed exception for **generated HTML of the committed bundle only**.[^site-lane]

## Decision

Keep canonical records as inert OKF Markdown in the repository. Generate the review HTML site, `pages.json`, `catalog.json`, `llms.txt`, and `validation.json` from the committed bundle.[^okf-plan][^site-lane]

Public HTML under `/knowledge/` on rocci.dev is allowed. Audience is signed-out visitors with the same access as the rest of the site; there is no extra authentication. The published set is that generated tree, not a source archive of `knowledge/` plus linked repository files. The review queue stays public. Knowledge URLs are listed via `/knowledge/sitemap.xml`, mentioned from the site `robots.txt`.[^site-lane]

Do not publish a verbatim bundle archive, `archive/`, untracked research, or a zip of the bundle. The Knowledge GitHub workflow stays validation-only: it may build and compare temporary copies, but it does not retain or upload them.[^workflow][^okf-plan]

Generated outputs remain derived artifacts rather than canonical inputs. `rocci-ops package site` is the public deploy artifact path for the HTML tree.[^site-lane]

## Consequences

Visitors can browse architecture, decisions, plans, research, and audits through the existing OKF review viewer without re-authoring records as Rocdown. Contributors still preview locally with `rocci-okf view`. Redistribution of the Markdown bundle and linked evidence as an archive stays unresolved and forbidden.[^site-lane][^okf-plan]

A hosted query, MCP, or review-decision service, a `knowledge.rocci.dev` hostname, and a downloadable tarball each still need their own reviewed change.[^site-lane]

## Current disposition

This amendment is the reviewed public-HTML exception named by the site-lane plan. Local `rocci-okf view` and Knowledge CI remain unprefixed validation and preview paths. This record remains `draft` until a human reviews the publication decision and its evidence.

[^okf-plan]: Approved local-first publication baseline and Phase 5 archive safety condition.
[^workflow]: CI validates and compares temporary generated artifacts without uploading them.
[^builder]: Historical deterministic local artifact set and canonical/derived boundary.
[^site-lane]: Phase 0 gate: public HTML under `/knowledge/`, no verbatim archive.
