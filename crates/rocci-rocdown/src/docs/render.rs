use std::collections::BTreeMap;

use rocci_template::Diagnostic;

use crate::article::{
    StaticRender, collect_md_interpolation_gates, render_md, render_static_image,
};
use crate::ast::MdNode;
use crate::registry::{self, KindSpec, PaintType};

use super::fields::{attr_bool, attr_str};
use super::{ArticleNode, DocsAttrs, DocsNode, PlannedNode, PlannedProp, PlannedWidget};

pub fn collect_article_interpolation_gates(nodes: &[ArticleNode], out: &mut Vec<Diagnostic>) {
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => collect_md_interpolation_gates(md, out),
            ArticleNode::Block(docs) => collect_article_interpolation_gates(&docs.children, out),
            ArticleNode::Image(_) | ArticleNode::Island => {}
        }
    }
}

pub fn render_article_gated(nodes: &[ArticleNode]) -> StaticRender {
    let mut diagnostics = Vec::new();
    collect_article_interpolation_gates(nodes, &mut diagnostics);
    StaticRender {
        html: render_article(nodes),
        diagnostics,
    }
}

pub fn markdown_fragment_gated(nodes: &[ArticleNode]) -> (String, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    collect_article_interpolation_gates(nodes, &mut diagnostics);
    (markdown_fragment(nodes), diagnostics)
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
        ArticleNode::Block(docs) => render_docs(docs),
        ArticleNode::Image(image) => render_static_image(image),
        ArticleNode::Island => crate::article::ISLAND_PLACEHOLDER.to_string(),
    }
}

fn render_docs(docs: &DocsNode) -> String {
    if let Some(level) = registry::heading_level(&docs.kind) {
        let tag = format!("h{level}");
        let class = format!("rd-header-{level}");
        let id = docs.attrs.id.as_deref().unwrap_or("");
        return crate::article::render_heading(
            &tag,
            &class,
            id,
            &docs
                .children
                .iter()
                .map(|child| match child {
                    ArticleNode::Markdown(md) => render_md(md),
                    other => render_node(other),
                })
                .collect::<Vec<_>>(),
        );
    }
    if docs.kind == "include"
        && let Some(child) = docs.children.first()
        && let ArticleNode::Markdown(MdNode::CodeBlock { info, literal, .. }) = child
    {
        let line_start = docs
            .origin
            .as_ref()
            .and_then(|origin| origin.line_start)
            .unwrap_or(1);
        return crate::article::render_code_block_with_lines(info, literal, Some(line_start));
    }
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
            ArticleNode::Block(docs) => out.push_str(&docs_to_markdown(docs)),
            ArticleNode::Image(image) => {
                out.push_str(&format!("![{}]({})", image.alt, image.src));
                out.push('\n');
            }
            ArticleNode::Island => {}
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
        kind if registry::is_aside(kind) => {
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
                    ArticleNode::Block(step) if step.kind == "step" => Some(step),
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
                ArticleNode::Block(tab) if tab.kind == "tab" => Some(format!(
                    "### {}\n\n{}\n",
                    tab.attrs.label.as_deref().unwrap_or(""),
                    markdown_fragment(&tab.children).trim()
                )),
                _ => None,
            })
            .collect(),
        kind if registry::heading_level(kind).is_some() => {
            let level = registry::heading_level(kind).unwrap();
            format!("{} {}\n\n", "#".repeat(level as usize), inner.trim())
        }
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
        MdNode::Interpolation { .. } => String::new(),
        _ => node.text_content(),
    }
}

pub fn plan_segments(
    article_name: &str,
    nodes: &[ArticleNode],
    rewrite: &BTreeMap<String, String>,
) -> (Vec<PlannedNode>, Vec<(String, String)>) {
    plan_segments_with_islands(article_name, nodes, rewrite, &[])
}

pub fn plan_segments_with_islands(
    article_name: &str,
    nodes: &[ArticleNode],
    rewrite: &BTreeMap<String, String>,
    islands: &[String],
) -> (Vec<PlannedNode>, Vec<(String, String)>) {
    let mut files = Vec::new();
    let mut counter = 0u32;
    let mut island_index = 0usize;
    let segments = plan_nodes(
        article_name,
        nodes,
        rewrite,
        islands,
        &mut island_index,
        &mut files,
        &mut counter,
    );
    (segments, files)
}

fn plan_nodes(
    article_name: &str,
    nodes: &[ArticleNode],
    rewrite: &BTreeMap<String, String>,
    islands: &[String],
    island_index: &mut usize,
    files: &mut Vec<(String, String)>,
    counter: &mut u32,
) -> Vec<PlannedNode> {
    let mut segments = Vec::new();
    let mut markdown = Vec::new();
    let flush = |markdown: &mut Vec<MdNode>,
                 files: &mut Vec<(String, String)>,
                 counter: &mut u32,
                 segments: &mut Vec<PlannedNode>| {
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
        segments.push(PlannedNode::Html { path });
    };
    for node in nodes {
        match node {
            ArticleNode::Markdown(md) => markdown.push(md.clone()),
            ArticleNode::Island => {
                flush(&mut markdown, files, counter, &mut segments);
                let path = format!("articles/{article_name}.{counter}.html");
                *counter += 1;
                let html = islands
                    .get(*island_index)
                    .cloned()
                    .unwrap_or_else(|| crate::article::ISLAND_PLACEHOLDER.to_string());
                *island_index += 1;
                files.push((path.clone(), html));
                segments.push(PlannedNode::Html { path });
            }
            ArticleNode::Image(image) => {
                flush(&mut markdown, files, counter, &mut segments);
                let html = rewrite_urls(&render_static_image(image), rewrite);
                let path = format!("articles/{article_name}.{counter}.html");
                *counter += 1;
                files.push((path.clone(), html));
                segments.push(PlannedNode::Html { path });
            }
            ArticleNode::Block(docs)
                if registry::lookup(&docs.kind).is_some_and(|spec| spec.paints_as_widget()) =>
            {
                flush(&mut markdown, files, counter, &mut segments);
                segments.push(widget_node(
                    article_name,
                    docs,
                    rewrite,
                    islands,
                    island_index,
                    files,
                    counter,
                ));
            }
            ArticleNode::Block(docs) => {
                flush(&mut markdown, files, counter, &mut segments);
                let html = rewrite_urls(&render_docs(docs), rewrite);
                let path = format!("articles/{article_name}.{counter}.html");
                *counter += 1;
                files.push((path.clone(), html));
                segments.push(PlannedNode::Html { path });
            }
        }
    }
    flush(&mut markdown, files, counter, &mut segments);
    segments
}

fn widget_node(
    article_name: &str,
    docs: &DocsNode,
    rewrite: &BTreeMap<String, String>,
    islands: &[String],
    island_index: &mut usize,
    files: &mut Vec<(String, String)>,
    counter: &mut u32,
) -> PlannedNode {
    let spec = registry::lookup(&docs.kind).expect("validated widget kind");
    let children = if spec.paint_content() {
        plan_nodes(
            article_name,
            &docs.children,
            rewrite,
            islands,
            island_index,
            files,
            counter,
        )
    } else {
        Vec::new()
    };
    PlannedNode::Widget(PlannedWidget {
        kind: docs.kind.clone(),
        component: spec.component.to_string(),
        props: paint_props(*spec, &docs.attrs, rewrite),
        children,
        paint_content: spec.paint_content(),
    })
}

fn paint_props(
    spec: KindSpec,
    attrs: &DocsAttrs,
    rewrite: &BTreeMap<String, String>,
) -> Vec<PlannedProp> {
    spec.paint_fields()
        .iter()
        .map(|field| match field.ty {
            PaintType::Str => {
                let mut value = attr_str(attrs, field.attr);
                if field.prop == "href" {
                    value = rewrite_urls(&value, rewrite);
                }
                PlannedProp::Str {
                    name: field.prop.to_string(),
                    value,
                }
            }
            PaintType::Bool => PlannedProp::Bool {
                name: field.prop.to_string(),
                value: attr_bool(attrs, field.attr),
            },
        })
        .collect()
}

pub(crate) fn rewrite_urls(text: &str, map: &BTreeMap<String, String>) -> String {
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
