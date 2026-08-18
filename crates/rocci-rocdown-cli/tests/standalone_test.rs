use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rocci_cli::driver::GenericAppPlan;
use rocci_cli::error_page;
use rocci_rocdown::{
    StandaloneReady, ThemeArgs, ThemeOptions, linked_standalone_inputs, plan_standalone,
};

fn temp_app(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("rocdown-run-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn plan_ready(path: &Path) -> GenericAppPlan {
    match plan_standalone(path, &ThemeOptions::default()).unwrap() {
        StandaloneReady::Ready(plan) => GenericAppPlan {
            primary_name: plan.primary_name,
            modules: plan
                .modules
                .into_iter()
                .map(|m| rocci_cli::driver::GenericModule {
                    type_name: m.type_name,
                    roc: m.roc,
                    state_type: m.state_type,
                    init: m.init,
                    routes: m.routes,
                    mapped: m.mapped,
                    local_assets: m.local_assets,
                })
                .collect(),
            redirect_trailing_slash: plan.redirect_trailing_slash,
        },
        StandaloneReady::Failed(files) => {
            panic!(
                "expected successful compile, got {} failed file(s)",
                files.len()
            )
        }
    }
}

fn dispatch_handler<'a>(main: &'a str, method: &str, path: &str) -> &'a str {
    let needle = format!("(\"{method}\", \"{path}\") =>");
    let start = main
        .find(&needle)
        .unwrap_or_else(|| panic!("missing route {needle} in {main}"));
    let after = &main[start + needle.len()..];
    let match_at = after
        .find("match ")
        .unwrap_or_else(|| panic!("missing handler for {needle}"));
    after[match_at + "match ".len()..]
        .lines()
        .next()
        .unwrap()
        .trim()
        .trim_end_matches('{')
        .trim()
}

#[test]
fn linked_standalone_inputs_puts_primary_first() {
    let dir = temp_app("linked-inputs");
    let home = dir.join("Home.rocdown");
    let about = dir.join("About.rocdown");
    fs::write(&home, "").unwrap();
    fs::write(&about, "").unwrap();
    let inputs = linked_standalone_inputs(&home).unwrap();
    assert_eq!(inputs[0], home.canonicalize().unwrap());
    assert!(inputs.contains(&about.canonicalize().unwrap()));
    cleanup(&dir);
}

#[test]
fn standalone_rocdown_serves_sibling_page_routes() {
    let dir = temp_app("linked-pages");
    fs::write(
        dir.join("Home.rocdown"),
        r#"
@page { route: "/home/" }

# Home

See [[About]]
"#,
    )
    .unwrap();
    fs::write(
        dir.join("About.rocdown"),
        r#"
@page { route: "/about/" }

@on:get("/") = |_| {
    rocci_page({})
}

# About
"#,
    )
    .unwrap();
    let plan = plan_ready(&dir.join("Home.rocdown"));
    assert_eq!(plan.primary_name, "Home");
    assert!(
        plan.modules
            .iter()
            .any(|module| module.type_name == "About")
    );
    let main = plan.main_roc();
    assert!(main.contains("import Home"));
    assert!(main.contains("import About"));
    assert!(main.contains("(\"GET\", \"/about/\")"));
    assert!(main.contains("About.on_get_about!"));
    assert_eq!(
        dispatch_handler(&main, "GET", "/"),
        "Home.on_get_home!(context)"
    );
    assert_eq!(
        dispatch_handler(&main, "GET", "/about/"),
        "About.on_get_about!(context)"
    );
    assert!(main.contains("html_status(404, not_found_html("));
    assert!(!main.contains("Not found"));
    assert!(!main.contains("About.on_get_root!"));
    cleanup(&dir);
}

#[test]
fn standalone_mounts_document_relative_images() {
    let dir = temp_app("doc-rel-img");
    fs::create_dir_all(dir.join("img")).unwrap();
    fs::write(dir.join("img/dot.png"), b"png").unwrap();
    fs::write(
        dir.join("Page.rocdown"),
        r#"
@page { route: "/page/" }

@img {
    src: "./img/dot.png"
    alt: "Dot"
}
"#,
    )
    .unwrap();
    let plan = plan_ready(&dir.join("Page.rocdown"));
    assert!(
        plan.modules[0]
            .local_assets
            .iter()
            .any(|url| url == "./img/dot.png")
    );
    let main = plan.main_roc();
    assert!(main.contains("Path.utf8(\"media/img\")"), "{main}");
    assert!(main.contains("at: \"/img\""), "{main}");
    assert!(main.contains("at: \"/page/img\""), "{main}");
    cleanup(&dir);
}

#[test]
fn guide_example_serves_interactive_route() {
    let guide = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/rocdown/Guide.rocdown")
        .canonicalize()
        .unwrap();
    let plan = plan_ready(&guide);
    let main = plan.main_roc();
    assert!(main.contains("import Interactive"));
    assert_eq!(
        dispatch_handler(&main, "GET", "/guides/rocdown-interactive/"),
        "Interactive.on_get_guides_rocdown_interactive!(context)"
    );
    assert_eq!(
        dispatch_handler(&main, "GET", "/"),
        "Guide.on_get_guides_rocdown!(context)"
    );
    assert_eq!(
        dispatch_handler(&main, "POST", "/actions/reveal/show"),
        "Interactive.on_post_actions_reveal_show!(context)"
    );
    assert!(!main.contains("Interactive.on_get_root!"));
}

#[test]
fn errors_example_lists_error_demo_route_on_404() {
    let demo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/errors/ErrorDemo.rocdown")
        .canonicalize()
        .unwrap();
    let plan = plan_ready(&demo);
    let main = plan.main_roc();
    assert_eq!(
        dispatch_handler(&main, "GET", "/error-demo/"),
        "ErrorDemo.on_get_error_demo!(context)"
    );
    assert!(main.contains("html_status(404, not_found_html("));
    assert!(main.contains("/error-demo/"));
    assert!(main.contains("(\"GET\", \"/error-demo\") =>"));
    assert!(main.contains("redirect_slash(\"/error-demo/\")"));
    assert!(main.contains("Response.from_status(308)"));
    assert!(main.contains("\"/error-demo\" => Ok(\"/error-demo/\")"));
}

#[test]
fn errors_parse_example_builds_error_page() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/errors/parse/Broken.rocdown")
        .canonicalize()
        .unwrap();
    let StandaloneReady::Failed(files) = plan_standalone(&path, &ThemeOptions::default()).unwrap()
    else {
        panic!("expected template failure");
    };
    let failed_files: Vec<error_page::FailedFile> = files
        .into_iter()
        .map(|f| error_page::FailedFile {
            name: f.name,
            src: f.src,
            diagnostics: f.diagnostics,
        })
        .collect();
    let html = error_page::render_template_errors(&failed_files);
    assert!(html.contains("Broken.rocdown"));
    assert!(html.contains("@page"));
    assert!(html.contains("error"));
}

#[test]
fn errors_roc_example_compiles_as_template() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/errors/roc/BrokenRoc.rocdown")
        .canonicalize()
        .unwrap();
    let plan = plan_ready(&path);
    assert_eq!(plan.primary_name, "BrokenRoc");
    let html = error_page::render_roc_compile_error(
        "Found 1 error and 0 warnings for main.roc.\n── TYPE MISMATCH in BrokenRoc.roc:2 ──\n",
        &plan.maps(),
    );
    assert!(html.contains("Roc compile error"));
    assert!(html.contains("Found 1 error"));
    assert!(html.contains("BrokenRoc"));
}

#[test]
fn standalone_markdown_serves_themed_page() {
    let dir = temp_app("standalone-md");
    let path = dir.join("Plan.md");
    fs::write(
        &path,
        "# Implementation Plan\n\nThis is a plan.\n\n```roc\nmain = \"hello\"\n```\n",
    )
    .unwrap();
    let plan = plan_ready(&path);
    assert_eq!(plan.primary_name, "Plan");
    let main = plan.main_roc();
    assert!(main.contains("import Plan"));
    assert_eq!(
        dispatch_handler(&main, "GET", "/"),
        "Plan.on_get_root!(context)"
    );
    let module_roc = &plan.modules[0].roc;
    assert!(module_roc.contains("Implementation Plan"));
    assert!(module_roc.contains("rd-document"));
    assert!(module_roc.contains("data-rd-theme"));
    cleanup(&dir);
}

#[test]
fn standalone_markdown_supports_custom_theme() {
    let dir = temp_app("custom-theme-md");
    let path = dir.join("Plan.md");
    fs::write(&path, "# Custom Themed Plan\n\nSome body text.\n").unwrap();
    let theme_args = ThemeArgs {
        theme: Some("rocci".to_string()),
        color_scheme: Some("dark".to_string()),
    };
    let theme_options = theme_args.compile_options(Some(&path)).theme;
    let StandaloneReady::Ready(plan) = plan_standalone(&path, &theme_options).unwrap() else {
        panic!("expected standalone plan ready");
    };
    let module_roc = &plan.modules[0].roc;
    assert!(module_roc.contains("data-rd-theme"));
    assert!(module_roc.contains("rocci"));
    assert!(module_roc.contains("data-rd-color-scheme"));
    assert!(module_roc.contains("dark"));
    cleanup(&dir);
}

#[test]
fn linked_standalone_inputs_for_markdown_is_single_file() {
    let dir = temp_app("linked-md-inputs");
    let home = dir.join("Home.md");
    let about = dir.join("About.markdown");
    let guide = dir.join("Guide.rocdown");
    let other_guide = dir.join("Other.rocdown");
    fs::write(&home, "# Home").unwrap();
    fs::write(&about, "# About").unwrap();
    fs::write(&guide, "# Guide").unwrap();
    fs::write(&other_guide, "# Other Guide").unwrap();

    let md_inputs = linked_standalone_inputs(&home).unwrap();
    assert_eq!(md_inputs, vec![home.canonicalize().unwrap()]);

    let rocdown_inputs = linked_standalone_inputs(&guide).unwrap();
    assert_eq!(rocdown_inputs[0], guide.canonicalize().unwrap());
    assert!(rocdown_inputs.contains(&other_guide.canonicalize().unwrap()));
    assert!(!rocdown_inputs.contains(&home.canonicalize().unwrap()));
    cleanup(&dir);
}

#[test]
fn standalone_compile_failure_builds_error_page() {
    let dir = temp_app("compile-fail");
    let path = dir.join("Broken.rocdown");
    fs::write(&path, "@page {\n").unwrap();
    let StandaloneReady::Failed(files) = plan_standalone(&path, &ThemeOptions::default()).unwrap()
    else {
        cleanup(&dir);
        panic!("expected template failure");
    };
    let failed_files: Vec<error_page::FailedFile> = files
        .into_iter()
        .map(|f| error_page::FailedFile {
            name: f.name,
            src: f.src,
            diagnostics: f.diagnostics,
        })
        .collect();
    let html = error_page::render_template_errors(&failed_files);
    assert!(html.contains("Broken.rocdown") || html.contains("@page"));
    assert!(html.contains("error"));
    cleanup(&dir);
}
