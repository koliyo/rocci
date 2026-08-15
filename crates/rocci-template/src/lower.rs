use std::collections::HashMap;

use crate::ast::{
    Attr, AttrValue, ComponentCall, ComponentDecl, Document, Element, FixtureDecl, ForDirective,
    Fragment, Ident, IfDirective, Interpolation, MatchDirective, ModuleItem, TemplateBlock,
    TemplateItem, parse_component_params, strip_param_defaults,
};
use crate::source_map::{OriginKind, Segment};
use crate::span::{SourceFile, Span};

#[derive(Clone, Debug)]
pub struct LowerOptions {
    pub html_module: String,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            html_module: "Html".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoweredModule {
    pub roc: String,
    pub segments: Vec<Segment>,
    pub components: Vec<ComponentInfo>,
    pub fixtures: Vec<FixtureInfo>,
}

#[derive(Clone, Debug)]
pub struct ComponentInfo {
    pub name: String,
    pub body_params: Vec<String>,
    pub param_names: Vec<String>,
    pub optional_params: Vec<String>,
    pub param_defaults: Vec<(String, String)>,
    pub param_types: Vec<(String, String)>,
    pub first_param_is_record: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FixtureInfo {
    pub name: String,
    pub target: String,
    pub value: String,
    pub span: Span,
}

pub fn lower(source: SourceFile<'_>, document: &Document, options: &LowerOptions) -> LoweredModule {
    // Workaround: fill `??` defaults at call sites until Roc accepts them in patterns.
    // See `strip_param_defaults`.
    let mut field_defaults = HashMap::new();
    for item in &document.items {
        if let ModuleItem::Component(component) = item {
            let parsed = parse_component_params(source.src, component.params);
            let prop_count = parsed.param_names.len() - parsed.body_params.len();
            let defaults = parsed
                .param_defaults
                .into_iter()
                .filter(|(name, _)| {
                    parsed
                        .param_names
                        .iter()
                        .take(prop_count)
                        .any(|n| n == name)
                })
                .collect();
            field_defaults.insert(component.name.name.clone(), defaults);
        }
    }
    let mut emitter = Emitter {
        src: source.src,
        html: &options.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        components: Vec::new(),
        fixtures: Vec::new(),
        field_defaults,
    };
    let inject_datastar =
        document_has_action(document) && !document_imports_datastar(source.src, document);
    let mut injected = false;
    if inject_datastar && !matches!(document.items.first(), Some(ModuleItem::Roc { .. })) {
        emitter.emit("import Datastar\n\n");
        injected = true;
    }
    for item in &document.items {
        match item {
            ModuleItem::Roc { span } => {
                if inject_datastar && !injected {
                    emitter.emit_roc_with_datastar_import(*span);
                    injected = true;
                } else {
                    emitter.emit_source(*span, OriginKind::OrdinaryRoc);
                }
            }
            ModuleItem::Component(component) => emitter.lower_component(component),
            ModuleItem::Fixture(fixture) => emitter.lower_fixture(fixture),
        }
    }
    if !emitter.roc.ends_with('\n') && !emitter.roc.is_empty() {
        emitter.roc.push('\n');
    }
    LoweredModule {
        roc: emitter.roc,
        segments: emitter.segments,
        components: emitter.components,
        fixtures: emitter.fixtures,
    }
}

struct Emitter<'a> {
    src: &'a str,
    html: &'a str,
    roc: String,
    segments: Vec<Segment>,
    indent: usize,
    at_line_start: bool,
    components: Vec<ComponentInfo>,
    fixtures: Vec<FixtureInfo>,
    field_defaults: HashMap<String, Vec<(String, String)>>,
}

impl<'a> Emitter<'a> {
    fn lower_component(&mut self, component: &ComponentDecl) {
        let parsed = parse_component_params(self.src, component.params);
        let body_params = parsed.body_params.clone();
        self.components.push(ComponentInfo {
            name: component.name.name.clone(),
            body_params: body_params.clone(),
            param_names: parsed.param_names,
            optional_params: parsed.optional_params,
            param_defaults: parsed.param_defaults,
            param_types: parsed.param_types,
            first_param_is_record: parsed.first_param_is_record,
            span: component.span,
        });

        self.emit_mapped(
            &component.name.name,
            component.name.span,
            OriginKind::ComponentSignature,
        );
        self.emit(" = ");
        // Workaround: emit params without `??` until Roc accepts that pattern syntax.
        // See `strip_param_defaults`.
        self.emit_mapped(
            &strip_param_defaults(component.params.of(self.src).trim()),
            component.params,
            OriginKind::ComponentSignature,
        );
        self.emit(" {\n");
        self.indent += 1;
        self.push_indent();
        self.lower_block(&component.body, &body_params);
        self.emit("\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    fn lower_fixture(&mut self, fixture: &FixtureDecl) {
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

    fn lower_block(&mut self, block: &TemplateBlock, body_params: &[String]) {
        let (lets, rest) = split_lets(&block.items);
        for let_dir in &lets {
            self.emit_mapped(
                &let_dir.binder.name,
                let_dir.binder.span,
                OriginKind::Directive,
            );
            self.emit(" = ");
            self.emit_mapped(
                let_dir.expr.of(self.src).trim(),
                let_dir.expr,
                OriginKind::Directive,
            );
            self.emit("\n\n");
            self.push_indent();
        }
        self.lower_html_value(rest, body_params);
    }

    fn lower_html_value(&mut self, items: &[TemplateItem], body_params: &[String]) {
        match items {
            [] => self.emit_html(".empty"),
            [item] => self.lower_item(item, body_params, ValueCtx::Node),
            _ => {
                self.emit_html(".fragment(\n");
                self.indent += 1;
                self.push_indent();
                self.lower_node_list(items, body_params);
                self.emit(",\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
        }
    }

    fn lower_item(&mut self, item: &TemplateItem, body_params: &[String], ctx: ValueCtx) {
        match item {
            TemplateItem::Element(el) => self.lower_element(el, body_params),
            TemplateItem::ComponentCall(call) => self.lower_call(call, body_params),
            TemplateItem::Fragment(frag) => self.lower_fragment(frag, body_params),
            TemplateItem::Text(text) => {
                self.emit_html(".text(");
                self.emit_string(&text.value, text.span, OriginKind::StaticMarkup);
                self.emit(")");
            }
            TemplateItem::Interpolation(interp) => self.lower_interpolation(interp, body_params),
            TemplateItem::If(dir) => self.lower_if(dir, body_params),
            TemplateItem::For(dir) => {
                if ctx == ValueCtx::Node {
                    self.emit_html(".fragment(");
                    self.lower_for_map(dir, body_params);
                    self.emit(")");
                } else {
                    self.lower_for_map(dir, body_params);
                }
            }
            TemplateItem::Match(dir) => self.lower_match(dir, body_params),
            TemplateItem::Let(_) => {}
        }
    }

    fn lower_element(&mut self, el: &Element, body_params: &[String]) {
        let void_el = el.self_closing && is_void(&el.name.name);
        if void_el {
            self.emit_html(".void_element(\n");
        } else {
            self.emit_html(".element(\n");
        }
        self.indent += 1;
        self.push_indent();
        self.emit_string(&el.name.name, el.name.span, OriginKind::StaticMarkup);
        self.emit(",\n");
        self.push_indent();
        self.lower_html_attrs(&el.attrs);
        if !void_el {
            self.emit(",\n");
            self.push_indent();
            self.lower_node_list(&el.children, body_params);
        }
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn lower_html_attrs(&mut self, attrs: &[Attr]) {
        if attrs.is_empty() {
            self.emit("[]");
            return;
        }
        self.emit("[\n");
        self.indent += 1;
        for attr in attrs {
            self.push_indent();
            match &attr.value {
                AttrValue::Static { span, value } => {
                    self.emit_html(".attribute(");
                    self.emit_string(&attr.name.name, attr.name.span, OriginKind::StaticMarkup);
                    self.emit(", ");
                    self.emit_string(value, *span, OriginKind::StaticMarkup);
                    self.emit(")");
                }
                AttrValue::Expr { expr } => {
                    self.emit_html(".attribute(");
                    self.emit_string(&attr.name.name, attr.name.span, OriginKind::StaticMarkup);
                    self.emit(", ");
                    self.emit_mapped(
                        expr.of(self.src).trim(),
                        *expr,
                        OriginKind::AttributeExpression,
                    );
                    self.emit(")");
                }
                AttrValue::Action { name, args } => {
                    self.emit_html(".attribute(");
                    self.emit_string(&attr.name.name, attr.name.span, OriginKind::StaticMarkup);
                    self.emit(", ");
                    self.lower_action_call(name, *args);
                    self.emit(")");
                }
                AttrValue::Boolean => {
                    self.emit_html(".boolean_attribute(");
                    self.emit_string(&attr.name.name, attr.name.span, OriginKind::StaticMarkup);
                    self.emit(", Bool.true)");
                }
            }
            self.emit(",\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    fn lower_call(&mut self, call: &ComponentCall, body_params: &[String]) {
        self.emit_mapped(
            &call.path.roc_name,
            call.path.span,
            OriginKind::ComponentTag,
        );
        self.emit("(\n");
        self.indent += 1;
        self.push_indent();
        self.lower_props(&call.attrs, &call.path.roc_name);
        if let Some(children) = &call.children {
            self.emit(",\n");
            self.push_indent();
            self.lower_html_value(children, body_params);
        }
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn lower_props(&mut self, attrs: &[Attr], roc_name: &str) {
        // Workaround: insert omitted `??` defaults here until Roc accepts them in patterns.
        // See `strip_param_defaults`.
        let missing_defaults: Vec<(String, String)> = self
            .field_defaults
            .get(roc_name)
            .into_iter()
            .flatten()
            .filter(|(name, _)| !attrs.iter().any(|attr| attr.name.name == *name))
            .cloned()
            .collect();
        if attrs.is_empty() && missing_defaults.is_empty() {
            self.emit("{}");
            return;
        }
        self.emit("{ ");
        for (i, attr) in attrs.iter().enumerate() {
            if i > 0 {
                self.emit(", ");
            }
            match &attr.value {
                AttrValue::Static { span, value } => {
                    self.emit_mapped(&attr.name.name, attr.name.span, OriginKind::ComponentTag);
                    self.emit(": ");
                    self.emit_string(value, *span, OriginKind::StaticMarkup);
                }
                AttrValue::Expr { expr } => {
                    let expr_text = expr.of(self.src).trim();
                    self.emit_mapped(&attr.name.name, attr.name.span, OriginKind::ComponentTag);
                    self.emit(": ");
                    self.emit_mapped(expr_text, *expr, OriginKind::AttributeExpression);
                }
                AttrValue::Action { name, args } => {
                    self.emit_mapped(&attr.name.name, attr.name.span, OriginKind::ComponentTag);
                    self.emit(": ");
                    self.lower_action_call(name, *args);
                }
                AttrValue::Boolean => {
                    self.emit_mapped(&attr.name.name, attr.name.span, OriginKind::ComponentTag);
                    self.emit(": Bool.true");
                }
            }
        }
        for (i, (name, default)) in missing_defaults.iter().enumerate() {
            if !attrs.is_empty() || i > 0 {
                self.emit(", ");
            }
            self.emit(name);
            self.emit(": ");
            self.emit(default);
        }
        self.emit(" }");
    }

    fn lower_fragment(&mut self, frag: &Fragment, body_params: &[String]) {
        self.emit_html(".fragment(\n");
        self.indent += 1;
        self.push_indent();
        self.lower_node_list(&frag.children, body_params);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn lower_interpolation(&mut self, interp: &Interpolation, body_params: &[String]) {
        let expr = interp.expr.of(self.src).trim();
        if body_params.iter().any(|name| name == expr) {
            self.emit_mapped(expr, interp.expr, OriginKind::TextExpression);
            return;
        }
        self.emit_html(".text(");
        self.emit_mapped(expr, interp.expr, OriginKind::TextExpression);
        self.emit(")");
    }

    fn lower_if(&mut self, dir: &IfDirective, body_params: &[String]) {
        self.emit("if ");
        self.emit_mapped(
            dir.condition.of(self.src).trim(),
            dir.condition,
            OriginKind::Directive,
        );
        self.emit(" {\n");
        self.indent += 1;
        self.push_indent();
        self.lower_block(&dir.then_body, body_params);
        self.emit("\n");
        self.indent -= 1;
        for (cond, body) in &dir.else_ifs {
            self.push_indent();
            self.emit("} else if ");
            self.emit_mapped(cond.of(self.src).trim(), *cond, OriginKind::Directive);
            self.emit(" {\n");
            self.indent += 1;
            self.push_indent();
            self.lower_block(body, body_params);
            self.emit("\n");
            self.indent -= 1;
        }
        self.push_indent();
        self.emit("} else {\n");
        self.indent += 1;
        self.push_indent();
        if let Some(body) = &dir.else_body {
            self.lower_block(body, body_params);
        } else {
            self.emit_html(".empty");
        }
        self.emit("\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("}");
    }

    fn lower_for_map(&mut self, dir: &ForDirective, body_params: &[String]) {
        self.emit("List.map(");
        self.emit_mapped(
            dir.collection.of(self.src).trim(),
            dir.collection,
            OriginKind::Directive,
        );
        self.emit(", |");
        self.emit_mapped(&dir.binder.name, dir.binder.span, OriginKind::Directive);
        self.emit("| {\n");
        self.indent += 1;
        self.push_indent();
        self.lower_block(&dir.body, body_params);
        self.emit("\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("})");
    }

    fn lower_match(&mut self, dir: &MatchDirective, body_params: &[String]) {
        self.emit("match ");
        self.emit_mapped(
            dir.scrutinee.of(self.src).trim(),
            dir.scrutinee,
            OriginKind::Directive,
        );
        self.emit(" {\n");
        self.indent += 1;
        for arm in &dir.arms {
            self.push_indent();
            self.emit_mapped(
                arm.pattern.of(self.src).trim(),
                arm.pattern,
                OriginKind::Directive,
            );
            self.emit(" => ");
            self.lower_item(&arm.value, body_params, ValueCtx::Node);
            self.emit("\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("}");
    }

    fn lower_node_list(&mut self, items: &[TemplateItem], body_params: &[String]) {
        if items.is_empty() {
            self.emit("[]");
            return;
        }
        let groups = group_children(items);
        self.emit_concat_groups(&groups, body_params);
    }

    fn emit_concat_groups(&mut self, groups: &[ChildGroup<'_>], body_params: &[String]) {
        match groups {
            [] => self.emit("[]"),
            [group] => self.emit_child_group(group, body_params),
            [first, rest @ ..] => {
                self.emit("List.concat(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_child_group(first, body_params);
                self.emit(",\n");
                self.push_indent();
                self.emit_concat_groups(rest, body_params);
                self.emit(",\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
        }
    }

    fn emit_child_group(&mut self, group: &ChildGroup<'_>, body_params: &[String]) {
        match group {
            ChildGroup::Nodes(group_items) => self.emit_node_array(group_items, body_params),
            ChildGroup::List(item) => self.lower_item(item, body_params, ValueCtx::List),
        }
    }

    fn emit_node_array(&mut self, items: &[&TemplateItem], body_params: &[String]) {
        self.emit("[\n");
        self.indent += 1;
        for item in items {
            self.push_indent();
            self.lower_item(item, body_params, ValueCtx::Node);
            self.emit(",\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    fn lower_action_call(&mut self, name: &Ident, args: Span) {
        let args_text = args.of(self.src).trim();
        self.emit("Datastar.");
        self.emit_mapped(&name.name, name.span, OriginKind::AttributeExpression);
        if has_top_level_comma(args_text) {
            self.emit("_with");
        }
        self.emit("(");
        self.emit_mapped(args_text, args, OriginKind::AttributeExpression);
        self.emit(")");
    }

    fn emit_roc_with_datastar_import(&mut self, span: Span) {
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

    fn emit_html(&mut self, suffix: &str) {
        self.maybe_indent();
        self.roc.push_str(self.html);
        self.at_line_start = false;
        self.emit(suffix);
    }

    fn emit(&mut self, text: &str) {
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

    fn emit_mapped(&mut self, text: &str, source: Span, origin: OriginKind) {
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

    fn emit_source(&mut self, span: Span, origin: OriginKind) {
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

    fn emit_string(&mut self, value: &str, source: Span, origin: OriginKind) {
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

    fn push_indent(&mut self) {
        self.maybe_indent();
    }

    fn maybe_indent(&mut self) -> usize {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.roc.push_str("    ");
            }
            self.at_line_start = false;
        }
        self.roc.len()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueCtx {
    Node,
    List,
}

enum ChildGroup<'a> {
    Nodes(Vec<&'a TemplateItem>),
    List(&'a TemplateItem),
}

fn group_children(items: &[TemplateItem]) -> Vec<ChildGroup<'_>> {
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

fn split_lets(items: &[TemplateItem]) -> (Vec<&crate::ast::LetDirective>, &[TemplateItem]) {
    let mut count = 0;
    for item in items {
        if matches!(item, TemplateItem::Let(_)) {
            count += 1;
        } else {
            break;
        }
    }
    let lets = items[..count]
        .iter()
        .filter_map(|item| match item {
            TemplateItem::Let(dir) => Some(dir),
            _ => None,
        })
        .collect();
    (lets, &items[count..])
}

fn is_void(name: &str) -> bool {
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

fn document_has_action(document: &Document) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Component(component) => items_have_action(&component.body.items),
        _ => false,
    })
}

fn items_have_action(items: &[TemplateItem]) -> bool {
    items.iter().any(item_has_action)
}

fn item_has_action(item: &TemplateItem) -> bool {
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

fn attrs_have_action(attrs: &[Attr]) -> bool {
    attrs
        .iter()
        .any(|attr| matches!(attr.value, AttrValue::Action { .. }))
}

fn document_imports_datastar(src: &str, document: &Document) -> bool {
    document.items.iter().any(|item| match item {
        ModuleItem::Roc { span } => span.of(src).lines().any(line_imports_datastar),
        _ => false,
    })
}

fn line_imports_datastar(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed == "import Datastar"
        || trimmed.starts_with("import Datastar ")
        || trimmed.starts_with("import Datastar.")
}

fn import_insert_offset(text: &str) -> usize {
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

fn has_top_level_comma(text: &str) -> bool {
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
