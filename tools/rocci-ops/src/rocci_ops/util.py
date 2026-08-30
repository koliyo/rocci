from __future__ import annotations

import platform
import shlex
import subprocess
import time
from pathlib import Path

from rocci_ops.paths import repo_root


def run(argv: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
    started = time.monotonic()
    print(
        f"[rocci-ops] phase=command status=start command={shlex.join(argv)}",
        flush=True,
    )
    try:
        subprocess.run(argv, cwd=cwd or repo_root(), env=env, check=True)
    except subprocess.CalledProcessError:
        elapsed_ms = int((time.monotonic() - started) * 1000)
        print(f"[rocci-ops] phase=command status=failed elapsed_ms={elapsed_ms}", flush=True)
        raise
    elapsed_ms = int((time.monotonic() - started) * 1000)
    print(f"[rocci-ops] phase=command status=done elapsed_ms={elapsed_ms}", flush=True)


def require_darwin(kind: str) -> None:
    if platform.system() != "Darwin":
        raise SystemExit(f"{kind} can only be built on macOS.")
