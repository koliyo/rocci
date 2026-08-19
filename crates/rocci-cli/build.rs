use std::{fs, path::PathBuf};

fn main() {
    let dist = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../playground/dist");
    fs::create_dir_all(&dist).expect("failed to create playground/dist");
    for name in &["app.js", "compiler-worker.js", "styles.css", "compiler.wasm"] {
        let path = dist.join(name);
        if !path.exists() {
            fs::write(&path, b"").expect("failed to create placeholder");
        }
    }
    println!("cargo:rerun-if-changed=../../playground/dist");
}
