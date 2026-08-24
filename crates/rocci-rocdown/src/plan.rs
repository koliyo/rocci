use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, Segment, SourceFile, compile, format_diagnostic, wrap_type_module,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::article::PageKind;
use crate::catalog::{self, NavLink, NavSection, PageHeading, ResolvedPage, ResolvedSite};
use crate::config::SiteConfig;
use crate::runtime;
use crate::service::{IslandRoute, island_routes_with_service, live_csp};

pub const DEFAULT_CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";
pub const PLAYGROUND_CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'";
const PLAYGROUND_HTML_REASON: &str = "HTML preview is not available in WASM mode. The browser cannot dynamically compile generated Roc to WebAssembly.";

const HASH_LEN: usize = 16;

pub use rocci_ui::{
    BreadcrumbView, CollectionItemView, LaneView, NavGroupView, NavItemView, OutlineView, PageView,
    ResourceView, SiteView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPage {
    pub article_path: String,
    pub output_path: String,
    pub article_html: String,
    pub fragments: Vec<(String, String)>,
    pub segments: Vec<crate::docs::PlannedNode>,
    pub view: PageView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRedirect {
    pub route: String,
    pub output_path: String,
    pub target: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedAsset {
    pub kind: &'static str,
    pub logical_path: String,
    pub hashed_url: String,
    pub output_path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    pub kind: &'static str,
    pub route: String,
    pub output_path: String,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize)]
struct PageIndexEntry<'a> {
    title: &'a str,
    route: &'a str,
    path: &'a str,
    kind: PageKind,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    datastar: bool,
    #[serde(skip_serializing_if = "str::is_empty")]
    description: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublishPage {
    pub id: String,
    pub route: String,
    pub kind: PageKind,
    pub datastar: bool,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishReport {
    pub pages: Vec<PublishPage>,
    pub datastar: bool,
    pub service_origin: String,
    pub service_routes: Vec<IslandRoute>,
    pub artifacts: Vec<ArtifactInspect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactInspect {
    pub kind: &'static str,
    pub route: String,
    pub output_path: String,
}

#[derive(Debug, Clone)]
pub struct CompiledThemeModule {
    pub type_name: String,
    pub source_name: String,
    pub src: String,
    pub roc: String,
    pub segments: Vec<Segment>,
    pub styles: Vec<rocci_template::StyleArtifact>,
    pub components: Vec<String>,
    pub component_infos: Vec<rocci_template::ComponentInfo>,
}

#[derive(Debug, Clone)]
pub struct BuildPlan {
    pub pages: Vec<PlannedPage>,
    pub redirects: Vec<PlannedRedirect>,
    pub assets: Vec<PlannedAsset>,
    pub files: Vec<PlannedFile>,
    pub theme_modules: Vec<CompiledThemeModule>,
    pub snippet_paths: std::collections::BTreeSet<String>,
    pub publish_pages: Vec<PublishPage>,
    pub datastar: bool,
    pub service_origin: String,
    pub service_routes: Vec<IslandRoute>,
    pub widget_render_arms: String,
}

impl BuildPlan {
    pub fn artifacts(&self) -> Vec<ArtifactInspect> {
        let mut items = Vec::new();
        for page in &self.pages {
            items.push(ArtifactInspect {
                kind: if page.output_path == "404.html" {
                    "not_found"
                } else {
                    "page"
                },
                route: page.view.route.clone(),
                output_path: page.output_path.clone(),
            });
        }
        for redirect in &self.redirects {
            items.push(ArtifactInspect {
                kind: "redirect",
                route: redirect.route.clone(),
                output_path: redirect.output_path.clone(),
            });
        }
        for asset in &self.assets {
            items.push(ArtifactInspect {
                kind: asset.kind,
                route: asset.hashed_url.clone(),
                output_path: asset.output_path.clone(),
            });
        }
        for file in &self.files {
            items.push(ArtifactInspect {
                kind: file.kind,
                route: file.route.clone(),
                output_path: file.output_path.clone(),
            });
        }
        items
    }

    pub fn publish_report(&self) -> PublishReport {
        PublishReport {
            pages: self.publish_pages.clone(),
            datastar: self.datastar,
            service_origin: self.service_origin.clone(),
            service_routes: self.service_routes.clone(),
            artifacts: self.artifacts(),
        }
    }

    pub fn pages_roc(&self) -> String {
        pages_roc(&self.pages)
    }
}

pub fn plan(root: &Path, config: &SiteConfig, site: &ResolvedSite) -> Result<BuildPlan> {
    plan_with_preview(root, config, site, false)
}

pub fn plan_preview(root: &Path, config: &SiteConfig, site: &ResolvedSite) -> Result<BuildPlan> {
    plan_with_preview(root, config, site, true)
}

pub(crate) fn validate_theme_painters(root: &Path, config: &SiteConfig) -> Result<()> {
    compile_theme_with_painters(root, config, false)?;
    Ok(())
}

fn plan_with_preview(
    root: &Path,
    config: &SiteConfig,
    site: &ResolvedSite,
    preview: bool,
) -> Result<BuildPlan> {
    let (theme_modules, inferred) = compile_theme_with_painters(root, config, preview)?;
    let _guard = crate::registry::install_pack_kinds(&inferred);
    let mut assets = hash_site_assets(root, config)?;
    let mut theme_css = theme_modules
        .iter()
        .flat_map(|m| m.styles.iter())
        .map(|style| style.css.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for page in &site.pages {
        if page.island_css.is_empty() {
            continue;
        }
        if !theme_css.is_empty() {
            theme_css.push('\n');
        }
        theme_css.push_str(&page.island_css);
    }
    let stylesheet = hashed_asset("theme.css", theme_css.as_bytes());
    assets.push(stylesheet.clone());
    let goto_asset = hashed_asset("goto.js", rocci_ui::chrome_script().as_bytes());
    let chrome_script_url = goto_asset.hashed_url.clone();
    assets.push(goto_asset);
    assets.sort_by(|a, b| a.output_path.cmp(&b.output_path));

    let rewrite = rewrite_map(&assets);
    let rewritten_css = rewrite_urls(&theme_css, &rewrite);
    if let Some(asset) = assets.iter_mut().find(|asset| asset.kind == "stylesheet")
        && rewritten_css != theme_css
    {
        *asset = hashed_asset("theme.css", rewritten_css.as_bytes());
    }
    let rewrite = rewrite_map(&assets);
    let stylesheet_url = assets
        .iter()
        .find(|asset| asset.kind == "stylesheet")
        .map(|asset| asset.hashed_url.clone())
        .expect("theme stylesheet");

    let csp = if config.site.csp.trim().is_empty() {
        DEFAULT_CSP.to_string()
    } else {
        config.site.csp.clone()
    };
    let social_image = rewrite_urls(&config.site.social_image, &rewrite);
    let favicon = rewrite_urls(&config.site.favicon, &rewrite);
    let apple_touch_icon = rewrite_urls(&config.site.apple_touch_icon, &rewrite);
    let site_view = SiteView {
        title: config.site.title.clone(),
        description: config.site.description.clone(),
        base_url: config.site.base_url.clone(),
        language: config.site.language.clone(),
        repository: config.site.repository.clone(),
        social_image,
        favicon,
        apple_touch_icon,
        subtitle: config.site.subtitle.clone(),
        footer: config.site.footer.clone(),
    };

    let published: Vec<_> = site
        .pages
        .iter()
        .filter(|page| !page.draft)
        .cloned()
        .collect();

    let has_playground = published.iter().any(page_uses_playground);

    let (playground_app_url, playground_css_url, playground_session_url) = if has_playground {
        let app_asset = hashed_asset("playground-app.js", runtime::PLAYGROUND_APP_JS);
        let worker_asset = hashed_asset("playground-worker.js", runtime::PLAYGROUND_WORKER_JS);
        let css_asset = hashed_asset("playground-styles.css", runtime::PLAYGROUND_STYLES_CSS);
        let wasm_asset = hashed_asset("compiler.wasm", runtime::PLAYGROUND_COMPILER_WASM);
        let session_asset = hashed_asset(
            "playground-session.json",
            &playground_session_bytes(root, &worker_asset.hashed_url, &wasm_asset.hashed_url)?,
        );

        let app_url = app_asset.hashed_url.clone();
        let css_url = css_asset.hashed_url.clone();
        let session_url = session_asset.hashed_url.clone();

        assets.push(app_asset);
        assets.push(worker_asset);
        assets.push(css_asset);
        assets.push(wasm_asset);
        assets.push(session_asset);

        (Some(app_url), Some(css_url), Some(session_url))
    } else {
        (None, None, None)
    };

    let has_live = published.iter().any(|page| page.kind == PageKind::Live);
    let datastar_url = if has_live {
        let bytes = datastar_js_bytes()?;
        let asset = hashed_asset("datastar.js", &bytes);
        let url = asset.hashed_url.clone();
        assets.push(asset);
        Some(url)
    } else {
        None
    };
    let service_routes = if has_live {
        island_routes_with_service(root, site, &config.http.service)?
    } else {
        Vec::new()
    };

    let mut news_items: Vec<CollectionItemView> = published
        .iter()
        .filter(|page| {
            page.collection == "news"
                || (page.route.starts_with("/news/") && page.layout == "news-post")
        })
        .map(|page| CollectionItemView {
            route: page.route.clone(),
            title: page.title.clone(),
            summary: page.description.clone(),
            published: page.published.clone(),
            updated: page.updated.clone(),
            authors: page.authors.clone(),
            tags: page.tags.clone(),
        })
        .collect();

    news_items.sort_by(|a, b| {
        b.published
            .cmp(&a.published)
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.route.cmp(&b.route))
    });
    let mut pages = Vec::new();
    for page in &published {
        let collection_items = if page.layout == "news-index" {
            news_items.clone()
        } else if page.layout == "home" {
            news_items.iter().take(3).cloned().collect()
        } else {
            Vec::new()
        };
        pages.push(planned_page(
            page,
            &site_view,
            &site.navigation,
            &stylesheet_url,
            &csp,
            &chrome_script_url,
            playground_app_url.as_deref(),
            playground_css_url.as_deref(),
            playground_session_url.as_deref(),
            datastar_url.as_deref(),
            &config.http.service_origin,
            &rewrite,
            &published,
            collection_items,
            false,
        ));
    }
    pages.push(not_found_page(
        &site_view,
        &site.navigation,
        &stylesheet_url,
        &csp,
        &chrome_script_url,
    ));
    pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));

    let mut redirects = Vec::new();
    for page in &published {
        for alias in &page.aliases {
            redirects.push(PlannedRedirect {
                route: alias.clone(),
                output_path: catalog::route_output_path(alias),
                target: page.route.clone(),
                html: redirect_html(&page.route),
            });
        }
    }
    redirects.sort_by(|a, b| a.output_path.cmp(&b.output_path));

    let files = discovery_files(config, &published, &news_items, &service_routes);
    let publish_pages = publish_pages(&published);
    let service_origin = config.http.service_origin.clone();

    Ok(BuildPlan {
        pages,
        redirects,
        assets,
        files,
        theme_modules,
        snippet_paths: site.snippet_paths.clone(),
        publish_pages,
        datastar: has_live,
        service_origin,
        service_routes,
        widget_render_arms: widget_kind_render_arms(),
    })
}

fn document_title(title: &str, site_title: &str) -> String {
    let title = title.trim();
    let site_title = site_title.trim();

    if title.contains(site_title) {
        title.to_string()
    } else {
        format!("{title} · {site_title}")
    }
}

fn compile_theme_with_painters(
    root: &Path,
    config: &SiteConfig,
    preview: bool,
) -> Result<(Vec<CompiledThemeModule>, Vec<crate::registry::InferredKind>)> {
    let (mut modules, pack_names) = compile_theme_modules(root, config)?;
    let inferred = inferred_kinds_from_pack(&modules, &pack_names)?;
    let allow_debug = preview || config.blocks.debug;
    let _guard = crate::registry::install_pack_kinds(&inferred);
    modules.push(block_painters_module(
        &modules,
        &pack_names,
        &config.blocks.override_map,
        allow_debug,
    )?);
    Ok((modules, inferred))
}

fn inferred_kinds_from_pack(
    modules: &[CompiledThemeModule],
    pack_names: &[String],
) -> Result<Vec<crate::registry::InferredKind>> {
    let infos: Vec<_> = modules
        .iter()
        .filter(|module| pack_names.contains(&module.type_name))
        .flat_map(|module| module.component_infos.iter().cloned())
        .collect();
    crate::registry::infer_pack_kinds(&infos)
}

pub(crate) fn site_has_block_pack(root: &Path, config: &SiteConfig) -> bool {
    if config.blocks.pack.is_some() {
        return true;
    }
    let Some(theme_dir) = theme_dir_for_pack(root, config) else {
        return false;
    };
    theme_dir.join("Blocks.rocci").is_file() || theme_dir.join("blocks").is_dir()
}

fn theme_dir_for_pack(root: &Path, config: &SiteConfig) -> Option<PathBuf> {
    if let Some(theme) = &config.build.theme {
        let path = root.join(theme);
        if path.is_dir() {
            return Some(path);
        }
        if path.is_file() {
            return path.parent().map(Path::to_path_buf);
        }
        return None;
    }
    let theme_dir = root.join("theme");
    theme_dir.is_dir().then_some(theme_dir)
}

pub(crate) fn infer_site_pack_kinds(
    root: &Path,
    config: &SiteConfig,
) -> Result<Vec<crate::registry::InferredKind>> {
    let mut modules = Vec::new();
    let theme_dir = theme_dir_for_pack(root, config);
    let pack_names = compile_block_pack(root, theme_dir.as_deref(), config, &mut modules)?;
    inferred_kinds_from_pack(&modules, &pack_names)
}

fn compile_theme_modules(
    root: &Path,
    config: &SiteConfig,
) -> Result<(Vec<CompiledThemeModule>, Vec<String>)> {
    let target = if let Some(theme) = &config.build.theme {
        let p = root.join(theme);
        if !p.exists() {
            bail!(
                "configured theme path `{theme}` does not exist in {}",
                root.display()
            );
        }
        Some(p)
    } else {
        let theme_dir = root.join("theme");
        let site_shell = root.join("theme/SiteShell.rocci");
        let rocdown_theme = root.join("theme/RocdownTheme.rocci");
        let root_site_shell = root.join("SiteShell.rocci");
        if site_shell.is_file()
            || rocdown_theme.is_file()
            || (theme_dir.is_dir() && has_rocci_files(&theme_dir))
        {
            Some(theme_dir)
        } else if root_site_shell.is_file() {
            Some(root_site_shell)
        } else {
            None
        }
    };

    if let Some(target) = target {
        let theme_dir = if target.is_dir() {
            target.clone()
        } else {
            target.parent().unwrap_or(root).to_path_buf()
        };
        let mut modules = compile_project_theme(root, &target)?;
        let pack_names = compile_block_pack(root, Some(&theme_dir), config, &mut modules)?;
        Ok((modules, pack_names))
    } else {
        let mut modules = compile_builtin_theme()?;
        let pack_names = compile_block_pack(root, None, config, &mut modules)?;
        Ok((modules, pack_names))
    }
}

fn has_rocci_files(dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().extension().is_some_and(|ext| ext == "rocci") {
                return true;
            }
        }
    }
    false
}

fn compile_single_module(
    source_name: &str,
    type_name: &str,
    src: &str,
) -> Result<CompiledThemeModule> {
    let source_file = SourceFile::new(source_name, src);
    let compiled = compile(
        source_file,
        &LowerOptions {
            embed_css: false,
            scope_file_css: type_name != "RocdownBase",
            ..LowerOptions::default()
        },
    );
    for diagnostic in &compiled.diagnostics {
        eprintln!("{}", format_diagnostic(source_file, diagnostic));
    }
    if compiled.has_errors() {
        bail!("{source_name} compilation failed");
    }
    if compiled.roc.contains("import Datastar") {
        bail!("{source_name} uses Datastar, which the rocdown runtime does not stage");
    }
    Ok(CompiledThemeModule {
        type_name: type_name.to_string(),
        source_name: source_name.to_string(),
        src: src.to_string(),
        roc: wrap_type_module(&compiled.roc, type_name),
        segments: compiled.segments,
        styles: compiled.styles,
        components: compiled
            .components
            .iter()
            .map(|component| component.name.clone())
            .collect(),
        component_infos: compiled.components.clone(),
    })
}

fn compile_project_theme(root: &Path, target: &Path) -> Result<Vec<CompiledThemeModule>> {
    let mut modules = Vec::new();
    let theme_dir = if target.is_dir() {
        target.to_path_buf()
    } else {
        target.parent().unwrap_or(root).to_path_buf()
    };

    let mut rocci_files = Vec::new();
    if theme_dir.is_dir() {
        for entry in std::fs::read_dir(&theme_dir)
            .with_context(|| format!("failed to read {}", theme_dir.display()))?
        {
            let path = entry?.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
                rocci_files.push(path);
            }
        }
    } else if target.is_file() {
        rocci_files.push(target.to_path_buf());
    }
    rocci_files.sort();

    for file in &rocci_files {
        let type_name = rocci_template::type_name_from_path(file);
        let src = std::fs::read_to_string(file)
            .with_context(|| format!("failed to read {}", file.display()))?;
        let rel_name = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        let module = compile_single_module(&rel_name, &type_name, &src)?;
        modules.push(module);
    }

    if !modules.iter().any(|m| m.type_name == "RocdownBase") {
        let base = compile_single_module("RocdownBase.rocci", "RocdownBase", runtime::BASE)?;
        modules.insert(0, base);
    }

    if !modules.iter().any(|m| m.type_name == "Breadcrumbs") {
        let breadcrumbs =
            compile_single_module("Breadcrumbs.rocci", "Breadcrumbs", runtime::BREADCRUMBS)?;
        modules.push(breadcrumbs);
    }

    if !modules.iter().any(|m| m.type_name == "NavList") {
        let nav_list = compile_single_module("NavList.rocci", "NavList", runtime::NAV_LIST)?;
        modules.push(nav_list);
    }

    if !modules.iter().any(|m| m.type_name == "PageOutline") {
        let page_outline =
            compile_single_module("PageOutline.rocci", "PageOutline", runtime::PAGE_OUTLINE)?;
        modules.push(page_outline);
    }

    if !modules.iter().any(|m| m.type_name == "DocsComponents") {
        let docs = compile_single_module("DocsComponents.rocci", "DocsComponents", runtime::DOCS)?;
        modules.push(docs);
    }

    if !modules.iter().any(|m| m.type_name == "BlockDebug") {
        let debug = compile_single_module("BlockDebug.rocci", "BlockDebug", runtime::BLOCK_DEBUG)?;
        modules.push(debug);
    }

    if !modules.iter().any(|m| m.type_name == "RocdownTheme") {
        if modules.iter().any(|m| m.type_name == "SiteShell") {
            let synth_roc = "import Html\nimport SiteShell\n\nRocdownTheme := [].{\n    siteShell = |view, content|\n        SiteShell.siteShell(view, content)\n}\n";
            modules.push(CompiledThemeModule {
                type_name: "RocdownTheme".to_string(),
                source_name: "RocdownTheme.roc".to_string(),
                src: synth_roc.to_string(),
                roc: synth_roc.to_string(),
                segments: Vec::new(),
                styles: Vec::new(),
                components: Vec::new(),
                component_infos: Vec::new(),
            });
        } else {
            bail!(
                "project theme in {} must define at least SiteShell.rocci or RocdownTheme.rocci",
                theme_dir.display()
            );
        }
    }

    Ok(modules)
}

fn compile_builtin_theme() -> Result<Vec<CompiledThemeModule>> {
    let base = compile_single_module("RocdownBase.rocci", "RocdownBase", runtime::BASE)?;
    let breadcrumbs =
        compile_single_module("Breadcrumbs.rocci", "Breadcrumbs", runtime::BREADCRUMBS)?;
    let nav_list = compile_single_module("NavList.rocci", "NavList", runtime::NAV_LIST)?;
    let page_outline =
        compile_single_module("PageOutline.rocci", "PageOutline", runtime::PAGE_OUTLINE)?;
    let theme = compile_single_module("RocdownTheme.rocci", "RocdownTheme", runtime::THEME)?;
    let docs = compile_single_module("DocsComponents.rocci", "DocsComponents", runtime::DOCS)?;
    let debug = compile_single_module("BlockDebug.rocci", "BlockDebug", runtime::BLOCK_DEBUG)?;
    Ok(vec![
        base,
        breadcrumbs,
        nav_list,
        page_outline,
        theme,
        docs,
        debug,
    ])
}

fn compile_block_pack(
    root: &Path,
    theme_dir: Option<&Path>,
    config: &SiteConfig,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    if let Some(pack) = &config.blocks.pack {
        return compile_pack_path(root, pack, modules);
    }
    if modules.iter().any(|module| module.type_name == "Blocks") {
        return Ok(vec!["Blocks".to_string()]);
    }
    let Some(theme_dir) = theme_dir else {
        return Ok(Vec::new());
    };
    let blocks_file = theme_dir.join("Blocks.rocci");
    if blocks_file.is_file() {
        return Ok(vec![ensure_theme_module(root, &blocks_file, modules)?]);
    }
    let pack_dir = theme_dir.join("blocks");
    if !pack_dir.is_dir() {
        return Ok(Vec::new());
    }
    compile_pack_dir(root, &pack_dir, modules)
}

fn compile_pack_path(
    root: &Path,
    pack: &str,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    let path = root.join(pack);
    if !path.exists() {
        bail!("blocks.pack `{pack}` does not exist in {}", root.display());
    }
    if path.is_dir() {
        return compile_pack_dir(root, &path, modules);
    }
    if path.extension().is_some_and(|ext| ext == "rocci") {
        Ok(vec![ensure_theme_module(root, &path, modules)?])
    } else {
        bail!("blocks.pack `{pack}` must be a .rocci file or directory of .rocci files");
    }
}

fn compile_pack_dir(
    root: &Path,
    dir: &Path,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?
    {
        let path = entry?.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rocci") {
            files.push(path);
        }
    }
    files.sort();
    let mut names = Vec::new();
    for file in files {
        names.push(ensure_theme_module(root, &file, modules)?);
    }
    Ok(names)
}

fn ensure_theme_module(
    root: &Path,
    file: &Path,
    modules: &mut Vec<CompiledThemeModule>,
) -> Result<String> {
    let type_name = rocci_template::type_name_from_path(file);
    let rel_name = file
        .strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    if let Some(existing) = modules.iter().find(|module| module.source_name == rel_name) {
        return Ok(existing.type_name.clone());
    }
    let src = std::fs::read_to_string(file)
        .with_context(|| format!("failed to read {}", file.display()))?;
    modules.push(compile_single_module(&rel_name, &type_name, &src)?);
    Ok(type_name)
}

fn roc_fn_name(component: &str) -> String {
    let mut chars = component.chars();
    let first = chars.next().unwrap();
    first.to_lowercase().chain(chars).collect()
}

fn widget_kind_render_arms() -> String {
    let mut out = String::new();
    for spec in crate::registry::widget_specs() {
        if !spec.paints_as_widget() {
            continue;
        }
        out.push_str(&widget_kind_render_arm(*spec));
    }
    out
}

fn widget_kind_render_arm(spec: crate::registry::KindSpec) -> String {
    let painter = roc_fn_name(spec.component);
    let record = paint_record(spec);
    match spec.name {
        "tabs" => format!(
            "        Tabs(seg) => {{\n            (items, after) = render_tab_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.tabs({record}, body), after))\n        }}\n"
        ),
        "steps" => format!(
            "        Steps(seg) => {{\n            (items, after) = render_step_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.steps({record}, body), after))\n        }}\n"
        ),
        "card-grid" => format!(
            "        CardGrid(seg) => {{\n            (items, after) = render_card_items!(segments, index + 1, seg.child_count)?\n            body = html_from_records(items)\n            Ok((BlockPainters.cardGrid({record}, body), after))\n        }}\n"
        ),
        _ if spec.paint_content() => format!(
            "        {}(seg) => {{\n            (body, after) = render_children!(segments, index + 1, seg.child_count)?\n            Ok((BlockPainters.{painter}({record}, body), after))\n        }}\n",
            spec.component
        ),
        _ => format!(
            "        {}(seg) =>\n            Ok((BlockPainters.{painter}({record}), index + 1))\n",
            spec.component
        ),
    }
}

fn paint_record(spec: crate::registry::KindSpec) -> String {
    let props: Vec<String> = spec
        .paint_fields()
        .iter()
        .map(|field| format!("{}: seg.{}", field.prop, field.prop))
        .collect();
    if props.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", props.join(", "))
    }
}

fn module_exports_component(module: &CompiledThemeModule, component: &str) -> bool {
    let key = roc_fn_name(component);
    module
        .components
        .iter()
        .any(|name| name == component || roc_fn_name(name) == key)
}

fn lookup_painter_source<'a>(
    component: &str,
    pack_modules: &[&'a CompiledThemeModule],
    docs: Option<&'a CompiledThemeModule>,
) -> Option<&'a str> {
    for module in pack_modules {
        if module_exports_component(module, component) {
            return Some(module.type_name.as_str());
        }
    }
    if docs.is_some_and(|module| module_exports_component(module, component)) {
        return Some("DocsComponents");
    }
    None
}

fn debug_params_roc(spec: crate::registry::KindSpec) -> String {
    let fields = spec.paint_fields();
    if fields.is_empty() {
        return "\"\"".to_string();
    }
    let parts: Vec<String> = fields
        .iter()
        .map(|field| match field.ty {
            crate::registry::PaintType::Str => field.prop.to_string(),
            crate::registry::PaintType::Bool => {
                format!("(if {} {{ \"true\" }} else {{ \"false\" }})", field.prop)
            }
        })
        .collect();
    if parts.len() == 1 {
        parts.into_iter().next().unwrap()
    } else {
        format!("Str.join_with([{}], \"\\n\")", parts.join(", "))
    }
}

fn props_pattern(spec: crate::registry::KindSpec) -> String {
    let fields = spec.paint_fields();
    if fields.is_empty() {
        "{}".to_string()
    } else {
        format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| field.prop)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn emit_debug_painter(roc: &mut String, spec: crate::registry::KindSpec) {
    let painter = roc_fn_name(spec.component);
    let props = props_pattern(spec);
    let params = debug_params_roc(spec);
    roc.push_str("    ");
    roc.push_str(&painter);
    if spec.paint_content() {
        roc.push_str(" = |");
        roc.push_str(&props);
        roc.push_str(", content|\n        BlockDebug.debug({ kind: \"");
        roc.push_str(spec.name);
        roc.push_str("\", params: ");
        roc.push_str(&params);
        roc.push_str(" }, content)\n");
    } else {
        roc.push_str(" = |");
        roc.push_str(&props);
        roc.push_str("|\n        BlockDebug.debug({ kind: \"");
        roc.push_str(spec.name);
        roc.push_str("\", params: ");
        roc.push_str(&params);
        roc.push_str(" }, Html.empty)\n");
    }
}

fn block_painters_module(
    modules: &[CompiledThemeModule],
    pack_names: &[String],
    overrides: &BTreeMap<String, String>,
    allow_debug: bool,
) -> Result<CompiledThemeModule> {
    let pack_modules: Vec<&CompiledThemeModule> = modules
        .iter()
        .filter(|module| pack_names.contains(&module.type_name))
        .collect();
    let docs = modules
        .iter()
        .find(|module| module.type_name == "DocsComponents");
    enum Binding {
        Component {
            painter: String,
            source: String,
            callee: String,
            paint_content: bool,
        },
        Debug(crate::registry::KindSpec),
    }
    let mut bindings = Vec::new();
    let mut uses_debug = false;
    let mut uses_html = false;
    for spec in crate::registry::widget_specs()
        .into_iter()
        .copied()
        .filter(|spec| spec.paints_as_widget())
    {
        let painter = roc_fn_name(spec.component);
        let (paint_as, required) = match overrides.get(spec.name) {
            Some(component) => (component.as_str(), true),
            None => (spec.component, false),
        };
        let callee = roc_fn_name(paint_as);
        match lookup_painter_source(paint_as, &pack_modules, docs) {
            Some(source) => bindings.push(Binding::Component {
                painter,
                source: source.to_string(),
                callee,
                paint_content: spec.paint_content(),
            }),
            None if required => {
                bail!(
                    "blocks.override `{}` names unknown component `{paint_as}`",
                    spec.name
                );
            }
            None if allow_debug => {
                uses_debug = true;
                if !spec.paint_content() {
                    uses_html = true;
                }
                bindings.push(Binding::Debug(spec));
            }
            None => {
                bail!(
                    "no renderer bound for kind `{}`; set [blocks] debug = true to paint a debug placeholder",
                    spec.name
                );
            }
        }
    }
    let mut imports = vec!["DocsComponents".to_string()];
    if uses_debug {
        imports.push("BlockDebug".to_string());
    }
    if uses_html {
        imports.push("Html".to_string());
    }
    for binding in &bindings {
        if let Binding::Component { source, .. } = binding
            && !imports.contains(source)
        {
            imports.push(source.clone());
        }
    }
    let mut roc = String::new();
    for import in &imports {
        roc.push_str("import ");
        roc.push_str(import);
        roc.push('\n');
    }
    roc.push_str("\nBlockPainters := [].{\n");
    for binding in &bindings {
        match binding {
            Binding::Component {
                painter,
                source,
                callee,
                paint_content,
            } => {
                roc.push_str("    ");
                roc.push_str(painter);
                if *paint_content {
                    roc.push_str(" = |props, content|\n        ");
                    roc.push_str(source);
                    roc.push('.');
                    roc.push_str(callee);
                    roc.push_str("(props, content)\n");
                } else {
                    roc.push_str(" = |props|\n        ");
                    roc.push_str(source);
                    roc.push('.');
                    roc.push_str(callee);
                    roc.push_str("(props)\n");
                }
            }
            Binding::Debug(spec) => emit_debug_painter(&mut roc, *spec),
        }
    }
    roc.push_str("}\n");
    Ok(CompiledThemeModule {
        type_name: "BlockPainters".to_string(),
        source_name: "BlockPainters.roc".to_string(),
        src: roc.clone(),
        roc,
        segments: Vec::new(),
        styles: Vec::new(),
        components: Vec::new(),
        component_infos: Vec::new(),
    })
}

#[allow(clippy::too_many_arguments)]
fn planned_page(
    page: &ResolvedPage,
    site: &SiteView,
    navigation: &[NavSection],
    stylesheet: &str,
    csp: &str,
    chrome_script: &str,
    playground_app: Option<&str>,
    playground_css: Option<&str>,
    playground_session: Option<&str>,
    datastar_url: Option<&str>,
    service_origin: &str,
    rewrite: &BTreeMap<String, String>,
    all_pages: &[ResolvedPage],
    collection_items: Vec<CollectionItemView>,
    not_found: bool,
) -> PlannedPage {
    let current_id = if not_found {
        None
    } else {
        Some(page.id.as_str())
    };
    let (lanes, mut sidebar) = lanes_and_sidebar(navigation, current_id);
    attach_example_source_tree(&mut sidebar, current_id, all_pages);
    if page.layout == "home" || page.layout == "playground" {
        sidebar.clear();
    } else if !sidebar_has_current(&sidebar, &page.route) {
        sidebar.push(NavGroupView::new(
            "Current page",
            "",
            true,
            vec![NavItemView::new(
                &page.title,
                &page.route,
                "nav-link nav-child is-current",
            )],
        ));
    }
    let canonical = if not_found || site.base_url.is_empty() {
        String::new()
    } else {
        format!("{}{}", site.base_url, page.route)
    };
    let article_html = rewrite_urls(&page.article_html, rewrite);
    let article_name = if not_found {
        "NotFound".to_string()
    } else {
        format!("Page{}", &hex_sha256(page.id.as_bytes())[..HASH_LEN])
    };
    let (segments, fragments) = {
        let (segments, fragments) = crate::docs::plan_segments_with_islands(
            &article_name,
            &page.article,
            rewrite,
            &page.island_html,
        );
        let fragments = if fragments.is_empty() {
            vec![(
                format!("articles/{article_name}.html"),
                article_html.clone(),
            )]
        } else {
            fragments
        };
        let segments = if segments.is_empty() {
            vec![crate::docs::PlannedNode::Html {
                path: format!("articles/{article_name}.html"),
            }]
        } else {
            segments
        };
        (segments, fragments)
    };

    let page_has_playground = !not_found && page_uses_playground(page);

    let (page_csp, module_script, playground_css_val, playground_session_val) =
        if page_has_playground {
            (
                PLAYGROUND_CSP.to_string(),
                playground_app.unwrap_or_default().to_string(),
                playground_css.unwrap_or_default().to_string(),
                playground_session.unwrap_or_default().to_string(),
            )
        } else if !not_found && page.kind == PageKind::Live {
            (
                live_csp(service_origin),
                datastar_url.unwrap_or_default().to_string(),
                String::new(),
                String::new(),
            )
        } else {
            (csp.to_string(), String::new(), String::new(), String::new())
        };

    PlannedPage {
        article_path: format!("articles/{article_name}.html"),
        output_path: page.output_path.clone(),
        article_html,
        fragments,
        segments,
        view: PageView {
            site: site.clone(),
            lanes,
            sidebar,
            route: page.route.clone(),
            title: page.title.clone(),
            document_title: document_title(&page.title, &site.title),
            description: page.description.clone(),
            layout: page.layout.clone(),
            published: page.published.clone(),
            updated: page.updated.clone(),
            authors: page.authors.clone(),
            tags: page.tags.clone(),
            collection: page.collection.clone(),
            collection_items,
            outline: page
                .headings
                .iter()
                .filter(|heading| (2..=3).contains(&heading.level))
                .map(outline_view)
                .collect(),
            breadcrumbs: normalized_breadcrumbs(page, site, navigation, all_pages),
            previous: optional_link(page.previous.as_ref()),
            next: optional_link(page.next.as_ref()),
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: page_csp,
                canonical,
                module_script,
                chrome_script: chrome_script.to_string(),
                playground_css: playground_css_val,
                playground_session: playground_session_val,
            },
        },
    }
}

fn not_found_page(
    site: &SiteView,
    navigation: &[NavSection],
    stylesheet: &str,
    csp: &str,
    chrome_script: &str,
) -> PlannedPage {
    let home = find_route_id(navigation, "/");
    let (lanes, _) = lanes_and_sidebar(navigation, home);
    let mut recovery_items = lanes
        .iter()
        .map(|lane| NavItemView::new(&lane.label, &lane.href, "nav-link nav-child"))
        .collect::<Vec<_>>();
    for (title, route) in [
        ("Start with Rocci", "/docs/"),
        ("Project status", "/project/status/"),
    ] {
        if !recovery_items.iter().any(|item| item.href == route) {
            recovery_items.push(NavItemView::new(title, route, "nav-link nav-child"));
        }
    }
    let sidebar = vec![NavGroupView::new("Recover", "", true, recovery_items)];
    PlannedPage {
        article_path: "articles/NotFound.html".into(),
        output_path: "404.html".into(),
        article_html: not_found_html(),
        fragments: vec![("articles/NotFound.html".into(), not_found_html())],
        segments: vec![crate::docs::PlannedNode::Html {
            path: "articles/NotFound.html".into(),
        }],
        view: PageView {
            site: site.clone(),
            lanes,
            sidebar,
            route: "/404.html".into(),
            title: "Page not found".into(),
            document_title: document_title("Page not found", &site.title),
            description: "This page does not exist.".into(),
            layout: "not-found".into(),
            published: String::new(),
            updated: String::new(),
            authors: Vec::new(),
            tags: Vec::new(),
            collection: String::new(),
            collection_items: Vec::new(),
            outline: Vec::new(),
            breadcrumbs: vec![
                BreadcrumbView::new(&site.title, "/"),
                BreadcrumbView::new("Page not found", "/404.html"),
            ],
            previous: NavItemView::default(),
            next: NavItemView::default(),
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: csp.to_string(),
                canonical: String::new(),
                module_script: String::new(),
                chrome_script: chrome_script.to_string(),
                playground_css: String::new(),
                playground_session: String::new(),
            },
        },
    }
}

fn not_found_html() -> String {
    String::from(
        "<h1 class=\"rd-header-1\">Page not found</h1>\n\
<p class=\"rd-paragraph\">This URL is not part of the current Rocci manual. \
The stack-first docs moved several academy routes; use the links below or \
<strong>Go to</strong> (Cmd/Ctrl+K) to open a live page.</p>\n\
<p class=\"rd-paragraph\" id=\"rocci-not-found-hint\" hidden></p>\n\
<ul class=\"rd-list\">\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/\">Documentation home</a></p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/five-minutes/\">Rocci in five minutes</a> — preview a component</p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/docs/install/\">Install Rocci</a></p></li>\n\
<li class=\"rd-list-item\"><p class=\"rd-paragraph\"><a class=\"rd-link\" href=\"/\">Site home</a></p></li>\n\
</ul>\n",
    )
}

fn lanes_and_sidebar(
    navigation: &[NavSection],
    current_id: Option<&str>,
) -> (Vec<LaneView>, Vec<NavGroupView>) {
    let has_nested = navigation
        .iter()
        .any(|section| !section.children.is_empty());
    let current_top = current_id.and_then(|id| current_section(navigation, id));
    let lanes = if has_nested {
        navigation
            .iter()
            .map(|section| LaneView {
                label: section.label.clone(),
                href: catalog::first_nav_item(section)
                    .map(|item| item.route.clone())
                    .unwrap_or_else(|| "/".into()),
                current: current_top.is_some_and(|current| current.label == section.label),
            })
            .collect()
    } else {
        Vec::new()
    };
    let groups: Vec<&NavSection> = if has_nested {
        current_top
            .map(|section| {
                if section.children.is_empty() {
                    vec![section]
                } else {
                    section.children.iter().collect()
                }
            })
            .unwrap_or_default()
    } else {
        navigation.iter().collect()
    };
    let sidebar = groups
        .into_iter()
        .filter_map(|section| nav_group_view(section, current_id))
        .collect();
    (lanes, sidebar)
}

fn current_section<'a>(navigation: &'a [NavSection], id: &str) -> Option<&'a NavSection> {
    navigation.iter().find(|section| {
        catalog::section_contains(section, id)
            || catalog::first_nav_item(section).is_some_and(|item| {
                item.id
                    .split('/')
                    .next()
                    .zip(id.split('/').next())
                    .is_some_and(|(lane, current)| lane == current)
            })
    })
}

fn sidebar_has_current(sidebar: &[NavGroupView], route: &str) -> bool {
    sidebar.iter().any(|group| {
        group.covers_href(route)
            || group
                .items
                .iter()
                .any(|item| item.class_name.contains("is-current"))
            || (group.items.is_empty() && group.open)
    })
}

fn normalized_breadcrumbs(
    page: &ResolvedPage,
    site: &SiteView,
    navigation: &[NavSection],
    all_pages: &[ResolvedPage],
) -> Vec<BreadcrumbView> {
    if page.route == "/" {
        return Vec::new();
    }
    let mut crumbs = vec![BreadcrumbView::new(&site.title, "/")];
    if let Some(section) = current_section(navigation, &page.id)
        && let Some(first) = catalog::first_nav_item(section)
    {
        push_breadcrumb(
            &mut crumbs,
            BreadcrumbView::new(&section.label, &first.route),
        );
    }
    if let Some(rest) = page.id.strip_prefix("examples/")
        && let Some(example) = rest.split('/').next()
    {
        let parent_id = format!("examples/{example}/index");
        if let Some(parent) = all_pages.iter().find(|candidate| candidate.id == parent_id) {
            push_breadcrumb(
                &mut crumbs,
                BreadcrumbView::new(&parent.title, &parent.route),
            );
        }
    }
    for link in page.breadcrumbs.iter().filter(|link| link.route != "/") {
        let mut next = breadcrumb_from_link(link);
        if crumbs.last().is_some_and(|previous| {
            previous.href == next.href
                && !previous
                    .title
                    .trim()
                    .eq_ignore_ascii_case(next.title.trim())
        }) && let Some(group) = find_group_for_page(navigation, &link.title, &page.id)
            && let Some(item) = group.items.iter().find(|item| {
                crumbs
                    .last()
                    .is_none_or(|previous| item.route != previous.href)
            })
        {
            next.href = item.route.clone();
        }
        push_breadcrumb(&mut crumbs, next);
    }
    crumbs
}

fn find_group_for_page<'a>(
    navigation: &'a [NavSection],
    label: &str,
    page_id: &str,
) -> Option<&'a NavSection> {
    navigation.iter().find_map(|section| {
        if section.label.eq_ignore_ascii_case(label) && catalog::section_contains(section, page_id)
        {
            Some(section)
        } else {
            find_group_for_page(&section.children, label, page_id)
        }
    })
}

fn push_breadcrumb(crumbs: &mut Vec<BreadcrumbView>, next: BreadcrumbView) {
    let duplicate = crumbs.last().is_some_and(|previous| {
        previous.href == next.href
            || previous
                .title
                .trim()
                .eq_ignore_ascii_case(next.title.trim())
    });
    if !duplicate {
        crumbs.push(next);
    }
}

fn find_route_id<'a>(navigation: &'a [NavSection], route: &str) -> Option<&'a str> {
    for section in navigation {
        if let Some(item) = section.items.iter().find(|item| item.route == route) {
            return Some(item.id.as_str());
        }
        if let Some(id) = find_route_id(&section.children, route) {
            return Some(id);
        }
    }
    None
}

fn nav_leaf(item: &catalog::NavItem, current_id: Option<&str>) -> NavItemView {
    NavItemView {
        title: item.title.clone(),
        href: item.route.clone(),
        class_name: if current_id == Some(item.id.as_str()) {
            "nav-link nav-child is-current".into()
        } else {
            "nav-link nav-child".into()
        },
    }
}

fn nav_item_owns_page(item_id: &str, current_id: &str) -> bool {
    if current_id == item_id {
        return true;
    }
    let Some(example) = item_id.strip_prefix("examples/") else {
        return false;
    };
    let Some(slug) = example.strip_suffix("/index") else {
        return false;
    };
    current_id
        .strip_prefix("examples/")
        .is_some_and(|rest| rest == slug || rest.starts_with(&format!("{slug}/")))
}

fn nav_group_view(section: &NavSection, current_id: Option<&str>) -> Option<NavGroupView> {
    let open = current_id.is_some_and(|id| {
        catalog::section_contains(section, id)
            || section
                .items
                .iter()
                .any(|item| nav_item_owns_page(&item.id, id))
    });
    match section.items.as_slice() {
        [] => None,
        items => Some(NavGroupView {
            title: section.label.clone(),
            href: String::new(),
            open,
            items: items
                .iter()
                .map(|item| nav_leaf(item, current_id))
                .collect(),
        }),
    }
}

fn selected_example_slug(page_id: &str) -> Option<&str> {
    let rest = page_id.strip_prefix("examples/")?;
    let slug = rest.split('/').next()?;
    if slug.is_empty() || slug == "index" {
        None
    } else {
        Some(slug)
    }
}

fn example_source_prefix(slug: &str) -> String {
    format!("examples/{slug}/source/")
}

fn attach_example_source_tree(
    sidebar: &mut Vec<NavGroupView>,
    current_id: Option<&str>,
    pages: &[ResolvedPage],
) {
    let Some(current_id) = current_id else {
        return;
    };
    let Some(slug) = selected_example_slug(current_id) else {
        return;
    };
    let example_href = format!("/examples/{slug}/");
    let prefix = example_source_prefix(slug);
    let mut source_pages: Vec<&ResolvedPage> = pages
        .iter()
        .filter(|page| page.id.starts_with(&prefix))
        .collect();
    if source_pages.is_empty() {
        return;
    }
    source_pages.sort_by(|left, right| left.id.cmp(&right.id));
    let Some(group_index) = sidebar
        .iter()
        .position(|group| group.items.iter().any(|item| item.href == example_href))
    else {
        return;
    };
    let group = sidebar.remove(group_index);
    let source_items: Vec<NavItemView> = source_pages
        .iter()
        .map(|page| {
            let current = current_id == page.id.as_str();
            NavItemView {
                title: page.title.clone(),
                href: page.route.clone(),
                class_name: if current {
                    "nav-link nav-child nav-source is-current".into()
                } else {
                    "nav-link nav-child nav-source".into()
                },
            }
        })
        .collect();
    let mut replacement = Vec::new();
    for item in group.items {
        let selected = item.href == example_href;
        replacement.push(NavGroupView {
            title: item.title,
            href: item.href,
            open: false,
            items: Vec::new(),
        });
        if selected {
            replacement.push(NavGroupView {
                title: "Source".into(),
                href: String::new(),
                open: true,
                items: source_items.clone(),
            });
        }
    }
    sidebar.splice(group_index..group_index, replacement);
}

fn outline_view(heading: &PageHeading) -> OutlineView {
    OutlineView {
        id: heading.id.clone(),
        title: heading.text.clone(),
        level: heading.level.to_string(),
    }
}

fn breadcrumb_from_link(link: &NavLink) -> BreadcrumbView {
    BreadcrumbView::new(&link.title, &link.route)
}

fn nav_from_link(link: &NavLink) -> NavItemView {
    NavItemView {
        title: link.title.clone(),
        href: link.route.clone(),
        class_name: String::new(),
    }
}

fn optional_link(link: Option<&NavLink>) -> NavItemView {
    match link {
        Some(link) => nav_from_link(link),
        None => NavItemView {
            title: String::new(),
            href: String::new(),
            class_name: String::new(),
        },
    }
}

fn hash_site_assets(root: &Path, config: &SiteConfig) -> Result<Vec<PlannedAsset>> {
    if config.build.assets.trim().is_empty() {
        return Ok(Vec::new());
    }
    let source = root.join(&config.build.assets);
    if !source.exists() {
        return Ok(Vec::new());
    }
    if !source.is_dir() {
        bail!(
            "configured assets path {} is not a directory",
            source.display()
        );
    }
    let mut files = Vec::new();
    collect_asset_files(&source, Path::new(""), &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files
        .into_iter()
        .map(|(relative, bytes)| hashed_asset(&relative, &bytes))
        .collect())
}

fn collect_asset_files(
    dir: &Path,
    prefix: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let from = entry.path();
        let relative = prefix.join(&name);
        if from.is_dir() {
            collect_asset_files(&from, &relative, files)?;
        } else {
            let bytes = std::fs::read(&from)
                .with_context(|| format!("failed to read {}", from.display()))?;
            files.push((relative.to_string_lossy().replace('\\', "/"), bytes));
        }
    }
    Ok(())
}

fn datastar_js_bytes() -> Result<Vec<u8>> {
    let path =
        rocci_cli::datastar_asset::ensure_cached(rocci_cli::datastar_asset::DEFAULT_VERSION)?;
    std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))
}

fn page_uses_playground(page: &ResolvedPage) -> bool {
    page.layout == "playground"
        || crate::docs::collect_kinds(&page.article)
            .iter()
            .any(|k| k == "playground")
}

fn playground_session_bytes(root: &Path, worker_url: &str, wasm_url: &str) -> Result<Vec<u8>> {
    let documents = load_playground_documents(root)?;
    let selected = documents
        .first()
        .and_then(|doc| doc.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("counter")
        .to_string();
    Ok(serde_json::json!({
        "protocol_version": 1,
        "documents": documents,
        "selected_document": selected,
        "compiler_wasm_url": wasm_url,
        "worker_url": worker_url,
        "mode": "wasm",
        "compile_url": "",
        "native_languages": [],
        "html_runtime": {
            "available": false,
            "reason": PLAYGROUND_HTML_REASON,
        },
    })
    .to_string()
    .into_bytes())
}

#[derive(Debug, Deserialize)]
struct PlaygroundExamplesManifest {
    #[serde(default, rename = "example")]
    examples: Vec<PlaygroundExampleSpec>,
}

#[derive(Debug, Deserialize)]
struct PlaygroundExampleSpec {
    id: String,
    file: String,
    #[serde(default)]
    language: String,
}

fn default_playground_documents() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "counter",
            "filename": "Counter.rocci",
            "language": "rocci",
            "source": "@component Counter = |{ count }| {\n    <p>{count.to_str()}</p>\n}\n",
        }),
        serde_json::json!({
            "id": "guide",
            "filename": "Guide.rocdown",
            "language": "rocdown",
            "source": "# Guide\n\nHello from Rocdown.\n",
        }),
    ]
}

fn playground_language(spec: &PlaygroundExampleSpec, filename: &str) -> Result<&'static str> {
    let raw = if spec.language.is_empty() {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
    } else {
        spec.language.as_str()
    };
    match raw {
        "rocci" => Ok("rocci"),
        "rocdown" | "md" | "markdown" => Ok("rocdown"),
        other => bail!(
            "playground example `{}` has unknown language `{other}`",
            spec.id
        ),
    }
}

fn load_playground_documents(root: &Path) -> Result<Vec<serde_json::Value>> {
    let manifest_path = root.join("playground/examples.toml");
    if !manifest_path.is_file() {
        return Ok(default_playground_documents());
    }
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PlaygroundExamplesManifest = toml::from_str(&text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    if manifest.examples.is_empty() {
        bail!(
            "{} must list at least one [[example]]",
            manifest_path.display()
        );
    }
    let mut documents = Vec::new();
    for spec in &manifest.examples {
        if spec.id.trim().is_empty() {
            bail!("playground example id must not be empty");
        }
        let relative = Path::new(&spec.file);
        if relative.is_absolute() || spec.file.contains('\0') {
            bail!(
                "playground example `{}` path must be a relative file path",
                spec.id
            );
        }
        let path = root.join(relative);
        let source = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read playground example `{}` from {}",
                spec.id,
                path.display()
            )
        })?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("playground example `{}` has a non-utf8 filename", spec.id)
            })?
            .to_string();
        let language = playground_language(spec, &filename)?;
        documents.push(serde_json::json!({
            "id": spec.id,
            "filename": filename,
            "language": language,
            "source": source,
        }));
    }
    Ok(documents)
}

fn hashed_asset(relative: &str, bytes: &[u8]) -> PlannedAsset {
    let hash = hex_sha256(bytes);
    let hashed_name = hashed_file_name(relative, &hash);
    let kind = if Path::new(relative)
        .file_stem()
        .is_some_and(|stem| stem == "theme")
        && Path::new(relative)
            .extension()
            .is_some_and(|ext| ext == "css")
    {
        "stylesheet"
    } else {
        "asset"
    };
    PlannedAsset {
        kind,
        logical_path: format!("/assets/{relative}"),
        hashed_url: format!("/assets/{hashed_name}"),
        output_path: format!("assets/{hashed_name}"),
        bytes: bytes.to_vec(),
    }
}

fn hashed_file_name(relative: &str, hash: &str) -> String {
    let path = Path::new(relative);
    let hash = &hash[..HASH_LEN];
    let name = match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => format!(
            "{}.{hash}.{}",
            stem.to_string_lossy(),
            ext.to_string_lossy()
        ),
        (Some(stem), None) => format!("{}.{hash}", stem.to_string_lossy()),
        _ => format!("asset.{hash}"),
    };
    match path.parent().filter(|parent| *parent != Path::new("")) {
        Some(parent) => format!("{}/{name}", parent.to_string_lossy()),
        None => name,
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn rewrite_map(assets: &[PlannedAsset]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for asset in assets {
        if asset.kind == "stylesheet" {
            continue;
        }
        map.insert(asset.logical_path.clone(), asset.hashed_url.clone());
        if let Some(rest) = asset.logical_path.strip_prefix('/') {
            map.insert(rest.to_string(), asset.hashed_url.clone());
        }
    }
    map
}

fn rewrite_urls(text: &str, map: &BTreeMap<String, String>) -> String {
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort_by(|a, b| b.len().cmp(&a.len()).then(a.cmp(b)));
    let mut out = text.to_string();
    for key in keys {
        if let Some(hashed) = map.get(key) {
            out = out.replace(key, hashed);
        }
    }
    out
}

fn discovery_files(
    config: &SiteConfig,
    pages: &[ResolvedPage],
    news_items: &[CollectionItemView],
    service_routes: &[IslandRoute],
) -> Vec<PlannedFile> {
    let mut files = Vec::new();
    let mut llms = format!("# {}\n\n{}\n\n", config.site.title, config.site.description);
    for page in pages {
        let url = format!("{}{}", config.site.base_url, page.route);
        if page.description.is_empty() {
            llms.push_str(&format!("- [{}]({url})\n", page.title));
        } else {
            llms.push_str(&format!(
                "- [{}]({url}): {}\n",
                page.title, page.description
            ));
        }
    }
    files.push(PlannedFile {
        kind: "llms",
        route: "/llms.txt".into(),
        output_path: "llms.txt".into(),
        contents: llms,
    });
    files.push(PlannedFile {
        kind: "pages",
        route: "/pages.json".into(),
        output_path: "pages.json".into(),
        contents: pages_json(pages),
    });
    if pages.iter().any(|page| page.kind == PageKind::Live) {
        files.push(PlannedFile {
            kind: "islands",
            route: "/islands.json".into(),
            output_path: "islands.json".into(),
            contents: islands_json(&config.http.service_origin, pages, service_routes),
        });
    }
    if !config.site.base_url.is_empty() {
        let mut sitemap = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for page in pages {
            sitemap.push_str("  <url><loc>");
            sitemap.push_str(&escape_xml(&format!(
                "{}{}",
                config.site.base_url, page.route
            )));
            sitemap.push_str("</loc></url>\n");
        }
        sitemap.push_str("</urlset>\n");
        files.push(PlannedFile {
            kind: "sitemap",
            route: "/sitemap.xml".into(),
            output_path: "sitemap.xml".into(),
            contents: sitemap,
        });
        files.push(PlannedFile {
            kind: "robots",
            route: "/robots.txt".into(),
            output_path: "robots.txt".into(),
            contents: format!(
                "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
                config.site.base_url
            ),
        });
        if !news_items.is_empty() {
            files.push(PlannedFile {
                kind: "feed",
                route: "/news/feed.xml".into(),
                output_path: "news/feed.xml".into(),
                contents: atom_feed(config, news_items),
            });
        }
    }
    files.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    files
}

fn pages_json(pages: &[ResolvedPage]) -> String {
    let mut entries: Vec<PageIndexEntry<'_>> = pages
        .iter()
        .map(|page| PageIndexEntry {
            title: &page.title,
            route: &page.route,
            path: &page.source_path,
            kind: page.kind,
            datastar: page.kind == PageKind::Live,
            description: &page.description,
        })
        .collect();
    entries.sort_by(|left, right| left.route.cmp(right.route));
    match serde_json::to_string_pretty(&entries) {
        Ok(json) => format!("{json}\n"),
        Err(_) => "[]\n".into(),
    }
}

fn islands_json(service_origin: &str, pages: &[ResolvedPage], routes: &[IslandRoute]) -> String {
    #[derive(Serialize)]
    struct IslandsPage<'a> {
        id: &'a str,
        route: &'a str,
        kind: PageKind,
    }
    #[derive(Serialize)]
    struct IslandsFile<'a> {
        service_origin: &'a str,
        pages: Vec<IslandsPage<'a>>,
        routes: &'a [IslandRoute],
    }
    let mut island_pages: Vec<IslandsPage<'_>> = pages
        .iter()
        .filter(|page| page.kind == PageKind::Live)
        .map(|page| IslandsPage {
            id: &page.id,
            route: &page.route,
            kind: page.kind,
        })
        .collect();
    island_pages.sort_by(|left, right| left.route.cmp(right.route).then(left.id.cmp(right.id)));
    let file = IslandsFile {
        service_origin,
        pages: island_pages,
        routes,
    };
    match serde_json::to_string_pretty(&file) {
        Ok(json) => format!("{json}\n"),
        Err(_) => "{\n  \"service_origin\": \"\",\n  \"pages\": [],\n  \"routes\": []\n}\n".into(),
    }
}

fn publish_pages(pages: &[ResolvedPage]) -> Vec<PublishPage> {
    let mut entries: Vec<PublishPage> = pages
        .iter()
        .map(|page| PublishPage {
            id: page.id.clone(),
            route: page.route.clone(),
            kind: page.kind,
            datastar: page.kind == PageKind::Live,
            output_path: page.output_path.clone(),
        })
        .collect();
    entries.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then_with(|| left.id.cmp(&right.id))
    });
    entries
}

fn atom_feed(config: &SiteConfig, news_items: &[CollectionItemView]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">\n",
    );
    xml.push_str(&format!(
        "  <title>{} News</title>\n",
        escape_xml(&config.site.title)
    ));
    let feed_url = format!("{}/news/feed.xml", config.site.base_url);
    let site_news_url = format!("{}/news/", config.site.base_url);
    xml.push_str(&format!(
        "  <link href=\"{}\" rel=\"self\" />\n",
        escape_xml(&feed_url)
    ));
    xml.push_str(&format!(
        "  <link href=\"{}\" />\n",
        escape_xml(&site_news_url)
    ));
    xml.push_str(&format!("  <id>{}</id>\n", escape_xml(&site_news_url)));

    let latest_date = news_items
        .first()
        .map(|item| {
            if !item.updated.is_empty() {
                item.updated.as_str()
            } else if !item.published.is_empty() {
                item.published.as_str()
            } else {
                "2026-01-01"
            }
        })
        .unwrap_or("2026-01-01");
    xml.push_str(&format!("  <updated>{}T00:00:00Z</updated>\n", latest_date));

    for item in news_items {
        xml.push_str("  <entry>\n");
        xml.push_str(&format!("    <title>{}</title>\n", escape_xml(&item.title)));
        let entry_url = format!("{}{}", config.site.base_url, item.route);
        xml.push_str(&format!(
            "    <link href=\"{}\" />\n",
            escape_xml(&entry_url)
        ));
        xml.push_str(&format!("    <id>{}</id>\n", escape_xml(&entry_url)));
        let pub_date = if !item.published.is_empty() {
            &item.published
        } else {
            "2026-01-01"
        };
        xml.push_str(&format!(
            "    <published>{}T00:00:00Z</published>\n",
            pub_date
        ));
        let upd_date = if !item.updated.is_empty() {
            &item.updated
        } else {
            pub_date
        };
        xml.push_str(&format!("    <updated>{}T00:00:00Z</updated>\n", upd_date));
        if !item.summary.is_empty() {
            xml.push_str(&format!(
                "    <summary>{}</summary>\n",
                escape_xml(&item.summary)
            ));
        }
        for author in &item.authors {
            xml.push_str(&format!(
                "    <author><name>{}</name></author>\n",
                escape_xml(author)
            ));
        }
        xml.push_str("  </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

fn redirect_html(target: &str) -> String {
    let target = escape_xml(target);
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Redirect</title>\n<link rel=\"canonical\" href=\"{target}\">\n<meta http-equiv=\"refresh\" content=\"0; url={target}\">\n</head>\n<body>\n<p>Moved to <a href=\"{target}\">{target}</a>.</p>\n</body>\n</html>\n"
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn pages_roc(pages: &[PlannedPage]) -> String {
    let mut pages = pages.to_vec();
    pages.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    let mut out = String::from("RocdownPages := [].{\n    pages = [\n");
    for page in &pages {
        out.push_str("        {\n            article_path: ");
        push_roc_string(&mut out, &page.article_path);
        out.push_str(",\n            output_path: ");
        push_roc_string(&mut out, &page.output_path);
        out.push_str(",\n            segments: ");
        push_nodes(&mut out, &page.segments, 3);
        out.push_str(
            ",\n            view: {\n                site: {\n                    title: ",
        );
        push_roc_string(&mut out, &page.view.site.title);
        out.push_str(",\n                    description: ");
        push_roc_string(&mut out, &page.view.site.description);
        out.push_str(",\n                    base_url: ");
        push_roc_string(&mut out, &page.view.site.base_url);
        out.push_str(",\n                    language: ");
        push_roc_string(&mut out, &page.view.site.language);
        out.push_str(",\n                    repository: ");
        push_roc_string(&mut out, &page.view.site.repository);
        out.push_str(",\n                    social_image: ");
        push_roc_string(&mut out, &page.view.site.social_image);
        out.push_str(",\n                    favicon: ");
        push_roc_string(&mut out, &page.view.site.favicon);
        out.push_str(",\n                    apple_touch_icon: ");
        push_roc_string(&mut out, &page.view.site.apple_touch_icon);
        out.push_str(",\n                    subtitle: ");
        push_roc_string(&mut out, &page.view.site.subtitle);
        out.push_str(",\n                    footer: ");
        push_roc_string(&mut out, &page.view.site.footer);
        out.push_str("\n                },\n                lanes: [\n");
        for lane in &page.view.lanes {
            out.push_str("                    { label: ");
            push_roc_string(&mut out, &lane.label);
            out.push_str(", href: ");
            push_roc_string(&mut out, &lane.href);
            out.push_str(", current: ");
            out.push_str(if lane.current { "True" } else { "False" });
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                sidebar: [\n");
        for group in &page.view.sidebar {
            out.push_str("                    { title: ");
            push_roc_string(&mut out, &group.title);
            out.push_str(", href: ");
            push_roc_string(&mut out, &group.href);
            out.push_str(", open: ");
            out.push_str(if group.open { "True" } else { "False" });
            out.push_str(", items: [\n");
            for item in &group.items {
                out.push_str("                        { title: ");
                push_roc_string(&mut out, &item.title);
                out.push_str(", href: ");
                push_roc_string(&mut out, &item.href);
                out.push_str(", class_name: ");
                push_roc_string(&mut out, &item.class_name);
                out.push_str(" },\n");
            }
            out.push_str("                    ] },\n");
        }
        out.push_str("                ],\n                route: ");
        push_roc_string(&mut out, &page.view.route);
        out.push_str(",\n                title: ");
        push_roc_string(&mut out, &page.view.title);
        out.push_str(",\n                document_title: ");
        push_roc_string(&mut out, &page.view.document_title);
        out.push_str(",\n                description: ");
        push_roc_string(&mut out, &page.view.description);
        out.push_str(",\n                layout: ");
        push_roc_string(&mut out, &page.view.layout);
        out.push_str(",\n                published: ");
        push_roc_string(&mut out, &page.view.published);
        out.push_str(",\n                updated: ");
        push_roc_string(&mut out, &page.view.updated);
        out.push_str(",\n                authors: [\n");
        for author in &page.view.authors {
            out.push_str("                    ");
            push_roc_string(&mut out, author);
            out.push_str(",\n");
        }
        out.push_str("                ],\n                tags: [\n");
        for tag in &page.view.tags {
            out.push_str("                    ");
            push_roc_string(&mut out, tag);
            out.push_str(",\n");
        }
        out.push_str("                ],\n                collection: ");
        push_roc_string(&mut out, &page.view.collection);
        out.push_str(",\n                collection_items: [\n");
        for item in &page.view.collection_items {
            out.push_str("                    {\n                        route: ");
            push_roc_string(&mut out, &item.route);
            out.push_str(",\n                        title: ");
            push_roc_string(&mut out, &item.title);
            out.push_str(",\n                        summary: ");
            push_roc_string(&mut out, &item.summary);
            out.push_str(",\n                        published: ");
            push_roc_string(&mut out, &item.published);
            out.push_str(",\n                        updated: ");
            push_roc_string(&mut out, &item.updated);
            out.push_str(",\n                        authors: [\n");
            for author in &item.authors {
                out.push_str("                            ");
                push_roc_string(&mut out, author);
                out.push_str(",\n");
            }
            out.push_str("                        ],\n                        tags: [\n");
            for tag in &item.tags {
                out.push_str("                            ");
                push_roc_string(&mut out, tag);
                out.push_str(",\n");
            }
            out.push_str("                        ]\n                    },\n");
        }
        out.push_str("                ],\n                outline: [\n");
        for heading in &page.view.outline {
            out.push_str("                    { id: ");
            push_roc_string(&mut out, &heading.id);
            out.push_str(", title: ");
            push_roc_string(&mut out, &heading.title);
            out.push_str(", level: ");
            push_roc_string(&mut out, &heading.level);
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                breadcrumbs: [\n");
        for crumb in &page.view.breadcrumbs {
            out.push_str("                    { title: ");
            push_roc_string(&mut out, &crumb.title);
            out.push_str(", href: ");
            push_roc_string(&mut out, &crumb.href);
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                previous: { title: ");
        push_roc_string(&mut out, &page.view.previous.title);
        out.push_str(", href: ");
        push_roc_string(&mut out, &page.view.previous.href);
        out.push_str(" },\n                next: { title: ");
        push_roc_string(&mut out, &page.view.next.title);
        out.push_str(", href: ");
        push_roc_string(&mut out, &page.view.next.href);
        out.push_str(" },\n                resources: {\n                    stylesheet: ");
        push_roc_string(&mut out, &page.view.resources.stylesheet);
        out.push_str(",\n                    csp: ");
        push_roc_string(&mut out, &page.view.resources.csp);
        out.push_str(",\n                    canonical: ");
        push_roc_string(&mut out, &page.view.resources.canonical);
        out.push_str(",\n                    module_script: ");
        push_roc_string(&mut out, &page.view.resources.module_script);
        out.push_str(",\n                    chrome_script: ");
        push_roc_string(&mut out, &page.view.resources.chrome_script);
        out.push_str(",\n                    playground_css: ");
        push_roc_string(&mut out, &page.view.resources.playground_css);
        out.push_str(",\n                    playground_session: ");
        push_roc_string(&mut out, &page.view.resources.playground_session);
        out.push_str("\n                }\n            }\n        },\n");
    }
    out.push_str("    ]\n}\n");
    out
}

fn push_nodes(out: &mut String, nodes: &[crate::docs::PlannedNode], indent: usize) {
    let mut flat = Vec::new();
    collect_flat(nodes, &mut flat);
    out.push_str("[\n");
    for node in flat {
        for _ in 0..indent + 1 {
            out.push_str("    ");
        }
        push_node(out, node);
        out.push_str(",\n");
    }
    for _ in 0..indent {
        out.push_str("    ");
    }
    out.push(']');
}

fn collect_flat<'a>(
    nodes: &'a [crate::docs::PlannedNode],
    out: &mut Vec<&'a crate::docs::PlannedNode>,
) {
    for node in nodes {
        out.push(node);
        if let crate::docs::PlannedNode::Widget(widget) = node {
            collect_flat(&widget.children, out);
        }
    }
}

fn push_node(out: &mut String, node: &crate::docs::PlannedNode) {
    match node {
        crate::docs::PlannedNode::Html { path } => {
            out.push_str("HtmlFile({ path: ");
            push_roc_string(out, path);
            out.push_str(" })");
        }
        crate::docs::PlannedNode::Widget(widget) => {
            out.push_str(&widget.component);
            out.push_str("({ ");
            let spec = crate::registry::lookup(&widget.kind);
            let paint_content =
                widget.paint_content || spec.is_some_and(|kind| kind.paint_content());
            for (index, prop) in widget.props.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                match prop {
                    crate::docs::PlannedProp::Str { name, value } => {
                        out.push_str(name);
                        out.push_str(": ");
                        push_roc_string(out, value);
                    }
                    crate::docs::PlannedProp::Bool { name, value } => {
                        out.push_str(name);
                        out.push_str(": ");
                        out.push_str(if *value { "True" } else { "False" });
                    }
                }
            }
            if paint_content {
                if !widget.props.is_empty() {
                    out.push_str(", ");
                }
                out.push_str("child_count: ");
                out.push_str(&widget.children.len().to_string());
            }
            out.push_str(" })");
        }
    }
}

fn push_roc_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::site::{InspectKind, inspect, load_site, resolve_loaded};
    use std::{env, fs, path::PathBuf};

    #[test]
    fn document_title_adds_the_brand_exactly_once() {
        assert_eq!(document_title("Guide", "Rocci"), "Guide · Rocci");
        assert_eq!(
            document_title("Contributing to Rocci", "Rocci"),
            "Contributing to Rocci"
        );
        assert_eq!(
            document_title("Rocci · Native interfaces", "Rocci"),
            "Rocci · Native interfaces"
        );
    }

    fn temp(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("rocdown-plan-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn nav_item(id: &str, title: &str, route: &str) -> catalog::NavItem {
        catalog::NavItem {
            id: id.into(),
            title: title.into(),
            route: route.into(),
        }
    }

    fn nav_section(
        label: &str,
        items: Vec<catalog::NavItem>,
        children: Vec<NavSection>,
    ) -> NavSection {
        NavSection {
            label: label.into(),
            items,
            children,
        }
    }

    #[test]
    fn sidebar_lists_all_flat_sections_and_opens_current() {
        let navigation = vec![
            nav_section(
                "Start",
                vec![
                    nav_item("index", "Home", "/"),
                    nav_item("start/install", "Install", "/start/install/"),
                ],
                vec![],
            ),
            nav_section(
                "Tutorials",
                vec![
                    nav_item("tutorials/index", "Tutorials", "/tutorials/"),
                    nav_item(
                        "tutorials/first-component",
                        "Build your first component",
                        "/tutorials/first-component/",
                    ),
                ],
                vec![],
            ),
        ];
        let (lanes, sidebar) = lanes_and_sidebar(&navigation, Some("tutorials/first-component"));
        assert!(lanes.is_empty());
        assert_eq!(sidebar.len(), 2);
        assert!(!sidebar[0].open);
        assert!(sidebar[1].open);
        assert!(sidebar[1].href.is_empty());
        assert_eq!(sidebar[1].items[0].title, "Tutorials");
        assert_eq!(sidebar[1].items[1].title, "Build your first component");
        assert!(sidebar[1].items[1].class_name.contains("is-current"));
    }

    fn source_page(id: &str, title: &str, route: &str) -> ResolvedPage {
        ResolvedPage {
            id: id.into(),
            source_path: format!("{id}.rocdown"),
            kind: PageKind::Static,
            title: title.into(),
            description: String::new(),
            layout: "docs".into(),
            published: String::new(),
            updated: String::new(),
            authors: Vec::new(),
            tags: Vec::new(),
            collection: String::new(),
            headings: Vec::new(),
            outgoing_links: Vec::new(),
            article_html: String::new(),
            island_css: String::new(),
            island_html: Vec::new(),
            route: route.into(),
            output_path: String::new(),
            aliases: Vec::new(),
            draft: false,
            suppress_unlisted_warning: true,
            unlisted: true,
            breadcrumbs: Vec::new(),
            previous: None,
            next: None,
            article: Vec::new(),
            examples: Vec::new(),
            includes: Vec::new(),
            docs_kinds: Vec::new(),
        }
    }

    #[test]
    fn selected_example_lists_source_tree_below_the_example() {
        let navigation = vec![nav_section(
            "Examples",
            vec![
                nav_item("examples/index", "Examples", "/examples/"),
                nav_item("examples/blocks/index", "Rocci Blocks", "/examples/blocks/"),
                nav_item("examples/snake/index", "Snake", "/examples/snake/"),
            ],
            vec![],
        )];
        let pages = [
            source_page(
                "examples/blocks/source/backend--Blocks-rocci",
                "backend/Blocks.rocci",
                "/examples/blocks/source/backend--Blocks-rocci/",
            ),
            source_page(
                "examples/snake/source/Snake-rocci",
                "Snake.rocci",
                "/examples/snake/source/Snake-rocci/",
            ),
        ];
        let (_, mut sidebar) = lanes_and_sidebar(
            &navigation,
            Some("examples/blocks/source/backend--Blocks-rocci"),
        );
        attach_example_source_tree(
            &mut sidebar,
            Some("examples/blocks/source/backend--Blocks-rocci"),
            &pages,
        );
        assert_eq!(sidebar.len(), 4);
        assert_eq!(sidebar[0].title, "Examples");
        assert!(sidebar[0].items.is_empty());
        assert_eq!(sidebar[1].title, "Rocci Blocks");
        assert_eq!(sidebar[1].href, "/examples/blocks/");
        assert_eq!(sidebar[2].title, "Source");
        assert!(sidebar[2].open);
        assert_eq!(sidebar[2].items.len(), 1);
        assert_eq!(sidebar[2].items[0].title, "backend/Blocks.rocci");
        assert!(sidebar[2].items[0].class_name.contains("is-current"));
        assert_eq!(sidebar[3].title, "Snake");
        assert!(sidebar[3].items.is_empty());
        assert!(sidebar_has_current(
            &sidebar,
            "/examples/blocks/source/backend--Blocks-rocci/"
        ));
    }

    #[test]
    fn examples_index_does_not_attach_a_source_tree() {
        let navigation = vec![nav_section(
            "Examples",
            vec![nav_item("examples/index", "Examples", "/examples/")],
            vec![],
        )];
        let pages = [source_page(
            "examples/blocks/source/App-rocci",
            "App.rocci",
            "/examples/blocks/source/App-rocci/",
        )];
        let (_, mut sidebar) = lanes_and_sidebar(&navigation, Some("examples/index"));
        attach_example_source_tree(&mut sidebar, Some("examples/index"), &pages);
        assert_eq!(sidebar.len(), 1);
        assert_eq!(sidebar[0].title, "Examples");
        assert_eq!(sidebar[0].items.len(), 1);
    }

    #[test]
    fn nested_groups_keep_lanes_and_current_docs_sidebar() {
        let navigation = vec![
            nav_section(
                "Docs",
                vec![],
                vec![
                    nav_section(
                        "Tutorials",
                        vec![
                            nav_item("docs/tutorials/index", "Tutorials", "/docs/tutorials/"),
                            nav_item(
                                "docs/tutorials/first-component",
                                "Build your first component",
                                "/docs/tutorials/first-component/",
                            ),
                        ],
                        vec![],
                    ),
                    nav_section(
                        "Status",
                        vec![nav_item("docs/status", "Status", "/docs/status/")],
                        vec![],
                    ),
                ],
            ),
            nav_section(
                "News",
                vec![nav_item("news/index", "News", "/news/")],
                vec![],
            ),
        ];
        let (lanes, sidebar) =
            lanes_and_sidebar(&navigation, Some("docs/tutorials/first-component"));
        assert_eq!(lanes.len(), 2);
        assert!(lanes[0].current);
        assert!(!lanes[1].current);
        assert_eq!(sidebar[0].title, "Tutorials");
        assert!(sidebar[0].open);
        assert_eq!(sidebar[1].title, "Status");
        assert_eq!(sidebar[1].items.len(), 1);
        assert_eq!(sidebar[1].items[0].title, "Status");
    }

    #[test]
    fn single_page_lane_is_an_expandable_current_group() {
        let navigation = vec![nav_section(
            "FAQ",
            vec![nav_item("faq/index", "FAQ", "/faq/")],
            vec![],
        )];
        let (_, sidebar) = lanes_and_sidebar(&navigation, Some("faq/index"));
        assert_eq!(sidebar.len(), 1);
        assert!(sidebar[0].open);
        assert!(sidebar[0].href.is_empty());
        assert_eq!(sidebar[0].items.len(), 1);
        assert!(sidebar_has_current(&sidebar, "/faq/"));
    }

    fn write_site(root: &Path) {
        fs::create_dir_all(root.join("assets/icons")).unwrap();
        fs::write(root.join("assets/og.png"), b"og-bytes").unwrap();
        fs::write(
            root.join("assets/favicon.svg"),
            b"<svg xmlns='http://www.w3.org/2000/svg'/>",
        )
        .unwrap();
        fs::write(root.join("assets/apple-touch-icon.png"), b"touch-bytes").unwrap();
        fs::write(root.join("assets/icons/logo.png"), b"logo-bytes").unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"
social_image = "/assets/og.png"
favicon = "/assets/favicon.svg"
apple_touch_icon = "/assets/apple-touch-icon.png"
subtitle = "Tools"
footer = "Experimental."

[[nav]]
label = "Start"
items = ["index", "guide"]
"#,
        )
        .unwrap();
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n",
        )
        .unwrap();
        fs::write(
            root.join("guide.rocdown"),
            "@page {\n    aliases: [\"/old-guide/\"],\n    meta: { title: \"Guide\" },\n}\n\n# Guide\n\n## Details\n\nLogo: ![logo](/assets/icons/logo.png)\n",
        )
        .unwrap();
    }

    #[test]
    fn playground_styles_do_not_override_host_root_tokens() {
        let css = include_str!("../../../playground/src/styles.css");
        assert!(
            !css.contains(":root {"),
            "playground CSS must inherit host theme tokens when embedded"
        );
        assert!(
            !css.contains("#faf9f6") && !css.contains("#161413") && !css.contains("#e64b2f"),
            "standalone playground must not keep the old warm palette\n{css}"
        );
        assert!(css.contains("html:has(body > #playground-root)"), "{css}");
        assert!(css.contains("background: var(--code)"), "{css}");
    }

    #[test]
    fn default_csp_is_strict_and_stable() {
        assert_eq!(
            DEFAULT_CSP,
            "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
        );
        assert!(!DEFAULT_CSP.contains("unsafe-eval"));
        assert!(!DEFAULT_CSP.contains("unsafe-inline"));
    }

    #[test]
    fn playground_layout_gets_hashed_assets_session_and_csp() {
        let root = temp("playground-layout");
        write_site(&root);
        fs::write(
            root.join("playground.rocdown"),
            "@page {\n    layout: \"playground\",\n    route: \"/playground/\",\n    meta: { title: \"Playground\" },\n}\n\n# Playground\n",
        )
        .unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"

[[nav]]
label = "Start"
items = ["index", "guide"]

[[nav]]
label = "Playground"
items = ["playground"]
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

        let playground = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/playground/")
            .expect("playground page");
        let guide = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/guide/")
            .expect("guide page");

        assert_eq!(playground.view.layout, "playground");
        assert_eq!(playground.view.resources.csp, PLAYGROUND_CSP);
        assert!(
            playground
                .view
                .resources
                .module_script
                .contains("playground-app."),
            "{}",
            playground.view.resources.module_script
        );
        assert!(
            playground
                .view
                .resources
                .playground_css
                .contains("playground-styles."),
            "{}",
            playground.view.resources.playground_css
        );
        assert!(
            playground
                .view
                .resources
                .playground_session
                .contains("playground-session."),
            "{}",
            playground.view.resources.playground_session
        );
        assert!(playground.view.sidebar.is_empty());

        assert_eq!(guide.view.resources.csp, DEFAULT_CSP);
        assert!(guide.view.resources.module_script.is_empty());
        assert!(guide.view.resources.playground_css.is_empty());
        assert!(guide.view.resources.playground_session.is_empty());

        let logical: Vec<_> = planned
            .assets
            .iter()
            .map(|asset| asset.logical_path.as_str())
            .collect();
        for expected in [
            "/assets/playground-app.js",
            "/assets/playground-worker.js",
            "/assets/playground-styles.css",
            "/assets/compiler.wasm",
            "/assets/playground-session.json",
        ] {
            assert!(logical.contains(&expected), "{logical:?}");
        }

        let session = planned
            .assets
            .iter()
            .find(|asset| asset.logical_path == "/assets/playground-session.json")
            .expect("session asset");
        let worker = planned
            .assets
            .iter()
            .find(|asset| asset.logical_path == "/assets/playground-worker.js")
            .expect("worker asset");
        let wasm = planned
            .assets
            .iter()
            .find(|asset| asset.logical_path == "/assets/compiler.wasm")
            .expect("wasm asset");
        let parsed: serde_json::Value = serde_json::from_slice(&session.bytes).unwrap();
        assert_eq!(parsed["mode"], "wasm");
        assert_eq!(parsed["html_runtime"]["available"], false);
        assert_eq!(parsed["compiler_wasm_url"], wasm.hashed_url);
        assert_eq!(parsed["worker_url"], worker.hashed_url);
        let languages: Vec<&str> = parsed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|doc| doc["language"].as_str().unwrap())
            .collect();
        assert!(languages.contains(&"rocci"), "{languages:?}");
        assert!(languages.contains(&"rocdown"), "{languages:?}");

        let roc = planned.pages_roc();
        assert!(roc.contains("playground_session: "));
        assert!(
            !guide.view.resources.csp.contains("wasm-unsafe-eval"),
            "{}",
            guide.view.resources.csp
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn playground_examples_manifest_loads_checked_in_sources() {
        let root = temp("playground-examples");
        write_site(&root);
        fs::create_dir_all(root.join("playground")).unwrap();
        fs::write(
            root.join("playground/index.rocdown"),
            "@page {\n    layout: \"playground\",\n    route: \"/playground/\",\n    meta: { title: \"Playground\" },\n}\n\n# Playground\n",
        )
        .unwrap();
        fs::write(
            root.join("sample.rocci"),
            "@component Hello = |_| {\n    <p>hi</p>\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("sample.rocdown"),
            "# Sample\n\nFrom the manifest.\n",
        )
        .unwrap();
        fs::write(
            root.join("playground/examples.toml"),
            r#"
[[example]]
id = "hello"
file = "sample.rocci"
language = "rocci"

[[example]]
id = "sample"
file = "sample.rocdown"
language = "rocdown"
"#,
        )
        .unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"

[[nav]]
label = "Playground"
items = ["playground/index"]
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let session = planned
            .assets
            .iter()
            .find(|asset| asset.logical_path == "/assets/playground-session.json")
            .expect("session");
        let parsed: serde_json::Value = serde_json::from_slice(&session.bytes).unwrap();
        assert_eq!(parsed["selected_document"], "hello");
        assert_eq!(
            parsed["documents"][0]["source"],
            "@component Hello = |_| {\n    <p>hi</p>\n}\n"
        );
        assert_eq!(
            parsed["documents"][1]["source"],
            "# Sample\n\nFrom the manifest.\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn site_without_playground_omits_playground_assets() {
        let root = temp("no-playground");
        write_site(&root);
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        assert!(planned.assets.iter().all(|asset| {
            !asset.logical_path.contains("playground")
                && !asset.logical_path.contains("compiler.wasm")
        }));
        let home = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/")
            .unwrap();
        assert!(home.view.resources.playground_session.is_empty());
        assert!(!home.view.resources.csp.contains("wasm-unsafe-eval"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hashed_names_are_deterministic_and_keep_directories() {
        let first = hashed_asset("icons/logo.png", b"logo-bytes");
        let second = hashed_asset("icons/logo.png", b"logo-bytes");
        assert_eq!(first.output_path, second.output_path);
        assert!(first.output_path.starts_with("assets/icons/logo."));
        assert!(first.output_path.ends_with(".png"));
        assert_ne!(first.output_path, "assets/icons/logo.png");
        let other = hashed_asset("icons/logo.png", b"other");
        assert_ne!(first.output_path, other.output_path);
    }

    #[test]
    fn plan_rewrites_article_html_and_social_image() {
        let root = temp("rewrite");
        write_site(&root);
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let home = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/")
            .unwrap();
        assert!(home.article_html.contains("/assets/og."));
        assert!(!home.article_html.contains("/assets/og.png"));
        assert!(home.view.site.social_image.starts_with("/assets/og."));
        assert_ne!(home.view.site.social_image, "/assets/og.png");
        assert!(home.view.site.favicon.starts_with("/assets/favicon."));
        assert_ne!(home.view.site.favicon, "/assets/favicon.svg");
        assert!(
            home.view
                .site
                .apple_touch_icon
                .starts_with("/assets/apple-touch-icon.")
        );
        assert_ne!(
            home.view.site.apple_touch_icon,
            "/assets/apple-touch-icon.png"
        );
        let guide = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/guide/")
            .unwrap();
        assert!(guide.article_html.contains("/assets/icons/logo."));
        assert!(!guide.article_html.contains("/assets/icons/logo.png"));
        assert_eq!(
            guide
                .view
                .breadcrumbs
                .iter()
                .map(|crumb| crumb.title.as_str())
                .collect::<Vec<_>>(),
            ["Rocci", "Guide"]
        );
        assert!(home.view.breadcrumbs.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plan_lists_404_stylesheet_redirects_and_discovery() {
        let root = temp("artifacts");
        write_site(&root);
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let artifacts = planned.artifacts();
        let kinds: Vec<_> = artifacts.iter().map(|item| item.kind).collect();
        assert!(kinds.contains(&"not_found"));
        assert!(kinds.contains(&"stylesheet"));
        assert!(kinds.contains(&"redirect"));
        assert!(kinds.contains(&"llms"));
        assert!(kinds.contains(&"pages"));
        assert!(kinds.contains(&"sitemap"));
        assert!(kinds.contains(&"robots"));
        let pages_json = planned
            .files
            .iter()
            .find(|file| file.output_path == "pages.json")
            .unwrap();
        assert_eq!(pages_json.route, "/pages.json");
        let listed: serde_json::Value = serde_json::from_str(&pages_json.contents).unwrap();
        assert!(listed.as_array().unwrap().iter().any(|page| {
            page["route"] == "/guide/" && page["title"] == "Guide" && page["kind"] == "static"
        }));
        assert!(artifacts.iter().any(|item| item.output_path == "404.html"));
        assert!(
            artifacts
                .iter()
                .any(|item| item.output_path == "old-guide/index.html")
        );
        assert!(planned.assets.iter().any(|asset| asset.kind == "stylesheet"
            && String::from_utf8_lossy(&asset.bytes).contains("forced-colors")));
        let recovery = planned
            .pages
            .iter()
            .find(|page| page.output_path == "404.html")
            .unwrap();
        assert_eq!(recovery.view.resources.csp, DEFAULT_CSP);
        assert_eq!(
            recovery
                .view
                .breadcrumbs
                .iter()
                .map(|crumb| crumb.title.as_str())
                .collect::<Vec<_>>(),
            ["Rocci", "Page not found"]
        );
        assert!(!recovery.view.sidebar.is_empty());
        let roc = planned.pages_roc();
        let not_found = roc.find("output_path: \"404.html\"").unwrap();
        let guide = roc.find("output_path: \"guide/index.html\"").unwrap();
        let index = roc.find("output_path: \"index.html\"").unwrap();
        assert!(not_found < guide);
        assert!(guide < index);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn inspect_artifacts_uses_the_plan() {
        let root = temp("inspect");
        write_site(&root);
        let json = inspect(&root, InspectKind::Artifacts, None).unwrap();
        assert!(json.contains("404.html"), "{json}");
        assert!(json.contains("old-guide/index.html"), "{json}");
        assert!(json.contains("theme."), "{json}");
        let report: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(report["datastar"], false);
        assert!(report["service_routes"].as_array().unwrap().is_empty());
        assert!(
            report["pages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|page| page["kind"] == "static" && page["route"] == "/"),
            "{json}"
        );
        assert!(
            report["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["output_path"] == "pages.json"),
            "{json}"
        );
        assert!(
            !report["artifacts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["output_path"] == "islands.json"),
            "{json}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pages_roc_is_stable_for_body_only_edits() {
        let root = temp("hash-body");
        write_site(&root);
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let first = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let first_roc = first.pages_roc();

        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n\nExtra paragraph.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let second = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        assert_eq!(first_roc, second.pages_roc());
        let home = |plan: &BuildPlan| {
            plan.pages
                .iter()
                .find(|page| page.view.route == "/")
                .unwrap()
                .article_html
                .clone()
        };
        assert_ne!(home(&first), home(&second));

        fs::write(
            root.join("index.rocdown"),
            "# Home changed\n\n![og](/assets/og.png)\n\nSee the [guide](/guide/).\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let third = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        assert_ne!(first_roc, third.pages_roc());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pages_roc_is_stable_for_docs_body_only_edits() {
        let root = temp("hash-docs-body");
        write_site(&root);
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:note[title: \"Watch\"] First body.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let first = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let first_roc = first.pages_roc();
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:note[title: \"Watch\"] Second body, still a note.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let second = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        assert_eq!(first_roc, second.pages_roc());
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:note[title: \"Changed\"] Second body, still a note.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        let third = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        assert_ne!(first_roc, third.pages_roc());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pages_roc_emits_typed_widget_tags_not_segment_bag() {
        let root = temp("typed-props");
        write_site(&root);
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:note[title: \"Watch\"] Body text.\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let roc = planned.pages_roc();
        assert!(roc.contains("HtmlFile({ path:"), "{roc}");
        assert!(roc.contains("Note({"), "{roc}");
        assert!(roc.contains("title: \"Watch\""), "{roc}");
        assert!(roc.contains("child_count:"), "{roc}");
        assert!(!roc.contains("tab_id"), "{roc}");
        assert!(!roc.contains("kind: \"note\""), "{roc}");
        let home = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/")
            .unwrap();
        assert!(
            home.segments
                .iter()
                .any(|node| node.widget_kind() == Some("note"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pages_roc_emits_tab_ids_on_typed_children() {
        let root = temp("tab-ids");
        write_site(&root);
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:tabs.begin[group: \"os\", kind: \"platform\"]\n    :tab[id: \"mac\", label: \"macOS\"] Mac panel.\n    :tab[id: \"linux\", label: \"Linux\"] Linux panel.\n:tabs.end\n",
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let roc = planned.pages_roc();
        assert!(roc.contains("Tabs({"), "{roc}");
        assert!(roc.contains("group: \"os\""), "{roc}");
        assert!(roc.contains("kind: \"platform\""), "{roc}");
        assert!(roc.contains("Tab({"), "{roc}");
        assert!(roc.contains("id: \"mac\""), "{roc}");
        assert!(roc.contains("id: \"linux\""), "{roc}");
        assert!(roc.contains("label: \"macOS\""), "{roc}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn hybrid_pages_keep_static_widgets_and_islands_in_authored_order() {
        let root = temp("dual-apply");
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Dual"
base_url = "https://rocci.dev"

[[nav]]
label = "Start"
items = ["index", "widgets"]
"#,
        )
        .unwrap();
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:note[title: \"Watch\"] Body text.\n",
        )
        .unwrap();
        fs::write(
            root.join("widgets.rocdown"),
            r#"
@page {
    route: "/widgets/",
    meta: { title: "Widgets" },
}

@component
FeatureCount = |_| {
    <p class="feature-count">3 core ideas</p>
}

# Widgets

:card-grid.begin
    :link-card[href: "/", title: "Start", summary: "First path."]
    :link-card[href: "/widgets/", title: "Project", summary: "Second path."]
:card-grid.end

@render FeatureCount({})
"#,
        )
        .unwrap();
        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let home_page = resolved
            .site
            .pages
            .iter()
            .find(|page| page.route == "/")
            .unwrap();
        let widgets_page = resolved
            .site
            .pages
            .iter()
            .find(|page| page.route == "/widgets/")
            .unwrap();
        assert_eq!(home_page.kind, PageKind::Static);
        assert_eq!(widgets_page.kind, PageKind::Hydrate);
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let roc = planned.pages_roc();
        assert!(roc.contains("Note({"), "{roc}");
        assert!(roc.contains("HtmlFile({ path:"), "{roc}");
        let home = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/")
            .unwrap();
        assert!(
            home.segments
                .iter()
                .any(|node| node.widget_kind() == Some("note")),
            "{:?}",
            home.segments
        );
        let widgets = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/widgets/")
            .unwrap();
        let card_segment = widgets
            .segments
            .iter()
            .position(|node| node.widget_kind() == Some("card-grid"))
            .expect("card-grid segment");
        assert!(
            card_segment + 1 < widgets.segments.len(),
            "{:?}",
            widgets.segments
        );
        assert!(
            widgets
                .fragments
                .iter()
                .any(|(_, html)| html.contains(crate::islands::PLACEHOLDER)),
            "{:?}",
            widgets.fragments
        );
        let roc = planned.pages_roc();
        let card_at = roc.find("CardGrid({").expect("card-grid in generated Roc");
        let island_at = roc[card_at..]
            .find("HtmlFile({ path:")
            .map(|offset| card_at + offset)
            .expect("island fragment after card-grid");
        assert!(card_at < island_at, "{roc}");
        assert!(widgets.view.resources.module_script.is_empty());
        assert!(widgets.view.resources.chrome_script.contains("goto."));
        assert!(rocci_ui::chrome_script().contains("__rocciCopy"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_local_theme_compiles_and_is_staged() {
        let root = temp("custom-theme");
        write_site(&root);
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
@component SiteShell = |view, content| {
    <html>
        <head>
            <title>{view.title} - Custom Theme</title>
        </head>
        <body>
            <header>Custom Header</header>
            <main>{content}</main>
        </body>
    </html>
}
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

        let type_names: Vec<_> = planned
            .theme_modules
            .iter()
            .map(|m| m.type_name.as_str())
            .collect();
        assert!(type_names.contains(&"SiteShell"));
        assert!(type_names.contains(&"RocdownTheme"));
        assert!(type_names.contains(&"DocsComponents"));
        assert!(type_names.contains(&"BlockPainters"));
        assert!(type_names.contains(&"RocdownBase"));

        let css = planned
            .theme_modules
            .iter()
            .flat_map(|module| module.styles.iter())
            .map(|style| style.css.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(css.contains("--canvas"), "{css}");
        assert!(css.contains(".rd-header-1"), "{css}");
        assert!(
            !css.contains("data-rocci-css~=\"RocdownBase"),
            "base article CSS must apply without a document stamp\n{css}"
        );

        let site_shell = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "SiteShell")
            .unwrap();
        assert!(
            !site_shell.roc.contains("Html.text(content)"),
            "{}",
            site_shell.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn block_pack_binds_note_to_pack_module() {
        let root = temp("block-pack-overlay");
        write_site(&root);
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
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

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let painters = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters.roc.contains("Blocks.note(props, content)"),
            "{}",
            painters.roc
        );
        assert!(
            painters.roc.contains("DocsComponents.tip(props, content)"),
            "{}",
            painters.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn generated_dispatcher_covers_widget_kinds_without_editing_runtime() {
        let arms = widget_kind_render_arms();
        for spec in crate::registry::KINDS
            .iter()
            .filter(|kind| kind.paints_as_widget())
        {
            let painter = roc_fn_name(spec.component);
            assert!(
                arms.contains(&format!("{}(seg)", spec.component)),
                "generated dispatcher missing tag `{}`: {arms}",
                spec.component
            );
            assert!(
                arms.contains(&format!("BlockPainters.{painter}")),
                "generated dispatcher missing `{painter}`: {arms}"
            );
        }
        let src = include_str!("../runtime/RocdownBuild.roc");
        assert!(
            src.contains("# rocci-widget-kind-arms"),
            "runtime dispatcher should splice generated arms"
        );
        assert!(
            !src.contains("Note(seg)"),
            "builtin widget arms should not be handwritten in RocdownBuild.roc"
        );
        assert!(src.contains("render_forest!"), "{src}");
        assert!(src.contains("HtmlFile"), "{src}");
    }

    fn write_shell(root: &Path) {
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Html

@component SiteShell = |view, content| {
    <html>
        <body>{content}</body>
    </html>
}
"#,
        )
        .unwrap();
    }

    #[test]
    fn blocks_pack_path_replaces_theme_blocks_convention() {
        let root = temp("blocks-pack-path");
        write_site(&root);
        write_shell(&root);
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Note = |{ title }, content|
    <section data-convention-note>{content}</section>
"#,
        )
        .unwrap();
        fs::write(
            root.join("theme/AltPack.rocci"),
            r#"
import Html

@component Note = |{ title }, content|
    <section data-pack-note>{content}</section>
"#,
        )
        .unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"

[blocks]
pack = "theme/AltPack.rocci"
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let painters = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters.roc.contains("AltPack.note(props, content)"),
            "{}",
            painters.roc
        );
        assert!(
            !painters.roc.contains("Blocks.note(props, content)"),
            "{}",
            painters.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn blocks_override_remaps_kind_to_pack_component() {
        let root = temp("blocks-override");
        write_site(&root);
        write_shell(&root);
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Callout = |{ title }, content|
    <section data-callout data-title={title}>{content}</section>
"#,
        )
        .unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"

[blocks.override]
note = "Callout"
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let painters = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters
                .roc
                .contains("note = |props, content|\n        Blocks.callout(props, content)"),
            "{}",
            painters.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pack_custom_kind_is_planned_and_dispatched() {
        let root = temp("pack-custom-callout");
        write_site(&root);
        write_shell(&root);
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Callout = |{ tone ?? "note" }, content|
    <aside data-test-callout data-tone={tone}>{content}</aside>
"#,
        )
        .unwrap();
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n:callout[tone: \"warn\"] {{\n    Watch this.\n}}\n",
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let roc = planned.pages_roc();
        assert!(roc.contains("Callout({"), "{roc}");
        assert!(roc.contains("tone: \"warn\""), "{roc}");
        assert!(
            planned
                .widget_render_arms
                .contains("BlockPainters.callout({ tone: seg.tone }, body)"),
            "{}",
            planned.widget_render_arms
        );
        assert!(
            planned
                .widget_render_arms
                .contains("BlockPainters.note({ title: seg.title }, body)"),
            "{}",
            planned.widget_render_arms
        );
        let painters = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters.roc.contains("callout = |props, content|"),
            "{}",
            painters.roc
        );
        assert!(
            painters.roc.contains("Blocks.callout(props, content)"),
            "{}",
            painters.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pack_reserved_helper_name_fails_site_load() {
        let root = temp("pack-reserved-page");
        write_site(&root);
        write_shell(&root);
        fs::write(
            root.join("theme/Blocks.rocci"),
            r#"
import Html

@component Page = |{ title }, content|
    <div>{content}</div>
"#,
        )
        .unwrap();

        let err = load_site(&root).unwrap_err().to_string();
        assert!(
            err.contains("reserved name")
                && err.contains("helpers must not live in the block pack"),
            "{err}"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn blocks_override_unknown_component_fails_theme_compile() {
        let root = temp("blocks-override-missing");
        write_site(&root);
        write_shell(&root);
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"

[blocks.override]
note = "Missing"
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let err = plan(&loaded.root, &loaded.config, &resolved.site)
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown component `Missing`"), "{err}");

        let _ = fs::remove_dir_all(root);
    }

    fn write_incomplete_docs(root: &Path) {
        write_shell(root);
        fs::write(
            root.join("theme/DocsComponents.rocci"),
            r#"
import Html

@component Tip = |{ title }, content|
    <p data-stub-tip data-title={title}>{content}</p>
"#,
        )
        .unwrap();
    }

    #[test]
    fn missing_painter_errors_unless_debug_or_preview() {
        let root = temp("missing-painter");
        write_site(&root);
        write_incomplete_docs(&root);

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let err = plan(&loaded.root, &loaded.config, &resolved.site)
            .unwrap_err()
            .to_string();
        assert!(err.contains("no renderer bound for kind `note`"), "{err}");

        let preview = plan_preview(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let painters = preview
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters.roc.contains("BlockDebug.debug({ kind: \"note\""),
            "{}",
            painters.roc
        );
        assert!(
            painters.roc.contains("data-rocci-block-debug")
                || preview
                    .theme_modules
                    .iter()
                    .any(|module| module.type_name == "BlockDebug"
                        && (module.src.contains("data-rocci-block-debug")
                            || module.roc.contains("data-rocci-block-debug"))),
            "debug component missing data-rocci-block-debug"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_painter_binds_debug_when_flag_set() {
        let root = temp("missing-painter-debug");
        write_site(&root);
        write_incomplete_docs(&root);
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

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let painters = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "BlockPainters")
            .unwrap();
        assert!(
            painters.roc.contains("BlockDebug.debug({ kind: \"note\""),
            "{}",
            painters.roc
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn project_layout_article_slot_is_html_body_param() {
        let root = temp("layout-body-param");
        write_site(&root);
        fs::create_dir_all(root.join("theme")).unwrap();
        fs::write(
            root.join("theme/SiteShell.rocci"),
            r#"
import Layouts

@component SiteShell = |view, content| {
    <html>
        <body>
            <Layouts.Home view={view}>{content}</Layouts.Home>
        </body>
    </html>
}
"#,
        )
        .unwrap();
        fs::write(
            root.join("theme/Layouts.rocci"),
            r#"
@component Home = |{ view }, content| {
    <article class="article">{content}</article>
}
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let layouts = planned
            .theme_modules
            .iter()
            .find(|module| module.type_name == "Layouts")
            .unwrap();
        assert!(
            !layouts.roc.contains("Html.text(content)"),
            "{}",
            layouts.roc
        );
        assert!(layouts.roc.contains("content"), "{}", layouts.roc);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn named_layouts_and_collection_metadata_are_propagated() {
        let root = temp("layouts-meta");
        write_site(&root);
        fs::write(
            root.join("guide.rocdown"),
            r#"
@page {
    layout: "plain",
    published: "2026-08-18",
    updated: "2026-08-19",
    authors: ["Nils", "Collaborator"],
    tags: ["guide", "release"],
    collection: "guides",
    summary: "A plain guide without docs sidebar",
}

# Guide

Content here.
"#,
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

        let guide = planned
            .pages
            .iter()
            .find(|p| p.view.route == "/guide/")
            .unwrap();
        assert_eq!(guide.view.layout, "plain");
        assert_eq!(guide.view.published, "2026-08-18");
        assert_eq!(guide.view.updated, "2026-08-19");
        assert_eq!(guide.view.authors, vec!["Nils", "Collaborator"]);
        assert_eq!(guide.view.tags, vec!["guide", "release"]);
        assert_eq!(guide.view.collection, "guides");
        assert_eq!(guide.view.description, "A plain guide without docs sidebar");

        let roc = planned.pages_roc();
        assert!(roc.contains("layout: \"plain\""));
        assert!(roc.contains("published: \"2026-08-18\""));
        assert!(roc.contains("authors: ["));
        assert!(roc.contains("\"Nils\""));
        assert!(roc.contains("collection: \"guides\""));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn unknown_layout_returns_rd2007_diagnostic() {
        let root = temp("bad-layout");
        write_site(&root);
        fs::write(
            root.join("guide.rocdown"),
            "@page {\n    layout: \"nonexistent_layout\"\n}\n\n# Guide\n",
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(resolved.has_errors());
        assert!(resolved.diagnostics.iter().any(
            |d| d.code == "RD2007" && d.message.contains("unknown layout `nonexistent_layout`")
        ));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn collection_sorting_and_feed_generation_in_plan() {
        let root = temp("news-collection");
        write_site(&root);
        fs::create_dir_all(root.join("news")).unwrap();
        fs::write(
            root.join("news/index.rocdown"),
            "@page {\n    layout: \"news-index\",\n}\n\n# News\n",
        )
        .unwrap();
        fs::write(
            root.join("news/older.rocdown"),
            "@page {\n    layout: \"news-post\",\n    published: \"2026-08-10\",\n    collection: \"news\",\n    summary: \"Older post\",\n}\n\n# Older\n",
        )
        .unwrap();
        fs::write(
            root.join("news/newer.rocdown"),
            "@page {\n    layout: \"news-post\",\n    published: \"2026-08-18\",\n    collection: \"news\",\n    summary: \"Newer post\",\n}\n\n# Newer\n",
        )
        .unwrap();
        fs::write(
            root.join("news/draft.rocdown"),
            "@page {\n    draft: True,\n    layout: \"news-post\",\n    published: \"2026-08-20\",\n    collection: \"news\",\n}\n\n# Draft\n",
        )
        .unwrap();

        let loaded = load_site(&root).unwrap();
        let resolved = resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{}", resolved.error_summary());
        let planned = plan(&loaded.root, &loaded.config, &resolved.site).unwrap();

        let news_index = planned
            .pages
            .iter()
            .find(|p| p.view.route == "/news/")
            .unwrap();
        assert_eq!(news_index.view.collection_items.len(), 2);
        assert_eq!(news_index.view.collection_items[0].title, "Newer");
        assert_eq!(news_index.view.collection_items[0].published, "2026-08-18");
        assert_eq!(news_index.view.collection_items[1].title, "Older");
        assert_eq!(news_index.view.collection_items[1].published, "2026-08-10");

        let home = planned.pages.iter().find(|p| p.view.route == "/").unwrap();
        assert_eq!(home.view.collection_items.len(), 2);
        assert_eq!(home.view.collection_items[0].title, "Newer");

        let feed = planned
            .files
            .iter()
            .find(|f| f.output_path == "news/feed.xml")
            .unwrap();
        assert_eq!(feed.kind, "feed");
        assert!(feed.contents.contains("<title>Newer</title>"));
        assert!(feed.contents.contains("<title>Older</title>"));
        assert!(!feed.contents.contains("Draft"));

        let roc = planned.pages_roc();
        assert!(roc.contains("collection_items: ["));
        assert!(roc.contains("title: \"Newer\""));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn builtin_theme_keeps_phone_menu_and_table_wrap() {
        let theme = runtime::THEME;
        assert!(theme.contains("id=\"site-nav\""));
        assert!(theme.contains("class=\"mobile-menu\""));
        assert!(theme.contains("@media (max-width: 70rem)"));
        assert!(theme.contains("@media (max-width: 48rem)"));
        assert!(theme.contains(".mobile-menu { position: relative; display: block"));
        assert!(theme.contains("100dvh"));
        assert!(theme.contains("env(safe-area-inset-top"));
        assert!(theme.contains(
            "min-height: calc(100vh - var(--header-height) - var(--rocci-chrome-bottom, 0px))"
        ));
        assert!(theme.contains(
            "height: calc(100vh - var(--header-height) - var(--rocci-chrome-bottom, 0px))"
        ));
        assert!(theme.contains(
            "max-height: calc(100vh - var(--header-height) - env(safe-area-inset-top, 0px) - var(--rocci-chrome-bottom, 0px))"
        ));
        let panel = theme.find("class=\"mobile-panel\"").expect("mobile panel");
        let details_end = theme[panel..].find("</details>").expect("details close");
        assert!(
            theme[panel..panel + details_end].contains("PageOutline.pageOutline"),
            "phone menu must include on-this-page links"
        );
        assert!(runtime::BASE.contains("rd-table-wrap"));
        assert!(runtime::BASE.contains("overflow-x: auto"));
        assert!(!runtime::BASE.contains("overflow: hidden"));
    }
}
