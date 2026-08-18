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

    let highlights = compute_highlights(LanguageId::Rocdown, &request.source, &output.roc, &ast);

    CompileResponse {
        protocol_version: PROTOCOL_VERSION,
        revision: request.revision,
        language: Language::Rocdown,
        roc: output.roc,
        ast,
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

fn convert_spans(src: &str, spans: Vec<HighlightSpan>) -> Vec<PlaygroundHighlightSpan> {
    spans
        .into_iter()
        .map(|span| {
            let start_byte = span.start();
            let end_byte = span.end();
            let (from, to) = byte_range_to_utf16(src, start_byte, end_byte);
            PlaygroundHighlightSpan {
                from,
                to,
                kind: span.kind.css_class().to_string(),
                modifiers: span.modifiers,
            }
        })
        .collect()
}

fn compute_highlights(
    source_lang: LanguageId,
    source: &str,
    roc: &str,
    ast: &str,
) -> PlaygroundHighlights {
    let source_spans = highlight(source_lang, source);
    let roc_spans = highlight(LanguageId::Roc, roc);
    // Simple S-expression highlighting for AST if desired, or empty
    let ast_spans = Vec::new();

    PlaygroundHighlights {
        source: convert_spans(source, source_spans),
        roc: convert_spans(roc, roc_spans),
        ast: convert_spans(ast, ast_spans),
    }
}
