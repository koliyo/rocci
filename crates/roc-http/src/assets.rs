use std::{
    borrow::Cow,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use roc_core::{Error, Result};

#[derive(Clone, Debug)]
pub struct Asset {
    pub content_type: Cow<'static, str>,
    pub bytes: Cow<'static, [u8]>,
}

#[derive(Clone, Debug, Default)]
pub struct AssetMap {
    files: HashMap<String, Asset>,
}

impl AssetMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        path: impl Into<String>,
        content_type: impl Into<Cow<'static, str>>,
        bytes: impl Into<Cow<'static, [u8]>>,
    ) -> &mut Self {
        let path = normalize_asset_path(&path.into());
        self.files.insert(
            path,
            Asset {
                content_type: content_type.into(),
                bytes: bytes.into(),
            },
        );
        self
    }

    pub fn get(&self, path: &str) -> Option<&Asset> {
        self.files.get(&normalize_asset_path(path))
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn from_directory(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let mut assets = Self::new();
        collect_assets(root, root, &mut assets)?;
        Ok(assets)
    }
}

#[derive(Clone, Debug)]
pub enum AssetSource {
    Memory(AssetMap),
    Directory(PathBuf),
}

impl AssetSource {
    pub fn memory(map: AssetMap) -> Self {
        Self::Memory(map)
    }

    pub fn directory(path: impl Into<PathBuf>) -> Self {
        Self::Directory(path.into())
    }

    pub fn get(&self, path: &str) -> Result<Option<Asset>> {
        let path = normalize_asset_path(path);
        if path.is_empty() || path.contains("..") {
            return Ok(None);
        }
        match self {
            Self::Memory(map) => Ok(map.get(&path).cloned()),
            Self::Directory(root) => read_file(root, &path),
        }
    }
}

impl From<AssetMap> for AssetSource {
    fn from(map: AssetMap) -> Self {
        Self::Memory(map)
    }
}

fn collect_assets(root: &Path, dir: &Path, assets: &mut AssetMap) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_assets(root, &path, assets)?;
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(Error::backend)?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = fs::read(&path)?;
        assets.insert(relative, content_type_for(&path), bytes);
    }
    Ok(())
}

fn read_file(root: &Path, path: &str) -> Result<Option<Asset>> {
    let mut joined = PathBuf::from(root);
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return Ok(None);
        }
        joined.push(segment);
    }
    if !joined.starts_with(root) || !joined.is_file() {
        return Ok(None);
    }
    Ok(Some(Asset {
        content_type: content_type_for(&joined),
        bytes: fs::read(joined)?.into(),
    }))
}

fn content_type_for(path: &Path) -> Cow<'static, str> {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "css" => "text/css; charset=utf-8".into(),
        "js" | "mjs" => "text/javascript; charset=utf-8".into(),
        "html" => "text/html; charset=utf-8".into(),
        "svg" => "image/svg+xml".into(),
        "png" => "image/png".into(),
        "jpg" | "jpeg" => "image/jpeg".into(),
        "woff2" => "font/woff2".into(),
        "json" => "application/json".into(),
        _ => "application/octet-stream".into(),
    }
}

fn normalize_asset_path(path: &str) -> String {
    path.trim_start_matches('/').replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_assets_are_looked_up_without_a_leading_slash() {
        let mut map = AssetMap::new();
        map.insert("app.css", "text/css", b"body{}" as &[u8]);
        assert!(map.get("/app.css").is_some());
        assert!(AssetSource::from(map).get("../app.css").unwrap().is_none());
    }
}
