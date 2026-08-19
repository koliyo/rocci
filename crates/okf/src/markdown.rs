use std::collections::{BTreeSet, HashMap};

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};

use crate::ast::{Heading, HeadingSection, Link, Span};
use crate::diagnostic::Diagnostic;
use crate::frontmatter::{LineIndex, lines_with_offsets, location};
use crate::graph::published_href;

pub struct MarkdownOutput {
    pub headings: Vec<Heading>,
    pub heading_sections: Vec<HeadingSection>,
    pub links: Vec<Link>,
    pub footnote_ids: BTreeSet<String>,
    pub defined_footnotes: BTreeSet<String>,
    pub article_html: String,
}

pub fn comrak_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.footnotes = true;
    options
}

pub fn parse_markdown_body(
    relative: &str,
    source: &str,
    body: Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> MarkdownOutput {
    reject_declarations(relative, source, body, diagnostics);

    let body_str = body.of(source);
    let lines = LineIndex::new(source);
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, body_str, &options);

    let mut walker = MarkdownWalker {
        relative,
        source,
        lines: &lines,
        body_offset: body.start as usize,
        body_str,
        headings: Vec::new(),
        links: Vec::new(),
        footnote_ids: BTreeSet::new(),
        defined_footnotes: BTreeSet::new(),
        heading_ids: HashMap::new(),
        diagnostics,
    };

    walker.walk(root);
    rewrite_article_links(root, relative);

    // Extract heading sections for search indexing
    let mut heading_sections = Vec::new();
    let mut current_section: Option<HeadingSection> = None;
    for child in root.children() {
        let data = child.data.borrow();
        if let NodeValue::Heading(_) = &data.value {
            if let Some(prev) = current_section.take() {
                heading_sections.push(prev);
            }
            let text = collect_text(child);
            let mut ids_copy = walker.heading_ids.clone();
            let id = assign_heading_id(&mut ids_copy, &text);
            current_section = Some(HeadingSection {
                id,
                heading_text: text,
                body_texts: Vec::new(),
            });
        } else if let Some(section) = &mut current_section {
            let text = collect_text(child);
            if !text.trim().is_empty() {
                section.body_texts.push(text);
            }
        }
        drop(data);
    }
    if let Some(prev) = current_section {
        heading_sections.push(prev);
    }

    let mut article_html = String::new();
    let _ = comrak::format_html(root, &options, &mut article_html);

    MarkdownOutput {
        headings: walker.headings,
        heading_sections,
        links: walker.links,
        footnote_ids: walker.footnote_ids,
        defined_footnotes: walker.defined_footnotes,
        article_html,
    }
}

struct MarkdownWalker<'a> {
    relative: &'a str,
    source: &'a str,
    lines: &'a LineIndex,
    body_offset: usize,
    body_str: &'a str,
    headings: Vec<Heading>,
    links: Vec<Link>,
    footnote_ids: BTreeSet<String>,
    defined_footnotes: BTreeSet<String>,
    heading_ids: HashMap<String, usize>,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> MarkdownWalker<'a> {
    fn walk(&mut self, node: &'a AstNode<'a>) {
        let data = node.data.borrow();
        let span = source_span(self.body_str, self.body_offset, data.sourcepos);

        match &data.value {
            NodeValue::Heading(heading) => {
                let text = collect_text(node);
                let id = assign_heading_id(&mut self.heading_ids, &text);
                self.headings.push(Heading {
                    level: heading.level,
                    id,
                    text,
                    location: self.lines.location(self.source, span),
                });
            }
            NodeValue::Link(link) => {
                self.links.push(Link {
                    url: link.url.clone(),
                    location: self.lines.location(self.source, span),
                });
            }
            NodeValue::FootnoteReference(reference) => {
                self.footnote_ids.insert(reference.name.clone());
            }
            NodeValue::FootnoteDefinition(definition) => {
                self.defined_footnotes.insert(definition.name.clone());
            }
            NodeValue::HtmlBlock(block) => {
                self.diagnostics.push(Diagnostic::error(
                    "OKF2009",
                    self.relative,
                    Some(self.lines.location(self.source, span)),
                    format!(
                        "raw HTML is forbidden in knowledge records: `{}`",
                        block.literal.trim()
                    ),
                ));
            }
            NodeValue::HtmlInline(html) => {
                self.diagnostics.push(Diagnostic::error(
                    "OKF2009",
                    self.relative,
                    Some(self.lines.location(self.source, span)),
                    format!(
                        "raw HTML is forbidden in knowledge records: `{}`",
                        html.trim()
                    ),
                ));
            }
            NodeValue::Text(text) => {
                self.footnote_ids.extend(footnote_labels(text));
            }
            _ => {}
        }

        drop(data);

        for child in node.children() {
            self.walk(child);
        }
    }
}

fn rewrite_article_links<'a>(node: &'a AstNode<'a>, source_path: &str) {
    {
        let mut data = node.data.borrow_mut();
        if let NodeValue::Link(link) = &mut data.value
            && let Some(href) = published_href(source_path, &link.url)
        {
            link.url = href;
        }
    }
    for child in node.children() {
        rewrite_article_links(child, source_path);
    }
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    let data = node.data.borrow();
    if let NodeValue::Text(text) = &data.value {
        out.push_str(text);
    } else if let NodeValue::Code(code) = &data.value {
        out.push_str(&code.literal);
    }
    drop(data);

    for child in node.children() {
        out.push_str(&collect_text(child));
    }
    out
}

fn assign_heading_id(heading_ids: &mut HashMap<String, usize>, text: &str) -> String {
    let mut base = slugify(text);
    if base.is_empty() {
        base = "heading".to_string();
    }
    let count = heading_ids.entry(base.clone()).or_insert(0);
    let id = if *count == 0 {
        base.clone()
    } else {
        format!("{base}-{count}")
    };
    *count += 1;
    id
}

pub fn slugify(text: &str) -> String {
    let mut out = String::new();
    let mut hyphen = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            hyphen = false;
        } else if !out.is_empty() && !hyphen {
            out.push('-');
            hyphen = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

pub fn footnote_labels(text: &str) -> Vec<String> {
    let mut labels = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("[^") {
        let after = &remaining[start + 2..];
        let Some(end) = after.find(']') else {
            break;
        };
        let label = &after[..end];
        if !label.is_empty()
            && !label
                .chars()
                .any(|c| c.is_whitespace() || c == '[' || c == ']')
        {
            labels.push(label.to_string());
        }
        remaining = &after[end + 1..];
    }
    labels
}

fn source_span(body_str: &str, body_offset: usize, pos: comrak::nodes::Sourcepos) -> Span {
    let start = line_col_offset(body_str, pos.start.line, pos.start.column);
    let end = line_col_offset(body_str, pos.end.line, pos.end.column.saturating_add(1));
    Span::new(body_offset + start, body_offset + end.max(start))
}

fn line_col_offset(src: &str, line: usize, column: usize) -> usize {
    let mut cur_line = 1usize;
    let mut cur_col = 1usize;
    for (i, ch) in src.char_indices() {
        if cur_line == line && cur_col == column {
            return i;
        }
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
    }
    src.len()
}

pub fn reject_declarations(
    relative: &str,
    source: &str,
    body: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    const DECLARATIONS: &[&str] = &[
        "page",
        "roc",
        "render",
        "component",
        "fixture",
        "css",
        "context",
        "init",
        "on",
    ];
    let mut fence: Option<char> = None;
    for (relative_offset, line) in lines_with_offsets(body.of(source)) {
        let trimmed = line.trim_start();
        let fence_marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if let Some(marker) = fence_marker {
            match fence {
                Some(active) if active == marker => fence = None,
                None => fence = Some(marker),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix('@') else {
            continue;
        };
        let name = rest
            .split(|character: char| character.is_whitespace() || character == '{')
            .next()
            .unwrap_or("");
        if DECLARATIONS.contains(&name) {
            let indent = line.len() - trimmed.len();
            let start = body.start as usize + relative_offset + indent;
            diagnostics.push(Diagnostic::error(
                "OKF2007",
                relative,
                Some(location(source, Span::new(start, start + name.len() + 1))),
                format!("Rocdown declaration `@{name}` is forbidden in knowledge records"),
            ));
        }
    }
}
