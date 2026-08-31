---
type: Implementation Plan
title: rocci-ops DX alignment
description: Okmate-aligned rocci-ops command tree, operator release that writes the workspace version, and hosted Cut release without a Homebrew tap.
tags: [domain/ops, domain/rocci, concern/ci, concern/tooling]
status: draft
generated: { by: process:cursor, at: 2026-08-30T21:50:00Z }
stale_after: 2026-11-28
authority: exploratory
owners: [human:nils]
sources:
  - id: cli
    resource: ../../../rocci-ops/src/rocci_ops/cli.py
    title: rocci-ops command list
    author: process:cursor
    last_modified: 2026-08-30
  - id: release
    resource: ../../../rocci-ops/src/rocci_ops/release.py
    title: operator release after version commit
    author: process:cursor
    last_modified: 2026-08-30
  - id: archive
    resource: ../../../rocci-ops/src/rocci_ops/archive.py
    title: hosted archive helpers
    author: process:cursor
    last_modified: 2026-08-30
  - id: cut
    resource: ../../../.github/workflows/cut-release.yml
    title: Cut release workflow_dispatch
    author: process:cursor
    last_modified: 2026-08-30
  - id: readme
    resource: ../../../README.md
    title: Rocci README publish section
    author: process:cursor
    last_modified: 2026-08-30
---

# rocci-ops DX alignment

## Goal

Give CI and localhost one `uv run --no-dev rocci-ops` surface so hosted
jobs and local replay cannot drift, and give operators the same `release
patch|minor|major|v*|dev` verb as okmate-ops, plus Rocci-only site
lanes.[^cli][^release][^readme]

## Commands

- `ci` — lint, test, fixtures-and-docs, editors, knowledge
- `check` — deps, docs, zed
- `build` / `build playground`, `install`, `package macos|vscode|zed|site|icons`
- `site`, `serve`, `deploy`, `origin`, `pr-checkout`, `push-worktrees`
- `promote staging|production` — site lanes only
- `release patch|minor|major|vX.Y.Z` — workspace version commit on the
  target branch, wait for hosted lint and Test Workspace, push an
  immutable `v*` tag[^release]
- `release dev` — wait for those checks and force-move the rolling
  prerelease tag
- `archive` — hosted `version|package|params|wait-ci|publish` used by
  `release.yml`[^archive]
- Hosted **Cut release** runs `rocci-ops release` via `workflow_dispatch`
  from `main` (no `release` / `staging` / `production` environment).
  Hosted **Release** packages an existing tag only[^cut]

## Out of bound

Sparkle, Homebrew, h35-ops `package desktop` for Rocci.app, rewriting
operator tools in Roc, waiting for fixtures/editors/knowledge before a
tag, and changing CI job bodies or origin lanes.

## Constraints that do not move

Python 3.12, hatchling, stdlib-only runtime, pytest in the `dev` group,
committed `uv.lock`. One `[workspace.package]` version. Do not
force-fetch all git tags.

## Phase 1 — Command tree and module split

**Bound:** dispatch, help, split of `local.py`, lazy h35. `promote tag`
still worked.
**Exit:** `uv run --directory rocci-ops --group dev pytest`;
`uv run --no-dev rocci-ops -h` lists the new tree.

## Phase 2 — Free the `release` name

**Bound:** hosted helpers as `archive`; `ghutil.py`; `release.yml` calls
`archive`.
**Exit:** pytest; workflows do not call `rocci-ops release version|package|…`.

## Phase 3 — Operator `release`

**Bound:** `version.py` plus operator `release.py`; `promote` is
staging/production only.
**Exit:** pytest; `release --help` and `promote --help`.

## Phase 4 — Hosted cut-release and docs

**Bound:** `cut-release.yml`, README, devops skill, this record.
**Exit:** `okmate check knowledge --profile base`; README documents
`release patch` as the only `v*` path.[^readme][^cut]

[^cli]: Console script `rocci-ops` in `rocci-ops`.
[^release]: Version commit on the target branch, then annotated `v*` or force-moved `dev`.
[^archive]: Archive naming and GitHub release publish stay on the tag workflow.
[^cut]: `workflow_dispatch` inputs `spec`, `from`, `force`, `dry_run`.
[^readme]: Development, site promote, and rolling `dev` tag documentation.
