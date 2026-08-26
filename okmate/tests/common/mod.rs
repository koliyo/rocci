use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn okmate_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_okmate"))
}

pub fn temp_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "okmate-test-{}-{}-{}",
        name,
        std::process::id(),
        nonce
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

pub fn write_index(root: &Path) {
    fs::write(
        root.join("index.md"),
        "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
    )
    .unwrap();
}

pub fn valid_rocci_concept(id: &str, extra_yaml: &str, body: &str) -> String {
    format!(
        "---\ntype: Architecture\ntitle: {id}\ndescription: Test concept {id}.\ntags: [domain/rocci, concern/architecture]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-17T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n{extra_yaml}---\n\n# {id}\n\n{body}\n"
    )
}
