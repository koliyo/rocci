use std::collections::HashSet;

use rocci_template::{
    ComponentDecl, ComponentInfo, Document, ModuleItem, TemplateBlock, TemplateItem,
    component_matches,
};

use super::{BrowseParam, CatalogEntry, ParamKind};

pub(crate) fn infer_params(
    src: &str,
    info: &ComponentInfo,
    document: &Document,
) -> Vec<BrowseParam> {
    let decl = find_decl(document, &info.name);
    info.param_names
        .iter()
        .map(|name| {
            let required = !info.optional_params.iter().any(|param| param == name);
            let is_body = info.body_params.iter().any(|param| param == name);
            let default_roc = info
                .param_defaults
                .iter()
                .find(|(param, _)| param == name)
                .map(|(_, value)| value.clone());
            let annotation = info
                .param_types
                .iter()
                .find(|(param, _)| param == name)
                .map(|(_, ty)| ty.as_str());
            let inferred = infer_one(
                name,
                is_body,
                annotation,
                default_roc.as_deref(),
                src,
                decl.map(|decl| &decl.body),
            );
            let (kind, reason) = match inferred {
                Inferred::Scalar(kind) => (Some(kind), String::new()),
                Inferred::Unsupported(reason) => (None, reason),
            };
            let default_display = match &kind {
                Some(kind) => display_default(kind, default_roc.as_deref()),
                None => String::new(),
            };
            BrowseParam {
                name: name.clone(),
                required,
                kind,
                reason,
                default_roc,
                default_display,
                is_body,
            }
        })
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Inferred {
    Scalar(ParamKind),
    Unsupported(String),
}

pub(crate) fn infer_one(
    name: &str,
    is_body: bool,
    annotation: Option<&str>,
    default_roc: Option<&str>,
    src: &str,
    body: Option<&TemplateBlock>,
) -> Inferred {
    if is_body {
        return Inferred::Scalar(ParamKind::BodyHtml);
    }
    if let Some(ty) = annotation {
        return match ParamKind::from_annotation(ty) {
            Some(kind) => Inferred::Scalar(kind),
            None => Inferred::Unsupported(format!("type `{ty}`")),
        };
    }
    if let Some(default) = default_roc {
        match infer_from_default(default) {
            Inferred::Scalar(kind) => return Inferred::Scalar(kind),
            Inferred::Unsupported(_) => {}
        }
    }
    if let Some(body) = body
        && let Some(inferred) = infer_from_usage(src, body, name)
    {
        return inferred;
    }
    Inferred::Unsupported("no scalar type".into())
}

pub(crate) fn infer_from_default(expr: &str) -> Inferred {
    let trimmed = expr.trim();
    if trimmed == "Bool.true" || trimmed == "Bool.false" || trimmed == "True" || trimmed == "False"
    {
        return Inferred::Scalar(ParamKind::Bool);
    }
    if is_i64(trimmed) {
        return Inferred::Scalar(ParamKind::I64);
    }
    if is_float(trimmed) {
        return Inferred::Scalar(ParamKind::F64);
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return Inferred::Scalar(ParamKind::Str);
    }
    if trimmed.starts_with('[') {
        return Inferred::Unsupported("list".into());
    }
    if trimmed.starts_with('{') {
        return Inferred::Unsupported("record".into());
    }
    if trimmed
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        return Inferred::Unsupported(format!("tag `{trimmed}`"));
    }
    Inferred::Unsupported(format!("default `{trimmed}`"))
}

pub(crate) fn is_i64(value: &str) -> bool {
    let rest = value.strip_prefix('-').unwrap_or(value);
    !rest.is_empty() && rest.bytes().all(|ch| ch.is_ascii_digit())
}

pub(crate) fn is_float(value: &str) -> bool {
    let rest = value.strip_prefix('-').unwrap_or(value);
    if rest.is_empty() || rest == "." || !rest.contains('.') {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for ch in rest.chars() {
        if ch == '.' {
            if seen_dot {
                return false;
            }
            seen_dot = true;
        } else if ch.is_ascii_digit() {
            seen_digit = true;
        } else {
            return false;
        }
    }
    seen_digit
}

pub(crate) fn display_default(kind: &ParamKind, default_roc: Option<&str>) -> String {
    match default_roc {
        Some(value) => display_roc_literal(value),
        None => kind.zero_display().to_string(),
    }
}

pub(crate) fn display_roc_literal(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "Bool.true" || trimmed == "True" {
        return "true".to_string();
    }
    if trimmed == "Bool.false" || trimmed == "False" {
        return "false".to_string();
    }
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return unescape_roc_string(&trimmed[1..trimmed.len() - 1]);
    }
    trimmed.to_string()
}

pub(crate) fn unescape_roc_string(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub(crate) fn find_decl<'a>(document: &'a Document, name: &str) -> Option<&'a ComponentDecl> {
    document.items.iter().find_map(|item| match item {
        ModuleItem::Component(decl) if component_matches(&decl.name.name, name) => Some(decl),
        _ => None,
    })
}

pub(crate) fn component_is_html_document(document: &Document, roc_name: &str) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Component(decl) if component_matches(&decl.name.name, roc_name) => {
            matches!(
                decl.body.items.iter().find(|item| !item.is_preamble()),
                Some(TemplateItem::Element(el)) if el.name.name == "html"
            )
        }
        _ => false,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UsageHint {
    Str,
    Bool,
    I64,
    Record,
    List,
    Tag,
}

pub(crate) fn infer_from_usage(src: &str, body: &TemplateBlock, param: &str) -> Option<Inferred> {
    let mut hints = Vec::new();
    walk_block(src, body, param, &mut hints);
    if hints.is_empty() {
        return None;
    }
    if hints.contains(&UsageHint::List) {
        return Some(Inferred::Unsupported("list".into()));
    }
    if hints.contains(&UsageHint::Tag) {
        return Some(Inferred::Unsupported("tag".into()));
    }
    if hints.contains(&UsageHint::Record) {
        return Some(Inferred::Unsupported("record".into()));
    }
    if hints.contains(&UsageHint::I64) {
        return Some(Inferred::Scalar(ParamKind::I64));
    }
    if hints.contains(&UsageHint::Bool) && !hints.contains(&UsageHint::Str) {
        return Some(Inferred::Scalar(ParamKind::Bool));
    }
    if hints.contains(&UsageHint::Str) {
        return Some(Inferred::Scalar(ParamKind::Str));
    }
    if hints.contains(&UsageHint::Bool) {
        return Some(Inferred::Scalar(ParamKind::Bool));
    }
    None
}

pub(crate) fn walk_block(src: &str, body: &TemplateBlock, param: &str, hints: &mut Vec<UsageHint>) {
    for item in &body.items {
        walk_item(src, item, param, hints);
    }
}

pub(crate) fn walk_item(src: &str, item: &TemplateItem, param: &str, hints: &mut Vec<UsageHint>) {
    match item {
        TemplateItem::Element(el) => {
            for attr in &el.attrs {
                match attr.value {
                    rocci_template::AttrValue::Expr { expr } => {
                        classify_expr(param, expr.of(src), hints);
                    }
                    rocci_template::AttrValue::Action { args, .. } => {
                        classify_expr(param, args.of(src), hints);
                    }
                    _ => {}
                }
            }
            for child in &el.children {
                walk_item(src, child, param, hints);
            }
        }
        TemplateItem::ComponentCall(call) => {
            for attr in &call.attrs {
                match attr.value {
                    rocci_template::AttrValue::Expr { expr } => {
                        classify_expr(param, expr.of(src), hints);
                    }
                    rocci_template::AttrValue::Action { args, .. } => {
                        classify_expr(param, args.of(src), hints);
                    }
                    _ => {}
                }
            }
            if let Some(children) = &call.children {
                for child in children {
                    walk_item(src, child, param, hints);
                }
            }
        }
        TemplateItem::Fragment(frag) => {
            for child in &frag.children {
                walk_item(src, child, param, hints);
            }
        }
        TemplateItem::Interpolation(interp) => {
            let expr = interp.expr.of(src).trim();
            if expr == param {
                hints.push(UsageHint::Str);
            } else {
                classify_expr(param, expr, hints);
            }
        }
        TemplateItem::If(dir) => {
            let cond = dir.condition.of(src).trim();
            if cond == param || cond == format!("!{param}") {
                hints.push(UsageHint::Bool);
            } else {
                classify_expr(param, cond, hints);
            }
            walk_block(src, &dir.then_body, param, hints);
            for (cond, body) in &dir.else_ifs {
                classify_expr(param, cond.of(src), hints);
                walk_block(src, body, param, hints);
            }
            if let Some(body) = &dir.else_body {
                walk_block(src, body, param, hints);
            }
        }
        TemplateItem::For(dir) => {
            let collection = dir.collection.of(src).trim();
            if collection == param {
                hints.push(UsageHint::List);
            } else {
                classify_expr(param, collection, hints);
            }
            walk_block(src, &dir.body, param, hints);
        }
        TemplateItem::Match(dir) => {
            let scrutinee = dir.scrutinee.of(src).trim();
            if scrutinee == param {
                hints.push(UsageHint::Tag);
            } else {
                classify_expr(param, scrutinee, hints);
            }
            for arm in &dir.arms {
                walk_item(src, &arm.value, param, hints);
            }
        }
        TemplateItem::Let(dir) => {
            classify_expr(param, dir.expr.of(src), hints);
        }
        TemplateItem::Text(_) | TemplateItem::Css(_) => {}
    }
}

pub(crate) fn classify_expr(param: &str, expr: &str, hints: &mut Vec<UsageHint>) {
    let expr = expr.trim();
    if expr.is_empty() || expr == param {
        return;
    }
    if expr == format!("{param}.to_str()")
        || expr == format!("Num.toStr({param})")
        || expr == format!("Num.to_str({param})")
    {
        hints.push(UsageHint::I64);
        return;
    }
    if is_list_expr(param, expr) {
        hints.push(UsageHint::List);
        return;
    }
    if is_record_expr(param, expr) {
        hints.push(UsageHint::Record);
    }
}

pub(crate) fn is_record_expr(param: &str, expr: &str) -> bool {
    let expr = expr.strip_prefix('!').unwrap_or(expr);
    expr.starts_with(param)
        && expr[param.len()..].starts_with('.')
        && expr != format!("{param}.to_str()")
        && expr != format!("{param}.toStr()")
}

pub(crate) fn is_list_expr(param: &str, expr: &str) -> bool {
    [
        format!("List.is_empty({param})"),
        format!("List.isEmpty({param})"),
        format!("List.len({param})"),
        format!("List.map({param}"),
        format!("List.keep_if({param}"),
        format!("List.fold({param}"),
        format!("List.concat({param}"),
        format!("List.get({param}"),
        format!("List.append({param}"),
    ]
    .iter()
    .any(|needle| expr.contains(needle.as_str()))
}

pub(crate) fn roc_imports(src: &str) -> Vec<String> {
    src.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("import ")
                .map(|rest| rest.split_whitespace().next().unwrap_or(rest).to_string())
        })
        .collect()
}

pub(crate) fn missing_imports(src: &str, available: &HashSet<String>) -> Vec<String> {
    roc_imports(src)
        .into_iter()
        .filter(|name| {
            !name.starts_with("pf.") && !name.starts_with("http.") && !available.contains(name)
        })
        .collect()
}

pub(crate) fn form_params(entry: &CatalogEntry) -> Vec<&BrowseParam> {
    entry
        .params
        .iter()
        .filter(|param| param.kind.is_some())
        .collect()
}

pub(crate) fn fixture_scalars(value: &str) -> Vec<(String, String)> {
    top_level_fields(value)
        .into_iter()
        .filter_map(|(name, expr)| fixture_scalar_display(&expr).map(|display| (name, display)))
        .collect()
}

pub(crate) fn fixture_scalar_display(expr: &str) -> Option<String> {
    let trimmed = strip_num_suffix(expr.trim());
    if trimmed.starts_with('{')
        || trimmed.starts_with('[')
        || trimmed.starts_with('(')
        || trimmed.contains('(')
    {
        return None;
    }
    match infer_from_default(trimmed) {
        Inferred::Scalar(
            ParamKind::Str
            | ParamKind::I64
            | ParamKind::U64
            | ParamKind::F64
            | ParamKind::Dec
            | ParamKind::Bool,
        ) => Some(display_roc_literal(trimmed)),
        _ => None,
    }
}

pub(crate) fn strip_num_suffix(value: &str) -> &str {
    for suffix in [".I64", ".U64", ".F64", ".Dec"] {
        if let Some(rest) = value.strip_suffix(suffix) {
            return rest;
        }
    }
    value
}

pub(crate) fn record_has_field(record: &str, name: &str) -> bool {
    top_level_fields(record)
        .iter()
        .any(|(field, _)| field == name)
}

pub(crate) fn top_level_fields(record: &str) -> Vec<(String, String)> {
    let trimmed = record.trim();
    let Some(inner) = trimmed.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return Vec::new();
    };
    split_top_level(inner, ',')
        .into_iter()
        .filter_map(|part| {
            let (name, value) = split_top_level_once(part, ':')?;
            let name = name.trim();
            if name.is_empty() {
                None
            } else {
                Some((name.to_string(), value.trim().to_string()))
            }
        })
        .collect()
}

pub(crate) fn split_top_level(inner: &str, sep: char) -> Vec<&str> {
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

pub(crate) fn split_top_level_once(part: &str, sep: char) -> Option<(&str, &str)> {
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
            c if c == sep && depth == 0 => {
                return Some((&part[..i], &part[i + c.len_utf8()..]));
            }
            _ => {}
        }
    }
    None
}
