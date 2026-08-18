use crate::protocol::{
    Capability, CompileRequest, CompileResponse, DiagnosticSeverity, HtmlCapability, Language,
    PROTOCOL_VERSION, PlaygroundCapabilities, PlaygroundDiagnostic, PlaygroundHighlightSpan,
    PlaygroundHighlights,
};
use crate::utf16::byte_range_to_utf16;
use rocci_highlight::{HighlightSpan, LanguageId, highlight};

/// Compiles a playground request for either `.rocci` or `.rocdown` input, producing
/// generated Roc, formatted AST, diagnostics with UTF-16 ranges, highlight spans,
/// and capabilities.
pub fn compile(request: &CompileRequest) -> CompileResponse {
    let language = request.resolved_language();
    match language {
        Language::Rocci => compile_rocci(request),
        Language::Rocdown => compile_rocdown(request),
    }
}

fn compile_rocci(request: &CompileRequest) -> CompileResponse {
    let source_file = rocci_template::SourceFile::new(&request.filename, &request.source);
    let output = rocci_template::compile(source_file, &rocci_template::LowerOptions::default());
    let ast = rocci_template::format_ast(&request.source, &output.document);
    let has_errors = output.has_errors();

    let diagnostics = output
        .diagnostics
        .into_iter()
        .map(|d| {
            let start_byte = d.span.start as usize;
            let end_byte = d.span.end as usize;
            let (from, to) = byte_range_to_utf16(&request.source, start_byte, end_byte);
            let severity = match d.severity {
                rocci_template::Severity::Error => DiagnosticSeverity::Error,
                rocci_template::Severity::Warning => DiagnosticSeverity::Warning,
            };
            PlaygroundDiagnostic {
                severity,
                message: d.message,
                start_byte,
                end_byte,
                from,
                to,
            }
        })
        .collect();

    let highlights = compute_highlights(LanguageId::Rocci, &request.source, &output.roc, &ast);

    CompileResponse {
        protocol_version: PROTOCOL_VERSION,
        revision: request.revision,
        language: Language::Rocci,
        roc: output.roc,
        ast,
        html: String::new(),
        diagnostics,
        highlights,
        capabilities: PlaygroundCapabilities {
            roc: Capability { available: true },
            ast: Capability { available: true },
            html: HtmlCapability::default(),
        },
        has_errors,
    }
}

fn compile_rocdown(request: &CompileRequest) -> CompileResponse {
    let source_file = rocci_rocdown::SourceFile::new(&request.filename, &request.source);
    // Browser-safe compile options: no filesystem or network access
    let options = rocci_rocdown::CompileOptions {
        resolve_links: false,
        resolve_includes: false,
        check_assets: false,
        ..rocci_rocdown::CompileOptions::default()
    };
    let output = rocci_rocdown::compile(source_file, &options);
    let ast = rocci_rocdown::format_ast(&request.source, &output.document);
    let has_errors = output.has_errors();

    let mut diagnostics: Vec<PlaygroundDiagnostic> = output
        .diagnostics
        .into_iter()
        .map(|d| {
            let start_byte = d.span.start as usize;
            let end_byte = d.span.end as usize;
            let (from, to) = byte_range_to_utf16(&request.source, start_byte, end_byte);
            let severity = match d.severity {
                rocci_template::Severity::Error => DiagnosticSeverity::Error,
                rocci_template::Severity::Warning => DiagnosticSeverity::Warning,
            };
            PlaygroundDiagnostic {
                severity,
                message: d.message,
                start_byte,
                end_byte,
                from,
                to,
            }
        })
        .collect();

    if let Some(workspace) = &request.workspace {
        validate_and_check_workspace(
            workspace,
            &output.document,
            &request.source,
            &mut diagnostics,
        );
    }

    let highlights = compute_highlights(LanguageId::Rocdown, &request.source, &output.roc, &ast);

    CompileResponse {
        protocol_version: PROTOCOL_VERSION,
        revision: request.revision,
        language: Language::Rocdown,
        roc: output.roc,
        ast,
        html: String::new(),
        diagnostics,
        highlights,
        capabilities: PlaygroundCapabilities {
            roc: Capability { available: true },
            ast: Capability { available: true },
            html: HtmlCapability::default(),
        },
        has_errors,
    }
}

pub fn highlight_sexp(ast: &str) -> Vec<HighlightSpan> {
    let mut spans = Vec::new();
    let bytes = ast.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'(' || b == b')' || b == b'[' || b == b']' {
            spans.push(HighlightSpan::new(
                rocci_template::Span::new(i, i + 1),
                rocci_highlight::HighlightKind::Punctuation,
                0,
                10,
            ));
            i += 1;
        } else if b == b'"' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else if bytes[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            spans.push(HighlightSpan::new(
                rocci_template::Span::new(start, i),
                rocci_highlight::HighlightKind::String,
                0,
                20,
            ));
        } else if b.is_ascii_digit()
            || (b == b'-' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit())
        {
            let start = i;
            i += 1;
            while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                i += 1;
            }
            spans.push(HighlightSpan::new(
                rocci_template::Span::new(start, i),
                rocci_highlight::HighlightKind::Number,
                0,
                20,
            ));
        } else if b == b':' {
            let start = i;
            i += 1;
            while i < bytes.len()
                && !bytes[i].is_ascii_whitespace()
                && bytes[i] != b')'
                && bytes[i] != b']'
            {
                i += 1;
            }
            spans.push(HighlightSpan::new(
                rocci_template::Span::new(start, i),
                rocci_highlight::HighlightKind::Property,
                0,
                20,
            ));
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
            {
                i += 1;
            }
            let word = &ast[start..i];
            let kind = if word == "true" || word == "false" || word == "null" {
                rocci_highlight::HighlightKind::Keyword
            } else if start > 0 && bytes[start - 1] == b'(' {
                rocci_highlight::HighlightKind::Keyword
            } else {
                rocci_highlight::HighlightKind::Variable
            };
            spans.push(HighlightSpan::new(
                rocci_template::Span::new(start, i),
                kind,
                0,
                15,
            ));
        } else {
            i += 1;
        }
    }
    spans
}

fn convert_spans(src: &str, spans: Vec<HighlightSpan>) -> Vec<PlaygroundHighlightSpan> {
    let utf16_total = crate::utf16::byte_to_utf16_offset(src, src.len());
    let mut converted: Vec<PlaygroundHighlightSpan> = spans
        .into_iter()
        .filter_map(|span| {
            let start_byte = span.start();
            let end_byte = span.end();
            if start_byte >= end_byte || start_byte >= src.len() {
                return None;
            }
            let (from, to) = byte_range_to_utf16(src, start_byte, end_byte);
            if from >= to || to > utf16_total {
                return None;
            }
            Some(PlaygroundHighlightSpan {
                from,
                to,
                kind: span.kind.css_class().to_string(),
                modifiers: span.modifiers,
            })
        })
        .collect();

    converted.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| b.to.cmp(&a.to)));

    let mut result = Vec::new();
    let mut prev_to = 0;
    for span in converted {
        if span.from >= prev_to {
            prev_to = span.to;
            result.push(span);
        }
    }
    result
}

fn compute_highlights(
    source_lang: LanguageId,
    source: &str,
    roc: &str,
    ast: &str,
) -> PlaygroundHighlights {
    let source_spans = highlight(source_lang, source);
    let roc_spans = highlight(LanguageId::Roc, roc);
    let ast_spans = highlight_sexp(ast);

    PlaygroundHighlights {
        source: convert_spans(source, source_spans),
        roc: convert_spans(roc, roc_spans),
        ast: convert_spans(ast, ast_spans),
    }
}

const MAX_WORKSPACE_FILES: usize = 50;
const MAX_WORKSPACE_BYTES: usize = 1_048_576; // 1 MB

fn validate_and_check_workspace(
    workspace: &crate::protocol::VirtualWorkspace,
    _doc: &rocci_rocdown::Document,
    _source: &str,
    diagnostics: &mut Vec<PlaygroundDiagnostic>,
) {
    if workspace.files.len() > MAX_WORKSPACE_FILES {
        diagnostics.push(PlaygroundDiagnostic {
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "virtual workspace exceeds file limit ({} > {MAX_WORKSPACE_FILES})",
                workspace.files.len()
            ),
            start_byte: 0,
            end_byte: 0,
            from: 0,
            to: 0,
        });
    }

    let mut total_bytes = 0;
    for f in &workspace.files {
        total_bytes += f.path.len() + f.content.len();
        if f.path.starts_with('/') || f.path.contains("..") || f.path.contains('\0') {
            diagnostics.push(PlaygroundDiagnostic {
                severity: DiagnosticSeverity::Warning,
                message: format!(
                    "virtual workspace path `{}` contains invalid or traversal characters",
                    f.path
                ),
                start_byte: 0,
                end_byte: 0,
                from: 0,
                to: 0,
            });
        }
    }

    if total_bytes > MAX_WORKSPACE_BYTES {
        diagnostics.push(PlaygroundDiagnostic {
            severity: DiagnosticSeverity::Warning,
            message: format!(
                "virtual workspace exceeds size limit ({total_bytes} > {MAX_WORKSPACE_BYTES} bytes)"
            ),
            start_byte: 0,
            end_byte: 0,
            from: 0,
            to: 0,
        });
    }
}
