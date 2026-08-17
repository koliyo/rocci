use rocci_rocdown::MdNode;
use rocci_template::Span;

use super::tree_sitter::HighlightToken;
use crate::tokens::*;

pub fn collect_markdown_node(src: &str, node: &MdNode, tokens: &mut Vec<HighlightToken>) {
    match node {
        MdNode::Heading {
            level,
            span,
            children,
            ..
        } => {
            if let Some(marker) = heading_marker(src, *span, *level) {
                tokens.push(HighlightToken {
                    span: marker,
                    kind: TOKEN_KEYWORD,
                    modifiers: 0,
                    priority: 50,
                });
            }
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
        }
        MdNode::Code { span, .. } => {
            if !span.is_empty() {
                tokens.push(HighlightToken {
                    span: *span,
                    kind: TOKEN_STRING,
                    modifiers: 0,
                    priority: 40,
                });
            }
        }
        MdNode::Link {
            children,
            span,
            url,
            ..
        } => {
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
            if !url.is_empty() {
                let text = span.of(src);
                if let Some(idx) = text.rfind(url) {
                    let start = span.start as usize + idx;
                    tokens.push(HighlightToken {
                        span: Span::new(start, start + url.len()),
                        kind: TOKEN_STRING,
                        modifiers: 0,
                        priority: 40,
                    });
                }
            }
        }
        MdNode::Image { span, url, .. } => {
            if !url.is_empty() {
                let text = span.of(src);
                if let Some(idx) = text.rfind(url) {
                    let start = span.start as usize + idx;
                    tokens.push(HighlightToken {
                        span: Span::new(start, start + url.len()),
                        kind: TOKEN_STRING,
                        modifiers: 0,
                        priority: 40,
                    });
                }
            }
        }
        MdNode::ThematicBreak { span } => {
            if !span.is_empty() {
                tokens.push(HighlightToken {
                    span: *span,
                    kind: TOKEN_OPERATOR,
                    modifiers: 0,
                    priority: 40,
                });
            }
        }
        MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. } => {
            for child in children {
                collect_markdown_node(src, child, tokens);
            }
        }
        MdNode::CodeBlock { .. }
        | MdNode::Text { .. }
        | MdNode::SoftBreak { .. }
        | MdNode::LineBreak { .. }
        | MdNode::FootnoteReference { .. }
        | MdNode::RawHtml { .. } => {}
    }
}

pub fn heading_marker(src: &str, span: Span, level: u8) -> Option<Span> {
    let text = span.of(src);
    let indent = text.len() - text.trim_start_matches([' ', '\t']).len();
    let trimmed = &text[indent..];
    let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
    if hashes == 0 {
        return None;
    }
    let start = span.start as usize + indent;
    Some(Span::new(start, start + hashes.min(level as usize)))
}
