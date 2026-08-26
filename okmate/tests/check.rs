use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use okf::Profile;

fn okmate_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_okmate"))
}

fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "okmate-check-{}-{}-{}",
        name,
        std::process::id(),
        nonce
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_index(root: &Path) {
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
}

fn valid_rocci_concept(id: &str, body: &str) -> String {
    format!(
        "---\ntype: Architecture\ntitle: {id}\ndescription: Test concept {id}.\ntags: [domain/rocci, concern/architecture]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-17T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n---\n\n# {id}\n\n{body}\n"
    )
}

fn run_check_json(root: &Path) -> (bool, String) {
    let output = Command::new(okmate_bin())
        .arg("check")
        .arg(root)
        .arg("--profile")
        .arg("rocci")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

#[test]
fn no_subcommand_prints_help() {
    let output = Command::new(okmate_bin()).output().unwrap();
    assert!(!output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("Usage:") && text.contains("check"),
        "expected help mentioning check, got: {text}"
    );
}

#[test]
fn check_json_matches_okf_on_valid_bundle() {
    let root = temp_dir("ok");
    write_index(&root);
    fs::write(
        root.join("hello.md"),
        valid_rocci_concept("Hello", "A valid concept body."),
    )
    .unwrap();

    let engine = okf::check(&root, Profile::Rocci).unwrap();
    let (ok, stdout) = run_check_json(&root);
    assert_eq!(ok, !engine.has_errors());
    let cli: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&engine.json().unwrap()).unwrap();
    assert_eq!(cli, expected);
}

#[test]
fn check_json_matches_okf_on_invalid_bundle() {
    let root = temp_dir("err");
    write_index(&root);
    fs::write(
        root.join("broken.md"),
        "---\ntype: Architecture\ntitle: Broken\n---\n\n# Broken\n\nMissing required fields.\n",
    )
    .unwrap();

    let engine = okf::check(&root, Profile::Rocci).unwrap();
    assert!(engine.has_errors(), "fixture should fail rocci check");
    let (ok, stdout) = run_check_json(&root);
    assert!(!ok);
    let cli: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&engine.json().unwrap()).unwrap();
    assert_eq!(cli, expected);
}
