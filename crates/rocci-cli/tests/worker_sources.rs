use std::{fs, path::Path};

use rocci_template::{LowerOptions, SourceFile, compile};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[test]
fn worker_golden_sources_are_frozen_and_lower_without_source_adaptations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/worker");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(root.join("manifest.json")).unwrap()).unwrap();
    for (fixture, definition) in manifest["fixtures"].as_object().unwrap() {
        assert!(definition["source_edits"].as_array().unwrap().is_empty());
        for (name, metadata) in definition["files"].as_object().unwrap() {
            let path = root.join(fixture).join(name);
            let bytes = fs::read(&path).unwrap();
            assert_eq!(bytes.len() as u64, metadata["bytes"].as_u64().unwrap());
            assert_eq!(
                Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                metadata["sha256"].as_str().unwrap(),
                "frozen input drift: {fixture}/{name}"
            );
            if path.extension().is_some_and(|ext| ext == "rocci") {
                let source = std::str::from_utf8(&bytes).unwrap();
                let lowered = compile(
                    SourceFile::new(path.to_str().unwrap(), source),
                    &LowerOptions::default(),
                );
                assert!(
                    !lowered.has_errors(),
                    "{fixture}/{name}: {:?}",
                    lowered.diagnostics
                );
            }
        }
    }
}
