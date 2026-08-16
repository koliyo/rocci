use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "rocs.toml";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SiteConfig {
    pub site: SiteMeta,
    pub build: BuildConfig,
    #[serde(rename = "nav")]
    pub navigation: Vec<NavConfig>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            site: SiteMeta::default(),
            build: BuildConfig::default(),
            navigation: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SiteMeta {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub language: String,
    pub repository: String,
    pub social_image: String,
}

impl Default for SiteMeta {
    fn default() -> Self {
        Self {
            title: "Documentation".into(),
            description: String::new(),
            base_url: String::new(),
            language: "en".into(),
            repository: String::new(),
            social_image: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BuildConfig {
    pub output: String,
    pub assets: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output: "dist".into(),
            assets: "assets".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct NavConfig {
    pub label: String,
    pub items: Vec<String>,
    pub directory: Option<String>,
}

pub fn load_config(root: &Path) -> Result<SiteConfig> {
    let path = root.join(CONFIG_FILE);
    if !path.exists() {
        return Ok(SiteConfig::default());
    }
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut config: SiteConfig =
        toml::from_str(&source).with_context(|| format!("failed to parse {}", path.display()))?;
    validate(&config, &path)?;
    config.site.base_url = config.site.base_url.trim_end_matches('/').to_string();
    Ok(config)
}

fn validate(config: &SiteConfig, path: &Path) -> Result<()> {
    if config.site.title.trim().is_empty() {
        bail!("site.title must not be empty in {}", path.display());
    }
    if config.site.language.trim().is_empty() {
        bail!("site.language must not be empty in {}", path.display());
    }
    if !config.site.base_url.is_empty()
        && !(config.site.base_url.starts_with("https://")
            || config.site.base_url.starts_with("http://"))
    {
        bail!(
            "site.base_url must start with http:// or https:// in {}",
            path.display()
        );
    }
    if config.build.output.trim().is_empty() {
        bail!("build.output must not be empty in {}", path.display());
    }
    for (index, section) in config.navigation.iter().enumerate() {
        if section.label.trim().is_empty() {
            bail!(
                "nav section {} has an empty label in {}",
                index + 1,
                path.display()
            );
        }
        let directory = section
            .directory
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if section.items.is_empty() && directory.is_none() {
            bail!(
                "nav section `{}` has no items or directory in {}",
                section.label,
                path.display()
            );
        }
        if let Some(directory) = directory
            && (directory.contains("..") || directory.starts_with('/'))
        {
            bail!(
                "nav section `{}` has an invalid directory `{directory}` in {}",
                section.label,
                path.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};

    fn temp(name: &str) -> std::path::PathBuf {
        let path = env::temp_dir().join(format!("rocs-config-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn defaults_without_a_file() {
        let root = temp("default");
        let config = load_config(&root).unwrap();
        assert_eq!(config.site.title, "Documentation");
        assert_eq!(config.build.output, "dist");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reads_site_build_and_navigation() {
        let root = temp("full");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"
base_url = "https://rocci.dev/"

[build]
output = "../dist/docs"

[[nav]]
label = "Start"
items = ["index", "quickstart"]
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.site.base_url, "https://rocci.dev");
        assert_eq!(config.build.output, "../dist/docs");
        assert_eq!(config.navigation[0].items[1], "quickstart");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reads_directory_navigation() {
        let root = temp("dir-nav");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[[nav]]
label = "Guides"
directory = "guides"
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.navigation[0].directory.as_deref(), Some("guides"));
        assert!(config.navigation[0].items.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unknown_keys() {
        let root = temp("unknown");
        fs::write(root.join(CONFIG_FILE), "[site]\ntitel = \"typo\"\n").unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("failed to parse"), "{err}");
        let _ = fs::remove_dir_all(root);
    }
}
