---
type: Research Report
title: Tangled as canonical host with a GitHub macOS CI mirror
description: "Operational research for making Tangled the Rocci forge and Linux CI, while keeping GitHub as a one-way git mirror that supplies macOS GitHub Actions runners. Exploratory; not an approved hosting decision."
tags: [domain/rocci, concern/ci, concern/governance, concern/publication, concern/community]
status: draft
generated: { by: process:cursor, at: 2026-08-19T16:10:00Z }
stale_after: 2026-11-19
authority: exploratory
owners: [human:nils]
sources:
  - id: hosting-research
    resource: ../repository-hosting-and-distributed-governance.md
    title: Repository hosting for Rocci's distributed governance
    author: process:codex
    last_modified: 2026-08-18
  - id: preview-plan
    resource: ../../plans/site/public-preview-community.md
    title: Rocci public-preview branding and community plan
    author: process:cursor
    last_modified: 2026-08-19
  - id: ci-workflow
    resource: ../../../.github/workflows/ci.yml
    title: Rocci GitHub Actions CI workflow
    author: process:git
    last_modified: 2026-08-19
  - id: knowledge-workflow
    resource: ../../../.github/workflows/knowledge.yml
    title: Rocci GitHub Actions knowledge workflow
    author: process:git
    last_modified: 2026-08-17
  - id: release-workflow
    resource: ../../../.github/workflows/release.yml
    title: Rocci GitHub Actions release workflow
    author: process:git
    last_modified: 2026-08-18
  - id: ci-local
    resource: ../../../rocci-ops/src/rocci_ops/ci.py
    title: Local CI job runner
    author: process:git
    last_modified: 2026-08-19
  - id: devops-skill
    resource: ../../../.agents/skills/rocci-devops/SKILL.md
    title: Rocci GitHub Actions devops skill
    author: process:git
    last_modified: 2026-08-19
  - id: knowledge-skill
    resource: ../../../.agents/skills/manage-rocci-knowledge/SKILL.md
    title: Rocci knowledge-bundle skill
    author: process:git
    last_modified: 2026-08-18
  - id: docs-config
    resource: ../../../docs/rocdown.toml
    title: rocci.dev site configuration
    author: process:git
    last_modified: 2026-08-19
  - id: tangled-intro
    resource: https://docs.tangled.org/introduction.html
    title: Tangled introduction
    author: organization:tangled
  - id: tangled-docs
    resource: https://docs.tangled.org/single-page
    title: Tangled documentation
    author: organization:tangled
  - id: tangled-spindles
    resource: https://docs.tangled.org/spindles
    title: Tangled spindle pipelines
    author: organization:tangled
  - id: tangled-pages
    resource: https://docs.tangled.org/hosting-websites-on-tangled
    title: Hosting websites on Tangled
    author: organization:tangled
  - id: tangled-ci
    resource: https://blog.tangled.org/ci/
    title: Introducing spindle
    author: organization:tangled
    last_modified: 2025-08-06
  - id: tangled-migrations
    resource: https://docs.tangled.org/migrating-knots-and-spindles
    title: Migrating Tangled knots and spindles
    author: organization:tangled
  - id: spindle-macos
    resource: https://tangled.org/tangled.org/core/issues/730
    title: "Proposal: spindle external workers (macOS reference worker)"
    author: organization:tangled
  - id: tangled-groups
    resource: https://tangled.org/tangled.org/core/issues/550
    title: "Repository Groups issue"
    author: organization:tangled
  - id: tangled-repo-lexicon
    resource: https://tangled.org/tangled.org/core/blob/master/lexicons/repo/repo.json
    title: sh.tangled.repo lexicon
    author: organization:tangled
  - id: tangled-handle
    resource: https://tangled.org/tangled.org
    title: tangled.org profile listing sibling repositories
    author: organization:tangled
  - id: no-orgs-review
    resource: https://codeka.io/en/2026/05/22/tangled/
    title: Independent review of Tangled (no orgs or groups)
    author: human:julien-wittouck
    last_modified: 2026-05-22
  - id: workspace
    resource: ../../../Cargo.toml
    title: Rocci Cargo workspace
    author: process:git
    last_modified: 2026-08-19
---

# Tangled as canonical host with a GitHub macOS CI mirror

## Scope and authority

This record is operational research for a Tangled-first hosting and devops
flow. It does not approve a canonical forge, a governance model, or a public
launch. The earlier comparison recommended GitHub as the launch host and
Tangled as a bounded mirror; this record studies the inverse topology the
project now wants to try: Tangled owns git, issues, pull requests, and Linux
CI; GitHub remains a mirrored repository that supplies macOS GitHub Actions
runners.[^hosting-research]

Rocci is intended to become public open source shortly. The public-preview
plan is still the publication gate and does not itself flip the repository.
Because the first public clone should not treat GitHub as canonical, the
Tangled origin flip is launch-blocking. The earlier comparison’s “two public
release cycles before reconsidering the host” does not apply to this
path.[^preview-plan][^hosting-research]

Tangled's public appview is tangled.org. Handles on Tangled's PDS use
`tngl.sh`. Colloquial `tangled.sh` refers to that product, not a second
forge.[^tangled-intro]

## Current Rocci coupling

The repository remote is `https://github.com/koliyo/rocci.git`. Validation,
knowledge checks, and releases are GitHub Actions workflows. Day-to-day Linux
and macOS tests, lint, fixtures, editor builds, and OKF validation all run
there. The knowledge skill and several plans treat green GitHub CI and
Knowledge runs as the phase-completion gate.[^ci-workflow][^knowledge-workflow][^release-workflow][^knowledge-skill][^devops-skill]

Those jobs do not all need macOS:

| Job | Runner today | Needs Apple hardware or Darwin? |
| --- | --- | --- |
| `lint` | `ubuntu-latest` | No |
| `test` (`ubuntu-latest`) | Linux | No |
| `test` (`macos-latest`) | macOS | Yes, as a platform matrix |
| `fixtures-and-docs` | `ubuntu-latest` | No |
| `editors` | `macos-latest` | VS Code extension-host tests are the Darwin-sensitive part; Zed `wasm32-wasip1` can run on Linux |
| `knowledge` | `macos-latest` | No; `fetch-depth: 0` git provenance is OS-neutral |
| release `x86_64-unknown-linux-gnu` | `ubuntu-latest` | No |
| release `aarch64-apple-darwin` | `macos-latest` | Yes |

`uv run rocci-ops ci` already encodes the OS-neutral job bodies. Spindle
workflows should call those same commands rather than forking a third copy of
lint, test, fixture, and knowledge steps.[^ci-local]

Public site metadata still points at GitHub
(`docs/rocdown.toml` `repository`). Tangled Pages cannot attach a custom
domain, so `rocci.dev` cannot move onto Tangled Sites until that exists.[^docs-config][^tangled-pages]

## What Tangled supplies

Tangled is an AT Protocol forge: knots store git repositories, spindles run
CI, and the appview presents repositories, issues, and pull requests across
knots. Managed knots and CI at tangled.org are documented as free to use.
Everything is self-hostable.[^tangled-intro][^tangled-docs]

Spindle pipelines live in `.tangled/workflows` as YAML. Triggers are `push`,
`pull_request`, and `manual`, with branch and tag globs. Engines are `nixery`
(per-step Nixery containers, workspace persisted at `/tangled/workspace`) and
`microvm` (one guest for the whole workflow, with NixOS `dependencies`,
`services`, `caches`, and Docker-in-VM). Documented Rust recipes install
`rustc`, `cargo`, `clippy`, `rustfmt`, and C libraries through nixpkgs. There
is no GitHub Actions–style `runs-on: macos-latest`.[^tangled-spindles]

Default clone depth is 1. Knowledge validation needs full history, so that
workflow must set a deep or unshallow clone.[^tangled-spindles][^knowledge-workflow]

Hosted and self-hosted spindle defaults document a 5-minute workflow timeout.
A Rocci workspace clippy plus `cargo test --workspace` will exceed that
unless the spindle operator raises
`SPINDLE_*_PIPELINES_WORKFLOW_TIMEOUT`. That is evidence to collect in the
pilot, not a reason to assume the managed spindle can run full CI as-is.[^tangled-spindles][^tangled-docs]

Webhooks cover push, repository rename, and pull-request lifecycle events,
with optional HMAC secrets. They notify; they do not copy git objects. A
GitHub mirror therefore needs a pusher: a spindle step with a deploy key, a
small webhook consumer, or maintainer dual-push. Tangled has no documented
tooling to import GitHub issues or pull requests.[^tangled-docs]

Sites are static trees from a branch and deploy directory, served at
`handle.tngl.sh` or a claimed `*.tngl.io` path. Custom domains are explicitly
unimplemented.[^tangled-pages]

Knot and spindle APIs have been moving through incompatible alpha upgrades.
Pipeline history did not automatically survive the v1.16 spindle-owned-data
change. Treat availability, timeout policy, and upgrade cadence as part of
the hosting contract.[^tangled-migrations]

Native macOS spindle workers are a proposal with a Swift reference worker,
not a documented public runner. Until that ships, Apple CI stays on GitHub
Actions `macos-latest`.[^spindle-macos]

## Inverse topology

```text
contributors  -->  Tangled knot (canonical git, issues, PRs)
                       |
                       | spindle (Linux lint, test, fixtures, knowledge, linux release)
                       |
                       v
                 one-way git mirror  -->  GitHub repo (no canonical PRs)
                                              |
                                              v
                                        Actions macos-latest
                                        (Darwin tests, VS Code host tests,
                                         aarch64-apple-darwin archives)
```

This inverts the 2026-08-18 recommendation. GitHub remains necessary for Apple
runners, GitHub Releases as a binary channel, and marketplace `repository`
URLs. It stops being the place where Rocci accepts patches or tracks
issues.[^hosting-research][^ci-workflow][^release-workflow]

## Hard problems

### Mirror is SHA-faithful, metadata is not

Branches and tags can be fast-forwarded from Tangled to GitHub. Issues, pull
requests, check runs, and release notes do not sync. GitHub pull requests
opened against the mirror must be non-canonical, or they will fork the
project's review history.[^tangled-docs][^hosting-research]

### Pull-request macOS coverage

A Tangled pull request from a fork is not on the GitHub mirror unless a
service pushes that SHA. Without that pusher, Darwin jobs run only for
branches already mirrored (typically `main` and maintainer feature branches).
The merge policy has to say whether macOS is required on every Tangled PR or
only on mirrored refs and tags.[^tangled-spindles][^ci-workflow]

### Status is split

Spindle status appears on Tangled. GitHub Actions status appears on GitHub.
Nothing today aggregates them into one merge gate. Options are a spindle step
that polls `gh run`, a Tangled webhook comment, or an explicit policy that
Linux is blocking on Tangled and Darwin is blocking on the mirrored SHA.[^tangled-ci][^devops-skill]

### Releases are split

Linux archives can be built on spindle; Darwin archives need GitHub
`macos-latest`. Today's `ci-gate` waits for named GitHub check runs and
`gh release create` publishes both archives. A Tangled-first flow must either
keep tagged Linux builds on GitHub as well, or ferry spindle artifacts into
the same GitHub Release.[^release-workflow]

### Contributor and agent surface

`rocci-devops`, the knowledge completion rule, README CI instructions, and
site `repository` URLs all assume GitHub Actions and
`github.com/koliyo/rocci`. Those are documentation and skill changes, not
blockers, but they are part of making Tangled the devops flow rather than a
silent extra remote.[^devops-skill][^knowledge-skill][^docs-config][^preview-plan]

## Repository grouping

Tangled has no shipped GitHub-style organization and no GitLab-style group
that nests `rocci/rocci-okf` under an umbrella path. Repositories are
records (`sh.tangled.repo`) owned by one AT Protocol identity. The public
URL is `tangled.org/<handle>/<name>` with a flat name, plus optional
`topics`, `website`, and `description`. An open issue asks for profile
groups that would not change those URLs; it is not implemented. Independent
reviews of the current product reach the same conclusion.[^tangled-repo-lexicon][^tangled-groups][^no-orgs-review]

What exists instead:

| Mechanism | What it groups | What it is not |
| --- | --- | --- |
| One handle owning several repos | Profile listing, e.g. `tangled.org` holds `core`, `blog`, `infra`, `knot-docker` as siblings | An org with members, roles, and a nested namespace |
| Pinned repos on a profile | Featured subset of that handle’s repos | A product umbrella with its own ACL |
| Knot members | Who can use a knot | Per-product grouping; a knot is infrastructure |
| Per-repo collaborators | Write access on one repo | Inherited group permissions |
| Topics | Search-ish tags on a repo record | Folders or sub-orgs |
| Dedicated AT account named `rocci` | Cosmetic umbrella: `tangled.org/rocci.dev/rocci-okf` still looks like `owner/repo` | GitHub `github.com/rocci/rocci-okf` org semantics |

Rocci today is one Cargo workspace git repository, not several git remotes.
Splitting into `rocci`, `rocci-okf`, and `rocci-rocdown` remotes is a
separate product-boundary choice. Tangled can host those as sibling repos
under one handle; it cannot present them as a nested org, share one set of
org teams across them, or give `rocci` a URL prefix that owns the others.
A community AT Protocol org design exists outside Tangled core and is not
the current appview contract.[^workspace][^tangled-handle][^tangled-groups]

## Working recommendation

Use a managed Tangled knot as the canonical git host once a spindle can run
the Linux job set with a realistic timeout, and complete that origin flip
before the repository is public. Keep GitHub as a fast-forward-only
mirror whose only required jobs are Darwin tests, VS Code extension-host
tests, and `aarch64-apple-darwin` release archives. Do not treat Tangled Pages
as `rocci.dev`. Do not migrate GitHub issue or PR history. Do not split
authoritative review across both forges.[^tangled-intro][^tangled-pages][^hosting-research][^preview-plan]

Self-host a spindle if the managed timeout or Nix/GTK dependency set cannot
run `lint` plus workspace tests. Self-host a knot only when backup, upgrade,
and collaborator recovery need to leave Tangled's managed servers.[^tangled-migrations][^tangled-spindles]

## Questions that remain open

1. Which AT Protocol handle and knot host the canonical `rocci` repository?
2. Can hosted spindle complete the Linux job set, or is a self-hosted spindle
   with a longer timeout required?
3. Are Darwin jobs required on every Tangled pull request, or only on
   mirrored branches and tags?
4. Do tagged Linux binaries stay on GitHub Actions, or get uploaded from
   spindle into GitHub Releases?
5. When does the knowledge completion gate cite Tangled pipeline IDs instead
   of, or in addition to, GitHub run IDs?
6. What calendar date is the public open-source clone? Only “shortly” is
   decided.
7. One git workspace versus sibling Tangled repos under a dedicated `rocci`
   handle? Grouping cannot go further than that until Tangled ships orgs or
   profile groups.

[^hosting-research]: Earlier exploratory comparison kept GitHub canonical and Tangled as a pilot mirror.
[^preview-plan]: Near-term public open-source intent; Phase 0 remains the publication gate and does not set a date.
[^ci-workflow]: Current Linux and macOS validation matrix on GitHub Actions.
[^knowledge-workflow]: OKF validation currently pinned to `macos-latest` without a Darwin-specific command.
[^release-workflow]: Tag-gated Linux and Darwin archives published through GitHub Releases.
[^ci-local]: Shared local commands for lint, test, fixtures, editors, and knowledge.
[^devops-skill]: Current agent devops surface is GitHub Actions via `gh`.
[^knowledge-skill]: Phase-complete logging currently requires green GitHub CI and Knowledge workflows.
[^docs-config]: Published documentation still names the GitHub repository URL.
[^tangled-intro]: Open-source, self-hostable knots, spindles, and a shared appview; managed hosting is free.
[^tangled-docs]: Migration of git remotes, dual-push mirrors, webhooks, and alpha knot/spindle upgrades.
[^tangled-spindles]: `.tangled/workflows` YAML, nixery and microVM engines, clone depth, timeout defaults, Rust recipe.
[^tangled-pages]: Static sites on `*.tngl.sh` / `*.tngl.io` without custom domains.
[^tangled-ci]: Spindle consumes knot pipeline events and streams status; hosted secrets use OpenBao.
[^tangled-migrations]: Incompatible knot/spindle API upgrades and non-portable pipeline history.
[^spindle-macos]: External macOS workers are proposed, not a public Tangled runner.
[^tangled-groups]: Open request for profile groups that would not change repo URLs; not shipped.
[^tangled-repo-lexicon]: Repo records are flat `name` plus optional topics, website, description, knot, spindle.
[^tangled-handle]: One identity listing several sibling repositories; Tangled itself stays a monorepo in `core`.
[^no-orgs-review]: Current product has no org detached from a human account; workaround is a dedicated AT account plus per-repo collaborators.
[^workspace]: Rocci is one Cargo workspace git tree today.
