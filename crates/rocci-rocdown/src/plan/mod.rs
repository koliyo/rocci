use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::article::PageKind;
use crate::catalog::{self, NavSection, ResolvedPage, ResolvedSite};
use crate::config::SiteConfig;
use crate::runtime;
use crate::service::{IslandRoute, island_routes_with_service, live_csp};

mod assets;
mod emit;
mod nav;
mod playground;
mod theme;

pub const DEFAULT_CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

#[allow(unused_imports)]
pub use rocci_ui::{
    BreadcrumbView, CollectionItemView, LaneView, NavGroupView, NavItemView, OutlineView, PageView,
    ResourceView, SiteView,
};

pub use assets::PlannedAsset;
pub use theme::CompiledThemeModule;

pub(crate) use assets::{
    HASH_LEN, datastar_js_bytes, hash_site_assets, hashed_asset, hex_sha256, rewrite_map,
    rewrite_urls,
};
pub(crate) use emit::{discovery_files, not_found_page, pages_roc, publish_pages, redirect_html};
pub(crate) use nav::{
    attach_example_source_tree, lanes_and_sidebar, normalized_breadcrumbs, optional_link,
    outline_view, sidebar_has_current,
};
pub(crate) use playground::{PLAYGROUND_CSP, page_uses_playground, playground_session_bytes};
pub(crate) use theme::{
    compile_theme_with_painters, infer_site_pack_kinds, site_has_block_pack,
    validate_theme_painters, widget_kind_render_arms,
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
pub struct PlannedFile {
    pub kind: &'static str,
    pub route: String,
    pub output_path: String,
    pub contents: String,
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
        pages.push(planned_page(PagePlanInput {
            page,
            site: &site_view,
            navigation: &site.navigation,
            stylesheet: &stylesheet_url,
            csp: &csp,
            chrome_script: &chrome_script_url,
            playground_app: playground_app_url.as_deref(),
            playground_css: playground_css_url.as_deref(),
            playground_session: playground_session_url.as_deref(),
            datastar_url: datastar_url.as_deref(),
            service_origin: &config.http.service_origin,
            rewrite: &rewrite,
            all_pages: &published,
            collection_items,
            not_found: false,
        }));
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

pub(crate) fn document_title(title: &str, site_title: &str) -> String {
    let title = title.trim();
    let site_title = site_title.trim();

    if title.contains(site_title) {
        title.to_string()
    } else {
        format!("{title} · {site_title}")
    }
}

struct PagePlanInput<'a> {
    page: &'a ResolvedPage,
    site: &'a SiteView,
    navigation: &'a [NavSection],
    stylesheet: &'a str,
    csp: &'a str,
    chrome_script: &'a str,
    playground_app: Option<&'a str>,
    playground_css: Option<&'a str>,
    playground_session: Option<&'a str>,
    datastar_url: Option<&'a str>,
    service_origin: &'a str,
    rewrite: &'a BTreeMap<String, String>,
    all_pages: &'a [ResolvedPage],
    collection_items: Vec<CollectionItemView>,
    not_found: bool,
}

fn planned_page(input: PagePlanInput<'_>) -> PlannedPage {
    let PagePlanInput {
        page,
        site,
        navigation,
        stylesheet,
        csp,
        chrome_script,
        playground_app,
        playground_css,
        playground_session,
        datastar_url,
        service_origin,
        rewrite,
        all_pages,
        collection_items,
        not_found,
    } = input;
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

#[cfg(test)]
mod tests;
