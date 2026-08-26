mod common;

use std::fs;
use std::process::Command;

use common::{okmate_bin, temp_dir, write_index};

#[test]
fn roots_json_lists_directory_without_secrets() {
    let root = temp_dir("roots-dir");
    write_index(&root);
    let cfg_dir = temp_dir("roots-cfg");
    let config = cfg_dir.join("config.toml");
    fs::write(
        &config,
        format!(
            "poll = \"5m\"\n[[roots]]\nid = \"docs\"\nkind = \"directory\"\npath = \"{}\"\n",
            root.display()
        ),
    )
    .unwrap();

    let output = Command::new(okmate_bin())
        .arg("roots")
        .arg("--format")
        .arg("json")
        .arg("--no-sync")
        .env("OKMATE_CONFIG", &config)
        .env("OKMATE_CACHE", cfg_dir.join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"id\": \"docs\""), "{stdout}");
    assert!(stdout.contains("\"kind\": \"directory\""), "{stdout}");
    assert!(!stdout.contains("token"), "{stdout}");
}

#[test]
fn roots_help_mentions_no_sync() {
    let output = Command::new(okmate_bin())
        .arg("roots")
        .arg("--help")
        .output()
        .unwrap();
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("--no-sync"), "{help}");
    assert!(help.contains("--format"), "{help}");
}
