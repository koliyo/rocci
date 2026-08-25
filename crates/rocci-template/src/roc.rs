use std::path::Path;

use crate::source_map::{Segment, remap_segments};

pub fn type_name_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("View");
    let mut out = String::new();
    let mut cap_next = true;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            if cap_next {
                out.extend(ch.to_uppercase());
                cap_next = false;
            } else {
                out.push(ch);
            }
        } else {
            cap_next = true;
        }
    }
    if out.is_empty() {
        "View".to_string()
    } else {
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeModuleProjection {
    pub roc: String,
    pub segments: Vec<Segment>,
}

pub fn wrap_type_module(src: &str, type_name: &str) -> String {
    wrap_type_module_mapped(src, type_name).0
}

pub fn project_type_module(
    generated: &str,
    segments: &[Segment],
    type_name: &str,
) -> TypeModuleProjection {
    let (roc, map) = wrap_type_module_mapped(generated, type_name);
    TypeModuleProjection {
        roc,
        segments: remap_segments(&map, segments),
    }
}

struct SrcLine<'a> {
    start: usize,
    text: &'a str,
}

fn src_lines(src: &str) -> Vec<SrcLine<'_>> {
    let mut lines = Vec::new();
    let mut offset = 0usize;
    let bytes = src.as_bytes();
    for text in src.lines() {
        lines.push(SrcLine {
            start: offset,
            text,
        });
        offset += text.len();
        if offset < src.len() {
            if bytes[offset] == b'\r' {
                offset += 1;
            }
            if offset < src.len() && bytes[offset] == b'\n' {
                offset += 1;
            }
        }
    }
    lines
}

fn wrap_type_module_mapped(src: &str, type_name: &str) -> (String, Vec<Option<u32>>) {
    let lines = src_lines(src);
    let mut imports = Vec::new();
    let mut body = Vec::new();
    for line in &lines {
        let trimmed = line.text.trim();
        if trimmed.starts_with("module ") && trimmed.contains(" exposing ") {
            continue;
        }
        if line.text.starts_with("import ") {
            imports.push(line);
        } else {
            body.push(line);
        }
    }
    while body.first().is_some_and(|line| line.text.trim().is_empty()) {
        body.remove(0);
    }
    while body.last().is_some_and(|line| line.text.trim().is_empty()) {
        body.pop();
    }

    let mut out = String::new();
    let mut map = vec![None; src.len()];
    if !imports.is_empty() {
        for (idx, line) in imports.iter().enumerate() {
            if idx > 0 {
                out.push('\n');
            }
            copy_line(&mut map, line, out.len() as u32);
            out.push_str(line.text);
        }
        out.push_str("\n\n");
    }
    out.push_str(type_name);
    out.push_str(" := [].{\n");
    for (idx, line) in body.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        let dest = if line.text.is_empty() {
            out.len() as u32
        } else {
            (out.len() + 4) as u32
        };
        copy_line(&mut map, line, dest);
        if !line.text.is_empty() {
            out.push_str("    ");
        }
        out.push_str(line.text);
    }
    out.push_str("\n}\n");
    (out, map)
}

fn copy_line(map: &mut [Option<u32>], line: &SrcLine<'_>, dest: u32) {
    for i in 0..line.text.len() {
        map[line.start + i] = Some(dest + i as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_map::OriginKind;
    use crate::span::Span;

    #[test]
    fn type_name_pascal_cases_file_stem() {
        assert_eq!(type_name_from_path(Path::new("foo.rocci")), "Foo");
        assert_eq!(type_name_from_path(Path::new("Counter.rocci")), "Counter");
        assert_eq!(type_name_from_path(Path::new("foo-bar.rocci")), "FooBar");
    }

    #[test]
    fn wrap_type_module_strips_header_and_keeps_imports() {
        let src = "\
module CounterPage exposing [hello]

import Html

hello = |{ name }| {
    Html.text(name)
}
";
        let wrapped = wrap_type_module(src, "Foo");
        assert!(!wrapped.contains("module CounterPage"));
        assert!(wrapped.starts_with("import Html\n\nFoo := [].{\n"));
        assert!(wrapped.contains("    hello = |{ name }| {"));
        assert!(wrapped.ends_with("}\n"));
    }

    #[test]
    fn project_type_module_matches_wrap_and_shifts_body_segments() {
        let src = "\
import Html

hello = |{ name }| {
    Html.text(name)
}
";
        let hello = src.find("hello").expect("hello");
        let ident = src.find("Html.text(name)").expect("call") + "Html.text(".len();
        let segments = vec![
            Segment::new(
                Span::new(hello, hello + 5),
                Span::new(0, 5),
                OriginKind::OrdinaryRoc,
            ),
            Segment::new(
                Span::new(ident, ident + 4),
                Span::new(0, 4),
                OriginKind::OrdinaryRoc,
            ),
        ];
        let projection = project_type_module(src, &segments, "Foo");
        assert_eq!(projection.roc, wrap_type_module(src, "Foo"));
        assert_eq!(
            projection.segments[0].generated.of(&projection.roc),
            "hello"
        );
        assert_eq!(projection.segments[1].generated.of(&projection.roc), "name");
        assert!(projection.segments[0].generated.start as usize > hello);
    }
}
