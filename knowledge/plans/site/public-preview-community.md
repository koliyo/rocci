---
type: Implementation Plan
title: Rocci public-preview branding and community plan
description: Prepare a reversible Rocci public preview for a near-term open-source repository, collect Roc and Datastar feedback, and turn the results into explicit naming, visual, and governance decisions.
tags: [domain/rocci, domain/rocdown, domain/rocs, concern/branding, concern/community, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-31T08:00:00Z }
stale_after: 2026-10-01
authority: exploratory
owners: [human:nils]
sources:
  - id: branding-research
    resource: ../../research/site/branding-community-foundation.md
    title: Rocci branding and community foundation research
    author: process:codex
    last_modified: 2026-08-18
  - id: root-readme
    resource: ../../../README.md
    title: Rocci workspace overview
    author: process:git
    last_modified: 2026-08-19
  - id: cargo-workspace
    resource: ../../../Cargo.toml
    title: Workspace package metadata including license
    author: process:git
    last_modified: 2026-08-21
  - id: roc-community
    resource: https://roc-lang.org/community
    title: Roc community
    author: organization:roc-programming-language-foundation
  - id: datastar-community
    resource: https://data-star.dev/star_federation
    title: Star Federation community and purpose
    author: organization:star-federation
  - id: github-health
    resource: https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/creating-a-default-community-health-file
    title: GitHub community health files
    author: organization:github
    last_modified: 2026-08-17
---

# Rocci public-preview branding and community plan

## Goal

Make Rocci safe and understandable to evaluate publicly, collect structured
feedback from the Roc community and then the Datastar community, and decide the
name, hierarchy, and first visual identity from evidence. This plan does not
authorize publication by itself and does not treat the preview recommendations
as permanent brand decisions.[^branding-research]

## Working position

The repository is intended to become public open source shortly. The root
`LICENSE` is Apache-2.0 and every workspace crate packages that text. Conduct,
contribution, and the rest of Phase 0 are still the publication gate. This
record does not flip the repository public and does not set a calendar date.[^cargo-workspace][^root-readme]

Use Rocci as the preview masterbrand, Rocdown as the endorsed document format,
and Rocci Docs as the public label for the current Rocs engine and command. Use
“Rocci — Composable authoring for applications and content” as the descriptor,
describe the project as independent, and retain implementation names for
compatibility during the feedback period.[^branding-research]

## Phase 0: repository launch gate

- Root Apache-2.0 license text is present and is packaged with every workspace crate.
- Conduct, contribution, security, support, and governance documents plus
  focused issue forms are in the repository root and `.github/ISSUE_TEMPLATE`.
  Discussions remain a later announcement gate.[^github-health]
- Publish one support matrix for Roc revision, Datastar pin, operating systems,
  editor state, packaging state, and known limitations.
- Prove one clean installation and five-minute example from a tagged revision.
- Distinguish working, experimental, proposed, and historical behavior in the
  launch copy.[^root-readme]

Exit when an unfamiliar developer can install, render the first component,
identify current limitations, find help, and understand the license without
private instructions.

## Phase 1: preview identity and message

- Replace the home proposition with a short task-oriented Roc headline.
- Present one architecture family: Rocci templates, Rocdown, Rocci Docs, and
  runtime, with Roc and Datastar shown as ecosystem relationships.
- Add a favicon family and a consistent social card only after comparing three
  zero-based, one-color vector routes: folded letter, non-letter modular mark,
  and wordmark only. Do not use the existing placeholder as design input. Treat
  the orange folded R as a maintainer preference to test, not the default.
- Fix the identified small-text contrast failures and run a broader light/dark,
  keyboard, zoom, forced-color, reduced-motion, and print audit.
- Add focused discovery pages for Rocdown, Rocci Docs, and Datastar integration.

Exit when the site explains the project, relationship, maturity, and first task
within one screen at mobile and laptop widths.

## Phase 2: Roc community feedback

Introduce the preview in Roc Zulip, which Roc identifies as the most active
community gathering place. Confirm the current appropriate projects or
show-and-tell location with community regulars. Share a short demo, architecture
view, public feedback link, and independent-project disclaimer. Adapt the
reviewed Roc-first draft rather than composing the announcement at posting
time.[^roc-community]

Ask separately about name pronunciation and recall, brand hierarchy,
“Roc-native” wording, most valuable next workflow, and whether the folded visual
direction feels related or derivative. Keep syntax, compiler compatibility, and
governance as separate topics so participants can answer bounded questions.

Exit after a published two-week synthesis records responses, disagreements,
decisions, and explicit deferrals without treating silence as approval.

## Phase 3: Datastar community feedback

Share one focused server-driven interaction example in the Datastar Discord,
which Star Federation identifies as its community support venue. Present Rocci
as an independent backend integration and request review of patch semantics,
documentation accuracy, and whether a Roc example or SDK would be useful.[^datastar-community]

Do not call the integration official unless Star Federation accepts that status.
Record requested changes and compatibility expectations separately from Rocci
branding feedback.

## Phase 4: decision and contributor loop

- Decide or reopen the Rocci name and publish the pronunciation.
- Confirm the masterbrand hierarchy or document evidence for a different model.
- Request Roc-project guidance before any direct Roc-logo lockup or co-branding.
- Select and commission one vector identity only after the name decision and
  comparison of the folded-letter, non-letter, and wordmark-only routes.
- Publish the next narrow milestone and bounded contributor issues.
- Track clean installs, time to first render, independent examples, repeat
  contributors, support response, unresolved feedback themes, and failed site
  searches rather than optimizing for stars alone.

## Execution assets and deferred work

The preliminary exact-name registry/domain screen, announcement and feedback
templates, and a responsive landing-page direction are complete as exploratory
work. They still
require current fact and link checks before publication.

Trademark and full namespace clearance, 8–12 naming interviews, zero-based
vector route comparison and logo similarity review, a complete accessibility
audit, first-impression and message testing, production landing-page
implementation, and durable governance design remain separate investigations.
A legal foundation, separate subbrand communities, broad crate renames, and
merchandise are out of scope until those investigations and the preview are
complete.

[^branding-research]: Canonical exploratory hierarchy, naming, SEO, design, and community synthesis.
[^root-readme]: Current shipped and planned workspace behavior used to bound public claims.
[^cargo-workspace]: Workspace package metadata makes the root `LICENSE` available to every crate; that file contains the Apache License 2.0 text.
[^roc-community]: Current Roc participation and community-venue guidance.
[^datastar-community]: Current Datastar community-support venue and ecosystem framing.
[^github-health]: Supported GitHub community-health artifacts and their purpose.
