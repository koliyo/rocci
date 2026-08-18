use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result, bail};
use rocci_roc_host::{InputFingerprint, NativeHost, compute_compile_hash, compute_gen_hash};
use rocci_template::{
    ComponentInfo, Document, FixtureInfo, LowerOptions, SourceFile, compile, format_diagnostic,
    type_name_from_path, wrap_type_module,
};

use crate::error_page::MappedModule;
use crate::view::{
    build_component_call, component_is_html_document, copy_sibling_roc, find_component,
};

const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";
pub const HTML_NO_TARGET_REASON: &str =
    "HTML preview needs a @fixture or a component whose required parameters all have defaults.";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlTarget {
    pub call: String,
    pub wrap_in_shell: bool,
    pub component_name: String,
}

pub fn select_html_target(
    document: &Document,
    components: &[ComponentInfo],
    fixtures: &[FixtureInfo],
    type_name: &str,
) -> Result<HtmlTarget, String> {
    if let Some(fixture) = fixtures.first() {
        let Some(component) = find_component(components, &fixture.target) else {
            return Err(format!(
                "HTML preview fixture `{}` targets `{}`, which was not found.",
                fixture.name, fixture.target
            ));
        };
        return Ok(HtmlTarget {
            call: format!("{type_name}.{}({})", component.name, fixture.value),
            wrap_in_shell: !component_is_html_document(document, &component.name),
            component_name: component.name.clone(),
        });
    }

    let Some(component) = components
        .iter()
        .find(|component| can_render_with_defaults(component))
    else {
        return Err(HTML_NO_TARGET_REASON.to_string());
    };

    Ok(HtmlTarget {
        call: build_component_call(type_name, component, &HashMap::new()),
        wrap_in_shell: !component_is_html_document(document, &component.name),
        component_name: component.name.clone(),
    })
}

fn can_render_with_defaults(component: &ComponentInfo) -> bool {
    component.param_names.iter().all(|name| {
        component
            .optional_params
            .iter()
            .any(|optional| optional == name)
            || component
                .param_defaults
                .iter()
                .any(|(param, _)| param == name)
    })
}

pub fn render_html_snapshot(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    document: &Document,
    components: &[ComponentInfo],
    fixtures: &[FixtureInfo],
    src_dir: Option<&Path>,
) -> Result<String, String> {
    render_html(
        filename, source, roc, segments, document, components, fixtures, src_dir, false,
    )
}

pub fn render_html_fragment(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    document: &Document,
    components: &[ComponentInfo],
    fixtures: &[FixtureInfo],
    src_dir: Option<&Path>,
) -> Result<String, String> {
    render_html(
        filename, source, roc, segments, document, components, fixtures, src_dir, true,
    )
}

fn render_html(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    document: &Document,
    components: &[ComponentInfo],
    fixtures: &[FixtureInfo],
    src_dir: Option<&Path>,
    fragment: bool,
) -> Result<String, String> {
    let type_name = type_name_from_path(Path::new(filename));
    let mut target = select_html_target(document, components, fixtures, &type_name)?;
    if fragment {
        target.wrap_in_shell = false;
    }
    stage_and_render(
        filename, source, roc, segments, &type_name, &target, src_dir,
    )
    .map_err(|err| err.to_string())
}

pub fn render_file(input: &Path, fragment: bool, output: Option<&Path>) -> Result<()> {
    if !input.is_file() {
        bail!("no such file: {}", input.display());
    }
    if input.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
        bail!(
            "unsupported file extension for `rocci render`: {}; expected a .rocci file",
            input.display()
        );
    }
    let src =
        fs::read_to_string(input).with_context(|| format!("failed to read {}", input.display()))?;
    let name = input
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("template.rocci")
        .to_string();
    let source = SourceFile::new(&name, &src);
    let compiled = compile(source, &LowerOptions::default());
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source, diagnostic));
    }
    if compiled.has_errors() {
        bail!("template compilation failed");
    }
    let src_dir = input
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let html = if fragment {
        render_html_fragment(
            &name,
            &src,
            &compiled.roc,
            &compiled.segments,
            &compiled.document,
            &compiled.components,
            &compiled.fixtures,
            src_dir,
        )
    } else {
        render_html_snapshot(
            &name,
            &src,
            &compiled.roc,
            &compiled.segments,
            &compiled.document,
            &compiled.components,
            &compiled.fixtures,
            src_dir,
        )
    }
    .map_err(|err| anyhow::anyhow!(err))?;
    match output {
        Some(path) => {
            let mut body = html;
            if !body.ends_with('\n') {
                body.push('\n');
            }
            if let Some(parent) = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))?;
        }
        None => print!("{html}"),
    }
    Ok(())
}

fn stage_and_render(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    type_name: &str,
    target: &HtmlTarget,
    src_dir: Option<&Path>,
) -> Result<String> {
    let workspace = unique_temp("playground-html")?;
    fs::write(workspace.join("Html.roc"), rocci_ui::HTML_ROC)
        .with_context(|| format!("failed to write {}/Html.roc", workspace.display()))?;
    if let Some(src_dir) = src_dir {
        copy_sibling_roc(src_dir, &workspace, type_name)?;
    }
    fs::write(
        workspace.join(format!("{type_name}.roc")),
        wrap_type_module(roc, type_name),
    )
    .with_context(|| format!("failed to write {type_name}.roc"))?;
    let main = generate_snapshot_main(type_name, &target.call, target.wrap_in_shell);
    fs::write(workspace.join("main.roc"), &main).context("failed to write main.roc")?;

    let wrapped = wrap_type_module(roc, type_name);
    let html_roc = rocci_ui::HTML_ROC;
    let gen_hash = compute_gen_hash(
        env!("CARGO_PKG_VERSION"),
        "playground-html",
        &[
            (&format!("{type_name}.roc"), wrapped.as_bytes()),
            ("main.roc", main.as_bytes()),
        ],
        &[("Html.roc", html_roc.as_bytes())],
    );
    let compile_hash = compute_compile_hash(
        &gen_hash,
        "roc",
        &format!("native:{}", env::consts::ARCH),
        "dev",
        BASIC_CLI_PLATFORM,
        env!("CARGO_PKG_VERSION"),
    );
    let fingerprints = [
        InputFingerprint::from_bytes(&format!("{type_name}.roc"), wrapped.as_bytes()),
        InputFingerprint::from_bytes("main.roc", main.as_bytes()),
        InputFingerprint::from_bytes("Html.roc", html_roc.as_bytes()),
    ];

    let host = NativeHost::default();
    let (apply_bin, _) = host
        .compile_or_cached(&workspace, &compile_hash, &fingerprints)
        .map_err(|err| {
            anyhow::anyhow!(annotate_roc_error(
                filename,
                source,
                roc,
                segments,
                type_name,
                &err.to_string(),
            ))
        })?;

    let output = Command::new(&apply_bin)
        .current_dir(&workspace)
        .output()
        .with_context(|| format!("failed to run {}", apply_bin.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        let combined = format!("{stdout}{stderr}");
        bail!(annotate_roc_error(
            filename, source, roc, segments, type_name, &combined,
        ));
    }

    let html = stdout.trim_end_matches(['\r', '\n']).to_string();
    if html.is_empty() {
        bail!("roc snapshot produced no HTML");
    }
    let _ = fs::remove_dir_all(&workspace);
    Ok(html)
}

fn annotate_roc_error(
    filename: &str,
    source: &str,
    roc: &str,
    segments: &[rocci_template::Segment],
    type_name: &str,
    output: &str,
) -> String {
    let mapped = rocci_template::remap_roc_output(
        output,
        &[MappedModule {
            type_name: type_name.to_string(),
            generated: roc.to_string(),
            source_name: filename.to_string(),
            source_src: source.to_string(),
            segments: segments.to_vec(),
        }],
    );
    if mapped.is_empty() {
        return format!("roc failed to render HTML:\n{}", output.trim());
    }
    let frames: Vec<String> = mapped.iter().map(|frame| frame.message.clone()).collect();
    format!(
        "roc failed to render HTML:\n{}\n{}",
        frames.join("\n"),
        output.trim()
    )
}

fn generate_snapshot_main(type_name: &str, call: &str, wrap_in_shell: bool) -> String {
    let render = if wrap_in_shell {
        format!(
            "Html.element(\n                \"html\",\n                [Html.attribute(\"lang\", \"en\")],\n                [\n                    Html.element(\n                        \"head\",\n                        [],\n                        [\n                            Html.void_element(\"meta\", [Html.attribute(\"charset\", \"utf-8\")]),\n                            Html.element(\"title\", [], [Html.text(\"playground\")]),\n                        ],\n                    ),\n                    Html.element(\"body\", [], [{call}]),\n                ],\n            )"
        )
    } else {
        call.to_string()
    };
    format!(
        r#"app [main!] {{ pf: platform "{BASIC_CLI_PLATFORM}" }}

import pf.Stdout
import {type_name}
import Html

main! = |_args| {{
    _ = Stdout.line!(Html.render({render}))
    Ok({{}})
}}
"#
    )
}

fn unique_temp(kind: &str) -> Result<PathBuf> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("rocci-{kind}-{}-{n}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_template::{LowerOptions, SourceFile, compile};

    fn compile_src(src: &str) -> rocci_template::CompileOutput {
        compile(SourceFile::new("Card.rocci", src), &LowerOptions::default())
    }

    #[test]
    fn selects_first_fixture() {
        let out = compile_src(
            r#"
@component Card = |{ title }| { <p>{title}</p> }

@fixture{target: Card}
sample = { title: "Hi" }
"#,
        );
        assert!(!out.has_errors());
        let target = select_html_target(&out.document, &out.components, &out.fixtures, "Card")
            .expect("fixture target");
        assert_eq!(target.component_name, "card");
        assert!(target.call.contains("Card.card({ title: \"Hi\" })"));
        assert!(target.wrap_in_shell);
    }

    #[test]
    fn selects_defaultable_component_without_fixture() {
        let out = compile_src(
            r#"
@component Hello = |{ name ?? "Roc" }| { <p>{name}</p> }
"#,
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let target = select_html_target(&out.document, &out.components, &out.fixtures, "Card")
            .expect("defaultable component");
        assert_eq!(target.component_name, "hello");
        assert!(target.call.contains("name:"));
        assert!(target.wrap_in_shell);
    }

    #[test]
    fn rejects_required_params_without_fixture() {
        let out = compile_src("@component Card = |{ title }| { <p>{title}</p> }");
        assert!(!out.has_errors());
        let err = select_html_target(&out.document, &out.components, &out.fixtures, "Card")
            .expect_err("required params");
        assert_eq!(err, HTML_NO_TARGET_REASON);
    }

    #[test]
    fn does_not_wrap_full_html_document() {
        let out = compile_src(
            r#"
@component Page = |{}| {
    <html lang="en">
        <head><title>Hi</title></head>
        <body><p>Hi</p></body>
    </html>
}
"#,
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let target = select_html_target(&out.document, &out.components, &out.fixtures, "Page")
            .expect("html document");
        assert!(!target.wrap_in_shell);
    }

    #[test]
    fn type_name_from_path_matches_view() {
        assert_eq!(type_name_from_path(Path::new("Counter.rocci")), "Counter");
    }

    fn skip_without_roc() -> bool {
        use std::process::Command;
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

    #[test]
    fn snapshots_fixture_component_html() {
        if skip_without_roc() {
            return;
        }
        let src = r#"
import Html

@component Hello = |{}| { <p>hello</p> }

@fixture{target: Hello}
helloTest = {}
"#;
        let out = compile_src(src);
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let html = render_html_snapshot(
            "Card.rocci",
            src,
            &out.roc,
            &out.segments,
            &out.document,
            &out.components,
            &out.fixtures,
            None,
        )
        .expect("html snapshot");
        assert!(html.contains("<p>hello</p>"), "{html}");
        assert!(html.contains("<html"), "{html}");
    }

    #[test]
    fn snapshots_fixture_component_fragment() {
        if skip_without_roc() {
            return;
        }
        let src = r#"
import Html

@component Hello = |{}| { <p>hello</p> }

@fixture{target: Hello}
helloTest = {}
"#;
        let out = compile_src(src);
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        let html = render_html_fragment(
            "Card.rocci",
            src,
            &out.roc,
            &out.segments,
            &out.document,
            &out.components,
            &out.fixtures,
            None,
        )
        .expect("html fragment");
        assert!(html.contains("<p>hello</p>"), "{html}");
        assert!(!html.contains("<html"), "{html}");
    }

    #[test]
    fn snapshot_main_prints_render() {
        let main = generate_snapshot_main("Card", "Card.hello({ name: \"x\" })", true);
        assert!(main.contains("import pf.Stdout"));
        assert!(main.contains("_ = Stdout.line!(Html.render("));
        assert!(main.contains("Card.hello({ name: \"x\" })"));
        assert!(main.contains("Html.element(\"body\""));
    }

    #[test]
    fn snapshot_main_fragment_skips_shell() {
        let main = generate_snapshot_main("Card", "Card.hello({})", false);
        assert!(main.contains("Html.render(Card.hello({}))"));
        assert!(!main.contains("Html.element(\"body\""));
    }
}
