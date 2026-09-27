use quarry_ledger::{open, LookupQuery};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let index = PathBuf::from(args.next().ok_or("missing index path")?);
    let key = args.next().ok_or("missing record key")?;
    let library = open(index)?;
    let query = LookupQuery::new(key.to_string_lossy().into_owned())?;
    match library.find_record(&query)? {
        Some(record) => println!("{}\t{}", record.key, record.value),
        None => println!("not-found"),
    }
    Ok(())
}
