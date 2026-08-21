from __future__ import annotations

import sys

from rocci_ops import ci, deploy, docs_coverage, local, origin, release, workspace_deps

USAGE = """\
usage: rocci-ops <command> [args...]

commands:
  check-deps    check workspace package edges against the product boundary
  check-docs    check coverage.toml, search-queries.toml, and first-use-sessions.toml
  ci            run GitHub Actions validation jobs on this machine
  release       package binaries, wait for CI, or publish a GitHub release
  deploy        probe, bootstrap, or push origin artifacts over SSH
  origin        publish, up, or backup on the origin host
  install-cli   build and install rocci, rocdown, and rocci-okf
  package       package vscode, zed, or the rocci.dev site (docs + live apps)
  verify-zed    check Zed manifest and build the language server WASM
  bundle        macOS app bundles
  install-cursor-extension
  build-playground
  render-brand-icons
  serve         docker compose helpers (hybrid, static, site, app)
  push-worktrees
"""

LOCAL_COMMANDS = {
    "install-cli",
    "package",
    "verify-zed",
    "bundle",
    "install-cursor-extension",
    "build-playground",
    "render-brand-icons",
    "serve",
    "push-worktrees",
}


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
    if command == "check-docs":
        raise SystemExit(docs_coverage.main(rest))
    if command == "ci":
        raise SystemExit(ci.main(rest))
    if command == "release":
        raise SystemExit(release.main(rest))
    if command == "deploy":
        raise SystemExit(deploy.main(rest))
    if command == "origin":
        raise SystemExit(origin.main(rest))
    if command in LOCAL_COMMANDS:
        raise SystemExit(local.main([command, *rest]))
    sys.stderr.write(f"unknown command: {command}\n")
    sys.stderr.write(USAGE)
    raise SystemExit(2)
