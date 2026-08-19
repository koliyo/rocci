use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rocci_browser::{
    AdapterHandler, Document, ListDocumentsResult, OpenParams, OpenResult, ProbeResult,
    RunSessions, documents_from_pages_json, serve_stdio,
};
use rocci_rocdown::{discover_rocdown, find_site_root, load_config, site_preview_route};

pub fn run() -> Result<()> {
    serve_stdio(RocdownAdapter {
        bin: env::current_exe()?,
        sessions: RunSessions::new(),
    })
    .map_err(|error| anyhow::anyhow!("{error}"))
}

struct RocdownAdapter {
    bin: PathBuf,
    sessions: RunSessions,
}

impl AdapterHandler for RocdownAdapter {
    fn adapter_id(&self) -> &str {
        "rocdown"
    }

    fn probe(&mut self, path: &str) -> rocci_browser::Result<ProbeResult> {
        let path = Path::new(path);
        let claimed = if path.is_dir() {
            path.join(rocci_rocdown::CONFIG_FILE).is_file()
        } else if path.is_file() {
            path.extension().and_then(|ext| ext.to_str()) == Some("rocdown")
                || find_site_root(path).is_some()
        } else {
            false
        };
        if claimed {
            return Ok(ProbeResult {
                claimed: true,
                label: Some(label_for(path)),
                detail: None,
            });
        }
        Ok(ProbeResult {
            claimed: false,
            label: None,
            detail: None,
        })
    }

    fn list_documents(&mut self, root: &str) -> rocci_browser::Result<ListDocumentsResult> {
        let root = Path::new(root);
        if root.is_file() {
            return Ok(ListDocumentsResult {
                documents: document_for(
                    find_site_root(root)
                        .as_deref()
                        .unwrap_or_else(|| root.parent().unwrap_or(root)),
                    root,
                )
                .into_iter()
                .collect(),
            });
        }
        if let Some(documents) = read_pages_json(root) {
            return Ok(ListDocumentsResult { documents });
        }
        Ok(ListDocumentsResult {
            documents: walk_documents(root).unwrap_or_default(),
        })
    }

    fn open(&mut self, params: OpenParams) -> rocci_browser::Result<OpenResult> {
        let path = self.document_path(&params.root, params.document.as_deref());
        let args = run_args(Path::new(&params.root), None);
        let title = params
            .document
            .clone()
            .unwrap_or_else(|| label_for(Path::new(&params.root)));
        self.sessions.open(
            &params.root,
            &self.bin.display().to_string(),
            &args,
            title,
            &path,
        )
    }

    fn shutdown(&mut self) -> rocci_browser::Result<()> {
        self.sessions.shutdown();
        Ok(())
    }
}

impl RocdownAdapter {
    fn document_path(&mut self, root: &str, document: Option<&str>) -> String {
        let Some(document) = document else {
            return "/".into();
        };
        self.list_documents(root)
            .ok()
            .and_then(|listed| listed.documents.into_iter().find(|row| row.id == document))
            .and_then(|row| row.route)
            .unwrap_or_else(|| format!("/{document}"))
    }
}

fn label_for(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("site")
        .to_string()
}

fn read_pages_json(root: &Path) -> Option<Vec<Document>> {
    let config = load_config(root).ok()?;
    let file = root.join(&config.build.output).join("pages.json");
    let raw = fs::read_to_string(file).ok()?;
    documents_from_pages_json(&raw).ok()
}

fn walk_documents(root: &Path) -> Result<Vec<Document>> {
    let mut files = Vec::new();
    if let Ok(found) = discover_rocdown(root) {
        files.extend(found);
    }
    if let Ok(config) = load_config(root) {
        for mount in config.mounts {
            let source = root.join(&mount.source);
            if source.is_dir()
                && let Ok(found) = discover_rocdown(&source)
            {
                files.extend(found);
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files
        .into_iter()
        .filter_map(|file| document_for(root, &file))
        .collect())
}

fn document_for(root: &Path, file: &Path) -> Option<Document> {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let path = rel.to_string_lossy().replace('\\', "/");
    let title = title_from_file(file).unwrap_or_else(|| path.clone());
    let route = site_preview_route(root, file);
    Some(Document {
        id: path.clone(),
        title,
        path,
        route: Some(route),
    })
}

fn title_from_file(path: &Path) -> Option<String> {
    let source = fs::read_to_string(path).ok()?;
    source.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("# ")
            .map(|title| title.trim().to_string())
    })
}

pub fn run_args(root: &Path, document: Option<&str>) -> Vec<String> {
    let target = match document {
        Some(document) => root.join(document),
        None => root.to_path_buf(),
    };
    vec!["run".into(), target.display().to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rocdown-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn probe_requires_config_file() {
        let dir = temp();
        let mut adapter = RocdownAdapter {
            bin: PathBuf::from("rocdown"),
            sessions: RunSessions::new(),
        };
        assert!(!adapter.probe(&dir.display().to_string()).unwrap().claimed);
        fs::write(
            dir.join(rocci_rocdown::CONFIG_FILE),
            "[site]\ntitle = \"Docs\"\n",
        )
        .unwrap();
        let claimed = adapter.probe(&dir.display().to_string()).unwrap();
        assert!(claimed.claimed);
        assert_eq!(
            run_args(&dir, Some("guides/page.rocdown"))[1],
            dir.join("guides/page.rocdown").display().to_string()
        );
        fs::write(dir.join("home.rocdown"), "# Home\n").unwrap();
        let listed = adapter.list_documents(&dir.display().to_string()).unwrap();
        assert_eq!(listed.documents.len(), 1);
        assert_eq!(listed.documents[0].id, "home.rocdown");
        assert!(
            adapter
                .probe(&dir.join("home.rocdown").display().to_string())
                .unwrap()
                .claimed
        );
    }
}
