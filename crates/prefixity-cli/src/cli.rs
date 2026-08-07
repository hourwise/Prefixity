//! Command-line definition and dispatch for the `prefixity` binary.

use crate::load;
use crate::output;
use clap::{Parser, Subcommand};
use prefixity_core::error::PrefixityError;
use prefixity_core::model::CostProfile;
use std::path::PathBuf;

/// Root CLI structure. `--json` and `--provider-profile` are global options
/// accepted before or after the subcommand.
#[derive(Debug, Parser)]
#[command(
    name = "prefixity",
    version,
    about = "Experimental offline context-efficiency profiler (Phase 0). Does NOT modify live LLM requests."
)]
pub struct Cli {
    /// Emit stable machine-readable JSON instead of human text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Path to a provider cost profile JSON file. When omitted, `simulate`
    /// uses a built-in SYNTHETIC profile; `analyse`/`compare` omit the cost
    /// section.
    #[arg(long, global = true)]
    pub provider_profile: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

/// The four Phase 0 commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate a trace file's structure without analysing it.
    Validate {
        /// Path to a trace JSON file.
        trace: PathBuf,
    },
    /// Analyse a single trace: accounting, prefixity scores, cost.
    Analyse {
        /// Path to a trace JSON file.
        trace: PathBuf,
    },
    /// Compare two traces: divergence, reusable prefix, cache economics.
    Compare {
        /// Path to the earlier trace JSON file.
        trace_a: PathBuf,
        /// Path to the later trace JSON file.
        trace_b: PathBuf,
    },
    /// Simulate a context policy on a trace (offline; never mutates the trace).
    Simulate {
        /// Path to a trace JSON file.
        trace: PathBuf,
        /// Policy name: baseline, stable-prefix, defer-volatile,
        /// prune-stale-tool-output, combined.
        #[arg(long)]
        policy: String,
    },
}

/// Run the parsed CLI and print its output. Errors are reported by the caller.
pub fn run(cli: &Cli) -> Result<(), PrefixityError> {
    let profile = match &cli.provider_profile {
        Some(path) => Some(load::load_cost_profile(path)?),
        None => None,
    };

    match &cli.command {
        Command::Validate { trace } => {
            let trace_data = load::load_trace(trace)?;
            let report = prefixity_core::validation::validate_trace(&trace_data, Some(trace))?;
            output::print_validation(cli, trace, &report);
        }
        Command::Analyse { trace } => {
            let trace_data = load::load_trace(trace)?;
            let analysis = prefixity_core::analysis::analyze_trace(&trace_data, profile.as_ref())?;
            output::print_analysis(cli, &analysis);
        }
        Command::Compare { trace_a, trace_b } => {
            let a = load::load_trace(trace_a)?;
            let b = load::load_trace(trace_b)?;
            let comparison = prefixity_core::compare::compare_traces(&a, &b, profile.as_ref())?;
            output::print_comparison(cli, &comparison);
        }
        Command::Simulate { trace, policy } => {
            let trace_data = load::load_trace(trace)?;
            let policy_obj = prefixity_core::policy::policy_from_name(policy)?;
            let profile_ref = resolve_profile(profile.as_ref())?;
            let result = prefixity_core::policy::simulate_policy(
                &trace_data,
                policy_obj.as_ref(),
                profile_ref,
            )?;
            output::print_simulation(cli, &result);
        }
    }
    Ok(())
}

/// Use the supplied profile, or fall back to the built-in SYNTHETIC profile.
fn resolve_profile(profile: Option<&CostProfile>) -> Result<&CostProfile, PrefixityError> {
    match profile {
        Some(p) => Ok(p),
        None => {
            // NOTE: printed to stderr so `--json` stdout stays a single
            // parseable document.
            eprintln!(
                "note: no --provider-profile given; using built-in SYNTHETIC profile 'synthetic-example'"
            );
            static DEFAULT: std::sync::OnceLock<CostProfile> = std::sync::OnceLock::new();
            Ok(DEFAULT.get_or_init(prefixity_core::cost::default_synthetic_profile))
        }
    }
}
