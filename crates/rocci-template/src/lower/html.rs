use crate::ast::{
    Attr, AttrValue, ComponentCall, Element, ForDirective, Fragment, Ident, IfDirective,
    Interpolation, MatchDirective, TemplateBlock, TemplateItem,
};
use crate::source_map::OriginKind;
use crate::span::Span;

use super::emitter::{
    ChildGroup, Emitter, ValueCtx, group_children, has_top_level_comma, is_void, scope_css,
    split_preamble,
};

#[derive(Clone, Copy)]
pub(crate) enum HeadInject<'a> {
    EmbeddedStyle(&'a str),
    StylesheetLink(&'a str),
}

impl<'a> Emitter<'a> {
    pub(crate) fn lower_block(&mut self, block: &TemplateBlock, body_params: &[String]) {
        let (preamble, rest) = split_preamble(&block.items);
        self.emit_lets(preamble);
        self.lower_html_value(rest, body_params);
    }

    pub(crate) fn emit_lets(&mut self, preamble: &[TemplateItem]) {
        for item in preamble {
            let TemplateItem::Let(let_dir) = item else {
                continue;
            };
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
    }

    pub(crate) fn injected_css(
        &self,
        component_css: &str,
        component_id: Option<&str>,
    ) -> Option<String> {
        let mut parts = Vec::new();
        if !self.file_css.is_empty()
            && let Some(id) = &self.file_scope_id
        {
            parts.push(scope_css(&self.file_css, id, false));
        }
        if !component_css.is_empty()
            && let Some(id) = component_id
        {
            parts.push(scope_css(component_css, id, false));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    pub(crate) fn lower_html_value_with_head(
        &mut self,
        items: &[TemplateItem],
        body_params: &[String],
        inject: HeadInject<'_>,
    ) {
        if let [TemplateItem::Element(el)] = items
            && el.name.name == "html"
            && !el.self_closing
            && !el
                .children
                .iter()
                .any(|item| matches!(item, TemplateItem::For(_)))
        {
            match inject {
                HeadInject::EmbeddedStyle(css) => {
                    let css = self.prepend_theme_css(css);
                    self.lower_html_document_with_head(
                        el,
                        body_params,
                        HeadInject::EmbeddedStyle(&css),
                    );
                }
                HeadInject::StylesheetLink(href) => {
                    self.lower_html_document_with_head(
                        el,
                        body_params,
                        HeadInject::StylesheetLink(href),
                    );
                }
            }
            return;
        }
        match inject {
            HeadInject::StylesheetLink(_) => {
                self.lower_html_value(items, body_params);
            }
            HeadInject::EmbeddedStyle(css) => {
                self.emit_html(".fragment(\n");
                self.indent += 1;
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.lower_style_element(css);
                self.emit(",\n");
                self.push_indent();
                self.lower_html_value(items, body_params);
                self.emit(",\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
        }
    }

    pub(crate) fn lower_html_document_with_head(
        &mut self,
        el: &Element,
        body_params: &[String],
        inject: HeadInject<'_>,
    ) {
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(&el.name.name, el.name.span, OriginKind::StaticMarkup);
        self.emit(",\n");
        self.push_indent();
        self.lower_html_attrs_with_theme(&el.name.name, &el.attrs);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        let head = el
            .children
            .iter()
            .enumerate()
            .find_map(|(i, item)| match item {
                TemplateItem::Element(head) if head.name.name == "head" => Some((i, head)),
                _ => None,
            });
        if let Some((head_idx, head)) = head {
            for (i, item) in el.children.iter().enumerate() {
                self.push_indent();
                if i == head_idx {
                    self.lower_head_with_inject(head, body_params, inject);
                } else {
                    self.lower_item(item, body_params, ValueCtx::Node);
                }
                self.emit(",\n");
            }
        } else {
            self.push_indent();
            self.lower_synthetic_head(inject);
            self.emit(",\n");
            for item in &el.children {
                self.push_indent();
                self.lower_item(item, body_params, ValueCtx::Node);
                self.emit(",\n");
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_head_with_inject(
        &mut self,
        el: &Element,
        body_params: &[String],
        inject: HeadInject<'_>,
    ) {
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(&el.name.name, el.name.span, OriginKind::StaticMarkup);
        self.emit(",\n");
        self.push_indent();
        self.lower_html_attrs(&el.name.name, &el.attrs);
        self.emit(",\n");
        self.push_indent();
        self.emit("List.concat(\n");
        self.indent += 1;
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.lower_head_inject(inject);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.push_indent();
        self.lower_node_list(&el.children, body_params);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_synthetic_head(&mut self, inject: HeadInject<'_>) {
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit("\"head\",\n");
        self.push_indent();
        self.lower_html_attrs("head", &[]);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.lower_head_inject(inject);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_head_inject(&mut self, inject: HeadInject<'_>) {
        match inject {
            HeadInject::EmbeddedStyle(css) => self.lower_style_element(css),
            HeadInject::StylesheetLink(href) => self.lower_stylesheet_link_element(href),
        }
    }

    pub(crate) fn lower_stylesheet_link_element(&mut self, href: &str) {
        self.emit_html(".void_element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit("\"link\",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".attribute(\"rel\", ");
        self.emit_string("stylesheet", Span::point(0), OriginKind::Scaffolding);
        self.emit("),\n");
        self.push_indent();
        self.emit_html(".attribute(\"href\", ");
        self.emit_string(href, Span::point(0), OriginKind::Scaffolding);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_style_element(&mut self, css: &str) {
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit("\"style\",\n");
        self.push_indent();
        self.emit("[],\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".text(");
        self.emit_string(css, Span::point(0), OriginKind::Css);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_html_value(&mut self, items: &[TemplateItem], body_params: &[String]) {
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

    pub(crate) fn lower_item(
        &mut self,
        item: &TemplateItem,
        body_params: &[String],
        ctx: ValueCtx,
    ) {
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
            TemplateItem::Let(_) | TemplateItem::Css(_) => {}
        }
    }

    pub(crate) fn lower_element(&mut self, el: &Element, body_params: &[String]) {
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
        self.lower_html_attrs(&el.name.name, &el.attrs);
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

    pub(crate) fn prepend_theme_css(&self, css: &str) -> String {
        match &self.theme_css {
            Some(theme) if css.is_empty() => theme.clone(),
            Some(theme) => format!("{theme}\n{css}"),
            None => css.to_string(),
        }
    }

    pub(crate) fn lower_html_attrs_with_theme(&mut self, tag: &str, attrs: &[Attr]) {
        let mut class_emitted = false;
        let stamp = self.css_stamp.clone();
        let theme_id = self.theme_id.clone();
        let scheme = self.color_scheme_attr.clone();
        let inject = self.should_inject_live_init(tag, attrs);
        if attrs.is_empty() && stamp.is_none() && theme_id.is_none() && !inject {
            self.emit("[]");
            return;
        }
        self.emit("[\n");
        self.indent += 1;
        for attr in attrs {
            self.push_indent();
            if attr.name.name == "class" {
                class_emitted = true;
                if let (Some(_), AttrValue::Static { span, value }) = (&theme_id, &attr.value) {
                    let merged = if value.split_whitespace().any(|part| part == "rd-document") {
                        value.clone()
                    } else if value.is_empty() {
                        "rd-document".to_string()
                    } else {
                        format!("{value} rd-document")
                    };
                    self.emit_html(".attribute(");
                    self.emit_string(&attr.name.name, attr.name.span, OriginKind::StaticMarkup);
                    self.emit(", ");
                    self.emit_string(&merged, *span, OriginKind::StaticMarkup);
                    self.emit(")");
                    self.emit(",\n");
                    continue;
                }
            }
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
                    self.emit(", True)");
                }
            }
            self.emit(",\n");
        }
        if let Some(id) = &theme_id {
            if !class_emitted {
                self.push_indent();
                self.emit_html(".attribute(");
                self.emit_string("class", Span::point(0), OriginKind::Scaffolding);
                self.emit(", ");
                self.emit_string("rd-document", Span::point(0), OriginKind::Scaffolding);
                self.emit("),\n");
            }
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("data-rd-theme", Span::point(0), OriginKind::Scaffolding);
            self.emit(", ");
            self.emit_string(id, Span::point(0), OriginKind::Scaffolding);
            self.emit("),\n");
            if let Some(scheme) = &scheme {
                self.push_indent();
                self.emit_html(".attribute(");
                self.emit_string(
                    "data-rd-color-scheme",
                    Span::point(0),
                    OriginKind::Scaffolding,
                );
                self.emit(", ");
                self.emit_string(scheme, Span::point(0), OriginKind::Scaffolding);
                self.emit("),\n");
            }
        }
        if let Some(stamp) = stamp {
            self.push_indent();
            self.emit_html(".attribute(\"data-rocci-css\", ");
            self.emit_string(&stamp, Span::point(0), OriginKind::Scaffolding);
            self.emit(")");
            self.emit(",\n");
        }
        if inject {
            self.emit_live_init_attr();
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    pub(crate) fn lower_html_attrs(&mut self, tag: &str, attrs: &[Attr]) {
        let stamp = self.css_stamp.clone();
        let inject = self.should_inject_live_init(tag, attrs);
        if attrs.is_empty() && stamp.is_none() && !inject {
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
                    self.emit(", True)");
                }
            }
            self.emit(",\n");
        }
        if let Some(stamp) = stamp {
            self.push_indent();
            self.emit_html(".attribute(\"data-rocci-css\", ");
            self.emit_string(&stamp, Span::point(0), OriginKind::Scaffolding);
            self.emit(")");
            self.emit(",\n");
        }
        if inject {
            self.emit_live_init_attr();
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    pub(crate) fn should_inject_live_init(&self, tag: &str, attrs: &[Attr]) -> bool {
        self.inject_live_path.is_some()
            && tag.eq_ignore_ascii_case("body")
            && !attrs.iter().any(|attr| attr.name.name == "data-init")
    }

    pub(crate) fn emit_live_init_attr(&mut self) {
        let (path, path_span) = self
            .inject_live_path
            .clone()
            .expect("live init is emitted only for a singleton local path");
        self.push_indent();
        self.emit_html(".attribute(");
        self.emit_string("data-init", Span::point(0), OriginKind::Scaffolding);
        self.emit(", Datastar.get_with(");
        self.emit_string(&path, path_span, OriginKind::Scaffolding);
        self.emit(", [OpenWhenHidden(True)])");
        self.emit("),\n");
    }

    pub(crate) fn lower_call(&mut self, call: &ComponentCall, body_params: &[String]) {
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

    pub(crate) fn lower_props(&mut self, attrs: &[Attr], roc_name: &str) {
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
                    self.emit(": True");
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

    pub(crate) fn lower_fragment(&mut self, frag: &Fragment, body_params: &[String]) {
        self.emit_html(".fragment(\n");
        self.indent += 1;
        self.push_indent();
        self.lower_node_list(&frag.children, body_params);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    pub(crate) fn lower_interpolation(&mut self, interp: &Interpolation, body_params: &[String]) {
        let expr = interp.expr.of(self.src).trim();
        if body_params.iter().any(|name| name == expr) {
            self.emit_mapped(expr, interp.expr, OriginKind::TextExpression);
            return;
        }
        self.emit_html(".text(");
        self.emit_mapped(expr, interp.expr, OriginKind::TextExpression);
        self.emit(")");
    }

    pub(crate) fn lower_if(&mut self, dir: &IfDirective, body_params: &[String]) {
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

    pub(crate) fn lower_for_map(&mut self, dir: &ForDirective, body_params: &[String]) {
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

    pub(crate) fn lower_match(&mut self, dir: &MatchDirective, body_params: &[String]) {
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

    pub(crate) fn lower_node_list(&mut self, items: &[TemplateItem], body_params: &[String]) {
        if items.is_empty() {
            self.emit("[]");
            return;
        }
        let groups = group_children(items);
        self.emit_concat_groups(&groups, body_params);
    }

    pub(crate) fn emit_concat_groups(&mut self, groups: &[ChildGroup<'_>], body_params: &[String]) {
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

    pub(crate) fn emit_child_group(&mut self, group: &ChildGroup<'_>, body_params: &[String]) {
        match group {
            ChildGroup::Nodes(group_items) => self.emit_node_array(group_items, body_params),
            ChildGroup::List(item) => self.lower_item(item, body_params, ValueCtx::List),
        }
    }

    pub(crate) fn emit_node_array(&mut self, items: &[&TemplateItem], body_params: &[String]) {
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

    pub(crate) fn lower_action_call(&mut self, name: &Ident, args: Span) {
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
}
