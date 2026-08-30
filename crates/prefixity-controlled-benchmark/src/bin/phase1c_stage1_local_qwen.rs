use prefixity_controlled_benchmark::{execute_stage1_smoke, preflight_stage1_smoke};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("preflight") if args.next().is_none() => match preflight_stage1_smoke() {
            Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        },
        Some("run")
            if args.next().as_deref() == Some("--confirm-fresh-runtime")
                && args.next().is_none() =>
        {
            match execute_stage1_smoke(true) {
                Ok(record) => println!("{}", serde_json::to_string_pretty(&record).unwrap()),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!(
                "usage: prefixity-phase1c-stage1-smoke preflight | run --confirm-fresh-runtime"
            );
            std::process::exit(2);
        }
    }
}
