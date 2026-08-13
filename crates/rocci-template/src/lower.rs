use crate::ast::{
    Attr, AttrValue, ComponentCall, ComponentDecl, Document, Element, ForDirective, Fragment,
    IfDirective, Interpolation, MatchDirective, ModuleItem, TemplateBlock, TemplateItem,
    extra_param_names,
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
}

#[derive(Clone, Debug)]
pub struct ComponentInfo {
    pub name: String,
    pub body_params: Vec<String>,
    pub span: Span,
}

pub fn lower(source: SourceFile<'_>, document: &Document, options: &LowerOptions) -> LoweredModule {
    let mut emitter = Emitter {
        src: source.src,
        html: &options.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        components: Vec::new(),
    };
    for item in &document.items {
        match item {
            ModuleItem::Roc { span } => emitter.emit_source(*span, OriginKind::OrdinaryRoc),
            ModuleItem::Component(component) => emitter.lower_component(component),
        }
    }
    if !emitter.roc.ends_with('\n') && !emitter.roc.is_empty() {
        emitter.roc.push('\n');
    }
    LoweredModule {
        roc: emitter.roc,
        segments: emitter.segments,
        components: emitter.components,
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
}

impl<'a> Emitter<'a> {
    fn lower_component(&mut self, component: &ComponentDecl) {
        let body_params = extra_param_names(self.src, component.params);
        self.components.push(ComponentInfo {
            name: component.name.name.clone(),
            body_params: body_params.clone(),
            span: component.span,
        });

        self.emit_mapped(
            &component.name.name,
            component.name.span,
            OriginKind::ComponentSignature,
        );
        self.emit(" = ");
        self.emit_mapped(
            component.params.of(self.src).trim(),
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
        self.lower_props(&call.attrs);
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

    fn lower_props(&mut self, attrs: &[Attr]) {
        if attrs.is_empty() {
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
                AttrValue::Boolean => {
                    self.emit_mapped(&attr.name.name, attr.name.span, OriginKind::ComponentTag);
                    self.emit(": Bool.true");
                }
            }
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
        if groups.len() == 1 {
            match &groups[0] {
                ChildGroup::Nodes(group_items) => self.emit_node_array(group_items, body_params),
                ChildGroup::List(item) => self.lower_item(item, body_params, ValueCtx::List),
            }
            return;
        }
        self.emit("List.concat([\n");
        self.indent += 1;
        for group in &groups {
            self.push_indent();
            match group {
                ChildGroup::Nodes(group_items) => self.emit_node_array(group_items, body_params),
                ChildGroup::List(item) => self.lower_item(item, body_params, ValueCtx::List),
            }
            self.emit(",\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("])");
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
