use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use anyhow::{Context, Result, bail};

use crate::catalog;
use crate::plan::{self, BuildPlan};
use crate::runtime;
use crate::site::{LoadedSite, load_site, resolve_loaded};

mod invoke;
use invoke::{
    apply_html, invoke_roc_build, invoke_roc_wasm_build, main_roc, roc_source_hash,
    staged_build_roc, staged_fingerprints, theme_maps,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn build(root: &Path, output: &Path) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    build_loaded(&loaded, &absolute(output)?, false)
}

pub fn build_with_host(
    root: &Path,
    output: &Path,
    host: rocci_roc_host::HostChoice,
) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    build_loaded_with_host(&loaded, &absolute(output)?, host, false)
}

pub fn build_configured(root: &Path, output_override: Option<&Path>) -> Result<BuildReport> {
    build_configured_with_host(root, output_override, None)
}

pub fn build_configured_with_host(
    root: &Path,
    output_override: Option<&Path>,
    host_override: Option<rocci_roc_host::HostChoice>,
) -> Result<BuildReport> {
    build_configured_with_options(
        root,
        output_override,
        BuildOptions {
            host: host_override,
            cdn_only: false,
        },
    )
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BuildOptions {
    pub host: Option<rocci_roc_host::HostChoice>,
    pub cdn_only: bool,
}

pub fn build_configured_with_options(
    root: &Path,
    output_override: Option<&Path>,
    options: BuildOptions,
) -> Result<BuildReport> {
    let loaded = load_site(root)?;
    let output = match output_override {
        Some(output) => absolute(output)?,
        None => loaded.root.join(&loaded.config.build.output),
    };
    let host = options
        .host
        .or(loaded.config.build.host)
        .unwrap_or_default()
        .resolve();
    build_loaded_with_host(&loaded, &output, host, options.cdn_only)
}

pub(crate) fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn build_loaded(loaded: &LoadedSite, output: &Path, cdn_only: bool) -> Result<BuildReport> {
    let host = loaded.config.build.host.unwrap_or_default().resolve();
    build_loaded_with_host(loaded, output, host, cdn_only)
}

fn build_loaded_with_host(
    loaded: &LoadedSite,
    output: &Path,
    host: rocci_roc_host::HostChoice,
    cdn_only: bool,
) -> Result<BuildReport> {
    let plan_started = Instant::now();
    let plan = prepare_plan(loaded, cdn_only, false)?;
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
    let fingerprints = staged_fingerprints(&plan, is_wasm);
    let host_label = if is_wasm { "wasm" } else { "native" };
    let hash_short = &staged.roc_hash[..8.min(staged.roc_hash.len())];
    let (apply_path, recompiled, compile_ms) = match cache.inspect_renderer(
        &staged.roc_hash,
        &target,
        &fingerprints,
    ) {
        rocci_roc_host::RendererInspect::Hit(cached) => {
            eprintln!(
                "{}",
                rocci_cli::style::cli_line(&format!(
                    "rocdown: using cached {host_label} renderer for {hash_short} ({} inputs)",
                    fingerprints.len()
                ))
            );
            (cached, false, 0)
        }
        miss => {
            match miss {
                rocci_roc_host::RendererInspect::Stale { detail } => {
                    eprintln!(
                        "{}",
                        rocci_cli::style::cli_line(&format!(
                            "rocdown: cached {host_label} renderer {hash_short} stale ({detail})"
                        ))
                    );
                }
                rocci_roc_host::RendererInspect::Corrupt => {
                    eprintln!(
                        "{}",
                        rocci_cli::style::cli_line(&format!(
                            "rocdown: cached {host_label} renderer {hash_short} corrupt"
                        ))
                    );
                }
                rocci_roc_host::RendererInspect::Missing
                | rocci_roc_host::RendererInspect::Hit(_) => {}
            }
            eprintln!(
                "{}",
                rocci_cli::style::cli_line(&format!(
                    "rocdown: generated {} of Roc, compiling ({host_label}) with roc",
                    rocci_cli::style::human_bytes(staged.generated_roc_bytes as u64)
                ))
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
            eprintln!(
                "{}",
                rocci_cli::style::cli_line(&format!("rocdown: roc finished in {roc_ms}ms"))
            );
            if !roc_output.is_empty() {
                eprint!("{roc_output}");
            }
            let bytes = fs::read(&apply_bin)?;
            let stored = cache.store_renderer(&staged.roc_hash, &target, &bytes, &fingerprints)?;
            (stored, true, roc_ms)
        }
    };

    let roc_started = Instant::now();
    let roc_output = apply_html(&workspace, &staging, &maps, is_wasm, &apply_path, &plan)
        .with_context(|| format!("workspace {}", workspace.display()))?;
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

    Ok(report_from_plan(
        &plan,
        staged.generated_roc_bytes,
        0,
        plan_ms,
        generate_ms,
        compile_ms,
        roc_ms,
        write_ms,
        recompiled,
    ))
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
        let plan = prepare_plan(loaded, false, true)?;
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
        let fingerprints = staged_fingerprints(&plan, is_wasm);
        let host_label = if is_wasm { "wasm" } else { "native" };
        let hash_short = &staged.roc_hash[..8.min(staged.roc_hash.len())];
        let mut recompiled = false;
        let mut compile_ms = 0;
        let apply_bin = if self.roc_hash.as_deref() == Some(&staged.roc_hash)
            && self.apply_bin.is_file()
        {
            eprintln!(
                "{}",
                rocci_cli::style::cli_line("rocdown: content changed, applying without recompile")
            );
            self.apply_bin.clone()
        } else {
            match cache.inspect_renderer(&staged.roc_hash, &target, &fingerprints) {
                rocci_roc_host::RendererInspect::Hit(cached) => {
                    eprintln!(
                        "{}",
                        rocci_cli::style::cli_line(&format!(
                            "rocdown: using cached {host_label} renderer for {hash_short} ({} inputs)",
                            fingerprints.len()
                        ))
                    );
                    self.roc_hash = Some(staged.roc_hash.clone());
                    cached
                }
                miss => {
                    match miss {
                        rocci_roc_host::RendererInspect::Stale { detail } => {
                            eprintln!(
                                "{}",
                                rocci_cli::style::cli_line(&format!(
                                    "rocdown: cached {host_label} renderer {hash_short} stale ({detail})"
                                ))
                            );
                        }
                        rocci_roc_host::RendererInspect::Corrupt => {
                            eprintln!(
                                "{}",
                                rocci_cli::style::cli_line(&format!(
                                    "rocdown: cached {host_label} renderer {hash_short} corrupt"
                                ))
                            );
                        }
                        rocci_roc_host::RendererInspect::Missing
                        | rocci_roc_host::RendererInspect::Hit(_) => {}
                    }
                    eprintln!(
                        "{}",
                        rocci_cli::style::cli_line(&format!(
                            "rocdown: generated {} of Roc, compiling ({host_label}) with roc",
                            rocci_cli::style::human_bytes(staged.generated_roc_bytes as u64)
                        ))
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
                    eprintln!(
                        "{}",
                        rocci_cli::style::cli_line(&format!(
                            "rocdown: roc finished in {compile_ms}ms"
                        ))
                    );
                    if !roc_output.is_empty() {
                        eprint!("{roc_output}");
                    }
                    self.roc_hash = Some(staged.roc_hash.clone());
                    let bytes = fs::read(&self.apply_bin)?;
                    let stored =
                        cache.store_renderer(&staged.roc_hash, &target, &bytes, &fingerprints)?;
                    recompiled = true;
                    stored
                }
            }
        };

        let roc_started = Instant::now();
        let roc_output = apply_html(&self.workspace, &staging, &maps, is_wasm, &apply_bin, &plan)
            .with_context(|| format!("workspace {}", self.workspace.display()))?;
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

        Ok(report_from_plan(
            &plan,
            staged.generated_roc_bytes,
            0,
            plan_ms,
            generate_ms,
            compile_ms,
            roc_ms,
            write_ms,
            recompiled,
        ))
    }
}

impl Drop for BuildSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

fn prepare_plan(loaded: &LoadedSite, cdn_only: bool, preview: bool) -> Result<BuildPlan> {
    let mut result = resolve_loaded(loaded);
    for diagnostic in &result.diagnostics {
        if diagnostic.severity == catalog::Severity::Warning {
            eprintln!("{diagnostic}");
        }
    }
    if result.has_errors() {
        bail!("{}", result.error_summary());
    }
    if cdn_only {
        let live_errors = crate::site::cdn_only_live_errors(&result.site);
        if !live_errors.is_empty() {
            for diagnostic in &live_errors {
                eprintln!("{diagnostic}");
            }
            bail!(
                "{}",
                live_errors
                    .iter()
                    .map(|diagnostic| diagnostic.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }
    splice_islands(loaded, &mut result.site)?;
    if preview {
        plan::plan_preview(&loaded.root, &loaded.config, &result.site)
    } else {
        plan::plan(&loaded.root, &loaded.config, &result.site)
    }
}

fn splice_islands(loaded: &LoadedSite, site: &mut catalog::ResolvedSite) -> Result<()> {
    for page in &mut site.pages {
        if !matches!(
            page.kind,
            crate::article::PageKind::Hydrate | crate::article::PageKind::Live
        ) {
            continue;
        }
        let path = loaded.root.join(&page.source_path);
        let src = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let source_name = page.source_path.as_str();
        let service = if loaded.config.http.service.is_empty() {
            None
        } else {
            Some(loaded.root.join(&loaded.config.http.service))
        };
        let evaluated = crate::islands::evaluate_page(&path, source_name, &src, service.as_deref())
            .with_context(|| format!("failed to evaluate islands in {}", page.source_path))?;
        page.island_html = evaluated.html.clone();
        page.article_html = crate::islands::fill_placeholders(&page.article_html, &evaluated.html)
            .with_context(|| format!("failed to splice islands into {}", page.source_path))?;
        if page.kind == crate::article::PageKind::Live {
            page.article_html = crate::service::prefix_action_urls(
                &page.article_html,
                &loaded.config.http.service_origin,
            );
        }
        page.island_css = evaluated.css;
    }
    Ok(())
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
    let build_roc = staged_build_roc(plan, is_wasm);
    fs::write(workspace.join("RocdownBuild.roc"), &build_roc)
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
    fs::create_dir_all(staging.join("articles")).context("failed to create staging articles")?;
    for page in &plan.pages {
        fs::write(workspace.join(&page.article_path), &page.article_html)
            .with_context(|| format!("failed to write {}", page.article_path))?;
        fs::write(staging.join(&page.article_path), &page.article_html)
            .with_context(|| format!("failed to stage article blob {}", page.article_path))?;
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
        roc_hash: roc_source_hash(&pages_roc, &plan.theme_modules, &main, &build_roc),
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
    pub pages: Vec<plan::PublishPage>,
    pub datastar: bool,
    pub service_origin: String,
    pub service_routes: Vec<crate::service::IslandRoute>,
    pub artifacts: Vec<plan::ArtifactInspect>,
}

impl BuildReport {
    pub fn render_publish(&self) -> String {
        let mut static_n = 0;
        let mut hydrate_n = 0;
        let mut live_n = 0;
        for page in &self.pages {
            match page.kind {
                crate::article::PageKind::Static => static_n += 1,
                crate::article::PageKind::Hydrate => hydrate_n += 1,
                crate::article::PageKind::Live => live_n += 1,
            }
        }
        let mut out = format!(
            "published {} pages ({} static, {} hydrate, {} live)\n",
            self.pages.len(),
            static_n,
            hydrate_n,
            live_n
        );
        out.push_str(&format!(
            "datastar: {}\n",
            if self.datastar { "yes" } else { "no" }
        ));
        if self.datastar || !self.service_routes.is_empty() {
            let origin = if self.service_origin.is_empty() {
                "(same-origin)"
            } else {
                self.service_origin.as_str()
            };
            out.push_str(&format!("service origin: {origin}\n"));
            if self.service_routes.is_empty() {
                out.push_str("service routes: (none)\n");
            } else {
                out.push_str("service routes:\n");
                for route in &self.service_routes {
                    out.push_str(&format!("  {} {}\n", route.method, route.path));
                }
            }
        }
        out
    }
}

#[allow(clippy::too_many_arguments)]
fn report_from_plan(
    plan: &BuildPlan,
    generated_roc_bytes: usize,
    load_ms: u128,
    plan_ms: u128,
    generate_ms: u128,
    compile_ms: u128,
    roc_ms: u128,
    write_ms: u128,
    recompiled: bool,
) -> BuildReport {
    BuildReport {
        generated_roc_bytes,
        load_ms,
        plan_ms,
        generate_ms,
        compile_ms,
        roc_ms,
        write_ms,
        recompiled,
        pages: plan.publish_pages.clone(),
        datastar: plan.datastar,
        service_origin: plan.service_origin.clone(),
        service_routes: plan.service_routes.clone(),
        artifacts: plan.artifacts(),
    }
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
pub(crate) mod tests;
