//! Okmate library: CLI and later HTTP/desktop surfaces over the portable `okf` engine.

pub mod cli;

use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use okf::{CheckReport, Profile};

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
