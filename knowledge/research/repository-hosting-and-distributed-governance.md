---
type: Research Report
title: Repository hosting for Rocci's distributed governance
description: Exploratory comparison of GitHub and Tangled for Rocci's public launch, contributor workflow, infrastructure ownership, and future distributed governance.
tags: [domain/rocci, concern/community, concern/governance, concern/publication, concern/ci]
status: draft
generated: { by: process:codex, at: 2026-08-18T18:06:17Z }
stale_after: 2026-11-18
authority: exploratory
owners: [human:nils]
sources:
  - id: preview-plan
    resource: ../plans/public-preview-community.md
    title: Rocci public-preview branding and community plan
    author: process:codex
    last_modified: 2026-08-18
  - id: ci-workflow
    resource: ../../.github/workflows/ci.yml
    title: Rocci GitHub Actions CI workflow
    author: process:git
    last_modified: 2026-08-17
  - id: release-workflow
    resource: ../../.github/workflows/release.yml
    title: Rocci GitHub Actions release workflow
    author: process:git
    last_modified: 2026-08-17
  - id: github-rulesets
    resource: https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets
    title: About GitHub repository rulesets
    author: organization:github
  - id: github-roles
    resource: https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/repository-roles-for-an-organization
    title: GitHub repository roles for organizations
    author: organization:github
  - id: tangled-docs
    resource: https://docs.tangled.org/
    title: Tangled documentation
    author: organization:tangled
    last_modified: 2025-12-21
  - id: tangled-federation
    resource: https://blog.tangled.org/federation/
    title: We need a federation of forges
    author: organization:tangled
    last_modified: 2026-04-29
  - id: tangled-ci
    resource: https://blog.tangled.org/ci/
    title: Introducing spindle
    author: organization:tangled
    last_modified: 2025-08-06
  - id: tangled-migrations
    resource: https://docs.tangled.org/migrating-knots-and-spindles
    title: Migrating Tangled knots and spindles
    author: organization:tangled
---

# Repository hosting for Rocci's distributed governance

## Scope and authority

This record compares GitHub and Tangled for Rocci's intended public open-source
launch and later distributed governance. It is exploratory: it neither approves
a canonical forge nor defines the project's governance model. Repository
hosting, contributor authority, legal ownership, release authority, and control
of durable project assets are related but separate decisions.[^preview-plan]

## Current Rocci constraints

Rocci's public-preview plan requires licensing, conduct, contribution,
security, support, governance, focused feedback forms, a support matrix, and a
reproducible tagged installation before publication. It currently recommends
maintainer-led open development and defers durable governance design until
multiple maintainers, funding, or shared assets create the need.[^preview-plan]

The repository is operationally coupled to GitHub today. Its CI uses GitHub
Actions for macOS and Linux workspace tests, formatting, lints, syntax
fixtures, documentation validation, and editor builds. Its release workflow
uses GitHub check runs, Actions artifacts, tags, generated release notes, and
GitHub Releases to publish platform archives.[^ci-workflow][^release-workflow]

## Platform comparison

| Concern | GitHub | Tangled |
| --- | --- | --- |
| Public discovery and onboarding | Large existing contributor network and familiar pull-request workflow | Smaller network and a less familiar contribution model |
| Maintainer delegation | Organization roles, teams, review controls, and visible branch or tag rulesets | Repository collaborators and server membership, with fewer documented policy controls |
| CI and releases | Current Rocci workflows already cover validation, platform matrices, artifacts, and releases | Spindle supplies self-hostable event-driven CI, but Rocci would need new pipeline and release automation |
| Infrastructure ownership | One proprietary service remains the forge-level dependency | Open-source knots host Git repositories and spindles run CI on independently operated infrastructure |
| Cross-host collaboration | Git repositories and forks are portable, but issues, pull requests, identities, and Actions remain GitHub services | AT Protocol events support issues and pull requests across knots, including cross-server forks |
| Operational maturity | Established APIs, integrations, permissions, and contributor expectations | Rapidly evolving; current migration guidance documents incompatible APIs and upgrade-sensitive event behavior |

GitHub rulesets can require reviews, checks, signed commits, linear history, or
restricted updates on selected branches and tags. Organization roles separate
read, triage, write, maintain, and administration responsibilities. These
controls make shared authority enforceable without giving every maintainer full
administrative access.[^github-rulesets][^github-roles]

Tangled separates repository storage into lightweight self-hostable knots and
CI into self-hostable spindles, while a shared appview presents repositories
across the network. Its federation model allows a contributor to host a fork on
one server and submit a pull request to a repository on another. This directly
reduces dependence on a single repository or CI operator.[^tangled-docs][^tangled-federation][^tangled-ci]

That architectural fit comes with present operational risk. Tangled's current
migration guide describes non-backward-compatible knot and spindle APIs,
alpha-version upgrades, CI history that cannot be migrated automatically in
one transition, and older knots that may silently drop newer repository events
until upgraded. Rocci should therefore treat Tangled availability and workflow
compatibility as evidence to establish, not as settled infrastructure.[^tangled-migrations]

## Governance interpretation

Decentralized hosting does not by itself produce distributed governance. A
GitHub organization can delegate review, maintenance, and release authority to
several people, while a self-hosted Tangled knot can remain controlled by one
operator. Conversely, repository governance can be distributed while issues,
CI, releases, domains, package namespaces, and credentials remain concentrated
in one account.

Rocci should evaluate governance across at least these control planes:

- decisions and maintainer succession;
- merge and branch-policy authority;
- release signing and package publication;
- repository, CI, domain, and documentation infrastructure;
- community moderation and security response; and
- recovery when a maintainer or service becomes unavailable.

## Exploratory recommendation

Use GitHub as the canonical host for the initial public launch, under a Rocci
organization with documented governance, multiple maintainers, protected
branches, required checks, and shared release authority. This preserves the
existing CI and release path and minimizes contributor onboarding risk.

At the same time, establish Tangled as a first-class mirror and bounded
infrastructure pilot: mirror all branches and tags, run a non-blocking subset of
checks on a spindle, test cross-knot contribution, and document which metadata
does and does not synchronize. Do not split authoritative issue or pull-request
state until ownership, synchronization, moderation, backup, and recovery rules
are explicit.

Reconsider the canonical host after at least two public release cycles. A move
to Tangled, or an equal multi-forge model, should require demonstrated CI and
release reliability, contributor participation outside GitHub, tested knot and
spindle recovery, and a governance policy that assigns authority independently
of whichever service displays the repository.

## Questions requiring a decision

1. Is the first public repository owned by a Rocci organization, and who holds
   owner, maintainer, triage, and release roles?
2. Which decisions require one maintainer, multiple maintainers, or community
   consultation?
3. Are GitHub issues and pull requests initially canonical, and what Tangled
   interactions are accepted during the pilot?
4. Who operates the Tangled knot and spindle, and how are backups, upgrades,
   credentials, and recovery shared?
5. What evidence and review date would permit Tangled to become canonical or
   an equal submission channel?

[^preview-plan]: Current exploratory public-preview gates and explicit deferral of durable governance design.
[^ci-workflow]: Current repository validation and platform matrix implemented on GitHub Actions.
[^release-workflow]: Current check-gated artifact and GitHub Release publication path.
[^github-rulesets]: Public repository rules for reviews, checks, history, signatures, and protected references.
[^github-roles]: Organization repository roles and their graduated permissions.
[^tangled-docs]: Tangled's documented open-source, self-hostable knot and federated appview architecture.
[^tangled-federation]: Tangled's documented cross-server fork, issue, and pull-request model over AT Protocol.
[^tangled-ci]: Tangled's spindle CI architecture and pipeline events.
[^tangled-migrations]: Current operational migration and compatibility risks for self-hosted knots and spindles.
