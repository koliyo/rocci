use rocci_playground::{
    CompileRequest, Language, PROTOCOL_VERSION, PlaygroundHighlightSpan, compile,
};

fn verify_span_invariants(label: &str, doc_utf16_len: usize, spans: &[PlaygroundHighlightSpan]) {
    let mut prev_to = 0;
    for (i, span) in spans.iter().enumerate() {
        assert!(
            span.from <= span.to,
            "[{label}] Span {i} from > to: from={}, to={}",
            span.from,
            span.to
        );
        assert!(
            span.to <= doc_utf16_len,
            "[{label}] Span {i} to > doc_utf16_len: to={}, doc_len={}",
            span.to,
            doc_utf16_len
        );
        assert!(
            span.from >= prev_to,
            "[{label}] Overlapping/unordered span {i}: from={}, prev_to={}",
            span.from,
            prev_to
        );
        assert!(
            span.kind.starts_with("tok-"),
            "[{label}] Span {i} invalid kind CSS class: {}",
            span.kind
        );
        prev_to = span.to;
    }
}

#[test]
fn test_highlight_invariants_rocci_counter() {
    let source = "@component Counter = |{ count }| {\n  <button class=\"btn\">{count}</button>\n}";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 1,
        filename: "Counter.rocci".to_string(),
        language: Some(Language::Rocci),
        source: source.to_string(),
        workspace: None,
    };
    let resp = compile(&req);
    let src_len = rocci_playground::byte_to_utf16_offset(source, source.len());
    let roc_len = rocci_playground::byte_to_utf16_offset(&resp.roc, resp.roc.len());
    let ast_len = rocci_playground::byte_to_utf16_offset(&resp.ast, resp.ast.len());

    verify_span_invariants("source", src_len, &resp.highlights.source);
    verify_span_invariants("roc", roc_len, &resp.highlights.roc);
    verify_span_invariants("ast", ast_len, &resp.highlights.ast);
    assert!(!resp.highlights.source.is_empty());
    assert!(!resp.highlights.roc.is_empty());
    assert!(!resp.highlights.ast.is_empty());
}

#[test]
fn test_highlight_invariants_rocdown_guide() {
    let source = "# Guide\n\nWelcome to **Rocdown**.\n\n```rocci\n@component Card = |{}| { <div></div> }\n```\n";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 2,
        filename: "Guide.rocdown".to_string(),
        language: Some(Language::Rocdown),
        source: source.to_string(),
        workspace: None,
    };
    let resp = compile(&req);
    let src_len = rocci_playground::byte_to_utf16_offset(source, source.len());
    let roc_len = rocci_playground::byte_to_utf16_offset(&resp.roc, resp.roc.len());
    let ast_len = rocci_playground::byte_to_utf16_offset(&resp.ast, resp.ast.len());

    verify_span_invariants("source", src_len, &resp.highlights.source);
    verify_span_invariants("roc", roc_len, &resp.highlights.roc);
    verify_span_invariants("ast", ast_len, &resp.highlights.ast);
}

#[test]
fn test_highlight_invariants_unicode_emoji() {
    let source =
        "/* 🚀 Emoji comments */\n@component Emoji = |{ title }| {\n  <h1>🎉 {title} 🎈</h1>\n}";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 3,
        filename: "Emoji.rocci".to_string(),
        language: Some(Language::Rocci),
        source: source.to_string(),
        workspace: None,
    };
    let resp = compile(&req);
    let src_len = rocci_playground::byte_to_utf16_offset(source, source.len());
    let roc_len = rocci_playground::byte_to_utf16_offset(&resp.roc, resp.roc.len());
    let ast_len = rocci_playground::byte_to_utf16_offset(&resp.ast, resp.ast.len());

    verify_span_invariants("source", src_len, &resp.highlights.source);
    verify_span_invariants("roc", roc_len, &resp.highlights.roc);
    verify_span_invariants("ast", ast_len, &resp.highlights.ast);
}
