---
type: Implementation Plan
title: Python and uv operator pipeline
description: "Replace CI, deploy, origin, and local maintainer shell with a pinned tools/rocci-ops uv package. POSIX remains for container PID 1, install-roc.sh, and OpenSSH ProxyCommand. Roc port is a later branch. Exploratory; Phases 1–6 implemented in this revision; not CI-complete."
tags: [domain/rocci, concern/ci, concern/tooling, concern/publication]
status: draft
generated: { by: process:cursor, at: 2026-08-21T10:00:00Z }
stale_after: 2026-11-21
authority: exploratory
owners: [human:nils]
sources:
  - id: ops-cli
    resource: ../../../tools/rocci-ops/src/rocci_ops/cli.py
    title: rocci-ops command dispatch
    author: process:cursor
    last_modified: 2026-08-21
  - id: ops-ci
    resource: ../../../tools/rocci-ops/src/rocci_ops/ci.py
    title: CI job bodies shared with GitHub Actions
    author: process:cursor
    last_modified: 2026-08-21
  - id: ops-deploy
    resource: ../../../tools/rocci-ops/src/rocci_ops/deploy.py
    title: SSH probe, bootstrap, and artifact push
    author: process:cursor
    last_modified: 2026-08-21
  - id: ops-origin
    resource: ../../../tools/rocci-ops/src/rocci_ops/origin.py
    title: Origin publish, up, and SQLite backup
    author: process:cursor
    last_modified: 2026-08-21
  - id: proxy
    resource: ../../../docker/prod/access-ssh-proxy.sh
    title: OpenSSH ProxyCommand for cloudflared Access
    author: process:git
    last_modified: 2026-08-21
  - id: research
    resource: ../../research/ops/python-uv-ops-pipeline.md
    title: Findings after the Python plus uv migration
    author: process:cursor
    last_modified: 2026-08-21
---

# Python and uv operator pipeline

## Goal

Give CI, the origin VPS, and localhost one `uv run rocci-ops`
surface so job lists and deploy steps cannot drift across YAML and bash.[^ops-cli][^ops-ci]

## Out of bound

Rewriting operator scripts in Roc. Installing Python or uv (assumed present).
Putting `rocci` / `rocdown` / `roc` / rustc on the origin. Changing Docker
image `ENTRYPOINT` scripts or the pinned Roc nightly installer.

## Constraints that do not move

CI, the VPS, and developers have CPython 3.12 and uv. Origin has no product
toolchain. Access SSH stays a POSIX `ProxyCommand`.[^proxy]

## Phase 1 — uv package and workspace-deps

**Bound:** `tools/rocci-ops` scaffold; workspace-deps in Python; lint uses uv.
**Exit:** `uv run rocci-ops check-deps`

## Phase 2 — CI job runner

**Bound:** Job bodies in `rocci_ops.ci`; thin `ci.yml` / `knowledge.yml`.
**Exit:** `uv run rocci-ops ci --list`; pytest

## Phase 3 — Release packaging and ci-gate

**Bound:** `rocci-ops archive version|package|wait-ci|params|publish` (was `release` before the DX split)
**Exit:** pytest archive naming; `release.yml` calls the CLI

## Phase 4 — Deploy client and origin

**Bound:** `deploy probe|bootstrap|push` and `origin publish|up|backup`
**Exit:** mocked SSH/health tests; `site.yml` deploy uses uv[^ops-deploy][^ops-origin]

## Phase 5 — Local maintainer commands

**Bound:** install, package, bundle, serve, worktrees; delete maintainer `.sh`
**Exit:** pytest; docs cite `uv run rocci-ops`

## Phase 6 — Docs and knowledge findings

**Bound:** AGENTS, devops skill, this plan, research report, log.
**Exit:** `cargo run -q -p rocci-okf -- check knowledge --profile base --format terminal`

Roc follow-on is a **new branch** after this plan is green on CI.[^research]

[^ops-cli]: Console script `rocci-ops` in `tools/rocci-ops`.
[^ops-ci]: Shared lint/test/fixtures/editors/knowledge command lists.
[^ops-deploy]: Laptop and Actions SSH using Access `ProxyCommand`.
[^ops-origin]: Remote `uv run rocci-ops origin publish SHA`.
[^proxy]: `exec cloudflared access ssh`.
[^research]: Roc-port gaps recorded after the Python implementation.
