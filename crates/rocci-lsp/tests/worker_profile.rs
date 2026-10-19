use lsp_types::Position;
use rocci_lsp::{ProfiledRocBackend, RocBackend, projection_workspace};
use std::{fs, path::PathBuf};

fn workspace() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "rocci Worker editor {} {}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn worker_projection_uses_build_sources_and_decodes_spaces() {
    let root = workspace();
    fs::write(
        root.join("rocci.toml"),
        "[worker]\nprovider='cloudflare'\nexecution='stateless'\n",
    )
    .unwrap();
    let uri = url::Url::from_file_path(root.join("Counter.rocci"))
        .unwrap()
        .as_str()
        .parse()
        .unwrap();
    assert_eq!(
        projection_workspace::source_dir(&uri, "Counter.rocci"),
        Some(root.clone())
    );
    let projected = root.join("projection");
    projection_workspace::stage_package(&projected, "Counter", Some(&root)).unwrap();
    assert!(
        fs::read_to_string(projected.join("main.roc"))
            .unwrap()
            .contains("platform/main.roc")
    );
    assert!(projected.join("worker-profile.json").is_file());
    for (name, source) in rocci_worker_platform::sources::ROC_SOURCES {
        assert_eq!(
            fs::read_to_string(projected.join("platform").join(name)).unwrap(),
            rocci_worker_platform::sources::compatibility_source(
                name.trim_end_matches(".roc"),
                source
            )
        );
    }
    let (rewritten, _) = projection_workspace::worker_module("import Html\nimport Datastar\n", &[]);
    assert_eq!(rewritten, "import pf.Html\nimport pf.Datastar\n");
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requires the qualified Worker compiler experimental LSP"]
fn real_worker_child_hovers_with_the_build_compiler() {
    let compiler =
        std::env::var("ROCCI_WORKER_ROC").expect("Set ROCCI_WORKER_ROC to the qualified compiler");
    let root = workspace();
    fs::create_dir(root.join(".rocci")).unwrap();
    fs::write(
        root.join("rocci.toml"),
        "[worker]\nprovider='cloudflare'\nexecution='stateless'\n",
    )
    .unwrap();
    fs::write(
        root.join(".rocci/worker-editor.json"),
        serde_json::json!({"compiler":compiler}).to_string(),
    )
    .unwrap();
    let projected = root.join("projection");
    projection_workspace::stage_package(&projected, "Counter", Some(&root)).unwrap();
    let path = projected.join("Counter.roc");
    let mut backend = ProfiledRocBackend::new(true);
    backend
        .sync_projection(
            &path,
            "Counter := [].{\n    answer : U64\n    answer = 42\n}\n",
        )
        .unwrap();
    assert!(
        backend.hover(&path, Position::new(2, 5)).is_some(),
        "Worker compiler returned no hover"
    );
    drop(backend);
    fs::remove_dir_all(root).unwrap();
}
