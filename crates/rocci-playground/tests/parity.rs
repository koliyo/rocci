use rocci_playground::{
    Capability, CompileRequest, DiagnosticSeverity, HTML_UNAVAILABLE_REASON, Language,
    PROTOCOL_VERSION, compile,
};

#[test]
fn test_rocci_valid_compilation() {
    let source = "@component Counter = |{ count }| { <button>{count}</button> }";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 42,
        filename: "Counter.rocci".to_string(),
        language: Some(Language::Rocci),
        source: source.to_string(),
        workspace: None,
    };

    let resp = compile(&req);
    assert_eq!(resp.protocol_version, PROTOCOL_VERSION);
    assert_eq!(resp.revision, 42);
    assert_eq!(resp.language, Language::Rocci);
    assert!(!resp.has_errors);
    assert!(resp.diagnostics.is_empty());
    assert!(resp.roc.contains("counter = |{ count }|"));
    assert!(resp.ast.contains("component Counter"));
    assert_eq!(resp.capabilities.roc, Capability { available: true });
    assert_eq!(resp.capabilities.ast, Capability { available: true });
    assert!(!resp.capabilities.html.available);
    assert_eq!(resp.capabilities.html.reason, HTML_UNAVAILABLE_REASON);
    assert!(resp.html.is_empty());
}

#[test]
fn test_rocdown_valid_compilation() {
    let source = "# Guide\n\nWelcome to **Rocdown** documentation.";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 101,
        filename: "Guide.rocdown".to_string(),
        language: None, // Dispatched from filename
        source: source.to_string(),
        workspace: None,
    };

    let resp = compile(&req);
    assert_eq!(resp.protocol_version, PROTOCOL_VERSION);
    assert_eq!(resp.revision, 101);
    assert_eq!(resp.language, Language::Rocdown);
    assert!(!resp.has_errors);
    assert!(resp.roc.contains("Guide"));
    assert!(resp.ast.contains("(h 1 guide"));
}

#[test]
fn test_rocci_diagnostic_utf16_mapping_with_unicode() {
    // String contains a 4-byte non-BMP emoji '🚀' before an unclosed bracket
    // "Party 🚀 @component { "
    let source = "Party 🚀 @component {";
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 1,
        filename: "Test.rocci".to_string(),
        language: Some(Language::Rocci),
        source: source.to_string(),
        workspace: None,
    };

    let resp = compile(&req);
    assert!(resp.has_errors);
    assert!(!resp.diagnostics.is_empty());

    let diag = &resp.diagnostics[0];
    assert_eq!(diag.severity, DiagnosticSeverity::Error);
    // Ensure UTF-16 from/to are within bounds
    let utf16_total_len = rocci_playground::byte_to_utf16_offset(source, source.len());
    assert!(diag.from <= diag.to);
    assert!(diag.to <= utf16_total_len);
}

#[test]
fn test_json_roundtrip() {
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 5,
        filename: "App.rocci".to_string(),
        language: Some(Language::Rocci),
        source: "<div>Hello</div>".to_string(),
        workspace: None,
    };

    let json_str = serde_json::to_string(&req).expect("serialize request");
    let deserialized: CompileRequest =
        serde_json::from_str(&json_str).expect("deserialize request");
    assert_eq!(req, deserialized);

    let resp = compile(&deserialized);
    let resp_json = serde_json::to_string(&resp).expect("serialize response");
    assert!(resp_json.contains("\"roc\""));
    assert!(resp_json.contains("\"ast\""));
    assert!(resp_json.contains("\"capabilities\""));
}
