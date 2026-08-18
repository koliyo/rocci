use rocci_playground::{CompileRequest, CompileResponse, Language, PROTOCOL_VERSION};
use rocci_playground_wasm::{compile_json, init_playground};

#[test]
fn test_init_metadata() {
    let init_str = init_playground();
    assert!(init_str.contains("\"protocol_version\":1"));
    assert!(init_str.contains("\"rocci\""));
    assert!(init_str.contains("\"rocdown\""));
}

#[test]
fn test_compile_json_valid_rocci() {
    let req = CompileRequest {
        protocol_version: PROTOCOL_VERSION,
        revision: 1,
        filename: "Test.rocci".to_string(),
        language: Some(Language::Rocci),
        source: "@component Button = |{ label }| { <button>{label}</button> }".to_string(),
        workspace: None,
    };
    let req_json = serde_json::to_string(&req).unwrap();
    let resp_json = compile_json(&req_json);
    let resp: CompileResponse = serde_json::from_str(&resp_json).expect("valid CompileResponse");
    assert_eq!(resp.language, Language::Rocci);
    assert!(!resp.has_errors);
    assert!(resp.roc.contains("button = |{ label }|"));
}

#[test]
fn test_compile_json_invalid_json() {
    let resp_json = compile_json("not valid json");
    assert!(resp_json.contains("\"error\""));
    assert!(resp_json.contains("\"has_errors\":true"));
}
