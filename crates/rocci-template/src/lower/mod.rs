use std::collections::HashMap;

use crate::ast::{
    Document, LeadingComments, ModuleItem, RouteDecl, TemplateItem, component_props_type_anno,
    parse_component_params,
};
use crate::resolve::pascal_to_camel;
use crate::source_map::{OriginKind, Segment};
use crate::span::{SourceFile, Span};

mod emitter;
mod html;
mod routes;

pub use emitter::file_scope_id;
pub use routes::route_fn_name;

use emitter::{
    Emitter, ValueCtx, document_has_action, document_imports_datastar, file_stem,
    items_have_action, scope_css,
};

#[derive(Clone, Debug)]
pub struct LowerOptions {
    pub html_module: String,
    pub html_type: String,
    pub theme_css: Option<String>,
    pub theme_id: Option<String>,
    pub color_scheme_attr: Option<String>,
    pub embed_css: bool,
    pub stylesheet_href: Option<String>,
    pub scope_file_css: bool,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            html_module: "Html".to_string(),
            html_type: "Html".to_string(),
            theme_css: None,
            theme_id: None,
            color_scheme_attr: None,
            embed_css: true,
            stylesheet_href: None,
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
    pub tests: Vec<TestInfo>,
    pub styles: Vec<StyleArtifact>,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub lives: Vec<LiveInfo>,
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
pub struct TestInfo {
    pub name: String,
    pub fixture: Option<String>,
    pub expr: String,
    pub docs: Option<String>,
    pub span: Span,
}

pub fn format_expect_trailer(tests: &[TestInfo]) -> String {
    let mut out = String::new();
    for test in tests {
        if test.expr.trim().is_empty() {
            continue;
        }
        if let Some(docs) = &test.docs {
            out.push_str(docs);
            if !docs.ends_with('\n') {
                out.push('\n');
            }
        }
        let expr = test.expr.trim();
        if expr.contains('\n') {
            out.push_str("expect ");
            out.push_str(expr);
            if !expr.ends_with('\n') {
                out.push('\n');
            }
        } else {
            out.push_str("expect ");
            out.push_str(expr);
            out.push('\n');
        }
    }
    out
}

pub(crate) fn test_docs(src: &str, leading: &Option<LeadingComments>) -> Option<String> {
    let leading = leading.as_ref()?;
    if leading.docs.is_empty() {
        return None;
    }
    let mut out = String::new();
    for span in &leading.docs {
        out.push_str(span.of(src).trim_end());
        out.push('\n');
    }
    Some(out)
}

#[derive(Clone, Debug)]
pub struct InitInfo {
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LiveInfo {
    pub method: String,
    pub path: String,
    pub fn_name: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RespondKind {
    #[default]
    Document,
    Fragment,
    Command,
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
        html_type: &options.html_type,
        roc: String::new(),
        segments: Vec::new(),
        indent,
        at_line_start: false,
        components: Vec::new(),
        fixtures: Vec::new(),
        tests: Vec::new(),
        styles: Vec::new(),
        state_type: None,
        init: None,
        lives: Vec::new(),
        routes: Vec::new(),
        field_defaults: field_defaults.clone(),
        file_css: String::new(),
        file_scope_id: None,
        css_stamp,
        theme_css: options.theme_css.clone(),
        theme_id: options.theme_id.clone(),
        color_scheme_attr: options.color_scheme_attr.clone(),
        embed_css: options.embed_css,
        stylesheet_href: options.stylesheet_href.clone(),
        inject_live_path: None,
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
pub fn lower(source: SourceFile<'_>, document: &Document, options: &LowerOptions) -> LoweredModule {
    let mut field_defaults = HashMap::new();
    for item in &document.items {
        if let ModuleItem::Component(component) = item {
            let parsed = parse_component_params(source.src, component.params);
            if component_props_type_anno(&parsed).is_some() {
                continue;
            }
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
            scope_css(&file_css, id, !options.embed_css)
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
    let local_live_paths = document
        .items
        .iter()
        .filter_map(|item| match item {
            ModuleItem::Route(RouteDecl::Live(live)) => Some((live.path.clone(), live.path_span)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let inject_live_path = match local_live_paths.as_slice() {
        [path] => Some(path.clone()),
        _ => None,
    };
    let mut emitter = Emitter {
        src: source.src,
        file_name: source.name,
        html: &options.html_module,
        html_type: &options.html_type,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        components: Vec::new(),
        fixtures: Vec::new(),
        tests: Vec::new(),
        styles,
        state_type: None,
        init: None,
        lives: Vec::new(),
        routes: Vec::new(),
        field_defaults,
        file_css,
        file_scope_id,
        css_stamp: None,
        theme_css: options.theme_css.clone(),
        theme_id: options.theme_id.clone(),
        color_scheme_attr: options.color_scheme_attr.clone(),
        embed_css: options.embed_css,
        stylesheet_href: options.stylesheet_href.clone(),
        inject_live_path,
    };
    let inject_datastar = (document_has_action(document) || emitter.inject_live_path.is_some())
        && !document_imports_datastar(source.src, document);
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
            ModuleItem::Test(test) => emitter.lower_test(test),
            ModuleItem::Css(css) => emitter.emit_css_leading(css),
            ModuleItem::Context(context) => emitter.lower_context(context),
            ModuleItem::Init(init) => emitter.lower_init(init),
            ModuleItem::Route(route) => match route {
                RouteDecl::Live(live) => emitter.lower_live(live),
                RouteDecl::View(view) => emitter.lower_view(view),
                RouteDecl::Fragment(fragment) => emitter.lower_fragment_decl(fragment),
                RouteDecl::Command(command) => emitter.lower_command(command),
            },
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
        tests: emitter.tests,
        styles: emitter.styles,
        state_type: emitter.state_type,
        init: emitter.init,
        lives: emitter.lives,
        routes: emitter.routes,
    }
}
