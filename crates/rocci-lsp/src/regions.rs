use lsp_types::{Position, Range};
use rocci_template::{PositionEncoding, SourceFile};
use serde::{Deserialize, Serialize};

pub use rocci_highlight::LanguageId as Language;
pub use rocci_highlight::regions::{
    Region, RegionContext, RegionPurpose, RegionSpan, RegionTree, RegionValidationError,
    extract_rocci_regions,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedRegion {
    pub id: usize,
    pub language: String,
    pub context: String,
    pub purpose: String,
    pub span: RegionSpan,
    pub range: Range,
    pub parent: Option<usize>,
    pub priority: u32,
}

pub fn inspect_regions(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<InspectedRegion> {
    tree.regions
        .iter()
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            InspectedRegion {
                id: r.id,
                language: r.language.canonical_name().to_string(),
                context: r.context.as_str().to_string(),
                purpose: r.purpose.as_str().to_string(),
                span: r.span.into(),
                range: Range {
                    start: Position::new(start_line, start_col),
                    end: Position::new(end_line, end_col),
                },
                parent: r.parent,
                priority: r.priority,
            }
        })
        .collect()
}

pub fn executable_roc_ranges(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<Range> {
    tree.regions
        .iter()
        .filter(|r| r.language == Language::Roc && r.purpose == RegionPurpose::Executable)
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            Range {
                start: Position::new(start_line, start_col),
                end: Position::new(end_line, end_col),
            }
        })
        .collect()
}

pub fn css_ranges(
    source: SourceFile<'_>,
    tree: &RegionTree,
    encoding: PositionEncoding,
) -> Vec<Range> {
    tree.regions
        .iter()
        .filter(|r| r.language == Language::Css)
        .map(|r| {
            let ((start_line, start_col), (end_line, end_col)) = source.range(r.span, encoding);
            Range {
                start: Position::new(start_line, start_col),
                end: Position::new(end_line, end_col),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_regions() {
        let src = "@component Card = |{ title }| { <div class=\"card\">{title}</div> }";
        let doc = rocci_template::parse(SourceFile::new("test.rocci", src)).document;
        let tree = extract_rocci_regions("test.rocci", src, &doc);
        assert!(tree.validate(src.len()).is_ok());
    }
}
