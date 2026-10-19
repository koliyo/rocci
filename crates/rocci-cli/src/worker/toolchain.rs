use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub format: u32,
    pub abi: u32,
    pub compiler: String,
    pub compiler_sha256: std::collections::BTreeMap<String, String>,
    pub host_sha256: String,
    pub runtime_sha256: String,
}

pub fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
pub fn file_hash(path: &Path) -> Result<String> {
    Ok(hash(&fs::read(path)?))
}
pub fn cache() -> Result<PathBuf> {
    Ok(crate::datastar_asset::cache_dir()?.join("worker"))
}

fn download(url: &str, limit: u64) -> Result<Vec<u8>> {
    eprintln!("[rocci worker] download {url}");
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("Download failed: {url}"))?;
    let mut bytes = Vec::new();
    response
        .body_mut()
        .as_reader()
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Toolchain download exceeded {limit} bytes"
    );
    eprintln!("[rocci worker] downloaded {} bytes", bytes.len());
    Ok(bytes)
}
fn on_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?)
        .map(|path| path.join(name))
        .find(|path| path.is_file())
}
fn check_compiler(path: &Path) -> Result<()> {
    let output = Command::new(path)
        .arg("version")
        .output()
        .with_context(|| format!("Run compiler {}", path.display()))?;
    ensure!(
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).trim()
                == format!(
                    "Roc compiler version {}",
                    rocci_worker_platform::COMPILER_VERSION
                ),
        "Worker builds require {}; native .roc-version is independent",
        rocci_worker_platform::COMPILER_VERSION
    );
    Ok(())
}
pub fn compiler(explicit: Option<&Path>, offline: bool) -> Result<PathBuf> {
    if let Some(path) = explicit {
        let path = if path.components().count() == 1 && !path.is_file() {
            on_path(path.to_str().context("Compiler path must be Unicode")?)
                .context("Compiler is not on PATH")?
        } else {
            path.to_path_buf()
        };
        let path = fs::canonicalize(path)?;
        check_compiler(&path)?;
        return Ok(path);
    }
    if let Some(path) = on_path("roc")
        && check_compiler(&path).is_ok()
    {
        return Ok(fs::canonicalize(path)?);
    }
    let (host, checksum) = match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => (
            "macos_apple_silicon",
            "aa6339579730fcea4753940bec21f5d8cf8be8e25659ec001abccbc3463a60b2",
        ),
        ("linux", "x86_64") => (
            "linux_x86_64",
            "dcb99abdf17d062246c05e394357ad682b883a6bcff7b2e581e20564ea88ffc4",
        ),
        _ => bail!(
            "Automatic Worker toolchain installation supports Apple Silicon macOS and x64 Linux; supply --roc and --host-archive for a separately qualified host"
        ),
    };
    let directory = cache()?
        .join(rocci_worker_platform::COMPILER_VERSION)
        .join(host);
    let executable = directory.join("roc");
    if executable.is_file() {
        check_compiler(&executable)?;
        return Ok(executable);
    }
    ensure!(
        !offline,
        "Worker compiler is not cached; preseed the cache or supply --roc"
    );
    let filename = format!("roc_nightly-{host}-2026-10-09-258ab27.tar.gz");
    let url = format!(
        "https://github.com/roc-lang/nightlies/releases/download/{}/{filename}",
        rocci_worker_platform::COMPILER_VERSION
    );
    let bytes = download(&url, 256 * 1024 * 1024)?;
    ensure!(
        hash(&bytes) == checksum,
        "Roc compiler archive checksum mismatch"
    );
    fs::create_dir_all(&directory)?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(Cursor::new(bytes)));
    let mut found = false;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if path.file_name().is_some_and(|name| name == "roc")
            && entry.header().entry_type().is_file()
        {
            ensure!(
                entry.size() < 256 * 1024 * 1024,
                "Compiler executable exceeds size limit"
            );
            let temporary = directory.join(format!("roc.next-{}", std::process::id()));
            let mut output = fs::File::create(&temporary)?;
            std::io::copy(&mut entry, &mut output)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))?;
            }
            check_compiler(&temporary)?;
            fs::rename(temporary, &executable)?;
            found = true;
            break;
        }
    }
    ensure!(found, "Pinned archive contains no Roc executable");
    eprintln!(
        "[rocci worker] cached {} at {}",
        rocci_worker_platform::COMPILER_VERSION,
        executable.display()
    );
    Ok(executable)
}
pub fn host_archive(explicit: Option<&Path>, offline: bool) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return fs::canonicalize(path).context("Read --host-archive");
    }
    let in_tree = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-unknown-unknown/release/librocci_worker_platform.a");
    if in_tree.is_file() {
        return Ok(fs::canonicalize(in_tree)?);
    }
    let tag = crate::dispatch::github_platform_release_tag();
    let directory = cache()?.join(format!("host-{tag}"));
    let archive = directory.join("rocci-worker-host.a");
    let checksum = directory.join("rocci-worker-host.a.sha256");
    if archive.is_file() && checksum.is_file() {
        ensure!(
            file_hash(&archive)?
                == fs::read_to_string(checksum)?
                    .split_whitespace()
                    .next()
                    .unwrap_or(""),
            "Cached Worker host checksum mismatch"
        );
        return Ok(archive);
    }
    ensure!(
        !offline,
        "Worker host is not cached; preseed it or supply --host-archive"
    );
    let url =
        format!("https://github.com/koliyo/rocci/releases/download/{tag}/rocci-worker-host.a");
    let expected = String::from_utf8(download(&format!("{url}.sha256"), 1024)?)?;
    let bytes = download(&url, 128 * 1024 * 1024)?;
    ensure!(
        hash(&bytes) == expected.split_whitespace().next().unwrap_or(""),
        "Worker host checksum mismatch"
    );
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("host.next"), bytes)?;
    fs::rename(directory.join("host.next"), &archive)?;
    fs::write(&checksum, expected)?;
    Ok(archive)
}
