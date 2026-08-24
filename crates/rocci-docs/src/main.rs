use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::Parser;
use rocci_docs::{generate_with, live_apps, load_catalog};

#[derive(Parser)]
#[command(
    name = "rocci-docs",
    about = "Stage colocated Rocdown and published source for cataloged Rocci apps."
)]
struct Cli {
    /// Catalog TOML (typically examples/rocci/apps.toml).
    #[arg(long)]
    catalog: PathBuf,
    /// Output directory for the Rocdown staging tree.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Print live-hosting catalog rows as `id\tpath\tentry` and exit.
    #[arg(long)]
    print_live: bool,
    /// Stage apps with `site = false` as well (local preview; not for package site).
    #[arg(long)]
    all: bool,
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
    let cli = Cli::parse();
    if cli.print_live {
        let catalog = load_catalog(&cli.catalog)?;
        for app in live_apps(&catalog) {
            println!("{}\t{}\t{}", app.id, app.path, app.entry);
        }
        return Ok(());
    }
    let Some(output) = cli.output else {
        bail!("--output is required unless --print-live is set");
    };
    let report = generate_with(&cli.catalog, &output, cli.all)?;
    println!(
        "staged {} apps ({} files) into {}",
        report.apps,
        report.files,
        output.display()
    );
    Ok(())
}
