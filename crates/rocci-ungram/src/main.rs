use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};
use rocci_ungram::{check_languages, find_workspace_root, write_languages};

#[derive(Parser)]
#[command(
    name = "rocci-ungram",
    about = "Generate owned AST structs from ungrammar tree specs. Does not generate scanners or parsers."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Write snapshot AST Rust for both language ungrams.
    Generate {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    /// Fail if snapshot AST Rust is stale.
    Check {
        #[arg(long)]
        root: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    match Cli::parse().command {
        Commands::Generate { root } => {
            let root = workspace_root(root)?;
            for path in write_languages(&root)? {
                println!("wrote {}", path.display());
            }
            Ok(())
        }
        Commands::Check { root } => {
            let root = workspace_root(root)?;
            check_languages(&root)?;
            println!("ok: AST snapshots are current");
            Ok(())
        }
    }
}

fn workspace_root(root: Option<PathBuf>) -> Result<PathBuf> {
    match root {
        Some(path) => Ok(find_workspace_root(&path)?),
        None => Ok(find_workspace_root(&std::env::current_dir()?)?),
    }
}
