#!/usr/bin/env python3
"""Compatibility wrapper. Prefer `uv run rocci-ops check-deps`."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    return subprocess.call(
        ["uv", "run", "rocci-ops", "check-deps"],
        cwd=ROOT,
    )


if __name__ == "__main__":
    raise SystemExit(main())
