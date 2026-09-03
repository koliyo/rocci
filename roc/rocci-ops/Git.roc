Git := [].{
    version_from_ref = do_version_from_ref
    release_params = do_release_params
    archive_stem = do_archive_stem
    github_output = do_github_output
    parse_release = do_parse_release
    parse_archive = do_parse_archive
    parse_promote = do_parse_promote
    parse_check_line = do_parse_check_line
    parse_pr_ref = do_parse_pr_ref
    local_pr_branch = do_local_pr_branch
    pr_label = do_pr_label
    parse_worktrees = do_parse_worktrees
    parse_pr_argv = do_parse_pr_argv
    parse_push_argv = do_parse_push_argv
    strip = trim
    join_argv = do_join_argv
    default_checks = default_check_names
    release_usage = release_usage_text
    promote_usage = promote_usage_text
    pr_help = pr_help_text
    push_help = push_help_text
    archive_help = archive_help_text
}

release_usage_text = "usage: rocci-ops release <patch|minor|major|vX.Y.Z|dev> [--from BRANCH] [--force] [--dry-run]"
promote_usage_text = "usage: rocci-ops promote staging|production"
pr_help_text = "usage: rocci-ops pr-checkout [-h] [-n] [ref]\n"
push_help_text = "usage: rocci-ops push-worktrees [-h] [-n] [-r REMOTE]\n"
archive_help_text = "usage: rocci-ops archive [-h] {version,package,params,wait-ci,publish} ...\n"

default_check_names = [
    "Code Formatting & Lints",
    "Test Workspace (macos-latest)",
    "Test Workspace (ubuntu-latest)",
]

ArchiveReq : [
    ArchiveUsage,
    ArchiveHelp,
    ArchiveVersion,
    ArchiveParams,
    ArchiveOther(Str),
]

ReleaseReq : [
    RelUsage,
    RelRun({ tag : Str, from_ref : Str, force : Bool, dry_run : Bool }),
]

PromoteReq : [PromoteUsage, PromoteStaging, PromoteProduction]

CheckLine : [CheckEmpty, CheckStat({ status : Str, conclusion : Str })]

starts_with = |hay, needle| {
    h = Str.to_utf8(hay)
    n = Str.to_utf8(needle)
    if List.len(h) < List.len(n) {
        Bool.False
    } else {
        var $i = 0.U64
        var $ok = Bool.True
        while $ok and $i < List.len(n) {
            match (List.get(h, $i), List.get(n, $i)) {
                (Ok(a), Ok(b)) => {
                    if a == b {
                        {}
                    } else {
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

bytes_range = |bytes, start, end| {
    var $i = start
    var $out = []
    while $i < end {
        match List.get(bytes, $i) {
            Ok(b) => {
                $out = List.concat($out, [b])
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out
}

from_bytes = |bytes, start, end| Str.from_utf8_lossy(bytes_range(bytes, start, end))

is_ws = |b| b == 32 or b == 9 or b == 10 or b == 13

trim = |s| {
    bytes = Str.to_utf8(s)
    len = List.len(bytes)
    var $a = 0.U64
    var $b = len
    var $go = Bool.True
    while $go and $a < $b {
        match List.get(bytes, $a) {
            Ok(ch) => {
                if is_ws(ch) {
                    $a = $a + 1
                } else {
                    $go = Bool.False
                }
            }
            Err(_) => {
                $go = Bool.False
            }
        }
    }
    $go = Bool.True
    while $go and $b > $a {
        match List.get(bytes, $b - 1) {
            Ok(ch) => {
                if is_ws(ch) {
                    $b = $b - 1
                } else {
                    $go = Bool.False
                }
            }
            Err(_) => {
                $go = Bool.False
            }
        }
    }
    from_bytes(bytes, $a, $b)
}

first_line = |s| {
    bytes = Str.to_utf8(s)
    var $i = 0.U64
    var $found = Bool.False
    while !$found and $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(10) => {
                $found = Bool.True
            }
            _ => {
                $i = $i + 1
            }
        }
    }
    from_bytes(bytes, 0, $i)
}

is_digits = |s| {
    bytes = Str.to_utf8(s)
    if List.len(bytes) == 0 {
        Bool.False
    } else {
        var $i = 0.U64
        var $ok = Bool.True
        while $ok and $i < List.len(bytes) {
            match List.get(bytes, $i) {
                Ok(ch) => {
                    if ch >= 48 and ch <= 57 {
                        {}
                    } else {
                        $ok = Bool.False
                    }
                }
                Err(_) => {
                    $ok = Bool.False
                }
            }
            $i = $i + 1
        }
        $ok
    }
}

find_at = |hay, needle| {
    h = Str.to_utf8(hay)
    n = Str.to_utf8(needle)
    nlen = List.len(n)
    hlen = List.len(h)
    if nlen == 0 or hlen < nlen {
        { hit: Bool.False, at: 0.U64 }
    } else {
        var $i = 0.U64
        var $hit = Bool.False
        var $at = 0.U64
        while !$hit and $i + nlen <= hlen {
            var $j = 0.U64
            var $eq = Bool.True
            while $eq and $j < nlen {
                match (List.get(h, $i + $j), List.get(n, $j)) {
                    (Ok(a), Ok(b)) => {
                        if a == b {
                            {}
                        } else {
                            $eq = Bool.False
                        }
                    }
                    _ => {
                        $eq = Bool.False
                    }
                }
                $j = $j + 1
            }
            if $eq {
                $hit = Bool.True
                $at = $i
            } else {
                $i = $i + 1
            }
        }
        { hit: $hit, at: $at }
    }
}

drop_prefix = |s, prefix| {
    if starts_with(s, prefix) {
        bytes = Str.to_utf8(s)
        from_bytes(bytes, List.len(Str.to_utf8(prefix)), List.len(bytes))
    } else {
        s
    }
}

take_digits = |s| {
    bytes = Str.to_utf8(s)
    var $i = 0.U64
    var $go = Bool.True
    while $go and $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(ch) => {
                if ch >= 48 and ch <= 57 {
                    $i = $i + 1
                } else {
                    $go = Bool.False
                }
            }
            Err(_) => {
                $go = Bool.False
            }
        }
    }
    from_bytes(bytes, 0, $i)
}

pr_url_number = |raw| {
    t = trim(raw)
    http = starts_with(t, "http://") or starts_with(t, "https://")
    host = find_at(t, "github.com/")
    pull = find_at(t, "/pull/")
    if http and host.hit and pull.hit {
        rest = drop_prefix(from_bytes(Str.to_utf8(t), pull.at, List.len(Str.to_utf8(t))), "/pull/")
        num = take_digits(rest)
        after = drop_prefix(rest, num)
        if num == "" {
            ""
        } else {
            if after == "" or starts_with(after, "/") {
                num
            } else {
                ""
            }
        }
    } else {
        ""
    }
}

do_parse_pr_ref = |raw| {
    t = trim(raw)
    url_n = pr_url_number(t)
    if t == "" {
        { num: "", branch: "" }
    } else if url_n != "" {
        { num: url_n, branch: "" }
    } else if starts_with(t, "#") {
        rest = drop_prefix(t, "#")
        if is_digits(rest) {
            { num: rest, branch: "" }
        } else {
            { num: "", branch: t }
        }
    } else if is_digits(t) {
        { num: t, branch: "" }
    } else {
        { num: "", branch: drop_prefix(t, "refs/heads/") }
    }
}

do_local_pr_branch = |head| {
    name = trim(drop_prefix(head, "refs/heads/"))
    if name == "" {
        ""
    } else if name == "pr" {
        name
    } else if starts_with(name, "pr/") {
        name
    } else {
        "pr/${name}"
    }
}

do_pr_label = |spec| {
    if spec.num != "" {
        "#${spec.num}"
    } else {
        spec.branch
    }
}

do_parse_check_line = |result| {
    line = trim(first_line(result))
    if line == "" {
        CheckEmpty
    } else {
        sp = find_at(line, " ")
        if sp.hit {
            status = from_bytes(Str.to_utf8(line), 0, sp.at)
            conclusion = from_bytes(Str.to_utf8(line), sp.at + 1, List.len(Str.to_utf8(line)))
            if conclusion == "" {
                CheckStat({ status: status, conclusion: "pending" })
            } else {
                CheckStat({ status: status, conclusion: conclusion })
            }
        } else {
            CheckStat({ status: line, conclusion: "pending" })
        }
    }
}

line_kind = |line| {
    if starts_with(line, "worktree ") {
        "worktree"
    } else if starts_with(line, "branch ") {
        "branch"
    } else {
        "other"
    }
}

do_parse_worktrees = |porcelain| {
    bytes = Str.to_utf8(porcelain)
    var $i = 0.U64
    var $start = 0.U64
    var $path = ""
    var $branch = ""
    var $entries = []
    while $i <= List.len(bytes) {
        at_end = $i == List.len(bytes)
        is_nl = match List.get(bytes, $i) {
            Ok(10) => Bool.True
            _ => Bool.False
        }
        if at_end or is_nl {
            line = from_bytes(bytes, $start, $i)
            match line_kind(line) {
                "worktree" => {
                    if $path != "" {
                        $entries = List.concat($entries, [{ path: $path, branch: $branch }])
                    }
                    $path = drop_prefix(line, "worktree ")
                    $branch = ""
                }
                "branch" => {
                    $branch = drop_prefix(line, "branch ")
                }
                _ => {}
            }
            $i = $i + 1
            $start = $i
        } else {
            $i = $i + 1
        }
    }
    if $path != "" {
        $entries = List.concat($entries, [{ path: $path, branch: $branch }])
    }
    $entries
}

do_parse_pr_argv = |args| {
    var $i = 0.U64
    var $help = Bool.False
    var $dry = Bool.False
    var $ref = ""
    var $bad = Bool.False
    while !$bad and $i < List.len(args) {
        match List.get(args, $i) {
            Err(_) => {
                $i = List.len(args)
            }
            Ok("-h") => {
                $help = Bool.True
            }
            Ok("--help") => {
                $help = Bool.True
            }
            Ok("-n") => {
                $dry = Bool.True
            }
            Ok("--dry-run") => {
                $dry = Bool.True
            }
            Ok(value) => {
                if starts_with(value, "-") {
                    $bad = Bool.True
                } else if $ref == "" {
                    $ref = value
                } else {
                    $bad = Bool.True
                }
            }
        }
        $i = $i + 1
    }
    { help: $help, dry: $dry, ref: $ref, bad: $bad }
}

do_parse_push_argv = |args| {
    var $i = 0.U64
    var $help = Bool.False
    var $dry = Bool.False
    var $remote = ""
    var $bad = Bool.False
    while !$bad and $i < List.len(args) {
        match List.get(args, $i) {
            Err(_) => {
                $i = List.len(args)
            }
            Ok("-h") => {
                $help = Bool.True
            }
            Ok("--help") => {
                $help = Bool.True
            }
            Ok("-n") => {
                $dry = Bool.True
            }
            Ok("--dry-run") => {
                $dry = Bool.True
            }
            Ok("-r") => {
                $i = $i + 1
                match List.get(args, $i) {
                    Ok(name) => {
                        $remote = name
                    }
                    Err(_) => {
                        $bad = Bool.True
                    }
                }
            }
            Ok("--remote") => {
                $i = $i + 1
                match List.get(args, $i) {
                    Ok(name) => {
                        $remote = name
                    }
                    Err(_) => {
                        $bad = Bool.True
                    }
                }
            }
            Ok(_) => {
                $bad = Bool.True
            }
        }
        $i = $i + 1
    }
    { help: $help, dry: $dry, remote: $remote, bad: $bad }
}

short_sha = |sha| {
    bytes = Str.to_utf8(sha)
    if List.len(bytes) <= 7 {
        sha
    } else {
        from_bytes(bytes, 0, 7)
    }
}

do_version_from_ref = |ref_type, ref_name, sha| {
    if ref_type == "tag" {
        if ref_name == "dev" {
            { version: "dev-${short_sha(sha)}", prerelease: Bool.True }
        } else {
            { version: ref_name, prerelease: Bool.False }
        }
    } else {
        { version: "dev-${short_sha(sha)}", prerelease: Bool.True }
    }
}

do_release_params = |ref_type, ref_name, sha| {
    if ref_type == "tag" {
        if ref_name == "dev" {
            { tag: "dev", name: "Development Build (${short_sha(sha)})", prerelease: Bool.True }
        } else {
            { tag: ref_name, name: ref_name, prerelease: Bool.False }
        }
    } else {
        { tag: "dev", name: "Development Build (${short_sha(sha)})", prerelease: Bool.True }
    }
}

do_join_argv = |argv|
    List.fold(
        argv,
        "",
        |acc, part| {
            if acc == "" {
                part
            } else {
                "${acc} ${part}"
            }
        },
    )

do_archive_stem = |version, target| "rocci-${version}-${target}"

do_github_output = |pairs|
    List.fold(pairs, "", |acc, pair| Str.concat(acc, "${pair.key}=${pair.val}\n"))

do_parse_archive = |args|
    match List.get(args, 0) {
        Err(_) => ArchiveUsage
        Ok("-h") => ArchiveHelp
        Ok("--help") => ArchiveHelp
        Ok("version") => {
            if List.len(args) == 1 {
                ArchiveVersion
            } else {
                ArchiveUsage
            }
        }
        Ok("params") => {
            if List.len(args) == 1 {
                ArchiveParams
            } else {
                ArchiveUsage
            }
        }
        Ok(other) => ArchiveOther(other)
    }

do_parse_promote = |args|
    match List.get(args, 0) {
        Err(_) => PromoteUsage
        Ok("-h") => PromoteUsage
        Ok("--help") => PromoteUsage
        Ok("staging") => {
            if List.len(args) == 1 {
                PromoteStaging
            } else {
                PromoteUsage
            }
        }
        Ok("production") => {
            if List.len(args) == 1 {
                PromoteProduction
            } else {
                PromoteUsage
            }
        }
        Ok(_) => PromoteUsage
    }

do_parse_release = |args| {
    if List.len(args) == 0 {
        RelUsage
    } else {
        match List.get(args, 0) {
            Ok("-h") => RelUsage
            Ok("--help") => RelUsage
            _ => {
                var $i = 0.U64
                var $tag = ""
                var $from_ref = "main"
                var $force = Bool.False
                var $dry = Bool.False
                var $bad = Bool.False
                while !$bad and $i < List.len(args) {
                    match List.get(args, $i) {
                        Err(_) => {
                            $i = List.len(args)
                        }
                        Ok("--from") => {
                            $i = $i + 1
                            match List.get(args, $i) {
                                Ok(branch) => {
                                    $from_ref = branch
                                }
                                Err(_) => {
                                    $bad = Bool.True
                                }
                            }
                        }
                        Ok("--force") => {
                            $force = Bool.True
                        }
                        Ok("--dry-run") => {
                            $dry = Bool.True
                        }
                        Ok(value) => {
                            if $tag == "" {
                                $tag = value
                            } else {
                                $bad = Bool.True
                            }
                        }
                    }
                    $i = $i + 1
                }
                if $bad or $tag == "" {
                    RelUsage
                } else {
                    RelRun({ tag: $tag, from_ref: $from_ref, force: $force, dry_run: $dry })
                }
            }
        }
    }
}

expect
    match do_version_from_ref("tag", "v1.2.3", "abcdef012345") {
        { version: "v1.2.3", prerelease } => prerelease == Bool.False
        _ => Bool.False
    }

expect
    match do_version_from_ref("tag", "dev", "abcdef012345") {
        { version: "dev-abcdef0", prerelease } => prerelease == Bool.True
        _ => Bool.False
    }

expect
    match do_version_from_ref("branch", "main", "abcdef012345") {
        { version: "dev-abcdef0", prerelease } => prerelease == Bool.True
        _ => Bool.False
    }

expect do_archive_stem("dev-abcdef0", "x86_64-unknown-linux-gnu") == "rocci-dev-abcdef0-x86_64-unknown-linux-gnu"

expect
    match do_release_params("tag", "dev", "abcdef012345") {
        { tag: "dev", name: "Development Build (abcdef0)", prerelease } => prerelease == Bool.True
        _ => Bool.False
    }

expect
    match do_parse_release(["patch", "--dry-run"]) {
        RelRun({ tag: "patch", dry_run, force, from_ref: "main" }) => dry_run == Bool.True and force == Bool.False
        _ => Bool.False
    }

expect
    match do_github_output([{ key: "version", val: "v1.2.3" }, { key: "prerelease", val: "false" }]) {
        text => text == "version=v1.2.3\nprerelease=false\n"
    }

expect
    match do_parse_check_line("completed success") {
        CheckStat({ status: "completed", conclusion: "success" }) => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_check_line("") {
        CheckEmpty => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_pr_ref("39") {
        { num: "39", branch: "" } => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_pr_ref("#39") {
        { num: "39", branch: "" } => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_pr_ref("https://github.com/koliyo/rocci/pull/39/files") {
        { num: "39", branch: "" } => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_pr_ref("refs/heads/feat/foo") {
        { num: "", branch: "feat/foo" } => Bool.True
        _ => Bool.False
    }

expect do_local_pr_branch("feat/example-source-sidebar") == "pr/feat/example-source-sidebar"
expect do_local_pr_branch("pr/feat/example-source-sidebar") == "pr/feat/example-source-sidebar"
expect do_local_pr_branch("refs/heads/fix/typo") == "pr/fix/typo"

expect
    match do_parse_worktrees("worktree /repo\nHEAD abc\nbranch refs/heads/main\n\nworktree /repo-feature\nHEAD def\nbranch refs/heads/feature\n\nworktree /repo-detach\nHEAD ghi\ndetached\n") {
        entries => {
            match (List.get(entries, 0), List.get(entries, 1), List.get(entries, 2)) {
                (Ok({ path: "/repo", branch: "refs/heads/main" }), Ok({ path: "/repo-feature", branch: "refs/heads/feature" }), Ok({ path: "/repo-detach", branch: "" })) => Bool.True
                _ => Bool.False
            }
        }
    }

expect
    match do_parse_pr_argv(["39", "--dry-run"]) {
        { help: Bool.False, dry: Bool.True, ref: "39", bad: Bool.False } => Bool.True
        _ => Bool.False
    }

expect
    match List.get(default_check_names, 0) {
        Ok("Code Formatting & Lints") => Bool.True
        _ => Bool.False
    }
