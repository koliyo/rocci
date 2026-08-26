from __future__ import annotations

import sys

from rocci_ops import ci, deploy, docs_coverage, local, origin, pr_checkout, release, workspace_deps

USAGE = """\
usage: rocci-ops <command> [args...]

commands:
  check         deps | docs | zed
  ci            run GitHub Actions validation jobs on this machine
  release       package binaries, wait for CI, or publish a GitHub release
  deploy        probe, bootstrap, or push origin artifacts over SSH
  origin        publish, up, or backup on the origin host
  install       cli | vscode | cursor
  package       package vscode, zed, or the rocci.dev site (docs + live apps)
  site          stage generated examples, check, test, and build rocci.dev
  bundle        macOS app bundles (`macos` for Rocci; `okf` is retired, use okmate)
  build-playground
  render-brand-icons
  serve         docker compose helpers (hybrid, static, site, app)
  push-worktrees
  pr-checkout   list open PRs, or checkout one here as pr/<branch>
  promote       staging | production | tag
"""

CHECK_USAGE = """\
usage: rocci-ops check deps|docs|zed [args...]

subcommands:
  deps    check workspace package edges against the product boundary
  docs    check coverage.toml, search-queries.toml, and first-use-sessions.toml
  zed     check Zed manifest and build the language server WASM
"""

LOCAL_COMMANDS = {
    "install",
    "package",
    "site",
    "bundle",
    "build-playground",
    "render-brand-icons",
    "serve",
    "push-worktrees",
    "promote",
}


def main(argv: list[str] | None = None) -> None:
    args = sys.argv[1:] if argv is None else argv
    if not args or args[0] in ("-h", "--help"):
        sys.stdout.write(USAGE)
        if not args:
            raise SystemExit(2)
        raise SystemExit(0)
    command, rest = args[0], args[1:]
    if command == "check":
        raise SystemExit(check_main(rest))
    if command == "ci":
        raise SystemExit(ci.main(rest))
    if command == "release":
        raise SystemExit(release.main(rest))
    if command == "deploy":
        raise SystemExit(deploy.main(rest))
    if command == "origin":
        raise SystemExit(origin.main(rest))
    if command == "pr-checkout":
        raise SystemExit(pr_checkout.main(rest))
    if command in LOCAL_COMMANDS:
        raise SystemExit(local.main([command, *rest]))
    sys.stderr.write(f"unknown command: {command}\n")
    sys.stderr.write(USAGE)
    raise SystemExit(2)


def check_main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        sys.stdout.write(CHECK_USAGE)
        return 0 if argv else 2
    sub, rest = argv[0], argv[1:]
    if sub == "deps":
        if rest:
            sys.stderr.write("usage: rocci-ops check deps\n")
            return 2
        return workspace_deps.main()
    if sub == "docs":
        return docs_coverage.main(rest)
    if sub == "zed":
        if rest:
            sys.stderr.write("usage: rocci-ops check zed\n")
            return 2
        return local.verify_zed()
    sys.stderr.write(f"unknown check subcommand: {sub}\n")
    sys.stderr.write(CHECK_USAGE)
    return 2
