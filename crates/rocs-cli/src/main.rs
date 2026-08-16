use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
        }
    }
}
