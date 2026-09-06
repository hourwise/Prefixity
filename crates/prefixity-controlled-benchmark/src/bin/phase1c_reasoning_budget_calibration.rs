use prefixity_controlled_benchmark::{
    attempt_002_exclusivity_preflight, attempt_003_poststart, attempt_005_poststart,
    attempt_006_poststart, dry_run_attempt_002, dry_run_attempt_003, dry_run_attempt_004,
    dry_run_attempt_005, dry_run_attempt_006, dry_run_calibration, execute_attempt_002,
    execute_attempt_003, execute_calibration, fingerprint_attempt_002, fingerprint_attempt_003,
    fingerprint_attempt_004, fingerprint_attempt_005, fingerprint_attempt_006,
    fingerprint_calibration, parse_calibration_cli_args, preflight_attempt_002,
    preflight_attempt_003, preflight_attempt_004, preflight_attempt_005, preflight_attempt_006,
    preflight_calibration, summarize_attempt_002_budget, summarize_calibration_budget,
    CalibrationCliCommand,
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
        Ok(CalibrationCliCommand::Attempt003Fingerprint) => fingerprint_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003Preflight) => preflight_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003DryRun) => dry_run_attempt_003(),
        Ok(CalibrationCliCommand::Attempt003Poststart) => attempt_003_poststart(),
        Ok(CalibrationCliCommand::RunAttempt003) => execute_attempt_003(),
        Ok(CalibrationCliCommand::Attempt004Fingerprint) => fingerprint_attempt_004(),
        Ok(CalibrationCliCommand::Attempt004Preflight) => preflight_attempt_004(),
        Ok(CalibrationCliCommand::Attempt004DryRun) => dry_run_attempt_004(),
        Ok(CalibrationCliCommand::Attempt005Fingerprint) => fingerprint_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005Preflight) => preflight_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005DryRun) => dry_run_attempt_005(),
        Ok(CalibrationCliCommand::Attempt005Poststart) => attempt_005_poststart(),
        Ok(CalibrationCliCommand::Attempt006Fingerprint) => fingerprint_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006Preflight) => preflight_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006DryRun) => dry_run_attempt_006(),
        Ok(CalibrationCliCommand::Attempt006Poststart) => attempt_006_poststart(),
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
