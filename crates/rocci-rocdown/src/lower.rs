use std::collections::HashMap;

use rocci_template::{
    ComponentInfo, Diagnostic, Document as RocciDocument, FixtureInfo, InitInfo, LowerOptions,
    LoweredTemplate, ModuleItem, OriginKind, RouteInfo, Segment, SourceFile, Span, StyleArtifact,
    TemplateItem, TemplateValueCtx, file_scope_id, lower_template_items, route_fn_name,
    template_items_have_action, validate, validate_template_items,
};

use crate::ast::{Document, Item, MdNode, PageMeta, RenderDecl};
use crate::page::{extract_page, imports_html, roc_binding_names, split_roc_body};

const META_NAME: &str = "rocci_meta";
const CONTENT_NAME: &str = "rocci_content";
const PAGE_NAME: &str = "rocci_page";

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
}

pub fn lower(
    source: SourceFile<'_>,
    document: &Document,
    options: &LowerOptions,
    diagnostics: &mut Vec<Diagnostic>,
) -> Lowered {
    let mut page_meta = PageMeta::default();
    let mut page_count = 0;
    for item in &document.items {
        if let Item::Page(page) = item {
            page_count += 1;
            if page_count > 1 {
                diagnostics.push(Diagnostic::error(
                    page.span,
                    "duplicate `@page`; a document may declare page metadata once",
                ));
            } else {
                page_meta = extract_page(source.src, page.body, diagnostics);
            }
        }
        if let Item::Roc(roc) = item {
            for (name, span) in roc_binding_names(source.src, roc.body) {
                if matches!(name.as_str(), META_NAME | CONTENT_NAME | PAGE_NAME) {
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
    let lowered_rocci = rocci_template::lower(source, &rocci_doc, options);

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
    let page_datastar = document.items.iter().any(|item| match item {
        Item::Template(template) => template_items_have_action(std::slice::from_ref(template)),
        _ => false,
    });

    let mut emitter = Emitter {
        source,
        options,
        html: &options.html_module,
        roc: String::new(),
        segments: Vec::new(),
        indent: 0,
        at_line_start: true,
        css_stamp,
        field_defaults,
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
    if let Some(layout) = &page_meta.layout {
        emitter.emit(layout);
        emitter.emit("({ meta: rocci_meta, content: rocci_content({}) })\n");
    } else {
        emitter.emit_default_page(&page_meta, &lowered_rocci.styles);
    }
    emitter.indent -= 1;
    emitter.push_indent();
    emitter.emit("}\n");

    let mut routes = lowered_rocci.routes;
    let page_route = page_meta.route.clone().unwrap_or_else(|| "/".to_string());
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

    Lowered {
        roc: emitter.roc,
        segments,
        components: lowered_rocci.components,
        fixtures: lowered_rocci.fixtures,
        styles: lowered_rocci.styles,
        state_type: lowered_rocci.state_type,
        init: lowered_rocci.init,
        routes,
        page_meta,
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
                self.indent -= 1;
                self.push_indent();
                self.emit("])");
            }
            _ => {
                self.emit_html(".fragment(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_concat_groups(&groups);
                self.emit("\n");
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
            self.push_indent();
            match node {
                ContentPiece::Markdown(md) => self.lower_md(md),
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

    fn lower_md(&mut self, node: &MdNode) {
        match node {
            MdNode::Heading {
                level,
                id,
                children,
                span,
            } => {
                self.emit_element(
                    &format!("h{level}"),
                    &[("id", id.as_str())],
                    children,
                    false,
                    *span,
                );
            }
            MdNode::Paragraph { children, span } => {
                self.emit_element("p", &[], children, false, *span);
            }
            MdNode::BlockQuote { children, span } => {
                self.emit_element("blockquote", &[], children, false, *span);
            }
            MdNode::List {
                ordered,
                start,
                children,
                span,
            } => {
                let name = if *ordered { "ol" } else { "ul" };
                let start_value = start.to_string();
                let attrs: Vec<(&str, &str)> = if *ordered && *start != 1 {
                    vec![("start", start_value.as_str())]
                } else {
                    Vec::new()
                };
                self.emit_element(name, &attrs, children, false, *span);
            }
            MdNode::Item { children, span } => {
                self.emit_element("li", &[], children, false, *span);
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
                self.emit_attrs(&[], *span);
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
                let class = if info.is_empty() {
                    String::new()
                } else {
                    format!("language-{info}")
                };
                let code_attrs: Vec<(&str, &str)> = if class.is_empty() {
                    Vec::new()
                } else {
                    vec![("class", class.as_str())]
                };
                self.emit_html(".element(\n");
                self.indent += 1;
                self.push_indent();
                self.emit_string("pre", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[], *span);
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
                self.emit_void("hr", &[], *span);
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
                self.emit_string("table", *span, OriginKind::MarkdownStructure);
                self.emit(",\n");
                self.push_indent();
                self.emit_attrs(&[], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                if !head.is_empty() {
                    self.push_indent();
                    self.emit_element("thead", &[], &head, false, *span);
                    self.emit(",\n");
                }
                if !body.is_empty() {
                    self.push_indent();
                    self.emit_element("tbody", &[], &body, false, *span);
                    self.emit(",\n");
                }
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
                self.emit_attrs(&[], *span);
                self.emit(",\n");
                self.push_indent();
                self.emit("[\n");
                self.indent += 1;
                for child in children {
                    self.push_indent();
                    if *header {
                        if let MdNode::TableCell { children, span } = child {
                            self.emit_element("th", &[], children, false, *span);
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
                self.emit_element("td", &[], children, false, *span);
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
                    &[],
                    &[MdNode::Text {
                        value: value.clone(),
                        span: *span,
                    }],
                    false,
                    *span,
                );
            }
            MdNode::Emph { children, span } => {
                self.emit_element("em", &[], children, false, *span);
            }
            MdNode::Strong { children, span } => {
                self.emit_element("strong", &[], children, false, *span);
            }
            MdNode::Strikethrough { children, span } => {
                self.emit_element("del", &[], children, false, *span);
            }
            MdNode::Link {
                url,
                title,
                children,
                span,
            } => {
                let mut attrs = vec![("href", url.as_str())];
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
                let mut attrs = vec![("src", url.as_str()), ("alt", alt.as_str())];
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
        self.emit(", Bool.true),\n");
        if checked {
            self.push_indent();
            self.emit_html(".boolean_attribute(");
            self.emit_string("checked", span, OriginKind::MarkdownStructure);
            self.emit(", Bool.true),\n");
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

    fn emit_default_page(&mut self, page_meta: &PageMeta, styles: &[StyleArtifact]) {
        let title = page_meta
            .title
            .clone()
            .unwrap_or_else(|| "Rocdown".to_string());
        let file_css = styles.iter().find_map(|style| {
            if matches!(style.kind, rocci_template::StyleKind::File) {
                Some(style.css.as_str())
            } else {
                None
            }
        });
        let span = Span::point(0);
        self.emit_html(".element(\n");
        self.indent += 1;
        self.push_indent();
        self.emit_string("html", span, OriginKind::MarkdownBoilerplate);
        self.emit(",\n");
        self.push_indent();
        self.emit_attrs(&[("lang", "en")], span);
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
        if let Some(css) = file_css {
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
