app [main!] {
    pf: platform "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst",
}

import Cli
import Ci
import DocsCoverage
import Git
import Local
import Origin
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

usage_exit! = |msg| {
    Stderr.line!(msg)?
    Err(Exit(1))
}

run_argv! = |root, argv, cwd_rel| {
    step = { argv: argv, cwd: cwd_rel, stdout_path: "", extra_env: [] }
    code = run_step!(step, root)?
    if code == 0.I32 {
        Ok({})
    } else {
        Err(Exit(code))
    }
}

exec_capture! = |root, argv| {
    before = Env.cwd!()?
    Env.set_cwd!(root)?
    cmd = build_cmd({ argv: argv, cwd: "", stdout_path: "", extra_env: [] })
    captured = match cmd.exec_output!() {
        Ok(out) => { code: 0.I32, out: out.stdout_utf8, err: "" }
        Err(NonZeroExitCode({ command: _c, exit_code, stdout_utf8_lossy, stderr_utf8_lossy })) => {
            { code: exit_code, out: stdout_utf8_lossy, err: stderr_utf8_lossy }
        }
        Err(_) => { code: 1.I32, out: "", err: "" }
    }
    Env.set_cwd!(before)?
    Ok(captured)
}

platform_os! = |_| {
    plat = Env.platform!()
    match plat.os {
        MACOS => "macos"
        LINUX => "linux"
        WINDOWS => "windows"
        OTHER(_) => "other"
    }
}

run_build! = |args| {
    match Local.parse_build(args) {
        BuildUsage => usage_exit!(Local.build_usage)
        BuildRelease => {
            root = repo_root!({})?
            run_argv!(root, Local.build_release_argv, "")
        }
        BuildPlayground => {
            root = repo_root!({})?
            Path.create_all!(join_root(root, "playground/dist"))?
            listed = Cmd.new_str("rustup").arg_str("target").arg_str("list").arg_str("--installed").exec_output!()
            need_add = match listed {
                Ok(out) => {
                    match Str.split_first(out.stdout_utf8, "wasm32-unknown-unknown") {
                        Ok({ before: _b, after: _a }) => Bool.False
                        Err(_) => Bool.True
                    }
                }
                Err(_) => Bool.True
            }
            if need_add {
                run_argv!(root, ["rustup", "target", "add", "wasm32-unknown-unknown"], "")?
            }
            run_argv!(root, ["cargo", "build", "-p", "rocci-playground-wasm", "--target", "wasm32-unknown-unknown", "--release"], "")?
            wasm = join_root(root, "target/wasm32-unknown-unknown/release/rocci_playground_wasm.wasm")
            run_argv!(root, ["cp", path_str(wasm), "playground/dist/compiler.wasm"], "")?
            node_modules = join_root(root, "playground/node_modules")
            has_nm = match Path.is_dir!(node_modules) {
                Ok(flag) => flag
                Err(_) => Bool.False
            }
            if !has_nm {
                run_argv!(root, ["npm", "install"], "playground")?
            }
            run_argv!(root, ["node", "build.js"], "playground")
        }
        _ => usage_exit!(Local.build_usage)
    }
}

run_install! = |args| {
    match Local.parse_install(args) {
        InstallUsage => usage_exit!(Local.install_usage)
        InstallCli => not_impl!("install cli")
        InstallVscode => not_impl!("install vscode")
        InstallCursor => not_impl!("install cursor")
        _ => usage_exit!(Local.install_usage)
    }
}

run_package! = |args| {
    match Local.parse_package(args) {
        PackageUsage => usage_exit!(Local.package_usage)
        PackageOkf => usage_exit!("Rocci Knowledge.app is no longer built here; use https://github.com/koliyo/okmate")
        PackageMacos => {
            if Local.darwin_ok(platform_os!({})) {
                root = repo_root!({})?
                run_argv!(root, ["cargo", "run", "-p", "rocci-cli", "--", "bundle", "--config", "rocci.toml"], "")
            } else {
                usage_exit!("The macOS app bundle can only be built on macOS.")
            }
        }
        PackageVscode => {
            root = repo_root!({})?
            run_argv!(root, ["npm", "install"], "editors/vscode")?
            run_argv!(root, ["npm", "run", "vscode:package"], "editors/vscode")
        }
        PackageZed => not_impl!("package zed")
        PackageIcons => not_impl!("package icons")
        PackageSite(_) => not_impl!("package site")
        _ => usage_exit!(Local.package_usage)
    }
}

run_serve! = |args| {
    match Local.parse_serve(args) {
        ServeUsage => usage_exit!(Local.serve_usage)
        ServeHybridUsage => usage_exit!("usage: rocci-ops serve hybrid DIST_DIR ISLANDS_BIN [compose args...]")
        ServeHybrid(_) => not_impl!("serve hybrid")
        ServeStatic(_) => not_impl!("serve static")
        ServeSitePath(_) => not_impl!("serve site")
        ServeApp(_) => not_impl!("serve app")
        _ => usage_exit!(Local.serve_usage)
    }
}

run_site! = |args| {
    if List.len(args) == 0 {
        root = repo_root!({})?
        run_argv!(root, ["cargo", "run", "-q", "-p", "rocci-docs", "--", "--catalog", "examples/rocci/apps.toml", "--output", "dist/example-docs"], "")?
        run_argv!(root, ["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "check", "site"], "")?
        run_argv!(root, ["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "test", "site"], "")?
        run_argv!(root, ["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "build", "site"], "")
    } else {
        usage_exit!(Local.site_usage)
    }
}

env_or_empty! = |name| {
    match Env.var_str!(OsStr.utf8(name)) {
        Ok(v) => v
        Err(_) => ""
    }
}

write_gh_out! = |text| {
    match Env.var_str!(OsStr.utf8("GITHUB_OUTPUT")) {
        Ok(path) => {
            prev = match Path.read_utf8!(Path.utf8(path)) {
                Ok(p) => p
                Err(_) => ""
            }
            Path.write_utf8!(Path.utf8(path), Str.concat(prev, text))?
            Ok({})
        }
        Err(_) => Stdout.write!(text)
    }
}

run_archive! = |args| {
    match Git.parse_archive(args) {
        ArchiveHelp => {
            Stdout.write!(Git.archive_help)?
            Ok({})
        }
        ArchiveUsage => {
            Stderr.write!("usage: rocci-ops archive [-h] {version,package,params,wait-ci,publish} ...\n")?
            Err(Exit(2))
        }
        ArchiveVersion => {
            info = Git.version_from_ref(env_or_empty!("GITHUB_REF_TYPE"), env_or_empty!("GITHUB_REF_NAME"), env_or_empty!("GITHUB_SHA"))
            pre = if info.prerelease { "true" } else { "false" }
            write_gh_out!(Git.github_output([{ key: "version", val: info.version }, { key: "prerelease", val: pre }]))
        }
        ArchiveParams => {
            info = Git.release_params(env_or_empty!("GITHUB_REF_TYPE"), env_or_empty!("GITHUB_REF_NAME"), env_or_empty!("GITHUB_SHA"))
            pre = if info.prerelease { "true" } else { "false" }
            write_gh_out!(Git.github_output([{ key: "tag", val: info.tag }, { key: "name", val: info.name }, { key: "prerelease", val: pre }]))
        }
        ArchiveOther(name) => not_impl!("archive ${name}")
        _ => not_impl!("archive")
    }
}

run_release! = |args| {
    match Git.parse_release(args) {
        RelUsage => usage_exit!(Git.release_usage)
        RelRun({ tag, dry_run, force: _f, from_ref }) => {
            if dry_run {
                root = repo_root!({})?
                run_argv!(root, ["git", "fetch", "origin", "refs/heads/${from_ref}:refs/remotes/origin/${from_ref}"], "")?
                verify = exec_capture!(root, ["git", "rev-parse", "--verify", "origin/${from_ref}"])?
                if verify.code != 0.I32 {
                    usage_exit!("release requires origin/${from_ref}")
                } else {
                    Stdout.line!("rocci-ops release ${tag}")?
                    if tag == "dev" {
                        Stdout.line!("dry-run: would move dev")?
                    } else {
                        Stdout.line!("dry-run: release files match=unchecked")?
                    }
                    Ok({})
                }
            } else {
                not_impl!("release ${tag}")
            }
        }
        _ => usage_exit!(Git.release_usage)
    }
}

abort_merge_if_needed! = |root| {
    merge = exec_capture!(root, ["git", "rev-parse", "-q", "--verify", "MERGE_HEAD"])?
    if merge.code == 0.I32 {
        run_argv!(root, ["git", "merge", "--abort"], "")
    } else {
        Ok({})
    }
}

promote_staging_body! = |root, original| {
    run_argv!(root, ["git", "fetch", "origin"], "")?
    if original != "staging" {
        run_argv!(root, ["git", "switch", "staging"], "")?
    } else {
        Ok({})
    }?
    match run_argv!(root, ["git", "merge", "--ff-only", "origin/staging"], "") {
        Ok(_) => Ok({})
        Err(e) => {
            abort_merge_if_needed!(root)?
            Err(e)
        }
    }?
    match run_argv!(root, ["git", "merge", "origin/main", "-m", "Promote main into staging"], "") {
        Ok(_) => Ok({})
        Err(e) => {
            abort_merge_if_needed!(root)?
            Err(e)
        }
    }?
    match run_argv!(root, ["git", "push", "origin", "staging"], "") {
        Ok(_) => Ok({})
        Err(e) => {
            abort_merge_if_needed!(root)?
            Err(e)
        }
    }
}

run_promote! = |args| {
    match Git.parse_promote(args) {
        PromoteUsage => usage_exit!(Git.promote_usage)
        PromoteStaging => {
            root = repo_root!({})?
            shown = exec_capture!(root, ["git", "branch", "--show-current"])?
            original = Git.strip(shown.out)
            if original == "" {
                usage_exit!("promote staging requires a named starting branch")
            } else {
                body = promote_staging_body!(root, original)
                restored = if original != "staging" {
                    run_argv!(root, ["git", "switch", original], "")
                } else {
                    Ok({})
                }
                match (body, restored) {
                    (Ok(_), Ok(_)) => Ok({})
                    (Err(e), _) => Err(e)
                    (_, Err(e)) => Err(e)
                }
            }
        }
        PromoteProduction => {
            root = repo_root!({})?
            run_argv!(root, ["git", "fetch", "origin"], "")?
            verify = exec_capture!(root, ["git", "rev-parse", "--verify", "origin/staging"])?
            if verify.code != 0.I32 {
                usage_exit!("promote production requires origin/staging")
            } else {
                run_argv!(root, ["git", "push", "origin", "origin/staging:refs/heads/production"], "")
            }
        }
        _ => usage_exit!(Git.promote_usage)
    }
}

gh_head_ref! = |root, number| {
    viewed = exec_capture!(root, ["gh", "pr", "view", number, "--json", "headRefName", "-q", ".headRefName"])?
    name = Git.strip(viewed.out)
    if viewed.code != 0.I32 {
        err = Git.strip(viewed.err)
        msg = if err == "" { Git.strip(viewed.out) } else { err }
        fallback = if msg == "" { "gh pr view failed" } else { msg }
        usage_exit!("could not resolve PR #${number}: ${fallback}")
    } else if name == "" {
        usage_exit!("could not resolve PR #${number}: empty headRefName")
    } else {
        Ok(name)
    }
}

run_pr_checkout! = |args| {
    parsed = Git.parse_pr_argv(args)
    if parsed.help {
        Stdout.write!(Git.pr_help)?
        Ok({})
    } else if parsed.bad {
        Stderr.write!(Git.pr_help)?
        Err(Exit(2))
    } else if parsed.ref == "" {
        root = repo_root!({})?
        run_argv!(root, ["gh", "pr", "list", "--state", "open"], "")
    } else {
        spec = Git.parse_pr_ref(parsed.ref)
        if spec.num == "" and spec.branch == "" {
            usage_exit!("missing PR number, GitHub PR URL, or branch")
        } else {
            root = repo_root!({})?
            head = if spec.num != "" {
                gh_head_ref!(root, spec.num)?
            } else {
                spec.branch
            }
            local = Git.local_pr_branch(head)
            if local == "" {
                usage_exit!("PR head branch is empty")
            } else if parsed.dry {
                Stdout.line!("${Git.pr_label(spec)} (${head}) -> ${local}")?
                Ok({})
            } else {
                dirty = exec_capture!(root, ["git", "status", "--porcelain"])?
                if Git.strip(dirty.out) != "" {
                    usage_exit!("this worktree has uncommitted changes; commit or stash them first")
                } else {
                    ref = if spec.num != "" { "pull/${spec.num}/head" } else { spec.branch }
                    fetched = exec_capture!(root, ["git", "fetch", "origin", ref])?
                    if fetched.code != 0.I32 {
                        err = Git.strip(fetched.err)
                        msg = if err == "" { Git.strip(fetched.out) } else { err }
                        fallback = if msg == "" { "git fetch failed" } else { msg }
                        usage_exit!("could not fetch ${ref} from origin: ${fallback}")
                    } else {
                        sha_cap = exec_capture!(root, ["git", "rev-parse", "--verify", "FETCH_HEAD"])?
                        sha = Git.strip(sha_cap.out)
                        switched = exec_capture!(root, ["git", "switch", "-C", local, sha])?
                        if switched.code != 0.I32 {
                            err = Git.strip(switched.err)
                            msg = if err == "" { Git.strip(switched.out) } else { err }
                            fallback = if msg == "" { "git switch failed" } else { msg }
                            usage_exit!(fallback)
                        } else {
                            _up = exec_capture!(root, ["git", "branch", "--set-upstream-to", "origin/${head}"])?
                            Stdout.line!("${Git.pr_label(spec)} (${head}) -> ${local}")?
                            Ok({})
                        }
                    }
                }
            }
        }
    }
}

drop_refs_heads = |branch_ref| Git.parse_pr_ref(branch_ref).branch

first_slash = |full| {
    bytes = Str.to_utf8(full)
    var $i = 0.U64
    var $found = Bool.False
    while !$found and $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(47) => {
                $found = Bool.True
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    if $found {
        Str.from_utf8_lossy(bytes_range(bytes, 0, $i))
    } else {
        full
    }
}

run_push_worktrees! = |args| {
    parsed = Git.parse_push_argv(args)
    if parsed.help {
        Stdout.write!(Git.push_help)?
        Ok({})
    } else if parsed.bad {
        Stderr.write!(Git.push_help)?
        Err(Exit(2))
    } else {
        root = repo_root!({})?
        remote = if parsed.remote != "" {
            parsed.remote
        } else {
                up = exec_capture!(root, ["git", "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"])?
                if up.code == 0.I32 {
                    first_slash(Git.strip(up.out))
                } else {
                    "origin"
                }
        }
        url = exec_capture!(root, ["git", "remote", "get-url", remote])?
        if url.code != 0.I32 {
            usage_exit!("Remote '${remote}' is not configured in ${path_str(root)}")
        } else {
            listed = exec_capture!(root, ["git", "worktree", "list", "--porcelain"])?
            entries = Git.parse_worktrees(listed.out)
            push_entries!(root, remote, parsed.dry, entries, 0.U64, 0.U64, 0.U64)
        }
    }
}

push_entries! = |root, remote, dry, entries, idx, pushed, skipped| {
    if idx >= List.len(entries) {
        Stdout.line!("")?
        pstr = pushed.to_str()
        sstr = skipped.to_str()
        Stdout.line!("Summary: pushed ${pstr}, skipped ${sstr}")?
        Ok({})
    } else {
        match List.get(entries, idx) {
            Err(_) => Ok({})
            Ok(entry) => {
                if entry.branch == "" {
                    Stdout.line!("Skipping ${entry.path} (detached HEAD)")?
                    push_entries!(root, remote, dry, entries, idx + 1, pushed, skipped + 1)
                } else {
                    branch_name = drop_refs_heads(entry.branch)
                    head_ok = exec_capture!(Path.utf8(entry.path), ["git", "rev-parse", "--verify", "HEAD"])?
                    if head_ok.code != 0.I32 {
                        Stdout.line!("Skipping ${entry.path} (no HEAD)")?
                        push_entries!(root, remote, dry, entries, idx + 1, pushed, skipped + 1)
                    } else {
                        up = exec_capture!(Path.utf8(entry.path), ["git", "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}"])?
                        argv = if up.code == 0.I32 {
                            if Git.strip(up.out) != "" {
                                ahead = exec_capture!(Path.utf8(entry.path), ["git", "rev-list", "--count", "${Git.strip(up.out)}..HEAD"])?
                                if Git.strip(ahead.out) == "0" {
                                    []
                                } else {
                                    ["git", "-C", entry.path, "push", remote, "HEAD:${branch_name}"]
                                }
                            } else {
                                ["git", "-C", entry.path, "push", "-u", remote, "HEAD:${branch_name}"]
                            }
                        } else {
                            ["git", "-C", entry.path, "push", "-u", remote, "HEAD:${branch_name}"]
                        }
                        if List.len(argv) == 0 {
                            Stdout.line!("Skipping ${branch_name} (${entry.path}): no commits ahead of ${Git.strip(up.out)}")?
                            push_entries!(root, remote, dry, entries, idx + 1, pushed, skipped + 1)
                        } else if dry {
                            Stdout.line!("  ${Git.join_argv(argv)}")?
                            push_entries!(root, remote, dry, entries, idx + 1, pushed + 1, skipped)
                        } else {
                            run_argv!(root, argv, "")?
                            push_entries!(root, remote, dry, entries, idx + 1, pushed + 1, skipped)
                        }
                    }
                }
            }
        }
    }
}

lane_env! = |_| {
    keys = ["ROCCI_LANE", "ROCCI_ORIGIN_ROOT", "ROCCI_HTTP_PORT", "COMPOSE_PROJECT_NAME", "ROCCI_PUBLISH_LIVE", "ROCCI_IMAGE_TAG", "ROCCI_BOOTSTRAP_DEST"]
    var $i = 0.U64
    var $pairs = []
    while $i < List.len(keys) {
        match List.get(keys, $i) {
            Ok(key) => {
                match Env.var_str!(OsStr.utf8(key)) {
                    Ok(val) => {
                        $pairs = List.concat($pairs, [{ key: key, val: val }])
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $pairs
}

run_origin! = |args| {
    match Origin.parse_origin(args) {
        OriginHelp => {
            Stdout.write!(Origin.origin_help)?
            Ok({})
        }
        OriginUsage => {
            Stderr.write!(Origin.origin_help)?
            Err(Exit(2))
        }
        OriginPublish(sha) => {
            if Origin.validate_sha(sha) {
                cfg = Origin.resolved_lane(lane_env!({}))
                if cfg.err != "" {
                    usage_exit!(cfg.err)
                } else {
                    not_impl!("origin publish")
                }
            } else {
                usage_exit!("error: SHA must be hex")
            }
        }
        OriginUp(_) => not_impl!("origin up")
        OriginBackup(_) => not_impl!("origin backup")
        _ => not_impl!("origin")
    }
}

run_deploy! = |args| {
    match Origin.parse_deploy(args) {
        DeployHelp => {
            Stdout.write!(Origin.deploy_help)?
            Ok({})
        }
        DeployUsage => {
            Stderr.write!(Origin.deploy_help)?
            Err(Exit(2))
        }
        DeployProbe => not_impl!("deploy probe")
        DeployBootstrap => not_impl!("deploy bootstrap")
        DeployPush({ dir: _d, sha }) => {
            if Origin.validate_sha(sha) {
                cfg = Origin.resolved_lane(lane_env!({}))
                if cfg.err != "" {
                    usage_exit!(cfg.err)
                } else {
                    _cmd = Origin.origin_publish_cmd(sha, cfg.origin_root, cfg)
                    not_impl!("deploy push")
                }
            } else {
                usage_exit!("error: SHA must be hex")
            }
        }
        _ => not_impl!("deploy")
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
        BuildArgs(rest) => run_build!(rest)
        InstallArgs(rest) => run_install!(rest)
        PackageArgs(rest) => run_package!(rest)
        ServeArgs(rest) => run_serve!(rest)
        SiteArgs(rest) => run_site!(rest)
        ArchiveArgs(rest) => run_archive!(rest)
        ReleaseArgs(rest) => run_release!(rest)
        PromoteArgs(rest) => run_promote!(rest)
        PrCheckoutArgs(rest) => run_pr_checkout!(rest)
        PushWorktreesArgs(rest) => run_push_worktrees!(rest)
        OriginArgs(rest) => run_origin!(rest)
        DeployArgs(rest) => run_deploy!(rest)
        NotImpl(name) => not_impl!(name)
    }
}
