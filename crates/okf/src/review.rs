use serde_json::Value;

use crate::ast::Concept;
use crate::diagnostic::{Diagnostic, Severity};
use crate::search::concept_is_stale;
use crate::validate::string_field;

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
        let detail = concept_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join(" · ");
        return ConceptAction {
            kind: ActionKind::FixErrors,
            label: "Fix Errors".into(),
            detail,
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
                if let Some(start) = d.message.find("source `")
                    && let Some(end) = d.message[start + 8..].find('`')
                {
                    return d.message[start + 8..start + 8 + end].to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Concept, Span};
    use crate::diagnostic::{Diagnostic, SourceLocation};
    use serde_json::json;
    use std::collections::BTreeSet;

    fn concept(id: &str, path: &str) -> Concept {
        Concept {
            id: id.into(),
            path: path.into(),
            metadata: serde_json::from_value(json!({
                "type": "Architecture",
                "title": id,
                "status": "draft",
                "authority": "descriptive"
            }))
            .unwrap(),
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

    #[test]
    fn fix_errors_action_includes_diagnostic_messages() {
        let record = concept("plans/example", "plans/example.md");
        let diagnostics = vec![Diagnostic::error(
            "OKF2001",
            "plans/example.md",
            None,
            "missing required field `tags`",
        )];
        let action = classify_concept_action(&record, &diagnostics);
        assert_eq!(action.kind, ActionKind::FixErrors);
        assert!(action.detail.contains("OKF2001"));
        assert!(action.detail.contains("missing required field `tags`"));
    }
}
