DocsCoverage := [].{
    slugify = do_slugify
    heading_ids = do_heading_ids
    split_route = do_split_route
    docs_candidates = do_docs_candidates
    check_coverage_text = do_check_coverage_text
    check_queries_text = do_check_queries_text
    check_sessions_text = do_check_sessions_text
}

PageHit : [PageMissing, PageText(Str)]

is_ascii_alnum = |b| {
    (b >= 48 and b <= 57) or (b >= 65 and b <= 90) or (b >= 97 and b <= 122)
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

do_slugify = |text| {
    bytes = Str.to_utf8(text)
    var $i = 0.U64
    var $out = []
    var $hyphen = Bool.False
    while $i < List.len(bytes) {
        match List.get(bytes, $i) {
            Ok(b) => {
                if is_ascii_alnum(b) {
                    lower = if b >= 65 and b <= 90 { b + 32 } else { b }
                    $out = List.concat($out, [lower])
                    $hyphen = Bool.False
                } else if List.len($out) > 0 and !$hyphen {
                    $out = List.concat($out, [45])
                    $hyphen = Bool.True
                }
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    var $end = List.len($out)
    var $trim = Bool.True
    while $trim and $end > 0 {
        match List.get($out, $end - 1) {
            Ok(45) => { $end = $end - 1 }
            _ => { $trim = Bool.False }
        }
    }
    Str.from_utf8_lossy(bytes_range($out, 0, $end))
}

trim_ascii = |text| {
    bytes = Str.to_utf8(text)
    var $start = 0.U64
    var $end = List.len(bytes)
    var $loop = Bool.True
    while $loop and $start < $end {
        match List.get(bytes, $start) {
            Ok(32) => { $start = $start + 1 }
            Ok(9) => { $start = $start + 1 }
            Ok(13) => { $start = $start + 1 }
            _ => { $loop = Bool.False }
        }
    }
    $loop = Bool.True
    while $loop and $end > $start {
        match List.get(bytes, $end - 1) {
            Ok(32) => { $end = $end - 1 }
            Ok(9) => { $end = $end - 1 }
            Ok(13) => { $end = $end - 1 }
            _ => { $loop = Bool.False }
        }
    }
    var $i = $start
    var $out = []
    while $i < $end {
        match List.get(bytes, $i) {
            Ok(b) => { $out = List.concat($out, [b]) }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.from_utf8_lossy($out)
}

ends_with_slash = |text| {
    bytes = Str.to_utf8(text)
    if List.len(bytes) == 0 {
        Bool.False
    } else {
        match List.get(bytes, List.len(bytes) - 1) {
            Ok(47) => Bool.True
            _ => Bool.False
        }
    }
}

starts_with = |text, prefix|
    match Str.split_first(text, prefix) {
        Ok({ before, after: _rest }) => before == ""
        Err(_) => Bool.False
    }

do_split_route = |route| {
    match Str.split_first(route, "#") {
        Ok({ before, after }) => {
            path = if ends_with_slash(before) {
                before
            } else {
                Str.concat(before, "/")
            }
            { path: path, fragment: after }
        }
        Err(_) => {
            path = if ends_with_slash(route) { route } else { Str.concat(route, "/") }
            { path: path, fragment: "" }
        }
    }
}

strip_prefix = |text, prefix| {
    match Str.split_first(text, prefix) {
        Ok({ before, after }) => {
            if before == "" {
                after
            } else {
                text
            }
        }
        _ => text
    }
}

trim_slashes = |path| {
    bytes = Str.to_utf8(path)
    var $start = 0.U64
    var $end = List.len(bytes)
    while $start < $end and (
        match List.get(bytes, $start) {
            Ok(47) => Bool.True
            _ => Bool.False
        }
    ) {
        $start = $start + 1
    }
    while $end > $start and (
        match List.get(bytes, $end - 1) {
            Ok(47) => Bool.True
            _ => Bool.False
        }
    ) {
        $end = $end - 1
    }
    var $i = $start
    var $out = []
    while $i < $end {
        match List.get(bytes, $i) {
            Ok(b) => { $out = List.concat($out, [b]) }
            Err(_) => {}
        }
        $i = $i + 1
    }
    Str.from_utf8_lossy($out)
}

do_docs_candidates = |path_part| {
    if starts_with(path_part, "/examples/") or path_part == "/examples/" {
        ["examples/rocci/apps.toml"]
    } else {
        trimmed = strip_prefix(trim_slashes(path_part), "docs/")
        [
            "docs/${trimmed}.rocdown",
            "docs/${trimmed}/index.rocdown",
            "site/${trimmed}.rocdown",
            "site/${trimmed}/index.rocdown",
        ]
    }
}

do_heading_ids = |text| {
    lines = Str.split_on(text, "\n")
    List.fold(
        lines,
        [],
        |acc, line| {
            bytes = Str.to_utf8(line)
            var $i = 0.U64
            var $hashes = 0.U64
            while $hashes < 6 and (
                match List.get(bytes, $i) {
                    Ok(35) => Bool.True
                    _ => Bool.False
                }
            ) {
                $hashes = $hashes + 1
                $i = $i + 1
            }
            if $hashes == 0 {
                acc
            } else {
                match List.get(bytes, $i) {
                    Ok(32) => List.concat(acc, [do_slugify(Str.from_utf8_lossy(bytes_range(bytes, $i + 1, List.len(bytes))))])
                    Ok(9) => List.concat(acc, [do_slugify(Str.from_utf8_lossy(bytes_range(bytes, $i + 1, List.len(bytes))))])
                    _ => acc
                }
            }
        },
    )
}

parse_quoted = |raw| {
    t = trim_ascii(raw)
    bytes = Str.to_utf8(t)
    if List.len(bytes) < 2 {
        t
    } else {
        match (List.get(bytes, 0), List.get(bytes, List.len(bytes) - 1)) {
            (Ok(34), Ok(34)) => Str.from_utf8_lossy(bytes_range(bytes, 1, List.len(bytes) - 1))
            _ => t
        }
    }
}

table_get = |pairs, key| {
    List.fold(
        pairs,
        "",
        |acc, pair| {
            if pair.key == key {
                pair.value
            } else {
                acc
            }
        },
    )
}

parse_tables = |text, kind| {
    lines = Str.split_on(text, "\n")
    collected = List.fold(
        lines,
        { current: [], tables: [], in_table: Bool.False },
        |state, raw_line| {
            line = trim_ascii(raw_line)
            if line == "" or starts_with(line, "#") {
                state
            } else if starts_with(line, "[[") {
                flushed = if state.in_table {
                    { current: [], tables: List.concat(state.tables, [state.current]), in_table: Bool.True }
                } else {
                    { current: [], tables: state.tables, in_table: Bool.True }
                }
                marker = "[[${kind}]]"
                if line == marker {
                    { current: [], tables: flushed.tables, in_table: Bool.True }
                } else {
                    { current: [], tables: flushed.tables, in_table: Bool.False }
                }
            } else if state.in_table {
                match Str.split_first(line, "=") {
                    Ok({ before, after }) => {
                        pair = { key: trim_ascii(before), value: parse_quoted(after) }
                        { current: List.concat(state.current, [pair]), tables: state.tables, in_table: Bool.True }
                    }
                    Err(_) => state
                }
            } else {
                state
            }
        },
    )
    if collected.in_table {
        List.concat(collected.tables, [collected.current])
    } else {
        collected.tables
    }
}

header_get = |text, key| {
    lines = Str.split_on(text, "\n")
    List.fold(
        lines,
        "",
        |acc, raw_line| {
            line = trim_ascii(raw_line)
            if starts_with(line, "[[") {
                acc
            } else {
                match Str.split_first(line, "=") {
                    Ok({ before, after }) => {
                        if trim_ascii(before) == key {
                            parse_quoted(after)
                        } else {
                            acc
                        }
                    }
                    _ => acc
                }
            }
        },
    )
}

list_has = |xs, needle|
    List.fold(xs, Bool.False, |acc, x| if acc { Bool.True } else { x == needle })

allowed_status = ["current", "experimental", "planned", "removed"]
owned_status = ["current", "experimental"]
allowed_entry = ["roc-first", "web-first"]
allowed_disposition = ["page-fix", "product-issue", "non-goal"]

lookup_page = |pages, rel| {
    List.fold(
        pages,
        PageMissing,
        |acc, page| {
            match acc {
                PageText(_) => acc
                PageMissing => {
                    if page.path == rel {
                        PageText(page.text)
                    } else {
                        PageMissing
                    }
                }
            }
        },
    )
}

resolve_page = |pages, path_part| {
    candidates = do_docs_candidates(path_part)
    List.fold(
        candidates,
        PageMissing,
        |acc, rel| {
            match acc {
                PageText(_) => acc
                PageMissing => lookup_page(pages, rel)
            }
        },
    )
}

do_check_coverage_text = |toml_text, pages| {
    tables = parse_tables(toml_text, "feature")
    folded = List.fold(
        tables,
        { errors: [], seen: [] },
        |state, pairs| {
            fid = table_get(pairs, "id")
            status = table_get(pairs, "status")
            canonical = table_get(pairs, "canonical")
            dup = if list_has(state.seen, fid) {
                ["duplicate feature id `${fid}`"]
            } else {
                []
            }
            seen = List.concat(state.seen, [fid])
            rest = if !list_has(allowed_status, status) {
                ["${fid}: unknown status `${status}`"]
            } else if status == "current" and (
                match Str.split_first(fid, "removed") {
                    Ok(_) => Bool.True
                    Err(_) => Bool.False
                }
            ) {
                ["${fid}: removed feature labeled current"]
            } else if !list_has(owned_status, status) {
                []
            } else if canonical == "" {
                ["${fid}: current/experimental feature has no canonical page"]
            } else {
                route = do_split_route(canonical)
                match resolve_page(pages, route.path) {
                    PageMissing => ["${fid}: missing canonical page ${canonical}"]
                    PageText(body) => {
                        if route.fragment == "" {
                            []
                        } else if list_has(do_heading_ids(body), route.fragment) {
                            []
                        } else {
                            ["${fid}: canonical fragment `#${route.fragment}` missing"]
                        }
                    }
                }
            }
            { errors: List.concat(List.concat(state.errors, dup), rest), seen: seen }
        },
    )
    folded.errors
}

do_check_queries_text = |toml_text, pages| {
    tables = parse_tables(toml_text, "query")
    List.fold(
        tables,
        [],
        |acc, pairs| {
            q = table_get(pairs, "q")
            target = table_get(pairs, "expect")
            route = do_split_route(target)
            match resolve_page(pages, route.path) {
                PageMissing => List.concat(acc, ["search `${q}`: missing page ${target}"])
                PageText(body) => {
                    if route.fragment == "" {
                        acc
                    } else if list_has(do_heading_ids(body), route.fragment) {
                        acc
                    } else {
                        List.concat(acc, ["search `${q}`: missing fragment `#${route.fragment}` on ${target}"])
                    }
                }
            }
        },
    )
}

do_check_sessions_text = |toml_text| {
    schema = header_get(toml_text, "schema_version")
    product = header_get(toml_text, "product")
    header_errors = List.concat(
        if schema == "1" { [] } else { ["first-use-sessions: schema_version must be 1"] },
        if product == "rocci" { [] } else { ["first-use-sessions: product must be rocci"] },
    )
    tables = parse_tables(toml_text, "session")
    session_errors = List.fold(
        tables,
        [],
        |acc, pairs| {
            sid = table_get(pairs, "id")
            prefix = if sid == "" { "session" } else { "session `${sid}`" }
            entry = table_get(pairs, "entry")
            date = table_get(pairs, "date")
            success = table_get(pairs, "success")
            e1 = if list_has(allowed_entry, entry) {
                []
            } else {
                ["${prefix}: entry must be roc-first or web-first"]
            }
            e2 = if date == "" { ["${prefix}: missing date"] } else { [] }
            e3 = if success == "" {
                ["${prefix}: missing success"]
            } else if success == "true" {
                minutes = table_get(pairs, "minutes_to_visible")
                if minutes == "" {
                    ["${prefix}: successful session needs minutes_to_visible"]
                } else {
                    []
                }
            } else {
                failed = table_get(pairs, "failed_step")
                disposition = table_get(pairs, "disposition")
                f1 = if failed == "" { ["${prefix}: failed session needs failed_step"] } else { [] }
                f2 = if list_has(allowed_disposition, disposition) {
                    []
                } else {
                    ["${prefix}: disposition must be page-fix, product-issue, or non-goal"]
                }
                List.concat(f1, f2)
            }
            List.concat(acc, List.concat(List.concat(e1, e2), e3))
        },
    )
    List.concat(header_errors, session_errors)
}

expect do_slugify("Build release") == "build-release"
expect do_slugify("Serve options") == "serve-options"
expect do_slugify("App") == "app"

expect
    match do_split_route("/docs/install/#app") {
        { path: "/docs/install/", fragment: "app" } => Bool.True
        _ => Bool.False
    }

expect
    match do_check_sessions_text("schema_version = 1\nproduct = \"rocci\"\n\n[[session]]\nid = \"roc-first-20990101\"\nentry = \"roc-first\"\ndate = \"2099-01-01\"\nsuccess = false\n") {
        errors => {
            list_has(errors, "session `roc-first-20990101`: failed session needs failed_step")
            and list_has(errors, "session `roc-first-20990101`: disposition must be page-fix, product-issue, or non-goal")
        }
    }

expect
    match do_check_coverage_text("[[feature]]\nid = \"syntax.ghost\"\nstatus = \"current\"\ncanonical = \"/docs/reference/missing/\"\n", []) {
        errors => list_has(errors, "syntax.ghost: missing canonical page /docs/reference/missing/")
    }
