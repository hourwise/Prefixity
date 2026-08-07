//! `prefixity` — CLI entry point.
//!
//! Prefixity is experimental research software. Phase 0 does not modify live
//! LLM requests; every command operates offline on trace files.

mod cli;
mod load;
mod output;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    let exit_code = match cli::run(&cli) {
        Ok(()) => 0,
        Err(error) => {
            output::print_error(&cli, &error);
            1
        }
    };
    std::process::exit(exit_code);
}
