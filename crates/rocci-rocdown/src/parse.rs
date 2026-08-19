use comrak::{Arena, Options, parse_document};
use rocci_template::{
    Diagnostic, ModuleItem, SourceFile, Span, parse_declaration_from, parse_template_item_from,
};

use crate::ast::{DocsDecl, Document, ImgDecl, Item, MdNode, PageDecl, RenderDecl, RocDecl};
use crate::markdown::{self, BlockOrHole};
use crate::scan::{
    self, Reserved, ScannedDecl, ScannedKind, docs_inner_span, docs_kind_span, inner_span,
};

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
    let converted = markdown::convert_document(root, &synthetic, &map, raw_html, &mut diagnostics);

    let mut items = Vec::new();
    for block in converted.blocks {
        match block {
            BlockOrHole::Block(node) => items.push(Item::Markdown(node)),
            BlockOrHole::Hole(index) => {
                if let Some(decl) = scanned.get(index) {
                    items.push(fill_decl(source.src, decl, &mut diagnostics));
                }
            }
        }
    }

    let document = Document {
        items: crate::docs::normalize_blocks(source.src, items),
        span: Span::new(0, source.src.len()),
    };
    validate_footnotes(source.src, &document, &mut diagnostics);

    ParseOutput {
        document,
        diagnostics,
        headings: converted.headings,
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
        &map,
        body_options.raw_html,
        &mut diagnostics,
    );
    let items = converted
        .blocks
        .into_iter()
        .filter_map(|block| match block {
            BlockOrHole::Block(node) => Some(Item::Markdown(node)),
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
    let converted = markdown::convert_document(root, &synthetic, &map, raw_html, &mut diagnostics);

    let mut items = Vec::new();
    for block in converted.blocks {
        match block {
            BlockOrHole::Block(node) => items.push(Item::Markdown(node)),
            BlockOrHole::Hole(index) => {
                if let Some(decl) = scanned.get(index) {
                    items.push(fill_decl(source.src, decl, &mut diagnostics));
                }
            }
        }
    }

    ParseOutput {
        document: Document {
            items: crate::docs::normalize_blocks(source.src, items),
            span: body,
        },
        diagnostics,
        headings: converted.headings,
        links: converted.links,
    }
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

fn fill_decl(src: &str, decl: &ScannedDecl, diagnostics: &mut Vec<Diagnostic>) -> Item {
    match decl.kind {
        ScannedKind::Html => match parse_template_item_from(src, decl.at) {
            Some(parsed) => {
                diagnostics.extend(parsed.diagnostics);
                Item::Template(parsed.item)
            }
            None => Item::Roc(RocDecl {
                body: Span::new(decl.at, decl.end),
                span: Span::new(decl.at, decl.end),
            }),
        },
        ScannedKind::At(kind) => fill_at_decl(src, decl, kind, diagnostics),
    }
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
        Reserved::Render => {
            let expr = inner_span(src, decl.at);
            Item::Render(RenderDecl {
                expr,
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Docs => {
            let kind_span = docs_kind_span(src, decl.at);
            Item::Docs(DocsDecl {
                kind: kind_span.of(src).to_string(),
                kind_span,
                body: docs_inner_span(src, decl.at),
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Img => {
            let body = inner_span(src, decl.at);
            Item::Img(ImgDecl {
                body,
                span: Span::new(decl.at, decl.end),
            })
        }
        Reserved::Component
        | Reserved::Fixture
        | Reserved::Css
        | Reserved::Context
        | Reserved::Init
        | Reserved::On => match parse_declaration_from(src, decl.at) {
            Some(parsed) => {
                diagnostics.extend(parsed.diagnostics);
                match parsed.item {
                    ModuleItem::Component(item) => Item::Component(item),
                    ModuleItem::Fixture(item) => Item::Fixture(item),
                    ModuleItem::Css(item) => Item::Css(item),
                    ModuleItem::Context(item) => Item::Context(item),
                    ModuleItem::Init(item) => Item::Init(item),
                    ModuleItem::On(item) => Item::On(item),
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
