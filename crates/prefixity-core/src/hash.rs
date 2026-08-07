//! Content hashing helpers.
//!
//! Trace blocks carry a SHA-256 hex digest of their content. When `content`
//! is present in a trace, validation recomputes the digest and rejects the
//! trace if it does not match — this is what makes the format self-checking
//! and the analysis deterministic.

use sha2::{Digest, Sha256};

/// Compute the lowercase hex SHA-256 digest of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Hash a string's UTF-8 bytes to a lowercase hex SHA-256 digest.
pub fn hash_content(content: &str) -> String {
    sha256_hex(content.as_bytes())
}

/// Whether `s` looks like a valid 64-character lowercase hex SHA-256 digest.
pub fn is_valid_sha256_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
