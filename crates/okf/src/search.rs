use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::ast::{Bundle, Concept, KnowledgeFilter, TrustTier};
use crate::validate::{
    is_date, latest_human_verification, metadata_string_array, parse_timestamp, string_field,
};

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

impl KnowledgeFilter {
    pub fn matches(&self, concept: &Concept) -> bool {
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

pub fn search(bundle: &Bundle, query: &str, filter: &KnowledgeFilter) -> Vec<SearchChunk> {
    matching_search_chunks(bundle, query, filter)
}

pub fn search_index(bundle: &Bundle) -> Vec<SearchChunk> {
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

        for section in &concept.heading_sections {
            let section_text = section.body_texts.join(" ");
            chunks.push(SearchChunk {
                id: format!("{}#{}", concept.id, section.id),
                concept_id: concept.id.clone(),
                path: concept.path.clone(),
                kind: "heading".into(),
                heading: Some(section.heading_text.clone()),
                title: title.clone(),
                description: description.clone(),
                concept_type: concept_type.clone(),
                tags: tags.clone(),
                status: status.clone(),
                authority: authority.clone(),
                trust_tier,
                stale,
                url: format!("/{}/#{}", concept.id, section.id),
                text: normalize_search_text(&format!("{} {}", section.heading_text, section_text)),
            });
        }
    }
    chunks
}

pub fn matching_search_chunks(
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

pub fn normalize_search_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn concept_trust_tier(metadata: &BTreeMap<String, Value>) -> TrustTier {
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

pub fn concept_is_stale(metadata: &BTreeMap<String, Value>) -> bool {
    let Some(today) = crate::validate::current_utc_date() else {
        return false;
    };
    string_field(metadata, "stale_after").is_some_and(|date| is_date(date) && date < today.as_str())
}
