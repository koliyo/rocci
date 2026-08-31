use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::catalog::ResolvedPage;

pub const PLAYGROUND_CSP: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self' blob:; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'";
const PLAYGROUND_HTML_REASON: &str = "HTML preview is not available in WASM mode. The browser cannot dynamically compile generated Roc to WebAssembly.";

pub(crate) fn page_uses_playground(page: &ResolvedPage) -> bool {
    page.layout == "playground"
        || crate::docs::collect_kinds(&page.article)
            .iter()
            .any(|k| k == "playground")
}

pub(crate) fn playground_session_bytes(
    root: &Path,
    worker_url: &str,
    wasm_url: &str,
) -> Result<Vec<u8>> {
    let documents = load_playground_documents(root)?;
    let selected = documents
        .first()
        .and_then(|doc| doc.get("id"))
        .and_then(|id| id.as_str())
        .unwrap_or("counter")
        .to_string();
    Ok(serde_json::json!({
        "protocol_version": 1,
        "documents": documents,
        "selected_document": selected,
        "compiler_wasm_url": wasm_url,
        "worker_url": worker_url,
        "mode": "wasm",
        "compile_url": "",
        "native_languages": [],
        "html_runtime": {
            "available": false,
            "reason": PLAYGROUND_HTML_REASON,
        },
    })
    .to_string()
    .into_bytes())
}

#[derive(Debug, Deserialize)]
struct PlaygroundExamplesManifest {
    #[serde(default, rename = "example")]
    examples: Vec<PlaygroundExampleSpec>,
}

#[derive(Debug, Deserialize)]
struct PlaygroundExampleSpec {
    id: String,
    file: String,
    #[serde(default)]
    language: String,
}

fn default_playground_documents() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "counter",
            "filename": "Counter.rocci",
            "language": "rocci",
            "source": "@component Counter = |{ count }| {\n    <p>{count.to_str()}</p>\n}\n",
        }),
        serde_json::json!({
            "id": "guide",
            "filename": "Guide.rocdown",
            "language": "rocdown",
            "source": "# Guide\n\nHello from Rocdown.\n",
        }),
    ]
}

fn playground_language(spec: &PlaygroundExampleSpec, filename: &str) -> Result<&'static str> {
    let raw = if spec.language.is_empty() {
        Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
    } else {
        spec.language.as_str()
    };
    match raw {
        "rocci" => Ok("rocci"),
        "rocdown" | "md" | "markdown" => Ok("rocdown"),
        other => bail!(
            "playground example `{}` has unknown language `{other}`",
            spec.id
        ),
    }
}

fn load_playground_documents(root: &Path) -> Result<Vec<serde_json::Value>> {
    let manifest_path = root.join("playground/examples.toml");
    if !manifest_path.is_file() {
        return Ok(default_playground_documents());
    }
    let text = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PlaygroundExamplesManifest = toml::from_str(&text)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    if manifest.examples.is_empty() {
        bail!(
            "{} must list at least one [[example]]",
            manifest_path.display()
        );
    }
    let mut documents = Vec::new();
    for spec in &manifest.examples {
        if spec.id.trim().is_empty() {
            bail!("playground example id must not be empty");
        }
        let relative = Path::new(&spec.file);
        if relative.is_absolute() || spec.file.contains('\0') {
            bail!(
                "playground example `{}` path must be a relative file path",
                spec.id
            );
        }
        let path = root.join(relative);
        let source = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "failed to read playground example `{}` from {}",
                spec.id,
                path.display()
            )
        })?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("playground example `{}` has a non-utf8 filename", spec.id)
            })?
            .to_string();
        let language = playground_language(spec, &filename)?;
        documents.push(serde_json::json!({
            "id": spec.id,
            "filename": filename,
            "language": language,
            "source": source,
        }));
    }
    Ok(documents)
}
