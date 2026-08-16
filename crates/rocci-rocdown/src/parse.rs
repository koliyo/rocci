use comrak::{Arena, Options, parse_document};
use rocci_template::{
    Diagnostic, ModuleItem, SourceFile, Span, parse_declaration_from, parse_template_item_from,
};

use crate::ast::{Document, Item, PageDecl, RenderDecl, RocDecl};
use crate::markdown::{self, BlockOrHole};
use crate::scan::{self, Reserved, ScannedDecl, ScannedKind, inner_span};

pub struct ParseOutput {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
    pub headings: Vec<crate::ast::HeadingInfo>,
    pub links: Vec<crate::ast::LinkInfo>,
}

pub fn parse(source: SourceFile<'_>, raw_html: bool) -> ParseOutput {
    let mut diagnostics = Vec::new();
    let scanned = scan::scan(source.src, &mut diagnostics);
    let (synthetic, map) = markdown::punch_holes(source.src, &scanned);
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.autolink = true;
    options.extension.wikilinks_title_after_pipe = true;
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
