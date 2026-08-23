use std::collections::HashSet;

use comrak::{Arena, Options, parse_document};
use rocci_template::{
    ComponentPath, Cursor, Diagnostic, Ident, ModuleItem, SourceFile, Span, camel_to_pascal,
    component_roc_name, is_ambiguous_pascal, parse_declaration_from, parse_template_item_from,
};

use crate::ast::{
    BlockCall, BlockContent, BraceSection, BracketRecord, Document, EndMarker, EndSection, Item,
    LineContent, MdNode, PageDecl, ParamField, ParamValue, RenderDecl, RocDecl, UseDecl,
};
use crate::markdown::{self, BlockOrHole};
use crate::params;
use crate::scan::{self, Reserved, ScannedDecl, ScannedKind, inner_span};

pub struct ParseOutput {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
    pub headings: Vec<crate::ast::HeadingInfo>,
    pub links: Vec<crate::ast::LinkInfo>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MarkdownBodyOptions {
    pub raw_html: bool,
    pub footnotes: bool,
}

pub fn parse(source: SourceFile<'_>, raw_html: bool) -> ParseOutput {
    let mut diagnostics = Vec::new();
    let scanned = scan::scan(source.src, &mut diagnostics);
    let (synthetic, map) = markdown::punch_holes(source.src, &scanned);
    let arena = Arena::new();
    let options = markdown_options(true, true);
    let root = parse_document(&arena, &synthetic, &options);
    let converted = markdown::convert_document(
        root,
        &synthetic,
        source.src,
        &map,
        raw_html,
        &mut diagnostics,
    );

    let mut items = Vec::new();
    for block in converted.blocks {
        match block {
            BlockOrHole::Block(node) => {
                if let Some(item) = map_md_block_to_item(source.src, node) {
                    items.push(item);
                }
            }
            BlockOrHole::Hole(index) => {
                if let Some(decl) = scanned.get(index)
                    && let Some(item) = fill_decl(source.src, decl, &mut diagnostics)
                {
                    items.push(item);
                }
            }
        }
    }

    let document = Document {
        items,
        span: Span::new(0, source.src.len()),
    };
    diagnose_heading_interpolations(source, &document.items, &mut diagnostics);
    let imported = crate::imports::imported_kind_names(source, &document, &mut diagnostics);
    validate_colon_tree(
        source.src,
        &document.items,
        None,
        &imported,
        &mut diagnostics,
    );
    validate_footnotes(source.src, &document, &mut diagnostics);

    let mut headings = converted.headings;
    let mut seen = headings
        .iter()
        .map(|h| (h.level, h.id.clone()))
        .collect::<HashSet<_>>();
    for item in &document.items {
        let Item::Block(call) = item else {
            continue;
        };
        if !call.is_colon(source.src) {
            continue;
        }
        let Some(level) = crate::registry::heading_level(&call.name) else {
            continue;
        };
        let Some(id) = heading_id_from_params(call) else {
            continue;
        };
        if !seen.insert((level, id.clone())) {
            continue;
        }
        let text = heading_text_from_call(source, call);
        headings.push(crate::ast::HeadingInfo {
            level,
            id,
            text,
            span: call.span,
        });
    }

    ParseOutput {
        document,
        diagnostics,
        headings,
        links: converted.links,
    }
}

pub fn parse_markdown_body(
    source: SourceFile<'_>,
    body: Span,
    body_options: MarkdownBodyOptions,
) -> ParseOutput {
    let body_src = source.slice(body);
    let map = markdown::OffsetMap::from_original(body.as_range());
    let arena = Arena::new();
    let options = markdown_options(body_options.footnotes, false);
    let root = parse_document(&arena, body_src, &options);
    let mut diagnostics = Vec::new();
    let converted = markdown::convert_document(
        root,
        body_src,
        source.src,
        &map,
        body_options.raw_html,
        &mut diagnostics,
    );
    let items = converted
        .blocks
        .into_iter()
        .filter_map(|block| match block {
            BlockOrHole::Block(node) => map_md_block_to_item(source.src, node),
            BlockOrHole::Hole(_) => None,
        })
        .collect();

    ParseOutput {
        document: Document { items, span: body },
        diagnostics,
        headings: converted.headings,
        links: converted.links,
    }
}

pub fn nested_items(src: &str, call: &BlockCall) -> Vec<Item> {
    let Some(span) = call.content_span() else {
        return Vec::new();
    };
    parse_fragment(SourceFile::new("block", src), span, false)
        .document
        .items
}

pub fn parse_fragment(source: SourceFile<'_>, body: Span, raw_html: bool) -> ParseOutput {
    let start = body.start as usize;
    let end = body.end as usize;
    if start >= end || start >= source.src.len() {
        return ParseOutput {
            document: Document {
                items: Vec::new(),
                span: body,
            },
            diagnostics: Vec::new(),
            headings: Vec::new(),
            links: Vec::new(),
        };
    }
    let end = end.min(source.src.len());
    let mut diagnostics = Vec::new();
    let scanned = scan::scan_range(source.src, start, end, &mut diagnostics);
    let (synthetic, map) = markdown::punch_holes_range(source.src, start, end, &scanned);
    let arena = Arena::new();
    let options = markdown_options(true, true);
    let root = parse_document(&arena, &synthetic, &options);
    let converted = markdown::convert_document(
        root,
        &synthetic,
        source.src,
        &map,
        raw_html,
        &mut diagnostics,
    );

    let mut items = Vec::new();
    for block in converted.blocks {
        match block {
            BlockOrHole::Block(node) => {
                if let Some(item) = map_md_block_to_item(source.src, node) {
                    items.push(item);
                }
            }
            BlockOrHole::Hole(index) => {
                if let Some(decl) = scanned.get(index)
                    && let Some(item) = fill_decl(source.src, decl, &mut diagnostics)
                {
                    items.push(item);
                }
            }
        }
    }

    ParseOutput {
        document: Document { items, span: body },
        diagnostics,
        headings: converted.headings,
        links: converted.links,
    }
}

fn diagnose_heading_interpolations(
    source: SourceFile<'_>,
    items: &[Item],
    diagnostics: &mut Vec<Diagnostic>,
) {
    const MESSAGE: &str = "Markdown interpolation `@{` is not allowed in headings";
    for item in items {
        let Item::Block(call) = item else {
            continue;
        };
        if crate::registry::heading_level(&call.name).is_none() {
            continue;
        }
        if !call.is_colon(source.src) {
            continue;
        }
        let Some(span) = call.content_span() else {
            continue;
        };
        let mut parsed = parse_fragment(source, span, false);
        diagnostics.extend(std::mem::take(&mut parsed.diagnostics));
        for nested in &mut parsed.document.items {
            if let Item::Markdown(md) = nested {
                markdown::restore_interpolations(
                    source.src,
                    std::slice::from_mut(md),
                    diagnostics,
                    MESSAGE,
                );
            }
        }
    }
}

fn heading_id_from_params(call: &BlockCall) -> Option<String> {
    let params = call.params.as_ref()?;
    params
        .fields
        .iter()
        .find(|field| field.name == "id")
        .and_then(|field| match &field.value {
            ParamValue::StringLit { value, .. } => Some(value.clone()),
            ParamValue::Ident { name, .. } => Some(name.clone()),
            _ => None,
        })
}

fn map_md_block_to_item(src: &str, node: MdNode) -> Option<Item> {
    match node {
        MdNode::Heading {
            level,
            id,
            children,
            span,
            ..
        } => {
            let content_span = if children.is_empty() {
                rocci_template::trim_span(src, Span::new(span.end as usize, span.end as usize))
            } else {
                let start = children
                    .iter()
                    .map(|child| child.span().start as usize)
                    .min()
                    .unwrap_or(span.start as usize);
                let end = children
                    .iter()
                    .map(|child| child.span().end as usize)
                    .max()
                    .unwrap_or(span.end as usize);
                rocci_template::trim_span(src, Span::new(start, end))
            };
            let params = Some(BracketRecord {
                fields: vec![ParamField {
                    name: "id".to_string(),
                    name_span: span,
                    value: ParamValue::StringLit {
                        value: id,
                        span: content_span,
                    },
                }],
                span,
            });
            Some(Item::Block(BlockCall {
                name: format!("h{level}"),
                name_span: span,
                params,
                content: Some(BlockContent::Line(LineContent { span: content_span })),
                span,
            }))
        }
        MdNode::Paragraph { children, span } => {
            if children.len() == 1
                && let MdNode::Image {
                    url,
                    alt,
                    title,
                    span: image_span,
                } = &children[0]
            {
                let mut fields = vec![ParamField {
                    name: "src".to_string(),
                    name_span: *image_span,
                    value: ParamValue::StringLit {
                        value: url.clone(),
                        span: *image_span,
                    },
                }];
                if !title.is_empty() {
                    fields.push(ParamField {
                        name: "title".to_string(),
                        name_span: *image_span,
                        value: ParamValue::StringLit {
                            value: title.clone(),
                            span: *image_span,
                        },
                    });
                }
                if alt.is_empty() {
                    fields.push(ParamField {
                        name: "decorative".to_string(),
                        name_span: *image_span,
                        value: ParamValue::BoolLit {
                            value: true,
                            span: *image_span,
                        },
                    });
                } else {
                    fields.push(ParamField {
                        name: "alt".to_string(),
                        name_span: *image_span,
                        value: ParamValue::StringLit {
                            value: alt.clone(),
                            span: *image_span,
                        },
                    });
                }
                return Some(Item::Block(BlockCall {
                    name: "img".to_string(),
                    name_span: span,
                    params: Some(BracketRecord { fields, span }),
                    content: None,
                    span,
                }));
            }
            Some(Item::Markdown(MdNode::Paragraph { children, span }))
        }
        other => Some(Item::Markdown(other)),
    }
}

fn heading_text_from_call(source: SourceFile<'_>, call: &BlockCall) -> String {
    let content = call
        .content_span()
        .unwrap_or_else(|| Span::point(call.span.end as usize));
    if content.is_empty() {
        return String::new();
    }
    let mut parsed = parse_fragment(source, content, false);
    for item in &mut parsed.document.items {
        if let Item::Markdown(md) = item {
            markdown::restore_interpolations(
                source.src,
                std::slice::from_mut(md),
                &mut Vec::new(),
                "Markdown interpolation `@{` is not allowed in headings",
            );
        }
    }
    let mut parts = Vec::new();
    for item in parsed.document.items {
        if let Item::Markdown(md) = item {
            let text = md.text_content();
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }
    parts.join(" ").trim().to_string()
}

fn markdown_options(footnotes: bool, wikilinks: bool) -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.footnotes = footnotes;
    options.extension.wikilinks_title_after_pipe = wikilinks;
    options
}

fn validate_footnotes(src: &str, document: &Document, diagnostics: &mut Vec<Diagnostic>) {
    let mut definitions = Vec::new();
    let mut references = Vec::new();
    for item in &document.items {
        let Item::Markdown(node) = item else {
            continue;
        };
        node.walk(&mut |child| match child {
            MdNode::FootnoteDefinition { name, span, .. } => {
                definitions.push((name.clone(), *span));
            }
            MdNode::FootnoteReference { name, span, .. } => {
                references.push((name.clone(), *span));
            }
            _ => {}
        });
    }
    let mut seen = std::collections::BTreeMap::<String, Span>::new();
    for (name, span) in &definitions {
        if seen.insert(name.clone(), *span).is_some() {
            diagnostics.push(Diagnostic::error(
                *span,
                format!("duplicate footnote definition `[^{name}]`"),
            ));
        }
    }
    for (name, span) in scan_source_footnote_refs(src) {
        if !seen.contains_key(&name) && !references.iter().any(|(existing, _)| existing == &name) {
            references.push((name, span));
        }
    }
    for (name, span) in references {
        if !seen.contains_key(&name) {
            diagnostics.push(Diagnostic::error(
                span,
                format!("footnote `[^{name}]` has no definition"),
            ));
        }
    }
}

fn scan_source_footnote_refs(src: &str) -> Vec<(String, Span)> {
    let mut refs = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut offset = 0;

    for line in src.split_inclusive('\n') {
        let line_offset = offset;
        offset += line.len();

        let line_no_newline = line.strip_suffix('\n').unwrap_or(line);
        let line_trimmed = line_no_newline
            .strip_suffix('\r')
            .unwrap_or(line_no_newline);

        if let Some((ch, n)) = fence {
            if scan::is_fence_close(line_trimmed, ch, n) {
                fence = None;
            }
            continue;
        }

        let stripped = scan::skip_0_3_spaces(line_trimmed);
        if let Some(open) = scan::fence_open(stripped) {
            fence = Some(open);
            continue;
        }

        let bytes = line_trimmed.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            if bytes[i] == b'`' {
                let code_start = i;
                let mut tick_count = 0;
                while i < len && bytes[i] == b'`' {
                    tick_count += 1;
                    i += 1;
                }
                let rest = &line_trimmed[i..];
                let needle = &line_trimmed[code_start..code_start + tick_count];
                if let Some(found) = rest.find(needle) {
                    i += found + tick_count;
                }
                continue;
            }

            if bytes[i] == b'[' && i + 1 < len && bytes[i + 1] == b'^' {
                let ref_start = i;
                i += 2;
                let name_start = i;
                let mut name_end = i;
                let mut closed = false;
                while i < len {
                    if bytes[i] == b']' {
                        name_end = i;
                        i += 1;
                        closed = true;
                        break;
                    }
                    if bytes[i] == b' ' || bytes[i] == b'\t' {
                        break;
                    }
                    i += 1;
                }

                if closed && name_end > name_start {
                    if i < len && bytes[i] == b':' {
                        continue;
                    }
                    let name = &line_trimmed[name_start..name_end];
                    let span = Span::new(line_offset + ref_start, line_offset + i);
                    refs.push((name.to_string(), span));
                }
                continue;
            }

            i += 1;
        }
    }

    refs
}

fn fill_decl(src: &str, decl: &ScannedDecl, diagnostics: &mut Vec<Diagnostic>) -> Option<Item> {
    match decl.kind {
        ScannedKind::Html => match parse_template_item_from(src, decl.at) {
            Some(parsed) => {
                diagnostics.extend(parsed.diagnostics);
                Some(Item::Template(parsed.item))
            }
            None => Some(Item::Roc(RocDecl {
                body: Span::new(decl.at, decl.end),
                span: Span::new(decl.at, decl.end),
            })),
        },
        ScannedKind::At(kind) => Some(fill_at_decl(src, decl, kind, diagnostics)),
        ScannedKind::Colon => parse_colon_call(src, decl, diagnostics),
        ScannedKind::ColonEnd | ScannedKind::RemovedAt => None,
    }
}

fn parse_colon_call(
    src: &str,
    decl: &ScannedDecl,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Item> {
    let mut cur = Cursor::at(src, decl.at);
    if !cur.eat(':') {
        return None;
    }
    let name_span = {
        let Some(span) = cur.scan_tag_name() else {
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected a block kind after `:`",
            ));
            return None;
        };
        span
    };
    let name = name_span.of(src).to_string();
    if cur.peek() == Some('.') {
        cur.bump();
        let _suffix = cur.scan_tag_name();
    }
    let (params, after_params) = params::parse_article_params(src, cur.pos, diagnostics);
    cur.pos = after_params;
    cur.skip_spaces_tabs();
    let content = if cur.starts_with("{{") {
        let inner_start = cur.pos + 2;
        let close = decl.end;
        let inner_end = if src.get(close.saturating_sub(2)..close) == Some("}}") {
            close - 2
        } else {
            close
        };
        Some(BlockContent::Brace(BraceSection {
            span: rocci_template::trim_span(src, Span::new(inner_start, inner_end)),
        }))
    } else if line_has_non_ws(src, cur.pos) {
        let end = line_end_at(src, decl.at.max(cur.pos));
        Some(BlockContent::Line(LineContent {
            span: rocci_template::trim_span(src, Span::new(cur.pos, end)),
        }))
    } else if let Some(marker) = end_marker_at(src, decl.end) {
        let body_start = next_line_at(src, decl.at);
        let body_end = marker_line_start(src, decl.end);
        Some(BlockContent::End(EndSection {
            span: rocci_template::trim_span(src, Span::new(body_start, body_end)),
            marker,
        }))
    } else {
        None
    };
    Some(Item::Block(BlockCall {
        name,
        name_span,
        params,
        content,
        span: Span::new(decl.at, decl.end),
    }))
}

fn line_has_non_ws(src: &str, pos: usize) -> bool {
    let rest = src.get(pos..).unwrap_or("");
    let line = rest.split('\n').next().unwrap_or(rest);
    line.chars().any(|ch| !ch.is_whitespace())
}

fn line_end_at(src: &str, pos: usize) -> usize {
    match src.get(pos..).and_then(|rest| rest.find('\n')) {
        Some(i) => pos + i,
        None => src.len(),
    }
}

fn next_line_at(src: &str, pos: usize) -> usize {
    match src.get(pos..).and_then(|rest| rest.find('\n')) {
        Some(i) => pos + i + 1,
        None => src.len(),
    }
}

fn marker_line_start(src: &str, end: usize) -> usize {
    src.get(..end)
        .and_then(|head| head.rfind('\n'))
        .map(|i| i + 1)
        .unwrap_or(0)
        .min(end)
}

fn end_marker_at(src: &str, end: usize) -> Option<EndMarker> {
    let start = marker_line_start(src, end);
    let mut at = start;
    while at < end && matches!(src.as_bytes().get(at), Some(b' ' | b'\t')) {
        at += 1;
    }
    let mut cur = Cursor::at(src, at);
    if !cur.eat(':') {
        return None;
    }
    let kind_span = cur.scan_tag_name()?;
    if !cur.eat('.') {
        return None;
    }
    let end_word = cur.scan_tag_name()?;
    if end_word.of(src) != "end" {
        return None;
    }
    Some(EndMarker {
        name: kind_span.of(src).to_string(),
        span: Span::new(at, end_word.end as usize),
    })
}

fn validate_colon_tree(
    src: &str,
    items: &[Item],
    parent: Option<&str>,
    imported: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(parent_name) = parent
        && let Some(spec) = crate::registry::lookup(parent_name)
    {
        validate_colon_children(src, items, spec, diagnostics);
    }
    for item in items {
        if let Item::Use(decl) = item
            && parent.is_some()
        {
            diagnostics.push(Diagnostic::error(
                decl.span,
                "`@use` is only valid at document root, not inside an article block",
            ));
            continue;
        }
        let Item::Block(call) = item else {
            continue;
        };
        if !call.is_colon(src) {
            continue;
        }
        validate_colon_call(call, parent, imported, diagnostics);
        if let Some(content) = call.content_span()
            && !content.is_empty()
        {
            let nested = parse_fragment(SourceFile::new("fragment", src), content, false);
            diagnostics.extend(nested.diagnostics);
            validate_colon_tree(
                src,
                &nested.document.items,
                Some(&call.name),
                imported,
                diagnostics,
            );
        }
    }
}

fn validate_colon_call(
    call: &BlockCall,
    parent: Option<&str>,
    imported: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if crate::registry::module_collision(&call.name) {
        diagnostics.push(Diagnostic::error(
            call.name_span,
            format!(
                "`:{}` collides with a reserved module name; article blocks cannot use `@` names",
                call.name
            ),
        ));
        return;
    }
    if imported.contains(&call.name) {
        return;
    }
    let Some(spec) = crate::registry::lookup(&call.name) else {
        diagnostics.push(Diagnostic::error(
            call.name_span,
            format!("unknown article kind `:{}`", call.name),
        ));
        return;
    };
    if !spec.authorable {
        diagnostics.push(Diagnostic::error(
            call.name_span,
            format!("`:{}` is not an authorable article kind", call.name),
        ));
        return;
    }
    if !crate::registry::parent_allowed(spec, parent) {
        diagnostics.push(Diagnostic::error(
            call.name_span,
            format!(
                "`:{}` is only valid inside `:{}`",
                spec.name, spec.parents[0]
            ),
        ));
    }
    let check_required = spec.parents.is_empty() || crate::registry::parent_allowed(spec, parent);
    if check_required {
        for field in spec.required_fields {
            if !colon_has_field(call, field) {
                diagnostics.push(Diagnostic::error(
                    call.name_span,
                    format!("`:{}` requires `{field}`", spec.name),
                ));
            }
        }
        for group in spec.required_one_of {
            if !group.iter().any(|field| colon_has_field(call, field)) {
                let joined = group.join("` or `");
                diagnostics.push(Diagnostic::error(
                    call.name_span,
                    format!("`:{}` requires `{joined}`", spec.name),
                ));
            }
        }
    }
}

fn validate_colon_children(
    src: &str,
    items: &[Item],
    spec: &crate::registry::KindSpec,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for item in items {
        match item {
            Item::Block(call) if call.is_colon(src) => {
                if !spec.accepts_block_child(&call.name) {
                    diagnostics.push(Diagnostic::error(
                        call.name_span,
                        format!("`:{}` cannot contain `:{}`", spec.name, call.name),
                    ));
                }
            }
            Item::Markdown(md) if spec.rejects_markdown() && !md.is_whitespace_only_paragraph() => {
                diagnostics.push(Diagnostic::error(
                    md.span(),
                    format!("`:{}` cannot contain Markdown", spec.name),
                ));
            }
            _ => {}
        }
    }
}

fn colon_has_field(call: &BlockCall, field: &str) -> bool {
    call.params
        .as_ref()
        .is_some_and(|params| params.fields.iter().any(|item| item.name == field))
}

fn fill_at_decl(
    src: &str,
    decl: &ScannedDecl,
    kind: Reserved,
    diagnostics: &mut Vec<Diagnostic>,
) -> Item {
    match kind {
        Reserved::Page => {
            let body = inner_span(src, decl.at);
            Item::Page(PageDecl {
                body,
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Roc => {
            let body = inner_span(src, decl.at);
            Item::Roc(RocDecl {
                body,
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Render => Item::Render(parse_render(src, decl, diagnostics)),
        Reserved::Use => {
            let (path, path_span) = crate::imports::parse_use_path(src, decl.at, decl.end);
            Item::Use(UseDecl {
                path,
                path_span,
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Component
        | Reserved::Fixture
        | Reserved::Css
        | Reserved::Context
        | Reserved::Init
        | Reserved::Get
        | Reserved::Post
        | Reserved::Put
        | Reserved::Live
        | Reserved::View
        | Reserved::Patch
        | Reserved::Delete
        | Reserved::Command
        | Reserved::On => match parse_declaration_from(src, decl.at) {
            Some(parsed) => {
                diagnostics.extend(parsed.diagnostics);
                match parsed.item {
                    ModuleItem::Component(item) => Item::Component(item),
                    ModuleItem::Fixture(item) => Item::Fixture(item),
                    ModuleItem::Css(item) => Item::Css(item),
                    ModuleItem::Context(item) => Item::Context(item),
                    ModuleItem::Init(item) => Item::Init(item),
                    ModuleItem::Route(rocci_template::RouteDecl::Live(item)) => Item::Live(item),
                    ModuleItem::Route(rocci_template::RouteDecl::View(item)) => Item::View(item),
                    ModuleItem::Route(rocci_template::RouteDecl::Fragment(item)) => {
                        Item::Fragment(item)
                    }
                    ModuleItem::Route(rocci_template::RouteDecl::Command(item)) => {
                        Item::Command(item)
                    }
                    ModuleItem::Roc { .. } => Item::Roc(RocDecl {
                        body: Span::new(decl.at, decl.end),
                        span: Span::new(decl.at, decl.end),
                    }),
                }
            }
            None => Item::Roc(RocDecl {
                body: Span::new(decl.at, decl.end),
                span: Span::new(decl.at, decl.end),
            }),
        },
        Reserved::If | Reserved::For | Reserved::Match | Reserved::Let => {
            match parse_template_item_from(src, decl.at) {
                Some(parsed) => {
                    diagnostics.extend(parsed.diagnostics);
                    Item::Template(parsed.item)
                }
                None => Item::Roc(RocDecl {
                    body: Span::new(decl.at, decl.end),
                    span: Span::new(decl.at, decl.end),
                }),
            }
        }
    }
}

fn parse_render(src: &str, decl: &ScannedDecl, diagnostics: &mut Vec<Diagnostic>) -> RenderDecl {
    let span = Span::new(decl.at, decl.end);
    let mut cur = Cursor::at(src, decl.at);
    cur.eat('@');
    cur.scan_ident();
    cur.skip_trivia();
    if cur.peek() == Some('{') {
        diagnostics.push(Diagnostic::error(
            span,
            "`@render` takes a PascalCase call, not a `{ }` body; write `@render MyComponent({ ... })`. For tags, write `<MyComponent />` as a standalone HTML block.",
        ));
        return RenderDecl {
            path: empty_render_path(Span::point(cur.pos)),
            args: Span::point(cur.pos),
            span,
        };
    }
    if cur.peek() == Some('<') {
        diagnostics.push(Diagnostic::error(
            Span::point(cur.pos),
            "`@render` takes a PascalCase call, not an HTML tag; write `@render MyComponent({ ... })`. For tags, write `<MyComponent />` as a standalone HTML block.",
        ));
        return RenderDecl {
            path: empty_render_path(Span::point(cur.pos)),
            args: Span::point(cur.pos),
            span,
        };
    }
    let Some(path) = parse_render_path(&mut cur, diagnostics) else {
        return RenderDecl {
            path: empty_render_path(Span::point(cur.pos)),
            args: Span::point(cur.pos),
            span,
        };
    };
    if let Some(last) = path.parts.last()
        && let Some(message) = render_target_error(&last.name)
    {
        diagnostics.push(Diagnostic::error(last.span, message));
    }
    cur.skip_trivia();
    if cur.peek() != Some('(') {
        return RenderDecl {
            path,
            args: Span::point(cur.pos),
            span,
        };
    }
    let paren_start = cur.pos;
    cur.skip_balanced_parens();
    let args = if cur.pos > paren_start + 1
        && src.as_bytes().get(cur.pos.saturating_sub(1)) == Some(&b')')
    {
        rocci_template::trim_span(src, Span::new(paren_start + 1, cur.pos - 1))
    } else {
        Span::point(paren_start.saturating_add(1))
    };
    RenderDecl { path, args, span }
}

fn parse_render_path(
    cur: &mut Cursor<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ComponentPath> {
    let first = cur.scan_ident()?;
    let mut parts = vec![Ident {
        name: cur.ident_text(first).to_string(),
        span: first,
    }];
    while cur.eat('.') {
        if let Some(next) = cur.scan_ident() {
            parts.push(Ident {
                name: cur.ident_text(next).to_string(),
                span: next,
            });
        } else {
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected identifier after `.`",
            ));
            break;
        }
    }
    let span = Span::new(
        parts.first().unwrap().span.start as usize,
        parts.last().unwrap().span.end as usize,
    );
    if parts
        .last()
        .is_some_and(|part| is_ambiguous_pascal(&part.name))
    {
        diagnostics.push(Diagnostic::error(
            parts.last().unwrap().span,
            format!(
                "ambiguous render target `{}`; write `@render HtmlShell(...)` rather than `@render {}(...)`",
                parts.last().unwrap().name,
                parts.last().unwrap().name
            ),
        ));
    }
    let roc_name = component_roc_name(&parts);
    Some(ComponentPath {
        parts,
        roc_name,
        span,
    })
}

fn empty_render_path(span: Span) -> ComponentPath {
    ComponentPath {
        parts: Vec::new(),
        roc_name: String::new(),
        span,
    }
}

fn render_target_error(name: &str) -> Option<String> {
    if name.is_empty() || is_ambiguous_pascal(name) {
        return None;
    }
    if !name
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        Some(format!(
            "render targets must be PascalCase; write `@render {}(...)`",
            camel_to_pascal(name)
        ))
    } else {
        None
    }
}
