use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_sqlite3_c() -> PathBuf {
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .expect("CARGO_HOME or HOME");
    let src = cargo_home.join("registry/src");
    find_named(&src, "sqlite3.c", "libsqlite3-sys").unwrap_or_else(|| {
        panic!(
            "sqlite3.c not found under {}. Add libsqlite3-sys to the cargo cache.",
            src.display()
        )
    })
}

fn find_named(root: &Path, file: &str, crate_needle: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|name| name.to_str()) == Some(file)
                && path.to_string_lossy().contains(crate_needle)
            {
                return Some(path);
            }
        }
    }
    None
}

fn compile_sqlite3(out_dir: &Path) -> PathBuf {
    let dest = out_dir.join("sqlite3.o");
    let source = find_sqlite3_c();
    println!("cargo:rerun-if-changed={}", source.display());
    let zig = env::var_os("ZIG")
        .map(PathBuf::from)
        .or_else(|| which("zig"))
        .unwrap_or_else(|| PathBuf::from("/opt/homebrew/bin/zig"));
    let status = Command::new(&zig)
        .env("ZIG_GLOBAL_CACHE_DIR", out_dir.join("zig-global"))
        .env("ZIG_LOCAL_CACHE_DIR", out_dir.join("zig-local"))
        .args([
            "cc",
            "-target",
            "wasm32-wasi-musl",
            "-c",
            "-O2",
            "-DSQLITE_THREADSAFE=0",
            "-DSQLITE_OMIT_LOAD_EXTENSION",
            "-DSQLITE_DEFAULT_MEMSTATUS=0",
            "-DSQLITE_OMIT_DEPRECATED",
            "-DSQLITE_OMIT_SHARED_CACHE",
        ])
        .arg(&source)
        .arg("-o")
        .arg(&dest)
        .status()
        .unwrap_or_else(|err| panic!("zig cc sqlite3.c: {err}"));
    if !status.success() {
        panic!("zig cc sqlite3.c failed: {status}");
    }
    dest
}

fn which(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            candidate.is_file().then_some(candidate)
        })
    })
}

fn main() {
    println!("cargo:rerun-if-changed=fixtures/sqlite_row.o");
    println!("cargo:rerun-if-env-changed=ROCCI_BASIC_WEBSERVER");
    println!("cargo:rerun-if-env-changed=ZIG");
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() != Ok("wasm") {
        return;
    }
    let fixture = env::var_os("ROCCI_ROC_APP_O")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("fixtures/sqlite_row.o")
        });
    println!("cargo:rerun-if-env-changed=ROCCI_ROC_APP_O");
    println!("cargo:rerun-if-changed={}", fixture.display());
    if !fixture.is_file() {
        panic!(
            "missing Roc wasm32 object {} (set ROCCI_ROC_APP_O or fixtures/sqlite_row.o)",
            fixture.display()
        );
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bytes =
        std::fs::read(&fixture).unwrap_or_else(|err| panic!("read {}: {err}", fixture.display()));
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&bytes, &mut hasher);
    let linked = out_dir.join(format!(
        "roc_app-{:016x}.o",
        std::hash::Hasher::finish(&hasher)
    ));
    std::fs::write(&linked, bytes)
        .unwrap_or_else(|err| panic!("write {}: {err}", linked.display()));
    let sqlite = compile_sqlite3(&out_dir);
    println!("cargo:rustc-cdylib-link-arg={}", linked.display());
    println!("cargo:rustc-cdylib-link-arg={}", sqlite.display());
}
