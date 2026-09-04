use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    ComponentInfo, Document, LowerOptions, ModuleItem, SourceFile, TemplateItem, camel_to_pascal,
    compile, component_matches, format_diagnostic,
};

use crate::datastar_asset;
use crate::error_page::{self, FailedFile, ListedRoute, MappedModule};
use crate::logs::{self, LogHub, LogLevel, Progress};
use crate::roc_module::{type_name_from_path, wrap_type_module};
use crate::serve;
use crate::style;

const HTTP_PKG: &str = "https://github.com/roc-lang/http/releases/download/1.0.0/6ZUwqYhCS8PU9Mo6MF7oV82ET2o7KYb57CLKDq4cq4sS.tar.zst";

#[allow(clippy::too_many_arguments)]
pub fn view(
    input: &Path,
    component: &str,
    raw_args: &[String],
    no_window: bool,
    port: serve::PortArg,
    live_reload: bool,
    verbose: bool,
    public: bool,
) -> Result<()> {
    if !input.is_file() {
        bail!("no such file: {}", input.display());
    }
    if input.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
        bail!(
            "unsupported file extension for `rocci view`: {}; expected a .rocci file",
            input.display()
        );
    }
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input.display().to_string();
    let source = SourceFile::new(&name, &src);
    let compiled = compile(source, &LowerOptions::default());
    let has_errors = compiled.has_errors();
    let inspect_page = crate::inspect::InspectPage::from_rocci_compile("/", &name, &src, &compiled);
    let wrap_in_shell = !component_is_html_document(&compiled.document, component);
    let roc = compiled.roc;
    let diagnostics = compiled.diagnostics;
    let components = compiled.components;
    let segments = compiled.segments;

    for diagnostic in &diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    if has_errors {
        let html = error_page::render_template_errors(&[FailedFile {
            name,
            src,
            diagnostics,
        }]);
        let title = format!("rocci view · {component}");
        let port = port.resolve()?;
        return serve::serve_html(port, 500, &html, &title, no_window, live_reload, public);
    }

    let info = find_component(&components, component).with_context(|| {
        let available = components
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if available.is_empty() {
            format!("component `{component}` not found (file has no components)")
        } else {
            format!("component `{component}` not found; available: {available}")
        }
    })?;

    let provided = parse_view_args(raw_args)?;
    let args = assign_args(&info.param_names, &info.optional_params, provided)?;
    let encoded: HashMap<String, String> = args
        .into_iter()
        .map(|(key, value)| (key, encode_roc_value(&value)))
        .collect();

    let type_name = type_name_from_path(input);
    let call = build_component_call(&type_name, info, &encoded);
    let src_dir = input.parent().unwrap_or_else(|| Path::new("."));
    let sibling_assets = src_dir.join("assets");
    let stage_version = datastar_asset::stage_version_for_dir(src_dir);

    let workspace = crate::driver::TempDir::create("view")?;
    copy_sibling_roc(src_dir, &workspace.path, &type_name)?;
    crate::driver::rewrite_workspace_runtime_imports(&workspace.path)?;
    let workspace_assets = workspace.path.join("assets");
    if sibling_assets.is_dir() {
        copy_tree(&sibling_assets, &workspace_assets)?;
    }
    if let Some(version) = &stage_version {
        datastar_asset::stage_into(&workspace_assets, version)?;
        datastar_asset::print_hint(version);
    } else {
        fs::create_dir_all(&workspace_assets)?;
    }
    fs::write(
        workspace.path.join(format!("{type_name}.roc")),
        wrap_type_module(
            &crate::dispatch::rewrite_runtime_imports_for_pin(&roc, None),
            &type_name,
        ),
    )
    .with_context(|| format!("failed to write {type_name}.roc"))?;
    fs::write(
        workspace.path.join("main.roc"),
        generate_main_roc(
            &type_name,
            &call,
            wrap_in_shell,
            &crate::dispatch::platform_pin_for_app_dir(&workspace.path),
        ),
    )
    .context("failed to write main.roc")?;

    let port = port.resolve()?;
    let url = format!("http://127.0.0.1:{port}/");
    let invocation = crate::driver::RocInvocation {
        program: "roc",
        app_dir: workspace.path.clone(),
        roc_file: PathBuf::from("main.roc"),
        args: Vec::new(),
    };
    Progress::from_verbose(verbose).step(logs::run_phase_start("roc", ""));
    let cmd = match crate::driver::prepare_roc_process(&invocation, port, public, verbose) {
        Ok(cmd) => cmd,
        Err(err) => {
            let html = error_page::render_roc_compile_error(&format!("{err:#}"), &[]);
            let title = format!("rocci view · {component}");
            return serve::serve_html(port, 500, &html, &title, no_window, live_reload, public);
        }
    };
    let logs = Arc::new(LogHub::new());
    let (mut child, mut tee) = serve::spawn_roc_with_logs(cmd, Some(logs.clone()))?;
    let title = format!("rocci view · {}", info.name);
    match serve::wait_for_roc(
        &mut child,
        &mut tee,
        port,
        "/",
        Progress::from_verbose(verbose),
    )? {
        serve::RocStart::Ready => {
            tee.flush_to_hub();
            logs::tee(
                &logs,
                LogLevel::Info,
                style::viewing(&format!("{} from {}", info.name, input.display()), &url),
            );
            let mut inspect = crate::inspect::InspectSnapshot::with_pages(
                crate::profile::ProfileSnapshot::default(),
                vec![inspect_page],
            );
            inspect.capture_html_from_origin(&format!("http://127.0.0.1:{port}"));
            serve::with_window_and_inspector(
                &mut child,
                &url,
                &title,
                no_window,
                live_reload,
                Some(inspect),
                None,
                Some(logs),
            )
        }
        serve::RocStart::Failed(output) => {
            let html = error_page::render_roc_compile_error(
                &output,
                &[MappedModule {
                    type_name: type_name.clone(),
                    generated: roc.clone(),
                    source_name: name,
                    source_src: src,
                    segments,
                }],
            );
            serve::serve_html(port, 500, &html, &title, no_window, live_reload, public)
        }
    }
}

pub(crate) fn find_component<'a>(
    components: &'a [ComponentInfo],
    name: &str,
) -> Option<&'a ComponentInfo> {
    components
        .iter()
        .find(|component| component.name == name || camel_to_pascal(&component.name) == name)
}

pub(crate) fn component_is_html_document(document: &Document, roc_name: &str) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Component(decl) if component_matches(&decl.name.name, roc_name) => {
            matches!(
                decl.body.items.iter().find(|item| !item.is_preamble()),
                Some(TemplateItem::Element(el)) if el.name.name == "html"
            )
        }
        _ => false,
    })
}

pub(crate) fn parse_view_args(raw: &[String]) -> Result<Vec<(String, String)>> {
    let mut args = Vec::new();
    let mut seen = HashMap::new();
    for item in raw {
        let Some((name, value)) = item.split_once('=') else {
            bail!("expected `--arg name=value`, got `{item}`");
        };
        if name.is_empty() {
            bail!("expected `--arg name=value`, got `{item}`");
        }
        if seen.insert(name.to_string(), ()).is_some() {
            bail!("duplicate argument `{name}`");
        }
        args.push((name.to_string(), value.to_string()));
    }
    Ok(args)
}

pub(crate) fn assign_args(
    param_names: &[String],
    optional_params: &[String],
    provided: Vec<(String, String)>,
) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    for (name, value) in provided {
        if !param_names.iter().any(|param| param == &name) {
            let expected = if param_names.is_empty() {
                "none".to_string()
            } else {
                param_names.join(", ")
            };
            bail!("unknown argument `{name}` (component expects: {expected})");
        }
        values.insert(name, value);
    }
    let missing: Vec<&str> = param_names
        .iter()
        .filter(|name| {
            !values.contains_key(*name) && !optional_params.iter().any(|optional| optional == *name)
        })
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        bail!(
            "missing argument{}: {} (required by component)",
            if missing.len() == 1 { "" } else { "s" },
            missing.join(", ")
        );
    }
    Ok(values)
}

pub(crate) fn encode_roc_value(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "true" {
        return "True".to_string();
    }
    if trimmed == "false" {
        return "False".to_string();
    }
    if is_number(trimmed) {
        return trimmed.to_string();
    }
    if let Some(first) = trimmed.chars().next()
        && matches!(first, '"' | '{' | '[' | '(')
    {
        return trimmed.to_string();
    }
    format!("\"{}\"", escape_roc_string(trimmed))
}

fn is_number(value: &str) -> bool {
    let rest = value.strip_prefix('-').unwrap_or(value);
    if rest.is_empty() || rest == "." {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for ch in rest.chars() {
        if ch == '.' {
            if seen_dot {
                return false;
            }
            seen_dot = true;
        } else if ch.is_ascii_digit() {
            seen_digit = true;
        } else {
            return false;
        }
    }
    seen_digit
}

fn escape_roc_string(value: &str) -> String {
    let mut out = String::new();
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
    out
}

pub(crate) fn build_component_call(
    type_name: &str,
    component: &ComponentInfo,
    args: &HashMap<String, String>,
) -> String {
    let prop_count = component.param_names.len() - component.body_params.len();
    let prop_names = &component.param_names[..prop_count];
    let mut call_args = Vec::new();
    if component.first_param_is_record {
        let fields = prop_names
            .iter()
            .filter_map(|name| {
                args.get(name)
                    .cloned()
                    .map(|value| format!("{name}: {value}"))
            })
            .collect::<Vec<_>>();
        if fields.is_empty() {
            call_args.push("{}".to_string());
        } else {
            call_args.push(format!("{{ {} }}", fields.join(", ")));
        }
    } else if let Some(name) = prop_names.first() {
        if let Some(value) = args.get(name) {
            call_args.push(value.clone());
        } else if let Some((_, default)) = component
            .param_defaults
            .iter()
            .find(|(param, _)| param == name)
        {
            call_args.push(default.clone());
        }
    }
    for name in &component.body_params {
        if let Some(value) = args.get(name) {
            call_args.push(value.clone());
        } else if let Some((_, default)) = component
            .param_defaults
            .iter()
            .find(|(param, _)| param == name)
        {
            call_args.push(default.clone());
        }
    }
    format!("{type_name}.{}({})", component.name, call_args.join(", "))
}

pub(crate) fn generate_main_roc(
    type_name: &str,
    call: &str,
    wrap_in_shell: bool,
    platform: &str,
) -> String {
    let render = if wrap_in_shell {
        format!(
            "Html.element(\n                \"html\",\n                [Html.attribute(\"lang\", \"en\")],\n                [\n                    Html.element(\n                        \"head\",\n                        [],\n                        [\n                            Html.void_element(\"meta\", [Html.attribute(\"charset\", \"utf-8\")]),\n                            Html.element(\"title\", [], [Html.text(\"rocci view\")]),\n                            Html.element(\"script\", [Html.attribute(\"type\", \"module\"), Html.attribute(\"src\", \"/assets/datastar.js\")], []),\n                        ],\n                    ),\n                    Html.element(\"body\", [], [{call}]),\n                ],\n            )"
        )
    } else {
        call.to_string()
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
import {type_name}
import pf.Html

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

    match (Method.to_str(request.method()), path) {{
        ("GET", "/") => html_ok(Html.render({render}))
{not_found}    }}
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
"#,
        not_found = error_page::roc_not_found_arm(),
    );
    out.push_str(&error_page::roc_runtime_helpers(&[ListedRoute::new(
        "GET", "/", "view",
    )]));
    out.push_str(serve::ROC_LISTEN_PORT_HELPER);
    out.push_str(serve::ROC_LISTEN_HOST_HELPER);
    out
}

pub(crate) fn copy_sibling_roc(src_dir: &Path, dest: &Path, type_name: &str) -> Result<()> {
    let skip = format!("{type_name}.roc");
    if !src_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("roc") {
            continue;
        }
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name == "main.roc" || name == skip {
            continue;
        }
        fs::copy(&path, dest.join(&file_name))
            .with_context(|| format!("failed to copy {}", path.display()))?;
    }
    Ok(())
}

fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    if from.is_file() {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(from, to)?;
        return Ok(());
    }
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        copy_tree(&entry.path(), &to.join(entry.file_name()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::Span;

    fn component(
        name: &str,
        param_names: &[&str],
        body_params: &[&str],
        optional_params: &[&str],
        first_param_is_record: bool,
    ) -> ComponentInfo {
        ComponentInfo {
            name: name.to_string(),
            body_params: body_params.iter().map(|s| s.to_string()).collect(),
            param_names: param_names.iter().map(|s| s.to_string()).collect(),
            optional_params: optional_params.iter().map(|s| s.to_string()).collect(),
            param_defaults: Vec::new(),
            param_types: Vec::new(),
            first_param_is_record,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn parse_args_splits_on_first_equals() {
        let parsed =
            parse_view_args(&["name=bart".into(), "person={ name: \"x\" }".into()]).unwrap();
        assert_eq!(
            parsed,
            vec![
                ("name".into(), "bart".into()),
                ("person".into(), "{ name: \"x\" }".into()),
            ]
        );
    }

    #[test]
    fn parse_args_rejects_duplicates_and_malformed() {
        assert!(parse_view_args(&["nope".into()]).is_err());
        assert!(parse_view_args(&["=value".into()]).is_err());
        assert!(parse_view_args(&["name=a".into(), "name=b".into()]).is_err());
    }

    #[test]
    fn assign_args_requires_all_and_rejects_unknown() {
        let params = vec!["name".into(), "count".into()];
        let none: Vec<String> = vec![];
        assert!(assign_args(&params, &none, vec![("name".into(), "bart".into())]).is_err());
        assert!(
            assign_args(
                &params,
                &none,
                vec![
                    ("name".into(), "bart".into()),
                    ("count".into(), "1".into()),
                    ("extra".into(), "x".into()),
                ]
            )
            .is_err()
        );
        let values = assign_args(
            &params,
            &none,
            vec![("name".into(), "bart".into()), ("count".into(), "1".into())],
        )
        .unwrap();
        assert_eq!(values.get("name").unwrap(), "bart");
        assert_eq!(values.get("count").unwrap(), "1");
    }

    #[test]
    fn assign_args_allows_omitting_defaults() {
        let params = vec!["name".into(), "count".into()];
        let optional = vec!["name".into()];
        let values = assign_args(&params, &optional, vec![("count".into(), "1".into())]).unwrap();
        assert!(!values.contains_key("name"));
        assert_eq!(values.get("count").unwrap(), "1");
        assert!(assign_args(&params, &optional, vec![("extra".into(), "x".into())]).is_err());
    }

    #[test]
    fn encode_roc_value_chooses_literals() {
        assert_eq!(encode_roc_value("bart"), "\"bart\"");
        assert_eq!(encode_roc_value("5"), "5");
        assert_eq!(encode_roc_value("-3"), "-3");
        assert_eq!(encode_roc_value("1.5"), "1.5");
        assert_eq!(encode_roc_value("true"), "True");
        assert_eq!(encode_roc_value("false"), "False");
        assert_eq!(encode_roc_value("\"already\""), "\"already\"");
        assert_eq!(encode_roc_value("{ name: \"x\" }"), "{ name: \"x\" }");
        assert_eq!(encode_roc_value("[1, 2]"), "[1, 2]");
        assert_eq!(encode_roc_value("say \"hi\""), "\"say \\\"hi\\\"\"");
    }

    #[test]
    fn builds_record_and_positional_calls() {
        let hello = component("hello", &["name"], &[], &[], true);
        let args = HashMap::from([("name".into(), "\"bart\"".into())]);
        assert_eq!(
            build_component_call("Foo", &hello, &args),
            "Foo.hello({ name: \"bart\" })"
        );

        let hello_default = component("hello", &["name"], &[], &["name"], true);
        let mut hello_default = hello_default;
        hello_default.param_defaults = vec![("name".into(), "\"Roc\"".into())];
        assert_eq!(
            build_component_call("Foo", &hello_default, &HashMap::new()),
            "Foo.hello({})"
        );

        let badge = component("badge", &["tone", "content"], &["content"], &[], true);
        let args = HashMap::from([
            ("tone".into(), "Positive".into()),
            ("content".into(), "Html.text(\"hi\")".into()),
        ]);
        assert_eq!(
            build_component_call("Foo", &badge, &args),
            "Foo.badge({ tone: Positive }, Html.text(\"hi\"))"
        );

        let model = component("modelView", &["model"], &[], &[], false);
        let args = HashMap::from([("model".into(), "{ count: 1 }".into())]);
        assert_eq!(
            build_component_call("Foo", &model, &args),
            "Foo.modelView({ count: 1 })"
        );

        let empty = component("empty", &[], &[], &[], true);
        assert_eq!(
            build_component_call("Foo", &empty, &HashMap::new()),
            "Foo.empty({})"
        );
    }

    #[test]
    fn generate_main_roc_renders_call_and_assets() {
        let main = generate_main_roc(
            "Foo",
            "Foo.hello({ name: \"bart\" })",
            true,
            crate::dispatch::IN_TREE_PLATFORM_PIN,
        );
        assert!(main.contains("import Foo"));
        assert!(main.contains("Html.render("));
        assert!(main.contains("Foo.hello({ name: \"bart\" })"));
        assert!(main.contains("Html.element(\"body\", [], [Foo.hello({ name: \"bart\" })])"));
        assert!(main.contains("import pf.Path"));
        assert!(main.contains("Server.static_mount"));
        assert!(main.contains("/assets/datastar.js"));
        assert!(main.contains("with_listen"));
        assert!(main.contains("ROC_BASIC_WEBSERVER_PORT"));
        assert!(main.contains("ROC_BASIC_WEBSERVER_HOST"));
        assert!(main.contains("host: listen_host!({})"));
        assert!(
            main.contains("crates/rocci-platform/platform/main.roc"),
            "{main}"
        );
        assert!(
            !main.contains("basic-webserver/releases/download/0.16.0"),
            "{main}"
        );
        assert!(main.contains("import pf.Html"), "{main}");

        let page = generate_main_roc(
            "Counter",
            "Counter.counterPage({ count: 0 })",
            false,
            crate::dispatch::IN_TREE_PLATFORM_PIN,
        );
        assert!(page.contains("import pf.Path"));
        assert!(page.contains("Server.static_mount"));
        assert!(page.contains("Html.render(Counter.counterPage({ count: 0 }))"));
        assert!(!page.contains("rocci view"));
        assert!(!page.contains("/assets/datastar.js"));
    }

    #[test]
    fn view_rejects_unsupported_extensions() {
        let temp_dir = std::env::temp_dir().join(format!("rocci-test-view-{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let md_file = temp_dir.join("test.md");
        fs::write(&md_file, "# Hello").unwrap();
        let err = view(
            &md_file,
            "main",
            &[],
            true,
            serve::PortArg::Auto,
            true,
            false,
            false,
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("unsupported file extension for `rocci view`"));
        assert!(err.contains("expected a .rocci file"));
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
