use std::collections::HashSet;
use std::path::Path;

use rocci_template::{InitInfo, LiveInfo, RespondKind, RouteInfo};

use crate::error_page::{self, ListedRoute};
use crate::serve;

pub const PLATFORM: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
pub const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";

#[derive(Clone, Copy, Debug)]
pub struct DispatchSource<'a> {
    pub type_name: &'a str,
    pub routes: &'a [RouteInfo],
}

pub fn merge_standalone_routes<'a>(
    primary: DispatchSource<'a>,
    siblings: &[DispatchSource<'a>],
) -> Vec<(&'a str, &'a RouteInfo)> {
    let mut bound = Vec::new();
    let mut seen = HashSet::new();
    for route in primary.routes {
        if seen.insert((route.method.as_str(), route.path.as_str())) {
            bound.push((primary.type_name, route));
        }
    }
    for sibling in siblings {
        for route in sibling.routes {
            if route.method == "GET" && route.path == "/" {
                continue;
            }
            if seen.insert((route.method.as_str(), route.path.as_str())) {
                bound.push((sibling.type_name, route));
            }
        }
    }
    bound
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchOptions {
    pub redirect_trailing_slash: bool,
    pub media_dirs: Vec<String>,
    pub log_handlers: bool,
    pub log_handlers_color: bool,
}

impl Default for DispatchOptions {
    fn default() -> Self {
        Self {
            redirect_trailing_slash: true,
            media_dirs: Vec::new(),
            log_handlers: false,
            log_handlers_color: false,
        }
    }
}

pub fn generate_bound_main_roc(
    type_name: &str,
    state_type: Option<&str>,
    init: Option<&InitInfo>,
    live: Option<&LiveInfo>,
    bound: &[(&str, &RouteInfo)],
    options: DispatchOptions,
) -> String {
    let context_ty = if state_type.is_some() {
        format!("{type_name}.State")
    } else {
        "{}".to_string()
    };
    let context_init = if init.is_some() {
        format!("{type_name}.init!() ? |_| Exit(2)")
    } else {
        "{}".to_string()
    };

    let mut imports = String::new();
    let mut imported = HashSet::new();
    for (module, _) in bound {
        if imported.insert(*module) {
            imports.push_str("import ");
            imports.push_str(module);
            imports.push('\n');
        }
    }
    if imported.insert(type_name) {
        imports.push_str("import ");
        imports.push_str(type_name);
        imports.push('\n');
    }

    let mut arms = String::new();
    let mut listed = Vec::new();
    let mut has_health = false;
    let has_sse = bound
        .iter()
        .any(|(_, route)| route.method == "GET" && route.path == "/sse");
    if live.is_some() && !has_sse {
        listed.push(ListedRoute::new(
            "GET",
            "/sse",
            format!("{type_name}.live!"),
        ));
        arms.push_str(&live_sse_arm(type_name, options.log_handlers));
    }
    for (module, route) in bound {
        if route.method == "GET" && route.path == "/health" {
            has_health = true;
        }
        listed.push(listed_route(module, route));
        arms.push_str(&route_arm(module, route, options.log_handlers));
    }
    if !has_health {
        listed.push(ListedRoute::new("GET", "/health", "health"));
        arms.push_str(
            r#"        ("GET", "/health") =>
            Ok(
                Server.respond(
                    Response.from_status(200)
                    .with_headers([{ name: "Content-Type", value: "text/plain; charset=utf-8" }])
                    .with_body(Str.to_utf8("ok")),
                ),
            )
"#,
        );
    }

    let (slash_binding, slash_arms) = if options.redirect_trailing_slash {
        let slash_arms = error_page::roc_slash_redirect_arms(&listed);
        let slash_binding = if slash_arms.is_empty() {
            String::new()
        } else {
            error_page::roc_redirect_slash_binding().to_string()
        };
        (slash_binding, slash_arms)
    } else {
        (String::new(), String::new())
    };

    let mut static_mounts = vec![
        "                Server.static_mount({ at: \"/assets\", files: assets }),".to_string(),
    ];
    let mut seen_mounts = HashSet::new();
    seen_mounts.insert("/assets".to_string());
    let mut file_root_bindings = String::from(
        r#"    assets = Server.file_root({
        id: "assets",
        path: Path.utf8("assets"),
    })
"#,
    );
    let mut file_root_ids = vec!["assets".to_string()];
    for dir in &options.media_dirs {
        let id = media_root_id(dir);
        file_root_bindings.push_str(&format!(
            "    {id} = Server.file_root({{\n        id: \"{id}\",\n        path: Path.utf8(\"media/{dir}\"),\n    }})\n"
        ));
        file_root_ids.push(id.clone());
        let suffix = format!("/{dir}");
        if seen_mounts.insert(suffix.clone()) {
            static_mounts.push(format!(
                "                Server.static_mount({{ at: \"{suffix}\", files: {id} }}),"
            ));
        }
    }
    for (_, route) in bound {
        let trimmed = route.path.trim_matches('/');
        if !trimmed.is_empty() {
            let mut accum = String::new();
            for segment in trimmed.split('/') {
                accum.push('/');
                accum.push_str(segment);
                let at = format!("{accum}/assets");
                if seen_mounts.insert(at.clone()) {
                    static_mounts.push(format!(
                        "                Server.static_mount({{ at: \"{at}\", files: assets }}),"
                    ));
                }
                for dir in &options.media_dirs {
                    let id = media_root_id(dir);
                    let at = format!("{accum}/{dir}");
                    if seen_mounts.insert(at.clone()) {
                        static_mounts.push(format!(
                            "                Server.static_mount({{ at: \"{at}\", files: {id} }}),"
                        ));
                    }
                }
            }
        }
    }
    let static_mounts_code = static_mounts.join("\n");
    let file_roots_list = file_root_ids.join(", ");

    let stderr_import = if options.log_handlers {
        "import pf.Stderr\n"
    } else {
        ""
    };
    let handler_log_helper = if options.log_handlers {
        handler_log_helper_roc(options.log_handlers_color)
    } else {
        ""
    };

    let mut out = format!(
        r#"app [Context, program] {{
    pf: platform "{PLATFORM}",
    http: "{HTTP_PKG}",
}}

import pf.Env
import pf.Path
import pf.Server
import pf.Sse
{stderr_import}import http.Method
import http.Response
{imports}import Datastar
import Html

Context : {context_ty}

program = {{ init!, respond!, shutdown! }}

init! : () => Try({{ config : Server.Config, context : Context }}, [Exit(I64), ..])
init! = || {{
    context = {context_init}
{file_root_bindings}    config =
        Server.default_config
        .with_listen({{ host: listen_host!({{}}), port: listen_port!({{}}) }})
        .with_file_roots([{file_roots_list}])
        .with_native_routes({{
            files: [
{static_mounts_code}
            ],
            liveness: [],
            readiness: [],
        }})
    Ok({{ config, context }})
}}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |request, context| {{
    path =
        match request.target() {{
            Resource({{ raw_path, .. }}) => raw_path
            _ => ""
        }}
{slash_binding}
    match (Method.to_str(request.method()), path) {{
{arms}{slash_arms}{not_found}    }}
}}

shutdown! : Server.ShutdownReason, Context => Try({{}}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({{}})
{handler_log_helper}
html_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{{ name: "Content-Type", value: "text/html; charset=utf-8" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )

patch_html! = |node| {{
    event = Datastar.patch_elements(node)
    Server.stream(
        Sse.unfold!(0, |state|
            match state {{
                0 => Ok(Emit({{ event, state: 1, wake: Immediately }}))
                _ => Ok(End)
            }}
        ),
    )
}}

no_content =
    Ok(
        Server.respond(
            Response.from_status(204)
            .with_body([]),
        ),
    )

json_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{{ name: "Content-Type", value: "application/json" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )

json_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{{ name: "Content-Type", value: "application/json" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )

datastar_request = |request|
    List.any(
        request.headers(),
        |header|
            (
                header.name == "datastar-request"
                or header.name == "Datastar-Request"
                or header.name == "DATASTAR-REQUEST"
            )
            and (
                header.value == "true"
                or header.value == "True"
                or header.value == "TRUE"
            ),
    )
"#,
        not_found = error_page::roc_not_found_arm(),
    );
    out.push_str(&error_page::roc_runtime_helpers(&listed));
    out.push_str(
        r#"
json_error_body = |err| {
    bs = html_join(Str.split_on(err, "\\"), "\\\\")
    escaped = html_join(Str.split_on(bs, "\""), "\\\"")
    "{\"error\":\"${escaped}\"}"
}
"#,
    );
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out.push_str(serve::ROC_LISTEN_HOST_HELPER);
    out
}

pub fn json_encoder_probe_main_roc() -> String {
    let mut out = format!(
        r#"app [Context, program] {{
    pf: platform "{PLATFORM}",
    http: "{HTTP_PKG}",
}}

import pf.Env
import pf.Path
import pf.Server
import pf.Sse
import http.Method
import http.Response
import Datastar
import Html

Probe : {{
    name : Str,
    ok : Bool,
    count : I64,
    maybe : [Some(Str), None],
    items : List(Str),
}}

Context : {{}}

program = {{ init!, respond!, shutdown! }}

init! : () => Try({{ config : Server.Config, context : Context }}, [Exit(I64), ..])
init! = || {{
    context = {{}}
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
    Ok({{ config, context }})
}}

respond! : Server.Request, Context => Try(Server.Outcome, [ServerErr(Str), ..])
respond! = |_request, _context| {{
    name =
        match Env.var_str!("PROBE_NAME") {{
            Ok(value) if value != "" => value
            _ => "handler"
        }}
    probe : Probe
    probe = {{
        name: name,
        ok: True,
        count: 42,
        maybe: Some("x"),
        items: ["a", "b"],
    }}
    total_record = Encoding.Json.to_str(probe)
    total_str = Encoding.Json.to_str("plain")
    record_body = Encoding.Json.to_str_try(probe) ?? total_record
    str_body = Encoding.Json.to_str_try("plain") ?? total_str
    json_ok("${{record_body}},${{str_body}}")
}}

shutdown! : Server.ShutdownReason, Context => Try({{}}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({{}})

json_ok = |body|
    Ok(
        Server.respond(
            Response.from_status(200)
            .with_headers([{{ name: "Content-Type", value: "application/json" }}])
            .with_body(Str.to_utf8(body)),
        ),
    )
"#,
    );
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out.push_str(serve::ROC_LISTEN_HOST_HELPER);
    out
}

fn handler_log_helper_roc(color: bool) -> &'static str {
    if color {
        r#"
handler_log! = |method, path, status| {
    reset = "\u(1b)[0m"
    method_sgr =
        match method {
            "GET" => "\u(1b)[1;32m"
            "HEAD" => "\u(1b)[32m"
            "POST" => "\u(1b)[1;33m"
            "PUT" => "\u(1b)[1;35m"
            "PATCH" => "\u(1b)[1;35m"
            "DELETE" => "\u(1b)[1;31m"
            _ => "\u(1b)[1;36m"
        }
    status_sgr =
        if status == "ok" {
            "\u(1b)[1;32m"
        } else {
            "\u(1b)[1;31m"
        }
    match Stderr.line!("${method_sgr}${method}${reset} ${path} -> ${status_sgr}${status}${reset}") {
        Ok({}) => {}
        Err(_) => {}
    }
}
"#
    } else {
        r#"
handler_log! = |method, path, status| {
    match Stderr.line!("${method} ${path} -> ${status}") {
        Ok({}) => {}
        Err(_) => {}
    }
}
"#
    }
}

fn media_root_id(dir: &str) -> String {
    let mut id = String::from("media_");
    for ch in dir.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch);
        } else {
            id.push('_');
        }
    }
    if id.ends_with('_') {
        id.push('x');
    }
    id
}

pub fn media_dirs_from_urls<'a, I>(urls: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a String>,
{
    let mut dirs = Vec::new();
    for url in urls {
        let Some(relative) = normalize_local_asset_url(url) else {
            continue;
        };
        let Some((dir, _)) = relative.split_once('/') else {
            continue;
        };
        if !dir.is_empty() && !dirs.iter().any(|existing| existing == dir) {
            dirs.push(dir.to_string());
        }
    }
    dirs.sort();
    dirs
}

fn normalize_local_asset_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty()
        || url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("//")
        || url.starts_with('/')
    {
        return None;
    }
    let path = Path::new(url);
    if path.is_absolute() {
        return None;
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => {
                parts.push(part.to_string_lossy().into_owned());
            }
            std::path::Component::ParentDir => {
                parts.pop()?;
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

fn listed_route(type_name: &str, route: &RouteInfo) -> ListedRoute {
    ListedRoute::new(
        route.method.clone(),
        route.path.clone(),
        format!("{type_name}.{}!", route.fn_name.trim_end_matches('!')),
    )
}

fn live_sse_arm(type_name: &str, log_handlers: bool) -> String {
    let handler = format!("{type_name}.live!");
    let ok_log = if log_handlers {
        "handler_log!(\"GET\", \"/sse\", \"ok\")\n                ".to_string()
    } else {
        String::new()
    };
    format!(
        r#"        ("GET", "/sse") =>
            Ok(
                Server.stream(
                    Sse.unfold!(
                        "",
                        |prev| {{
                            match {handler}(context, request) {{
                                Ok(html) => {{
                                {ok_log}rendered = Html.render(html)
                                    if rendered == prev {{
                                        Ok(Wait({{ state: prev, wake: After(100) }}))
                                    }} else {{
                                        event = Datastar.patch_elements(html)
                                        Ok(Emit({{ event, state: rendered, wake: After(100) }}))
                                    }}
                                }}
                                Err(err) => {{
                                    event = Datastar.patch_elements(error_overlay_html("GET", "/sse", "{handler}", Str.inspect(err)))
                                    Ok(Emit({{ event, state: prev, wake: After(100) }}))
                                }}
                            }}
                        }},
                    ),
                ),
            )
"#,
        handler = handler,
        ok_log = ok_log,
    )
}

fn route_arm(type_name: &str, route: &RouteInfo, log_handlers: bool) -> String {
    let handler = format!("{type_name}.{}!", route.fn_name.trim_end_matches('!'));
    let call = format!("{handler}(context, request)");
    let ok_log = if log_handlers {
        format!(
            "handler_log!(\"{method}\", \"{path}\", \"ok\")\n                ",
            method = route.method,
            path = route.path,
        )
    } else {
        String::new()
    };
    let err_log = if log_handlers {
        format!(
            "handler_log!(\"{method}\", \"{path}\", \"err\")\n                ",
            method = route.method,
            path = route.path,
        )
    } else {
        String::new()
    };
    if route.method == "GET" {
        format!(
            r#"        ("{method}", "{path}") =>
            match {call} {{
                Ok(html) => {{
                {ok_log}html_ok(Html.render(html))
                }}
                Err(err) => {{
                {err_log}html_status(500, handler_error_html("{method}", "{path}", "{handler}", Str.inspect(err)))
                }}
            }}
"#,
            method = route.method,
            path = route.path,
            call = call,
            handler = handler,
            ok_log = ok_log,
            err_log = err_log,
        )
    } else if route.respond == RespondKind::Json {
        format!(
            r#"        ("{method}", "{path}") =>
            match {call} {{
                Ok(body) => {{
                {ok_log}if datastar_request(request) {{
                    no_content
                }} else {{
                    json_ok(body)
                }}
                }}
                Err(err) => {{
                {err_log}if datastar_request(request) {{
                    Ok(patch_html!(error_overlay_html("{method}", "{path}", "{handler}", Str.inspect(err))))
                }} else {{
                    json_status(500, json_error_body(Str.inspect(err)))
                }}
                }}
            }}
"#,
            method = route.method,
            path = route.path,
            call = call,
            handler = handler,
            ok_log = ok_log,
            err_log = err_log,
        )
    } else {
        format!(
            r#"        ("{method}", "{path}") =>
            match {call} {{
                Ok(html) => {{
                {ok_log}Ok(patch_html!(html))
                }}
                Err(err) => {{
                {err_log}Ok(patch_html!(error_overlay_html("{method}", "{path}", "{handler}", Str.inspect(err))))
                }}
            }}
"#,
            method = route.method,
            path = route.path,
            call = call,
            handler = handler,
            ok_log = ok_log,
            err_log = err_log,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::Span;
    use std::env;
    use std::fs;
    use std::process::Command;
    use std::sync::Mutex;

    static ROC_LOCK: Mutex<()> = Mutex::new(());

    fn route(method: &str, path: &str, fn_name: &str) -> RouteInfo {
        RouteInfo {
            method: method.to_string(),
            path: path.to_string(),
            fn_name: fn_name.to_string(),
            respond: rocci_template::RespondKind::Patch,
            span: Span::new(0, 0),
        }
    }

    fn route_json(method: &str, path: &str, fn_name: &str) -> RouteInfo {
        RouteInfo {
            method: method.to_string(),
            path: path.to_string(),
            fn_name: fn_name.to_string(),
            respond: rocci_template::RespondKind::Json,
            span: Span::new(0, 0),
        }
    }

    fn generate_main_roc(
        type_name: &str,
        state_type: Option<&str>,
        init: Option<&InitInfo>,
        routes: &[RouteInfo],
    ) -> String {
        generate_bound_main_roc(
            type_name,
            state_type,
            init,
            None,
            &merge_standalone_routes(DispatchSource { type_name, routes }, &[]),
            DispatchOptions::default(),
        )
    }

    #[test]
    fn generates_state_init_get_and_patch_routes() {
        let init = InitInfo {
            span: Span::new(0, 0),
        };
        let main = generate_main_roc(
            "Counter",
            Some("{ db : Sqlite.Db }"),
            Some(&init),
            &[
                route("GET", "/", "on_get_root!"),
                route(
                    "POST",
                    "/actions/counter/increment",
                    "on_post_actions_counter_increment!",
                ),
            ],
        );
        assert!(main.contains("Context : Counter.State"));
        assert!(main.contains("context = Counter.init!() ? |_| Exit(2)"));
        assert!(main.contains("Counter.on_get_root!(context, request)"));
        assert!(main.contains("Counter.on_post_actions_counter_increment!(context, request)"));
        assert!(main.contains("html_ok(Html.render(html))"));
        assert!(main.contains("Ok(patch_html!(html))"));
        assert!(main.contains("html_status(404, not_found_html("));
        assert!(main.contains("handler_error_html(\"GET\", \"/\", \"Counter.on_get_root!\""));
        assert!(main.contains(
            "error_overlay_html(\"POST\", \"/actions/counter/increment\", \"Counter.on_post_actions_counter_increment!\""
        ));
        assert!(!main.contains("Not found"));
        assert!(main.contains("(\"GET\", \"/health\")"));
        assert!(main.contains("Datastar.patch_elements"));
        assert!(main.contains("ROC_BASIC_WEBSERVER_PORT"));
        assert!(main.contains("ROC_BASIC_WEBSERVER_HOST"));
        assert!(main.contains("host: listen_host!({})"));
        assert!(!main.contains("on_get_root!!"));
        assert!(!main.contains("import pf.Stderr"));
        assert!(!main.contains("handler_log!"));
    }

    #[test]
    fn log_handlers_wraps_route_arms() {
        let main = generate_bound_main_roc(
            "Counter",
            None,
            None,
            None,
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "Counter",
                    routes: &[route(
                        "POST",
                        "/actions/counter/increment",
                        "on_post_actions_counter_increment!",
                    )],
                },
                &[],
            ),
            DispatchOptions {
                log_handlers: true,
                ..DispatchOptions::default()
            },
        );
        assert!(main.contains("import pf.Stderr"), "{main}");
        assert!(main.contains("handler_log!"), "{main}");
        assert!(
            main.contains("handler_log!(\"POST\", \"/actions/counter/increment\", \"ok\")"),
            "{main}"
        );
        assert!(
            main.contains("handler_log!(\"POST\", \"/actions/counter/increment\", \"err\")"),
            "{main}"
        );
        assert!(!main.contains("\\u(1b)"), "{main}");
    }

    #[test]
    fn log_handlers_color_emits_ansi_escapes() {
        let main = generate_bound_main_roc(
            "Counter",
            None,
            None,
            None,
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "Counter",
                    routes: &[route(
                        "POST",
                        "/actions/counter/increment",
                        "on_post_actions_counter_increment!",
                    )],
                },
                &[],
            ),
            DispatchOptions {
                log_handlers: true,
                log_handlers_color: true,
                ..DispatchOptions::default()
            },
        );
        assert!(main.contains("\\u(1b)[1;33m"), "{main}");
        assert!(main.contains("\\u(1b)[1;32m"), "{main}");
        assert!(main.contains("\\u(1b)[1;31m"), "{main}");
        assert!(
            main.contains("handler_log!(\"POST\", \"/actions/counter/increment\", \"ok\")"),
            "{main}"
        );
    }

    #[test]
    fn empty_app_uses_unit_context() {
        let main = generate_main_roc("Page", None, None, &[]);
        assert!(main.contains("Context : {}"));
        assert!(main.contains("context = {}"));
        assert!(!main.contains("Page.init!"));
        assert!(main.contains("(\"GET\", \"/health\")"));
    }

    #[test]
    fn sibling_pages_keep_their_routes_but_not_root() {
        let primary = [
            route("GET", "/home/", "on_get_home!"),
            route("GET", "/", "on_get_root!"),
        ];
        let sibling = [
            route("GET", "/about/", "on_get_about!"),
            route("GET", "/", "on_get_root!"),
            route(
                "POST",
                "/actions/reveal/show",
                "on_post_actions_reveal_show!",
            ),
        ];
        let bound = merge_standalone_routes(
            DispatchSource {
                type_name: "Home",
                routes: &primary,
            },
            &[DispatchSource {
                type_name: "About",
                routes: &sibling,
            }],
        );
        let paths: Vec<_> = bound
            .iter()
            .map(|(module, route)| (*module, route.method.as_str(), route.path.as_str()))
            .collect();
        assert_eq!(
            paths,
            vec![
                ("Home", "GET", "/home/"),
                ("Home", "GET", "/"),
                ("About", "GET", "/about/"),
                ("About", "POST", "/actions/reveal/show"),
            ]
        );

        let main =
            generate_bound_main_roc("Home", None, None, None, &bound, DispatchOptions::default());
        assert!(main.contains("import Home"));
        assert!(main.contains("import About"));
        assert!(main.contains("Home.on_get_root!(context, request)"));
        assert!(main.contains("About.on_get_about!(context, request)"));
        assert!(main.contains("About.on_post_actions_reveal_show!(context, request)"));
        assert!(main.contains("(\"GET\", \"/about/\")"));
        assert!(main.contains("match Home.on_get_root!(context, request)"));
        assert!(!main.contains("About.on_get_root!"));
        assert!(main.contains("(\"GET\", \"/about\") =>"));
        assert!(main.contains("redirect_slash(\"/about/\")"));
        assert!(main.contains("Response.from_status(308)"));
    }

    #[test]
    fn slash_redirect_can_be_disabled() {
        let main = generate_bound_main_roc(
            "Dx",
            None,
            None,
            None,
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "Dx",
                    routes: &[route("GET", "/dx/", "on_get_dx!")],
                },
                &[],
            ),
            DispatchOptions {
                redirect_trailing_slash: false,
                ..DispatchOptions::default()
            },
        );
        assert!(!main.contains("redirect_slash("));
        assert!(!main.contains("from_status(308)"));
        assert!(main.contains("\"/dx\" => Ok(\"/dx/\")"));
    }

    #[test]
    fn registered_slash_pair_stays_distinct() {
        let main = generate_main_roc(
            "App",
            None,
            None,
            &[
                route("GET", "/dx", "on_get_dx!"),
                route("GET", "/dx/", "on_get_dx_slash!"),
            ],
        );
        assert!(!main.contains("redirect_slash(\"/dx\")"));
        assert!(!main.contains("redirect_slash(\"/dx/\")"));
        assert!(main.contains("redirect_slash(\"/health\")"));
        assert!(main.contains("(\"GET\", \"/dx\") =>"));
        assert!(main.contains("(\"GET\", \"/dx/\") =>"));
        assert!(main.contains("App.on_get_dx!(context, request)"));
        assert!(main.contains("App.on_get_dx_slash!(context, request)"));
    }

    #[test]
    fn static_mounts_include_route_prefixes() {
        let main = generate_main_roc(
            "Report",
            None,
            None,
            &[
                route(
                    "GET",
                    "/branding-and-community-foundation/",
                    "on_get_report!",
                ),
                route("GET", "/nested/sub/path/", "on_get_sub!"),
            ],
        );
        assert!(main.contains("Server.static_mount({ at: \"/assets\", files: assets })"));
        assert!(main.contains(
            "Server.static_mount({ at: \"/branding-and-community-foundation/assets\", files: assets })"
        ));
        assert!(main.contains("Server.static_mount({ at: \"/nested/assets\", files: assets })"));
        assert!(
            main.contains("Server.static_mount({ at: \"/nested/sub/assets\", files: assets })")
        );
        assert!(
            main.contains(
                "Server.static_mount({ at: \"/nested/sub/path/assets\", files: assets })"
            )
        );
    }

    #[test]
    fn static_mounts_include_document_relative_media() {
        let main = generate_bound_main_roc(
            "Page",
            None,
            None,
            None,
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "Page",
                    routes: &[route("GET", "/all-syntax/", "on_get_all_syntax!")],
                },
                &[],
            ),
            DispatchOptions {
                media_dirs: vec!["img".to_string()],
                ..DispatchOptions::default()
            },
        );
        assert!(main.contains("path: Path.utf8(\"media/img\")"), "{main}");
        assert!(
            main.contains("Server.static_mount({ at: \"/img\", files: media_img })"),
            "{main}"
        );
        assert!(
            main.contains("Server.static_mount({ at: \"/all-syntax/img\", files: media_img })"),
            "{main}"
        );
    }

    #[test]
    fn live_emits_sse_unfold_and_poll() {
        let live = LiveInfo {
            span: Span::new(0, 0),
        };
        let main = generate_bound_main_roc(
            "Counter",
            None,
            None,
            Some(&live),
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "Counter",
                    routes: &[route("GET", "/", "on_get_root!")],
                },
                &[],
            ),
            DispatchOptions::default(),
        );
        assert!(main.contains("Sse.unfold!"), "{main}");
        assert!(main.contains("After(100)"), "{main}");
        assert!(main.contains("(\"GET\", \"/sse\")"), "{main}");
        assert!(main.contains("Counter.live!(context, request)"), "{main}");
        assert!(main.contains("Datastar.patch_elements"), "{main}");
    }

    #[test]
    fn json_post_checks_datastar_request_header() {
        let main = generate_main_roc(
            "Counter",
            None,
            None,
            &[route_json(
                "POST",
                "/actions/counter/increment",
                "on_post_actions_counter_increment!",
            )],
        );
        assert!(main.contains("Datastar-Request"), "{main}");
        assert!(main.contains("datastar_request(request)"), "{main}");
        assert!(main.contains("no_content"), "{main}");
        assert!(main.contains("json_ok(body)"), "{main}");
        assert!(main.contains("application/json"), "{main}");
        assert!(!main.contains("Ok(patch_html!(html))"), "{main}");
    }

    #[test]
    fn unmarked_post_still_uses_patch_html() {
        let main = generate_main_roc(
            "Counter",
            None,
            None,
            &[route(
                "POST",
                "/actions/counter/increment",
                "on_post_actions_counter_increment!",
            )],
        );
        assert!(main.contains("Ok(patch_html!(html))"), "{main}");
        assert!(!main.contains("json_ok(body)"), "{main}");
    }

    #[test]
    fn without_live_omits_sse_arm() {
        let main = generate_main_roc("Page", None, None, &[route("GET", "/", "on_get_root!")]);
        assert!(!main.contains("(\"GET\", \"/sse\")"), "{main}");
        assert!(!main.contains("Page.live!"), "{main}");
    }

    #[test]
    fn live_does_not_duplicate_authored_sse_route() {
        let live = LiveInfo {
            span: Span::new(0, 0),
        };
        let main = generate_bound_main_roc(
            "App",
            None,
            None,
            Some(&live),
            &merge_standalone_routes(
                DispatchSource {
                    type_name: "App",
                    routes: &[route("GET", "/sse", "on_get_sse!")],
                },
                &[],
            ),
            DispatchOptions::default(),
        );
        assert_eq!(main.matches("(\"GET\", \"/sse\")").count(), 1, "{main}");
        assert!(main.contains("App.on_get_sse!(context, request)"), "{main}");
        assert!(!main.contains("App.live!(context, request)"), "{main}");
    }

    #[test]
    fn json_encoder_probe_uses_platform_imports_and_both_encoders() {
        let main = json_encoder_probe_main_roc();
        assert!(main.contains(PLATFORM), "{main}");
        assert!(main.contains(HTTP_PKG), "{main}");
        assert!(main.contains("import pf.Server"), "{main}");
        assert!(main.contains("import Datastar"), "{main}");
        assert!(main.contains("import Html"), "{main}");
        assert!(main.contains("Encoding.Json.to_str(probe)"), "{main}");
        assert!(main.contains("Encoding.Json.to_str_try(probe)"), "{main}");
        assert!(main.contains("Encoding.Json.to_str(\"plain\")"), "{main}");
        assert!(
            main.contains("Encoding.Json.to_str_try(\"plain\")"),
            "{main}"
        );
        assert!(!main.contains("{\\\"count\\\""), "{main}");
        assert!(!main.contains("${count.to_str()}"), "{main}");
    }

    #[test]
    fn json_encoder_probe_compiles_through_rocci_platform() {
        if skip_without_roc() {
            return;
        }
        let _guard = ROC_LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let workspace =
            crate::driver::TempDir::create("json-encoder-probe").expect("create probe workspace");
        crate::runtime_assets::stage_into(&workspace.path).expect("stage Html and Datastar");
        fs::create_dir_all(workspace.path.join("assets")).expect("create assets");
        fs::write(
            workspace.path.join("main.roc"),
            json_encoder_probe_main_roc(),
        )
        .expect("write probe main.roc");
        let output = workspace.path.join("server");
        crate::native_target::build_roc_server(&workspace.path, &output, None)
            .unwrap_or_else(|err| panic!("json encoder probe roc build failed: {err:#}"));
        assert!(output.is_file(), "probe roc build did not write a server");
    }

    fn skip_without_roc() -> bool {
        let help_ok = Command::new("roc")
            .arg("help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !help_ok {
            if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
                panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
            }
            eprintln!("skipping: roc not on PATH");
            return true;
        }
        false
    }
}
