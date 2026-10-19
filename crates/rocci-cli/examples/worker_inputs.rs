//! Maintainer input staging for the opt-in actual-Roc Worker conformance lane.
use std::{env, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let input = args.next().expect("application source");
    let output = args.next().expect("output directory");
    let archive = args.next().expect("wasm32 Worker host archive");
    rocci_cli::worker::stage(&input, &output, &archive)
}
