use prefixity_controlled_benchmark::{
    dry_run_h001, execute_h001_arm, fingerprint_h001, preflight_h001, score_h001_arm, H001Arm,
};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("preflight") if args.next().is_none() => preflight_h001(),
        Some("fingerprint") if args.next().is_none() => fingerprint_h001(),
        Some("dry-run") => match (args.next(), args.next()) {
            (Some(arm), None) => H001Arm::parse(&arm).and_then(dry_run_h001),
            _ => Err(usage()),
        },
        Some("run") => match (args.next(), args.next(), args.next()) {
            (Some(arm), Some(flag), None) if flag == "--confirm-fresh-runtime" => {
                H001Arm::parse(&arm).and_then(|arm| execute_h001_arm(arm, true))
            }
            _ => Err(usage()),
        },
        Some("score") => match (args.next(), args.next()) {
            (Some(arm), None) => H001Arm::parse(&arm).and_then(score_h001_arm),
            _ => Err(usage()),
        },
        _ => Err(usage()),
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

fn usage() -> prefixity_controlled_benchmark::H001Error {
    prefixity_controlled_benchmark::H001Error::Validation(
        "usage: prefixity-phase1c-h001 [preflight|dry-run ARM|run ARM --confirm-fresh-runtime|score ARM]".to_string(),
    )
}
