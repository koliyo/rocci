---
type: Audit
title: Worktree landings and origin/main push conflicts
description: Why merging Cursor worktrees into local main repeatedly produced non-fast-forward pushes and pull-rebase file conflicts, and which agent workflow changes address that loop.
tags: [domain/rocci, concern/tooling, concern/migration, audience/maintainer]
status: draft
generated: { by: process:cursor, at: 2026-08-20T07:18:12Z }
stale_after: 2026-11-20
authority: descriptive
owners: [human:nils]
sources:
  - id: phase-runner
    resource: ../../.cursor/rules/phase-runner.mdc
    title: Phased Runner Instructions
    author: process:cursor
    last_modified: 2026-08-20
  - id: merge-skill
    resource: ../../.agents/skills/merge-worktree-to-main/SKILL.md
    title: Merge worktree to main skill
    author: process:cursor
    last_modified: 2026-08-20
  - id: push-worktrees
    resource: ../../tools/rocci-ops/src/rocci_ops/local.py
    title: Batch push command for branch-backed worktrees
    author: process:cursor
    last_modified: 2026-08-19
---

# Worktree landings and origin/main push conflicts

## Symptom

After landing local Cursor worktree work onto `main`, `git push origin main` often failed with a non-fast-forward rejection. A following `git pull` then produced file conflicts (frequently on shared knowledge files such as `knowledge/log.md`). The failure looked like a merge problem at push time; it was a **history divergence** problem that pull-with-rebase then turned into content conflicts.

`git push` does not merge trees. It accepts an update only when the remote tip is an ancestor of the commit being pushed. Diverged tips are rejected as non-fast-forward.

## Observed causes

Inspection of `main`'s reflog and worktree list (2026-08-19) showed several interacting habits, not a single Git bug.

### 1. Detached HEAD worktrees

Many Cursor worktrees were detached at old SHAs rather than on a named feature branch. Phase commits then had no stable branch tip. Landing that work into `main` often used cherry-pick or ad-hoc history surgery instead of `merge <branch>`. The merge-worktree skill refuses detached HEAD for that reason.[^merge-skill]

### 2. Rebasing `main` onto a feature branch

`main` reflog entries of the form `rebase (finish): refs/heads/main onto <feature-sha>` rewrote `main` so it was no longer a descendant of `origin/main`. After that rewrite, a normal push cannot fast-forward even when the working tree looks correct. Safe landing is the opposite direction: rebase the **feature** onto `main`, then merge into `main`.[^merge-skill]

### 3. Cherry-pick then later merge the same work

Cherry-picking worktree commits onto `main` creates new SHAs for the same diffs. Merging the original branch later reintroduces the old SHAs. Git treats those as distinct commits that often touch the same lines.

### 4. Stale worktree bases

Named feature branches frequently did not contain current `origin/main`. Cursor snapshots `main` when it creates a worktree; meanwhile other landings advance `main`. Merging an unrebased branch replays its changes against a tree it never saw. Shared append-only files such as `knowledge/log.md` collide often.

### 5. `pull.rebase=true` after a rejected push

Global `pull.rebase=true` means the recovery path after a rejected push replays local `main` (including merge and cherry-pick commits) onto `origin/main`. That is where repeated file conflicts appear. The push rejection itself is only the first signal.

### 6. Agents commit locally and do not push

The phased runner authorizes phase commits and forbids push.[^phase-runner] Work accumulates on local `main` while `origin/main` stays still. One later push then tries to publish a merge-heavy or rewritten history in a single step.

## Resolution in agent workflow

Two checked-in workflow pieces close the main loop.

### Named plan branch in the worktree

The phased runner now requires phase work on a named branch in the current worktree. Branch name defaults to the plan filename without `.md` (for example `knowledge/plans/live-reload-follow-ons.md` → `live-reload-follow-ons`). Detached HEAD and commits on `main` are disallowed. The agent must `git switch` / `git switch -c` before editing, must not steal a branch checked out in another worktree, must not rebase `main` onto the feature, and must not push.[^phase-runner]

Hand-off reports the branch name so a later landing has a mergeable tip.

### Land with rebase-on-feature, merge-into-main

The merge-worktree skill lands a **named** source branch by rebasing it onto local `main` in the source worktree (conflicts stay there), then creating a `--no-ff` merge commit on `main`. It stops on detached HEAD or when the source is `main`. It does not push, delete the worktree, or rewrite `main` onto the feature.[^merge-skill]

Together: phase work stays on a plan-named branch; landing rebases that branch and merges into `main`; `main`'s history remains pushable as a fast-forward relative to `origin/main` when `origin/main` has not moved independently.

## Residual risks

Named branches remove the detached-HEAD rewrite loop. They do not remove:

- Stale bases: old worktrees still need `git rebase main` (or equivalent) before merge, or content conflicts remain.
- Long delays before pushing `main` after each landing.
- Cherry-picking a branch and later merging that same branch.
- Accidental `git rebase <feature>` while on `main` (explicitly forbidden in the runner; still a human/agent mistake mode).
- Batch-pushing many stale branch-backed worktrees without rebasing them first.[^push-worktrees]

## Recommended landing habit

1. In the worktree: ensure a named plan branch; rebase onto current `main`.
2. In the main checkout: merge with `--no-ff` (or use `/merge-worktree-to-main`).
3. Push `origin main` after each successful landing before starting the next worktree merge.
4. Treat leftover Cursor worktrees as stale snapshots; do not merge detached HEADs.

[^phase-runner]: Named worktree branch, no commit on `main` or detached HEAD, no rebase of `main` onto the feature, no push.
[^merge-skill]: Rebase source branch onto `main` in the worktree; `--no-ff` merge into `main`; refuse detached HEAD and source `main`.
[^push-worktrees]: Pushes commits from every branch-backed worktree; detached worktrees are skipped; does not rebase onto `main`.
