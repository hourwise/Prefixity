#[path = "support/claim2_domain_report.rs"]
mod report;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (bytes, path): (Vec<u8>, &str) = match args.as_slice() {
        [flag, path] if flag == "--write-ledger" => (report::ledger_bytes(&root)?, path.as_str()),
        [flag, path] if flag == "--write-successor" => {
            (report::successor_report_bytes(&root)?, path.as_str())
        }
        [] => (report::successor_report_bytes(&root)?, "-"),
        _ => {
            return Err(
                "usage: claim2_protocol_successor [--write-ledger|--write-successor PATH]".into(),
            )
        }
    };
    if path == "-" {
        std::io::stdout().write_all(&bytes)?;
    } else {
        std::fs::write(path, bytes)?;
    }
    Ok(())
}
