import sys

from rocci_ops import (
    archive,
    build,
    ci,
    deploy,
    docs_coverage,
    install,
    origin,
    package,
    pr_checkout,
    promote,
    release,
    serve,
    site,
    worktrees,
    workspace_deps,
)

USAGE = """\
usage: rocci-ops <command> [args...]

commands:
  build         cargo release build of rocci, rocdown, and language-server; playground
  ci            run GitHub Actions validation jobs on this machine
  check         deps | docs | zed
  install       cli | vscode | cursor
  package       macos, vscode, zed, site, icons
  site          stage generated examples, check, test, and build rocci.dev
  serve         hybrid, static, site, app
  deploy        probe, bootstrap, or push origin artifacts over SSH
  origin        publish, up, or backup on the origin host
  push-worktrees
  pr-checkout   list open PRs, or checkout one here as pr/<branch>
  promote       staging | production
  release       patch, minor, major, v*, or dev
  archive       version, package, params, wait-ci, publish
"""


CHECK_USAGE = """\
usage: rocci-ops check deps|docs|zed [args...]

subcommands:
  deps    check workspace package edges against the product boundary
  docs    check coverage.toml, search-queries.toml, and first-use-sessions.toml
  zed     check Zed manifest and build the language server WASM
"""


def main(argv: list[str] | None = None) -> None:
    args = sys.argv[1:] if argv is None else argv
    if not args or args[0] in ("-h", "--help"):
        sys.stdout.write(USAGE)
        if not args:
            raise SystemExit(2)
        raise SystemExit(0)
    command, rest = args[0], args[1:]
    if command == "build":
        raise SystemExit(build.build_command(rest))
    if command == "check":
        raise SystemExit(check_main(rest))
    if command == "ci":
        raise SystemExit(ci.main(rest))
    if command == "archive":
        raise SystemExit(archive.main(rest))
    if command == "release":
        raise SystemExit(release.release_command(rest))
    if command == "deploy":
        raise SystemExit(deploy.main(rest))
    if command == "origin":
        raise SystemExit(origin.main(rest))
    if command == "pr-checkout":
        raise SystemExit(pr_checkout.main(rest))
    if command == "install":
        raise SystemExit(install.install_command(rest))
    if command == "package":
        raise SystemExit(package.package_command(rest))
    if command == "site":
        if rest:
            raise SystemExit("usage: rocci-ops site")
        raise SystemExit(site.build_site())
    if command == "serve":
        raise SystemExit(serve.serve_command(rest))
    if command == "push-worktrees":
        raise SystemExit(worktrees.push_worktrees_command(rest))
    if command == "promote":
        raise SystemExit(promote.promote_command(rest))
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
        return package.verify_zed()
    sys.stderr.write(f"unknown check subcommand: {sub}\n")
    sys.stderr.write(CHECK_USAGE)
    return 2
