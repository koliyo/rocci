use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocsError {
    #[error("failed to read catalog `{}`: {source}", .path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid catalog `{}`: {source}", .path.display())]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Hosting {
    Docs,
    Live,
}

impl Hosting {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Live => "live",
        }
    }

    pub fn public_label(self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Live => "planned live",
        }
    }
}

fn default_site() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppEntry {
    pub id: String,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub entry: String,
    pub hosting: Hosting,
    #[serde(default = "default_site")]
    pub site: bool,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub complexity: String,
    #[serde(default)]
    pub persistence: String,
    #[serde(default)]
    pub support: String,
    #[serde(default)]
    pub live_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub path: PathBuf,
    pub root: PathBuf,
    pub apps: Vec<AppEntry>,
}

#[derive(Deserialize)]
struct CatalogFile {
    #[serde(rename = "app")]
    apps: Vec<AppEntry>,
}

pub fn load_catalog(path: &Path) -> Result<Catalog, DocsError> {
    let text = fs::read_to_string(path).map_err(|source| DocsError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let parsed: CatalogFile = toml::from_str(&text).map_err(|source| DocsError::Toml {
        path: path.to_path_buf(),
        source,
    })?;
    let root = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    validate(&parsed.apps, &root)?;
    Ok(Catalog {
        path: path.to_path_buf(),
        root,
        apps: parsed.apps,
    })
}

fn validate(apps: &[AppEntry], root: &Path) -> Result<(), DocsError> {
    let mut ids = HashSet::new();
    for app in apps {
        if app.id.is_empty()
            || !app
                .id
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return Err(DocsError::Message(format!(
                "invalid app id `{}`: use lowercase slugs",
                app.id
            )));
        }
        if !ids.insert(app.id.clone()) {
            return Err(DocsError::Message(format!("duplicate app id `{}`", app.id)));
        }
        if app.hosting == Hosting::Live && !app.site {
            return Err(DocsError::Message(format!(
                "app `{}` hosting = \"live\" requires site = true",
                app.id
            )));
        }
        if app.path.is_empty()
            || Path::new(&app.path).is_absolute()
            || app.path.split('/').any(|part| part == "..")
        {
            return Err(DocsError::Message(format!(
                "app `{}` path must be catalog-relative without `..`",
                app.id
            )));
        }
        let dir = root.join(&app.path);
        if !dir.is_dir() {
            return Err(DocsError::Message(format!(
                "app `{}` path `{}` does not exist",
                app.id,
                dir.display()
            )));
        }
        if app.entry != "." {
            let entry = dir.join(&app.entry);
            if !entry.is_file() {
                return Err(DocsError::Message(format!(
                    "app `{}` entry `{}` is missing",
                    app.id,
                    entry.display()
                )));
            }
        }
        let docs = dir.join("index.rocdown");
        if !docs.is_file() {
            return Err(DocsError::Message(format!(
                "app `{}` is missing index.rocdown",
                app.id
            )));
        }
    }
    Ok(())
}
