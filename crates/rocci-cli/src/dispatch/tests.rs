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
        respond: if method == "GET" {
            rocci_template::RespondKind::Document
        } else {
            rocci_template::RespondKind::Fragment
        },
        span: Span::new(0, 0),
    }
}

fn route_command(method: &str, path: &str, fn_name: &str) -> RouteInfo {
    RouteInfo {
        method: method.to_string(),
        path: path.to_string(),
        fn_name: fn_name.to_string(),
        respond: rocci_template::RespondKind::Command,
        span: Span::new(0, 0),
    }
}

fn route_fragment(method: &str, path: &str, fn_name: &str) -> RouteInfo {
    RouteInfo {
        method: method.to_string(),
        path: path.to_string(),
        fn_name: fn_name.to_string(),
        respond: rocci_template::RespondKind::Fragment,
        span: Span::new(0, 0),
    }
}

fn live(path: &str, fn_name: &str) -> LiveInfo {
    LiveInfo {
        method: "GET".to_string(),
        path: path.to_string(),
        fn_name: fn_name.to_string(),
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
        &[],
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
    assert!(
        main.contains("crates/rocci-platform/platform/main.roc"),
        "{main}"
    );
    assert!(!main.contains(BASIC_WEBSERVER_0_16_URL), "{main}");
    assert!(main.contains("import pf.Datastar"), "{main}");
    assert!(main.contains("import pf.Html"), "{main}");
}

#[test]
fn http_module_platform_override_replaces_release_url() {
    let main = generate_bound_main_roc(
        "App",
        None,
        None,
        &[],
        &[],
        DispatchOptions {
            platform: Some("/tmp/fork/platform/main.roc".into()),
            ..DispatchOptions::default()
        },
    );
    assert!(
        main.contains("pf: platform \"/tmp/fork/platform/main.roc\""),
        "{main}"
    );
    assert!(!main.contains(BASIC_WEBSERVER_0_16_URL), "{main}");
    assert!(
        !main.contains("crates/rocci-platform/platform/main.roc"),
        "{main}"
    );
}

#[test]
fn rocci_platform_pin_writes_in_tree_path() {
    let pin = resolve_platform_pin(Some("rocci")).expect("resolve rocci pin");
    let pin = pin.expect("rocci pin");
    assert!(
        pin.contains("crates/rocci-platform/platform/main.roc"),
        "{pin}"
    );
    let main = generate_bound_main_roc(
        "App",
        None,
        None,
        &[],
        &[],
        DispatchOptions {
            platform: Some(pin.clone()),
            ..DispatchOptions::default()
        },
    );
    assert!(
        main.contains("crates/rocci-platform/platform/main.roc"),
        "{main}"
    );
    assert!(main.contains("import pf.Datastar"), "{main}");
    assert!(main.contains("import pf.Html"), "{main}");
    assert!(!main.contains("\nimport Datastar\n"), "{main}");
    assert!(!main.contains(BASIC_WEBSERVER_0_16_URL), "{main}");
}

#[test]
fn default_pin_uses_path_when_main_roc_exists() {
    let dir = env::temp_dir().join(format!("rocci-pin-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.roc");
    fs::write(&path, "platform \"rocci\"\n").unwrap();
    let pin = resolve_default_platform_pin(&path, "https://example.invalid/skip.tar.zst".into());
    assert!(pin.ends_with("main.roc"), "{pin}");
    assert!(!pin.starts_with("https://"), "{pin}");
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn default_pin_falls_back_to_url_when_path_missing() {
    let missing = env::temp_dir().join("rocci-no-platform/main.roc");
    let fallback = "https://github.com/koliyo/rocci/releases/download/dev/rocci-platform.tar.zst";
    let pin = resolve_default_platform_pin(&missing, fallback.into());
    assert_eq!(pin, fallback);
}

#[test]
fn github_platform_url_is_the_rocci_platform() {
    let url = github_platform_pin();
    assert!(
        url.starts_with("https://github.com/koliyo/rocci/releases/download/"),
        "{url}"
    );
    assert!(url.ends_with("/rocci-platform.tar.zst"), "{url}");
    assert!(uses_rocci_platform(Some(&url)), "{url}");
    let older = "https://github.com/koliyo/rocci/releases/download/v0.1.0/rocci-platform.tar.zst";
    assert!(uses_rocci_platform(Some(older)), "{older}");
    assert!(!uses_rocci_platform(Some(BASIC_WEBSERVER_0_16_URL)));
    let src = "import Html\nimport Datastar\n";
    let rewritten = rewrite_runtime_imports_for_pin(src, Some(&url));
    assert!(rewritten.contains("import pf.Html\n"), "{rewritten}");
    let main = generate_bound_main_roc(
        "App",
        None,
        None,
        &[],
        &[],
        DispatchOptions {
            platform: Some(url.clone()),
            ..DispatchOptions::default()
        },
    );
    assert!(main.contains(&format!("pf: platform \"{url}\"")), "{main}");
    assert!(main.contains("import pf.Datastar"), "{main}");
    assert!(!main.contains(BASIC_WEBSERVER_0_16_URL), "{main}");
}

#[test]
fn rocci_pin_rewrites_sibling_html_import() {
    let src = "import Html\nimport Datastar\nhello = 1\n";
    let rewritten = rewrite_runtime_imports_for_pin(src, None);
    assert!(rewritten.contains("import pf.Html\n"), "{rewritten}");
    assert!(rewritten.contains("import pf.Datastar\n"), "{rewritten}");
    assert!(!rewritten.contains("\nimport Html\n"), "{rewritten}");
    let kept = rewrite_runtime_imports_for_pin(src, Some(BASIC_WEBSERVER_0_16_URL));
    assert!(kept.contains("import Html\n"), "{kept}");
    assert!(kept.contains("import Datastar\n"), "{kept}");
}

#[test]
fn unknown_platform_pin_is_an_error() {
    let err = resolve_platform_pin(Some("basic-webserver")).unwrap_err();
    assert!(err.contains("unknown --platform"), "{err}");
}

#[test]
fn log_handlers_wraps_route_arms() {
    let main = generate_bound_main_roc(
        "Counter",
        None,
        None,
        &[],
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
        &[],
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
fn sibling_pages_keep_distinct_routes_and_duplicate_roots_are_rejected() {
    let primary = [
        route("GET", "/home/", "on_get_home!"),
        route("GET", "/", "on_get_root!"),
    ];
    let sibling = [
        route("GET", "/about/", "on_get_about!"),
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

    let main = generate_bound_main_roc("Home", None, None, &[], &bound, DispatchOptions::default());
    assert!(main.contains("import Home"));
    assert!(main.contains("import About"));
    assert!(main.contains("Home.on_get_root!(context, request)"));
    assert!(main.contains("About.on_get_about!(context, request)"));
    assert!(main.contains("About.on_post_actions_reveal_show!(context, request)"));
    assert!(main.contains("(\"GET\", \"/about/\")"));
    assert!(main.contains("match Home.on_get_root!(context, request)"));
    assert!(main.contains("(\"GET\", \"/about\") =>"));
    assert!(main.contains("redirect_slash(\"/about/\")"));
    assert!(main.contains("Response.from_status(308)"));

    let duplicate_root = [route("GET", "/", "on_get_sibling_root!")];
    let err = validate_standalone_dispatch(
        DispatchSource {
            type_name: "Home",
            routes: &primary,
        },
        &[DispatchSource {
            type_name: "About",
            routes: &duplicate_root,
        }],
        LiveSource {
            type_name: "Home",
            lives: &[],
        },
        &[],
    )
    .unwrap_err();
    assert!(err.contains("duplicate app route `GET /`"), "{err}");
}

#[test]
fn slash_redirect_can_be_disabled() {
    let main = generate_bound_main_roc(
        "Dx",
        None,
        None,
        &[],
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
    assert!(main.contains("Server.static_mount({ at: \"/nested/sub/assets\", files: assets })"));
    assert!(
        main.contains("Server.static_mount({ at: \"/nested/sub/path/assets\", files: assets })")
    );
}

#[test]
fn static_mounts_include_document_relative_media() {
    let main = generate_bound_main_roc(
        "Page",
        None,
        None,
        &[],
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
    let live = live("/events/counter", "on_get_events_counter!");
    let main = generate_bound_main_roc(
        "Counter",
        None,
        None,
        &[("Counter", &live)],
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
    assert!(main.contains("(\"GET\", \"/events/counter\")"), "{main}");
    assert!(
        main.contains("Counter.on_get_events_counter!(context, request)"),
        "{main}"
    );
    assert!(main.contains("Datastar.patch_elements"), "{main}");
    assert!(main.contains("Sse.Event.data(\"\")"), "{main}");
    assert!(main.contains("if prev.quiet >= 149"), "{main}");
    assert!(
        main.contains(
            "Ok(Wait({ state: { html: prev.html, quiet: prev.quiet + 1 }, wake: After(100) }))"
        ),
        "{main}"
    );
}

#[test]
fn live_logs_once_when_the_stream_opens() {
    let live = live("/events/counter", "on_get_events_counter!");
    let main = generate_bound_main_roc(
        "Counter",
        None,
        None,
        &[("Counter", &live)],
        &merge_standalone_routes(
            DispatchSource {
                type_name: "Counter",
                routes: &[route("GET", "/", "on_get_root!")],
            },
            &[],
        ),
        DispatchOptions {
            log_handlers: true,
            ..DispatchOptions::default()
        },
    );
    let log = "handler_log!(\"GET\", \"/events/counter\", \"ok\")";
    let stream = "Sse.unfold!";
    assert_eq!(main.matches(log).count(), 1, "{main}");
    assert!(
        main.find(log).unwrap() < main.find(stream).unwrap(),
        "{main}"
    );
}

#[test]
fn primary_and_sibling_live_routes_each_get_an_sse_arm() {
    let primary_lives = [live("/events/home", "on_get_events_home!")];
    let sibling_lives = [live("/events/shared", "on_get_events_shared!")];
    let bound_lives = merge_standalone_lives(
        LiveSource {
            type_name: "Home",
            lives: &primary_lives,
        },
        &[LiveSource {
            type_name: "Shared",
            lives: &sibling_lives,
        }],
    );
    let main = generate_bound_main_roc(
        "Home",
        None,
        None,
        &bound_lives,
        &[],
        DispatchOptions::default(),
    );
    assert!(main.contains("(\"GET\", \"/events/home\")"), "{main}");
    assert!(main.contains("(\"GET\", \"/events/shared\")"), "{main}");
    assert!(
        main.contains("Home.on_get_events_home!(context, request)"),
        "{main}"
    );
    assert!(
        main.contains("Shared.on_get_events_shared!(context, request)"),
        "{main}"
    );
}

#[test]
fn command_returns_empty_sse_or_no_content_without_json() {
    let main = generate_main_roc(
        "Counter",
        None,
        None,
        &[route_command(
            "POST",
            "/actions/counter/increment",
            "on_post_actions_counter_increment!",
        )],
    );
    assert!(main.contains("Datastar-Request"), "{main}");
    assert!(main.contains("datastar_request(request)"), "{main}");
    assert!(main.contains("empty_sse!()"), "{main}");
    assert!(main.contains("Ok(End)"), "{main}");
    assert!(main.contains("from_status(204)"), "{main}");
    assert!(main.contains("no_content()"), "{main}");
    assert!(
        main.contains("Counter.on_post_actions_counter_increment!(context, request)"),
        "{main}"
    );
    assert!(!main.contains("_json!"), "{main}");
    assert!(!main.contains("Encoding.Json.to_str_try(data)"), "{main}");
    assert!(!main.contains("json_ok"), "{main}");
    assert!(!main.contains("json_status"), "{main}");
    assert!(!main.contains("ApiError"), "{main}");
    assert!(!main.contains("application/json"), "{main}");
    assert!(
        main.contains("text_status(500, \"handler failed\")"),
        "{main}"
    );
    assert!(!main.contains("Ok(patch_html!(html))"), "{main}");
    assert!(!main.contains("{\"error\":\"${escaped}\"}"), "{main}");
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
fn get_fragment_uses_one_shot_patch_not_document_html() {
    let main = generate_main_roc(
        "Catalog",
        None,
        None,
        &[route_fragment(
            "GET",
            "/fragments/item",
            "on_get_fragments_item!",
        )],
    );
    assert!(main.contains("Ok(patch_html!(html))"), "{main}");
    assert!(!main.contains("html_ok(Html.render(html))"), "{main}");
}

#[test]
fn without_live_omits_sse_arm() {
    let main = generate_main_roc("Page", None, None, &[route("GET", "/", "on_get_root!")]);
    assert!(!main.contains("(\"GET\", \"/sse\")"), "{main}");
    assert!(!main.contains("Page.live!"), "{main}");
}

#[test]
fn route_and_live_collision_is_rejected() {
    let routes = [route("GET", "/events", "on_get_events!")];
    let lives = [live("/events", "on_get_events_live!")];
    let err = validate_standalone_dispatch(
        DispatchSource {
            type_name: "App",
            routes: &routes,
        },
        &[],
        LiveSource {
            type_name: "App",
            lives: &lives,
        },
        &[],
    )
    .unwrap_err();
    assert!(err.contains("duplicate app route `GET /events`"), "{err}");
}

#[test]
fn json_encoder_probe_uses_platform_imports_and_both_encoders() {
    let main = json_encoder_probe_main_roc();
    assert!(main.contains("rocci-platform/platform/main.roc"), "{main}");
    assert!(main.contains(HTTP_PKG), "{main}");
    assert!(main.contains("import pf.Server"), "{main}");
    assert!(main.contains("import pf.Datastar"), "{main}");
    assert!(main.contains("import pf.Html"), "{main}");
    assert!(!main.contains(BASIC_WEBSERVER_0_16_URL), "{main}");
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
    if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() != Some("1") {
        eprintln!("skipping: ROCCI_REQUIRE_ROC is not 1");
        return true;
    }
    let help_ok = Command::new("roc")
        .arg("help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !help_ok {
        panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
    }
    false
}
