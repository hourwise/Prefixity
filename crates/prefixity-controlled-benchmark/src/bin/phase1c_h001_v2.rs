use prefixity_controlled_benchmark::{
    dry_run_v2_attempt_002, execute_v2_baseline, fingerprint_v2, parse_v2_cli_args,
    preflight_v2_attempt_002, score_v2_baseline, V2CliCommand,
};
use std::env;

fn main() {
    let result = match parse_v2_cli_args(env::args().skip(1)) {
        Ok(V2CliCommand::Preflight) => preflight_v2_attempt_002(),
        Ok(V2CliCommand::Fingerprint) => fingerprint_v2(),
        Ok(V2CliCommand::DryRun) => dry_run_v2_attempt_002(),
        Ok(V2CliCommand::Score) => score_v2_baseline(),
        Ok(V2CliCommand::RunBaseline) => execute_v2_baseline(true),
        Err(error) => Err(error),
    };
    match result {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).expect("result serializes")
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
