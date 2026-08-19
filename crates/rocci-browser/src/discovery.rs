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

#[derive(Clone, Debug, Default, Deserialize)]
struct RepoLocalFile {
    #[serde(default)]
    plugin: Vec<PluginSpec>,
}

pub fn load_plugin_manifest(path: &Path) -> Result<PluginSpec> {
    let raw = fs::read_to_string(path)?;
    toml::from_str(&raw).map_err(|error| crate::Error::message(error))
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

    let repo_local = paths.repo_local_path();
    if repo_local.is_file() {
        match fs::read_to_string(&repo_local) {
            Ok(raw) => match toml::from_str::<RepoLocalFile>(&raw) {
                Ok(parsed) => {
                    for spec in parsed.plugin {
                        upsert(&mut plugins, spec);
                    }
                }
                Err(error) => warnings.push(format!("{}: {error}", repo_local.display())),
            },
            Err(error) => warnings.push(format!("{}: {error}", repo_local.display())),
        }
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

pub fn resolve_bin(bin: &str) -> Option<PathBuf> {
    let path = Path::new(bin);
    if path.is_absolute() || bin.contains('/') || bin.contains('\\') {
        return path.exists().then(|| path.to_path_buf());
    }
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var).find_map(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file().then_some(candidate)
    })
}
