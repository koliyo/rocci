use std::{env, fs, path::PathBuf, process::Command};

fn rocdown_bin() -> PathBuf {
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rocdown");
    if !path.is_file() {
        path = PathBuf::from(env!("CARGO_BIN_EXE_rocdown"));
    }
    path
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn temp_dir(name: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("rocdown-e2e-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn check_docs_succeeds_in_terminal_and_json() {
    let root = repo_root();
    let bin = rocdown_bin();

    let output = Command::new(&bin)
        .arg("check")
        .arg(root.join("docs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(&bin)
        .arg("check")
        .arg(root.join("docs"))
        .arg("--format")
        .arg("json")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn inspect_config_reads_rocdown_toml() {
    let root = repo_root();
    let bin = rocdown_bin();

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("config")
        .arg(root.join("docs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let config: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(config["site"]["title"], "Rocci");
    assert_eq!(config["build"]["output"], "../dist/rocci.dev");
}

#[test]
fn inspect_catalog_and_graph() {
    let root = repo_root();
    let bin = rocdown_bin();

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("catalog")
        .arg(root.join("docs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let catalog: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(catalog.is_array());
    let has_index = catalog
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == "index");
    assert!(has_index);

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("graph")
        .arg(root.join("docs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn inspect_ast_and_roc_on_syntax_fixture() {
    let root = repo_root();
    let bin = rocdown_bin();
    let fixture = root.join("test/AllSyntax.rocdown");

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("ast")
        .arg(&fixture)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(rocdown"));

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("roc")
        .arg(&fixture)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("# components"));
    assert!(stdout.contains("# generated roc"));
}

#[test]
fn build_single_rocdown_file_writes_roc() {
    let root = repo_root();
    let bin = rocdown_bin();
    let temp = temp_dir("single");
    let input = temp.join("Doc.rocdown");
    let output_roc = temp.join("Doc.roc");

    fs::write(
        &input,
        "@page { route: \"/doc/\", meta: { title: \"Test Doc\" } }\n\n# Heading\n\nBody text.\n",
    )
    .unwrap();

    let output = Command::new(&bin)
        .arg("build")
        .arg(&input)
        .arg("-o")
        .arg(&output_roc)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "build single doc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_roc.is_file());
    let generated = fs::read_to_string(&output_roc).unwrap();
    assert!(generated.contains("Test Doc") || generated.contains("Heading"));
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn test_docs_examples() {
    let root = repo_root();
    let bin = rocdown_bin();

    let output = Command::new(&bin)
        .arg("test")
        .arg(root.join("docs"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "test examples failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn knowledge_check_adapter_passes() {
    let root = repo_root();
    let bin = rocdown_bin();

    let output = Command::new(&bin)
        .arg("knowledge")
        .arg("check")
        .arg(root.join("knowledge"))
        .arg("--profile")
        .arg("rocci")
        .arg("--format")
        .arg("json")
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "knowledge check failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
