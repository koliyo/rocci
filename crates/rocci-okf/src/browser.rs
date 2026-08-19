use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Child,
};

use anyhow::Result;
use okf::{parse_yaml_mapping, resolve_preview_path, split_frontmatter};
use rocci_browser::{
    AdapterHandler, Document, ListDocumentsResult, OpenParams, OpenResult, ProbeResult,
    serve_stdio, spawn_run_no_window,
};

pub fn run() -> Result<()> {
    serve_stdio(OkfAdapter {
        bin: env::current_exe()?,
        children: Vec::new(),
    })
    .map_err(|error| anyhow::anyhow!("{error}"))
}

struct OkfAdapter {
    bin: PathBuf,
    children: Vec<Child>,
}

impl AdapterHandler for OkfAdapter {
    fn adapter_id(&self) -> &str {
        "okf"
    }

    fn probe(&mut self, path: &str) -> rocci_browser::Result<ProbeResult> {
        let path = Path::new(path);
        let claimed = if path.is_dir() {
            is_bundle_root(path)
        } else {
            resolve_preview_path(path).is_ok()
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
        let bundle = if root.is_file() {
            resolve_preview_path(root)
                .map(|target| target.root)
                .unwrap_or_else(|_| root.to_path_buf())
        } else {
            root.to_path_buf()
        };
        Ok(ListDocumentsResult {
            documents: walk_concepts(&bundle),
        })
    }

    fn open(&mut self, params: OpenParams) -> rocci_browser::Result<OpenResult> {
        let args = run_args(Path::new(&params.root), params.document.as_deref());
        let (child, opened) = spawn_run_no_window(&self.bin.display().to_string(), &args)?;
        self.children.push(child);
        Ok(opened)
    }

    fn shutdown(&mut self) -> rocci_browser::Result<()> {
        for child in &mut self.children {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.children.clear();
        Ok(())
    }
}

fn label_for(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bundle")
        .to_string()
}

fn is_bundle_root(path: &Path) -> bool {
    let index = if path.is_dir() {
        path.join("index.md")
    } else {
        return false;
    };
    let Ok(source) = fs::read_to_string(index) else {
        return false;
    };
    let Ok(Some(frontmatter)) = split_frontmatter(&source, false) else {
        return false;
    };
    let Ok(metadata) = parse_yaml_mapping(frontmatter.yaml.of(&source)) else {
        return false;
    };
    metadata
        .get("okf_version")
        .and_then(serde_json::Value::as_str)
        .is_some()
}

fn walk_concepts(root: &Path) -> Vec<Document> {
    let mut documents = Vec::new();
    walk_md(root, root, &mut documents);
    documents.sort_by(|a, b| a.id.cmp(&b.id));
    documents
}

fn walk_md(root: &Path, dir: &Path, documents: &mut Vec<Document>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            walk_md(root, &path, documents);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name == "index.md" || name == "log.md" {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let id = rel.strip_suffix(".md").unwrap_or(&rel).to_string();
        let title = title_from_file(&path).unwrap_or_else(|| id.clone());
        documents.push(Document {
            id: id.clone(),
            title,
            path: rel,
            route: Some(format!("/{id}/")),
        });
    }
}

fn title_from_file(path: &Path) -> Option<String> {
    let source = fs::read_to_string(path).ok()?;
    source.lines().find_map(|line| {
        line.trim()
            .strip_prefix("# ")
            .map(|title| title.trim().to_string())
    })
}

pub fn run_args(root: &Path, document: Option<&str>) -> Vec<String> {
    let target = match document {
        Some(document) if document.ends_with(".md") => root.join(document),
        Some(document) => root.join(format!("{document}.md")),
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
            "okf-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn probe_true_false_and_list_ids() {
        let dir = temp();
        let mut adapter = OkfAdapter {
            bin: PathBuf::from("rocci-okf"),
            children: Vec::new(),
        };
        assert!(!adapter.probe(&dir.display().to_string()).unwrap().claimed);
        fs::write(
            dir.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Bundle\n",
        )
        .unwrap();
        fs::create_dir_all(dir.join("plans")).unwrap();
        fs::write(dir.join("plans/example.md"), "# Example\n").unwrap();
        assert!(adapter.probe(&dir.display().to_string()).unwrap().claimed);
        let listed = adapter.list_documents(&dir.display().to_string()).unwrap();
        assert_eq!(listed.documents.len(), 1);
        assert_eq!(listed.documents[0].id, "plans/example");
        assert!(
            adapter
                .probe(&dir.join("plans/example.md").display().to_string())
                .unwrap()
                .claimed
        );
        assert_eq!(
            run_args(&dir, Some("plans/example"))[1],
            dir.join("plans/example.md").display().to_string()
        );
    }
}
