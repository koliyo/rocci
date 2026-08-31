use std::collections::BTreeMap;

use rocci_template::{Span, TemplateItem};

use crate::ast::{BlockCall, Item, MdNode, ParamValue};
use crate::catalog::{CatalogDiagnostic, Edge, EdgeKind, PageHeading, ResolvedPage, Severity};
use crate::img::{StaticImage, img_fields_from_params};
use crate::parse::parse_fragment;
use crate::registry;

use super::examples::push_example;
use super::fields::{docs_fields_from_params, parse_attrs};
use super::includes::include_node;
use super::render::render_article;
use super::validate::validate_model;
use super::{ArticleNode, BuildCtx, DocsAttrs, DocsNode, line_number};

pub fn collect_kinds(nodes: &[ArticleNode]) -> Vec<String> {
    let mut kinds = Vec::new();
    walk_kinds(nodes, &mut kinds);
    kinds
}

fn walk_kinds(nodes: &[ArticleNode], kinds: &mut Vec<String>) {
    for node in nodes {
        if let ArticleNode::Block(docs) = node {
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

fn is_heading_sugar(call: &BlockCall, src: &str) -> bool {
    registry::heading_level(&call.name).is_some()
        && (call.is_colon(src) || atx_heading(src, call.span))
}

fn atx_heading(src: &str, span: Span) -> bool {
    src.get(span.start as usize..)
        .unwrap_or("")
        .trim_start_matches([' ', '\t'])
        .starts_with('#')
}

fn article_text(nodes: &[ArticleNode]) -> String {
    let mut parts = Vec::new();
    fn walk(nodes: &[ArticleNode], parts: &mut Vec<String>) {
        for node in nodes {
            match node {
                ArticleNode::Markdown(md) => {
                    let text = md.text_content();
                    if !text.is_empty() {
                        parts.push(text);
                    }
                }
                ArticleNode::Block(docs) => walk(&docs.children, parts),
                ArticleNode::Image(image) => {
                    if !image.alt.is_empty() {
                        parts.push(image.alt.clone());
                    }
                }
                ArticleNode::Island => {}
            }
        }
    }
    walk(nodes, &mut parts);
    parts.join(" ").trim().to_string()
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
            ArticleNode::Block(docs) if registry::heading_level(&docs.kind).is_some() => {
                if !in_tab {
                    let level = registry::heading_level(&docs.kind).unwrap();
                    headings.push(PageHeading {
                        level,
                        id: docs.attrs.id.clone().unwrap_or_default(),
                        text: article_text(&docs.children),
                    });
                }
            }
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
            ArticleNode::Block(docs) => {
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
            ArticleNode::Image(_) | ArticleNode::Island => {}
            ArticleNode::Block(docs) => {
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
            ArticleNode::Image(_) | ArticleNode::Island => {}
            ArticleNode::Block(docs) => {
                if let Some(href) = &docs.attrs.href
                    && let Some(rewritten) = map.get(href)
                {
                    docs.attrs.href = Some(rewritten.clone());
                }
                rewrite_nodes(&mut docs.children, map);
            }
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
        if let ArticleNode::Block(docs) = node {
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

pub(crate) fn nodes_from_items(
    ctx: &mut BuildCtx<'_>,
    items: &[Item],
    parent_kind: Option<&str>,
) -> Vec<ArticleNode> {
    let mut nodes = Vec::new();
    for item in items {
        match item {
            Item::Markdown(node) => nodes.push(ArticleNode::Markdown(node.clone())),
            Item::Block(call) if is_heading_sugar(call, ctx.source.src) => {
                nodes.push(heading_from_call(ctx, call, parent_kind));
            }
            Item::Block(call) if call.name == "img" => {
                nodes.push(image_from_call(ctx, call));
            }
            Item::Block(call) => {
                if let Some(node) = docs_node(ctx, call, parent_kind) {
                    nodes.push(ArticleNode::Block(node));
                }
            }
            Item::Page(_) if parent_kind.is_some() => illegal(ctx, item, "page"),
            Item::Page(_) => {}
            Item::Render(_) if parent_kind.is_none() => nodes.push(ArticleNode::Island),
            Item::Template(TemplateItem::Let(_)) if parent_kind.is_none() => {}
            Item::Template(_) if parent_kind.is_none() => nodes.push(ArticleNode::Island),
            Item::Roc(_)
            | Item::Component(_)
            | Item::Fixture(_)
            | Item::Css(_)
            | Item::Context(_)
            | Item::Init(_)
            | Item::Live(_)
            | Item::View(_)
            | Item::Fragment(_)
            | Item::Command(_)
                if parent_kind.is_none() => {}
            Item::Roc(_) => illegal(ctx, item, "roc"),
            Item::Render(_) => illegal(ctx, item, "render"),
            Item::Component(_) => illegal(ctx, item, "component"),
            Item::Fixture(_) => illegal(ctx, item, "fixture"),
            Item::Css(_) => illegal(ctx, item, "css"),
            Item::Context(_) => illegal(ctx, item, "context"),
            Item::Init(_) => illegal(ctx, item, "init"),
            Item::Live(_) => illegal(ctx, item, "live"),
            Item::View(_) => illegal(ctx, item, "view"),
            Item::Fragment(_) => illegal(ctx, item, "fragment"),
            Item::Command(_) => illegal(ctx, item, "command"),
            Item::Use(_) if parent_kind.is_some() => illegal(ctx, item, "use"),
            Item::Use(_) => {}
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
            "line {}: `@{kind}` is not allowed inside an article block",
            line_number(ctx.source.src, item.span().start as usize)
        ),
    ));
}

fn image_from_call(ctx: &mut BuildCtx<'_>, call: &BlockCall) -> ArticleNode {
    let mut diags = Vec::new();
    let body = call
        .params
        .as_ref()
        .map(|params| params.span)
        .unwrap_or(call.span);
    let fields = img_fields_from_params(call.params.as_ref(), body, &mut diags);
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
    ArticleNode::Image(StaticImage::from_fields(&fields, call.span))
}

fn heading_from_call(
    ctx: &mut BuildCtx<'_>,
    call: &BlockCall,
    parent_kind: Option<&str>,
) -> ArticleNode {
    let line = line_number(ctx.source.src, call.span.start as usize);
    let id = call
        .params
        .as_ref()
        .and_then(|params| params.fields.iter().find(|field| field.name == "id"))
        .and_then(|field| match &field.value {
            ParamValue::StringLit { value, .. } => Some(value.clone()),
            ParamValue::Ident { name, .. } => Some(name.clone()),
            _ => None,
        });

    let attrs = DocsAttrs {
        id,
        ..Default::default()
    };

    let content = heading_content_span(ctx.source.src, call);
    let children = if content.is_empty() {
        Vec::new()
    } else {
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
        unwrap_heading_children(nodes_from_items(
            ctx,
            &parsed.document.items,
            Some(&call.name),
        ))
    };
    let node = DocsNode {
        kind: call.name.clone(),
        attrs,
        children,
        origin: None,
        line,
    };
    validate_model(ctx, &node, parent_kind);
    ArticleNode::Block(node)
}

fn heading_content_span(src: &str, call: &BlockCall) -> Span {
    let span = call
        .content_span()
        .unwrap_or_else(|| Span::point(call.span.end as usize));
    if atx_heading(src, call.span) {
        clamp_span_to_first_line(src, span)
    } else {
        span
    }
}

fn clamp_span_to_first_line(src: &str, span: Span) -> Span {
    let start = floor_char_boundary(src, span.start as usize);
    let end = floor_char_boundary(src, span.end as usize).max(start);
    if start >= end {
        return Span::new(start, start);
    }
    let line_len = src[start..end].find('\n').unwrap_or(end - start);
    let mut line_end = start + line_len;
    if line_end > start && src.as_bytes()[line_end - 1] == b'\r' {
        line_end -= 1;
    }
    Span::new(start, line_end)
}

fn floor_char_boundary(src: &str, mut index: usize) -> usize {
    index = index.min(src.len());
    while index > 0 && !src.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn unwrap_heading_children(nodes: Vec<ArticleNode>) -> Vec<ArticleNode> {
    if let [ArticleNode::Markdown(MdNode::Paragraph { children, .. })] = nodes.as_slice() {
        return children
            .iter()
            .cloned()
            .map(ArticleNode::Markdown)
            .collect();
    }
    nodes
}

pub(crate) fn docs_node(
    ctx: &mut BuildCtx<'_>,
    call: &BlockCall,
    parent_kind: Option<&str>,
) -> Option<DocsNode> {
    let line = line_number(ctx.source.src, call.span.start as usize);
    let fields = docs_fields_from_params(call.params.as_ref());
    let content = call
        .content_span()
        .unwrap_or_else(|| Span::point(call.span.end as usize));
    let attrs = parse_attrs(ctx.source.src, &fields);
    if registry::is_reserved(&call.name) {
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2406",
            ctx.source_path,
            format!(
                "line {line}: `:api-operation` is reserved for generated API reference (Phase 6)"
            ),
        ));
        return None;
    }
    if !registry::is_docs_kind(&call.name) {
        if call.is_colon(ctx.source.src) {
            return None;
        }
        ctx.diagnostics.push(CatalogDiagnostic::error(
            "RD2401",
            ctx.source_path,
            format!("line {line}: unknown article kind `{}`", call.name),
        ));
        return None;
    }
    if call.name == "include" {
        return include_node(ctx, call.span, attrs, line);
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
    let children = nodes_from_items(ctx, &parsed.document.items, Some(&call.name));
    let node = DocsNode {
        kind: call.name.clone(),
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
