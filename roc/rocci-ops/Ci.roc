Ci := [].{
    job_names = job_names_list
    steps_for = do_steps_for
    parse = do_parse_ci
}

Step : {
    argv : List(Str),
    cwd : Str,
    stdout_path : Str,
    extra_env : List({ key : Str, val : Str }),
}

CiReq : [ListJobs, RunJobs({ jobs : List(Str), keep_going : Bool }), CiBad(Str)]

job_names_list = [
    "lint",
    "test",
    "fixtures-and-docs",
    "editors",
    "knowledge",
    "roc",
]

list_has = |xs, needle|
    List.fold(xs, Bool.False, |acc, x| if acc { Bool.True } else { x == needle })

plain = |argv| { argv: argv, cwd: "", stdout_path: "", extra_env: [] }

with_cwd = |argv, cwd| { argv: argv, cwd: cwd, stdout_path: "", extra_env: [] }

with_out = |argv, stdout_path| { argv: argv, cwd: "", stdout_path: stdout_path, extra_env: [] }

with_env = |argv, extra_env| { argv: argv, cwd: "", stdout_path: "", extra_env: extra_env }

okmate_cmd = |okmate_dir, args|
    List.concat(
        [
            "cargo",
            "run",
            "-q",
            "--no-default-features",
            "--manifest-path",
            "${okmate_dir}/Cargo.toml",
            "-p",
            "okmate",
            "--",
        ],
        args,
    )

lint_steps = |rustup| {
    steps = [
        plain(["uv", "run", "--no-dev", "rocci-ops", "check", "deps"]),
        plain(["cargo", "run", "-q", "-p", "rocci-ungram", "--", "check"]),
        plain(["cargo", "fmt", "--all", "--", "--check"]),
        plain(["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
        with_cwd(["uv", "run", "--group", "dev", "pytest"], "rocci-ops"),
    ]
    if rustup {
        List.concat([plain(["rustup", "component", "add", "rustfmt", "clippy"])], steps)
    } else {
        steps
    }
}

editors_steps = |rustup| {
    rest = [
        plain(["cargo", "build", "-p", "rocci-rocdown-lsp"]),
        plain(["npm", "--prefix", "editors/vscode", "ci"]),
        plain(["npm", "--prefix", "editors/vscode", "run", "lint"]),
        plain(["npm", "--prefix", "editors/vscode", "run", "compile"]),
        plain(["npm", "--prefix", "editors/vscode", "run", "vscode:prepublish"]),
        plain(["npm", "--prefix", "editors/vscode", "test"]),
        plain(["cargo", "check", "--manifest-path", "editors/zed/Cargo.toml", "--target", "wasm32-wasip1"]),
        plain(["cargo", "check", "--manifest-path", "editors/zed/Cargo.toml"]),
        plain(["uv", "run", "--no-dev", "rocci-ops", "check", "zed"]),
    ]
    if rustup {
        List.concat([plain(["rustup", "target", "add", "wasm32-wasip1", "wasm32-wasip2"])], rest)
    } else {
        rest
    }
}

knowledge_steps = |okmate_dir| {
    out = "target/knowledge-ci"
    [
        plain(["mkdir", "-p", out]),
        with_out(okmate_cmd(okmate_dir, ["check", "knowledge", "--profile", "base", "--format", "json"]), "${out}/validation.json"),
        with_out(okmate_cmd(okmate_dir, ["inspect", "--profile", "base", "graph", "knowledge"]), "${out}/graph.json"),
        with_out(okmate_cmd(okmate_dir, ["benchmark", "knowledge/retrieval-benchmark.toml", "knowledge", "--profile", "base"]), "${out}/retrieval.json"),
        plain(okmate_cmd(okmate_dir, ["build", "knowledge", "--output", "${out}/build-a", "--profile", "base"])),
        plain(okmate_cmd(okmate_dir, ["build", "knowledge", "--output", "${out}/build-b", "--profile", "base"])),
        plain(["diff", "-qr", "-x", "*.html", "${out}/build-a", "${out}/build-b"]),
    ]
}

do_steps_for = |job, okmate_dir, rustup| {
    match job {
        "lint" => lint_steps(rustup)
        "test" => [
            plain(["cargo", "test", "--workspace"]),
            plain(["cargo", "test", "--workspace", "--doc"]),
        ]
        "fixtures-and-docs" => [
            plain(["uv", "run", "--no-dev", "rocci-ops", "check", "docs"]),
            plain(["cargo", "run", "-q", "-p", "rocci-docs", "--", "--catalog", "examples/rocci/apps.toml", "--output", "dist/example-docs"]),
            plain(["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "check", "site"]),
            plain(["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", "check", "docs"]),
        ]
        "editors" => editors_steps(rustup)
        "knowledge" => knowledge_steps(okmate_dir)
        "roc" => [
            plain(["sudo", "./docker/install-roc.sh"]),
            with_env(
                ["cargo", "test", "-p", "rocci-cli", "-p", "rocci-rocdown", "-p", "rocci-rocdown-cli"],
                [{ key: "ROCCI_REQUIRE_ROC", val: "1" }],
            ),
        ]
        _ => []
    }
}

do_parse_ci = |args| {
    var $i = 0.U64
    var $list = Bool.False
    var $keep = Bool.False
    var $jobs = []
    var $bad = ""
    while $bad == "" and $i < List.len(args) {
        match List.get(args, $i) {
            Err(_) => {
                $i = List.len(args)
            }
            Ok("-h") => { $bad = "help" }
            Ok("--help") => { $bad = "help" }
            Ok("-l") => { $list = Bool.True }
            Ok("--list") => { $list = Bool.True }
            Ok("-k") => { $keep = Bool.True }
            Ok("--keep-going") => { $keep = Bool.True }
            Ok(job) => {
                if list_has(job_names_list, job) {
                    $jobs = List.concat($jobs, [job])
                } else {
                    $bad = job
                }
            }
        }
        $i = $i + 1
    }
    if $bad == "help" {
        CiBad("help")
    } else if $bad != "" {
        CiBad($bad)
    } else if $list {
        ListJobs
    } else if List.len($jobs) == 0 {
        RunJobs({ jobs: job_names_list, keep_going: $keep })
    } else {
        RunJobs({ jobs: $jobs, keep_going: $keep })
    }
}

step_argvs = |steps|
    List.fold(steps, [], |acc, step| List.concat(acc, [step.argv]))

argv_has = |argv, needle| list_has(argv, needle)

any_argv_has = |steps, needle|
    List.fold(steps, Bool.False, |acc, step| if acc { Bool.True } else { argv_has(step.argv, needle) })

any_tail2 = |steps, a, b|
    List.fold(
        steps,
        Bool.False,
        |acc, step| {
            if acc {
                Bool.True
            } else if List.len(step.argv) >= 2 {
                match (List.get(step.argv, List.len(step.argv) - 2), List.get(step.argv, List.len(step.argv) - 1)) {
                    (Ok(x), Ok(y)) => x == a and y == b
                    _ => Bool.False
                }
            } else {
                Bool.False
            }
        },
    )

stdout_paths = |steps|
    List.fold(
        steps,
        [],
        |acc, step| if step.stdout_path == "" { acc } else { List.concat(acc, [step.stdout_path]) },
    )

expect List.len(job_names_list) == 6
expect
    match List.get(job_names_list, 0) {
        Ok("lint") => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_ci(["--list"]) {
        ListJobs => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_ci(["lint", "test"]) {
        RunJobs({ jobs, keep_going }) => List.len(jobs) == 2 and keep_going == Bool.False
        _ => Bool.False
    }

expect
    match stdout_paths(do_steps_for("knowledge", "/tmp/okmate", Bool.False)) {
        paths => list_has(paths, "target/knowledge-ci/validation.json")
            and list_has(paths, "target/knowledge-ci/graph.json")
            and list_has(paths, "target/knowledge-ci/retrieval.json")
    }

expect any_argv_has(do_steps_for("knowledge", "/tmp/okmate", Bool.False), "okmate")
expect !any_argv_has(do_steps_for("knowledge", "/tmp/okmate", Bool.False), "rocci-okf")
expect any_tail2(do_steps_for("fixtures-and-docs", "/tmp/okmate", Bool.False), "check", "site")
expect any_tail2(do_steps_for("fixtures-and-docs", "/tmp/okmate", Bool.False), "check", "docs")
expect any_argv_has(do_steps_for("fixtures-and-docs", "/tmp/okmate", Bool.False), "rocci-docs")
expect any_tail2(do_steps_for("editors", "/tmp/okmate", Bool.False), "check", "zed")
expect any_tail2(do_steps_for("lint", "/tmp/okmate", Bool.False), "check", "deps")

expect
    match do_steps_for("lint", "/tmp/okmate", Bool.False) {
        steps => {
            pytest = List.fold(
                steps,
                "",
                |acc, step| {
                    if list_has(step.argv, "pytest") {
                        step.cwd
                    } else {
                        acc
                    }
                },
            )
            pytest == "rocci-ops"
        }
    }

expect
    match do_steps_for("roc", "/tmp/okmate", Bool.False) {
        steps => {
            List.fold(
                steps,
                Bool.False,
                |acc, step| {
                    if acc {
                        Bool.True
                    } else {
                        List.fold(
                            step.extra_env,
                            Bool.False,
                            |ok, pair| if ok { Bool.True } else { pair.key == "ROCCI_REQUIRE_ROC" and pair.val == "1" },
                        )
                    }
                },
            )
        }
    }
