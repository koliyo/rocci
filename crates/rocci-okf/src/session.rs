use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const MAX_RECENTS: usize = 10;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OkfSession {
    #[serde(default)]
    pub bundle: Option<PathBuf>,
    #[serde(default)]
    pub open_path: String,
    #[serde(default)]
    pub recents: Vec<RecentDoc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentDoc {
    pub route: String,
    pub title: String,
    pub collection: String,
    pub at: String,
}

pub fn session_path() -> Option<PathBuf> {
    Some(rocci_desktop::state::state_dir()?.join("okf.json"))
}

pub fn load() -> OkfSession {
    let Some(path) = session_path() else {
        return OkfSession::default();
    };
    load_from(&path)
}

pub fn load_from(path: &Path) -> OkfSession {
    let Ok(content) = fs::read_to_string(path) else {
        return OkfSession::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn save(session: &OkfSession) {
    let Some(path) = session_path() else {
        return;
    };
    let _ = save_to(&path, session);
}

pub fn save_to(path: &Path, session: &OkfSession) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(session)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json.as_bytes())?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn collection_of(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        return String::new();
    }
    trimmed
        .split('/')
        .next()
        .filter(|segment| *segment != "review")
        .unwrap_or("")
        .to_string()
}

pub fn is_dashboard_or_review(route: &str) -> bool {
    let path = route.split(['?', '#']).next().unwrap_or(route);
    path == "/" || path == "/review" || path == "/review/"
}

pub fn route_from_url(url: &str) -> String {
    rocci_desktop::display_path(url)
}

pub fn record_visit(session: &mut OkfSession, route: &str, title: &str) {
    let route = normalize_route(route);
    session.open_path = route.clone();
    if is_dashboard_or_review(&route) {
        return;
    }
    if session
        .bundle
        .as_deref()
        .is_some_and(|root| !is_leaf_document(root, &route))
    {
        return;
    }
    let collection = collection_of(&route);
    session.recents.retain(|item| item.route != route);
    session.recents.insert(
        0,
        RecentDoc {
            route,
            title: title.to_string(),
            collection,
            at: now_stamp(),
        },
    );
    session.recents.truncate(MAX_RECENTS);
}

pub fn resolve_saved_open_path(root: &Path, saved: &str) -> String {
    let route = normalize_route(saved);
    if route.is_empty() || route == "/" {
        return "/".into();
    }
    if is_dashboard_or_review(&route) {
        return route;
    }
    let rel = route.trim_matches('/');
    if root.join(format!("{rel}.md")).is_file() {
        return format!("/{rel}/");
    }
    if root.join(rel).join("index.md").is_file() {
        return format!("/{rel}/");
    }
    "/".into()
}

pub fn launched_as_app() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.to_str()
                .map(|value| value.contains(".app/Contents/MacOS"))
        })
        .unwrap_or(false)
}

pub fn filter_launch_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    args.into_iter()
        .map(Into::into)
        .filter(|arg| !arg.starts_with("-psn_"))
        .collect()
}

pub fn pick_bundle_folder() -> Result<PathBuf> {
    let output = Command::new("osascript")
        .args([
            "-e",
            "POSIX path of (choose folder with prompt \"Open Knowledge Bundle\")",
        ])
        .output()
        .context("failed to open folder picker")?;
    if !output.status.success() {
        bail!("no knowledge bundle selected");
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        bail!("no knowledge bundle selected");
    }
    Ok(PathBuf::from(path))
}

pub fn is_leaf_document(root: &Path, route: &str) -> bool {
    let rel = normalize_route(route).trim_matches('/').to_string();
    if rel.is_empty() || is_dashboard_or_review(route) {
        return false;
    }
    root.join(format!("{rel}.md")).is_file()
}

fn normalize_route(route: &str) -> String {
    let path = route.split(['?', '#']).next().unwrap_or(route).trim();
    if path.is_empty() {
        return "/".into();
    }
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn now_stamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_round_trip() {
        let dir = std::env::temp_dir().join(format!("rocci-okf-session-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("okf.json");
        let bundle = std::env::temp_dir().join(format!("rocci-okf-bundle-{}", uuid_like()));
        fs::create_dir_all(bundle.join("plans")).unwrap();
        fs::create_dir_all(bundle.join("research")).unwrap();
        fs::write(bundle.join("plans/foo.md"), "# Foo\n").unwrap();
        fs::write(bundle.join("research/bar.md"), "# Bar\n").unwrap();
        fs::write(bundle.join("plans/index.md"), "# Plans\n").unwrap();
        let mut session = OkfSession {
            bundle: Some(bundle.clone()),
            open_path: "/plans/foo/".into(),
            recents: Vec::new(),
        };
        record_visit(&mut session, "/plans/foo/", "Foo");
        record_visit(&mut session, "/research/bar/", "Bar");
        record_visit(&mut session, "/plans/", "Plans");
        record_visit(&mut session, "/", "Knowledge");
        save_to(&path, &session).unwrap();
        let loaded = load_from(&path);
        assert_eq!(loaded.bundle, Some(bundle.clone()));
        assert_eq!(loaded.open_path, "/");
        assert_eq!(loaded.recents.len(), 2);
        assert_eq!(loaded.recents[0].route, "/research/bar/");
        assert_eq!(loaded.recents[0].collection, "research");
        assert_eq!(loaded.recents[1].collection, "plans");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&bundle);
    }

    #[test]
    fn missing_document_falls_back_to_dashboard() {
        let dir = std::env::temp_dir().join(format!("rocci-okf-missing-{}", uuid_like()));
        fs::create_dir_all(dir.join("plans")).unwrap();
        fs::write(dir.join("plans/exists.md"), "# x\n").unwrap();
        assert_eq!(
            resolve_saved_open_path(&dir, "/plans/exists/"),
            "/plans/exists/"
        );
        assert_eq!(resolve_saved_open_path(&dir, "/plans/gone/"), "/");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn recents_omit_collection_indexes() {
        let dir = std::env::temp_dir().join(format!("rocci-okf-leaf-{}", uuid_like()));
        fs::create_dir_all(dir.join("plans").join("okf")).unwrap();
        fs::write(dir.join("plans/index.md"), "# Plans\n").unwrap();
        fs::write(dir.join("plans/okf/index.md"), "# OKF\n").unwrap();
        fs::write(dir.join("plans/okf/nested-collections.md"), "# Nested\n").unwrap();
        let mut session = OkfSession {
            bundle: Some(dir.clone()),
            ..OkfSession::default()
        };
        record_visit(&mut session, "/plans/", "Plans");
        record_visit(&mut session, "/plans/okf/", "OKF");
        record_visit(&mut session, "/plans/okf/nested-collections/", "Nested");
        assert_eq!(session.recents.len(), 1);
        assert_eq!(session.recents[0].route, "/plans/okf/nested-collections/");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn filter_drops_finder_psn() {
        let args = filter_launch_args(["rocci-okf", "-psn_0_123", "view"]);
        assert_eq!(args, ["rocci-okf", "view"]);
    }

    fn uuid_like() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(1)
    }
}
