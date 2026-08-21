from __future__ import annotations

import sys

from rocci_ops import workspace_deps

USAGE = """\
usage: rocci-ops <command> [args...]

commands:
  check-deps    check workspace package edges against the product boundary
"""


def main(argv: list[str] | None = None) -> None:
    args = sys.argv[1:] if argv is None else argv
    if not args or args[0] in ("-h", "--help"):
        sys.stdout.write(USAGE)
        if not args:
            raise SystemExit(2)
        raise SystemExit(0)
    command = args[0]
    if command == "check-deps":
        raise SystemExit(workspace_deps.main())
    sys.stderr.write(f"unknown command: {command}\n")
    sys.stderr.write(USAGE)
    raise SystemExit(2)
