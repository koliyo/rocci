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

pub fn build_with_host(
    root: &Path,
    output: &Path,
    host: rocci_roc_host::HostChoice,
) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    build_loaded_with_host(&loaded, &absolute(output)?, host)
}

pub fn build_configured(root: &Path, output_override: Option<&Path>) -> Result<BuildReport> {
    build_configured_with_host(root, output_override, None)
}

pub fn build_configured_with_host(
    root: &Path,
    output_override: Option<&Path>,
    host_override: Option<rocci_roc_host::HostChoice>,
) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    let output = match output_override {
        Some(output) => absolute(output)?,
        None => loaded.root.join(&loaded.config.build.output),
    };
    let host = host_override
        .or(loaded.config.build.host)
        .unwrap_or_default()
        .resolve();
    build_loaded_with_host(&loaded, &output, host)
}

pub(crate) fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn build_loaded(loaded: &LoadedSite, output: &Path) -> Result<BuildReport> {
    let host = loaded.config.build.host.unwrap_or_default().resolve();
    build_loaded_with_host(loaded, output, host)
}

fn build_loaded_with_host(
    loaded: &LoadedSite,
    output: &Path,
    host: rocci_roc_host::HostChoice,
) -> Result<BuildReport> {
    let plan_started = Instant::now();
    let plan = prepare_plan(loaded)?;
    let plan_ms = plan_started.elapsed().as_millis();
    let workspace = unique_temp("ws")?;
    let staging = unique_temp("stage")?;
    runtime::stage_into(&workspace)?;
    let is_wasm = host.resolve() == rocci_roc_host::HostChoice::Wasm;
    let generate_started = Instant::now();
    let staged = write_plan_files(&workspace, &staging, &plan, is_wasm)?;
    let generate_ms = generate_started.elapsed().as_millis();
    let maps = theme_maps(&plan);
    let cache = rocci_roc_host::TwoTierCache::default();

    let target = if is_wasm {
        "wasm32".to_string()
    } else {
        format!("native:{}", env::consts::ARCH)
    };

    let apply_bin = workspace.join(if is_wasm { "components.wasm" } else { "apply" });
    let (apply_path, recompiled, compile_ms) =
        if let Some(cached) = cache.lookup_renderer(&staged.roc_hash, &target) {
            eprintln!(
                "rocdown: using cached {} renderer for {}",
                if is_wasm { "wasm" } else { "native" },
                &staged.roc_hash[..8.min(staged.roc_hash.len())]
            );
            (cached, false, 0)
        } else {
            eprintln!(
                "rocdown: generated {} bytes of Roc, compiling ({}) with roc",
                staged.generated_roc_bytes,
                if is_wasm { "wasm32" } else { "native" }
            );
            let roc_started = Instant::now();
            let roc_output = if is_wasm {
                invoke_roc_wasm_build(&workspace, &apply_bin, &maps)
                    .with_context(|| format!("workspace {}", workspace.display()))?
            } else {
                invoke_roc_build(&workspace, &apply_bin, &maps)
                    .with_context(|| format!("workspace {}", workspace.display()))?
            };
            let roc_ms = roc_started.elapsed().as_millis();
            eprintln!("rocdown: roc finished in {roc_ms}ms");
            if !roc_output.is_empty() {
                eprint!("{roc_output}");
            }
            let bytes = fs::read(&apply_bin)?;
            let fp = staged_fingerprints(&plan, is_wasm);
            let stored = cache.store_renderer(&staged.roc_hash, &target, &bytes, &fp)?;
            (stored, true, roc_ms)
        };

    let roc_started = Instant::now();
    let roc_output = if is_wasm {
        invoke_wasm_apply(&apply_path, &staging)?
    } else {
        invoke_apply(&apply_path, &workspace, &staging, &maps)
            .with_context(|| format!("workspace {}", workspace.display()))?
    };
    let roc_ms = roc_started.elapsed().as_millis();
    if !roc_output.is_empty() {
        eprint!("{roc_output}");
    }
    if !is_wasm {
        ensure_apply_wrote_pages(&staging, &plan)?;
    }

    let write_started = Instant::now();
    write_planned_outputs(&staging, &plan)?;
    write_static_files(&staging, &loaded.static_files)?;
    ensure_apply_wrote_pages(&staging, &plan)?;
    commit_output(&staging, output)?;
    let write_ms = write_started.elapsed().as_millis();
    let _ = fs::remove_dir_all(&workspace);

    Ok(BuildReport {
        generated_roc_bytes: staged.generated_roc_bytes,
        load_ms: 0,
        plan_ms,
        generate_ms,
        compile_ms,
        roc_ms,
        write_ms,
        recompiled,
    })
}

pub struct BuildSession {
    workspace: PathBuf,
    apply_bin: PathBuf,
    roc_hash: Option<String>,
    pub host: rocci_roc_host::HostChoice,
    pub snippet_paths: std::collections::BTreeSet<String>,
}

impl BuildSession {
    pub fn create() -> Result<Self> {
        Self::create_with_host(rocci_roc_host::HostChoice::Auto)
    }

    pub fn create_with_host(host: rocci_roc_host::HostChoice) -> Result<Self> {
        let workspace = unique_temp("ws")?;
        runtime::stage_into(&workspace)?;
        let host = host.resolve();
        let is_wasm = host == rocci_roc_host::HostChoice::Wasm;
        let apply_bin = workspace.join(if is_wasm { "components.wasm" } else { "apply" });
        Ok(Self {
            workspace,
            apply_bin,
            roc_hash: None,
            host,
            snippet_paths: std::collections::BTreeSet::new(),
        })
    }

    pub fn rebuild(&mut self, root: &Path, output: &Path) -> Result<BuildReport> {
        let load_started = Instant::now();
        let loaded = load_site(root)?;
        let load_ms = load_started.elapsed().as_millis();
        let mut report = self.rebuild_loaded(&loaded, output)?;
        report.load_ms = load_ms;
        Ok(report)
    }

    pub fn rebuild_loaded(&mut self, loaded: &LoadedSite, output: &Path) -> Result<BuildReport> {
        let host = if self.host == rocci_roc_host::HostChoice::Auto {
            loaded.config.build.host.unwrap_or_default().resolve()
        } else {
            self.host.resolve()
        };
        self.rebuild_loaded_with_host(loaded, output, host)
    }

    pub fn rebuild_loaded_with_host(
        &mut self,
        loaded: &LoadedSite,
        output: &Path,
        host: rocci_roc_host::HostChoice,
    ) -> Result<BuildReport> {
        let plan_started = Instant::now();
        let plan = prepare_plan(loaded)?;
        let plan_ms = plan_started.elapsed().as_millis();
        let staging = unique_temp("stage")?;
        let is_wasm = host == rocci_roc_host::HostChoice::Wasm;
        let generate_started = Instant::now();
        let staged = write_plan_files(&self.workspace, &staging, &plan, is_wasm)?;
        let generate_ms = generate_started.elapsed().as_millis();
        let maps = theme_maps(&plan);
        let cache = rocci_roc_host::TwoTierCache::default();
        let target = if is_wasm {
            "wasm32".to_string()
        } else {
            format!("native:{}", env::consts::ARCH)
        };
        let mut recompiled = false;
        let compile_ms;
        let apply_bin =
            if self.roc_hash.as_deref() == Some(&staged.roc_hash) && self.apply_bin.is_file() {
                eprintln!("rocdown: content changed, applying without recompile");
                compile_ms = 0;
                self.apply_bin.clone()
            } else if let Some(cached) = cache.lookup_renderer(&staged.roc_hash, &target) {
                eprintln!(
                    "rocdown: using cached {} renderer for {}",
                    if is_wasm { "wasm" } else { "native" },
                    &staged.roc_hash[..8.min(staged.roc_hash.len())]
                );
                self.roc_hash = Some(staged.roc_hash.clone());
                compile_ms = 0;
                cached
            } else {
                eprintln!(
                    "rocdown: generated {} bytes of Roc, compiling ({}) with roc",
                    staged.generated_roc_bytes,
                    if is_wasm { "wasm32" } else { "native" }
                );
                let roc_started = Instant::now();
                let roc_output = if is_wasm {
                    invoke_roc_wasm_build(&self.workspace, &self.apply_bin, &maps)
                        .with_context(|| format!("workspace {}", self.workspace.display()))?
                } else {
                    invoke_roc_build(&self.workspace, &self.apply_bin, &maps)
                        .with_context(|| format!("workspace {}", self.workspace.display()))?
                };
                compile_ms = roc_started.elapsed().as_millis();
                eprintln!("rocdown: roc finished in {compile_ms}ms");
                if !roc_output.is_empty() {
                    eprint!("{roc_output}");
                }
                self.roc_hash = Some(staged.roc_hash.clone());
                let bytes = fs::read(&self.apply_bin)?;
                let fp = staged_fingerprints(&plan, is_wasm);
                let stored = cache.store_renderer(&staged.roc_hash, &target, &bytes, &fp)?;
                recompiled = true;
                stored
            };

        let roc_started = Instant::now();
        let roc_output = if is_wasm {
            invoke_wasm_apply(&apply_bin, &staging)?
        } else {
            invoke_apply(&apply_bin, &self.workspace, &staging, &maps)
                .with_context(|| format!("workspace {}", self.workspace.display()))?
        };
        let roc_ms = roc_started.elapsed().as_millis();
        if !roc_output.is_empty() {
            eprint!("{roc_output}");
        }
        if !is_wasm {
            ensure_apply_wrote_pages(&staging, &plan)?;
        }

        let write_started = Instant::now();
        write_planned_outputs(&staging, &plan)?;
        write_static_files(&staging, &loaded.static_files)?;
        ensure_apply_wrote_pages(&staging, &plan)?;
        commit_output(&staging, output)?;
        let write_ms = write_started.elapsed().as_millis();
        self.snippet_paths = plan.snippet_paths.clone();

        Ok(BuildReport {
            generated_roc_bytes: staged.generated_roc_bytes,
            load_ms: 0,
            plan_ms,
            generate_ms,
            compile_ms,
            roc_ms,
            write_ms,
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

fn write_plan_files(
    workspace: &Path,
    staging: &Path,
    plan: &BuildPlan,
    is_wasm: bool,
) -> Result<StagedBuild> {
    let build_roc = runtime::build_roc(is_wasm);
    fs::write(workspace.join("RocdownBuild.roc"), build_roc)
        .context("failed to write RocdownBuild.roc")?;
    let mut generated_roc_bytes = runtime::HTML.len() + build_roc.len();
    for module in &plan.theme_modules {
        generated_roc_bytes += module.roc.len();
        fs::write(
            workspace.join(format!("{}.roc", module.type_name)),
            &module.roc,
        )
        .with_context(|| format!("failed to write {}.roc", module.type_name))?;
    }

    let articles = workspace.join("articles");
    fs::create_dir_all(&articles).context("failed to create articles directory")?;
    for page in &plan.pages {
        fs::write(workspace.join(&page.article_path), &page.article_html)
            .with_context(|| format!("failed to write {}", page.article_path))?;
        for (path, html) in &page.fragments {
            if let Some(parent) = Path::new(path).parent()
                && parent != Path::new("")
            {
                fs::create_dir_all(workspace.join(parent)).with_context(|| {
                    format!("failed to create {}", workspace.join(parent).display())
                })?;
            }
            fs::write(workspace.join(path), html)
                .with_context(|| format!("failed to write {path}"))?;
        }
        if let Some(parent) = Path::new(&page.output_path).parent()
            && parent != Path::new("")
        {
            fs::create_dir_all(staging.join(parent))
                .with_context(|| format!("failed to create {}", staging.join(parent).display()))?;
        }
    }

    if is_wasm {
        rocci_roc_host::stage_wasm_platform_into(workspace)?;
    }

    let pages_roc = plan.pages_roc();
    generated_roc_bytes += pages_roc.len();
    fs::write(workspace.join("RocdownPages.roc"), &pages_roc)
        .context("failed to write RocdownPages.roc")?;
    let main = main_roc(is_wasm);
    generated_roc_bytes += main.len();
    fs::write(workspace.join("main.roc"), &main).context("failed to write main.roc")?;

    Ok(StagedBuild {
        generated_roc_bytes,
        roc_hash: roc_source_hash(&pages_roc, &plan.theme_modules, &main, build_roc),
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
    plan.theme_modules
        .iter()
        .filter(|m| !m.segments.is_empty())
        .map(|m| MappedModule {
            type_name: m.type_name.clone(),
            generated: m.roc.clone(),
            source_name: m.source_name.clone(),
            source_src: m.src.clone(),
            segments: m.segments.clone(),
        })
        .collect()
}

fn staged_fingerprints(plan: &BuildPlan, is_wasm: bool) -> Vec<rocci_roc_host::InputFingerprint> {
    let mut fps = Vec::new();
    for module in &plan.theme_modules {
        fps.push(rocci_roc_host::InputFingerprint::from_bytes(
            &format!("{}.roc", module.type_name),
            module.roc.as_bytes(),
        ));
    }
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "Html.roc",
        runtime::HTML.as_bytes(),
    ));
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "RocdownBuild.roc",
        runtime::build_roc(is_wasm).as_bytes(),
    ));
    fps
}

pub(crate) fn roc_source_hash(
    pages_roc: &str,
    theme_modules: &[crate::plan::CompiledThemeModule],
    main_roc: &str,
    build_roc: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(runtime::HTML.as_bytes());
    hasher.update(build_roc.as_bytes());
    hasher.update(pages_roc.as_bytes());
    for m in theme_modules {
        hasher.update(m.type_name.as_bytes());
        hasher.update(m.roc.as_bytes());
    }
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
    pub load_ms: u128,
    pub plan_ms: u128,
    pub generate_ms: u128,
    pub compile_ms: u128,
    pub roc_ms: u128,
    pub write_ms: u128,
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

pub(crate) fn discover_in(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
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

fn main_roc(is_wasm: bool) -> String {
    if is_wasm {
        String::from(
            "\
app [main!] { pf: platform \"platform/main.roc\" }

main! : {} => [Ok({}), Err([Exit(I32)])]
main! = |{}| {
    res : [Ok({}), Err([Exit(I32)])]
    res = Ok({})
    res
}
",
        )
    } else {
        format!(
            "\
app [main!] {{ pf: platform \"{BASIC_CLI_PLATFORM}\" }}

import RocdownBuild

main! = |_args| {{
    RocdownBuild.run!({{}})?
    Ok({{}})
}}
"
        )
    }
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

fn invoke_roc_wasm_build(
    workspace: &Path,
    wasm_file: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--target=wasm32")
        .arg(format!("--output={}", wasm_file.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build for wasm32")?;
    let combined = finish_roc(output, maps)?;
    if !wasm_file.is_file() {
        bail!("roc build did not write {}", wasm_file.display());
    }
    Ok(combined)
}

fn invoke_wasm_apply(wasm_file: &Path, staging: &Path) -> Result<String> {
    let host = rocci_roc_host::WasmHost::from_file(wasm_file)?;
    host.run_wasi(staging)
}

fn invoke_apply(
    apply_bin: &Path,
    workspace: &Path,
    staging: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new(apply_bin)
        .current_dir(workspace)
        .env("ROCDOWN_STAGING", staging)
        .output()
        .context("failed to run rocdown applicator")?;
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
    let hint = if combined.contains("does not support the wasm32 target") {
        "\n\nhint: The basic-cli platform only supports native compilation targets (x64mac, arm64mac, x64win, x64musl, arm64musl).\nWasm host (--host wasm) is planned for Phase 5 with a custom Roc wasm platform.\nPlease use '--host native' (or default '--host auto') instead."
    } else {
        ""
    };
    bail!(
        "roc rocdown build failed{}{hint}",
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
    for page in &plan.pages {
        let dest = staging.join(&page.output_path);
        if dest.exists() {
            continue;
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let html = if page.article_html.contains("<!DOCTYPE") {
            page.article_html.clone()
        } else {
            format!("<!DOCTYPE html>\n{}", page.article_html)
        };
        fs::write(&dest, html).with_context(|| format!("failed to write {}", page.output_path))?;
    }
    Ok(())
}

pub(crate) fn ensure_apply_wrote_pages(staging: &Path, plan: &BuildPlan) -> Result<()> {
    ensure_page_files(
        staging,
        plan.pages.iter().map(|page| page.output_path.as_str()),
    )
}

fn ensure_page_files<'a>(
    staging: &Path,
    output_paths: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let missing: Vec<&str> = output_paths
        .into_iter()
        .filter(|path| !staging.join(path).is_file())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    bail!("apply did not write page HTML: {}", missing.join(", "))
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
                    "failed to replace {} with staged rocdown output",
                    output.display()
                )
            });
        }
        let _ = fs::remove_dir_all(&prev);
    } else if let Err(err) = fs::rename(staging, output) {
        return Err(err).with_context(|| {
            format!(
                "failed to move staged rocdown output to {}",
                output.display()
            )
        });
    }
    Ok(())
}

pub(crate) fn unique_temp(kind: &str) -> Result<PathBuf> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("rocdown-{kind}-{}-{n}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    pub(crate) static ROC_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn skip_without_roc() -> bool {
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
        let test_dir = env::temp_dir().join(format!("roc-probe-{}", std::process::id()));
        let _ = fs::create_dir_all(&test_dir);
        let probe_file = test_dir.join("main.roc");
        let _ = fs::write(
            &probe_file,
            "app [main!] { pf: platform \"https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst\" }\nmain! = |_| Ok({})\n",
        );
        let build_ok = Command::new("roc")
            .arg("build")
            .arg(&probe_file)
            .arg("--opt=dev")
            .current_dir(&test_dir)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let _ = fs::remove_dir_all(&test_dir);
        if !build_ok {
            if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
                panic!("roc compilation failed during environment probe");
            }
            eprintln!("skipping: roc compilation not functional in this environment");
            return true;
        }
        false
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
    fn ensure_page_files_errors_when_html_is_missing() {
        let staging = temp_dir("missing-html");
        fs::write(staging.join("index.html"), "<!DOCTYPE html>").unwrap();
        let err = ensure_page_files(&staging, ["index.html", "404.html", "guide/index.html"])
            .unwrap_err()
            .to_string();
        assert!(err.contains("apply did not write page HTML"), "{err}");
        assert!(err.contains("404.html"), "{err}");
        assert!(err.contains("guide/index.html"), "{err}");
        assert!(!err.contains("index.html,"), "{err}");
        let _ = fs::remove_dir_all(&staging);
    }

    #[test]
    fn two_page_build_writes_shell_and_escapes() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown-site");
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
    fn docs_components_render_asides_tabs_and_includes() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("docs-src");
        fs::write(
            root.join("snippet.rs"),
            "// docs-region: hello\nfn hello() {}\n// docs-region-end: hello\n",
        )
        .unwrap();
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n\n:tabs[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:end.tabs\n\n:include[path: \"snippet.rs\", region: \"hello\"]\n",
        );
        let output = temp_dir("docs-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("data-rocci-docs=\"note\""), "{html}");
        assert!(html.contains("rd-docs-label"), "{html}");
        assert!(
            html.contains("<p class=\"rd-paragraph\">Read this.</p>"),
            "{html}"
        );
        assert!(!html.contains("&lt;p"), "{html}");
        assert!(html.contains("data-rocci-docs=\"tabs\""), "{html}");
        assert!(html.contains("aria-label=\"macOS\""), "{html}");
        assert!(html.contains("Linux panel"), "{html}");
        assert!(html.contains("fn hello()"), "{html}");
        assert!(!html.contains("role=\"tablist\""), "{html}");
        assert!(!html.contains("<script"), "{html}");
        assert!(html.contains("script-src 'none'") || html.contains("script-src &#39;none&#39;"));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn project_theme_renders_article_html_unescaped() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("theme-html-src");
        fs::create_dir_all(root.join("theme")).unwrap();
        write_page(
            &root,
            "index.rocdown",
            "# Welcome\n\nHello from Markdown.\n",
        );
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
            <link rel="stylesheet" href={view.resources.stylesheet} />
        </head>
        <body>
            <main id="main-content">{content}</main>
        </body>
    </html>
}
"#,
        )
        .unwrap();
        let output = temp_dir("theme-html-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
        assert!(!html.contains("&lt;h1"), "{html}");
        assert!(
            html.contains("<p class=\"rd-paragraph\">Hello from Markdown.</p>"),
            "{html}"
        );
        let stylesheet = html
            .split("href=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("stylesheet href");
        let css_path = output.join(stylesheet.trim_start_matches('/'));
        let css = fs::read_to_string(&css_path)
            .unwrap_or_else(|_| panic!("missing stylesheet {}", css_path.display()));
        assert!(css.contains("--canvas"), "{css}");
        assert!(css.contains(".rd-header-1"), "{css}");
        let _ = fs::remove_dir_all(&root);
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown-site");
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
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown-site");
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

    #[test]
    fn wasm_host_build() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("wasm-host-src");
        write_page(
            &root,
            "index.rocdown",
            "@page { route: \"/\", meta: { title: \"Wasm Test\" } }\n\n# Wasm Documentation\nThis was rendered via Wasm host.\n",
        );
        let output = temp_dir("wasm-host-out");
        let _report = build_with_host(&root, &output, rocci_roc_host::HostChoice::Wasm).unwrap();
        assert!(output.join("index.html").is_file());
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("Wasm Documentation"));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }
}
