#[path = "support/claim2_v2_token_evidence_inheritance.rs"]
mod inheritance;
#[path = "support/claim2_tokenization_report.rs"]
mod report;

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
            "usage: claim2_v2_token_evidence_inheritance [--write PATH] (V1 ledger: {}; map: {})",
            report::output_path(),
            inheritance::OUTPUT_PATH
        )
            .into())
        }
    };
    let regenerated_v1_ledger = report::report_bytes(&root)?;
    let bytes = inheritance::map_bytes(&root, &regenerated_v1_ledger)?;
    if output == "-" {
        std::io::stdout().write_all(&bytes)?;
    } else {
        std::fs::write(output, bytes)?;
    }
    Ok(())
}
