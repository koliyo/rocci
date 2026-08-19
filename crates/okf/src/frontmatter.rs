use serde_json::{Map, Value};
use std::collections::BTreeMap;
use yaml_rust::{Yaml, YamlLoader};

use crate::ast::Span;
use crate::diagnostic::SourceLocation;

#[derive(Clone, Copy)]
pub struct Frontmatter {
    pub yaml: Span,
    pub body: Span,
}

pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' && index + 1 < source.len() {
                starts.push(index + 1);
            }
        }
        Self { starts }
    }

    pub fn location(&self, source: &str, span: Span) -> SourceLocation {
        let start_pos = (span.start as usize).min(source.len());
        if start_pos >= source.len() {
            return SourceLocation {
                start: span.start,
                end: span.end,
                line: 1,
                column: 1,
            };
        }
        let (line, line_start) = match self.starts.binary_search(&start_pos) {
            Ok(index) => (index + 1, self.starts[index]),
            Err(0) => (1, 0),
            Err(index) => (index, self.starts[index - 1]),
        };
        SourceLocation {
            start: span.start,
            end: span.end,
            line: line as u32,
            column: (start_pos - line_start + 1) as u32,
        }
    }
}

pub fn lines_with_offsets(source: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        out.push((offset, line));
        offset += line.len();
    }
    if offset < source.len() {
        out.push((offset, &source[offset..]));
    }
    out
}

pub fn location(source: &str, span: Span) -> SourceLocation {
    LineIndex::new(source).location(source, span)
}

pub fn split_frontmatter(
    source: &str,
    required: bool,
) -> std::result::Result<Option<Frontmatter>, String> {
    let mut lines = lines_with_offsets(source).into_iter();
    let Some((_, first)) = lines.next() else {
        return if required {
            Err("concept requires YAML frontmatter".into())
        } else {
            Ok(None)
        };
    };
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return if required {
            Err("concept must start with `---` YAML frontmatter".into())
        } else {
            Ok(None)
        };
    }
    let yaml_start = first.len();
    for (offset, line) in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Ok(Some(Frontmatter {
                yaml: Span::new(yaml_start, offset),
                body: Span::new(offset + line.len(), source.len()),
            }));
        }
    }
    Err("frontmatter is missing its closing `---` delimiter".into())
}

pub fn parse_yaml_mapping(source: &str) -> std::result::Result<BTreeMap<String, Value>, String> {
    let documents = YamlLoader::load_from_str(source).map_err(|error| error.to_string())?;
    if documents.len() != 1 {
        return Err("frontmatter must contain exactly one YAML document".into());
    }
    let Some(mapping) = documents[0].as_hash() else {
        return Err("frontmatter must be a YAML mapping".into());
    };
    let mut out = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("frontmatter keys must be strings".into());
        };
        out.insert(key.to_string(), yaml_to_json(value)?);
    }
    Ok(out)
}

pub fn yaml_to_json(value: &Yaml) -> std::result::Result<Value, String> {
    match value {
        Yaml::Real(value) => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| format!("invalid YAML number `{value}`")),
        Yaml::Integer(value) => Ok(Value::Number((*value).into())),
        Yaml::String(value) => Ok(Value::String(value.clone())),
        Yaml::Boolean(value) => Ok(Value::Bool(*value)),
        Yaml::Array(values) => values
            .iter()
            .map(yaml_to_json)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(Value::Array),
        Yaml::Hash(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let Some(key) = key.as_str() else {
                    return Err("nested YAML mapping keys must be strings".into());
                };
                object.insert(key.to_string(), yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        Yaml::Null | Yaml::BadValue => Ok(Value::Null),
        Yaml::Alias(_) => Err("unresolved YAML aliases are not supported".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::{LineIndex, location};
    use crate::ast::Span;

    fn legacy_location(source: &str, span: Span) -> crate::diagnostic::SourceLocation {
        let start_pos = (span.start as usize).min(source.len());
        let mut line = 1;
        let mut column = 1;
        let mut current_offset = 0;
        for (line_idx, (_, line_str)) in super::lines_with_offsets(source).into_iter().enumerate() {
            let line_end = current_offset + line_str.len();
            if start_pos >= current_offset && start_pos < line_end {
                line = (line_idx + 1) as u32;
                column = (start_pos - current_offset + 1) as u32;
                break;
            }
            current_offset = line_end;
        }
        crate::diagnostic::SourceLocation {
            start: span.start,
            end: span.end,
            line,
            column,
        }
    }

    #[test]
    fn location_matches_legacy_offsets() {
        let source = "---\ntitle: Demo\n---\n\n# Heading\n\nSee [link](/a.md).\n";
        let index = LineIndex::new(source);
        for start in 0..=source.len() {
            let span = Span::new(start, start.saturating_add(1).min(source.len()));
            let expected = legacy_location(source, span);
            let via_index = index.location(source, span);
            let via_fn = location(source, span);
            assert_eq!(via_index, expected, "index mismatch at {start}");
            assert_eq!(via_fn, expected, "location() mismatch at {start}");
        }
    }

    #[test]
    fn location_maps_known_heading_offset() {
        let source = "# Title\n\nBody\n";
        let at = location(source, Span::new(0, 7));
        assert_eq!(at.line, 1);
        assert_eq!(at.column, 1);
        let body = location(source, Span::new(9, 13));
        assert_eq!(body.line, 3);
        assert_eq!(body.column, 1);
    }
}
