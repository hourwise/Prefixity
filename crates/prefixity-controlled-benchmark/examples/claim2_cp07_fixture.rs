#[path = "support/claim2_cp07_fixture.rs"]
mod fixture;

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let Some(flag) = args.next() else {
        return Err("usage: claim2_cp07_fixture --write DIRECTORY".into());
    };
    let Some(directory) = args.next() else {
        return Err("usage: claim2_cp07_fixture --write DIRECTORY".into());
    };
    if flag != "--write" || args.next().is_some() {
        return Err("usage: claim2_cp07_fixture --write DIRECTORY".into());
    }
    let target = Path::new(&directory);
    fixture::write_fixture(target)?;
    Ok(())
}
