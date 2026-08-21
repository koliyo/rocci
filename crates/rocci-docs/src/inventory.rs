use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::catalog::{AppEntry, DocsError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishedFile {
    pub relative: String,
    pub absolute: PathBuf,
}

const SKIP_DIRS: &[&str] = &["generated", "target", "dist", ".git"];

pub fn inventory_app(root: &Path, app: &AppEntry) -> Result<Vec<PublishedFile>, DocsError> {
    let dir = root.join(&app.path);
    let mut files = Vec::new();
    walk(&dir, &dir, app, &mut files)?;
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn is_published_rel(relative: &str, extra: &[String]) -> bool {
    if extra.iter().any(|path| path == relative) {
        return !is_excluded_name(relative);
    }
    let path = Path::new(relative);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if is_excluded_name(name) {
        return false;
    }
    if relative.split('/').any(|part| SKIP_DIRS.contains(&part)) {
        return false;
    }
    if is_asset_rel(relative) {
        return true;
    }
    if name == "rocci.toml" {
        return true;
    }
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rocci" | "roc")
    )
}

fn walk(
    app_dir: &Path,
    dir: &Path,
    app: &AppEntry,
    out: &mut Vec<PublishedFile>,
) -> Result<(), DocsError> {
    let entries = fs::read_dir(dir).map_err(|source| DocsError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| DocsError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|source| DocsError::Io {
            path: path.clone(),
            source,
        })?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk(app_dir, &path, app, out)?;
            continue;
        }
        if !ft.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(app_dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if !is_published_rel(&relative, &app.files) {
            continue;
        }
        if is_empty_keeper(&path, &relative) {
            continue;
        }
        out.push(PublishedFile {
            relative,
            absolute: path,
        });
    }
    Ok(())
}

fn is_asset_rel(relative: &str) -> bool {
    relative == "assets" || relative.starts_with("assets/")
}

fn is_excluded_name(name: &str) -> bool {
    if name == ".gitkeep"
        || name == "README.md"
        || name.ends_with(".rocdown")
        || name.ends_with(".db")
        || name.ends_with(".db-wal")
        || name.ends_with(".db-shm")
        || name.ends_with(".sqlite")
        || name.ends_with(".sqlite3")
        || name.ends_with('~')
        || name.ends_with(".swp")
        || name.ends_with(".swo")
    {
        return true;
    }
    name.starts_with(".#")
}

fn is_empty_keeper(path: &Path, relative: &str) -> bool {
    if !is_asset_rel(relative) {
        return false;
    }
    fs::metadata(path).is_ok_and(|meta| meta.len() == 0)
}
