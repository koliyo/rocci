//! Okmate library: CLI and later HTTP/desktop surfaces over the portable `okf` engine.

pub mod cli;
pub mod http;
pub mod preview;
pub mod site;
pub mod views;

use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use okf::{CheckReport, KnowledgeFilter, Profile, RetrievalReport};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CheckFormat {
    Terminal,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ProfileArg {
    Base,
    Rocci,
}

impl From<ProfileArg> for Profile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Base => Self::Base,
            ProfileArg::Rocci => Self::Rocci,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum TrustTierArg {
    HumanReviewed,
    Generated,
    Unverified,
}

impl From<TrustTierArg> for okf::TrustTier {
    fn from(value: TrustTierArg) -> Self {
        match value {
            TrustTierArg::HumanReviewed => Self::HumanReviewed,
            TrustTierArg::Generated => Self::Generated,
            TrustTierArg::Unverified => Self::Unverified,
        }
    }
}

pub fn check(root: &Path, profile: Profile) -> Result<CheckReport> {
    okf::check(root, profile)
}

pub fn print_check(report: &CheckReport, format: CheckFormat) -> Result<()> {
    match format {
        CheckFormat::Terminal => {
            let formatted = report.terminal();
            if !formatted.is_empty() {
                println!("{formatted}");
            }
        }
        CheckFormat::Json => {
            println!("{}", report.json()?);
        }
    }
    Ok(())
}

pub fn inspect(
    root: &Path,
    kind: okf::InspectKind,
    concept_id: Option<&str>,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    okf::inspect_filtered(root, kind, concept_id, profile, filter)
}

pub fn search(
    root: &Path,
    query: &str,
    profile: Profile,
    filter: &KnowledgeFilter,
) -> Result<String> {
    okf::search(root, query, profile, filter)
}

pub fn benchmark(root: &Path, benchmark_path: &Path, profile: Profile) -> Result<RetrievalReport> {
    okf::benchmark_retrieval(root, benchmark_path, profile)
}
