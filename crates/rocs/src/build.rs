use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use rocci_template::{MappedModule, remap_roc_output};
use sha2::{Digest, Sha256};

use crate::BASIC_CLI_PLATFORM;
use crate::catalog;
use crate::plan::{self, BuildPlan};
use crate::runtime;
use crate::site::{LoadedSite, load_site, resolve_loaded};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn build(root: &Path, output: &Path) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    build_loaded(&loaded, &absolute(output)?)
}

pub fn build_configured(root: &Path, output_override: Option<&Path>) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    let output = match output_override {
        Some(output) => absolute(output)?,
        None => loaded.root.join(&loaded.config.build.output),
    };
    build_loaded(&loaded, &output)
}

pub(crate) fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn build_loaded(loaded: &LoadedSite, output: &Path) -> Result<BuildReport> {
    let plan = prepare_plan(loaded)?;
    let workspace = unique_temp("ws")?;
    let staging = unique_temp("stage")?;
    runtime::stage_into(&workspace)?;
    let staged = write_plan_files(&workspace, &staging, &plan)?;
    let maps = theme_maps(&plan);

    eprintln!(
        "rocs: generated {} bytes of Roc, compiling with roc",
        staged.generated_roc_bytes
    );
    let roc_started = Instant::now();
    let roc_output = invoke_roc_main(&workspace, &staging, &maps)
        .with_context(|| format!("workspace {}", workspace.display()))?;
    let roc_ms = roc_started.elapsed().as_millis();
    eprintln!("rocs: roc finished in {roc_ms}ms");
    if !roc_output.is_empty() {
        eprint!("{roc_output}");
    }

    write_planned_outputs(&staging, &plan)?;
    write_static_files(&staging, &loaded.static_files)?;
    commit_output(&staging, output)?;
    let _ = fs::remove_dir_all(&workspace);

    Ok(BuildReport {
        generated_roc_bytes: staged.generated_roc_bytes,
        roc_ms,
        recompiled: true,
    })
}

pub struct BuildSession {
    workspace: PathBuf,
    apply_bin: PathBuf,
    roc_hash: Option<String>,
}

impl BuildSession {
    pub fn create() -> Result<Self> {
        let workspace = unique_temp("ws")?;
        runtime::stage_into(&workspace)?;
        let apply_bin = workspace.join("apply");
        Ok(Self {
            workspace,
            apply_bin,
            roc_hash: None,
        })
    }

    pub fn rebuild(&mut self, root: &Path, output: &Path) -> Result<BuildReport> {
        let loaded = load_site(root)?;
        self.rebuild_loaded(&loaded, output)
    }

    pub fn rebuild_loaded(&mut self, loaded: &LoadedSite, output: &Path) -> Result<BuildReport> {
        let plan = prepare_plan(loaded)?;
        let staging = unique_temp("stage")?;
        let staged = write_plan_files(&self.workspace, &staging, &plan)?;
        let maps = theme_maps(&plan);
        let mut recompiled = false;

        if self.roc_hash.as_deref() != Some(&staged.roc_hash) || !self.apply_bin.is_file() {
            eprintln!(
                "rocs: generated {} bytes of Roc, compiling with roc",
                staged.generated_roc_bytes
            );
            let roc_started = Instant::now();
            let roc_output = invoke_roc_build(&self.workspace, &self.apply_bin, &maps)
                .with_context(|| format!("workspace {}", self.workspace.display()))?;
            let roc_ms = roc_started.elapsed().as_millis();
            eprintln!("rocs: roc finished in {roc_ms}ms");
            if !roc_output.is_empty() {
                eprint!("{roc_output}");
            }
            self.roc_hash = Some(staged.roc_hash.clone());
            recompiled = true;
        } else {
            eprintln!("rocs: content changed, applying without recompile");
        }

        let roc_started = Instant::now();
        let roc_output = invoke_apply(&self.apply_bin, &self.workspace, &staging, &maps)
            .with_context(|| format!("workspace {}", self.workspace.display()))?;
        let roc_ms = roc_started.elapsed().as_millis();
        if !roc_output.is_empty() {
            eprint!("{roc_output}");
        }

        write_planned_outputs(&staging, &plan)?;
        write_static_files(&staging, &loaded.static_files)?;
        commit_output(&staging, output)?;

        Ok(BuildReport {
            generated_roc_bytes: staged.generated_roc_bytes,
            roc_ms,
            recompiled,
        })
    }
}

impl Drop for BuildSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

fn prepare_plan(loaded: &LoadedSite) -> Result<BuildPlan> {
    let result = resolve_loaded(loaded);
    for diagnostic in &result.diagnostics {
        if diagnostic.severity == catalog::Severity::Warning {
            eprintln!("{diagnostic}");
        }
    }
    if result.has_errors() {
        bail!("{}", result.error_summary());
    }
    plan::plan(&loaded.root, &loaded.config, &result.site)
}

struct StagedBuild {
    generated_roc_bytes: usize,
    roc_hash: String,
}

fn write_plan_files(workspace: &Path, staging: &Path, plan: &BuildPlan) -> Result<StagedBuild> {
    let mut generated_roc_bytes = runtime::runtime_bytes();
    generated_roc_bytes += plan.theme_roc.len();

    let articles = workspace.join("articles");
    fs::create_dir_all(&articles).context("failed to create articles directory")?;
    for page in &plan.pages {
        fs::write(workspace.join(&page.article_path), &page.article_html)
            .with_context(|| format!("failed to write {}", page.article_path))?;
        if let Some(parent) = Path::new(&page.output_path).parent()
            && parent != Path::new("")
        {
            fs::create_dir_all(staging.join(parent))
                .with_context(|| format!("failed to create {}", staging.join(parent).display()))?;
        }
    }

    let pages_roc = plan.pages_roc();
    generated_roc_bytes += pages_roc.len();
    fs::write(workspace.join("RocsPages.roc"), &pages_roc)
        .context("failed to write RocsPages.roc")?;
    fs::write(workspace.join("RocsTheme.roc"), &plan.theme_roc)
        .context("failed to write RocsTheme.roc")?;
    let main = main_roc();
    generated_roc_bytes += main.len();
    fs::write(workspace.join("main.roc"), &main).context("failed to write main.roc")?;

    Ok(StagedBuild {
        generated_roc_bytes,
        roc_hash: roc_source_hash(&pages_roc, &plan.theme_roc, &main),
    })
}

fn write_static_files(staging: &Path, files: &[crate::site::StaticFile]) -> Result<()> {
    for file in files {
        let destination = staging.join(&file.output_path);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::copy(&file.source, &destination).with_context(|| {
            format!(
                "failed to copy {} to {}",
                file.source.display(),
                destination.display()
            )
        })?;
    }
    Ok(())
}

fn theme_maps(plan: &BuildPlan) -> Vec<MappedModule> {
    vec![MappedModule {
        type_name: "RocsTheme".into(),
        generated: plan.theme_roc.clone(),
        source_name: "RocsTheme.rocci".into(),
        source_src: plan.theme_src.clone(),
        segments: plan.theme_segments.clone(),
    }]
}

pub(crate) fn roc_source_hash(pages_roc: &str, theme_roc: &str, main_roc: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(runtime::HTML.as_bytes());
    hasher.update(runtime::BUILD.as_bytes());
    hasher.update(pages_roc.as_bytes());
    hasher.update(theme_roc.as_bytes());
    hasher.update(main_roc.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub generated_roc_bytes: usize,
    pub roc_ms: u128,
    pub recompiled: bool,
}

pub fn discover_rocdown(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.is_dir() {
        bail!("{} is not a directory", root.display());
    }
    let mut files = Vec::new();
    discover_in(root, &mut files)?;
    files.sort();
    if files.is_empty() {
        bail!("no .rocdown files in {}", root.display());
    }
    Ok(files)
}

fn discover_in(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|name| name == "assets") {
                continue;
            }
            discover_in(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rocdown") {
            files.push(path);
        }
    }
    Ok(())
}

fn main_roc() -> String {
    format!(
        "\
app [main!] {{ pf: platform \"{BASIC_CLI_PLATFORM}\" }}

import RocsBuild

main! = |_args| {{
    RocsBuild.run!({{}})?
    Ok({{}})
}}
"
    )
}

fn invoke_roc_main(workspace: &Path, staging: &Path, maps: &[MappedModule]) -> Result<String> {
    let output = Command::new("roc")
        .arg("main.roc")
        .current_dir(workspace)
        .env("ROCS_STAGING", staging)
        .output()
        .context("failed to invoke roc")?;
    finish_roc(output, maps)
}

fn invoke_roc_build(workspace: &Path, apply_bin: &Path, maps: &[MappedModule]) -> Result<String> {
    let output = Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--opt=dev")
        .arg(format!("--output={}", apply_bin.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build")?;
    let combined = finish_roc(output, maps)?;
    if !apply_bin.is_file() {
        bail!("roc build did not write {}", apply_bin.display());
    }
    Ok(combined)
}

fn invoke_apply(
    apply_bin: &Path,
    workspace: &Path,
    staging: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new(apply_bin)
        .current_dir(workspace)
        .env("ROCS_STAGING", staging)
        .output()
        .context("failed to run rocs applicator")?;
    finish_roc(output, maps)
}

fn finish_roc(output: std::process::Output, maps: &[MappedModule]) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = if stdout.is_empty() {
        stderr.clone()
    } else if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{stdout}{stderr}")
    };
    if output.status.success() {
        return Ok(combined);
    }
    let mapped = remap_roc_output(&combined, maps);
    for frame in mapped {
        eprintln!("{}", frame.render_for_stderr());
    }
    if !combined.trim().is_empty() {
        eprintln!("{}", combined.trim_end());
    }
    bail!(
        "roc rocs build failed{}",
        if combined.trim().is_empty() {
            String::new()
        } else {
            format!("\n{}", combined.trim_end())
        }
    );
}

fn write_planned_outputs(staging: &Path, plan: &BuildPlan) -> Result<()> {
    for asset in &plan.assets {
        let dest = staging.join(&asset.output_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&dest, &asset.bytes)
            .with_context(|| format!("failed to write {}", dest.display()))?;
    }
    for redirect in &plan.redirects {
        let dest = staging.join(&redirect.output_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&dest, &redirect.html)
            .with_context(|| format!("failed to write {}", dest.display()))?;
    }
    for file in &plan.files {
        fs::write(staging.join(&file.output_path), &file.contents)
            .with_context(|| format!("failed to write {}", file.output_path))?;
    }
    Ok(())
}

pub(crate) fn commit_output(staging: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let prev_name = match output.file_name() {
        Some(name) => {
            let mut prev = name.to_os_string();
            prev.push(".prev");
            prev
        }
        None => "output.prev".into(),
    };
    let prev = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(prev_name);
    if prev.exists() {
        fs::remove_dir_all(&prev).with_context(|| format!("failed to clear {}", prev.display()))?;
    }
    if output.exists() {
        fs::rename(output, &prev)
            .with_context(|| format!("failed to move {} aside", output.display()))?;
        if let Err(err) = fs::rename(staging, output) {
            let _ = fs::rename(&prev, output);
            return Err(err).with_context(|| {
                format!(
                    "failed to replace {} with staged rocs output",
                    output.display()
                )
            });
        }
        let _ = fs::remove_dir_all(&prev);
    } else if let Err(err) = fs::rename(staging, output) {
        return Err(err)
            .with_context(|| format!("failed to move staged rocs output to {}", output.display()));
    }
    Ok(())
}

pub(crate) fn unique_temp(kind: &str) -> Result<PathBuf> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("rocs-{kind}-{}-{n}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ROC_LOCK: Mutex<()> = Mutex::new(());

    fn skip_without_roc() -> bool {
        let available = Command::new("roc")
            .arg("help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if available {
            return false;
        }
        if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
            panic!("roc is required (ROCCI_REQUIRE_ROC=1) but was not found on PATH");
        }
        eprintln!("skipping: roc not on PATH");
        true
    }

    fn temp_dir(name: &str) -> PathBuf {
        unique_temp(name).unwrap()
    }

    fn write_page(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(name), body).unwrap();
    }

    fn collect_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut files = Vec::new();
        fn walk(dir: &Path, prefix: &Path, files: &mut Vec<(String, Vec<u8>)>) {
            let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
            entries.sort_by_key(|entry| entry.file_name());
            for entry in entries {
                let path = entry.path();
                let rel = prefix.join(entry.file_name());
                if path.is_dir() {
                    walk(&path, &rel, files);
                } else {
                    files.push((rel.to_string_lossy().into_owned(), fs::read(path).unwrap()));
                }
            }
        }
        walk(dir, Path::new(""), &mut files);
        files
    }

    #[test]
    fn two_page_build_writes_shell_and_escapes() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocs");
        let output = temp_dir("out");
        let report = build(&root, &output).unwrap();
        assert!(report.generated_roc_bytes > 0);
        let index = fs::read_to_string(output.join("index.html")).unwrap();
        let guide = fs::read_to_string(output.join("guide/index.html")).unwrap();
        assert!(index.contains("Ampersand &amp; Company"));
        assert!(!index.contains("Ampersand & Company"));
        assert!(guide.contains("Guide"));
        for html in [&index, &guide] {
            assert!(html.contains("skip-link"));
            assert!(html.contains("<main"));
            assert!(html.contains("id=\"main-content\"") || html.contains("id='main-content'"));
            assert!(html.contains("<!DOCTYPE html>"));
        }
        assert!(index.contains("href=\"/guide/\""));
        assert!(index.contains("class=\"rd-paragraph\""));
        for html in [&index, &guide] {
            assert!(html.contains("rel=\"stylesheet\""));
            assert!(html.contains("Content-Security-Policy"));
            assert!(!html.contains("<script"));
            let style_idx = html.find("<style");
            if let Some(idx) = style_idx {
                let window = &html[idx..idx.saturating_add(80).min(html.len())];
                assert!(
                    !window.contains(":scope") && !window.contains("--canvas"),
                    "theme CSS should not be inlined: {window}"
                );
            }
        }
        let not_found = fs::read_to_string(output.join("404.html")).unwrap();
        assert!(not_found.contains("skip-link"));
        assert!(not_found.contains("Page not found"));
        assert!(
            not_found.contains("id=\"main-content\"") || not_found.contains("id='main-content'")
        );
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn duplicate_routes_fail_in_catalog_and_preserve_output() {
        let root = temp_dir("dup-src");
        write_page(
            &root,
            "alpha.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Alpha\" } }\n\n# Alpha\n",
        );
        write_page(
            &root,
            "beta.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Beta\" } }\n\n# Beta\n",
        );
        let output = temp_dir("dup-out");
        fs::write(output.join("keep.txt"), "preserve me").unwrap();
        let err = build(&root, &output).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("alpha.rocdown"), "{message}");
        assert!(message.contains("beta.rocdown"), "{message}");
        assert!(message.contains("duplicate route"), "{message}");
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "preserve me"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn island_pages_are_rejected_without_roc() {
        let root = temp_dir("island-src");
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n@render {\n    Html.text(\"nope\")\n}\n",
        );
        let output = temp_dir("island-out");
        let err = build(&root, &output).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("@render"), "{message}");
        assert!(message.contains("islands"), "{message}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn repeat_build_is_byte_identical() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocs");
        let first = temp_dir("det-a");
        let second = temp_dir("det-b");
        build(&root, &first).unwrap();
        build(&root, &second).unwrap();
        assert_eq!(collect_files(&first), collect_files(&second));
        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    #[test]
    fn session_reuses_apply_binary_when_roc_sources_are_unchanged() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocs");
        let output = temp_dir("session-out");
        let mut session = BuildSession::create().unwrap();
        let first = session.rebuild(&root, &output).unwrap();
        assert!(first.recompiled);
        assert!(output.join("index.html").is_file());
        let second = session.rebuild(&root, &output).unwrap();
        assert!(!second.recompiled);
        assert!(output.join("index.html").is_file());
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn session_rebuild_failure_preserves_output() {
        let root = temp_dir("session-fail-src");
        write_page(
            &root,
            "alpha.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Alpha\" } }\n\n# Alpha\n",
        );
        write_page(
            &root,
            "beta.rocdown",
            "@page { route: \"/same/\", meta: { title: \"Beta\" } }\n\n# Beta\n",
        );
        let output = temp_dir("session-fail-out");
        fs::write(output.join("keep.txt"), "preserve me").unwrap();
        let mut session = BuildSession::create().unwrap();
        let err = session.rebuild(&root, &output).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("duplicate route"), "{message}");
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "preserve me"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }
}
