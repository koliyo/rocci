app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
}

import Cli
import pf.OsStr
import pf.Stderr
import pf.Stdout

Metadata : {
    workspace_members : List(Str),
    pkgs : List(Pkg),
}

Pkg : {
    name : Str,
    id : Str,
    dependencies : List(Dep),
}

Dep : {
    name : Str,
}

subset_json = "{\"workspace_members\":[\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"beta 0.1.0 (path+file:///tmp/ws/beta)\"],\"packages\":[{\"name\":\"alpha\",\"id\":\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"dependencies\":[{\"name\":\"beta\"},{\"name\":\"serde\"}]},{\"name\":\"beta\",\"id\":\"beta 0.1.0 (path+file:///tmp/ws/beta)\",\"dependencies\":[]}]}"

rename_packages_key = |src| {
    match Str.split_first(src, "\"packages\":") {
        Ok({ before, after }) => Str.concat(before, Str.concat("\"pkgs\":", after))
        Err(_) => src
    }
}

decode_metadata : Str -> Try(Metadata, [InvalidJson(Str), MissingRequiredField(Str), ..])
decode_metadata = |src|
    Encoding.Json.parse(rename_packages_key(src))

os_utf8 = |arg|
    match OsStr.to_raw(arg) {
        Utf8(s) => Ok(s)
        UnixBytes(bytes) => Ok(Str.from_utf8_lossy(bytes))
        _ => Err({})
    }

decode_argv = |args|
    List.fold(
        args.drop_first(1),
        Ok([]),
        |acc, arg| {
            match acc {
                Err(e) => Err(e)
                Ok(strs) => {
                    match os_utf8(arg) {
                        Ok(s) => Ok(List.concat(strs, [s]))
                        Err(_) => Err(Exit(2))
                    }
                }
            }
        },
    )

not_impl! = |name| {
    Stderr.write!("not implemented: ${name}\n")?
    Err(Exit(3))
}

main! = |args| {
    strs = decode_argv(args)?
    match Cli.parse(strs) {
        Help => {
            Stdout.write!(Cli.usage)?
            Ok({})
        }
        NoArgs => {
            Stdout.write!(Cli.usage)?
            Err(Exit(2))
        }
        Unknown(cmd) => {
            Stderr.write!("unknown command: ${cmd}\n")?
            Stderr.write!(Cli.usage)?
            Err(Exit(2))
        }
        CheckHelp => {
            Stdout.write!(Cli.check_usage)?
            Ok({})
        }
        CheckNoArgs => {
            Stdout.write!(Cli.check_usage)?
            Err(Exit(2))
        }
        CheckUnknown(cmd) => {
            Stderr.write!("unknown check subcommand: ${cmd}\n")?
            Stderr.write!(Cli.check_usage)?
            Err(Exit(2))
        }
        CheckDepsUsage => {
            Stderr.write!("usage: rocci-ops check deps\n")?
            Err(Exit(2))
        }
        CheckZedUsage => {
            Stderr.write!("usage: rocci-ops check zed\n")?
            Err(Exit(2))
        }
        CheckDeps => not_impl!("check deps")
        CheckDocs(_) => not_impl!("check docs")
        CheckZed => not_impl!("check zed")
        NotImpl(name) => not_impl!(name)
    }
}

expect
    match decode_metadata(subset_json) {
        Ok(meta) => List.len(meta.pkgs) == 2 and List.len(meta.workspace_members) == 2
        Err(_) => Bool.False
    }

expect
    match decode_metadata(subset_json) {
        Ok(meta) => {
            match List.get(meta.pkgs, 0) {
                Ok(pkg) => pkg.name == "alpha" and List.len(pkg.dependencies) == 2
                Err(_) => Bool.False
            }
        }
        Err(_) => Bool.False
    }

expect
    match decode_metadata("{\"workspace_members\":[],\"packages\":[],\"version\":1}") {
        Ok(meta) => List.len(meta.pkgs) == 0 and List.len(meta.workspace_members) == 0
        Err(_) => Bool.False
    }

expect
    match decode_metadata("{\"workspace_members\":[\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\"],\"packages\":[{\"name\":\"alpha\",\"id\":\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"version\":\"0.1.0\",\"source\":null,\"dependencies\":[{\"name\":\"beta\",\"source\":null}]}]}") {
        Ok(meta) => {
            match List.get(meta.pkgs, 0) {
                Ok(pkg) => pkg.name == "alpha" and List.len(pkg.dependencies) == 1
                Err(_) => Bool.False
            }
        }
        Err(_) => Bool.False
    }
