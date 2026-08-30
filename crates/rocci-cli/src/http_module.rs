use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::dispatch::PLATFORM;
use crate::driver;
use crate::run;

pub fn build_http_module(input: &Path, dest: &Path) -> Result<()> {
    let _roc = resolve_roc()?;
    let fork = resolve_basic_webserver()?;
    let platform = fork.join("platform/main.roc");
    if !platform.is_file() {
        bail!(
            "fork at {} has no platform/main.roc (wasm32 target required)",
            fork.display()
        );
    }
    let script = fork.join("scripts/build_wasm32_object.py");
    if !script.is_file() {
        bail!(
            "fork at {} has no scripts/build_wasm32_object.py",
            fork.display()
        );
    }

    let plan = run::standalone_island_app_plan(input)?;
    let src_dir = input
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let workspace = driver::stage_app_workspace(&plan, src_dir, "http-module")?;
    let generated = plan.main_roc();
    if !generated.contains(PLATFORM) {
        bail!("generated main.roc did not pin the 0.16.0 platform URL");
    }
    let platform_ref = platform
        .canonicalize()
        .unwrap_or(platform)
        .to_string_lossy()
        .into_owned();
    fs::write(
        workspace.path.join("main.roc"),
        plan.main_roc_with_platform(&platform_ref),
    )
    .context("write http-module main.roc with local fork platform")?;

    let output = Command::new("python3")
        .arg(&script)
        .arg("--app")
        .arg(workspace.path.join("main.roc"))
        .output()
        .context("run fork scripts/build_wasm32_object.py")?;
    if !output.status.success() {
        let log = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("roc build --target=wasm32 for --http-module failed\n{log}{err}");
    }
    let captured = fork.join("platform/targets/wasm32/roc_app.o");
    if !captured.is_file() {
        bail!(
            "wasm32 object missing after roc build: {}",
            captured.display()
        );
    }
    let object = dest.with_extension("roc_app.o");
    if let Some(parent) = object.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&captured, &object)
        .with_context(|| format!("copy {} -> {}", captured.display(), object.display()))?;

    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("../..");
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .args([
            "build",
            "-p",
            "rocci-wasi-http-component",
            "--target",
            "wasm32-wasip2",
        ])
        .env("ROCCI_ROC_APP_O", &object)
        .current_dir(&workspace_root)
        .status()
        .context("build rocci-wasi-http-component for --http-module")?;
    if !status.success() {
        bail!("cargo build -p rocci-wasi-http-component --target wasm32-wasip2 failed");
    }

    let bytes = read_component_bytes(&workspace_root)?;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(dest, bytes).with_context(|| format!("write {}", dest.display()))
}

pub fn resolve_basic_webserver() -> Result<PathBuf> {
    if let Ok(explicit) = env::var("ROCCI_BASIC_WEBSERVER") {
        let path = PathBuf::from(explicit);
        if path.join("platform/main.roc").is_file() {
            return Ok(path);
        }
        bail!(
            "ROCCI_BASIC_WEBSERVER={} has no platform/main.roc",
            path.display()
        );
    }
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let sibling = workspace.join("../roc-basic-webserver");
    if sibling.join("platform/main.roc").is_file() {
        return sibling
            .canonicalize()
            .with_context(|| format!("canonicalize {}", sibling.display()));
    }
    bail!(
        "missing ../roc-basic-webserver (set ROCCI_BASIC_WEBSERVER); --http-module compiles the .rocci against the fork wasm32 platform"
    )
}

fn resolve_roc() -> Result<PathBuf> {
    if let Ok(explicit) = env::var("ROC") {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return Ok(path);
        }
        bail!("ROC={} is not a file", path.display());
    }
    which("roc").ok_or_else(|| {
        anyhow::anyhow!("`roc` not on PATH; --http-module compiles the input .rocci")
    })
}

fn which(name: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(name);
            candidate.is_file().then_some(candidate)
        })
    })
}

fn read_component_bytes(workspace: &Path) -> Result<Vec<u8>> {
    if let Ok(explicit) = env::var("ROCCI_HTTP_MODULE_WASM") {
        let path = PathBuf::from(explicit);
        if path.is_file() {
            return fs::read(&path)
                .with_context(|| format!("read component artifact {}", path.display()));
        }
    }
    let mut candidates = Vec::new();
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        candidates
            .push(PathBuf::from(target).join("wasm32-wasip2/debug/rocci_wasi_http_component.wasm"));
    }
    candidates.push(workspace.join("target/wasm32-wasip2/debug/rocci_wasi_http_component.wasm"));
    for path in &candidates {
        if path.is_file() {
            return fs::read(path)
                .with_context(|| format!("read component artifact {}", path.display()));
        }
    }
    bail!("component wasm not found after build; set ROCCI_HTTP_MODULE_WASM")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fork_error_names_the_flag() {
        if resolve_basic_webserver().is_ok() {
            return;
        }
        let err = resolve_basic_webserver().unwrap_err().to_string();
        assert!(err.contains("--http-module"), "{err}");
        assert!(err.contains("roc-basic-webserver"), "{err}");
    }
}
