use prefixity_controlled_benchmark::validate_stage1_reasoning_off_evidence;

fn main() {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("validate") || args.next().is_some() {
        eprintln!("usage: prefixity-phase1c-stage1-reasoning-off-postrun validate");
        std::process::exit(2);
    }
    match validate_stage1_reasoning_off_evidence() {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
