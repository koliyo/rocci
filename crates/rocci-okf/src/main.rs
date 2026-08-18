use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use okf::InspectKind;
use rocci_cli::serve::{PortArg, parse_port_arg};

mod dev;
mod presentation;
mod runtime;

#[derive(Parser)]
#[command(
    name = "rocci-okf",
    about = "Rocci Open Knowledge Format (OKF) review and query application"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Watch an OKF bundle or a concept file inside one, rebuild, and serve with live reload.
    Run {
        /// Knowledge bundle directory or a Markdown file inside one.
        #[arg(default_value = "knowledge")]
        path: PathBuf,
        /// Write preview output here instead of a temp directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        /// Execution host runtime for evaluating templates (native [default], auto; wasm is planned).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
        /// Skip the embedded window; print the URL and keep serving.
        #[arg(long)]
        no_window: bool,
        /// TCP port to listen on. Defaults to a free port with the embedded window,
        /// or 8000 with `--no-window`. Pass `auto` to pick a free port.
        #[arg(
            long,
            default_value = "auto",
            default_value_if("no_window", "true", "8000"),
            value_name = "PORT",
            value_parser = parse_port_arg,
            env = "ROC_BASIC_WEBSERVER_PORT"
        )]
        port: PortArg,
    },
    /// Validate an OKF bundle without writing output.
    Check {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Print normalized concepts or the bundle graph as JSON.
    Inspect {
        #[command(subcommand)]
        target: KnowledgeInspectTarget,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
    /// Search metadata and heading chunks as JSON.
    Search {
        query: String,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        #[command(flatten)]
        filters: KnowledgeFiltersArg,
    },
    /// Run a retrieval benchmark TOML file against a knowledge bundle.
    Benchmark {
        benchmark: PathBuf,
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
    /// Emit derived bundle artifacts and the minimal static review site.
    Build {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(short, long, default_value = "dist/knowledge")]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
        /// Execution host runtime for evaluating templates (native [default], auto; wasm is planned).
        #[arg(long, value_enum, default_value_t = HostArg::Auto)]
        host: HostArg,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum, Default)]
enum HostArg {
    /// Pick host automatically (resolves to native).
    #[default]
    Auto,
    /// Compile and run native host executable (requires roc on PATH).
    Native,
    /// In-process Wasmtime host (planned for Phase 5; requires custom Roc wasm platform).
    Wasm,
}

impl From<HostArg> for rocci_roc_host::HostChoice {
    fn from(arg: HostArg) -> Self {
        match arg {
            HostArg::Auto => rocci_roc_host::HostChoice::Auto,
            HostArg::Native => rocci_roc_host::HostChoice::Native,
            HostArg::Wasm => rocci_roc_host::HostChoice::Wasm,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum KnowledgeProfileArg {
    Base,
    Rocci,
}

impl From<KnowledgeProfileArg> for okf::Profile {
    fn from(value: KnowledgeProfileArg) -> Self {
        match value {
            KnowledgeProfileArg::Base => Self::Base,
            KnowledgeProfileArg::Rocci => Self::Rocci,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum TrustTierArg {
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

#[derive(Args, Default)]
struct KnowledgeFiltersArg {
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

impl From<&KnowledgeFiltersArg> for okf::KnowledgeFilter {
    fn from(value: &KnowledgeFiltersArg) -> Self {
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
enum KnowledgeInspectTarget {
    Catalog {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[command(flatten)]
        filters: KnowledgeFiltersArg,
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check {
            root,
            profile,
            format,
        } => {
            let report = okf::check(&root, profile.into())?;
            match format {
                CheckFormatArg::Terminal => {
                    let formatted = report.terminal();
                    if !formatted.is_empty() {
                        println!("{formatted}");
                    }
                }
                CheckFormatArg::Json => {
                    println!("{}", report.json()?);
                }
            }
            if report.has_errors() {
                bail!("knowledge check failed with errors");
            }
            Ok(())
        }
        Commands::Inspect { target, profile } => {
            let json = match target {
                KnowledgeInspectTarget::Catalog { root, filters } => okf::inspect_filtered(
                    &root,
                    InspectKind::Catalog,
                    None,
                    profile.into(),
                    &(&filters).into(),
                )?,
                KnowledgeInspectTarget::Concept { concept, root } => {
                    okf::inspect(&root, InspectKind::Concept, Some(&concept), profile.into())?
                }
                KnowledgeInspectTarget::Graph { root } => {
                    okf::inspect(&root, InspectKind::Graph, None, profile.into())?
                }
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
            let json = okf::search(&root, &query, profile.into(), &(&filters).into())?;
            println!("{json}");
            Ok(())
        }
        Commands::Benchmark {
            benchmark,
            root,
            profile,
        } => {
            let report = okf::benchmark_retrieval(&root, &benchmark, profile.into())?;
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
            host,
        } => {
            let summary = okf::build(&root, &output, profile.into())?;
            let _ = host;
            eprintln!(
                "rocci-okf: built {} concepts and {} indexes into {}",
                summary.concepts, summary.indexes, summary.output
            );
            Ok(())
        }
        Commands::Run {
            path,
            output,
            profile,
            host,
            no_window,
            port,
        } => {
            let target = okf::resolve_preview_path(&path)?;
            let port = port.resolve()?;
            let server = dev::run_knowledge(
                &target.root,
                output.as_deref(),
                port,
                profile.into(),
                &target.open_path,
                Some(host.into()),
            )?;
            eprintln!("rocci-okf: serving {} at {}", server.title, server.url);
            if no_window {
                server.wait();
                return Ok(());
            }
            let result = rocci_desktop::preview(rocci_desktop::PreviewOptions {
                url: server.url.clone(),
                title: format!("{} — Rocci Knowledge", server.title),
                state_key: Some("rocci:knowledge".to_string()),
                width: 1200.0,
                height: 800.0,
                ..rocci_desktop::PreviewOptions::default()
            })
            .map_err(|error| anyhow::anyhow!("{error}"));
            drop(server);
            result
        }
    }
}
