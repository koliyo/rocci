use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use rocci_template::{Cursor, SourceFile, Span, is_ident_start};
use serde::Serialize;

use crate::article::{render_md, render_static_image};
use crate::ast::{DocsDecl, Document, Item, MdNode};
use crate::catalog::{CatalogDiagnostic, Edge, EdgeKind, PageHeading, ResolvedPage, Severity};
use crate::img::{StaticImage, extract_img_fields};
use crate::page::{bool_literal, skip_value, string_list, string_literal};
use crate::parse::parse_fragment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsField {
    pub name: String,
    pub name_span: Span,
    pub value: Span,
}

pub fn split_docs_body(src: &str, body: Span) -> (Vec<DocsField>, Span) {
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;
    let mut fields = Vec::new();
    loop {
        cur.skip_trivia();
        if cur.pos >= end {
            return (fields, Span::new(end, end));
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let checkpoint = cur.pos;
        let Some(name_span) = cur.scan_ident() else {
            return (fields, remainder(src, checkpoint, end));
        };
        if !name_span.of(src).chars().next().is_some_and(is_ident_start) {
            return (fields, remainder(src, checkpoint, end));
        }
        cur.skip_trivia();
        if !cur.eat(':') {
            return (fields, remainder(src, checkpoint, end));
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        skip_value(&mut cur, end);
        if cur.pos == value_start {
            return (fields, remainder(src, checkpoint, end));
        }
        let name = name_span.of(src).to_string();
        fields.push(DocsField {
            name,
            name_span,
            value: Span::new(value_start, cur.pos.min(end)),
        });
        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }
}

fn remainder(src: &str, start: usize, end: usize) -> Span {
    rocci_template::trim_span(src, Span::new(start, end))
}

pub fn field_string(src: &str, field: &DocsField) -> Option<String> {
    string_literal(src, field.value)
}

pub fn field_bool(src: &str, field: &DocsField) -> Option<bool> {
    bool_literal(src, field.value)
}

pub fn field_strings(src: &str, field: &DocsField) -> Option<Vec<String>> {
    string_list(src, field.value)
}

pub fn include_path_error(path: &str) -> Option<String> {
    if path.is_empty() {
        return Some("include path must not be empty".into());
    }
    if path.contains('\0') {
        return Some("include path must not contain NUL bytes".into());
    }
    if Path::new(path).is_absolute() {
        return Some("include path must be relative".into());
    }
    if path.contains("..") {
        return Some("include path must not contain `..`".into());
    }
    None
}

pub fn extract_region(text: &str, name: &str) -> Result<(String, usize, usize), String> {
    let start_marker = format!("docs-region: {name}");
    let end_marker = format!("docs-region-end: {name}");
    let mut start_line = None;
    let mut end_line = None;
    for (index, line) in text.lines().enumerate() {
        if line.contains(&start_marker) {
            if start_line.is_some() {
                return Err(format!("duplicate region `{name}`"));
            }
            start_line = Some(index);
        }
        if line.contains(&end_marker) {
            if end_line.is_some() {
                return Err(format!("duplicate region end `{name}`"));
            }
            end_line = Some(index);
        }
    }
    let Some(start) = start_line else {
        return Err(format!("missing region `{name}`"));
    };
    let Some(end) = end_line else {
        return Err(format!("unclosed region `{name}`"));
    };
    if end <= start {
        return Err(format!("region `{name}` ends before it starts"));
    }
    let excerpt = text
        .lines()
        .skip(start + 1)
        .take(end.saturating_sub(start + 1))
        .collect::<Vec<_>>()
        .join("\n");
    Ok((excerpt, start + 2, end))
}

pub fn extract_lines(text: &str, start: u32, end: u32) -> Result<(String, usize, usize), String> {
    if start == 0 || end == 0 || end < start {
        return Err("line range must be 1-based with end >= start".into());
    }
    let lines: Vec<&str> = text.lines().collect();
    let from = start as usize;
    let to = end as usize;
    if to > lines.len() {
        return Err(format!(
            "line range {start}-{end} is past the end of the file"
        ));
    }
    Ok((lines[from - 1..to].join("\n"), from, to))
}

pub fn resolve_include_path(from_file: &str, path: &str) -> Result<PathBuf, String> {
    if let Some(err) = include_path_error(path) {
        return Err(err);
    }
    let from_file = from_file.strip_prefix("file://").unwrap_or(from_file);
    let base = Path::new(from_file)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let joined = base.join(path);
    let mut out = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::ParentDir => {
                return Err("include path must not contain `..`".into());
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    Ok(out)
}

const ASIDES: &[&str] = &["note", "tip", "caution", "danger", "deprecated"];
const TAB_KINDS: &[&str] = &["language", "platform", "tool"];
const BADGE_TONES: &[&str] = &["stable", "beta", "preview", "deprecated", "removed"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageDocs {
    pub article: Vec<ArticleNode>,
    pub examples: Vec<ExampleRecord>,
    pub includes: Vec<IncludeOrigin>,
    pub snippet_paths: Vec<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleNode {
    Markdown(MdNode),
    Docs(DocsNode),
    Image(StaticImage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsNode {
    pub kind: String,
    pub attrs: DocsAttrs,
    pub children: Vec<ArticleNode>,
    pub origin: Option<IncludeOrigin>,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocsAttrs {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub label: Option<String>,
    pub term: Option<String>,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub credit: Option<String>,
    pub tone: Option<String>,
    pub page: Option<String>,
    pub href: Option<String>,
    pub group: Option<String>,
    pub tab_kind: Option<String>,
    pub id: Option<String>,
    pub path: Option<String>,
    pub region: Option<String>,
    pub language: Option<String>,
    pub start: Option<u32>,
    pub end: Option<u32>,
    pub test: Vec<String>,
    pub expect: Option<String>,
    pub open: bool,
    pub verify: bool,
    pub allow_network: bool,
    pub unknown: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct IncludeOrigin {
    pub source_path: String,
    pub region: Option<String>,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct ExampleRecord {
    pub id: String,
    pub language: String,
    pub path: Option<String>,
    pub region: Option<String>,
    pub test: Vec<String>,
    pub expect: Option<String>,
    pub allow_network: bool,
    pub origin: IncludeOrigin,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedSegment {
    pub tag: String,
    pub kind: String,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub label: String,
    pub href: String,
    pub tone: String,
    pub group: String,
    pub tab_kind: String,
    pub tab_id: String,
    pub origin: String,
    pub caption: String,
    pub credit: String,
    pub alt: String,
    pub language: String,
    pub open: bool,
    pub verify: bool,
    pub children: Vec<PlannedSegment>,
}

#[derive(Debug, Clone)]
pub struct IncludeOptions<'a> {
    pub root: &'a Path,
    pub snippet_roots: &'a [PathBuf],
}

struct BuildCtx<'a> {
    source: SourceFile<'a>,
    source_path: &'a str,
    includes: IncludeOptions<'a>,
    stack: Vec<PathBuf>,
    diagnostics: &'a mut Vec<CatalogDiagnostic>,
    examples: Vec<ExampleRecord>,
    origins: Vec<IncludeOrigin>,
    snippet_paths: BTreeSet<String>,
}

pub fn load_page_docs(
    source: SourceFile<'_>,
    document: &Document,
    source_path: &str,
    includes: IncludeOptions<'_>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) -> PageDocs {
    let mut ctx = BuildCtx {
        source,
        source_path,
        includes,
        stack: Vec::new(),
        diagnostics,
        examples: Vec::new(),
        origins: Vec::new(),
        snippet_paths: BTreeSet::new(),
    };
    let article = nodes_from_items(&mut ctx, &document.items, None);
    PageDocs {
        article,
        examples: ctx.examples,
        includes: ctx.origins,
        snippet_paths: ctx.snippet_paths.into_iter().collect(),
    }
}

pub fn collect_kinds(nodes: &[ArticleNode]) -> Vec<String> {
    let mut kinds = Vec::new();
    walk_kinds(nodes, &mut kinds);
    kinds
}

fn walk_kinds(nodes: &[ArticleNode], kinds: &mut Vec<String>) {
    for node in nodes {
        if let ArticleNode::Docs(docs) = node {
            kinds.push(docs.kind.clone());
            walk_kinds(&docs.children, kinds);
        }
    }
}

pub fn collect_headings(nodes: &[ArticleNode]) -> Vec<PageHeading> {
    let mut headings = Vec::new();
    collect_headings_in(nodes, false, false, &mut headings);
    headings
}

fn collect_headings_in(
    nodes: &[ArticleNode],
    in_tab: bool,
    in_tree: bool,
    headings: &mut Vec<PageHeading>,
) {
    if in_tree {
        return;
    }
    for node in nodes {
        match node {
            ArticleNode::Markdown(MdNode::Heading {
                level,
                id,
                children,
                ..
            }) if !in_tab => headings.push(PageHeading {
                level: *level,
                id: id.clone(),
                text: children.iter().map(MdNode::text_content).collect(),
            }),
            ArticleNode::Markdown(md) if !in_tab => {
                for child in md.children_mut_slice() {
                    collect_headings_in(
                        &[ArticleNode::Markdown(child.clone())],
                        in_tab,
                        in_tree,
                        headings,
                    );
                }
            }
            ArticleNode::Docs(docs) => {
                let tab = in_tab || docs.kind == "tab";
                let tree = in_tree || docs.kind == "file-tree";
                collect_headings_in(&docs.children, tab, tree, headings);
            }
            _ => {}
        }
    }
}

trait MdChildren {
    fn children_mut_slice(&self) -> &[MdNode];
}

impl MdChildren for MdNode {
    fn children_mut_slice(&self) -> &[MdNode] {
        match self {
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
            | MdNode::Link { children, .. } => children,
            _ => &[],
        }
    }
}

pub fn collect_links(nodes: &[ArticleNode]) -> Vec<String> {
    let mut urls = Vec::new();
    walk_links(nodes, &mut urls, false);
    urls
}

pub fn collect_images(nodes: &[ArticleNode]) -> Vec<String> {
    let mut urls = Vec::new();
    walk_links(nodes, &mut urls, true);
    urls
}

fn walk_links(nodes: &[ArticleNode], urls: &mut Vec<String>, images: bool) {
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => walk_md_links(md, urls, images),
            ArticleNode::Image(image) if images => urls.push(image.src.clone()),
            ArticleNode::Image(_) => {}
            ArticleNode::Docs(docs) => {
                if let Some(href) = &docs.attrs.href
                    && !images
                {
                    urls.push(href.clone());
                }
                walk_links(&docs.children, urls, images);
            }
        }
    }
}

fn walk_md_links(node: &MdNode, urls: &mut Vec<String>, images: bool) {
    match node {
        MdNode::Link { url, .. } if !images => urls.push(url.clone()),
        MdNode::Image { url, .. } if images => urls.push(url.clone()),
        _ => {}
    }
    for child in node.children_mut_slice() {
        walk_md_links(child, urls, images);
    }
}

pub fn fill_link_cards(pages: &mut [ResolvedPage]) {
    let mut lookup: BTreeMap<String, (String, String, String)> = BTreeMap::new();
    for page in pages.iter() {
        let val = (
            page.route.clone(),
            page.title.clone(),
            page.description.clone(),
        );
        lookup.insert(page.id.clone(), val.clone());
        lookup.insert(page.route.clone(), val.clone());
        for alias in &page.aliases {
            lookup.insert(alias.clone(), val.clone());
        }
        if let Some(stripped) = page.id.strip_prefix("docs/") {
            lookup.insert(stripped.to_string(), val.clone());
            lookup.insert(format!("/{stripped}/"), val.clone());
        }
    }
    for page in pages {
        fill_cards_in(&mut page.article, &lookup);
    }
}

pub fn rewrite_resolved_links(pages: &mut [ResolvedPage], graph: &[Edge]) {
    let routes: BTreeMap<String, String> = pages
        .iter()
        .map(|page| (page.id.clone(), page.route.clone()))
        .collect();
    let mut maps: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for edge in graph {
        if edge.raw.starts_with('#') {
            continue;
        }
        let href = match edge.kind {
            EdgeKind::Page => routes.get(&edge.target).cloned(),
            EdgeKind::Heading => edge.target.split_once('#').and_then(|(id, fragment)| {
                routes.get(id).map(|route| format!("{route}#{fragment}"))
            }),
            EdgeKind::Asset | EdgeKind::External => None,
        };
        if let Some(href) = href {
            maps.entry(edge.from_id.clone())
                .or_default()
                .insert(edge.raw.clone(), href);
        }
    }
    for page in pages {
        let Some(map) = maps.get(&page.id) else {
            continue;
        };
        rewrite_nodes(&mut page.article, map);
        if !page.article.is_empty() {
            page.article_html = render_article(&page.article);
        }
    }
}

fn rewrite_nodes(nodes: &mut [ArticleNode], map: &BTreeMap<String, String>) {
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => rewrite_md(md, map),
            ArticleNode::Docs(docs) => {
                if let Some(href) = &docs.attrs.href
                    && let Some(rewritten) = map.get(href)
                {
                    docs.attrs.href = Some(rewritten.clone());
                }
                rewrite_nodes(&mut docs.children, map);
            }
            ArticleNode::Image(_) => {}
        }
    }
}

fn rewrite_md(node: &mut MdNode, map: &BTreeMap<String, String>) {
    if let MdNode::Link { url, .. } = node
        && let Some(rewritten) = map.get(url.as_str())
    {
        *url = rewritten.clone();
    }
    for child in node.children_mut() {
        rewrite_md(child, map);
    }
}

fn fill_cards_in(nodes: &mut [ArticleNode], lookup: &BTreeMap<String, (String, String, String)>) {
    for node in nodes {
        if let ArticleNode::Docs(docs) = node {
            if docs.kind == "link-card"
                && let Some(id) = &docs.attrs.page
                && let Some((route, title, description)) = lookup.get(id)
            {
                if docs.attrs.href.is_none() {
                    docs.attrs.href = Some(route.clone());
                }
                if docs.attrs.title.is_none() {
                    docs.attrs.title = Some(title.clone());
                }
                if docs.attrs.summary.is_none() && !description.is_empty() {
                    docs.attrs.summary = Some(description.clone());
                }
            }
            fill_cards_in(&mut docs.children, lookup);
        }
    }
}

pub fn validate_resolved(pages: &[ResolvedPage], diagnostics: &mut Vec<CatalogDiagnostic>) {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for page in pages {
        ids.insert(page.id.clone());
        ids.insert(page.route.clone());
        for alias in &page.aliases {
            ids.insert(alias.clone());
        }
        if let Some(stripped) = page.id.strip_prefix("docs/") {
            ids.insert(stripped.to_string());
            ids.insert(format!("/{stripped}/"));
        }
    }
    for page in pages {
        validate_link_cards(&page.article, &page.source_path, &ids, diagnostics);
    }
}

fn validate_link_cards(
    nodes: &[ArticleNode],
    path: &str,
    ids: &BTreeSet<String>,
    diagnostics: &mut Vec<CatalogDiagnostic>,
) {
    for node in nodes {
        if let ArticleNode::Docs(docs) = node {
            if docs.kind == "link-card"
                && let Some(page) = &docs.attrs.page
                && !ids.contains(page.as_str())
                && !ids.contains(page.strip_prefix("docs/").unwrap_or(page))
                && !ids.contains(&format!("docs/{page}"))
                && !ids.contains(page.strip_prefix('/').unwrap_or(page))
            {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2101",
                    path,
                    format!(
                        "line {}: `@docs link-card` targets unknown page `{page}`",
                        docs.line
                    ),
                ));
            }
            validate_link_cards(&docs.children, path, ids, diagnostics);
        }
    }
}

fn nodes_from_items(
    ctx: &mut BuildCtx<'_>,
    items: &[Item],
    parent_kind: Option<&str>,
) -> Vec<ArticleNode> {
    let mut nodes = Vec::new();
    for item in items {
        match item {
            Item::Markdown(node) => nodes.push(ArticleNode::Markdown(node.clone())),
            Item::Docs(decl) => {
                if let Some(node) = docs_node(ctx, decl, parent_kind) {
                    nodes.push(ArticleNode::Docs(node));
                }
            }
            Item::Img(img) => {
                let mut diags = Vec::new();
                let fields = extract_img_fields(ctx.source.src, img.body, &mut diags);
                for diagnostic in diags {
                    ctx.diagnostics.push(CatalogDiagnostic {
                        code: if diagnostic.is_error() {
                            "RD1001"
                        } else {
                            "RD1002"
                        },
                        severity: if diagnostic.is_error() {
                            Severity::Error
                        } else {
                            Severity::Warning
                        },
                        path: ctx.source_path.to_string(),
                        message: diagnostic.message,
                    });
                }
                nodes.push(ArticleNode::Image(StaticImage::from_fields(
                    &fields, img.span,
                )));
            }
            Item::Page(_) if parent_kind.is_some() => illegal(ctx, item, "page"),
            Item::Page(_) => {}
            Item::Roc(_) => illegal(ctx, item, "roc"),
            Item::Render(_) => illegal(ctx, item, "render"),
            Item::Component(_) => illegal(ctx, item, "component"),
            Item::Fixture(_) => illegal(ctx, item, "fixture"),
            Item::Css(_) => illegal(ctx, item, "css"),
            Item::Context(_) => illegal(ctx, item, "context"),
            Item::Init(_) => illegal(ctx, item, "init"),
            Item::On(_) => illegal(ctx, item, "on"),
            Item::Template(_) => illegal(ctx, item, "template"),
        }
    }
    nodes
}

fn illegal(ctx: &mut BuildCtx<'_>, item: &Item, kind: &str) {
    ctx.diagnostics.push(CatalogDiagnostic::error(
        "RD2403",
        ctx.source_path,
        format!(
            "line {}: `@{kind}` is not allowed inside `@docs`",
            line_number(ctx.source.src, item.span().start as usize)
        ),
    ));
}

fn docs_node(
    ctx: &mut BuildCtx<'_>,
    decl: &DocsDecl,
    parent_kind: Option<&str>,
) -> Option<DocsNode> {
    let line = line_number(ctx.source.src, decl.span.start as usize);
    let (fields, content) = split_docs_body(ctx.source.src, decl.body);
    let attrs = parse_attrs(ctx.source.src, &fields);
    if decl.kind == "api-operation" {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2406",
            ctx.source_path,
            format!("line {line}: `@docs api-operation` is reserved for generated API reference (Phase 6)"),
        ));
        return None;
    }
    if !known_kind(&decl.kind) {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2401",
            ctx.source_path,
            format!("line {line}: unknown `@docs` kind `{}`", decl.kind),
        ));
        return None;
    }
    if decl.kind == "include" {
        return include_node(ctx, decl, attrs, line);
    }
    let parsed = parse_fragment(ctx.source, content, false);
    for diagnostic in parsed.diagnostics {
        let code = if diagnostic.is_error() {
            "RD1001"
        } else {
            "RD1002"
        };
        ctx.diagnostics.push(CatalogDiagnostic {
            code,
            severity: if diagnostic.is_error() {
                Severity::Error
            } else {
                Severity::Warning
            },
            path: ctx.source_path.to_string(),
            message: format!("line {line}: {}", diagnostic.message),
        });
    }
    let children = nodes_from_items(ctx, &parsed.document.items, Some(&decl.kind));
    let node = DocsNode {
        kind: decl.kind.clone(),
        attrs,
        children,
        origin: None,
        line,
    };
    validate_model(ctx, &node, parent_kind);
    if node.kind == "example" {
        push_example(ctx, &node);
    }
    Some(node)
}

fn known_kind(kind: &str) -> bool {
    ASIDES.contains(&kind)
        || matches!(
            kind,
            "details"
                | "steps"
                | "step"
                | "figure"
                | "definition"
                | "badge"
                | "compatibility"
                | "card-grid"
                | "link-card"
                | "file-tree"
                | "tabs"
                | "tab"
                | "example"
                | "include"
                | "playground"
        )
}

fn parse_attrs(src: &str, fields: &[DocsField]) -> DocsAttrs {
    let mut attrs = DocsAttrs::default();
    for field in fields {
        match field.name.as_str() {
            "title" => attrs.title = field_string(src, field),
            "summary" => attrs.summary = field_string(src, field),
            "label" => attrs.label = field_string(src, field),
            "term" => attrs.term = field_string(src, field),
            "caption" => attrs.caption = field_string(src, field),
            "credit" => attrs.credit = field_string(src, field),
            "tone" => attrs.tone = field_string(src, field),
            "page" => attrs.page = field_string(src, field),
            "href" => attrs.href = field_string(src, field),
            "group" => attrs.group = field_string(src, field),
            "kind" => attrs.tab_kind = field_string(src, field),
            "id" => attrs.id = field_string(src, field),
            "path" => attrs.path = field_string(src, field),
            "region" => attrs.region = field_string(src, field),
            "language" => attrs.language = field_string(src, field),
            "expect" => attrs.expect = field_string(src, field),
            "start" => attrs.start = field.value.of(src).trim().parse().ok(),
            "end" => attrs.end = field.value.of(src).trim().parse().ok(),
            "open" => attrs.open = field_bool(src, field).unwrap_or(false),
            "verify" => attrs.verify = field_bool(src, field).unwrap_or(false),
            "allow_network" => attrs.allow_network = field_bool(src, field).unwrap_or(false),
            "test" => {
                if let Some(items) = field_strings(src, field) {
                    attrs.test = items;
                } else if let Some(value) = field_string(src, field) {
                    attrs.test = split_argv(&value);
                } else {
                    attrs.unknown.push(field.name.clone());
                }
            }
            other => attrs.unknown.push(other.to_string()),
        }
    }
    attrs
}

fn split_argv(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}

fn validate_model(ctx: &mut BuildCtx<'_>, node: &DocsNode, parent_kind: Option<&str>) {
    let path = ctx.source_path;
    let line = node.line;
    if !node.attrs.unknown.is_empty() {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2402",
            path,
            format!(
                "line {line}: unknown `@docs {}` field `{}`",
                node.kind,
                node.attrs.unknown.join(", ")
            ),
        ));
    }
    match node.kind.as_str() {
        kind if ASIDES.contains(&kind) => {
            if node
                .children
                .iter()
                .any(|child| matches!(child, ArticleNode::Docs(docs) if docs.kind == "tabs"))
            {
                malformed(ctx, line, "asides cannot contain tabs");
            }
        }
        "details" => {
            if node.attrs.summary.as_deref().unwrap_or("").is_empty() {
                malformed(ctx, line, "`@docs details` requires `summary`");
            }
        }
        "steps" => {
            let has_step = node
                .children
                .iter()
                .any(|child| matches!(child, ArticleNode::Docs(docs) if docs.kind == "step"));
            let has_list = node.children.iter().any(|child| {
                matches!(
                    child,
                    ArticleNode::Markdown(MdNode::List { ordered: true, .. })
                )
            });
            let extra = node.children.iter().any(|child| match child {
                ArticleNode::Docs(docs) => docs.kind != "step",
                ArticleNode::Markdown(MdNode::List { ordered: true, .. }) => false,
                ArticleNode::Markdown(_) | ArticleNode::Image(_) => true,
            });
            if has_step && has_list {
                malformed(
                    ctx,
                    line,
                    "`@docs steps` cannot mix a list with `@docs step`",
                );
            } else if extra {
                malformed(
                    ctx,
                    line,
                    "`@docs steps` body must be an ordered list or `@docs step` children",
                );
            } else if !has_step && !has_list {
                malformed(ctx, line, "`@docs steps` requires steps");
            }
        }
        "step" => {
            if parent_kind != Some("steps") {
                malformed(ctx, line, "`@docs step` is only valid inside `@docs steps`");
            }
        }
        "figure" => {
            let images = count_images(&node.children);
            if images != 1 {
                malformed(
                    ctx,
                    line,
                    "`@docs figure` body must contain exactly one image",
                );
            }
        }
        "definition" => {
            if node.attrs.term.as_deref().unwrap_or("").is_empty() {
                malformed(ctx, line, "`@docs definition` requires `term`");
            }
        }
        "badge" => {
            if node.attrs.label.as_deref().unwrap_or("").is_empty() {
                malformed(ctx, line, "`@docs badge` requires `label`");
            }
            if let Some(tone) = &node.attrs.tone
                && !BADGE_TONES.contains(&tone.as_str())
            {
                malformed(ctx, line, &format!("invalid badge tone `{tone}`"));
            }
        }
        "compatibility" => {
            if !contains_table(&node.children) {
                malformed(ctx, line, "`@docs compatibility` body must be a table");
            }
        }
        "link-card" => {
            if node.attrs.page.is_none() && node.attrs.href.is_none() {
                malformed(ctx, line, "`@docs link-card` requires `page` or `href`");
            }
        }
        "card-grid" => {
            if !node
                .children
                .iter()
                .any(|child| matches!(child, ArticleNode::Docs(docs) if docs.kind == "link-card"))
            {
                malformed(
                    ctx,
                    line,
                    "`@docs card-grid` requires `@docs link-card` children",
                );
            }
        }
        "file-tree" => {
            if !node.children.iter().any(|child| {
                matches!(
                    child,
                    ArticleNode::Markdown(MdNode::List { ordered: false, .. })
                )
            }) {
                malformed(
                    ctx,
                    line,
                    "`@docs file-tree` body must be an unordered list",
                );
            }
        }
        "tabs" => {
            if node.attrs.group.as_deref().unwrap_or("").is_empty() {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2405",
                    path,
                    format!("line {line}: `@docs tabs` requires `group`"),
                ));
            }
            let Some(kind) = node.attrs.tab_kind.as_deref() else {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2405",
                    path,
                    format!("line {line}: `@docs tabs` requires `kind`"),
                ));
                return;
            };
            if !TAB_KINDS.contains(&kind) {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2405",
                    path,
                    format!("line {line}: `@docs tabs` kind must be language, platform, or tool"),
                ));
            }
            let tabs: Vec<_> = node
                .children
                .iter()
                .filter_map(|child| match child {
                    ArticleNode::Docs(docs) if docs.kind == "tab" => Some(docs),
                    _ => None,
                })
                .collect();
            if tabs.is_empty() {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2405",
                    path,
                    format!("line {line}: `@docs tabs` requires `@docs tab` children"),
                ));
            }
            let mut seen = BTreeSet::new();
            for tab in tabs {
                let id = tab.attrs.id.as_deref().unwrap_or("");
                if id.is_empty() {
                    ctx.diagnostics.push(CatalogDiagnostic::error(
                        "RD2405",
                        path,
                        format!("line {}: `@docs tab` requires `id`", tab.line),
                    ));
                } else if !seen.insert(id) {
                    ctx.diagnostics.push(CatalogDiagnostic::error(
                        "RD2405",
                        path,
                        format!("line {}: duplicate tab id `{id}`", tab.line),
                    ));
                }
                if tab.attrs.label.as_deref().unwrap_or("").is_empty() {
                    ctx.diagnostics.push(CatalogDiagnostic::error(
                        "RD2405",
                        path,
                        format!("line {}: `@docs tab` requires `label`", tab.line),
                    ));
                }
                if tab.children.is_empty() {
                    ctx.diagnostics.push(CatalogDiagnostic::error(
                        "RD2405",
                        path,
                        format!("line {}: `@docs tab` cannot be empty", tab.line),
                    ));
                }
            }
        }
        "tab" => {
            if parent_kind != Some("tabs") {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2405",
                    path,
                    format!("line {line}: `@docs tab` is only valid inside `@docs tabs`"),
                ));
            }
        }
        "example" => validate_example(ctx, node),
        _ => {}
    }
}

fn validate_example(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let has_path = !node.attrs.path.as_deref().unwrap_or("").is_empty();
    let has_code = contains_code(&node.children);
    if has_path && has_code && !only_caption(&node.children) {
        malformed(
            ctx,
            node.line,
            "`@docs example` cannot combine `path` with a code body unless the body is a caption",
        );
    }
    if !node.attrs.test.is_empty() {
        if argv_unsafe(&node.attrs.test) {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!(
                    "line {}: example `test` must be a simple argument list without shell metacharacters",
                    node.line
                ),
            ));
        }
        if node.attrs.language.as_deref().unwrap_or("").is_empty() && !has_path {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!(
                    "line {}: example with `test` requires `language` or `path`",
                    node.line
                ),
            ));
        }
        if node.attrs.expect.is_none() {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2602",
                ctx.source_path,
                format!("line {}: example with `test` requires `expect`", node.line),
            ));
        }
    } else {
        ctx.diagnostics.push(CatalogDiagnostic::warning(
            "RD2601",
            ctx.source_path,
            format!("line {}: untested `@docs example`", node.line),
        ));
    }
}

fn argv_unsafe(argv: &[String]) -> bool {
    argv.iter().any(|part| {
        part.chars()
            .any(|ch| matches!(ch, '|' | '&' | ';' | '$' | '`' | '\n' | '>' | '<'))
    })
}

fn malformed(ctx: &mut BuildCtx<'_>, line: u32, message: &str) {
    ctx.diagnostics.push(CatalogDiagnostic::error(
        "RD2402",
        ctx.source_path,
        format!("line {line}: {message}"),
    ));
}

fn count_images(nodes: &[ArticleNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            ArticleNode::Markdown(MdNode::Paragraph { children, .. }) => children
                .iter()
                .filter(|child| matches!(child, MdNode::Image { .. }))
                .count(),
            ArticleNode::Markdown(MdNode::Image { .. }) | ArticleNode::Image(_) => 1,
            ArticleNode::Docs(docs) => count_images(&docs.children),
            _ => 0,
        })
        .sum()
}

fn contains_table(nodes: &[ArticleNode]) -> bool {
    nodes.iter().any(|node| match node {
        ArticleNode::Markdown(MdNode::Table { .. }) => true,
        ArticleNode::Docs(docs) => contains_table(&docs.children),
        _ => false,
    })
}

fn contains_code(nodes: &[ArticleNode]) -> bool {
    nodes.iter().any(|node| match node {
        ArticleNode::Markdown(MdNode::CodeBlock { .. }) => true,
        ArticleNode::Docs(docs) => contains_code(&docs.children),
        _ => false,
    })
}

fn only_caption(nodes: &[ArticleNode]) -> bool {
    nodes.iter().all(|node| match node {
        ArticleNode::Markdown(MdNode::Paragraph { .. }) => true,
        ArticleNode::Markdown(MdNode::CodeBlock { .. }) => false,
        ArticleNode::Docs(_) => false,
        _ => true,
    })
}

fn include_node(
    ctx: &mut BuildCtx<'_>,
    decl: &DocsDecl,
    attrs: DocsAttrs,
    line: u32,
) -> Option<DocsNode> {
    let Some(path) = attrs.path.clone() else {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2501",
            ctx.source_path,
            format!("line {line}: `@docs include` requires `path`"),
        ));
        return None;
    };
    if attrs.start.is_some() || attrs.end.is_some() {
        ctx.diagnostics.push(CatalogDiagnostic::warning(
            "RD2504",
            ctx.source_path,
            format!("line {line}: include line ranges are fragile; prefer a named region"),
        ));
    }
    let resolved = match resolve_allowed_path(ctx, &path) {
        Ok(path) => path,
        Err(err) => {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2501",
                ctx.source_path,
                format!("line {line}: {err}"),
            ));
            return None;
        }
    };
    if ctx.stack.iter().any(|seen| seen == &resolved) {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2505",
            ctx.source_path,
            format!(
                "line {line}: cyclic include `{}`",
                display_rel(ctx, &resolved)
            ),
        ));
        return None;
    }
    let contents = match std::fs::read_to_string(&resolved) {
        Ok(contents) => contents,
        Err(_) => {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2501",
                ctx.source_path,
                format!(
                    "line {line}: missing include `{}`",
                    display_rel(ctx, &resolved)
                ),
            ));
            return None;
        }
    };
    let (excerpt, line_start, line_end) = if let Some(region) = attrs.region.as_deref() {
        match extract_region(&contents, region) {
            Ok((excerpt, start, end)) => (excerpt, Some(start as u32), Some(end as u32)),
            Err(err) => {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2502",
                    ctx.source_path,
                    format!("line {line}: {err}"),
                ));
                return None;
            }
        }
    } else if let (Some(start), Some(end)) = (attrs.start, attrs.end) {
        match extract_lines(&contents, start, end) {
            Ok((excerpt, from, to)) => (excerpt, Some(from as u32), Some(to as u32)),
            Err(err) => {
                ctx.diagnostics.push(CatalogDiagnostic::error(
                    "RD2501",
                    ctx.source_path,
                    format!("line {line}: {err}"),
                ));
                return None;
            }
        }
    } else {
        (contents, None, None)
    };
    let origin = IncludeOrigin {
        source_path: authored_origin_path(ctx, &path, &resolved),
        region: attrs.region.clone(),
        line_start,
        line_end,
    };
    ctx.origins.push(origin.clone());
    ctx.snippet_paths.insert(origin.source_path.clone());
    if resolved.extension().and_then(|ext| ext.to_str()) == Some("rocdown") {
        ctx.stack.push(resolved.clone());
        let included = crate::parse(SourceFile::new(&origin.source_path, &excerpt), false);
        if included
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.is_error())
        {
            ctx.diagnostics.push(CatalogDiagnostic::error(
                "RD2503",
                ctx.source_path,
                format!(
                    "line {line}: included Rocdown `{}` has parse errors",
                    origin.source_path
                ),
            ));
        }
        let children = nodes_from_items(ctx, &included.document.items, Some("include"));
        ctx.stack.pop();
        return Some(DocsNode {
            kind: "include".into(),
            attrs,
            children,
            origin: Some(origin),
            line,
        });
    }
    let language = attrs
        .language
        .clone()
        .or_else(|| {
            resolved
                .extension()
                .and_then(|ext| ext.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    let code = ArticleNode::Markdown(MdNode::CodeBlock {
        info: language,
        literal: excerpt,
        span: decl.body,
    });
    Some(DocsNode {
        kind: "include".into(),
        attrs,
        children: vec![code],
        origin: Some(origin),
        line,
    })
}

fn resolve_allowed_path(ctx: &BuildCtx<'_>, path: &str) -> Result<PathBuf, String> {
    let from = ctx.includes.root.join(ctx.source_path);
    let relative = resolve_include_path(&from.to_string_lossy(), path)?;
    let mut candidates = vec![
        ctx.includes.root.join(path),
        ctx.includes.root.join(&relative),
    ];
    if let Some(parent) = Path::new(ctx.source_path).parent() {
        candidates.push(ctx.includes.root.join(parent).join(path));
    }
    for root in ctx.includes.snippet_roots {
        candidates.push(root.join(path));
    }
    let mut allowed = vec![ctx.includes.root.to_path_buf()];
    allowed.extend(ctx.includes.snippet_roots.iter().cloned());
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        let canonical = std::fs::canonicalize(&candidate)
            .map_err(|_| format!("include path `{}` is not readable", path))?;
        if !allowed.iter().any(|root| {
            std::fs::canonicalize(root)
                .ok()
                .is_some_and(|root| canonical.starts_with(root))
        }) {
            return Err(format!(
                "include path `{path}` escapes allowed snippet roots"
            ));
        }
        return Ok(canonical);
    }
    Err(format!("missing include `{path}`"))
}

fn authored_origin_path(ctx: &BuildCtx<'_>, authored: &str, resolved: &Path) -> String {
    if include_path_error(authored).is_none() && !authored.is_empty() {
        authored.replace('\\', "/")
    } else {
        display_rel(ctx, resolved)
    }
}

fn display_rel(ctx: &BuildCtx<'_>, path: &Path) -> String {
    let root = std::fs::canonicalize(ctx.includes.root)
        .unwrap_or_else(|_| ctx.includes.root.to_path_buf());
    if let Ok(rel) = path.strip_prefix(&root) {
        return rel.to_string_lossy().replace('\\', "/");
    }
    if let Ok(canonical) = std::fs::canonicalize(path)
        && let Ok(rel) = canonical.strip_prefix(&root)
    {
        return rel.to_string_lossy().replace('\\', "/");
    }
    for snippet_root in ctx.includes.snippet_roots {
        let snippet_root =
            std::fs::canonicalize(snippet_root).unwrap_or_else(|_| snippet_root.clone());
        if let Ok(rel) = path.strip_prefix(&snippet_root) {
            return rel.to_string_lossy().replace('\\', "/");
        }
    }
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("include")
        .to_string()
}

fn push_example(ctx: &mut BuildCtx<'_>, node: &DocsNode) {
    let origin = node.origin.clone().unwrap_or(IncludeOrigin {
        source_path: node
            .attrs
            .path
            .clone()
            .unwrap_or_else(|| ctx.source_path.to_string()),
        region: node.attrs.region.clone(),
        line_start: node.attrs.start,
        line_end: node.attrs.end,
    });
    if node.attrs.path.is_some() {
        ctx.snippet_paths.insert(origin.source_path.clone());
    }
    ctx.examples.push(ExampleRecord {
        id: node.attrs.id.clone().unwrap_or_default(),
        language: node.attrs.language.clone().unwrap_or_default(),
        path: node.attrs.path.clone(),
        region: node.attrs.region.clone(),
        test: node.attrs.test.clone(),
        expect: node.attrs.expect.clone(),
        allow_network: node.attrs.allow_network,
        origin,
        line: node.line,
    });
}

pub fn render_article(nodes: &[ArticleNode]) -> String {
    let mut parts = Vec::new();
    let mut footnotes = Vec::new();
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) if matches!(md, MdNode::FootnoteDefinition { .. }) => {
                footnotes.push(render_md(md));
            }
            other => parts.push(render_node(other)),
        }
    }
    if !footnotes.is_empty() {
        parts.push(crate::article::render_footnote_section(&footnotes));
    }
    parts.concat()
}

fn render_node(node: &ArticleNode) -> String {
    match node {
        ArticleNode::Markdown(md) => render_md(md),
        ArticleNode::Docs(docs) => render_docs(docs),
        ArticleNode::Image(image) => render_static_image(image),
    }
}

fn render_docs(docs: &DocsNode) -> String {
    docs.children.iter().map(render_node).collect::<String>()
}

fn aside_label(kind: &str) -> &'static str {
    match kind {
        "note" => "Note",
        "tip" => "Tip",
        "caution" => "Caution",
        "danger" => "Danger",
        "deprecated" => "Deprecated",
        _ => "Note",
    }
}

pub fn markdown_fragment(nodes: &[ArticleNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => {
                out.push_str(&md_to_markdown(md));
                out.push('\n');
            }
            ArticleNode::Docs(docs) => out.push_str(&docs_to_markdown(docs)),
            ArticleNode::Image(image) => {
                out.push_str(&format!("![{}]({})", image.alt, image.src));
                out.push('\n');
            }
        }
    }
    out
}

pub fn search_text(nodes: &[ArticleNode]) -> String {
    markdown_fragment(nodes)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn docs_to_markdown(docs: &DocsNode) -> String {
    let inner = markdown_fragment(&docs.children);
    match docs.kind.as_str() {
        kind if ASIDES.contains(&kind) => {
            let label = docs
                .attrs
                .title
                .clone()
                .unwrap_or_else(|| aside_label(kind).to_string());
            format!("> **{label}:** {}\n\n", inner.trim())
        }
        "details" => format!(
            "**{}**\n\n{}\n\n",
            docs.attrs.summary.as_deref().unwrap_or("Details"),
            inner.trim()
        ),
        "steps" => {
            let steps: Vec<_> = docs
                .children
                .iter()
                .filter_map(|child| match child {
                    ArticleNode::Docs(step) if step.kind == "step" => Some(step),
                    _ => None,
                })
                .collect();
            if steps.is_empty() {
                format!("{}\n\n", inner.trim())
            } else {
                steps
                    .into_iter()
                    .enumerate()
                    .map(|(index, step)| {
                        let title = step.attrs.title.as_deref().unwrap_or("Step");
                        let marker = if step.attrs.verify { "Verify: " } else { "" };
                        format!(
                            "{}. {marker}{title}\n\n{}\n",
                            index + 1,
                            markdown_fragment(&step.children).trim()
                        )
                    })
                    .collect()
            }
        }
        "figure" => {
            let caption = docs.attrs.caption.as_deref().unwrap_or("");
            let credit = docs
                .attrs
                .credit
                .as_deref()
                .map(|credit| format!("\n*{credit}*"))
                .unwrap_or_default();
            format!("{}\n\n{caption}{credit}\n\n", inner.trim())
        }
        "definition" => format!(
            "**{}:** {}\n\n",
            docs.attrs.term.as_deref().unwrap_or("Term"),
            inner.trim()
        ),
        "badge" => format!("**{}**\n\n", docs.attrs.label.as_deref().unwrap_or("")),
        "compatibility" => format!("{}\n\n", inner.trim()),
        "link-card" => {
            let title = docs.attrs.title.as_deref().unwrap_or("Link");
            let href = docs.attrs.href.as_deref().unwrap_or("#");
            format!("[{title}]({href})\n\n")
        }
        "file-tree" => format!("{}\n\n", inner.trim()),
        "tabs" => docs
            .children
            .iter()
            .filter_map(|child| match child {
                ArticleNode::Docs(tab) if tab.kind == "tab" => Some(format!(
                    "### {}\n\n{}\n",
                    tab.attrs.label.as_deref().unwrap_or(""),
                    markdown_fragment(&tab.children).trim()
                )),
                _ => None,
            })
            .collect(),
        "include" | "example" => {
            let mut out = inner;
            if let Some(origin) = &docs.origin {
                let region = origin
                    .region
                    .as_deref()
                    .map(|region| format!(" (region {region})"))
                    .unwrap_or_default();
                out.push_str(&format!("\n*Source: {}{region}*\n", origin.source_path));
            } else if let Some(path) = &docs.attrs.path {
                out.push_str(&format!("\n*Source: {path}*\n"));
            }
            out
        }
        _ => inner,
    }
}

fn md_to_markdown(node: &MdNode) -> String {
    match node {
        MdNode::Heading {
            level, children, ..
        } => {
            format!(
                "{} {}",
                "#".repeat(*level as usize),
                children
                    .iter()
                    .map(MdNode::text_content)
                    .collect::<String>()
            )
        }
        MdNode::Paragraph { children, .. } => children.iter().map(MdNode::text_content).collect(),
        MdNode::BlockQuote { children, .. } => format!(
            "> {}",
            children
                .iter()
                .map(md_to_markdown)
                .collect::<Vec<_>>()
                .join("\n> ")
        ),
        MdNode::List {
            ordered, children, ..
        } => children
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let marker = if *ordered {
                    format!("{}. ", index + 1)
                } else {
                    "- ".into()
                };
                format!("{marker}{}", md_to_markdown(child))
            })
            .collect::<Vec<_>>()
            .join("\n"),
        MdNode::Item { children, .. } | MdNode::TaskItem { children, .. } => {
            children.iter().map(md_to_markdown).collect()
        }
        MdNode::CodeBlock { info, literal, .. } => format!("```{info}\n{literal}\n```"),
        MdNode::Table { .. } => node.text_content(),
        _ => node.text_content(),
    }
}

pub fn plan_segments(
    article_name: &str,
    nodes: &[ArticleNode],
    rewrite: &BTreeMap<String, String>,
) -> (Vec<PlannedSegment>, Vec<(String, String)>) {
    let mut files = Vec::new();
    let mut counter = 0u32;
    let segments = plan_nodes(article_name, nodes, rewrite, &mut files, &mut counter);
    (segments, files)
}

fn plan_nodes(
    article_name: &str,
    nodes: &[ArticleNode],
    rewrite: &BTreeMap<String, String>,
    files: &mut Vec<(String, String)>,
    counter: &mut u32,
) -> Vec<PlannedSegment> {
    let mut segments = Vec::new();
    let mut markdown = Vec::new();
    let flush = |markdown: &mut Vec<MdNode>,
                 files: &mut Vec<(String, String)>,
                 counter: &mut u32,
                 segments: &mut Vec<PlannedSegment>| {
        if markdown.is_empty() {
            return;
        }
        let nodes: Vec<ArticleNode> = markdown
            .iter()
            .cloned()
            .map(ArticleNode::Markdown)
            .collect();
        let html = rewrite_urls(&render_article(&nodes), rewrite);
        let path = format!("articles/{article_name}.{counter}.html");
        *counter += 1;
        files.push((path.clone(), html));
        markdown.clear();
        segments.push(html_segment(path));
    };
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => markdown.push(md.clone()),
            ArticleNode::Image(image) => {
                flush(&mut markdown, files, counter, &mut segments);
                let html = rewrite_urls(&render_static_image(image), rewrite);
                let path = format!("articles/{article_name}.{counter}.html");
                *counter += 1;
                files.push((path.clone(), html));
                segments.push(html_segment(path));
            }
            ArticleNode::Docs(docs) => {
                flush(&mut markdown, files, counter, &mut segments);
                segments.push(docs_segment(article_name, docs, rewrite, files, counter));
            }
        }
    }
    flush(&mut markdown, files, counter, &mut segments);
    segments
}

fn html_segment(path: String) -> PlannedSegment {
    PlannedSegment {
        tag: "html".into(),
        path,
        ..empty_segment()
    }
}

fn empty_segment() -> PlannedSegment {
    PlannedSegment {
        tag: String::new(),
        kind: String::new(),
        path: String::new(),
        title: String::new(),
        summary: String::new(),
        label: String::new(),
        href: String::new(),
        tone: String::new(),
        group: String::new(),
        tab_kind: String::new(),
        tab_id: String::new(),
        origin: String::new(),
        caption: String::new(),
        credit: String::new(),
        alt: String::new(),
        language: String::new(),
        open: false,
        verify: false,
        children: Vec::new(),
    }
}

fn docs_segment(
    article_name: &str,
    docs: &DocsNode,
    rewrite: &BTreeMap<String, String>,
    files: &mut Vec<(String, String)>,
    counter: &mut u32,
) -> PlannedSegment {
    let href = docs.attrs.href.clone().unwrap_or_else(|| {
        docs.attrs
            .page
            .as_deref()
            .map(|page| format!("/{}/", page.trim_matches('/')))
            .unwrap_or_default()
    });
    PlannedSegment {
        tag: "docs".into(),
        kind: docs.kind.clone(),
        path: String::new(),
        title: docs
            .attrs
            .title
            .clone()
            .or_else(|| docs.attrs.term.clone())
            .unwrap_or_default(),
        summary: docs.attrs.summary.clone().unwrap_or_default(),
        label: docs.attrs.label.clone().unwrap_or_default(),
        href: rewrite_urls(&href, rewrite),
        tone: docs.attrs.tone.clone().unwrap_or_default(),
        group: docs.attrs.group.clone().unwrap_or_default(),
        tab_kind: docs.attrs.tab_kind.clone().unwrap_or_default(),
        tab_id: docs.attrs.id.clone().unwrap_or_default(),
        origin: docs
            .origin
            .as_ref()
            .map(|origin| origin.source_path.clone())
            .unwrap_or_default(),
        caption: docs.attrs.caption.clone().unwrap_or_default(),
        credit: docs.attrs.credit.clone().unwrap_or_default(),
        alt: docs.attrs.alt.clone().unwrap_or_default(),
        language: docs.attrs.language.clone().unwrap_or_default(),
        open: docs.attrs.open,
        verify: docs.attrs.verify,
        children: plan_nodes(article_name, &docs.children, rewrite, files, counter),
    }
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

fn line_number(src: &str, offset: usize) -> u32 {
    src.get(..offset.min(src.len()))
        .unwrap_or("")
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
}

#[derive(Debug, Clone)]
pub struct ExampleTestOptions {
    pub root: PathBuf,
    pub timeout: Duration,
    pub allow_network: bool,
    pub update: bool,
}

pub fn run_examples(
    examples: &[ExampleRecord],
    options: &ExampleTestOptions,
) -> Vec<CatalogDiagnostic> {
    let mut diagnostics = Vec::new();
    for example in examples {
        if example.test.is_empty() {
            continue;
        }
        if example.allow_network && !options.allow_network {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2603",
                &example.origin.source_path,
                format!(
                    "line {}: example requires network but examples.allow_network is false",
                    example.line
                ),
            ));
            continue;
        }
        let cwd = example
            .path
            .as_deref()
            .and_then(|path| options.root.join(path).parent().map(Path::to_path_buf))
            .unwrap_or_else(|| options.root.clone());
        let Some((program, args)) = example.test.split_first() else {
            continue;
        };
        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(&cwd)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let output = match run_timed(&mut command, options.timeout) {
            Ok(output) => output,
            Err(CommandError::Timeout) => {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!("line {}: example command timed out", example.line),
                ));
                continue;
            }
            Err(CommandError::Io(err)) => {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!("line {}: failed to run example: {err}", example.line),
                ));
                continue;
            }
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        if let Some(expect) = &example.expect {
            let expect_path = options.root.join(expect);
            if expect_path.is_file() {
                if options.update {
                    let _ = std::fs::write(&expect_path, stdout.as_bytes());
                } else {
                    let golden = std::fs::read_to_string(&expect_path).unwrap_or_default();
                    if stdout != golden {
                        diagnostics.push(CatalogDiagnostic::error(
                            "RD2603",
                            &example.origin.source_path,
                            format!(
                                "line {}: example output did not match golden file `{expect}`",
                                example.line
                            ),
                        ));
                    }
                }
            } else if !combined.contains(expect) {
                diagnostics.push(CatalogDiagnostic::error(
                    "RD2603",
                    &example.origin.source_path,
                    format!(
                        "line {}: example output did not contain `{expect}`",
                        example.line
                    ),
                ));
            }
        }
        if !output.status.success() {
            diagnostics.push(CatalogDiagnostic::error(
                "RD2603",
                &example.origin.source_path,
                format!("line {}: example command failed", example.line),
            ));
        }
    }
    diagnostics
}

struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

enum CommandError {
    Timeout,
    Io(std::io::Error),
}

fn run_timed(command: &mut Command, timeout: Duration) -> Result<CommandOutput, CommandError> {
    let mut child = command.spawn().map_err(CommandError::Io)?;
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let stdout_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stdout_pipe.take() {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    });
    let stderr_thread = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stderr_pipe.take() {
            let _ = pipe.read_to_end(&mut bytes);
        }
        bytes
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CommandError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(err) => return Err(CommandError::Io(err)),
        }
    };
    Ok(CommandOutput {
        status,
        stdout: stdout_thread.join().unwrap_or_default(),
        stderr: stderr_thread.join().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompileOptions, compile};

    fn load(src: &str) -> (PageDocs, Vec<CatalogDiagnostic>) {
        let compiled = compile(
            SourceFile::new("guide.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        assert!(!compiled.has_errors(), "{:?}", compiled.diagnostics);
        let mut diagnostics = Vec::new();
        let docs = load_page_docs(
            SourceFile::new("guide.rocdown", src),
            &compiled.document,
            "guide.rocdown",
            IncludeOptions {
                root: Path::new("."),
                snippet_roots: &[],
            },
            &mut diagnostics,
        );
        (docs, diagnostics)
    }

    #[test]
    fn note_projects_markdown_and_search() {
        let (docs, diagnostics) = load(
            "# Guide\n\n@docs note {\n    title: \"Watch\"\n\n    See the [next](/next/).\n}\n",
        );
        assert!(
            !diagnostics.iter().any(CatalogDiagnostic::is_error),
            "{diagnostics:?}"
        );
        assert!(collect_links(&docs.article).contains(&"/next/".to_string()));
        let markdown = markdown_fragment(&docs.article);
        assert!(markdown.contains("**Watch:**"), "{markdown}");
        assert!(search_text(&docs.article).contains("next"));
    }

    #[test]
    fn unknown_kind_is_rd2401() {
        let (_docs, diagnostics) = load("@docs widget {\n    Hi\n}\n");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "RD2401")
        );
    }

    #[test]
    fn api_operation_is_rd2406() {
        let (_docs, diagnostics) = load("@docs api-operation {\n    id: \"get\"\n}\n");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "RD2406")
        );
    }

    #[test]
    fn untested_example_warns() {
        let (_docs, diagnostics) =
            load("@docs example {\n    language: \"sh\"\n\n    ```sh\n    echo hi\n    ```\n}\n");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "RD2601")
        );
        assert!(!diagnostics.iter().any(CatalogDiagnostic::is_error));
    }

    #[test]
    fn tabs_project_all_panels_and_omit_outline_headings() {
        let (docs, diagnostics) = load(
            "# Guide\n\n@docs tabs {\n    group: \"os\"\n    kind: \"platform\"\n\n    @docs tab {\n        id: \"mac\"\n        label: \"macOS\"\n\n        ## Inside\n\n        Mac panel.\n    }\n    @docs tab {\n        id: \"linux\"\n        label: \"Linux\"\n\n        Linux panel.\n    }\n}\n",
        );
        assert!(
            !diagnostics.iter().any(CatalogDiagnostic::is_error),
            "{diagnostics:?}"
        );
        let markdown = markdown_fragment(&docs.article);
        assert!(markdown.contains("### macOS"), "{markdown}");
        assert!(markdown.contains("Linux panel"), "{markdown}");
        assert!(
            !collect_headings(&docs.article)
                .iter()
                .any(|heading| heading.text == "Inside")
        );
    }

    #[test]
    fn figure_with_markdown_image_does_not_require_figure_alt() {
        let (_docs, diagnostics) = load("@docs figure {\n    ![x](/x.png)\n}\n");
        assert!(
            !diagnostics.iter().any(CatalogDiagnostic::is_error),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn figure_level_alt_is_unknown() {
        let (_docs, diagnostics) =
            load("@docs figure {\n    alt: \"Diagram\"\n\n    ![x](/x.png)\n}\n");
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "RD2402" && diagnostic.message.contains("unknown")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn include_reads_region_and_warns_on_line_range() {
        let root =
            std::env::temp_dir().join(format!("rocdown-docs-include-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("snippet.rs"),
            "// docs-region: install\nfn install() {}\n// docs-region-end: install\nfn other() {}\n",
        )
        .unwrap();
        let src = "@docs include {\n    path: \"snippet.rs\"\n    region: \"install\"\n}\n";
        let compiled = compile(
            SourceFile::new("guide.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        let mut diagnostics = Vec::new();
        let docs = load_page_docs(
            SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), src),
            &compiled.document,
            "guide.rocdown",
            IncludeOptions {
                root: &root,
                snippet_roots: &[],
            },
            &mut diagnostics,
        );
        assert!(
            !diagnostics.iter().any(CatalogDiagnostic::is_error),
            "{diagnostics:?}"
        );
        let markdown = markdown_fragment(&docs.article);
        assert!(markdown.contains("fn install()"), "{markdown}");
        assert!(!markdown.contains("fn other()"), "{markdown}");
        assert!(markdown.contains("Source: snippet.rs"), "{markdown}");

        let ranged = "@docs include {\n    path: \"snippet.rs\"\n    start: 1\n    end: 2\n}\n";
        let compiled = compile(
            SourceFile::new("guide.rocdown", ranged),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        let mut diagnostics = Vec::new();
        load_page_docs(
            SourceFile::new(root.join("guide.rocdown").to_str().unwrap(), ranged),
            &compiled.document,
            "guide.rocdown",
            IncludeOptions {
                root: &root,
                snippet_roots: &[],
            },
            &mut diagnostics,
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "RD2504")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn cyclic_include_is_rd2505() {
        let root = std::env::temp_dir().join(format!("rocdown-docs-cycle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("a.rocdown"),
            "@docs include {\n    path: \"b.rocdown\"\n}\n",
        )
        .unwrap();
        std::fs::write(
            root.join("b.rocdown"),
            "@docs include {\n    path: \"a.rocdown\"\n}\n",
        )
        .unwrap();
        let src = std::fs::read_to_string(root.join("a.rocdown")).unwrap();
        let compiled = compile(
            SourceFile::new("a.rocdown", &src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        let mut diagnostics = Vec::new();
        load_page_docs(
            SourceFile::new(root.join("a.rocdown").to_str().unwrap(), &src),
            &compiled.document,
            "a.rocdown",
            IncludeOptions {
                root: &root,
                snippet_roots: &[],
            },
            &mut diagnostics,
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "RD2505")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn echo_example_matches_expect() {
        let examples = [ExampleRecord {
            id: "echo".into(),
            language: "sh".into(),
            test: vec!["/bin/echo".into(), "hello".into()],
            expect: Some("hello".into()),
            origin: IncludeOrigin {
                source_path: "guide.rocdown".into(),
                ..IncludeOrigin::default()
            },
            line: 1,
            ..ExampleRecord::default()
        }];
        let diagnostics = run_examples(
            &examples,
            &ExampleTestOptions {
                root: PathBuf::from("."),
                timeout: Duration::from_secs(5),
                allow_network: false,
                update: false,
            },
        );
        assert!(
            !diagnostics.iter().any(CatalogDiagnostic::is_error),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn body_only_docs_edit_keeps_segment_paths() {
        let first = load("# Guide\n\n@docs note {\n    title: \"Watch\"\n\n    First.\n}\n").0;
        let second =
            load("# Guide\n\n@docs note {\n    title: \"Watch\"\n\n    Second paragraph.\n}\n").0;
        let rewrite = BTreeMap::new();
        let (first_segs, first_files) = plan_segments("Page", &first.article, &rewrite);
        let (second_segs, second_files) = plan_segments("Page", &second.article, &rewrite);
        let paths = |segs: &[PlannedSegment]| {
            segs.iter()
                .map(|seg| {
                    (
                        seg.tag.clone(),
                        seg.kind.clone(),
                        seg.path.clone(),
                        seg.title.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(paths(&first_segs), paths(&second_segs));
        assert_ne!(first_files, second_files);
    }

    #[test]
    fn footnotes_render_in_accessible_section() {
        let (docs, diags) = load("Claim.[^source]\n\n[^source]: Evidence.\n");
        assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
        let html = render_article(&docs.article);
        assert!(html.contains("data-footnote-ref"), "{html}");
        assert!(html.contains("aria-label=\"Footnotes\""), "{html}");
        assert!(html.contains("data-footnote-backref"), "{html}");
        assert!(html.contains("id=\"fn-source\""), "{html}");
        assert!(html.contains("id=\"fnref-source\""), "{html}");
        assert!(html.contains("Evidence."), "{html}");
    }

    #[test]
    fn img_decl_preserves_all_fields() {
        let (docs, diags) = load(
            "# Guide\n\n@img {\n    src: \"img/banner.png\"\n    alt: \"Banner\"\n    title: \"Hero\"\n    width: \"300px\"\n    height: \"120px\"\n    class: \"hero\"\n    loading: \"lazy\"\n    decoding: \"async\"\n}\n",
        );
        assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
        assert_eq!(collect_images(&docs.article), vec!["img/banner.png"]);
        let html = render_article(&docs.article);
        assert!(html.contains("src=\"img/banner.png\""), "{html}");
        assert!(html.contains("alt=\"Banner\""), "{html}");
        assert!(html.contains("title=\"Hero\""), "{html}");
        assert!(html.contains("width=\"300px\""), "{html}");
        assert!(html.contains("height=\"120px\""), "{html}");
        assert!(html.contains("class=\"rd-image hero\""), "{html}");
        assert!(html.contains("loading=\"lazy\""), "{html}");
        assert!(html.contains("decoding=\"async\""), "{html}");
    }

    #[test]
    fn figure_preserves_caption_credit_and_nested_img_fields() {
        let (docs, diags) = load(
            "@docs figure {\n    caption: \"Architecture\"\n    credit: \"Rocci docs\"\n\n    @img {\n        src: \"diagram.png\"\n        alt: \"Diagram\"\n        width: \"400px\"\n        loading: \"lazy\"\n        decoding: \"async\"\n    }\n}\n",
        );
        assert!(!diags.iter().any(CatalogDiagnostic::is_error), "{diags:?}");
        let (segments, _fragments) = plan_segments("fig", &docs.article, &Default::default());
        let figure_seg = segments
            .iter()
            .find(|s| s.kind == "figure")
            .expect("missing figure segment");
        assert_eq!(figure_seg.caption, "Architecture");
        assert_eq!(figure_seg.credit, "Rocci docs");
        let html = render_article(&docs.article);
        assert!(html.contains("src=\"diagram.png\""), "{html}");
        assert!(html.contains("alt=\"Diagram\""), "{html}");
        assert!(html.contains("width=\"400px\""), "{html}");
        assert!(html.contains("loading=\"lazy\""), "{html}");
        assert!(html.contains("decoding=\"async\""), "{html}");
    }
}
