use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::ast::{Bundle, KnowledgeFilter};
use crate::search::matching_search_chunks;
use crate::validate::string_field;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalBenchmark {
    pub version: u32,
    pub top_k: usize,
    pub minimum_hit_rate: f64,
    pub questions: Vec<RetrievalQuestion>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalQuestion {
    pub id: String,
    pub question: String,
    pub query: String,
    pub expected_concepts: Vec<String>,
    #[serde(default)]
    pub expected_status: Option<String>,
    #[serde(default)]
    pub expected_authority: Option<String>,
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

pub fn run_benchmark(bundle: &Bundle, benchmark_path: &Path) -> Result<RetrievalReport> {
    if bundle.has_errors() {
        bail!("knowledge bundle has validation errors");
    }
    let benchmark_source = fs::read_to_string(benchmark_path)
        .with_context(|| format!("failed to read {}", benchmark_path.display()))?;
    let benchmark: RetrievalBenchmark = toml::from_str(&benchmark_source)
        .with_context(|| format!("failed to parse {}", benchmark_path.display()))?;
    validate_retrieval_benchmark(&benchmark, bundle)?;

    let mut reciprocal_rank = 0.0;
    let mut passed = 0;
    let mut questions = Vec::with_capacity(benchmark.questions.len());
    for question in benchmark.questions {
        let mut returned_concepts = Vec::new();
        for chunk in matching_search_chunks(bundle, &question.query, &KnowledgeFilter::default()) {
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
