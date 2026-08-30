use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=fixtures/roc_app.o");
    println!("cargo:rerun-if-env-changed=ROCCI_BASIC_WEBSERVER");
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() != Ok("wasm") {
        return;
    }
    let fixture = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("fixtures/roc_app.o");
    if !fixture.is_file() {
        panic!("missing fixtures/roc_app.o (Phase 1 wasm32 app object)");
    }
    println!("cargo:rustc-cdylib-link-arg={}", fixture.display());
}
