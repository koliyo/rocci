use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use okf::{
    Bundle, Concept, ConceptAction, ConceptInspect, Diagnostic, STANDARD_FIELDS, Severity,
    TrustTier, classify_concept_action, external_url, published_href, slugify,
};
use rocci_cli::profile::{ProfileSnapshot, SpanRecorder};
pub use rocci_ui::escape;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatTone {
    #[default]
    Default,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatCardView {
    pub value: String,
    pub label: String,
    pub tone: StatTone,
    pub href: Option<String>,
}

impl StatCardView {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            tone: StatTone::Default,
            href: None,
        }
    }

    pub fn with_tone(mut self, tone: StatTone) -> Self {
        self.tone = tone;
        self
    }
}

pub fn render_stat_grid(cards: &[StatCardView]) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"okf-stat-grid\">\n");
    for card in cards {
        let tone_class = match card.tone {
            StatTone::Default => "",
            StatTone::Action => " is-action",
        };
        if let Some(href) = &card.href {
            out.push_str(&format!(
                "  <a href=\"{}\" class=\"okf-stat-card{}\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">{}</div></a>\n",
                escape(href),
                tone_class,
                escape(&card.value),
                escape(&card.label)
            ));
        } else {
            out.push_str(&format!(
                "  <div class=\"okf-stat-card{}\"><div class=\"okf-stat-value\">{}</div><div class=\"okf-stat-label\">{}</div></div>\n",
                tone_class,
                escape(&card.value),
                escape(&card.label)
            ));
        }
    }
    out.push_str("</div>\n");
    out
}

pub const PRIORITY_1_RECORDS: &[(&str, &str)] = &[
    (
        "architecture/rocdown-format",
        "Parser/README precedence over original report; root HTML template islands",
    ),
    (
        "architecture/rocdown-documentation-compiler",
        "Rocdown generator plus isolated OKF preview/retrieval path",
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

pub fn render_concept_meta(concept: &Concept, bundle: &Bundle) -> String {
    let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
    let authority = okf::string_field(&concept.metadata, "authority").unwrap_or("descriptive");
    let trust_tier = okf::search::concept_trust_tier(&concept.metadata);
    let stale = okf::search::concept_is_stale(&concept.metadata);
    let stale_after = okf::string_field(&concept.metadata, "stale_after").unwrap_or("");
    let concept_type = okf::string_field(&concept.metadata, "type").unwrap_or("Concept");
    let owners = okf::metadata_string_array(&concept.metadata, "owners");
    let tags = okf::metadata_string_array(&concept.metadata, "tags");
    let description = okf::string_field(&concept.metadata, "description").unwrap_or("");

    let action = classify_concept_action(concept, &bundle.diagnostics);

    let (trust_slug, trust_label) = match trust_tier {
        TrustTier::HumanReviewed => ("human", "human-reviewed"),
        TrustTier::Generated => ("generated", "generated"),
        TrustTier::Unverified => ("unverified", "unverified"),
    };

    let latest_verification = okf::latest_human_verification(&concept.metadata);
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

    if !description.is_empty() {
        out.push_str(&format!(
            "  <p class=\"okf-lead\">{}</p>\n",
            escape(description)
        ));
    }

    let mut provenance = Vec::new();
    if !owners.is_empty() {
        provenance.push(format!(
            "<li><span class=\"okf-meta-label\">Owners</span> <code>{}</code></li>",
            escape(&owners.join(", "))
        ));
    }
    if let Some((_, verifier_str)) = latest_verification {
        provenance.push(format!(
            "<li><span class=\"okf-meta-label\">Verified</span> <code>{}</code></li>",
            escape(verifier_str)
        ));
    } else {
        provenance.push(
            "<li><span class=\"okf-meta-label\">Verified</span> <em>Unverified</em></li>"
                .to_string(),
        );
    }
    if !generated_by.is_empty() {
        provenance.push(format!(
            "<li><span class=\"okf-meta-label\">Generated</span> <code>{} @ {}</code></li>",
            escape(generated_by),
            escape(generated_at)
        ));
    }
    if !stale_after.is_empty() {
        provenance.push(format!(
            "<li><span class=\"okf-meta-label\">Stale after</span> <code>{}</code></li>",
            escape(stale_after)
        ));
    }
    if !provenance.is_empty() {
        out.push_str("  <ul class=\"okf-provenance\">\n");
        for item in provenance {
            out.push_str("    ");
            out.push_str(&item);
            out.push('\n');
        }
        out.push_str("  </ul>\n");
    }

    if let Some(sources) = concept.metadata.get("sources").and_then(Value::as_array)
        && !sources.is_empty()
    {
        let drift_diags: Vec<&Diagnostic> = bundle
            .diagnostics
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
        out.push_str("    <div class=\"okf-table-container\">\n");
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
                "        <tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape(s_id),
                source_resource_cell(concept, s_res),
                escape(s_author),
                status_badge
            ));
        }
        out.push_str("      </tbody>\n");
        out.push_str("    </table>\n");
        out.push_str("    </div>\n");
        out.push_str("  </details>\n");
    }

    let unknown: Vec<(&String, &Value)> = concept
        .metadata
        .iter()
        .filter(|(key, _)| !STANDARD_FIELDS.contains(&key.as_str()))
        .collect();
    if !unknown.is_empty() {
        out.push_str("  <details class=\"okf-other-meta\">\n");
        out.push_str(&format!(
            "    <summary><strong>{} other field{}</strong></summary>\n",
            unknown.len(),
            if unknown.len() == 1 { "" } else { "s" }
        ));
        out.push_str("    <div class=\"okf-table-container\">\n");
        out.push_str("    <table class=\"okf-sources-table\">\n");
        out.push_str("      <thead><tr><th>Key</th><th>Value</th></tr></thead>\n");
        out.push_str("      <tbody>\n");
        for (key, value) in unknown {
            out.push_str(&format!(
                "        <tr><td><code>{}</code></td><td><code>{}</code></td></tr>\n",
                escape(key),
                escape(&compact_json_value(value))
            ));
        }
        out.push_str("      </tbody>\n");
        out.push_str("    </table>\n");
        out.push_str("    </div>\n");
        out.push_str("  </details>\n");
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

fn source_resource_cell(concept: &Concept, resource: &str) -> String {
    let label = format!("<code>{}</code>", escape(resource));
    match source_href(concept, resource) {
        Some(href) if external_url(resource) => format!(
            "<a href=\"{}\" rel=\"noopener noreferrer\">{}</a>",
            escape(&href),
            label
        ),
        Some(href) => format!("<a href=\"{}\">{}</a>", escape(&href), label),
        None => label,
    }
}

fn source_href(concept: &Concept, resource: &str) -> Option<String> {
    if external_url(resource) {
        return Some(resource.to_string());
    }
    published_href(&concept.path, resource)
}

fn compact_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".into(),
        other => serde_json::to_string(other).unwrap_or_else(|_| other.to_string()),
    }
}

pub fn render_priority_1_queue(bundle: &Bundle) -> String {
    let mut out = String::new();
    out.push_str("<div class=\"okf-priority-1-section\">\n");
    out.push_str(
        "  <h2 class=\"rd-header-2\" id=\"priority-1-review-queue\">Priority-1 Review Queue</h2>\n",
    );
    out.push_str("  <p class=\"rd-paragraph\">The evidence-based human review gate for stabilizing Rocci's core architectural and decision records.</p>\n\n");

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
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(id);
        let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
        let authority = okf::string_field(&concept.metadata, "authority").unwrap_or("descriptive");
        let action = classify_concept_action(concept, &bundle.diagnostics);
        let trust_tier = okf::search::concept_trust_tier(&concept.metadata);
        let (trust_slug, trust_label) = match trust_tier {
            TrustTier::HumanReviewed => ("human", "human-reviewed"),
            TrustTier::Generated => ("generated", "generated"),
            TrustTier::Unverified => ("unverified", "unverified"),
        };
        let verifier_text =
            if let Some((_, v_str)) = okf::latest_human_verification(&concept.metadata) {
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
        let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
        let stale = okf::search::concept_is_stale(&concept.metadata);
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
    out.push_str(&render_stat_grid(&governance_stat_cards(
        total_concepts,
        stable_count,
        draft_count,
        action_count,
        stale_count,
        warnings_count,
    )));
    out.push('\n');

    out.push_str(&render_priority_1_queue(bundle));

    out.push_str("  <div class=\"okf-cta-row\">\n");
    out.push_str(&format!(
        "    <a href=\"/review/\" class=\"okf-cta-btn\">Open Complete Review Queue (All {} Concepts) &rarr;</a>\n",
        total_concepts
    ));
    out.push_str("    <a href=\"/reference/priority-1-review/\" class=\"okf-secondary-link\">View Priority-1 Review Checklist &rarr;</a>\n");
    out.push_str("  </div>\n");
    out.push_str("  <hr class=\"rd-hr\" />\n");
    out.push_str(
        "  <h2 class=\"rd-header-2\" id=\"knowledge-collections\">Knowledge Collections</h2>\n",
    );
    out.push_str("</div>\n\n");
    out
}

pub fn governance_stat_cards(
    total_concepts: usize,
    stable_count: usize,
    draft_count: usize,
    action_count: usize,
    stale_count: usize,
    warnings_count: usize,
) -> Vec<StatCardView> {
    vec![
        StatCardView::new(total_concepts.to_string(), "Total Concepts"),
        StatCardView::new(stable_count.to_string(), "Stable"),
        StatCardView::new(draft_count.to_string(), "Draft"),
        StatCardView::new(action_count.to_string(), "Action Required").with_tone(
            if action_count > 0 {
                StatTone::Action
            } else {
                StatTone::Default
            },
        ),
        StatCardView::new(stale_count.to_string(), "Stale Records"),
        StatCardView::new(warnings_count.to_string(), "Diagnostics"),
    ]
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
        let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
        let authority = okf::string_field(&concept.metadata, "authority").unwrap_or("descriptive");
        let concept_type = okf::string_field(&concept.metadata, "type").unwrap_or("Concept");
        let stale = okf::search::concept_is_stale(&concept.metadata);
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

        let trust_tier = okf::search::concept_trust_tier(&concept.metadata);
        let (trust_slug, trust_label) = match trust_tier {
            TrustTier::HumanReviewed => ("human", "human-reviewed"),
            TrustTier::Generated => ("generated", "generated"),
            TrustTier::Unverified => ("unverified", "unverified"),
        };

        let verifier_text =
            if let Some((_, v_str)) = okf::latest_human_verification(&concept.metadata) {
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

    out.push_str(&render_stat_grid(&governance_stat_cards(
        total_concepts,
        stable_count,
        draft_count,
        action_count,
        stale_count,
        warnings_count,
    )));
    out.push('\n');

    out.push_str(&render_priority_1_queue(bundle));

    out.push_str(&format!(
        "  <h2 class=\"rd-header-2\" id=\"all-concepts-queue\">All Bundle Concepts ({} Records)</h2>\n",
        total_concepts
    ));
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
        let title = okf::string_field(&row.concept.metadata, "title").unwrap_or(&row.concept.id);
        let tags = okf::metadata_string_array(&row.concept.metadata, "tags");
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

#[allow(dead_code)]
pub fn build_review_site(bundle: &Bundle, site: &Path) -> Result<()> {
    let _ = build_review_site_with_host(bundle, site, None)?;
    Ok(())
}

const BASIC_CLI_PLATFORM: &str = "https://github.com/roc-lang/basic-cli/releases/download/0.22.0/F1JVZPYfWP71s8vk6tHcV1Qx1Ef6CZkwswGoCn8VHZmL.tar.zst";

#[derive(Clone)]
pub struct CompiledOkfModule {
    pub type_name: String,
    pub source_name: String,
    pub src: String,
    pub roc: String,
    pub segments: Vec<rocci_template::Segment>,
}

pub fn compile_okf_templates() -> Result<Vec<CompiledOkfModule>> {
    let raw = [
        ("PageOutline", crate::runtime::PAGE_OUTLINE),
        ("ConceptMeta", crate::runtime::CONCEPT_META),
        ("ReviewQueue", crate::runtime::REVIEW_QUEUE),
        ("OkfTheme", crate::runtime::OKF_THEME),
    ];

    let mut out = Vec::new();
    let lower_opts = rocci_template::LowerOptions::default();

    for (type_name, src) in raw {
        let source_file = rocci_template::SourceFile::new(type_name, src);
        let compiled = rocci_template::compile(source_file, &lower_opts);
        if compiled.has_errors() {
            let diags: Vec<String> = compiled
                .diagnostics
                .iter()
                .map(|d| rocci_template::format_diagnostic(source_file, d))
                .collect();
            bail!("failed to compile {type_name}.rocci:\n{}", diags.join("\n"));
        }
        out.push(CompiledOkfModule {
            type_name: type_name.to_string(),
            source_name: format!("{type_name}.rocci"),
            src: src.to_string(),
            roc: rocci_template::wrap_type_module(&compiled.roc, type_name),
            segments: compiled.segments,
        });
    }

    Ok(out)
}

fn is_roc_available() -> bool {
    std::process::Command::new("roc")
        .arg("help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn unique_temp(prefix: &str) -> Result<std::path::PathBuf> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let id = std::process::id();
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rocci-okf-{prefix}-{id}-{time}-{n}"));
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create temp dir {}", dir.display()))?;
    Ok(dir)
}

fn roc_version() -> String {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION
        .get_or_init(|| {
            std::process::Command::new("roc")
                .arg("version")
                .output()
                .ok()
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|text| text.trim().to_string())
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "roc-unknown".into())
        })
        .clone()
}

fn renderer_compile_hash(modules: &[CompiledOkfModule], main_code: &str, is_wasm: bool) -> String {
    let mut named: Vec<(String, Vec<u8>)> = Vec::new();
    for module in modules {
        named.push((
            format!("{}.roc", module.type_name),
            module.roc.as_bytes().to_vec(),
        ));
    }
    named.push((
        "Html.roc".into(),
        crate::runtime::HTML_ROC.as_bytes().to_vec(),
    ));
    named.push((
        "OkfBuild.roc".into(),
        crate::runtime::OKF_BUILD_ROC.as_bytes().to_vec(),
    ));
    named.push(("main.roc".into(), main_code.as_bytes().to_vec()));
    let module_refs: Vec<(&str, &[u8])> = named
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    let gen_hash = rocci_roc_host::compute_gen_hash(
        env!("CARGO_PKG_VERSION"),
        "okf-preview",
        &module_refs,
        &[],
    );
    let target = if is_wasm {
        "wasm32".to_string()
    } else {
        format!("native:{}", std::env::consts::ARCH)
    };
    let platform = if is_wasm {
        "rocci-roc-host-wasm32"
    } else {
        BASIC_CLI_PLATFORM
    };
    rocci_roc_host::compute_compile_hash(
        &gen_hash,
        &roc_version(),
        &target,
        "dev",
        platform,
        env!("CARGO_PKG_VERSION"),
    )
}

fn okf_fingerprints(
    modules: &[CompiledOkfModule],
    main_code: &str,
) -> Vec<rocci_roc_host::InputFingerprint> {
    let mut fps = Vec::new();
    for module in modules {
        fps.push(rocci_roc_host::InputFingerprint::from_bytes(
            &format!("{}.roc", module.type_name),
            module.roc.as_bytes(),
        ));
    }
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "Html.roc",
        crate::runtime::HTML_ROC.as_bytes(),
    ));
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "OkfBuild.roc",
        crate::runtime::OKF_BUILD_ROC.as_bytes(),
    ));
    fps.push(rocci_roc_host::InputFingerprint::from_bytes(
        "main.roc",
        main_code.as_bytes(),
    ));
    fps
}

fn main_roc(is_wasm: bool) -> String {
    if is_wasm {
        "\
app [main!] { pf: platform \"platform/main.roc\" }

import OkfBuild

main! : {} => [Ok({}), Err([Exit(I32)])]
main! = |{}| {
    _ = OkfBuild.parse_pages(\"{}\")
    res : [Ok({}), Err([Exit(I32)])]
    res = Ok({})
    res
}
"
        .to_string()
    } else {
        format!(
            "\
app [main!] {{ pf: platform \"{BASIC_CLI_PLATFORM}\" }}

import pf.Env
import pf.Path
import OkfBuild

load_one! = |record| {{
    html = Path.utf8(record.article_path).read_utf8!()?
    Ok(OkfBuild.with_article(record, html))
}}

load_all! = |records| {{
    match List.get(records, 0) {{
        Err(_) => Ok([])
        Ok(first) => {{
            page = load_one!(first)?
            rest = load_all!(List.drop_first(records, 1))?
            Ok(List.concat([page], rest))
        }}
    }}
}}

ensure_parent! = |dest| {{
    parts = Str.split_on(dest, \"/\")
    len = List.len(parts)
    if len <= 1 {{
        Ok({{}})
    }} else {{
        parent = Str.join_with(List.drop_last(parts, 1), \"/\")
        Path.utf8(parent).create_all!()?
        Ok({{}})
    }}
}}

write_page! = |staging, item| {{
    dest = \"${{staging}}/${{item.output_path}}\"
    ensure_parent!(dest)?
    Path.utf8(dest).write_utf8!(OkfBuild.render_page(item))?
    Ok({{}})
}}

write_all! = |staging, pages| {{
    for page in pages {{
        write_page!(staging, page)?
    }}
    Ok({{}})
}}

main! = |_args| {{
    staging = Env.var_str!(\"OKF_STAGING\")?
    json = Path.utf8(\"okf-pages.json\").read_utf8!()?
    pages = load_all!(OkfBuild.parse_pages(json))?
    write_all!(staging, pages)?
    Ok({{}})
}}
"
        )
    }
}

#[derive(Debug, Serialize)]
struct OkfPagesFile {
    pages: Vec<OkfPageRecord>,
}

#[derive(Debug, Serialize)]
struct OkfPageRecord {
    output_path: String,
    article_path: String,
    title: String,
    has_outline: bool,
    outline: Vec<OkfOutlineItem>,
    has_meta: bool,
    meta: OkfPageMeta,
}

#[derive(Debug, Serialize)]
struct OkfOutlineItem {
    id: String,
    title: String,
    level: String,
}

#[derive(Debug, Serialize)]
struct OkfPageMeta {
    concept_type: String,
    status: String,
    authority: String,
    trust_slug: String,
    trust_label: String,
    stale: bool,
    stale_after: String,
    is_action_required: bool,
    action_detail: String,
    description: String,
    has_provenance: bool,
    owners: String,
    verifier: String,
    generated: String,
    has_sources: bool,
    source_count: String,
    drift_summary: String,
    sources: Vec<OkfSourceItem>,
    has_other_meta: bool,
    other_meta_count: String,
    other_meta: Vec<OkfOtherMeta>,
    has_tags: bool,
    tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OkfSourceItem {
    id: String,
    resource: String,
    href: String,
    author: String,
    is_drifted: bool,
}

#[derive(Debug, Serialize)]
struct OkfOtherMeta {
    key: String,
    val: String,
}

struct GeneratedOkfPages {
    json: String,
    articles: Vec<(String, String)>,
    output_paths: Vec<String>,
}

fn empty_page_meta() -> OkfPageMeta {
    OkfPageMeta {
        concept_type: String::new(),
        status: String::new(),
        authority: String::new(),
        trust_slug: String::new(),
        trust_label: String::new(),
        stale: false,
        stale_after: String::new(),
        is_action_required: false,
        action_detail: String::new(),
        description: String::new(),
        has_provenance: false,
        owners: String::new(),
        verifier: String::new(),
        generated: String::new(),
        has_sources: false,
        source_count: "0".into(),
        drift_summary: String::new(),
        sources: Vec::new(),
        has_other_meta: false,
        other_meta_count: "0".into(),
        other_meta: Vec::new(),
        has_tags: false,
        tags: Vec::new(),
    }
}

fn outline_items(headings: &[TocHeading]) -> Vec<OkfOutlineItem> {
    headings
        .iter()
        .map(|heading| OkfOutlineItem {
            id: heading.id.clone(),
            title: heading.text.clone(),
            level: heading.level.to_string(),
        })
        .collect()
}

fn page_record(
    output_path: String,
    article_path: String,
    title: String,
    headings: &[TocHeading],
    meta: Option<OkfPageMeta>,
) -> OkfPageRecord {
    OkfPageRecord {
        output_path,
        article_path,
        title,
        has_outline: !headings.is_empty(),
        outline: outline_items(headings),
        has_meta: meta.is_some(),
        meta: meta.unwrap_or_else(empty_page_meta),
    }
}

fn concept_page_meta(concept: &Concept, bundle: &Bundle) -> OkfPageMeta {
    let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
    let authority = okf::string_field(&concept.metadata, "authority").unwrap_or("descriptive");
    let trust_tier = okf::search::concept_trust_tier(&concept.metadata);
    let stale = okf::search::concept_is_stale(&concept.metadata);
    let stale_after = okf::string_field(&concept.metadata, "stale_after").unwrap_or("");
    let concept_type = okf::string_field(&concept.metadata, "type").unwrap_or("Concept");
    let owners = okf::metadata_string_array(&concept.metadata, "owners");
    let tags = okf::metadata_string_array(&concept.metadata, "tags");
    let description = okf::string_field(&concept.metadata, "description").unwrap_or("");
    let action = classify_concept_action(concept, &bundle.diagnostics);
    let (trust_slug, trust_label) = match trust_tier {
        TrustTier::HumanReviewed => ("human", "human-reviewed"),
        TrustTier::Generated => ("generated", "generated"),
        TrustTier::Unverified => ("unverified", "unverified"),
    };
    let verifier = okf::latest_human_verification(&concept.metadata)
        .map(|(_, v)| v.to_string())
        .unwrap_or_default();
    let generated = concept
        .metadata
        .get("generated")
        .and_then(Value::as_object)
        .map(|g| {
            let by = g.get("by").and_then(Value::as_str).unwrap_or("");
            let at = g.get("at").and_then(Value::as_str).unwrap_or("");
            if !by.is_empty() && !at.is_empty() {
                format!("{by} @ {at}")
            } else {
                by.to_string()
            }
        })
        .unwrap_or_default();
    let sources_arr = concept.metadata.get("sources").and_then(Value::as_array);
    let has_sources = sources_arr.is_some_and(|s| !s.is_empty());
    let source_count = sources_arr
        .map(|s| s.len().to_string())
        .unwrap_or_else(|| "0".into());
    let drift_diags: Vec<&Diagnostic> = bundle
        .diagnostics
        .iter()
        .filter(|d| d.path == concept.path && d.code == "OKF4006")
        .collect();
    let drift_summary = if has_sources {
        if !drift_diags.is_empty() {
            format!("({} drifted)", drift_diags.len())
        } else {
            "(all clean)".to_string()
        }
    } else {
        String::new()
    };
    let sources = if let Some(sources) = sources_arr {
        sources
            .iter()
            .map(|source| {
                let s_id = source.get("id").and_then(Value::as_str).unwrap_or("-");
                let s_res = source
                    .get("resource")
                    .and_then(Value::as_str)
                    .unwrap_or("-");
                let s_author = source.get("author").and_then(Value::as_str).unwrap_or("-");
                let s_href = source_href(concept, s_res).unwrap_or_default();
                let is_drifted = drift_diags
                    .iter()
                    .any(|d| d.message.contains(&format!("`{s_id}`")));
                OkfSourceItem {
                    id: s_id.to_string(),
                    resource: s_res.to_string(),
                    href: s_href,
                    author: s_author.to_string(),
                    is_drifted,
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    let unknown_fields: Vec<(&String, &Value)> = concept
        .metadata
        .iter()
        .filter(|(k, _)| !STANDARD_FIELDS.contains(&k.as_str()))
        .collect();
    let other_meta = unknown_fields
        .iter()
        .map(|(k, v)| OkfOtherMeta {
            key: (*k).clone(),
            val: compact_json_value(v),
        })
        .collect();
    let has_prov = !owners.is_empty()
        || !verifier.is_empty()
        || !generated.is_empty()
        || !stale_after.is_empty();
    OkfPageMeta {
        concept_type: concept_type.to_string(),
        status: status.to_string(),
        authority: authority.to_string(),
        trust_slug: trust_slug.to_string(),
        trust_label: trust_label.to_string(),
        stale,
        stale_after: stale_after.to_string(),
        is_action_required: action.is_action_required,
        action_detail: action.detail,
        description: description.to_string(),
        has_provenance: has_prov,
        owners: owners.join(", "),
        verifier,
        generated,
        has_sources,
        source_count,
        drift_summary,
        sources,
        has_other_meta: !unknown_fields.is_empty(),
        other_meta_count: unknown_fields.len().to_string(),
        other_meta,
        has_tags: !tags.is_empty(),
        tags,
    }
}

fn generate_okf_page_data(bundle: &Bundle) -> Result<GeneratedOkfPages> {
    let mut pages = Vec::new();
    let mut articles = Vec::new();
    let mut output_paths = Vec::new();

    for concept in &bundle.concepts {
        let article = article_with_validation(bundle, Some(&concept.path), &concept.article_html);
        let (stamped_article, headings) = stamp_and_collect_headings(&article);
        let article_rel = format!("articles/{}.html", concept.id.replace('/', "-"));
        articles.push((article_rel.clone(), stamped_article));
        let out_path = format!("{}/index.html", concept.id);
        output_paths.push(out_path.clone());
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        pages.push(page_record(
            out_path,
            article_rel,
            title.to_string(),
            &headings,
            Some(concept_page_meta(concept, bundle)),
        ));
    }

    if let Some(index) = bundle.indexes.iter().find(|i| i.path == "index.md") {
        let home = article_with_validation(
            bundle,
            Some("index.md"),
            &format!(
                "{}{}",
                render_home_page_governance(bundle),
                index.article_html
            ),
        );
        let (stamped, headings) = stamp_and_collect_headings(&home);
        let article_rel = "articles/index.html".to_string();
        articles.push((article_rel.clone(), stamped));
        output_paths.push("index.html".to_string());
        pages.push(page_record(
            "index.html".into(),
            article_rel,
            "Knowledge".into(),
            &headings,
            None,
        ));
    }

    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        let article = article_with_validation(bundle, Some(&index.path), &index.article_html);
        let (stamped, headings) = stamp_and_collect_headings(&article);
        let article_rel = format!("articles/{collection}-index.html");
        articles.push((article_rel.clone(), stamped));
        let out_path = format!("{collection}/index.html");
        output_paths.push(out_path.clone());
        pages.push(page_record(
            out_path,
            article_rel,
            collection.to_string(),
            &headings,
            None,
        ));
    }

    let review = article_with_validation(bundle, None, &render_review_page(bundle));
    let (stamped, headings) = stamp_and_collect_headings(&review);
    let article_rel = "articles/review.html".to_string();
    articles.push((article_rel.clone(), stamped));
    output_paths.push("review/index.html".to_string());
    pages.push(page_record(
        "review/index.html".into(),
        article_rel,
        "Knowledge Governance & Review Queue".into(),
        &headings,
        None,
    ));

    for path in diagnostic_only_concept_paths(bundle) {
        let id = concept_id_from_path(path);
        let (stamped, headings) = stamp_and_collect_headings(&stub_article(bundle, path));
        let article_rel = format!("articles/{}-unparsed.html", id.replace('/', "-"));
        articles.push((article_rel.clone(), stamped));
        let out_path = format!("{id}/index.html");
        output_paths.push(out_path.clone());
        pages.push(page_record(
            out_path,
            article_rel,
            id.to_string(),
            &headings,
            None,
        ));
    }

    let json = serde_json::to_string(&OkfPagesFile { pages })
        .context("failed to serialize OKF page records")?;
    Ok(GeneratedOkfPages {
        json,
        articles,
        output_paths,
    })
}

fn invoke_roc_build(
    workspace: &Path,
    apply_bin: &Path,
    maps: &[rocci_template::MappedModule],
) -> Result<String> {
    let output = std::process::Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--opt=dev")
        .arg(format!("--output={}", apply_bin.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build")?;
    let combined = finish_roc_output(output, maps)?;
    if !apply_bin.is_file() {
        bail!("roc build did not write {}", apply_bin.display());
    }
    Ok(combined)
}

fn invoke_roc_wasm_build(
    workspace: &Path,
    wasm_file: &Path,
    maps: &[rocci_template::MappedModule],
) -> Result<String> {
    let output = std::process::Command::new("roc")
        .arg("build")
        .arg("main.roc")
        .arg("--target=wasm32")
        .arg(format!("--output={}", wasm_file.display()))
        .current_dir(workspace)
        .output()
        .context("failed to invoke roc build for wasm32")?;
    let combined = finish_roc_output(output, maps)?;
    if !wasm_file.is_file() {
        bail!("roc build did not write {}", wasm_file.display());
    }
    Ok(combined)
}

fn invoke_wasm_apply(wasm_file: &Path, staging: &Path) -> Result<String> {
    let host = rocci_roc_host::WasmHost::from_file(wasm_file)?;
    host.run_wasi(staging)
}

fn invoke_apply(
    apply_bin: &Path,
    workspace: &Path,
    staging: &Path,
    maps: &[rocci_template::MappedModule],
) -> Result<String> {
    let output = std::process::Command::new(apply_bin)
        .current_dir(workspace)
        .env("OKF_STAGING", staging)
        .output()
        .context("failed to run okf applicator")?;
    finish_roc_output(output, maps)
}

fn finish_roc_output(
    output: std::process::Output,
    maps: &[rocci_template::MappedModule],
) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = if stdout.is_empty() {
        stderr.clone()
    } else if stderr.is_empty() {
        stdout.clone()
    } else {
        format!("{stdout}{stderr}")
    };
    if output.status.success() {
        return Ok(combined);
    }
    let mapped = rocci_template::remap_roc_output(&combined, maps);
    for frame in mapped {
        eprintln!("{}", frame.render_for_stderr());
    }
    let hint = if combined.contains("does not support the wasm32 target") {
        "\n\nhint: The basic-cli platform only supports native compilation targets (x64mac, arm64mac, x64win, x64musl, arm64musl).\nWasm host (--host wasm) is planned for Phase 5 with a custom Roc wasm platform.\nPlease use '--host native' (or default '--host auto') instead."
    } else {
        ""
    };
    bail!(
        "roc okf build failed{}{hint}",
        if combined.trim().is_empty() {
            String::new()
        } else {
            format!("\n{}", combined.trim_end())
        }
    );
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn write_site_chrome(bundle: &Bundle, site: &Path) -> Result<()> {
    let okf_static_dir = site.join("__rocci_okf");
    fs::create_dir_all(&okf_static_dir)
        .with_context(|| format!("failed to create {}", okf_static_dir.display()))?;
    fs::write(okf_static_dir.join("goto.js"), rocci_ui::chrome_script())
        .context("failed to write knowledge goto script")?;

    let catalog = bundle
        .concepts
        .iter()
        .map(ConceptInspect::from)
        .collect::<Vec<_>>();
    fs::write(
        site.join("catalog.json"),
        format!("{}\n", serde_json::to_string_pretty(&catalog)?),
    )
    .context("failed to write knowledge catalog")?;

    fs::write(
        site.join("pages.json"),
        format!("{}\n", serde_json::to_string_pretty(&nav_pages(bundle))?),
    )
    .context("failed to write knowledge page index")?;
    Ok(())
}

#[derive(Serialize)]
struct NavPage {
    title: String,
    route: String,
    path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
}

fn nav_pages(bundle: &Bundle) -> Vec<NavPage> {
    let mut pages = Vec::new();
    pages.push(NavPage {
        title: "Knowledge".into(),
        route: "/".into(),
        path: "index.md".into(),
        description: String::new(),
    });
    pages.push(NavPage {
        title: "Governance & Review".into(),
        route: "/review/".into(),
        path: "review".into(),
        description: String::new(),
    });
    for concept in &bundle.concepts {
        let id = concept.id.trim_matches('/');
        pages.push(NavPage {
            title: okf::string_field(&concept.metadata, "title")
                .unwrap_or(&concept.id)
                .to_string(),
            route: format!("/{id}/"),
            path: concept.path.clone(),
            description: okf::string_field(&concept.metadata, "description")
                .unwrap_or("")
                .to_string(),
        });
    }
    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        pages.push(NavPage {
            title: collection.to_string(),
            route: format!("/{collection}/"),
            path: index.path.clone(),
            description: String::new(),
        });
    }
    pages.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.path.cmp(&right.path))
    });
    pages
}

#[allow(dead_code)]
pub fn build_review_site_pure_rust(bundle: &Bundle, site: &Path) -> Result<()> {
    fs::create_dir_all(site).with_context(|| format!("failed to create {}", site.display()))?;

    for concept in &bundle.concepts {
        let destination = site.join(&concept.id).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        let meta_header = render_concept_meta(concept, bundle);
        let article = article_with_validation(bundle, Some(&concept.path), &concept.article_html);
        let full_html = with_meta_and_article(&meta_header, &article);
        fs::write(&destination, html_page(title, &full_html))
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        let governance_header = render_home_page_governance(bundle);
        let article = article_with_validation(bundle, Some("index.md"), &index.article_html);
        let full_index = with_meta_and_article(&governance_header, &article);
        fs::write(site.join("index.html"), html_page("Knowledge", &full_index))
            .context("failed to write knowledge index")?;
    }
    for index in &bundle.indexes {
        let Some(collection) = index.path.strip_suffix("/index.md") else {
            continue;
        };
        let destination = site.join(collection).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(
            &destination,
            html_page(
                collection,
                &select_root_article(&article_with_validation(
                    bundle,
                    Some(&index.path),
                    &index.article_html,
                )),
            ),
        )
        .with_context(|| format!("failed to write {}", destination.display()))?;
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
            &select_root_article(&article_with_validation(
                bundle,
                None,
                &render_review_page(bundle),
            )),
        ),
    )
    .context("failed to write knowledge review page")?;

    for path in diagnostic_only_concept_paths(bundle) {
        let id = concept_id_from_path(path);
        let destination = site.join(id).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(
            &destination,
            html_page(id, &select_root_article(&stub_article(bundle, path))),
        )
        .with_context(|| format!("failed to write {}", destination.display()))?;
    }

    let okf_static_dir = site.join("__rocci_okf");
    fs::create_dir_all(&okf_static_dir)
        .with_context(|| format!("failed to create {}", okf_static_dir.display()))?;
    fs::write(okf_static_dir.join("app.css"), DEFAULT_CSS)
        .context("failed to write knowledge review stylesheet")?;
    write_site_chrome(bundle, site)?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ApplySession {
    pub compile_hash: String,
    pub apply_path: PathBuf,
    pub is_wasm: bool,
}

pub fn build_review_site_with_host(
    bundle: &Bundle,
    site: &Path,
    host: Option<rocci_roc_host::HostChoice>,
) -> Result<ProfileSnapshot> {
    let (snapshot, _) = build_review_site_with_session(bundle, site, host, None)?;
    Ok(snapshot)
}

pub fn build_review_site_with_session(
    bundle: &Bundle,
    site: &Path,
    host: Option<rocci_roc_host::HostChoice>,
    session: Option<&ApplySession>,
) -> Result<(ProfileSnapshot, Option<ApplySession>)> {
    let mut rec = SpanRecorder::new();
    let host_choice = host.unwrap_or(rocci_roc_host::HostChoice::Auto).resolve();
    let is_wasm = host_choice == rocci_roc_host::HostChoice::Wasm;
    let force_roc =
        host.is_some() || std::env::var("ROCCI_REQUIRE_ROC").ok().as_deref() == Some("1");

    if !force_roc && !is_roc_available() && !is_wasm {
        rec.span("write", || build_review_site_pure_rust(bundle, site))?;
        return Ok((rec.finish(), None));
    }

    let modules = rec.span("compile templates", compile_okf_templates)?;
    let generated = rec.span("generate", || generate_okf_page_data(bundle))?;

    let workspace = unique_temp("ws")?;
    let staging = unique_temp("stage")?;

    crate::runtime::stage_into(&workspace)?;

    for module in &modules {
        fs::write(
            workspace.join(format!("{}.roc", module.type_name)),
            &module.roc,
        )?;
    }

    let articles_dir = workspace.join("articles");
    fs::create_dir_all(&articles_dir)?;
    for (rel_path, html) in &generated.articles {
        let dest = workspace.join(rel_path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, html)?;
    }

    for out_path in &generated.output_paths {
        if let Some(parent) = Path::new(out_path).parent()
            && parent != Path::new("")
        {
            fs::create_dir_all(staging.join(parent))?;
        }
    }

    if is_wasm {
        rocci_roc_host::stage_wasm_platform_into(&workspace)?;
    }

    fs::write(workspace.join("okf-pages.json"), &generated.json)?;
    let main_code = main_roc(is_wasm);
    fs::write(workspace.join("main.roc"), &main_code)?;

    let roc_hash = renderer_compile_hash(&modules, &main_code, is_wasm);
    let cache = rocci_roc_host::TwoTierCache::default();
    let target = if is_wasm {
        "wasm32".to_string()
    } else {
        format!("native:{}", std::env::consts::ARCH)
    };

    let apply_bin = workspace.join(if is_wasm { "components.wasm" } else { "apply" });
    let maps: Vec<rocci_template::MappedModule> = modules
        .iter()
        .map(|m| rocci_template::MappedModule {
            type_name: m.type_name.clone(),
            generated: m.roc.clone(),
            source_name: m.source_name.clone(),
            source_src: m.src.clone(),
            segments: m.segments.clone(),
        })
        .collect();

    let apply_path = if let Some(prev) = session
        && prev.compile_hash == roc_hash
        && prev.is_wasm == is_wasm
        && prev.apply_path.is_file()
    {
        eprintln!(
            "rocci-okf: reusing {} applicator {}",
            if is_wasm { "wasm" } else { "native" },
            &roc_hash[..8.min(roc_hash.len())]
        );
        rec.push("compile", 0, Some("cached".into()));
        prev.apply_path.clone()
    } else if let Some(cached) = cache.lookup_renderer(&roc_hash, &target) {
        eprintln!(
            "rocci-okf: using cached {} renderer for {}",
            if is_wasm { "wasm" } else { "native" },
            &roc_hash[..8.min(roc_hash.len())]
        );
        rec.push("compile", 0, Some("cached".into()));
        cached
    } else {
        eprintln!(
            "rocci-okf: compiling ({}) with roc at {}",
            if is_wasm { "wasm32" } else { "native" },
            workspace.display()
        );
        rec.span("compile", || {
            let roc_started = Instant::now();
            let roc_output = if is_wasm {
                invoke_roc_wasm_build(&workspace, &apply_bin, &maps)
                    .with_context(|| format!("workspace {}", workspace.display()))?
            } else {
                invoke_roc_build(&workspace, &apply_bin, &maps)
                    .with_context(|| format!("workspace {}", workspace.display()))?
            };
            eprintln!(
                "rocci-okf: roc finished in {}ms",
                roc_started.elapsed().as_millis()
            );
            if !roc_output.is_empty() {
                eprint!("{roc_output}");
            }
            let bytes = fs::read(&apply_bin)?;
            let fp = okf_fingerprints(&modules, &main_code);
            cache.store_renderer(&roc_hash, &target, &bytes, &fp)
        })?
    };

    let reused_apply = session.is_some_and(|prev| {
        prev.compile_hash == roc_hash && prev.is_wasm == is_wasm && prev.apply_path == apply_path
    });
    rec.span_with_note("render", reused_apply.then(|| "reuse".into()), || {
        let roc_output = if is_wasm {
            invoke_wasm_apply(&apply_path, &staging)
        } else {
            invoke_apply(&apply_path, &workspace, &staging, &maps)
        }?;
        if !roc_output.is_empty() {
            eprint!("{roc_output}");
        }
        Ok::<(), anyhow::Error>(())
    })?;

    rec.span("write", || {
        for concept in &bundle.concepts {
            let destination = staging.join(&concept.id).join("index.html");
            if !destination.exists() {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
                let meta_header = render_concept_meta(concept, bundle);
                let full_html = with_meta_and_article(&meta_header, &concept.article_html);
                fs::write(&destination, html_page(title, &full_html))?;
            }
        }
        if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
            let destination = staging.join("index.html");
            if !destination.exists() {
                let governance_header = render_home_page_governance(bundle);
                let full_index = with_meta_and_article(&governance_header, &index.article_html);
                fs::write(&destination, html_page("Knowledge", &full_index))?;
            }
        }
        for index in &bundle.indexes {
            let Some(collection) = index.path.strip_suffix("/index.md") else {
                continue;
            };
            let destination = staging.join(collection).join("index.html");
            if !destination.exists() {
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(
                    &destination,
                    html_page(collection, &select_root_article(&index.article_html)),
                )?;
            }
        }
        let review_dest = staging.join("review").join("index.html");
        if !review_dest.exists() {
            if let Some(parent) = review_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                &review_dest,
                html_page(
                    "Knowledge Governance & Review Queue",
                    &select_root_article(&render_review_page(bundle)),
                ),
            )?;
        }

        fs::create_dir_all(site).with_context(|| format!("failed to create {}", site.display()))?;
        copy_dir_recursive(&staging, site)?;

        let okf_static_dir = site.join("__rocci_okf");
        fs::create_dir_all(&okf_static_dir)
            .with_context(|| format!("failed to create {}", okf_static_dir.display()))?;
        fs::write(okf_static_dir.join("app.css"), DEFAULT_CSS)
            .context("failed to write knowledge review stylesheet")?;
        write_site_chrome(bundle, site)?;

        let _ = fs::remove_dir_all(&workspace);
        let _ = fs::remove_dir_all(&staging);
        Ok::<(), anyhow::Error>(())
    })?;

    let next_session = ApplySession {
        compile_hash: roc_hash,
        apply_path,
        is_wasm,
    };
    Ok((rec.finish(), Some(next_session)))
}

pub const DEFAULT_CSS: &str = r#"
:root {
  color-scheme: dark;
  --rd-font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
  --rd-font-mono: ui-monospace, "SF Mono", Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  --rd-bg: #282c34;
  --rd-fg: #abb2bf;
  --rd-muted: #9da5b4;
  --rd-border: #3e4451;
  --rd-border-subtle: #21252b;
  --rd-bg-subtle: #21252b;
  --rd-primary: #61afef;
  --rd-green: #98c379;
  --rd-orange: #d19a66;
  --rd-red: #e06c75;
  --rd-purple: #c678dd;
}
html.rd-document, body {
  font-family: var(--rd-font-sans);
  background: var(--rd-bg);
  color: var(--rd-fg);
  margin: 0;
  min-height: calc(100vh - var(--rocci-chrome-top, 0px) - var(--rocci-chrome-bottom, 0px));
  line-height: 1.65;
}
html.rd-document { scroll-behavior: smooth; }
.rd-shell {
  display: grid;
  grid-template-columns: 16.5rem minmax(0, 1fr);
  align-items: start;
  min-height: calc(100vh - var(--rocci-chrome-top, 0px) - var(--rocci-chrome-bottom, 0px));
}
.okf-chrome {
  position: sticky;
  top: var(--rocci-chrome-top, 0px);
  box-sizing: border-box;
  min-width: 0;
  max-height: calc(100vh - var(--rocci-chrome-top, 0px) - var(--rocci-chrome-bottom, 0px));
  padding: 2.15rem 1.2rem 2rem 1.5rem;
  overflow-x: hidden;
  overflow-y: auto;
  user-select: none;
}
.okf-global-nav {
  display: grid;
  gap: 0.45rem;
  margin-bottom: 1.5rem;
}
.okf-global-label,
.rd-toc-label,
.outline-label {
  margin: 0 0 0.65rem;
  color: var(--rd-muted);
  font-size: 0.68rem;
  font-weight: 700;
  letter-spacing: 0.105em;
  text-transform: uppercase;
}
.okf-global-link {
  color: var(--rd-fg);
  font-size: 0.85rem;
  font-weight: 600;
  line-height: 1.35;
  text-decoration: none;
  min-height: 1.5rem;
}
.okf-global-link:hover { color: var(--rd-primary); text-decoration: none; }
main {
  box-sizing: border-box;
  min-width: 0;
  width: min(42rem, calc(100% - 2rem));
  margin: 0 auto;
  padding: 2.5rem 0 4rem;
}
.rd-toc {
  box-sizing: border-box;
  min-width: 0;
  padding: 0;
  overflow: visible;
  user-select: none;
}
.rd-toc .outline {
  display: block;
  padding: 0;
  margin: 0;
  background: transparent;
  border: 0;
}
.rd-toc-items,
.outline-items {
  display: grid;
  gap: 0.45rem;
  border-left: 1px solid var(--rd-border);
}
.rd-toc-link,
.outline-link {
  margin-left: -1px;
  padding-left: 0.8rem;
  border-left: 1px solid transparent;
  color: var(--rd-muted);
  font-size: 0.78rem;
  line-height: 1.35;
  text-decoration: none;
  overflow-wrap: anywhere;
}
.rd-toc-link:hover,
.outline-link:hover {
  border-color: var(--rd-primary);
  color: var(--rd-fg);
  text-decoration: none;
}
.rd-toc-link.rd-toc-level-3,
.outline-link.level-3 { padding-left: 1.35rem; }
.rd-toc:not(:has(.rd-toc-link)):not(:has(.outline-link)) { display: none; }
.okf-outline-menu { display: none; }
.okf-outline-menu .outline-label { display: none; }
@media (max-width: 48rem) {
  .rd-shell { display: block; }
  .okf-chrome {
    position: static;
    max-height: none;
    padding: 1rem 1.25rem 0.25rem;
    overflow: visible;
  }
  .okf-global-nav {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.35rem 1.1rem;
    margin-bottom: 0.35rem;
  }
  .okf-global-label { margin: 0; }
  .rd-toc { display: none; }
  .okf-outline-menu { display: block; margin: 0 0 1.25rem; }
  .okf-outline-menu summary {
    display: flex;
    align-items: center;
    min-height: 2.75rem;
    padding: 0 0.85rem;
    border: 1px solid var(--rd-border);
    border-radius: 0.5rem;
    background: var(--rd-bg-subtle);
    color: var(--rd-fg);
    font-size: 0.85rem;
    font-weight: 650;
    cursor: pointer;
    list-style: none;
  }
  .okf-outline-menu summary::-webkit-details-marker { display: none; }
  .okf-outline-panel {
    margin-top: 0.65rem;
    padding: 0.75rem 0.15rem 0.25rem;
  }
}
@media print {
  .rd-toc, .okf-outline-menu { display: none; }
}
@media (prefers-reduced-motion: reduce) {
  html.rd-document { scroll-behavior: auto; }
}
h1, h2, h3, h4, h5, h6,
.rd-header-1, .rd-header-2, .rd-header-3, .rd-header-4, .rd-header-5, .rd-header-6 {
  color: var(--rd-fg);
  font-weight: 700;
  line-height: 1.25;
  scroll-margin-top: calc(1.25rem + var(--rocci-chrome-top, 0px));
}
h1, .rd-header-1 { margin: 0 0 0.75rem; font-size: 2rem; letter-spacing: -0.03em; }
h2, .rd-header-2 { margin: 2rem 0 0.6rem; font-size: 1.35rem; }
h3, .rd-header-3 { margin: 1.5rem 0 0.5rem; font-size: 1.15rem; }
p, .rd-paragraph { margin: 0 0 1rem; color: var(--rd-fg); }
a { color: var(--rd-primary); text-decoration: none; }
a:hover { color: var(--rd-fg); text-decoration: underline; }
ul, ol { color: var(--rd-fg); }
blockquote {
  margin: 0 0 1rem;
  padding: 0.2rem 0 0.2rem 1rem;
  border-left: 3px solid var(--rd-primary);
  color: var(--rd-muted);
}
pre {
  margin: 0 0 1.25rem;
  padding: 1rem 1.1rem;
  overflow-x: auto;
  border: 1px solid var(--rd-border);
  border-radius: 0.5rem;
  background: var(--rd-bg-subtle);
}
code { font-family: var(--rd-font-mono); font-size: 0.9em; color: var(--rd-red); background: var(--rd-bg-subtle); padding: 0.2em 0.4em; border-radius: 4px; }
pre code { color: var(--rd-fg); background: transparent; padding: 0; }
table { width: 100%; border-collapse: collapse; margin: 0 0 1.25rem; }
th, td { padding: 0.4rem 0.6rem; border: 1px solid var(--rd-border); text-align: left; }
th { background: var(--rd-bg-subtle); color: var(--rd-fg); }
hr { border: 0; border-top: 1px solid var(--rd-border); margin: 1.5rem 0; }
.okf-badge-group { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.75rem; }
.okf-badge { font-size: 0.8rem; padding: 0.2rem 0.5rem; border-radius: 9999px; border: 1px solid var(--rd-border); font-weight: 500; }
.okf-type { background: var(--rd-bg-subtle); }
.okf-status-stable, .okf-trust-human, .pill-clean { background: rgba(152, 195, 121, 0.15); color: var(--rd-green); border-color: var(--rd-green); }
.okf-status-draft, .okf-trust-generated, .pill-action { background: rgba(209, 154, 102, 0.15); color: var(--rd-orange); border-color: var(--rd-orange); }
.okf-status-deprecated, .pill-error { background: rgba(224, 108, 117, 0.15); color: var(--rd-red); border-color: var(--rd-red); }
.okf-auth-normative, .pill-info { background: rgba(97, 175, 239, 0.15); color: var(--rd-primary); border-color: var(--rd-primary); }
.okf-auth-exploratory { background: rgba(198, 120, 221, 0.15); color: var(--rd-purple); border-color: var(--rd-purple); }
.okf-auth-descriptive, .okf-trust-unverified { background: var(--rd-bg-subtle); color: var(--rd-muted); }
.okf-alert-banner { display: flex; gap: 0.5rem; background: rgba(209, 154, 102, 0.12); border: 1px solid var(--rd-orange); padding: 0.75rem 1rem; border-radius: 6px; margin: 1rem 0; }
.okf-error-banner { display: flex; flex-direction: column; gap: 0.5rem; background: rgba(224, 108, 117, 0.12); border: 1px solid var(--rd-red); padding: 0.75rem 1rem; border-radius: 6px; margin: 0 0 1.25rem; }
.okf-error-banner p, .okf-alert-banner p { margin: 0; }
.okf-diagnostics-list { display: flex; flex-direction: column; gap: 0.35rem; }
.okf-diagnostic-item { font-size: 0.9rem; }
.okf-concept-meta { margin-bottom: 1.5rem; padding-bottom: 1rem; border-bottom: 1px solid var(--rd-border); user-select: none; }
.okf-lead { color: var(--rd-muted); margin: 0 0 0.75rem; }
.okf-provenance { display: flex; flex-wrap: wrap; gap: 0.25rem 1.25rem; list-style: none; padding: 0; margin: 0 0 0.75rem; font-size: 0.9rem; }
.okf-meta-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; background: var(--rd-bg-subtle); padding: 1rem; border-radius: 6px; margin-bottom: 1rem; font-size: 0.9rem; }
.okf-meta-label { font-weight: 600; margin-right: 0.5rem; }
.okf-sources-drawer, .okf-other-meta { margin: 0.5rem 0; }
.okf-sources-table { width: 100%; border-collapse: collapse; margin-top: 0.5rem; font-size: 0.85rem; }
.okf-sources-table th, .okf-sources-table td { padding: 0.4rem 0.5rem; border: 1px solid var(--rd-border); text-align: left; vertical-align: top; }
.okf-tags { display: flex; flex-wrap: wrap; gap: 0.35rem; margin-top: 0.5rem; }
.okf-tag { font-size: 0.8rem; color: var(--rd-muted); }
.okf-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 1rem; margin-bottom: 2rem; }
.okf-stat-card { background: var(--rd-bg-subtle); border: 1px solid var(--rd-border); padding: 1rem; border-radius: 8px; text-align: center; }
.okf-stat-value { font-size: 1.8rem; font-weight: bold; }
.okf-stat-label { font-size: 0.85rem; color: var(--rd-muted); }
.okf-stat-card.is-action .okf-stat-value { color: var(--rd-red); }
.okf-review-table { width: 100%; border-collapse: collapse; margin-top: 1rem; font-size: 0.9rem; }
.okf-review-table th, .okf-review-table td { padding: 0.75rem; border: 1px solid var(--rd-border); text-align: left; vertical-align: top; }
.okf-review-table th { background: var(--rd-bg-subtle); }
.okf-action-pill { display: inline-block; padding: 0.2rem 0.6rem; border-radius: 9999px; font-size: 0.8rem; font-weight: 600; }
.okf-action-detail-text { font-size: 0.8rem; color: var(--rd-muted); margin-top: 0.25rem; }
.okf-table-container { overflow-x: auto; -webkit-overflow-scrolling: touch; }
.okf-filter-bar { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 1rem; align-items: center; }
.okf-filter-btn { padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--rd-border); background: var(--rd-bg); color: var(--rd-fg); cursor: pointer; font-size: 0.85rem; }
.okf-filter-btn.is-active { background: var(--rd-primary); color: #282c34; border-color: var(--rd-primary); }
.okf-search-input { flex: 1; padding: 0.4rem 0.8rem; border-radius: 6px; border: 1px solid var(--rd-border); background: var(--rd-bg); color: var(--rd-fg); }
.okf-cta-row { display: flex; gap: 1rem; align-items: center; margin: 1.5rem 0; }
.okf-cta-btn { background: var(--rd-primary); color: #282c34; padding: 0.6rem 1.2rem; border-radius: 6px; font-weight: 500; }
.okf-cta-btn:hover { text-decoration: none; opacity: 0.9; }
"#;

const TOC_SCRIPT: &str = rocci_ui::TOC_SCRIPT;

struct TocHeading {
    level: u8,
    id: String,
    text: String,
}

fn select_root_article(html: &str) -> String {
    format!("<article class=\"rd-article\">{html}</article>")
}

fn with_meta_and_article(meta: &str, article: &str) -> String {
    format!("{meta}{}", select_root_article(article))
}

pub(crate) fn diagnostic_only_concept_paths(bundle: &Bundle) -> Vec<&str> {
    let published: HashSet<&str> = bundle
        .concepts
        .iter()
        .map(|concept| concept.path.as_str())
        .chain(bundle.indexes.iter().map(|index| index.path.as_str()))
        .chain(bundle.logs.iter().map(|log| log.path.as_str()))
        .collect();
    let mut paths: Vec<&str> = bundle
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.path.as_str())
        .filter(|path| {
            !published.contains(path)
                && path.ends_with(".md")
                && *path != "index.md"
                && *path != "log.md"
                && !path.ends_with("/index.md")
                && !path.ends_with("/log.md")
        })
        .collect();
    paths.sort_unstable();
    paths.dedup();
    paths
}

fn concept_id_from_path(path: &str) -> &str {
    path.strip_suffix(".md").unwrap_or(path)
}

fn article_with_validation(bundle: &Bundle, path: Option<&str>, article: &str) -> String {
    format!("{}{article}", render_validation_notice(bundle, path))
}

fn stub_article(bundle: &Bundle, path: &str) -> String {
    format!(
        "{}<p class=\"rd-paragraph\">This document could not be fully parsed. The viewer still shows the validation errors above.</p>\n",
        render_validation_notice(bundle, Some(path))
    )
}

pub fn render_validation_notice(bundle: &Bundle, current_path: Option<&str>) -> String {
    let path_diagnostics: Vec<&Diagnostic> = current_path
        .map(|path| {
            bundle
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.path == path)
                .collect()
        })
        .unwrap_or_default();
    let other_errors: Vec<&Diagnostic> = bundle
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.severity == Severity::Error
                && current_path.is_none_or(|path| diagnostic.path != path)
        })
        .collect();
    if path_diagnostics.is_empty() && other_errors.is_empty() {
        return String::new();
    }
    let has_error = path_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
        || !other_errors.is_empty();
    let class = if has_error {
        "okf-error-banner"
    } else {
        "okf-alert-banner"
    };
    let heading = if has_error {
        "Validation errors"
    } else {
        "Validation warnings"
    };
    let mut out = format!("<div class=\"{class}\" role=\"alert\">\n");
    out.push_str(&format!(
        "  <p><strong>{}</strong> — the document below still renders as far as possible.</p>\n",
        heading
    ));
    out.push_str("  <div class=\"okf-diagnostics-list\">\n");
    for diagnostic in path_diagnostics.iter().chain(other_errors.iter()) {
        let sev_badge = match diagnostic.severity {
            Severity::Error => "<span class=\"okf-badge okf-status-deprecated\">Error</span>",
            Severity::Warning => "<span class=\"okf-badge okf-status-draft\">Warning</span>",
        };
        out.push_str(&format!(
            "    <div class=\"okf-diagnostic-item\">{} <code>{}</code> <strong>{}</strong>: {}</div>\n",
            sev_badge,
            escape(diagnostic.code),
            escape(&diagnostic.path),
            escape(&diagnostic.message)
        ));
    }
    out.push_str("  </div>\n");
    out.push_str("  <p><a href=\"/review/#diagnostics\">All bundle diagnostics</a></p>\n");
    out.push_str("</div>\n");
    out
}

const OKF_GLOBAL_NAV: &str = concat!(
    "<nav class=\"okf-global-nav\" aria-label=\"Knowledge\">",
    "<p class=\"okf-global-label\">Knowledge</p>",
    "<a class=\"okf-global-link\" href=\"/\">Home</a>",
    "<a class=\"okf-global-link\" href=\"/review/\">Governance &amp; Review</a>",
    "</nav>",
);

pub fn html_page(title: &str, article: &str) -> String {
    let (article, headings) = stamp_and_collect_headings(article);
    let mut chrome = String::from("<div class=\"okf-chrome\">");
    chrome.push_str(OKF_GLOBAL_NAV);
    if !headings.is_empty() {
        chrome.push_str(&render_toc(&headings));
    }
    chrome.push_str("</div>");
    let mut main = String::from("<main>");
    if !headings.is_empty() {
        main.push_str(&render_outline_details(&headings));
    }
    main.push_str(&article);
    main.push_str("</main>");
    let script = if headings.is_empty() {
        String::new()
    } else {
        format!("<script>{TOC_SCRIPT}</script>")
    };
    let body = format!("<div class=\"rd-shell\">{chrome}{main}</div>{script}");
    format!(
        "<!doctype html><html lang=\"en\" class=\"rd-document\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark\"><title>{}</title><link rel=\"stylesheet\" href=\"/__rocci_okf/app.css\"><script src=\"/__rocci_okf/goto.js\" defer></script><script src=\"/__rocci_okf/reload.js\" defer></script></head><body>{body}</body></html>\n",
        escape(title)
    )
}

fn toc_links(headings: &[TocHeading], class_for_level3: &str, class_default: &str) -> String {
    let mut out = String::new();
    for heading in headings {
        let class = if heading.level == 3 {
            class_for_level3
        } else {
            class_default
        };
        out.push_str(&format!(
            "<a class=\"{class}\" href=\"#{}\">{}</a>",
            escape(&heading.id),
            escape(&heading.text)
        ));
    }
    out
}

fn render_toc(headings: &[TocHeading]) -> String {
    format!(
        "<nav class=\"rd-toc\" aria-label=\"On this page\"><p class=\"rd-toc-label\">On this page</p><div class=\"rd-toc-items\">{}</div></nav>",
        toc_links(headings, "rd-toc-link rd-toc-level-3", "rd-toc-link")
    )
}

fn render_outline_details(headings: &[TocHeading]) -> String {
    format!(
        "<details class=\"okf-outline-menu\"><summary>On this page</summary><div class=\"okf-outline-panel\"><nav class=\"outline\" aria-label=\"On this page\"><div class=\"outline-items\">{}</div></nav></div></details>",
        toc_links(headings, "outline-link level-3", "outline-link")
    )
}

fn stamp_and_collect_headings(html: &str) -> (String, Vec<TocHeading>) {
    let mut out = String::with_capacity(html.len() + 32);
    let mut headings = Vec::new();
    let mut used_ids = HashSet::new();
    let mut i = 0;
    while i < html.len() {
        let before = i;
        if let Some((consumed, fragment, heading)) = try_heading(html, i, &mut used_ids) {
            out.push_str(&fragment);
            if let Some(heading) = heading
                && (2..=3).contains(&heading.level)
            {
                headings.push(heading);
            }
            i += consumed;
        } else {
            let ch = html[i..].chars().next().unwrap_or('\0');
            out.push(ch);
            i += ch.len_utf8();
        }
        if i <= before {
            i += 1;
        }
    }
    (out, headings)
}

fn try_heading(
    html: &str,
    i: usize,
    used_ids: &mut HashSet<String>,
) -> Option<(usize, String, Option<TocHeading>)> {
    let rest = html.get(i..)?;
    let bytes = rest.as_bytes();
    if bytes.len() < 4 || bytes[0] != b'<' || bytes[1] != b'h' {
        return None;
    }
    let level_byte = bytes[2];
    if !(b'1'..=b'6').contains(&level_byte) {
        return None;
    }
    let after = *bytes.get(3)?;
    if after != b'>' && after != b'/' && !after.is_ascii_whitespace() {
        return None;
    }
    let gt = rest.find('>')?;
    let open = rest.get(..gt + 1)?;
    let level = level_byte - b'0';
    let close = format!("</h{level}>");
    let inner_start = gt + 1;
    let inner_html = rest.get(inner_start..)?;
    let close_at = inner_html.find(&close)?;
    let inner = &inner_html[..close_at];
    let consumed = inner_start + close_at + close.len();
    let text = unescape_text(&strip_tags(inner));
    let id = match heading_id(open) {
        Some(existing) => {
            used_ids.insert(existing.clone());
            existing
        }
        None => unique_heading_id(used_ids, &text),
    };
    let open_with_id = ensure_heading_id(open, &id);
    let fragment = format!("{open_with_id}{inner}{close}");
    let heading = TocHeading { level, id, text };
    Some((consumed, fragment, Some(heading)))
}

fn heading_id(open: &str) -> Option<String> {
    let lower = open.to_ascii_lowercase();
    let key = "id=";
    let pos = lower.find(key)?;
    let rest = open.get(pos + key.len()..)?;
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)?;
        Some(rest[1..1 + end].to_string())
    } else {
        let end = rest
            .find(|ch: char| ch.is_ascii_whitespace() || ch == '>')
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn ensure_heading_id(open: &str, id: &str) -> String {
    if heading_id(open).is_some() {
        return open.to_string();
    }
    let mut tagged = open.to_string();
    let insert_at = tagged.len().saturating_sub(1);
    tagged.insert_str(insert_at, &format!(" id=\"{}\"", escape(id)));
    tagged
}

fn unique_heading_id(used_ids: &mut HashSet<String>, text: &str) -> String {
    let mut base = slugify(text);
    if base.is_empty() {
        base = "heading".into();
    }
    let mut id = base.clone();
    let mut n = 1;
    while used_ids.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    used_ids.insert(id.clone());
    id
}

fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn unescape_text(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    use okf::{SourceLocation, Span};
    use serde_json::json;

    fn concept_with(metadata: BTreeMap<String, Value>) -> Concept {
        Concept {
            id: "plans/example".into(),
            path: "plans/example.md".into(),
            metadata,
            body_span: Span::new(0, 0),
            body_location: SourceLocation {
                start: 0,
                end: 0,
                line: 1,
                column: 1,
            },
            headings: Vec::new(),
            heading_sections: Vec::new(),
            links: Vec::new(),
            source_ids: BTreeSet::new(),
            footnote_ids: BTreeSet::new(),
            article_html: String::new(),
        }
    }

    fn bundle_with(concepts: Vec<Concept>) -> Bundle {
        Bundle {
            root: std::path::PathBuf::from("knowledge"),
            version: Some("0.2".into()),
            concepts,
            indexes: Vec::new(),
            logs: Vec::new(),
            graph: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn concept_meta_uses_badges_and_preserves_unknown_fields() {
        let metadata: BTreeMap<String, Value> = serde_json::from_value(json!({
            "type": "Implementation Plan",
            "status": "draft",
            "authority": "exploratory",
            "description": "Choose CLI entry points.",
            "custom_note": "preserved"
        }))
        .unwrap();
        let concept = concept_with(metadata);
        let html = render_concept_meta(&concept, &bundle_with(vec![concept.clone()]));
        assert!(html.contains("okf-badge okf-type"));
        assert!(html.contains("Implementation Plan"));
        assert!(html.contains("okf-lead"));
        assert!(html.contains("Choose CLI entry points."));
        assert!(html.contains("okf-provenance"));
        assert!(html.contains("1 other field"));
        assert!(html.contains("custom_note"));
        assert!(html.contains("preserved"));
        assert!(!html.contains("okf-meta-grid"));
        assert!(html.contains("okf-table-container"));
    }

    #[test]
    fn validation_notice_lists_errors_and_keeps_empty_when_clean() {
        let mut overview = concept_with(BTreeMap::new());
        overview.path = "overview.md".into();
        let mut bundle = bundle_with(vec![overview.clone()]);
        assert!(render_validation_notice(&bundle, Some("overview.md")).is_empty());
        bundle.diagnostics.push(Diagnostic::error(
            "OKF2001",
            "overview.md",
            None,
            "missing required field `tags`",
        ));
        let html = render_validation_notice(&bundle, Some("overview.md"));
        assert!(html.contains("okf-error-banner"));
        assert!(html.contains("OKF2001"));
        assert!(html.contains("missing required field `tags`"));
        assert!(html.contains("/review/#diagnostics"));
    }

    #[test]
    fn cited_sources_link_to_bundle_concepts_and_external_urls() {
        let sibling = Concept {
            id: "plans/other".into(),
            path: "plans/other.md".into(),
            ..concept_with(BTreeMap::new())
        };
        let metadata: BTreeMap<String, Value> = serde_json::from_value(json!({
            "type": "Implementation Plan",
            "status": "draft",
            "authority": "exploratory",
            "sources": [
                {
                    "id": "other",
                    "resource": "other.md",
                    "author": "human:nils"
                },
                {
                    "id": "readme",
                    "resource": "../../README.md",
                    "author": "human:nils"
                },
                {
                    "id": "docs",
                    "resource": "https://example.com/docs",
                    "author": "organization:example"
                }
            ]
        }))
        .unwrap();
        let concept = concept_with(metadata);
        let html = render_concept_meta(&concept, &bundle_with(vec![concept.clone(), sibling]));
        assert!(html.contains("<a href=\"/plans/other/\"><code>other.md</code></a>"));
        assert!(html.contains(
            "<a href=\"https://example.com/docs\" rel=\"noopener noreferrer\"><code>https://example.com/docs</code></a>"
        ));
        assert!(html.contains("<code>../../README.md</code>"));
        assert!(!html.contains("href=\"../../README.md\""));
        assert!(html.contains("okf-table-container"));
    }

    fn class_inner<'a>(html: &'a str, class: &str, close: &str) -> &'a str {
        let marker = format!("class=\"{class}\"");
        let start = html.find(&marker).unwrap_or_else(|| panic!("{class}"));
        let open_end = start + html[start..].find('>').expect("open tag") + 1;
        let close_at = open_end
            + html[open_end..]
                .find(close)
                .unwrap_or_else(|| panic!("{close}"));
        &html[open_end..close_at]
    }

    fn assert_global_nav_outside_toc(html: &str) {
        let nav = class_inner(html, "okf-global-nav", "</nav>");
        assert!(nav.contains("href=\"/\""));
        assert!(nav.contains("href=\"/review/\""));
        assert!(nav.contains("Home"));
        assert!(nav.contains("Governance"));
        if html.contains("class=\"rd-toc\"") {
            let toc = class_inner(html, "rd-toc", "</nav>");
            assert!(!toc.contains("href=\"/\""));
            assert!(!toc.contains("/review/"));
            assert!(!toc.contains("Home"));
            assert!(!toc.contains("Governance"));
        }
    }

    #[test]
    fn html_page_declares_dark_color_scheme() {
        let html = html_page("Knowledge", "<p>body</p>");
        assert!(html.contains("name=\"color-scheme\""));
        assert!(html.contains("content=\"dark\""));
        assert!(html.contains("class=\"rd-document\""));
        assert!(html.contains("/__rocci_okf/goto.js"));
        assert!(html.contains("class=\"rd-shell\""));
        assert!(!html.contains("class=\"rd-toc\""));
        assert!(!html.contains("okf-outline-menu"));
        assert_global_nav_outside_toc(&html);
        assert!(DEFAULT_CSS.contains(".okf-table-container { overflow-x: auto"));
        assert!(DEFAULT_CSS.contains(
            "min-height: calc(100vh - var(--rocci-chrome-top, 0px) - var(--rocci-chrome-bottom, 0px))"
        ));
        assert!(DEFAULT_CSS.contains(
            "max-height: calc(100vh - var(--rocci-chrome-top, 0px) - var(--rocci-chrome-bottom, 0px))"
        ));
        assert!(!DEFAULT_CSS.contains("max-height: calc(100vh - var(--rocci-chrome-top, 0px));"));
        assert!(DEFAULT_CSS.contains(".okf-filter-bar { display: flex; flex-wrap: wrap"));
        assert!(DEFAULT_CSS.contains(".rd-toc { display: none; }"));
        assert!(DEFAULT_CSS.contains(".okf-outline-menu { display: block"));
    }

    #[test]
    fn html_page_emits_left_toc_for_h2_and_h3() {
        let html = html_page(
            "Plan",
            "<h1>Title</h1><h2>Alpha</h2><p>x</p><h3>Beta</h3><h4>Gamma</h4>",
        );
        assert!(html.contains("class=\"rd-shell\""));
        assert!(html.contains("class=\"rd-toc\""));
        assert!(html.contains("class=\"okf-outline-menu\""));
        assert!(html.contains("On this page"));
        assert!(html.contains("href=\"#alpha\""));
        assert!(html.contains("href=\"#beta\""));
        assert!(html.contains("rd-toc-level-3"));
        assert!(html.contains("id=\"alpha\""));
        assert!(html.contains("id=\"beta\""));
        assert!(!html.contains("href=\"#title\""));
        assert!(!html.contains("href=\"#gamma\""));
        assert!(html.contains("__rdTocScroll"));
        assert!(html.contains("rocci-preview-nav"));
        assert!(html.contains("removeAttribute"));
        assert_global_nav_outside_toc(&html);
        let outline_menu = html
            .find("class=\"okf-outline-menu\"")
            .expect("outline menu");
        let main = html.find("<main>").expect("main");
        assert!(main < outline_menu);
    }

    #[test]
    fn html_page_select_root_excludes_toc_and_meta() {
        let html = html_page(
            "Plan",
            &with_meta_and_article(
                "<section class=\"okf-concept-meta\">governance</section>",
                "<h2>Alpha</h2><p>body</p>",
            ),
        );
        let toc = html.find("class=\"rd-toc\"").expect("toc");
        let main = html.find("<main>").expect("main");
        let meta = html.find("okf-concept-meta").expect("meta");
        let article = html.find("class=\"rd-article\"").expect("article");
        assert!(toc < main);
        assert!(main < meta);
        assert!(meta < article);
        let article_html = &html[article..];
        assert!(article_html.contains("body"));
        assert!(!article_html.contains("okf-concept-meta"));
        assert!(!article_html.contains("class=\"rd-toc\""));
        assert!(DEFAULT_CSS.contains("user-select: none"));
    }

    #[test]
    fn html_page_preserves_existing_heading_ids() {
        let html = html_page(
            "Review",
            "<h2 class=\"rd-header-2\" id=\"all-concepts-queue\">All Bundle Concepts</h2>",
        );
        assert!(html.contains("href=\"#all-concepts-queue\""));
        assert!(html.contains(">All Bundle Concepts</a>"));
        assert_eq!(html.matches("id=\"all-concepts-queue\"").count(), 1);
    }

    #[test]
    fn review_site_writes_collection_indexes() {
        let site =
            std::env::temp_dir().join(format!("rocci-okf-collection-{}", std::process::id()));
        let _ = fs::remove_dir_all(&site);
        fs::create_dir_all(&site).unwrap();

        let mut overview = concept_with(BTreeMap::new());
        overview.id = "architecture/overview".into();
        overview.path = "architecture/overview.md".into();
        overview.article_html = "<p>See <a href=\"/decisions/choice/\">choice</a>.</p>".into();

        let mut bundle = bundle_with(vec![overview]);
        bundle.indexes = vec![
            okf::Index {
                path: "index.md".into(),
                version: Some("0.2".into()),
                body_span: Span::new(0, 0),
                article_html: "<p><a href=\"/architecture/\">Architecture</a></p>".into(),
            },
            okf::Index {
                path: "architecture/index.md".into(),
                version: None,
                body_span: Span::new(0, 0),
                article_html: "<h1>Architecture</h1>".into(),
            },
        ];

        build_review_site(&bundle, &site).unwrap();
        assert!(site.join("architecture").join("index.html").is_file());
        assert!(site.join("catalog.json").is_file());
        assert!(site.join("pages.json").is_file());
        assert!(site.join("__rocci_okf").join("goto.js").is_file());
        let home = fs::read_to_string(site.join("index.html")).unwrap();
        assert!(home.contains("href=\"/architecture/\""));
        assert!(home.contains("/__rocci_okf/goto.js"));
        let collection = fs::read_to_string(site.join("architecture").join("index.html")).unwrap();
        assert!(collection.contains("Architecture"));
        assert!(collection.contains("id=\"architecture\""));
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn review_site_wasm_host() {
        if !is_roc_available() {
            return;
        }
        let site = unique_temp("site-wasm").unwrap();
        let mut overview = concept_with(BTreeMap::new());
        overview.id = "overview".into();
        overview.path = "overview.md".into();
        overview.article_html = "<h1>System Overview</h1>\n<p>Architecture description.</p>".into();

        let bundle = bundle_with(vec![overview]);
        build_review_site_with_host(&bundle, &site, Some(rocci_roc_host::HostChoice::Wasm))
            .unwrap();
        assert!(site.join("overview").join("index.html").is_file());
        let html = fs::read_to_string(site.join("overview").join("index.html")).unwrap();
        assert!(html.contains("System Overview"));
        assert!(html.contains("okf-concept-meta"));
        assert!(html.contains("class=\"rd-article\""));
        assert!(html.contains("/__rocci_okf/goto.js"));
        let meta = html.find("okf-concept-meta").unwrap();
        let article = html.find("class=\"rd-article\"").unwrap();
        assert!(meta < article);
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn okf_templates_compile_cleanly() {
        let modules =
            compile_okf_templates().expect("all OKF .rocci templates should compile cleanly");
        assert_eq!(modules.len(), 4);
        assert!(modules.iter().any(|m| m.type_name == "PageOutline"));
        assert!(modules.iter().any(|m| m.type_name == "ConceptMeta"));
        assert!(modules.iter().any(|m| m.type_name == "ReviewQueue"));
        assert!(modules.iter().any(|m| m.type_name == "OkfTheme"));
        let theme = modules.iter().find(|m| m.type_name == "OkfTheme").unwrap();
        assert!(theme.src.contains("class=\"rd-article\""));
        assert!(theme.roc.contains("rd-article"));
        assert!(theme.src.contains("class=\"okf-global-nav\""));
        assert!(theme.src.contains("class=\"okf-outline-menu\""));
        let global = theme.src.find("okf-global-nav").expect("global nav");
        let outline_gate = theme.src.find("@if has_outline").expect("outline gate");
        assert!(
            global < outline_gate,
            "Home/Review must render even when has_outline is false"
        );
        let toc = theme.src.find("class=\"rd-toc\"").expect("toc");
        let toc_home = theme.src[toc..].find("href=\"/\"");
        let next_details = theme.src[toc..].find("okf-outline-menu");
        if let (Some(home_rel), Some(details_rel)) = (toc_home, next_details) {
            assert!(home_rel > details_rel, "Home must not live inside .rd-toc");
        }
        let concept_meta = modules
            .iter()
            .find(|m| m.type_name == "ConceptMeta")
            .unwrap();
        assert!(concept_meta.src.contains("okf-table-container"));
        let review_queue = modules
            .iter()
            .find(|m| m.type_name == "ReviewQueue")
            .unwrap();
        assert!(review_queue.src.contains("okf-table-container"));
        assert!(review_queue.src.contains("okf-filter-bar"));
    }

    fn two_concept_bundle(second_body: &str) -> Bundle {
        let mut first = concept_with(BTreeMap::new());
        first.id = "plans/alpha".into();
        first.path = "plans/alpha.md".into();
        first.article_html = "<p>first body</p>".into();
        let mut second = concept_with(BTreeMap::new());
        second.id = "plans/beta".into();
        second.path = "plans/beta.md".into();
        second.article_html = second_body.into();
        bundle_with(vec![first, second])
    }

    #[test]
    fn renderer_hash_ignores_markdown_body() {
        let modules = compile_okf_templates().unwrap();
        let main_code = main_roc(false);
        let hash = renderer_compile_hash(&modules, &main_code, false);
        let gen_a =
            generate_okf_page_data(&two_concept_bundle("<p>UNIQUE_BODY_SENTENCE_aaa</p>")).unwrap();
        let gen_b =
            generate_okf_page_data(&two_concept_bundle("<p>UNIQUE_BODY_SENTENCE_bbb</p>")).unwrap();
        assert_eq!(gen_a.json, gen_b.json);
        assert!(gen_a.json.contains("plans/beta"));
        assert!(!gen_a.json.contains("UNIQUE_BODY_SENTENCE_aaa"));
        assert!(
            gen_a
                .articles
                .iter()
                .any(|(_, html)| html.contains("UNIQUE_BODY_SENTENCE_aaa"))
        );
        assert!(
            gen_b
                .articles
                .iter()
                .any(|(_, html)| html.contains("UNIQUE_BODY_SENTENCE_bbb"))
        );
        assert_eq!(hash, renderer_compile_hash(&modules, &main_code, false));
        assert!(!crate::runtime::OKF_BUILD_ROC.contains("UNIQUE_BODY_SENTENCE_aaa"));
        assert!(!main_code.contains("UNIQUE_BODY_SENTENCE_aaa"));
        assert!(!main_roc(false).contains("article_html:"));
    }

    #[test]
    fn renderer_hash_changes_when_template_roc_changes() {
        let modules = compile_okf_templates().unwrap();
        let main_code = main_roc(false);
        let hash = renderer_compile_hash(&modules, &main_code, false);
        let mut changed = modules.clone();
        changed[0].roc.push_str("\n# template-change\n");
        let hash2 = renderer_compile_hash(&changed, &main_code, false);
        assert_ne!(hash, hash2);
    }

    #[test]
    fn lookup_renderer_hits_when_only_markdown_changes() {
        let modules = compile_okf_templates().unwrap();
        let main_code = main_roc(false);
        let hash = renderer_compile_hash(&modules, &main_code, false);
        let dir = unique_temp("cache").unwrap();
        let cache = rocci_roc_host::TwoTierCache::new(dir.clone());
        let target = format!("native:{}", std::env::consts::ARCH);
        let fp = okf_fingerprints(&modules, &main_code);
        cache
            .store_renderer(&hash, &target, b"dummy-apply", &fp)
            .unwrap();
        assert!(cache.lookup_renderer(&hash, &target).is_some());
        let _ = generate_okf_page_data(&two_concept_bundle("<p>changed body</p>")).unwrap();
        assert_eq!(hash, renderer_compile_hash(&modules, &main_code, false));
        assert!(cache.lookup_renderer(&hash, &target).is_some());
        let mut changed = modules.clone();
        changed[0].roc.push('\n');
        let hash2 = renderer_compile_hash(&changed, &main_code, false);
        assert!(cache.lookup_renderer(&hash2, &target).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn native_host_writes_rocci_knowledge_shell() {
        if !is_roc_available() {
            return;
        }
        let site = unique_temp("site-native-shell").unwrap();
        let mut overview = concept_with(BTreeMap::new());
        overview.id = "overview".into();
        overview.path = "overview.md".into();
        overview.article_html =
            "<h1>System Overview</h1>\n<h2>Architecture</h2>\n<p>Body.</p>".into();
        let bundle = bundle_with(vec![overview]);
        let snapshot =
            build_review_site_with_host(&bundle, &site, Some(rocci_roc_host::HostChoice::Native))
                .unwrap();
        assert!(
            snapshot.spans.iter().any(|span| span.name == "render"),
            "apply still runs"
        );
        let html = fs::read_to_string(site.join("overview").join("index.html")).unwrap();
        assert!(html.contains("class=\"outline-label\""));
        assert!(html.contains("On this page"));
        assert!(html.contains("href=\"/\""));
        assert!(html.contains("Home"));
        assert!(html.contains("href=\"/review/\""));
        assert!(html.contains("Governance"));
        assert!(html.contains("<aside class=\"rd-toc\""));
        assert!(html.contains("System Overview"));
        assert!(!html.contains("<nav class=\"rd-toc\""));
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn rust_only_path_still_builds_without_host() {
        let site = unique_temp("site-rust-only").unwrap();
        let mut overview = concept_with(BTreeMap::new());
        overview.id = "overview".into();
        overview.path = "overview.md".into();
        overview.article_html =
            "<h1>System Overview</h1>\n<h2>Architecture</h2>\n<p>Body.</p>".into();
        let bundle = bundle_with(vec![overview]);
        build_review_site_pure_rust(&bundle, &site).unwrap();
        let html = fs::read_to_string(site.join("overview").join("index.html")).unwrap();
        assert!(html.contains("System Overview"));
        assert!(html.contains("href=\"/\""));
        assert!(html.contains("Home"));
        assert!(html.contains("<nav class=\"rd-toc\""));
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn preview_renders_document_when_bundle_has_errors() {
        let site = unique_temp("site-with-errors").unwrap();
        let mut overview = concept_with(BTreeMap::new());
        overview.id = "overview".into();
        overview.path = "overview.md".into();
        overview.article_html = "<h1>System Overview</h1>\n<p>Recovered body.</p>".into();
        let mut bundle = bundle_with(vec![overview]);
        bundle.diagnostics.push(Diagnostic::error(
            "OKF2001",
            "overview.md",
            None,
            "missing required field `tags`",
        ));
        build_review_site_pure_rust(&bundle, &site).unwrap();
        let html = fs::read_to_string(site.join("overview").join("index.html")).unwrap();
        assert!(html.contains("Recovered body"), "{html}");
        assert!(html.contains("okf-error-banner"), "{html}");
        assert!(html.contains("OKF2001"), "{html}");
        assert!(html.contains("missing required field `tags`"), "{html}");
        assert!(
            !html.contains("knowledge bundle has validation errors"),
            "{html}"
        );
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn preview_writes_stub_page_for_unparsed_error_path() {
        let site = unique_temp("site-stub").unwrap();
        let bundle = Bundle {
            diagnostics: vec![Diagnostic::error(
                "OKF1001",
                "plans/broken.md",
                None,
                "document is not valid UTF-8",
            )],
            ..bundle_with(Vec::new())
        };
        build_review_site_pure_rust(&bundle, &site).unwrap();
        let html =
            fs::read_to_string(site.join("plans").join("broken").join("index.html")).unwrap();
        assert!(html.contains("okf-error-banner"), "{html}");
        assert!(html.contains("OKF1001"), "{html}");
        assert!(html.contains("document is not valid UTF-8"), "{html}");
        let _ = fs::remove_dir_all(&site);
    }

    #[test]
    fn watch_session_reuses_apply_without_roc_build() {
        if !is_roc_available() {
            return;
        }
        let site1 = unique_temp("site-session-1").unwrap();
        let site2 = unique_temp("site-session-2").unwrap();
        let mut overview = concept_with(BTreeMap::new());
        overview.id = "overview".into();
        overview.path = "overview.md".into();
        overview.article_html = "<h1>System Overview</h1>\n<p>Body.</p>".into();
        let bundle = bundle_with(vec![overview]);
        let host = Some(rocci_roc_host::HostChoice::Native);
        let (first, session) = build_review_site_with_session(&bundle, &site1, host, None).unwrap();
        let session = session.expect("apply session after first build");
        assert!(
            first
                .spans
                .iter()
                .any(|span| span.name == "compile" && span.note.as_deref() == Some("cached"))
                || first.spans.iter().any(|span| span.name == "compile")
        );
        let (second, next) =
            build_review_site_with_session(&bundle, &site2, host, Some(&session)).unwrap();
        let next = next.expect("apply session after second build");
        assert_eq!(session.compile_hash, next.compile_hash);
        assert_eq!(session.apply_path, next.apply_path);
        let compile = second
            .spans
            .iter()
            .find(|span| span.name == "compile")
            .expect("compile span");
        assert_eq!(compile.duration_ms, 0);
        assert_eq!(compile.note.as_deref(), Some("cached"));
        let render = second
            .spans
            .iter()
            .find(|span| span.name == "render")
            .expect("render span");
        assert_eq!(render.note.as_deref(), Some("reuse"));
        let html = fs::read_to_string(site2.join("overview").join("index.html")).unwrap();
        assert!(html.contains("System Overview"));
        let _ = fs::remove_dir_all(&site1);
        let _ = fs::remove_dir_all(&site2);
    }
}
