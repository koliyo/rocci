use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rocci_browser::{
    AdapterHandler, Document, ListDocumentsResult, OpenParams, OpenResult, ProbeResult,
    RunSessions, serve_stdio,
};

pub fn run() -> Result<()> {
    serve_stdio(RocciAdapter {
        bin: env::current_exe()?,
        sessions: RunSessions::new(),
    })
    .map_err(|error| anyhow::anyhow!("{error}"))
}

struct RocciAdapter {
    bin: PathBuf,
    sessions: RunSessions,
}

impl AdapterHandler for RocciAdapter {
    fn adapter_id(&self) -> &str {
        "rocci"
    }

    fn probe(&mut self, path: &str) -> rocci_browser::Result<ProbeResult> {
        let path = Path::new(path);
        if claimed(path) {
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
        Ok(ListDocumentsResult {
            documents: walk_entries(Path::new(root)),
        })
    }

    fn open(&mut self, params: OpenParams) -> rocci_browser::Result<OpenResult> {
        let args = run_args(Path::new(&params.root), params.document.as_deref());
        let key = args.last().cloned().unwrap_or_else(|| params.root.clone());
        let title = params
            .document
            .clone()
            .unwrap_or_else(|| label_for(Path::new(&params.root)));
        self.sessions
            .open(&key, &self.bin.display().to_string(), &args, title, "/")
    }

    fn shutdown(&mut self) -> rocci_browser::Result<()> {
        self.sessions.shutdown();
        Ok(())
    }
}

fn claimed(path: &Path) -> bool {
    if path.is_file() {
        return is_rocci_file(path) || is_main_roc(path) || is_rocci_toml(path);
    }
    if !path.is_dir() {
        return false;
    }
    path.join("rocci.toml").is_file()
        || path.join("main.roc").is_file()
        || path.file_name().is_some_and(|name| name == ".rocci")
        || has_top_level_rocci(path)
}

fn is_rocci_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("rocci")
}

fn is_main_roc(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("main.roc")
}

fn is_rocci_toml(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("rocci.toml")
}

fn has_top_level_rocci(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| is_rocci_file(&entry.path()))
}

fn label_for(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("app")
        .to_string()
}

fn walk_entries(root: &Path) -> Vec<Document> {
    if root.is_file() {
        return document_for(root.parent().unwrap_or(root), root)
            .into_iter()
            .collect();
    }
    let mut documents = Vec::new();
    walk_dir(root, root, &mut documents);
    documents.sort_by(|a, b| a.id.cmp(&b.id));
    documents
}

fn walk_dir(root: &Path, dir: &Path, documents: &mut Vec<Document>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.starts_with('.') || name == "target" || name == "dist" || name == "node_modules"
            {
                continue;
            }
            walk_dir(root, &path, documents);
            continue;
        }
        if is_rocci_file(&path) || is_main_roc(&path) {
            if let Some(document) = document_for(root, &path) {
                documents.push(document);
            }
        }
    }
}

fn document_for(root: &Path, file: &Path) -> Option<Document> {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let path = rel.to_string_lossy().replace('\\', "/");
    Some(Document {
        id: path.clone(),
        title: file
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or(&path)
            .to_string(),
        path,
        route: None,
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
            "rocci-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn probe_true_false_and_list_ids() {
        let dir = temp();
        let mut adapter = RocciAdapter {
            bin: PathBuf::from("rocci"),
            sessions: RunSessions::new(),
        };
        assert!(!adapter.probe(&dir.display().to_string()).unwrap().claimed);
        fs::write(dir.join("App.rocci"), "App := []\n").unwrap();
        assert!(adapter.probe(&dir.display().to_string()).unwrap().claimed);
        let listed = adapter.list_documents(&dir.display().to_string()).unwrap();
        assert_eq!(listed.documents.len(), 1);
        assert_eq!(listed.documents[0].id, "App.rocci");
        assert_eq!(
            run_args(&dir, Some("App.rocci"))[1],
            dir.join("App.rocci").display().to_string()
        );
    }
}
