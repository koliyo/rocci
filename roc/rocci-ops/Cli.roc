Cli := [].{
    usage = usage_text
    check_usage = check_usage_text
    parse = do_parse
}

Request : [
    Help,
    NoArgs,
    Unknown(Str),
    CheckHelp,
    CheckNoArgs,
    CheckUnknown(Str),
    CheckDeps,
    CheckDepsUsage,
    CheckDocs(List(Str)),
    CheckZed,
    CheckZedUsage,
    CiArgs(List(Str)),
    BuildArgs(List(Str)),
    InstallArgs(List(Str)),
    PackageArgs(List(Str)),
    ServeArgs(List(Str)),
    SiteArgs(List(Str)),
    ArchiveArgs(List(Str)),
    ReleaseArgs(List(Str)),
    PromoteArgs(List(Str)),
    PrCheckoutArgs(List(Str)),
    PushWorktreesArgs(List(Str)),
    OriginArgs(List(Str)),
    DeployArgs(List(Str)),
    NotImpl(Str),
]

usage_text = "usage: rocci-ops <command> [args...]\n\ncommands:\n  build         cargo release build of rocci, rocdown, and language-server; playground\n  ci            run GitHub Actions validation jobs on this machine\n  check         deps | docs | zed\n  install       cli | vscode | cursor\n  package       macos, vscode, zed, site, icons\n  site          stage generated examples, check, test, and build rocci.dev\n  serve         hybrid, static, site, app\n  deploy        probe, bootstrap, or push origin artifacts over SSH\n  origin        publish, up, or backup on the origin host\n  push-worktrees\n  pr-checkout   list open PRs, or checkout one here as pr/<branch>\n  promote       staging | production\n  release       patch, minor, major, v*, or dev\n  archive       version, package, params, wait-ci, publish\n"

check_usage_text = "usage: rocci-ops check deps|docs|zed [args...]\n\nsubcommands:\n  deps    check workspace package edges against the product boundary\n  docs    check coverage.toml, search-queries.toml, and first-use-sessions.toml\n  zed     check Zed manifest and build the language server WASM\n"

has_text = |hay, needle|
    match Str.split_first(hay, needle) {
        Ok(_) => Bool.True
        Err(_) => Bool.False
    }

do_parse_check = |args|
    match List.get(args, 0) {
        Err(_) => CheckNoArgs
        Ok("-h") => CheckHelp
        Ok("--help") => CheckHelp
        Ok("deps") => {
            if List.len(args) > 1 {
                CheckDepsUsage
            } else {
                CheckDeps
            }
        }
        Ok("docs") => CheckDocs(args.drop_first(1))
        Ok("zed") => {
            if List.len(args) > 1 {
                CheckZedUsage
            } else {
                CheckZed
            }
        }
        Ok(other) => CheckUnknown(other)
    }

do_parse = |args|
    match List.get(args, 0) {
        Err(_) => NoArgs
        Ok("-h") => Help
        Ok("--help") => Help
        Ok("check") => do_parse_check(args.drop_first(1))
        Ok("build") => BuildArgs(args.drop_first(1))
        Ok("ci") => CiArgs(args.drop_first(1))
        Ok("install") => InstallArgs(args.drop_first(1))
        Ok("package") => PackageArgs(args.drop_first(1))
        Ok("site") => SiteArgs(args.drop_first(1))
        Ok("serve") => ServeArgs(args.drop_first(1))
        Ok("deploy") => DeployArgs(args.drop_first(1))
        Ok("origin") => OriginArgs(args.drop_first(1))
        Ok("push-worktrees") => PushWorktreesArgs(args.drop_first(1))
        Ok("pr-checkout") => PrCheckoutArgs(args.drop_first(1))
        Ok("promote") => PromoteArgs(args.drop_first(1))
        Ok("release") => ReleaseArgs(args.drop_first(1))
        Ok("archive") => ArchiveArgs(args.drop_first(1))
        Ok(other) => Unknown(other)
    }

expect has_text(usage_text, "usage: rocci-ops <command> [args...]")
expect has_text(usage_text, "  ci            run GitHub Actions validation jobs on this machine")
expect has_text(usage_text, "  check         deps | docs | zed")
expect has_text(usage_text, "  release       patch, minor, major, v*, or dev")
expect has_text(usage_text, "  archive       version, package, params, wait-ci, publish")
expect has_text(usage_text, "  origin        publish, up, or backup on the origin host")
expect !has_text(usage_text, "verify-zed")
expect has_text(check_usage_text, "usage: rocci-ops check deps|docs|zed [args...]")
expect has_text(check_usage_text, "  deps    check workspace package edges against the product boundary")

expect
    match do_parse([]) {
        NoArgs => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["-h"]) {
        Help => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["--help"]) {
        Help => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["verify-zed"]) {
        Unknown("verify-zed") => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["ci", "--list"]) {
        CiArgs(rest) => {
            match List.get(rest, 0) {
                Ok("--list") => Bool.True
                _ => Bool.False
            }
        }
        _ => Bool.False
    }

expect
    match do_parse(["check"]) {
        CheckNoArgs => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["check", "--help"]) {
        CheckHelp => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["check", "wasm"]) {
        CheckUnknown("wasm") => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["check", "deps"]) {
        CheckDeps => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["check", "deps", "extra"]) {
        CheckDepsUsage => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["check", "zed", "x"]) {
        CheckZedUsage => Bool.True
        _ => Bool.False
    }

expect
    match do_parse(["archive", "version"]) {
        ArchiveArgs(rest) => {
            match List.get(rest, 0) {
                Ok("version") => Bool.True
                _ => Bool.False
            }
        }
        _ => Bool.False
    }

expect
    match do_parse(["release", "dev", "--dry-run"]) {
        ReleaseArgs(rest) => List.len(rest) == 2
        _ => Bool.False
    }

expect
    match do_parse(["pr-checkout", "-n", "39"]) {
        PrCheckoutArgs(rest) => List.len(rest) == 2
        _ => Bool.False
    }

expect
    match do_parse(["origin", "publish", "abc"]) {
        OriginArgs(rest) => List.len(rest) == 2
        _ => Bool.False
    }

expect
    match do_parse(["deploy", "probe"]) {
        DeployArgs(rest) => {
            match List.get(rest, 0) {
                Ok("probe") => Bool.True
                _ => Bool.False
            }
        }
        _ => Bool.False
    }
