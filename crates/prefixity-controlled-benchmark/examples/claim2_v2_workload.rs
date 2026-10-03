#[path = "support/claim2_v2_workload.rs"]
mod workload;

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (bytes, path) = match args.as_slice() {
        [kind] if kind == "domain" => (workload::domain_bytes(&root)?, workload::DOMAIN_PATH),
        [kind] if kind == "ledger" => (workload::ledger_bytes(&root)?, workload::LEDGER_PATH),
        [kind] if kind == "successor" => {
            (workload::successor_bytes(&root)?, workload::SUCCESSOR_PATH)
        }
        _ => return Err("usage: claim2_v2_workload domain|ledger|successor".into()),
    };
    std::fs::write(root.join(path), bytes)?;
    Ok(())
}
