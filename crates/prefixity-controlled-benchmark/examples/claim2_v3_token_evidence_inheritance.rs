// The V3 audit composes the frozen V1 and V2 generators in one example target.
#![allow(clippy::duplicate_mod)]

#[path = "support/claim2_v3_token_evidence_inheritance.rs"]
mod inheritance;
#[path = "support/claim2_tokenization_report.rs"]
#[allow(dead_code)]
mod v1_renderer;

use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let v1_ledger = v1_renderer::report_bytes(&root)?;
    let v3_ledger = std::fs::read(root.join("fixtures/claim2/workload-request-ledger-v3.json"))?;
    let bytes = inheritance::map_bytes(&root, &v1_ledger, &v3_ledger)?;
    match args.as_slice() {
        [] => std::io::stdout().write_all(&bytes)?,
        [flag] if flag == "--write" => std::fs::write(root.join(inheritance::OUTPUT_PATH), bytes)?,
        [flag, path] if flag == "--write" => std::fs::write(root.join(path), bytes)?,
        _ => return Err("usage: claim2_v3_token_evidence_inheritance [--write [PATH]]".into()),
    }
    Ok(())
}
