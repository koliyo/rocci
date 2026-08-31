use std::collections::HashMap;

use rocci_template::{
    Diagnostic, LowerOptions, OriginKind, Segment, SourceFile, Span, StyleArtifact, TemplateItem,
    TemplateValueCtx,
};

use crate::ast::{BlockCall, Document, HeadingInfo, Item, MdNode, PageMeta, RenderDecl};

use super::docs_kind::is_heading_sugar;

pub(crate) struct Emitter<'a> {
    pub(crate) source: SourceFile<'a>,
    pub(crate) options: &'a LowerOptions,
    pub(crate) html: &'a str,
    pub(crate) roc: String,
    pub(crate) segments: Vec<Segment>,
    pub(crate) indent: usize,
    pub(crate) at_line_start: bool,
    pub(crate) css_stamp: Option<String>,
    pub(crate) field_defaults: HashMap<String, Vec<(String, String)>>,
    pub(crate) imported_kinds: HashMap<String, crate::imports::ImportedKind>,
    pub(crate) theme: Option<rocci_theme::ResolvedTheme>,
    pub(crate) diagnostics: &'a mut Vec<Diagnostic>,
    pub(crate) resolve_includes: bool,
}

pub(crate) enum ContentPiece<'a> {
    Markdown(&'a MdNode),
    Block(&'a BlockCall),
    Render(&'a RenderDecl),
    Template(&'a TemplateItem),
}

pub(crate) enum ContentGroup<'a> {
    Nodes(Vec<ContentPiece<'a>>),
    For(&'a TemplateItem),
}

pub(crate) fn group_content(document: &Document) -> Vec<ContentGroup<'_>> {
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

pub(crate) fn document_has_footnotes(document: &Document) -> bool {
    document
        .items
        .iter()
        .any(|item| matches!(item, Item::Markdown(MdNode::FootnoteDefinition { .. })))
}

impl<'a> Emitter<'a> {
    pub(crate) fn emit_content_lets(&mut self, document: &Document) {
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

    pub(crate) fn lower_content_value(&mut self, document: &Document) {
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

    pub(crate) fn emit_concat_groups(&mut self, groups: &[ContentGroup<'_>]) {
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

    pub(crate) fn emit_content_group(&mut self, group: &ContentGroup<'_>) {
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

    pub(crate) fn emit_nodes(&mut self, nodes: &[ContentPiece<'_>]) {
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
                    self.lower_render_call(render);
                }
                ContentPiece::Template(item) => {
                    self.splice_template(std::slice::from_ref(item), TemplateValueCtx::Node);
                }
            }
            self.emit(",\n");
        }
    }
}

impl<'a> Emitter<'a> {
    pub(crate) fn emit_text_element(&mut self, tag: &str, class: &str, value: &str, span: Span) {
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
}

impl<'a> Emitter<'a> {
    pub(crate) fn emit_element(
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

    pub(crate) fn emit_void(&mut self, name: &str, attrs: &[(&str, &str)], span: Span) {
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

    pub(crate) fn emit_checkbox(&mut self, checked: bool, span: Span) {
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

    pub(crate) fn emit_attrs(&mut self, attrs: &[(&str, &str)], span: Span) {
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

    pub(crate) fn emit_toc(&mut self, headings: &[HeadingInfo], span: Span) {
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

    pub(crate) fn emit_toc_links(&mut self, outline: &[&HeadingInfo], hrefs: &[String]) {
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

    pub(crate) fn emit_toc_menu(&mut self, outline: &[&HeadingInfo], hrefs: &[String], span: Span) {
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

    pub(crate) fn emit_main(&mut self, span: Span) {
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

    pub(crate) fn emit_tagged_text(
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

    pub(crate) fn emit_default_page(
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

    pub(crate) fn emit_toc_script(&mut self, span: Span) {
        let html = format!("<script>{}</script>", rocci_theme::TOC_SCRIPT.trim());
        self.push_indent();
        self.emit_html(".dangerously_include_unescaped_html(");
        self.emit_string(&html, span, OriginKind::Scaffolding);
        self.emit("),\n");
    }

    pub(crate) fn emit_style_element(&mut self, css: &str, span: Span) {
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

    pub(crate) fn emit_source(&mut self, span: Span, text: &str, origin: OriginKind) {
        self.maybe_indent();
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

    pub(crate) fn maybe_indent(&mut self) {
        if self.at_line_start {
            for _ in 0..self.indent {
                self.roc.push_str("    ");
            }
            self.at_line_start = false;
        }
    }
}
