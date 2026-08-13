use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

const DEFAULT_CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-eval'; connect-src 'self'; img-src 'self' data:; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'";

/// Runtime configuration for windows, HTTP, security, assets, and development.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default = "default_windows")]
    pub windows: Vec<WindowConfig>,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub assets: AssetConfig,
    #[serde(default)]
    pub development: DevelopmentConfig,
    #[serde(default)]
    pub bundle: BundleConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    #[serde(default = "default_app_name")]
    pub name: String,
    #[serde(default = "default_identifier")]
    pub identifier: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindowConfig {
    pub label: String,
    #[serde(default = "default_app_name")]
    pub title: String,
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_width")]
    pub width: f64,
    #[serde(default = "default_height")]
    pub height: f64,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    #[serde(default = "default_true")]
    pub visible: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HttpConfig {
    #[serde(default = "default_http_host")]
    pub host: String,
    #[serde(default)]
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub csp: Option<String>,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetConfig {
    pub directory: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub embed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DevelopmentConfig {
    pub frontend_url: Option<String>,
    pub backend_url: Option<String>,
    #[serde(default = "default_true")]
    pub reload: bool,
    #[serde(default = "default_true")]
    pub devtools: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleConfig {
    pub identifier: Option<String>,
    pub package: Option<String>,
    pub binary: Option<String>,
    pub macos_plist: Option<PathBuf>,
    #[serde(default)]
    pub resources: Vec<BundleResource>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleResource {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            windows: default_windows(),
            http: HttpConfig::default(),
            security: SecurityConfig::default(),
            assets: AssetConfig::default(),
            development: DevelopmentConfig::default(),
            bundle: BundleConfig::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            identifier: default_identifier(),
            version: None,
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            label: "main".into(),
            title: default_app_name(),
            url: default_url(),
            width: default_width(),
            height: default_height(),
            min_width: Some(720.0),
            min_height: Some(560.0),
            visible: true,
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: default_http_host(),
            port: 0,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            csp: Some(DEFAULT_CSP.into()),
            allowed_origins: Vec::new(),
        }
    }
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            directory: None,
            embed: true,
        }
    }
}

impl Default for DevelopmentConfig {
    fn default() -> Self {
        Self {
            frontend_url: None,
            backend_url: None,
            reload: true,
            devtools: true,
        }
    }
}

impl Config {
    pub fn from_toml(source: &str) -> Result<Self> {
        let config: Self = toml::from_str(source).map_err(Error::config)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|error| {
            Error::config(format!("failed to read {}: {error}", path.display()))
        })?;
        Self::from_toml(&source)
            .map_err(|error| Error::config(format!("{}: {error}", path.display())))
    }

    pub fn load() -> Result<Self> {
        if let Ok(path) = env::var("ROC_CONFIG") {
            return Self::from_file(path);
        }
        if let Some(path) = find_config() {
            return Self::from_file(path);
        }
        Err(Error::config(
            "no roc.toml found; set ROC_CONFIG or run from a project directory",
        ))
    }

    pub fn validate(&self) -> Result<()> {
        if self.app.name.trim().is_empty() {
            return Err(Error::config("app.name must not be empty"));
        }
        validate_identifier(&self.app.identifier)?;
        if self.windows.is_empty() {
            return Err(Error::config("at least one [[windows]] entry is required"));
        }

        let mut labels = Vec::new();
        for window in &self.windows {
            validate_window(window)?;
            if labels.iter().any(|label| label == &window.label) {
                return Err(Error::config(format!(
                    "duplicate window label {}",
                    window.label
                )));
            }
            labels.push(window.label.clone());
        }

        if !is_loopback_host(&self.http.host) {
            return Err(Error::config(
                "http.host must be a loopback address (127.0.0.1 or localhost)",
            ));
        }

        if let Some(csp) = &self.security.csp
            && csp.trim().is_empty()
        {
            return Err(Error::config("security.csp must not be empty when set"));
        }

        validate_optional_loopback_url(
            "development.frontend_url",
            self.development.frontend_url.as_deref(),
        )?;
        validate_optional_loopback_url(
            "development.backend_url",
            self.development.backend_url.as_deref(),
        )?;

        for origin in &self.security.allowed_origins {
            validate_http_url("security.allowed_origins", origin)?;
        }

        for resource in &self.bundle.resources {
            if resource.from.as_os_str().is_empty() || resource.to.as_os_str().is_empty() {
                return Err(Error::config(
                    "bundle.resources entries need both `from` and `to`",
                ));
            }
        }

        Ok(())
    }

    pub fn csp(&self) -> &str {
        self.security.csp.as_deref().unwrap_or(DEFAULT_CSP)
    }

    pub fn window(&self, label: &str) -> Result<&WindowConfig> {
        self.windows
            .iter()
            .find(|window| window.label == label)
            .ok_or_else(|| Error::WindowNotFound(label.into()))
    }
}

fn validate_window(window: &WindowConfig) -> Result<()> {
    if !is_label(&window.label) {
        return Err(Error::config(format!(
            "window label {:?} must match [A-Za-z0-9_-]+",
            window.label
        )));
    }
    if window.title.trim().is_empty() {
        return Err(Error::config(format!(
            "window {} title must not be empty",
            window.label
        )));
    }
    if window.url.trim().is_empty() {
        return Err(Error::config(format!(
            "window {} url must not be empty",
            window.label
        )));
    }
    if window.width <= 0.0 || window.height <= 0.0 {
        return Err(Error::config(format!(
            "window {} size must be positive",
            window.label
        )));
    }
    if let Some(min_width) = window.min_width
        && min_width > window.width
    {
        return Err(Error::config(format!(
            "window {} min_width cannot exceed width",
            window.label
        )));
    }
    if let Some(min_height) = window.min_height
        && min_height > window.height
    {
        return Err(Error::config(format!(
            "window {} min_height cannot exceed height",
            window.label
        )));
    }
    Ok(())
}

fn validate_identifier(identifier: &str) -> Result<()> {
    if identifier.len() < 3
        || !identifier.contains('.')
        || identifier.starts_with('.')
        || identifier.ends_with('.')
        || identifier.contains("..")
        || !identifier
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-')
    {
        return Err(Error::config(
            "app.identifier must be a reverse-DNS string such as dev.roc.app",
        ));
    }
    Ok(())
}

fn validate_optional_loopback_url(field: &str, value: Option<&str>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    validate_http_url(field, value)?;
    let host = value
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split(['/', ':'])
        .next()
        .unwrap_or_default();
    if !is_loopback_host(host) {
        return Err(Error::config(format!("{field} must use a loopback host")));
    }
    Ok(())
}

fn validate_http_url(field: &str, value: &str) -> Result<()> {
    if !(value.starts_with("http://") || value.starts_with("https://")) {
        return Err(Error::config(format!("{field} must be an http(s) URL")));
    }
    Ok(())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost")
}

fn is_label(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

fn find_config() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join("roc.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = dir.parent()?.to_path_buf();
    }
}

fn default_windows() -> Vec<WindowConfig> {
    vec![WindowConfig::default()]
}

fn default_app_name() -> String {
    "Roc".into()
}

fn default_identifier() -> String {
    "dev.roc.app".into()
}

fn default_url() -> String {
    "/".into()
}

fn default_width() -> f64 {
    1040.0
}

fn default_height() -> f64 {
    760.0
}

fn default_http_host() -> String {
    "127.0.0.1".into()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        Config::default().validate().unwrap();
    }

    #[test]
    fn parses_a_multi_window_profile() {
        let config = Config::from_toml(
            r#"
            [app]
            name = "Demo"
            identifier = "dev.roc.demo"

            [[windows]]
            label = "main"
            title = "Main"
            url = "/"

            [[windows]]
            label = "htmx"
            title = "htmx"
            url = "/htmx"
            width = 800
            height = 600
            visible = false

            [development]
            frontend_url = "http://127.0.0.1:5173"
            reload = true
            "#,
        )
        .unwrap();
        assert_eq!(config.windows.len(), 2);
        assert!(!config.window("htmx").unwrap().visible);
        assert_eq!(
            config.development.frontend_url.as_deref(),
            Some("http://127.0.0.1:5173")
        );
    }

    #[test]
    fn rejects_duplicate_window_labels() {
        let error = Config::from_toml(
            r#"
            [app]
            identifier = "dev.roc.demo"
            [[windows]]
            label = "main"
            [[windows]]
            label = "main"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate window label"));
    }

    #[test]
    fn rejects_non_loopback_http_hosts() {
        let error = Config::from_toml(
            r#"
            [app]
            identifier = "dev.roc.demo"
            [http]
            host = "0.0.0.0"
            "#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("loopback"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = Config::from_toml("[app]\nnope = 1\n").unwrap_err();
        assert!(error.to_string().contains("unknown"));
    }
}
