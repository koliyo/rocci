use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use rocci_template::{
    LowerOptions, Segment, SourceFile, compile, format_diagnostic, type_name_from_path,
    wrap_type_module,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::catalog::{self, NavLink, NavSection, PageHeading, ResolvedPage, ResolvedSite};
use crate::config::SiteConfig;
use crate::runtime;

pub const DEFAULT_CSP: &str =
    "default-src 'none'; script-src 'none'; style-src 'self'; img-src 'self' data:; font-src 'self'; connect-src 'none'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'";

const HASH_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageView {
    pub site: SiteView,
    pub lanes: Vec<LaneView>,
    pub sidebar: Vec<NavItemView>,
    pub route: String,
    pub title: String,
    pub description: String,
    pub outline: Vec<OutlineView>,
    pub breadcrumbs: Vec<NavItemView>,
    pub previous: NavItemView,
    pub next: NavItemView,
    pub resources: ResourceView,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteView {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
    pub repository: String,
    pub social_image: String,
    pub subtitle: String,
    pub footer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneView {
    pub label: String,
    pub href: String,
    pub current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavItemView {
    pub title: String,
    pub href: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineView {
    pub id: String,
    pub title: String,
    pub level: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceView {
    pub stylesheet: String,
    pub csp: String,
    pub canonical: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedPage {
    pub article_path: String,
    pub output_path: String,
    pub article_html: String,
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
    pub theme_roc: String,
    pub theme_src: String,
    pub theme_segments: Vec<Segment>,
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

    pub fn pages_roc(&self) -> String {
        pages_roc(&self.pages)
    }
}

pub fn plan(root: &Path, config: &SiteConfig, site: &ResolvedSite) -> Result<BuildPlan> {
    let compiled = compile_theme()?;
    let mut assets = hash_site_assets(root, config)?;
    let theme_css = compiled
        .styles
        .iter()
        .map(|style| style.css.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let stylesheet = hashed_asset("theme.css", theme_css.as_bytes());
    assets.push(stylesheet.clone());
    assets.sort_by(|a, b| a.output_path.cmp(&b.output_path));

    let rewrite = rewrite_map(&assets);
    let rewritten_css = rewrite_urls(&theme_css, &rewrite);
    if let Some(asset) = assets.iter_mut().find(|asset| asset.kind == "stylesheet") {
        if rewritten_css != theme_css {
            *asset = hashed_asset("theme.css", rewritten_css.as_bytes());
        }
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

    let mut pages = Vec::new();
    for page in &published {
        pages.push(planned_page(
            page,
            &site_view,
            &site.navigation,
            &stylesheet_url,
            &csp,
            &rewrite,
            false,
        ));
    }
    pages.push(not_found_page(
        &site_view,
        &site.navigation,
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

    let files = discovery_files(config, &published);

    Ok(BuildPlan {
        pages,
        redirects,
        assets,
        files,
        theme_roc: compiled.roc,
        theme_src: compiled.src,
        theme_segments: compiled.segments,
    })
}

struct CompiledTheme {
    roc: String,
    src: String,
    segments: Vec<Segment>,
    styles: Vec<rocci_template::StyleArtifact>,
}

fn compile_theme() -> Result<CompiledTheme> {
    let src = runtime::THEME.to_string();
    let compiled = compile(
        SourceFile::new("RocsTheme.rocci", &src),
        &LowerOptions {
            embed_css: false,
            ..LowerOptions::default()
        },
    );
    for diagnostic in &compiled.diagnostics {
        eprintln!(
            "{}",
            format_diagnostic(SourceFile::new("RocsTheme.rocci", &src), diagnostic)
        );
    }
    if compiled.has_errors() {
        bail!("RocsTheme.rocci compilation failed");
    }
    if compiled.roc.contains("import Datastar") {
        bail!("RocsTheme.rocci uses Datastar, which the rocs runtime does not stage");
    }
    Ok(CompiledTheme {
        roc: wrap_type_module(&compiled.roc, "RocsTheme"),
        src,
        segments: compiled.segments,
        styles: compiled.styles,
    })
}

fn planned_page(
    page: &ResolvedPage,
    site: &SiteView,
    navigation: &[NavSection],
    stylesheet: &str,
    csp: &str,
    rewrite: &BTreeMap<String, String>,
    not_found: bool,
) -> PlannedPage {
    let current_id = if not_found { None } else { Some(page.id.as_str()) };
    let (lanes, sidebar) = lanes_and_sidebar(navigation, current_id);
    let canonical = if not_found || site.base_url.is_empty() {
        String::new()
    } else {
        format!("{}{}", site.base_url, page.route)
    };
    let article_html = rewrite_urls(&page.article_html, rewrite);
    let type_name = if not_found {
        "NotFound".to_string()
    } else {
        type_name_from_path(Path::new(&page.source_path))
    };
    PlannedPage {
        article_path: format!("articles/{type_name}.html"),
        output_path: page.output_path.clone(),
        article_html,
        view: PageView {
            site: site.clone(),
            lanes,
            sidebar,
            route: page.route.clone(),
            title: page.title.clone(),
            description: page.description.clone(),
            outline: page
                .headings
                .iter()
                .filter(|heading| (2..=3).contains(&heading.level))
                .map(outline_view)
                .collect(),
            breadcrumbs: page.breadcrumbs.iter().map(nav_from_link).collect(),
            previous: optional_link(page.previous.as_ref()),
            next: optional_link(page.next.as_ref()),
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: csp.to_string(),
                canonical,
            },
        },
    }
}

fn not_found_page(
    site: &SiteView,
    navigation: &[NavSection],
    stylesheet: &str,
    csp: &str,
) -> PlannedPage {
    let home = navigation
        .iter()
        .find(|section| section.items.iter().any(|item| item.route == "/"))
        .and_then(|section| section.items.iter().find(|item| item.route == "/"))
        .map(|item| item.id.as_str());
    let (lanes, sidebar) = lanes_and_sidebar(navigation, home);
    PlannedPage {
        article_path: "articles/NotFound.html".into(),
        output_path: "404.html".into(),
        article_html: not_found_html(),
        view: PageView {
            site: site.clone(),
            lanes,
            sidebar,
            route: "/404.html".into(),
            title: "Page not found".into(),
            description: "This page does not exist.".into(),
            outline: Vec::new(),
            breadcrumbs: Vec::new(),
            previous: NavItemView {
                title: String::new(),
                href: String::new(),
            },
            next: NavItemView {
                title: String::new(),
                href: String::new(),
            },
            resources: ResourceView {
                stylesheet: stylesheet.to_string(),
                csp: csp.to_string(),
                canonical: String::new(),
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
) -> (Vec<LaneView>, Vec<NavItemView>) {
    let current_section = current_id.and_then(|id| {
        navigation
            .iter()
            .find(|section| section.items.iter().any(|item| item.id == id))
    });
    let lanes = navigation
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
        .collect();
    let sidebar = current_section
        .map(|section| {
            section
                .items
                .iter()
                .map(|item| NavItemView {
                    title: item.title.clone(),
                    href: item.route.clone(),
                })
                .collect()
        })
        .unwrap_or_default();
    (lanes, sidebar)
}

fn outline_view(heading: &PageHeading) -> OutlineView {
    OutlineView {
        id: heading.id.clone(),
        title: heading.text.clone(),
        level: heading.level.to_string(),
    }
}

fn nav_from_link(link: &NavLink) -> NavItemView {
    NavItemView {
        title: link.title.clone(),
        href: link.route.clone(),
    }
}

fn optional_link(link: Option<&NavLink>) -> NavItemView {
    match link {
        Some(link) => nav_from_link(link),
        None => NavItemView {
            title: String::new(),
            href: String::new(),
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

fn collect_asset_files(dir: &Path, prefix: &Path, files: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
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

fn discovery_files(config: &SiteConfig, pages: &[ResolvedPage]) -> Vec<PlannedFile> {
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
    }
    files.sort_by(|a, b| a.output_path.cmp(&b.output_path));
    files
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
    let mut out = String::from("RocsPages := [].{\n    pages = [\n");
    for page in &pages {
        out.push_str("        {\n            article_path: ");
        push_roc_string(&mut out, &page.article_path);
        out.push_str(",\n            output_path: ");
        push_roc_string(&mut out, &page.output_path);
        out.push_str(",\n            view: {\n                site: {\n                    title: ");
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
            out.push_str(" },\n");
        }
        out.push_str("                ],\n                route: ");
        push_roc_string(&mut out, &page.view.route);
        out.push_str(",\n                title: ");
        push_roc_string(&mut out, &page.view.title);
        out.push_str(",\n                description: ");
        push_roc_string(&mut out, &page.view.description);
        out.push_str(",\n                outline: [\n");
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
        out.push_str("\n                }\n            }\n        },\n");
    }
    out.push_str("    ]\n}\n");
    out
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
        let path = env::temp_dir().join(format!("rocs-plan-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_site(root: &Path) {
        fs::create_dir_all(root.join("assets/icons")).unwrap();
        fs::write(root.join("assets/og.png"), b"og-bytes").unwrap();
        fs::write(root.join("assets/icons/logo.png"), b"logo-bytes").unwrap();
        fs::write(
            root.join("rocs.toml"),
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
        assert!(
            !resolved.has_errors(),
            "{}",
            resolved.error_summary()
        );
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
        assert!(kinds.contains(&"sitemap"));
        assert!(kinds.contains(&"robots"));
        assert!(
            artifacts
                .iter()
                .any(|item| item.output_path == "404.html")
        );
        assert!(
            artifacts
                .iter()
                .any(|item| item.output_path == "old-guide/index.html")
        );
        assert!(
            planned
                .assets
                .iter()
                .any(|asset| asset.kind == "stylesheet"
                    && String::from_utf8_lossy(&asset.bytes).contains("forced-colors"))
        );
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
        let _ = fs::remove_dir_all(root);
    }
}
