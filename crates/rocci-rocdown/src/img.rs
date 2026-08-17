use std::path::{Path, PathBuf};

use rocci_template::{Cursor, Diagnostic, Span};

use crate::ast::{Document, Item, MdNode};
use crate::docs::split_docs_body;
use crate::page::{bool_literal, skip_value, string_literal};
use crate::parse::parse_fragment;
use crate::{CompileOptions, SourceFile};

const ALLOWED_FIELDS: &[&str] = &[
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

const LOADING_VALUES: &[&str] = &["lazy", "eager"];
const DECODING_VALUES: &[&str] = &["async", "auto", "sync"];

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ImgFields {
    pub src: Option<(String, Span)>,
    pub alt: Option<(String, Span)>,
    pub title: Option<(String, Span)>,
    pub width: Option<(String, Span)>,
    pub height: Option<(String, Span)>,
    pub class: Option<(String, Span)>,
    pub loading: Option<(String, Span)>,
    pub decoding: Option<(String, Span)>,
    pub decorative: Option<(bool, Span)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticImage {
    pub src: String,
    pub alt: String,
    pub title: Option<String>,
    pub width: Option<String>,
    pub height: Option<String>,
    pub class: Option<String>,
    pub loading: Option<String>,
    pub decoding: Option<String>,
    pub span: Span,
    src_span: Span,
    alt_span: Span,
    title_span: Option<Span>,
    width_span: Option<Span>,
    height_span: Option<Span>,
    class_span: Option<Span>,
    loading_span: Option<Span>,
    decoding_span: Option<Span>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImgHtmlAttr {
    pub name: &'static str,
    pub value: String,
    pub span: Span,
}

impl StaticImage {
    pub fn from_fields(fields: &ImgFields, span: Span) -> Self {
        let decorative = fields
            .decorative
            .as_ref()
            .map(|(value, _)| *value)
            .unwrap_or(false);
        let (alt, alt_span) = if decorative {
            (
                String::new(),
                fields
                    .decorative
                    .as_ref()
                    .map(|(_, span)| *span)
                    .unwrap_or(span),
            )
        } else {
            fields.alt.clone().unwrap_or_else(|| (String::new(), span))
        };
        Self {
            src: fields
                .src
                .as_ref()
                .map(|(value, _)| value.clone())
                .unwrap_or_default(),
            alt,
            title: fields.title.as_ref().map(|(value, _)| value.clone()),
            width: fields.width.as_ref().map(|(value, _)| value.clone()),
            height: fields.height.as_ref().map(|(value, _)| value.clone()),
            class: fields.class.as_ref().map(|(value, _)| value.clone()),
            loading: fields.loading.as_ref().map(|(value, _)| value.clone()),
            decoding: fields.decoding.as_ref().map(|(value, _)| value.clone()),
            span,
            src_span: fields.src.as_ref().map(|(_, span)| *span).unwrap_or(span),
            alt_span,
            title_span: fields.title.as_ref().map(|(_, span)| *span),
            width_span: fields.width.as_ref().map(|(_, span)| *span),
            height_span: fields.height.as_ref().map(|(_, span)| *span),
            class_span: fields.class.as_ref().map(|(_, span)| *span),
            loading_span: fields.loading.as_ref().map(|(_, span)| *span),
            decoding_span: fields.decoding.as_ref().map(|(_, span)| *span),
        }
    }

    pub fn class_value(&self) -> String {
        match &self.class {
            Some(custom) => format!("rd-image {custom}"),
            None => "rd-image".to_string(),
        }
    }

    pub fn html_attrs(&self) -> Vec<ImgHtmlAttr> {
        let mut attrs = vec![
            ImgHtmlAttr {
                name: "class",
                value: self.class_value(),
                span: self.class_span.unwrap_or(self.span),
            },
            ImgHtmlAttr {
                name: "src",
                value: self.src.clone(),
                span: self.src_span,
            },
            ImgHtmlAttr {
                name: "alt",
                value: self.alt.clone(),
                span: self.alt_span,
            },
        ];
        push_opt(&mut attrs, "title", self.title.as_ref(), self.title_span);
        push_opt(&mut attrs, "width", self.width.as_ref(), self.width_span);
        push_opt(&mut attrs, "height", self.height.as_ref(), self.height_span);
        push_opt(
            &mut attrs,
            "loading",
            self.loading.as_ref(),
            self.loading_span,
        );
        push_opt(
            &mut attrs,
            "decoding",
            self.decoding.as_ref(),
            self.decoding_span,
        );
        attrs
    }
}

fn push_opt(
    attrs: &mut Vec<ImgHtmlAttr>,
    name: &'static str,
    value: Option<&String>,
    span: Option<Span>,
) {
    if let (Some(value), Some(span)) = (value, span) {
        attrs.push(ImgHtmlAttr {
            name,
            value: value.clone(),
            span,
        });
    }
}

pub fn extract_img_fields(src: &str, body: Span, diagnostics: &mut Vec<Diagnostic>) -> ImgFields {
    let mut fields = ImgFields::default();
    let mut cur = Cursor::at(src, body.start as usize);
    let end = body.end as usize;

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
            diagnostics.push(Diagnostic::error(
                Span::point(cur.pos),
                "expected a field name in `@img`",
            ));
            break;
        };
        let name = cur.ident_text(name_span).to_string();
        cur.skip_trivia();
        if !cur.eat(':') {
            diagnostics.push(Diagnostic::error(
                name_span,
                format!("expected `:` after `@img` field `{name}`"),
            ));
            break;
        }
        cur.skip_trivia();
        let value_start = cur.pos;
        skip_value(&mut cur, end);
        let value = Span::new(value_start, cur.pos.min(end));

        if !ALLOWED_FIELDS.contains(&name.as_str()) {
            diagnostics.push(Diagnostic::error(
                name_span,
                format!(
                    "unknown field `{name}` in `@img`; expected one of {}",
                    ALLOWED_FIELDS.join(", ")
                ),
            ));
            skip_comma(&mut cur);
            continue;
        }

        if name == "decorative" {
            let Some(flag) = bool_literal(src, value) else {
                diagnostics.push(Diagnostic::error(
                    value,
                    "`decorative` must be `Bool.true` or `Bool.false`",
                ));
                skip_comma(&mut cur);
                continue;
            };
            fields.decorative = Some((flag, value));
            skip_comma(&mut cur);
            continue;
        }

        let Some(str_val) = string_literal(src, value) else {
            diagnostics.push(Diagnostic::error(
                value,
                format!("`{name}` must be a compile-time string literal"),
            ));
            skip_comma(&mut cur);
            continue;
        };

        match name.as_str() {
            "src" => fields.src = Some((str_val, value)),
            "alt" => fields.alt = Some((str_val, value)),
            "title" => fields.title = Some((str_val, value)),
            "width" => fields.width = Some((str_val, value)),
            "height" => fields.height = Some((str_val, value)),
            "class" => fields.class = Some((str_val, value)),
            "loading" => {
                if !LOADING_VALUES.contains(&str_val.as_str()) {
                    diagnostics.push(Diagnostic::error(
                        value,
                        format!(
                            "`loading` must be one of {}",
                            LOADING_VALUES
                                .iter()
                                .map(|value| format!("`{value}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    ));
                    skip_comma(&mut cur);
                    continue;
                }
                fields.loading = Some((str_val, value));
            }
            "decoding" => {
                if !DECODING_VALUES.contains(&str_val.as_str()) {
                    diagnostics.push(Diagnostic::error(
                        value,
                        format!(
                            "`decoding` must be one of {}",
                            DECODING_VALUES
                                .iter()
                                .map(|value| format!("`{value}`"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    ));
                    skip_comma(&mut cur);
                    continue;
                }
                fields.decoding = Some((str_val, value));
            }
            _ => unreachable!(),
        }

        skip_comma(&mut cur);
    }

    if fields.src.is_none() {
        diagnostics.push(Diagnostic::error(
            body,
            "missing required field `src` in `@img`",
        ));
    }

    let decorative = fields
        .decorative
        .as_ref()
        .map(|(value, _)| *value)
        .unwrap_or(false);
    let alt = fields.alt.as_ref().map(|(value, _)| value.as_str());
    if decorative {
        if alt.is_some_and(|value| !value.is_empty()) {
            diagnostics.push(Diagnostic::error(
                fields.alt.as_ref().map(|(_, span)| *span).unwrap_or(body),
                "decorative `@img` must not set a non-empty `alt`",
            ));
        }
    } else if alt.is_none() || alt.is_some_and(str::is_empty) {
        diagnostics.push(Diagnostic::error(
            fields.alt.as_ref().map(|(_, span)| *span).unwrap_or(body),
            "`@img` requires `alt` for meaningful images; set `alt` or `decorative: Bool.true`",
        ));
    }

    fields
}

fn skip_comma(cur: &mut Cursor<'_>) {
    cur.skip_trivia();
    if cur.peek() == Some(',') {
        cur.bump();
    }
}

pub fn is_remote_asset(url: &str) -> bool {
    let url = url.trim();
    url.contains("://") || url.starts_with("data:") || url.starts_with('#')
}

pub fn normalize_local_asset_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() || is_remote_asset(url) || url.starts_with('/') {
        return None;
    }
    let path = Path::new(url);
    if path.is_absolute() {
        return None;
    }
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(part) => {
                parts.push(part.to_string_lossy().into_owned());
            }
            std::path::Component::ParentDir => {
                parts.pop()?;
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

pub fn resolve_local_asset(source_dir: &Path, url: &str) -> Option<PathBuf> {
    let relative = normalize_local_asset_url(url)?;
    Some(source_dir.join(relative))
}

pub fn collect_local_media(source: SourceFile<'_>, document: &Document) -> Vec<(String, Span)> {
    let mut urls = Vec::new();
    collect_media_in(source, document, &mut urls);
    urls
}

fn collect_media_in(source: SourceFile<'_>, document: &Document, urls: &mut Vec<(String, Span)>) {
    for item in &document.items {
        match item {
            Item::Img(img) => {
                let mut diags = Vec::new();
                let fields = extract_img_fields(source.src, img.body, &mut diags);
                if let Some((src, span)) = fields.src
                    && !is_remote_asset(&src)
                {
                    urls.push((src, span));
                }
            }
            Item::Markdown(node) => node.walk(&mut |child| {
                if let MdNode::Image { url, span, .. } = child
                    && !is_remote_asset(url)
                {
                    urls.push((url.clone(), *span));
                }
            }),
            Item::Docs(docs) => {
                let (_, content) = split_docs_body(source.src, docs.body);
                let parsed = parse_fragment(source, content, false);
                collect_media_in(source, &parsed.document, urls);
            }
            _ => {}
        }
    }
}

pub fn check_document_assets(
    source: SourceFile<'_>,
    document: &Document,
    options: &CompileOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(source_dir) = options
        .theme
        .source_dir
        .clone()
        .or_else(|| Path::new(source.name).parent().map(Path::to_path_buf))
    else {
        return;
    };
    for (url, span) in collect_local_media(source, document) {
        if is_remote_asset(&url) || url.trim().starts_with('/') {
            continue;
        }
        let Some(path) = resolve_local_asset(&source_dir, &url) else {
            diagnostics.push(Diagnostic::error(
                span,
                format!("local asset `{url}` is not a path under the source file"),
            ));
            continue;
        };
        if !path.is_file() {
            diagnostics.push(Diagnostic::error(
                span,
                format!("missing local asset `{url}`"),
            ));
        }
    }
}
