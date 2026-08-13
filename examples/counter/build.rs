fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let config = manifest.join("../../roc.toml");
    println!("cargo:rerun-if-changed={}", config.display());
    roc_core::Config::from_file(&config).expect("roc.toml must be valid");
}
