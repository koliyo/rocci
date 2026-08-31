import subprocess

from rocci_ops.paths import repo_root
from rocci_ops.util import run

PROMOTE_USAGE = "usage: rocci-ops promote staging|production"


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
    raise SystemExit(PROMOTE_USAGE)
