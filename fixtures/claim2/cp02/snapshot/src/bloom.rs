/// A small deterministic membership hint used before reading a segment.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<bool>,
}

impl BloomFilter {
    /// Allocate a filter with a fixed bit count.
    pub fn new(bit_count: usize) -> Self {
        Self { bits: vec![false; bit_count.max(1)] }
    }

    /// Mark a key's stable hash position as present.
    pub fn insert(&mut self, key: &str) {
        let index = super::checksum::hash_key(key) as usize % self.bits.len();
        self.bits[index] = true;
    }

    /// Return false only when a key is definitely absent.
    pub fn might_contain(&self, key: &str) -> bool {
        let index = super::checksum::hash_key(key) as usize % self.bits.len();
        self.bits[index]
    }
}
