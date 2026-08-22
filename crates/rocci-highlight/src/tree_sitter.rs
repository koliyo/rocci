use std::cell::RefCell;

use rocci_template::Span;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};

use crate::token::{HighlightKind, HighlightSpan};

thread_local! {
    static PARSER: RefCell<Parser> = RefCell::new(Parser::new());
    static CURSOR: RefCell<QueryCursor> = RefCell::new(QueryCursor::new());
}

pub struct TreeSitterHighlighter {
    language: Language,
    query: Query,
}

impl TreeSitterHighlighter {
    pub fn new(language: Language, query_source: &str) -> Result<Self, tree_sitter::QueryError> {
        let query = Query::new(&language, query_source)?;
        Ok(Self { language, query })
    }

    pub fn highlight(
        &self,
        src: &str,
        map_capture: impl Fn(&str) -> Option<(HighlightKind, u32, u32)>,
    ) -> Vec<HighlightSpan> {
        let tree = PARSER.with(|parser_cell| {
            let mut parser = parser_cell.borrow_mut();
            if parser.set_language(&self.language).is_err() {
                return None;
            }
            parser.parse(src, None)
        });

        let Some(tree) = tree else {
            return Vec::new();
        };

        let capture_names = self.query.capture_names();
        let mut tokens = Vec::new();

        CURSOR.with(|cursor_cell| {
            let mut cursor = cursor_cell.borrow_mut();
            let mut matches = cursor.matches(&self.query, tree.root_node(), src.as_bytes());
            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let name = &capture_names[capture.index as usize];
                    if let Some((kind, modifiers, priority)) = map_capture(name) {
                        let node = capture.node;
                        let start = node.start_byte();
                        let end = node.end_byte();
                        if start < end && end <= src.len() {
                            tokens.push(HighlightSpan::new(
                                Span::new(start, end),
                                kind,
                                modifiers,
                                priority,
                            ));
                        }
                    }
                }
            }
        });

        tokens
    }
}
