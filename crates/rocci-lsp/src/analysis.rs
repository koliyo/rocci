use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Diagnostic, DiagnosticSeverity,
    DocumentSymbol, DocumentSymbolResponse, GotoDefinitionResponse, Hover, HoverContents, Location,
    MarkupContent, MarkupKind, Position, Range, SymbolKind,
};
use rocci_template::{
    CompileOutput, ComponentCall, ComponentDecl, ContextDecl, Document, FixtureDecl, InitDecl,
    LowerOptions, ModuleItem, OnDecl, PositionEncoding, Severity, SourceFile, Span, TemplateItem,
    compile, component_matches,
};

#[rustfmt::skip]
const HTML_TAGS: &[&str] = &[
    "a", "article", "aside", "blockquote", "button", "code", "div", "em", "footer", "form",
    "h1", "h2", "h3", "h4", "h5", "h6", "header", "hr", "img", "input", "label", "li", "main",
    "nav", "ol", "option", "output", "p", "pre", "section", "select", "span", "strong",
    "table", "tbody", "td", "textarea", "th", "thead", "tr", "ul",
];

const DIRECTIVES: &[&str] = &[
    "if", "else", "else if", "for", "match", "let", "css", "context", "init", "on",
];

pub fn compile_text(name: &str, text: &str) -> CompileOutput {
    compile(SourceFile::new(name, text), &LowerOptions::default())
}

pub fn diagnostics(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    map_diagnostics(name, text, &compiled.diagnostics, encoding)
}

pub(crate) fn map_diagnostics(
    name: &str,
    text: &str,
    diagnostics: &[rocci_template::Diagnostic],
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    let source = SourceFile::new(name, text);
    diagnostics
        .iter()
        .map(|diagnostic| Diagnostic {
            range: lsp_range(source, diagnostic.span, encoding),
            severity: Some(match diagnostic.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
            }),
            code: None,
            code_description: None,
            source: Some("rocci".to_string()),
            message: diagnostic.message.clone(),
            tags: None,
            related_information: None,
            data: None,
        })
        .collect()
}

pub fn document_symbols(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    encoding: PositionEncoding,
) -> DocumentSymbolResponse {
    let source = SourceFile::new(name, text);
    let symbols = compiled
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Component(component) => Some(component_symbol(source, component, encoding)),
            ModuleItem::Fixture(fixture) => Some(fixture_symbol(source, fixture, encoding)),
            ModuleItem::Context(context) => Some(context_symbol(source, context, encoding)),
            ModuleItem::Init(init) => Some(init_symbol(source, init, encoding)),
            ModuleItem::On(on) => Some(on_symbol(source, on, encoding)),
            ModuleItem::Roc { .. } | ModuleItem::Css(_) => None,
        })
        .collect();
    DocumentSymbolResponse::Nested(symbols)
}

pub fn hover(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    offset: u32,
    encoding: PositionEncoding,
) -> Option<Hover> {
    let components: Vec<_> = components(&compiled.document).collect();
    hover_components(name, text, &components, offset, encoding)
}

pub(crate) fn hover_components(
    name: &str,
    text: &str,
    components: &[&ComponentDecl],
    offset: u32,
    encoding: PositionEncoding,
) -> Option<Hover> {
    let source = SourceFile::new(name, text);
    match hit_at(components, offset)? {
        Hit::ComponentName(component) => Some(Hover {
            contents: markdown(component_signature(source, component)),
            range: Some(lsp_range(source, component.name.span, encoding)),
        }),
        Hit::ComponentCall(call) => {
            let contents = match local_component(components, &call.path.roc_name) {
                Some(component) => component_signature(source, component),
                None => call.path.roc_name.clone(),
            };
            Some(Hover {
                contents: markdown(contents),
                range: Some(lsp_range(source, call.path.span, encoding)),
            })
        }
    }
}

pub fn goto_definition(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    offset: u32,
    encoding: PositionEncoding,
    uri: lsp_types::Uri,
) -> Option<GotoDefinitionResponse> {
    let components: Vec<_> = components(&compiled.document).collect();
    goto_definition_components(name, text, &components, offset, encoding, uri)
}

pub(crate) fn goto_definition_components(
    name: &str,
    text: &str,
    components: &[&ComponentDecl],
    offset: u32,
    encoding: PositionEncoding,
    uri: lsp_types::Uri,
) -> Option<GotoDefinitionResponse> {
    let source = SourceFile::new(name, text);
    let Hit::ComponentCall(call) = hit_at(components, offset)? else {
        return None;
    };
    let component = local_component(components, &call.path.roc_name)?;
    Some(GotoDefinitionResponse::Scalar(Location {
        uri,
        range: lsp_range(source, component.name.span, encoding),
    }))
}

pub fn completion(text: &str, compiled: &CompileOutput, offset: u32) -> CompletionResponse {
    let components: Vec<_> = components(&compiled.document).collect();
    completion_in_template(text, &components, offset)
}

pub(crate) fn completion_in_template(
    text: &str,
    components: &[&ComponentDecl],
    offset: u32,
) -> CompletionResponse {
    let offset = (offset as usize).min(text.len());
    match completion_context(text, offset) {
        CompletionContext::Directive { prefix } => CompletionResponse::Array(
            DIRECTIVES
                .iter()
                .filter(|label| label.starts_with(&prefix))
                .map(|label| completion_item(label, CompletionItemKind::KEYWORD, None))
                .collect(),
        ),
        CompletionContext::Tag { prefix } => {
            let mut items = local_component_tags(components, &prefix);
            if prefix.is_empty()
                || prefix
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_lowercase())
            {
                items.extend(html_tag_items(&prefix));
            }
            CompletionResponse::Array(items)
        }
        CompletionContext::Html { prefix } => CompletionResponse::Array(html_tag_items(&prefix)),
    }
}

pub(crate) fn component_symbol(
    source: SourceFile<'_>,
    component: &ComponentDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: component.name.name.clone(),
        detail: Some(component_signature(source, component)),
        kind: SymbolKind::FUNCTION,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, component.span, encoding),
        selection_range: lsp_range(source, component.name.span, encoding),
        children: None,
    }
}

pub(crate) fn fixture_symbol(
    source: SourceFile<'_>,
    fixture: &FixtureDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: fixture.name.name.clone(),
        detail: Some(format!(
            "@fixture {{target: {}}}",
            fixture.target.source_name()
        )),
        kind: SymbolKind::CONSTANT,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, fixture.span, encoding),
        selection_range: lsp_range(source, fixture.name.span, encoding),
        children: None,
    }
}

pub(crate) fn context_symbol(
    source: SourceFile<'_>,
    context: &ContextDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: "State".to_string(),
        detail: Some(format!("@context {}", source.slice(context.ty).trim())),
        kind: SymbolKind::STRUCT,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, context.span, encoding),
        selection_range: lsp_range(source, context.ty, encoding),
        children: None,
    }
}

pub(crate) fn init_symbol(
    source: SourceFile<'_>,
    init: &InitDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: "init!".to_string(),
        detail: Some("@init".to_string()),
        kind: SymbolKind::FUNCTION,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, init.span, encoding),
        selection_range: lsp_range(source, init.span, encoding),
        children: None,
    }
}

pub(crate) fn css_symbol(
    source: SourceFile<'_>,
    css: &rocci_template::CssDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    named_symbol(
        "@css",
        Some("@css".to_string()),
        SymbolKind::CONSTANT,
        source,
        css.span,
        css.span,
        encoding,
    )
}

pub(crate) fn named_symbol(
    name: &str,
    detail: Option<String>,
    kind: SymbolKind,
    source: SourceFile<'_>,
    range: Span,
    selection: Span,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: name.to_string(),
        detail,
        kind,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, range, encoding),
        selection_range: lsp_range(source, selection, encoding),
        children: None,
    }
}

pub(crate) fn on_symbol(
    source: SourceFile<'_>,
    on: &OnDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    DocumentSymbol {
        name: format!("{} {}", on.method.name.to_ascii_uppercase(), on.path),
        detail: Some(format!("@on:{}(\"{}\")", on.method.name, on.path)),
        kind: SymbolKind::FUNCTION,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, on.span, encoding),
        selection_range: lsp_range(source, on.method.span, encoding),
        children: None,
    }
}

fn component_signature(source: SourceFile<'_>, component: &ComponentDecl) -> String {
    format!(
        "@component {} = {}",
        component.name.name,
        source.slice(component.params).trim()
    )
}

fn local_component<'a>(
    components: &[&'a ComponentDecl],
    roc_name: &str,
) -> Option<&'a ComponentDecl> {
    components
        .iter()
        .copied()
        .find(|component| component_matches(&component.name.name, roc_name))
}

fn components(document: &Document) -> impl Iterator<Item = &ComponentDecl> {
    document.items.iter().filter_map(|item| match item {
        ModuleItem::Component(component) => Some(component),
        ModuleItem::Roc { .. }
        | ModuleItem::Fixture(_)
        | ModuleItem::Css(_)
        | ModuleItem::Context(_)
        | ModuleItem::Init(_)
        | ModuleItem::On(_) => None,
    })
}

fn local_component_tags(components: &[&ComponentDecl], prefix: &str) -> Vec<CompletionItem> {
    components
        .iter()
        .map(|component| component.name.name.clone())
        .filter(|label| label.starts_with(prefix))
        .map(|label| {
            completion_item(
                &label,
                CompletionItemKind::FUNCTION,
                Some(format!("@component {label}")),
            )
        })
        .collect()
}

fn html_tag_items(prefix: &str) -> Vec<CompletionItem> {
    HTML_TAGS
        .iter()
        .filter(|tag| tag.starts_with(prefix))
        .map(|tag| completion_item(tag, CompletionItemKind::PROPERTY, Some("HTML".to_string())))
        .collect()
}

pub(crate) fn completion_item(
    label: &str,
    kind: CompletionItemKind,
    detail: Option<String>,
) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(kind),
        detail,
        ..CompletionItem::default()
    }
}

fn markdown(value: String) -> HoverContents {
    HoverContents::Markup(MarkupContent {
        kind: MarkupKind::Markdown,
        value: format!("```rocci\n{value}\n```"),
    })
}

enum Hit<'a> {
    ComponentName(&'a ComponentDecl),
    ComponentCall(&'a ComponentCall),
}

fn hit_at<'a>(components: &[&'a ComponentDecl], offset: u32) -> Option<Hit<'a>> {
    let mut best: Option<(u32, Hit<'_>)> = None;
    for component in components {
        consider(
            &mut best,
            component.name.span,
            offset,
            Hit::ComponentName(component),
        );
        walk_items(&component.body.items, offset, &mut best);
    }
    best.map(|(_, hit)| hit)
}

fn walk_items<'a>(items: &'a [TemplateItem], offset: u32, best: &mut Option<(u32, Hit<'a>)>) {
    for item in items {
        match item {
            TemplateItem::Element(el) => walk_items(&el.children, offset, best),
            TemplateItem::ComponentCall(call) => {
                consider(best, call.path.span, offset, Hit::ComponentCall(call));
                if let Some(children) = &call.children {
                    walk_items(children, offset, best);
                }
            }
            TemplateItem::Fragment(frag) => walk_items(&frag.children, offset, best),
            TemplateItem::If(dir) => {
                walk_items(&dir.then_body.items, offset, best);
                for (_, body) in &dir.else_ifs {
                    walk_items(&body.items, offset, best);
                }
                if let Some(body) = &dir.else_body {
                    walk_items(&body.items, offset, best);
                }
            }
            TemplateItem::For(dir) => walk_items(&dir.body.items, offset, best),
            TemplateItem::Match(dir) => {
                for arm in &dir.arms {
                    walk_items(std::slice::from_ref(&*arm.value), offset, best);
                }
            }
            TemplateItem::Text(_)
            | TemplateItem::Interpolation(_)
            | TemplateItem::Let(_)
            | TemplateItem::Css(_) => {}
        }
    }
}

fn consider<'a>(best: &mut Option<(u32, Hit<'a>)>, span: Span, offset: u32, hit: Hit<'a>) {
    if !span.contains(offset) {
        return;
    }
    let len = span.len() as u32;
    if best.as_ref().is_none_or(|(best_len, _)| len <= *best_len) {
        *best = Some((len, hit));
    }
}

enum CompletionContext {
    Directive { prefix: String },
    Tag { prefix: String },
    Html { prefix: String },
}

fn completion_context(text: &str, offset: usize) -> CompletionContext {
    let line_start = text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &text[line_start..offset];

    if let Some(at) = line.rfind('@') {
        let after = &line[at + 1..];
        if is_directive_prefix(after) {
            return CompletionContext::Directive {
                prefix: after.to_string(),
            };
        }
    }

    if let Some(lt) = line.rfind('<') {
        let after = &line[lt + 1..];
        let tag = after.strip_prefix('/').unwrap_or(after);
        if is_tag_prefix(tag) {
            if tag.is_empty() || tag.chars().next().is_some_and(|ch| ch.is_ascii_uppercase()) {
                return CompletionContext::Tag {
                    prefix: tag.to_string(),
                };
            }
            return CompletionContext::Html {
                prefix: tag.to_string(),
            };
        }
    }

    CompletionContext::Html {
        prefix: ident_prefix(line).to_string(),
    }
}

fn is_directive_prefix(after: &str) -> bool {
    !after.contains('<')
        && after
            .chars()
            .all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
}

fn is_tag_prefix(tag: &str) -> bool {
    tag.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.')
}

fn ident_prefix(line: &str) -> &str {
    let start = line
        .char_indices()
        .rev()
        .find_map(|(i, ch)| {
            if ch.is_ascii_alphanumeric() || ch == '.' {
                None
            } else {
                Some(i + ch.len_utf8())
            }
        })
        .unwrap_or(0);
    &line[start..]
}

pub fn lsp_range(source: SourceFile<'_>, span: Span, encoding: PositionEncoding) -> Range {
    let ((start_line, start_col), (end_line, end_col)) = source.range(span, encoding);
    Range {
        start: Position::new(start_line, start_col),
        end: Position::new(end_line, end_col),
    }
}

pub fn offset_at(source: SourceFile<'_>, position: Position, encoding: PositionEncoding) -> u32 {
    source.offset_at(position.line, position.character, encoding)
}
