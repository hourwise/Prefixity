use prefixity_controlled_benchmark::{
    attempt_002_exclusivity_preflight, dry_run_attempt_002, dry_run_calibration,
    execute_attempt_002, execute_calibration, fingerprint_attempt_002, fingerprint_calibration,
    parse_calibration_cli_args, preflight_attempt_002, preflight_calibration,
    summarize_attempt_002_budget, summarize_calibration_budget, CalibrationCliCommand,
};
use std::env;

fn main() {
    let result = match parse_calibration_cli_args(env::args().skip(1)) {
        Ok(CalibrationCliCommand::Preflight) => preflight_calibration(),
        Ok(CalibrationCliCommand::Fingerprint) => fingerprint_calibration(),
        Ok(CalibrationCliCommand::DryRun) => dry_run_calibration(),
        Ok(CalibrationCliCommand::Run { budget }) => execute_calibration(budget, true),
        Ok(CalibrationCliCommand::Summarize { budget }) => summarize_calibration_budget(budget),
        Ok(CalibrationCliCommand::Attempt002Fingerprint) => fingerprint_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002Preflight) => preflight_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002DryRun) => dry_run_attempt_002(),
        Ok(CalibrationCliCommand::Attempt002ExclusivityPreflight) => {
            attempt_002_exclusivity_preflight(true)
        }
        Ok(CalibrationCliCommand::RunAttempt002 { budget, server_pid }) => {
            execute_attempt_002(budget, server_pid, true, true)
        }
        Ok(CalibrationCliCommand::SummarizeAttempt002 { budget }) => {
            summarize_attempt_002_budget(budget)
        }
        Err(error) => Err(error),
    };
    match result {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("calibration result serializes")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
