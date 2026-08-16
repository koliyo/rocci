use std::{
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use anyhow::{Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "rocs", about = "Static documentation generator built on Rocci")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile Rocdown pages into a static site.
    Build {
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Override `build.output` from rocs.toml.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Watch sources, rebuild, and serve with live reload.
    Run {
        #[arg(default_value = ".")]
        root: PathBuf,
        /// Write preview output here instead of a temp directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
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
    /// Validate the documentation catalog without writing output.
    Check {
        #[arg(default_value = ".")]
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = CheckFormatArg::Terminal)]
        format: CheckFormatArg,
    },
    /// Print resolved catalog data.
    Inspect {
        #[command(subcommand)]
        target: InspectTarget,
    },
    /// Validate, inspect, or build an Open Knowledge Format bundle.
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommand,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PortArg {
    Auto,
    Exact(u16),
}

impl PortArg {
    fn resolve(self) -> Result<u16> {
        match self {
            Self::Auto => free_port(),
            Self::Exact(port) => {
                if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                    bail!("port {port} is already in use; pass --port auto or choose another port");
                }
                Ok(port)
            }
        }
    }
}

fn parse_port_arg(value: &str) -> Result<PortArg, String> {
    if value.eq_ignore_ascii_case("auto") {
        return Ok(PortArg::Auto);
    }
    match value.parse::<u16>() {
        Ok(0) => Err("port 0 is invalid; pass --port auto to pick a free port".into()),
        Ok(port) => Ok(PortArg::Exact(port)),
        Err(_) => Err(format!(
            "invalid port `{value}`; expected a number 1-65535 or `auto`"
        )),
    }
}

fn free_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    Ok(listener.local_addr()?.port())
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum KnowledgeProfileArg {
    Base,
    Rocci,
}

impl From<KnowledgeProfileArg> for rocs::okf::Profile {
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

impl From<TrustTierArg> for rocs::okf::TrustTier {
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

impl From<&KnowledgeFiltersArg> for rocs::okf::KnowledgeFilter {
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
enum KnowledgeCommand {
    /// Watch an OKF bundle, rebuild, and serve with live reload.
    Run {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        /// Write preview output here instead of a temp directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
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
    /// Measure lexical retrieval against a fixed question set.
    Benchmark {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        /// Question set. Defaults to retrieval-benchmark.toml inside the bundle.
        #[arg(long, value_name = "PATH")]
        questions: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
    /// Render a validated bundle and emit its normalized catalog.
    Build {
        #[arg(default_value = "knowledge")]
        root: PathBuf,
        #[arg(short, long, default_value = "dist/knowledge")]
        output: PathBuf,
        #[arg(long, value_enum, default_value_t = KnowledgeProfileArg::Rocci)]
        profile: KnowledgeProfileArg,
    },
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

#[derive(Subcommand)]
enum InspectTarget {
    Config {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Catalog {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Page {
        page: String,
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Graph {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Nav {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
    Artifacts {
        #[arg(default_value = ".")]
        root: PathBuf,
    },
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    match Cli::parse().command {
        Commands::Build { root, output } => {
            rocs::build_configured(&root, output.as_deref())?;
            Ok(())
        }
        Commands::Run {
            root,
            output,
            no_window,
            port,
        } => run(&root, output.as_deref(), no_window, port),
        Commands::Check { root, format } => {
            let report = rocs::check(&root)?;
            let rendered = report.render(match format {
                CheckFormatArg::Terminal => rocs::CheckFormat::Terminal,
                CheckFormatArg::Json => rocs::CheckFormat::Json,
            })?;
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            if report.has_errors() {
                bail!("documentation catalog has errors");
            }
            Ok(())
        }
        Commands::Inspect { target } => {
            let (kind, root, page) = match &target {
                InspectTarget::Config { root } => (rocs::InspectKind::Config, root, None),
                InspectTarget::Catalog { root } => (rocs::InspectKind::Catalog, root, None),
                InspectTarget::Page { page, root } => {
                    (rocs::InspectKind::Page, root, Some(page.as_str()))
                }
                InspectTarget::Graph { root } => (rocs::InspectKind::Graph, root, None),
                InspectTarget::Nav { root } => (rocs::InspectKind::Nav, root, None),
                InspectTarget::Artifacts { root } => (rocs::InspectKind::Artifacts, root, None),
            };
            println!("{}", rocs::inspect(root, kind, page)?);
            Ok(())
        }
        Commands::Knowledge { command } => knowledge(command),
    }
}

fn knowledge(command: KnowledgeCommand) -> Result<()> {
    match command {
        KnowledgeCommand::Run {
            root,
            output,
            profile,
            no_window,
            port,
        } => {
            let port = port.resolve()?;
            let server = rocs::run_knowledge(&root, output.as_deref(), port, profile.into())?;
            preview(server, no_window)
        }
        KnowledgeCommand::Check {
            root,
            profile,
            format,
        } => {
            let report = rocs::okf::check(&root, profile.into())?;
            let rendered = match format {
                CheckFormatArg::Terminal => report.terminal(),
                CheckFormatArg::Json => report.json()?,
            };
            if !rendered.is_empty() {
                println!("{rendered}");
            }
            if report.has_errors() {
                bail!("knowledge bundle has errors");
            }
            Ok(())
        }
        KnowledgeCommand::Inspect { target, profile } => {
            let (kind, root, concept, filter) = match &target {
                KnowledgeInspectTarget::Catalog { root, filters } => (
                    rocs::okf::InspectKind::Catalog,
                    root,
                    None,
                    rocs::okf::KnowledgeFilter::from(filters),
                ),
                KnowledgeInspectTarget::Concept { concept, root } => (
                    rocs::okf::InspectKind::Concept,
                    root,
                    Some(concept.as_str()),
                    rocs::okf::KnowledgeFilter::default(),
                ),
                KnowledgeInspectTarget::Graph { root } => (
                    rocs::okf::InspectKind::Graph,
                    root,
                    None,
                    rocs::okf::KnowledgeFilter::default(),
                ),
            };
            println!(
                "{}",
                rocs::okf::inspect_filtered(root, kind, concept, profile.into(), &filter)?
            );
            Ok(())
        }
        KnowledgeCommand::Search {
            query,
            root,
            profile,
            filters,
        } => {
            println!(
                "{}",
                rocs::okf::search(&root, &query, profile.into(), &(&filters).into())?
            );
            Ok(())
        }
        KnowledgeCommand::Benchmark {
            root,
            questions,
            profile,
        } => {
            let questions = questions.unwrap_or_else(|| root.join("retrieval-benchmark.toml"));
            let report = rocs::okf::benchmark_retrieval(&root, &questions, profile.into())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.threshold_met {
                bail!(
                    "knowledge retrieval hit rate {:.3} is below the required {:.3}",
                    report.hit_rate,
                    report.minimum_hit_rate
                );
            }
            Ok(())
        }
        KnowledgeCommand::Build {
            root,
            output,
            profile,
        } => {
            let summary = rocs::okf::build(&root, &output, profile.into())?;
            println!("{}", serde_json::to_string_pretty(&summary)?);
            Ok(())
        }
    }
}

fn run(
    root: &std::path::Path,
    output: Option<&std::path::Path>,
    no_window: bool,
    port: PortArg,
) -> Result<()> {
    let port = port.resolve()?;
    let server = rocs::run(root, output, port)?;
    preview(server, no_window)
}

fn preview(server: rocs::DevServer, no_window: bool) -> Result<()> {
    eprintln!("rocs: serving {} at {}", server.title, server.url);
    if no_window {
        server.wait();
        return Ok(());
    }
    let result = rocci_wry::preview(rocci_wry::PreviewOptions {
        url: server.url.clone(),
        title: server.title.clone(),
        ..rocci_wry::PreviewOptions::default()
    })
    .map_err(|error| anyhow::anyhow!("{error}"));
    drop(server);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn build_parses() {
        let cli = Cli::try_parse_from(["rocs", "build", "examples/rocs", "--output", "tmp-dist"])
            .unwrap();
        match cli.command {
            Commands::Build { root, output } => {
                assert_eq!(root, PathBuf::from("examples/rocs"));
                assert_eq!(output, Some(PathBuf::from("tmp-dist")));
            }
            _ => panic!("expected build"),
        }
    }

    #[test]
    fn run_parses_no_window_and_port() {
        let cli =
            Cli::try_parse_from(["rocs", "run", "docs", "--no-window", "--port", "8000"]).unwrap();
        match cli.command {
            Commands::Run {
                root,
                output,
                no_window,
                port,
            } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(output.is_none());
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn run_defaults_port_to_8000_without_window() {
        if std::env::var_os("ROC_BASIC_WEBSERVER_PORT").is_some() {
            return;
        }
        let cli = Cli::try_parse_from(["rocs", "run", "--no-window"]).unwrap();
        match cli.command {
            Commands::Run {
                no_window, port, ..
            } => {
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8000));
            }
            _ => panic!("expected run"),
        }
    }

    #[test]
    fn check_and_inspect_parse() {
        let cli = Cli::try_parse_from(["rocs", "check", "docs", "--format", "json"]).unwrap();
        match cli.command {
            Commands::Check { root, format } => {
                assert_eq!(root, PathBuf::from("docs"));
                assert!(matches!(format, CheckFormatArg::Json));
            }
            _ => panic!("expected check"),
        }
        let cli = Cli::try_parse_from(["rocs", "inspect", "page", "index", "docs"]).unwrap();
        match cli.command {
            Commands::Inspect { target } => match target {
                InspectTarget::Page { page, root } => {
                    assert_eq!(page, "index");
                    assert_eq!(root, PathBuf::from("docs"));
                }
                _ => panic!("expected page"),
            },
            _ => panic!("expected inspect"),
        }
    }

    #[test]
    fn knowledge_commands_parse() {
        let cli = Cli::try_parse_from([
            "rocs",
            "knowledge",
            "run",
            "knowledge",
            "--profile",
            "base",
            "--no-window",
            "--port",
            "8123",
        ])
        .unwrap();
        match cli.command {
            Commands::Knowledge {
                command:
                    KnowledgeCommand::Run {
                        root,
                        output,
                        profile,
                        no_window,
                        port,
                    },
            } => {
                assert_eq!(root, PathBuf::from("knowledge"));
                assert!(output.is_none());
                assert!(matches!(profile, KnowledgeProfileArg::Base));
                assert!(no_window);
                assert_eq!(port, PortArg::Exact(8123));
            }
            _ => panic!("expected knowledge run"),
        }
        let cli = Cli::try_parse_from([
            "rocs",
            "knowledge",
            "check",
            "knowledge",
            "--profile",
            "base",
            "--format",
            "json",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Knowledge {
                command: KnowledgeCommand::Check { .. }
            }
        ));
        let cli = Cli::try_parse_from([
            "rocs",
            "knowledge",
            "inspect",
            "concept",
            "architecture/system-overview",
            "knowledge",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Knowledge {
                command: KnowledgeCommand::Inspect { .. }
            }
        ));
        let cli = Cli::try_parse_from([
            "rocs",
            "knowledge",
            "search",
            "theme resolver",
            "knowledge",
            "--type",
            "Architecture",
            "--tag",
            "domain/rocdown",
            "--status",
            "stable",
            "--authority",
            "descriptive",
            "--trust-tier",
            "human-reviewed",
            "--stale",
            "false",
        ])
        .unwrap();
        match cli.command {
            Commands::Knowledge {
                command:
                    KnowledgeCommand::Search {
                        query,
                        root,
                        filters,
                        ..
                    },
            } => {
                assert_eq!(query, "theme resolver");
                assert_eq!(root, PathBuf::from("knowledge"));
                assert_eq!(filters.types, ["Architecture"]);
                assert_eq!(filters.tags, ["domain/rocdown"]);
                assert_eq!(filters.statuses, ["stable"]);
                assert_eq!(filters.authorities, ["descriptive"]);
                assert!(matches!(
                    filters.trust_tiers.as_slice(),
                    [TrustTierArg::HumanReviewed]
                ));
                assert_eq!(filters.stale, Some(false));
            }
            _ => panic!("expected filtered knowledge search"),
        }
        let cli = Cli::try_parse_from([
            "rocs",
            "knowledge",
            "benchmark",
            "knowledge",
            "--questions",
            "knowledge/retrieval-benchmark.toml",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Knowledge {
                command: KnowledgeCommand::Benchmark { .. }
            }
        ));
    }
}
