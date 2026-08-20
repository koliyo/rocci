use rocci_playground::{CompileRequest, Language, compile};
use std::fs;
use std::path::Path;

#[test]
fn test_golden_all_syntax_rocci() {
    let path = Path::new("../../test/AllSyntax.rocci");
    if !path.exists() {
        return;
    }
    let source = fs::read_to_string(path).expect("read AllSyntax.rocci");
    let req = CompileRequest {
        protocol_version: 1,
        revision: 1,
        filename: "AllSyntax.rocci".to_string(),
        language: Some(Language::Rocci),
        source,
        workspace: None,
    };

    let resp = compile(&req);
    assert_eq!(resp.language, Language::Rocci);
    assert!(!resp.has_errors);
    assert!(!resp.roc.is_empty());
    assert!(!resp.ast.is_empty());
    assert!(!resp.highlights.roc.is_empty());
    assert!(!resp.highlights.ast.is_empty());
}

#[test]
fn test_golden_all_syntax_rocdown() {
    let path = Path::new("../../test/AllSyntax.rocdown");
    if !path.exists() {
        return;
    }
    let source = fs::read_to_string(path).expect("read AllSyntax.rocdown");
    let req = CompileRequest {
        protocol_version: 1,
        revision: 2,
        filename: "AllSyntax.rocdown".to_string(),
        language: Some(Language::Rocdown),
        source,
        workspace: None,
    };

    let resp = compile(&req);
    assert_eq!(resp.language, Language::Rocdown);
    assert!(!resp.has_errors);
    assert!(!resp.roc.is_empty());
    assert!(!resp.ast.is_empty());
}

#[test]
fn test_golden_counter_rocci() {
    let path = Path::new("../../examples/rocci/standalone/counter/Counter.rocci");
    if !path.exists() {
        return;
    }
    let source = fs::read_to_string(path).expect("read Counter.rocci");
    let req = CompileRequest {
        protocol_version: 1,
        revision: 3,
        filename: "Counter.rocci".to_string(),
        language: Some(Language::Rocci),
        source,
        workspace: None,
    };

    let resp = compile(&req);
    assert_eq!(resp.language, Language::Rocci);
    assert!(!resp.has_errors);
    assert!(resp.roc.contains("counterCard ="));
    assert!(resp.ast.contains("(component CounterCard"));
}
