from __future__ import annotations

import sys

from rocci_ops import ci, release, workspace_deps

USAGE = """\
usage: rocci-ops <command> [args...]

commands:
  check-deps    check workspace package edges against the product boundary
  ci            run GitHub Actions validation jobs on this machine
  release       package binaries, wait for CI, or publish a GitHub release
"""


def main(argv: list[str] | None = None) -> None:
    args = sys.argv[1:] if argv is None else argv
    if not args or args[0] in ("-h", "--help"):
        sys.stdout.write(USAGE)
        if not args:
            raise SystemExit(2)
        raise SystemExit(0)
    command, rest = args[0], args[1:]
    if command == "check-deps":
        raise SystemExit(workspace_deps.main())
    if command == "ci":
        raise SystemExit(ci.main(rest))
    if command == "release":
        raise SystemExit(release.main(rest))
    sys.stderr.write(f"unknown command: {command}\n")
    sys.stderr.write(USAGE)
    raise SystemExit(2)
