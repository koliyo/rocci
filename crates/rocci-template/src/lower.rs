use std::collections::HashMap;

use crate::ast::{
    Attr, AttrValue, CommandDecl, ComponentCall, ComponentDecl, ContextDecl, CssDecl, Document,
    Element, FixtureDecl, ForDirective, Fragment, Ident, IfDirective, InitDecl, Interpolation,
    LiveDecl, MatchDirective, ModuleItem, PatchDecl, TemplateBlock, TemplateItem, ViewDecl,
    ensure_handler_request_param, parse_component_params, strip_param_defaults,
};
use crate::resolve::pascal_to_camel;
use crate::source_map::{OriginKind, Segment};
use crate::span::{SourceFile, Span};

#[derive(Clone, Debug)]
pub struct LowerOptions {
    pub html_module: String,
    pub theme_css: Option<String>,
    pub theme_id: Option<String>,
    pub color_scheme_attr: Option<String>,
    pub embed_css: bool,
    pub scope_file_css: bool,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            html_module: "Html".to_string(),
            theme_css: None,
            theme_id: None,
            color_scheme_attr: None,
            embed_css: true,
            scope_file_css: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoweredModule {
    pub roc: String,
    pub segments: Vec<Segment>,
    pub components: Vec<ComponentInfo>,
    pub fixtures: Vec<FixtureInfo>,
    pub styles: Vec<StyleArtifact>,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub live: Option<LiveInfo>,
    pub routes: Vec<RouteInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleKind {
    Theme,
    File,
    Component,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleArtifact {
    pub kind: StyleKind,
    pub name: String,
    pub css: String,
    pub span: Span,
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

#[derive(Clone, Debug)]
pub struct InitInfo {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LiveInfo {
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RespondKind {
    #[default]
    Patch,
    Json,
}

#[derive(Clone, Debug)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub fn_name: String,
    pub respond: RespondKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateValueCtx {
    Node,
    List,
}

#[derive(Clone, Debug)]
pub struct LoweredTemplate {
    pub roc: String,
    pub segments: Vec<Segment>,
    pub uses_datastar: bool,
}

pub fn lower_template_items(
    source: SourceFile<'_>,
    items: &[TemplateItem],
    options: &LowerOptions,
    field_defaults: &HashMap<String, Vec<(String, String)>>,
    indent: usize,
    ctx: TemplateValueCtx,
    css_stamp: Option<String>,
) -> LoweredTemplate {
    let uses_datastar = items_have_action(items);
    let mut emitter = Emitter {
        src: source.src,
        file_name: source.name,
        html: &options.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent,
        at_line_start: false,
        components: Vec::new(),
        fixtures: Vec::new(),
        styles: Vec::new(),
        state_type: None,
        init: None,
        live: None,
        routes: Vec::new(),
        field_defaults: field_defaults.clone(),
        file_css: String::new(),
        file_scope_id: None,
        css_stamp,
        theme_css: options.theme_css.clone(),
        theme_id: options.theme_id.clone(),
        color_scheme_attr: options.color_scheme_attr.clone(),
        embed_css: options.embed_css,
        inject_live_init: false,
    };
    match items {
        [] => emitter.emit_html(".empty"),
        [item] => emitter.lower_item(
            item,
            &[],
            match ctx {
                TemplateValueCtx::Node => ValueCtx::Node,
                TemplateValueCtx::List => ValueCtx::List,
            },
        ),
        _ => emitter.lower_html_value(items, &[]),
    }
    LoweredTemplate {
        roc: emitter.roc,
        segments: emitter.segments,
        uses_datastar,
    }
}

pub fn template_items_have_action(items: &[TemplateItem]) -> bool {
    items_have_action(items)
}

pub fn route_fn_name(method: &str, path: &str) -> String {
    let method = method.to_ascii_lowercase();
    let path_part = if path.is_empty() || path == "/" {
        "root".to_string()
    } else {
        path.trim_matches('/')
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
            .collect::<String>()
    };
    format!("on_{method}_{path_part}!")
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
            field_defaults.insert(pascal_to_camel(&component.name.name), defaults);
        }
    }
    let file_css_parts: Vec<(String, Span)> = document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Css(css) => {
                let text = css.body.of(source.src);
                if text.trim().is_empty() {
                    None
                } else {
                    Some((text.to_string(), css.span))
                }
            }
            _ => None,
        })
        .collect();
    let file_css = file_css_parts
        .iter()
        .map(|(text, _)| text.trim())
        .collect::<Vec<_>>()
        .join("\n\n");
    let file_scope_id = if file_css.is_empty() || !options.scope_file_css {
        None
    } else {
        Some(file_scope_id(source.name))
    };
    let mut styles = Vec::new();
    if !file_css.is_empty() {
        let css = if let Some(id) = &file_scope_id {
            scope_css(&file_css, id)
        } else {
            file_css.clone()
        };
        styles.push(StyleArtifact {
            kind: StyleKind::File,
            name: file_stem(source.name),
            css,
            span: file_css_parts[0].1,
        });
    }
    let mut emitter = Emitter {
        src: source.src,
        file_name: source.name,
        html: &options.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        components: Vec::new(),
        fixtures: Vec::new(),
        styles,
        state_type: None,
        init: None,
        live: None,
        routes: Vec::new(),
        field_defaults,
        file_css,
        file_scope_id,
        css_stamp: None,
        theme_css: options.theme_css.clone(),
        theme_id: options.theme_id.clone(),
        color_scheme_attr: options.color_scheme_attr.clone(),
        embed_css: options.embed_css,
        inject_live_init: document
            .items
            .iter()
            .any(|item| matches!(item, ModuleItem::Live(_))),
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
            ModuleItem::Css(_) => {}
            ModuleItem::Context(context) => emitter.lower_context(context),
            ModuleItem::Init(init) => emitter.lower_init(init),
            ModuleItem::Live(live) => emitter.lower_live(live),
            ModuleItem::View(view) => emitter.lower_view(view),
            ModuleItem::Patch(patch) => emitter.lower_patch(patch),
            ModuleItem::Command(command) => emitter.lower_command(command),
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
        styles: emitter.styles,
        state_type: emitter.state_type,
        init: emitter.init,
        live: emitter.live,
        routes: emitter.routes,
    }
}

struct Emitter<'a> {
    src: &'a str,
    file_name: &'a str,
    html: &'a str,
    roc: String,
    segments: Vec<Segment>,
    indent: usize,
    at_line_start: bool,
    components: Vec<ComponentInfo>,
    fixtures: Vec<FixtureInfo>,
    styles: Vec<StyleArtifact>,
    state_type: Option<String>,
    init: Option<InitInfo>,
    live: Option<LiveInfo>,
    routes: Vec<RouteInfo>,
    field_defaults: HashMap<String, Vec<(String, String)>>,
    file_css: String,
    file_scope_id: Option<String>,
    css_stamp: Option<String>,
    theme_css: Option<String>,
    theme_id: Option<String>,
    color_scheme_attr: Option<String>,
    embed_css: bool,
    inject_live_init: bool,
}

impl<'a> Emitter<'a> {
    fn lower_component(&mut self, component: &ComponentDecl) {
        let parsed = parse_component_params(self.src, component.params);
        let body_params = parsed.body_params.clone();
        let roc_name = pascal_to_camel(&component.name.name);
        self.components.push(ComponentInfo {
            name: roc_name.clone(),
            body_params: body_params.clone(),
            param_names: parsed.param_names,
            optional_params: parsed.optional_params,
            param_defaults: parsed.param_defaults,
            param_types: parsed.param_types,
            first_param_is_record: parsed.first_param_is_record,
            span: component.span,
        });

        self.emit_mapped(
            &roc_name,
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
                css: scope_css(&component_css, id),
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
                self.lower_html_value_with_style(rest, &body_params, &css);
            } else if self.theme_css.is_some() && is_html_document(rest) {
                self.lower_html_value_with_style(rest, &body_params, "");
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

    fn lower_context(&mut self, context: &ContextDecl) {
        let ty = context.ty.of(self.src).trim();
        if self.state_type.is_none() {
            self.state_type = Some(ty.to_string());
        }
        self.emit("State : ");
        self.emit_mapped(ty, context.ty, OriginKind::OrdinaryRoc);
        self.emit("\n");
    }

    fn lower_init(&mut self, init: &InitDecl) {
        self.init = Some(InitInfo { span: init.span });
        self.emit("init! = || {\n");
        self.indent += 1;
        self.emit_try_block(init.body, "rocci_state");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    fn lower_live(&mut self, live: &LiveDecl) {
        self.live = Some(LiveInfo { span: live.span });
        let params = live
            .params
            .map(|span| {
                ensure_handler_request_param(&strip_param_defaults(span.of(self.src).trim()))
            })
            .unwrap_or_else(|| "|state, _request|".to_string());
        self.emit_mapped("live!", live.span, OriginKind::OrdinaryRoc);
        self.emit(" = ");
        if let Some(span) = live.params {
            self.emit_mapped(&params, span, OriginKind::OrdinaryRoc);
        } else {
            self.emit(&params);
        }
        self.emit(" {\n");
        self.indent += 1;
        self.emit_try_block(live.body, "rocci_value");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    fn lower_view(&mut self, view: &ViewDecl) {
        self.lower_route(
            "GET",
            &view.path,
            RespondKind::Patch,
            view.params,
            view.body,
            view.span,
        );
    }

    fn lower_patch(&mut self, patch: &PatchDecl) {
        let method = patch
            .method
            .as_ref()
            .map(|ident| ident.name.as_str())
            .unwrap_or("post");
        self.lower_route(
            method,
            &patch.path,
            RespondKind::Patch,
            patch.params,
            patch.body,
            patch.span,
        );
    }

    fn lower_command(&mut self, command: &CommandDecl) {
        let method = command
            .method
            .as_ref()
            .map(|ident| ident.name.as_str())
            .unwrap_or("post");
        self.lower_route(
            method,
            &command.path,
            RespondKind::Json,
            command.params,
            command.body,
            command.span,
        );
    }

    fn lower_route(
        &mut self,
        method: &str,
        path: &str,
        respond: RespondKind,
        params: Option<Span>,
        body: Span,
        span: Span,
    ) {
        let method_upper = method.to_ascii_uppercase();
        let fn_name = route_fn_name(method, path);
        self.routes.push(RouteInfo {
            method: method_upper,
            path: path.to_string(),
            fn_name: fn_name.clone(),
            respond,
            span,
        });
        let adapted = params
            .map(|param_span| {
                ensure_handler_request_param(&strip_param_defaults(param_span.of(self.src).trim()))
            })
            .unwrap_or_else(|| "|state, _request|".to_string());
        self.emit_mapped(&fn_name, span, OriginKind::OrdinaryRoc);
        self.emit(" = ");
        if let Some(param_span) = params {
            self.emit_mapped(&adapted, param_span, OriginKind::OrdinaryRoc);
        } else {
            self.emit(&adapted);
        }
        self.emit(" {\n");
        self.indent += 1;
        self.emit_try_block(body, "rocci_value");
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
    }

    fn emit_try_block(&mut self, body: Span, result_name: &str) {
        let text = body.of(self.src).trim();
        self.push_indent();
        if text.is_empty() {
            self.emit("Ok({})\n");
            return;
        }
        self.emit(result_name);
        self.emit(" = {\n");
        self.indent += 1;
        for line in text.lines() {
            self.push_indent();
            self.emit(line.trim_end());
            self.emit("\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("}\n");
        self.push_indent();
        self.emit("Ok(");
        self.emit(result_name);
        self.emit(")\n");
    }

    fn lower_block(&mut self, block: &TemplateBlock, body_params: &[String]) {
        let (preamble, rest) = split_preamble(&block.items);
        self.emit_lets(preamble);
        self.lower_html_value(rest, body_params);
    }

    fn emit_lets(&mut self, preamble: &[TemplateItem]) {
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

    fn injected_css(&self, component_css: &str, component_id: Option<&str>) -> Option<String> {
        let mut parts = Vec::new();
        if !self.file_css.is_empty()
            && let Some(id) = &self.file_scope_id
        {
            parts.push(scope_css(&self.file_css, id));
        }
        if !component_css.is_empty()
            && let Some(id) = component_id
        {
            parts.push(scope_css(component_css, id));
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n"))
        }
    }

    fn lower_html_value_with_style(
        &mut self,
        items: &[TemplateItem],
        body_params: &[String],
        css: &str,
    ) {
        if let [TemplateItem::Element(el)] = items
            && el.name.name == "html"
            && !el.self_closing
            && !el
                .children
                .iter()
                .any(|item| matches!(item, TemplateItem::For(_)))
        {
            let css = self.prepend_theme_css(css);
            self.lower_html_document_with_style(el, body_params, &css);
            return;
        }
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

    fn lower_html_document_with_style(&mut self, el: &Element, body_params: &[String], css: &str) {
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
                    self.lower_head_with_style(head, body_params, css);
                } else {
                    self.lower_item(item, body_params, ValueCtx::Node);
                }
                self.emit(",\n");
            }
        } else {
            self.push_indent();
            self.lower_synthetic_head(css);
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

    fn lower_head_with_style(&mut self, el: &Element, body_params: &[String], css: &str) {
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
        self.lower_style_element(css);
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

    fn lower_synthetic_head(&mut self, css: &str) {
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
        self.lower_style_element(css);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn lower_style_element(&mut self, css: &str) {
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
            TemplateItem::Let(_) | TemplateItem::Css(_) => {}
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

    fn prepend_theme_css(&self, css: &str) -> String {
        match &self.theme_css {
            Some(theme) if css.is_empty() => theme.clone(),
            Some(theme) => format!("{theme}\n{css}"),
            None => css.to_string(),
        }
    }

    fn lower_html_attrs_with_theme(&mut self, tag: &str, attrs: &[Attr]) {
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

    fn lower_html_attrs(&mut self, tag: &str, attrs: &[Attr]) {
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

    fn should_inject_live_init(&self, tag: &str, attrs: &[Attr]) -> bool {
        self.inject_live_init
            && tag.eq_ignore_ascii_case("body")
            && !attrs.iter().any(|attr| attr.name.name == "data-init")
    }

    fn emit_live_init_attr(&mut self) {
        self.push_indent();
        self.emit_html(".attribute(");
        self.emit_string("data-init", Span::point(0), OriginKind::Scaffolding);
        self.emit(", Datastar.get_with(\"/sse\", [OpenWhenHidden(True)])");
        self.emit("),\n");
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

fn split_preamble(items: &[TemplateItem]) -> (&[TemplateItem], &[TemplateItem]) {
    let count = items.iter().take_while(|item| item.is_preamble()).count();
    (&items[..count], &items[count..])
}

fn concat_css<'a>(src: &'a str, decls: impl Iterator<Item = &'a CssDecl>) -> String {
    decls
        .map(|decl| decl.body.of(src).trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn is_html_document(items: &[TemplateItem]) -> bool {
    matches!(
        items,
        [TemplateItem::Element(el)]
            if el.name.name == "html"
                && !el.self_closing
                && !el.children.iter().any(|item| matches!(item, TemplateItem::For(_)))
    )
}

fn scope_css(css: &str, id: &str) -> String {
    format!("@scope ([data-rocci-css~=\"{id}\"]) {{\n{}\n}}", css.trim())
}

pub fn file_scope_id(file_name: &str) -> String {
    format!(
        "{}-{:08x}",
        file_stem(file_name),
        fnv1a32(file_name.as_bytes())
    )
}

fn component_scope_id(file_name: &str, component: &str) -> String {
    let mut bytes = file_name.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(component.as_bytes());
    format!("{}-{:08x}", sanitize_ident(component), fnv1a32(&bytes))
}

fn file_stem(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = base
        .strip_suffix(".rocci")
        .or_else(|| base.rsplit_once('.').map(|(stem, _)| stem))
        .unwrap_or(base);
    sanitize_ident(stem)
}

fn sanitize_ident(name: &str) -> String {
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

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5;
    for &b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
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
        ModuleItem::Live(_) => true,
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
