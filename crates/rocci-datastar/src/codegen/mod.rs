use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::{DatastarError, Result};

pub const DATASTAR_ROC_TEMPLATE: &str = r#"import pf.Html as PlatformHtml
import pf.Sse

ActionOpt : [
    OpenWhenHidden(Bool),
    ContentType([Json, Form]),
    Header(Str, Str),
    Retry([Auto, Error, Always, Never]),
    RequestCancellation([Auto, Cleanup, Disabled]),
]

PatchSignalsOpt : [OnlyIfMissing(Bool)]

Datastar := [].{
    patch_elements = |node|
        Sse.Event.keyed(
            "datastar-patch-elements",
            "elements",
            PlatformHtml.render_without_doc_type(node),
        )

    patch_signals = patch_signals_event
    patch_signals_with = patch_signals_event_with

    get = |uri| backend("get", uri, [])
    post = |uri| backend("post", uri, [])
    put = |uri| backend("put", uri, [])
    patch = |uri| backend("patch", uri, [])
    delete = |uri| backend("delete", uri, [])

    get_with = |uri, opts| backend("get", uri, opts)
    post_with = |uri, opts| backend("post", uri, opts)
    put_with = |uri, opts| backend("put", uri, opts)
    patch_with = |uri, opts| backend("patch", uri, opts)
    delete_with = |uri, opts| backend("delete", uri, opts)
}

patch_signals_event = |signals| patch_signals_event_with(signals, [])

patch_signals_event_with = |signals, opts| {
    only_if_missing =
        List.any(opts, |opt|
            match opt {
                OnlyIfMissing(value) => value
            },
        )
    option_fields = if only_if_missing { ["onlyIfMissing true"] } else { [] }
    lf = Str.join_with(Str.split_on(signals, "\r\n"), "\n")
    normalized = Str.join_with(Str.split_on(lf, "\r"), "\n")
    signal_fields = Str.split_on(normalized, "\n").map(|line| "signals ${line}")
    Sse.Event.named(
        "datastar-patch-signals",
        List.concat(option_fields, signal_fields),
    )
}

backend = |method, uri, opts| {
    fields = option_fields(opts)
    if List.is_empty(fields) {
        "@${method}(${js_str(uri)})"
    } else {
        "@${method}(${js_str(uri)}, {${join_comma(fields)}})"
    }
}

option_fields = |opts| {
    fields = List.fold(
        opts,
        [],
        |acc, opt|
            match opt {
                Header(_, _) => acc
                other => List.append(acc, opt_field(other))
            },
    )
    headers = List.fold(
        opts,
        [],
        |acc, opt|
            match opt {
                Header(name, value) => List.append(acc, "${js_str(name)}: ${js_str(value)}")
                _ => acc
            },
    )
    if List.is_empty(headers) {
        fields
    } else {
        List.append(fields, "headers: {${join_comma(headers)}}")
    }
}

opt_field = |opt|
    match opt {
        OpenWhenHidden(value) => "openWhenHidden: ${js_bool(value)}"
        ContentType(Json) => "contentType: 'json'"
        ContentType(Form) => "contentType: 'form'"
        Header(_, _) => ""
        Retry(Auto) => "retry: 'auto'"
        Retry(Error) => "retry: 'error'"
        Retry(Always) => "retry: 'always'"
        Retry(Never) => "retry: 'never'"
        RequestCancellation(Auto) => "requestCancellation: 'auto'"
        RequestCancellation(Cleanup) => "requestCancellation: 'cleanup'"
        RequestCancellation(Disabled) => "requestCancellation: 'disabled'"
    }

js_bool = |value| if value { "true" } else { "false" }

js_str = |text| "'${escape_js(text)}'"

escape_js = |text|
    List.fold(
        Str.to_utf8(text),
        "",
        |acc, byte|
            if byte == 39 or byte == 92 {
                "${acc}\\${Str.from_utf8_lossy([byte])}"
            } else if byte == 10 {
                "${acc}\\n"
            } else if byte == 13 {
                "${acc}\\r"
            } else {
                "${acc}${Str.from_utf8_lossy([byte])}"
            },
    )

join_comma = |items|
    List.fold(
        items,
        "",
        |acc, item|
            if acc == "" {
                item
            } else {
                "${acc}, ${item}"
            },
    )
"#;

pub fn stage_datastar_roc(dest_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(dest_dir).map_err(|source| DatastarError::CreateDir {
        path: dest_dir.to_path_buf(),
        source,
    })?;
    let dest = dest_dir.join("Datastar.roc");
    fs::write(&dest, DATASTAR_ROC_TEMPLATE).map_err(|source| DatastarError::WriteFile {
        path: dest.clone(),
        source,
    })?;
    Ok(dest)
}
