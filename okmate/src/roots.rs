use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::config::{
    DirectoryRoot, GitRoot, Incoming, PollSetting, RootConfig, UserConfig, cache_dir,
};

const ASKPASS_SCRIPT: &str = r#"#!/bin/sh
case "$1" in
  *[Uu]sername*) echo "${OKMATE_GIT_USERNAME:-x-access-token}" ;;
  *) echo "$OKMATE_GIT_ASKPASS_TOKEN" ;;
esac
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncMode {
    Auto,
    Force,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootsFormat {
    Paths,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRoot {
    pub id: String,
    pub kind: ResolvedKind,
    pub path: Option<PathBuf>,
    pub revision: Option<String>,
    pub incoming: Incoming,
    pub error: Option<String>,
    pub warning: Option<String>,
}

impl ResolvedRoot {
    pub fn enabled(&self) -> bool {
        self.path.as_ref().is_some_and(|path| path.is_dir())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedKind {
    Directory,
    Git,
}

impl ResolvedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Git => "git",
        }
    }
}

#[derive(Serialize)]
struct RootJson {
    id: String,
    kind: String,
    path: Option<String>,
    revision: Option<String>,
    incoming: String,
    enabled: bool,
    error: Option<String>,
}

impl From<&ResolvedRoot> for RootJson {
    fn from(root: &ResolvedRoot) -> Self {
        Self {
            id: root.id.clone(),
            kind: root.kind.as_str().into(),
            path: root.path.as_ref().map(|path| path.display().to_string()),
            revision: root.revision.clone(),
            incoming: root.incoming.as_str().into(),
            enabled: root.enabled(),
            error: root.error.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct GitRootMeta {
    url: String,
    branch: String,
    #[serde(default)]
    bundle: String,
    last_commit: Option<String>,
    last_fetch_unix: Option<u64>,
    last_error: Option<String>,
}

pub fn print_roots(format: RootsFormat, mode: SyncMode) -> Result<()> {
    let config = crate::config::load()?;
    if config.roots.is_empty() {
        return print_fallback(format);
    }
    print_resolved(&resolve_all(&config, &cache_dir(), mode), format)
}

pub fn sync(id: Option<String>) -> Result<()> {
    let config = crate::config::load()?;
    let cache = cache_dir();
    if let Some(id) = id {
        match config.roots.iter().find(|root| root.id() == id) {
            Some(RootConfig::Directory(_)) => Ok(()),
            Some(RootConfig::Git(git)) => {
                let token = git.resolved_token();
                report_sync(&sync_git_root(git, &cache, token.as_deref()))
            }
            None => bail!("unknown root id `{id}`"),
        }
    } else {
        let mut failed = false;
        for root in &config.roots {
            let RootConfig::Git(git) = root else {
                continue;
            };
            let token = git.resolved_token();
            if report_sync(&sync_git_root(git, &cache, token.as_deref())).is_err() {
                failed = true;
            }
        }
        if failed {
            bail!("git sync failed");
        }
        Ok(())
    }
}

pub fn resolve_all(config: &UserConfig, cache_parent: &Path, mode: SyncMode) -> Vec<ResolvedRoot> {
    let now = now_unix();
    let mut roots: Vec<ResolvedRoot> = config
        .roots
        .iter()
        .map(|root| resolve_one(config, root, cache_parent, now, mode))
        .collect();
    roots.sort_by(|left, right| left.id.cmp(&right.id));
    roots
}

pub fn roots_json(roots: &[ResolvedRoot]) -> Result<String> {
    let payload: Vec<RootJson> = roots.iter().map(RootJson::from).collect();
    Ok(serde_json::to_string_pretty(&payload)?)
}

fn resolve_one(
    config: &UserConfig,
    root: &RootConfig,
    cache_parent: &Path,
    now: u64,
    mode: SyncMode,
) -> ResolvedRoot {
    match root {
        RootConfig::Directory(dir) => resolve_directory(dir),
        RootConfig::Git(git) => {
            let poll = config.effective_poll(root);
            let should_sync = match mode {
                SyncMode::Force => true,
                SyncMode::Never => false,
                SyncMode::Auto => git_needs_sync(git, cache_parent, poll, now),
            };
            if should_sync {
                let token = git.resolved_token();
                sync_git_root(git, cache_parent, token.as_deref())
            } else {
                resolve_cached_git(git, cache_parent)
            }
        }
    }
}

fn resolve_directory(root: &DirectoryRoot) -> ResolvedRoot {
    let expanded = root.expanded_path();
    match expanded.canonicalize() {
        Ok(path) if path.is_dir() => ResolvedRoot {
            id: root.id.clone(),
            kind: ResolvedKind::Directory,
            path: Some(path),
            revision: None,
            incoming: root.incoming,
            error: None,
            warning: None,
        },
        Ok(path) => ResolvedRoot {
            id: root.id.clone(),
            kind: ResolvedKind::Directory,
            path: None,
            revision: None,
            incoming: root.incoming,
            error: Some(format!("{} is not a directory", path.display())),
            warning: None,
        },
        Err(err) => ResolvedRoot {
            id: root.id.clone(),
            kind: ResolvedKind::Directory,
            path: None,
            revision: None,
            incoming: root.incoming,
            error: Some(err.to_string()),
            warning: None,
        },
    }
}

fn resolve_cached_git(root: &GitRoot, cache_parent: &Path) -> ResolvedRoot {
    let repo = git_root_dir(cache_parent, &root.id).join("repo");
    let path = bundle_path(&repo, &root.bundle);
    let meta = read_meta(&git_root_dir(cache_parent, &root.id).join("meta.toml"));
    ResolvedRoot {
        id: root.id.clone(),
        kind: ResolvedKind::Git,
        path: path.is_dir().then_some(path),
        revision: meta.as_ref().and_then(|meta| meta.last_commit.clone()),
        incoming: root.incoming,
        error: if repo.join(".git").exists() {
            meta.and_then(|meta| meta.last_error)
        } else {
            Some(format!("git root `{}` has not been synced", root.id))
        },
        warning: None,
    }
}

fn git_needs_sync(root: &GitRoot, cache_parent: &Path, poll: PollSetting, now: u64) -> bool {
    let dir = git_root_dir(cache_parent, &root.id);
    if !dir.join("repo").join(".git").exists() {
        return true;
    }
    match read_meta(&dir.join("meta.toml")) {
        None => true,
        Some(meta) => match poll {
            PollSetting::Off => false,
            PollSetting::Interval(duration) => match meta.last_fetch_unix {
                None => true,
                Some(fetched) => now.saturating_sub(fetched) >= duration.as_secs(),
            },
        },
    }
}

pub fn sync_git_root(root: &GitRoot, cache_parent: &Path, secrets: Option<&str>) -> ResolvedRoot {
    let (token, warning) = token_for_sync(&root.url, secrets);
    let token = token.as_deref();
    let root_dir = git_root_dir(cache_parent, &root.id);
    let repo_dir = root_dir.join("repo");
    let meta_path = root_dir.join("meta.toml");

    if let Err(err) = fs::create_dir_all(&root_dir) {
        return failed(root, warning, None, None, err.to_string());
    }

    let synced = if repo_dir.join(".git").exists() {
        fetch_and_checkout(root, &repo_dir, cache_parent, token)
    } else {
        if repo_dir.exists() {
            let _ = fs::remove_dir_all(&repo_dir);
        }
        clone_repo(root, &repo_dir, cache_parent, token)
    };

    match synced {
        Ok(revision) => {
            let path = bundle_path(&repo_dir, &root.bundle);
            if !path.is_dir() {
                let error = format!("bundle path {} does not exist", path.display());
                write_meta(&meta_path, root, Some(&revision), Some(&error));
                return ResolvedRoot {
                    id: root.id.clone(),
                    kind: ResolvedKind::Git,
                    path: None,
                    revision: Some(revision),
                    incoming: root.incoming,
                    error: Some(error),
                    warning,
                };
            }
            write_meta(&meta_path, root, Some(&revision), None);
            ResolvedRoot {
                id: root.id.clone(),
                kind: ResolvedKind::Git,
                path: Some(path),
                revision: Some(revision),
                incoming: root.incoming,
                error: None,
                warning,
            }
        }
        Err(error) => {
            let error = redact(&error, token);
            let revision = rev_parse(&repo_dir).ok();
            let path = bundle_path(&repo_dir, &root.bundle);
            let path = path.is_dir().then_some(path);
            write_meta(&meta_path, root, revision.as_deref(), Some(&error));
            failed(root, warning, path, revision, error)
        }
    }
}

fn token_for_sync(url: &str, secrets: Option<&str>) -> (Option<String>, Option<String>) {
    if secrets.is_some() && (url.starts_with("ssh://") || url.starts_with("git@")) {
        (
            None,
            Some("token / token_env is ignored for SSH git roots; using the local agent".into()),
        )
    } else {
        (
            secrets
                .map(str::to_string)
                .filter(|token| !token.is_empty()),
            None,
        )
    }
}

fn failed(
    root: &GitRoot,
    warning: Option<String>,
    path: Option<PathBuf>,
    revision: Option<String>,
    error: String,
) -> ResolvedRoot {
    ResolvedRoot {
        id: root.id.clone(),
        kind: ResolvedKind::Git,
        path,
        revision,
        incoming: root.incoming,
        error: Some(error),
        warning,
    }
}

fn clone_repo(
    root: &GitRoot,
    repo_dir: &Path,
    cache_parent: &Path,
    token: Option<&str>,
) -> Result<String, String> {
    let mut cmd = git_command(cache_parent, token).map_err(|err| err.to_string())?;
    cmd.args([
        "clone",
        "--branch",
        &root.branch,
        "--single-branch",
        &root.url,
    ])
    .arg(repo_dir);
    run_git(&mut cmd, token)?;
    rev_parse(repo_dir)
}

fn fetch_and_checkout(
    root: &GitRoot,
    repo_dir: &Path,
    cache_parent: &Path,
    token: Option<&str>,
) -> Result<String, String> {
    let mut fetch = git_command(cache_parent, token).map_err(|err| err.to_string())?;
    fetch
        .arg("-C")
        .arg(repo_dir)
        .args(["fetch", "origin", &root.branch]);
    run_git(&mut fetch, token)?;
    let mut checkout = git_command(cache_parent, token).map_err(|err| err.to_string())?;
    checkout
        .arg("-C")
        .arg(repo_dir)
        .args(["checkout", "--force", "FETCH_HEAD"]);
    run_git(&mut checkout, token)?;
    rev_parse(repo_dir)
}

fn rev_parse(repo_dir: &Path) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(repo_dir).args(["rev-parse", "HEAD"]);
    cmd.stdin(Stdio::null());
    run_git(&mut cmd, None)
}

fn git_command(cache_parent: &Path, token: Option<&str>) -> Result<Command> {
    let mut cmd = Command::new("git");
    cmd.stdin(Stdio::null());
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.env("GCM_INTERACTIVE", "never");
    if let Some(token) = token {
        let askpass = ensure_askpass(cache_parent)?;
        cmd.env("GIT_ASKPASS", &askpass);
        cmd.env("SSH_ASKPASS", &askpass);
        cmd.env("GIT_USERNAME", "x-access-token");
        cmd.env("OKMATE_GIT_USERNAME", "x-access-token");
        cmd.env("OKMATE_GIT_ASKPASS_TOKEN", token);
    }
    Ok(cmd)
}

fn ensure_askpass(cache_parent: &Path) -> Result<PathBuf> {
    let dir = cache_parent.join("okf-roots");
    fs::create_dir_all(&dir)?;
    let path = dir.join("git-askpass.sh");
    fs::write(&path, ASKPASS_SCRIPT)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o700);
        fs::set_permissions(&path, perms)?;
    }
    Ok(path)
}

fn run_git(cmd: &mut Command, token: Option<&str>) -> Result<String, String> {
    let output = cmd.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let detail = if stderr.is_empty() { stdout } else { stderr };
        Err(redact(&detail, token))
    }
}

fn redact(text: &str, token: Option<&str>) -> String {
    match token {
        Some(secret) if !secret.is_empty() => text.replace(secret, "<redacted>"),
        _ => text.to_string(),
    }
}

fn git_root_dir(cache_parent: &Path, id: &str) -> PathBuf {
    cache_parent.join("okf-roots").join(id)
}

fn bundle_path(repo_dir: &Path, bundle: &str) -> PathBuf {
    if bundle.is_empty() || bundle == "." {
        repo_dir.to_path_buf()
    } else {
        repo_dir.join(bundle)
    }
}

fn write_meta(path: &Path, root: &GitRoot, revision: Option<&str>, error: Option<&str>) {
    let meta = GitRootMeta {
        url: root.url.clone(),
        branch: root.branch.clone(),
        bundle: root.bundle.clone(),
        last_commit: revision.map(str::to_string),
        last_fetch_unix: Some(now_unix()),
        last_error: error.map(str::to_string),
    };
    if let Ok(encoded) = toml::to_string_pretty(&meta) {
        let _ = fs::write(path, encoded);
    }
}

fn read_meta(path: &Path) -> Option<GitRootMeta> {
    let source = fs::read_to_string(path).ok()?;
    toml::from_str(&source).ok()
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn print_fallback(format: RootsFormat) -> Result<()> {
    let knowledge = PathBuf::from("knowledge");
    if !knowledge.is_dir() {
        return Ok(());
    }
    let path = knowledge.canonicalize().unwrap_or(knowledge);
    match format {
        RootsFormat::Paths => {
            println!("{}", path.display());
            Ok(())
        }
        RootsFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&[RootJson {
                    id: "knowledge".into(),
                    kind: "directory".into(),
                    path: Some(path.display().to_string()),
                    revision: None,
                    incoming: "allow".into(),
                    enabled: true,
                    error: None,
                }])?
            );
            Ok(())
        }
    }
}

fn print_resolved(roots: &[ResolvedRoot], format: RootsFormat) -> Result<()> {
    match format {
        RootsFormat::Paths => {
            let mut unresolved = false;
            for root in roots {
                if let Some(path) = &root.path {
                    println!("{}", path.display());
                } else {
                    unresolved = true;
                    match &root.error {
                        Some(error) => eprintln!("okmate: root `{}` unresolved: {error}", root.id),
                        None => eprintln!("okmate: root `{}` unresolved", root.id),
                    }
                }
            }
            if unresolved {
                bail!("one or more knowledge roots could not be resolved");
            }
            Ok(())
        }
        RootsFormat::Json => {
            println!("{}", roots_json(roots)?);
            if roots.iter().any(|root| !root.enabled()) {
                bail!("one or more knowledge roots could not be resolved");
            }
            Ok(())
        }
    }
}

fn report_sync(resolved: &ResolvedRoot) -> Result<()> {
    if let Some(error) = &resolved.error {
        eprintln!("okmate: git root `{}` sync failed: {error}", resolved.id);
        if !resolved.enabled() {
            bail!("git root `{}` could not be resolved", resolved.id);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_json_omits_tokens_and_secrets() {
        let json = roots_json(&[ResolvedRoot {
            id: "notes".into(),
            kind: ResolvedKind::Git,
            path: Some(PathBuf::from("/tmp/notes")),
            revision: Some("abc".into()),
            incoming: Incoming::Deny,
            error: None,
            warning: None,
        }])
        .unwrap();
        assert!(!json.contains("token"), "{json}");
        assert!(!json.contains("secret"), "{json}");
        assert!(json.contains("\"id\": \"notes\""));
        assert!(json.contains("\"kind\": \"git\""));
        assert!(json.contains("\"incoming\": \"deny\""));
    }

    #[test]
    fn ssh_urls_ignore_tokens() {
        let (token, warning) = token_for_sync("git@github.com:example/notes.git", Some("secret"));
        assert!(token.is_none());
        assert!(warning.unwrap().contains("ignored"));
        let (token, warning) =
            token_for_sync("https://github.com/example/notes.git", Some("secret"));
        assert_eq!(token.as_deref(), Some("secret"));
        assert!(warning.is_none());
    }

    #[test]
    fn clone_file_remote_and_failed_fetch_keeps_last() {
        let dir = temp_dir("clone");
        let remote = dir.join("remote");
        fs::create_dir_all(remote.join("knowledge")).unwrap();
        init_repo(
            &remote,
            &[(
                "knowledge/index.md",
                "---\nokf_version: \"0.2\"\n---\n\n# k\n",
            )],
        );
        let url = format!("file://{}", remote.canonicalize().unwrap().display());
        let cache = dir.join("cache");
        let root = GitRoot {
            id: "notes".into(),
            url,
            branch: "main".into(),
            bundle: "knowledge".into(),
            token: None,
            token_env: None,
            incoming: Incoming::Deny,
            poll: None,
        };
        let resolved = sync_git_root(&root, &cache, None);
        assert!(resolved.error.is_none(), "{:?}", resolved.error);
        let path = resolved.path.clone().unwrap();
        assert!(path.ends_with("knowledge"));
        assert!(path.join("index.md").is_file());
        let revision = resolved.revision.clone().unwrap();

        let repo = git_root_dir(&cache, "notes").join("repo");
        git(
            &repo,
            &[
                "remote",
                "set-url",
                "origin",
                "file:///no-such-okmate-remote",
            ],
        );
        let resolved = sync_git_root(&root, &cache, Some("unused-token"));
        assert!(resolved.error.is_some());
        assert!(!resolved.error.as_ref().unwrap().contains("unused-token"));
        assert_eq!(resolved.revision.as_deref(), Some(revision.as_str()));
        assert_eq!(resolved.path.as_ref(), Some(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    fn init_repo(dir: &Path, files: &[(&str, &str)]) {
        fs::create_dir_all(dir).unwrap();
        git(dir, &["init", "-b", "main"]);
        git(dir, &["config", "user.email", "okmate@example.com"]);
        git(dir, &["config", "user.name", "Okmate Test"]);
        for (relative, body) in files {
            let path = dir.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, body).unwrap();
            git(dir, &["add", relative]);
        }
        git(dir, &["commit", "-m", "init"]);
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "okmate-git-{label}-{}-{}",
            std::process::id(),
            now_unix()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
