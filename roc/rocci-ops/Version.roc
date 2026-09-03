Version := [].{
    parse_release_version = do_parse_release_version
    next_release_version = do_next_release_version
    first_package_version = do_first_package_version
    workspace_crate_names = do_workspace_crate_names
    replace_package_versions = do_replace_package_versions
    replace_lock_crate_versions = do_replace_lock_crate_versions
}

is_digit = |b| b >= 48 and b <= 57

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

parse_u64 = |text| {
    bytes = Str.to_utf8(text)
    if List.len(bytes) == 0 {
        CoreNope
    } else {
        var $i = 0.U64
        var $n = 0.U64
        var $ok = Bool.True
        while $ok and $i < List.len(bytes) {
            match List.get(bytes, $i) {
                Ok(b) => {
                    if is_digit(b) {
                        $n = $n * 10 + (b - 48).to_u64()
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
        if $ok {
            CoreNum($n)
        } else {
            CoreNope
        }
    }
}

parse_core_semver = |current| {
    parts = Str.split_on(current, ".")
    if List.len(parts) != 3 {
        CoreNope
    } else {
        match (List.get(parts, 0), List.get(parts, 1), List.get(parts, 2)) {
            (Ok(maj_s), Ok(min_s), Ok(pat_s)) => {
                match (parse_u64(maj_s), parse_u64(min_s), parse_u64(pat_s)) {
                    (CoreNum(maj), CoreNum(mi), CoreNum(pat)) => CoreSemver({ major: maj, minor: mi, patch: pat })
                    _ => CoreNope
                }
            }
            _ => CoreNope
        }
    }
}

do_next_release_version = |current, level| {
    match parse_core_semver(current) {
        CoreNope => Nope("cannot bump ${current}; expected X.Y.Z")
        CoreNum(_) => Nope("cannot bump ${current}; expected X.Y.Z")
        CoreSemver(parts) => {
            match level {
                "patch" => Got("${parts.major.to_str()}.${parts.minor.to_str()}.${(parts.patch + 1).to_str()}")
                "minor" => Got("${parts.major.to_str()}.${(parts.minor + 1).to_str()}.0")
                "major" => Got("${(parts.major + 1).to_str()}.0.0")
                _ => Nope("unknown bump level: ${level}")
            }
        }
    }
}

do_parse_release_version = |tag| {
    bytes = Str.to_utf8(tag)
    match List.get(bytes, 0) {
        Ok(118) => {
            if List.len(bytes) < 2 {
                Nope("release requires a v* name, bump level, or the movable dev tag")
            } else {
                rest = Str.from_utf8_lossy(bytes_range(bytes, 1, List.len(bytes)))
                match parse_core_semver(rest) {
                    CoreSemver(_) => Got(rest)
                    _ => {
                        match Str.split_first(rest, "-") {
                            Ok({ before, after: _prerelease }) => {
                                match parse_core_semver(before) {
                                    CoreSemver(_) => Got(rest)
                                    _ => Nope("release ${tag} is not a vX.Y.Z version")
                                }
                            }
                            Err(_) => Nope("release ${tag} is not a vX.Y.Z version")
                        }
                    }
                }
            }
        }
        _ => Nope("release requires a v* name, bump level, or the movable dev tag")
    }
}

do_first_package_version = |text| {
    match Str.split_first(text, "\nversion = \"") {
        Ok({ before: _prefix, after }) => {
            match Str.split_first(after, "\"") {
                Ok({ before, after: _rest }) => Got(before)
                Err(_) => Nope("could not find package version")
            }
        }
        Err(_) => {
            match Str.split_first(text, "version = \"") {
                Ok({ before, after }) => {
                    if before == "" {
                        match Str.split_first(after, "\"") {
                            Ok({ before: ver, after: _rest }) => Got(ver)
                            Err(_) => Nope("could not find package version")
                        }
                    } else {
                        Nope("could not find package version")
                    }
                }
                Err(_) => Nope("could not find package version")
            }
        }
    }
}

do_workspace_crate_names = |cargo_text| {
    var $rest = cargo_text
    var $names = []
    var $loop = Bool.True
    while $loop {
        match Str.split_first($rest, "\"crates/") {
            Err(_) => {
                $loop = Bool.False
            }
            Ok({ before: _prefix, after }) => {
                match Str.split_first(after, "\"") {
                    Err(_) => {
                        $loop = Bool.False
                    }
                    Ok({ before, after: tail }) => {
                        $names = List.concat($names, [before])
                        $rest = tail
                    }
                }
            }
        }
    }
    $names
}

do_replace_package_versions = |text, version| {
    replacement = "version = \"${version}\""
    var $rest = text
    var $out = ""
    var $count = 0.U64
    var $loop = Bool.True
    while $loop {
        match Str.split_first($rest, "\nversion = \"") {
            Err(_) => {
                $out = Str.concat($out, $rest)
                $loop = Bool.False
            }
            Ok({ before, after }) => {
                match Str.split_first(after, "\"") {
                    Err(_) => {
                        $out = Str.concat($out, $rest)
                        $loop = Bool.False
                    }
                    Ok({ before: _old, after: tail }) => {
                        $out = Str.concat($out, before)
                        $out = Str.concat($out, "\n")
                        $out = Str.concat($out, replacement)
                        $count = $count + 1
                        $rest = tail
                    }
                }
            }
        }
    }
    if $count == 0 {
        Nope("could not find package version")
    } else {
        Got($out)
    }
}

lock_name_is_member = |name, names|
    List.fold(names, Bool.False, |acc, n| if acc { Bool.True } else { n == name })

do_replace_lock_crate_versions = |text, version, names| {
    var $rest = text
    var $out = ""
    var $count = 0.U64
    var $loop = Bool.True
    while $loop {
        match Str.split_first($rest, "[[package]]\nname = \"") {
            Err(_) => {
                $out = Str.concat($out, $rest)
                $loop = Bool.False
            }
            Ok({ before, after }) => {
                $out = Str.concat($out, before)
                $out = Str.concat($out, "[[package]]\nname = \"")
                match Str.split_first(after, "\"\nversion = \"") {
                    Err(_) => {
                        $out = Str.concat($out, after)
                        $loop = Bool.False
                    }
                    Ok({ before: name, after: after_ver }) => {
                        $out = Str.concat($out, name)
                        $out = Str.concat($out, "\"\nversion = \"")
                        match Str.split_first(after_ver, "\"") {
                            Err(_) => {
                                $out = Str.concat($out, after_ver)
                                $loop = Bool.False
                            }
                            Ok({ before: old_ver, after: tail }) => {
                                if lock_name_is_member(name, names) {
                                    $out = Str.concat($out, version)
                                    $count = $count + 1
                                } else {
                                    $out = Str.concat($out, old_ver)
                                }
                                $out = Str.concat($out, "\"")
                                $rest = tail
                            }
                        }
                    }
                }
            }
        }
    }
    if $count == 0 {
        Nope("could not find workspace crate versions in Cargo.lock")
    } else {
        Got($out)
    }
}

expect
    match do_next_release_version("1.2.3", "patch") {
        Got("1.2.4") => Bool.True
        _ => Bool.False
    }

expect
    match do_next_release_version("0.1.2", "minor") {
        Got("0.2.0") => Bool.True
        _ => Bool.False
    }

expect
    match do_next_release_version("0.1.2", "major") {
        Got("1.0.0") => Bool.True
        _ => Bool.False
    }

expect
    match do_parse_release_version("v1.2.3") {
        Got("1.2.3") => Bool.True
        _ => Bool.False
    }

expect
    match do_first_package_version("[workspace.package]\nversion = \"0.1.0\"\n\nclap = { version = \"4.5\" }\n") {
        Got("0.1.0") => Bool.True
        _ => Bool.False
    }

expect
    match do_replace_package_versions("[workspace.package]\nversion = \"0.1.0\"\n\nclap = { version = \"4.5\" }\n", "2.0.0") {
        Got(updated) => {
            match do_first_package_version(updated) {
                Got("2.0.0") => {
                    match Str.split_first(updated, "clap = { version = \"4.5\" }") {
                        Ok(_) => Bool.True
                        Err(_) => Bool.False
                    }
                }
                _ => Bool.False
            }
        }
        _ => Bool.False
    }

expect List.len(do_workspace_crate_names("[workspace]\nmembers = [\n    \"crates/rocci-cli\",\n    \"crates/rocci-core\",\n]\n")) == 2

expect
    match do_replace_lock_crate_versions("[[package]]\nname = \"rocci-cli\"\nversion = \"0.1.0\"\n\n[[package]]\nname = \"rocci-core\"\nversion = \"0.1.0\"\n", "3.1.4", ["rocci-cli", "rocci-core"]) {
        Got(updated) => List.len(Str.split_on(updated, "version = \"3.1.4\"")) == 3
        _ => Bool.False
    }
