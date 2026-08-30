from __future__ import annotations

import os
import subprocess
import tempfile
import time
from pathlib import Path

from rocci_ops.ghutil import DEFAULT_CHECKS, gh_run, wait_for_check
from rocci_ops.paths import repo_root
from rocci_ops.version import (
    BUMP_LEVELS,
    CARGO_LOCK,
    CARGO_TOML,
    apply_release_version,
    crate_versions,
    first_package_version,
    next_release_version,
    parse_release_version,
    release_files_match,
    workspace_crate_names,
)

RELEASE_USAGE = (
    "usage: rocci-ops release <patch|minor|major|vX.Y.Z|dev> "
    "[--from BRANCH] [--force] [--dry-run]"
)


def run(argv: list[str], *, cwd: Path | None = None) -> None:
    subprocess.run(argv, cwd=cwd or repo_root(), check=True)


def git_capture(argv: list[str], *, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        argv,
        cwd=cwd or repo_root(),
        capture_output=True,
        text=True,
    )


def github_repo() -> str:
    result = subprocess.run(
        ["gh", "repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"],
        cwd=repo_root(),
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def dispatch_hosted_ci(from_ref: str) -> None:
    gh_run(["workflow", "run", "ci.yml", "--ref", from_ref])


def dispatch_hosted_release(tag: str) -> None:
    gh_run(["workflow", "run", "release.yml", "--ref", tag])


def wait_for_release_ci(sha: str, from_ref: str = "main") -> None:
    if os.environ.get("GITHUB_ACTIONS"):
        print(
            "GITHUB_TOKEN pushes do not start CI; dispatching ci.yml on "
            f"{from_ref}",
            flush=True,
        )
        dispatch_hosted_ci(from_ref)
    repo = github_repo()

    def gh(args: list[str]) -> str:
        return gh_run(args).stdout

    for check in DEFAULT_CHECKS:
        wait_for_check(repo=repo, sha=sha, check=check, gh=gh, sleep=time.sleep)


def push_version_update(version: str, from_ref: str, remote_sha: str) -> str:
    root = repo_root()
    with tempfile.TemporaryDirectory() as tmp:
        worktree = Path(tmp) / "wt"
        run(["git", "worktree", "add", "--detach", str(worktree), remote_sha], cwd=root)
        try:
            if release_files_match(worktree, version):
                return remote_sha
            paths = apply_release_version(worktree, version)
            run(["git", "add", *[str(path) for path in paths]], cwd=worktree)
            status = git_capture(["git", "status", "--porcelain"], cwd=worktree)
            if status.returncode != 0:
                raise SystemExit("release could not read worktree status")
            if not status.stdout.strip():
                return remote_sha
            run(["git", "commit", "-m", f"chore(release): set version {version}"], cwd=worktree)
            run(["git", "push", "origin", f"HEAD:{from_ref}"], cwd=worktree)
            pushed = git_capture(["git", "rev-parse", "HEAD"], cwd=worktree)
            if pushed.returncode != 0:
                raise SystemExit("release could not read version commit")
            return pushed.stdout.strip()
        finally:
            run(["git", "worktree", "remove", "--force", str(worktree)], cwd=root)


def git_show(sha: str, path: Path) -> str:
    shown = git_capture(["git", "show", f"{sha}:{path.as_posix()}"])
    if shown.returncode != 0:
        raise SystemExit(f"release could not read {path} at {sha}")
    return shown.stdout


def crate_versions_at_sha(sha: str) -> str:
    return crate_versions(git_show(sha, CARGO_TOML), git_show(sha, CARGO_LOCK))


def resolve_release_tag(spec: str, sha: str) -> str:
    if spec == "dev":
        return spec
    if spec in BUMP_LEVELS:
        return f"v{next_release_version(crate_versions_at_sha(sha), spec)}"
    parse_release_version(spec)
    return spec


def release_files_match_at_sha(sha: str, version: str) -> bool:
    cargo = git_show(sha, CARGO_TOML)
    lock = git_show(sha, CARGO_LOCK)
    names = workspace_crate_names(cargo)
    if first_package_version(cargo) != version:
        return False
    return all(f'name = "{name}"\nversion = "{version}"' in lock for name in names)


def run_release(
    spec: str,
    from_ref: str = "main",
    *,
    force: bool = False,
    dry_run: bool = False,
) -> int:
    if spec not in ("dev", *BUMP_LEVELS):
        parse_release_version(spec)
    run(
        [
            "git",
            "fetch",
            "origin",
            f"refs/heads/{from_ref}:refs/remotes/origin/{from_ref}",
        ]
    )
    remote_ref = f"origin/{from_ref}"
    verify = git_capture(["git", "rev-parse", "--verify", remote_ref])
    if verify.returncode != 0:
        raise SystemExit(f"release requires {remote_ref}")
    sha = verify.stdout.strip()
    tag = resolve_release_tag(spec, sha)
    print(f"rocci-ops release {tag}", flush=True)
    movable = tag == "dev"
    if dry_run:
        if movable:
            print("dry-run: would move dev", flush=True)
        else:
            matched = release_files_match_at_sha(sha, parse_release_version(tag))
            print(f"dry-run: release files match={str(matched).lower()}", flush=True)
        return 0
    if not movable:
        sha = push_version_update(parse_release_version(tag), from_ref, sha)
    wait_for_release_ci(sha, from_ref=from_ref)
    tag_argv = ["git", "tag", "-a", tag, "-m", tag, sha]
    push_argv = ["git", "push", "origin", tag]
    if movable or force:
        tag_argv = ["git", "tag", "-a", "-f", tag, "-m", tag, sha]
        push_argv = ["git", "push", "--force", "origin", tag]
    run(tag_argv)
    run(push_argv)
    if os.environ.get("GITHUB_ACTIONS"):
        print(
            "GITHUB_TOKEN tag pushes do not start Release; dispatching "
            f"release.yml on {tag}",
            flush=True,
        )
        dispatch_hosted_release(tag)
    return 0


def parse_release_argv(argv: list[str], usage: str) -> tuple[str, str, bool, bool]:
    from_ref = "main"
    tag: str | None = None
    force = False
    dry_run = False
    i = 0
    while i < len(argv):
        if argv[i] == "--from":
            if i + 1 >= len(argv):
                raise SystemExit(usage)
            from_ref = argv[i + 1]
            i += 2
            continue
        if argv[i] == "--force":
            force = True
            i += 1
            continue
        if argv[i] == "--dry-run":
            dry_run = True
            i += 1
            continue
        if tag is not None:
            raise SystemExit(usage)
        tag = argv[i]
        i += 1
    if tag is None:
        raise SystemExit(usage)
    return tag, from_ref, force, dry_run


def release_command(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        raise SystemExit(RELEASE_USAGE)
    tag, from_ref, force, dry_run = parse_release_argv(argv, RELEASE_USAGE)
    return run_release(tag, from_ref=from_ref, force=force, dry_run=dry_run)
