use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use rocci_rocdown::{
    Document, Item, MarkdownBodyOptions, MdNode, SourceFile, Span, parse_markdown_body,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use yaml_rust::{Yaml, YamlLoader};

use crate::article::render_document;
use crate::build::{commit_output, unique_temp};
use crate::catalog::{CatalogDiagnostic, PageHeading, RouteHint, Severity, SourcePage};
use crate::config::{NavConfig, SiteConfig};
use crate::site::{LoadedSite, StaticFile};

const STANDARD_FIELDS: &[&str] = &[
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "sources",
    "usage_window",
    "generated",
    "verified",
    "status",
    "stale_after",
    "authority",
    "owners",
];

const PROFILE_TYPES: &[&str] = &[
    "Architecture",
    "Decision",
    "Specification",
    "Status",
    "Implementation Plan",
    "Research Report",
    "Audit",
    "Case Study",
    "Reference",
    "Design Standard",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Base,
    Rocci,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustTier {
    HumanReviewed,
    Generated,
    Unverified,
}

impl TrustTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HumanReviewed => "human-reviewed",
            Self::Generated => "generated",
            Self::Unverified => "unverified",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeFilter {
    pub types: Vec<String>,
    pub tags: Vec<String>,
    pub statuses: Vec<String>,
    pub authorities: Vec<String>,
    pub trust_tiers: Vec<TrustTier>,
    pub stale: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectKind {
    Catalog,
    Concept,
    Graph,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
    pub message: String,
}

impl Diagnostic {
    fn error(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            path: path.into(),
            location,
            message: message.into(),
        }
    }

    fn warning(
        code: &'static str,
        path: impl Into<String>,
        location: Option<SourceLocation>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            path: path.into(),
            location,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        if let Some(location) = &self.location {
            write!(
                f,
                "{} {severity} {}:{}:{}: {}",
                self.code, self.path, location.line, location.column, self.message
            )
        } else {
            write!(
                f,
                "{} {severity} {}: {}",
                self.code, self.path, self.message
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Link {
    pub url: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub raw: String,
    pub broken: bool,
}

#[derive(Debug, Clone)]
pub struct Concept {
    pub id: String,
    pub path: String,
    pub metadata: BTreeMap<String, Value>,
    pub body_span: Span,
    pub body_location: SourceLocation,
    pub document: Document,
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub source_ids: BTreeSet<String>,
    pub footnote_ids: BTreeSet<String>,
    pub article_html: String,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub path: String,
    pub version: Option<String>,
    pub body_span: Span,
    pub document: Document,
    pub article_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Log {
    pub path: String,
    pub body_span: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct Bundle {
    pub root: PathBuf,
    pub version: Option<String>,
    pub concepts: Vec<Concept>,
    pub indexes: Vec<Index>,
    pub logs: Vec<Log>,
    pub graph: Vec<Edge>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Bundle {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

pub struct CheckReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl CheckReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn terminal(&self) -> String {
        self.diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.diagnostics)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildSummary {
    pub concepts: usize,
    pub indexes: usize,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchChunk {
    pub id: String,
    pub concept_id: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub concept_type: String,
    pub tags: Vec<String>,
    pub status: String,
    pub authority: String,
    pub trust_tier: TrustTier,
    pub stale: bool,
    pub url: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetrievalBenchmark {
    version: u32,
    top_k: usize,
    minimum_hit_rate: f64,
    questions: Vec<RetrievalQuestion>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetrievalQuestion {
    id: String,
    question: String,
    query: String,
    expected_concepts: Vec<String>,
    #[serde(default)]
    expected_status: Option<String>,
    #[serde(default)]
    expected_authority: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalReport {
    pub benchmark: String,
    pub top_k: usize,
    pub total: usize,
    pub passed: usize,
    pub hit_rate: f64,
    pub mean_reciprocal_rank: f64,
    pub minimum_hit_rate: f64,
    pub threshold_met: bool,
    pub questions: Vec<RetrievalQuestionResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RetrievalQuestionResult {
    pub id: String,
    pub question: String,
    pub query: String,
    pub expected_concepts: Vec<String>,
    pub returned_concepts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_relevant_rank: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_authority: Option<String>,
    pub lifecycle_matched: bool,
    pub passed: bool,
}

pub fn check(root: &Path, profile: Profile) -> Result<CheckReport> {
    let bundle = load(root, profile)?;
    Ok(CheckReport {
        diagnostics: bundle.diagnostics,
    })
}

pub fn load(root: &Path, profile: Profile) -> Result<Bundle> {
    let root = absolute(root)?;
    if !root.is_dir() {
        bail!("knowledge bundle {} is not a directory", root.display());
    }
    let mut paths = Vec::new();
    discover_markdown(&root, &mut paths)?;
    paths.sort();

    let mut concepts = Vec::new();
    let mut indexes = Vec::new();
    let mut logs = Vec::new();
    let mut diagnostics = Vec::new();

    for path in paths {
        let relative = relative_path(&root, &path);
        let bytes =
            fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let source = match String::from_utf8(bytes) {
            Ok(source) => source,
            Err(_) => {
                diagnostics.push(Diagnostic::error(
                    "OKF1001",
                    relative,
                    None,
                    "document is not valid UTF-8",
                ));
                continue;
            }
        };
        match path.file_name().and_then(|name| name.to_str()) {
            Some("index.md") => {
                parse_index(&root, &relative, &source, &mut indexes, &mut diagnostics)
            }
            Some("log.md") => parse_log(&relative, &source, &mut logs, &mut diagnostics),
            _ => parse_concept(&relative, &source, profile, &mut concepts, &mut diagnostics),
        }
    }

    concepts.sort_by(|a, b| a.id.cmp(&b.id));
    indexes.sort_by(|a, b| a.path.cmp(&b.path));
    logs.sort_by(|a, b| a.path.cmp(&b.path));
    validate_unique_ids(&concepts, &mut diagnostics);
    let graph = resolve_graph(&concepts, &indexes, &mut diagnostics);
    if profile == Profile::Rocci {
        validate_lifecycle_and_sources(&root, &concepts, &mut diagnostics);
    }
    diagnostics.sort_by(|a, b| {
        (&a.path, a.location.as_ref().map(|span| span.start), a.code).cmp(&(
            &b.path,
            b.location.as_ref().map(|span| span.start),
            b.code,
        ))
    });
    let version = indexes
        .iter()
        .find(|index| index.path == "index.md")
        .and_then(|index| index.version.clone());

    Ok(Bundle {
        root,
        version,
        concepts,
        indexes,
        logs,
        graph,
        diagnostics,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionKind {
    FixErrors,
    ReverifySources,
    ReverifyRegenerated,
    RefreshStale,
    UncommittedChanges,
    UntrackedSources,
    InitialVerification,
    PendingPromotion,
    Exploratory,
    Clean,
}

#[derive(Debug, Clone)]
pub struct ConceptAction {
    pub kind: ActionKind,
    pub label: String,
    pub detail: String,
    pub is_action_required: bool,
    pub pill_class: &'static str,
}

pub fn classify_concept_action(
    concept: &Concept,
    bundle_diagnostics: &[Diagnostic],
) -> ConceptAction {
    let status = string_field(&concept.metadata, "status").unwrap_or("draft");
    let authority = string_field(&concept.metadata, "authority").unwrap_or("descriptive");
    let stale = concept_is_stale(&concept.metadata);
    let stale_after = string_field(&concept.metadata, "stale_after").unwrap_or("");

    let concept_diagnostics: Vec<&Diagnostic> = bundle_diagnostics
        .iter()
        .filter(|d| d.path == concept.path)
        .collect();

    let has_errors = concept_diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error);
    if has_errors {
        return ConceptAction {
            kind: ActionKind::FixErrors,
            label: "Fix Errors".into(),
            detail: "Validation errors in record must be resolved".into(),
            is_action_required: true,
            pill_class: "pill-error",
        };
    }

    if stale {
        return ConceptAction {
            kind: ActionKind::RefreshStale,
            label: "Refresh Stale".into(),
            detail: format!("Record reached stale_after limit ({stale_after})"),
            is_action_required: true,
            pill_class: "pill-action",
        };
    }

    let drift_diagnostics: Vec<&Diagnostic> = concept_diagnostics
        .iter()
        .copied()
        .filter(|d| d.code == "OKF4006")
        .collect();

    if !drift_diagnostics.is_empty() {
        let drifted_sources: Vec<String> = drift_diagnostics
            .iter()
            .map(|d| {
                if let Some(start) = d.message.find("source `") {
                    if let Some(end) = d.message[start + 8..].find('`') {
                        return d.message[start + 8..start + 8 + end].to_string();
                    }
                }
                d.message.clone()
            })
            .collect();
        return ConceptAction {
            kind: ActionKind::ReverifySources,
            label: "Re-verify".into(),
            detail: format!(
                "{} source(s) changed after human verification ({})",
                drift_diagnostics.len(),
                drifted_sources.join(", ")
            ),
            is_action_required: true,
            pill_class: "pill-action",
        };
    }

    let regenerated_diag = concept_diagnostics.iter().any(|d| d.code == "OKF4005");
    if regenerated_diag {
        return ConceptAction {
            kind: ActionKind::ReverifyRegenerated,
            label: "Re-verify".into(),
            detail: "Substantively modified by generation after last human verification".into(),
            is_action_required: true,
            pill_class: "pill-action",
        };
    }

    let uncommitted_diag = concept_diagnostics.iter().any(|d| d.code == "OKF4008");
    if uncommitted_diag {
        return ConceptAction {
            kind: ActionKind::UncommittedChanges,
            label: "Commit Evidence".into(),
            detail: "Local working tree contains uncommitted source edits".into(),
            is_action_required: true,
            pill_class: "pill-action",
        };
    }

    let untracked_diag = concept_diagnostics.iter().any(|d| d.code == "OKF4007");
    if untracked_diag {
        return ConceptAction {
            kind: ActionKind::UntrackedSources,
            label: "Track Evidence".into(),
            detail: "Cited source is untracked with no git provenance".into(),
            is_action_required: true,
            pill_class: "pill-action",
        };
    }

    if status == "draft" && authority == "exploratory" {
        return ConceptAction {
            kind: ActionKind::Exploratory,
            label: "Exploratory".into(),
            detail: "Intentional draft exploratory research (no action required)".into(),
            is_action_required: false,
            pill_class: "pill-info",
        };
    }

    let has_human_verification = concept
        .metadata
        .get("verified")
        .and_then(Value::as_array)
        .is_some_and(|arr| {
            arr.iter().any(|v| {
                v.get("by")
                    .and_then(Value::as_str)
                    .is_some_and(|actor| actor.starts_with("human:"))
            })
        });

    if status == "draft" {
        if !has_human_verification {
            return ConceptAction {
                kind: ActionKind::InitialVerification,
                label: "Verify".into(),
                detail: "Initial human review and verification needed to stabilize".into(),
                is_action_required: true,
                pill_class: "pill-action",
            };
        } else {
            return ConceptAction {
                kind: ActionKind::PendingPromotion,
                label: "Verify".into(),
                detail: "Draft revision pending human verification to promote to stable".into(),
                is_action_required: true,
                pill_class: "pill-action",
            };
        }
    }

    if status == "stable" {
        return ConceptAction {
            kind: ActionKind::Clean,
            label: "Stable".into(),
            detail: "Human-verified and in sync with repository evidence".into(),
            is_action_required: false,
            pill_class: "pill-clean",
        };
    }

    ConceptAction {
        kind: ActionKind::Clean,
        label: status.to_string(),
        detail: "Archived or historical record".into(),
        is_action_required: false,
        pill_class: "pill-info",
    }
}

pub fn render_concept_meta(concept: &Concept, bundle_diagnostics: &[Diagnostic]) -> String {
    let status = string_field(&concept.metadata, "status").unwrap_or("draft");
    let authority = string_field(&concept.metadata, "authority").unwrap_or("descriptive");
    let trust_tier = concept_trust_tier(&concept.metadata);
    let stale = concept_is_stale(&concept.metadata);
    let stale_after = string_field(&concept.metadata, "stale_after").unwrap_or("");
    let concept_type = string_field(&concept.metadata, "type").unwrap_or("Concept");
    let owners = metadata_string_array(&concept.metadata, "owners");
    let tags = metadata_string_array(&concept.metadata, "tags");

    let action = classify_concept_action(concept, bundle_diagnostics);

    let (trust_slug, trust_label) = match trust_tier {
        TrustTier::HumanReviewed => ("human", "human-reviewed"),
        TrustTier::Generated => ("generated", "generated"),
        TrustTier::Unverified => ("unverified", "unverified"),
    };

    let latest_verification = latest_human_verification(&concept.metadata);
    let generated_by = concept
        .metadata
        .get("generated")
        .and_then(Value::as_object)
        .and_then(|g| g.get("by"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let generated_at = concept
        .metadata
        .get("generated")
        .and_then(Value::as_object)
        .and_then(|g| g.get("at"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let mut out = String::new();
    out.push_str(
        "<section class=\"okf-concept-meta\" aria-label=\"Concept Metadata &amp; Governance\">\n",
    );
    out.push_str("  <div class=\"okf-badge-group\">\n");
    out.push_str(&format!(
        "    <span class=\"okf-badge okf-type\">{}</span>\n",
        escape(concept_type)
    ));
    out.push_str(&format!(
        "    <span class=\"okf-badge okf-status okf-status-{}\">{}</span>\n",
        escape(status),
        escape(status)
    ));
    out.push_str(&format!(
        "    <span class=\"okf-badge okf-auth okf-auth-{}\">{}</span>\n",
        escape(authority),
        escape(authority)
    ));
    out.push_str(&format!(
        "    <span class=\"okf-badge okf-trust okf-trust-{}\">{}</span>\n",
        trust_slug, trust_label
    ));
    if stale {
        out.push_str(&format!(
            "    <span class=\"okf-badge okf-badge-stale\">Stale (expired {})</span>\n",
            escape(stale_after)
        ));
    }
    out.push_str("  </div>\n");

    if action.is_action_required {
        out.push_str("  <div class=\"okf-alert-banner\" role=\"alert\">\n");
        out.push_str("    <span class=\"okf-alert-icon\" aria-hidden=\"true\">⚠️</span>\n");
        out.push_str("    <div class=\"okf-alert-content\">\n");
        out.push_str(&format!(
            "      <strong>Review Action Required:</strong> {}\n",
            escape(&action.detail)
        ));
        out.push_str("    </div>\n");
        out.push_str("  </div>\n");
    }

    out.push_str("  <div class=\"okf-meta-grid\">\n");
    if !owners.is_empty() {
        out.push_str(&format!(
            "    <div><span class=\"okf-meta-label\">Owners:</span> <code>{}</code></div>\n",
            escape(&owners.join(", "))
        ));
    }
    if let Some((_, verifier_str)) = latest_verification {
        out.push_str(&format!(
            "    <div><span class=\"okf-meta-label\">Verified:</span> <code>{}</code></div>\n",
            escape(verifier_str)
        ));
    } else {
        out.push_str(
            "    <div><span class=\"okf-meta-label\">Verified:</span> <em>Unverified</em></div>\n",
        );
    }
    if !generated_by.is_empty() {
        out.push_str(&format!(
            "    <div><span class=\"okf-meta-label\">Generated:</span> <code>{} @ {}</code></div>\n",
            escape(generated_by),
            escape(generated_at)
        ));
    }
    if !stale_after.is_empty() {
        out.push_str(&format!(
            "    <div><span class=\"okf-meta-label\">Stale after:</span> <code>{}</code></div>\n",
            escape(stale_after)
        ));
    }
    out.push_str("  </div>\n");

    if let Some(sources) = concept.metadata.get("sources").and_then(Value::as_array) {
        if !sources.is_empty() {
            let drift_diags: Vec<&Diagnostic> = bundle_diagnostics
                .iter()
                .filter(|d| d.path == concept.path && d.code == "OKF4006")
                .collect();
            let drift_summary = if !drift_diags.is_empty() {
                format!("({} drifted)", drift_diags.len())
            } else {
                "(all clean)".to_string()
            };

            out.push_str("  <details class=\"okf-sources-drawer\">\n");
            out.push_str(&format!(
                "    <summary><strong>{} Cited Sources</strong> <span class=\"okf-sources-drift-note\">{}</span></summary>\n",
                sources.len(),
                escape(&drift_summary)
            ));
            out.push_str("    <table class=\"okf-sources-table\">\n");
            out.push_str("      <thead><tr><th>ID</th><th>Resource</th><th>Author</th><th>Status</th></tr></thead>\n");
            out.push_str("      <tbody>\n");
            for source in sources {
                let s_id = source.get("id").and_then(Value::as_str).unwrap_or("-");
                let s_res = source
                    .get("resource")
                    .and_then(Value::as_str)
                    .unwrap_or("-");
                let s_author = source.get("author").and_then(Value::as_str).unwrap_or("-");
                let is_drifted = drift_diags
                    .iter()
                    .any(|d| d.message.contains(&format!("`{s_id}`")));
                let status_badge = if is_drifted {
                    "<span class=\"okf-badge okf-status-draft\">Modified since verification</span>"
                } else {
                    "<span class=\"okf-badge okf-status-stable\">Clean</span>"
                };
                out.push_str(&format!(
                    "        <tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>\n",
                    escape(s_id),
                    escape(s_res),
                    escape(s_author),
                    status_badge
                ));
            }
            out.push_str("      </tbody>\n");
            out.push_str("    </table>\n");
            out.push_str("  </details>\n");
        }
    }

    if !tags.is_empty() {
        out.push_str("  <div class=\"okf-tags\">\n");
        for tag in tags {
            out.push_str(&format!(
                "    <span class=\"okf-tag\">#{}</span>\n",
                escape(&tag)
            ));
        }
        out.push_str("  </div>\n");
    }

    out.push_str("</section>\n\n");
    out
}

const PRIORITY_1_RECORDS: &[(&str, &str)] = &[
    (
        "architecture/rocdown-format",
        "Parser/README precedence over original report; root HTML template islands",
    ),
    (
        "architecture/rocs-documentation-compiler",
        "Ordinary Rocs compiler plus isolated OKF preview/retrieval path",
    ),
    (
        "architecture/theming",
        "Two current surfaces versus DTCG research-only boundary",
    ),
    (
        "status/implementation",
        "Snapshot accuracy; shipped, approved, proposed separation",
    ),
    (
        "status/known-limitations",
        "Current absences; ordinary-site versus OKF search boundary",
    ),
    (
        "architecture/system-overview",
        "Workspace and product boundaries",
    ),
    (
        "decisions/pure-render-components",
        "Implemented render semantics versus application architecture",
    ),
    (
        "decisions/server-owned-state",
        "Current direction versus optional browser state",
    ),
    (
        "decisions/markdown-first-explicit-islands",
        "Implemented syntax boundary versus unimplemented @island",
    ),
    (
        "decisions/rust-catalog-rocci-shell",
        "Implemented ownership boundary and remaining splice path",
    ),
];

pub fn render_priority_1_queue(bundle: &Bundle) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"okf-priority-1-section\">\n");
    out.push_str("  <h2 class=\"rd-header-2\" id=\"priority-1-review-queue\">Priority-1 Review Queue (5 Actionable Records)</h2>\n");
    out.push_str("  <p class=\"rd-paragraph\">The evidence-based human review gate for stabilizing Rocci's core architectural and decision records. Below are the 10 priority-1 records, explicitly highlighting the <strong>5 records requiring human re-verification</strong>.</p>\n\n");

    out.push_str("  <div class=\"okf-table-container\">\n");
    out.push_str("    <table class=\"okf-review-table\">\n");
    out.push_str("      <thead>\n");
    out.push_str("        <tr>\n");
    out.push_str("          <th style=\"width: 25%;\">Priority-1 Record</th>\n");
    out.push_str("          <th style=\"width: 27%;\">Review Focus</th>\n");
    out.push_str("          <th style=\"width: 15%;\">Lifecycle / Authority</th>\n");
    out.push_str("          <th style=\"width: 15%;\">Trust &amp; Verifier</th>\n");
    out.push_str("          <th style=\"width: 18%;\">Required Action</th>\n");
    out.push_str("        </tr>\n");
    out.push_str("      </thead>\n");
    out.push_str("      <tbody>\n");

    for &(id, focus) in PRIORITY_1_RECORDS {
        let Some(concept) = bundle.concepts.iter().find(|c| c.id == id) else {
            continue;
        };
        let title = string_field(&concept.metadata, "title").unwrap_or(id);
        let status = string_field(&concept.metadata, "status").unwrap_or("draft");
        let authority = string_field(&concept.metadata, "authority").unwrap_or("descriptive");
        let action = classify_concept_action(concept, &bundle.diagnostics);
        let trust_tier = concept_trust_tier(&concept.metadata);
        let (trust_slug, trust_label) = match trust_tier {
            TrustTier::HumanReviewed => ("human", "human-reviewed"),
            TrustTier::Generated => ("generated", "generated"),
            TrustTier::Unverified => ("unverified", "unverified"),
        };
        let verifier_text = if let Some((_, v_str)) = latest_human_verification(&concept.metadata) {
            v_str.to_string()
        } else {
            "None".to_string()
        };

        out.push_str("        <tr>\n");
        out.push_str(&format!(
            "          <td><a href=\"/{}/\" class=\"okf-concept-title-link\"><strong>{}</strong></a><div class=\"okf-concept-id-sub\"><code>{}</code></div></td>\n",
            concept.id,
            escape(title),
            escape(&concept.id)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-focus-text\">{}</span></td>\n",
            escape(focus)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-badge okf-status okf-status-{}\">{}</span> <span class=\"okf-badge okf-auth okf-auth-{}\">{}</span></td>\n",
            escape(status),
            escape(status),
            escape(authority),
            escape(authority)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-badge okf-trust okf-trust-{}\">{}</span><div class=\"okf-verifier-sub\"><code>{}</code></div></td>\n",
            trust_slug,
            trust_label,
            escape(&verifier_text)
        ));
        out.push_str(&format!(
            "          <td><div class=\"okf-action-wrapper\"><span class=\"okf-action-pill {}\">{}</span><div class=\"okf-action-detail-text\">{}</div></div></td>\n",
            action.pill_class,
            escape(&action.label),
            escape(&action.detail)
        ));
        out.push_str("        </tr>\n");
    }

    out.push_str("      </tbody>\n");
    out.push_str("    </table>\n");
    out.push_str("  </div>\n");
    out.push_str("</div>\n\n");
    out
}

pub fn render_home_page_governance(bundle: &Bundle) -> String {
    let total_concepts = bundle.concepts.len();
    let mut stable_count = 0;
    let mut draft_count = 0;
    let mut action_count = 0;
    let mut stale_count = 0;

    for concept in &bundle.concepts {
        let status = string_field(&concept.metadata, "status").unwrap_or("draft");
        let stale = concept_is_stale(&concept.metadata);
        if stale {
            stale_count += 1;
        }
        if status == "stable" {
            stable_count += 1;
        } else {
            draft_count += 1;
        }
        let action = classify_concept_action(concept, &bundle.diagnostics);
        if action.is_action_required {
            action_count += 1;
        }
    }
    let warnings_count = bundle.diagnostics.len();

    let mut out = String::new();
    out.push_str("<div class=\"okf-home-governance\">\n");
    out.push_str("  <div class=\"okf-stat-grid\">\n");
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Total Concepts</div></div>\n",
        total_concepts
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Stable</div></div>\n",
        stable_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Draft</div></div>\n",
        draft_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card is-action\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Action Required</div></div>\n",
        action_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Stale Records</div></div>\n",
        stale_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Diagnostics</div></div>\n",
        warnings_count
    ));
    out.push_str("  </div>\n\n");

    out.push_str(&render_priority_1_queue(bundle));

    out.push_str("  <div class=\"okf-cta-row\">\n");
    out.push_str("    <a href=\"/review/\" class=\"okf-cta-btn\">Open Complete Review Queue (All 19 Concepts) &rarr;</a>\n");
    out.push_str("    <a href=\"/reference/priority-1-review/\" class=\"okf-secondary-link\">View Priority-1 Review Checklist &rarr;</a>\n");
    out.push_str("  </div>\n");
    out.push_str("  <hr class=\"rd-hr\" />\n");
    out.push_str(
        "  <h2 class=\"rd-header-2\" id=\"knowledge-collections\">Knowledge Collections</h2>\n",
    );
    out.push_str("</div>\n\n");
    out
}

pub fn render_review_page(bundle: &Bundle) -> String {
    let total_concepts = bundle.concepts.len();
    let mut stable_count = 0;
    let mut draft_count = 0;
    let mut action_count = 0;
    let mut stale_count = 0;

    struct RowData<'a> {
        concept: &'a Concept,
        action: ConceptAction,
        status: &'a str,
        authority: &'a str,
        concept_type: &'a str,
        trust_slug: &'static str,
        trust_label: &'static str,
        verifier_text: String,
    }

    let mut rows = Vec::new();
    for concept in &bundle.concepts {
        let status = string_field(&concept.metadata, "status").unwrap_or("draft");
        let authority = string_field(&concept.metadata, "authority").unwrap_or("descriptive");
        let concept_type = string_field(&concept.metadata, "type").unwrap_or("Concept");
        let stale = concept_is_stale(&concept.metadata);
        if stale {
            stale_count += 1;
        }
        if status == "stable" {
            stable_count += 1;
        } else {
            draft_count += 1;
        }

        let action = classify_concept_action(concept, &bundle.diagnostics);
        if action.is_action_required {
            action_count += 1;
        }

        let trust_tier = concept_trust_tier(&concept.metadata);
        let (trust_slug, trust_label) = match trust_tier {
            TrustTier::HumanReviewed => ("human", "human-reviewed"),
            TrustTier::Generated => ("generated", "generated"),
            TrustTier::Unverified => ("unverified", "unverified"),
        };

        let verifier_text = if let Some((_, v_str)) = latest_human_verification(&concept.metadata) {
            v_str.to_string()
        } else {
            "None".to_string()
        };

        rows.push(RowData {
            concept,
            action,
            status,
            authority,
            concept_type,
            trust_slug,
            trust_label,
            verifier_text,
        });
    }

    let warnings_count = bundle.diagnostics.len();

    let mut out = String::new();
    out.push_str("<div class=\"okf-review-view\">\n");
    out.push_str("  <h1 class=\"rd-header-1\">Knowledge Governance &amp; Review Queue</h1>\n");
    out.push_str("  <p class=\"rd-paragraph\">Deterministic overview of bundle lifecycle status, trust tiers, source drift, and required human actions.</p>\n\n");

    out.push_str("  <div class=\"okf-stat-grid\">\n");
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Total Concepts</div></div>\n",
        total_concepts
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Stable</div></div>\n",
        stable_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Draft</div></div>\n",
        draft_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card is-action\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Action Required</div></div>\n",
        action_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Stale Records</div></div>\n",
        stale_count
    ));
    out.push_str(&format!(
        "    <div class=\"okf-stat-card\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">Diagnostics</div></div>\n",
        warnings_count
    ));
    out.push_str("  </div>\n\n");

    out.push_str(&render_priority_1_queue(bundle));

    out.push_str("  <h2 class=\"rd-header-2\" id=\"all-concepts-queue\">2. All Bundle Concepts (19 Records)</h2>\n");
    out.push_str("  <div class=\"okf-filter-bar\">\n");
    out.push_str(&format!(
        "    <button type=\"button\" class=\"okf-filter-btn is-active\" data-filter=\"all\">All ({})</button>\n",
        total_concepts
    ));
    out.push_str(&format!(
        "    <button type=\"button\" class=\"okf-filter-btn\" data-filter=\"action\">Needs Action ({})</button>\n",
        action_count
    ));
    out.push_str(&format!(
        "    <button type=\"button\" class=\"okf-filter-btn\" data-filter=\"draft\">Draft ({})</button>\n",
        draft_count
    ));
    out.push_str(&format!(
        "    <button type=\"button\" class=\"okf-filter-btn\" data-filter=\"stable\">Stable ({})</button>\n",
        stable_count
    ));
    out.push_str("    <input type=\"search\" id=\"okf-search-input\" class=\"okf-search-input\" placeholder=\"Filter by concept, path, tag, or action...\" aria-label=\"Search review queue\" />\n");
    out.push_str("  </div>\n\n");

    out.push_str("  <div class=\"okf-table-container\">\n");
    out.push_str("    <table class=\"okf-review-table\" id=\"okf-review-table\">\n");
    out.push_str("      <thead>\n");
    out.push_str("        <tr>\n");
    out.push_str("          <th style=\"width: 28%;\">Concept</th>\n");
    out.push_str("          <th style=\"width: 14%;\">Collection</th>\n");
    out.push_str("          <th style=\"width: 18%;\">Lifecycle / Authority</th>\n");
    out.push_str("          <th style=\"width: 18%;\">Trust &amp; Verification</th>\n");
    out.push_str("          <th style=\"width: 22%;\">Required Action</th>\n");
    out.push_str("        </tr>\n");
    out.push_str("      </thead>\n");
    out.push_str("      <tbody>\n");

    for row in &rows {
        let title = string_field(&row.concept.metadata, "title").unwrap_or(&row.concept.id);
        let tags = metadata_string_array(&row.concept.metadata, "tags");
        let search_haystack = format!(
            "{} {} {} {} {} {} {}",
            title,
            row.concept.id,
            row.concept_type,
            row.status,
            row.authority,
            row.action.detail,
            tags.join(" ")
        )
        .to_lowercase();

        out.push_str(&format!(
            "        <tr class=\"okf-row\" data-status=\"{}\" data-action=\"{}\" data-search=\"{}\">\n",
            escape(row.status),
            if row.action.is_action_required { "true" } else { "false" },
            escape(&search_haystack)
        ));
        out.push_str(&format!(
            "          <td><a href=\"/{}/\" class=\"okf-concept-title-link\"><strong>{}</strong></a><div class=\"okf-concept-id-sub\"><code>{}</code></div></td>\n",
            row.concept.id,
            escape(title),
            escape(&row.concept.id)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-badge okf-type\">{}</span></td>\n",
            escape(row.concept_type)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-badge okf-status okf-status-{}\">{}</span> <span class=\"okf-badge okf-auth okf-auth-{}\">{}</span></td>\n",
            escape(row.status),
            escape(row.status),
            escape(row.authority),
            escape(row.authority)
        ));
        out.push_str(&format!(
            "          <td><span class=\"okf-badge okf-trust okf-trust-{}\">{}</span><div class=\"okf-verifier-sub\"><code>{}</code></div></td>\n",
            row.trust_slug,
            row.trust_label,
            escape(&row.verifier_text)
        ));
        out.push_str(&format!(
            "          <td><div class=\"okf-action-wrapper\"><span class=\"okf-action-pill {}\">{}</span><div class=\"okf-action-detail-text\">{}</div></div></td>\n",
            row.action.pill_class,
            escape(&row.action.label),
            escape(&row.action.detail)
        ));
        out.push_str("        </tr>\n");
    }

    out.push_str("      </tbody>\n");
    out.push_str("    </table>\n");
    out.push_str("  </div>\n\n");

    out.push_str("  <h2 class=\"rd-header-2\" id=\"diagnostics\">Active Bundle Diagnostics</h2>\n");
    if bundle.diagnostics.is_empty() {
        out.push_str("  <p class=\"rd-paragraph\">No validation errors or provenance warnings detected.</p>\n");
    } else {
        out.push_str("  <div class=\"okf-diagnostics-list\">\n");
        for diag in &bundle.diagnostics {
            let sev_badge = match diag.severity {
                Severity::Error => "<span class=\"okf-badge okf-status-deprecated\">Error</span>",
                Severity::Warning => "<span class=\"okf-badge okf-status-draft\">Warning</span>",
            };
            out.push_str(&format!(
                "    <div class=\"okf-diagnostic-item\">{} <code>{}</code> <strong>{}</strong>: {}</div>\n",
                sev_badge,
                escape(diag.code),
                escape(&diag.path),
                escape(&diag.message)
            ));
        }
        out.push_str("  </div>\n");
    }

    out.push_str(
        r#"  <script>
    (function() {
      var currentFilter = 'all';
      var searchQuery = '';
      var buttons = document.querySelectorAll('.okf-filter-btn');
      var searchInput = document.getElementById('okf-search-input');
      var rows = document.querySelectorAll('.okf-row');

      function updateRows() {
        for (var i = 0; i < rows.length; i++) {
          var row = rows[i];
          var status = row.getAttribute('data-status');
          var isAction = row.getAttribute('data-action') === 'true';
          var searchData = row.getAttribute('data-search') || '';

          var matchesFilter = true;
          if (currentFilter === 'action') {
            matchesFilter = isAction;
          } else if (currentFilter === 'draft') {
            matchesFilter = status === 'draft';
          } else if (currentFilter === 'stable') {
            matchesFilter = status === 'stable';
          }

          var matchesSearch = true;
          if (searchQuery.length > 0) {
            matchesSearch = searchData.indexOf(searchQuery) !== -1;
          }

          row.style.display = (matchesFilter && matchesSearch) ? '' : 'none';
        }
      }

      for (var i = 0; i < buttons.length; i++) {
        buttons[i].addEventListener('click', function(e) {
          for (var j = 0; j < buttons.length; j++) {
            buttons[j].classList.remove('is-active');
          }
          this.classList.add('is-active');
          currentFilter = this.getAttribute('data-filter') || 'all';
          updateRows();
        });
      }

      if (searchInput) {
        searchInput.addEventListener('input', function(e) {
          searchQuery = (e.target.value || '').trim().toLowerCase();
          updateRows();
        });
      }
    })();
  </script>
"#,
    );

    out.push_str("</div>\n");
    out
}

pub(crate) fn load_site(root: &Path, profile: Profile) -> Result<LoadedSite> {
    let bundle = load(root, profile)?;
    let mut config = SiteConfig::default();
    config.sidebar_tree = true;
    config.site.title = bundle
        .indexes
        .iter()
        .find(|index| index.path == "index.md")
        .and_then(|index| document_headings(&index.document).into_iter().next())
        .map(|heading| heading.text)
        .unwrap_or_else(|| "Knowledge".into());
    config.navigation = bundle
        .indexes
        .iter()
        .filter_map(|index| {
            let directory = index.path.strip_suffix("/index.md")?;
            let label = document_headings(&index.document)
                .into_iter()
                .next()
                .map(|heading| heading.text)
                .unwrap_or_else(|| directory.replace(['-', '_'], " "));
            Some(NavConfig {
                label,
                items: Vec::new(),
                directory: Some(directory.to_string()),
            })
        })
        .collect();

    config.navigation.insert(
        0,
        NavConfig {
            label: "Governance & Review".into(),
            items: vec!["review".into()],
            directory: None,
        },
    );

    let routes = bundle
        .indexes
        .iter()
        .map(|index| {
            let id = index.path.strip_suffix(".md").unwrap_or(&index.path);
            (index.path.clone(), crate::catalog::derived_route(id))
        })
        .chain(bundle.concepts.iter().map(|concept| {
            (
                concept.path.clone(),
                crate::catalog::derived_route(&concept.id),
            )
        }))
        .chain(std::iter::once((
            "review.md".to_string(),
            crate::catalog::derived_route("review"),
        )))
        .collect::<BTreeMap<_, _>>();

    let mut files = BTreeSet::new();
    let mut static_files = Vec::new();
    collect_bundle_files(&bundle.root, &bundle.root, &mut files, &mut static_files)?;

    let mut sources = Vec::new();
    sources.push(SourcePage {
        id: "review".to_string(),
        id_explicit: true,
        source_path: "review.md".to_string(),
        route_hint: RouteHint::Derived,
        aliases: Vec::new(),
        draft: false,
        title: "Knowledge Governance & Review Queue".to_string(),
        description: "Deterministic overview of bundle lifecycle status, trust tiers, source drift, and required actions.".to_string(),
        headings: vec![
            PageHeading {
                level: 2,
                id: "summary-metrics".to_string(),
                text: "Summary Metrics".to_string(),
            },
            PageHeading {
                level: 2,
                id: "priority-1-review-queue".to_string(),
                text: "Priority-1 Review Queue".to_string(),
            },
            PageHeading {
                level: 2,
                id: "all-concepts-queue".to_string(),
                text: "All Bundle Concepts".to_string(),
            },
            PageHeading {
                level: 2,
                id: "diagnostics".to_string(),
                text: "Active Bundle Diagnostics".to_string(),
            },
        ],
        outgoing_links: Vec::new(),
        image_urls: Vec::new(),
        article_html: render_review_page(&bundle),
        docs: crate::docs::PageDocs::default(),
    });

    for index in &bundle.indexes {
        let id = index
            .path
            .strip_suffix(".md")
            .unwrap_or(&index.path)
            .to_string();
        let headings = document_headings(&index.document);
        let title = headings
            .first()
            .map(|heading| heading.text.clone())
            .unwrap_or_else(|| id.clone());
        let document = rewrite_document_urls(&index.document, &index.path, &routes, &files);
        let mut article_html = render_document(&document);
        if index.path == "index.md" {
            let governance_header = render_home_page_governance(&bundle);
            article_html = format!("{governance_header}{article_html}");
        }
        sources.push(SourcePage {
            id,
            id_explicit: false,
            source_path: index.path.clone(),
            route_hint: RouteHint::Derived,
            aliases: Vec::new(),
            draft: false,
            title,
            description: String::new(),
            headings,
            outgoing_links: Vec::new(),
            image_urls: Vec::new(),
            article_html,
            docs: crate::docs::PageDocs::default(),
        });
    }
    for concept in &bundle.concepts {
        let document = rewrite_document_urls(&concept.document, &concept.path, &routes, &files);
        let rendered_doc = render_document(&document);
        let meta_header = render_concept_meta(concept, &bundle.diagnostics);
        let article_html = format!("{meta_header}{rendered_doc}");
        sources.push(SourcePage {
            id: concept.id.clone(),
            id_explicit: true,
            source_path: concept.path.clone(),
            route_hint: RouteHint::Derived,
            aliases: Vec::new(),
            draft: false,
            title: string_field(&concept.metadata, "title")
                .unwrap_or(&concept.id)
                .to_string(),
            description: string_field(&concept.metadata, "description")
                .unwrap_or_default()
                .to_string(),
            headings: concept
                .headings
                .iter()
                .map(|heading| PageHeading {
                    level: heading.level,
                    id: heading.id.clone(),
                    text: heading.text.clone(),
                })
                .collect(),
            outgoing_links: Vec::new(),
            image_urls: Vec::new(),
            article_html,
            docs: crate::docs::PageDocs::default(),
        });
    }
    if config.navigation.is_empty() {
        config.navigation.push(NavConfig {
            label: config.site.title.clone(),
            items: sources.iter().map(|page| page.id.clone()).collect(),
            directory: None,
        });
    }

    Ok(LoadedSite {
        root: bundle.root,
        config,
        sources,
        files,
        static_files,
        diagnostics: bundle
            .diagnostics
            .iter()
            .map(|diagnostic| CatalogDiagnostic {
                code: diagnostic.code,
                severity: diagnostic.severity,
                path: diagnostic.path.clone(),
                message: diagnostic.message.clone(),
            })
            .collect(),
    })
}

fn document_headings(document: &Document) -> Vec<PageHeading> {
    fn walk(node: &MdNode, headings: &mut Vec<PageHeading>) {
        if let MdNode::Heading {
            level,
            id,
            children,
            ..
        } = node
        {
            headings.push(PageHeading {
                level: *level,
                id: id.clone(),
                text: children.iter().map(MdNode::text_content).collect(),
            });
        }
        for child in node_children(node) {
            walk(child, headings);
        }
    }

    let mut headings = Vec::new();
    for item in &document.items {
        if let Item::Markdown(node) = item {
            walk(node, &mut headings);
        }
    }
    headings
}

fn rewrite_document_urls(
    document: &Document,
    source_path: &str,
    routes: &BTreeMap<String, String>,
    files: &BTreeSet<String>,
) -> Document {
    fn rewritten_url(
        url: &str,
        source_path: &str,
        routes: &BTreeMap<String, String>,
        files: &BTreeSet<String>,
    ) -> String {
        if url.starts_with('#') || external_url(url) {
            return url.to_string();
        }
        let (path, fragment) = split_fragment(url);
        let Some(resolved) = resolve_bundle_path(source_path, path) else {
            return url.to_string();
        };
        let index = resolved
            .strip_suffix('/')
            .map(|directory| format!("{directory}/index.md"));
        let target = routes
            .get(&resolved)
            .or_else(|| index.as_ref().and_then(|index| routes.get(index)))
            .cloned()
            .or_else(|| files.contains(&resolved).then(|| format!("/{resolved}")));
        let Some(mut target) = target else {
            return url.to_string();
        };
        if let Some(fragment) = fragment {
            target.push('#');
            target.push_str(fragment);
        }
        target
    }

    fn walk(
        node: &mut MdNode,
        source_path: &str,
        routes: &BTreeMap<String, String>,
        files: &BTreeSet<String>,
    ) {
        match node {
            MdNode::Link { url, children, .. } => {
                *url = rewritten_url(url, source_path, routes, files);
                for child in children {
                    walk(child, source_path, routes, files);
                }
            }
            MdNode::Image { url, .. } => {
                *url = rewritten_url(url, source_path, routes, files);
            }
            _ => {
                for child in node.children_mut() {
                    walk(child, source_path, routes, files);
                }
            }
        }
    }

    let mut document = document.clone();
    for item in &mut document.items {
        if let Item::Markdown(node) = item {
            walk(node, source_path, routes, files);
        }
    }
    document
}

fn collect_bundle_files(
    root: &Path,
    directory: &Path,
    files: &mut BTreeSet<String>,
    static_files: &mut Vec<StaticFile>,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name().as_encoded_bytes().starts_with(b".") {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_bundle_files(root, &path, files, static_files)?;
            continue;
        }
        let relative = relative_path(root, &path);
        files.insert(relative.clone());
        if path.extension().is_none_or(|extension| extension != "md") {
            static_files.push(StaticFile {
                source: path,
                output_path: relative,
            });
        }
    }
    Ok(())
}

pub fn inspect(
    root: &Path,
    kind: InspectKind,
    target: Option<&str>,
    profile: Profile,
) -> Result<String> {
    inspect_filtered(root, kind, target, profile, &KnowledgeFilter::default())
}

pub fn inspect_filtered(
    root: &Path,
    kind: InspectKind,
    target: Option<&str>,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    let bundle = load(root, profile)?;
    match kind {
        InspectKind::Catalog => {
            let catalog = bundle
                .concepts
                .iter()
                .filter(|concept| filter.matches(concept))
                .map(ConceptInspect::from)
                .collect::<Vec<_>>();
            Ok(serde_json::to_string_pretty(&catalog)?)
        }
        InspectKind::Concept => {
            let target = target.filter(|target| !target.is_empty()).ok_or_else(|| {
                anyhow::anyhow!("knowledge inspect concept requires a concept id")
            })?;
            let target = target.strip_suffix(".md").unwrap_or(target);
            let concept = bundle
                .concepts
                .iter()
                .find(|concept| concept.id == target)
                .ok_or_else(|| anyhow::anyhow!("unknown knowledge concept `{target}`"))?;
            Ok(serde_json::to_string_pretty(&ConceptInspect::from(
                concept,
            ))?)
        }
        InspectKind::Graph => Ok(serde_json::to_string_pretty(&bundle.graph)?),
    }
}

pub fn search(
    root: &Path,
    query: &str,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    let bundle = load(root, profile)?;
    Ok(serde_json::to_string_pretty(&matching_search_chunks(
        &bundle, query, filter,
    ))?)
}

fn matching_search_chunks(
    bundle: &Bundle,
    query: &str,
    filter: &KnowledgeFilter,
) -> Vec<SearchChunk> {
    let terms = query
        .split_whitespace()
        .map(|term| term.to_lowercase())
        .collect::<Vec<_>>();
    search_index(bundle)
        .into_iter()
        .filter(|chunk| {
            let Some(concept) = bundle
                .concepts
                .iter()
                .find(|concept| concept.id == chunk.concept_id)
            else {
                return false;
            };
            if !filter.matches(concept) {
                return false;
            }
            let haystack = format!(
                "{} {} {} {} {}",
                chunk.title,
                chunk.description,
                chunk.heading.as_deref().unwrap_or_default(),
                chunk.text,
                chunk.tags.join(" ")
            )
            .to_lowercase();
            terms.iter().all(|term| haystack.contains(term))
        })
        .collect()
}

pub fn benchmark_retrieval(
    root: &Path,
    benchmark_path: &Path,
    profile: Profile,
) -> Result<RetrievalReport> {
    let bundle = load(root, profile)?;
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    let benchmark_source = fs::read_to_string(benchmark_path)
        .with_context(|| format!("failed to read {}", benchmark_path.display()))?;
    let benchmark: RetrievalBenchmark = toml::from_str(&benchmark_source)
        .with_context(|| format!("failed to parse {}", benchmark_path.display()))?;
    validate_retrieval_benchmark(&benchmark, &bundle)?;

    let mut reciprocal_rank = 0.0;
    let mut passed = 0;
    let mut questions = Vec::with_capacity(benchmark.questions.len());
    for question in benchmark.questions {
        let mut returned_concepts = Vec::new();
        for chunk in matching_search_chunks(&bundle, &question.query, &KnowledgeFilter::default()) {
            if !returned_concepts.contains(&chunk.concept_id) {
                returned_concepts.push(chunk.concept_id);
            }
            if returned_concepts.len() == benchmark.top_k {
                break;
            }
        }
        let first_relevant = returned_concepts
            .iter()
            .enumerate()
            .find(|(_, id)| question.expected_concepts.contains(id));
        let first_relevant_rank = first_relevant.map(|(index, _)| index + 1);
        if let Some(rank) = first_relevant_rank {
            reciprocal_rank += 1.0 / rank as f64;
        }
        let matched_concept = first_relevant.and_then(|(_, expected)| {
            bundle
                .concepts
                .iter()
                .find(|concept| concept.id.as_str() == expected.as_str())
        });
        let actual_status = matched_concept
            .and_then(|concept| string_field(&concept.metadata, "status"))
            .map(str::to_owned);
        let actual_authority = matched_concept
            .and_then(|concept| string_field(&concept.metadata, "authority"))
            .map(str::to_owned);
        let lifecycle_matched = matched_concept.is_some_and(|_| {
            question
                .expected_status
                .as_deref()
                .is_none_or(|status| actual_status.as_deref() == Some(status))
                && question
                    .expected_authority
                    .as_deref()
                    .is_none_or(|authority| actual_authority.as_deref() == Some(authority))
        });
        let question_passed = first_relevant_rank.is_some() && lifecycle_matched;
        if question_passed {
            passed += 1;
        }
        questions.push(RetrievalQuestionResult {
            id: question.id,
            question: question.question,
            query: question.query,
            expected_concepts: question.expected_concepts,
            returned_concepts,
            first_relevant_rank,
            expected_status: question.expected_status,
            expected_authority: question.expected_authority,
            actual_status,
            actual_authority,
            lifecycle_matched,
            passed: question_passed,
        });
    }
    let total = questions.len();
    let hit_rate = passed as f64 / total as f64;
    let mean_reciprocal_rank = reciprocal_rank / total as f64;
    Ok(RetrievalReport {
        benchmark: benchmark_path.to_string_lossy().into_owned(),
        top_k: benchmark.top_k,
        total,
        passed,
        hit_rate,
        mean_reciprocal_rank,
        minimum_hit_rate: benchmark.minimum_hit_rate,
        threshold_met: hit_rate >= benchmark.minimum_hit_rate,
        questions,
    })
}

fn validate_retrieval_benchmark(benchmark: &RetrievalBenchmark, bundle: &Bundle) -> Result<()> {
    if benchmark.version != 1 {
        bail!(
            "unsupported retrieval benchmark version {}; expected 1",
            benchmark.version
        );
    }
    if benchmark.top_k == 0 {
        bail!("retrieval benchmark top_k must be greater than zero");
    }
    if !(0.0..=1.0).contains(&benchmark.minimum_hit_rate) {
        bail!("retrieval benchmark minimum_hit_rate must be between 0 and 1");
    }
    if benchmark.questions.is_empty() {
        bail!("retrieval benchmark must contain at least one question");
    }
    let concept_ids = bundle
        .concepts
        .iter()
        .map(|concept| concept.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut question_ids = BTreeSet::new();
    for question in &benchmark.questions {
        if question.id.trim().is_empty()
            || question.question.trim().is_empty()
            || question.query.trim().is_empty()
        {
            bail!("retrieval benchmark question ids, text, and queries must be non-empty");
        }
        if !question_ids.insert(question.id.as_str()) {
            bail!(
                "duplicate retrieval benchmark question id `{}`",
                question.id
            );
        }
        if question.expected_concepts.is_empty() {
            bail!(
                "retrieval benchmark question `{}` has no expected concepts",
                question.id
            );
        }
        for concept in &question.expected_concepts {
            if !concept_ids.contains(concept.as_str()) {
                bail!(
                    "retrieval benchmark question `{}` references unknown concept `{concept}`",
                    question.id
                );
            }
        }
    }
    Ok(())
}

pub fn build(root: &Path, output: &Path, profile: Profile) -> Result<BuildSummary> {
    let bundle = load(root, profile)?;
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    let output = absolute(output)?;
    let staging = unique_temp("knowledge-stage")?;
    let site = staging.join("site");
    fs::create_dir_all(&site).with_context(|| format!("failed to create {}", site.display()))?;

    for concept in &bundle.concepts {
        let destination = site.join(&concept.id).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let title = string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        let meta_header = render_concept_meta(concept, &bundle.diagnostics);
        let full_html = format!("{meta_header}{}", &concept.article_html);
        fs::write(&destination, html_page(title, &full_html))
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        let governance_header = render_home_page_governance(&bundle);
        let full_index = format!("{governance_header}{}", &index.article_html);
        fs::write(site.join("index.html"), html_page("Knowledge", &full_index))
            .context("failed to write knowledge index")?;
    }
    let review_dest = site.join("review").join("index.html");
    if let Some(parent) = review_dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &review_dest,
        html_page(
            "Knowledge Governance & Review Queue",
            &render_review_page(&bundle),
        ),
    )
    .context("failed to write knowledge review page")?;
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
        format!(
            "{}\n",
            serde_json::to_string_pretty(&search_index(&bundle))?
        ),
    )
    .context("failed to write knowledge search index")?;
    fs::write(staging.join("llms.txt"), llms_text(&bundle))
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

#[derive(Serialize)]
struct ConceptInspect<'a> {
    id: &'a str,
    path: &'a str,
    metadata: &'a BTreeMap<String, Value>,
    trust_tier: TrustTier,
    stale: bool,
    body_span: SourceLocation,
    headings: &'a [Heading],
    links: &'a [Link],
    source_ids: &'a BTreeSet<String>,
    footnote_ids: &'a BTreeSet<String>,
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

impl KnowledgeFilter {
    fn matches(&self, concept: &Concept) -> bool {
        let metadata = &concept.metadata;
        let string_matches = |values: &[String], key: &str| {
            values.is_empty()
                || string_field(metadata, key).is_some_and(|actual| {
                    values
                        .iter()
                        .any(|expected| actual.eq_ignore_ascii_case(expected))
                })
        };
        if !string_matches(&self.types, "type")
            || !string_matches(&self.statuses, "status")
            || !string_matches(&self.authorities, "authority")
        {
            return false;
        }
        if !self.tags.is_empty() {
            let tags = metadata_string_array(metadata, "tags");
            if !self
                .tags
                .iter()
                .all(|expected| tags.iter().any(|actual| actual == expected))
            {
                return false;
            }
        }
        if !self.trust_tiers.is_empty() && !self.trust_tiers.contains(&concept_trust_tier(metadata))
        {
            return false;
        }
        if let Some(stale) = self.stale
            && concept_is_stale(metadata) != stale
        {
            return false;
        }
        true
    }
}

fn search_index(bundle: &Bundle) -> Vec<SearchChunk> {
    let mut chunks = Vec::new();
    for concept in &bundle.concepts {
        let title = string_field(&concept.metadata, "title")
            .unwrap_or(&concept.id)
            .to_string();
        let description = string_field(&concept.metadata, "description")
            .unwrap_or_default()
            .to_string();
        let concept_type = string_field(&concept.metadata, "type")
            .unwrap_or_default()
            .to_string();
        let tags = metadata_string_array(&concept.metadata, "tags");
        let status = string_field(&concept.metadata, "status")
            .unwrap_or_default()
            .to_string();
        let authority = string_field(&concept.metadata, "authority")
            .unwrap_or_default()
            .to_string();
        let trust_tier = concept_trust_tier(&concept.metadata);
        let stale = concept_is_stale(&concept.metadata);

        chunks.push(SearchChunk {
            id: format!("{}#metadata", concept.id),
            concept_id: concept.id.clone(),
            path: concept.path.clone(),
            kind: "metadata".into(),
            heading: None,
            title: title.clone(),
            description: description.clone(),
            concept_type: concept_type.clone(),
            tags: tags.clone(),
            status: status.clone(),
            authority: authority.clone(),
            trust_tier,
            stale,
            url: format!("/{}/", concept.id),
            text: normalize_search_text(&format!(
                "{title} {description} {concept_type} {}",
                tags.join(" ")
            )),
        });

        let mut current: Option<(String, String, Vec<String>)> = None;
        for item in &concept.document.items {
            let Item::Markdown(node) = item else {
                continue;
            };
            if let MdNode::Heading { id, children, .. } = node {
                if let Some((id, heading, text)) = current.take() {
                    chunks.push(search_heading_chunk(
                        concept,
                        &title,
                        &description,
                        &concept_type,
                        &tags,
                        &status,
                        &authority,
                        trust_tier,
                        stale,
                        id,
                        heading,
                        text,
                    ));
                }
                current = Some((
                    id.clone(),
                    children.iter().map(MdNode::text_content).collect(),
                    Vec::new(),
                ));
            } else if let Some((_, _, text)) = &mut current {
                let value = normalize_search_text(&node.text_content());
                if !value.is_empty() {
                    text.push(value);
                }
            }
        }
        if let Some((id, heading, text)) = current {
            chunks.push(search_heading_chunk(
                concept,
                &title,
                &description,
                &concept_type,
                &tags,
                &status,
                &authority,
                trust_tier,
                stale,
                id,
                heading,
                text,
            ));
        }
    }
    chunks
}

#[allow(clippy::too_many_arguments)]
fn search_heading_chunk(
    concept: &Concept,
    title: &str,
    description: &str,
    concept_type: &str,
    tags: &[String],
    status: &str,
    authority: &str,
    trust_tier: TrustTier,
    stale: bool,
    id: String,
    heading: String,
    text: Vec<String>,
) -> SearchChunk {
    SearchChunk {
        id: format!("{}#{id}", concept.id),
        concept_id: concept.id.clone(),
        path: concept.path.clone(),
        kind: "heading".into(),
        heading: Some(heading),
        title: title.to_string(),
        description: description.to_string(),
        concept_type: concept_type.to_string(),
        tags: tags.to_vec(),
        status: status.to_string(),
        authority: authority.to_string(),
        trust_tier,
        stale,
        url: format!("/{}/#{id}", concept.id),
        text: normalize_search_text(&text.join(" ")),
    }
}

fn normalize_search_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn llms_text(bundle: &Bundle) -> String {
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

fn concept_trust_tier(metadata: &BTreeMap<String, Value>) -> TrustTier {
    let generated_at = metadata
        .get("generated")
        .and_then(Value::as_object)
        .and_then(|generated| generated.get("at"))
        .and_then(Value::as_str)
        .and_then(parse_timestamp);
    let human_at = latest_human_verification(metadata).map(|(timestamp, _)| timestamp);
    match (generated_at, human_at) {
        (Some(generated), Some(human)) if human >= generated => TrustTier::HumanReviewed,
        (Some(_), _) => TrustTier::Generated,
        (None, Some(_)) => TrustTier::HumanReviewed,
        (None, None) => TrustTier::Unverified,
    }
}

fn concept_is_stale(metadata: &BTreeMap<String, Value>) -> bool {
    let Some(today) = current_utc_date() else {
        return false;
    };
    string_field(metadata, "stale_after").is_some_and(|date| is_date(date) && date < today.as_str())
}

fn metadata_string_array(metadata: &BTreeMap<String, Value>, key: &str) -> Vec<String> {
    metadata
        .get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn parse_concept(
    relative: &str,
    source: &str,
    profile: Profile,
    concepts: &mut Vec<Concept>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
    let frontmatter = match split_frontmatter(source, true) {
        Ok(Some(frontmatter)) => frontmatter,
        Ok(None) => unreachable!(),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(file, Span::point(0))),
                message,
            ));
            return;
        }
    };
    let metadata = match parse_yaml_mapping(frontmatter.yaml.of(source)) {
        Ok(metadata) => metadata,
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1003",
                relative,
                Some(location(file, frontmatter.yaml)),
                message,
            ));
            return;
        }
    };
    validate_metadata(
        relative,
        source,
        frontmatter.yaml,
        &metadata,
        profile,
        diagnostics,
    );
    reject_declarations(relative, source, frontmatter.body, diagnostics);

    let parsed = parse_markdown_body(
        file,
        frontmatter.body,
        MarkdownBodyOptions {
            raw_html: false,
            footnotes: true,
        },
    );
    for diagnostic in &parsed.diagnostics {
        diagnostics.push(Diagnostic::error(
            "OKF2008",
            relative,
            Some(location(file, diagnostic.span)),
            diagnostic.message.clone(),
        ));
    }
    let headings = parsed
        .headings
        .iter()
        .map(|heading| Heading {
            level: heading.level,
            id: heading.id.clone(),
            text: heading.text.clone(),
            location: location(file, heading.span),
        })
        .collect::<Vec<_>>();
    let links = parsed
        .links
        .iter()
        .map(|link| Link {
            url: link.url.clone(),
            location: location(file, link.span),
        })
        .collect::<Vec<_>>();
    let (footnote_ids, defined_footnotes) = collect_footnotes(&parsed.document);
    let source_ids = collect_source_ids(relative, &metadata, diagnostics);
    for footnote in &footnote_ids {
        if !source_ids.contains(footnote) {
            diagnostics.push(Diagnostic::error(
                "OKF4001",
                relative,
                None,
                format!("footnote `{footnote}` has no matching sources[].id"),
            ));
        }
        if !defined_footnotes.contains(footnote) {
            diagnostics.push(Diagnostic::error(
                "OKF4003",
                relative,
                None,
                format!("footnote `{footnote}` has no definition"),
            ));
        }
    }
    for source_id in source_ids.difference(&footnote_ids) {
        diagnostics.push(Diagnostic::warning(
            "OKF4002",
            relative,
            None,
            format!("source id `{source_id}` is not used by a body footnote"),
        ));
    }

    let id = relative.strip_suffix(".md").unwrap_or(relative).to_string();
    let article_html = render_document(&parsed.document);
    concepts.push(Concept {
        id,
        path: relative.to_string(),
        metadata,
        body_span: frontmatter.body,
        body_location: location(file, frontmatter.body),
        document: parsed.document,
        headings,
        links,
        source_ids,
        footnote_ids,
        article_html,
    });
}

fn parse_index(
    root: &Path,
    relative: &str,
    source: &str,
    indexes: &mut Vec<Index>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
    let is_root = root.join(relative) == root.join("index.md");
    let mut version = None;
    let body = match split_frontmatter(source, false) {
        Ok(Some(frontmatter)) if is_root => {
            match parse_yaml_mapping(frontmatter.yaml.of(source)) {
                Ok(metadata) => {
                    for key in metadata.keys() {
                        if key != "okf_version" {
                            diagnostics.push(Diagnostic::error(
                                "OKF1011",
                                relative,
                                Some(location(file, frontmatter.yaml)),
                                format!("root index frontmatter may only contain `okf_version`, found `{key}`"),
                            ));
                        }
                    }
                    match metadata.get("okf_version").and_then(Value::as_str) {
                        Some("0.2") => version = Some("0.2".to_string()),
                        Some(other) => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(file, frontmatter.yaml)),
                            format!("unsupported okf_version `{other}`"),
                        )),
                        None => diagnostics.push(Diagnostic::error(
                            "OKF1012",
                            relative,
                            Some(location(file, frontmatter.yaml)),
                            "okf_version must be a string",
                        )),
                    }
                }
                Err(message) => diagnostics.push(Diagnostic::error(
                    "OKF1003",
                    relative,
                    Some(location(file, frontmatter.yaml)),
                    message,
                )),
            }
            frontmatter.body
        }
        Ok(Some(frontmatter)) => {
            diagnostics.push(Diagnostic::error(
                "OKF1011",
                relative,
                Some(location(file, frontmatter.yaml)),
                "non-root index.md must not contain frontmatter",
            ));
            frontmatter.body
        }
        Ok(None) => Span::new(0, source.len()),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "OKF1002",
                relative,
                Some(location(file, Span::point(0))),
                message,
            ));
            return;
        }
    };
    let parsed = parse_markdown_body(
        file,
        body,
        MarkdownBodyOptions {
            raw_html: false,
            footnotes: true,
        },
    );
    for diagnostic in &parsed.diagnostics {
        diagnostics.push(Diagnostic::error(
            "OKF2008",
            relative,
            Some(location(file, diagnostic.span)),
            diagnostic.message.clone(),
        ));
    }
    indexes.push(Index {
        path: relative.to_string(),
        version,
        body_span: body,
        article_html: render_document(&parsed.document),
        document: parsed.document,
    });
}

fn parse_log(relative: &str, source: &str, logs: &mut Vec<Log>, diagnostics: &mut Vec<Diagnostic>) {
    let file = SourceFile::new(relative, source);
    if source.starts_with("---\n") || source.starts_with("---\r\n") {
        diagnostics.push(Diagnostic::error(
            "OKF1021",
            relative,
            Some(location(file, Span::point(0))),
            "log.md must not contain frontmatter",
        ));
    }
    for (offset, line) in lines_with_offsets(source) {
        if let Some(date) = line.trim_end_matches(['\r', '\n']).strip_prefix("## ")
            && !is_date(date)
        {
            diagnostics.push(Diagnostic::error(
                "OKF1022",
                relative,
                Some(location(file, Span::new(offset, offset + line.len()))),
                "log date headings must use YYYY-MM-DD",
            ));
        }
    }
    logs.push(Log {
        path: relative.to_string(),
        body_span: location(file, Span::new(0, source.len())),
    });
}

fn validate_metadata(
    relative: &str,
    source: &str,
    span: Span,
    metadata: &BTreeMap<String, Value>,
    profile: Profile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let file = SourceFile::new(relative, source);
    let at = Some(location(file, span));
    match metadata.get("type").and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => {
            if profile == Profile::Rocci && !PROFILE_TYPES.contains(&value) {
                diagnostics.push(Diagnostic::warning(
                    "OKF2002",
                    relative,
                    at.clone(),
                    format!("unknown Rocci concept type `{value}`"),
                ));
            }
        }
        _ => diagnostics.push(Diagnostic::error(
            "OKF1004",
            relative,
            at.clone(),
            "concept frontmatter requires a non-empty string `type`",
        )),
    }
    validate_optional_string(relative, metadata, "title", at.clone(), diagnostics);
    validate_optional_string(relative, metadata, "description", at.clone(), diagnostics);
    validate_optional_string(relative, metadata, "resource", at.clone(), diagnostics);
    if let Some(status) = metadata.get("status") {
        match status.as_str() {
            Some("draft" | "stable" | "deprecated") => {}
            _ => diagnostics.push(Diagnostic::error(
                "OKF1005",
                relative,
                at.clone(),
                "status must be draft, stable, or deprecated",
            )),
        }
    }
    if let Some(stale_after) = metadata.get("stale_after")
        && !stale_after.as_str().is_some_and(is_date)
    {
        diagnostics.push(Diagnostic::error(
            "OKF1006",
            relative,
            at.clone(),
            "stale_after must use YYYY-MM-DD",
        ));
    }
    if let Some(tags) = metadata.get("tags")
        && !string_array(tags)
    {
        diagnostics.push(Diagnostic::error(
            "OKF1007",
            relative,
            at.clone(),
            "tags must be a list of strings",
        ));
    }
    if let Some(generated) = metadata.get("generated")
        && !generated.as_object().is_some_and(|object| {
            object.get("by").is_some_and(Value::is_string)
                && object
                    .get("at")
                    .and_then(Value::as_str)
                    .is_some_and(|value| parse_timestamp(value).is_some())
        })
    {
        diagnostics.push(Diagnostic::error(
            "OKF1008",
            relative,
            at.clone(),
            "generated must be a mapping with string `by` and RFC 3339 `at`",
        ));
    }
    if let Some(verified) = metadata.get("verified")
        && !verified.as_array().is_some_and(|events| {
            events.iter().all(|event| {
                event.as_object().is_some_and(|object| {
                    object.get("by").is_some_and(Value::is_string)
                        && object
                            .get("at")
                            .and_then(Value::as_str)
                            .is_some_and(|value| parse_timestamp(value).is_some())
                })
            })
        })
    {
        diagnostics.push(Diagnostic::error(
            "OKF1010",
            relative,
            at.clone(),
            "verified must be a list of mappings with string `by` and RFC 3339 `at`",
        ));
    }
    for key in metadata.keys() {
        if !STANDARD_FIELDS.contains(&key.as_str()) {
            diagnostics.push(Diagnostic::warning(
                "OKF2001",
                relative,
                at.clone(),
                format!("unknown metadata field `{key}` is preserved"),
            ));
        }
    }
    if profile == Profile::Rocci {
        for required in ["title", "description", "status", "generated"] {
            if !metadata.contains_key(required) {
                diagnostics.push(Diagnostic::error(
                    "OKF2003",
                    relative,
                    at.clone(),
                    format!("Rocci profile requires `{required}`"),
                ));
            }
        }
        let tags = metadata.get("tags").and_then(Value::as_array);
        if !tags.is_some_and(|tags| {
            !tags.is_empty()
                && tags
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|tag| tag.starts_with("domain/"))
        }) {
            diagnostics.push(Diagnostic::error(
                "OKF2004",
                relative,
                at.clone(),
                "Rocci profile requires tags with at least one domain/* value",
            ));
        }
        if let Some(tags) = tags {
            for tag in tags.iter().filter_map(Value::as_str) {
                let valid_prefix = ["domain/", "integration/", "concern/", "audience/"]
                    .iter()
                    .any(|prefix| tag.starts_with(prefix));
                if !valid_prefix {
                    diagnostics.push(Diagnostic::error(
                        "OKF2005",
                        relative,
                        at.clone(),
                        format!("unknown tag prefix in `{tag}`"),
                    ));
                }
            }
        }
        match metadata.get("authority").and_then(Value::as_str) {
            Some("normative" | "descriptive" | "exploratory" | "historical") => {}
            Some(_) => diagnostics.push(Diagnostic::error(
                "OKF2006",
                relative,
                at.clone(),
                "authority must be normative, descriptive, exploratory, or historical",
            )),
            None => diagnostics.push(Diagnostic::error(
                "OKF2003",
                relative,
                at.clone(),
                "Rocci profile requires `authority`",
            )),
        }
        if !metadata.get("owners").is_some_and(|owners| {
            owners
                .as_array()
                .is_some_and(|owners| !owners.is_empty() && owners.iter().all(Value::is_string))
        }) {
            diagnostics.push(Diagnostic::error(
                "OKF2003",
                relative,
                at,
                "Rocci profile requires string-list `owners`",
            ));
        }
    }
}

fn collect_source_ids(
    relative: &str,
    metadata: &BTreeMap<String, Value>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Some(sources) = metadata.get("sources") else {
        return ids;
    };
    let Some(sources) = sources.as_array() else {
        diagnostics.push(Diagnostic::error(
            "OKF1009",
            relative,
            None,
            "sources must be a list",
        ));
        return ids;
    };
    for (index, source) in sources.iter().enumerate() {
        let Some(source) = source.as_object() else {
            diagnostics.push(Diagnostic::error(
                "OKF1009",
                relative,
                None,
                format!("sources[{index}] must be a mapping"),
            ));
            continue;
        };
        if !source.get("resource").is_some_and(Value::is_string) {
            diagnostics.push(Diagnostic::error(
                "OKF1009",
                relative,
                None,
                format!("sources[{index}] requires string `resource`"),
            ));
        }
        if let Some(id) = source.get("id") {
            match id.as_str() {
                Some(id) if !id.is_empty() => {
                    if !ids.insert(id.to_string()) {
                        diagnostics.push(Diagnostic::error(
                            "OKF4010",
                            relative,
                            None,
                            format!("duplicate source id `{id}`"),
                        ));
                    }
                }
                _ => diagnostics.push(Diagnostic::error(
                    "OKF1009",
                    relative,
                    None,
                    format!("sources[{index}].id must be a non-empty string"),
                )),
            }
        }
    }
    ids
}

fn collect_footnotes(document: &Document) -> (BTreeSet<String>, BTreeSet<String>) {
    fn walk(node: &MdNode, references: &mut BTreeSet<String>, definitions: &mut BTreeSet<String>) {
        match node {
            MdNode::FootnoteReference { name, .. } => {
                references.insert(name.clone());
            }
            MdNode::FootnoteDefinition { name, children, .. } => {
                definitions.insert(name.clone());
                for child in children {
                    walk(child, references, definitions);
                }
            }
            MdNode::Text { value, .. } => {
                references.extend(footnote_labels(value));
            }
            _ => {
                for child in node_children(node) {
                    walk(child, references, definitions);
                }
            }
        }
    }
    let mut references = BTreeSet::new();
    let mut definitions = BTreeSet::new();
    for item in &document.items {
        if let Item::Markdown(node) = item {
            walk(node, &mut references, &mut definitions);
        }
    }
    (references, definitions)
}

fn node_children(node: &MdNode) -> &[MdNode] {
    match node {
        MdNode::Heading { children, .. }
        | MdNode::Paragraph { children, .. }
        | MdNode::BlockQuote { children, .. }
        | MdNode::List { children, .. }
        | MdNode::Item { children, .. }
        | MdNode::TaskItem { children, .. }
        | MdNode::Table { children, .. }
        | MdNode::TableRow { children, .. }
        | MdNode::TableCell { children, .. }
        | MdNode::Emph { children, .. }
        | MdNode::Strong { children, .. }
        | MdNode::Strikethrough { children, .. }
        | MdNode::FootnoteDefinition { children, .. }
        | MdNode::Link { children, .. } => children,
        _ => &[],
    }
}

fn validate_unique_ids(concepts: &[Concept], diagnostics: &mut Vec<Diagnostic>) {
    let mut ids = BTreeMap::new();
    for concept in concepts {
        let folded = concept.id.to_ascii_lowercase();
        if let Some(previous) = ids.insert(folded, concept.path.clone()) {
            diagnostics.push(Diagnostic::error(
                "OKF3003",
                &concept.path,
                None,
                format!("concept id conflicts case-insensitively with `{previous}`"),
            ));
        }
    }
}

fn validate_lifecycle_and_sources(
    root: &Path,
    concepts: &[Concept],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let today = current_utc_date();
    let repository = git_repository_root(root);

    for concept in concepts {
        if let (Some(today), Some(stale_after)) = (
            today.as_deref(),
            string_field(&concept.metadata, "stale_after"),
        ) && is_date(stale_after)
            && stale_after < today
        {
            diagnostics.push(Diagnostic::warning(
                "OKF4004",
                &concept.path,
                None,
                format!("record is stale: stale_after was {stale_after}"),
            ));
        }

        let generated_at = concept
            .metadata
            .get("generated")
            .and_then(Value::as_object)
            .and_then(|generated| generated.get("at"))
            .and_then(Value::as_str)
            .and_then(parse_timestamp);
        let human_verification = latest_human_verification(&concept.metadata);
        if let (Some(generated_at), Some((verified_at, _))) =
            (generated_at, human_verification.as_ref())
            && *verified_at < generated_at
        {
            diagnostics.push(Diagnostic::warning(
                "OKF4005",
                &concept.path,
                None,
                "latest human verification is older than generated.at",
            ));
        }

        let Some(repository) = repository.as_deref() else {
            continue;
        };
        let Some(sources) = concept.metadata.get("sources").and_then(Value::as_array) else {
            continue;
        };
        for source in sources {
            let Some(source) = source.as_object() else {
                continue;
            };
            let Some(resource) = source.get("resource").and_then(Value::as_str) else {
                continue;
            };
            if external_url(resource) || Path::new(resource).is_absolute() {
                continue;
            }
            let source_id = source.get("id").and_then(Value::as_str).unwrap_or(resource);
            let Some(path) = repository_source_path(root, repository, &concept.path, resource)
            else {
                continue;
            };
            let Some(relative) = path.strip_prefix(repository).ok() else {
                continue;
            };
            let relative_display = relative.to_string_lossy().replace('\\', "/");
            match git_last_modified(repository, relative) {
                GitModification::Tracked(modified_at) => {
                    if let Some((verified_at, verified_label)) = human_verification.as_ref()
                        && modified_at > *verified_at
                    {
                        diagnostics.push(Diagnostic::warning(
                            "OKF4006",
                            &concept.path,
                            None,
                            format!(
                                "source `{source_id}` ({relative_display}) changed after human verification at {verified_label}"
                            ),
                        ));
                    }
                    if let Some((verified_at, _)) = human_verification.as_ref()
                        && git_path_dirty(repository, relative)
                        && filesystem_modified_at(&path)
                            .is_none_or(|modified_at| modified_at > *verified_at)
                    {
                        diagnostics.push(Diagnostic::warning(
                            "OKF4008",
                            &concept.path,
                            None,
                            format!(
                                "source `{source_id}` ({relative_display}) has uncommitted changes and cannot be matched to its human verification"
                            ),
                        ));
                    }
                }
                GitModification::Untracked if path.exists() => {
                    diagnostics.push(Diagnostic::warning(
                        "OKF4007",
                        &concept.path,
                        None,
                        format!(
                            "source `{source_id}` ({relative_display}) is untracked and has no git provenance"
                        ),
                    ));
                }
                GitModification::Unknown | GitModification::Untracked => {}
            }
        }
    }
}

fn latest_human_verification(metadata: &BTreeMap<String, Value>) -> Option<(i64, &str)> {
    metadata
        .get("verified")?
        .as_array()?
        .iter()
        .filter_map(Value::as_object)
        .filter(|event| {
            event
                .get("by")
                .and_then(Value::as_str)
                .is_some_and(|actor| actor.starts_with("human:"))
        })
        .filter_map(|event| {
            let at = event.get("at")?.as_str()?;
            Some((parse_timestamp(at)?, at))
        })
        .max_by_key(|(timestamp, _)| *timestamp)
}

fn repository_source_path(
    root: &Path,
    repository: &Path,
    concept_path: &str,
    resource: &str,
) -> Option<PathBuf> {
    let parent = root.join(concept_path).parent()?.to_path_buf();
    let joined = parent.join(resource);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    let normalized = normalized.canonicalize().unwrap_or(normalized);
    normalized.starts_with(repository).then_some(normalized)
}

fn git_repository_root(root: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["-C", root.to_str()?, "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    output.status.success().then(|| {
        let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
        path.canonicalize().unwrap_or(path)
    })
}

enum GitModification {
    Tracked(i64),
    Untracked,
    Unknown,
}

fn git_last_modified(repository: &Path, relative: &Path) -> GitModification {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["log", "-1", "--format=%cI", "--"])
        .arg(relative)
        .output();
    let Ok(output) = output else {
        return GitModification::Unknown;
    };
    if !output.status.success() {
        return GitModification::Unknown;
    }
    let timestamp = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if timestamp.is_empty() {
        GitModification::Untracked
    } else if let Some(timestamp) = parse_timestamp(&timestamp) {
        GitModification::Tracked(timestamp)
    } else {
        GitModification::Unknown
    }
}

fn git_path_dirty(repository: &Path, relative: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(["status", "--porcelain", "--untracked-files=no", "--"])
        .arg(relative)
        .output()
        .is_ok_and(|output| output.status.success() && !output.stdout.is_empty())
}

fn filesystem_modified_at(path: &Path) -> Option<i64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let seconds = modified.duration_since(UNIX_EPOCH).ok()?.as_secs();
    i64::try_from(seconds).ok()
}

fn resolve_graph(
    concepts: &[Concept],
    indexes: &[Index],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Edge> {
    let concept_paths = concepts
        .iter()
        .map(|concept| (concept.path.as_str(), concept))
        .collect::<BTreeMap<_, _>>();
    let index_paths = indexes
        .iter()
        .map(|index| index.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for concept in concepts {
        for link in &concept.links {
            if external_url(&link.url) || link.url.starts_with('#') {
                continue;
            }
            let (path, fragment) = split_fragment(&link.url);
            let resolved = resolve_bundle_path(&concept.path, path);
            let Some(resolved) = resolved else {
                diagnostics.push(Diagnostic::error(
                    "OKF3001",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("link `{}` escapes the knowledge bundle", link.url),
                ));
                continue;
            };
            let directory_index = if resolved.ends_with('/') {
                format!("{resolved}index.md")
            } else {
                String::new()
            };
            let target = concept_paths.get(resolved.as_str()).copied();
            let valid_index = index_paths.contains(resolved.as_str())
                || (!directory_index.is_empty() && index_paths.contains(directory_index.as_str()));
            let broken = target.is_none() && !valid_index;
            if broken {
                diagnostics.push(Diagnostic::warning(
                    "OKF3002",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("broken concept link `{}`", link.url),
                ));
            } else if let (Some(target), Some(fragment)) = (target, fragment)
                && !target.headings.iter().any(|heading| heading.id == fragment)
            {
                diagnostics.push(Diagnostic::warning(
                    "OKF3004",
                    &concept.path,
                    Some(link.location.clone()),
                    format!("unknown heading `{fragment}` in `{resolved}`"),
                ));
            }
            edges.push(Edge {
                from: concept.id.clone(),
                to: resolved
                    .strip_suffix(".md")
                    .unwrap_or(&resolved)
                    .to_string(),
                raw: link.url.clone(),
                broken,
            });
        }
    }
    edges.sort_by(|a, b| (&a.from, &a.to, &a.raw).cmp(&(&b.from, &b.to, &b.raw)));
    edges
}

fn reject_declarations(
    relative: &str,
    source: &str,
    body: Span,
    diagnostics: &mut Vec<Diagnostic>,
) {
    const DECLARATIONS: &[&str] = &[
        "page",
        "roc",
        "render",
        "component",
        "fixture",
        "css",
        "context",
        "init",
        "on",
    ];
    let file = SourceFile::new(relative, source);
    let mut fence: Option<char> = None;
    for (relative_offset, line) in lines_with_offsets(body.of(source)) {
        let trimmed = line.trim_start();
        let fence_marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if let Some(marker) = fence_marker {
            match fence {
                Some(active) if active == marker => fence = None,
                None => fence = Some(marker),
                _ => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix('@') else {
            continue;
        };
        let name = rest
            .split(|character: char| character.is_whitespace() || character == '{')
            .next()
            .unwrap_or("");
        if DECLARATIONS.contains(&name) {
            let indent = line.len() - trimmed.len();
            let start = body.start as usize + relative_offset + indent;
            diagnostics.push(Diagnostic::error(
                "OKF2007",
                relative,
                Some(location(file, Span::new(start, start + name.len() + 1))),
                format!("Rocdown declaration `@{name}` is forbidden in knowledge records"),
            ));
        }
    }
}

#[derive(Clone, Copy)]
struct Frontmatter {
    yaml: Span,
    body: Span,
}

fn split_frontmatter(
    source: &str,
    required: bool,
) -> std::result::Result<Option<Frontmatter>, String> {
    let mut lines = lines_with_offsets(source).into_iter();
    let Some((_, first)) = lines.next() else {
        return if required {
            Err("concept requires YAML frontmatter".into())
        } else {
            Ok(None)
        };
    };
    if first.trim_end_matches(['\r', '\n']) != "---" {
        return if required {
            Err("concept must start with `---` YAML frontmatter".into())
        } else {
            Ok(None)
        };
    }
    let yaml_start = first.len();
    for (offset, line) in lines {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Ok(Some(Frontmatter {
                yaml: Span::new(yaml_start, offset),
                body: Span::new(offset + line.len(), source.len()),
            }));
        }
    }
    Err("frontmatter is missing its closing `---` delimiter".into())
}

fn parse_yaml_mapping(source: &str) -> std::result::Result<BTreeMap<String, Value>, String> {
    let documents = YamlLoader::load_from_str(source).map_err(|error| error.to_string())?;
    if documents.len() != 1 {
        return Err("frontmatter must contain exactly one YAML document".into());
    }
    let Some(mapping) = documents[0].as_hash() else {
        return Err("frontmatter must be a YAML mapping".into());
    };
    let mut out = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("frontmatter keys must be strings".into());
        };
        out.insert(key.to_string(), yaml_to_json(value)?);
    }
    Ok(out)
}

fn yaml_to_json(value: &Yaml) -> std::result::Result<Value, String> {
    match value {
        Yaml::Real(value) => value
            .parse::<f64>()
            .ok()
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .ok_or_else(|| format!("invalid YAML number `{value}`")),
        Yaml::Integer(value) => Ok(Value::Number((*value).into())),
        Yaml::String(value) => Ok(Value::String(value.clone())),
        Yaml::Boolean(value) => Ok(Value::Bool(*value)),
        Yaml::Array(values) => values
            .iter()
            .map(yaml_to_json)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map(Value::Array),
        Yaml::Hash(values) => {
            let mut object = Map::new();
            for (key, value) in values {
                let Some(key) = key.as_str() else {
                    return Err("nested YAML mapping keys must be strings".into());
                };
                object.insert(key.to_string(), yaml_to_json(value)?);
            }
            Ok(Value::Object(object))
        }
        Yaml::Null | Yaml::BadValue => Ok(Value::Null),
        Yaml::Alias(_) => Err("unresolved YAML aliases are not supported".into()),
    }
}

fn discover_markdown(directory: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read {}", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if entry.file_name().as_encoded_bytes().starts_with(b".") {
            continue;
        }
        if path.is_dir() {
            discover_markdown(&path, out)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn resolve_bundle_path(source_path: &str, raw: &str) -> Option<String> {
    let raw = raw.replace('\\', "/");
    let directory = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = if let Some(absolute) = raw.strip_prefix('/') {
        PathBuf::from(absolute)
    } else {
        directory.join(&raw)
    };
    let trailing_slash = raw.ends_with('/');
    let mut parts = Vec::new();
    for component in joined.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    let mut path = parts.join("/");
    if trailing_slash && !path.is_empty() {
        path.push('/');
    }
    Some(path)
}

fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (url, None),
    }
}

fn external_url(url: &str) -> bool {
    url.contains("://")
        || url.starts_with("mailto:")
        || url.starts_with("tel:")
        || url.starts_with("data:")
}

fn validate_optional_string(
    relative: &str,
    metadata: &BTreeMap<String, Value>,
    field: &str,
    location: Option<SourceLocation>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if metadata.get(field).is_some_and(|value| !value.is_string()) {
        diagnostics.push(Diagnostic::error(
            "OKF1005",
            relative,
            location,
            format!("{field} must be a string"),
        ));
    }
}

fn string_array(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|values| values.iter().all(Value::is_string))
}

fn string_field<'a>(metadata: &'a BTreeMap<String, Value>, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(Value::as_str)
}

fn is_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !(bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit()))
    {
        return false;
    }
    let month = value[5..7].parse::<u8>().unwrap_or(0);
    let day = value[8..10].parse::<u8>().unwrap_or(0);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

fn parse_timestamp(value: &str) -> Option<i64> {
    if !value.is_ascii()
        || value.len() < 20
        || &value[4..5] != "-"
        || &value[7..8] != "-"
        || &value[10..11] != "T"
        || &value[13..14] != ":"
        || &value[16..17] != ":"
    {
        return None;
    }
    let year = value[0..4].parse::<i64>().ok()?;
    let month = value[5..7].parse::<i64>().ok()?;
    let day = value[8..10].parse::<i64>().ok()?;
    let hour = value[11..13].parse::<i64>().ok()?;
    let minute = value[14..16].parse::<i64>().ok()?;
    let second = value[17..19].parse::<i64>().ok()?;
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
        || !(0..=60).contains(&second)
    {
        return None;
    }
    let mut timezone = &value[19..];
    if let Some(fraction_and_timezone) = timezone.strip_prefix('.') {
        let timezone_start = fraction_and_timezone.find(['Z', '+', '-'])?;
        let fraction = &fraction_and_timezone[..timezone_start];
        if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        timezone = &fraction_and_timezone[timezone_start..];
    }
    let offset = if timezone == "Z" {
        0
    } else if timezone.len() == 6
        && (&timezone[0..1] == "+" || &timezone[0..1] == "-")
        && &timezone[3..4] == ":"
    {
        let hours = timezone[1..3].parse::<i64>().ok()?;
        let minutes = timezone[4..6].parse::<i64>().ok()?;
        if hours > 23 || minutes > 59 {
            return None;
        }
        let seconds = hours * 3_600 + minutes * 60;
        if &timezone[0..1] == "+" {
            seconds
        } else {
            -seconds
        }
    } else {
        return None;
    };
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second - offset)
}

fn days_from_civil(mut year: i64, month: i64, day: i64) -> i64 {
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn current_utc_date() -> Option<String> {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let days = (seconds / 86_400) as i64 + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn footnote_labels(text: &str) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();
    let bytes = text.as_bytes();
    let mut index = 0;
    while index + 3 < bytes.len() {
        if bytes[index] == b'['
            && bytes[index + 1] == b'^'
            && (index == 0 || bytes[index - 1] != b'\\')
            && let Some(end) = text[index + 2..].find(']')
        {
            let end = index + 2 + end;
            let label = &text[index + 2..end];
            if !label.is_empty() && !label.chars().any(char::is_whitespace) {
                labels.insert(label.to_string());
            }
            index = end + 1;
        } else {
            index += 1;
        }
    }
    labels
}

fn lines_with_offsets(source: &str) -> Vec<(usize, &str)> {
    let mut offset = 0;
    let mut lines = Vec::new();
    for line in source.split_inclusive('\n') {
        lines.push((offset, line));
        offset += line.len();
    }
    if source.is_empty() {
        return lines;
    }
    if !source.ends_with('\n') && lines.is_empty() {
        lines.push((0, source));
    }
    lines
}

fn location(source: SourceFile<'_>, span: Span) -> SourceLocation {
    let (line, column) = source.line_col(span.start);
    SourceLocation {
        start: span.start,
        end: span.end,
        line,
        column,
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn html_page(title: &str, article: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title></head><body><main class=\"rd-document\">{article}</main></body></html>\n",
        escape(title)
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("rocs-okf-{name}-{nonce}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn concept(extra: &str, body: &str) -> String {
        format!(
            "---\ntype: Architecture\ntitle: Test\ndescription: A test concept.\ntags: [domain/rocs]\nstatus: draft\ngenerated: {{ by: process:test, at: 2026-08-16T00:00:00Z }}\nauthority: descriptive\nowners: [human:nils]\n{extra}---\n\n{body}"
        )
    }

    fn git(root: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(root)
            .env("GIT_AUTHOR_NAME", "Rocs Test")
            .env("GIT_AUTHOR_EMAIL", "rocs@example.invalid")
            .env("GIT_COMMITTER_NAME", "Rocs Test")
            .env("GIT_COMMITTER_EMAIL", "rocs@example.invalid")
            .env("GIT_AUTHOR_DATE", "2026-08-16T12:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-08-16T12:00:00Z")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }

    #[test]
    fn preserves_unknown_metadata_and_original_body_offsets() {
        let root = temp("unknown");
        fs::write(
            root.join("test.md"),
            concept("extension_data: { nested: true }\n", "# Héading\n"),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        let concept = &bundle.concepts[0];
        assert_eq!(concept.metadata["extension_data"]["nested"], true);
        assert!(concept.body_span.start > 0);
        assert!(concept.headings[0].location.start >= concept.body_span.start);
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF2001")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn keyed_footnotes_render_accessibly_and_validate_sources() {
        let root = temp("footnotes");
        fs::write(
            root.join("test.md"),
            concept(
                "sources:\n  - id: parser\n    resource: ../parser.rs\n",
                "# Test\n\nA claim.[^parser]\n\n[^parser]: Parser evidence.\n",
            ),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert!(!bundle.has_errors(), "{:?}", bundle.diagnostics);
        let html = &bundle.concepts[0].article_html;
        assert!(html.contains("data-footnote-ref"), "{html}");
        assert!(html.contains("aria-label=\"Footnotes\""), "{html}");
        assert!(html.contains("data-footnote-backref"), "{html}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recognizes_reserved_files_and_warns_for_broken_links() {
        let root = temp("reserved");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(
            root.join("log.md"),
            "# Log\n\n## 2026-08-16\n\n* Created.\n",
        )
        .unwrap();
        fs::write(
            root.join("test.md"),
            concept("", "# Test\n\n[Missing](/missing.md)\n"),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert_eq!(bundle.version.as_deref(), Some("0.2"));
        assert_eq!(bundle.indexes.len(), 1);
        assert_eq!(bundle.logs.len(), 1);
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF3002"
                    && diagnostic.severity == Severity::Warning)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn base_and_rocci_profiles_are_separate() {
        let root = temp("profiles");
        fs::write(root.join("minimal.md"), "---\ntype: Note\n---\n\n# Note\n").unwrap();
        assert!(!load(&root, Profile::Base).unwrap().has_errors());
        assert!(load(&root, Profile::Rocci).unwrap().has_errors());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unresolved_keyed_footnote_is_an_error_but_fenced_declarations_are_inert() {
        let root = temp("unresolved-footnote");
        fs::write(
            root.join("test.md"),
            concept(
                "sources:\n  - id: missing\n    resource: ../missing.rs\n",
                "# Test\n\nClaim.[^missing]\n\n```rocdown\n@roc { x = 1 }\n```\n",
            ),
        )
        .unwrap();
        let bundle = load(&root, Profile::Rocci).unwrap();
        assert!(
            bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF4003")
        );
        assert!(
            !bundle
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "OKF2007")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn build_emits_catalog_search_llms_validation_and_site() {
        let root = temp("build");
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(root.join("test.md"), concept("", "# Test\n")).unwrap();
        let output = root.join("dist");
        let summary = build(&root, &output, Profile::Rocci).unwrap();
        assert_eq!(summary.concepts, 1);
        assert!(output.join("catalog.json").is_file());
        assert!(output.join("search.json").is_file());
        assert!(output.join("llms.txt").is_file());
        assert!(output.join("validation.json").is_file());
        assert!(output.join("site/test/index.html").is_file());
        let first_catalog = fs::read(output.join("catalog.json")).unwrap();
        let first_search = fs::read(output.join("search.json")).unwrap();
        let first_llms = fs::read(output.join("llms.txt")).unwrap();
        let first_page = fs::read(output.join("site/test/index.html")).unwrap();
        build(&root, &output, Profile::Rocci).unwrap();
        assert_eq!(
            first_catalog,
            fs::read(output.join("catalog.json")).unwrap()
        );
        assert_eq!(first_search, fs::read(output.join("search.json")).unwrap());
        assert_eq!(first_llms, fs::read(output.join("llms.txt")).unwrap());
        assert_eq!(
            first_page,
            fs::read(output.join("site/test/index.html")).unwrap()
        );
        fs::write(root.join("test.md"), "# Invalid knowledge record\n").unwrap();
        assert!(build(&root, &output, Profile::Rocci).is_err());
        assert_eq!(
            first_page,
            fs::read(output.join("site/test/index.html")).unwrap()
        );
        fs::remove_file(root.join("test.md")).unwrap();
        let summary = build(&root, &output, Profile::Rocci).unwrap();
        assert_eq!(summary.concepts, 0);
        assert!(!output.join("site/test/index.html").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_and_search_apply_lifecycle_trust_and_stale_filters() {
        let root = temp("search-filter");
        fs::write(
            root.join("reviewed.md"),
            concept(
                "verified:\n  - { by: human:nils, at: 2026-08-16T00:00:00Z }\nstale_after: 2999-01-01\n",
                "# Reviewed theme\n\nCurrent resolver contract.\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("generated.md"),
            concept("", "# Generated note\n\nCurrent parser contract.\n"),
        )
        .unwrap();

        let filter = KnowledgeFilter {
            types: vec!["architecture".into()],
            tags: vec!["domain/rocs".into()],
            statuses: vec!["DRAFT".into()],
            authorities: vec!["descriptive".into()],
            trust_tiers: vec![TrustTier::HumanReviewed],
            stale: Some(false),
        };
        let catalog =
            inspect_filtered(&root, InspectKind::Catalog, None, Profile::Rocci, &filter).unwrap();
        assert!(catalog.contains("\"id\": \"reviewed\""), "{catalog}");
        assert!(!catalog.contains("\"id\": \"generated\""), "{catalog}");
        assert!(catalog.contains("human-reviewed"), "{catalog}");

        let results = search(&root, "resolver contract", Profile::Rocci, &filter).unwrap();
        assert!(results.contains("reviewed#reviewed-theme"), "{results}");
        assert!(!results.contains("generated"), "{results}");
        let missing = search(&root, "parser contract", Profile::Rocci, &filter).unwrap();
        assert_eq!(missing, "[]");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retrieval_benchmark_measures_hits_rank_and_lifecycle() {
        let root = temp("retrieval-benchmark");
        fs::write(
            root.join("reviewed.md"),
            concept(
                "verified:\n  - { by: human:nils, at: 2026-08-16T00:00:00Z }\nstale_after: 2999-01-01\n",
                "# Reviewed theme\n\nCurrent resolver contract.\n",
            ),
        )
        .unwrap();
        let benchmark = root.join("retrieval.toml");
        fs::write(
            &benchmark,
            r#"version = 1
top_k = 3
minimum_hit_rate = 1.0

[[questions]]
id = "theme"
question = "Where is the current resolver contract documented?"
query = "resolver contract"
expected_concepts = ["reviewed"]
expected_status = "draft"
expected_authority = "descriptive"
"#,
        )
        .unwrap();

        let report = benchmark_retrieval(&root, &benchmark, Profile::Rocci).unwrap();
        assert_eq!(report.total, 1);
        assert_eq!(report.passed, 1);
        assert_eq!(report.hit_rate, 1.0);
        assert_eq!(report.mean_reciprocal_rank, 1.0);
        assert!(report.threshold_met);
        assert_eq!(report.questions[0].first_relevant_rank, Some(1));
        assert!(report.questions[0].lifecycle_matched);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn loaded_site_includes_indexes_rewrites_links_and_copies_static_files() {
        let root = temp("loaded-site");
        fs::create_dir_all(root.join("architecture")).unwrap();
        fs::create_dir_all(root.join("decisions")).unwrap();
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n\n[Architecture](architecture/)\n\n[Matrix](matrix.tsv)\n",
        )
        .unwrap();
        fs::write(
            root.join("architecture/index.md"),
            "# Architecture\n\n[System](system.md)\n",
        )
        .unwrap();
        fs::write(
            root.join("architecture/system.md"),
            "---\ntype: Note\ntitle: System\n---\n\n# System\n\n[Home](/index.md)\n",
        )
        .unwrap();
        fs::write(root.join("decisions/index.md"), "# Decisions\n").unwrap();
        fs::write(
            root.join("decisions/choice.md"),
            "---\ntype: Note\ntitle: Choice\n---\n\n# Choice\n",
        )
        .unwrap();
        fs::write(root.join("matrix.tsv"), "name\tstatus\nSystem\tstable\n").unwrap();
        let loaded = load_site(&root, Profile::Base).unwrap();
        assert_eq!(loaded.config.site.title, "Knowledge");
        assert_eq!(loaded.config.navigation.len(), 3);
        assert_eq!(loaded.config.navigation[0].label, "Governance & Review");
        assert_eq!(loaded.config.navigation[1].label, "Architecture");
        assert_eq!(
            loaded.config.navigation[1].directory.as_deref(),
            Some("architecture")
        );
        assert!(loaded.sources.iter().any(|page| page.id == "index"));
        assert!(loaded.sources.iter().any(|page| page.id == "review"));
        assert!(
            loaded
                .sources
                .iter()
                .any(|page| page.id == "architecture/index")
        );
        let home = loaded
            .sources
            .iter()
            .find(|page| page.id == "index")
            .unwrap();
        assert!(home.article_html.contains("href=\"/architecture/\""));
        assert!(home.article_html.contains("href=\"/matrix.tsv\""));
        assert!(home.article_html.contains("Priority-1 Review Queue"));
        let section = loaded
            .sources
            .iter()
            .find(|page| page.id == "architecture/index")
            .unwrap();
        assert!(
            section
                .article_html
                .contains("href=\"/architecture/system/\"")
        );
        let system = loaded
            .sources
            .iter()
            .find(|page| page.id == "architecture/system")
            .unwrap();
        assert!(system.article_html.contains("href=\"/\""));
        assert!(system.article_html.contains("okf-concept-meta"));
        let review = loaded
            .sources
            .iter()
            .find(|page| page.id == "review")
            .unwrap();
        assert!(review.article_html.contains("okf-review-table"));
        assert_eq!(loaded.static_files.len(), 1);
        assert_eq!(loaded.static_files[0].output_path, "matrix.tsv");
        let resolved = crate::site::resolve_loaded(&loaded);
        assert!(!resolved.has_errors(), "{:?}", resolved.diagnostics);
        assert_eq!(resolved.site.navigation.len(), 3);
        assert_eq!(resolved.site.navigation[0].label, "Governance & Review");
        assert_eq!(resolved.site.navigation[0].items[0].route, "/review/");
        assert_eq!(resolved.site.navigation[1].label, "Architecture");
        assert_eq!(resolved.site.navigation[1].items[0].route, "/architecture/");
        assert!(!resolved.site.unlisted.iter().any(|id| id == "index"));
        let plan = crate::plan::plan(&loaded.root, &loaded.config, &resolved.site).unwrap();
        let article_paths = plan
            .pages
            .iter()
            .map(|page| page.article_path.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(article_paths.len(), plan.pages.len());
        let system_page = plan
            .pages
            .iter()
            .find(|page| page.view.route == "/architecture/system/")
            .unwrap();
        assert!(system_page.view.lanes.is_empty());
        assert_eq!(
            system_page
                .view
                .sidebar
                .iter()
                .map(|item| item.title.as_str())
                .collect::<Vec<_>>(),
            ["Governance & Review", "Architecture", "System", "Decisions"]
        );
        assert_eq!(
            system_page
                .view
                .sidebar
                .iter()
                .find(|item| item.href == "/architecture/system/")
                .unwrap()
                .class_name,
            "nav-link nav-child is-current"
        );
        assert_eq!(
            system_page.view.sidebar[0].class_name,
            "nav-link nav-category"
        );
        assert!(
            system_page.view.sidebar[1]
                .class_name
                .contains("is-expanded")
        );
        assert!(system_page.view.sidebar[2].class_name.contains("nav-child"));
        assert!(
            system_page.view.sidebar[2]
                .class_name
                .contains("is-current")
        );
        assert_eq!(
            system_page.view.sidebar[3].class_name,
            "nav-link nav-category"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lifecycle_and_git_source_drift_are_warnings() {
        let repository = temp("source-drift");
        git(&repository, &["init", "-q"]);
        git(&repository, &["config", "core.hooksPath", ".git/hooks"]);
        fs::write(repository.join("tracked.txt"), "tracked evidence\n").unwrap();
        git(&repository, &["add", "tracked.txt"]);
        git(&repository, &["commit", "-q", "-m", "tracked evidence"]);
        fs::write(repository.join("untracked.txt"), "untracked evidence\n").unwrap();
        fs::write(repository.join("tracked.txt"), "changed evidence\n").unwrap();

        let root = repository.join("knowledge");
        fs::create_dir_all(root.join("architecture")).unwrap();
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# Knowledge\n",
        )
        .unwrap();
        fs::write(
            root.join("architecture/test.md"),
            "---\n\
type: Architecture\n\
title: Drift test\n\
description: Exercises lifecycle and repository drift diagnostics.\n\
tags: [domain/rocs]\n\
status: stable\n\
generated: { by: process:test, at: 2026-08-16T13:00:00Z }\n\
verified:\n\
  - { by: human:test, at: 2026-08-15T12:00:00Z }\n\
stale_after: 2000-01-01\n\
authority: descriptive\n\
owners: [human:test]\n\
sources:\n\
  - { id: tracked, resource: ../../tracked.txt }\n\
  - { id: untracked, resource: ../../untracked.txt }\n\
---\n\n\
# Drift test\n\nClaims.[^tracked][^untracked]\n\n\
[^tracked]: Tracked evidence.\n\
[^untracked]: Untracked evidence.\n",
        )
        .unwrap();

        let discovered_repository = git_repository_root(&root).unwrap();
        let canonical_repository = repository.canonicalize().unwrap();
        assert_eq!(discovered_repository, canonical_repository);
        let tracked_path = repository_source_path(
            &root,
            &discovered_repository,
            "architecture/test.md",
            "../../tracked.txt",
        )
        .unwrap();
        assert_eq!(tracked_path, canonical_repository.join("tracked.txt"));
        assert!(matches!(
            git_last_modified(&repository, Path::new("tracked.txt")),
            GitModification::Tracked(_)
        ));
        assert!(matches!(
            git_last_modified(&repository, Path::new("untracked.txt")),
            GitModification::Untracked
        ));

        let bundle = load(&root, Profile::Rocci).unwrap();
        for code in ["OKF4004", "OKF4005", "OKF4006", "OKF4007", "OKF4008"] {
            assert!(
                bundle
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == code
                        && diagnostic.severity == Severity::Warning),
                "missing {code}: {:?}",
                bundle.diagnostics
            );
        }
        assert!(!bundle.has_errors(), "{:?}", bundle.diagnostics);
        fs::remove_dir_all(repository).unwrap();
    }

    #[test]
    fn timestamps_compare_across_offsets() {
        assert_eq!(
            parse_timestamp("2026-08-16T14:00:00Z"),
            parse_timestamp("2026-08-16T16:00:00+02:00")
        );
        assert_eq!(
            parse_timestamp("2026-08-16T14:00:00.123Z"),
            parse_timestamp("2026-08-16T14:00:00Z")
        );
        assert_eq!(parse_timestamp("é026-08-16T14:00:00Z"), None);
        assert!(current_utc_date().as_deref().is_some_and(is_date));
    }
}
