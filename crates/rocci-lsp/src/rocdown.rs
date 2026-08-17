use std::path::{Path, PathBuf};

use lsp_types::{
    CompletionItemKind, CompletionResponse, Diagnostic, DocumentSymbol, DocumentSymbolResponse,
    GotoDefinitionResponse, Hover, HoverContents, MarkupContent, MarkupKind, SymbolKind,
};
use rocci_rocdown::{
    CompileOptions, CompileOutput, DocsDecl, ImgDecl, Item, PageDecl, PageMeta, discovered_ids,
    index_pages_in_dir, page_ref_from_source,
};
use rocci_template::{ComponentDecl, PositionEncoding, SourceFile, Span, TemplateItem};

use crate::analysis::{
    completion_in_template, completion_item, component_symbol, context_symbol, css_symbol,
    fixture_symbol, goto_definition_components, hover_components, init_symbol, lsp_range,
    map_diagnostics, named_symbol, on_symbol,
};

const ROOT_DECLARATIONS: &[&str] = &[
    "page",
    "roc",
    "render",
    "component",
    "fixture",
    "css",
    "context",
    "init",
    "on",
    "docs",
    "img",
    "if",
    "for",
    "match",
    "let",
];

pub fn compile_text(name: &str, text: &str) -> CompileOutput {
    let mut options = CompileOptions::default();
    if let Some(path) = filesystem_path(name)
        && let Some(dir) = path
            .parent()
            .filter(|dir| dir.is_dir() && *dir != Path::new("/"))
    {
        let mut pages = index_pages_in_dir(dir);
        let current = page_ref_from_source(&path, text);
        pages.retain(|page| page.file_name != current.file_name);
        pages.push(current);
        options.pages = pages;
    }
    rocci_rocdown::compile(SourceFile::new(name, text), &options)
}

fn filesystem_path(name: &str) -> Option<PathBuf> {
    let path = if let Some(rest) = name.strip_prefix("file://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(name)
    };
    if path.extension().is_some_and(|ext| ext == "rocdown") {
        Some(path)
    } else {
        None
    }
}

pub fn diagnostics(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    encoding: PositionEncoding,
) -> Vec<Diagnostic> {
    map_diagnostics(name, text, &compiled.diagnostics, encoding)
}

pub fn document_symbols(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    encoding: PositionEncoding,
) -> DocumentSymbolResponse {
    let source = SourceFile::new(name, text);
    let mut symbols: Vec<(u32, DocumentSymbol)> = compiled
        .headings
        .iter()
        .map(|heading| {
            (
                heading.span.start,
                named_symbol(
                    &heading.text,
                    Some(format!("{{#{}}}", heading.id)),
                    SymbolKind::STRING,
                    source,
                    heading.span,
                    heading.span,
                    encoding,
                ),
            )
        })
        .collect();

    for item in &compiled.document.items {
        let symbol = match item {
            Item::Markdown(_) => continue,
            Item::Page(page) => page_symbol(source, page, &compiled.page_meta, encoding),
            Item::Roc(roc) => named_symbol(
                "@roc",
                Some("@roc".to_string()),
                SymbolKind::MODULE,
                source,
                roc.span,
                keyword_selection(text, roc.span, "@roc"),
                encoding,
            ),
            Item::Render(render) => named_symbol(
                "@render",
                Some("@render".to_string()),
                SymbolKind::FUNCTION,
                source,
                render.span,
                keyword_selection(text, render.span, "@render"),
                encoding,
            ),
            Item::Component(component) => component_symbol(source, component, encoding),
            Item::Fixture(fixture) => fixture_symbol(source, fixture, encoding),
            Item::Css(css) => css_symbol(source, css, encoding),
            Item::Context(context) => context_symbol(source, context, encoding),
            Item::Init(init) => init_symbol(source, init, encoding),
            Item::On(on) => on_symbol(source, on, encoding),
            Item::Template(item) => template_symbol(source, text, item, encoding),
            Item::Docs(docs) => docs_symbol(source, docs, encoding),
            Item::Img(img) => img_symbol(source, img, encoding),
        };
        symbols.push((item.span().start, symbol));
    }

    symbols.sort_by_key(|(start, _)| *start);
    DocumentSymbolResponse::Nested(symbols.into_iter().map(|(_, symbol)| symbol).collect())
}

pub fn hover(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    offset: u32,
    encoding: PositionEncoding,
) -> Option<Hover> {
    let components = components(compiled);
    let extra = template_items(compiled);
    if let Some(hover) = hover_components(name, text, &components, &extra, offset, encoding) {
        return Some(hover);
    }
    let source = SourceFile::new(name, text);
    if let Some(page) = compiled.document.items.iter().find_map(|item| match item {
        Item::Page(page) if page.span.contains(offset) => Some(page),
        _ => None,
    }) {
        return Some(page_hover(source, page, compiled, encoding));
    }
    if let Some(docs) = compiled.document.items.iter().find_map(|item| match item {
        Item::Docs(docs) if docs.span.contains(offset) => Some(docs),
        _ => None,
    }) {
        return Some(docs_hover(source, docs, encoding));
    }
    if let Some(img) = compiled.document.items.iter().find_map(|item| match item {
        Item::Img(img) if img.span.contains(offset) => Some(img),
        _ => None,
    }) {
        return Some(img_hover(source, img, encoding));
    }
    compiled.headings.iter().find_map(|heading| {
        if !heading.span.contains(offset) {
            return None;
        }
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "{} {}\n\n`{{#{}}}`",
                    "#".repeat(heading.level as usize),
                    heading.text,
                    heading.id
                ),
            }),
            range: Some(lsp_range(source, heading.span, encoding)),
        })
    })
}

pub fn goto_definition(
    name: &str,
    text: &str,
    compiled: &CompileOutput,
    offset: u32,
    encoding: PositionEncoding,
    uri: lsp_types::Uri,
) -> Option<GotoDefinitionResponse> {
    let components = components(compiled);
    let extra = template_items(compiled);
    goto_definition_components(name, text, &components, &extra, offset, encoding, uri)
}

pub fn completion(text: &str, compiled: &CompileOutput, offset: u32) -> CompletionResponse {
    let offset = (offset as usize).min(text.len());
    if component_at(compiled, offset as u32).is_some() || template_at(compiled, offset as u32) {
        let components = components(compiled);
        return completion_in_template(text, &components, offset as u32);
    }
    if let Some(page) = compiled.document.items.iter().find_map(|item| match item {
        Item::Page(page) if page.body.contains(offset as u32) => Some(page),
        _ => None,
    }) {
        return page_completion(text, page, offset, compiled);
    }
    if let Some(docs) = compiled.document.items.iter().find_map(|item| match item {
        Item::Docs(docs) if docs.span.contains(offset as u32) => Some(docs),
        _ => None,
    }) {
        return docs_completion(text, docs, offset);
    }
    if let Some(img) = compiled.document.items.iter().find_map(|item| match item {
        Item::Img(img) if img.span.contains(offset as u32) => Some(img),
        _ => None,
    }) {
        return img_completion(text, img, offset);
    }
    if let Some(prefix) = root_declaration_prefix(text, offset) {
        return CompletionResponse::Array(
            ROOT_DECLARATIONS
                .iter()
                .filter(|label| label.starts_with(&prefix))
                .map(|label| {
                    completion_item(
                        label,
                        CompletionItemKind::KEYWORD,
                        Some(format!("@{label}")),
                    )
                })
                .collect(),
        );
    }
    CompletionResponse::Array(Vec::new())
}

fn page_hover(
    source: SourceFile<'_>,
    page: &PageDecl,
    compiled: &CompileOutput,
    encoding: PositionEncoding,
) -> Hover {
    let mut value = String::from("```rocdown\n@page\n```\n");
    if let Some(theme) = &compiled.theme {
        value.push_str(&format!(
            "\n**theme** `{}` ({})\n\n- color-scheme: `{}`\n{}\n",
            theme.id,
            match theme.origin {
                rocci_rocdown::ThemeOrigin::Builtin => "builtin",
                rocci_rocdown::ThemeOrigin::Local => "local",
                rocci_rocdown::ThemeOrigin::None => "none",
            },
            theme.policy,
            theme
                .path
                .as_ref()
                .map(|path| format!("- path: `{}`\n", path.display()))
                .unwrap_or_default(),
        ));
    }
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(lsp_range(source, page.span, encoding)),
    }
}

fn page_completion(
    text: &str,
    page: &PageDecl,
    offset: usize,
    compiled: &CompileOutput,
) -> CompletionResponse {
    let body = page.body.of(text);
    let rel = offset
        .saturating_sub(page.body.start as usize)
        .min(body.len());
    let prefix = field_prefix(&body[..rel]);
    if let Some(value) = after_field(&body[..rel], "theme") {
        let items = discovered_ids()
            .into_iter()
            .filter(|id| id.starts_with(&value))
            .map(|id| completion_item(&id, CompletionItemKind::ENUM_MEMBER, Some("theme".into())))
            .collect();
        return CompletionResponse::Array(items);
    }
    if let Some(value) = after_field(&body[..rel], "color_scheme") {
        let items = ["auto", "light", "dark"]
            .into_iter()
            .filter(|id| id.starts_with(&value))
            .map(|id| {
                completion_item(
                    id,
                    CompletionItemKind::ENUM_MEMBER,
                    Some("color_scheme".into()),
                )
            })
            .collect();
        return CompletionResponse::Array(items);
    }
    let fields = [
        "id",
        "route",
        "aliases",
        "layout",
        "draft",
        "meta",
        "theme",
        "color_scheme",
    ];
    let items = fields
        .into_iter()
        .filter(|field| field.starts_with(&prefix))
        .map(|field| completion_item(field, CompletionItemKind::FIELD, Some("@page".into())))
        .collect();
    let _ = compiled;
    CompletionResponse::Array(items)
}

fn field_prefix(before: &str) -> String {
    let trimmed = before.trim_end();
    let ident_start = trimmed
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map(|i| i + 1)
        .unwrap_or(0);
    trimmed[ident_start..].to_string()
}

fn after_field(before: &str, field: &str) -> Option<String> {
    let trimmed = before.trim_end();
    let needle = format!("{field}:");
    let idx = trimmed.rfind(&needle)?;
    let rest = trimmed[idx + needle.len()..].trim_start();
    if rest.contains(',') || rest.contains('\n') {
        return None;
    }
    Some(rest.trim_matches('"').to_string())
}

fn page_symbol(
    source: SourceFile<'_>,
    page: &PageDecl,
    meta: &PageMeta,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    let mut detail = String::from("@page");
    if let Some(route) = &meta.route {
        detail.push(' ');
        detail.push_str(route);
    }
    if let Some(title) = &meta.title {
        if meta.route.is_some() {
            detail.push_str(" / ");
        } else {
            detail.push(' ');
        }
        detail.push_str(title);
    }
    if let Some(theme) = &meta.theme {
        detail.push_str(" · ");
        detail.push_str(theme);
    }
    named_symbol(
        "@page",
        Some(detail),
        SymbolKind::MODULE,
        source,
        page.span,
        keyword_selection(source.src, page.span, "@page"),
        encoding,
    )
}

fn components(compiled: &CompileOutput) -> Vec<&ComponentDecl> {
    compiled
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Component(component) => Some(component),
            _ => None,
        })
        .collect()
}

fn component_at(compiled: &CompileOutput, offset: u32) -> Option<&ComponentDecl> {
    compiled.document.items.iter().find_map(|item| match item {
        Item::Component(component) if component.span.contains(offset) => Some(component),
        _ => None,
    })
}

fn template_items(compiled: &CompileOutput) -> Vec<&TemplateItem> {
    compiled
        .document
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Template(item) => Some(item),
            _ => None,
        })
        .collect()
}

fn template_at(compiled: &CompileOutput, offset: u32) -> bool {
    compiled.document.items.iter().any(|item| match item {
        Item::Template(template) => template.span().contains(offset),
        _ => false,
    })
}

fn template_symbol(
    source: SourceFile<'_>,
    text: &str,
    item: &TemplateItem,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    let (name, kind) = match item {
        TemplateItem::If(_) => ("@if", SymbolKind::BOOLEAN),
        TemplateItem::For(_) => ("@for", SymbolKind::ARRAY),
        TemplateItem::Match(_) => ("@match", SymbolKind::ENUM),
        TemplateItem::Let(dir) => {
            return named_symbol(
                &dir.binder.name,
                Some("@let".to_string()),
                SymbolKind::VARIABLE,
                source,
                item.span(),
                dir.binder.span,
                encoding,
            );
        }
        _ => ("@template", SymbolKind::KEY),
    };
    named_symbol(
        name,
        Some(name.to_string()),
        kind,
        source,
        item.span(),
        keyword_selection(text, item.span(), name),
        encoding,
    )
}

fn root_declaration_prefix(text: &str, offset: usize) -> Option<String> {
    let line_start = text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = &text[line_start..offset];
    let indent = line.len() - line.trim_start_matches([' ', '\t']).len();
    let after_indent = &line[indent..];
    let prefix = after_indent.strip_prefix('@')?;
    if prefix
        .chars()
        .all(|ch| ch.is_ascii_alphabetic() || ch == ' ')
    {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn keyword_selection(src: &str, span: Span, keyword: &str) -> Span {
    let text = span.of(src);
    let indent = text.len() - text.trim_start_matches([' ', '\t']).len();
    let start = span.start as usize + indent;
    if src.get(start..start + keyword.len()) == Some(keyword) {
        Span::new(start, start + keyword.len())
    } else {
        span
    }
}

fn docs_hover(source: SourceFile<'_>, docs: &DocsDecl, encoding: PositionEncoding) -> Hover {
    let (fields, _) = rocci_rocdown::split_docs_body(source.src, docs.body);
    let mut value = format!("```rocdown\n@docs {}\n```\n", docs.kind);
    for field in fields {
        value.push_str(&format!(
            "\n- **{}**: `{}`",
            field.name,
            source.slice(field.value).trim()
        ));
    }
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        }),
        range: Some(lsp_range(source, docs.span, encoding)),
    }
}

fn docs_completion(text: &str, docs: &DocsDecl, offset: usize) -> CompletionResponse {
    let body = docs.body.of(text);
    let rel = offset
        .saturating_sub(docs.body.start as usize)
        .min(body.len());
    let prefix = field_prefix(&body[..rel]);
    if docs.kind == "tabs"
        && let Some(value) = after_field(&body[..rel], "kind")
    {
        let items = ["language", "platform", "tool"]
            .into_iter()
            .filter(|id| id.starts_with(&value))
            .map(|id| completion_item(id, CompletionItemKind::ENUM_MEMBER, Some("kind".into())))
            .collect();
        return CompletionResponse::Array(items);
    }
    if let Some(value) = after_field(&body[..rel], "open") {
        let items = ["true", "false", "Bool.true", "Bool.false"]
            .into_iter()
            .filter(|id| id.starts_with(&value))
            .map(|id| completion_item(id, CompletionItemKind::ENUM_MEMBER, Some("open".into())))
            .collect();
        return CompletionResponse::Array(items);
    }
    let fields: &[&str] = match docs.kind.as_str() {
        "include" => &["path", "region", "language", "start_line", "end_line"],
        "tabs" => &["group", "kind"],
        "tab" => &["id", "label"],
        "link-card" => &["page", "href", "title", "description"],
        "details" => &["summary", "open"],
        "figure" => &["caption", "credit"],
        "definition" => &["term"],
        "badge" => &["label", "tone"],
        "example" => &["name", "command", "language"],
        _ => &["title", "summary", "open", "label"],
    };
    let items = fields
        .iter()
        .filter(|field| field.starts_with(&prefix))
        .map(|field| {
            completion_item(
                field,
                CompletionItemKind::FIELD,
                Some(format!("@docs {}", docs.kind)),
            )
        })
        .collect();
    CompletionResponse::Array(items)
}

fn docs_symbol(
    source: SourceFile<'_>,
    docs: &DocsDecl,
    encoding: PositionEncoding,
) -> DocumentSymbol {
    let (fields, content) = rocci_rocdown::split_docs_body(source.src, docs.body);
    let title = fields
        .iter()
        .find(|field| field.name == "title" || field.name == "label" || field.name == "summary")
        .and_then(|field| rocci_rocdown::field_string(source.src, field));
    let mut detail = format!("@docs {}", docs.kind);
    if let Some(title) = title {
        detail.push_str(" · ");
        detail.push_str(&title);
    }
    let mut children = Vec::new();
    if !content.is_empty() && (content.start as usize) < source.src.len() {
        let parsed = rocci_rocdown::parse_fragment(source, content, false);
        for heading in &parsed.headings {
            children.push((
                heading.span.start,
                named_symbol(
                    &heading.text,
                    Some(format!("{{#{}}}", heading.id)),
                    SymbolKind::STRING,
                    source,
                    heading.span,
                    heading.span,
                    encoding,
                ),
            ));
        }
        for item in &parsed.document.items {
            if let Item::Docs(nested) = item {
                children.push((nested.span.start, docs_symbol(source, nested, encoding)));
            }
        }
    }
    children.sort_by_key(|(start, _)| *start);
    let children = if children.is_empty() {
        None
    } else {
        Some(children.into_iter().map(|(_, sym)| sym).collect())
    };

    DocumentSymbol {
        name: format!("@docs {}", docs.kind),
        detail: Some(detail),
        kind: SymbolKind::STRUCT,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
        range: lsp_range(source, docs.span, encoding),
        selection_range: lsp_range(source, docs.kind_span, encoding),
        children,
    }
}

fn img_symbol(source: SourceFile<'_>, img: &ImgDecl, encoding: PositionEncoding) -> DocumentSymbol {
    let mut diags = Vec::new();
    let fields = rocci_rocdown::extract_img_fields(source.src, img.body, &mut diags);
    let detail = fields.src.as_ref().map(|(s, _)| s.clone());
    named_symbol(
        "@img",
        detail,
        SymbolKind::OBJECT,
        source,
        img.span,
        img.span,
        encoding,
    )
}

fn img_hover(source: SourceFile<'_>, img: &ImgDecl, encoding: PositionEncoding) -> Hover {
    let mut diags = Vec::new();
    let fields = rocci_rocdown::extract_img_fields(source.src, img.body, &mut diags);
    let mut doc = String::from("```rocdown\n@img\n```\n\nNative Rocdown image element.\n");
    if let Some((src, _)) = &fields.src {
        doc.push_str(&format!("\n- **src**: `{src}`"));
    }
    if let Some((alt, _)) = &fields.alt {
        doc.push_str(&format!("\n- **alt**: `{alt}`"));
    }
    if fields.decorative.as_ref().is_some_and(|(value, _)| *value) {
        doc.push_str("\n- **decorative**: `Bool.true`");
    }
    if let Some((width, _)) = &fields.width {
        doc.push_str(&format!("\n- **width**: `{width}`"));
    }
    if let Some((height, _)) = &fields.height {
        doc.push_str(&format!("\n- **height**: `{height}`"));
    }
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc,
        }),
        range: Some(lsp_range(source, img.span, encoding)),
    }
}

const IMG_FIELDS: &[&str] = &[
    "src",
    "alt",
    "title",
    "width",
    "height",
    "class",
    "loading",
    "decoding",
    "decorative",
];

fn img_completion(_text: &str, _img: &ImgDecl, _offset: usize) -> CompletionResponse {
    CompletionResponse::Array(
        IMG_FIELDS
            .iter()
            .map(|name| {
                let insert = if *name == "decorative" {
                    format!("{name}: Bool.true")
                } else {
                    format!("{name}: \"\"")
                };
                completion_item(*name, CompletionItemKind::PROPERTY, Some(insert))
            })
            .collect(),
    )
}
