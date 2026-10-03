#[path = "support/claim2_v3_tokenization_plans.rs"]
mod plans;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (hybrid, full) = plans::plan_pair(&root)?;
    let outputs = [
        (plans::HYBRID_PATH, hybrid, "HYBRID_8_NEW"),
        (plans::FULL_FRESH_PATH, full, "FULL_FRESH_22"),
    ];
    let write = match args.as_slice() {
        [] => false,
        [flag] if flag == "--check" => false,
        [flag] if flag == "--write" => true,
        _ => return Err("usage: claim2_v3_tokenization_plans [--check|--write]".into()),
    };
    for (path, bytes, mode) in outputs {
        let plan: Value = serde_json::from_slice(&bytes)?;
        if write {
            std::fs::write(root.join(path), &bytes)?;
        } else if std::fs::read(root.join(path))? != bytes {
            return Err(format!("frozen plan differs from regeneration: {path}").into());
        }
        println!(
            "{mode} {} {}",
            plan["entries"].as_array().map(Vec::len).unwrap_or_default(),
            digest(&bytes)
        );
    }
    Ok(())
}
