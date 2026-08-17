use rocci_template::{Cursor, Diagnostic, Span};

use crate::page::{skip_value, string_literal};

const ALLOWED_FIELDS: &[&str] = &[
    "src", "alt", "title", "width", "height", "class", "loading", "decoding",
];

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
            cur.skip_trivia();
            if cur.peek() == Some(',') {
                cur.bump();
            }
            continue;
        }

        let Some(str_val) = string_literal(src, value) else {
            diagnostics.push(Diagnostic::error(
                value,
                format!("`{name}` must be a compile-time string literal"),
            ));
            cur.skip_trivia();
            if cur.peek() == Some(',') {
                cur.bump();
            }
            continue;
        };

        match name.as_str() {
            "src" => fields.src = Some((str_val, value)),
            "alt" => fields.alt = Some((str_val, value)),
            "title" => fields.title = Some((str_val, value)),
            "width" => fields.width = Some((str_val, value)),
            "height" => fields.height = Some((str_val, value)),
            "class" => fields.class = Some((str_val, value)),
            "loading" => fields.loading = Some((str_val, value)),
            "decoding" => fields.decoding = Some((str_val, value)),
            _ => unreachable!(),
        }

        cur.skip_trivia();
        if cur.peek() == Some(',') {
            cur.bump();
        }
    }

    if fields.src.is_none() {
        diagnostics.push(Diagnostic::error(
            body,
            "missing required field `src` in `@img`",
        ));
    }

    fields
}
