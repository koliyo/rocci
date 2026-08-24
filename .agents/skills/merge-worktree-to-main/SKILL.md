---
name: merge-worktree-to-main
description: >-
  Commit remaining work in the current git worktree, rebase that branch onto
  main (conflicts stay in the worktree), then merge into main with a merge
  commit (--no-ff). Use only when explicitly invoked via
  /merge-worktree-to-main, or when the user asks to merge a worktree back to
  main.
disable-model-invocation: true
---

# Merge worktree to main

Land the current worktree (or a named one) on `main` with a merge commit.
Invoking this skill is permission to commit leftover worktree changes, rebase
the source branch onto `main`, resolve conflicts whose correct outcome is
clear, and create the merge commit. Do not push. Do
not delete the worktree. Do not interactive-rebase, squash, fast-forward the
merge, or amend unless the user already asked in this invocation.

If extra text follows the command, treat it as the source branch, worktree
path, or worktree name.

## 1. Identify source and main

Run these in the current workspace, in parallel:

```sh
git rev-parse --show-toplevel
git rev-parse --abbrev-ref HEAD
git rev-parse --path-format=absolute --git-common-dir
git worktree list --porcelain
git status --porcelain
```

- **Source worktree** defaults to the current checkout when it is not `main`.
- If the current checkout is `main` and the user named a branch or path, use
  that. If they named nothing and exactly one other worktree exists, use it.
  If several exist, list them and stop.
- **Main worktree** is the entry whose branch is `refs/heads/main`. If none,
  the primary worktree is `dirname` of `--git-common-dir`.
- Stop if this is not a git repo, HEAD is detached, or the source branch is
  `main`.

Save `SOURCE_WT`, `SOURCE_BRANCH`, and `MAIN_WT` before changing anything.

## 2. Commit everything in the source worktree

Working directory: `SOURCE_WT`. Request `git_write` for git mutations.

1. `git status --short`, `git diff`, and `git log -8 --oneline` in parallel.
2. If a rebase is already in progress, skip to the rebase step. Do not
   commit conflict state.
3. If the worktree is already clean, skip to the rebase step.
4. Stage remaining work with `git add -A` (respects gitignore; never
   `git add -f`). Do not commit secrets (`.env`, credentials, keys).
5. Commit with a HEREDOC message that matches recent `git log` style:

```sh
git commit -m "$(cat <<'EOF'
Commit message here.

EOF
)"
```

6. Re-run `git status --porcelain` in `SOURCE_WT`. It must be empty. If a
   hook rejected the commit, fix and create a **new** commit; do not amend
   unless the user asked and the amend rules are met.
7. Do not stash as a substitute for committing.

## 3. Rebase onto main in the source worktree

Working directory: `SOURCE_WT`. Conflicts must be resolved here, not on
`main`.

1. If no rebase is in progress, confirm `git status --porcelain` is empty.
2. If a rebase is already in progress with unmerged paths, resolve them
   using the conflict rule below. If all conflicts are already resolved:
   `GIT_EDITOR=true git rebase --continue`.
3. Otherwise rebase onto local `main` (never `-i`):

```sh
git rebase main
```

4. On conflict, inspect each file and both sides. If the correct resolution
   is clear, apply it, `git add` the files, and
   `GIT_EDITOR=true git rebase --continue`. Repeat until the rebase
   finishes or a conflict is not understood. Do not use `-X ours` / `-X
   theirs` as a blanket strategy. `knowledge/log.md` uses Git `merge=union`
   (see `/research/okf/knowledge-log-concurrency.md`): concurrent **new**
   bullets should auto-combine; leftover conflicts or duplicated rewrites
   still need a human look. Collection `index.md` files are ordinary merges.
5. Pause only when the resolution is ambiguous or would guess at intent.
   Then list the remaining files, explain what is unclear, and leave the
   rebase in progress in `SOURCE_WT`. Do not `--abort` unless the user asks.
6. Do not merge into `main` until the rebase finishes. After it succeeds,
   `git merge-base --is-ancestor main HEAD` must succeed and the worktree
   must be clean.

## 4. Merge into main with a merge commit

Working directory: `MAIN_WT`.

1. Confirm `MAIN_WT` is on `main` and `git status --porcelain` is empty.
   If `main` is not checked out there, stop; do not steal another branch.
   If `MAIN_WT` is dirty, stop.
2. If `SOURCE_BRANCH` is already an ancestor of `main`, report that and stop.
3. Merge with `--no-ff` even when a fast-forward is possible. Always pass
   `-m` so git does not open an editor:

```sh
git merge --no-ff "$SOURCE_BRANCH" -m "$(cat <<EOF
Merge branch '$SOURCE_BRANCH'

EOF
)"
```

4. A merge conflict here is unexpected after a successful rebase. Resolve
   it in `MAIN_WT` if the correct resolution is clear, then
   `GIT_EDITOR=true git merge --continue`. Pause only when the resolution
   is ambiguous; then leave the merge in progress. Do not `--abort` unless
   the user asks.
5. Verify with `git status -sb` and `git log -1 --format='%H %P %s'` in
   `MAIN_WT`. The merge commit must have two parents.

Do not `git pull`, `git push`, `git worktree remove`, or delete the branch.

## 5. Hand-off

Report:

- Source worktree path and branch
- Leftover commit created, or that the worktree was already clean
- Rebase onto `main` succeeded, including conflicts resolved in the
  worktree, or that it paused on an ambiguous conflict
- Merge commit SHA on `main` (if the merge ran)
- That the worktree is still present and nothing was pushed
