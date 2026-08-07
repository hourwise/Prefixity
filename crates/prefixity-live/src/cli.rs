//! Command-line interface for the `prefixity-live` binary.
//!
//! Safety model: no command makes a network call unless `--execute-live` is
//! passed on the `run` subcommand. `dry-run` always makes zero network
//! requests. `--max-requests` has a hard Phase 0B ceiling of 10.

use crate::credentials::Credentials;
use crate::error::LiveError;
use crate::experiment::{describe_dry_run, execute_live_experiment, DryRunInfo, ExperimentConfig};
use crate::scenario::Scenario;
use crate::transport::ReqwestTransport;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Hard Phase 0B ceiling on requests per command.
pub const MAX_REQUESTS_CEILING: usize = 10;
/// Default request-count guard.
pub const DEFAULT_MAX_REQUESTS: usize = 3;
/// Default approximate target prefix size in tokens.
pub const DEFAULT_TARGET_PREFIX_TOKENS: u64 = 8_000;
/// Default local input-token safety ceiling.
pub const DEFAULT_MAX_INPUT_TOKENS: u64 = 50_000;
/// Default per-request timeout in milliseconds.
pub const DEFAULT_TIMEOUT_MS: u64 = 60_000;
/// Default runs directory.
pub const DEFAULT_RUNS_DIR: &str = "experiments/runs";

/// Root CLI.
#[derive(Debug, Parser)]
#[command(
    name = "prefixity-live",
    version,
    about = "Phase 0B controlled live validation harness (experimental research). Makes paid API calls ONLY with --execute-live."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: LiveCommand,
}

/// The two live commands.
#[derive(Debug, Subcommand)]
pub enum LiveCommand {
    /// Print exactly what a live run would send. Makes zero network requests.
    DryRun {
        #[command(flatten)]
        opts: LiveRunOptions,
    },
    /// Run the experiment. Requires --execute-live to make calls; without it
    /// this subcommand is a dry run that makes zero network requests.
    Run {
        #[command(flatten)]
        opts: LiveRunOptions,
        /// Explicit permission to make paid/network calls.
        #[arg(long)]
        execute_live: bool,
        /// Emit the result as JSON to stdout.
        #[arg(long)]
        json: bool,
    },
}

/// Options shared by both commands.
#[derive(Debug, Clone, Args)]
pub struct LiveRunOptions {
    /// Provider id: openai, anthropic, deepseek.
    #[arg(long)]
    pub provider: String,
    /// Exact provider model id (never substituted on failure).
    #[arg(long)]
    pub model: String,
    /// Scenario: schema-smoke, stable-prefix, early-divergence, late-divergence (or A-D).
    #[arg(long)]
    pub scenario: String,
    /// Seed for deterministic synthetic content.
    #[arg(long, default_value_t = 42)]
    pub seed: u64,
    /// Approximate target stable-prefix size in tokens.
    #[arg(long, default_value_t = DEFAULT_TARGET_PREFIX_TOKENS)]
    pub target_prefix_tokens: u64,
    /// Request-count guard (default 3, hard ceiling 10).
    #[arg(long, default_value_t = DEFAULT_MAX_REQUESTS)]
    pub max_requests: usize,
    /// Local input-token safety ceiling.
    #[arg(long, default_value_t = DEFAULT_MAX_INPUT_TOKENS)]
    pub max_input_tokens: u64,
    /// Per-request timeout in milliseconds.
    #[arg(long, default_value_t = DEFAULT_TIMEOUT_MS)]
    pub timeout_ms: u64,
    /// Experiment id (sanitized). Default is generated.
    #[arg(long)]
    pub experiment_id: Option<String>,
    /// Optional experiment notes.
    #[arg(long)]
    pub notes: Option<String>,
}

/// Run the parsed CLI.
pub fn run(cli: &Cli) -> Result<(), LiveError> {
    match &cli.command {
        LiveCommand::DryRun { opts } => {
            let config = build_config(opts)?;
            let info = describe_dry_run(&config)?;
            print_dry_run(&info);
            Ok(())
        }
        LiveCommand::Run {
            opts,
            execute_live,
            json,
        } => {
            let config = build_config(opts)?;
            if !*execute_live {
                // Safety gate: `run` without --execute-live is a dry run.
                let info = describe_dry_run(&config)?;
                print_dry_run(&info);
                return Ok(());
            }
            let provider = crate::providers::provider_from_id(&config.provider_id)?;
            let credential = Credentials::from_env(provider.credential_env_var())?;
            let transport = ReqwestTransport::new()?;
            let result = execute_live_experiment(&config, &transport, Some(&credential))?;
            if *json {
                let text = serde_json::to_string_pretty(&result).map_err(|e| {
                    LiveError::InvalidResponse {
                        message: format!("result serialization failed: {e}"),
                    }
                })?;
                println!("{text}");
            } else {
                println!("{}", result.summary);
            }
            println!(
                "artifacts: {}",
                config.runs_dir.join(&config.experiment_id).display()
            );
            Ok(())
        }
    }
}

/// Build an [`ExperimentConfig`] from CLI options, applying argument
/// validation (including the hard request ceiling).
fn build_config(opts: &LiveRunOptions) -> Result<ExperimentConfig, LiveError> {
    if opts.provider.trim().is_empty() {
        return Err(LiveError::argument("--provider must not be empty"));
    }
    if opts.max_requests == 0 {
        return Err(LiveError::argument("--max-requests must be at least 1"));
    }
    if opts.max_requests > MAX_REQUESTS_CEILING {
        return Err(LiveError::argument(format!(
            "--max-requests {} exceeds the Phase 0B ceiling of {MAX_REQUESTS_CEILING}",
            opts.max_requests
        )));
    }
    if opts.target_prefix_tokens == 0 {
        return Err(LiveError::argument(
            "--target-prefix-tokens must be at least 1",
        ));
    }
    if opts.max_input_tokens == 0 {
        return Err(LiveError::argument("--max-input-tokens must be at least 1"));
    }
    if opts.timeout_ms == 0 {
        return Err(LiveError::argument("--timeout-ms must be at least 1"));
    }

    let scenario = Scenario::parse(&opts.scenario)?;
    let experiment_id = match &opts.experiment_id {
        Some(id) => crate::artifacts::sanitize_experiment_id(id)?,
        None => default_experiment_id(&opts.provider, &scenario)?,
    };
    // The public CLI always writes under the Phase 0B artifact root; there
    // is no user-selectable runs directory. Tests inject temporary roots
    // through `ExperimentConfig` directly.
    let runs_dir = PathBuf::from(DEFAULT_RUNS_DIR);

    Ok(ExperimentConfig {
        provider_id: opts.provider.trim().to_string(),
        model: opts.model.trim().to_string(),
        scenario,
        seed: opts.seed,
        target_prefix_tokens: opts.target_prefix_tokens,
        max_requests: opts.max_requests,
        max_input_tokens: opts.max_input_tokens,
        timeout_ms: opts.timeout_ms,
        runs_dir,
        experiment_id,
        notes: opts.notes.clone(),
    })
}

/// Generate a sanitized default experiment id.
fn default_experiment_id(provider: &str, scenario: &Scenario) -> Result<String, LiveError> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    crate::artifacts::sanitize_experiment_id(&format!("{provider}-{}-{millis}", scenario.as_str()))
}

/// Print a dry-run report. Contains no credential value.
fn print_dry_run(info: &DryRunInfo) {
    println!("=== Prefixity Phase 0B dry run ===");
    println!("provider:             {}", info.provider);
    println!("model:                {}", info.model);
    println!("scenario:             {}", info.scenario);
    println!("planned requests:     {}", info.request_count);
    println!("estimated bytes:      {}", info.estimated_bytes);
    println!("estimated tokens:     {}", info.estimated_tokens);
    println!("artifact destination: {}", info.artifact_dir.display());
    println!("required environment: {}", info.required_env_var);
    println!("max requests (guard): {}", info.max_requests);
    println!("max input tokens:     {}", info.max_input_tokens);
    match &info.guard_reason {
        Some(reason) => println!("guard: REFUSED — {reason}"),
        None => println!("guard: ok"),
    }
    println!("NO NETWORK REQUESTS WERE MADE.");
}
