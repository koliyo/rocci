from __future__ import annotations

import os
import subprocess
from pathlib import Path

H35_DESKTOP_GIT = "https://github.com/koliyo/h35-desktop.git"


def repo_root() -> Path:
    env = os.environ.get("ROCCI_REPO_ROOT")
    if env:
        return Path(env)
    here = Path(__file__).resolve()
    for parent in here.parents:
        if not (parent / "tools" / "rocci-ops").is_dir():
            continue
        if (parent / "Cargo.toml").is_file() or (
            parent / "docker" / "compose.hybrid.yml"
        ).is_file():
            return parent
    raise SystemExit("could not find rocci repository root")


def h35_desktop_dir(root: Path | None = None) -> Path:
    return ((root or repo_root()) / ".." / "h35-desktop").resolve()


def ensure_h35_desktop(root: Path | None = None) -> Path:
    dest = h35_desktop_dir(root)
    if (dest / "Cargo.toml").is_file():
        return dest
    dest.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["git", "clone", "--depth", "1", H35_DESKTOP_GIT, str(dest)],
        check=True,
    )
    return dest
