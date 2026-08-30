use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    ComponentInfo, FixtureInfo, LowerOptions, ModuleItem, SourceFile, TestInfo, compile,
    format_diagnostic, format_expect_trailer, lower, pascal_to_camel, type_name_from_path,
    wrap_type_module,
};

use crate::view::copy_sibling_roc;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const DATASTAR_STUB: &str = r#"
Datastar := [].{
    get = |uri| uri
    post = |uri| uri
    put = |uri| uri
    patch = |uri| uri
    delete = |uri| uri
    get_with = |uri, _opts| uri
    post_with = |uri, _opts| uri
    put_with = |uri, _opts| uri
    patch_with = |uri, _opts| uri
    delete_with = |uri, _opts| uri
}
"#;

enum FileOutcome {
    Skipped,
    Passed,
}

pub fn run(path: &Path) -> Result<()> {
    let files = collect_rocci_files(path)?;
    let mut failed = false;
    for file in files {
        match run_file(&file) {
            Ok(FileOutcome::Skipped | FileOutcome::Passed) => {}
            Err(err) => {
                eprintln!("{}: {err}", file.display());
                failed = true;
            }
        }
    }
    if failed {
        bail!("rocci test failed");
    }
    Ok(())
}

fn collect_rocci_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) != Some("rocci") {
            bail!(
                "unsupported file extension for `rocci test`: {}; expected a .rocci file",
                path.display()
            );
        }
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        bail!("no such file or directory: {}", path.display());
    }
    let mut files = Vec::new();
    collect_dir(path, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<_, _>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rocci") {
            files.push(path);
        }
    }
    Ok(())
}

fn run_file(path: &Path) -> Result<FileOutcome> {
    let src =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let name = path
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
    if compiled.tests.is_empty() {
        return Ok(FileOutcome::Skipped);
    }
    let type_name = type_name_from_path(path);
    let mut document = compiled.document.clone();
    document.items.retain(|item| match item {
        ModuleItem::Context(_) | ModuleItem::Init(_) | ModuleItem::Route(_) => false,
        ModuleItem::Roc { span } => roc_item_safe_for_tests(source.src, *span),
        _ => true,
    });
    let lowered = lower(source, &document, &LowerOptions::default());
    let roc = rewrite_html_annos_for_string_runtime(&lowered.roc);
    let staged = stage_type_module(
        &roc,
        &type_name,
        &compiled.tests,
        &compiled.fixtures,
        &compiled.components,
    );
    let workspace = unique_temp("test")?;
    fs::write(workspace.join("Html.roc"), rocci_ui::HTML_ROC)
        .with_context(|| format!("failed to write {}/Html.roc", workspace.display()))?;
    fs::write(workspace.join("Datastar.roc"), DATASTAR_STUB)
        .with_context(|| format!("failed to write {}/Datastar.roc", workspace.display()))?;
    if let Some(src_dir) = path.parent() {
        copy_sibling_roc(src_dir, &workspace, &type_name)?;
    }
    let staged_path = workspace.join(format!("{type_name}.roc"));
    fs::write(&staged_path, &staged)
        .with_context(|| format!("failed to write {}", staged_path.display()))?;
    let output = Command::new("roc")
        .arg("test")
        .arg(&staged_path)
        .current_dir(&workspace)
        .output()
        .with_context(|| "failed to run `roc test`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    print!("{stdout}");
    eprint!("{stderr}");
    let _ = fs::remove_dir_all(&workspace);
    if !output.status.success() {
        bail!("roc test failed for {}", path.display());
    }
    Ok(FileOutcome::Passed)
}

pub fn stage_type_module(
    roc: &str,
    type_name: &str,
    tests: &[TestInfo],
    fixtures: &[FixtureInfo],
    components: &[ComponentInfo],
) -> String {
    let mut body = wrap_type_module(roc, type_name);
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(&format_test_aliases(type_name, fixtures, components));
    body.push_str(&format_expect_trailer(tests));
    body
}

fn roc_item_safe_for_tests(src: &str, span: rocci_template::Span) -> bool {
    let text = span.of(src);
    let trimmed = text.trim_start();
    if trimmed.starts_with("import ") {
        return true;
    }
    !text.contains("Sqlite.") && !text.contains("Env.") && !text.contains("Stderr.")
}

fn rewrite_html_annos_for_string_runtime(roc: &str) -> String {
    roc.replace(", Html -> Html", ", Str -> Str")
        .replace(" -> Html\n", " -> Str\n")
}

fn format_test_aliases(
    type_name: &str,
    fixtures: &[FixtureInfo],
    components: &[ComponentInfo],
) -> String {
    let mut names = Vec::new();
    for fixture in fixtures {
        names.push(fixture.name.clone());
    }
    for component in components {
        names.push(pascal_to_camel(&component.name));
    }
    names.sort();
    names.dedup();
    let mut out = String::new();
    for name in names {
        out.push_str(&name);
        out.push_str(" = ");
        out.push_str(type_name);
        out.push('.');
        out.push_str(&name);
        out.push('\n');
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out
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
    use rocci_template::Span;

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

    #[test]
    fn empty_directory_is_success() {
        let dir = unique_temp("test-empty").unwrap();
        run(&dir).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn skips_rocci_file_without_tests() {
        let dir = unique_temp("test-skip").unwrap();
        let path = dir.join("Widget.rocci");
        fs::write(
            &path,
            r#"
@component Hello = |{ name }| {
    <p>{name}</p>
}
"#,
        )
        .unwrap();
        run(&path).unwrap();
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn staged_module_keeps_expect_outside_type_body() {
        let tests = [TestInfo {
            name: "helloRenders".into(),
            fixture: Some("helloSample".into()),
            expr: r#"helloSample.name == "Roc""#.into(),
            docs: Some("## Greeting for the sample name.\n".into()),
            span: Span::point(0),
        }];
        let staged = stage_type_module(
            "helloSample = { name: \"Roc\" }\n",
            "Widget",
            &tests,
            &[],
            &[],
        );
        assert!(staged.contains("Widget := [].{"));
        let type_end = staged.find("\n}\n").expect("type closer");
        let expect_at = staged.find("expect helloSample.name").expect("expect");
        assert!(expect_at > type_end);
        assert!(
            staged
                .contains("## Greeting for the sample name.\nexpect helloSample.name == \"Roc\"\n")
        );
        let aliased = stage_type_module(
            "helloSample = { name: \"Roc\" }\n",
            "Widget",
            &tests,
            &[FixtureInfo {
                name: "helloSample".into(),
                target: "Hello".into(),
                value: r#"{ name: "Roc" }"#.into(),
                span: Span::point(0),
            }],
            &[],
        );
        let type_end = aliased.find("\n}\n").expect("type closer");
        let alias_at = aliased
            .find("helloSample = Widget.helloSample")
            .expect("alias");
        assert!(alias_at > type_end);
        assert!(alias_at < aliased.find("expect helloSample.name").unwrap());
    }

    #[test]
    fn rocci_test_tiny_fixture_when_roc_required() {
        if skip_without_roc() {
            return;
        }
        let dir = unique_temp("test-roc").unwrap();
        let path = dir.join("Hello.rocci");
        fs::write(
            &path,
            r#"
import Html

@fixture{target: Hello}
helloSample = { name: "Roc" }

@test{fixture: helloSample}
helloRenders = helloSample.name == "Roc"

@component Hello = |{ name }| {
    <p>{name}</p>
}
"#,
        )
        .unwrap();
        run(&path).expect("rocci test should pass when Roc is required");
        let _ = fs::remove_dir_all(&dir);
    }
}
