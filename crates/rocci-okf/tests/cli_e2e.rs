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
fn no_subcommand_prints_help() {
    let output = Command::new(rocci_okf_bin()).output().unwrap();
    assert!(
        !output.status.success(),
        "bare rocci-okf should not start the viewer"
    );
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("Usage:") && text.contains("view"),
        "expected help mentioning view, got: {text}"
    );
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

#[test]
fn roots_falls_back_to_cwd_knowledge() {
    let root = repo_root();
    let bin = rocci_okf_bin();
    let temp = temp_dir("roots-fallback");
    fs::write(temp.join("okf.toml"), "poll = \"5m\"\n").unwrap();
    fs::create_dir_all(temp.join("knowledge")).unwrap();

    let output = Command::new(&bin)
        .arg("roots")
        .env("ROCCI_OKF_CONFIG", temp.join("okf.toml"))
        .env("ROCCI_CACHE", temp.join("cache"))
        .current_dir(&temp)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let printed = PathBuf::from(stdout.trim());
    assert_eq!(printed, temp.join("knowledge").canonicalize().unwrap());

    let output = Command::new(&bin)
        .arg("roots")
        .arg("--format")
        .arg("json")
        .env("ROCCI_OKF_CONFIG", temp.join("okf.toml"))
        .env("ROCCI_CACHE", temp.join("cache"))
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert!(parsed.as_array().unwrap().is_empty() || parsed[0]["id"] == "knowledge");
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn roots_json_redacts_token_and_lists_directory() {
    let bin = rocci_okf_bin();
    let temp = temp_dir("roots-json");
    let bundle = temp.join("bundle");
    fs::create_dir_all(&bundle).unwrap();
    fs::write(
        temp.join("okf.toml"),
        format!(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "{}"

[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/private-notes.git"
token = "super-secret-token"
"#,
            bundle.display()
        ),
    )
    .unwrap();

    let output = Command::new(&bin)
        .arg("roots")
        .arg("--format")
        .arg("json")
        .arg("--no-sync")
        .env("ROCCI_OKF_CONFIG", temp.join("okf.toml"))
        .env("ROCCI_CACHE", temp.join("cache"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains("super-secret-token"), "{stdout}");
    assert!(!stderr.contains("super-secret-token"), "{stderr}");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed[0]["id"], "notes");
    assert_eq!(parsed[1]["id"], "rocci");
    assert_eq!(parsed[1]["kind"], "directory");
    assert_eq!(parsed[1]["enabled"], true);
    assert_eq!(parsed[0]["enabled"], false);
    assert!(!output.status.success());
    let _ = fs::remove_dir_all(temp);
}

#[test]
fn roots_and_sync_file_git_remote() {
    let bin = rocci_okf_bin();
    let temp = temp_dir("roots-git");
    let remote = temp.join("remote");
    fs::create_dir_all(&remote).unwrap();
    git(&remote, &["init", "-b", "main"]);
    git(&remote, &["config", "user.email", "okf@example.com"]);
    git(&remote, &["config", "user.name", "OKF Test"]);
    fs::write(remote.join("index.md"), "okf_version: 1\n").unwrap();
    git(&remote, &["add", "index.md"]);
    git(&remote, &["commit", "-m", "init"]);
    let url = format!("file://{}", remote.canonicalize().unwrap().display());
    fs::write(
        temp.join("okf.toml"),
        format!(
            r#"
[[roots]]
id = "notes"
kind = "git"
url = "{url}"
"#
        ),
    )
    .unwrap();

    let output = Command::new(&bin)
        .arg("sync")
        .env("ROCCI_OKF_CONFIG", temp.join("okf.toml"))
        .env("ROCCI_CACHE", temp.join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(&bin)
        .arg("roots")
        .arg("--no-sync")
        .env("ROCCI_OKF_CONFIG", temp.join("okf.toml"))
        .env("ROCCI_CACHE", temp.join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        PathBuf::from(&printed).join("index.md").is_file(),
        "{printed}"
    );
    let _ = fs::remove_dir_all(temp);
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
