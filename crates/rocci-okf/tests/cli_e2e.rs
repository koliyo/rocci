use std::{env, fs, path::PathBuf, process::Command};

fn rocci_okf_bin() -> PathBuf {
    let mut path = env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("rocci-okf");
    if !path.is_file() {
        path = PathBuf::from(env!("CARGO_BIN_EXE_rocci-okf"));
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
    let path = env::temp_dir().join(format!("rocci-okf-e2e-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn knowledge_check_succeeds_in_terminal_and_json() {
    let root = repo_root();
    let bin = rocci_okf_bin();

    let output = Command::new(&bin)
        .arg("check")
        .arg(root.join("knowledge"))
        .arg("--profile")
        .arg("rocci")
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
        .arg(root.join("knowledge"))
        .arg("--profile")
        .arg("rocci")
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
fn inspect_catalog_and_graph() {
    let root = repo_root();
    let bin = rocci_okf_bin();

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("catalog")
        .arg(root.join("knowledge"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let catalog: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(catalog.is_array());

    let output = Command::new(&bin)
        .arg("inspect")
        .arg("graph")
        .arg(root.join("knowledge"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn search_knowledge_returns_json_chunks() {
    let root = repo_root();
    let bin = rocci_okf_bin();

    let output = Command::new(&bin)
        .arg("search")
        .arg("system overview")
        .arg(root.join("knowledge"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let results: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(results.is_array());
}

#[test]
fn build_knowledge_emits_artifacts() {
    let root = repo_root();
    let bin = rocci_okf_bin();
    let temp = temp_dir("build");

    let output = Command::new(&bin)
        .arg("build")
        .arg(root.join("knowledge"))
        .arg("-o")
        .arg(&temp)
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temp.join("catalog.json").is_file());
    assert!(temp.join("search.json").is_file());
    assert!(temp.join("llms.txt").is_file());
    assert!(temp.join("validation.json").is_file());

    let _ = fs::remove_dir_all(temp);
}
