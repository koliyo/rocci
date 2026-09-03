app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
}

import Cli
import DocsCoverage
import WorkspaceDeps
import pf.Cmd
import pf.Env
import pf.OsStr
import pf.Path
import pf.Stderr
import pf.Stdout

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

path_str = |path|
    match Path.to_str(path) {
        Ok(s) => s
        Err(_) => Path.display(path)
    }

path_filename = |path| {
    match Path.filename(path) {
        Ok(fp) => path_str(fp)
        Err(_) => path_str(path)
    }
}

join_root = |root, rel| Path.utf8("${path_str(root)}/${rel}")

repo_root! = |_| {
    match Env.var_str!(OsStr.utf8("ROCCI_REPO_ROOT")) {
        Ok(p) => Ok(Path.utf8(p))
        Err(_) => Env.cwd!()
    }
}

ends_with_rocdown = |name| {
    bytes = Str.to_utf8(name)
    suf = Str.to_utf8(".rocdown")
    if List.len(bytes) < List.len(suf) {
        Bool.False
    } else {
        var $i = 0.U64
        var $ok = Bool.True
        base = List.len(bytes) - List.len(suf)
        while $ok and $i < List.len(suf) {
            match (List.get(bytes, base + $i), List.get(suf, $i)) {
                (Ok(a), Ok(b)) => {
                    if a != b {
                        $ok = Bool.False
                    }
                }
                _ => {
                    $ok = Bool.False
                }
            }
            $i = $i + 1
        }
        $ok
    }
}

collect_tree! = |root, start_rel| {
    var $queue = [start_rel]
    var $pages = []
    var $qi = 0.U64
    while $qi < List.len($queue) {
        match List.get($queue, $qi) {
            Err(_) => {}
            Ok(rel) => {
                match Path.list!(join_root(root, rel)) {
                    Err(_) => {}
                    Ok(entries) => {
                        var $ei = 0.U64
                        while $ei < List.len(entries) {
                            match List.get(entries, $ei) {
                                Err(_) => {}
                                Ok(entry) => {
                                    name = path_filename(entry)
                                    child_rel = "${rel}/${name}"
                                    match Path.type!(entry) {
                                        Ok(IsDir) => {
                                            $queue = List.concat($queue, [child_rel])
                                        }
                                        Ok(IsFile) => {
                                            if ends_with_rocdown(name) or child_rel == "examples/rocci/apps.toml" {
                                                match Path.read_utf8!(entry) {
                                                    Ok(text) => {
                                                        $pages = List.concat($pages, [{ path: child_rel, text: text }])
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            $ei = $ei + 1
                        }
                    }
                }
            }
        }
        $qi = $qi + 1
    }
    $pages
}

write_errors! = |lines| {
    var $i = 0.U64
    while $i < List.len(lines) {
        match List.get(lines, $i) {
            Ok(line) => Stderr.line!("  ${line}")?
            Err(_) => {}
        }
        $i = $i + 1
    }
    Ok({})
}

run_check_deps! = |_| {
    root = repo_root!({})?
    before = Env.cwd!()?
    Env.set_cwd!(root)?
    output = Cmd.new("cargo").args_str(["metadata", "--format-version", "1", "--no-deps"]).exec_output!()
    Env.set_cwd!(before)?
    match output {
        Ok(out) => {
            match WorkspaceDeps.decode(out.stdout_utf8) {
                Ok(meta) => {
                    result = WorkspaceDeps.check(meta)
                    if List.len(result.errors) == 0 {
                        Stdout.line!("ok: ${result.package_count.to_str()} workspace packages, ${List.len(result.notes).to_str()} allowlisted reverse edges")?
                        Ok({})
                    } else {
                        Stderr.line!("workspace dependency check failed:")?
                        write_errors!(result.errors)?
                        Err(Exit(1))
                    }
                }
                Err(_) => {
                    Stderr.line!("could not parse cargo metadata")?
                    Err(Exit(1))
                }
            }
        }
        Err(NonZeroExitCode({ command: _cmd, exit_code, stdout_utf8_lossy: _out, stderr_utf8_lossy })) => {
            Stderr.write!(stderr_utf8_lossy)?
            Err(Exit(exit_code))
        }
        Err(_) => Err(Exit(1))
    }
}

run_check_docs! = |_| {
    root = repo_root!({})?
    coverage = Path.read_utf8!(join_root(root, "docs/coverage.toml"))?
    queries = Path.read_utf8!(join_root(root, "docs/search-queries.toml"))?
    sessions_path = join_root(root, "docs/first-use-sessions.toml")
    session_errors = match Path.is_file!(sessions_path) {
        Ok(Bool.True) => DocsCoverage.check_sessions_text(Path.read_utf8!(sessions_path)?)
        _ => ["missing docs/first-use-sessions.toml"]
    }
    pages = List.concat(
        List.concat(collect_tree!(root, "docs"), collect_tree!(root, "site")),
        collect_tree!(root, "examples/rocci"),
    )
    errors = List.concat(
        List.concat(DocsCoverage.check_coverage_text(coverage, pages), DocsCoverage.check_queries_text(queries, pages)),
        session_errors,
    )
    if List.len(errors) == 0 {
        Stdout.line!("docs coverage ok")?
        Ok({})
    } else {
        Stderr.line!("docs coverage failed:")?
        write_errors!(errors)?
        Err(Exit(1))
    }
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
        CheckDeps => run_check_deps!({})
        CheckDocs(rest) => {
            if List.len(rest) == 0 {
                run_check_docs!({})
            } else {
                Stderr.write!("usage: rocci-ops check docs\n")?
                Err(Exit(2))
            }
        }
        CheckZed => not_impl!("check zed")
        NotImpl(name) => not_impl!(name)
    }
}
