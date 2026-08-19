use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, Segment, SourceFile, compile, format_diagnostic, wrap_type_module,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::article::PageKind;
use crate::catalog::{self, NavLink, NavSection, PageHeading, ResolvedPage, ResolvedSite};
use crate::config::SiteConfig;
use crate::runtime;
use crate::service::{IslandRoute, island_routes, live_csp};

pub const DEFAULT_CSP: &str = "default-src 'none'; script-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'none'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

const HASH_LEN: usize = 16;

pub use rocci_ui::{
    BreadcrumbView, CollectionItemView, LaneView, NavItemView, OutlineView, PageView, ResourceView,
    SiteView,
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

#[derive(Debug, Clone, Serialize)]
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
    let theme_modules = compile_theme_modules(root, config)?;
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
    let site_view = SiteView {
        title: config.site.title.clone(),
        description: config.site.description.clone(),
        base_url: config.site.base_url.clone(),
        language: config.site.language.clone(),
        repository: config.site.repository.clone(),
        social_image,
        subtitle: config.site.subtitle.clone(),
        footer: config.site.footer.clone(),
    };

    let published: Vec<_> = site
        .pages
        .iter()
        .filter(|page| !page.draft)
        .cloned()
        .collect();

    let has_playground = published.iter().any(|p| {
        crate::docs::collect_kinds(&p.article)
            .iter()
            .any(|k| k == "playground")
    });

    let (playground_app_url, playground_css_url) = if has_playground {
        let app_asset = hashed_asset("playground-app.js", runtime::PLAYGROUND_APP_JS);
        let worker_asset = hashed_asset("playground-worker.js", runtime::PLAYGROUND_WORKER_JS);
        let css_asset = hashed_asset("playground-styles.css", runtime::PLAYGROUND_STYLES_CSS);
        let wasm_asset = hashed_asset("compiler.wasm", runtime::PLAYGROUND_COMPILER_WASM);

        let app_url = app_asset.hashed_url.clone();
        let css_url = css_asset.hashed_url.clone();

        assets.push(app_asset);
        assets.push(worker_asset);
        assets.push(css_asset);
        assets.push(wasm_asset);

        (Some(app_url), Some(css_url))
    } else {
        (None, None)
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
        island_routes(root, site)?
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
            config.sidebar_tree,
            &stylesheet_url,
            &csp,
            playground_app_url.as_deref(),
            playground_css_url.as_deref(),
            datastar_url.as_deref(),
            &config.http.service_origin,
            &rewrite,
            collection_items,
            false,
        ));
    }
    pages.push(not_found_page(
        &site_view,
        &site.navigation,
        config.sidebar_tree,
        &stylesheet_url,
        &csp,
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
    })
}

fn compile_theme_modules(root: &Path, config: &SiteConfig) -> Result<Vec<CompiledThemeModule>> {
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
        compile_project_theme(root, &target)
    } else {
        compile_builtin_theme()
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
    Ok(vec![base, breadcrumbs, nav_list, page_outline, theme, docs])
}

#[allow(clippy::too_many_arguments)]
fn planned_page(
    page: &ResolvedPage,
    site: &SiteView,
    navigation: &[NavSection],
    sidebar_tree: bool,
    stylesheet: &str,
    csp: &str,
    playground_app: Option<&str>,
    playground_css: Option<&str>,
    datastar_url: Option<&str>,
    service_origin: &str,
    rewrite: &BTreeMap<String, String>,
    collection_items: Vec<CollectionItemView>,
    not_found: bool,
) -> PlannedPage {
    let current_id = if not_found {
        None
    } else {
        Some(page.id.as_str())
    };
    let (lanes, sidebar) = lanes_and_sidebar(navigation, current_id, sidebar_tree);
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
    let (segments, fragments) = crate::docs::plan_segments(&article_name, &page.article, rewrite);
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

    let page_has_playground = !not_found
        && crate::docs::collect_kinds(&page.article)
            .iter()
            .any(|k| k == "playground");

    let (page_csp, module_script, playground_css_val) = if page_has_playground {
        (
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'".to_string(),
            playground_app.unwrap_or_default().to_string(),
            playground_css.unwrap_or_default().to_string(),
        )
    } else if !not_found && page.kind == PageKind::Live {
        (
            live_csp(service_origin),
            datastar_url.unwrap_or_default().to_string(),
            String::new(),
        )
    } else {
        (csp.to_string(), String::new(), String::new())
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
            breadcrumbs: page.breadcrumbs.iter().map(breadcrumb_from_link).collect(),
            previous: optional_link(page.previous.as_ref()),
            next: optional_link(page.next.as_ref()),
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: page_csp,
                canonical,
                module_script,
                playground_css: playground_css_val,
            },
        },
    }
}

fn not_found_page(
    site: &SiteView,
    navigation: &[NavSection],
    sidebar_tree: bool,
    stylesheet: &str,
    csp: &str,
) -> PlannedPage {
    let home = navigation
        .iter()
        .find(|section| section.items.iter().any(|item| item.route == "/"))
        .and_then(|section| section.items.iter().find(|item| item.route == "/"))
        .map(|item| item.id.as_str());
    let (lanes, sidebar) = lanes_and_sidebar(navigation, home, sidebar_tree);
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
            description: "This page does not exist.".into(),
            layout: "not-found".into(),
            published: String::new(),
            updated: String::new(),
            authors: Vec::new(),
            tags: Vec::new(),
            collection: String::new(),
            collection_items: Vec::new(),
            outline: Vec::new(),
            breadcrumbs: Vec::new(),
            previous: NavItemView {
                title: String::new(),
                href: String::new(),
                class_name: String::new(),
            },
            next: NavItemView {
                title: String::new(),
                href: String::new(),
                class_name: String::new(),
            },
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: csp.to_string(),
                canonical: String::new(),
                module_script: String::new(),
                playground_css: String::new(),
            },
        },
    }
}

fn not_found_html() -> String {
    String::from(
        "<h1 class=\"rd-header-1\">Page not found</h1>\n<p class=\"rd-paragraph\">This page does not exist. Return to the <a class=\"rd-link\" href=\"/\">home page</a>.</p>\n",
    )
}

fn lanes_and_sidebar(
    navigation: &[NavSection],
    current_id: Option<&str>,
    sidebar_tree: bool,
) -> (Vec<LaneView>, Vec<NavItemView>) {
    let current_section = current_id.and_then(|id| {
        navigation
            .iter()
            .find(|section| section.items.iter().any(|item| item.id == id))
    });
    let lanes = if sidebar_tree {
        Vec::new()
    } else {
        navigation
            .iter()
            .map(|section| LaneView {
                label: section.label.clone(),
                href: section
                    .items
                    .first()
                    .map(|item| item.route.clone())
                    .unwrap_or_else(|| "/".into()),
                current: current_section.is_some_and(|current| current.label == section.label),
            })
            .collect()
    };
    let sidebar = if sidebar_tree {
        let mut items = Vec::new();
        for section in navigation {
            let Some(category) = section.items.first() else {
                continue;
            };
            let expanded = current_section.is_some_and(|current| current.label == section.label);
            let category_current = current_id == Some(category.id.as_str());
            let class_name = match (expanded, category_current) {
                (true, true) => "nav-link nav-category is-expanded is-current",
                (true, false) => "nav-link nav-category is-expanded",
                (false, _) => "nav-link nav-category",
            };
            items.push(NavItemView {
                title: section.label.clone(),
                href: category.route.clone(),
                class_name: class_name.into(),
            });
            if expanded {
                items.extend(section.items.iter().skip(1).map(|item| NavItemView {
                    title: item.title.clone(),
                    href: item.route.clone(),
                    class_name: if current_id == Some(item.id.as_str()) {
                        "nav-link nav-child is-current".into()
                    } else {
                        "nav-link nav-child".into()
                    },
                }));
            }
        }
        items
    } else {
        current_section
            .map(|section| {
                section
                    .items
                    .iter()
                    .map(|item| NavItemView {
                        title: item.title.clone(),
                        href: item.route.clone(),
                        class_name: if current_id == Some(item.id.as_str()) {
                            "nav-link is-current".into()
                        } else {
                            "nav-link".into()
                        },
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    (lanes, sidebar)
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
        for item in &page.view.sidebar {
            out.push_str("                    { title: ");
            push_roc_string(&mut out, &item.title);
            out.push_str(", href: ");
            push_roc_string(&mut out, &item.href);
            out.push_str(", class_name: ");
            push_roc_string(&mut out, &item.class_name);
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                route: ");
        push_roc_string(&mut out, &page.view.route);
        out.push_str(",\n                title: ");
        push_roc_string(&mut out, &page.view.title);
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
        out.push_str(",\n                    playground_css: ");
        push_roc_string(&mut out, &page.view.resources.playground_css);
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
            let paint_content = spec.is_some_and(|kind| kind.paint_content());
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

    fn temp(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("rocdown-plan-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_site(root: &Path) {
        fs::create_dir_all(root.join("assets/icons")).unwrap();
        fs::write(root.join("assets/og.png"), b"og-bytes").unwrap();
        fs::write(root.join("assets/icons/logo.png"), b"logo-bytes").unwrap();
        fs::write(
            root.join("rocdown.toml"),
            r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev"
social_image = "/assets/og.png"
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
    fn default_csp_is_strict_and_stable() {
        assert_eq!(
            DEFAULT_CSP,
            "default-src 'none'; script-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'none'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'"
        );
        assert!(!DEFAULT_CSP.contains("unsafe-eval"));
        assert!(!DEFAULT_CSP.contains("unsafe-inline"));
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
        let guide = planned
            .pages
            .iter()
            .find(|page| page.view.route == "/guide/")
            .unwrap();
        assert!(guide.article_html.contains("/assets/icons/logo."));
        assert!(!guide.article_html.contains("/assets/icons/logo.png"));
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
        assert_eq!(
            planned
                .pages
                .iter()
                .find(|page| page.output_path == "404.html")
                .unwrap()
                .view
                .resources
                .csp,
            DEFAULT_CSP
        );
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
            "@page {\n    draft: Bool.true,\n    layout: \"news-post\",\n    published: \"2026-08-20\",\n    collection: \"news\",\n}\n\n# Draft\n",
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
}
