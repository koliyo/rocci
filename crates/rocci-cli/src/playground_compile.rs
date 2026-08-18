use std::path::Path;

use rocci_highlight::{HighlightSpan, LanguageId, highlight};
use rocci_template::{LowerOptions, SourceFile, compile, format_ast};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::playground_html::{HTML_NO_TARGET_REASON, render_html_snapshot};

const PROTOCOL_VERSION: u32 = 1;

#[derive(Deserialize)]
struct IncomingRequest {
    #[serde(default)]
    protocol_version: u32,
    #[serde(default)]
    revision: u64,
    filename: String,
    #[serde(default)]
    language: Option<String>,
    source: String,
}

pub fn compile_rocci(body: &[u8], src_dir: Option<&Path>) -> Vec<u8> {
    let incoming: IncomingRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => {
            return error_json(0, format!("invalid JSON request: {err}"));
        }
    };
    match compile_rocci_request(&incoming, src_dir) {
        Ok(value) => serde_json::to_vec(&value).unwrap_or_else(|err| {
            error_json(incoming.revision, format!("serialization error: {err}"))
        }),
        Err(err) => error_json(incoming.revision, err),
    }
}

fn compile_rocci_request(
    request: &IncomingRequest,
    src_dir: Option<&Path>,
) -> Result<Value, String> {
    let language = request
        .language
        .as_deref()
        .or_else(|| {
            let lower = request.filename.to_ascii_lowercase();
            if lower.ends_with(".rocci") {
                Some("rocci")
            } else {
                None
            }
        })
        .unwrap_or("rocci");
    if language != "rocci" {
        return Err("local rocci compile hook only accepts .rocci sources".to_string());
    }

    let source_file = SourceFile::new(&request.filename, &request.source);
    let output = compile(source_file, &LowerOptions::default());
    let ast = format_ast(&request.source, &output.document);
    let has_errors = output.has_errors();

    let diagnostics: Vec<Value> = output
        .diagnostics
        .iter()
        .map(|d| {
            let start_byte = d.span.start as usize;
            let end_byte = d.span.end as usize;
            let (from, to) = byte_range_to_utf16(&request.source, start_byte, end_byte);
            let severity = match d.severity {
                rocci_template::Severity::Error => "error",
                rocci_template::Severity::Warning => "warning",
            };
            json!({
                "severity": severity,
                "message": d.message,
                "start_byte": start_byte,
                "end_byte": end_byte,
                "from": from,
                "to": to,
            })
        })
        .collect();

    let highlights = json!({
        "source": convert_spans(&request.source, highlight(LanguageId::Rocci, &request.source)),
        "roc": convert_spans(&output.roc, highlight(LanguageId::Roc, &output.roc)),
        "ast": convert_spans(&ast, Vec::new()),
    });

    let mut html = String::new();
    let mut html_available = false;
    let mut html_reason = if has_errors {
        "Fix template errors before HTML can be rendered.".to_string()
    } else {
        HTML_NO_TARGET_REASON.to_string()
    };
    if !has_errors {
        match render_html_snapshot(
            &request.filename,
            &request.source,
            &output.roc,
            &output.segments,
            &output.document,
            &output.components,
            &output.fixtures,
            src_dir,
        ) {
            Ok(rendered) => {
                html = rendered;
                html_available = true;
                html_reason.clear();
            }
            Err(reason) => html_reason = reason,
        }
    }

    Ok(json!({
        "protocol_version": if request.protocol_version == 0 {
            PROTOCOL_VERSION
        } else {
            request.protocol_version
        },
        "revision": request.revision,
        "language": "rocci",
        "roc": output.roc,
        "ast": ast,
        "html": html,
        "diagnostics": diagnostics,
        "highlights": highlights,
        "capabilities": {
            "roc": { "available": true },
            "ast": { "available": true },
            "html": { "available": html_available, "reason": html_reason },
        },
        "has_errors": has_errors,
    }))
}

fn error_json(revision: u64, error: String) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol_version": PROTOCOL_VERSION,
        "revision": revision,
        "error": error,
        "has_errors": true,
    }))
    .unwrap_or_default()
}

fn byte_to_utf16_offset(src: &str, byte_offset: usize) -> usize {
    let clamped_byte = byte_offset.min(src.len());
    let valid_byte = if src.is_char_boundary(clamped_byte) {
        clamped_byte
    } else {
        let mut b = clamped_byte;
        while b > 0 && !src.is_char_boundary(b) {
            b -= 1;
        }
        b
    };
    src[..valid_byte].chars().map(|ch| ch.len_utf16()).sum()
}

fn byte_range_to_utf16(src: &str, start_byte: usize, end_byte: usize) -> (usize, usize) {
    let from = byte_to_utf16_offset(src, start_byte);
    let to = byte_to_utf16_offset(src, end_byte.max(start_byte));
    (from, to)
}

fn convert_spans(src: &str, spans: Vec<HighlightSpan>) -> Vec<Value> {
    let utf16_total = byte_to_utf16_offset(src, src.len());
    let mut converted: Vec<(usize, usize, String, u32)> = spans
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
            Some((from, to, span.kind.css_class().to_string(), span.modifiers))
        })
        .collect();
    converted.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
    let mut result = Vec::new();
    let mut prev_to = 0;
    for (from, to, kind, modifiers) in converted {
        if from >= prev_to {
            prev_to = to;
            result.push(json!({
                "from": from,
                "to": to,
                "kind": kind,
                "modifiers": modifiers,
            }));
        }
    }
    result
}
