use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::ast::{BuildSummary, Bundle, Concept, Heading, Link, TrustTier};
use crate::diagnostic::SourceLocation;
use crate::search::{concept_is_stale, concept_trust_tier, search_index};
use crate::validate::string_field;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
pub struct ConceptInspect<'a> {
    pub id: &'a str,
    pub path: &'a str,
    pub metadata: &'a BTreeMap<String, Value>,
    pub trust_tier: TrustTier,
    pub stale: bool,
    pub body_span: SourceLocation,
    pub headings: &'a [Heading],
    pub links: &'a [Link],
    pub source_ids: &'a BTreeSet<String>,
    pub footnote_ids: &'a BTreeSet<String>,
}

impl<'a> From<&'a Concept> for ConceptInspect<'a> {
    fn from(concept: &'a Concept) -> Self {
        Self {
            id: &concept.id,
            path: &concept.path,
            metadata: &concept.metadata,
            trust_tier: concept_trust_tier(&concept.metadata),
            stale: concept_is_stale(&concept.metadata),
            body_span: concept.body_location.clone(),
            headings: &concept.headings,
            links: &concept.links,
            source_ids: &concept.source_ids,
            footnote_ids: &concept.footnote_ids,
        }
    }
}

pub fn build_artifacts(bundle: &Bundle, output: &Path) -> Result<BuildSummary> {
    let output = absolute(output)?;
    let staging = unique_temp("okf-stage")?;
    fs::create_dir_all(&staging)
        .with_context(|| format!("failed to create {}", staging.display()))?;

    let catalog = bundle
        .concepts
        .iter()
        .map(ConceptInspect::from)
        .collect::<Vec<_>>();
    fs::write(
        staging.join("catalog.json"),
        format!("{}\n", serde_json::to_string_pretty(&catalog)?),
    )
    .context("failed to write knowledge catalog")?;

    fs::write(
        staging.join("search.json"),
        format!("{}\n", serde_json::to_string_pretty(&search_index(bundle))?),
    )
    .context("failed to write knowledge search index")?;

    fs::write(staging.join("llms.txt"), llms_text(bundle))
        .context("failed to write knowledge llms index")?;

    fs::write(
        staging.join("validation.json"),
        format!("{}\n", serde_json::to_string_pretty(&bundle.diagnostics)?),
    )
    .context("failed to write knowledge validation report")?;

    commit_output(&staging, &output)?;

    Ok(BuildSummary {
        concepts: bundle.concepts.len(),
        indexes: bundle.indexes.len(),
        output: output.to_string_lossy().into_owned(),
    })
}

pub fn llms_text(bundle: &Bundle) -> String {
    let mut output = String::from(
        "# Rocci knowledge\n\n> Local, generated index. Canonical records remain in knowledge/**/*.md.\n\n",
    );
    for concept in &bundle.concepts {
        let title = string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        let description = string_field(&concept.metadata, "description").unwrap_or_default();
        let status = string_field(&concept.metadata, "status").unwrap_or("unknown");
        let authority = string_field(&concept.metadata, "authority").unwrap_or("unspecified");
        let trust = concept_trust_tier(&concept.metadata).as_str();
        let stale = if concept_is_stale(&concept.metadata) {
            ", stale"
        } else {
            ""
        };
        output.push_str(&format!(
            "## {title}\n\n- ID: `{}`\n- Lifecycle: `{status}`; authority: `{authority}`; trust: `{trust}`{stale}\n- URL: /{}/\n\n{description}\n\n",
            concept.id, concept.id
        ));
    }
    output
}

pub fn commit_output(staging: &Path, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let prev_name = match output.file_name() {
        Some(name) => {
            let mut prev = name.to_os_string();
            prev.push(".prev");
            prev
        }
        None => "output.prev".into(),
    };
    let prev = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(prev_name);
    if prev.exists() {
        fs::remove_dir_all(&prev).with_context(|| format!("failed to clear {}", prev.display()))?;
    }
    if output.exists() {
        fs::rename(output, &prev)
            .with_context(|| format!("failed to move {} aside", output.display()))?;
        if let Err(err) = fs::rename(staging, output) {
            let _ = fs::rename(&prev, output);
            return Err(err).with_context(|| {
                format!(
                    "failed to replace {} with staged okf output",
                    output.display()
                )
            });
        }
        let _ = fs::remove_dir_all(&prev);
    } else if let Err(err) = fs::rename(staging, output) {
        return Err(err)
            .with_context(|| format!("failed to move staged okf output to {}", output.display()));
    }
    Ok(())
}

pub fn unique_temp(prefix: &str) -> Result<PathBuf> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = env::temp_dir().join(format!("okf-{prefix}-{nonce}-{counter}"));
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create temp dir {}", path.display()))?;
    Ok(path)
}

pub fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}
