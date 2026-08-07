//! `prefixity-live` — Phase 0B controlled live validation harness.
//!
//! Experimental research software. Makes paid/network API calls ONLY when
//! the `run` subcommand is invoked with `--execute-live`.

use clap::Parser;

fn main() {
    let cli = prefixity_live::cli::Cli::parse();
    if let Err(error) = prefixity_live::cli::run(&cli) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
