#[path = "support/claim2_cp10_fixture.rs"]
mod fixture;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let flag = args.next().ok_or("expected --write")?;
    let destination = args.next().ok_or("expected fixture directory")?;
    if flag != "--write" || args.next().is_some() {
        return Err("usage: claim2_cp10_fixture --write <fixture-directory>".into());
    }
    let path = PathBuf::from(destination);
    fixture::write_fixture(&path)?;
    println!("wrote deterministic CP10 fixture to {}", path.display());
    Ok(())
}
