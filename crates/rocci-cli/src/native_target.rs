use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};
use clap::ValueEnum;

/// Roc `roc build --target=` names for process binaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum NativeTarget {
    /// Linux x86_64 with musl (static linking)
    #[value(name = "x64musl")]
    X64Musl,
    /// Linux ARM64 with musl (static linking)
    #[value(name = "arm64musl")]
    Arm64Musl,
    /// Linux x86_64 with glibc (dynamic linking)
    #[value(name = "x64glibc")]
    X64Glibc,
    /// Linux ARM64 with glibc (dynamic linking)
    #[value(name = "arm64glibc")]
    Arm64Glibc,
    /// macOS x86_64
    #[value(name = "x64mac")]
    X64Mac,
    /// macOS ARM64 (Apple Silicon)
    #[value(name = "arm64mac")]
    Arm64Mac,
    /// Windows x86_64
    #[value(name = "x64win")]
    X64Win,
    /// Windows ARM64
    #[value(name = "arm64win")]
    Arm64Win,
    /// WebAssembly (WASI / freestanding wasm32)
    #[value(name = "wasm32")]
    Wasm32,
}

impl NativeTarget {
    pub fn as_roc_target(self) -> &'static str {
        match self {
            Self::X64Musl => "x64musl",
            Self::Arm64Musl => "arm64musl",
            Self::X64Glibc => "x64glibc",
            Self::Arm64Glibc => "arm64glibc",
            Self::X64Mac => "x64mac",
            Self::Arm64Mac => "arm64mac",
            Self::X64Win => "x64win",
            Self::Arm64Win => "arm64win",
            Self::Wasm32 => "wasm32",
        }
    }
}

impl std::fmt::Display for NativeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_roc_target())
    }
}

pub fn roc_build_args(main_roc: &str, output: &Path, target: Option<NativeTarget>) -> Vec<String> {
    let mut args = vec!["build".to_string(), main_roc.to_string()];
    if let Some(target) = target {
        args.push(format!("--target={}", target.as_roc_target()));
    }
    args.push(format!("--output={}", output.display()));
    args
}

pub fn build_roc_server(app_dir: &Path, output: &Path, target: Option<NativeTarget>) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let args = roc_build_args("main.roc", output, target);
    let result = Command::new("roc")
        .current_dir(app_dir)
        .args(&args)
        .output()
        .context("failed to run `roc build`; is roc on PATH?")?;
    if !result.status.success() {
        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);
        let target_note = match target {
            Some(target) => format!(
                "\nroc build --target={} failed; not falling back to a host-native binary",
                target.as_roc_target()
            ),
            None => String::new(),
        };
        bail!("roc build failed:{target_note}\n{stdout}{stderr}");
    }
    if !output.is_file() {
        bail!("roc build did not write {}", output.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn musl_target_is_passed_to_roc_and_not_to_host_apply() {
        let output = PathBuf::from("/tmp/server");
        let args = roc_build_args("main.roc", &output, Some(NativeTarget::X64Musl));
        assert_eq!(
            args,
            vec![
                "build".to_string(),
                "main.roc".to_string(),
                "--target=x64musl".to_string(),
                "--output=/tmp/server".to_string(),
            ]
        );
        let host_native = roc_build_args("main.roc", &output, None);
        assert!(!host_native.iter().any(|arg| arg.starts_with("--target=")));
    }

    #[test]
    fn all_roc_targets_round_trip_through_as_roc_target() {
        for target in NativeTarget::value_variants() {
            assert_eq!(
                NativeTarget::from_str(target.as_roc_target(), true),
                Ok(*target)
            );
        }
    }

    #[test]
    fn target_x64musl_produces_elf_or_fails_with_roc_output() {
        if skip_without_roc() {
            return;
        }
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/datastar");
        let out_dir = std::env::temp_dir().join(format!(
            "rocci-target-x64musl-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&out_dir).unwrap();
        let binary = out_dir.join("server");
        let result = build_roc_server(&root, &binary, Some(NativeTarget::X64Musl));
        match result {
            Ok(()) => {
                let bytes = fs::read(&binary).unwrap();
                assert!(
                    bytes.starts_with(&[0x7f, b'E', b'L', b'F']),
                    "x64musl output must be ELF, not a host-native fallback"
                );
            }
            Err(err) => {
                let message = format!("{err:#}");
                assert!(
                    message.contains("roc build failed"),
                    "failure must include Roc output, got {message}"
                );
                assert!(
                    !binary.is_file()
                        || fs::read(&binary)
                            .unwrap()
                            .starts_with(&[0x7f, b'E', b'L', b'F']),
                    "must not leave a host-native binary after a failed musl build"
                );
            }
        }
        let _ = fs::remove_dir_all(&out_dir);
    }

    fn skip_without_roc() -> bool {
        !Command::new("roc")
            .arg("help")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}
