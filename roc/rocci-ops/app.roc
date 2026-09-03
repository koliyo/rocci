app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
}

import Cli
import Ci
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

bytes_range = |bytes, start, end| {
    var $i = start
    var $out = []
    while $i < end {
        match List.get(bytes, $i) {
            Ok(b) => { $out = List.concat($out, [b]) }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out
}

parent_of = |rel| {
    bytes = Str.to_utf8(rel)
    var $i = List.len(bytes)
    var $found = Bool.False
    while !$found and $i > 0 {
        $i = $i - 1
        match List.get(bytes, $i) {
            Ok(47) => { $found = Bool.True }
            _ => {}
        }
    }
    if $found {
        Str.from_utf8_lossy(bytes_range(bytes, 0, $i))
    } else {
        ""
    }
}

okmate_dir_str! = |root| {
    match Env.var_str!(OsStr.utf8("OKMATE_DIR")) {
        Ok(p) => Ok(p)
        Err(_) => {
            sibling = "${path_str(root)}/../okmate/Cargo.toml"
            match Path.is_file!(Path.utf8(sibling)) {
                Ok(is_file) => {
                    if is_file {
                        Ok("${path_str(root)}/../okmate")
                    } else {
                        Ok("${path_str(root)}/.okmate-tool")
                    }
                }
                Err(_) => Ok("${path_str(root)}/.okmate-tool")
            }
        }
    }
}

print_job_names! = |_| {
    var $i = 0.U64
    names = Ci.job_names
    while $i < List.len(names) {
        match List.get(names, $i) {
            Ok(name) => Stdout.line!(name)?
            Err(_) => {}
        }
        $i = $i + 1
    }
    Ok({})
}

build_cmd = |step| {
    match List.get(step.argv, 0) {
        Err(_) => Cmd.new_str("true")
        Ok(prog) => {
            var $cmd = Cmd.new_str(prog)
            var $ai = 1.U64
            while $ai < List.len(step.argv) {
                match List.get(step.argv, $ai) {
                    Ok(part) => {
                        $cmd = $cmd.arg_str(part)
                    }
                    Err(_) => {}
                }
                $ai = $ai + 1
            }
            var $ei = 0.U64
            while $ei < List.len(step.extra_env) {
                match List.get(step.extra_env, $ei) {
                    Ok(pair) => {
                        $cmd = $cmd.env_str(pair.key, pair.val)
                    }
                    Err(_) => {}
                }
                $ei = $ei + 1
            }
            $cmd
        }
    }
}

run_step! = |step, root| {
    before = Env.cwd!()?
    target = if step.cwd == "" { root } else { join_root(root, step.cwd) }
    Env.set_cwd!(target)?
    cmd = build_cmd(step)
    code = if step.stdout_path == "" {
        match cmd.exec_exit_code!() {
            Ok(n) => n
            Err(_) => 1.I32
        }
    } else {
        parent = parent_of(step.stdout_path)
        if parent != "" {
            Path.create_all!(join_root(root, parent))?
        }
        dest = join_root(root, step.stdout_path)
        match cmd.exec_output!() {
            Ok(out) => {
                Path.write_utf8!(dest, out.stdout_utf8)?
                0.I32
            }
            Err(NonZeroExitCode({ command: _c, exit_code, stdout_utf8_lossy, stderr_utf8_lossy: _e })) => {
                Path.write_utf8!(dest, stdout_utf8_lossy)?
                exit_code
            }
            Err(_) => 1.I32
        }
    }
    Env.set_cwd!(before)?
    Ok(code)
}

run_job! = |job, root, okmate_dir, rustup| {
    Stdout.line!("==> ${job}")?
    steps = Ci.steps_for(job, okmate_dir, rustup)
    var $i = 0.U64
    var $code = 0.I32
    while $code == 0.I32 and $i < List.len(steps) {
        match List.get(steps, $i) {
            Err(_) => {
                $i = List.len(steps)
            }
            Ok(step) => {
                joined = List.fold(step.argv, "", |acc, part| if acc == "" { part } else { "${acc} ${part}" })
                Stdout.line!("+ ${joined}")?
                $code = run_step!(step, root)?
                if $code != 0.I32 and step.stdout_path != "" {
                    match Path.read_utf8!(join_root(root, step.stdout_path)) {
                        Ok(text) => Stdout.write!(text)?
                        Err(_) => {}
                    }
                }
            }
        }
        $i = $i + 1
    }
    Ok($code)
}

run_ci! = |args| {
    match Ci.parse(args) {
        ListJobs => {
            print_job_names!({})?
            Ok({})
        }
        CiBad("help") => {
            Stdout.write!("usage: rocci-ops ci [-h] [-k] [-l] [jobs ...]\n")?
            Ok({})
        }
        CiBad(job) => {
            Stderr.write!("unknown ci job: ${job}\n")?
            Err(Exit(2))
        }
        RunJobs({ jobs, keep_going }) => {
            root = repo_root!({})?
            okmate_dir = okmate_dir_str!(root)?
            rustup = Cmd.check_available!("rustup")
            var $i = 0.U64
            var $failed = Bool.False
            var $code = 0.I32
            while $i < List.len(jobs) {
                match List.get(jobs, $i) {
                    Err(_) => {
                        $i = List.len(jobs)
                    }
                    Ok(job) => {
                        job_code = run_job!(job, root, okmate_dir, rustup)?
                        if job_code != 0.I32 {
                            $failed = Bool.True
                            $code = job_code
                            if !keep_going {
                                $i = List.len(jobs)
                            }
                        }
                    }
                }
                $i = $i + 1
            }
            if $failed {
                Err(Exit($code))
            } else {
                Ok({})
            }
        }
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
        CiArgs(rest) => run_ci!(rest)
        NotImpl(name) => not_impl!(name)
    }
}
