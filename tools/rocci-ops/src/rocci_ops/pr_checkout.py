from __future__ import annotations

import argparse
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

from rocci_ops.paths import repo_root

_PR_URL = re.compile(
    r"^https?://github\.com/[^/]+/[^/]+/pull/(\d+)(?:/.*)?$",
    re.IGNORECASE,
)
_PR_NUMBER = re.compile(r"^#?(\d+)$")


@dataclass(frozen=True)
class PrRef:
    number: int | None
    branch: str | None

    def label(self) -> str:
        if self.number is not None:
            return f"#{self.number}"
        assert self.branch is not None
        return self.branch


def parse_pr_ref(raw: str) -> PrRef:
    text = raw.strip()
    if not text:
        raise SystemExit("missing PR number, GitHub PR URL, or branch")
    match = _PR_URL.fullmatch(text)
    if match:
        return PrRef(number=int(match.group(1)), branch=None)
    match = _PR_NUMBER.fullmatch(text)
    if match:
        return PrRef(number=int(match.group(1)), branch=None)
    return PrRef(number=None, branch=text.removeprefix("refs/heads/"))


def local_pr_branch(head_branch: str) -> str:
    name = head_branch.strip().removeprefix("refs/heads/")
    if not name:
        raise SystemExit("PR head branch is empty")
    if name == "pr" or name.startswith("pr/"):
        return name
    return f"pr/{name}"


def _git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        check=check,
        capture_output=True,
        text=True,
    )


def _gh_head_ref(root: Path, number: int) -> str:
    listed = subprocess.run(
        ["gh", "pr", "view", str(number), "--json", "headRefName", "-q", ".headRefName"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if listed.returncode != 0:
        err = listed.stderr.strip() or listed.stdout.strip() or "gh pr view failed"
        raise SystemExit(f"could not resolve PR #{number}: {err}")
    name = listed.stdout.strip()
    if not name:
        raise SystemExit(f"could not resolve PR #{number}: empty headRefName")
    return name


def _fetch_tip(root: Path, spec: PrRef) -> str:
    if spec.number is not None:
        ref = f"pull/{spec.number}/head"
    else:
        assert spec.branch is not None
        ref = spec.branch
    fetched = _git(root, "fetch", "origin", ref, check=False)
    if fetched.returncode != 0:
        err = fetched.stderr.strip() or fetched.stdout.strip() or "git fetch failed"
        raise SystemExit(f"could not fetch {ref} from origin: {err}")
    return _git(root, "rev-parse", "--verify", "FETCH_HEAD").stdout.strip()


def _worktree_dirty(root: Path) -> bool:
    return bool(_git(root, "status", "--porcelain").stdout.strip())


def checkout_pr(
    spec: PrRef,
    *,
    root: Path | None = None,
    dry_run: bool = False,
) -> str:
    repo = root or repo_root()
    if spec.number is not None:
        head = _gh_head_ref(repo, spec.number)
    else:
        assert spec.branch is not None
        head = spec.branch.removeprefix("refs/heads/")
    local_branch = local_pr_branch(head)

    if dry_run:
        print(f"{spec.label()} ({head}) -> {local_branch}")
        return local_branch

    if _worktree_dirty(repo):
        raise SystemExit("this worktree has uncommitted changes; commit or stash them first")

    sha = _fetch_tip(repo, spec)
    switched = _git(repo, "switch", "-C", local_branch, sha, check=False)
    if switched.returncode != 0:
        err = switched.stderr.strip() or switched.stdout.strip() or "git switch failed"
        raise SystemExit(err)
    _git(repo, "branch", "--set-upstream-to", f"origin/{head}", check=False)
    print(f"{spec.label()} ({head}) -> {local_branch}")
    return local_branch


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="rocci-ops pr-checkout",
        description=(
            "Fetch a pull request or branch and check it out here as pr/<branch> "
            "so the original branch can stay checked out in an agent worktree."
        ),
    )
    parser.add_argument(
        "ref",
        help="PR number (#39), GitHub pull request URL, or branch (feat/example-source-sidebar)",
    )
    parser.add_argument("-n", "--dry-run", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    ns = parse_args(argv)
    checkout_pr(parse_pr_ref(ns.ref), dry_run=ns.dry_run)
    return 0
