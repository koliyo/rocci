use rocci_rocdown::{Document, Item, MdNode};

pub fn is_static_document(document: &Document) -> Result<(), &'static str> {
    for item in &document.items {
        match item {
            Item::Markdown(_) | Item::Page(_) | Item::Docs(_) | Item::Img(_) => {}
            Item::Render(_) => return Err("@render"),
            Item::Roc(_) => return Err("@roc"),
            Item::Component(_) => return Err("@component"),
            Item::Fixture(_) => return Err("@fixture"),
            Item::Css(_) => return Err("@css"),
            Item::Context(_) => return Err("@context"),
            Item::Init(_) => return Err("@init"),
            Item::On(_) => return Err("@on"),
            Item::Template(_) => return Err("Rocci template"),
        }
    }
    Ok(())
}

pub fn render_document(document: &Document) -> String {
    let mut parts = Vec::new();
    let mut footnotes = Vec::new();
    for item in &document.items {
        if let Item::Markdown(node) = item {
            if matches!(node, MdNode::FootnoteDefinition { .. }) {
                footnotes.push(render_md(node));
            } else {
                parts.push(render_md(node));
            }
        }
    }
    if !footnotes.is_empty() {
        let list = element("ol", &[attribute("class", "rd-footnote-list")], &footnotes);
        parts.push(element(
            "section",
            &[
                attribute("class", "rd-footnotes"),
                boolean_attribute("data-footnotes", true),
                attribute("aria-label", "Footnotes"),
            ],
            &[list],
        ));
    }
    fragment(&parts)
}

pub(crate) fn render_md(node: &MdNode) -> String {
    match node {
        MdNode::Heading {
            level,
            id,
            children,
            ..
        } => {
            let tag = format!("h{level}");
            let class = format!("rd-header-{level}");
            element(
                &tag,
                &[attribute("class", &class), attribute("id", id)],
                &render_all(children),
            )
        }
        MdNode::Paragraph { children, .. } => element(
            "p",
            &[attribute("class", "rd-paragraph")],
            &render_all(children),
        ),
        MdNode::BlockQuote { children, .. } => element(
            "blockquote",
            &[attribute("class", "rd-blockquote")],
            &render_all(children),
        ),
        MdNode::List {
            ordered,
            start,
            children,
            ..
        } => {
            let name = if *ordered { "ol" } else { "ul" };
            let class = if *ordered {
                "rd-list-ordered"
            } else {
                "rd-list"
            };
            let mut attrs = vec![attribute("class", class)];
            if *ordered && *start != 1 {
                attrs.push(attribute("start", &start.to_string()));
            }
            element(name, &attrs, &render_all(children))
        }
        MdNode::Item { children, .. } => element(
            "li",
            &[attribute("class", "rd-list-item")],
            &render_all(children),
        ),
        MdNode::TaskItem {
            checked, children, ..
        } => {
            let mut kids = vec![checkbox(*checked), text(" ")];
            kids.extend(render_all(children));
            element("li", &[attribute("class", "rd-task-item")], &kids)
        }
        MdNode::CodeBlock { info, literal, .. } => {
            let code_class = if info.is_empty() {
                "rd-code".to_string()
            } else {
                format!("rd-code language-{info}")
            };
            let code = element("code", &[attribute("class", &code_class)], &[text(literal)]);
            element("pre", &[attribute("class", "rd-code-block")], &[code])
        }
        MdNode::ThematicBreak { .. } => {
            void_element("hr", &[attribute("class", "rd-thematic-break")])
        }
        MdNode::Table { children, .. } => {
            let mut head = Vec::new();
            let mut body = Vec::new();
            for child in children {
                match child {
                    MdNode::TableRow { header: true, .. } => head.push(render_md(child)),
                    _ => body.push(render_md(child)),
                }
            }
            let mut sections = Vec::new();
            if !head.is_empty() {
                sections.push(element(
                    "thead",
                    &[attribute("class", "rd-table-head")],
                    &head,
                ));
            }
            if !body.is_empty() {
                sections.push(element(
                    "tbody",
                    &[attribute("class", "rd-table-body")],
                    &body,
                ));
            }
            element("table", &[attribute("class", "rd-table")], &sections)
        }
        MdNode::TableRow {
            header, children, ..
        } => {
            let cells = children
                .iter()
                .map(|child| {
                    if *header {
                        if let MdNode::TableCell { children, .. } = child {
                            element(
                                "th",
                                &[attribute("class", "rd-table-header")],
                                &render_all(children),
                            )
                        } else {
                            render_md(child)
                        }
                    } else {
                        render_md(child)
                    }
                })
                .collect::<Vec<_>>();
            element("tr", &[attribute("class", "rd-table-row")], &cells)
        }
        MdNode::TableCell { children, .. } => element(
            "td",
            &[attribute("class", "rd-table-cell")],
            &render_all(children),
        ),
        MdNode::Text { value, .. } => text(value),
        MdNode::SoftBreak { .. } => text("\n"),
        MdNode::LineBreak { .. } => void_element("br", &[]),
        MdNode::Code { value, .. } => {
            element("code", &[attribute("class", "rd-code")], &[text(value)])
        }
        MdNode::Emph { children, .. } => element(
            "em",
            &[attribute("class", "rd-emphasis")],
            &render_all(children),
        ),
        MdNode::Strong { children, .. } => element(
            "strong",
            &[attribute("class", "rd-strong")],
            &render_all(children),
        ),
        MdNode::Strikethrough { children, .. } => element(
            "del",
            &[attribute("class", "rd-strikethrough")],
            &render_all(children),
        ),
        MdNode::FootnoteDefinition {
            name,
            total_references,
            children,
            ..
        } => {
            let mut body = render_all(children);
            let mut backlinks = Vec::new();
            for reference_number in 1..=*total_references {
                let suffix = if reference_number == 1 {
                    String::new()
                } else {
                    format!("-{reference_number}")
                };
                backlinks.push(element(
                    "a",
                    &[
                        attribute("class", "rd-footnote-backref"),
                        attribute("href", &format!("#fnref-{name}{suffix}")),
                        boolean_attribute("data-footnote-backref", true),
                        attribute("aria-label", &format!("Back to reference {name}{suffix}")),
                    ],
                    &[text("↩")],
                ));
            }
            body.push(element(
                "span",
                &[attribute("class", "rd-footnote-backlinks")],
                &backlinks,
            ));
            element(
                "li",
                &[
                    attribute("class", "rd-footnote-definition"),
                    attribute("id", &format!("fn-{name}")),
                ],
                &body,
            )
        }
        MdNode::FootnoteReference {
            name,
            reference_number,
            index,
            ..
        } => {
            let suffix = if *reference_number == 1 {
                String::new()
            } else {
                format!("-{reference_number}")
            };
            let link = element(
                "a",
                &[
                    attribute("href", &format!("#fn-{name}")),
                    attribute("id", &format!("fnref-{name}{suffix}")),
                    boolean_attribute("data-footnote-ref", true),
                    attribute("aria-label", &format!("Footnote {index}")),
                ],
                &[text(&index.to_string())],
            );
            element("sup", &[attribute("class", "rd-footnote-ref")], &[link])
        }
        MdNode::Link {
            url,
            title,
            children,
            ..
        } => {
            let mut attrs = vec![attribute("class", "rd-link"), attribute("href", url)];
            if !title.is_empty() {
                attrs.push(attribute("title", title));
            }
            element("a", &attrs, &render_all(children))
        }
        MdNode::Image {
            url, title, alt, ..
        } => {
            let mut attrs = vec![
                attribute("class", "rd-image"),
                attribute("src", url),
                attribute("alt", alt),
            ];
            if !title.is_empty() {
                attrs.push(attribute("title", title));
            }
            void_element("img", &attrs)
        }
        MdNode::RawHtml { html, .. } => html.clone(),
    }
}

fn render_all(children: &[MdNode]) -> Vec<String> {
    children.iter().map(render_md).collect()
}

fn checkbox(checked: bool) -> String {
    let mut attrs = vec![
        attribute("type", "checkbox"),
        boolean_attribute("disabled", true),
    ];
    if checked {
        attrs.push(boolean_attribute("checked", true));
    }
    void_element("input", &attrs)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn attribute(name: &str, value: &str) -> String {
    format!(" {name}=\"{}\"", escape(value))
}

fn boolean_attribute(name: &str, enabled: bool) -> String {
    if enabled {
        format!(" {name}")
    } else {
        String::new()
    }
}

fn text(value: &str) -> String {
    escape(value)
}

fn element(name: &str, attrs: &[String], children: &[String]) -> String {
    format!("<{name}{}>{}</{name}>", attrs.concat(), children.concat())
}

fn void_element(name: &str, attrs: &[String]) -> String {
    format!("<{name}{} />", attrs.concat())
}

fn fragment(nodes: &[String]) -> String {
    nodes.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocci_rocdown::{CompileOptions, SourceFile, compile};

    fn html(src: &str) -> String {
        let out = compile(
            SourceFile::new("test.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        assert!(is_static_document(&out.document).is_ok());
        render_document(&out.document)
    }

    #[test]
    fn renders_heading_paragraph_link_and_escapes() {
        let rendered = html("# Welcome\n\nSee the [Guide](/guide/).\n\nA & B < C\n");
        assert_eq!(
            rendered,
            "<h1 class=\"rd-header-1\" id=\"welcome\">Welcome</h1>\
<p class=\"rd-paragraph\">See the <a class=\"rd-link\" href=\"/guide/\">Guide</a>.</p>\
<p class=\"rd-paragraph\">A &amp; B &lt; C</p>"
        );
    }

    #[test]
    fn renders_lists_code_task_and_table() {
        let rendered = html(
            "\
- one
- two

1. first

```roc
1 < 2
```

- [x] done

| A | B |
| --- | --- |
| 1 | 2 |
",
        );
        assert!(rendered.contains("<ul class=\"rd-list\">"));
        assert!(rendered.contains("<li class=\"rd-list-item\">"));
        assert!(rendered.contains("one"));
        assert!(rendered.contains("<ol class=\"rd-list-ordered\">"));
        assert!(rendered.contains("<pre class=\"rd-code-block\">"));
        assert!(rendered.contains("<code class=\"rd-code language-roc\">"));
        assert!(rendered.contains("1 &lt; 2"));
        assert!(rendered.contains("<li class=\"rd-task-item\">"));
        assert!(rendered.contains("<input type=\"checkbox\" disabled checked />"));
        assert!(rendered.contains("<table class=\"rd-table\">"));
        assert!(rendered.contains("<th class=\"rd-table-header\">A</th>"));
        assert!(rendered.contains("<td class=\"rd-table-cell\">1</td>"));
    }

    #[test]
    fn rejects_render_islands() {
        let src = "# Hi\n\n@render {\n    Html.text(\"x\")\n}\n";
        let out = compile(
            SourceFile::new("island.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        assert_eq!(is_static_document(&out.document), Err("@render"));
    }

    #[test]
    fn quotes_in_attributes_and_text() {
        let rendered = html("Say \"hi\" and [Go](https://ex.com \"T & T\")\n");
        assert!(rendered.contains("Say &quot;hi&quot;"));
        assert!(rendered.contains("title=\"T &amp; T\""));
    }
}
