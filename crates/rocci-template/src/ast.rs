use crate::span::Span;

#[path = "ast.generated.rs"]
mod ast_generated;
pub use ast_generated::*;

#[path = "node_kind.generated.rs"]
mod node_kind;
#[cfg(test)]
pub(crate) use node_kind::NodeKind;

impl TemplateItem {
    pub fn is_let(&self) -> bool {
        matches!(self, Self::Let(_))
    }

    pub fn is_preamble(&self) -> bool {
        matches!(self, Self::Let(_) | Self::Css(_))
    }
}

impl ComponentPath {
    pub fn source_name(&self) -> String {
        self.parts
            .iter()
            .map(|part| part.name.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParsedParams {
    pub first_param_is_record: bool,
    pub param_names: Vec<String>,
    pub optional_params: Vec<String>,
    pub param_defaults: Vec<(String, String)>,
    pub param_types: Vec<(String, String)>,
    pub body_params: Vec<String>,
}

pub fn parse_component_params(src: &str, params: Span) -> ParsedParams {
    let raw = params.of(src).trim();
    let inner = raw
        .strip_prefix('|')
        .and_then(|s| s.strip_suffix('|'))
        .unwrap_or(raw);
    let parts = split_top_level(inner, ',');
    let (first_param_is_record, mut named) = match parts.first() {
        Some(first) => first_param_names(first.trim()),
        None => (false, Vec::new()),
    };
    let body: Vec<NamedParam> = parts
        .iter()
        .skip(1)
        .filter_map(|part| named_param(part))
        .collect();
    let body_params: Vec<String> = body.iter().map(|param| param.name.clone()).collect();
    named.extend(body);
    let mut optional_params = Vec::new();
    let mut param_defaults = Vec::new();
    let mut param_types = Vec::new();
    let mut param_names = Vec::new();
    for param in named {
        if let Some(default) = param.default {
            optional_params.push(param.name.clone());
            param_defaults.push((param.name.clone(), default));
        }
        if let Some(ty) = param.ty {
            param_types.push((param.name.clone(), ty));
        }
        param_names.push(param.name);
    }
    ParsedParams {
        first_param_is_record,
        param_names,
        optional_params,
        param_defaults,
        param_types,
        body_params,
    }
}

/// Rewrite `| { name ?? "Roc" } |` to `|{ name }|`.
///
/// Workaround: Roc nightly-2026-08-08 rejects `??` in record patterns. Strip
/// defaults from generated params and apply them at call sites instead. Remove
/// this (and the matching call-site fill in `lower` / `rocci view`) once
/// `|{ name ?? "Roc" }|` typechecks.
pub fn strip_param_defaults(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(inner) = trimmed.strip_prefix('|').and_then(|s| s.strip_suffix('|')) else {
        return trimmed.to_string();
    };
    let parts: Vec<String> = split_top_level(inner, ',')
        .into_iter()
        .map(strip_defaults_in_param)
        .collect();
    format!("|{}|", parts.join(", "))
}

/// Count top-level `|a, b|` parameters. A record `{ db }` is one argument.
/// `||` is zero.
pub fn handler_param_arity(raw: &str) -> usize {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix('|')
        .and_then(|s| s.strip_suffix('|'))
        .unwrap_or(trimmed)
        .trim();
    if inner.is_empty() {
        return 0;
    }
    split_top_level(inner, ',').len()
}

/// Generated dispatch always calls `handler!(context, request)`.
/// Empty and one-parameter handlers get an unused `_request` appended.
pub fn ensure_handler_request_param(raw: &str) -> String {
    match handler_param_arity(raw) {
        0 => "|_, _request|".to_string(),
        1 => {
            let trimmed = raw.trim();
            let inner = trimmed
                .strip_prefix('|')
                .and_then(|s| s.strip_suffix('|'))
                .unwrap_or(trimmed)
                .trim();
            format!("|{inner}, _request|")
        }
        _ => raw.to_string(),
    }
}

#[derive(Clone, Debug)]
struct NamedParam {
    name: String,
    ty: Option<String>,
    default: Option<String>,
}

fn first_param_names(first: &str) -> (bool, Vec<NamedParam>) {
    if first.starts_with('{') && first.ends_with('}') {
        let record_inner = &first[1..first.len() - 1];
        let fields = split_top_level(record_inner, ',')
            .into_iter()
            .filter_map(named_param)
            .collect();
        (true, fields)
    } else {
        (false, named_param(first).into_iter().collect())
    }
}

fn named_param(part: &str) -> Option<NamedParam> {
    ident_from_param(part).map(|name| NamedParam {
        ty: type_annot(part, &name),
        name,
        default: default_expr(part),
    })
}

fn type_annot(part: &str, name: &str) -> Option<String> {
    let trimmed = part.trim();
    let after_junk = trimmed.trim_start_matches(|ch: char| !ch.is_ascii_alphabetic() && ch != '_');
    let after_name = after_junk.strip_prefix(name)?;
    let before_default = match find_top_level_qq(after_name) {
        Some(idx) => after_name[..idx].trim(),
        None => after_name.trim(),
    };
    let ty = before_default.strip_prefix(':')?.trim();
    if ty.is_empty() {
        None
    } else {
        Some(ty.to_string())
    }
}

fn default_expr(part: &str) -> Option<String> {
    let idx = find_top_level_qq(part)?;
    let expr = part[idx + 2..].trim();
    if expr.is_empty() {
        None
    } else {
        Some(expr.to_string())
    }
}

fn find_top_level_qq(part: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut chars = part.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '"' => {
                while let Some((_, next)) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == '"' {
                        break;
                    }
                }
            }
            '?' if depth == 0 && chars.peek().is_some_and(|(_, next)| *next == '?') => {
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

fn strip_defaults_in_param(part: &str) -> String {
    let trimmed = part.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() >= 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        let fields: Vec<String> = split_top_level(inner, ',')
            .into_iter()
            .map(strip_default_suffix)
            .filter(|field| !field.is_empty())
            .collect();
        if fields.is_empty() {
            "{}".to_string()
        } else {
            format!("{{ {} }}", fields.join(", "))
        }
    } else {
        strip_default_suffix(trimmed)
    }
}

fn strip_default_suffix(part: &str) -> String {
    match find_top_level_qq(part) {
        Some(idx) => part[..idx].trim().to_string(),
        None => part.trim().to_string(),
    }
}

fn split_top_level(inner: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut part_start = 0;
    let mut chars = inner.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '"' => {
                while let Some((_, next)) = chars.next() {
                    if next == '\\' {
                        chars.next();
                    } else if next == '"' {
                        break;
                    }
                }
            }
            c if c == sep && depth == 0 => {
                parts.push(&inner[part_start..i]);
                part_start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&inner[part_start..]);
    parts
}

#[cfg(test)]
mod handler_params {
    use super::{ensure_handler_request_param, handler_param_arity};

    #[test]
    fn counts_top_level_params_not_record_fields() {
        assert_eq!(handler_param_arity("||"), 0);
        assert_eq!(handler_param_arity("|{ db }|"), 1);
        assert_eq!(handler_param_arity("|{ db, count }|"), 1);
        assert_eq!(handler_param_arity("|state|"), 1);
        assert_eq!(handler_param_arity("|_|"), 1);
        assert_eq!(handler_param_arity("|{ db }, request|"), 2);
        assert_eq!(handler_param_arity("|state, _request|"), 2);
        assert_eq!(handler_param_arity("|a, b, c|"), 3);
    }

    #[test]
    fn injects_unused_request_for_short_handlers() {
        assert_eq!(ensure_handler_request_param("||"), "|_, _request|");
        assert_eq!(
            ensure_handler_request_param("|{ db }|"),
            "|{ db }, _request|"
        );
        assert_eq!(ensure_handler_request_param("|_|"), "|_, _request|");
        assert_eq!(
            ensure_handler_request_param("|{ db }, request|"),
            "|{ db }, request|"
        );
        assert_eq!(
            ensure_handler_request_param("|state, _request|"),
            "|state, _request|"
        );
    }
}

fn ident_from_param(part: &str) -> Option<String> {
    let trimmed = part.trim();
    let ident = trimmed
        .trim_start_matches(|ch: char| !ch.is_ascii_alphabetic() && ch != '_')
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect::<String>();
    if ident.is_empty() { None } else { Some(ident) }
}

#[cfg(test)]
mod node_kind_highlight {
    use super::NodeKind;

    fn host_paints(kind: NodeKind) -> bool {
        match kind {
            NodeKind::Document
            | NodeKind::ModuleItem
            | NodeKind::ComponentDecl
            | NodeKind::FixtureDecl
            | NodeKind::CssDecl
            | NodeKind::ContextDecl
            | NodeKind::InitDecl
            | NodeKind::LiveDecl
            | NodeKind::ViewDecl
            | NodeKind::PatchDecl
            | NodeKind::CommandDecl
            | NodeKind::TemplateBlock
            | NodeKind::TemplateItem
            | NodeKind::Element
            | NodeKind::ComponentCall
            | NodeKind::ComponentPath
            | NodeKind::Fragment
            | NodeKind::IfDirective
            | NodeKind::ForDirective
            | NodeKind::MatchDirective
            | NodeKind::MatchArm
            | NodeKind::LetDirective
            | NodeKind::Attr
            | NodeKind::AttrValue => true,
            NodeKind::Ident | NodeKind::Interpolation | NodeKind::TextNode => false,
        }
    }

    #[test]
    fn every_kind_is_painted_or_omitted() {
        for &kind in NodeKind::ALL {
            assert_eq!(
                host_paints(kind),
                !kind.highlight_omitted(),
                "{kind:?} must be painted by the host collector or listed in [highlight.omit]"
            );
        }
    }
}
