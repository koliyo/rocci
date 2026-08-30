from __future__ import annotations

import subprocess
import time

from rocci_ops.paths import repo_root
from rocci_ops.release import DEFAULT_CHECKS, gh_run, wait_for_check
from rocci_ops.util import run

PROMOTE_USAGE = "usage: rocci-ops promote staging|production|tag"
PROMOTE_TAG_USAGE = "usage: rocci-ops promote tag <tag> [--from BRANCH]"


def _git_merge_in_progress() -> bool:
    return (
        subprocess.run(
            ["git", "rev-parse", "-q", "--verify", "MERGE_HEAD"],
            cwd=repo_root(),
            capture_output=True,
        ).returncode
        == 0
    )


def promote_staging() -> int:
    original = subprocess.run(
        ["git", "branch", "--show-current"],
        cwd=repo_root(),
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    if not original:
        raise SystemExit("promote staging requires a named starting branch")

    try:
        run(["git", "fetch", "origin"])
        if original != "staging":
            run(["git", "switch", "staging"])
        run(["git", "merge", "--ff-only", "origin/staging"])
        run(["git", "merge", "origin/main", "-m", "Promote main into staging"])
        run(["git", "push", "origin", "staging"])
    except BaseException:
        if _git_merge_in_progress():
            run(["git", "merge", "--abort"])
        raise
    finally:
        if original != "staging":
            run(["git", "switch", original])
    return 0


def promote_production() -> int:
    run(["git", "fetch", "origin"])
    verify = subprocess.run(
        ["git", "rev-parse", "--verify", "origin/staging"],
        cwd=repo_root(),
        capture_output=True,
        text=True,
    )
    if verify.returncode != 0:
        raise SystemExit("promote production requires origin/staging")
    run(["git", "push", "origin", "origin/staging:refs/heads/production"])
    return 0


def github_repo() -> str:
    result = subprocess.run(
        ["gh", "repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"],
        cwd=repo_root(),
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def wait_for_promote_ci(sha: str) -> None:
    repo = github_repo()

    def gh(args: list[str]) -> str:
        return gh_run(args).stdout

    for check in DEFAULT_CHECKS:
        wait_for_check(repo=repo, sha=sha, check=check, gh=gh, sleep=time.sleep)


def promote_tag(tag: str, from_ref: str = "main") -> int:
    movable = tag == "dev"
    if not movable and (not tag.startswith("v") or len(tag) < 2):
        raise SystemExit("promote tag requires a v* name or the movable dev tag")
    run(["git", "fetch", "origin"])
    remote_ref = f"origin/{from_ref}"
    verify = subprocess.run(
        ["git", "rev-parse", "--verify", remote_ref],
        cwd=repo_root(),
        capture_output=True,
        text=True,
    )
    if verify.returncode != 0:
        raise SystemExit(f"promote tag requires {remote_ref}")
    wait_for_promote_ci(verify.stdout.strip())
    tag_argv = ["git", "tag", "-a", tag, "-m", tag, remote_ref]
    push_argv = ["git", "push", "origin", tag]
    if movable:
        tag_argv = ["git", "tag", "-a", "-f", tag, "-m", tag, remote_ref]
        push_argv = ["git", "push", "--force", "origin", tag]
    run(tag_argv)
    run(push_argv)
    return 0


def promote_tag_command(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        raise SystemExit(PROMOTE_TAG_USAGE)
    from_ref = "main"
    tag: str | None = None
    i = 0
    while i < len(argv):
        if argv[i] == "--from":
            if i + 1 >= len(argv):
                raise SystemExit(PROMOTE_TAG_USAGE)
            from_ref = argv[i + 1]
            i += 2
            continue
        if tag is not None:
            raise SystemExit(PROMOTE_TAG_USAGE)
        tag = argv[i]
        i += 1
    if tag is None:
        raise SystemExit(PROMOTE_TAG_USAGE)
    return promote_tag(tag, from_ref=from_ref)


def promote_command(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        raise SystemExit(PROMOTE_USAGE)
    if argv[0] == "staging":
        if len(argv) != 1:
            raise SystemExit(PROMOTE_USAGE)
        return promote_staging()
    if argv[0] == "production":
        if len(argv) != 1:
            raise SystemExit(PROMOTE_USAGE)
        return promote_production()
    if argv[0] == "tag":
        return promote_tag_command(argv[1:])
    raise SystemExit(PROMOTE_USAGE)
