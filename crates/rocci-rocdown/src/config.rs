use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "rocdown.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SiteConfig {
    pub site: SiteMeta,
    pub build: BuildConfig,
    #[serde(rename = "mount", default)]
    pub mounts: Vec<MountConfig>,
    #[serde(rename = "nav")]
    pub navigation: Vec<NavConfig>,
    #[serde(default)]
    pub snippets: SnippetsConfig,
    #[serde(default)]
    pub examples: ExamplesConfig,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub blocks: BlocksConfig,
    #[serde(skip)]
    pub sidebar_tree: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct MountConfig {
    pub source: String,
    pub prefix: String,
    #[serde(default)]
    pub layout: Option<String>,
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
    pub subtitle: String,
    pub footer: String,
    pub csp: String,
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
            subtitle: String::new(),
            footer: String::new(),
            csp: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BuildConfig {
    pub output: String,
    pub assets: String,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub host: Option<rocci_roc_host::HostChoice>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output: "dist".into(),
            assets: "assets".into(),
            theme: None,
            host: None,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct SnippetsConfig {
    pub roots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ExamplesConfig {
    pub timeout_ms: u64,
    pub allow_network: bool,
}

impl Default for ExamplesConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            allow_network: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct HttpConfig {
    pub service_origin: String,
    pub service: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BlocksConfig {
    pub pack: Option<String>,
    pub debug: bool,
    #[serde(rename = "override")]
    pub override_map: BTreeMap<String, String>,
}

pub fn load_config(root: &Path) -> Result<SiteConfig> {
    load_config_named(root, CONFIG_FILE)
}

fn load_config_named(root: &Path, filename: &str) -> Result<SiteConfig> {
    let path = root.join(filename);
    if !path.exists() {
        return Ok(SiteConfig::default());
    }
    let source =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut config: SiteConfig =
        toml::from_str(&source).with_context(|| format!("failed to parse {}", path.display()))?;
    validate(&config, &path)?;
    config.site.base_url = config.site.base_url.trim_end_matches('/').to_string();
    config.http.service_origin = config
        .http
        .service_origin
        .trim()
        .trim_end_matches('/')
        .to_string();
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
    if let Some(theme) = &config.build.theme
        && (theme.trim().is_empty() || theme.contains('\0') || Path::new(theme).is_absolute())
    {
        bail!(
            "build.theme `{theme}` must be a relative path in {}",
            path.display()
        );
    }
    if let Some(pack) = &config.blocks.pack
        && (pack.trim().is_empty() || pack.contains('\0') || Path::new(pack).is_absolute())
    {
        bail!(
            "blocks.pack `{pack}` must be a relative path in {}",
            path.display()
        );
    }
    for (kind, component) in &config.blocks.override_map {
        let valid = crate::registry::lookup(kind).is_some_and(|spec| spec.paints_as_widget());
        if !valid {
            bail!(
                "blocks.override unknown kind `{kind}` in {}",
                path.display()
            );
        }
        if component.trim().is_empty() {
            bail!(
                "blocks.override `{kind}` must name a component in {}",
                path.display()
            );
        }
    }
    for (index, mount) in config.mounts.iter().enumerate() {
        if mount.source.trim().is_empty()
            || mount.source.contains('\0')
            || Path::new(&mount.source).is_absolute()
        {
            bail!(
                "mount[{}] source `{}` must be a valid relative path in {}",
                index + 1,
                mount.source,
                path.display()
            );
        }
        let prefix = mount.prefix.trim();
        if prefix.contains("..") || prefix.starts_with('/') || prefix.ends_with('/') {
            bail!(
                "mount[{}] prefix `{}` must not start/end with '/' or contain '..' in {}",
                index + 1,
                mount.prefix,
                path.display()
            );
        }
        if let Some(layout) = &mount.layout {
            const VALID_LAYOUTS: &[&str] = &[
                "home",
                "product",
                "section",
                "docs",
                "news-index",
                "news-post",
                "plain",
                "not-found",
            ];
            if !VALID_LAYOUTS.contains(&layout.trim().trim_matches('"')) {
                bail!(
                    "mount[{}] layout `{}` must be one of {} in {}",
                    index + 1,
                    layout,
                    VALID_LAYOUTS.join(", "),
                    path.display()
                );
            }
        }
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
    for (index, root) in config.snippets.roots.iter().enumerate() {
        if root.trim().is_empty() || root.contains('\0') || Path::new(root).is_absolute() {
            bail!(
                "snippets.roots[{}] `{}` must be a relative path in {}",
                index + 1,
                root,
                path.display()
            );
        }
    }
    if !config.http.service_origin.is_empty()
        && !(config.http.service_origin.starts_with("https://")
            || config.http.service_origin.starts_with("http://"))
    {
        bail!(
            "http.service_origin must start with http:// or https:// in {}",
            path.display()
        );
    }
    if !config.http.service.is_empty() {
        let service = config.http.service.trim();
        if service.is_empty()
            || service.contains('\0')
            || Path::new(service).is_absolute()
            || !service.ends_with(".rocci")
        {
            bail!(
                "http.service `{service}` must be a relative .rocci path in {}",
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
        let path = env::temp_dir().join(format!("rocdown-config-{}-{name}", std::process::id()));
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
    fn reads_subtitle_footer_and_csp() {
        let root = temp("chrome");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"
subtitle = "Interface tools"
footer = "Experimental."
csp = "default-src 'self'"
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.site.subtitle, "Interface tools");
        assert_eq!(config.site.footer, "Experimental.");
        assert_eq!(config.site.csp, "default-src 'self'");
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
    fn reads_mounts() {
        let root = temp("mounts");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[[mount]]
source = "../docs"
prefix = "docs"
layout = "docs"

[[nav]]
label = "Docs"
items = ["docs/index"]
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.mounts.len(), 1);
        assert_eq!(config.mounts[0].source, "../docs");
        assert_eq!(config.mounts[0].prefix, "docs");
        assert_eq!(config.mounts[0].layout.as_deref(), Some("docs"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_mounts() {
        let root = temp("invalid-mount");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[[mount]]
source = "/absolute/path"
prefix = "docs"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("must be a valid relative path"), "{err}");

        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[[mount]]
source = "../docs"
prefix = "/docs/"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("must not start/end with '/'"), "{err}");

        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[[mount]]
source = "../docs"
prefix = "docs"
layout = "invalid_layout"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("must be one of"), "{err}");
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

    #[test]
    fn reads_blocks_pack_debug_and_override() {
        let root = temp("blocks");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[blocks]
pack = "theme/widgets"
debug = true

[blocks.override]
note = "Callout"
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.blocks.pack.as_deref(), Some("theme/widgets"));
        assert!(config.blocks.debug);
        assert_eq!(
            config.blocks.override_map.get("note").map(String::as_str),
            Some("Callout")
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_unknown_blocks_keys_and_override_kinds() {
        let root = temp("blocks-unknown");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[blocks]
painters = "theme/Blocks.rocci"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("failed to parse"), "{err}");

        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[blocks.override]
notee = "Note"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("unknown kind `notee`"), "{err}");

        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[blocks]
pack = "/absolute/pack"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("must be a relative path"), "{err}");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reads_http_service_origin_and_sibling_service() {
        let root = temp("http");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[http]
service_origin = "https://islands.example.com/"
service = "theme/Islands.rocci"
"#,
        )
        .unwrap();
        let config = load_config(&root).unwrap();
        assert_eq!(config.http.service_origin, "https://islands.example.com");
        assert_eq!(config.http.service, "theme/Islands.rocci");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_invalid_http_service_origin() {
        let root = temp("http-bad");
        fs::write(
            root.join(CONFIG_FILE),
            r#"
[site]
title = "Rocci"

[http]
service_origin = "islands.example.com"
"#,
        )
        .unwrap();
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("http.service_origin"), "{err}");
        let _ = fs::remove_dir_all(root);
    }
}
