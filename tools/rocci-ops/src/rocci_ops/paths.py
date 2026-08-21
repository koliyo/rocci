from __future__ import annotations

from pathlib import Path


def repo_root() -> Path:
    here = Path(__file__).resolve()
    for parent in here.parents:
        if (parent / "Cargo.toml").is_file() and (parent / "tools" / "rocci-ops").is_dir():
            return parent
    raise SystemExit("could not find rocci repository root")
