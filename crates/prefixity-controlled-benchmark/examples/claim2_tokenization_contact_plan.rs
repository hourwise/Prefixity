use prefixity_controlled_benchmark::phase1c_claim2_tokenization::{
    contact_plan_bytes, LEDGER_RELATIVE_PATH, PLAN_RELATIVE_PATH,
};
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let output = match args.as_slice() {
        [] => "-",
        [flag, path] if flag == "--write" => path.as_str(),
        _ => {
            return Err(format!(
                "usage: claim2_tokenization_contact_plan [--write PATH] (ledger: {LEDGER_RELATIVE_PATH}; plan: {PLAN_RELATIVE_PATH})"
            )
            .into())
        }
    };
    let ledger = std::fs::read(root.join(LEDGER_RELATIVE_PATH))?;
    let bytes = contact_plan_bytes(&ledger)?;
    if output == "-" {
        std::io::stdout().write_all(&bytes)?;
    } else {
        std::fs::write(output, bytes)?;
    }
    Ok(())
}
