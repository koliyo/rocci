use rocci_template::Span;
use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Clone, Debug)]
pub struct HighlightToken {
    pub span: Span,
    pub kind: u32,
    pub modifiers: u32,
    pub priority: u32,
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
        map_capture: impl Fn(&str) -> Option<(u32, u32, u32)>,
    ) -> Vec<HighlightToken> {
        let mut parser = Parser::new();
        if parser.set_language(&self.language).is_err() {
            return Vec::new();
        }
        parser.set_timeout_micros(200_000);
        let Some(tree) = parser.parse(src, None) else {
            return Vec::new();
        };

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&self.query, tree.root_node(), src.as_bytes());
        let capture_names = self.query.capture_names();
        let mut tokens = Vec::new();

        for m in matches {
            for capture in m.captures {
                let name = &capture_names[capture.index as usize];
                if let Some((kind, modifiers, priority)) = map_capture(name) {
                    let node = capture.node;
                    let start = node.start_byte();
                    let end = node.end_byte();
                    if start < end && end <= src.len() {
                        tokens.push(HighlightToken {
                            span: Span::new(start, end),
                            kind,
                            modifiers,
                            priority,
                        });
                    }
                }
            }
        }

        tokens
    }
}
