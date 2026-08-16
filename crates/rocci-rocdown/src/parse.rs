use comrak::{Arena, Options, parse_document};
use rocci_template::{
    Diagnostic, ModuleItem, SourceFile, Span, parse_declaration_from, parse_template_item_from,
};

use crate::ast::{DocsDecl, Document, Item, PageDecl, RenderDecl, RocDecl};
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
    let options = markdown_options(false, true);
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
            items,
            span: Span::new(0, source.src.len()),
        },
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
    let options = markdown_options(false, true);
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
        document: Document { items, span: body },
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
