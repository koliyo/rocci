use std::path::{Path, PathBuf};

use crate::error::{Error, Result, unknown_theme};
use crate::scheme::ColorSchemePolicy;

pub const NONE_ID: &str = "none";
pub const PAPER_ID: &str = "paper";
pub const ROCCI_ID: &str = "rocci";
pub const DEFAULT_THEME_ID: &str = PAPER_ID;

const PAPER_CSS: &str = include_str!("themes/paper.css");
const ROCCI_CSS: &str = include_str!("themes/rocci.css");
const CHROME_CSS: &str = include_str!("themes/chrome.css");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeOptions {
    pub default_id: Option<String>,
    pub color_scheme: Option<ColorSchemePolicy>,
    pub source_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeOrigin {
    Builtin,
    Local,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTheme {
    pub id: String,
    pub name: String,
    pub origin: ThemeOrigin,
    pub path: Option<PathBuf>,
    pub policy: ColorSchemePolicy,
    pub css: String,
}

impl ResolvedTheme {
    pub fn disabled(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: "none".into(),
            origin: ThemeOrigin::None,
            path: None,
            policy: ColorSchemePolicy::Auto,
            css: String::new(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.origin == ThemeOrigin::None
    }
}

pub fn builtin_ids() -> Vec<&'static str> {
    vec![NONE_ID, PAPER_ID, ROCCI_ID]
}

pub fn discovered_ids() -> Vec<String> {
    let mut ids: Vec<String> = builtin_ids().into_iter().map(str::to_string).collect();
    if let Some(root) = themes_dir() {
        ids.extend(fs_theme_names(&root));
    }
    ids.sort();
    ids.dedup();
    ids
}

pub fn resolve(
    page_theme: Option<&str>,
    page_color_scheme: Option<&str>,
    options: &ThemeOptions,
) -> Result<ResolvedTheme> {
    let requested = page_theme
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .or(options
            .default_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty()))
        .unwrap_or(DEFAULT_THEME_ID);
    if canonical_id(requested) == NONE_ID {
        return Ok(ResolvedTheme::disabled(NONE_ID));
    }
    let mut resolved = resolve_id(requested, options.source_dir.as_deref())?;
    resolved.policy = match page_color_scheme {
        Some(value) => value.parse().map_err(Error::Config)?,
        None => options.color_scheme.unwrap_or(ColorSchemePolicy::Auto),
    };
    Ok(resolved)
}

pub fn resolve_id(id: &str, source_dir: Option<&Path>) -> Result<ResolvedTheme> {
    let trimmed = id.trim();
    if canonical_id(trimmed) == NONE_ID {
        return Ok(ResolvedTheme::disabled(NONE_ID));
    }
    if looks_like_path(trimmed) {
        return load_path(&expand_path(trimmed, source_dir), trimmed);
    }
    if let Some(builtin) = lookup_builtin(trimmed) {
        return Ok(builtin);
    }
    let mut searched = Vec::new();
    if let Some(root) = themes_dir() {
        searched.push(root.clone());
        if let Some(path) = named_theme_path(&root, trimmed) {
            return load_path(&path, trimmed);
        }
    }
    Err(unknown_theme(trimmed, &searched))
}

fn lookup_builtin(id: &str) -> Option<ResolvedTheme> {
    let name = canonical_id(id);
    let css = match name {
        PAPER_ID => PAPER_CSS,
        ROCCI_ID => ROCCI_CSS,
        _ => return None,
    };
    Some(ResolvedTheme {
        id: name.to_string(),
        name: name.to_string(),
        origin: ThemeOrigin::Builtin,
        path: None,
        policy: ColorSchemePolicy::Auto,
        css: compose_css(css),
    })
}

fn compose_css(theme: &str) -> String {
    format!("{}\n{}", theme.trim_end(), CHROME_CSS)
}

fn canonical_id(id: &str) -> &str {
    match id {
        "rocdown:paper" | "paper" => PAPER_ID,
        "rocdown:rocci" | "rocci" => ROCCI_ID,
        other => other,
    }
}

fn looks_like_path(value: &str) -> bool {
    value.starts_with('.')
        || value.starts_with('~')
        || value.contains('/')
        || value.contains('\\')
        || Path::new(value)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("css"))
}

fn expand_path(value: &str, source_dir: Option<&Path>) -> PathBuf {
    let path = expand_tilde(value);
    if path.is_absolute() {
        path
    } else if let Some(dir) = source_dir {
        dir.join(path)
    } else {
        path
    }
}

fn expand_tilde(value: &str) -> PathBuf {
    if let Some(rest) = value.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    if value == "~"
        && let Some(home) = home_dir()
    {
        return home;
    }
    PathBuf::from(value)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn themes_dir() -> Option<PathBuf> {
    Some(home_dir()?.join(".rocci").join("themes"))
}

fn named_theme_path(root: &Path, name: &str) -> Option<PathBuf> {
    let file = root.join(format!("{name}.css"));
    if file.is_file() {
        return Some(file);
    }
    let nested = root.join(name).join("theme.css");
    if nested.is_file() {
        return Some(nested);
    }
    None
}

fn fs_theme_names(root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "css") {
            if let Some(stem) = path.file_stem().and_then(|name| name.to_str()) {
                names.push(stem.to_string());
            }
        } else if path.is_dir()
            && path.join("theme.css").is_file()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            names.push(name.to_string());
        }
    }
    names
}

fn load_path(path: &Path, requested: &str) -> Result<ResolvedTheme> {
    let file = if path.is_dir() {
        let nested = path.join("theme.css");
        if nested.is_file() {
            nested
        } else {
            return Err(Error::Resolve(format!(
                "no theme.css in {}",
                path.display()
            )));
        }
    } else if path.is_file() {
        path.to_path_buf()
    } else {
        return Err(Error::Resolve(format!(
            "theme file not found: {}",
            path.display()
        )));
    };
    let source = std::fs::read_to_string(&file)?;
    let name = file
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| *name != "theme")
        .map(str::to_string)
        .or_else(|| {
            file.parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| requested.to_string());
    Ok(ResolvedTheme {
        id: name.clone(),
        name,
        origin: ThemeOrigin::Local,
        path: Some(file),
        policy: ColorSchemePolicy::Auto,
        css: compose_css(&source),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_paper() {
        let theme = resolve(None, None, &ThemeOptions::default()).unwrap();
        assert_eq!(theme.id, PAPER_ID);
        assert!(!theme.css.is_empty());
        assert!(theme.css.contains("--rd-color-bg:"));
        assert!(theme.css.contains("light-dark("));
        assert!(theme.css.contains(".rd-header-1"));
        assert!(theme.css.contains(".rd-shell"));
        assert!(
            theme.css.contains("@media (max-width: 48rem)"),
            "standalone TOC must stay visible in the 1040px default preview window"
        );
        assert!(theme.css.contains(".rd-document body"));
        assert!(theme.css.contains("--rd-chrome-top"));
        assert!(!theme.css.contains("max-width: 70rem"));
    }

    #[test]
    fn none_disables_css() {
        let theme = resolve(Some("none"), None, &ThemeOptions::default()).unwrap();
        assert!(theme.is_none());
        assert!(theme.css.is_empty());
    }

    #[test]
    fn page_theme_wins_and_aliases_resolve() {
        let options = ThemeOptions {
            default_id: Some(PAPER_ID.into()),
            ..Default::default()
        };
        let theme = resolve(Some("rocdown:rocci"), Some("dark"), &options).unwrap();
        assert_eq!(theme.id, ROCCI_ID);
        assert_eq!(theme.policy.as_str(), "dark");
        assert!(theme.css.contains("#48eda4"));
        assert!(theme.css.contains("light-dark("));
    }

    #[test]
    fn unknown_theme_is_an_error() {
        let err = resolve(Some("nope"), None, &ThemeOptions::default()).unwrap_err();
        assert!(err.to_string().contains("unknown theme `nope`"));
    }

    #[test]
    fn loads_css_file_from_path() {
        let dir = std::env::temp_dir().join("rocci-theme-path-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("custom.css");
        std::fs::write(&file, ".rd-document { --rd-color-accent: #ff00aa; }\n").unwrap();
        let theme = resolve_id(&file.to_string_lossy(), None).unwrap();
        assert_eq!(theme.origin, ThemeOrigin::Local);
        assert!(theme.css.contains("#ff00aa"));
        assert!(theme.css.contains(".rd-header-1"));
    }
}
