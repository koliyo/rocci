//! Cached git checkouts for configured OKF roots under `ROCCI_CACHE`.
#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{GitRoot, Incoming};

const ASKPASS_SCRIPT: &str = r#"#!/bin/sh
case "$1" in
  *[Uu]sername*) echo "${ROCCI_OKF_GIT_USERNAME:-x-access-token}" ;;
  *) echo "$ROCCI_OKF_GIT_ASKPASS_TOKEN" ;;
esac
"#;

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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GitRootMeta {
    pub(crate) url: String,
    pub(crate) branch: String,
    #[serde(default)]
    pub(crate) bundle: String,
    pub(crate) last_commit: Option<String>,
    pub(crate) last_fetch_unix: Option<u64>,
    pub(crate) last_error: Option<String>,
}

pub fn git_root_dir(cache_parent: &Path, id: &str) -> PathBuf {
    cache_parent.join("okf-roots").join(id)
}

pub fn bundle_path(repo_dir: &Path, bundle: &str) -> PathBuf {
    if bundle.is_empty() || bundle == "." {
        repo_dir.to_path_buf()
    } else {
        repo_dir.join(bundle)
    }
}

pub fn token_for_sync(url: &str, secrets: Option<&str>) -> (Option<String>, Option<String>) {
    if secrets.is_some() && is_ssh_url(url) {
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

pub fn sync_git_root(root: &GitRoot, cache_parent: &Path, secrets: Option<&str>) -> ResolvedRoot {
    let (token, warning) = token_for_sync(&root.url, secrets);
    let token = token.as_deref();
    let root_dir = git_root_dir(cache_parent, &root.id);
    let repo_dir = root_dir.join("repo");
    let meta_path = root_dir.join("meta.toml");

    if let Err(err) = fs::create_dir_all(&root_dir) {
        return failed(root, warning, None, None, err.to_string());
    }

    let synced = if repo_ready(&repo_dir) {
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
        cmd.env("ROCCI_OKF_GIT_USERNAME", "x-access-token");
        cmd.env("ROCCI_OKF_GIT_ASKPASS_TOKEN", token);
    }
    Ok(cmd)
}

fn ensure_askpass(cache_parent: &Path) -> Result<PathBuf> {
    let dir = cache_parent.join("okf-roots");
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = dir.join("git-askpass.sh");
    fs::write(&path, ASKPASS_SCRIPT)
        .with_context(|| format!("failed to write {}", path.display()))?;
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

fn repo_ready(repo_dir: &Path) -> bool {
    repo_dir.join(".git").exists()
}

fn is_ssh_url(url: &str) -> bool {
    url.starts_with("ssh://") || url.starts_with("git@")
}

fn redact(text: &str, token: Option<&str>) -> String {
    match token {
        Some(secret) if !secret.is_empty() => text.replace(secret, "<redacted>"),
        _ => text.to_string(),
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

pub(crate) fn read_meta(path: &Path) -> Option<GitRootMeta> {
    let source = fs::read_to_string(path).ok()?;
    toml::from_str(&source).ok()
}

pub(crate) fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GitRoot;

    #[test]
    fn ssh_urls_ignore_tokens() {
        let (token, warning) = token_for_sync("git@github.com:example/notes.git", Some("secret"));
        assert!(token.is_none());
        assert!(warning.unwrap().contains("ignored"));
        let (token, warning) =
            token_for_sync("ssh://git@github.com/example/notes.git", Some("secret"));
        assert!(token.is_none());
        assert!(warning.is_some());
        let (token, warning) =
            token_for_sync("https://github.com/example/notes.git", Some("secret"));
        assert_eq!(token.as_deref(), Some("secret"));
        assert!(warning.is_none());
    }

    #[test]
    fn clone_and_fetch_file_remote() {
        let dir = temp_dir("clone");
        let remote = dir.join("remote");
        init_repo(&remote, "main", "hello");
        let url = file_url(&remote);
        let cache = dir.join("cache");
        let root = sample_root("notes", &url, "");
        let resolved = sync_git_root(&root, &cache, None);
        assert!(resolved.error.is_none(), "{:?}", resolved.error);
        assert!(resolved.enabled());
        let path = resolved.path.unwrap();
        assert!(path.join("index.md").is_file());
        let first = resolved.revision.clone().unwrap();

        fs::write(remote.join("index.md"), "okf_version: 1\n\n# updated\n").unwrap();
        git(&remote, &["add", "index.md"]);
        git(&remote, &["commit", "-m", "update"]);
        let resolved = sync_git_root(&root, &cache, Some("unused-token"));
        assert!(resolved.error.is_none(), "{:?}", resolved.error);
        let second = resolved.revision.unwrap();
        assert_ne!(first, second);
        let origin = git(&path, &["remote", "get-url", "origin"]);
        assert_eq!(origin, url);
        assert!(!origin.contains("unused-token"));
        let meta = read_meta(&git_root_dir(&cache, "notes").join("meta.toml")).unwrap();
        assert_eq!(meta.last_commit.as_deref(), Some(second.as_str()));
        assert!(meta.last_error.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundle_subdirectory_and_failed_fetch_keeps_last() {
        let dir = temp_dir("bundle");
        let remote = dir.join("remote");
        fs::create_dir_all(remote.join("knowledge")).unwrap();
        init_repo_with(
            &remote,
            "main",
            &[("knowledge/index.md", "okf_version: 1\n\n# k\n")],
        );
        let url = file_url(&remote);
        let cache = dir.join("cache");
        let root = sample_root("notes", &url, "knowledge");
        let resolved = sync_git_root(&root, &cache, None);
        assert!(resolved.error.is_none(), "{:?}", resolved.error);
        let path = resolved.path.clone().unwrap();
        assert!(path.ends_with("knowledge"));
        assert!(path.join("index.md").is_file());
        let revision = resolved.revision.clone().unwrap();
        let body = fs::read_to_string(path.join("index.md")).unwrap();

        let repo = git_root_dir(&cache, "notes").join("repo");
        git(
            &repo,
            &["remote", "set-url", "origin", "file:///no-such-okf-remote"],
        );
        let resolved = sync_git_root(&root, &cache, None);
        assert!(resolved.error.is_some());
        assert_eq!(resolved.revision.as_deref(), Some(revision.as_str()));
        assert_eq!(resolved.path.as_ref(), Some(&path));
        assert_eq!(fs::read_to_string(path.join("index.md")).unwrap(), body);
        let meta = read_meta(&git_root_dir(&cache, "notes").join("meta.toml")).unwrap();
        assert!(meta.last_error.is_some());
        let _ = fs::remove_dir_all(&dir);
    }

    fn sample_root(id: &str, url: &str, bundle: &str) -> GitRoot {
        GitRoot {
            id: id.into(),
            url: url.into(),
            branch: "main".into(),
            bundle: bundle.into(),
            token: None,
            token_env: None,
            incoming: Incoming::Deny,
            allow_from: Vec::new(),
            deny_from: Vec::new(),
            poll: None,
            extra: toml::Table::new(),
        }
    }

    fn init_repo(dir: &Path, branch: &str, body: &str) {
        init_repo_with(dir, branch, &[("index.md", body)]);
    }

    fn init_repo_with(dir: &Path, branch: &str, files: &[(&str, &str)]) {
        fs::create_dir_all(dir).unwrap();
        git(dir, &["init", "-b", branch]);
        git(dir, &["config", "user.email", "okf@example.com"]);
        git(dir, &["config", "user.name", "OKF Test"]);
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

    fn file_url(path: &Path) -> String {
        format!("file://{}", path.canonicalize().unwrap().display())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rocci-okf-git-{label}-{}", unique()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn unique() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(1)
    }
}
