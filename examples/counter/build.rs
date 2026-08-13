fn main() {
    let manifest = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let config = manifest.join("../../rocci.toml");
    println!("cargo:rerun-if-changed={}", config.display());
    rocci_core::Config::from_file(&config).expect("rocci.toml must be valid");
}
