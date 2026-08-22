use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::frontmatter::{parse_yaml_mapping, split_frontmatter};
use crate::{absolute, relative_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewTarget {
    pub root: PathBuf,
    pub open_path: String,
}

impl PreviewTarget {
    pub fn bundle(root: PathBuf) -> Self {
        Self {
            root,
            open_path: "/".into(),
        }
    }

    pub fn concept(root: PathBuf, id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            root,
            open_path: format!("/{id}/"),
        }
    }
}

pub fn resolve_preview_path(path: &Path) -> Result<PreviewTarget> {
    let path = absolute(path)?;
    if path.is_dir() {
        let root = fs::canonicalize(&path)
            .with_context(|| format!("failed to resolve knowledge root {}", path.display()))?;
        return Ok(PreviewTarget::bundle(root));
    }
    if !path.is_file() {
        bail!("no such knowledge path: {}", path.display());
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        bail!(
            "unsupported file for `rocci-okf view`: {}; expected a knowledge bundle directory or a .md file",
            path.display()
        );
    }
    let canonical =
        fs::canonicalize(&path).with_context(|| format!("failed to resolve {}", path.display()))?;
    let Some(root) = enclosing_bundle_root(&canonical)? else {
        bail!(
            "{} is not inside an OKF bundle; pass a bundle directory or a Markdown file under one",
            path.display()
        );
    };
    let relative = relative_path(&root, &canonical);
    match relative.rsplit('/').next() {
        Some("index.md") if relative == "index.md" => Ok(PreviewTarget::bundle(root)),
        Some("index.md") => bail!(
            "{} is a collection index, not a concept; preview the bundle with `rocci-okf view {}`",
            path.display(),
            root.display()
        ),
        Some("log.md") => bail!(
            "{} is a knowledge log, not a concept; preview the bundle with `rocci-okf view {}`",
            path.display(),
            root.display()
        ),
        _ => {
            let id = relative.strip_suffix(".md").unwrap_or(&relative);
            Ok(PreviewTarget::concept(root, id))
        }
    }
}

fn enclosing_bundle_root(file: &Path) -> Result<Option<PathBuf>> {
    let mut dir = file.parent();
    while let Some(current) = dir {
        let index = current.join("index.md");
        if index.is_file() {
            let source = fs::read_to_string(&index)
                .with_context(|| format!("failed to read {}", index.display()))?;
            if is_bundle_root_index(&source) {
                let root = fs::canonicalize(current).with_context(|| {
                    format!("failed to resolve knowledge root {}", current.display())
                })?;
                return Ok(Some(root));
            }
        }
        dir = current.parent();
    }
    Ok(None)
}

fn is_bundle_root_index(source: &str) -> bool {
    let Ok(Some(frontmatter)) = split_frontmatter(source, false) else {
        return false;
    };
    let Ok(metadata) = parse_yaml_mapping(frontmatter.yaml.of(source)) else {
        return false;
    };
    metadata
        .get("okf_version")
        .and_then(Value::as_str)
        .is_some()
}
