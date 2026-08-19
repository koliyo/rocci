use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rocci_template::{SourceFile, format_diagnostic};
use serde::Serialize;

use crate::{CompileOptions, Document, Item, MdNode, compile};

use crate::article::{PageClass, PageKind, classify_document, render_document};
use crate::catalog::{
    self, CatalogDiagnostic, Edge, NavSection, PageHeading, ResolveOptions, ResolvedPage,
    ResolvedSite, RouteHint, Severity, SourcePage,
};
use crate::config::{SiteConfig, load_config};
use crate::docs::{self, IncludeOptions};

#[derive(Debug, Clone)]
pub struct LoadedSite {
    pub root: PathBuf,
    pub config: SiteConfig,
    pub sources: Vec<SourcePage>,
    pub files: BTreeSet<String>,
    pub static_files: Vec<StaticFile>,
    pub diagnostics: Vec<CatalogDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct StaticFile {
    pub source: PathBuf,
    pub output_path: String,
}

pub fn find_site_root(path: &Path) -> Option<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    let mut dir = if path.is_dir() {
        path
    } else {
        path.parent()?.to_path_buf()
    };
    loop {
        if dir.join(crate::config::CONFIG_FILE).is_file() {
            return Some(dir);
        }
        dir = dir.parent()?.to_path_buf();
    }
}

pub fn site_preview_route(root: &Path, file: &Path) -> String {
    let file = if file.is_absolute() {
        file.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(file))
            .unwrap_or_else(|_| file.to_path_buf())
    };
    let src = std::fs::read_to_string(&file).unwrap_or_default();
    let page = crate::page_ref_from_source(&file, &src);
    if page.explicit_route {
        return catalog::with_trailing_slash(&page.route);
    }
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let file = std::fs::canonicalize(&file).unwrap_or(file);
    let rel = file
        .strip_prefix(&root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| {
            file.file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
    let id = rel
        .strip_suffix(".rocdown")
        .or_else(|| rel.strip_suffix(".markdown"))
        .or_else(|| rel.strip_suffix(".md"))
        .unwrap_or(&rel);
    catalog::derived_route(id)
}

pub fn content_root(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if path.is_file() {
        let is_doc = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "rocdown" || ext == "md" || ext == "markdown");
        if !is_doc {
            bail!(
                "{} is not a documentation root, .rocdown, or .md file",
                path.display()
            );
        }
        return path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow::anyhow!("{} has no parent directory", path.display()));
    }
    if !path.is_dir() {
        bail!(
            "{} is not a directory or documentation file",
            path.display()
        );
    }
    Ok(path)
}

struct DiscoveredPage {
    path: PathBuf,
    relative_name: String,
    mount_prefix: String,
    default_layout: Option<String>,
}

pub fn load_site(root: &Path) -> Result<LoadedSite> {
    let root = content_root(root)?;
    let config = load_config(&root)?;
    let mut discovered_pages = Vec::new();
    let mut root_files = Vec::new();
    crate::build::discover_in(&root, &mut root_files)?;
    root_files.sort();
    for path in root_files {
        let relative_name = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        discovered_pages.push(DiscoveredPage {
            path,
            relative_name,
            mount_prefix: String::new(),
            default_layout: None,
        });
    }

    for (index, mount) in config.mounts.iter().enumerate() {
        let mount_dir = root.join(&mount.source);
        if !mount_dir.is_dir() {
            bail!(
                "mount[{}] source `{}` does not exist or is not a directory in {}",
                index + 1,
                mount.source,
                root.display()
            );
        }
        let mut mount_files = Vec::new();
        crate::build::discover_in(&mount_dir, &mut mount_files)?;
        mount_files.sort();
        for path in mount_files {
            let rel_in_mount = path
                .strip_prefix(&mount_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let relative_name = if mount.prefix.is_empty() {
                rel_in_mount
            } else {
                format!("{}/{}", mount.prefix, rel_in_mount)
            };
            discovered_pages.push(DiscoveredPage {
                path,
                relative_name,
                mount_prefix: mount.prefix.clone(),
                default_layout: mount.layout.clone(),
            });
        }
    }

    if discovered_pages.is_empty() {
        bail!("no .rocdown files in {}", root.display());
    }

    let files = collect_files(&root, &config)?;
    let snippet_roots = snippet_roots(&root, &config)?;
    let mut sources = Vec::new();
    let mut diagnostics = Vec::new();

    for page_info in &discovered_pages {
        let path = &page_info.path;
        let relative_name = page_info.relative_name.clone();
        let src = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let name = path.display().to_string();
        let compiled = compile(
            SourceFile::new(&name, &src),
            &CompileOptions {
                resolve_links: false,
                resolve_includes: false,
                ..CompileOptions::default()
            },
        );
        for diagnostic in &compiled.diagnostics {
            let code = if diagnostic.is_error() {
                "RD1001"
            } else {
                "RD1002"
            };
            let severity = if diagnostic.is_error() {
                Severity::Error
            } else {
                Severity::Warning
            };
            diagnostics.push(CatalogDiagnostic {
                code,
                severity,
                path: relative_name.clone(),
                message: diagnostic.message.clone(),
            });
            if std::env::var_os("ROCDOWN_QUIET").is_none() {
                eprintln!(
                    "{}",
                    format_diagnostic(SourceFile::new(&name, &src), diagnostic)
                );
            }
        }
        let has_use = compiled
            .document
            .items
            .iter()
            .any(|item| matches!(item, Item::Use(_)));
        if has_use {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2301",
                &relative_name,
                format!(
                    "{relative_name} contains `@use`; custom static blocks belong in the compiled theme"
                ),
            ));
        }
        if compiled.has_errors() || has_use {
            continue;
        }
        let class = classify_document(&compiled.document, compiled.roc.contains("import Datastar"));
        if let Some(diagnostic) = page_kind_diagnostic(&relative_name, class) {
            diagnostics.push(diagnostic);
        }
        const VALID_LAYOUTS: &[&str] = &[
            "home",
            "product",
            "section",
            "docs",
            "news-index",
            "news-post",
            "plain",
            "not-found",
        ];
        let layout = if let Some(layout) = compiled.page_meta.layout.as_deref() {
            let clean = layout.trim().trim_matches('"');
            if !VALID_LAYOUTS.contains(&clean) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2007",
                    &relative_name,
                    format!(
                        "unknown layout `{layout}`; expected one of {}",
                        VALID_LAYOUTS.join(", ")
                    ),
                ));
            }
            clean.to_string()
        } else if let Some(default_layout) = &page_info.default_layout {
            default_layout.clone()
        } else if relative_name == "index.rocdown" || relative_name == "index.md" {
            "home".to_string()
        } else {
            "docs".to_string()
        };
        let derived_id = relative_name
            .strip_suffix(".rocdown")
            .unwrap_or(&relative_name)
            .to_string();
        let id_explicit = compiled.page_meta.id.is_some();
        let page_id = match compiled.page_meta.id.clone() {
            Some(id) => {
                if page_info.mount_prefix.is_empty()
                    || id.starts_with(&format!("{}/", page_info.mount_prefix))
                {
                    id
                } else {
                    format!("{}/{}", page_info.mount_prefix, id)
                }
            }
            None => derived_id.clone(),
        };
        let title = compiled
            .page_meta
            .title
            .clone()
            .or_else(|| {
                compiled
                    .headings
                    .first()
                    .map(|heading| heading.text.clone())
            })
            .unwrap_or_else(|| page_id.clone());
        let summary = compiled.page_meta.summary.clone().unwrap_or_default();
        let description = compiled
            .page_meta
            .description
            .clone()
            .filter(|d| !d.is_empty())
            .or_else(|| {
                if !summary.is_empty() {
                    Some(summary.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let published = compiled.page_meta.published.clone().unwrap_or_default();
        let updated = compiled.page_meta.updated.clone().unwrap_or_default();
        let authors = compiled.page_meta.authors.clone();
        let tags = compiled.page_meta.tags.clone();
        let collection = compiled.page_meta.collection.clone().unwrap_or_default();
        let route_hint = match compiled.page_meta.route {
            Some(route) => {
                if page_info.mount_prefix.is_empty()
                    || route.starts_with(&format!("/{}/", page_info.mount_prefix))
                    || route == format!("/{}", page_info.mount_prefix)
                {
                    RouteHint::Explicit(route)
                } else {
                    RouteHint::Explicit(format!("/{}{route}", page_info.mount_prefix))
                }
            }
            None => RouteHint::Derived,
        };
        let page_docs = docs::load_page_docs(
            SourceFile::new(&name, &src),
            &compiled.document,
            &relative_name,
            IncludeOptions {
                root: &root,
                snippet_roots: &snippet_roots,
            },
            &mut diagnostics,
        );
        let headings = if page_docs.article.is_empty() {
            compiled
                .headings
                .iter()
                .map(|heading| PageHeading {
                    level: heading.level,
                    id: heading.id.clone(),
                    text: heading.text.clone(),
                })
                .collect()
        } else {
            let mut headings = compiled
                .headings
                .iter()
                .map(|heading| PageHeading {
                    level: heading.level,
                    id: heading.id.clone(),
                    text: heading.text.clone(),
                })
                .collect::<Vec<_>>();
            let nested = docs::collect_headings(&page_docs.article);
            for heading in nested {
                if !headings
                    .iter()
                    .any(|existing| existing.id == heading.id && existing.level == heading.level)
                {
                    headings.push(heading);
                }
            }
            headings
        };
        let mut outgoing_links: Vec<String> =
            compiled.links.iter().map(|link| link.url.clone()).collect();
        outgoing_links.extend(docs::collect_links(&page_docs.article));
        outgoing_links.sort();
        outgoing_links.dedup();
        let mut image_urls = collect_image_urls(&compiled.document);
        image_urls.extend(docs::collect_images(&page_docs.article));
        image_urls.sort();
        image_urls.dedup();
        let article_html = if page_docs.article.is_empty() {
            render_document(&compiled.document)
        } else {
            docs::render_article(&page_docs.article)
        };
        sources.push(SourcePage {
            id: page_id,
            id_explicit,
            source_path: relative_name,
            route_hint,
            aliases: compiled.page_meta.aliases.clone(),
            draft: compiled.page_meta.draft,
            layout,
            published,
            updated,
            authors,
            tags,
            collection,
            title,
            description,
            headings,
            outgoing_links,
            image_urls,
            article_html,
            kind: class.kind,
            docs: page_docs,
        });
    }

    Ok(LoadedSite {
        root,
        config,
        sources,
        files,
        static_files: Vec::new(),
        diagnostics,
    })
}

pub fn resolve_loaded(loaded: &LoadedSite) -> catalog::ResolveResult {
    let mut result = catalog::resolve(
        &loaded.sources,
        &ResolveOptions {
            navigation: loaded.config.navigation.clone(),
            files: loaded.files.clone(),
        },
    );
    let mut diagnostics = loaded.diagnostics.clone();
    diagnostics.append(&mut result.diagnostics);
    docs::validate_resolved(&result.site.pages, &mut diagnostics);
    result.diagnostics = diagnostics;
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckFormat {
    Terminal,
    Json,
}

pub struct CheckReport {
    pub diagnostics: Vec<CatalogDiagnostic>,
}

impl CheckReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(CatalogDiagnostic::is_error)
    }

    pub fn render(&self, format: CheckFormat) -> Result<String> {
        match format {
            CheckFormat::Terminal => Ok(catalog::format_diagnostics(&self.diagnostics)),
            CheckFormat::Json => Ok(serde_json::to_string_pretty(&self.diagnostics)?),
        }
    }
}

pub fn check(root: &Path) -> Result<CheckReport> {
    let loaded = load_site(root)?;
    let result = resolve_loaded(&loaded);
    Ok(CheckReport {
        diagnostics: result.diagnostics,
    })
}

pub fn test_examples(root: &Path, update: bool) -> Result<CheckReport> {
    let loaded = load_site(root)?;
    let result = resolve_loaded(&loaded);
    let mut diagnostics = result.diagnostics;
    if diagnostics.iter().any(CatalogDiagnostic::is_error) {
        return Ok(CheckReport { diagnostics });
    }
    let examples: Vec<_> = result
        .site
        .pages
        .iter()
        .flat_map(|page| page.examples.iter().cloned())
        .collect();
    diagnostics.extend(docs::run_examples(
        &examples,
        &docs::ExampleTestOptions {
            root: loaded.root.clone(),
            timeout: std::time::Duration::from_millis(loaded.config.examples.timeout_ms),
            allow_network: loaded.config.examples.allow_network,
            update,
        },
    ));
    Ok(CheckReport { diagnostics })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectKind {
    Config,
    Catalog,
    Page,
    Graph,
    Nav,
    Artifacts,
}

pub fn inspect(root: &Path, kind: InspectKind, target: Option<&str>) -> Result<String> {
    let loaded = load_site(root)?;
    match kind {
        InspectKind::Config => Ok(serde_json::to_string_pretty(&loaded.config)?),
        InspectKind::Artifacts => {
            let result = resolve_loaded(&loaded);
            let planned = crate::plan::plan(&loaded.root, &loaded.config, &result.site)?;
            Ok(serde_json::to_string_pretty(&planned.artifacts())?)
        }
        _ => {
            let result = resolve_loaded(&loaded);
            inspect_resolved(&result.site, kind, target)
        }
    }
}

fn inspect_resolved(
    site: &ResolvedSite,
    kind: InspectKind,
    target: Option<&str>,
) -> Result<String> {
    match kind {
        InspectKind::Config => unreachable!(),
        InspectKind::Catalog => {
            let pages: Vec<_> = site.pages.iter().map(CatalogInspect::from).collect();
            Ok(serde_json::to_string_pretty(&pages)?)
        }
        InspectKind::Page => {
            let Some(target) = target.filter(|value| !value.is_empty()) else {
                bail!("inspect page requires a page id or route");
            };
            let page = site.pages.iter().find(|page| {
                page.id == target
                    || page.route == target
                    || page.route == catalog::with_trailing_slash(target)
                    || page.aliases.iter().any(|alias| alias == target)
            });
            let Some(page) = page else {
                bail!("unknown page `{target}`");
            };
            let outgoing: Vec<_> = site
                .graph
                .iter()
                .filter(|edge| edge.from_id == page.id)
                .cloned()
                .collect();
            Ok(serde_json::to_string_pretty(&PageInspect {
                page,
                outgoing,
                docs_kinds: &page.docs_kinds,
                includes: &page.includes,
                examples: page
                    .examples
                    .iter()
                    .map(|example| example.id.as_str())
                    .collect(),
            })?)
        }
        InspectKind::Graph => Ok(serde_json::to_string_pretty(&site.graph)?),
        InspectKind::Nav => Ok(serde_json::to_string_pretty(&NavInspect {
            navigation: &site.navigation,
            pages: site
                .pages
                .iter()
                .map(|page| JourneyInspect {
                    id: &page.id,
                    breadcrumbs: &page.breadcrumbs,
                    previous: page.previous.as_ref(),
                    next: page.next.as_ref(),
                })
                .collect(),
        })?),
        InspectKind::Artifacts => unreachable!("artifacts are planned before inspect_resolved"),
    }
}

#[derive(Serialize)]
struct CatalogInspect<'a> {
    id: &'a str,
    route: &'a str,
    aliases: &'a [String],
    title: &'a str,
    kind: PageKind,
    draft: bool,
    unlisted: bool,
}

impl<'a> From<&'a ResolvedPage> for CatalogInspect<'a> {
    fn from(page: &'a ResolvedPage) -> Self {
        Self {
            id: &page.id,
            route: &page.route,
            aliases: &page.aliases,
            title: &page.title,
            kind: page.kind,
            draft: page.draft,
            unlisted: page.unlisted,
        }
    }
}

#[derive(Serialize)]
struct PageInspect<'a> {
    #[serde(flatten)]
    page: &'a ResolvedPage,
    outgoing: Vec<Edge>,
    docs_kinds: &'a [String],
    includes: &'a [docs::IncludeOrigin],
    examples: Vec<&'a str>,
}

#[derive(Serialize)]
struct NavInspect<'a> {
    navigation: &'a [NavSection],
    pages: Vec<JourneyInspect<'a>>,
}

#[derive(Serialize)]
struct JourneyInspect<'a> {
    id: &'a str,
    breadcrumbs: &'a [catalog::NavLink],
    previous: Option<&'a catalog::NavLink>,
    next: Option<&'a catalog::NavLink>,
}

fn collect_files(root: &Path, config: &SiteConfig) -> Result<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    collect_files_in(root, root, "", &mut files)?;
    for mount in &config.mounts {
        let mount_dir = root.join(&mount.source);
        if mount_dir.is_dir() {
            collect_files_in(&mount_dir, &mount_dir, &mount.prefix, &mut files)?;
        }
    }
    Ok(files)
}

fn collect_files_in(
    root: &Path,
    dir: &Path,
    prefix: &str,
    files: &mut BTreeSet<String>,
) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        if path.is_dir() {
            collect_files_in(root, &path, prefix, files)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let full_rel = if prefix.is_empty() {
                relative
            } else {
                format!("{prefix}/{relative}")
            };
            files.insert(full_rel);
        }
    }
    Ok(())
}

fn snippet_roots(root: &Path, config: &SiteConfig) -> Result<Vec<PathBuf>> {
    let mut roots = Vec::new();
    for entry in &config.snippets.roots {
        let path = root.join(entry);
        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            let canonical = std::fs::canonicalize(&path).unwrap_or(path);
            let ceiling_buf = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
            let ceiling = ceiling_buf.parent().unwrap_or(&ceiling_buf);
            if !canonical.starts_with(ceiling) {
                bail!("snippet root `{entry}` escapes the repository");
            }
            roots.push(canonical);
        } else {
            roots.push(path);
        }
    }
    for mount in &config.mounts {
        let mount_dir = root.join(&mount.source);
        let mount_snippets = mount_dir.join("snippets");
        if mount_snippets.is_dir() && !roots.contains(&mount_snippets) {
            roots.push(mount_snippets);
        }
    }
    Ok(roots)
}

fn page_kind_diagnostic(path: &str, class: PageClass) -> Option<CatalogDiagnostic> {
    match class.kind {
        PageKind::Static => None,
        PageKind::Hydrate => Some(CatalogDiagnostic::error(
            "RD2301",
            path,
            format!(
                "{path} is a hydrate page ({}); site builds cannot splice Rocci components yet",
                class.reason
            ),
        )),
        PageKind::Live => Some(CatalogDiagnostic::error(
            "RD2302",
            path,
            format!(
                "{path} is a live page ({}); site builds cannot include handlers or Datastar yet",
                class.reason
            ),
        )),
    }
}

fn collect_image_urls(document: &Document) -> Vec<String> {
    let mut urls = Vec::new();
    for item in &document.items {
        if let Item::Markdown(node) = item {
            walk_images(node, &mut urls);
        }
    }
    urls
}

fn walk_images(node: &MdNode, urls: &mut Vec<String>) {
    if let MdNode::Image { url, .. } = node {
        urls.push(url.clone());
    }
    match node {
        MdNode::Heading { children, .. }
        | MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. }
        | MdNode::Link { children, .. } => {
            for child in children {
                walk_images(child, urls);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    fn temp(name: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("rocdown-site-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn check_accepts_relative_links_and_aliases() {
        let root = temp("rel");
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\nSee the [guide](./guide.rocdown), [explicit](/guide/), [alias](/old-guide/), and [heading](./guide.rocdown#install).\n",
        )
        .unwrap();
        fs::write(
            root.join("guide.rocdown"),
            "@page {\n    aliases: [\"/old-guide/\"],\n    meta: { title: \"Guide\" },\n}\n\n# Guide\n\n## Install\n",
        )
        .unwrap();
        let report = check(&root).unwrap();
        assert!(
            !report.has_errors(),
            "{}",
            report.render(CheckFormat::Terminal).unwrap()
        );
        let resolved = resolve_loaded(&load_site(&root).unwrap());
        let home = resolved
            .site
            .pages
            .iter()
            .find(|page| page.id == "index")
            .unwrap();
        assert!(
            home.article_html.contains("href=\"/guide/\""),
            "{}",
            home.article_html
        );
        assert!(
            home.article_html.contains("href=\"/guide/#install\""),
            "{}",
            home.article_html
        );
        assert!(
            !home.article_html.contains("guide.rocdown"),
            "{}",
            home.article_html
        );
        let catalog = inspect(&root, InspectKind::Catalog, None).unwrap();
        assert!(catalog.contains("\"id\": \"guide\""));
        let artifacts = inspect(&root, InspectKind::Artifacts, None).unwrap();
        assert!(artifacts.contains("old-guide/index.html"));
        assert!(artifacts.contains("404.html"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_accepts_rocdown_file_as_root() {
        let root = temp("file-root");
        fs::write(root.join("report.rocdown"), "# Report\n\nHello.\n").unwrap();
        let report = check(&root.join("report.rocdown")).unwrap();
        assert!(
            !report.has_errors(),
            "{}",
            report.render(CheckFormat::Terminal).unwrap()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_hundred_page_fixture() {
        let root = temp("hundred");
        for index in 0..100 {
            let id = format!("p{index:03}");
            let mut body = format!("# {id}\n");
            if index + 1 < 100 {
                body.push_str(&format!("\n[next](/p{:03}/)\n", index + 1));
            }
            fs::write(root.join(format!("{id}.rocdown")), body).unwrap();
        }
        let report = check(&root).unwrap();
        assert!(
            !report.has_errors(),
            "{}",
            report.render(CheckFormat::Terminal).unwrap()
        );
        fs::write(root.join("p050.rocdown"), "# p050\n\n[bad](/nope/)\n").unwrap();
        let report = check(&root).unwrap();
        assert!(report.has_errors());
        let rendered = report.render(CheckFormat::Terminal).unwrap();
        assert!(rendered.contains("p050.rocdown"), "{rendered}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_reports_unknown_docs_kind() {
        let root = temp("docs-kind");
        fs::write(root.join("index.rocdown"), "# Home\n\n:widget Hi\n").unwrap();
        let report = check(&root).unwrap();
        assert!(report.has_errors());
        let rendered = report.render(CheckFormat::Terminal).unwrap();
        assert!(
            rendered.contains("unknown article kind `:widget`"),
            "{rendered}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_rejects_use_for_static_sites() {
        let root = temp("use-static");
        fs::write(
            root.join("Callout.rocci"),
            include_str!("../../../test/Callout.rocci"),
        )
        .unwrap();
        fs::write(
            root.join("index.rocdown"),
            "# Home\n\n@use \"./Callout.rocci\"\n\n:callout[tone: \"warn\"] Be careful.\n",
        )
        .unwrap();
        let report = check(&root).unwrap();
        assert!(report.has_errors());
        let rendered = report.render(CheckFormat::Terminal).unwrap();
        assert!(
            rendered.contains("custom static blocks belong in the compiled theme"),
            "{rendered}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn find_site_root_walks_to_rocdown_toml() {
        let root = temp("site-root");
        fs::create_dir_all(root.join("guides")).unwrap();
        fs::write(
            root.join(crate::config::CONFIG_FILE),
            "[site]\ntitle = \"Demo\"\n",
        )
        .unwrap();
        fs::write(root.join("guides/page.rocdown"), "# Page\n").unwrap();
        let found = find_site_root(&root.join("guides/page.rocdown")).unwrap();
        assert_eq!(
            fs::canonicalize(&found).unwrap(),
            fs::canonicalize(&root).unwrap()
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn check_diagnoses_hydrate_pages() {
        let root = temp("hydrate");
        fs::write(root.join("index.rocdown"), "# Home\n").unwrap();
        fs::write(
            root.join("widget.rocdown"),
            "# Widget\n\n@render {\n    Html.text(\"x\")\n}\n",
        )
        .unwrap();
        let report = check(&root).unwrap();
        assert!(report.has_errors());
        let rendered = report.render(CheckFormat::Terminal).unwrap();
        assert!(rendered.contains("RD2301"), "{rendered}");
        assert!(rendered.contains("hydrate"), "{rendered}");
        assert!(rendered.contains("@render"), "{rendered}");
        assert!(!rendered.contains("RD2302"), "{rendered}");
        let catalog = inspect(&root, InspectKind::Catalog, None).unwrap();
        assert!(catalog.contains("\"kind\": \"hydrate\""), "{catalog}");
        let resolved = resolve_loaded(&load_site(&root).unwrap());
        let widget = resolved
            .site
            .pages
            .iter()
            .find(|page| page.id == "widget")
            .unwrap();
        assert_eq!(widget.kind, PageKind::Hydrate);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn find_site_root_is_none_without_toml() {
        let root = temp("no-toml");
        fs::write(root.join("Guide.rocdown"), "# Guide\n").unwrap();
        assert_eq!(find_site_root(&root.join("Guide.rocdown")), None);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn site_preview_route_uses_derived_path() {
        let root = temp("preview-derived");
        fs::create_dir_all(root.join("guides")).unwrap();
        fs::write(
            root.join("guides/docs-components.rocdown"),
            "@page {\n    aliases: [\"/guides/docs-components/\"],\n}\n\n# Components\n",
        )
        .unwrap();
        assert_eq!(
            site_preview_route(&root, &root.join("guides/docs-components.rocdown")),
            "/guides/docs-components/"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn site_preview_route_honors_explicit_route() {
        let root = temp("preview-explicit");
        fs::write(
            root.join("page.rocdown"),
            "@page {\n    route: \"/custom-page/\",\n}\n\n# Custom\n",
        )
        .unwrap();
        assert_eq!(
            site_preview_route(&root, &root.join("page.rocdown")),
            "/custom-page/"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn docs_guide_previews_at_derived_site_route() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let file = manifest.join("../../docs/guides/docs-components.rocdown");
        let expected = manifest.join("../../docs");
        let found = find_site_root(&file).expect("docs/rocdown.toml");
        assert_eq!(
            fs::canonicalize(&found).unwrap(),
            fs::canonicalize(&expected).unwrap()
        );
        assert_eq!(
            site_preview_route(&found, &file),
            "/guides/docs-components/"
        );
    }

    #[test]
    fn check_diagnoses_live_pages() {
        let root = temp("live");
        fs::write(root.join("index.rocdown"), "# Home\n").unwrap();
        fs::write(
            root.join("counter.rocdown"),
            "# Counter\n\n@on:post(\"/inc\") = |_| {\n    Html.text(\"x\")\n}\n",
        )
        .unwrap();
        let report = check(&root).unwrap();
        assert!(report.has_errors());
        let rendered = report.render(CheckFormat::Terminal).unwrap();
        assert!(rendered.contains("RD2302"), "{rendered}");
        assert!(rendered.contains("live"), "{rendered}");
        assert!(rendered.contains("@on"), "{rendered}");
        let catalog = inspect(&root, InspectKind::Catalog, None).unwrap();
        assert!(catalog.contains("\"kind\": \"live\""), "{catalog}");
        let resolved = resolve_loaded(&load_site(&root).unwrap());
        let counter = resolved
            .site
            .pages
            .iter()
            .find(|page| page.id == "counter")
            .unwrap();
        assert_eq!(counter.kind, PageKind::Live);
        let home = resolved
            .site
            .pages
            .iter()
            .find(|page| page.id == "index")
            .unwrap();
        assert_eq!(home.kind, PageKind::Static);
        let _ = fs::remove_dir_all(root);
    }
}
