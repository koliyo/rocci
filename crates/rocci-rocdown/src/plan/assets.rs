use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::config::SiteConfig;

pub(crate) const HASH_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedAsset {
    pub kind: &'static str,
    pub logical_path: String,
    pub hashed_url: String,
    pub output_path: String,
    pub bytes: Vec<u8>,
}

pub(crate) fn hash_site_assets(root: &Path, config: &SiteConfig) -> Result<Vec<PlannedAsset>> {
    if config.build.assets.trim().is_empty() {
        return Ok(Vec::new());
    }
    let source = root.join(&config.build.assets);
    if !source.exists() {
        return Ok(Vec::new());
    }
    if !source.is_dir() {
        bail!(
            "configured assets path {} is not a directory",
            source.display()
        );
    }
    let mut files = Vec::new();
    collect_asset_files(&source, Path::new(""), &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files
        .into_iter()
        .map(|(relative, bytes)| hashed_asset(&relative, &bytes))
        .collect())
}

fn collect_asset_files(
    dir: &Path,
    prefix: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        if name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let from = entry.path();
        let relative = prefix.join(&name);
        if from.is_dir() {
            collect_asset_files(&from, &relative, files)?;
        } else {
            let bytes = std::fs::read(&from)
                .with_context(|| format!("failed to read {}", from.display()))?;
            files.push((relative.to_string_lossy().replace('\\', "/"), bytes));
        }
    }
    Ok(())
}

pub(crate) fn datastar_js_bytes() -> Result<Vec<u8>> {
    let path =
        rocci_cli::datastar_asset::ensure_cached(rocci_cli::datastar_asset::DEFAULT_VERSION)?;
    std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))
}

pub(crate) fn hashed_asset(relative: &str, bytes: &[u8]) -> PlannedAsset {
    let hash = hex_sha256(bytes);
    let hashed_name = hashed_file_name(relative, &hash);
    let kind = if Path::new(relative)
        .file_stem()
        .is_some_and(|stem| stem == "theme")
        && Path::new(relative)
            .extension()
            .is_some_and(|ext| ext == "css")
    {
        "stylesheet"
    } else {
        "asset"
    };
    PlannedAsset {
        kind,
        logical_path: format!("/assets/{relative}"),
        hashed_url: format!("/assets/{hashed_name}"),
        output_path: format!("assets/{hashed_name}"),
        bytes: bytes.to_vec(),
    }
}

fn hashed_file_name(relative: &str, hash: &str) -> String {
    let path = Path::new(relative);
    let hash = &hash[..HASH_LEN];
    let name = match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => format!(
            "{}.{hash}.{}",
            stem.to_string_lossy(),
            ext.to_string_lossy()
        ),
        (Some(stem), None) => format!("{}.{hash}", stem.to_string_lossy()),
        _ => format!("asset.{hash}"),
    };
    match path.parent().filter(|parent| *parent != Path::new("")) {
        Some(parent) => format!("{}/{name}", parent.to_string_lossy()),
        None => name,
    }
}

pub(crate) fn hex_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn rewrite_map(assets: &[PlannedAsset]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for asset in assets {
        if asset.kind == "stylesheet" {
            continue;
        }
        map.insert(asset.logical_path.clone(), asset.hashed_url.clone());
        if let Some(rest) = asset.logical_path.strip_prefix('/') {
            map.insert(rest.to_string(), asset.hashed_url.clone());
        }
    }
    map
}

pub(crate) fn rewrite_urls(text: &str, map: &BTreeMap<String, String>) -> String {
    crate::docs::rewrite_urls(text, map)
}
