use std::collections::HashMap;

use comrak::nodes::{AstNode, ListType, NodeValue};
use rocci_template::{Diagnostic, Span};

use crate::ast::{HeadingInfo, LinkInfo, MdNode};

pub struct MarkdownConvert {
    pub blocks: Vec<BlockOrHole>,
    pub headings: Vec<HeadingInfo>,
    pub links: Vec<LinkInfo>,
    heading_ids: HashMap<String, usize>,
    raw_html: bool,
}

#[derive(Clone, Debug)]
pub enum BlockOrHole {
    Block(MdNode),
    Hole(usize),
}

pub(crate) struct OffsetMap {
    regions: Vec<MappedRegion>,
}

struct MappedRegion {
    synthetic: std::ops::Range<usize>,
    original: Option<std::ops::Range<usize>>,
}

impl OffsetMap {
    pub(crate) fn from_original(original: std::ops::Range<usize>) -> Self {
        Self {
            regions: vec![MappedRegion {
                synthetic: 0..original.len(),
                original: Some(original),
            }],
        }
    }

    fn original(&self, syn: usize) -> usize {
        for region in &self.regions {
            if syn >= region.synthetic.start && syn <= region.synthetic.end {
                if let Some(orig) = &region.original {
                    let delta = syn - region.synthetic.start;
                    return orig.start + delta.min(orig.len());
                }
                if let Some(prev) = self
                    .regions
                    .iter()
                    .rev()
                    .find(|r| r.synthetic.end <= syn && r.original.is_some())
                {
                    return prev.original.as_ref().unwrap().end;
                }
                if let Some(orig) = self.regions.iter().find_map(|r| r.original.as_ref()) {
                    return orig.start;
                }
            }
        }
        syn
    }
}

pub fn punch_holes(src: &str, decls: &[crate::scan::ScannedDecl]) -> (String, OffsetMap) {
    let mut synthetic = String::new();
    let mut regions = Vec::new();
    let mut orig = crate::scan::bom_len(src);
    for (i, decl) in decls.iter().enumerate() {
        if orig < decl.line_start {
            let syn_start = synthetic.len();
            synthetic.push_str(&src[orig..decl.line_start]);
            regions.push(MappedRegion {
                synthetic: syn_start..synthetic.len(),
                original: Some(orig..decl.line_start),
            });
        }
        let syn_start = synthetic.len();
        synthetic.push_str("\n\n<!--rocdown:");
        synthetic.push_str(&i.to_string());
        synthetic.push_str("-->\n\n");
        regions.push(MappedRegion {
            synthetic: syn_start..synthetic.len(),
            original: None,
        });
        orig = decl.end;
    }
    if orig < src.len() {
        let syn_start = synthetic.len();
        synthetic.push_str(&src[orig..]);
        regions.push(MappedRegion {
            synthetic: syn_start..synthetic.len(),
            original: Some(orig..src.len()),
        });
    }
    (synthetic, OffsetMap { regions })
}

pub fn punch_holes_range(
    src: &str,
    start: usize,
    end: usize,
    decls: &[crate::scan::ScannedDecl],
) -> (String, OffsetMap) {
    let mut synthetic = String::new();
    let mut regions = Vec::new();
    let mut orig = start.min(end);
    let end = end.min(src.len());
    for (i, decl) in decls.iter().enumerate() {
        if orig < decl.line_start {
            let syn_start = synthetic.len();
            synthetic.push_str(&src[orig..decl.line_start.min(end)]);
            regions.push(MappedRegion {
                synthetic: syn_start..synthetic.len(),
                original: Some(orig..decl.line_start.min(end)),
            });
        }
        let syn_start = synthetic.len();
        synthetic.push_str("\n\n<!--rocdown:");
        synthetic.push_str(&i.to_string());
        synthetic.push_str("-->\n\n");
        regions.push(MappedRegion {
            synthetic: syn_start..synthetic.len(),
            original: None,
        });
        orig = decl.end.min(end);
    }
    if orig < end {
        let syn_start = synthetic.len();
        synthetic.push_str(&src[orig..end]);
        regions.push(MappedRegion {
            synthetic: syn_start..synthetic.len(),
            original: Some(orig..end),
        });
    }
    (synthetic, OffsetMap { regions })
}

enum Converted {
    Block(MdNode),
    Hole(usize),
    Skip,
}

pub fn convert_document<'a>(
    root: &'a AstNode<'a>,
    synthetic: &str,
    map: &OffsetMap,
    raw_html: bool,
    diagnostics: &mut Vec<Diagnostic>,
) -> MarkdownConvert {
    let mut out = MarkdownConvert {
        blocks: Vec::new(),
        headings: Vec::new(),
        links: Vec::new(),
        heading_ids: HashMap::new(),
        raw_html,
    };
    for child in root.children() {
        match out.convert_top(child, synthetic, map, diagnostics) {
            Converted::Block(node) => out.blocks.push(BlockOrHole::Block(node)),
            Converted::Hole(index) => out.blocks.push(BlockOrHole::Hole(index)),
            Converted::Skip => {}
        }
    }
    out
}

impl MarkdownConvert {
    fn convert_top<'a>(
        &mut self,
        node: &'a AstNode<'a>,
        synthetic: &str,
        map: &OffsetMap,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Converted {
        let data = node.data.borrow();
        match &data.value {
            NodeValue::HtmlBlock(html) => {
                if let Some(index) = placeholder_index(&html.literal) {
                    Converted::Hole(index)
                } else if let Some(md) = self.convert_node(node, synthetic, map, diagnostics) {
                    Converted::Block(md)
                } else {
                    Converted::Skip
                }
            }
            _ => {
                if let Some(md) = self.convert_node(node, synthetic, map, diagnostics) {
                    Converted::Block(md)
                } else {
                    Converted::Skip
                }
            }
        }
    }

    fn convert_node<'a>(
        &mut self,
        node: &'a AstNode<'a>,
        synthetic: &str,
        map: &OffsetMap,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<MdNode> {
        let data = node.data.borrow();
        let span = source_span(synthetic, map, data.sourcepos);
        match &data.value {
            NodeValue::Document => None,
            NodeValue::Paragraph => Some(MdNode::Paragraph {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::Heading(heading) => {
                let children = self.convert_children(node, synthetic, map, diagnostics);
                let text = children
                    .iter()
                    .map(MdNode::text_content)
                    .collect::<String>();
                let id = self.assign_heading_id(&text);
                self.headings.push(HeadingInfo {
                    level: heading.level,
                    id: id.clone(),
                    text,
                    span,
                });
                Some(MdNode::Heading {
                    level: heading.level,
                    id,
                    children,
                    span,
                })
            }
            NodeValue::BlockQuote => Some(MdNode::BlockQuote {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::List(list) => Some(MdNode::List {
                ordered: list.list_type == ListType::Ordered,
                start: list.start as u64,
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::Item(_) => Some(MdNode::Item {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::TaskItem(item) => Some(MdNode::TaskItem {
                checked: item.symbol.is_some(),
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::CodeBlock(block) => Some(MdNode::CodeBlock {
                info: block
                    .info
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string(),
                literal: block.literal.clone(),
                span,
            }),
            NodeValue::ThematicBreak => Some(MdNode::ThematicBreak { span }),
            NodeValue::Table(_) => Some(MdNode::Table {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::TableRow(header) => Some(MdNode::TableRow {
                header: *header,
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::TableCell => Some(MdNode::TableCell {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::Text(text) => Some(MdNode::Text {
                value: text.to_string(),
                span,
            }),
            NodeValue::SoftBreak => Some(MdNode::SoftBreak { span }),
            NodeValue::LineBreak => Some(MdNode::LineBreak { span }),
            NodeValue::Code(code) => Some(MdNode::Code {
                value: code.literal.clone(),
                span,
            }),
            NodeValue::Emph => Some(MdNode::Emph {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::Strong => Some(MdNode::Strong {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::Strikethrough => Some(MdNode::Strikethrough {
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::FootnoteDefinition(definition) => Some(MdNode::FootnoteDefinition {
                name: definition.name.clone(),
                total_references: definition.total_references,
                children: self.convert_children(node, synthetic, map, diagnostics),
                span,
            }),
            NodeValue::FootnoteReference(reference) => Some(MdNode::FootnoteReference {
                name: reference.name.clone(),
                reference_number: reference.ref_num,
                index: reference.ix,
                span,
            }),
            NodeValue::Link(link) => {
                self.links.push(LinkInfo {
                    url: link.url.clone(),
                    span,
                });
                Some(MdNode::Link {
                    url: link.url.clone(),
                    title: link.title.clone(),
                    children: self.convert_children(node, synthetic, map, diagnostics),
                    span,
                })
            }
            NodeValue::WikiLink(link) => {
                self.links.push(LinkInfo {
                    url: link.url.clone(),
                    span,
                });
                Some(MdNode::Link {
                    url: link.url.clone(),
                    title: String::new(),
                    children: self.convert_children(node, synthetic, map, diagnostics),
                    span,
                })
            }
            NodeValue::Image(link) => {
                let alt = node
                    .children()
                    .filter_map(|child| self.convert_node(child, synthetic, map, diagnostics))
                    .map(|child| child.text_content())
                    .collect::<String>();
                Some(MdNode::Image {
                    url: link.url.clone(),
                    title: link.title.clone(),
                    alt,
                    span,
                })
            }
            NodeValue::HtmlBlock(html) => {
                self.raw_html_node(html.literal.clone(), span, diagnostics)
            }
            NodeValue::HtmlInline(html) => self.raw_html_node(html.clone(), span, diagnostics),
            _ => {
                let children = self.convert_children(node, synthetic, map, diagnostics);
                if children.is_empty() {
                    None
                } else if children.len() == 1 {
                    children.into_iter().next()
                } else {
                    Some(MdNode::Paragraph { children, span })
                }
            }
        }
    }

    fn convert_children<'a>(
        &mut self,
        node: &'a AstNode<'a>,
        synthetic: &str,
        map: &OffsetMap,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Vec<MdNode> {
        node.children()
            .filter_map(|child| self.convert_node(child, synthetic, map, diagnostics))
            .collect()
    }

    fn raw_html_node(
        &mut self,
        html: String,
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<MdNode> {
        if self.raw_html {
            Some(MdNode::RawHtml { html, span })
        } else {
            diagnostics.push(Diagnostic::error(
                span,
                "raw HTML is disabled in Rocdown; use Markdown or @render { ... }",
            ));
            None
        }
    }

    fn assign_heading_id(&mut self, text: &str) -> String {
        let mut base = slugify(text);
        if base.is_empty() {
            base = "heading".to_string();
        }
        let count = self.heading_ids.entry(base.clone()).or_insert(0);
        let id = if *count == 0 {
            base.clone()
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        id
    }
}

fn placeholder_index(literal: &str) -> Option<usize> {
    let text = literal.trim();
    let rest = text.strip_prefix("<!--rocdown:")?;
    let rest = rest.strip_suffix("-->")?;
    rest.trim().parse().ok()
}

fn slugify(text: &str) -> String {
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

fn source_span(synthetic: &str, map: &OffsetMap, pos: comrak::nodes::Sourcepos) -> Span {
    let start = line_col_offset(synthetic, pos.start.line, pos.start.column);
    let end = line_col_offset(synthetic, pos.end.line, pos.end.column.saturating_add(1));
    Span::new(map.original(start), map.original(end.max(start)))
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
