use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use okf::{
    Bundle, Concept, ConceptAction, Diagnostic, Severity, TrustTier, classify_concept_action,
};
pub use rocci_ui::escape;
use rocci_ui::{StatCardView, StatTone, render_stat_grid};
use serde_json::Value;

pub const PRIORITY_1_RECORDS: &[(&str, &str)] = &[
    (
        "architecture/rocdown-format",
        "Parser/README precedence over original report; root HTML template islands",
    ),
    (
        "architecture/rocs-documentation-compiler",
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

pub fn render_concept_meta(concept: &Concept, bundle_diagnostics: &[Diagnostic]) -> String {
    let status = okf::string_field(&concept.metadata, "status").unwrap_or("draft");
    let authority = okf::string_field(&concept.metadata, "authority").unwrap_or("descriptive");
    let trust_tier = okf::search::concept_trust_tier(&concept.metadata);
    let stale = okf::search::concept_is_stale(&concept.metadata);
    let stale_after = okf::string_field(&concept.metadata, "stale_after").unwrap_or("");
    let concept_type = okf::string_field(&concept.metadata, "type").unwrap_or("Concept");
    let owners = okf::metadata_string_array(&concept.metadata, "owners");
    let tags = okf::metadata_string_array(&concept.metadata, "tags");

    let action = classify_concept_action(concept, bundle_diagnostics);

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

    if let Some(sources) = concept.metadata.get("sources").and_then(Value::as_array)
        && !sources.is_empty()
    {
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

pub fn build_review_site(bundle: &Bundle, site: &Path) -> Result<()> {
    fs::create_dir_all(site).with_context(|| format!("failed to create {}", site.display()))?;

    for concept in &bundle.concepts {
        let destination = site.join(&concept.id).join("index.html");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let title = okf::string_field(&concept.metadata, "title").unwrap_or(&concept.id);
        let meta_header = render_concept_meta(concept, &bundle.diagnostics);
        let full_html = format!("{meta_header}{}", concept.article_html);
        fs::write(&destination, html_page(title, &full_html))
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }
    if let Some(index) = bundle.indexes.iter().find(|index| index.path == "index.md") {
        let governance_header = render_home_page_governance(bundle);
        let full_index = format!("{governance_header}{}", index.article_html);
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
            &render_review_page(bundle),
        ),
    )
    .context("failed to write knowledge review page")?;

    Ok(())
}

pub fn html_page(title: &str, article: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><link rel=\"stylesheet\" href=\"/__rocci_okf/app.css\"><script src=\"/__rocci_okf/reload.js\" defer></script></head><body><main class=\"rd-document\">{article}</main></body></html>\n",
        escape(title)
    )
}
