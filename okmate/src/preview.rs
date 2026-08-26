use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use okf::Profile;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::http::{bind_addr, output_path, router};
use crate::site;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    #[serde(default)]
    pub bundle: Option<PathBuf>,
}

pub struct ViewOptions {
    pub path: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub profile: Profile,
    pub public: bool,
    pub port: u16,
}

pub fn run(options: ViewOptions) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new().context("failed to start tokio runtime")?;
    runtime.block_on(run_async(options))
}

async fn run_async(options: ViewOptions) -> Result<()> {
    let target = resolve_target(options.path.as_deref())?;
    persist_bundle(&target.root);
    let output = output_path(options.output.as_deref(), &target.root);
    site::build(&target.root, &output, options.profile)?;

    let addr = bind_addr(options.public, options.port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    let bound = listener
        .local_addr()
        .context("failed to read bound address")?;
    eprintln!(
        "okmate: serving {} at http://{}{}",
        target.root.display(),
        bound,
        target.open_path
    );

    let watch_root = target.root.clone();
    let watch_output = output.clone();
    let profile = options.profile;
    tokio::spawn(async move {
        if let Err(error) = watch_rebuild(watch_root, watch_output, profile).await {
            eprintln!("okmate: watch stopped: {error:#}");
        }
    });

    axum::serve(listener, router(output))
        .await
        .context("okmate view server stopped")
}

pub fn resolve_target(path: Option<&Path>) -> Result<okf::PreviewTarget> {
    if let Some(path) = path {
        return okf::resolve_preview_path(path);
    }
    if let Some(bundle) = load_session().bundle.filter(|path| path.is_dir()) {
        return Ok(okf::PreviewTarget::bundle(bundle));
    }
    let default = PathBuf::from("knowledge");
    if default.is_dir() {
        return okf::resolve_preview_path(&default);
    }
    bail!("pass a knowledge bundle path, or open one first so ~/.okmate/state remembers it");
}

pub fn state_dir() -> PathBuf {
    if let Some(path) = env::var_os("OKMATE_STATE") {
        return PathBuf::from(path);
    }
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".okmate")
        .join("state")
}

pub fn session_path() -> PathBuf {
    state_dir().join("session.json")
}

pub fn persist_bundle(root: &Path) {
    persist_bundle_to(&session_path(), root);
}

pub fn persist_bundle_to(path: &Path, root: &Path) {
    let mut session = load_session_from(path);
    session.bundle = Some(root.to_path_buf());
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&session) {
        let tmp = path.with_extension("tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(&tmp, path);
        }
    }
}

pub fn load_session() -> Session {
    load_session_from(&session_path())
}

pub fn load_session_from(path: &Path) -> Session {
    let Ok(content) = fs::read_to_string(path) else {
        return Session::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

async fn watch_rebuild(root: PathBuf, output: PathBuf, profile: Profile) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            let _ = tx.send(event);
        },
        Config::default(),
    )
    .context("failed to start knowledge watcher")?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .with_context(|| format!("failed to watch {}", root.display()))?;

    loop {
        let Some(event) = rx.recv().await else {
            break;
        };
        if event.is_err() {
            continue;
        }
        let debounce = tokio::time::sleep(Duration::from_millis(200));
        tokio::pin!(debounce);
        loop {
            tokio::select! {
                next = rx.recv() => {
                    if next.is_none() {
                        break;
                    }
                }
                _ = &mut debounce => break,
            }
        }
        if let Err(error) = site::build(&root, &output, profile) {
            eprintln!("okmate: rebuild failed: {error:#}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persist_bundle_writes_session_file() {
        let dir =
            std::env::temp_dir().join(format!("okmate-state-{}-{}", std::process::id(), "persist"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.json");
        persist_bundle_to(&path, Path::new("/tmp/knowledge"));
        let session = load_session_from(&path);
        assert_eq!(session.bundle.as_deref(), Some(Path::new("/tmp/knowledge")));
        let _ = fs::remove_dir_all(dir);
    }
}
