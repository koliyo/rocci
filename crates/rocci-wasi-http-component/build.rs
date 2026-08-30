use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=fixtures/env_log.o");
    println!("cargo:rerun-if-env-changed=ROCCI_BASIC_WEBSERVER");
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() != Ok("wasm") {
        return;
    }
    let fixture = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("fixtures/env_log.o");
    if !fixture.is_file() {
        panic!("missing fixtures/env_log.o (Phase 3 wasm32 env-log object)");
    }
    println!("cargo:rustc-cdylib-link-arg={}", fixture.display());
}
