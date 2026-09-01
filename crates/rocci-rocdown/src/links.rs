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
    pub path: PathBuf,
    pub route: String,
    pub explicit_route: bool,
    pub heading_ids: Vec<String>,
}

pub fn page_ref_from_source(path: &Path, src: &str) -> PageRef {
    let name = path.display().to_string();
    let parsed = crate::parse::parse(rocci_template::SourceFile::new(&name, src), false);
    page_ref_from_parsed(path, src, &parsed)
}

fn page_ref_from_parsed(path: &Path, src: &str, parsed: &ParseOutput) -> PageRef {
    let mut diagnostics = Vec::new();
    let explicit_route = parsed.document.items.iter().find_map(|item| match item {
        Item::Page(page) => extract_page(src, page.body, &mut diagnostics).route,
        _ => None,
    });
    let route = explicit_route.clone().unwrap_or_else(|| "/".to_string());
    PageRef {
        stem: path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        file_name: path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        path: path.to_path_buf(),
        route,
        explicit_route: explicit_route.is_some(),
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
            resolve_md(node, source, &headings, options, &mut parsed.diagnostics);
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
    if !explicit {
        return;
    }
    let others: Vec<&str> = pages
        .iter()
        .filter(|page| page.explicit_route && page.route == route && page.file_name != current_name)
        .map(|page| page.file_name.as_str())
        .collect();
    if !others.is_empty() {
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
    source: SourceFile<'_>,
    headings: &[HeadingInfo],
    options: &CompileOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let MdNode::Link { url, span, .. } = node {
        match resolve_url(url, *span, source, headings, options) {
            Ok(resolved) => *url = resolved,
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    for child in node.children_mut() {
        resolve_md(child, source, headings, options, diagnostics);
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
    source: SourceFile<'_>,
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
        return resolve_absolute(path, fragment, span, &decoded, source.name, options);
    }
    if let Some(page) = page_for_relative(source, path, options) {
        return page_destination(page, fragment, span);
    }
    if let Some(stem) = page_stem(path) {
        return resolve_page(stem, fragment, span, options);
    }
    Ok(decoded)
}

fn resolve_absolute(
    path: &str,
    fragment: Option<&str>,
    span: Span,
    decoded: &str,
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))] source_name: &str,
    options: &CompileOptions,
) -> Result<String, Diagnostic> {
    if let Some(page) = page_for_route(path, &options.pages) {
        return page_destination(page, fragment, span);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(page) = crate::site::workspace_page_for_route(source_name, path) {
        return page_destination(&page, fragment, span);
    }
    if is_document_href(path) {
        if let Some(page) = page_for_absolute_document(path, options) {
            return page_destination(page, fragment, span);
        }
        return Ok(decoded.to_string());
    }
    if !options.pages.is_empty() {
        return Err(Diagnostic::error(
            span,
            format!("unknown Rocdown route `{path}`"),
        ));
    }
    Ok(with_fragment(path, fragment))
}

fn page_for_route<'a>(path: &str, pages: &'a [PageRef]) -> Option<&'a PageRef> {
    pages.iter().find(|page| routes_match(&page.route, path))
}

pub(crate) fn routes_match(left: &str, right: &str) -> bool {
    let left = crate::catalog::with_trailing_slash(left);
    let right = crate::catalog::with_trailing_slash(right);
    if left == right {
        return true;
    }
    if let Some(stripped) = left.strip_prefix("/docs") {
        return stripped == right || (left == "/docs/" && right == "/");
    }
    if let Some(stripped) = right.strip_prefix("/docs") {
        return stripped == left || (right == "/docs/" && left == "/");
    }
    false
}

fn page_for_relative<'a>(
    source: SourceFile<'_>,
    url_path: &str,
    options: &'a CompileOptions,
) -> Option<&'a PageRef> {
    let parent = Path::new(source.name).parent()?;
    let candidate = normalize_join(parent, url_path);
    options
        .pages
        .iter()
        .find(|page| paths_eq(&page.path, &candidate))
}

fn page_for_absolute_document<'a>(path: &str, options: &'a CompileOptions) -> Option<&'a PageRef> {
    let needle = path.trim_start_matches('/');
    let mut matches = options.pages.iter().filter(|page| {
        if page.path.as_os_str().is_empty() {
            return false;
        }
        let page_path = unix_path(&page.path);
        page_path == needle || page_path.ends_with(&format!("/{needle}"))
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    Some(first)
}

fn page_href(page: &PageRef) -> String {
    crate::catalog::canonical_route(&page.route, page.stem == "index")
}

fn page_destination(
    page: &PageRef,
    fragment: Option<&str>,
    span: Span,
) -> Result<String, Diagnostic> {
    let route = page_href(page);
    if let Some(id) = fragment {
        if !page.heading_ids.iter().any(|heading| heading == id) && !is_source_line_anchor_id(id) {
            return Err(Diagnostic::error(
                span,
                format!("unknown heading `{id}` on page `{}`", page.stem),
            ));
        }
        return Ok(with_fragment(&route, Some(id)));
    }
    Ok(route)
}

fn same_page_heading(id: &str, span: Span, headings: &[HeadingInfo]) -> Result<String, Diagnostic> {
    if headings.iter().any(|heading| heading.id == id) || is_source_line_anchor_id(id) {
        Ok(format!("#{id}"))
    } else {
        Err(Diagnostic::error(span, format!("unknown heading `{id}`")))
    }
}

pub(crate) fn is_source_line_anchor_id(fragment: &str) -> bool {
    let Some(digits) = fragment.strip_prefix('L') else {
        return false;
    };
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn resolve_page(
    stem: &str,
    fragment: Option<&str>,
    span: Span,
    options: &CompileOptions,
) -> Result<String, Diagnostic> {
    let Some(page) = options.pages.iter().find(|page| {
        page.stem == stem
            || page.file_name == format!("{stem}.rocdown")
            || page.file_name == format!("{stem}.md")
            || page.file_name == format!("{stem}.markdown")
    }) else {
        return Err(Diagnostic::error(
            span,
            format!("unknown Rocdown page `{stem}`"),
        ));
    };
    if let Some(id) = fragment {
        if !page.heading_ids.iter().any(|heading| heading == id) && !is_source_line_anchor_id(id) {
            return Err(Diagnostic::error(
                span,
                format!("unknown heading `{id}` on page `{stem}`"),
            ));
        }
        return Ok(with_fragment(&page_href(page), Some(id)));
    }
    Ok(page_href(page))
}

fn page_stem(path: &str) -> Option<&str> {
    let trimmed = path
        .strip_prefix("./")
        .or_else(|| path.strip_prefix(".\\"))
        .unwrap_or(path);
    if trimmed.is_empty() || trimmed.contains('/') || trimmed.contains('\\') {
        return None;
    }
    if let Some(stem) = trimmed
        .strip_suffix(".rocdown")
        .or_else(|| trimmed.strip_suffix(".md"))
        .or_else(|| trimmed.strip_suffix(".markdown"))
    {
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

pub(crate) fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some(("", fragment)) => ("", Some(fragment)),
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

pub(crate) fn has_scheme(path: &str) -> bool {
    let Some((scheme, _)) = path.split_once(':') else {
        return false;
    };
    !scheme.is_empty() && scheme.chars().all(|ch| ch.is_ascii_alphabetic())
}

pub(crate) fn is_document_href(path: &str) -> bool {
    let trimmed = path.split_once('?').map(|(path, _)| path).unwrap_or(path);
    matches!(
        Path::new(trimmed)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("rocdown" | "md" | "markdown")
    )
}

pub(crate) fn normalize_join(base: &Path, rel: &str) -> PathBuf {
    let rel = rel
        .strip_prefix("./")
        .or_else(|| rel.strip_prefix(".\\"))
        .unwrap_or(rel);
    normalize_components(base.join(rel))
}

pub(crate) fn normalize_components(path: PathBuf) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn paths_eq(left: &Path, right: &Path) -> bool {
    if left.as_os_str().is_empty() {
        return false;
    }
    left == right || unix_path(left) == unix_path(right)
}

fn unix_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn with_fragment(path: &str, fragment: Option<&str>) -> String {
    match fragment {
        Some(id) => format!("{path}#{id}"),
        None => path.to_string(),
    }
}

pub(crate) fn percent_decode(input: &str) -> String {
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
