use rocci_template::OriginKind;

use crate::ast::{Document, Item, MdNode};

use super::emitter::Emitter;

impl<'a> Emitter<'a> {
    pub(crate) fn emit_footnote_section(&mut self, document: &Document) {
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

    pub(crate) fn emit_footnote_definition(&mut self, node: &MdNode) {
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
}

impl<'a> Emitter<'a> {
    pub(crate) fn lower_md(&mut self, node: &MdNode) {
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
            MdNode::Interpolation { expr, .. } => {
                self.emit_html(".text(");
                self.emit_mapped(
                    expr.of(self.source.src).trim(),
                    *expr,
                    OriginKind::TextExpression,
                );
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
}
