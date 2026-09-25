use prefixity_controlled_benchmark::{
    dry_run_v3_feasibility_gate, execute_v3_feasibility_gate, freeze_v3_feasibility_gate,
    preflight_v3_feasibility_gate, validate_v3_live_prerequisites,
};
use std::env;

const USAGE: &str = "usage: prefixity-phase1c-v3-feasibility [v3-dry-run|v3-dry-run-offline|v3-freeze|v3-preflight|v3-live-prerequisites|run-v3-feasibility-gate]";

fn main() {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["v3-dry-run"] => dry_run_v3_feasibility_gate(true),
        ["v3-dry-run-offline"] => dry_run_v3_feasibility_gate(false),
        ["v3-freeze"] => freeze_v3_feasibility_gate(),
        ["v3-preflight"] => preflight_v3_feasibility_gate(),
        ["v3-live-prerequisites"] => validate_v3_live_prerequisites(),
        ["run-v3-feasibility-gate"] => execute_v3_feasibility_gate(),
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
