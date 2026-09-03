Local := [].{
    build_usage = build_usage_text
    install_usage = install_usage_text
    package_usage = package_usage_text
    serve_usage = serve_usage_text
    site_usage = site_usage_text
    parse_build = do_parse_build
    parse_install = do_parse_install
    parse_package = do_parse_package
    parse_serve = do_parse_serve
    darwin_ok = do_darwin_ok
    build_release_argv = build_release_argv_list
}

build_usage_text = "usage: rocci-ops build [playground]"
install_usage_text = "usage: rocci-ops install cli|vscode|cursor"
package_usage_text = "usage: rocci-ops package macos|vscode|zed|site|icons"
serve_usage_text = "usage: rocci-ops serve hybrid|static|site|app ..."
site_usage_text = "usage: rocci-ops site"

LocalReq : [
    BuildUsage,
    BuildRelease,
    BuildPlayground,
    InstallUsage,
    InstallCli,
    InstallVscode,
    InstallCursor,
    PackageUsage,
    PackageMacos,
    PackageVscode,
    PackageZed,
    PackageIcons,
    PackageSite(Str),
    PackageOkf,
    ServeUsage,
    ServeHybrid({ dist : Str, bin : Str, extra : List(Str) }),
    ServeStatic({ dist : Str, extra : List(Str) }),
    ServeSitePath({ site : Str, extra : List(Str) }),
    ServeApp({ dir : Str, extra : List(Str) }),
    ServeHybridUsage,
]

do_darwin_ok = |os_name| os_name == "macos"

do_parse_build = |args|
    match List.get(args, 0) {
        Err(_) => BuildRelease
        Ok("-h") => BuildUsage
        Ok("--help") => BuildUsage
        Ok("playground") => {
            if List.len(args) == 1 {
                BuildPlayground
            } else {
                BuildUsage
            }
        }
        Ok(_) => BuildUsage
    }

do_parse_install = |args|
    match List.get(args, 0) {
        Err(_) => InstallUsage
        Ok("-h") => InstallUsage
        Ok("--help") => InstallUsage
        Ok(sub) => {
            if List.len(args) > 1 {
                InstallUsage
            } else {
                match sub {
                    "cli" => InstallCli
                    "vscode" => InstallVscode
                    "cursor" => InstallCursor
                    _ => InstallUsage
                }
            }
        }
    }

do_parse_package = |args|
    match List.get(args, 0) {
        Err(_) => PackageUsage
        Ok("-h") => PackageUsage
        Ok("--help") => PackageUsage
        Ok("okf") => PackageOkf
        Ok("macos") => {
            if List.len(args) > 1 { PackageUsage } else { PackageMacos }
        }
        Ok("vscode") => {
            if List.len(args) > 1 { PackageUsage } else { PackageVscode }
        }
        Ok("zed") => {
            if List.len(args) > 1 { PackageUsage } else { PackageZed }
        }
        Ok("icons") => {
            if List.len(args) > 1 { PackageUsage } else { PackageIcons }
        }
        Ok("site") => {
            rest = args.drop_first(1)
            match List.get(rest, 0) {
                Ok("--target") => {
                    match List.get(rest, 1) {
                        Ok(target) => {
                            if List.len(rest) == 2 {
                                PackageSite(target)
                            } else {
                                PackageUsage
                            }
                        }
                        Err(_) => PackageUsage
                    }
                }
                Err(_) => PackageSite("x64musl")
                Ok(_) => PackageUsage
            }
        }
        Ok(_) => PackageUsage
    }

do_parse_serve = |args|
    match List.get(args, 0) {
        Err(_) => ServeUsage
        Ok("-h") => ServeUsage
        Ok("--help") => ServeUsage
        Ok("hybrid") => {
            match (List.get(args, 1), List.get(args, 2)) {
                (Ok(dist), Ok(bin)) => ServeHybrid({ dist: dist, bin: bin, extra: args.drop_first(3) })
                _ => ServeHybridUsage
            }
        }
        Ok("static") => {
            match List.get(args, 1) {
                Ok(dist) => ServeStatic({ dist: dist, extra: args.drop_first(2) })
                Err(_) => ServeUsage
            }
        }
        Ok("site") => {
            match List.get(args, 1) {
                Ok(site) => ServeSitePath({ site: site, extra: args.drop_first(2) })
                Err(_) => ServeUsage
            }
        }
        Ok("app") => {
            match List.get(args, 1) {
                Ok(dir) => ServeApp({ dir: dir, extra: args.drop_first(2) })
                Err(_) => ServeUsage
            }
        }
        Ok(_) => ServeUsage
    }

build_release_argv_list = [
    "cargo",
    "build",
    "--release",
    "-p",
    "rocci-cli",
    "-p",
    "rocci-rocdown-cli",
    "-p",
    "rocci-rocdown-lsp",
]

expect
    match do_parse_build([]) {
        BuildRelease => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_build(["-h"]) {
        BuildUsage => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_build(["playground"]) {
        BuildPlayground => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_install([]) {
        InstallUsage => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_install(["cli"]) {
        InstallCli => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_package([]) {
        PackageUsage => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_package(["macos"]) {
        PackageMacos => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_serve([]) {
        ServeUsage => Bool.True
        _ => Bool.False
    }

expect do_darwin_ok("macos")
expect !do_darwin_ok("linux")
