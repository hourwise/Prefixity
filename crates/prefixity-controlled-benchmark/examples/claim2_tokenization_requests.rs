#[path = "support/claim2_tokenization_report.rs"]
mod report;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (bytes, output): (Vec<u8>, &str) = match args.as_slice() {
        [] => (report::report_bytes(&root)?, "-"),
        [flag, path] if flag == "--write" => (report::report_bytes(&root)?, path),
        _ => {
            return Err(format!(
                "usage: claim2_tokenization_requests [--write PATH] (ledger: {})",
                report::output_path()
            )
            .into())
        }
    };
    if output == "-" {
        std::io::stdout().write_all(&bytes)?;
    } else {
        std::fs::write(output, bytes)?;
    }
    Ok(())
}
