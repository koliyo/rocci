use rocci_template::{
    AttrValue, ComponentCall, ComponentDecl, ContextDecl, CssDecl, Document as RocciDocument,
    Element, FixtureDecl, InitDecl, ModuleItem, OnDecl, SourceFile, Span, TemplateItem,
};

use crate::embedded;
use crate::language::LanguageId;
use crate::regions::{RegionPurpose, RegionTree, extract_rocci_regions, extract_rocdown_regions};
use crate::token::{
    HighlightKind, HighlightSpan, MOD_DECLARATION, MOD_DEFAULT_LIBRARY, floor_char_boundary,
    resolve_and_sort_spans,
};

pub fn highlight(language: LanguageId, source: &str) -> Vec<HighlightSpan> {
    match language {
        LanguageId::Roc => {
            let raw = embedded::roc::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        LanguageId::Css => {
            let raw = embedded::css::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        LanguageId::Html => {
            let raw = embedded::html::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        LanguageId::Rocci => highlight_rocci(source),
        LanguageId::Rocdown | LanguageId::Markdown => highlight_rocdown(source),
        LanguageId::Shell | LanguageId::Toml | LanguageId::PlainText | LanguageId::Other(_) => {
            Vec::new()
        }
    }
}

pub fn highlight_rocci(source: &str) -> Vec<HighlightSpan> {
    let sf = SourceFile::new("snippet.rocci", source);
    let parsed = rocci_template::parse(sf);
    highlight_rocci_document(source, &parsed.document)
}

pub fn highlight_rocci_document(source: &str, document: &RocciDocument) -> Vec<HighlightSpan> {
    let regions = extract_rocci_regions("snippet.rocci", source, document);
    let mut raw_tokens = Vec::new();
    collect_rocci_document(source, &mut raw_tokens, document);
    collect_embedded_regions(source, &mut raw_tokens, &regions);
    resolve_and_sort_spans(source, &raw_tokens)
}

pub fn highlight_rocdown(source: &str) -> Vec<HighlightSpan> {
    let sf = SourceFile::new("snippet.rocdown", source);
    let parsed = rocci_rocdown::parse(sf, false);
    highlight_rocdown_document(source, &parsed.document, &parsed.headings)
}

pub fn highlight_rocdown_document(
    source: &str,
    document: &rocci_rocdown::Document,
    headings: &[rocci_rocdown::HeadingInfo],
) -> Vec<HighlightSpan> {
    let regions = extract_rocdown_regions("snippet.rocdown", source, document, headings);
    let mut raw_tokens = Vec::new();
    collect_rocdown(source, &mut raw_tokens, document, headings);
    collect_embedded_regions(source, &mut raw_tokens, &regions);
    resolve_and_sort_spans(source, &raw_tokens)
}

pub fn collect_embedded_regions(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    regions: &RegionTree,
) {
    for region in &regions.regions {
        let region_start = floor_char_boundary(src, region.span.start as usize);
        let region_end = floor_char_boundary(src, (region.span.end as usize).min(src.len()));
        if region_start >= region_end {
            continue;
        }
        let slice = &src[region_start..region_end];

        match region.language {
            LanguageId::Roc => {
                let hl_tokens = embedded::roc::highlight(slice);
                for tok in hl_tokens {
                    let tok_start =
                        floor_char_boundary(src, region_start + tok.span.start as usize);
                    let tok_end = floor_char_boundary(
                        src,
                        (region_start + tok.span.end as usize).min(region_end),
                    );
                    if tok_start < tok_end {
                        collector.push(HighlightSpan::new(
                            Span::new(tok_start, tok_end),
                            tok.kind,
                            tok.modifiers,
                            tok.priority,
                        ));
                    }
                }
            }
            LanguageId::Css => {
                let hl_tokens = embedded::css::highlight(slice);
                for tok in hl_tokens {
                    let tok_start =
                        floor_char_boundary(src, region_start + tok.span.start as usize);
                    let tok_end = floor_char_boundary(
                        src,
                        (region_start + tok.span.end as usize).min(region_end),
                    );
                    if tok_start < tok_end {
                        collector.push(HighlightSpan::new(
                            Span::new(tok_start, tok_end),
                            tok.kind,
                            tok.modifiers,
                            tok.priority,
                        ));
                    }
                }
            }
            LanguageId::Html => {
                if region.purpose == RegionPurpose::DisplayOnly {
                    let hl_tokens = embedded::html::highlight(slice);
                    for tok in hl_tokens {
                        let tok_start =
                            floor_char_boundary(src, region_start + tok.span.start as usize);
                        let tok_end = floor_char_boundary(
                            src,
                            (region_start + tok.span.end as usize).min(region_end),
                        );
                        if tok_start < tok_end {
                            collector.push(HighlightSpan::new(
                                Span::new(tok_start, tok_end),
                                tok.kind,
                                tok.modifiers,
                                tok.priority,
                            ));
                        }
                    }
                }
            }
            LanguageId::Markdown
            | LanguageId::Rocci
            | LanguageId::Rocdown
            | LanguageId::Shell
            | LanguageId::Toml
            | LanguageId::PlainText
            | LanguageId::Other(_) => {}
        }
    }
}

pub fn collect_rocci_document(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    document: &RocciDocument,
) {
    for item in &document.items {
        match item {
            ModuleItem::Roc { .. } => {}
            ModuleItem::Component(component) => collect_component(src, collector, component),
            ModuleItem::Fixture(fixture) => collect_fixture(src, collector, fixture),
            ModuleItem::Css(css) => collect_css(src, collector, css),
            ModuleItem::Context(context) => collect_context(src, collector, context),
            ModuleItem::Init(init) => collect_init(src, collector, init),
            ModuleItem::On(on) => collect_on(src, collector, on),
        }
    }
}

pub fn collect_rocdown(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    document: &rocci_rocdown::Document,
    headings: &[rocci_rocdown::HeadingInfo],
) {
    for heading in headings {
        if let Some(span) = heading_marker(src, heading.span, heading.level) {
            collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
        }
    }
    for item in &document.items {
        match item {
            rocci_rocdown::Item::Markdown(md_node) => {
                let mut md_tokens = Vec::new();
                embedded::markdown::collect_markdown_node(src, md_node, &mut md_tokens);
                for tok in md_tokens {
                    collector.push(HighlightSpan::new(
                        tok.span,
                        tok.kind,
                        tok.modifiers,
                        tok.priority,
                    ));
                }
            }
            rocci_rocdown::Item::Page(page) => {
                collect_keyword(src, collector, page.span, page.body.start, "@page");
            }
            rocci_rocdown::Item::Roc(roc) => {
                collect_keyword(src, collector, roc.span, roc.body.start, "@roc");
            }
            rocci_rocdown::Item::Render(render) => {
                collect_keyword(src, collector, render.span, render.expr.start, "@render");
            }
            rocci_rocdown::Item::Component(component) => {
                collect_component(src, collector, component)
            }
            rocci_rocdown::Item::Fixture(fixture) => collect_fixture(src, collector, fixture),
            rocci_rocdown::Item::Css(css) => collect_css(src, collector, css),
            rocci_rocdown::Item::Context(context) => collect_context(src, collector, context),
            rocci_rocdown::Item::Init(init) => collect_init(src, collector, init),
            rocci_rocdown::Item::On(on) => collect_on(src, collector, on),
            rocci_rocdown::Item::Template(item) => {
                collect_items(src, collector, std::slice::from_ref(item))
            }
            rocci_rocdown::Item::Docs(docs) => {
                collect_docs(src, collector, docs);
            }
            rocci_rocdown::Item::Img(img) => {
                collect_img(src, collector, img);
            }
        }
    }
}

fn collect_img(src: &str, collector: &mut Vec<HighlightSpan>, img: &rocci_rocdown::ImgDecl) {
    collect_keyword(src, collector, img.span, img.body.start, "@img");
    let mut cur = rocci_template::Cursor::at(src, img.body.start as usize);
    let end = img.body.end as usize;
    while cur.pos < end && !cur.is_eof() {
        cur.skip_trivia();
        if cur.pos >= end {
            break;
        }
        if cur.peek() == Some(',') {
            cur.bump();
            continue;
        }
        let Some(name_span) = cur.scan_ident() else {
            break;
        };
        collector.push(HighlightSpan::new(
            name_span,
            HighlightKind::Property,
            0,
            50,
        ));
        cur.skip_trivia();
        if !cur.eat(':') {
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        if cur.peek() == Some('"') {
            cur.skip_string();
            let value_span = Span::new(value_start, cur.pos.min(end));
            collector.push(HighlightSpan::new(value_span, HighlightKind::String, 0, 50));
        }
    }
}

fn collect_docs(src: &str, collector: &mut Vec<HighlightSpan>, docs: &rocci_rocdown::DocsDecl) {
    collect_keyword(src, collector, docs.span, docs.kind_span.start, "@docs");
    collector.push(HighlightSpan::new(
        docs.kind_span,
        HighlightKind::Type,
        0,
        55,
    ));
    let (fields, content) = rocci_rocdown::split_docs_body(src, docs.body);
    for field in fields {
        collector.push(HighlightSpan::new(
            field.name_span,
            HighlightKind::Property,
            0,
            50,
        ));
        let val_str = field.value.of(src).trim();
        if val_str.starts_with('"') {
            collector.push(HighlightSpan::new(
                field.value,
                HighlightKind::String,
                0,
                50,
            ));
        } else if val_str == "true"
            || val_str == "false"
            || val_str == "Bool.true"
            || val_str == "Bool.false"
        {
            collector.push(HighlightSpan::new(
                field.value,
                HighlightKind::Keyword,
                0,
                50,
            ));
        }
    }
    if !content.is_empty() && (content.start as usize) < src.len() {
        let source = SourceFile::new("docs", src);
        let parsed = rocci_rocdown::parse_fragment(source, content, false);
        collect_rocdown(src, collector, &parsed.document, &parsed.headings);
    }
}

fn collect_keyword(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    span: Span,
    before: u32,
    word: &str,
) {
    if let Some(keyword) = ident_between(src, span.start, before, word) {
        collector.push(HighlightSpan::new(keyword, HighlightKind::Keyword, 0, 55));
    }
}

fn heading_marker(src: &str, span: Span, level: u8) -> Option<Span> {
    let text = span.of(src);
    let indent = text.len() - text.trim_start_matches([' ', '\t']).len();
    let trimmed = &text[indent..];
    let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hashes == 0 {
        return None;
    }
    let start = span.start as usize + indent;
    Some(Span::new(start, start + hashes.min(level as usize)))
}

fn collect_css(src: &str, collector: &mut Vec<HighlightSpan>, css: &CssDecl) {
    if let Some(span) = ident_between(src, css.span.start, css.body.start, "@css") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

fn collect_context(src: &str, collector: &mut Vec<HighlightSpan>, context: &ContextDecl) {
    if let Some(span) = ident_between(src, context.span.start, context.ty.start, "@context") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

fn collect_init(src: &str, collector: &mut Vec<HighlightSpan>, init: &InitDecl) {
    if let Some(span) = ident_between(src, init.span.start, init.body.start, "@init") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

fn collect_on(src: &str, collector: &mut Vec<HighlightSpan>, on: &OnDecl) {
    if let Some(span) = ident_between(src, on.span.start, on.method.span.start, "@on") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
    collector.push(HighlightSpan::new(
        on.method.span,
        HighlightKind::Keyword,
        0,
        50,
    ));
    collector.push(HighlightSpan::new(
        on.path_span,
        HighlightKind::String,
        0,
        50,
    ));
}

fn collect_component(src: &str, collector: &mut Vec<HighlightSpan>, component: &ComponentDecl) {
    collector.push(HighlightSpan::new(
        component.name.span,
        HighlightKind::Function,
        MOD_DECLARATION,
        55,
    ));
    if let Some(span) = ident_between(
        src,
        component.span.start,
        component.name.span.start,
        "@component",
    ) {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
    collect_items(src, collector, &component.body.items);
}

fn collect_fixture(src: &str, collector: &mut Vec<HighlightSpan>, fixture: &FixtureDecl) {
    collector.push(HighlightSpan::new(
        fixture.name.span,
        HighlightKind::Function,
        MOD_DECLARATION,
        55,
    ));
    if let Some(span) = ident_between(src, fixture.span.start, fixture.name.span.start, "@fixture")
    {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
    if let Some(span) = ident_between(src, fixture.span.start, fixture.name.span.start, "target") {
        collector.push(HighlightSpan::new(span, HighlightKind::Property, 0, 50));
    }
    collect_path(collector, &fixture.target.parts);
}

fn collect_items(src: &str, collector: &mut Vec<HighlightSpan>, items: &[TemplateItem]) {
    for item in items {
        match item {
            TemplateItem::Element(el) => collect_element(src, collector, el),
            TemplateItem::ComponentCall(call) => collect_call(src, collector, call),
            TemplateItem::Fragment(frag) => collect_items(src, collector, &frag.children),
            TemplateItem::Interpolation(_) => {}
            TemplateItem::If(dir) => {
                collector.push(HighlightSpan::new(
                    directive_keyword(dir.span, "if"),
                    HighlightKind::Keyword,
                    0,
                    55,
                ));
                collect_items(src, collector, &dir.then_body.items);
                for (cond, body) in &dir.else_ifs {
                    if let Some(span) = else_if_keyword(src, cond.start) {
                        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
                    }
                    collect_items(src, collector, &body.items);
                }
                if let Some(body) = &dir.else_body {
                    if let Some(span) = keyword_before(src, body.span.start, "@else") {
                        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
                    }
                    collect_items(src, collector, &body.items);
                }
            }
            TemplateItem::For(dir) => {
                collector.push(HighlightSpan::new(
                    directive_keyword(dir.span, "for"),
                    HighlightKind::Keyword,
                    0,
                    55,
                ));
                collector.push(HighlightSpan::new(
                    dir.binder.span,
                    HighlightKind::Parameter,
                    MOD_DECLARATION,
                    50,
                ));
                if let Some(span) =
                    ident_between(src, dir.binder.span.end, dir.collection.start, "in")
                {
                    collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
                }
                collect_items(src, collector, &dir.body.items);
            }
            TemplateItem::Match(dir) => {
                collector.push(HighlightSpan::new(
                    directive_keyword(dir.span, "match"),
                    HighlightKind::Keyword,
                    0,
                    55,
                ));
                for arm in &dir.arms {
                    if let Some(span) =
                        ident_between(src, arm.pattern.end, arm.value.span().start, "=>")
                    {
                        collector.push(HighlightSpan::new(span, HighlightKind::Operator, 0, 50));
                    }
                    collect_items(src, collector, std::slice::from_ref(&*arm.value));
                }
            }
            TemplateItem::Let(dir) => {
                collector.push(HighlightSpan::new(
                    directive_keyword(dir.span, "let"),
                    HighlightKind::Keyword,
                    0,
                    55,
                ));
                collector.push(HighlightSpan::new(
                    dir.binder.span,
                    HighlightKind::Parameter,
                    MOD_DECLARATION,
                    50,
                ));
            }
            TemplateItem::Css(css) => collect_css(src, collector, css),
            TemplateItem::Text(_) => {}
        }
    }
}

fn collect_element(src: &str, collector: &mut Vec<HighlightSpan>, el: &Element) {
    collector.push(HighlightSpan::new(
        el.name.span,
        HighlightKind::Tag,
        MOD_DEFAULT_LIBRARY,
        55,
    ));
    collect_attrs(src, collector, &el.attrs);
    collect_items(src, collector, &el.children);
    if !el.self_closing
        && let Some(span) = closing_name(src, el.span, &el.name.name)
    {
        collector.push(HighlightSpan::new(
            span,
            HighlightKind::Tag,
            MOD_DEFAULT_LIBRARY,
            55,
        ));
    }
}

fn collect_call(src: &str, collector: &mut Vec<HighlightSpan>, call: &ComponentCall) {
    collect_path(collector, &call.path.parts);
    collect_attrs(src, collector, &call.attrs);
    if let Some(children) = &call.children {
        collect_items(src, collector, children);
        let path = path_source(&call.path.parts);
        if let Some(span) = closing_name(src, call.span, &path) {
            collect_path_at(collector, span, &call.path.parts);
        }
    }
}

fn collect_path(collector: &mut Vec<HighlightSpan>, parts: &[rocci_template::Ident]) {
    for (i, part) in parts.iter().enumerate() {
        let kind = if i + 1 == parts.len() {
            HighlightKind::Function
        } else {
            HighlightKind::Namespace
        };
        collector.push(HighlightSpan::new(part.span, kind, 0, 50));
    }
}

fn collect_path_at(
    collector: &mut Vec<HighlightSpan>,
    span: Span,
    parts: &[rocci_template::Ident],
) {
    let mut start = span.start as usize;
    for (i, part) in parts.iter().enumerate() {
        let kind = if i + 1 == parts.len() {
            HighlightKind::Function
        } else {
            HighlightKind::Namespace
        };
        collector.push(HighlightSpan::new(
            Span::new(start, start + part.name.len()),
            kind,
            0,
            50,
        ));
        start += part.name.len() + 1;
    }
}

fn collect_attrs(_src: &str, collector: &mut Vec<HighlightSpan>, attrs: &[rocci_template::Attr]) {
    for attr in attrs {
        collector.push(HighlightSpan::new(
            attr.name.span,
            HighlightKind::Property,
            0,
            50,
        ));
        match &attr.value {
            AttrValue::Static { span, .. } => {
                collector.push(HighlightSpan::new(*span, HighlightKind::String, 0, 50))
            }
            AttrValue::Expr { .. } => {}
            AttrValue::Action { name, .. } => {
                let at_start = (name.span.start as usize).saturating_sub(1);
                collector.push(HighlightSpan::new(
                    Span::new(at_start, name.span.end as usize),
                    HighlightKind::Keyword,
                    0,
                    55,
                ));
            }
            AttrValue::Boolean => {}
        }
    }
}

fn path_source(parts: &[rocci_template::Ident]) -> String {
    parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn directive_keyword(span: Span, name: &str) -> Span {
    Span::new(span.start as usize, span.start as usize + 1 + name.len())
}

fn closing_name(src: &str, span: Span, name: &str) -> Option<Span> {
    let text = span.of(src);
    let needle = format!("</{name}");
    let idx = text.rfind(&needle)?;
    let start = span.start as usize + idx + 2;
    Some(Span::new(start, start + name.len()))
}

fn ident_between(src: &str, start: u32, end: u32, word: &str) -> Option<Span> {
    let from = start as usize;
    let to = (end as usize).min(src.len());
    if from >= to {
        return None;
    }
    let hay = &src[from..to];
    let mut search = 0;
    while let Some(rel) = hay[search..].find(word) {
        let i = search + rel;
        let before = if i == 0 {
            true
        } else {
            !is_ident_char(hay[..i].chars().next_back().unwrap())
        };
        let after_i = i + word.len();
        let after = hay[after_i..]
            .chars()
            .next()
            .is_none_or(|ch| !is_ident_char(ch));
        if before && after {
            return Some(Span::new(from + i, from + after_i));
        }
        search = i + 1;
    }
    None
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn else_if_keyword(src: &str, before: u32) -> Option<Span> {
    let mut i = skip_ws_back(src, before as usize);
    i = match_back(src, i, "if")?;
    i = skip_ws_back(src, i);
    i = match_back(src, i, "else")?;
    i = match_back(src, i, "@")?;
    Some(Span::new(i, before as usize))
}

fn keyword_before(src: &str, before: u32, keyword: &str) -> Option<Span> {
    let i = skip_ws_back(src, before as usize);
    let start = match_back(src, i, keyword)?;
    Some(Span::new(start, i))
}

fn skip_ws_back(src: &str, mut i: usize) -> usize {
    while i > 0 {
        let ch = src[..i].chars().next_back().unwrap();
        if !ch.is_whitespace() {
            break;
        }
        i -= ch.len_utf8();
    }
    i
}

fn match_back(src: &str, end: usize, word: &str) -> Option<usize> {
    let start = end.checked_sub(word.len())?;
    if src.get(start..end) == Some(word) {
        Some(start)
    } else {
        None
    }
}
