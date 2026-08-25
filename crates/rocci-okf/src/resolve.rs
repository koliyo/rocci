//! Resolve configured OKF roots to local bundle directories.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use crate::config::{DirectoryRoot, GitRoot, OkfUserConfig, PollSetting, RootConfig};
use crate::git_root::{
    ResolvedKind, ResolvedRoot, bundle_path, git_root_dir, now_unix, read_meta, sync_git_root,
};

pub fn okf_cache_dir() -> PathBuf {
    rocci_roc_host::TwoTierCache::default_dir()
}

pub fn poll_is_stale(last_fetch_unix: Option<u64>, poll: PollSetting, now: u64) -> bool {
    match poll {
        PollSetting::Off => false,
        PollSetting::Interval(duration) => match last_fetch_unix {
            None => true,
            Some(fetched) => now.saturating_sub(fetched) >= duration.as_secs(),
        },
    }
}

pub fn git_needs_sync(root: &GitRoot, cache_parent: &Path, poll: PollSetting, now: u64) -> bool {
    let dir = git_root_dir(cache_parent, &root.id);
    if !dir.join("repo").join(".git").exists() {
        return true;
    }
    match read_meta(&dir.join("meta.toml")) {
        None => true,
        Some(meta) => poll_is_stale(meta.last_fetch_unix, poll, now),
    }
}

pub fn resolve_all(config: &OkfUserConfig, cache_parent: &Path) -> Vec<ResolvedRoot> {
    resolve_all_at(config, cache_parent, now_unix())
}

pub fn resolve_all_at(config: &OkfUserConfig, cache_parent: &Path, now: u64) -> Vec<ResolvedRoot> {
    let mut roots: Vec<ResolvedRoot> = config
        .roots
        .iter()
        .map(|root| resolve_one(config, root, cache_parent, now))
        .collect();
    roots.sort_by(|left, right| left.id.cmp(&right.id));
    roots
}

pub fn tick_git_roots(config: &OkfUserConfig, cache_parent: &Path) {
    tick_git_roots_at(config, cache_parent, now_unix());
}

pub fn tick_git_roots_at(config: &OkfUserConfig, cache_parent: &Path, now: u64) {
    for root in &config.roots {
        let RootConfig::Git(git) = root else {
            continue;
        };
        let poll = config.effective_poll(root);
        if !git_needs_sync(git, cache_parent, poll, now) {
            continue;
        }
        let token = git.resolved_token();
        let resolved = sync_git_root(git, cache_parent, token.as_deref());
        if let Some(error) = resolved.error.as_deref() {
            eprintln!("rocci-okf: git root `{}` sync failed: {error}", git.id);
        }
    }
}

fn resolve_one(
    config: &OkfUserConfig,
    root: &RootConfig,
    cache_parent: &Path,
    now: u64,
) -> ResolvedRoot {
    match root {
        RootConfig::Directory(dir) => resolve_directory(dir),
        RootConfig::Git(git) => {
            let poll = config.effective_poll(root);
            if git_needs_sync(git, cache_parent, poll, now) {
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
        revision: meta.as_ref().and_then(|meta| meta.last_commit.clone()),
        error: meta.and_then(|meta| meta.last_error),
        path: path.is_dir().then_some(path),
        incoming: root.incoming,
        warning: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DirectoryRoot, GitRoot, Incoming, OkfUserConfig, PollSetting};
    use std::{fs, process::Command, time::Duration};

    #[test]
    fn poll_skip_fresh_and_off() {
        let five_min = PollSetting::Interval(Duration::from_secs(300));
        assert!(poll_is_stale(None, five_min, 1_000));
        assert!(!poll_is_stale(Some(1_000), five_min, 1_100));
        assert!(!poll_is_stale(Some(1_000), five_min, 1_299));
        assert!(poll_is_stale(Some(1_000), five_min, 1_300));
        assert!(!poll_is_stale(Some(1), PollSetting::Off, 1_000_000));
    }

    #[test]
    fn missing_meta_needs_sync_even_when_fresh_clock() {
        let dir = temp_dir("meta");
        let cache = dir.join("cache");
        let git = sample_git("notes", "file:///unused");
        assert!(git_needs_sync(&git, &cache, PollSetting::Off, 0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_directory_canonicalizes() {
        let dir = temp_dir("dir");
        let bundle = dir.join("knowledge");
        fs::create_dir_all(&bundle).unwrap();
        let config = OkfUserConfig {
            roots: vec![RootConfig::Directory(DirectoryRoot {
                id: "rocci".into(),
                path: bundle.to_string_lossy().into(),
                incoming: Incoming::Allow,
                allow_from: Vec::new(),
                deny_from: Vec::new(),
                poll: None,
                extra: toml::Table::new(),
            })],
            ..OkfUserConfig::default()
        };
        let resolved = resolve_all(&config, &dir.join("cache"));
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0].enabled());
        assert_eq!(resolved[0].kind, ResolvedKind::Directory);
        assert_eq!(
            resolved[0].path.as_ref().unwrap(),
            &bundle.canonicalize().unwrap()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn git_resolve_skips_fetch_when_fresh_and_syncs_when_stale() {
        let dir = temp_dir("stale");
        let remote = dir.join("remote");
        init_repo(&remote, "first");
        let url = file_url(&remote);
        let cache = dir.join("cache");
        let git = sample_git("notes", &url);
        let config = OkfUserConfig {
            poll: PollSetting::Interval(Duration::from_secs(300)),
            roots: vec![RootConfig::Git(git.clone())],
            extra: toml::Table::new(),
        };

        let first = resolve_all_at(&config, &cache, 1_000);
        assert!(first[0].error.is_none(), "{:?}", first[0].error);
        let first_rev = first[0].revision.clone().unwrap();
        let meta_path = git_root_dir(&cache, "notes").join("meta.toml");
        let mut meta = read_meta(&meta_path).unwrap();
        meta.last_fetch_unix = Some(1_000);
        fs::write(&meta_path, toml::to_string_pretty(&meta).unwrap()).unwrap();

        fs::write(remote.join("index.md"), "okf_version: 1\n\n# second\n").unwrap();
        git_cmd(&remote, &["add", "index.md"]);
        git_cmd(&remote, &["commit", "-m", "second"]);

        let fresh = resolve_all_at(&config, &cache, 1_100);
        assert_eq!(fresh[0].revision.as_deref(), Some(first_rev.as_str()));

        let stale = resolve_all_at(&config, &cache, 1_400);
        assert_ne!(stale[0].revision.as_deref(), Some(first_rev.as_str()));

        let off = OkfUserConfig {
            poll: PollSetting::Off,
            roots: vec![RootConfig::Git(git)],
            extra: toml::Table::new(),
        };
        fs::write(remote.join("index.md"), "okf_version: 1\n\n# third\n").unwrap();
        git_cmd(&remote, &["add", "index.md"]);
        git_cmd(&remote, &["commit", "-m", "third"]);
        let skipped = resolve_all_at(&off, &cache, 9_999);
        assert_eq!(skipped[0].revision.as_deref(), stale[0].revision.as_deref());
        let _ = fs::remove_dir_all(&dir);
    }

    fn sample_git(id: &str, url: &str) -> GitRoot {
        GitRoot {
            id: id.into(),
            url: url.into(),
            branch: "main".into(),
            bundle: String::new(),
            token: None,
            token_env: None,
            incoming: Incoming::Deny,
            allow_from: Vec::new(),
            deny_from: Vec::new(),
            poll: None,
            extra: toml::Table::new(),
        }
    }

    fn init_repo(dir: &Path, body: &str) {
        fs::create_dir_all(dir).unwrap();
        git_cmd(dir, &["init", "-b", "main"]);
        git_cmd(dir, &["config", "user.email", "okf@example.com"]);
        git_cmd(dir, &["config", "user.name", "OKF Test"]);
        fs::write(
            dir.join("index.md"),
            format!("okf_version: 1\n\n# {body}\n"),
        )
        .unwrap();
        git_cmd(dir, &["add", "index.md"]);
        git_cmd(dir, &["commit", "-m", "init"]);
    }

    fn git_cmd(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn file_url(path: &Path) -> String {
        format!("file://{}", path.canonicalize().unwrap().display())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rocci-okf-resolve-{label}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(1)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
