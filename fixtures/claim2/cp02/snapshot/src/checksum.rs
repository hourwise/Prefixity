/// Stable non-cryptographic key hash used only for local lookup hints.
pub fn hash_key(key: &str) -> u64 {
    key.as_bytes().iter().fold(0xcbf29ce484222325, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

/// Fingerprint a segment body for manifest comparison.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |state, byte| state.rotate_left(5) ^ u64::from(*byte))
}
