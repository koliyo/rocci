import argparse
import subprocess
from pathlib import Path

from rocci_ops.paths import repo_root


def _git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(root), *args], check=check, capture_output=True, text=True)


def parse_worktrees(porcelain: str) -> list[tuple[str, str | None]]:
    entries: list[tuple[str, str | None]] = []
    path = ""
    branch: str | None = None
    for line in porcelain.splitlines():
        if line.startswith("worktree "):
            if path:
                entries.append((path, branch))
            path = line[len("worktree ") :]
            branch = None
        elif line.startswith("branch "):
            branch = line[len("branch ") :]
    if path:
        entries.append((path, branch))
    return entries


def push_worktrees(*, remote: str | None, dry_run: bool) -> int:
    root = repo_root()
    if remote is None:
        up = _git(root, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}", check=False)
        remote = up.stdout.strip().split("/", 1)[0] if up.returncode == 0 and up.stdout.strip() else "origin"
    if _git(root, "remote", "get-url", remote, check=False).returncode != 0:
        raise SystemExit(f"Remote '{remote}' is not configured in {root}")
    listed = _git(root, "worktree", "list", "--porcelain")
    pushed = skipped = 0
    for path, branch_ref in parse_worktrees(listed.stdout):
        if not branch_ref:
            print(f"Skipping {path} (detached HEAD)")
            skipped += 1
            continue
        branch_name = branch_ref.removeprefix("refs/heads/")
        worktree = Path(path)
        if _git(worktree, "rev-parse", "--verify", "HEAD", check=False).returncode != 0:
            print(f"Skipping {path} (no HEAD)")
            skipped += 1
            continue
        up = _git(
            worktree,
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
            check=False,
        )
        if up.returncode == 0 and up.stdout.strip():
            ahead = _git(worktree, "rev-list", "--count", f"{up.stdout.strip()}..HEAD")
            if ahead.stdout.strip() == "0":
                print(f"Skipping {branch_name} ({path}): no commits ahead of {up.stdout.strip()}")
                skipped += 1
                continue
            argv = ["git", "-C", path, "push", remote, f"HEAD:{branch_name}"]
        else:
            argv = ["git", "-C", path, "push", "-u", remote, f"HEAD:{branch_name}"]
        if dry_run:
            print("  " + " ".join(argv))
        else:
            subprocess.run(argv, check=True)
        pushed += 1
    print(f"\nSummary: pushed {pushed}, skipped {skipped}")
    return 0


def push_worktrees_command(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops push-worktrees")
    parser.add_argument("-n", "--dry-run", action="store_true")
    parser.add_argument("-r", "--remote")
    ns = parser.parse_args(argv)
    return push_worktrees(remote=ns.remote, dry_run=ns.dry_run)
