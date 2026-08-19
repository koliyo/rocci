use serde::Serialize;

use crate::{Document, Item, MdNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PageKind {
    Static,
    Hydrate,
    Live,
}

impl PageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Hydrate => "hydrate",
            Self::Live => "live",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageClass {
    pub kind: PageKind,
    pub reason: &'static str,
}

pub fn classify_document(document: &Document, uses_datastar: bool) -> PageClass {
    let mut class = PageClass {
        kind: PageKind::Static,
        reason: "Markdown",
    };
    for item in &document.items {
        let (kind, reason) = match item {
            Item::Markdown(_) | Item::Page(_) | Item::Block(_) | Item::Use(_) => continue,
            Item::Render(_) => (PageKind::Hydrate, "@render"),
            Item::Roc(_) => (PageKind::Hydrate, "@roc"),
            Item::Component(_) => (PageKind::Hydrate, "@component"),
            Item::Fixture(_) => (PageKind::Hydrate, "@fixture"),
            Item::Css(_) => (PageKind::Hydrate, "@css"),
            Item::Template(_) => (PageKind::Hydrate, "Rocci template"),
            Item::Context(_) => (PageKind::Live, "@context"),
            Item::Init(_) => (PageKind::Live, "@init"),
            Item::On(_) => (PageKind::Live, "@on"),
        };
        if kind > class.kind {
            class.kind = kind;
            class.reason = reason;
        }
    }
    if uses_datastar && class.kind < PageKind::Live {
        class.kind = PageKind::Live;
        class.reason = "import Datastar";
    }
    class
}

pub fn is_static_document(document: &Document) -> Result<(), &'static str> {
    for item in &document.items {
        match item {
            Item::Markdown(_) | Item::Page(_) | Item::Block(_) => {}
            Item::Render(_) => return Err("@render"),
            Item::Roc(_) => return Err("@roc"),
            Item::Component(_) => return Err("@component"),
            Item::Fixture(_) => return Err("@fixture"),
            Item::Css(_) => return Err("@css"),
            Item::Context(_) => return Err("@context"),
            Item::Init(_) => return Err("@init"),
            Item::On(_) => return Err("@on"),
            Item::Use(_) => return Err("@use"),
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
        parts.push(render_footnote_section(&footnotes));
    }
    fragment(&parts)
}

pub(crate) fn render_footnote_section(items: &[String]) -> String {
    let list = element("ol", &[attribute("class", "rd-footnote-list")], items);
    element(
        "section",
        &[
            attribute("class", "rd-footnotes"),
            boolean_attribute("data-footnotes", true),
            attribute("aria-label", "Footnotes"),
        ],
        &[list],
    )
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
            let lang = rocci_highlight::LanguageId::parse(info);
            let mut pre_attrs = vec![attribute("class", "rd-code-block")];
            if lang.is_highlighted() {
                pre_attrs.push(attribute("data-language", lang.canonical_name()));
            }
            let code_class = if info.is_empty() {
                "rd-code".to_string()
            } else {
                format!("rd-code language-{}", lang.canonical_name())
            };
            let code_html = if lang.is_highlighted() {
                render_highlighted_code(&lang, literal)
            } else {
                escape(literal)
            };
            let code = format!(
                "<code class=\"{}\">{}</code>",
                escape(&code_class),
                code_html
            );
            element("pre", &pre_attrs, &[code])
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

pub(crate) fn render_static_image(image: &crate::StaticImage) -> String {
    let attrs: Vec<String> = image
        .html_attrs()
        .iter()
        .map(|attr| attribute(attr.name, &attr.value))
        .collect();
    void_element("img", &attrs)
}

pub(crate) fn render_heading(tag: &str, class: &str, id: &str, children: &[String]) -> String {
    element(
        tag,
        &[attribute("class", class), attribute("id", id)],
        children,
    )
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

fn render_highlighted_code(lang: &rocci_highlight::LanguageId, literal: &str) -> String {
    let spans = match lang {
        rocci_highlight::LanguageId::Rocdown | rocci_highlight::LanguageId::Markdown => {
            crate::highlight_rocdown(literal)
        }
        _ => rocci_highlight::highlight(lang.clone(), literal),
    };
    if spans.is_empty() {
        return escape(literal);
    }
    let mut html = String::with_capacity(literal.len() * 2);
    let mut prev_end = 0usize;
    for span in spans {
        let start = span.start().min(literal.len());
        let end = span.end().min(literal.len());
        if start > prev_end {
            html.push_str(&escape(&literal[prev_end..start]));
        }
        if start < end {
            let kind_class = span.kind.css_class();
            let mod_classes = rocci_highlight::modifier_css_classes(span.modifiers);
            let class_str = if mod_classes.is_empty() {
                kind_class.to_string()
            } else {
                format!("{kind_class} {}", mod_classes.join(" "))
            };
            html.push_str("<span class=\"");
            html.push_str(&class_str);
            html.push_str("\">");
            html.push_str(&escape(&literal[start..end]));
            html.push_str("</span>");
        }
        prev_end = end;
    }
    if prev_end < literal.len() {
        html.push_str(&escape(&literal[prev_end..]));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docs::{IncludeOptions, load_page_docs, render_article};
    use crate::{CompileOptions, SourceFile, compile};
    use std::path::Path;

    fn html(src: &str) -> String {
        let source = SourceFile::new("test.rocdown", src);
        let out = compile(
            source,
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        assert!(!out.has_errors(), "{:?}", out.diagnostics);
        assert!(is_static_document(&out.document).is_ok());
        assert_eq!(
            classify_document(&out.document, false).kind,
            PageKind::Static
        );
        let mut diagnostics = Vec::new();
        let docs = load_page_docs(
            source,
            &out.document,
            "test.rocdown",
            IncludeOptions {
                root: Path::new("."),
                snippet_roots: &[],
            },
            &mut diagnostics,
        );
        assert!(!diagnostics.iter().any(|d| d.is_error()), "{diagnostics:?}");
        render_article(&docs.article)
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
        assert!(rendered.contains("<pre class=\"rd-code-block\" data-language=\"roc\">"));
        assert!(rendered.contains("<code class=\"rd-code language-roc\">"));
        assert!(rendered.contains("<span class=\"tok-number\">1</span>"));
        assert!(rendered.contains("<span class=\"tok-operator\">&lt;</span>"));
        assert!(rendered.contains("<span class=\"tok-number\">2</span>"));
        assert!(rendered.contains("<li class=\"rd-task-item\">"));
        assert!(rendered.contains("<input type=\"checkbox\" disabled checked />"));
        assert!(rendered.contains("<table class=\"rd-table\">"));
        assert!(rendered.contains("<th class=\"rd-table-header\">A</th>"));
        assert!(rendered.contains("<td class=\"rd-table-cell\">1</td>"));
    }

    #[test]
    fn renders_syntax_highlighted_fences_for_all_languages() {
        let rendered = html(
            "\
```html
<div class=\"card\"><p>Hello &amp; World</p></div>
```

```css
.card { color: #fff; margin: 10px; }
```

```rocci
@component Header = |{ title }| { <h1>{title}</h1> }
```

```rocdown
# Section

Text in rocdown.
```

```unknown_lang
<script>alert(1)</script>
```
",
        );
        assert!(rendered.contains("<pre class=\"rd-code-block\" data-language=\"html\"><code class=\"rd-code language-html\">"));
        assert!(rendered.contains("<span class=\"tok-tag tok-default-library\">div</span>"));
        assert!(rendered.contains("&amp;amp;"));

        assert!(rendered.contains("<pre class=\"rd-code-block\" data-language=\"css\"><code class=\"rd-code language-css\">"));
        assert!(rendered.contains("<span class=\"tok-property\">color</span>"));
        assert!(rendered.contains("<span class=\"tok-number\">#fff</span>"));

        assert!(rendered.contains("<pre class=\"rd-code-block\" data-language=\"rocci\"><code class=\"rd-code language-rocci\">"));
        assert!(rendered.contains("<span class=\"tok-keyword\">@component</span>"));
        assert!(rendered.contains("<span class=\"tok-function tok-definition\">Header</span>"));

        assert!(rendered.contains("<pre class=\"rd-code-block\" data-language=\"rocdown\"><code class=\"rd-code language-rocdown\">"));
        assert!(rendered.contains("<span class=\"tok-keyword\">#</span>"));

        // Fallback for unknown language safely escapes HTML and omits data-language
        assert!(rendered.contains("<pre class=\"rd-code-block\"><code class=\"rd-code language-unknown_lang\">&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    fn classify(src: &str) -> PageClass {
        classify_with_datastar(src, false)
    }

    fn classify_with_datastar(src: &str, uses_datastar: bool) -> PageClass {
        let out = compile(
            SourceFile::new("page.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        classify_document(&out.document, uses_datastar)
    }

    #[test]
    fn classifies_markdown_and_docs_as_static() {
        assert_eq!(classify("# Hi\n").kind, PageKind::Static);
        assert_eq!(
            classify("# Hi\n\n:note[title: \"Watch\"] {{\n    Read this.\n}}\n").kind,
            PageKind::Static
        );
        assert_eq!(
            classify("@page { meta: { title: \"Home\" } }\n\n# Hi\n").kind,
            PageKind::Static
        );
    }

    #[test]
    fn classifies_render_as_hydrate() {
        let class = classify("# Hi\n\n@render {\n    Html.text(\"x\")\n}\n");
        assert_eq!(class.kind, PageKind::Hydrate);
        assert_eq!(class.reason, "@render");
    }

    #[test]
    fn classifies_component_css_and_roc_as_hydrate() {
        assert_eq!(
            classify("@component Box = || { <div /> }\n\n# Hi\n").kind,
            PageKind::Hydrate
        );
        assert_eq!(
            classify("@css { body { margin: 0; } }\n\n# Hi\n").kind,
            PageKind::Hydrate
        );
        assert_eq!(classify("@roc { n = 1 }\n\n# Hi\n").kind, PageKind::Hydrate);
        assert_eq!(classify("<Box />\n\n# Hi\n").kind, PageKind::Hydrate);
    }

    #[test]
    fn classifies_on_as_live() {
        let class = classify("# Hi\n\n@on:post(\"/inc\") = |_| {\n    Html.text(\"x\")\n}\n");
        assert_eq!(class.kind, PageKind::Live);
        assert_eq!(class.reason, "@on");
    }

    #[test]
    fn classifies_context_and_init_as_live() {
        assert_eq!(classify("@context AppState\n\n# Hi\n").kind, PageKind::Live);
        assert_eq!(classify("@init {\n    0\n}\n\n# Hi\n").kind, PageKind::Live);
    }

    #[test]
    fn live_wins_over_hydrate() {
        let class = classify(
            "@component Box = || { <div /> }\n\n@on:post(\"/inc\") = |_| {\n    Html.text(\"x\")\n}\n\n# Hi\n",
        );
        assert_eq!(class.kind, PageKind::Live);
        assert_eq!(class.reason, "@on");
    }

    #[test]
    fn datastar_import_promotes_to_live() {
        let class = classify_with_datastar("@roc { n = 1 }\n\n# Hi\n", true);
        assert_eq!(class.kind, PageKind::Live);
        assert_eq!(class.reason, "import Datastar");
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
        assert_eq!(classify_document(&out.document, false).kind, PageKind::Hydrate);
    }

    #[test]
    fn rejects_use_imports() {
        let src = "# Hi\n\n@use \"./Callout.rocci\"\n";
        let out = compile(
            SourceFile::new("island.rocdown", src),
            &CompileOptions {
                resolve_links: false,
                ..CompileOptions::default()
            },
        );
        assert_eq!(is_static_document(&out.document), Err("@use"));
    }

    #[test]
    fn quotes_in_attributes_and_text() {
        let rendered = html("Say \"hi\" and [Go](https://ex.com \"T & T\")\n");
        assert!(rendered.contains("Say &quot;hi&quot;"));
        assert!(rendered.contains("title=\"T &amp; T\""));
    }
}
