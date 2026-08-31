use crate::ast::BracketRecord;
use crate::page::{bool_literal, string_list, string_literal};
use rocci_template::Span;

use super::DocsAttrs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsField {
    pub name: String,
    pub name_span: Span,
    pub value: Span,
}

pub fn field_string(src: &str, field: &DocsField) -> Option<String> {
    string_literal(src, field.value)
}

pub fn field_bool(src: &str, field: &DocsField) -> Option<bool> {
    bool_literal(src, field.value)
}

pub fn field_strings(src: &str, field: &DocsField) -> Option<Vec<String>> {
    string_list(src, field.value)
}

pub fn docs_fields_from_params(params: Option<&BracketRecord>) -> Vec<DocsField> {
    let Some(record) = params else {
        return Vec::new();
    };
    record
        .fields
        .iter()
        .map(|field| DocsField {
            name: field.name.clone(),
            name_span: field.name_span,
            value: field.value.span(),
        })
        .collect()
}

pub(crate) fn parse_attrs(src: &str, fields: &[DocsField]) -> DocsAttrs {
    let mut attrs = DocsAttrs::default();
    for field in fields {
        match field.name.as_str() {
            "title" => attrs.title = field_string(src, field),
            "summary" => attrs.summary = field_string(src, field),
            "label" => attrs.label = field_string(src, field),
            "term" => attrs.term = field_string(src, field),
            "caption" => attrs.caption = field_string(src, field),
            "credit" => attrs.credit = field_string(src, field),
            "tone" => attrs.tone = field_string(src, field),
            "page" => attrs.page = field_string(src, field),
            "href" => attrs.href = field_string(src, field),
            "group" => attrs.group = field_string(src, field),
            "kind" => attrs.tab_kind = field_string(src, field),
            "id" => attrs.id = field_string(src, field),
            "path" => attrs.path = field_string(src, field),
            "region" => attrs.region = field_string(src, field),
            "language" => attrs.language = field_string(src, field),
            "expect" => attrs.expect = field_string(src, field),
            "start" => attrs.start = field.value.of(src).trim().parse().ok(),
            "end" => attrs.end = field.value.of(src).trim().parse().ok(),
            "open" => attrs.open = field_bool(src, field).unwrap_or(false),
            "verify" => attrs.verify = field_bool(src, field).unwrap_or(false),
            "allow_network" => attrs.allow_network = field_bool(src, field).unwrap_or(false),
            "test" => {
                if let Some(items) = field_strings(src, field) {
                    attrs.test = items;
                } else if let Some(value) = field_string(src, field) {
                    attrs.test = split_argv(&value);
                } else {
                    attrs.unknown.push(field.name.clone());
                }
            }
            other => {
                if let Some(value) = field_bool(src, field) {
                    attrs.extra_bool.insert(other.to_string(), value);
                } else if let Some(value) = field_string(src, field) {
                    attrs.extra.insert(other.to_string(), value);
                } else {
                    attrs.unknown.push(other.to_string());
                }
            }
        }
    }
    attrs
}

fn split_argv(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_string).collect()
}

pub(crate) fn attr_nonempty(attrs: &DocsAttrs, field: &str) -> bool {
    if attrs.extra_bool.contains_key(field) {
        return true;
    }
    let value = match field {
        "title" => attrs.title.as_deref(),
        "summary" => attrs.summary.as_deref(),
        "label" => attrs.label.as_deref(),
        "term" => attrs.term.as_deref(),
        "alt" => attrs.alt.as_deref(),
        "caption" => attrs.caption.as_deref(),
        "credit" => attrs.credit.as_deref(),
        "tone" => attrs.tone.as_deref(),
        "page" => attrs.page.as_deref(),
        "href" => attrs.href.as_deref(),
        "group" => attrs.group.as_deref(),
        "kind" => attrs.tab_kind.as_deref(),
        "id" => attrs.id.as_deref(),
        "path" => attrs.path.as_deref(),
        "region" => attrs.region.as_deref(),
        "language" => attrs.language.as_deref(),
        "expect" => attrs.expect.as_deref(),
        other => attrs.extra.get(other).map(String::as_str),
    };
    value.is_some_and(|value| !value.is_empty())
}

pub(crate) fn attr_some(attrs: &DocsAttrs, field: &str) -> bool {
    match field {
        "page" => attrs.page.is_some(),
        "href" => attrs.href.is_some(),
        other => attr_nonempty(attrs, other),
    }
}

pub(crate) fn attr_str(attrs: &DocsAttrs, name: &str) -> String {
    match name {
        "title" => attrs.title.clone().unwrap_or_default(),
        "term" => attrs
            .term
            .clone()
            .or_else(|| attrs.title.clone())
            .unwrap_or_default(),
        "summary" => attrs.summary.clone().unwrap_or_default(),
        "label" => attrs.label.clone().unwrap_or_default(),
        "href" => attrs.href.clone().unwrap_or_else(|| {
            attrs
                .page
                .as_deref()
                .map(|page| format!("/{}/", page.trim_matches('/')))
                .unwrap_or_default()
        }),
        "caption" => attrs.caption.clone().unwrap_or_default(),
        "credit" => attrs.credit.clone().unwrap_or_default(),
        "id" => attrs.id.clone().unwrap_or_default(),
        "group" => attrs.group.clone().unwrap_or_default(),
        "kind" => attrs.tab_kind.clone().unwrap_or_default(),
        "tone" => attrs.tone.clone().unwrap_or_default(),
        other => attrs.extra.get(other).cloned().unwrap_or_default(),
    }
}

pub(crate) fn attr_bool(attrs: &DocsAttrs, name: &str) -> bool {
    match name {
        "open" => attrs.open,
        "verify" => attrs.verify,
        "allow_network" => attrs.allow_network,
        other => attrs.extra_bool.get(other).copied().unwrap_or(false),
    }
}
