use crate::error_page::{self, ListedRoute};
use crate::serve;

use super::infer::{
    form_params, is_float, is_i64, record_has_field, strip_num_suffix, top_level_fields,
};
use super::{
    BrowseFixture, BrowseParam, CatalogEntry, HTTP_PKG, ModuleGroup, ParamKind,
    can_preview_from_form,
};

pub(crate) fn generate_catalog_roc(groups: &[ModuleGroup]) -> String {
    let mut out = String::from("Catalog := [].{\n    groups = [\n");
    for (index, group) in groups.iter().enumerate() {
        out.push_str("        {\n");
        out.push_str(&format!(
            "            mod_name: {},\n",
            roc_string(&group.module)
        ));
        out.push_str("            entries: [\n");
        for (entry_index, entry) in group.entries.iter().enumerate() {
            out.push_str(&catalog_entry_roc(entry, "                "));
            if entry_index + 1 != group.entries.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("            ],\n        }");
        if index + 1 != groups.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(
        "    ]\n\n    find = |id|\n        List.fold(\n            groups,\n            Err(NotFound),\n            |acc, group|\n                match acc {\n                    Ok(found) => Ok(found)\n                    Err(err) =>\n                        match List.get(List.keep_if(group.entries, |entry| entry.id == id), 0) {\n                            Ok(entry) => Ok(entry)\n                            Err(_) => Err(err)\n                        }\n                },\n        )\n}\n",
    );
    out
}

pub(crate) fn catalog_entry_roc(entry: &CatalogEntry, indent: &str) -> String {
    let params = form_params(entry);
    let mut out = format!("{indent}{{\n");
    out.push_str(&format!("{indent}    id: {},\n", roc_string(&entry.id)));
    out.push_str(&format!(
        "{indent}    mod_name: {},\n",
        roc_string(&entry.module)
    ));
    out.push_str(&format!("{indent}    name: {},\n", roc_string(&entry.name)));
    out.push_str(&format!("{indent}    file: {},\n", roc_string(&entry.file)));
    out.push_str(&format!(
        "{indent}    previewable: {},\n",
        roc_bool(entry.previewable)
    ));
    out.push_str(&format!(
        "{indent}    reason: {},\n",
        roc_string(&entry.reason)
    ));
    if params.is_empty() {
        out.push_str(&format!("{indent}    params: [],\n"));
    } else {
        out.push_str(&format!("{indent}    params: [\n"));
        for (index, param) in params.iter().enumerate() {
            out.push_str(&format!(
                "{indent}        {{ name: {}, required: {}, kind: {}, value: {} }}",
                roc_string(&param.name),
                roc_bool(param.required),
                roc_string(param.kind.as_ref().map(ParamKind::as_str).unwrap_or("")),
                roc_string(&param.default_display),
            ));
            if index + 1 != params.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    if entry.fixtures.is_empty() {
        out.push_str(&format!("{indent}    fixtures: [],\n"));
    } else {
        out.push_str(&format!("{indent}    fixtures: [\n"));
        for (index, fixture) in entry.fixtures.iter().enumerate() {
            out.push_str(&fixture_roc(fixture, &format!("{indent}        ")));
            if index + 1 != entry.fixtures.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn fixture_roc(fixture: &BrowseFixture, indent: &str) -> String {
    let mut out = format!("{indent}{{\n");
    out.push_str(&format!(
        "{indent}    name: {},\n",
        roc_string(&fixture.name)
    ));
    out.push_str(&format!(
        "{indent}    source: {},\n",
        roc_string(&fixture.value)
    ));
    if fixture.scalars.is_empty() {
        out.push_str(&format!("{indent}    scalars: [],\n"));
    } else {
        out.push_str(&format!("{indent}    scalars: [\n"));
        for (index, (name, value)) in fixture.scalars.iter().enumerate() {
            out.push_str(&format!(
                "{indent}        {{ name: {}, value: {} }}",
                roc_string(name),
                roc_string(value)
            ));
            if index + 1 != fixture.scalars.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str(&format!("{indent}    ],\n"));
    }
    out.push_str(&format!("{indent}}}"));
    out
}

pub(crate) fn generate_preview_roc(groups: &[ModuleGroup]) -> String {
    let mut imports: Vec<String> = Vec::new();
    for group in groups {
        if !group.import_ok {
            continue;
        }
        for entry in &group.entries {
            if !entry.previewable {
                continue;
            }
            imports.push(group.module.clone());
            for fixture in &entry.fixtures {
                imports.push(fixture.module.clone());
            }
        }
    }
    imports.sort();
    imports.dedup();

    let mut out = String::from("import pf.Html\nimport Query\n");
    for name in &imports {
        out.push_str("import ");
        out.push_str(name);
        out.push('\n');
    }
    out.push_str("\nPreview := [].{\n    render = |id, args|\n        match id {\n");
    for group in groups {
        if !group.import_ok {
            continue;
        }
        for entry in &group.entries {
            if !entry.previewable {
                continue;
            }
            out.push_str(&preview_id_arm(&group.module, entry));
        }
    }
    out.push_str("            _ => shell(Html.text(\"Unknown component\"))\n        }\n}\n\n");
    out.push_str(
        r#"shell = |node|
    Html.element(
        "html",
        [Html.attribute("lang", "en")],
        [
            Html.element(
                "head",
                [],
                [
                    Html.void_element("meta", [Html.attribute("charset", "utf-8")]),
                    Html.element("title", [], [Html.text("rocci browse")]),
                ],
            ),
            Html.element("body", [], [node]),
        ],
    )
"#,
    );
    out
}

pub(crate) fn preview_id_arm(module: &str, entry: &CatalogEntry) -> String {
    let wrap = |call: String| {
        if entry.full_document {
            call
        } else {
            format!("shell({call})")
        }
    };
    if entry.fixtures.is_empty() {
        return format!(
            "            {} => {}\n",
            roc_string(&entry.id),
            wrap(generate_runtime_call(module, entry))
        );
    }
    let mut out = format!(
        "            {} =>\n                match Query.arg_str(args, \"fixture\") ?? \"\" {{\n",
        roc_string(&entry.id)
    );
    for fixture in &entry.fixtures {
        out.push_str(&format!(
            "                    {} => {}\n",
            roc_string(&fixture.name),
            wrap(generate_fixture_call(module, entry, fixture))
        ));
    }
    let fallback = if can_preview_from_form(entry) {
        generate_runtime_call(module, entry)
    } else {
        generate_fixture_call(module, entry, &entry.fixtures[0])
    };
    out.push_str(&format!(
        "                    _ => {}\n                }}\n",
        wrap(fallback)
    ));
    out
}

pub(crate) fn generate_fixture_call(
    module: &str,
    entry: &CatalogEntry,
    fixture: &BrowseFixture,
) -> String {
    let fixture_ref = format!("{}.{}", fixture.module, fixture.name);
    let fields: Vec<String> = entry
        .params
        .iter()
        .filter(|param| !param.is_body && record_has_field(&fixture.value, &param.name))
        .map(|param| {
            let value = match &param.kind {
                Some(_) => overlay_expr(param, &fixture_ref, fixture),
                None => format!("{fixture_ref}.{}", param.name),
            };
            format!("{}: {}", param.name, value)
        })
        .collect();
    let has_scalar_overlay = entry.params.iter().any(|param| {
        !param.is_body && param.kind.is_some() && record_has_field(&fixture.value, &param.name)
    });
    let bodies: Vec<&BrowseParam> = entry.params.iter().filter(|param| param.is_body).collect();
    let mut call_args = Vec::new();
    if entry.first_param_is_record {
        if !has_scalar_overlay {
            call_args.push(fixture_ref);
        } else {
            call_args.push(format!("{{ {} }}", fields.join(", ")));
        }
    } else {
        call_args.push(fixture_ref);
    }
    for param in bodies {
        call_args.push(value_expr(param));
    }
    format!("{module}.{}({})", entry.name, call_args.join(", "))
}

pub(crate) fn overlay_expr(
    param: &BrowseParam,
    fixture_ref: &str,
    fixture: &BrowseFixture,
) -> String {
    let quoted = roc_string(&param.name);
    let fallback = overlay_fallback(param, fixture_ref, fixture);
    match param.kind.as_ref().unwrap() {
        ParamKind::Str => format!("Query.arg_str(args, {quoted}) ?? {fallback}"),
        ParamKind::I64 => format!("Query.arg_i64(args, {quoted}) ?? {fallback}"),
        ParamKind::U64 => format!("Query.arg_u64(args, {quoted}) ?? {fallback}"),
        ParamKind::F64 => format!("Query.arg_f64(args, {quoted}) ?? {fallback}"),
        ParamKind::Dec => format!("Query.arg_dec(args, {quoted}) ?? {fallback}"),
        ParamKind::Bool => format!("Query.arg_bool(args, {quoted}) ?? {fallback}"),
        ParamKind::BodyHtml => {
            format!("Html.text(Query.arg_str(args, {quoted}) ?? {fallback})")
        }
    }
}

pub(crate) fn overlay_fallback(
    param: &BrowseParam,
    fixture_ref: &str,
    fixture: &BrowseFixture,
) -> String {
    let field = top_level_fields(&fixture.value)
        .into_iter()
        .find(|(name, _)| name == &param.name)
        .map(|(_, expr)| expr);
    match param.kind.as_ref().unwrap() {
        ParamKind::I64 => numeric_literal_fallback(field.as_deref(), "I64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::U64 => numeric_literal_fallback(field.as_deref(), "U64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::F64 => numeric_literal_fallback(field.as_deref(), "F64")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        ParamKind::Dec => numeric_literal_fallback(field.as_deref(), "Dec")
            .unwrap_or_else(|| format!("{fixture_ref}.{}", param.name)),
        _ => format!("{fixture_ref}.{}", param.name),
    }
}

pub(crate) fn numeric_literal_fallback(field: Option<&str>, suffix: &str) -> Option<String> {
    let bare = strip_num_suffix(field?.trim());
    let ok = match suffix {
        "I64" | "U64" => is_i64(bare),
        "F64" | "Dec" => is_i64(bare) || is_float(bare),
        _ => false,
    };
    ok.then(|| format!("{bare}.{suffix}"))
}

pub(crate) fn generate_runtime_call(module: &str, entry: &CatalogEntry) -> String {
    let props: Vec<&BrowseParam> = entry.params.iter().filter(|param| !param.is_body).collect();
    let bodies: Vec<&BrowseParam> = entry.params.iter().filter(|param| param.is_body).collect();
    let mut call_args = Vec::new();
    if entry.first_param_is_record {
        let fields: Vec<String> = props.iter().filter_map(|param| field_expr(param)).collect();
        if fields.is_empty() {
            call_args.push("{}".to_string());
        } else {
            call_args.push(format!("{{ {} }}", fields.join(", ")));
        }
    } else if let Some(param) = props.first() {
        call_args.push(value_expr(param));
    }
    for param in bodies {
        call_args.push(value_expr(param));
    }
    format!("{module}.{}({})", entry.name, call_args.join(", "))
}

pub(crate) fn field_expr(param: &BrowseParam) -> Option<String> {
    match &param.kind {
        Some(_) => Some(format!("{}: {}", param.name, value_expr(param))),
        None => param
            .default_roc
            .as_ref()
            .map(|default| format!("{}: {}", param.name, default)),
    }
}

pub(crate) fn value_expr(param: &BrowseParam) -> String {
    let Some(kind) = &param.kind else {
        return param
            .default_roc
            .clone()
            .unwrap_or_else(|| "Html.empty".to_string());
    };
    let fallback = param
        .default_roc
        .clone()
        .unwrap_or_else(|| kind.zero_roc().to_string());
    let quoted = roc_string(&param.name);
    match kind {
        ParamKind::Str => format!("Query.arg_str(args, {quoted}) ?? {fallback}"),
        ParamKind::I64 => format!("Query.arg_i64(args, {quoted}) ?? {fallback}"),
        ParamKind::U64 => format!("Query.arg_u64(args, {quoted}) ?? {fallback}"),
        ParamKind::F64 => format!("Query.arg_f64(args, {quoted}) ?? {fallback}"),
        ParamKind::Dec => format!("Query.arg_dec(args, {quoted}) ?? {fallback}"),
        ParamKind::Bool => format!("Query.arg_bool(args, {quoted}) ?? {fallback}"),
        ParamKind::BodyHtml => {
            format!("Html.text(Query.arg_str(args, {quoted}) ?? {fallback})")
        }
    }
}

pub(crate) fn generate_main_roc() -> String {
    let platform = crate::dispatch::default_platform_pin();
    let listed = [
        ListedRoute::new("GET", "/", "Browser.homePage"),
        ListedRoute::new("GET", "/c", "inspector"),
        ListedRoute::new("GET", "/preview", "preview"),
    ];
    let slash_arms = error_page::roc_slash_redirect_arms(&listed);
    let slash_binding = if slash_arms.is_empty() {
        String::new()
    } else {
        error_page::roc_redirect_slash_binding().to_string()
    };
    let mut out = format!(
        r#"app [Context, program] {{
    pf: platform "{platform}",
    http: "{HTTP_PKG}",
}}

import pf.Env
import pf.Path
import pf.Server
import http.Method
import http.Response
import Browser
import Catalog
import pf.Html
import Preview
import Query

Context : {{}}

program = {{ init!, respond!, shutdown! }}

init! : () => Try({{ config : Server.Config, context : Context }}, [Exit(I64), ..])
init! = || {{
    assets = Server.file_root({{
        id: "assets",
        path: Path.utf8("assets"),
    }})
    config =
        Server.default_config
        .with_listen({{ host: listen_host!({{}}), port: listen_port!({{}}) }})
        .with_file_roots([assets])
        .with_native_routes({{
            files: [
                Server.static_mount({{ at: "/assets", files: assets }}),
            ],
            liveness: [],
            readiness: [],
        }})
    Ok({{ config, context: {{}} }})
}}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, _context| {{
    path =
        match request.target() {{
            Resource({{ raw_path, .. }}) => raw_path
            _ => ""
        }}
    query =
        match request.target() {{
            Resource({{ raw_query: Present(q), .. }}) => q
            _ => ""
        }}
    args = Query.parse(query)
{slash_binding}
    match (Method.to_str(request.method()), path) {{
        ("GET", "/") => html_ok(Html.render(Browser.homePage({{ groups: Catalog.groups }})))
        ("GET", "/c") => inspector(args)
        ("GET", "/preview") => preview(args)
{slash_arms}{not_found}    }}
}}

shutdown! : Server.ShutdownReason, Context => Try({{}}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({{}})

inspector = |args| {{
    id = Query.arg_str(args, "id") ?? ""
    requested = Query.arg_str(args, "fixture") ?? ""
    match Catalog.find(id) {{
        Ok(selected) => {{
            chosen = chosen_fixture(selected, requested)
            html_ok(
                Html.render(
                    Browser.inspectorPage({{
                        groups: Catalog.groups,
                        selected: selected,
                        fields: fields(selected, args, chosen),
                        preview_url: preview_url(selected, args, chosen),
                        selected_fixture: chosen.name,
                    }}),
                ),
            )
        }}
        Err(_) => html_ok(Html.render(Browser.homePage({{ groups: Catalog.groups }})))
    }}
}}

preview = |args| {{
    id = Query.arg_str(args, "id") ?? ""
    html_ok(Html.render(Preview.render(id, args)))
}}

empty_fixture = {{ name: "", source: "", scalars: [] }}

chosen_fixture = |selected, requested|
    match List.get(List.keep_if(selected.fixtures, |item| item.name == requested), 0) {{
        Ok(found) => found
        Err(_) =>
            match List.get(selected.fixtures, 0) {{
                Ok(first) => first
                Err(_) => empty_fixture
            }}
    }}

fields = |selected, args, chosen|
    List.map(
        selected.params,
        |param| {{
            from_fixture =
                match List.get(List.keep_if(chosen.scalars, |item| item.name == param.name), 0) {{
                    Ok(item) => item.value
                    Err(_) => param.value
                }}
            {{
                name: param.name,
                required: param.required,
                kind: param.kind,
                value: Query.arg_str(args, param.name) ?? from_fixture,
            }}
        }},
    )

preview_url = |selected, args, chosen| {{
    fixture_q =
        if chosen.name == "" {{
            ""
        }} else {{
            "&fixture=${{Query.encode(chosen.name)}}"
        }}
    suffix =
        List.fold(
            selected.params,
            fixture_q,
            |acc, param| {{
                from_fixture =
                    match List.get(List.keep_if(chosen.scalars, |item| item.name == param.name), 0) {{
                        Ok(item) => item.value
                        Err(_) => param.value
                    }}
                value = Query.arg_str(args, param.name) ?? from_fixture
                "${{acc}}&${{Query.encode(param.name)}}=${{Query.encode(value)}}"
            }},
        )
    "/preview?id=${{Query.encode(selected.id)}}${{suffix}}"
}}

html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{{ name: "Content-Type", value: "text/html; charset=utf-8" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )
"#,
        not_found = error_page::roc_not_found_arm(),
    );
    out.push_str(&error_page::roc_runtime_helpers(&listed));
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out.push_str(serve::ROC_LISTEN_HOST_HELPER);
    out
}

pub(crate) fn roc_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

pub(crate) fn roc_bool(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}
