#[path = "support/claim2_report.rs"]
mod report;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let bytes = report::report_bytes(&root)?;
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [] => std::io::stdout().write_all(&bytes)?,
        [flag, path] if flag == "--write" => std::fs::write(path, bytes)?,
        _ => return Err("usage: claim2_materialization [--write PATH]".into()),
    }
    Ok(())
}
