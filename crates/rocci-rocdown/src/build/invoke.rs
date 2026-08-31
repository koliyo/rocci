use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};
use rocci_template::{MappedModule, remap_roc_output};
use sha2::{Digest, Sha256};

use crate::BASIC_CLI_PLATFORM;
use crate::plan::BuildPlan;
use crate::runtime;

pub(crate) fn theme_maps(plan: &BuildPlan) -> Vec<MappedModule> {
    plan.theme_modules
        .iter()
        .filter(|m| !m.segments.is_empty())
        .map(|m| MappedModule {
            type_name: m.type_name.clone(),
            generated: m.roc.clone(),
            source_name: m.source_name.clone(),
            source_src: m.src.clone(),
            segments: m.segments.clone(),
        })
        .collect()
}

pub(crate) fn staged_fingerprints(
    plan: &BuildPlan,
    is_wasm: bool,
) -> Vec<rocci_roc_host::InputFingerprint> {
    let mut fps = Vec::new();
    for module in &plan.theme_modules {
        fps.push(rocci_roc_host::InputFingerprint::from_bytes(
            &format!("{}.roc", module.type_name),
            module.roc.as_bytes(),
        ));
    }
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "Html.roc",
        runtime::HTML.as_bytes(),
    ));
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "RocdownBuild.roc",
        staged_build_roc(plan, is_wasm).as_bytes(),
    ));
    fps
}

pub(crate) fn staged_build_roc(plan: &BuildPlan, is_wasm: bool) -> String {
    runtime::build_roc(is_wasm).replace(
        "        # rocci-widget-kind-arms\n",
        &plan.widget_render_arms,
    )
}

pub(crate) fn roc_source_hash(
    pages_roc: &str,
    theme_modules: &[crate::plan::CompiledThemeModule],
    main_roc: &str,
    build_roc: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(runtime::HTML.as_bytes());
    hasher.update(build_roc.as_bytes());
    hasher.update(pages_roc.as_bytes());
    for m in theme_modules {
        hasher.update(m.type_name.as_bytes());
        hasher.update(m.roc.as_bytes());
    }
    hasher.update(main_roc.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn main_roc(is_wasm: bool) -> String {
    if is_wasm {
        String::from(
            "\
app [main!] { pf: platform \"platform/main.roc\" }

main! : {} => [Ok({}), Err([Exit(I32)])]
main! = |{}| {
    res : [Ok({}), Err([Exit(I32)])]
    res = Ok({})
    res
}
",
        )
    } else {
        format!(
            "\
app [main!] {{ pf: platform \"{BASIC_CLI_PLATFORM}\" }}

import RocdownBuild

main! = |_args| {{
    RocdownBuild.run!({{}})?
    Ok({{}})
}}
"
        )
    }
}

pub(crate) fn apply_html(
    workspace: &Path,
    staging: &Path,
    maps: &[MappedModule],
    is_wasm: bool,
    compiled: &Path,
    plan: &BuildPlan,
) -> Result<String> {
    if is_wasm {
        let wasm_out = invoke_wasm_apply(compiled, workspace, staging)?;
        fs::write(
            workspace.join("RocdownBuild.roc"),
            staged_build_roc(plan, false),
        )
        .context("failed to write native RocdownBuild.roc")?;
        fs::write(workspace.join("main.roc"), main_roc(false))
            .context("failed to write native apply main.roc")?;
        let native_bin = workspace.join("apply");
        let native_compile = invoke_roc_build(workspace, &native_bin, maps)?;
        let native_out = invoke_apply(&native_bin, workspace, staging, maps)?;
        Ok(format!("{wasm_out}{native_compile}{native_out}"))
    } else {
        invoke_apply(compiled, workspace, staging, maps)
    }
}

pub(crate) fn invoke_roc_build(
    workspace: &Path,
    apply_bin: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--opt=dev")
        .arg(format!("--output={}", apply_bin.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build")?;
    let combined = finish_roc(output, maps)?;
    if !apply_bin.is_file() {
        bail!("roc build did not write {}", apply_bin.display());
    }
    Ok(combined)
}

pub(crate) fn invoke_roc_wasm_build(
    workspace: &Path,
    wasm_file: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--target=wasm32")
        .arg(format!("--output={}", wasm_file.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build for wasm32")?;
    let combined = finish_roc(output, maps)?;
    if !wasm_file.is_file() {
        bail!("roc build did not write {}", wasm_file.display());
    }
    Ok(combined)
}

pub(crate) fn invoke_wasm_apply(
    wasm_file: &Path,
    workspace: &Path,
    staging: &Path,
) -> Result<String> {
    let host = rocci_roc_host::WasmHost::from_file(wasm_file)?;
    host.run_wasi_with_preopens(staging, &[workspace])
}

pub(crate) fn invoke_apply(
    apply_bin: &Path,
    workspace: &Path,
    staging: &Path,
    maps: &[MappedModule],
) -> Result<String> {
    let output = Command::new(apply_bin)
        .current_dir(workspace)
        .env("ROCDOWN_STAGING", staging)
        .output()
        .context("failed to run rocdown applicator")?;
    finish_roc(output, maps)
}

pub(crate) fn finish_roc(output: std::process::Output, maps: &[MappedModule]) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = if stdout.is_empty() {
        stderr.clone()
    } else if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{stdout}{stderr}")
    };
    if output.status.success() {
        return Ok(combined);
    }
    let mapped = remap_roc_output(&combined, maps);
    for frame in mapped {
        eprintln!("{}", frame.render_for_stderr());
    }
    let hint = if combined.contains("does not support the wasm32 target") {
        "\n\nhint: The basic-cli platform only supports native compilation targets (x64mac, arm64mac, x64win, x64musl, arm64musl).\nWasm host (--host wasm) is planned for Phase 5 with a custom Roc wasm platform.\nPlease use '--host native' (or default '--host auto') instead."
    } else {
        ""
    };
    bail!(
        "roc rocdown build failed{}{hint}",
        if combined.trim().is_empty() {
            String::new()
        } else {
            format!("\n{}", combined.trim_end())
        }
    );
}
