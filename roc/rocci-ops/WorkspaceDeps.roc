WorkspaceDeps := [].{
    decode = do_decode
    check = do_check
    classify = do_classify
    forbidden = do_forbidden
}

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

Class : [BaseRocci, Rocdown, NoClass]

CheckResult : {
    errors : List(Str),
    notes : List(Str),
    package_count : U64,
}

base_rocci = [
    "rocci-core",
    "rocci-docs",
    "rocci-template",
    "rocci-ungram",
    "rocci-desktop",
    "rocci-cli",
    "rocci-lsp",
    "rocci-highlight",
    "rocci-ui",
    "rocci-roc-host",
    "rocci-platform",
    "rocci-wasi-http",
    "rocci-wasi-http-component",
    "rocci-datastar",
]

rocdown = [
    "rocci-rocdown",
    "rocci-theme",
    "rocci-rocdown-cli",
    "rocci-rocdown-lsp",
    "rocci-playground-spike",
    "rocci-playground",
    "rocci-playground-wasm",
]

rename_packages_key = |src| {
    match Str.split_first(src, "\"packages\":") {
        Ok({ before, after }) => Str.concat(before, Str.concat("\"pkgs\":", after))
        Err(_) => src
    }
}

do_decode : Str -> Try(Metadata, [InvalidJson(Str), MissingRequiredField(Str), ..])
do_decode = |src|
    Encoding.Json.parse(rename_packages_key(src))

list_has = |xs, needle|
    List.fold(xs, Bool.False, |acc, x| if acc { Bool.True } else { x == needle })

list_set = |xs, idx, val| {
    var $i = 0.U64
    var $out = []
    while $i < List.len(xs) {
        match List.get(xs, $i) {
            Ok(x) => {
                if $i == idx {
                    $out = List.concat($out, [val])
                } else {
                    $out = List.concat($out, [x])
                }
            }
            Err(_) => {}
        }
        $i = $i + 1
    }
    $out
}

str_gt = |a, b| {
    a_bytes = Str.to_utf8(a)
    b_bytes = Str.to_utf8(b)
    var $i = 0.U64
    var $done = Bool.False
    var $gt = Bool.False
    while !$done {
        match (List.get(a_bytes, $i), List.get(b_bytes, $i)) {
            (Err(_), Err(_)) => {
                $done = Bool.True
                $gt = Bool.False
            }
            (Err(_), Ok(_)) => {
                $done = Bool.True
                $gt = Bool.False
            }
            (Ok(_), Err(_)) => {
                $done = Bool.True
                $gt = Bool.True
            }
            (Ok(x), Ok(y)) => {
                if x > y {
                    $done = Bool.True
                    $gt = Bool.True
                } else if x < y {
                    $done = Bool.True
                    $gt = Bool.False
                } else {
                    $i = $i + 1
                }
            }
        }
    }
    $gt
}

sort_strs = |items| {
    var $out = items
    var $i = 0.U64
    while $i < List.len($out) {
        var $j = $i + 1
        while $j < List.len($out) {
            match (List.get($out, $i), List.get($out, $j)) {
                (Ok(a), Ok(b)) => {
                    if str_gt(a, b) {
                        $out = list_set($out, $i, b)
                        $out = list_set($out, $j, a)
                    }
                }
                _ => {}
            }
            $j = $j + 1
        }
        $i = $i + 1
    }
    $out
}

classified_names = List.concat(base_rocci, rocdown)

do_classify = |name| {
    if list_has(base_rocci, name) {
        BaseRocci
    } else if list_has(rocdown, name) {
        Rocdown
    } else {
        NoClass
    }
}

do_forbidden = |src, dest| {
    match (do_classify(src), do_classify(dest)) {
        (BaseRocci, Rocdown) => Ok("base Rocci package ${src} must not depend on rocdown package ${dest}")
        _ => Err({})
    }
}

workspace_packages = |meta| {
    List.fold(
        meta.pkgs,
        [],
        |acc, pkg| {
            if list_has(meta.workspace_members, pkg.id) {
                List.concat(acc, [pkg])
            } else {
                acc
            }
        },
    )
}

package_names = |pkgs|
    List.fold(pkgs, [], |acc, pkg| List.concat(acc, [pkg.name]))

direct_workspace_edges = |pkgs| {
    names = package_names(pkgs)
    List.fold(
        pkgs,
        [],
        |acc, pkg| {
            seen_and_edges = List.fold(
                pkg.dependencies,
                { seen: [], edges: acc },
                |state, dep| {
                    if list_has(names, dep.name) and !list_has(state.seen, dep.name) {
                        {
                            seen: List.concat(state.seen, [dep.name]),
                            edges: List.concat(state.edges, [{ src: pkg.name, dest: dep.name }]),
                        }
                    } else {
                        state
                    }
                },
            )
            seen_and_edges.edges
        },
    )
}

do_check = |meta| {
    pkgs = workspace_packages(meta)
    names = package_names(pkgs)
    unclassified = sort_strs(
        List.fold(
            names,
            [],
            |acc, name| if list_has(classified_names, name) { acc } else { List.concat(acc, [name]) },
        ),
    )
    missing = sort_strs(
        List.fold(
            classified_names,
            [],
            |acc, name| if list_has(names, name) { acc } else { List.concat(acc, [name]) },
        ),
    )
    unclass_errors = List.fold(
        unclassified,
        [],
        |acc, name| List.concat(acc, ["unclassified workspace package ${name}"]),
    )
    missing_errors = List.fold(
        missing,
        [],
        |acc, name| List.concat(acc, ["classified name is not a workspace package: ${name}"]),
    )
    edges = direct_workspace_edges(pkgs)
    edge_errors = List.fold(
        edges,
        [],
        |acc, edge| {
            match do_forbidden(edge.src, edge.dest) {
                Ok(reason) => List.concat(acc, ["${edge.src} -> ${edge.dest}: ${reason}"])
                Err(_) => acc
            }
        },
    )
    {
        errors: List.concat(List.concat(unclass_errors, missing_errors), edge_errors),
        notes: [],
        package_count: List.len(pkgs),
    }
}

expect
    match do_classify("rocci-core") {
        BaseRocci => Bool.True
        _ => Bool.False
    }

expect
    match do_classify("rocci-rocdown") {
        Rocdown => Bool.True
        _ => Bool.False
    }

expect
    match do_forbidden("rocci-core", "rocci-rocdown") {
        Ok(_) => Bool.True
        Err(_) => Bool.False
    }

expect
    match do_forbidden("rocci-rocdown", "rocci-core") {
        Err(_) => Bool.True
        Ok(_) => Bool.False
    }

expect
    match do_decode("{\"workspace_members\":[\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\"],\"packages\":[{\"name\":\"alpha\",\"id\":\"alpha 0.1.0 (path+file:///tmp/ws/alpha)\",\"dependencies\":[{\"name\":\"beta\"}]}]}") {
        Ok(meta) => {
            result = do_check(meta)
            list_has(result.errors, "unclassified workspace package alpha")
        }
        Err(_) => Bool.False
    }
