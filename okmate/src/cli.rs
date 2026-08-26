use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand};
use okf::{InspectKind, KnowledgeFilter};

use crate::{
    CheckFormat, ProfileArg, TrustTierArg, benchmark, check, inspect, print_check, search,
};

#[derive(Parser)]
#[command(
    name = "okmate",
    about = "Okmate (open knowledge mate) — Askama + Axum knowledge application over the portable okf engine",
    arg_required_else_help = true,
    subcommand_required = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an OKF bundle without writing output.
    Check {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Rocci)]
        profile: ProfileArg,
        #[arg(long, value_enum, default_value_t = CheckFormat::Terminal)]
        format: CheckFormat,
    },
    /// Print normalized concepts or the bundle graph as JSON.
    Inspect {
        #[command(subcommand)]
        target: InspectTarget,
        #[arg(long, value_enum, default_value_t = ProfileArg::Rocci)]
        profile: ProfileArg,
    },
    /// Search metadata and heading chunks as JSON.
    Search {
        query: String,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Rocci)]
        profile: ProfileArg,
        #[command(flatten)]
        filters: FiltersArg,
    },
    /// Run a retrieval benchmark TOML file against a knowledge bundle.
    Benchmark {
        benchmark: PathBuf,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Rocci)]
        profile: ProfileArg,
    },
    /// Emit engine artifacts and the Askama HTML review tree.
    Build {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(short, long, default_value = "dist/knowledge")]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = ProfileArg::Rocci)]
        profile: ProfileArg,
    },
}

#[derive(Args, Default)]
struct FiltersArg {
    /// Match any of these concept types. Repeat to add alternatives.
    #[arg(long = "type")]
    types: Vec<String>,
    /// Require this tag. Repeat to require multiple tags.
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// Match any of these lifecycle statuses. Repeat to add alternatives.
    #[arg(long = "status")]
    statuses: Vec<String>,
    /// Match any of these authority levels. Repeat to add alternatives.
    #[arg(long = "authority")]
    authorities: Vec<String>,
    /// Match any of these derived trust tiers. Repeat to add alternatives.
    #[arg(long = "trust-tier", value_enum)]
    trust_tiers: Vec<TrustTierArg>,
    /// Match stale (`true`) or current (`false`) records.
    #[arg(long)]
    stale: Option<bool>,
}

impl From<&FiltersArg> for KnowledgeFilter {
    fn from(value: &FiltersArg) -> Self {
        Self {
            types: value.types.clone(),
            tags: value.tags.clone(),
            statuses: value.statuses.clone(),
            authorities: value.authorities.clone(),
            trust_tiers: value.trust_tiers.iter().copied().map(Into::into).collect(),
            stale: value.stale,
        }
    }
}

#[derive(Subcommand)]
enum InspectTarget {
    Catalog {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[command(flatten)]
        filters: FiltersArg,
    },
    Concept {
        concept: String,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
    },
    Graph {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check {
            root,
            profile,
            format,
        } => {
            let report = check(&root, profile.into())?;
            print_check(&report, format)?;
            if report.has_errors() {
                bail!("knowledge check failed with errors");
            }
            Ok(())
        }
        Commands::Inspect { target, profile } => {
            let json = match target {
                InspectTarget::Catalog { root, filters } => inspect(
                    &root,
                    InspectKind::Catalog,
                    None,
                    profile.into(),
                    &(&filters).into(),
                )?,
                InspectTarget::Concept { concept, root } => inspect(
                    &root,
                    InspectKind::Concept,
                    Some(&concept),
                    profile.into(),
                    &KnowledgeFilter::default(),
                )?,
                InspectTarget::Graph { root } => inspect(
                    &root,
                    InspectKind::Graph,
                    None,
                    profile.into(),
                    &KnowledgeFilter::default(),
                )?,
            };
            println!("{json}");
            Ok(())
        }
        Commands::Search {
            query,
            root,
            profile,
            filters,
        } => {
            let json = search(&root, &query, profile.into(), &(&filters).into())?;
            println!("{json}");
            Ok(())
        }
        Commands::Benchmark {
            benchmark: path,
            root,
            profile,
        } => {
            let report = benchmark(&root, &path, profile.into())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.threshold_met {
                bail!(
                    "retrieval benchmark failed: hit rate {:.2}% was below required minimum {:.2}%",
                    report.hit_rate * 100.0,
                    report.minimum_hit_rate * 100.0
                );
            }
            Ok(())
        }
        Commands::Build {
            root,
            output,
            profile,
        } => {
            let summary = crate::site::build(&root, &output, profile.into())?;
            eprintln!(
                "okmate: built {} concepts and {} indexes into {}",
                summary.concepts, summary.indexes, summary.output
            );
            Ok(())
        }
    }
}
