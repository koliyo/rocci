//! Shared CLI/editor platform source profile, embedded in distributed binaries.
include!(concat!(env!("OUT_DIR"), "/roc_sources.rs"));

pub fn compatibility_source(name: &str, source: &str) -> String {
    let mut source = source
        .lines()
        .map(|line| {
            for module in ["IOErr", "Path", "OsStr", "UnixTime"] {
                if line == format!("import {module} exposing [{module}]") {
                    return format!("import {module}");
                }
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n");
    source.push('\n');
    if matches!(name, "Path" | "OsStr") {
        let old = format!(
            "from_interpolation : Str, Iter((Str, Str)) -> {name}\n\tfrom_interpolation = |first, rest|\n\t\tUtf8(rest.fold(first, |acc, (interpolated, segment)| acc.concat(interpolated).concat(segment)))"
        );
        let new = format!(
            r#"from_interpolation : List(Str) -> Try(List(Str) -> {name}, [InvalidInterpolation(Str)])
    from_interpolation = |segments|
        Ok(|values| {{
            result = segments.fold({{ text: "", index: 0.U64 }}, |state, segment| {{
                value = values.get(state.index) ?? ""
                {{ text: state.text.concat(segment).concat(value), index: state.index + 1 }}
            }})
            Utf8(result.text)
        }})"#
        );
        source = source.replace(&old, &new);
    }
    if name == "Url" {
        source = source.replace("Err(Json.invalid_json)", "Err(Json.invalid_json({}))");
    }
    if name == "Sqlite" {
        // Object SQL prepares lazily: metadata belongs to the materialized
        // execution, after parameter binding, rather than a native statement.
        source = source.replace("columns = sqlite_columns!(host)?", "columns = []");
        source = source.replace("decode_exactly_one_row!(exec, stmt.columns, parse_row, limits)", "columns = sqlite_columns!(stmt_to_host(stmt))?\n            validate_unique_columns(columns)?\n            decode_exactly_one_row!(exec, columns, parse_row, limits)");
        source = source.replace("decode_rows!(exec, stmt.columns, parse_row, limits)", "columns = sqlite_columns!(stmt_to_host(stmt))?\n            validate_unique_columns(columns)?\n            decode_rows!(exec, columns, parse_row, limits)");
    }
    if name == "Datastar" {
        source = source.replacen("import Sse\n", "import Sse\nimport WorkerRoutes\n", 1);
        source = source.replacen("    fields = option_fields(opts)", r#"    live = method == "get" and List.contains(WorkerRoutes.live_paths, uri)
    explicit_retry = List.any(opts, |opt|
        match opt {
            Retry(_) => True
            _ => False
        },
    )
    effective_opts = if live and Bool.not(explicit_retry) { opts.append(Retry(Always)) } else { opts }
    fields = option_fields(effective_opts)"#, 1);
    }
    source
}
