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
    let roc_output = apply_html(&workspace, &staging, &maps, is_wasm, &apply_path)
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
        let roc_output = apply_html(&self.workspace, &staging, &maps, is_wasm, &apply_bin)
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
        let source_name = path.display().to_string();
        let evaluated = crate::islands::evaluate_page(&path, &source_name, &src)
            .with_context(|| format!("failed to evaluate islands in {}", page.source_path))?;
        page.article_html = crate::islands::fill_placeholders(&page.article_html, &evaluated.html)
            .with_context(|| format!("failed to splice islands into {}", page.source_path))?;
        if page.kind == crate::article::PageKind::Live {
            page.article_html = crate::service::prefix_action_urls(
                &page.article_html,
                &loaded.config.http.service_origin,
            );
        }
        if page.island_css.is_empty() {
            page.island_css = evaluated.css;
        }
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
        staged_build_roc(plan, is_wasm).as_bytes(),
    ));
    fps
}

fn staged_build_roc(plan: &BuildPlan, is_wasm: bool) -> String {
    runtime::build_roc(is_wasm).replace("        # rocci-pack-kind-arms\n", &plan.pack_render_arms)
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
    pub pages: Vec<plan::PublishPage>,
    pub datastar: bool,
    pub service_origin: String,
    pub service_routes: Vec<crate::service::IslandRoute>,
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

fn apply_html(
    workspace: &Path,
    staging: &Path,
    maps: &[MappedModule],
    is_wasm: bool,
    compiled: &Path,
) -> Result<String> {
    if is_wasm {
        let wasm_out = invoke_wasm_apply(compiled, workspace, staging)?;
        fs::write(
            workspace.join("RocdownBuild.roc"),
            runtime::build_roc(false),
        )
        .context("failed to write native RocdownBuild.roc")?;
        fs::write(workspace.join("main.roc"), main_roc(false))
            .context("failed to write native apply main.roc")?;
        let native_bin = workspace.join("apply");
        let native_compile = invoke_roc_build(workspace, &native_bin, maps)?;
        let native_out = invoke_apply(&native_bin, workspace, staging, maps)?;
        Ok(format!("{wasm_out}{native_compile}{native_out}"))
    } else {
        invoke_apply(compiled, workspace, staging, maps)
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

fn invoke_wasm_apply(wasm_file: &Path, workspace: &Path, staging: &Path) -> Result<String> {
    let host = rocci_roc_host::WasmHost::from_file(wasm_file)?;
    host.run_wasi_with_preopens(staging, &[workspace])
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
        let test_dir = unique_temp("roc-probe").unwrap();
        let probe_file = test_dir.join("main.roc");
        let _ = fs::write(
            &probe_file,
            "app [main!] { pf: platform \"https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst\" }\nmain! = |_| Ok({})\n",
        );
        let probe = Command::new("roc")
            .arg("build")
            .arg("main.roc")
            .arg("--opt=dev")
            .current_dir(&test_dir)
            .env_remove("CARGO_MANIFEST_DIR")
            .env_remove("CARGO")
            .output();
        let (build_ok, probe_out) = match probe {
            Ok(output) => {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                (output.status.success(), combined)
            }
            Err(err) => (false, err.to_string()),
        };
        let _ = fs::remove_dir_all(&test_dir);
        if !build_ok {
            if env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1") {
                panic!("roc compilation failed during environment probe:\n{probe_out}");
            }
            eprintln!("skipping: roc compilation not functional in this environment");
            return true;
        }
        false
    }

    fn temp_dir(name: &str) -> PathBuf {
        unique_temp(name).unwrap()
    }

    fn assert_goto_chrome(html: &str) {
        let lower = html.to_ascii_lowercase();
        assert!(
            lower.contains("<script") && html.contains("goto."),
            "expected hashed goto chrome script\n{html}"
        );
        assert!(
            html.contains("script-src 'self'") || html.contains("script-src &#39;self&#39;"),
            "{html}"
        );
        assert!(
            html.contains("connect-src 'self'") || html.contains("connect-src &#39;self&#39;"),
            "{html}"
        );
        assert!(
            !html.contains("script-src 'none'") && !html.contains("script-src &#39;none&#39;"),
            "{html}"
        );
        assert!(
            !html.contains("/assets/datastar") && !html.contains("datastar.js"),
            "{html}"
        );
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
            assert_goto_chrome(html);
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
            "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n\n:include[path: \"snippet.rs\", region: \"hello\"]\n",
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
        assert_goto_chrome(&html);
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn block_pack_overrides_note_html() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("block-pack-src");
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
        )
        .unwrap();
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Note = |{ title }, content|
    <section data-test-note data-title={title}>{content}</section>
"#,
        )
        .unwrap();
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n",
        );
        let output = temp_dir("block-pack-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("data-test-note"), "{html}");
        assert!(html.contains("data-title=\"Watch\""), "{html}");
        assert!(!html.contains("data-rocci-docs=\"note\""), "{html}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn pack_custom_kind_paints_callout_html() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("block-pack-callout-src");
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
        )
        .unwrap();
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Callout = |{ tone ?? "note" }, content|
    <aside data-test-callout data-tone={tone}>{content}</aside>
"#,
        )
        .unwrap();
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n:callout[tone: \"warn\"] {{\n    Watch this.\n}}\n",
        );
        let output = temp_dir("block-pack-callout-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("data-test-callout"), "{html}");
        assert!(html.contains("data-tone=\"warn\""), "{html}");
        assert!(html.contains("Watch this."), "{html}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn debug_painter_emits_unfinished_markup() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("block-debug-src");
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title}</title>
        </head>
        <body>{content}</body>
    </html>
}
"#,
        )
        .unwrap();
        fs::write(
            root.join("theme/DocsComponents.rocci"),
            r#"
import Html

@component Tip = |{ title }, content|
    <p data-stub-tip data-title={title}>{content}</p>
"#,
        )
        .unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"

[blocks]
debug = true
"#,
        )
        .unwrap();
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n",
        );
        let output = temp_dir("block-debug-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("data-rocci-block-debug"), "{html}");
        assert!(html.contains("data-kind=\"note\""), "{html}");
        assert!(html.contains("Watch"), "{html}");
        assert!(!html.contains("rd-docs-note"), "{html}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn dual_apply_paints_widgets_and_splices_islands() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("dual-apply-src");
        write_page(
            &root,
            "index.rocdown",
            "# Home\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n",
        );
        write_page(
            &root,
            "widgets.rocdown",
            r#"
@page {
    route: "/widgets/",
    meta: { title: "Widgets" },
}

@roc {
feature_count = 3.I64
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Widgets

<FeatureCount count={feature_count} />
"#,
        );
        write_page(
            &root,
            "live.rocdown",
            r#"
@page {
    route: "/live/",
    meta: { title: "Live" },
}

@component
RevealTip = |{ open }| {
    <div id="reveal-tip">
        @if open {
            <p>Hide tip</p>
        } @else {
            <p>This block is closed until the server sends the open markup.</p>
        }
    </div>
}

@on:post("/actions/reveal/show") = |_| {
    revealTip({ open: True })
}

# Live

@render {
    revealTip({ open: False })
}
"#,
        );
        write_page(
            &root,
            "about.rocdown",
            r#"
@page {
    route: "/about/",
    meta: { title: "About" },
}

# About

Static neighbor.
"#,
        );
        let output = temp_dir("dual-apply-out");
        let report = build(&root, &output).unwrap();
        assert!(report.datastar);
        let home = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(home.contains("data-rocci-docs=\"note\""), "{home}");
        assert!(home.contains("data-rocci-docs=\"tabs\""), "{home}");
        assert!(home.contains("Mac panel"), "{home}");
        assert_goto_chrome(&home);

        let widgets = fs::read_to_string(output.join("widgets/index.html")).unwrap();
        assert!(widgets.contains("3 core ideas"), "{widgets}");
        assert_goto_chrome(&widgets);

        let live = fs::read_to_string(output.join("live/index.html")).unwrap();
        assert!(live.contains("reveal-tip"), "{live}");
        assert!(
            live.contains("/assets/datastar.") || live.contains("datastar."),
            "{live}"
        );

        let about = fs::read_to_string(output.join("about/index.html")).unwrap();
        assert!(about.contains("Static neighbor."), "{about}");
        assert_goto_chrome(&about);
        let pages: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
        let kind = |route: &str| {
            pages
                .as_array()
                .unwrap()
                .iter()
                .find(|page| page["route"] == route)
                .unwrap()
                .clone()
        };
        assert_eq!(kind("/")["kind"], "static");
        assert_eq!(kind("/widgets/")["kind"], "hydrate");
        assert_eq!(kind("/live/")["kind"], "live");
        assert_eq!(kind("/about/")["kind"], "static");
        assert!(kind("/about/").get("datastar").is_none());
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
    fn cdn_only_rejects_live_pages_and_preserves_output() {
        let root = temp_dir("cdn-only-src");
        write_page(
            &root,
            "index.rocdown",
            "@page { route: \"/\", meta: { title: \"Live\" } }\n\n@on:post(\"/actions/x\") = |_| {\n    Html.text(\"x\")\n}\n\n# Live\n",
        );
        let output = temp_dir("cdn-only-out");
        fs::write(output.join("keep.txt"), "preserve me").unwrap();
        let err = build_configured_with_options(
            &root,
            Some(&output),
            BuildOptions {
                host: None,
                cdn_only: true,
            },
        )
        .unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("RD2302"), "{message}");
        assert!(message.contains("CDN-only"), "{message}");
        assert_eq!(
            fs::read_to_string(output.join("keep.txt")).unwrap(),
            "preserve me"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn cdn_only_allows_static_pages() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("cdn-only-static-src");
        write_page(
            &root,
            "index.rocdown",
            "@page { route: \"/\", meta: { title: \"Home\" } }\n\n# Home\n",
        );
        let output = temp_dir("cdn-only-static-out");
        let report = build_configured_with_options(
            &root,
            Some(&output),
            BuildOptions {
                host: None,
                cdn_only: true,
            },
        )
        .unwrap();
        assert!(!report.datastar);
        assert!(report.service_routes.is_empty());
        assert!(output.join("index.html").is_file());
        assert!(!output.join("islands.json").is_file());
        let pages: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
        assert_eq!(pages[0]["kind"], "static");
        assert!(pages[0].get("datastar").is_none());
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn hydrate_pages_splice_component_html_without_scripts() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("hydrate-src");
        write_page(
            &root,
            "index.rocdown",
            r#"
@page {
    route: "/",
    meta: { title: "Hydrate" },
}

@roc {
feature_count = 3.I64
}

@css {
    .feature-count { color: teal; }
}

@component
FeatureCount = |{ count }| {
    <p class="feature-count">{count.to_str()} core ideas</p>
}

# Rocdown

Email docs@example.com.

<FeatureCount count={feature_count} />

After the island.
"#,
        );
        let output = temp_dir("hydrate-out");
        build(&root, &output).unwrap();
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
        assert!(html.contains("Rocdown"), "{html}");
        assert!(html.contains("docs@example.com"), "{html}");
        assert!(html.contains("3 core ideas"), "{html}");
        assert!(html.contains("After the island."), "{html}");
        assert_goto_chrome(&html);
        let stylesheet = html
            .split("href=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("stylesheet href");
        let css_path = output.join(stylesheet.trim_start_matches('/'));
        let css = fs::read_to_string(&css_path)
            .unwrap_or_else(|_| panic!("missing stylesheet {}", css_path.display()));
        assert!(css.contains(".feature-count"), "{css}");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn live_pages_splice_component_html_and_stage_datastar() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = temp_dir("live-src");
        write_page(
            &root,
            "index.rocdown",
            r#"
@page {
    route: "/",
    meta: { title: "Live" },
}

@component
RevealTip = |{ open }| {
    <div id="reveal-tip">
        @if open {
            <p>Hide tip</p>
        } @else {
            <>
                <p>This block is closed until the server sends the open markup.</p>
                <button type="button" data-on:click=@post("/actions/reveal/show")>
                    Show tip
                </button>
            </>
        }
    </div>
}

@on:post("/actions/reveal/show") = |_| {
    revealTip({ open: True })
}

# Live

Prose stays Markdown.

@render {
    revealTip({ open: False })
}
"#,
        );
        write_page(
            &root,
            "about.rocdown",
            r#"
@page {
    route: "/about/",
    meta: { title: "About" },
}

# About

Static neighbor.
"#,
        );
        let output = temp_dir("live-out");
        let report = build(&root, &output).unwrap();
        assert!(report.datastar);
        assert!(
            report
                .pages
                .iter()
                .any(|page| page.kind == crate::article::PageKind::Live && page.datastar),
            "{:?}",
            report.pages
        );
        assert!(
            report
                .service_routes
                .iter()
                .any(|route| route.method == "POST" && route.path == "/actions/reveal/show"),
            "{:?}",
            report.service_routes
        );
        let html = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(html.contains("<h1 class=\"rd-header-1\""), "{html}");
        assert!(html.contains("Live"), "{html}");
        assert!(html.contains("Prose stays Markdown."), "{html}");
        assert!(
            html.contains("id=\"reveal-tip\"") || html.contains("id=&#34;reveal-tip&#34;"),
            "{html}"
        );
        assert!(html.contains("Show tip"), "{html}");
        assert!(html.contains("<script"), "{html}");
        assert!(
            html.contains("/assets/datastar.") || html.contains("datastar."),
            "{html}"
        );
        assert!(
            html.contains("script-src 'self'") || html.contains("script-src &#39;self&#39;"),
            "{html}"
        );
        assert!(
            html.contains("unsafe-eval") || html.contains("unsafe-eval"),
            "{html}"
        );
        assert!(
            html.contains("connect-src 'self'") || html.contains("connect-src &#39;self&#39;"),
            "{html}"
        );
        assert!(
            !html.contains("script-src 'none'") && !html.contains("script-src &#39;none&#39;"),
            "{html}"
        );

        let about = fs::read_to_string(output.join("about/index.html")).unwrap();
        assert!(about.contains("Static neighbor."), "{about}");
        assert_goto_chrome(&about);
        let pages: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("pages.json")).unwrap()).unwrap();
        let live = pages
            .as_array()
            .unwrap()
            .iter()
            .find(|page| page["route"] == "/")
            .unwrap();
        assert_eq!(live["kind"], "live");
        assert_eq!(live["datastar"], true);
        let about_entry = pages
            .as_array()
            .unwrap()
            .iter()
            .find(|page| page["route"] == "/about/")
            .unwrap();
        assert_eq!(about_entry["kind"], "static");
        assert!(about_entry.get("datastar").is_none());
        let islands: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output.join("islands.json")).unwrap())
                .unwrap();
        assert_eq!(islands["service_origin"], "");
        assert!(
            islands["routes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|route| route["method"] == "POST" && route["path"] == "/actions/reveal/show"),
            "{islands}"
        );
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
    }

    #[test]
    fn repeat_hybrid_build_is_byte_identical() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown-hybrid");
        let first = temp_dir("hybrid-det-a");
        let second = temp_dir("hybrid-det-b");
        build(&root, &first).unwrap();
        build(&root, &second).unwrap();
        assert_eq!(collect_files(&first), collect_files(&second));
        let _ = fs::remove_dir_all(&first);
        let _ = fs::remove_dir_all(&second);
    }

    #[test]
    fn counter_example_builds_live_with_static_neighbor() {
        if skip_without_roc() {
            return;
        }
        let _lock = ROC_LOCK.lock().unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/rocdown-counter");
        let output = temp_dir("counter-out");
        let report = build(&root, &output).unwrap();
        assert!(report.datastar);
        assert!(
            report
                .pages
                .iter()
                .any(|page| page.kind == crate::article::PageKind::Live && page.datastar),
            "{:?}",
            report.pages
        );
        assert!(
            report
                .pages
                .iter()
                .any(|page| page.kind == crate::article::PageKind::Static && !page.datastar),
            "{:?}",
            report.pages
        );
        let index = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(
            index.contains("id=\"counter\"") || index.contains("id='counter'"),
            "{index}"
        );
        assert!(
            index.contains("/assets/datastar.") || index.contains("datastar."),
            "{index}"
        );
        let about = fs::read_to_string(output.join("about/index.html")).unwrap();
        assert!(about.contains("static CDN HTML"), "{about}");
        assert_goto_chrome(&about);
        assert!(
            !about.contains("/assets/datastar") && !about.contains("datastar.js"),
            "{about}"
        );
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
        assert!(output.join("index.html").is_file());
        let second = session.rebuild(&root, &output).unwrap();
        assert!(
            !second.recompiled,
            "unchanged Roc sources should reuse the apply binary (first recompiled={})",
            first.recompiled
        );
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
        assert!(html.contains("Wasm Documentation"), "{html}");
        assert!(
            html.contains("<h1 class=\"rd-header-1\""),
            "wasm apply must splice the Markdown blob into the theme article slot\n{html}"
        );
        assert!(!html.contains("&lt;h1"), "{html}");
        let native_out = temp_dir("wasm-host-native-out");
        build_with_host(&root, &native_out, rocci_roc_host::HostChoice::Native).unwrap();
        let native = fs::read_to_string(native_out.join("index.html")).unwrap();
        let article = |page: &str| {
            let start = page
                .find("<article")
                .and_then(|idx| page[idx..].find('>').map(|rel| idx + rel + 1))
                .expect("article open");
            let end = page.find("</article>").expect("article close");
            page[start..end].to_string()
        };
        assert_eq!(article(&html), article(&native));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&output);
        let _ = fs::remove_dir_all(&native_out);
    }
}
