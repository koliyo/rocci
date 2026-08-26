use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use crate::{CheckFormat, ProfileArg, check, print_check};

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
    }
}
