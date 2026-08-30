use prefixity_controlled_benchmark::preflight_stage1_reasoning_off_smoke;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("preflight") if args.next().is_none() => {
            match preflight_stage1_reasoning_off_smoke() {
                Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            eprintln!("usage: prefixity-phase1c-stage1-reasoning-off-preflight preflight");
            std::process::exit(2);
        }
    }
}
