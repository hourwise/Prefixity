use prefixity_controlled_benchmark::{
    dry_run_calibration, execute_calibration, fingerprint_calibration, parse_calibration_cli_args,
    preflight_calibration, summarize_calibration_budget, CalibrationCliCommand,
};
use std::env;

fn main() {
    let result = match parse_calibration_cli_args(env::args().skip(1)) {
        Ok(CalibrationCliCommand::Preflight) => preflight_calibration(),
        Ok(CalibrationCliCommand::Fingerprint) => fingerprint_calibration(),
        Ok(CalibrationCliCommand::DryRun) => dry_run_calibration(),
        Ok(CalibrationCliCommand::Run { budget }) => execute_calibration(budget, true),
        Ok(CalibrationCliCommand::Summarize { budget }) => summarize_calibration_budget(budget),
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
