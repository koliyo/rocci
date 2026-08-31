use std::collections::HashMap;

use rocci_template::{
    ComponentInfo, Diagnostic, Document as RocciDocument, FixtureInfo, InitInfo, LiveInfo,
    ModuleItem, OriginKind, RouteInfo, Segment, SourceFile, Span, StyleArtifact, file_scope_id,
    route_fn_name, template_items_have_action, validate, validate_template_items,
};

use crate::CompileOptions;
use crate::ast::{Document, HeadingInfo, Item, PageMeta};
use crate::page::{extract_page, imports_html, roc_binding_names, split_roc_body};

mod docs_kind;
mod emitter;
mod islands;
mod markdown;

use emitter::Emitter;
use islands::filter_snapshot_roc;
pub(crate) use islands::island_item_count;

const META_NAME: &str = "rocci_meta";
const CONTENT_NAME: &str = "rocci_content";
const PAGE_NAME: &str = "rocci_page";
const ISLANDS_NAME: &str = "rocci_islands";

pub(crate) fn is_site_chrome_layout(layout: &str) -> bool {
    matches!(
        layout,
        "home"
            | "faq"
            | "product"
            | "section"
            | "docs"
            | "news-index"
            | "news-post"
            | "plain"
            | "not-found"
            | "playground"
    )
}

pub struct Lowered {
    pub roc: String,
    pub segments: Vec<Segment>,
    pub components: Vec<ComponentInfo>,
    pub fixtures: Vec<FixtureInfo>,
    pub styles: Vec<StyleArtifact>,
    pub state_type: Option<String>,
    pub init: Option<InitInfo>,
    pub lives: Vec<LiveInfo>,
    pub routes: Vec<RouteInfo>,
    pub page_meta: PageMeta,
    pub theme: Option<rocci_theme::ResolvedTheme>,
}

pub fn lower(
    source: SourceFile<'_>,
    document: &Document,
    headings: &[HeadingInfo],
    options: &CompileOptions,
    diagnostics: &mut Vec<Diagnostic>,
) -> Lowered {
    let mut page_meta = PageMeta::default();
    let mut page_count = 0;
    let mut page_span = Span::point(0);
    for item in &document.items {
        if let Item::Page(page) = item {
            page_count += 1;
            if page_count > 1 {
                diagnostics.push(Diagnostic::error(
                    page.span,
                    "duplicate `@page`; a document may declare page metadata once",
                ));
            } else {
                page_span = page.span;
                page_meta = extract_page(source.src, page.body, diagnostics);
            }
        }
        if let Item::Roc(roc) = item {
            for (name, span) in roc_binding_names(source.src, roc.body) {
                let reserved = matches!(
                    name.as_str(),
                    META_NAME | CONTENT_NAME | PAGE_NAME | ISLANDS_NAME
                );
                if reserved {
                    diagnostics.push(Diagnostic::error(
                        span,
                        format!("`{name}` is reserved for generated Rocdown exports"),
                    ));
                }
            }
        }
    }

    let mut rocci_items = Vec::new();
    for item in &document.items {
        match item {
            Item::Component(decl) => rocci_items.push(ModuleItem::Component(decl.clone())),
            Item::Fixture(decl) => rocci_items.push(ModuleItem::Fixture(decl.clone())),
            Item::Css(decl) => rocci_items.push(ModuleItem::Css(decl.clone())),
            Item::Context(decl) => rocci_items.push(ModuleItem::Context(decl.clone())),
            Item::Init(decl) => rocci_items.push(ModuleItem::Init(decl.clone())),
            Item::Live(decl) => rocci_items.push(ModuleItem::Route(
                rocci_template::RouteDecl::Live(decl.clone()),
            )),
            Item::View(decl) => rocci_items.push(ModuleItem::Route(
                rocci_template::RouteDecl::View(decl.clone()),
            )),
            Item::Fragment(decl) => rocci_items.push(ModuleItem::Route(
                rocci_template::RouteDecl::Fragment(decl.clone()),
            )),
            Item::Command(decl) => rocci_items.push(ModuleItem::Route(
                rocci_template::RouteDecl::Command(decl.clone()),
            )),
            _ => {}
        }
    }
    let rocci_doc = RocciDocument {
        items: rocci_items,
        span: document.span,
    };
    validate(source.src, &rocci_doc, diagnostics);
    for item in &document.items {
        if let Item::Template(template) = item {
            validate_template_items(std::slice::from_ref(template), diagnostics);
        }
    }

    let resolved_theme = match rocci_theme::resolve(
        page_meta.theme.as_deref(),
        page_meta.color_scheme.as_deref(),
        &options.theme,
    ) {
        Ok(theme) => Some(theme),
        Err(err) => {
            diagnostics.push(Diagnostic::error(page_span, err.to_string()));
            None
        }
    };
    let mut lower_opts = options.lower.clone();
    if let Some(theme) = resolved_theme.as_ref().filter(|theme| !theme.is_none()) {
        lower_opts.theme_css = Some(theme.css.clone());
        lower_opts.theme_id = Some(theme.id.clone());
        lower_opts.color_scheme_attr = theme.policy.html_attr().map(str::to_string);
    }
    let lowered_rocci = rocci_template::lower(source, &rocci_doc, &lower_opts);
    let used_modules = crate::imports::compile_modules(source, document, &lower_opts, diagnostics);

    let css_stamp = if lowered_rocci
        .styles
        .iter()
        .any(|style| matches!(style.kind, rocci_template::StyleKind::File))
        || used_modules.iter().any(|module| {
            module
                .styles
                .iter()
                .any(|style| matches!(style.kind, rocci_template::StyleKind::File))
        }) {
        Some(file_scope_id(source.name))
    } else {
        None
    };

    let mut field_defaults: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for component in &lowered_rocci.components {
        let parsed = rocci_template::ParsedParams {
            first_param_is_record: component.first_param_is_record,
            param_names: component.param_names.clone(),
            optional_params: component.optional_params.clone(),
            param_defaults: component.param_defaults.clone(),
            param_types: component.param_types.clone(),
            body_params: component.body_params.clone(),
        };
        if rocci_template::component_props_type_anno(&parsed).is_none() {
            field_defaults.insert(component.name.clone(), component.param_defaults.clone());
        }
    }
    let mut imported_kinds = HashMap::new();
    for module in &used_modules {
        for (name, defaults) in &module.defaults {
            field_defaults.insert(name.clone(), defaults.clone());
        }
        for kind in &module.kinds {
            imported_kinds.insert(kind.kind.clone(), kind.clone());
        }
    }
    let page_datastar = document.items.iter().any(|item| match item {
        Item::Template(template) => template_items_have_action(std::slice::from_ref(template)),
        _ => false,
    });

    let mut emitter = Emitter {
        source,
        options: &lower_opts,
        html: &lower_opts.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        css_stamp,
        field_defaults,
        imported_kinds,
        theme: resolved_theme.clone(),
        diagnostics,
        resolve_includes: options.resolve_includes,
    };

    let mut imports = Vec::new();
    let mut rest = Vec::new();
    for item in &document.items {
        if let Item::Roc(roc) = item {
            let (imp, body) = split_roc_body(source.src, roc.body);
            imports.extend(imp);
            rest.extend(body);
        }
    }
    let injected_html = !imports_html(source.src, &imports);
    if injected_html {
        emitter.emit("import Html\n");
    }
    let mut template_roc = lowered_rocci.roc;
    let mut has_datastar = false;
    if template_roc.starts_with("import Datastar\n") {
        has_datastar = true;
        template_roc = template_roc
            .strip_prefix("import Datastar\n")
            .unwrap_or(&template_roc)
            .trim_start_matches('\n')
            .to_string();
    }
    if used_modules.iter().any(|module| module.has_datastar) {
        has_datastar = true;
    }
    if has_datastar || page_datastar {
        emitter.emit("import Datastar\n");
    }
    for span in &imports {
        let text = span.of(source.src).trim_end();
        if !text.is_empty() {
            emitter.emit_source(*span, text, OriginKind::RocBlock);
            if !text.ends_with('\n') {
                emitter.emit("\n");
            }
        }
    }
    if injected_html || !imports.is_empty() || has_datastar || page_datastar {
        emitter.emit("\n");
    }
    for span in &rest {
        let text = span.of(source.src).trim_end();
        if text.is_empty() {
            continue;
        }
        emitter.emit_source(*span, text, OriginKind::RocBlock);
        if !text.ends_with('\n') {
            emitter.emit("\n");
        }
        emitter.emit("\n");
    }
    if !template_roc.trim().is_empty() {
        let template_start = emitter.roc.len();
        emitter.emit(template_roc.trim_start());
        if !emitter.roc.ends_with('\n') {
            emitter.emit("\n");
        }
        emitter.emit("\n");
        for mut segment in lowered_rocci.segments {
            segment.generated.start += template_start as u32;
            segment.generated.end += template_start as u32;
            emitter.segments.push(segment);
        }
    }
    for module in &used_modules {
        if module.roc.trim().is_empty() {
            continue;
        }
        emitter.emit(module.roc.trim_start());
        if !emitter.roc.ends_with('\n') {
            emitter.emit("\n");
        }
        emitter.emit("\n");
    }

    emitter.emit(META_NAME);
    emitter.emit(" = ");
    if let Some(span) = page_meta.meta {
        emitter.emit_mapped(span.of(source.src).trim(), span, OriginKind::PageRoc);
    } else {
        emitter.emit("{}");
    }
    emitter.emit("\n\n");

    emitter.emit(CONTENT_NAME);
    emitter.emit(" = |{}| {\n");
    emitter.indent += 1;
    emitter.emit_content_lets(document);
    emitter.push_indent();
    emitter.lower_content_value(document);
    emitter.emit("\n");
    emitter.indent -= 1;
    emitter.push_indent();
    emitter.emit("}\n\n");

    emitter.emit(PAGE_NAME);
    emitter.emit(" = |{}| {\n");
    emitter.indent += 1;
    emitter.push_indent();
    if let Some(layout) = &page_meta.layout
        && !is_site_chrome_layout(layout)
    {
        emitter.emit(layout);
        emitter.emit("({ meta: rocci_meta, content: rocci_content({}) })\n");
    } else {
        emitter.emit_default_page(&page_meta, &lowered_rocci.styles, headings);
    }
    emitter.indent -= 1;
    emitter.push_indent();
    emitter.emit("}\n");

    let mut routes = lowered_rocci.routes;
    let page_route = page_meta
        .route
        .clone()
        .or_else(|| options.default_route.clone())
        .unwrap_or_else(|| "/".to_string());
    let has_get = routes
        .iter()
        .any(|route| route.method == "GET" && route.path == page_route);
    if !has_get {
        let fn_name = route_fn_name("get", &page_route);
        emitter.emit("\n");
        emitter.emit(&fn_name);
        emitter.emit(" = |_state, _request| {\n");
        emitter.indent += 1;
        emitter.push_indent();
        emitter.emit("rocci_value = {\n");
        emitter.indent += 1;
        emitter.push_indent();
        emitter.emit("rocci_page({})\n");
        emitter.indent -= 1;
        emitter.push_indent();
        emitter.emit("}\n");
        emitter.push_indent();
        emitter.emit("Ok(rocci_value)\n");
        emitter.indent -= 1;
        emitter.push_indent();
        emitter.emit("}\n");
        routes.push(RouteInfo {
            method: "GET".to_string(),
            path: page_route.clone(),
            fn_name: fn_name.clone(),
            respond: rocci_template::RespondKind::Document,
            span: document.span,
        });
        if page_route != "/"
            && !routes
                .iter()
                .any(|route| route.method == "GET" && route.path == "/")
        {
            routes.push(RouteInfo {
                method: "GET".to_string(),
                path: "/".to_string(),
                fn_name,
                respond: rocci_template::RespondKind::Document,
                span: document.span,
            });
        }
    }

    if !emitter.roc.ends_with('\n') {
        emitter.roc.push('\n');
    }

    let segments = emitter.segments;
    let mut styles = lowered_rocci.styles;
    for module in used_modules {
        styles.extend(module.styles);
    }
    if let Some(theme) = resolved_theme.as_ref().filter(|theme| !theme.is_none()) {
        styles.insert(
            0,
            StyleArtifact {
                kind: rocci_template::StyleKind::Theme,
                name: theme.id.clone(),
                css: theme.css.clone(),
                span: Span::point(0),
            },
        );
    }

    Lowered {
        roc: emitter.roc,
        segments,
        components: lowered_rocci.components,
        fixtures: lowered_rocci.fixtures,
        styles,
        state_type: lowered_rocci.state_type,
        init: lowered_rocci.init,
        lives: lowered_rocci.lives,
        routes,
        page_meta,
        theme: resolved_theme,
    }
}

pub fn lower_islands(
    source: SourceFile<'_>,
    document: &Document,
    options: &CompileOptions,
    diagnostics: &mut Vec<Diagnostic>,
) -> Lowered {
    let mut page_meta = PageMeta::default();
    let mut page_count = 0;
    let mut page_span = Span::point(0);
    for item in &document.items {
        if let Item::Page(page) = item {
            page_count += 1;
            if page_count > 1 {
                diagnostics.push(Diagnostic::error(
                    page.span,
                    "duplicate `@page`; a document may declare page metadata once",
                ));
            } else {
                page_span = page.span;
                page_meta = extract_page(source.src, page.body, diagnostics);
            }
        }
        if let Item::Roc(roc) = item {
            for (name, span) in roc_binding_names(source.src, roc.body) {
                let reserved = matches!(
                    name.as_str(),
                    META_NAME | CONTENT_NAME | PAGE_NAME | ISLANDS_NAME
                );
                if reserved {
                    diagnostics.push(Diagnostic::error(
                        span,
                        format!("`{name}` is reserved for generated Rocdown exports"),
                    ));
                }
            }
        }
    }

    let mut rocci_items = Vec::new();
    for item in &document.items {
        match item {
            Item::Component(decl) => rocci_items.push(ModuleItem::Component(decl.clone())),
            Item::Fixture(decl) => rocci_items.push(ModuleItem::Fixture(decl.clone())),
            Item::Css(decl) => rocci_items.push(ModuleItem::Css(decl.clone())),
            _ => {}
        }
    }
    let rocci_doc = RocciDocument {
        items: rocci_items,
        span: document.span,
    };
    validate(source.src, &rocci_doc, diagnostics);
    for item in &document.items {
        if let Item::Template(template) = item {
            validate_template_items(std::slice::from_ref(template), diagnostics);
        }
    }

    if let Err(err) = rocci_theme::resolve(
        page_meta.theme.as_deref(),
        page_meta.color_scheme.as_deref(),
        &options.theme,
    ) {
        diagnostics.push(Diagnostic::error(page_span, err.to_string()));
    }
    let mut lower_opts = options.lower.clone();
    lower_opts.theme_css = None;
    lower_opts.theme_id = None;
    lower_opts.color_scheme_attr = None;
    let lowered_rocci = rocci_template::lower(source, &rocci_doc, &lower_opts);

    let css_stamp = if lowered_rocci
        .styles
        .iter()
        .any(|style| matches!(style.kind, rocci_template::StyleKind::File))
    {
        Some(file_scope_id(source.name))
    } else {
        None
    };

    let mut emitter = Emitter {
        source,
        options: &lower_opts,
        html: &lower_opts.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        css_stamp,
        field_defaults: HashMap::new(),
        imported_kinds: HashMap::new(),
        theme: None,
        diagnostics,
        resolve_includes: options.resolve_includes,
    };

    let mut imports = Vec::new();
    let mut rest = Vec::new();
    for item in &document.items {
        if let Item::Roc(roc) = item {
            let (imp, body) = split_roc_body(source.src, roc.body);
            imports.extend(imp);
            rest.extend(body);
        }
    }
    let (imports, rest) = filter_snapshot_roc(source, document, imports, rest);
    let page_datastar = document.items.iter().any(|item| match item {
        Item::Template(template) => template_items_have_action(std::slice::from_ref(template)),
        _ => false,
    });
    let injected_html = !imports_html(source.src, &imports);
    if injected_html {
        emitter.emit("import Html\n");
    }
    let mut template_roc = lowered_rocci.roc;
    let mut has_datastar = false;
    if template_roc.starts_with("import Datastar\n") {
        has_datastar = true;
        template_roc = template_roc
            .strip_prefix("import Datastar\n")
            .unwrap_or(&template_roc)
            .trim_start_matches('\n')
            .to_string();
    }
    if has_datastar || page_datastar {
        emitter.emit("import Datastar\n");
    }
    for span in &imports {
        let text = span.of(source.src).trim_end();
        if !text.is_empty() {
            emitter.emit_source(*span, text, OriginKind::RocBlock);
            if !text.ends_with('\n') {
                emitter.emit("\n");
            }
        }
    }
    if injected_html || !imports.is_empty() || has_datastar || page_datastar {
        emitter.emit("\n");
    }
    for span in &rest {
        let text = span.of(source.src).trim_end();
        if text.is_empty() {
            continue;
        }
        emitter.emit_source(*span, text, OriginKind::RocBlock);
        if !text.ends_with('\n') {
            emitter.emit("\n");
        }
        emitter.emit("\n");
    }
    if !template_roc.trim().is_empty() {
        let template_start = emitter.roc.len();
        emitter.emit(template_roc.trim_start());
        if !emitter.roc.ends_with('\n') {
            emitter.emit("\n");
        }
        emitter.emit("\n");
        for mut segment in lowered_rocci.segments {
            segment.generated.start += template_start as u32;
            segment.generated.end += template_start as u32;
            emitter.segments.push(segment);
        }
    }

    emitter.emit(ISLANDS_NAME);
    emitter.emit(" = |{}| {\n");
    emitter.indent += 1;
    emitter.emit_content_lets(document);
    emitter.push_indent();
    emitter.emit_island_list(document);
    emitter.emit("\n");
    emitter.indent -= 1;
    emitter.push_indent();
    emitter.emit("}\n");

    if !emitter.roc.ends_with('\n') {
        emitter.roc.push('\n');
    }

    Lowered {
        roc: emitter.roc,
        segments: emitter.segments,
        components: lowered_rocci.components,
        fixtures: lowered_rocci.fixtures,
        styles: lowered_rocci.styles,
        state_type: None,
        init: None,
        lives: Vec::new(),
        routes: Vec::new(),
        page_meta,
        theme: None,
    }
}
