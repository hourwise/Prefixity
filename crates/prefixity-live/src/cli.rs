//! Command-line interface for the `prefixity-live` binary.
//!
//! Safety model: no command makes a network call unless `--execute-live` is
//! passed on the `run` subcommand. `dry-run` always makes zero network
//! requests. `--max-requests` has a hard Phase 0B ceiling of 10.

use crate::credentials::Credentials;
use crate::error::LiveError;
use crate::experiment::{
    describe_dry_run, execute_live_experiment, DryRunInfo, ExperimentConfig, StdThreadSleeper,
};
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
/// Default local **Prefixity-estimate** input safety ceiling. This is a
/// conservative local estimate and is NOT a provider billing/tokenizer
/// guarantee.
pub const DEFAULT_MAX_ESTIMATED_INPUT_TOKENS: u64 = 50_000;
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
    /// Conservative LOCAL Prefixity-estimate input safety ceiling. This is
    /// NOT a provider billing/tokenizer guarantee.
    #[arg(long, default_value_t = DEFAULT_MAX_ESTIMATED_INPUT_TOKENS)]
    pub max_estimated_input_tokens: u64,
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
            let result =
                execute_live_experiment(&config, &transport, Some(&credential), &StdThreadSleeper)?;
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
    if opts.max_estimated_input_tokens == 0 {
        return Err(LiveError::argument(
            "--max-estimated-input-tokens must be at least 1",
        ));
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
        max_estimated_input_tokens: opts.max_estimated_input_tokens,
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
    print!("{}", format_dry_run(info));
}

/// Build the dry-run report text. Contains no credential value.
fn format_dry_run(info: &DryRunInfo) -> String {
    let mut lines = Vec::new();
    lines.push("=== Prefixity Phase 0B dry run ===".to_string());
    lines.push(format!("provider:             {}", info.provider));
    lines.push(format!("model:                {}", info.model));
    lines.push(format!("scenario:             {}", info.scenario));
    lines.push(format!("planned requests:     {}", info.request_count));
    for turn in &info.turns {
        lines.push(format!(
            "request {} ({}): pre-delay {} ms",
            turn.turn,
            crate::content::turn_label(turn.turn),
            turn.pre_request_delay_ms
        ));
    }
    lines.push(format!("estimated bytes:      {}", info.estimated_bytes));
    lines.push(format!("estimated tokens:     {}", info.estimated_tokens));
    lines.push(format!(
        "artifact destination: {}",
        info.artifact_dir.display()
    ));
    lines.push(format!("required environment: {}", info.required_env_var));
    lines.push(format!("max requests (guard): {}", info.max_requests));
    lines.push(format!(
        "max estimated input tokens: {} (conservative local Prefixity estimate; not a provider billing/tokenizer guarantee)",
        info.max_estimated_input_tokens
    ));
    match &info.guard_reason {
        Some(reason) => lines.push(format!("guard: REFUSED — {reason}")),
        None => lines.push("guard: ok".to_string()),
    }
    lines.push("NO NETWORK REQUESTS WERE MADE.".to_string());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experiment::TurnSpec;
    use clap::Parser;

    #[test]
    fn cli_exposes_max_estimated_input_tokens_not_max_input_tokens() {
        let cli = Cli::try_parse_from([
            "prefixity-live",
            "dry-run",
            "--provider",
            "deepseek",
            "--model",
            "deepseek-v4-flash",
            "--scenario",
            "schema-smoke",
            "--max-estimated-input-tokens",
            "5000",
        ])
        .unwrap();
        let opts = match &cli.command {
            LiveCommand::DryRun { opts } => opts,
            _ => unreachable!(),
        };
        assert_eq!(opts.max_estimated_input_tokens, 5000);
        // The old, misleading name must be rejected by the CLI.
        assert!(Cli::try_parse_from([
            "prefixity-live",
            "dry-run",
            "--provider",
            "deepseek",
            "--model",
            "deepseek-v4-flash",
            "--scenario",
            "schema-smoke",
            "--max-input-tokens",
            "5000",
        ])
        .is_err());
    }

    #[test]
    fn dry_run_report_labels_the_value_as_estimated() {
        let info = DryRunInfo {
            provider: "deepseek".to_string(),
            model: "deepseek-v4-flash".to_string(),
            scenario: "stable-prefix".to_string(),
            request_count: 3,
            turns: vec![
                TurnSpec {
                    turn: 1,
                    header: "h".to_string(),
                    prefix: "p".to_string(),
                    tail: "t1".to_string(),
                    pre_request_delay_ms: 0,
                },
                TurnSpec {
                    turn: 2,
                    header: "h".to_string(),
                    prefix: "p".to_string(),
                    tail: "t2".to_string(),
                    pre_request_delay_ms: 0,
                },
                TurnSpec {
                    turn: 3,
                    header: "h".to_string(),
                    prefix: "p".to_string(),
                    tail: "t3".to_string(),
                    pre_request_delay_ms: 10_000,
                },
            ],
            estimated_bytes: 2246,
            estimated_tokens: 563,
            artifact_dir: std::path::PathBuf::from("experiments/runs/x"),
            required_env_var: "DEEPSEEK_API_KEY",
            max_requests: 3,
            max_estimated_input_tokens: 25_000,
            guard_reason: None,
        };
        let report = format_dry_run(&info);
        assert!(report.contains("max estimated input tokens"));
        assert!(report.contains("25000"));
        assert!(report.contains("not a provider billing/tokenizer guarantee"));
        // The per-request settle plan is visible.
        assert!(report.contains("request 1 (A): pre-delay 0 ms"));
        assert!(report.contains("request 3 (C): pre-delay 10000 ms"));
    }
}
