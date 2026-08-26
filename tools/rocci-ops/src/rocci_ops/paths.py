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


def ensure_h35_desktop_at(dest: Path) -> Path:
    dest = dest.resolve()
    if (dest / "Cargo.toml").is_file():
        return dest
    dest.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["git", "clone", "--depth", "1", H35_DESKTOP_GIT, str(dest)],
        check=True,
    )
    return dest


def ensure_h35_desktop(root: Path | None = None) -> Path:
    rocci = root or repo_root()
    sibling = ensure_h35_desktop_at(h35_desktop_dir(rocci))
    if env := os.environ.get("OKMATE_DIR"):
        okmate = Path(env).expanduser().resolve()
    else:
        okmate = (rocci / ".." / "okmate").resolve()
        if not (okmate / "Cargo.toml").is_file():
            okmate = (rocci / ".okmate-tool").resolve()
    okmate_sibling = (okmate / ".." / "h35-desktop").resolve()
    if okmate_sibling != sibling and not (okmate_sibling / "Cargo.toml").is_file():
        if okmate_sibling.exists() or okmate_sibling.is_symlink():
            okmate_sibling.unlink()
        okmate_sibling.symlink_to(sibling, target_is_directory=True)
    return sibling
