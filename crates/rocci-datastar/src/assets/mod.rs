pub mod version;

pub use version::{DEFAULT_VERSION, parse_version, tag_name};

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::error::{DatastarError, Result};

pub const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;
pub const JSDELIVR_URL: &str =
    "https://cdn.jsdelivr.net/gh/starfederation/datastar@{tag}/bundles/datastar.js";
pub const GITHUB_RAW_URL: &str =
    "https://raw.githubusercontent.com/starfederation/datastar/{tag}/bundles/datastar.js";
pub const GITHUB_LATEST_URL: &str =
    "https://api.github.com/repos/starfederation/datastar/releases/latest";

pub fn cache_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("ROCCI_CACHE") {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return Err(DatastarError::CacheAccess(
                "ROCCI_CACHE must not be empty".to_string(),
            ));
        }
        return Ok(path);
    }
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| DatastarError::CacheAccess("cannot determine home directory".to_string()))?;
    Ok(PathBuf::from(home).join(".rocci").join("cache"))
}

pub fn looks_like_datastar_js(bytes: &[u8]) -> bool {
    let Ok(head) = std::str::from_utf8(&bytes[..bytes.len().min(1024)]) else {
        return false;
    };
    head.contains("datastar") || head.contains("Datastar") || head.contains("data-")
}

pub fn parse_version_comment(bytes: &[u8]) -> Option<String> {
    let head = std::str::from_utf8(&bytes[..bytes.len().min(512)]).ok()?;
    let line = head.lines().next()?;
    let marker = line.find("Datastar v")?;
    let rest = &line[marker + "Datastar v".len()..];
    let candidate = rest
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .next()?;
    parse_version(candidate).ok()
}

pub fn copy_if_changed(src: &Path, dest: &Path) -> Result<bool> {
    let src_bytes = fs::read(src).map_err(|source| DatastarError::ReadFile {
        path: src.to_path_buf(),
        source,
    })?;
    if dest.is_file()
        && let Ok(dest_bytes) = fs::read(dest)
        && dest_bytes == src_bytes
    {
        return Ok(false);
    }
    fs::write(dest, &src_bytes).map_err(|source| DatastarError::WriteFile {
        path: dest.to_path_buf(),
        source,
    })?;
    Ok(true)
}

#[cfg(feature = "fetch")]
pub fn ensure_cached(version: &str) -> Result<PathBuf> {
    let version = parse_version(version)?;
    let tag = tag_name(&version);
    let dir = cache_dir()?.join("datastar").join(&tag);
    let js = dir.join("datastar.js");
    let sha_path = dir.join("sha256");

    if js.is_file()
        && let Ok(bytes) = fs::read(&js)
        && looks_like_datastar_js(&bytes)
    {
        let actual = hex_sha256(&bytes);
        if let Ok(expected) = fs::read_to_string(&sha_path) {
            if expected.trim() == actual {
                return Ok(js);
            }
        } else {
            let _ = fs::write(&sha_path, &actual);
            return Ok(js);
        }
    }

    fs::create_dir_all(&dir).map_err(|source| DatastarError::CreateDir {
        path: dir.clone(),
        source,
    })?;

    let bytes = download_datastar(&tag)?;
    let hash = hex_sha256(&bytes);
    let tmp = dir.join("datastar.js.tmp");
    fs::write(&tmp, &bytes).map_err(|source| DatastarError::WriteFile {
        path: tmp.clone(),
        source,
    })?;
    fs::write(&sha_path, &hash).map_err(|source| DatastarError::WriteFile {
        path: sha_path,
        source,
    })?;
    fs::rename(&tmp, &js).map_err(|source| DatastarError::RenameFile {
        from: tmp,
        to: js.clone(),
        source,
    })?;
    Ok(js)
}

#[cfg(feature = "fetch")]
fn hex_sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(feature = "fetch")]
pub fn download_datastar(tag: &str) -> Result<Vec<u8>> {
    let jsdelivr = JSDELIVR_URL.replace("{tag}", tag);
    match http_get_bytes(&jsdelivr) {
        Ok(bytes) if looks_like_datastar_js(&bytes) => return Ok(bytes),
        _ => {}
    }

    let github_raw = GITHUB_RAW_URL.replace("{tag}", tag);
    match http_get_bytes(&github_raw) {
        Ok(bytes) if looks_like_datastar_js(&bytes) => Ok(bytes),
        Ok(_) => Err(DatastarError::CorruptedBundle {
            tag: tag.to_string(),
        }),
        Err(e) => Err(DatastarError::Download {
            tag: tag.to_string(),
            message: e.to_string(),
        }),
    }
}

#[cfg(feature = "fetch")]
fn http_get_bytes(url: &str) -> std::result::Result<Vec<u8>, String> {
    use std::io::Read;
    let response = ureq::get(url)
        .header("User-Agent", "rocci")
        .call()
        .map_err(|e| e.to_string())?;
    let mut reader = response.into_body().into_reader().take(2 * 1024 * 1024);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}

#[cfg(feature = "fetch")]
pub fn stage_into(assets_dir: &Path, version: &str) -> Result<PathBuf> {
    let cached = ensure_cached(version)?;
    fs::create_dir_all(assets_dir).map_err(|source| DatastarError::CreateDir {
        path: assets_dir.to_path_buf(),
        source,
    })?;
    let dest = assets_dir.join("datastar.js");
    copy_if_changed(&cached, &dest)?;
    Ok(dest)
}
