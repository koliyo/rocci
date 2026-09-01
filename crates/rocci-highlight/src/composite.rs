use rocci_template::{
    AttrValue, CommandDecl, ComponentCall, ComponentDecl, ContextDecl, CssDecl,
    Document as RocciDocument, Element, FixtureDecl, FragmentDecl, InitDecl, LeadingComments,
    LiveDecl, ModuleItem, RouteDecl, SourceFile, Span, TemplateItem, TestDecl, ViewDecl,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::embedded;
use crate::language::LanguageId;
use crate::regions::{RegionPurpose, RegionTree, extract_rocci_regions};
use crate::token::{
    HighlightKind, HighlightSpan, MOD_DECLARATION, MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION,
    floor_char_boundary, resolve_and_sort_spans,
};

pub fn highlight(language: LanguageId, source: &str) -> Vec<HighlightSpan> {
    match language {
        #[cfg(not(target_arch = "wasm32"))]
        LanguageId::Roc => {
            let raw = embedded::roc::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        #[cfg(not(target_arch = "wasm32"))]
        LanguageId::Css => {
            let raw = embedded::css::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        #[cfg(not(target_arch = "wasm32"))]
        LanguageId::Html => {
            let raw = embedded::html::highlight(source);
            resolve_and_sort_spans(source, &raw)
        }
        #[cfg(target_arch = "wasm32")]
        LanguageId::Roc => {
            let raw = crate::lex::highlight_roc(source);
            resolve_and_sort_spans(source, &raw)
        }
        #[cfg(target_arch = "wasm32")]
        LanguageId::Css => {
            let raw = crate::lex::highlight_css(source);
            resolve_and_sort_spans(source, &raw)
        }
        #[cfg(target_arch = "wasm32")]
        LanguageId::Html => {
            let raw = crate::lex::highlight_html(source);
            resolve_and_sort_spans(source, &raw)
        }
        LanguageId::Rocci => highlight_rocci(source),
        LanguageId::Markdown => {
            let raw = crate::markdown::highlight_markdown(source);
            resolve_and_sort_spans(source, &raw)
        }
        LanguageId::Rocdown
        | LanguageId::Shell
        | LanguageId::Toml
        | LanguageId::PlainText
        | LanguageId::Other(_) => Vec::new(),
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

fn highlight_embedded(language: LanguageId, slice: &str) -> Vec<HighlightSpan> {
    match language {
        LanguageId::Roc => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                embedded::roc::highlight(slice)
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::lex::highlight_roc(slice)
            }
        }
        LanguageId::Css => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                embedded::css::highlight(slice)
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::lex::highlight_css(slice)
            }
        }
        LanguageId::Html => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                embedded::html::highlight(slice)
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::lex::highlight_html(slice)
            }
        }
        LanguageId::Rocci => highlight_rocci(slice),
        LanguageId::Markdown => crate::markdown::highlight_markdown(slice),
        _ => Vec::new(),
    }
}

fn push_offset_tokens(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    region_start: usize,
    region_end: usize,
    tokens: Vec<HighlightSpan>,
) {
    for tok in tokens {
        let tok_start = floor_char_boundary(src, region_start + tok.span.start as usize);
        let tok_end =
            floor_char_boundary(src, (region_start + tok.span.end as usize).min(region_end));
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
                push_offset_tokens(
                    src,
                    collector,
                    region_start,
                    region_end,
                    highlight_embedded(LanguageId::Roc, slice),
                );
            }
            LanguageId::Css => {
                push_offset_tokens(
                    src,
                    collector,
                    region_start,
                    region_end,
                    highlight_embedded(LanguageId::Css, slice),
                );
            }
            LanguageId::Html => {
                if region.purpose == RegionPurpose::DisplayOnly {
                    push_offset_tokens(
                        src,
                        collector,
                        region_start,
                        region_end,
                        highlight_embedded(LanguageId::Html, slice),
                    );
                }
            }
            LanguageId::Rocci | LanguageId::Markdown => {
                if region.purpose == RegionPurpose::DisplayOnly {
                    push_offset_tokens(
                        src,
                        collector,
                        region_start,
                        region_end,
                        highlight_embedded(region.language.clone(), slice),
                    );
                }
            }
            LanguageId::Rocdown
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
            ModuleItem::Test(test) => collect_test(src, collector, test),
            ModuleItem::Css(css) => collect_css(src, collector, css),
            ModuleItem::Context(context) => collect_context(src, collector, context),
            ModuleItem::Init(init) => collect_init(src, collector, init),
            ModuleItem::Route(route) => match route {
                RouteDecl::Live(live) => collect_live(src, collector, live),
                RouteDecl::View(view) => collect_view(src, collector, view),
                RouteDecl::Fragment(fragment) => collect_fragment(src, collector, fragment),
                RouteDecl::Command(command) => collect_command(src, collector, command),
            },
        }
    }
}
pub fn collect_keyword(
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

pub fn heading_marker(src: &str, span: Span, level: u8) -> Option<Span> {
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

pub fn collect_leading(collector: &mut Vec<HighlightSpan>, leading: &Option<LeadingComments>) {
    let Some(leading) = leading else {
        return;
    };
    for span in &leading.comments {
        collector.push(HighlightSpan::new(*span, HighlightKind::Comment, 0, 40));
    }
    for span in &leading.docs {
        collector.push(HighlightSpan::new(
            *span,
            HighlightKind::Comment,
            MOD_DOCUMENTATION,
            40,
        ));
    }
}

pub fn collect_css(src: &str, collector: &mut Vec<HighlightSpan>, css: &CssDecl) {
    collect_leading(collector, &css.leading);
    if let Some(span) = ident_between(src, css.span.start, css.body.start, "@css") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

pub fn collect_context(src: &str, collector: &mut Vec<HighlightSpan>, context: &ContextDecl) {
    collect_leading(collector, &context.leading);
    if let Some(span) = ident_between(src, context.span.start, context.ty.start, "@context") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

pub fn collect_init(src: &str, collector: &mut Vec<HighlightSpan>, init: &InitDecl) {
    collect_leading(collector, &init.leading);
    if let Some(span) = ident_between(src, init.span.start, init.body.start, "@init") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
}

pub fn collect_live(src: &str, collector: &mut Vec<HighlightSpan>, live: &LiveDecl) {
    collect_leading(collector, &live.leading);
    collect_route_header(src, collector, &live.method, "live", live.path_span);
}

pub fn collect_view(src: &str, collector: &mut Vec<HighlightSpan>, view: &ViewDecl) {
    collect_leading(collector, &view.leading);
    collect_route_header(src, collector, &view.method, "view", view.path_span);
}

pub fn collect_fragment(src: &str, collector: &mut Vec<HighlightSpan>, fragment: &FragmentDecl) {
    collect_leading(collector, &fragment.leading);
    collect_route_header(
        src,
        collector,
        &fragment.method,
        "fragment",
        fragment.path_span,
    );
}

pub fn collect_command(src: &str, collector: &mut Vec<HighlightSpan>, command: &CommandDecl) {
    collect_leading(collector, &command.leading);
    collect_route_header(
        src,
        collector,
        &command.method,
        "command",
        command.path_span,
    );
}

fn collect_route_header(
    src: &str,
    collector: &mut Vec<HighlightSpan>,
    method: &rocci_template::Ident,
    role: &str,
    path_span: Span,
) {
    let at = method.span.start.saturating_sub(1);
    let method_span = if src.as_bytes().get(at as usize) == Some(&b'@') {
        Span::new(at as usize, method.span.end as usize)
    } else {
        method.span
    };
    collector.push(HighlightSpan::new(
        method_span,
        HighlightKind::Keyword,
        0,
        55,
    ));
    if let Some(role_span) = ident_between(src, method.span.end, path_span.start, role) {
        collector.push(HighlightSpan::new(
            role_span,
            HighlightKind::EnumMember,
            0,
            50,
        ));
    }
    collector.push(HighlightSpan::new(path_span, HighlightKind::String, 0, 50));
}

pub fn collect_component(src: &str, collector: &mut Vec<HighlightSpan>, component: &ComponentDecl) {
    collect_leading(collector, &component.leading);
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

pub fn collect_fixture(src: &str, collector: &mut Vec<HighlightSpan>, fixture: &FixtureDecl) {
    collect_leading(collector, &fixture.leading);
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

pub fn collect_test(src: &str, collector: &mut Vec<HighlightSpan>, test: &TestDecl) {
    collect_leading(collector, &test.leading);
    collector.push(HighlightSpan::new(
        test.name.span,
        HighlightKind::Function,
        MOD_DECLARATION,
        55,
    ));
    if let Some(span) = ident_between(src, test.span.start, test.name.span.start, "@test") {
        collector.push(HighlightSpan::new(span, HighlightKind::Keyword, 0, 55));
    }
    if let Some(span) = ident_between(src, test.span.start, test.name.span.start, "fixture") {
        collector.push(HighlightSpan::new(span, HighlightKind::Property, 0, 50));
    }
    if let Some(fixture) = &test.fixture {
        collector.push(HighlightSpan::new(
            fixture.span,
            HighlightKind::Variable,
            0,
            50,
        ));
    }
}

pub fn collect_items(src: &str, collector: &mut Vec<HighlightSpan>, items: &[TemplateItem]) {
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

pub fn collect_element(src: &str, collector: &mut Vec<HighlightSpan>, el: &Element) {
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

pub fn collect_call(src: &str, collector: &mut Vec<HighlightSpan>, call: &ComponentCall) {
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

pub fn collect_path(collector: &mut Vec<HighlightSpan>, parts: &[rocci_template::Ident]) {
    for (i, part) in parts.iter().enumerate() {
        let kind = if i + 1 == parts.len() {
            HighlightKind::Function
        } else {
            HighlightKind::Namespace
        };
        collector.push(HighlightSpan::new(part.span, kind, 0, 50));
    }
}

pub fn collect_path_at(
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

pub fn collect_attrs(
    _src: &str,
    collector: &mut Vec<HighlightSpan>,
    attrs: &[rocci_template::Attr],
) {
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

pub fn path_source(parts: &[rocci_template::Ident]) -> String {
    parts
        .iter()
        .map(|part| part.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

pub fn directive_keyword(span: Span, name: &str) -> Span {
    Span::new(span.start as usize, span.start as usize + 1 + name.len())
}

pub fn closing_name(src: &str, span: Span, name: &str) -> Option<Span> {
    let text = span.of(src);
    let needle = format!("</{name}");
    let idx = text.rfind(&needle)?;
    let start = span.start as usize + idx + 2;
    Some(Span::new(start, start + name.len()))
}

pub fn ident_between(src: &str, start: u32, end: u32, word: &str) -> Option<Span> {
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

pub fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

pub fn else_if_keyword(src: &str, before: u32) -> Option<Span> {
    let mut i = skip_ws_back(src, before as usize);
    i = match_back(src, i, "if")?;
    i = skip_ws_back(src, i);
    i = match_back(src, i, "else")?;
    i = match_back(src, i, "@")?;
    Some(Span::new(i, before as usize))
}

pub fn keyword_before(src: &str, before: u32, keyword: &str) -> Option<Span> {
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
