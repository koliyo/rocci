//! User-level OKF root registry (`ROCCI_OKF_CONFIG` or `~/.rocci/okf.toml`).
//!
//! Saving rewrites a canonical TOML file. Comments are not preserved; unknown
//! keys at the file and root level are kept on round-trip.
#![allow(dead_code)]

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
    time::Duration,
};

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
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }

    pub(crate) fn parse(value: &str) -> Result<Self> {
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
        format_poll(self)
    }

    fn to_toml_value(self) -> toml::Value {
        match self {
            Self::Off => toml::Value::Boolean(false),
            Self::Interval(duration) => toml::Value::String(format_duration(duration)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct OkfUserConfig {
    pub poll: PollSetting,
    pub roots: Vec<RootConfig>,
    pub extra: toml::Table,
}

impl Default for OkfUserConfig {
    fn default() -> Self {
        Self {
            poll: PollSetting::default(),
            roots: Vec::new(),
            extra: toml::Table::new(),
        }
    }
}

impl fmt::Debug for OkfUserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OkfUserConfig")
            .field("poll", &self.poll)
            .field("roots", &self.roots)
            .field("extra", &self.extra)
            .finish()
    }
}

impl fmt::Display for OkfUserConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OkfUserConfig {{ poll: {}, roots: [",
            format_poll(self.poll)
        )?;
        for (i, root) in self.roots.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{root}")?;
        }
        write!(f, "] }}")
    }
}

impl OkfUserConfig {
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

#[derive(Clone, PartialEq)]
pub enum RootConfig {
    Directory(DirectoryRoot),
    Git(GitRoot),
}

impl fmt::Debug for RootConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory(root) => root.fmt(f),
            Self::Git(root) => root.fmt(f),
        }
    }
}

impl fmt::Display for RootConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory(root) => write!(f, "{root}"),
            Self::Git(root) => write!(f, "{root}"),
        }
    }
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

    pub fn allow_from(&self) -> &[String] {
        match self {
            Self::Directory(root) => &root.allow_from,
            Self::Git(root) => &root.allow_from,
        }
    }

    pub fn deny_from(&self) -> &[String] {
        match self {
            Self::Directory(root) => &root.deny_from,
            Self::Git(root) => &root.deny_from,
        }
    }

    pub fn poll(&self) -> Option<PollSetting> {
        match self {
            Self::Directory(root) => root.poll,
            Self::Git(root) => root.poll,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DirectoryRoot {
    pub id: String,
    pub path: String,
    pub incoming: Incoming,
    pub allow_from: Vec<String>,
    pub deny_from: Vec<String>,
    pub poll: Option<PollSetting>,
    pub extra: toml::Table,
}

impl DirectoryRoot {
    pub fn expanded_path(&self) -> PathBuf {
        expand_tilde(&self.path)
    }
}

impl fmt::Display for DirectoryRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "directory {} path={} incoming={}",
            self.id,
            self.path,
            self.incoming.as_str()
        )
    }
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
    pub allow_from: Vec<String>,
    pub deny_from: Vec<String>,
    pub poll: Option<PollSetting>,
    pub extra: toml::Table,
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

impl fmt::Debug for GitRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GitRoot")
            .field("id", &self.id)
            .field("url", &self.url)
            .field("branch", &self.branch)
            .field("bundle", &self.bundle)
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("token_env", &self.token_env)
            .field("incoming", &self.incoming)
            .field("allow_from", &self.allow_from)
            .field("deny_from", &self.deny_from)
            .field("poll", &self.poll)
            .field("extra", &self.extra)
            .finish()
    }
}

impl fmt::Display for GitRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "git {} url={} branch={} incoming={} token={}",
            self.id,
            self.url,
            self.branch,
            self.incoming.as_str(),
            if self.token.as_ref().is_some_and(|token| !token.is_empty()) {
                "<redacted>"
            } else {
                "none"
            }
        )
    }
}

pub fn config_path() -> Option<PathBuf> {
    if let Ok(path) = env::var("ROCCI_OKF_CONFIG")
        && !path.is_empty()
    {
        return Some(PathBuf::from(path));
    }
    default_config_path()
}

pub fn default_config_path() -> Option<PathBuf> {
    Some(rocci_dir()?.join("okf.toml"))
}

pub fn rocci_dir() -> Option<PathBuf> {
    if let Ok(home) = env::var("ROCCI_HOME")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home).join(".rocci"));
    }
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".rocci"))
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

pub fn load() -> Result<OkfUserConfig> {
    let Some(path) = config_path() else {
        return Ok(OkfUserConfig::default());
    };
    load_from_path(&path, explicit_config_env())
}

pub fn load_from_path(path: &Path, required: bool) -> Result<OkfUserConfig> {
    match fs::read_to_string(path) {
        Ok(source) => parse(&source),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && !required => {
            Ok(OkfUserConfig::default())
        }
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

pub fn parse(source: &str) -> Result<OkfUserConfig> {
    let value: toml::Value = toml::from_str(source).context("invalid OKF user config TOML")?;
    let toml::Value::Table(mut table) = value else {
        bail!("OKF user config must be a TOML table");
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
    let config = OkfUserConfig {
        poll,
        roots,
        extra: table,
    };
    validate(&config)?;
    Ok(config)
}

pub fn save(config: &OkfUserConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        set_dir_mode_if_rocci(parent)?;
    }
    let encoded = to_toml(config)?;
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, encoded.as_bytes())
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    set_file_mode(&tmp, config.has_inline_token())?;
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

pub fn to_toml(config: &OkfUserConfig) -> Result<String> {
    toml::to_string_pretty(&to_table(config)).context("failed to encode OKF user config")
}

fn explicit_config_env() -> bool {
    env::var("ROCCI_OKF_CONFIG")
        .ok()
        .is_some_and(|value| !value.is_empty())
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
    let allow_from = take_string_list(&mut table, "allow_from")?;
    let deny_from = take_string_list(&mut table, "deny_from")?;
    let poll = match table.remove("poll") {
        Some(value) => Some(parse_poll(&value)?),
        None => None,
    };
    let incoming = match table.remove("incoming") {
        Some(toml::Value::String(value)) => Incoming::parse(&value)?,
        Some(_) => bail!("root `{id}` incoming must be a string"),
        None => default_incoming(&kind),
    };
    match kind.as_str() {
        "directory" => {
            let path = take_string(&mut table, "path")?
                .with_context(|| format!("directory root `{id}` is missing path"))?;
            Ok(RootConfig::Directory(DirectoryRoot {
                id,
                path,
                incoming,
                allow_from,
                deny_from,
                poll,
                extra: table,
            }))
        }
        "git" => {
            let url = take_string(&mut table, "url")?
                .with_context(|| format!("git root `{id}` is missing url"))?;
            if !valid_git_url(&url) {
                bail!("git root `{id}` has unsupported url `{url}`");
            }
            let branch =
                take_string(&mut table, "branch")?.unwrap_or_else(|| DEFAULT_GIT_BRANCH.into());
            let bundle = take_string(&mut table, "bundle")?.unwrap_or_default();
            let token = take_string(&mut table, "token")?;
            let token_env = take_string(&mut table, "token_env")?;
            Ok(RootConfig::Git(GitRoot {
                id,
                url,
                branch,
                bundle,
                token,
                token_env,
                incoming,
                allow_from,
                deny_from,
                poll,
                extra: table,
            }))
        }
        other => bail!("root `{id}` has unknown kind `{other}`"),
    }
}

fn default_incoming(kind: &str) -> Incoming {
    match kind {
        "git" => Incoming::Deny,
        _ => Incoming::Allow,
    }
}

pub(crate) fn validate_config(config: &OkfUserConfig) -> Result<()> {
    validate(config)
}

fn validate(config: &OkfUserConfig) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for root in &config.roots {
        if !seen.insert(root.id()) {
            bail!("duplicate root id `{}`", root.id());
        }
    }
    for root in &config.roots {
        let id = root.id();
        for other in root.allow_from() {
            if other == id {
                bail!("root `{id}` lists itself in allow_from");
            }
            if !seen.contains(other.as_str()) {
                bail!("root `{id}` allow_from names unknown id `{other}`");
            }
        }
        for other in root.deny_from() {
            if other == id {
                bail!("root `{id}` lists itself in deny_from");
            }
            if !seen.contains(other.as_str()) {
                bail!("root `{id}` deny_from names unknown id `{other}`");
            }
        }
        let allow: std::collections::BTreeSet<_> = root.allow_from().iter().collect();
        for other in root.deny_from() {
            if allow.contains(other) {
                bail!("root `{id}` lists `{other}` in both allow_from and deny_from");
            }
        }
    }
    Ok(())
}

fn parse_poll(value: &toml::Value) -> Result<PollSetting> {
    match value {
        toml::Value::Boolean(false) => Ok(PollSetting::Off),
        toml::Value::Boolean(true) => bail!("poll true is invalid; use a duration or false"),
        toml::Value::String(text) => parse_poll_str(text),
        _ => bail!("poll must be a duration string or false"),
    }
}

fn parse_poll_str(text: &str) -> Result<PollSetting> {
    let text = text.trim();
    if text.eq_ignore_ascii_case("off") || text.eq_ignore_ascii_case("false") {
        return Ok(PollSetting::Off);
    }
    let Some((digits, unit)) = split_duration(text) else {
        bail!("invalid poll duration `{text}`");
    };
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
    let unit = text.chars().next_back()?;
    if !unit.is_ascii_alphabetic() {
        return None;
    }
    let digits = &text[..text.len() - unit.len_utf8()];
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((digits, unit))
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs > 0 && secs.is_multiple_of(86400) {
        format!("{}d", secs / 86400)
    } else if secs > 0 && secs.is_multiple_of(3600) {
        format!("{}h", secs / 3600)
    } else if secs > 0 && secs.is_multiple_of(60) {
        format!("{}m", secs / 60)
    } else {
        format!("{secs}s")
    }
}

fn format_poll(poll: PollSetting) -> String {
    match poll {
        PollSetting::Off => "off".into(),
        PollSetting::Interval(duration) => format_duration(duration),
    }
}

fn valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() || id.len() > MAX_ID_LEN {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn valid_git_url(url: &str) -> bool {
    url.starts_with("https://")
        || url.starts_with("ssh://")
        || url.starts_with("git@")
        || url.starts_with("file://")
}

fn take_string(table: &mut toml::Table, key: &str) -> Result<Option<String>> {
    match table.remove(key) {
        Some(toml::Value::String(value)) => Ok(Some(value)),
        Some(_) => bail!("{key} must be a string"),
        None => Ok(None),
    }
}

fn take_string_list(table: &mut toml::Table, key: &str) -> Result<Vec<String>> {
    match table.remove(key) {
        Some(toml::Value::Array(items)) => items
            .into_iter()
            .map(|item| match item {
                toml::Value::String(value) => Ok(value),
                _ => bail!("{key} entries must be strings"),
            })
            .collect(),
        Some(_) => bail!("{key} must be an array of strings"),
        None => Ok(Vec::new()),
    }
}

fn to_table(config: &OkfUserConfig) -> toml::Table {
    let mut table = toml::Table::new();
    table.insert("poll".into(), config.poll.to_toml_value());
    for (key, value) in &config.extra {
        table.insert(key.clone(), value.clone());
    }
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
            insert_common(
                &mut table,
                dir.incoming,
                &dir.allow_from,
                &dir.deny_from,
                dir.poll,
                &dir.extra,
            );
        }
        RootConfig::Git(git) => {
            table.insert("kind".into(), toml::Value::String("git".into()));
            table.insert("url".into(), toml::Value::String(git.url.clone()));
            table.insert("branch".into(), toml::Value::String(git.branch.clone()));
            if !git.bundle.is_empty() && git.bundle != "." {
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
            insert_common(
                &mut table,
                git.incoming,
                &git.allow_from,
                &git.deny_from,
                git.poll,
                &git.extra,
            );
        }
    }
    table
}

fn insert_common(
    table: &mut toml::Table,
    incoming: Incoming,
    allow_from: &[String],
    deny_from: &[String],
    poll: Option<PollSetting>,
    extra: &toml::Table,
) {
    table.insert(
        "incoming".into(),
        toml::Value::String(incoming.as_str().into()),
    );
    if let Some(poll) = poll {
        table.insert("poll".into(), poll.to_toml_value());
    }
    if !allow_from.is_empty() {
        table.insert(
            "allow_from".into(),
            toml::Value::Array(
                allow_from
                    .iter()
                    .map(|id| toml::Value::String(id.clone()))
                    .collect(),
            ),
        );
    }
    if !deny_from.is_empty() {
        table.insert(
            "deny_from".into(),
            toml::Value::Array(
                deny_from
                    .iter()
                    .map(|id| toml::Value::String(id.clone()))
                    .collect(),
            ),
        );
    }
    for (key, value) in extra {
        table.insert(key.clone(), value.clone());
    }
}

fn set_dir_mode_if_rocci(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if path.file_name().is_some_and(|name| name == ".rocci") {
            let mut perms = fs::metadata(path)
                .with_context(|| format!("failed to stat {}", path.display()))?
                .permissions();
            perms.set_mode(0o700);
            fs::set_permissions(path, perms)
                .with_context(|| format!("failed to chmod {}", path.display()))?;
        }
    }
    let _ = path;
    Ok(())
}

fn set_file_mode(path: &Path, inline_token: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if inline_token {
            let mut perms = fs::metadata(path)
                .with_context(|| format!("failed to stat {}", path.display()))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)
                .with_context(|| format!("failed to chmod {}", path.display()))?;
        }
    }
    let _ = (path, inline_token);
    Ok(())
}

#[cfg(test)]
pub(crate) static CONFIG_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        CONFIG_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    const EXAMPLE: &str = r#"
poll = "5m"

[[roots]]
id = "rocci"
kind = "directory"
path = "~/Projects/rocci/knowledge"
incoming = "allow"

[[roots]]
id = "notes"
kind = "git"
url = "https://github.com/example/private-notes.git"
branch = "main"
bundle = "knowledge"
token_env = "GITHUB_TOKEN"
incoming = "deny"
poll = "15m"
allow_from = ["rocci"]
deny_from = []
"#;

    #[test]
    fn parses_plan_example() {
        let config = parse(EXAMPLE).unwrap();
        assert_eq!(config.poll, PollSetting::Interval(Duration::from_secs(300)));
        assert_eq!(config.roots.len(), 2);
        let RootConfig::Directory(dir) = &config.roots[0] else {
            panic!("expected directory root");
        };
        assert_eq!(dir.id, "rocci");
        assert_eq!(dir.path, "~/Projects/rocci/knowledge");
        assert_eq!(dir.incoming, Incoming::Allow);
        let RootConfig::Git(git) = &config.roots[1] else {
            panic!("expected git root");
        };
        assert_eq!(git.id, "notes");
        assert_eq!(git.branch, "main");
        assert_eq!(git.bundle, "knowledge");
        assert_eq!(git.token_env.as_deref(), Some("GITHUB_TOKEN"));
        assert_eq!(git.incoming, Incoming::Deny);
        assert_eq!(
            git.poll,
            Some(PollSetting::Interval(Duration::from_secs(900)))
        );
        assert_eq!(git.allow_from, ["rocci"]);
        assert_eq!(
            config.effective_poll(&config.roots[1]),
            PollSetting::Interval(Duration::from_secs(900))
        );
    }

    #[test]
    fn directory_incoming_defaults_to_allow_git_to_deny() {
        let config = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/tmp/knowledge"

[[roots]]
id = "notes"
kind = "git"
url = "https://github.com/example/notes.git"
"#,
        )
        .unwrap();
        assert_eq!(config.roots[0].incoming(), Incoming::Allow);
        assert_eq!(config.roots[1].incoming(), Incoming::Deny);
        let RootConfig::Git(git) = &config.roots[1] else {
            panic!("expected git");
        };
        assert_eq!(git.branch, "main");
        assert!(git.bundle.is_empty());
    }

    #[test]
    fn duplicate_id_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/a"

[[roots]]
id = "rocci"
kind = "directory"
path = "/b"
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("duplicate root id `rocci`"));
    }

    #[test]
    fn unknown_kind_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "s3"
path = "/a"
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown kind `s3`"));
    }

    #[test]
    fn git_missing_url_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "notes"
kind = "git"
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing url"));
    }

    #[test]
    fn directory_missing_path_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("missing path"));
    }

    #[test]
    fn unknown_edge_id_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/a"
allow_from = ["missing"]
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown id `missing`"));
    }

    #[test]
    fn self_edge_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/a"
deny_from = ["rocci"]
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("lists itself"));
    }

    #[test]
    fn both_allow_and_deny_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "rocci"
kind = "directory"
path = "/a"

[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
allow_from = ["rocci"]
deny_from = ["rocci"]
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("both allow_from and deny_from"));
    }

    #[test]
    fn invalid_id_is_error() {
        let err = parse(
            r#"
[[roots]]
id = "Rocci"
kind = "directory"
path = "/a"
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid root id"));
    }

    #[test]
    fn poll_parses_durations_and_off() {
        assert_eq!(
            parse("poll = \"1h\"").unwrap().poll,
            PollSetting::Interval(Duration::from_secs(3600))
        );
        assert_eq!(parse("poll = false").unwrap().poll, PollSetting::Off);
        assert_eq!(parse("poll = \"off\"").unwrap().poll, PollSetting::Off);
        assert!(parse("poll = true").is_err());
        assert!(parse("poll = \"5\"").is_err());
    }

    #[test]
    fn round_trip_preserves_unknown_keys() {
        let config = parse(
            r#"
poll = "5m"
label = "desk"

[[roots]]
id = "rocci"
kind = "directory"
path = "/tmp/k"
color = "blue"
"#,
        )
        .unwrap();
        assert_eq!(
            config.extra.get("label").and_then(toml::Value::as_str),
            Some("desk")
        );
        let RootConfig::Directory(dir) = &config.roots[0] else {
            panic!("directory");
        };
        assert_eq!(
            dir.extra.get("color").and_then(toml::Value::as_str),
            Some("blue")
        );
        let encoded = to_toml(&config).unwrap();
        let again = parse(&encoded).unwrap();
        assert_eq!(
            again.extra.get("label").and_then(toml::Value::as_str),
            Some("desk")
        );
        let RootConfig::Directory(dir) = &again.roots[0] else {
            panic!("directory");
        };
        assert_eq!(
            dir.extra.get("color").and_then(toml::Value::as_str),
            Some("blue")
        );
    }

    #[test]
    fn display_and_debug_redact_token() {
        let config = parse(
            r#"
[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
token = "super-secret"
"#,
        )
        .unwrap();
        let display = config.to_string();
        let debug = format!("{config:?}");
        assert!(!display.contains("super-secret"), "{display}");
        assert!(!debug.contains("super-secret"), "{debug}");
        assert!(display.contains("<redacted>"));
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn token_env_wins_over_inline_token() {
        let _lock = env_lock();
        let config = parse(
            r#"
[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
token = "inline"
token_env = "ROCCI_OKF_TEST_TOKEN"
"#,
        )
        .unwrap();
        let RootConfig::Git(git) = &config.roots[0] else {
            panic!("git");
        };
        unsafe { env::set_var("ROCCI_OKF_TEST_TOKEN", "from-env") };
        assert_eq!(git.resolved_token().as_deref(), Some("from-env"));
        unsafe { env::remove_var("ROCCI_OKF_TEST_TOKEN") };
        assert_eq!(git.resolved_token().as_deref(), Some("inline"));
    }

    #[test]
    fn tilde_expands_from_home() {
        let _lock = env_lock();
        let original = env::var("HOME").ok();
        unsafe { env::set_var("HOME", "/Users/tester") };
        assert_eq!(
            expand_tilde("~/Projects/rocci/knowledge"),
            PathBuf::from("/Users/tester/Projects/rocci/knowledge")
        );
        match original {
            Some(value) => unsafe { env::set_var("HOME", value) },
            None => unsafe { env::remove_var("HOME") },
        }
    }

    #[test]
    fn config_path_honors_env() {
        let _lock = env_lock();
        let original = env::var("ROCCI_OKF_CONFIG").ok();
        unsafe { env::set_var("ROCCI_OKF_CONFIG", "/tmp/custom-okf.toml") };
        assert_eq!(
            config_path().as_deref(),
            Some(Path::new("/tmp/custom-okf.toml"))
        );
        match original {
            Some(value) => unsafe { env::set_var("ROCCI_OKF_CONFIG", value) },
            None => unsafe { env::remove_var("ROCCI_OKF_CONFIG") },
        }
    }

    #[test]
    fn missing_default_config_is_empty() {
        let dir = std::env::temp_dir().join(format!("rocci-okf-missing-{}", unique()));
        let path = dir.join("okf.toml");
        let loaded = load_from_path(&path, false).unwrap();
        assert!(loaded.roots.is_empty());
        assert!(load_from_path(&path, true).is_err());
    }

    #[test]
    fn save_round_trip_and_token_mode() {
        let dir = std::env::temp_dir().join(format!("rocci-okf-save-{}", unique()));
        let rocci = dir.join(".rocci");
        let path = rocci.join("okf.toml");
        let config = parse(
            r#"
[[roots]]
id = "notes"
kind = "git"
url = "https://example.com/n.git"
token = "secret"
"#,
        )
        .unwrap();
        save(&config, &path).unwrap();
        let loaded = load_from_path(&path, true).unwrap();
        assert_eq!(loaded.roots.len(), 1);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = fs::metadata(&rocci).unwrap().permissions().mode() & 0o777;
            let file_mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
            assert_eq!(file_mode, 0o600);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    fn unique() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(1)
    }
}
