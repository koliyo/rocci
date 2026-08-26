mod common;

use std::fs;
use std::process::Command;

use common::{okmate_bin, temp_dir, valid_rocci_concept, write_index};
use okf::Profile;

fn run_check_json(root: &std::path::Path) -> (bool, String) {
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
        valid_rocci_concept("Hello", "", "A valid concept body."),
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
