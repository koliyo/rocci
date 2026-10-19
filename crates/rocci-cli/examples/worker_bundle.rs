//! Release tooling: export the shared Worker platform without a template frontend.
use std::{env, fs, path::PathBuf, process::Command};
fn main() -> anyhow::Result<()> {
    let mut args = env::args_os().skip(1);
    let output = fs::canonicalize(PathBuf::from(args.next().expect("output directory")))?;
    let archive = fs::canonicalize(PathBuf::from(args.next().expect("host archive")))?;
    let roc = args.next().unwrap_or_else(|| "roc".into());
    let version = Command::new(&roc).arg("version").output()?;
    anyhow::ensure!(
        version.status.success()
            && String::from_utf8_lossy(&version.stdout)
                .contains(rocci_worker_platform::COMPILER_VERSION),
        "Use the qualified Worker compiler"
    );
    let stage = output.join("staged");
    rocci_cli::worker::stage(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../rocci-worker-platform/examples/hello/main.roc"),
        &stage,
        &archive,
    )?;
    fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rocci-worker-platform/LICENSE-UPL"),
        stage.join("platform/LICENSE-UPL"),
    )?;
    let platform = stage.join("platform");
    let mut files = fs::read_dir(&platform)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.retain(|path| {
        path.extension().is_some_and(|extension| extension == "roc")
            || path.file_name().is_some_and(|name| name == "LICENSE-UPL")
    });
    files.sort();
    files.push(platform.join("targets/wasm32/libhost.a"));
    let result = Command::new(roc)
        .current_dir(&platform)
        .arg("bundle")
        .args(&files)
        .arg("--output-dir")
        .arg(&output)
        .output()?;
    anyhow::ensure!(
        result.status.success(),
        "Worker platform bundle failed: {}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let bundles = fs::read_dir(&output)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    let bundle = bundles
        .iter()
        .find(|path| path.to_string_lossy().ends_with(".tar.zst"))
        .expect("Roc emitted a bundle");
    fs::write(
        output.join("rocci-worker-platform.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"format":1,"abi":rocci_worker_platform::ABI_VERSION,"compiler":rocci_worker_platform::COMPILER_VERSION,"version":env!("CARGO_PKG_VERSION"),"bundle":bundle.file_name().unwrap().to_string_lossy(),"runtime":"rocci-worker-runtime.tar.gz"}),
        )?,
    )?;
    println!("{}", bundle.display());
    Ok(())
}
