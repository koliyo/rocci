use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};

const DEFAULT_POLL: Duration = Duration::from_secs(300);
const DEFAULT_GIT_BRANCH: &str = "main";
const MAX_ID_LEN: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incoming {
    Allow,
    Deny,
}

impl Incoming {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "allow" => Ok(Self::Allow),
            "deny" => Ok(Self::Deny),
            other => bail!("incoming must be allow or deny, got {other}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PollSetting {
    Off,
    Interval(Duration),
}

impl Default for PollSetting {
    fn default() -> Self {
        Self::Interval(DEFAULT_POLL)
    }
}

impl PollSetting {
    pub fn as_form_value(self) -> String {
        match self {
            Self::Off => "off".into(),
            Self::Interval(duration) => {
                let secs = duration.as_secs();
                if secs % 86400 == 0 {
                    format!("{}d", secs / 86400)
                } else if secs % 3600 == 0 {
                    format!("{}h", secs / 3600)
                } else if secs % 60 == 0 {
                    format!("{}m", secs / 60)
                } else {
                    format!("{secs}s")
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserConfig {
    pub poll: PollSetting,
    pub roots: Vec<RootConfig>,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            poll: PollSetting::default(),
            roots: Vec::new(),
        }
    }
}

impl UserConfig {
    pub fn has_inline_token(&self) -> bool {
        self.roots.iter().any(|root| match root {
            RootConfig::Git(git) => git.token.as_ref().is_some_and(|token| !token.is_empty()),
            RootConfig::Directory(_) => false,
        })
    }

    pub fn effective_poll(&self, root: &RootConfig) -> PollSetting {
        root.poll().unwrap_or(self.poll)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RootConfig {
    Directory(DirectoryRoot),
    Git(GitRoot),
}

impl RootConfig {
    pub fn id(&self) -> &str {
        match self {
            Self::Directory(root) => &root.id,
            Self::Git(root) => &root.id,
        }
    }

    pub fn incoming(&self) -> Incoming {
        match self {
            Self::Directory(root) => root.incoming,
            Self::Git(root) => root.incoming,
        }
    }

    pub fn set_incoming(&mut self, incoming: Incoming) {
        match self {
            Self::Directory(root) => root.incoming = incoming,
            Self::Git(root) => root.incoming = incoming,
        }
    }

    pub fn poll(&self) -> Option<PollSetting> {
        match self {
            Self::Directory(_) => None,
            Self::Git(root) => root.poll,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectoryRoot {
    pub id: String,
    pub path: String,
    pub incoming: Incoming,
}

#[derive(Clone, PartialEq)]
pub struct GitRoot {
    pub id: String,
    pub url: String,
    pub branch: String,
    pub bundle: String,
    pub token: Option<String>,
    pub token_env: Option<String>,
    pub incoming: Incoming,
    pub poll: Option<PollSetting>,
}

impl DirectoryRoot {
    pub fn expanded_path(&self) -> PathBuf {
        expand_tilde(&self.path)
    }
}

impl GitRoot {
    pub fn resolved_token(&self) -> Option<String> {
        if let Some(name) = &self.token_env
            && let Ok(value) = env::var(name)
            && !value.is_empty()
        {
            return Some(value);
        }
        self.token.clone().filter(|token| !token.is_empty())
    }
}

impl std::fmt::Debug for GitRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRoot")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("branch", &self.branch)
            .field("bundle", &self.bundle)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("token_env", &self.token_env)
            .field("incoming", &self.incoming)
            .field("poll", &self.poll)
            .finish()
    }
}

pub fn cache_dir() -> PathBuf {
    if let Some(path) = env::var_os("OKMATE_CACHE") {
        return PathBuf::from(path);
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".okmate")
        .join("cache")
}

pub fn config_path() -> PathBuf {
    if let Ok(path) = env::var("OKMATE_CONFIG")
        && !path.is_empty()
    {
        return PathBuf::from(path);
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".okmate")
        .join("config.toml")
}

pub fn load() -> Result<UserConfig> {
    let path = config_path();
    if path.is_file() {
        return load_from_path(&path);
    }
    if let Some(imported) = import_rocci_config()? {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        save(&imported, &path)?;
        return Ok(imported);
    }
    Ok(UserConfig::default())
}

pub fn load_or_default(path: &Path) -> UserConfig {
    if path.is_file() {
        load_from_path(path).unwrap_or_default()
    } else {
        UserConfig::default()
    }
}

pub fn load_from_path(path: &Path) -> Result<UserConfig> {
    let source =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    parse(&source)
}

pub fn parse(source: &str) -> Result<UserConfig> {
    let value: toml::Value = toml::from_str(source).context("invalid okmate config TOML")?;
    let toml::Value::Table(mut table) = value else {
        bail!("okmate config must be a TOML table");
    };
    let poll = match table.remove("poll") {
        Some(value) => parse_poll(&value)?,
        None => PollSetting::default(),
    };
    let roots = match table.remove("roots") {
        Some(toml::Value::Array(items)) => items
            .into_iter()
            .enumerate()
            .map(|(index, item)| parse_root(item, index))
            .collect::<Result<Vec<_>>>()?,
        Some(_) => bail!("roots must be an array of tables"),
        None => Vec::new(),
    };
    let config = UserConfig { poll, roots };
    validate(&config)?;
    Ok(config)
}

pub fn save(config: &UserConfig, path: &Path) -> Result<()> {
    validate(config)?;
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let encoded = to_toml(config)?;
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, encoded.as_bytes())
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

pub fn to_toml(config: &UserConfig) -> Result<String> {
    toml::to_string_pretty(&to_table(config)).context("failed to encode okmate config")
}

pub fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() || id.len() > MAX_ID_LEN {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

pub fn valid_git_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("ssh://")
        || url.starts_with("git@")
        || url.starts_with("file://")
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if (path == "~" || path.starts_with("~/") || path.starts_with("~\\"))
        && let Some(home) = home_dir()
    {
        let rest = if path.len() > 1 { &path[2..] } else { "" };
        return home.join(rest);
    }
    PathBuf::from(path)
}

fn import_rocci_config() -> Result<Option<UserConfig>> {
    let path = rocci_okf_config_path()?;
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(load_from_path(&path)?))
}

fn rocci_okf_config_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("ROCCI_OKF_CONFIG")
        && !path.is_empty()
    {
        return Ok(PathBuf::from(path));
    }
    Ok(home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".rocci")
        .join("okf.toml"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn parse_root(value: toml::Value, index: usize) -> Result<RootConfig> {
    let toml::Value::Table(mut table) = value else {
        bail!("roots[{index}] must be a table");
    };
    let id =
        take_string(&mut table, "id")?.with_context(|| format!("roots[{index}] is missing id"))?;
    if !valid_id(&id) {
        bail!("invalid root id `{id}`");
    }
    let kind =
        take_string(&mut table, "kind")?.with_context(|| format!("root `{id}` is missing kind"))?;
    let incoming = match table.remove("incoming") {
        Some(toml::Value::String(value)) => Incoming::parse(&value)?,
        Some(_) => bail!("root `{id}` incoming must be a string"),
        None => match kind.as_str() {
            "git" => Incoming::Deny,
            _ => Incoming::Allow,
        },
    };
    match kind.as_str() {
        "directory" => {
            let path = take_string(&mut table, "path")?
                .with_context(|| format!("directory root `{id}` is missing path"))?;
            Ok(RootConfig::Directory(DirectoryRoot { id, path, incoming }))
        }
        "git" => {
            let url = take_string(&mut table, "url")?
                .with_context(|| format!("git root `{id}` is missing url"))?;
            if !valid_git_url(&url) {
                bail!("git root `{id}` has unsupported url `{url}`");
            }
            Ok(RootConfig::Git(GitRoot {
                id,
                url,
                branch: take_string(&mut table, "branch")?
                    .unwrap_or_else(|| DEFAULT_GIT_BRANCH.into()),
                bundle: take_string(&mut table, "bundle")?.unwrap_or_default(),
                token: take_string(&mut table, "token")?,
                token_env: take_string(&mut table, "token_env")?,
                incoming,
                poll: match table.remove("poll") {
                    Some(value) => Some(parse_poll(&value)?),
                    None => None,
                },
            }))
        }
        other => bail!("root `{id}` has unknown kind `{other}`"),
    }
}

fn parse_poll(value: &toml::Value) -> Result<PollSetting> {
    match value {
        toml::Value::Boolean(false) => Ok(PollSetting::Off),
        toml::Value::String(text) => parse_poll_str(text),
        _ => bail!("poll must be a duration string or false"),
    }
}

fn parse_poll_str(text: &str) -> Result<PollSetting> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("off") || text.eq_ignore_ascii_case("false") {
        return Ok(PollSetting::Off);
    }
    let (digits, unit) =
        split_duration(text).with_context(|| format!("invalid poll duration `{text}`"))?;
    let amount: u64 = digits
        .parse()
        .with_context(|| format!("invalid poll duration `{text}`"))?;
    if amount == 0 {
        bail!("poll duration must be greater than zero");
    }
    let secs = match unit {
        's' | 'S' => amount,
        'm' | 'M' => amount.saturating_mul(60),
        'h' | 'H' => amount.saturating_mul(3600),
        'd' | 'D' => amount.saturating_mul(86400),
        _ => bail!("invalid poll duration `{text}`"),
    };
    Ok(PollSetting::Interval(Duration::from_secs(secs)))
}

fn split_duration(text: &str) -> Option<(&str, char)> {
    let unit = text.chars().last()?;
    if !unit.is_ascii_alphabetic() {
        return None;
    }
    Some((&text[..text.len() - unit.len_utf8()], unit))
}

fn validate(config: &UserConfig) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for root in &config.roots {
        if !seen.insert(root.id()) {
            bail!("duplicate root id `{}`", root.id());
        }
        if !valid_id(root.id()) {
            bail!("invalid root id `{}`", root.id());
        }
    }
    Ok(())
}

fn take_string(table: &mut toml::Table, key: &str) -> Result<Option<String>> {
    match table.remove(key) {
        Some(toml::Value::String(value)) => Ok(Some(value)),
        Some(_) => bail!("{key} must be a string"),
        None => Ok(None),
    }
}

fn to_table(config: &UserConfig) -> toml::Table {
    let mut table = toml::Table::new();
    table.insert(
        "poll".into(),
        match config.poll {
            PollSetting::Off => toml::Value::Boolean(false),
            PollSetting::Interval(_) => toml::Value::String(config.poll.as_form_value()),
        },
    );
    if !config.roots.is_empty() {
        table.insert(
            "roots".into(),
            toml::Value::Array(
                config
                    .roots
                    .iter()
                    .map(|root| toml::Value::Table(root_to_table(root)))
                    .collect(),
            ),
        );
    }
    table
}

fn root_to_table(root: &RootConfig) -> toml::Table {
    let mut table = toml::Table::new();
    table.insert("id".into(), toml::Value::String(root.id().into()));
    match root {
        RootConfig::Directory(dir) => {
            table.insert("kind".into(), toml::Value::String("directory".into()));
            table.insert("path".into(), toml::Value::String(dir.path.clone()));
            table.insert(
                "incoming".into(),
                toml::Value::String(dir.incoming.as_str().into()),
            );
        }
        RootConfig::Git(git) => {
            table.insert("kind".into(), toml::Value::String("git".into()));
            table.insert("url".into(), toml::Value::String(git.url.clone()));
            table.insert("branch".into(), toml::Value::String(git.branch.clone()));
            if !git.bundle.is_empty() {
                table.insert("bundle".into(), toml::Value::String(git.bundle.clone()));
            }
            if let Some(token_env) = &git.token_env {
                table.insert("token_env".into(), toml::Value::String(token_env.clone()));
            }
            if let Some(token) = &git.token
                && !token.is_empty()
            {
                table.insert("token".into(), toml::Value::String(token.clone()));
            }
            table.insert(
                "incoming".into(),
                toml::Value::String(git.incoming.as_str().into()),
            );
            if let Some(poll) = git.poll {
                table.insert(
                    "poll".into(),
                    match poll {
                        PollSetting::Off => toml::Value::Boolean(false),
                        PollSetting::Interval(_) => toml::Value::String(poll.as_form_value()),
                    },
                );
            }
        }
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_roundtrip_redacts_token_from_debug() {
        let config = parse(
            r#"
poll = "5m"
[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/notes.git"
token = "super-secret-token"
token_env = "GITHUB_TOKEN"
"#,
        )
        .unwrap();
        let debug = format!("{:?}", config);
        assert!(!debug.contains("super-secret-token"));
        assert!(debug.contains("<redacted>"));
        let encoded = to_toml(&config).unwrap();
        assert!(encoded.contains("super-secret-token"));
    }
}
