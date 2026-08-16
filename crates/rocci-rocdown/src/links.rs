use std::path::{Path, PathBuf};

use rocci_template::{Diagnostic, SourceFile, Span};

use crate::CompileOptions;
use crate::ast::{Document, HeadingInfo, Item, MdNode};
use crate::page::extract_page;
use crate::parse::ParseOutput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRef {
    pub stem: String,
    pub file_name: String,
    pub route: String,
    pub heading_ids: Vec<String>,
}

pub fn page_ref_from_source(path: &Path, src: &str) -> PageRef {
    let name = path.display().to_string();
    let parsed = crate::parse::parse(rocci_template::SourceFile::new(&name, src), false);
    page_ref_from_parsed(path, src, &parsed)
}

fn page_ref_from_parsed(path: &Path, src: &str, parsed: &ParseOutput) -> PageRef {
    let mut diagnostics = Vec::new();
    let route = parsed
        .document
        .items
        .iter()
        .find_map(|item| match item {
            Item::Page(page) => Some(extract_page(src, page.body, &mut diagnostics).route),
            _ => None,
        })
        .flatten()
        .unwrap_or_else(|| "/".to_string());
    PageRef {
        stem: path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        file_name: path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        route,
        heading_ids: parsed.headings.iter().map(|h| h.id.clone()).collect(),
    }
}

pub fn index_pages<'a, I>(files: I) -> Vec<PageRef>
where
    I: IntoIterator<Item = (&'a Path, &'a str)>,
{
    files
        .into_iter()
        .map(|(path, src)| page_ref_from_source(path, src))
        .collect()
}

pub fn index_pages_in_dir(dir: &Path) -> Vec<PageRef> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "rocdown"))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let src = std::fs::read_to_string(&path).ok()?;
            Some(page_ref_from_source(&path, &src))
        })
        .collect()
}

pub fn resolve_document(
    source: SourceFile<'_>,
    parsed: &mut ParseOutput,
    options: &CompileOptions,
) {
    report_route_collisions(
        source,
        &parsed.document,
        &options.pages,
        &mut parsed.diagnostics,
    );
    let headings = parsed.headings.clone();
    for item in &mut parsed.document.items {
        if let Item::Markdown(node) = item {
            resolve_md(node, &headings, options, &mut parsed.diagnostics);
        }
    }
    let mut resolved = Vec::new();
    for item in &parsed.document.items {
        if let Item::Markdown(node) = item {
            collect_link_urls(node, &mut resolved);
        }
    }
    for link in &mut parsed.links {
        if let Some((_, url)) = resolved.iter().find(|(span, _)| *span == link.span) {
            link.url = url.clone();
        }
    }
}

fn report_route_collisions(
    source: SourceFile<'_>,
    document: &Document,
    pages: &[PageRef],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let current_name = Path::new(source.name)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let page_decl = document.items.iter().find_map(|item| match item {
        Item::Page(page) => Some(page),
        _ => None,
    });
    let mut extract_diags = Vec::new();
    let (span, route, explicit) = match page_decl {
        Some(page) => {
            let meta = extract_page(source.src, page.body, &mut extract_diags);
            let explicit = meta.route.is_some();
            (
                page.span,
                meta.route.unwrap_or_else(|| "/".to_string()),
                explicit,
            )
        }
        None => (Span::point(0), "/".to_string(), false),
    };
    let in_index = pages.iter().any(|page| page.file_name == current_name);
    let others: Vec<&str> = pages
        .iter()
        .filter(|page| page.route == route && page.file_name != current_name)
        .map(|page| page.file_name.as_str())
        .collect();
    if !others.is_empty() && (explicit || in_index) {
        diagnostics.push(Diagnostic::error(
            span,
            format!(
                "`@page.route` `{route}` is also used by {}",
                others.join(", ")
            ),
        ));
    }
}

fn resolve_md(
    node: &mut MdNode,
    headings: &[HeadingInfo],
    options: &CompileOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let MdNode::Link { url, span, .. } = node {
        match resolve_url(url, *span, headings, options) {
            Ok(resolved) => *url = resolved,
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for child in node.children_mut() {
        resolve_md(child, headings, options, diagnostics);
    }
}

fn collect_link_urls(node: &MdNode, out: &mut Vec<(Span, String)>) {
    if let MdNode::Link { url, span, .. } = node {
        out.push((*span, url.clone()));
    }
    let children = match node {
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
        | MdNode::Link { children, .. } => children.as_slice(),
        _ => &[],
    };
    for child in children {
        collect_link_urls(child, out);
    }
}

fn resolve_url(
    url: &str,
    span: Span,
    headings: &[HeadingInfo],
    options: &CompileOptions,
) -> Result<String, Diagnostic> {
    let decoded = percent_decode(url);
    let (path, fragment) = split_fragment(&decoded);
    if path.is_empty() {
        if let Some(id) = fragment {
            return same_page_heading(id, span, headings);
        }
        return Ok(decoded);
    }
    if has_scheme(path) {
        return Ok(decoded);
    }
    if path.starts_with('/') {
        if !options.pages.is_empty() && !options.pages.iter().any(|page| page.route == path) {
            return Err(Diagnostic::error(
                span,
                format!("unknown Rocdown route `{path}`"),
            ));
        }
        return Ok(with_fragment(path, fragment));
    }
    if let Some(stem) = page_stem(path) {
        return resolve_page(stem, fragment, span, options);
    }
    Ok(decoded)
}

fn same_page_heading(id: &str, span: Span, headings: &[HeadingInfo]) -> Result<String, Diagnostic> {
    if headings.iter().any(|heading| heading.id == id) {
        Ok(format!("#{id}"))
    } else {
        Err(Diagnostic::error(span, format!("unknown heading `{id}`")))
    }
}

fn resolve_page(
    stem: &str,
    fragment: Option<&str>,
    span: Span,
    options: &CompileOptions,
) -> Result<String, Diagnostic> {
    let Some(page) = options
        .pages
        .iter()
        .find(|page| page.stem == stem || page.file_name == format!("{stem}.rocdown"))
    else {
        return Err(Diagnostic::error(
            span,
            format!("unknown Rocdown page `{stem}`"),
        ));
    };
    if let Some(id) = fragment {
        if !page.heading_ids.iter().any(|heading| heading == id) {
            return Err(Diagnostic::error(
                span,
                format!("unknown heading `{id}` on page `{stem}`"),
            ));
        }
        return Ok(with_fragment(&page.route, Some(id)));
    }
    Ok(page.route.clone())
}

fn page_stem(path: &str) -> Option<&str> {
    let trimmed = path
        .strip_prefix("./")
        .or_else(|| path.strip_prefix(".\\"))
        .unwrap_or(path);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') {
        return None;
    }
    if let Some(stem) = trimmed.strip_suffix(".rocdown") {
        if stem.is_empty() {
            return None;
        }
        return Some(stem);
    }
    let ext = Path::new(trimmed).extension();
    if ext.is_some() {
        return None;
    }
    Some(trimmed)
}

fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some(("", fragment)) => ("", Some(fragment)),
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

fn has_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    !scheme.is_empty() && scheme.chars().all(|ch| ch.is_ascii_alphabetic())
}

fn with_fragment(path: &str, fragment: Option<&str>) -> String {
    match fragment {
        Some(id) => format!("{path}#{id}"),
        None => path.to_string(),
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(high) = from_hex(bytes[i + 1])
            && let Some(low) = from_hex(bytes[i + 2])
        {
            out.push((high << 4) | low);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
