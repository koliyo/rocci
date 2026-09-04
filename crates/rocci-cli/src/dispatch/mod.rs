use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rocci_template::{InitInfo, LiveInfo, RespondKind, RouteInfo};

use crate::error_page::{self, ListedRoute};
use crate::serve;

pub const BASIC_WEBSERVER_0_16_URL: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
pub const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";
pub const ROCCI_PLATFORM_NAME: &str = "rocci";
pub const GITHUB_PLATFORM_ARCHIVE: &str = "rocci-platform.tar.zst";

pub fn rocci_platform_main_roc() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../rocci-platform/platform/main.roc")
}

pub fn github_platform_release_tag() -> &'static str {
    option_env!("ROCCI_RELEASE_TAG").unwrap_or("dev")
}

pub fn github_platform_pin() -> String {
    format!(
        "https://github.com/koliyo/rocci/releases/download/{}/{}",
        github_platform_release_tag(),
        GITHUB_PLATFORM_ARCHIVE
    )
}

pub fn default_platform_pin() -> String {
    resolve_default_platform_pin(&rocci_platform_main_roc(), github_platform_pin())
}

fn resolve_default_platform_pin(path: &Path, fallback_url: String) -> String {
    if path.is_file() {
        path.canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .into_owned()
    } else {
        fallback_url
    }
}

pub fn uses_rocci_platform(platform: Option<&str>) -> bool {
    let pin = match platform.map(str::trim).filter(|value| !value.is_empty()) {
        Some(pin) => pin.to_string(),
        None => default_platform_pin(),
    };
    is_rocci_platform_path(&pin)
}

fn is_github_rocci_platform_pin(pin: &str) -> bool {
    let pin = pin.trim();
    pin.starts_with("https://github.com/koliyo/rocci/releases/download/")
        && pin.ends_with("/rocci-platform.tar.zst")
}

fn is_rocci_platform_path(pin: &str) -> bool {
    if is_github_rocci_platform_pin(pin) {
        return true;
    }
    let rocci = rocci_platform_main_roc();
    Path::new(pin) == rocci
        || pin.contains("crates/rocci-platform/platform/main.roc")
        || pin.ends_with("rocci-platform/platform/main.roc")
}

pub fn source_pins_rocci_platform(src: &str) -> bool {
    src.contains("rocci-platform/platform/main.roc")
}

pub fn rewrite_runtime_imports_for_pin(src: &str, platform: Option<&str>) -> String {
    if uses_rocci_platform(platform) {
        rewrite_runtime_imports_to_pf(src)
    } else {
        src.to_string()
    }
}

fn rewrite_runtime_imports_to_pf(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 16);
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed == "import Html" {
            out.push_str("import pf.Html\n");
        } else if trimmed == "import Datastar" {
            out.push_str("import pf.Datastar\n");
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn resolve_platform_pin(spec: Option<&str>) -> Result<Option<String>, String> {
    match spec.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(None),
        Some(ROCCI_PLATFORM_NAME) => Ok(Some(default_platform_pin())),
        Some(other) => Err(format!(
            "unknown --platform `{other}`; use `{ROCCI_PLATFORM_NAME}` for the in-tree Rocci platform"
        )),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DispatchSource<'a> {
    pub type_name: &'a str,
    pub routes: &'a [RouteInfo],
}

#[derive(Clone, Copy, Debug)]
pub struct LiveSource<'a> {
    pub type_name: &'a str,
    pub lives: &'a [LiveInfo],
}

pub fn merge_standalone_routes<'a>(
    primary: DispatchSource<'a>,
    siblings: &[DispatchSource<'a>],
) -> Vec<(&'a str, &'a RouteInfo)> {
    let mut bound = Vec::new();
    for route in primary.routes {
        bound.push((primary.type_name, route));
    }
    for sibling in siblings {
        for route in sibling.routes {
            bound.push((sibling.type_name, route));
        }
    }
    bound
}

pub fn merge_standalone_lives<'a>(
    primary: LiveSource<'a>,
    siblings: &[LiveSource<'a>],
) -> Vec<(&'a str, &'a LiveInfo)> {
    let mut bound = primary
        .lives
        .iter()
        .map(|live| (primary.type_name, live))
        .collect::<Vec<_>>();
    for sibling in siblings {
        bound.extend(sibling.lives.iter().map(|live| (sibling.type_name, live)));
    }
    bound
}

pub fn validate_standalone_dispatch(
    primary: DispatchSource<'_>,
    siblings: &[DispatchSource<'_>],
    primary_lives: LiveSource<'_>,
    sibling_lives: &[LiveSource<'_>],
) -> Result<(), String> {
    let mut seen = HashSet::new();
    for source in std::iter::once(primary).chain(siblings.iter().copied()) {
        for (method, path) in source
            .routes
            .iter()
            .map(|route| (route.method.as_str(), route.path.as_str()))
        {
            if !seen.insert((method, path)) {
                return Err(format!(
                    "duplicate app route `{method} {path}`; routes must be unique across primary and sibling modules"
                ));
            }
        }
    }
    for source in std::iter::once(primary_lives).chain(sibling_lives.iter().copied()) {
        for (method, path) in source
            .lives
            .iter()
            .map(|live| (live.method.as_str(), live.path.as_str()))
        {
            if !seen.insert((method, path)) {
                return Err(format!(
                    "duplicate app route `{method} {path}`; routes must be unique across primary and sibling modules"
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DispatchOptions {
    pub redirect_trailing_slash: bool,
    pub media_dirs: Vec<String>,
    pub log_handlers: bool,
    pub log_handlers_color: bool,
    pub platform: Option<String>,
}

impl Default for DispatchOptions {
    fn default() -> Self {
        Self {
            redirect_trailing_slash: true,
            media_dirs: Vec::new(),
            log_handlers: false,
            log_handlers_color: false,
            platform: None,
        }
    }
}

pub fn generate_bound_main_roc(
    type_name: &str,
    state_type: Option<&str>,
    init: Option<&InitInfo>,
    lives: &[(&str, &LiveInfo)],
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
    for (module, _) in lives {
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
    for (module, live) in lives {
        listed.push(ListedRoute::new(
            live.method.clone(),
            live.path.clone(),
            format!("{module}.{}", live.fn_name),
        ));
        arms.push_str(&live_sse_arm(module, live, options.log_handlers));
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
    let default_pin = default_platform_pin();
    let platform = options.platform.as_deref().unwrap_or(&default_pin);
    let html_datastar = if uses_rocci_platform(Some(platform)) {
        "import pf.Datastar\nimport pf.Html\n"
    } else {
        "import Datastar\nimport Html\n"
    };

    let mut out = format!(
        r#"app [Context, program] {{
    pf: platform "{platform}",
    http: "{HTTP_PKG}",
}}

import pf.Env
import pf.Path
import pf.Server
import pf.Sse
{stderr_import}import http.Method
import http.Response
{imports}{html_datastar}
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

empty_sse! = ||
    Ok(Server.stream(Sse.unfold!(0, |_state| Ok(End))))

no_content = ||
    Ok(
        Server.respond(
            Response.from_status(204),
        ),
    )

text_status = |status, body|
    Ok(
        Server.respond(
            Response.from_status(status)
            .with_headers([{{ name: "Content-Type", value: "text/plain; charset=utf-8" }}])
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
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out.push_str(serve::ROC_LISTEN_HOST_HELPER);
    out
}

#[cfg(test)]
pub fn json_encoder_probe_main_roc() -> String {
    let platform = default_platform_pin();
    let mut out = format!(
        r#"app [Context, program] {{
    pf: platform "{platform}",
    http: "{HTTP_PKG}",
}}

import pf.Env
import pf.Path
import pf.Server
import pf.Sse
import http.Method
import http.Response
import pf.Datastar
import pf.Html

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

fn live_sse_arm(type_name: &str, live: &LiveInfo, log_handlers: bool) -> String {
    let handler = format!("{type_name}.{}", live.fn_name);
    let ok_log = if log_handlers {
        format!(
            "handler_log!(\"{}\", \"{}\", \"ok\")\n            ",
            live.method, live.path
        )
    } else {
        String::new()
    };
    format!(
        r#"        ("{method}", "{path}") => {{
            {ok_log}Ok(
                Server.stream(
                    Sse.unfold!(
                        {{ html: "", quiet: 0 }},
                        |prev| {{
                            match {handler}(context, request) {{
                                Ok(html) => {{
                                rendered = Html.render(html)
                                    if rendered == prev.html {{
                                        if prev.quiet >= 149 {{
                                            event = Sse.Event.data("")
                                            Ok(Emit({{ event, state: {{ html: prev.html, quiet: 0 }}, wake: After(100) }}))
                                        }} else {{
                                            Ok(Wait({{ state: {{ html: prev.html, quiet: prev.quiet + 1 }}, wake: After(100) }}))
                                        }}
                                    }} else {{
                                        event = Datastar.patch_elements(html)
                                        Ok(Emit({{ event, state: {{ html: rendered, quiet: 0 }}, wake: After(100) }}))
                                    }}
                                }}
                                Err(err) => {{
                                    event = Datastar.patch_elements(error_overlay_html("{method}", "{path}", "{handler}", Str.inspect(err)))
                                    Ok(Emit({{ event, state: {{ html: prev.html, quiet: 0 }}, wake: After(100) }}))
                                }}
                            }}
                        }},
                    ),
                ),
            )
        }}
"#,
        handler = handler,
        method = live.method,
        path = live.path,
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
    match route.respond {
        RespondKind::Document => format!(
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
        ),
        RespondKind::Command => format!(
            r#"        ("{method}", "{path}") =>
            match {call} {{
                Ok({{}}) => {{
                {ok_log}if datastar_request(request) {{
                        empty_sse!()
                    }} else {{
                        no_content()
                    }}
                }}
                Err(err) => {{
                {err_log}if datastar_request(request) {{
                        Ok(patch_html!(error_overlay_html("{method}", "{path}", "{handler}", Str.inspect(err))))
                    }} else {{
                        text_status(500, "handler failed")
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
        ),
        RespondKind::Fragment => format!(
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
        ),
    }
}

#[cfg(test)]
mod tests;
