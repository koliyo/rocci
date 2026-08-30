from __future__ import annotations

import subprocess
import time
from collections.abc import Callable

DEFAULT_CHECKS = (
    "Code Formatting & Lints",
    "Test Workspace (macos-latest)",
    "Test Workspace (ubuntu-latest)",
)


def parse_check_line(result: str) -> tuple[str, str] | None:
    line = result.strip().splitlines()[0] if result.strip() else ""
    if not line:
        return None
    status, _, conclusion = line.partition(" ")
    return status, conclusion or "pending"


def wait_for_check(
    *,
    repo: str,
    sha: str,
    check: str,
    gh: Callable[..., str],
    sleep: Callable[[float], None],
    deadline_s: float | None = None,
) -> None:
    started = time.monotonic()
    print(f"Waiting for: {check}", flush=True)
    while True:
        if deadline_s is not None and time.monotonic() - started > deadline_s:
            raise SystemExit(f"timed out waiting for {check}")
        raw = gh(
            [
                "api",
                f"repos/{repo}/commits/{sha}/check-runs",
                "--jq",
                f'.check_runs[] | select(.name == "{check}") | .status + " " + (.conclusion // "pending")',
            ]
        )
        parsed = parse_check_line(raw)
        if parsed is None:
            print("  Check not found yet, waiting...", flush=True)
        else:
            status, conclusion = parsed
            print(f"  Status: {status}, Conclusion: {conclusion}", flush=True)
            if status == "completed":
                if conclusion == "success":
                    print(f"  {check} passed", flush=True)
                    return
                raise SystemExit(f"{check} failed ({conclusion})")
        sleep(30)


def gh_run(args: list[str], check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(["gh", *args], check=check, capture_output=True, text=True)
