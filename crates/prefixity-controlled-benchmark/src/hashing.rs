use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn hash_text(text: &str) -> String {
    sha256_hex(text.as_bytes())
}

/// Return the canonical UTF-8 source bytes used for implementation identity.
///
/// Checkout line endings are transport details, so CRLF and lone CR are
/// normalized to LF before hashing. A BOM is rejected rather than silently
/// becoming part of the identity, and non-UTF-8 input is rejected explicitly.
pub fn canonicalize_source_bytes(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err("source fingerprint input contains a UTF-8 BOM");
    }
    std::str::from_utf8(bytes).map_err(|_| "source fingerprint input is not valid UTF-8")?;

    let mut canonical = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\r' {
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 1;
            }
            canonical.push(b'\n');
        } else {
            canonical.push(bytes[index]);
        }
        index += 1;
    }
    Ok(canonical)
}

/// Serialize JSON with recursively sorted object keys and stable array order.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let canonical = canonical_value(value);
    serde_json::to_vec(&canonical)
}

pub fn canonical_hash<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    canonical_json(value).map(|bytes| sha256_hex(&bytes))
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<String, Value> = object
                .into_iter()
                .map(|(key, value)| (key, canonical_value(value)))
                .collect();
            let mut canonical = Map::new();
            for (key, value) in sorted {
                canonical.insert(key, value);
            }
            Value::Object(canonical)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonical_value).collect()),
        scalar => scalar,
    }
}

#[cfg(test)]
mod tests {
    use super::canonicalize_source_bytes;

    #[test]
    fn source_canonicalization_normalizes_lf_crlf_and_lone_cr() {
        let lf = b"alpha\nbeta\ngamma\n";
        let crlf = b"alpha\r\nbeta\r\ngamma\r\n";
        let mixed = b"alpha\rbeta\r\ngamma\n";

        assert_eq!(canonicalize_source_bytes(lf).unwrap(), lf);
        assert_eq!(canonicalize_source_bytes(crlf).unwrap(), lf);
        assert_eq!(canonicalize_source_bytes(mixed).unwrap(), lf);
    }

    #[test]
    fn source_canonicalization_rejects_bom_and_non_utf8() {
        assert_eq!(
            canonicalize_source_bytes(&[0xef, 0xbb, 0xbf, b'a']).unwrap_err(),
            "source fingerprint input contains a UTF-8 BOM"
        );
        assert_eq!(
            canonicalize_source_bytes(&[0xff]).unwrap_err(),
            "source fingerprint input is not valid UTF-8"
        );
    }

    #[test]
    fn source_canonicalization_preserves_mutations() {
        assert_ne!(
            canonicalize_source_bytes(b"alpha\nbeta\ngamma\n").unwrap(),
            canonicalize_source_bytes(b"alpha\nbeta\ndelta\n").unwrap()
        );
    }
}
