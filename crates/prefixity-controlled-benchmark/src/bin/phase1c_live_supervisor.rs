use prefixity_controlled_benchmark::{
    persist_supervisor_result, run_supervised, H001Error, PRODUCTION_SUPERVISOR_TIMEOUT_MS,
};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let mut args = env::args().skip(1);
    let evidence_path = match (args.next().as_deref(), args.next()) {
        (Some("--evidence"), Some(path)) => PathBuf::from(path),
        _ => fail(usage()),
    };
    if args.next().as_deref() != Some("--") {
        fail(usage());
    }
    let program = match args.next() {
        Some(path) => PathBuf::from(path),
        None => fail(usage()),
    };
    let child_args = args.collect::<Vec<_>>();
    let result = match run_supervised(
        &program,
        &child_args,
        Duration::from_millis(PRODUCTION_SUPERVISOR_TIMEOUT_MS),
    ) {
        Ok(result) => result,
        Err(error) => fail(error),
    };
    if let Err(error) = persist_supervisor_result(&evidence_path, &result) {
        fail(error);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&result).expect("result serializes")
    );
    if result["state"] != "COMPLETED" {
        std::process::exit(1);
    }
}

fn usage() -> H001Error {
    H001Error::Validation(format!(
        "usage: prefixity-phase1c-live-supervisor --evidence PATH -- PROGRAM [ARGS...] (deadline fixed at {PRODUCTION_SUPERVISOR_TIMEOUT_MS}ms)"
    ))
}

fn fail(error: H001Error) -> ! {
    eprintln!("{error}");
    std::process::exit(2);
}
