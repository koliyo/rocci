//! Directed edges between configured knowledge roots.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use okf::{CheckReport, Diagnostic, Profile};

use crate::config::{Incoming, OkfUserConfig};
use crate::resolve::resolve_all;

pub fn edge_allowed(from: &str, to: &str, config: &OkfUserConfig) -> bool {
    if from == to {
        return true;
    }
    let Some(target) = config.roots.iter().find(|root| root.id() == to) else {
        return false;
    };
    if target.deny_from().iter().any(|id| id == from) {
        return false;
    }
    if target.allow_from().iter().any(|id| id == from) {
        return true;
    }
    target.incoming() == Incoming::Allow
}

pub fn parse_okf_href(raw: &str) -> Option<OkfHref> {
    let rest = raw.strip_prefix("okf:")?;
    let (rest, fragment) = match rest.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment).filter(|value| !value.is_empty())),
        None => (rest, None),
    };
    let (id, path) = rest.split_once('/')?;
    if id.is_empty() || path.is_empty() {
        return None;
    }
    Some(OkfHref {
        root_id: id.to_string(),
        path: path.to_string(),
        fragment: fragment.map(str::to_string),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfHref {
    pub root_id: String,
    pub path: String,
    pub fragment: Option<String>,
}

pub fn check_workspace(
    config: &OkfUserConfig,
    cache_parent: &Path,
    profile: Profile,
) -> CheckReport {
    let resolved = resolve_all(config, cache_parent);
    let paths: BTreeMap<String, PathBuf> = resolved
        .iter()
        .filter_map(|root| {
            root.path
                .as_ref()
                .map(|path| (root.id.clone(), path.clone()))
        })
        .collect();
    let mut diagnostics = Vec::new();
    for (id, path) in &paths {
        match okf::check(path, profile) {
            Ok(report) => diagnostics.extend(report.diagnostics),
            Err(error) => diagnostics.push(Diagnostic::error(
                "OKF3010",
                format!("okf:{id}"),
                None,
                format!("failed to check root `{id}`: {error:#}"),
            )),
        }
    }
    for (from_id, path) in &paths {
        let Ok(bundle) = okf::load(path, profile) else {
            continue;
        };
        for concept in &bundle.concepts {
            for link in &concept.links {
                if let Some(diagnostic) =
                    check_okf_link(from_id, &concept.path, link, config, &paths)
                {
                    diagnostics.push(diagnostic);
                }
            }
        }
    }
    CheckReport { diagnostics }
}

fn check_okf_link(
    from_id: &str,
    concept_path: &str,
    link: &okf::ast::Link,
    config: &OkfUserConfig,
    paths: &BTreeMap<String, PathBuf>,
) -> Option<Diagnostic> {
    if !link.url.starts_with("okf:") {
        return None;
    }
    let path = format!("okf:{from_id}/{concept_path}");
    let Some(href) = parse_okf_href(&link.url) else {
        return Some(Diagnostic::error(
            "OKF3010",
            path,
            Some(link.location.clone()),
            format!("unknown root id or path in `{}`", link.url),
        ));
    };
    let Some(target_root) = paths.get(&href.root_id) else {
        return Some(Diagnostic::error(
            "OKF3010",
            path,
            Some(link.location.clone()),
            format!("unknown root id or path in `{}`", link.url),
        ));
    };
    if !target_root.join(&href.path).is_file() {
        return Some(Diagnostic::error(
            "OKF3010",
            path,
            Some(link.location.clone()),
            format!("unknown root id or path in `{}`", link.url),
        ));
    }
    if !edge_allowed(from_id, &href.root_id, config) {
        return Some(Diagnostic::error(
            "OKF3011",
            path,
            Some(link.location.clone()),
            format!(
                "policy denies `{}` → `{}` for `{}`",
                from_id, href.root_id, link.url
            ),
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DirectoryRoot, Incoming, OkfUserConfig, RootConfig};
    use std::fs;

    #[test]
    fn edge_policy_order() {
        let config = two_roots(Incoming::Deny, &["rocci"], &[]);
        assert!(edge_allowed("rocci", "rocci", &config));
        assert!(edge_allowed("rocci", "notes", &config));
        assert!(edge_allowed("notes", "rocci", &config));
        let denied = two_roots(Incoming::Allow, &[], &["rocci"]);
        assert!(!edge_allowed("rocci", "notes", &denied));
        let closed = OkfUserConfig {
            roots: vec![
                dir_root("rocci", Path::new("/tmp/rocci"), Incoming::Deny, &[], &[]),
                dir_root(
                    "notes",
                    Path::new("/tmp/notes"),
                    Incoming::Deny,
                    &["rocci"],
                    &[],
                ),
            ],
            ..OkfUserConfig::default()
        };
        assert!(edge_allowed("rocci", "notes", &closed));
        assert!(!edge_allowed("notes", "rocci", &closed));
        let open = two_roots(Incoming::Allow, &[], &[]);
        assert!(edge_allowed("rocci", "notes", &open));
    }

    #[test]
    fn workspace_emits_3010_and_3011() {
        let dir = temp_dir();
        let rocci = dir.join("rocci");
        let notes = dir.join("notes");
        write_bundle(
            &rocci,
            "See [n](okf:notes/secret.md) and [missing](okf:notes/gone.md) and [unknown](okf:missing/x.md).\n",
        );
        write_bundle(&notes, "Body.\n");
        fs::write(notes.join("secret.md"), concept("Secret", "Secret.\n")).unwrap();
        let config = OkfUserConfig {
            roots: vec![
                dir_root("rocci", &rocci, Incoming::Allow, &[], &[]),
                dir_root("notes", &notes, Incoming::Deny, &[], &[]),
            ],
            ..OkfUserConfig::default()
        };
        let report = check_workspace(&config, &dir.join("cache"), Profile::Base);
        let codes: Vec<_> = report
            .diagnostics
            .iter()
            .filter(|d| d.code == "OKF3010" || d.code == "OKF3011")
            .map(|d| (d.code, d.message.clone()))
            .collect();
        assert!(
            codes
                .iter()
                .any(|(code, msg)| *code == "OKF3010" && msg.contains("gone.md")),
            "{codes:?}"
        );
        assert!(
            codes
                .iter()
                .any(|(code, msg)| *code == "OKF3010" && msg.contains("missing")),
            "{codes:?}"
        );
        assert!(
            codes
                .iter()
                .any(|(code, msg)| *code == "OKF3011" && msg.contains("secret.md")),
            "{codes:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_okf_href_splits_id_path_fragment() {
        let href = parse_okf_href("okf:notes/plans/okf/nested-collections.md#goal").unwrap();
        assert_eq!(href.root_id, "notes");
        assert_eq!(href.path, "plans/okf/nested-collections.md");
        assert_eq!(href.fragment.as_deref(), Some("goal"));
        assert!(parse_okf_href("okf:notes").is_none());
    }

    fn two_roots(incoming: Incoming, allow_from: &[&str], deny_from: &[&str]) -> OkfUserConfig {
        OkfUserConfig {
            roots: vec![
                dir_root("rocci", Path::new("/tmp/rocci"), Incoming::Allow, &[], &[]),
                dir_root(
                    "notes",
                    Path::new("/tmp/notes"),
                    incoming,
                    allow_from,
                    deny_from,
                ),
            ],
            ..OkfUserConfig::default()
        }
    }

    fn dir_root(
        id: &str,
        path: &Path,
        incoming: Incoming,
        allow_from: &[&str],
        deny_from: &[&str],
    ) -> RootConfig {
        RootConfig::Directory(DirectoryRoot {
            id: id.into(),
            path: path.display().to_string(),
            incoming,
            allow_from: allow_from.iter().map(|id| (*id).to_string()).collect(),
            deny_from: deny_from.iter().map(|id| (*id).to_string()).collect(),
            poll: None,
            extra: toml::Table::new(),
        })
    }

    fn write_bundle(root: &Path, body: &str) {
        fs::create_dir_all(root).unwrap();
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(root.join("doc.md"), concept("Doc", body)).unwrap();
    }

    fn concept(title: &str, body: &str) -> String {
        format!("---\ntype: Note\ntitle: {title}\n---\n\n# {title}\n\n{body}")
    }

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rocci-okf-edges-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(1)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
