from __future__ import annotations

import os
from pathlib import Path


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
