use crate::diagnostic::{Diagnostic, DiagnosticFrame, Severity};
use crate::source_map::Segment;
use crate::span::{SourceFile, Span};

#[derive(Clone, Debug)]
pub struct MappedModule {
    pub type_name: String,
    pub generated: String,
    pub source_name: String,
    pub source_src: String,
    pub segments: Vec<Segment>,
}

pub fn remap_roc_output(output: &str, modules: &[MappedModule]) -> Vec<DiagnosticFrame> {
    let mut frames = Vec::new();
    for (file, line) in roc_locations(output) {
        let stem = file
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(&file)
            .strip_suffix(".roc")
            .unwrap_or(&file);
        let Some(module) = modules.iter().find(|module| {
            module.type_name == stem
                || module.source_name.ends_with(&file)
                || file.ends_with(&format!("{}.roc", module.type_name))
        }) else {
            continue;
        };
        let Some(start) = offset_of_line(&module.generated, line) else {
            continue;
        };
        let end =
            offset_of_line(&module.generated, line + 1).unwrap_or(module.generated.len() as u32);
        let source_span = module
            .segments
            .iter()
            .find(|segment| segment.generated.start < end && segment.generated.end > start)
            .map(|segment| segment.source)
            .unwrap_or(Span::point(0));
        let diagnostic = Diagnostic {
            span: source_span,
            severity: Severity::Error,
            message: format!("generated {}.roc:{line}", module.type_name),
        };
        frames.push(DiagnosticFrame::from_source(
            SourceFile::new(&module.source_name, &module.source_src),
            &diagnostic,
        ));
    }
    frames
}

fn roc_locations(output: &str) -> Vec<(String, u32)> {
    let mut found = Vec::new();
    let mut rest = output;
    while let Some(idx) = rest.find(".roc") {
        let prefix = &rest[..idx];
        let start = prefix
            .rfind(|ch: char| {
                !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/' | '\\' | '.'))
            })
            .map(|i| i + 1)
            .unwrap_or(0);
        let file = format!("{}{}", &prefix[start..], ".roc");
        let after = &rest[idx + 4..];
        let mut line = 1u32;
        let mut skip = idx + 4;
        if let Some(digits) = after.strip_prefix(':') {
            let count = digits.chars().take_while(|ch| ch.is_ascii_digit()).count();
            if count > 0
                && let Ok(parsed) = digits[..count].parse()
            {
                line = parsed;
                skip += 1 + count;
            }
        }
        if file != ".roc"
            && !found
                .iter()
                .any(|(existing, existing_line)| existing == &file && *existing_line == line)
        {
            found.push((file, line));
        }
        rest = &rest[skip.min(rest.len())..];
        if skip == 0 {
            break;
        }
    }
    found
}

fn offset_of_line(src: &str, line: u32) -> Option<u32> {
    if line == 0 {
        return None;
    }
    if line == 1 {
        return Some(0);
    }
    let mut current = 1u32;
    for (i, ch) in src.char_indices() {
        if ch == '\n' {
            current += 1;
            if current == line {
                return Some((i + 1) as u32);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_map::{OriginKind, Segment};

    #[test]
    fn remaps_generated_line_to_source_span() {
        let generated = "module =\n    Html.text(\"hi\")\n";
        let src = "# Hello\n";
        let hello = src.find("Hello").unwrap();
        let modules = [MappedModule {
            type_name: "Guide".into(),
            generated: generated.into(),
            source_name: "Guide.rocdown".into(),
            source_src: src.into(),
            segments: vec![Segment::new(
                Span::new(generated.find("Html").unwrap(), generated.len()),
                Span::new(hello, hello + 5),
                OriginKind::MarkdownText,
            )],
        }];
        let frames = remap_roc_output("── TYPE MISMATCH in Guide.roc:2 ──", &modules);
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].file, "Guide.rocdown");
        assert_eq!(frames[0].source_line, "# Hello");
    }
}
