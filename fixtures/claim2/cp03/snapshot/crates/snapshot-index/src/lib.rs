//! Key range metadata for one immutable snapshot generation.

use segment_format::SegmentFrame;
use wire_schema::RecordEnvelope;

/// Ordered range and generation facts needed by a snapshot reader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexManifest {
    pub generation: u64,
    pub first_key: String,
    pub last_key: String,
}

impl IndexManifest {
    /// Check that a record frame lies in the manifest's key range.
    pub fn contains(&self, frame: &SegmentFrame) -> bool {
        let key = &frame.record.key;
        key >= &self.first_key && key <= &self.last_key && frame.is_readable()
    }

    /// Build one protocol record describing the current generation.
    pub fn as_record(&self) -> RecordEnvelope {
        RecordEnvelope::new(format!("generation-{}", self.generation), self.last_key.clone())
    }
}
