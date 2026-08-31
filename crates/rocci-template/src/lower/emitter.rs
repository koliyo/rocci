use std::collections::HashMap;

use crate::ast::{
    Attr, AttrValue, ComponentDecl, CssDecl, Document, FixtureDecl, ModuleItem, TemplateItem,
    TestDecl, component_param_pattern, component_props_type_anno, parse_component_params,
};
use crate::resolve::pascal_to_camel;
use crate::source_map::{OriginKind, Segment};
use crate::span::Span;

use super::html::HeadInject;
use super::{
    ComponentInfo, FixtureInfo, InitInfo, LiveInfo, RouteInfo, StyleArtifact, StyleKind, TestInfo,
    test_docs,
};

pub(crate) struct Emitter<'a> {
    pub(crate) src: &'a str,
    pub(crate) file_name: &'a str,
    pub(crate) html: &'a str,
    pub(crate) html_type: &'a str,
    pub(crate) roc: String,
    pub(crate) segments: Vec<Segment>,
    pub(crate) indent: usize,
    pub(crate) at_line_start: bool,
    pub(crate) components: Vec<ComponentInfo>,
    pub(crate) fixtures: Vec<FixtureInfo>,
    pub(crate) tests: Vec<TestInfo>,
    pub(crate) styles: Vec<StyleArtifact>,
    pub(crate) state_type: Option<String>,
    pub(crate) init: Option<InitInfo>,
    pub(crate) lives: Vec<LiveInfo>,
    pub(crate) routes: Vec<RouteInfo>,
    pub(crate) field_defaults: HashMap<String, Vec<(String, String)>>,
    pub(crate) file_css: String,
    pub(crate) file_scope_id: Option<String>,
    pub(crate) css_stamp: Option<String>,
    pub(crate) theme_css: Option<String>,
    pub(crate) theme_id: Option<String>,
    pub(crate) color_scheme_attr: Option<String>,
    pub(crate) embed_css: bool,
    pub(crate) stylesheet_href: Option<String>,
    pub(crate) inject_live_path: Option<(String, Span)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueCtx {
    Node,
    List,
}

pub(crate) enum ChildGroup<'a> {
    Nodes(Vec<&'a TemplateItem>),
    List(&'a TemplateItem),
}

pub(crate) fn group_children(items: &[TemplateItem]) -> Vec<ChildGroup<'_>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for item in items {
        if matches!(item, TemplateItem::For(_)) {
            if !current.is_empty() {
                groups.push(ChildGroup::Nodes(std::mem::take(&mut current)));
            }
            groups.push(ChildGroup::List(item));
        } else {
            current.push(item);
        }
    }
    if !current.is_empty() {
        groups.push(ChildGroup::Nodes(current));
    }
    groups
}

pub(crate) fn split_preamble(items: &[TemplateItem]) -> (&[TemplateItem], &[TemplateItem]) {
    let count = items.iter().take_while(|item| item.is_preamble()).count();
    (&items[..count], &items[count..])
}

pub(crate) fn concat_css<'a>(src: &'a str, decls: impl Iterator<Item = &'a CssDecl>) -> String {
    decls
        .map(|decl| decl.body.of(src).trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn is_html_document(items: &[TemplateItem]) -> bool {
    matches!(
        items,
        [TemplateItem::Element(el)]
            if el.name.name == "html"
                && !el.self_closing
                && !el.children.iter().any(|item| matches!(item, TemplateItem::For(_)))
    )
}

pub(crate) fn scope_css(css: &str, id: &str, external: bool) -> String {
    let css = css.trim();
    let native = format!("@scope ([data-rocci-css~=\"{id}\"]) {{\n{css}\n}}");
    if external {
        match external_scope_compatibility(css, id) {
            Some(compatibility) => format!("{native}\n{compatibility}"),
            None => native,
        }
    } else {
        native
    }
}

/// Linked stylesheets in the macOS preview can fail to apply a native scoped
/// rule even though the same rule works in an inline component stylesheet.
/// Keep the native rule and add an equivalent, attribute-prefixed rule for
/// uncomplicated CSS. Rules with nested at-rules retain the native form.
pub(crate) fn external_scope_compatibility(css: &str, id: &str) -> Option<String> {
    if css.contains('@') {
        return None;
    }
    let scope = format!("[data-rocci-css~=\"{id}\"]");
    let mut output = String::new();
    for rule in css.split('}') {
        let rule = rule.trim();
        if rule.is_empty() {
            continue;
        }
        let (selectors, declarations) = rule.split_once('{')?;
        let selectors = selectors
            .split(',')
            .map(str::trim)
            .map(|selector| format!("{scope}{selector}"))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&selectors);
        output.push_str(" { ");
        output.push_str(declarations.trim());
        output.push_str(" }\n");
    }
    (!output.is_empty()).then_some(output.trim_end().to_string())
}

pub fn file_scope_id(file_name: &str) -> String {
    let key = scope_file_key(file_name);
    format!("{}-{:08x}", file_stem(key), fnv1a32(key.as_bytes()))
}

pub(crate) fn component_scope_id(file_name: &str, component: &str) -> String {
    let key = scope_file_key(file_name);
    let mut bytes = key.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(component.as_bytes());
    format!("{}-{:08x}", sanitize_ident(component), fnv1a32(&bytes))
}

/// Snapshot CSS and island-service HTML must share stamps even when one
/// compile uses a basename and the other uses an absolute path.
pub(crate) fn scope_file_key(file_name: &str) -> &str {
    file_name.rsplit(['/', '\\']).next().unwrap_or(file_name)
}

pub(crate) fn file_stem(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = base
        .strip_suffix(".rocci")
        .or_else(|| base.rsplit_once('.').map(|(stem, _)| stem))
        .unwrap_or(base);
    sanitize_ident(stem)
}

pub(crate) fn sanitize_ident(name: &str) -> String {
    let mut out = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            out.push(ch);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    if out.is_empty() {
        "file".to_string()
    } else {
        out
    }
}

pub(crate) fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

pub(crate) fn is_void(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub(crate) fn document_has_action(document: &Document) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Component(component) => items_have_action(&component.body.items),
        _ => false,
    })
}

pub(crate) fn items_have_action(items: &[TemplateItem]) -> bool {
    items.iter().any(item_has_action)
}

pub(crate) fn item_has_action(item: &TemplateItem) -> bool {
    match item {
        TemplateItem::Element(el) => {
            attrs_have_action(&el.attrs) || items_have_action(&el.children)
        }
        TemplateItem::ComponentCall(call) => {
            attrs_have_action(&call.attrs)
                || call
                    .children
                    .as_ref()
                    .is_some_and(|children| items_have_action(children))
        }
        TemplateItem::Fragment(frag) => items_have_action(&frag.children),
        TemplateItem::If(dir) => {
            items_have_action(&dir.then_body.items)
                || dir
                    .else_ifs
                    .iter()
                    .any(|(_, body)| items_have_action(&body.items))
                || dir
                    .else_body
                    .as_ref()
                    .is_some_and(|body| items_have_action(&body.items))
        }
        TemplateItem::For(dir) => items_have_action(&dir.body.items),
        TemplateItem::Match(dir) => dir.arms.iter().any(|arm| item_has_action(&arm.value)),
        _ => false,
    }
}

pub(crate) fn attrs_have_action(attrs: &[Attr]) -> bool {
    attrs
        .iter()
        .any(|attr| matches!(attr.value, AttrValue::Action { .. }))
}

pub(crate) fn document_imports_datastar(src: &str, document: &Document) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Roc { span } => span.of(src).lines().any(line_imports_datastar),
        _ => false,
    })
}

pub(crate) fn line_imports_datastar(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "import Datastar"
        || trimmed.starts_with("import Datastar ")
        || trimmed.starts_with("import Datastar.")
}

pub(crate) fn import_insert_offset(text: &str) -> usize {
    let mut last = 0usize;
    let mut pos = 0usize;
    let mut saw_import = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            saw_import = true;
            last = pos + line.len();
        } else if !saw_import && (trimmed.starts_with("module ") || trimmed.starts_with("app ")) {
            last = pos + line.len();
        } else if saw_import && !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
        pos += line.len();
    }
    last
}

pub(crate) fn has_top_level_comma(text: &str) -> bool {
    let mut depth: usize = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((_, ch)) = chars.next() {
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
            ',' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

impl<'a> Emitter<'a> {
    pub(crate) fn emit_leading(&mut self, leading: &Option<crate::ast::LeadingComments>) {
        if let Some(leading) = leading {
            self.emit_source(leading.span, OriginKind::OrdinaryRoc);
        }
    }

    pub(crate) fn emit_css_leading(&mut self, css: &CssDecl) {
        self.emit_leading(&css.leading);
    }

    pub(crate) fn lower_component(&mut self, component: &ComponentDecl) {
        self.emit_leading(&component.leading);
        let parsed = parse_component_params(self.src, component.params);
        let body_params = parsed.body_params.clone();
        let roc_name = pascal_to_camel(&component.name.name);
        self.components.push(ComponentInfo {
            name: roc_name.clone(),
            body_params: body_params.clone(),
            param_names: parsed.param_names.clone(),
            optional_params: parsed.optional_params.clone(),
            param_defaults: parsed.param_defaults.clone(),
            param_types: parsed.param_types.clone(),
            first_param_is_record: parsed.first_param_is_record,
            span: component.span,
        });

        if let Some(props_ty) = component_props_type_anno(&parsed) {
            let mut anno = props_ty;
            for _ in &body_params {
                anno.push_str(", ");
                anno.push_str(self.html_type);
            }
            anno.push_str(" -> ");
            anno.push_str(self.html_type);
            self.emit_mapped(
                &roc_name,
                component.name.span,
                OriginKind::ComponentSignature,
            );
            self.emit(" : ");
            self.emit(&anno);
            self.emit("\n");
        }

        self.emit_mapped(
            &roc_name,
            component.name.span,
            OriginKind::ComponentSignature,
        );
        self.emit(" = ");
        self.emit_mapped(
            &component_param_pattern(&parsed),
            component.params,
            OriginKind::ComponentSignature,
        );
        self.emit(" {\n");
        self.indent += 1;
        self.push_indent();
        let (preamble, rest) = split_preamble(&component.body.items);
        self.emit_lets(preamble);
        let component_css = concat_css(
            self.src,
            preamble.iter().filter_map(|item| match item {
                TemplateItem::Css(css) => Some(css),
                _ => None,
            }),
        );
        let component_id = if component_css.is_empty() {
            None
        } else {
            Some(component_scope_id(self.file_name, &roc_name))
        };
        if let Some(id) = &component_id {
            let span = preamble
                .iter()
                .find_map(|item| match item {
                    TemplateItem::Css(css) => Some(css.span),
                    _ => None,
                })
                .unwrap_or(component.span);
            self.styles.push(StyleArtifact {
                kind: StyleKind::Component,
                name: roc_name.clone(),
                css: scope_css(&component_css, id, !self.embed_css),
                span,
            });
        }
        let mut stamp = Vec::new();
        if let Some(id) = &self.file_scope_id {
            stamp.push(id.clone());
        }
        if let Some(id) = &component_id {
            stamp.push(id.clone());
        }
        self.css_stamp = if stamp.is_empty() {
            None
        } else {
            Some(stamp.join(" "))
        };
        if self.embed_css {
            if let Some(css) = self.injected_css(&component_css, component_id.as_deref()) {
                self.lower_html_value_with_head(
                    rest,
                    &body_params,
                    HeadInject::EmbeddedStyle(&css),
                );
            } else if self.theme_css.is_some() && is_html_document(rest) {
                self.lower_html_value_with_head(rest, &body_params, HeadInject::EmbeddedStyle(""));
            } else {
                self.lower_html_value(rest, &body_params);
            }
        } else if let Some(href) = self.stylesheet_href.clone() {
            if is_html_document(rest) {
                self.lower_html_value_with_head(
                    rest,
                    &body_params,
                    HeadInject::StylesheetLink(&href),
                );
            } else {
                self.lower_html_value(rest, &body_params);
            }
        } else {
            self.lower_html_value(rest, &body_params);
        }
        self.css_stamp = None;
        self.emit("\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    pub(crate) fn lower_fixture(&mut self, fixture: &FixtureDecl) {
        self.emit_leading(&fixture.leading);
        let value = fixture.value.of(self.src).trim();
        self.fixtures.push(FixtureInfo {
            name: fixture.name.name.clone(),
            target: fixture.target.roc_name.clone(),
            value: value.to_string(),
            span: fixture.span,
        });
        if fixture.name.name.is_empty() {
            return;
        }
        self.emit_mapped(
            &fixture.name.name,
            fixture.name.span,
            OriginKind::OrdinaryRoc,
        );
        self.emit(" = ");
        self.emit_mapped(value, fixture.value, OriginKind::OrdinaryRoc);
        if !self.roc.ends_with('\n') {
            self.emit("\n");
        }
    }

    pub(crate) fn lower_test(&mut self, test: &TestDecl) {
        let expr = test.value.of(self.src).trim().to_string();
        self.tests.push(TestInfo {
            name: test.name.name.clone(),
            fixture: test.fixture.as_ref().map(|ident| ident.name.clone()),
            expr,
            docs: test_docs(self.src, &test.leading),
            span: test.span,
        });
    }
}

impl<'a> Emitter<'a> {
    pub(crate) fn emit_roc_with_datastar_import(&mut self, span: Span) {
        let text = span.of(self.src);
        if text.is_empty() {
            self.emit("import Datastar\n");
            return;
        }
        let insert_at = import_insert_offset(text);
        let start = span.start as usize;
        if insert_at > 0 {
            self.emit_source(Span::new(start, start + insert_at), OriginKind::OrdinaryRoc);
        }
        let needs_nl = insert_at > 0 && !text[..insert_at].ends_with('\n');
        if needs_nl {
            self.emit("\n");
        }
        self.emit("import Datastar\n");
        if insert_at < text.len() {
            self.emit_source(
                Span::new(start + insert_at, span.end as usize),
                OriginKind::OrdinaryRoc,
            );
        }
    }

    pub(crate) fn emit_html(&mut self, suffix: &str) {
        self.maybe_indent();
        self.roc.push_str(self.html);
        self.at_line_start = false;
        self.emit(suffix);
    }

    pub(crate) fn emit(&mut self, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                self.roc.push('\n');
                self.at_line_start = true;
            } else {
                self.maybe_indent();
                self.roc.push(ch);
                self.at_line_start = false;
            }
        }
    }

    pub(crate) fn emit_mapped(&mut self, text: &str, source: Span, origin: OriginKind) {
        self.maybe_indent();
        let start = self.roc.len();
        self.roc.push_str(text);
        self.at_line_start = text.ends_with('\n');
        self.segments.push(Segment::new(
            Span::new(start, self.roc.len()),
            source,
            origin,
        ));
    }

    pub(crate) fn emit_source(&mut self, span: Span, origin: OriginKind) {
        let text = span.of(self.src);
        if text.is_empty() {
            return;
        }
        let start = self.roc.len();
        self.roc.push_str(text);
        self.at_line_start = text.ends_with('\n');
        self.segments
            .push(Segment::new(Span::new(start, self.roc.len()), span, origin));
    }

    pub(crate) fn emit_string(&mut self, value: &str, source: Span, origin: OriginKind) {
        self.maybe_indent();
        let start = self.roc.len();
        self.roc.push('"');
        for ch in value.chars() {
            match ch {
                '\\' => self.roc.push_str("\\\\"),
                '"' => self.roc.push_str("\\\""),
                '\n' => self.roc.push_str("\\n"),
                '\r' => self.roc.push_str("\\r"),
                '\t' => self.roc.push_str("\\t"),
                _ => self.roc.push(ch),
            }
        }
        self.roc.push('"');
        self.at_line_start = false;
        self.segments.push(Segment::new(
            Span::new(start, self.roc.len()),
            source,
            origin,
        ));
    }

    pub(crate) fn push_indent(&mut self) {
        self.maybe_indent();
    }

    pub(crate) fn maybe_indent(&mut self) -> usize {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.roc.push_str("    ");
            }
            self.at_line_start = false;
        }
        self.roc.len()
    }
}
