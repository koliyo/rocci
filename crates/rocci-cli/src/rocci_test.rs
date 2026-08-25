use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, SourceFile, TestInfo, compile, format_diagnostic, format_expect_trailer,
    type_name_from_path, wrap_type_module,
};

use crate::view::copy_sibling_roc;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    let staged = stage_type_module(&compiled.roc, &type_name, &compiled.tests);
    let workspace = unique_temp("test")?;
    fs::write(workspace.join("Html.roc"), rocci_ui::HTML_ROC)
        .with_context(|| format!("failed to write {}/Html.roc", workspace.display()))?;
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

pub fn stage_type_module(roc: &str, type_name: &str, tests: &[TestInfo]) -> String {
    let mut body = wrap_type_module(roc, type_name);
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(&format_expect_trailer(tests));
    body
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
        let staged = stage_type_module("helloSample = { name: \"Roc\" }\n", "Widget", &tests);
        assert!(staged.contains("Widget := [].{"));
        let type_end = staged.find("\n}\n").expect("type closer");
        let expect_at = staged.find("expect helloSample.name").expect("expect");
        assert!(expect_at > type_end);
        assert!(
            staged
                .contains("## Greeting for the sample name.\nexpect helloSample.name == \"Roc\"\n")
        );
    }

    #[test]
    fn rocci_test_tiny_fixture_when_roc_required() {
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() != Some("1") {
            return;
        }
        let dir = unique_temp("test-roc").unwrap();
        let path = dir.join("Hello.rocci");
        fs::write(
            &path,
            r#"
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
