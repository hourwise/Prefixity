use prefixity_controlled_benchmark::{
    dry_run_local_9b_gate, execute_local_9b_feasibility_gate, freeze_local_9b_gate,
    preflight_local_9b_gate, validate_local_9b_live_prerequisites,
};
use std::env;

const USAGE: &str = "usage: prefixity-phase1c-capable-model-gate [local-9b-dry-run|local-9b-dry-run-offline|local-9b-freeze|local-9b-preflight|local-9b-live-prerequisites|run-local-9b-feasibility-gate]";

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["local-9b-dry-run"] => dry_run_local_9b_gate(true),
        ["local-9b-dry-run-offline"] => dry_run_local_9b_gate(false),
        ["local-9b-freeze"] => freeze_local_9b_gate(),
        ["local-9b-preflight"] => preflight_local_9b_gate(),
        ["local-9b-live-prerequisites"] => validate_local_9b_live_prerequisites(),
        ["run-local-9b-feasibility-gate"] => execute_local_9b_feasibility_gate(),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    match result {
        Ok(value) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).expect("result serializes")
            );
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
