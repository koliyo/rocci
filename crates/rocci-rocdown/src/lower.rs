use std::collections::HashMap;

use rocci_template::{
    ComponentInfo, Diagnostic, Document as RocciDocument, FixtureInfo, InitInfo, LowerOptions,
    LoweredTemplate, ModuleItem, OriginKind, RouteInfo, Segment, SourceFile, Span, StyleArtifact,
    TemplateItem, TemplateValueCtx, file_scope_id, lower_template_items, pascal_to_camel,
    route_fn_name, template_items_have_action, validate, validate_template_items,
};

use crate::CompileOptions;
use crate::ast::{
    BlockCall, Document, HeadingInfo, Item, MdNode, PageMeta, ParamValue, RenderDecl,
};
use crate::docs::{
    docs_fields_from_params, extract_lines, extract_region, field_bool, field_string,
    resolve_include_path,
};
use crate::page::{
    extract_page, import_local_name, imports_html, roc_binding_names, roc_name_appears,
    roc_rest_name, split_roc_body,
};
use crate::parse_fragment;

const META_NAME: &str = "rocci_meta";
const CONTENT_NAME: &str = "rocci_content";
const PAGE_NAME: &str = "rocci_page";
const ISLANDS_NAME: &str = "rocci_islands";

fn is_site_chrome_layout(layout: &str) -> bool {
    matches!(
        layout,
        "home"
            | "product"
            | "section"
            | "docs"
            | "news-index"
            | "news-post"
            | "plain"
            | "not-found"
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
            Item::On(decl) => rocci_items.push(ModuleItem::On(decl.clone())),
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

    let mut field_defaults: HashMap<String, Vec<(String, String)>> = lowered_rocci
        .components
        .iter()
        .map(|component| (component.name.clone(), component.param_defaults.clone()))
        .collect();
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
        emitter.emit(" = |_state| {\n");
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

    let field_defaults: HashMap<String, Vec<(String, String)>> = lowered_rocci
        .components
        .iter()
        .map(|component| (component.name.clone(), component.param_defaults.clone()))
        .collect();

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
    if has_datastar {
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
    if injected_html || !imports.is_empty() || has_datastar {
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
        routes: Vec::new(),
        page_meta,
        theme: None,
    }
}

fn filter_snapshot_roc(
    source: SourceFile<'_>,
    document: &Document,
    imports: Vec<Span>,
    rest: Vec<Span>,
) -> (Vec<Span>, Vec<Span>) {
    let mut kept_text = island_used_text(source.src, document);
    let names: Vec<Option<String>> = rest
        .iter()
        .map(|span| roc_rest_name(source.src, *span))
        .collect();
    let mut keep = vec![false; rest.len()];
    loop {
        let mut changed = false;
        for (i, span) in rest.iter().enumerate() {
            if keep[i] {
                continue;
            }
            let take = match names[i].as_deref() {
                None => true,
                Some(name) => roc_name_appears(name, &kept_text),
            };
            if take {
                keep[i] = true;
                kept_text.push('\n');
                kept_text.push_str(span.of(source.src));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let kept_rest: Vec<Span> = rest
        .iter()
        .zip(keep.iter())
        .filter_map(|(span, keep)| keep.then_some(*span))
        .collect();
    let kept_imports: Vec<Span> = imports
        .into_iter()
        .filter(|span| match import_local_name(source.src, *span) {
            None => true,
            Some(name) => roc_name_appears(&name, &kept_text),
        })
        .collect();
    (kept_imports, kept_rest)
}

fn island_used_text(src: &str, document: &Document) -> String {
    let mut text = String::new();
    for item in &document.items {
        match item {
            Item::Render(render) => {
                text.push('\n');
                text.push_str(render.span.of(src));
            }
            Item::Template(template) => {
                text.push('\n');
                text.push_str(template.span().of(src));
            }
            _ => {}
        }
    }
    let components: Vec<_> = document
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Component(decl) => Some(decl),
            _ => None,
        })
        .collect();
    let mut included = vec![false; components.len()];
    loop {
        let mut changed = false;
        for (i, component) in components.iter().enumerate() {
            if included[i] {
                continue;
            }
            let pascal = component.name.name.as_str();
            let camel = pascal_to_camel(pascal);
            if roc_name_appears(pascal, &text) || roc_name_appears(&camel, &text) {
                included[i] = true;
                text.push('\n');
                text.push_str(component.span.of(src));
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    text
}

pub(crate) fn island_item_count(document: &Document) -> usize {
    document
        .items
        .iter()
        .filter(|item| is_island_item(item))
        .count()
}

pub(crate) fn is_island_item(item: &Item) -> bool {
    match item {
        Item::Render(_) => true,
        Item::Template(TemplateItem::Let(_)) => false,
        Item::Template(_) => true,
        _ => false,
    }
}

struct Emitter<'a> {
    source: SourceFile<'a>,
    options: &'a LowerOptions,
    html: &'a str,
    roc: String,
    segments: Vec<Segment>,
    indent: usize,
    at_line_start: bool,
    css_stamp: Option<String>,
    field_defaults: HashMap<String, Vec<(String, String)>>,
    imported_kinds: HashMap<String, crate::imports::ImportedKind>,
    theme: Option<rocci_theme::ResolvedTheme>,
    diagnostics: &'a mut Vec<Diagnostic>,
    resolve_includes: bool,
}

impl<'a> Emitter<'a> {
    fn emit_content_lets(&mut self, document: &Document) {
        for item in &document.items {
            let Item::Template(TemplateItem::Let(let_dir)) = item else {
                continue;
            };
            self.push_indent();
            self.emit_mapped(
                &let_dir.binder.name,
                let_dir.binder.span,
                OriginKind::Directive,
            );
            self.emit(" = ");
            self.emit_mapped(
                let_dir.expr.of(self.source.src).trim(),
                let_dir.expr,
                OriginKind::Directive,
            );
            self.emit("\n\n");
        }
    }

    fn lower_content_value(&mut self, document: &Document) {
        let groups = group_content(document);
        match groups.as_slice() {
            [] => {
                self.emit_html(".fragment([\n");
                self.push_indent();
                self.emit("])");
            }
            [ContentGroup::For(item)] => {
                self.emit_html(".fragment(\n");
                self.indent += 1;
                self.push_indent();
                self.splice_template(std::slice::from_ref(item), TemplateValueCtx::List);
                self.emit("\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            [ContentGroup::Nodes(nodes)] => {
                self.emit_html(".fragment([\n");
                self.indent += 1;
                self.emit_nodes(nodes);
                self.emit_footnote_section(document);
                self.indent -= 1;
                self.push_indent();
                self.emit("])");
            }
            _ => {
                self.emit_html(".fragment(\n");
                self.indent += 1;
                self.push_indent();
                if document_has_footnotes(document) {
                    self.emit("List.concat(\n");
                    self.indent += 1;
                    self.push_indent();
                    self.emit_concat_groups(&groups);
                    self.emit(",\n");
                    self.push_indent();
                    self.emit("[\n");
                    self.indent += 1;
                    self.emit_footnote_section(document);
                    self.indent -= 1;
                    self.push_indent();
                    self.emit("],\n");
                    self.indent -= 1;
                    self.push_indent();
                    self.emit(")\n");
                } else {
                    self.emit_concat_groups(&groups);
                    self.emit("\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
        }
    }

    fn emit_concat_groups(&mut self, groups: &[ContentGroup<'_>]) {
        match groups {
            [] => self.emit("[]"),
            [group] => self.emit_content_group(group),
            [first, rest @ ..] => {
                self.emit("List.concat(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_content_group(first);
                self.emit(",\n");
                self.push_indent();
                self.emit_concat_groups(rest);
                self.emit(",\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
        }
    }

    fn emit_content_group(&mut self, group: &ContentGroup<'_>) {
        match group {
            ContentGroup::Nodes(nodes) => {
                self.emit("[\n");
                self.indent += 1;
                self.emit_nodes(nodes);
                self.indent -= 1;
                self.push_indent();
                self.emit("]");
            }
            ContentGroup::For(item) => {
                self.splice_template(std::slice::from_ref(item), TemplateValueCtx::List);
            }
        }
    }

    fn emit_nodes(&mut self, nodes: &[ContentPiece<'_>]) {
        for node in nodes {
            if matches!(
                node,
                ContentPiece::Markdown(MdNode::FootnoteDefinition { .. })
            ) {
                continue;
            }
            self.push_indent();
            match node {
                ContentPiece::Markdown(md) => self.lower_md(md),
                ContentPiece::Block(call) if is_heading_sugar(call, self.source.src) => {
                    self.lower_heading_sugar(call)
                }
                ContentPiece::Block(call) if call.name == "img" => self.lower_img(call),
                ContentPiece::Block(call) => self.lower_docs(call),
                ContentPiece::Render(render) => {
                    let expr = render.expr.of(self.source.src).trim();
                    self.emit_mapped(expr, render.expr, OriginKind::RenderRoc);
                }
                ContentPiece::Template(item) => {
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::Node);
                }
            }
            self.emit(",\n");
        }
    }

    fn emit_island_list(&mut self, document: &Document) {
        self.emit("[\n");
        self.indent += 1;
        for item in &document.items {
            match item {
                Item::Render(render) => {
                    self.push_indent();
                    let expr = render.expr.of(self.source.src).trim();
                    self.emit_mapped(expr, render.expr, OriginKind::RenderRoc);
                    self.emit(",\n");
                }
                Item::Template(TemplateItem::Let(_)) => {}
                Item::Template(item) if matches!(item, TemplateItem::For(_)) => {
                    self.push_indent();
                    self.emit_html(".fragment(\n");
                    self.indent += 1;
                    self.push_indent();
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::List);
                    self.emit("\n");
                    self.indent -= 1;
                    self.push_indent();
                    self.emit("),\n");
                }
                Item::Template(item) => {
                    self.push_indent();
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::Node);
                    self.emit(",\n");
                }
                _ => {}
            }
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    fn emit_footnote_section(&mut self, document: &Document) {
        let defs: Vec<&MdNode> = document
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Markdown(node @ MdNode::FootnoteDefinition { .. }) => Some(node),
                _ => None,
            })
            .collect();
        if defs.is_empty() {
            return;
        }
        let span = defs[0].span();
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("section", span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".attribute(");
        self.emit_string("class", span, OriginKind::MarkdownStructure);
        self.emit(", ");
        self.emit_string("rd-footnotes", span, OriginKind::MarkdownBoilerplate);
        self.emit("),\n");
        self.push_indent();
        self.emit_html(".boolean_attribute(");
        self.emit_string("data-footnotes", span, OriginKind::MarkdownStructure);
        self.emit(", True),\n");
        self.push_indent();
        self.emit_html(".attribute(");
        self.emit_string("aria-label", span, OriginKind::MarkdownStructure);
        self.emit(", ");
        self.emit_string("Footnotes", span, OriginKind::MarkdownBoilerplate);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("ol", span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", "rd-footnote-list")], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        for def in defs {
            self.push_indent();
            self.emit_footnote_definition(def);
            self.emit(",\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
    }

    fn emit_footnote_definition(&mut self, node: &MdNode) {
        let MdNode::FootnoteDefinition {
            name,
            total_references,
            children,
            span,
        } = node
        else {
            return;
        };
        let id = format!("fn-{name}");
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("li", *span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(
            &[("class", "rd-footnote-definition"), ("id", id.as_str())],
            *span,
        );
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        for child in children {
            self.push_indent();
            self.lower_md(child);
            self.emit(",\n");
        }
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("span", *span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", "rd-footnote-backlinks")], *span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        for reference_number in 1..=*total_references {
            let suffix = if reference_number == 1 {
                String::new()
            } else {
                format!("-{reference_number}")
            };
            let href = format!("#fnref-{name}{suffix}");
            let label = format!("Back to reference {name}{suffix}");
            self.push_indent();
            self.emit_html(".element(\n");
            self.indent += 1;
            self.push_indent();
            self.emit_string("a", *span, OriginKind::MarkdownStructure);
            self.emit(",\n");
            self.push_indent();
            self.emit("[\n");
            self.indent += 1;
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("class", *span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(
                "rd-footnote-backref",
                *span,
                OriginKind::MarkdownBoilerplate,
            );
            self.emit("),\n");
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("href", *span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(&href, *span, OriginKind::MarkdownBoilerplate);
            self.emit("),\n");
            self.push_indent();
            self.emit_html(".boolean_attribute(");
            self.emit_string(
                "data-footnote-backref",
                *span,
                OriginKind::MarkdownStructure,
            );
            self.emit(", True),\n");
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("aria-label", *span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(&label, *span, OriginKind::MarkdownBoilerplate);
            self.emit("),\n");
            self.indent -= 1;
            self.push_indent();
            self.emit("],\n");
            self.push_indent();
            self.emit("[\n");
            self.indent += 1;
            self.push_indent();
            self.emit_html(".text(");
            self.emit_string("↩", *span, OriginKind::MarkdownBoilerplate);
            self.emit("),\n");
            self.indent -= 1;
            self.push_indent();
            self.emit("],\n");
            self.indent -= 1;
            self.push_indent();
            self.emit("),\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn splice_template(&mut self, items: &[TemplateItem], ctx: TemplateValueCtx) {
        let lowered: LoweredTemplate = lower_template_items(
            self.source,
            items,
            self.options,
            &self.field_defaults,
            self.indent,
            ctx,
            self.css_stamp.clone(),
        );
        let start = self.roc.len();
        self.roc.push_str(&lowered.roc);
        self.at_line_start = lowered.roc.ends_with('\n');
        for mut segment in lowered.segments {
            segment.generated.start += start as u32;
            segment.generated.end += start as u32;
            self.segments.push(segment);
        }
    }

    fn lower_docs(&mut self, call: &BlockCall) {
        if let Some(imported) = self.imported_kinds.get(&call.name).cloned() {
            self.lower_imported_block(call, &imported);
            return;
        }
        let src = self.source.src;
        let fields = docs_fields_from_params(call.params.as_ref());
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let title = fields
            .iter()
            .find(|field| field.name == "title")
            .or_else(|| fields.iter().find(|field| field.name == "term"))
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let summary = fields
            .iter()
            .find(|field| field.name == "summary")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let label = fields
            .iter()
            .find(|field| field.name == "label")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let open = fields
            .iter()
            .find(|field| field.name == "open")
            .and_then(|field| field_bool(src, field))
            .unwrap_or(false);
        let caption = fields
            .iter()
            .find(|field| field.name == "caption")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let credit = fields
            .iter()
            .find(|field| field.name == "credit")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        if call.name == "include" {
            self.lower_docs_include(call, &fields);
            return;
        }
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        for item in &parsed.document.items {
            if let Some(kind) = illegal_docs_item(item) {
                self.diagnostics.push(Diagnostic::error(
                    item.span(),
                    format!("`@{kind}` is not allowed inside an article block"),
                ));
            }
        }
        let class = if crate::registry::is_aside(&call.name) {
            format!("rd-docs-aside rd-docs-block rd-docs-{}", call.name)
        } else {
            format!("rd-docs-{} rd-docs-block", call.name)
        };
        let tag = if crate::registry::is_aside(&call.name) {
            "aside"
        } else {
            match call.name.as_str() {
                "details" => "details",
                "figure" => "figure",
                "badge" => "p",
                _ => "section",
            }
        };
        let label_text = if crate::registry::is_aside(&call.name) {
            match call.name.as_str() {
                "note" => "Note",
                "tip" => "Tip",
                "caution" => "Caution",
                "danger" => "Danger",
                "deprecated" => "Deprecated",
                _ => "Note",
            }
        } else {
            ""
        };
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(tag, call.span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        let mut attrs = vec![
            ("class", class.as_str()),
            ("data-rocci-docs", call.name.as_str()),
        ];
        let aria = if call.name == "deprecated" {
            "Deprecated"
        } else if call.name == "file-tree" {
            "File tree"
        } else if call.name == "tab" && !label.is_empty() {
            label.as_str()
        } else {
            ""
        };
        if !aria.is_empty() {
            attrs.push(("aria-label", aria));
        }
        if call.name == "details" && open {
            attrs.push(("open", "open"));
        }
        self.emit_attrs(&attrs, call.span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        if call.name == "details" {
            self.push_indent();
            self.emit_text_element("summary", "rd-docs-summary", &summary, call.span);
            self.emit(",\n");
        } else if !label_text.is_empty() {
            self.push_indent();
            self.emit_text_element("p", "rd-docs-label", label_text, call.span);
            self.emit(",\n");
        }
        if call.name == "tab" && !label.is_empty() {
            self.push_indent();
            self.emit_text_element("h3", "rd-docs-tab-label", &label, call.span);
            self.emit(",\n");
        }
        if !title.is_empty() && call.name != "details" {
            self.push_indent();
            self.emit_text_element("p", "rd-docs-title", &title, call.span);
            self.emit(",\n");
        }
        if call.name == "badge" {
            self.push_indent();
            self.emit_text_element(
                "span",
                "rd-docs-badge-label",
                if label.is_empty() { &title } else { &label },
                call.span,
            );
            self.emit(",\n");
        }
        self.lower_docs_items(&parsed.document.items);
        if call.name == "figure" {
            if !caption.is_empty() {
                self.push_indent();
                self.emit_text_element("figcaption", "rd-docs-caption", &caption, call.span);
                self.emit(",\n");
            }
            if !credit.is_empty() {
                self.push_indent();
                self.emit_text_element("p", "rd-docs-credit", &credit, call.span);
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

    fn lower_imported_block(&mut self, call: &BlockCall, imported: &crate::imports::ImportedKind) {
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        for item in &parsed.document.items {
            if let Some(kind) = illegal_docs_item(item) {
                self.diagnostics.push(Diagnostic::error(
                    item.span(),
                    format!("`@{kind}` is not allowed inside an article block"),
                ));
            }
        }
        self.emit(&imported.roc_name);
        self.emit("(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_imported_props(call, &imported.roc_name);
        self.emit(",\n");
        self.push_indent();
        self.emit_html(".fragment([\n");
        self.indent += 1;
        self.lower_docs_items(&parsed.document.items);
        self.indent -= 1;
        self.push_indent();
        self.emit("]),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_imported_props(&mut self, call: &BlockCall, roc_name: &str) {
        let fields = call
            .params
            .as_ref()
            .map(|record| record.fields.as_slice())
            .unwrap_or(&[]);
        let missing_defaults: Vec<(String, String)> = self
            .field_defaults
            .get(roc_name)
            .into_iter()
            .flatten()
            .filter(|(name, _)| !fields.iter().any(|field| field.name == *name))
            .cloned()
            .collect();
        if fields.is_empty() && missing_defaults.is_empty() {
            self.emit("{}");
            return;
        }
        self.emit("{ ");
        let mut first = true;
        for field in fields {
            if !first {
                self.emit(", ");
            }
            first = false;
            self.emit(&field.name);
            self.emit(": ");
            self.emit_param_value(&field.value);
        }
        for (name, default) in missing_defaults {
            if !first {
                self.emit(", ");
            }
            first = false;
            self.emit(&name);
            self.emit(": ");
            self.emit(&default);
        }
        self.emit(" }");
    }

    fn emit_param_value(&mut self, value: &ParamValue) {
        match value {
            ParamValue::StringLit { value, span } => {
                self.emit_string(value, *span, OriginKind::StaticMarkup);
            }
            ParamValue::BoolLit { value, span } => {
                self.emit_mapped(
                    if *value { "True" } else { "False" },
                    *span,
                    OriginKind::StaticMarkup,
                );
            }
            ParamValue::NumberLit { value, span } => {
                self.emit_mapped(value, *span, OriginKind::StaticMarkup);
            }
            ParamValue::Ident { name, span } => {
                self.emit_string(name, *span, OriginKind::StaticMarkup);
            }
            ParamValue::Record(record) => {
                if record.fields.is_empty() {
                    self.emit("{}");
                    return;
                }
                self.emit("{ ");
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 {
                        self.emit(", ");
                    }
                    self.emit(&field.name);
                    self.emit(": ");
                    self.emit_param_value(&field.value);
                }
                self.emit(" }");
            }
            ParamValue::List(list) => {
                self.emit("[");
                for (index, item) in list.items.iter().enumerate() {
                    if index > 0 {
                        self.emit(", ");
                    }
                    self.emit_param_value(item);
                }
                self.emit("]");
            }
        }
    }

    fn lower_docs_items(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Markdown(node) => {
                    self.push_indent();
                    self.lower_md(node);
                    self.emit(",\n");
                }
                Item::Block(nested) if is_heading_sugar(nested, self.source.src) => {
                    self.push_indent();
                    self.lower_heading_sugar(nested);
                    self.emit(",\n");
                }
                Item::Block(nested) if nested.name == "img" => {
                    self.push_indent();
                    self.lower_img(nested);
                    self.emit(",\n");
                }
                Item::Block(nested) => {
                    self.push_indent();
                    self.lower_docs(nested);
                    self.emit(",\n");
                }
                _ => {}
            }
        }
    }

    fn lower_heading_sugar(&mut self, call: &BlockCall) {
        let level = crate::registry::heading_level(&call.name).unwrap_or(1);
        let id = heading_id_from_params(call).unwrap_or_default();
        let tag = format!("h{level}");
        let class = format!("rd-header-{level}");
        let content = call
            .content_span()
            .unwrap_or_else(|| Span::point(call.span.end as usize));
        let parsed = parse_fragment(self.source, content, false);
        self.diagnostics.extend(parsed.diagnostics);
        let children = heading_inline_nodes(&parsed.document.items);
        self.emit_element(
            &tag,
            &[("class", class.as_str()), ("id", id.as_str())],
            &children,
            false,
            call.span,
        );
    }

    fn lower_img(&mut self, call: &BlockCall) {
        let body = call
            .params
            .as_ref()
            .map(|params| params.span)
            .unwrap_or(call.span);
        let fields =
            crate::img::img_fields_from_params(call.params.as_ref(), body, self.diagnostics);
        let image = crate::img::StaticImage::from_fields(&fields, call.span);
        let attrs = image.html_attrs();

        self.emit_html(".void_element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("img", call.span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_img_attrs(&attrs, call.span);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_img_attrs(&mut self, attrs: &[crate::img::ImgHtmlAttr], decl_span: Span) {
        if attrs.is_empty() && self.css_stamp.is_none() {
            self.emit("[]");
            return;
        }
        self.emit("[\n");
        self.indent += 1;
        for attr in attrs {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string(attr.name, attr.span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(&attr.value, attr.span, OriginKind::MarkdownText);
            self.emit("),\n");
        }
        if let Some(stamp) = &self.css_stamp.clone() {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("data-rocci-css", decl_span, OriginKind::Scaffolding);
            self.emit(", ");
            self.emit_string(stamp, decl_span, OriginKind::Scaffolding);
            self.emit("),\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    fn lower_docs_include(&mut self, call: &BlockCall, fields: &[crate::docs::DocsField]) {
        if !self.resolve_includes {
            self.emit_html(".empty");
            return;
        }
        let src = self.source.src;
        let path = fields
            .iter()
            .find(|field| field.name == "path")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let region = fields
            .iter()
            .find(|field| field.name == "region")
            .and_then(|field| field_string(src, field));
        let start = fields
            .iter()
            .find(|field| field.name == "start")
            .and_then(|field| field.value.of(src).trim().parse::<u32>().ok());
        let end = fields
            .iter()
            .find(|field| field.name == "end")
            .and_then(|field| field.value.of(src).trim().parse::<u32>().ok());
        let language = fields
            .iter()
            .find(|field| field.name == "language")
            .and_then(|field| field_string(src, field))
            .unwrap_or_default();
        let resolved = match resolve_include_path(self.source.name, &path) {
            Ok(path) => path,
            Err(err) => {
                self.diagnostics.push(Diagnostic::error(call.span, err));
                self.emit_html(".empty");
                return;
            }
        };
        let contents = match std::fs::read_to_string(&resolved) {
            Ok(contents) => contents,
            Err(_) => {
                self.diagnostics.push(Diagnostic::error(
                    call.span,
                    format!("could not read include `{}`", resolved.display()),
                ));
                self.emit_html(".empty");
                return;
            }
        };
        let excerpt = if let Some(region) = region.as_deref() {
            match extract_region(&contents, region) {
                Ok((excerpt, _, _)) => excerpt,
                Err(err) => {
                    self.diagnostics.push(Diagnostic::error(call.span, err));
                    self.emit_html(".empty");
                    return;
                }
            }
        } else if let (Some(start), Some(end)) = (start, end) {
            match extract_lines(&contents, start, end) {
                Ok((excerpt, _, _)) => excerpt,
                Err(err) => {
                    self.diagnostics.push(Diagnostic::error(call.span, err));
                    self.emit_html(".empty");
                    return;
                }
            }
        } else {
            contents
        };
        let is_doc = matches!(
            resolved.extension().and_then(|ext| ext.to_str()),
            Some("rocdown" | "md" | "markdown")
        );
        if is_doc {
            let included = crate::parse(
                SourceFile::new(&resolved.to_string_lossy(), &excerpt),
                false,
            );
            self.diagnostics.extend(included.diagnostics);
            for item in &included.document.items {
                if let Some(kind) = illegal_docs_item(item) {
                    self.diagnostics.push(Diagnostic::error(
                        item.span(),
                        format!("`@{kind}` is not allowed inside `:include`"),
                    ));
                }
            }
            self.emit_html(".fragment([\n");
            self.indent += 1;
            self.lower_docs_items(&included.document.items);
            self.indent -= 1;
            self.push_indent();
            self.emit("])");
            return;
        }
        let info = if language.is_empty() {
            resolved
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string()
        } else {
            language
        };
        self.lower_md(&MdNode::CodeBlock {
            info,
            literal: excerpt,
            span: call.span,
        });
    }

    fn emit_text_element(&mut self, tag: &str, class: &str, value: &str, span: Span) {
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(tag, span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", class)], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".text(");
        self.emit_string(value, span, OriginKind::MarkdownText);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn lower_md(&mut self, node: &MdNode) {
        match node {
            MdNode::Heading {
                level,
                id,
                children,
                span,
            } => {
                let tag = format!("h{level}");
                let class = format!("rd-header-{level}");
                self.emit_element(
                    &tag,
                    &[("class", class.as_str()), ("id", id.as_str())],
                    children,
                    false,
                    *span,
                );
            }
            MdNode::Paragraph { children, span } => {
                self.emit_element("p", &[("class", "rd-paragraph")], children, false, *span);
            }
            MdNode::BlockQuote { children, span } => {
                self.emit_element(
                    "blockquote",
                    &[("class", "rd-blockquote")],
                    children,
                    false,
                    *span,
                );
            }
            MdNode::List {
                ordered,
                start,
                children,
                span,
            } => {
                let name = if *ordered { "ol" } else { "ul" };
                let class = if *ordered {
                    "rd-list-ordered"
                } else {
                    "rd-list"
                };
                let start_value = start.to_string();
                let mut attrs = vec![("class", class)];
                if *ordered && *start != 1 {
                    attrs.push(("start", start_value.as_str()));
                }
                self.emit_element(name, &attrs, children, false, *span);
            }
            MdNode::Item { children, span } => {
                self.emit_element("li", &[("class", "rd-list-item")], children, false, *span);
            }
            MdNode::TaskItem {
                checked,
                children,
                span,
            } => {
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("li", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-task-item")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_checkbox(*checked, *span);
                self.emit(",\n");
                self.push_indent();
                self.emit_html(".text(");
                self.emit_string(" ", *span, OriginKind::MarkdownBoilerplate);
                self.emit("),\n");
                for child in children {
                    self.push_indent();
                    self.lower_md(child);
                    self.emit(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            MdNode::CodeBlock {
                info,
                literal,
                span,
            } => {
                let code_class = if info.is_empty() {
                    "rd-code".to_string()
                } else {
                    format!("rd-code language-{info}")
                };
                let code_attrs = [("class", code_class.as_str())];
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("pre", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-code-block")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("code", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&code_attrs, *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".text(");
                self.emit_string(literal, *span, OriginKind::MarkdownText);
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            MdNode::ThematicBreak { span } => {
                self.emit_void("hr", &[("class", "rd-thematic-break")], *span);
            }
            MdNode::Table { children, span } => {
                let mut head = Vec::new();
                let mut body = Vec::new();
                for child in children {
                    match child {
                        MdNode::TableRow { header: true, .. } => head.push(child.clone()),
                        _ => body.push(child.clone()),
                    }
                }
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("div", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-table-wrap")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("table", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-table")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                if !head.is_empty() {
                    self.push_indent();
                    self.emit_element("thead", &[("class", "rd-table-head")], &head, false, *span);
                    self.emit(",\n");
                }
                if !body.is_empty() {
                    self.push_indent();
                    self.emit_element("tbody", &[("class", "rd-table-body")], &body, false, *span);
                    self.emit(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            MdNode::TableRow {
                header,
                children,
                span,
            } => {
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("tr", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-table-row")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                for child in children {
                    self.push_indent();
                    if *header {
                        if let MdNode::TableCell { children, span } = child {
                            self.emit_element(
                                "th",
                                &[("class", "rd-table-header")],
                                children,
                                false,
                                *span,
                            );
                        } else {
                            self.lower_md(child);
                        }
                    } else {
                        self.lower_md(child);
                    }
                    self.emit(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            MdNode::TableCell { children, span } => {
                self.emit_element("td", &[("class", "rd-table-cell")], children, false, *span);
            }
            MdNode::Text { value, span } => {
                self.emit_html(".text(");
                self.emit_string(value, *span, OriginKind::MarkdownText);
                self.emit(")");
            }
            MdNode::SoftBreak { span } => {
                self.emit_html(".text(");
                self.emit_string("\n", *span, OriginKind::MarkdownText);
                self.emit(")");
            }
            MdNode::LineBreak { span } => self.emit_void("br", &[], *span),
            MdNode::Code { value, span } => {
                self.emit_element(
                    "code",
                    &[("class", "rd-code")],
                    &[MdNode::Text {
                        value: value.clone(),
                        span: *span,
                    }],
                    false,
                    *span,
                );
            }
            MdNode::Emph { children, span } => {
                self.emit_element("em", &[("class", "rd-emphasis")], children, false, *span);
            }
            MdNode::Strong { children, span } => {
                self.emit_element("strong", &[("class", "rd-strong")], children, false, *span);
            }
            MdNode::Strikethrough { children, span } => {
                self.emit_element(
                    "del",
                    &[("class", "rd-strikethrough")],
                    children,
                    false,
                    *span,
                );
            }
            MdNode::FootnoteDefinition { .. } => self.emit_html(".empty"),
            MdNode::FootnoteReference {
                name,
                reference_number,
                index,
                span,
                ..
            } => {
                let suffix = if *reference_number == 1 {
                    String::new()
                } else {
                    format!("-{reference_number}")
                };
                let href = format!("#fn-{name}");
                let id = format!("fnref-{name}{suffix}");
                let label = format!("Footnote {index}");
                let number = index.to_string();
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("sup", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[("class", "rd-footnote-ref")], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("a", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".attribute(");
                self.emit_string("href", *span, OriginKind::MarkdownStructure);
                self.emit(", ");
                self.emit_string(&href, *span, OriginKind::MarkdownBoilerplate);
                self.emit("),\n");
                self.push_indent();
                self.emit_html(".attribute(");
                self.emit_string("id", *span, OriginKind::MarkdownStructure);
                self.emit(", ");
                self.emit_string(&id, *span, OriginKind::MarkdownBoilerplate);
                self.emit("),\n");
                self.push_indent();
                self.emit_html(".boolean_attribute(");
                self.emit_string("data-footnote-ref", *span, OriginKind::MarkdownStructure);
                self.emit(", True),\n");
                self.push_indent();
                self.emit_html(".attribute(");
                self.emit_string("aria-label", *span, OriginKind::MarkdownStructure);
                self.emit(", ");
                self.emit_string(&label, *span, OriginKind::MarkdownBoilerplate);
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                self.push_indent();
                self.emit_html(".text(");
                self.emit_string(&number, *span, OriginKind::MarkdownText);
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("),\n");
                self.indent -= 1;
                self.push_indent();
                self.emit("],\n");
                self.indent -= 1;
                self.push_indent();
                self.emit(")");
            }
            MdNode::Link {
                url,
                title,
                children,
                span,
            } => {
                let mut attrs = vec![("class", "rd-link"), ("href", url.as_str())];
                if !title.is_empty() {
                    attrs.push(("title", title.as_str()));
                }
                self.emit_element("a", &attrs, children, false, *span);
            }
            MdNode::Image {
                url,
                title,
                alt,
                span,
            } => {
                let mut attrs = vec![
                    ("class", "rd-image"),
                    ("src", url.as_str()),
                    ("alt", alt.as_str()),
                ];
                if !title.is_empty() {
                    attrs.push(("title", title.as_str()));
                }
                self.emit_void("img", &attrs, *span);
            }
            MdNode::RawHtml { html, span } => {
                self.emit_html(".dangerously_include_unescaped_html(");
                self.emit_string(html, *span, OriginKind::MarkdownText);
                self.emit(")");
            }
        }
    }

    fn emit_element(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
        children: &[MdNode],
        void: bool,
        span: Span,
    ) {
        if void {
            self.emit_void(name, attrs, span);
            return;
        }
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(name, span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(attrs, span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        for child in children {
            self.push_indent();
            self.lower_md(child);
            self.emit(",\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_void(&mut self, name: &str, attrs: &[(&str, &str)], span: Span) {
        self.emit_html(".void_element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(name, span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(attrs, span);
        self.emit(",\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_checkbox(&mut self, checked: bool, span: Span) {
        self.emit_html(".void_element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("input", span, OriginKind::MarkdownStructure);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".attribute(");
        self.emit_string("type", span, OriginKind::MarkdownStructure);
        self.emit(", ");
        self.emit_string("checkbox", span, OriginKind::MarkdownBoilerplate);
        self.emit("),\n");
        self.push_indent();
        self.emit_html(".boolean_attribute(");
        self.emit_string("disabled", span, OriginKind::MarkdownStructure);
        self.emit(", True),\n");
        if checked {
            self.push_indent();
            self.emit_html(".boolean_attribute(");
            self.emit_string("checked", span, OriginKind::MarkdownStructure);
            self.emit(", True),\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_attrs(&mut self, attrs: &[(&str, &str)], span: Span) {
        if attrs.is_empty() && self.css_stamp.is_none() {
            self.emit("[]");
            return;
        }
        self.emit("[\n");
        self.indent += 1;
        for (name, value) in attrs {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string(name, span, OriginKind::MarkdownStructure);
            self.emit(", ");
            self.emit_string(value, span, OriginKind::MarkdownText);
            self.emit("),\n");
        }
        if let Some(stamp) = &self.css_stamp.clone() {
            self.push_indent();
            self.emit_html(".attribute(");
            self.emit_string("data-rocci-css", span, OriginKind::Scaffolding);
            self.emit(", ");
            self.emit_string(stamp, span, OriginKind::Scaffolding);
            self.emit("),\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("]");
    }

    fn emit_toc(&mut self, headings: &[HeadingInfo], span: Span) {
        let outline: Vec<&HeadingInfo> = headings
            .iter()
            .filter(|heading| (2..=3).contains(&heading.level))
            .collect();
        if outline.is_empty() {
            return;
        }

        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("nav", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", "rd-toc"), ("aria-label", "On this page")], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.emit_tagged_text(
            "p",
            &[("class", "rd-toc-label")],
            "On this page",
            span,
            OriginKind::MarkdownBoilerplate,
        );
        self.emit(",\n");
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("div", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", "rd-toc-items")], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        let hrefs: Vec<String> = outline
            .iter()
            .map(|heading| format!("#{}", heading.id))
            .collect();
        self.emit_toc_links(&outline, &hrefs);
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.emit_toc_menu(&outline, &hrefs, span);
    }

    fn emit_toc_links(&mut self, outline: &[&HeadingInfo], hrefs: &[String]) {
        for (heading, href) in outline.iter().zip(hrefs.iter()) {
            let class = if heading.level == 3 {
                "rd-toc-link rd-toc-level-3"
            } else {
                "rd-toc-link"
            };
            self.emit_tagged_text(
                "a",
                &[("class", class), ("href", href)],
                &heading.text,
                heading.span,
                OriginKind::MarkdownText,
            );
            self.emit(",\n");
        }
    }

    fn emit_toc_menu(&mut self, outline: &[&HeadingInfo], hrefs: &[String], span: Span) {
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("details", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(
            &[("class", "rd-toc-menu"), ("aria-label", "On this page")],
            span,
        );
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.emit_tagged_text(
            "summary",
            &[],
            "On this page",
            span,
            OriginKind::MarkdownBoilerplate,
        );
        self.emit(",\n");
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("div", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("class", "rd-toc-items")], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.emit_toc_links(outline, hrefs);
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
    }

    fn emit_main(&mut self, span: Span) {
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("main", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit("rocci_content({}),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
    }

    fn emit_tagged_text(
        &mut self,
        name: &str,
        attrs: &[(&str, &str)],
        text: &str,
        span: Span,
        text_origin: OriginKind,
    ) {
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string(name, span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(attrs, span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".text(");
        self.emit_string(text, span, text_origin);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")");
    }

    fn emit_default_page(
        &mut self,
        page_meta: &PageMeta,
        styles: &[StyleArtifact],
        headings: &[HeadingInfo],
    ) {
        let title = page_meta
            .title
            .clone()
            .unwrap_or_else(|| "Rocdown".to_string());
        let file_css = styles.iter().find_map(|style| {
            if matches!(style.kind, rocci_template::StyleKind::File) {
                Some(style.css.clone())
            } else {
                None
            }
        });
        let theme_active = self.theme.as_ref().is_some_and(|theme| !theme.is_none());
        let theme_id = self
            .theme
            .as_ref()
            .filter(|theme| !theme.is_none())
            .map(|theme| theme.id.clone());
        let theme_css = self
            .theme
            .as_ref()
            .filter(|theme| !theme.is_none())
            .map(|theme| theme.css.clone());
        let scheme_attr = self
            .theme
            .as_ref()
            .and_then(|theme| theme.policy.html_attr())
            .map(str::to_string);
        let scheme_meta = self
            .theme
            .as_ref()
            .filter(|theme| !theme.is_none())
            .map(|theme| theme.policy.meta_content());
        let mut html_attrs: Vec<(String, String)> = vec![("lang".into(), "en".into())];
        if theme_active {
            html_attrs.push(("class".into(), "rd-document".into()));
            if let Some(id) = &theme_id {
                html_attrs.push(("data-rd-theme".into(), id.clone()));
            }
            if let Some(scheme) = &scheme_attr {
                html_attrs.push(("data-rd-color-scheme".into(), scheme.clone()));
            }
        }
        let html_attr_refs: Vec<(&str, &str)> = html_attrs
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        let span = Span::point(0);
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("html", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&html_attr_refs, span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("head", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit("[],\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        let stamp = self.css_stamp.take();
        self.push_indent();
        self.emit_void("meta", &[("charset", "utf-8")], span);
        self.emit(",\n");
        self.push_indent();
        self.emit_void(
            "meta",
            &[
                ("name", "viewport"),
                ("content", "width=device-width, initial-scale=1"),
            ],
            span,
        );
        self.emit(",\n");
        if let Some(content) = scheme_meta {
            self.push_indent();
            self.emit_void(
                "meta",
                &[("name", "color-scheme"), ("content", content)],
                span,
            );
            self.emit(",\n");
        }
        self.css_stamp = stamp;
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("title", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit("[],\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".text(");
        self.emit_string(&title, span, OriginKind::PageRoc);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        if let Some(css) = &theme_css {
            self.emit_style_element(css, span);
        }
        if let Some(css) = &file_css {
            self.emit_style_element(css, span);
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("body", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[], span);
        self.emit(",\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        let show_toc = theme_active
            && headings
                .iter()
                .any(|heading| (2..=3).contains(&heading.level));
        if show_toc {
            self.push_indent();
            self.emit_html(".element(\n");
            self.indent += 1;
            self.push_indent();
            self.emit_string("div", span, OriginKind::MarkdownBoilerplate);
            self.emit(",\n");
            self.push_indent();
            self.emit_attrs(&[("class", "rd-shell")], span);
            self.emit(",\n");
            self.push_indent();
            self.emit("[\n");
            self.indent += 1;
            self.emit_toc(headings, span);
            self.emit_main(span);
            self.indent -= 1;
            self.push_indent();
            self.emit("],\n");
            self.indent -= 1;
            self.push_indent();
            self.emit("),\n");
            self.emit_toc_script(span);
        } else {
            self.emit_main(span);
        }
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit(")\n");
    }

    fn emit_toc_script(&mut self, span: Span) {
        let html = format!("<script>{}</script>", rocci_theme::TOC_SCRIPT.trim());
        self.push_indent();
        self.emit_html(".dangerously_include_unescaped_html(");
        self.emit_string(&html, span, OriginKind::Scaffolding);
        self.emit("),\n");
    }

    fn emit_style_element(&mut self, css: &str, span: Span) {
        self.push_indent();
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("style", span, OriginKind::Css);
        self.emit(",\n");
        self.push_indent();
        self.emit("[],\n");
        self.push_indent();
        self.emit("[\n");
        self.indent += 1;
        self.push_indent();
        self.emit_html(".text(");
        self.emit_string(css, span, OriginKind::Css);
        self.emit("),\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("],\n");
        self.indent -= 1;
        self.push_indent();
        self.emit("),\n");
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

    fn emit_source(&mut self, span: Span, text: &str, origin: OriginKind) {
        self.maybe_indent();
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

    fn maybe_indent(&mut self) {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.roc.push_str("    ");
            }
            self.at_line_start = false;
        }
    }
}

enum ContentPiece<'a> {
    Markdown(&'a MdNode),
    Block(&'a BlockCall),
    Render(&'a RenderDecl),
    Template(&'a TemplateItem),
}

enum ContentGroup<'a> {
    Nodes(Vec<ContentPiece<'a>>),
    For(&'a TemplateItem),
}

fn group_content(document: &Document) -> Vec<ContentGroup<'_>> {
    let mut groups = Vec::new();
    let mut current = Vec::new();
    for item in &document.items {
        match item {
            Item::Markdown(node) => current.push(ContentPiece::Markdown(node)),
            Item::Block(call) => current.push(ContentPiece::Block(call)),
            Item::Render(render) => current.push(ContentPiece::Render(render)),
            Item::Template(TemplateItem::Let(_)) => {}
            Item::Template(item) if matches!(item, TemplateItem::For(_)) => {
                if !current.is_empty() {
                    groups.push(ContentGroup::Nodes(std::mem::take(&mut current)));
                }
                groups.push(ContentGroup::For(item));
            }
            Item::Template(item) => current.push(ContentPiece::Template(item)),
            _ => {}
        }
    }
    if !current.is_empty() {
        groups.push(ContentGroup::Nodes(current));
    }
    groups
}

fn document_has_footnotes(document: &Document) -> bool {
    document
        .items
        .iter()
        .any(|item| matches!(item, Item::Markdown(MdNode::FootnoteDefinition { .. })))
}

fn illegal_docs_item(item: &Item) -> Option<&'static str> {
    match item {
        Item::Markdown(_) | Item::Block(_) => None,
        Item::Page(_) => Some("page"),
        Item::Roc(_) => Some("roc"),
        Item::Render(_) => Some("render"),
        Item::Component(_) => Some("component"),
        Item::Fixture(_) => Some("fixture"),
        Item::Css(_) => Some("css"),
        Item::Context(_) => Some("context"),
        Item::Init(_) => Some("init"),
        Item::On(_) => Some("on"),
        Item::Use(_) => Some("use"),
        Item::Template(_) => Some("template"),
    }
}

fn is_heading_sugar(call: &BlockCall, src: &str) -> bool {
    crate::registry::heading_level(&call.name).is_some()
        && (call.is_colon(src)
            || src
                .get(call.span.start as usize..)
                .unwrap_or("")
                .trim_start_matches([' ', '\t'])
                .starts_with('#'))
}

fn heading_id_from_params(call: &BlockCall) -> Option<String> {
    call.params.as_ref().and_then(|params| {
        params
            .fields
            .iter()
            .find(|field| field.name == "id")
            .and_then(|field| match &field.value {
                ParamValue::StringLit { value, .. } => Some(value.clone()),
                ParamValue::Ident { name, .. } => Some(name.clone()),
                _ => None,
            })
    })
}

fn heading_inline_nodes(items: &[Item]) -> Vec<MdNode> {
    let mut nodes = Vec::new();
    for item in items {
        match item {
            Item::Markdown(MdNode::Paragraph { children, .. }) => {
                nodes.extend(children.iter().cloned());
            }
            Item::Markdown(node) => nodes.push(node.clone()),
            _ => {}
        }
    }
    nodes
}
