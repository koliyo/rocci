#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REMOTE=""
REMOTE_EXPLICIT=0
DRY_RUN=0

usage() {
  cat <<'EOF'
Usage: scripts/push-all-worktrees.sh [options]

Push commits from every branch-backed worktree in this repository.

Options:
  -n, --dry-run         print the pushes without running them
  -r, --remote <name>   remote to push to
  -h, --help            show this help
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    -n|--dry-run)
      DRY_RUN=1
      shift
      ;;
    -r|--remote)
      if [ $# -lt 2 ]; then
        echo "Missing remote name for $1" >&2
        exit 2
      fi
      REMOTE="$2"
      REMOTE_EXPLICIT=1
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v git >/dev/null 2>&1; then
  echo "git is required" >&2
  exit 1
fi

default_remote_from_main() {
  local upstream
  if upstream="$(git -C "$ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)"; then
    printf '%s\n' "${upstream%%/*}"
    return 0
  fi
  return 1
}

main_upstream_ref() {
  git -C "$ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null || true
}

if [ "$REMOTE_EXPLICIT" -eq 0 ]; then
  REMOTE="$(default_remote_from_main || true)"
  if [ -z "$REMOTE" ]; then
    REMOTE="origin"
  fi
fi

MAIN_UPSTREAM="$(main_upstream_ref)"

if ! git -C "$ROOT" remote get-url "$REMOTE" >/dev/null 2>&1; then
  echo "Remote '$REMOTE' is not configured in $ROOT" >&2
  exit 1
fi

current_path=""
current_branch=""
pushed=0
skipped=0

print_commit_list() {
  local worktree_path="$1"
  local compare_ref="$2"

  echo "  Commits:"
  if [ -n "$compare_ref" ]; then
    git -C "$worktree_path" log --reverse --oneline "${compare_ref}..HEAD" | while IFS= read -r commit; do
      printf '    %s\n' "$commit"
    done
  else
    git -C "$worktree_path" log --reverse --oneline HEAD | while IFS= read -r commit; do
      printf '    %s\n' "$commit"
    done
  fi
}

push_branch() {
  local worktree_path="$1"
  local branch_ref="$2"
  local branch_name upstream ahead_count dirty_suffix compare_ref compare_label push_args

  branch_name="${branch_ref#refs/heads/}"

  if ! git -C "$worktree_path" rev-parse --verify HEAD >/dev/null 2>&1; then
    echo "Skipping $worktree_path (no HEAD)"
    skipped=$((skipped + 1))
    return
  fi

  dirty_suffix=""
  if [ -n "$(git -C "$worktree_path" status --short)" ]; then
    dirty_suffix=" [dirty]"
  fi

  if upstream="$(git -C "$worktree_path" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null)"; then
    compare_ref="$upstream"
    compare_label="$upstream"
    ahead_count="$(git -C "$worktree_path" rev-list --count "${upstream}..HEAD")"
    if [ "$ahead_count" -eq 0 ]; then
      echo "Skipping ${branch_name} (${worktree_path})${dirty_suffix}: no commits ahead of ${upstream}"
      skipped=$((skipped + 1))
      return
    fi
    push_args=(push "$REMOTE" "HEAD:${branch_name}")
    echo "Pushing ${branch_name} (${worktree_path})${dirty_suffix}: ${ahead_count} commit(s) ahead of ${upstream}"
  elif git -C "$worktree_path" show-ref --verify --quiet "refs/remotes/${REMOTE}/${branch_name}"; then
    compare_ref="${REMOTE}/${branch_name}"
    compare_label="${REMOTE}/${branch_name}"
    ahead_count="$(git -C "$worktree_path" rev-list --count "${compare_ref}..HEAD")"
    if [ "$ahead_count" -eq 0 ]; then
      echo "Skipping ${branch_name} (${worktree_path})${dirty_suffix}: no commits ahead of ${compare_ref}"
      skipped=$((skipped + 1))
      return
    fi
    push_args=(push -u "$REMOTE" "HEAD:${branch_name}")
    echo "Pushing ${branch_name} (${worktree_path})${dirty_suffix}: ${ahead_count} commit(s) ahead of ${compare_ref}"
  else
    compare_ref="$MAIN_UPSTREAM"
    compare_label="${MAIN_UPSTREAM:-full branch history}"
    if [ -n "$compare_ref" ]; then
      ahead_count="$(git -C "$worktree_path" rev-list --count "${compare_ref}..HEAD")"
    else
      ahead_count="$(git -C "$worktree_path" rev-list --count HEAD)"
    fi
    push_args=(push -u "$REMOTE" "HEAD:${branch_name}")
    echo "Pushing ${branch_name} (${worktree_path})${dirty_suffix}: ${ahead_count} commit(s) to push, no upstream configured, using ${REMOTE} from main worktree"
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    printf '  git -C "%s" ' "$worktree_path"
    printf '%q ' "${push_args[@]}"
    printf '\n'
    printf '  Compared against: %s\n' "$compare_label"
    print_commit_list "$worktree_path" "$compare_ref"
  else
    git -C "$worktree_path" "${push_args[@]}"
  fi
  pushed=$((pushed + 1))
}

while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in
    worktree\ *)
      if [ -n "$current_path" ] && [ -n "$current_branch" ]; then
        push_branch "$current_path" "$current_branch"
      elif [ -n "$current_path" ]; then
        echo "Skipping $current_path (detached HEAD)"
        skipped=$((skipped + 1))
      fi
      current_path="${line#worktree }"
      current_branch=""
      ;;
    branch\ *)
      current_branch="${line#branch }"
      ;;
    "")
      ;;
  esac
done < <(git -C "$ROOT" worktree list --porcelain)

if [ -n "$current_path" ] && [ -n "$current_branch" ]; then
  push_branch "$current_path" "$current_branch"
elif [ -n "$current_path" ]; then
  echo "Skipping $current_path (detached HEAD)"
  skipped=$((skipped + 1))
fi

echo
echo "Summary: pushed ${pushed}, skipped ${skipped}"
