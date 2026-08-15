use rocci_template::{InitInfo, RouteInfo};

use crate::serve;

pub const PLATFORM: &str = "https://github.com/roc-lang/basic-webserver/releases/download/0.16.0/42jC1JT3auhHSmv2Ah8mW5F2MXiAakq1UQQ4NQceQjXw.tar.zst";
pub const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";

pub fn generate_main_roc(
    type_name: &str,
    state_type: Option<&str>,
    init: Option<&InitInfo>,
    routes: &[RouteInfo],
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

    let mut arms = String::new();
    let mut has_health = false;
    for route in routes {
        if route.method == "GET" && route.path == "/health" {
            has_health = true;
        }
        arms.push_str(&route_arm(type_name, route));
    }
    if !has_health {
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
import {type_name}
import Datastar
import Html

Context : {context_ty}

program = {{ init!, respond!, shutdown! }}

init! : () => Try({{ config : Server.Config, context : Context }}, [Exit(I64), ..])
init! = || {{
    context = {context_init}
    assets = Server.file_root({{
        id: "assets",
        path: Path.utf8("assets"),
    }})
    config =
        Server.default_config
        .with_listen({{ host: "127.0.0.1", port: listen_port!({{}}) }})
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
respond! = |request, context| {{
    path =
        match request.target() {{
            Resource({{ raw_path, .. }}) => raw_path
            _ => ""
        }}

    match (Method.to_str(request.method()), path) {{
{arms}        _ =>
            Ok(
                Server.respond(
                    Response.from_status(404)
                    .with_body(Str.to_utf8("Not found")),
                ),
            )
    }}
}}

shutdown! : Server.ShutdownReason, Context => Try({{}}, [Exit(I64), ..])
shutdown! = |_reason, _context| Ok({{}})

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
"#
    );
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out
}

fn route_arm(type_name: &str, route: &RouteInfo) -> String {
    let call = format!(
        "{type_name}.{}!(context) ? |err| ServerErr(Str.inspect(err))",
        route.fn_name.trim_end_matches('!')
    );
    if route.method == "GET" {
        format!(
            r#"        ("{}", "{}") => {{
            html = {call}
            html_ok(Html.render(html))
        }}
"#,
            route.method, route.path
        )
    } else {
        format!(
            r#"        ("{}", "{}") => {{
            html = {call}
            Ok(patch_html!(html))
        }}
"#,
            route.method, route.path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::Span;

    fn route(method: &str, path: &str, fn_name: &str) -> RouteInfo {
        RouteInfo {
            method: method.to_string(),
            path: path.to_string(),
            fn_name: fn_name.to_string(),
            span: Span::new(0, 0),
        }
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
        assert!(main.contains("Counter.on_get_root!(context)"));
        assert!(main.contains("Counter.on_post_actions_counter_increment!(context)"));
        assert!(main.contains("html_ok(Html.render(html))"));
        assert!(main.contains("Ok(patch_html!(html))"));
        assert!(main.contains("(\"GET\", \"/health\")"));
        assert!(main.contains("Datastar.patch_elements"));
        assert!(main.contains("ROC_BASIC_WEBSERVER_PORT"));
        assert!(!main.contains("on_get_root!!"));
    }

    #[test]
    fn empty_app_uses_unit_context() {
        let main = generate_main_roc("Page", None, None, &[]);
        assert!(main.contains("Context : {}"));
        assert!(main.contains("context = {}"));
        assert!(!main.contains("Page.init!"));
        assert!(main.contains("(\"GET\", \"/health\")"));
    }
}
