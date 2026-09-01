use rocci_roc_host::*;
use std::fs;
use std::path::PathBuf;

fn temp_cache() -> (TwoTierCache, PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "rocci-host-test-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    (TwoTierCache::new(dir.clone()), dir)
}

#[test]
fn test_fingerprints_and_hashes() {
    let gen_hash = compute_gen_hash(
        "0.1.0",
        "default",
        &[("Theme.rocci", b"import Html")],
        &[("Html.roc", b"module []")],
    );
    assert!(!gen_hash.is_empty());

    let compile_hash = compute_compile_hash(
        &gen_hash,
        "roc 0.22.0",
        "native:arm64",
        "dev",
        "basic-cli",
        "0.1.0",
    );
    assert!(!compile_hash.is_empty());
    assert_ne!(gen_hash, compile_hash);
}

#[test]
fn test_tier1_roc_cache_lifecycle() {
    let (cache, dir) = temp_cache();
    let gen_hash = "abc123gen";
    let modules = [("Theme.roc", "module []"), ("Base.roc", "base []")];
    let maps = [("Theme.map", "{\"mappings\":\"\"}")];
    let fp = vec![InputFingerprint::from_bytes("Theme.rocci", b"import Html")];

    assert!(cache.lookup_roc(gen_hash).is_none());
    let path = cache.store_roc(gen_hash, &modules, &maps, &fp).unwrap();
    assert!(path.is_dir());

    let hit = cache.lookup_roc(gen_hash).expect("should hit cache");
    assert!(hit.modules_dir.join("Theme.roc").is_file());
    assert!(hit.modules_dir.join("Base.roc").is_file());
    assert!(hit.maps_dir.join("Theme.map").is_file());
    assert_eq!(hit.manifest.version, env!("CARGO_PKG_VERSION"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_tier2_renderer_cache_and_integrity() {
    let (cache, dir) = temp_cache();
    let compile_hash = "def456compile";
    let target = "native:arm64";
    let artifact = b"fake binary executable";
    let fp = vec![InputFingerprint::from_bytes("Theme.rocci", b"import Html")];

    assert!(cache.lookup_renderer(compile_hash, target, &fp).is_none());
    let path = cache
        .store_renderer(compile_hash, target, artifact, &fp)
        .unwrap();
    assert!(path.is_file());

    let hit = cache
        .lookup_renderer(compile_hash, target, &fp)
        .expect("should hit cache");
    assert_eq!(fs::read(hit).unwrap(), artifact);

    let drifted = vec![InputFingerprint::from_bytes("Theme.rocci", b"import Css")];
    match cache.inspect_renderer(compile_hash, target, &drifted) {
        RendererInspect::Stale { detail } => {
            assert!(detail.contains("Theme.rocci"), "{detail}");
        }
        other => panic!("expected stale, got {other:?}"),
    }

    // Corrupt the binary to test integrity check
    let bin_path = cache.renderer_dir(compile_hash).join("apply");
    fs::write(&bin_path, b"corrupted").unwrap();
    assert!(
        cache.lookup_renderer(compile_hash, target, &fp).is_none(),
        "corrupted binary must fail check"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[cfg(feature = "wasmtime")]
#[test]
fn test_wasm_host_runner() {
    let wat = r#"
(module
  (import "env" "roc_host_get_view_len" (func $get_view_len (result i32)))
  (import "env" "roc_host_get_article_len" (func $get_article_len (result i32)))
  (import "env" "roc_host_read_view" (func $read_view (param i32) (result i32)))
  (import "env" "roc_host_read_article" (func $read_article (param i32) (result i32)))
  (import "env" "roc_host_write_output" (func $write_output (param i32 i32) (result i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) "<!DOCTYPE html><html><body><h1>Rendered</h1></body></html>")
  (func (export "render")
    (drop (call $write_output (i32.const 0) (i32.const 58)))
  )
)
"#;
    let wasm_bytes = wat::parse_str(wat).unwrap();
    let host = WasmHost::from_bytes(&wasm_bytes).unwrap();
    let html = host.render("{}", "<p>hello</p>").unwrap();
    assert_eq!(
        html,
        "<!DOCTYPE html><html><body><h1>Rendered</h1></body></html>"
    );
}
