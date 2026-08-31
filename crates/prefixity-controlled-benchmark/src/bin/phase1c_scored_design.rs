use prefixity_controlled_benchmark::{fingerprint_scored_design, validate_scored_design};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("fingerprint") if args.next().is_none() => match fingerprint_scored_design() {
            Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        },
        Some("validate") if args.next().is_none() => match validate_scored_design() {
            Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        },
        _ => {
            eprintln!("usage: prefixity-phase1c-scored-design [fingerprint|validate]");
            std::process::exit(2);
        }
    }
}
