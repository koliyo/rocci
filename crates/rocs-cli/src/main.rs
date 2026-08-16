use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand, ValueEnum};

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
}

#[derive(Clone, Copy, ValueEnum)]
enum CheckFormatArg {
    Terminal,
    Json,
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
    }
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
}
