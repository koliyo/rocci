use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, paths::Paths};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSpec {
    pub id: String,
    pub bin: String,
    #[serde(default)]
    pub argv: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoProject {
    pub id: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct RepoLocalFile {
    #[serde(default)]
    pub plugin: Vec<PluginSpec>,
    #[serde(default)]
    pub project: Vec<RepoProject>,
}

pub fn load_plugin_manifest(path: &Path) -> Result<PluginSpec> {
    let raw = fs::read_to_string(path)?;
    toml::from_str(&raw).map_err(|error| crate::Error::message(error))
}

pub fn load_repo_local(paths: &Paths) -> Result<Option<RepoLocalFile>> {
    let repo_local = paths.repo_local_path();
    if !repo_local.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&repo_local)?;
    let parsed = toml::from_str::<RepoLocalFile>(&raw)
        .map_err(|error| crate::Error::message(format!("{}: {error}", repo_local.display())))?;
    Ok(Some(parsed))
}

pub fn discover_plugins(paths: &Paths) -> Result<(Vec<PluginSpec>, Vec<String>)> {
    let mut plugins: Vec<PluginSpec> = Vec::new();
    let mut warnings = Vec::new();

    let plugins_dir = paths.plugins_dir();
    if plugins_dir.is_dir() {
        let mut files: Vec<PathBuf> = fs::read_dir(&plugins_dir)?
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect();
        files.sort();
        for file in files {
            match load_plugin_manifest(&file) {
                Ok(spec) => upsert(&mut plugins, spec),
                Err(error) => warnings.push(format!("{}: {error}", file.display())),
            }
        }
    }

    match load_repo_local(paths) {
        Ok(Some(parsed)) => {
            for spec in parsed.plugin {
                upsert(&mut plugins, spec);
            }
        }
        Ok(None) => {}
        Err(error) => warnings.push(error.to_string()),
    }

    if let Some(env_spec) = &paths.plugins_env {
        for part in env_spec.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let spec = match part.split_once('=') {
                Some((id, bin)) => PluginSpec {
                    id: id.trim().to_string(),
                    bin: bin.trim().to_string(),
                    argv: Vec::new(),
                },
                None => PluginSpec {
                    id: part.to_string(),
                    bin: part.to_string(),
                    argv: Vec::new(),
                },
            };
            upsert(&mut plugins, spec);
        }
    }

    Ok((plugins, warnings))
}

fn upsert(plugins: &mut Vec<PluginSpec>, spec: PluginSpec) {
    match plugins.iter_mut().find(|item| item.id == spec.id) {
        Some(existing) => *existing = spec,
        None => plugins.push(spec),
    }
}

pub fn resolve_bin(bin: &str, relative_to: Option<&Path>) -> Option<PathBuf> {
    let path = Path::new(bin);
    if path.is_absolute() {
        return path.exists().then(|| path.to_path_buf());
    }
    if bin.contains('/') || bin.contains('\\') {
        let candidate = match relative_to {
            Some(root) => root.join(path),
            None => path.to_path_buf(),
        };
        return candidate.is_file().then_some(candidate);
    }
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var).find_map(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file().then_some(candidate)
    })
}
