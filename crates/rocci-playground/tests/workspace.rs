use rocci_playground::{
    CompileRequest, DiagnosticSeverity, Language, VirtualFile, VirtualWorkspace, compile,
};

#[test]
fn test_virtual_workspace_normal_usage() {
    let req = CompileRequest {
        protocol_version: 1,
        revision: 1,
        filename: "Guide.rocdown".to_string(),
        language: Some(Language::Rocdown),
        source: "# Guide\nSee [Other](other.rocdown)".to_string(),
        workspace: Some(VirtualWorkspace {
            files: vec![VirtualFile {
                path: "other.rocdown".to_string(),
                content: "# Other".to_string(),
            }],
        }),
    };

    let resp = compile(&req);
    assert_eq!(resp.language, Language::Rocdown);
    assert!(!resp.has_errors);
    assert_eq!(resp.diagnostics.len(), 0);
}

#[test]
fn test_virtual_workspace_traversal_warning() {
    let req = CompileRequest {
        protocol_version: 1,
        revision: 2,
        filename: "Guide.rocdown".to_string(),
        language: Some(Language::Rocdown),
        source: "# Guide".to_string(),
        workspace: Some(VirtualWorkspace {
            files: vec![VirtualFile {
                path: "../secret.txt".to_string(),
                content: "secret".to_string(),
            }],
        }),
    };

    let resp = compile(&req);
    assert!(!resp.has_errors);
    assert!(
        resp.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning
                && d.message.contains("traversal characters"))
    );
}

#[test]
fn test_virtual_workspace_file_limit_warning() {
    let mut files = Vec::new();
    for i in 0..55 {
        files.push(VirtualFile {
            path: format!("file{i}.rocdown"),
            content: "# File".to_string(),
        });
    }

    let req = CompileRequest {
        protocol_version: 1,
        revision: 3,
        filename: "Main.rocdown".to_string(),
        language: Some(Language::Rocdown),
        source: "# Main".to_string(),
        workspace: Some(VirtualWorkspace { files }),
    };

    let resp = compile(&req);
    assert!(
        resp.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning && d.message.contains("file limit"))
    );
}
