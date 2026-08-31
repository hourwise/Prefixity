use prefixity_controlled_benchmark::{
    dry_run_v2, execute_v2_baseline, fingerprint_v2, preflight_v2, score_v2_baseline, H001Error,
};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("preflight") if args.next().is_none() => preflight_v2(),
        Some("fingerprint") if args.next().is_none() => fingerprint_v2(),
        Some("dry-run") if args.next().is_none() => dry_run_v2(),
        Some("score") if args.next().is_none() => score_v2_baseline(),
        Some("run") => match (args.next(), args.next(), args.next()) {
            (None, Some(flag), None) if flag == "--confirm-fresh-runtime" => {
                execute_v2_baseline(true)
            }
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

fn usage() -> H001Error {
    H001Error::Validation(
        "usage: prefixity-phase1c-h001-v2 [preflight|fingerprint|dry-run|score|run --confirm-fresh-runtime]".to_string(),
    )
}
