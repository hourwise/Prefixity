//! Text segment framing for protocol envelopes.

use wire_schema::RecordEnvelope;

/// One decoded record and its framing checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentFrame {
    pub record: RecordEnvelope,
    pub checksum: u32,
}

impl SegmentFrame {
    /// Encode a record as a stable tab-delimited frame.
    pub fn encode(record: RecordEnvelope) -> Self {
        let bytes = format!("{}\t{}\t{}", record.key, record.wire_revision, record.payload);
        Self { record, checksum: checksum(bytes.as_bytes()) }
    }

    /// Validate a frame against the supported wire revision.
    pub fn is_readable(&self) -> bool {
        self.checksum != 0 && RecordEnvelope::is_supported_by(self.record.wire_revision)
    }
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0x811c9dc5, |state, byte| (state ^ u32::from(*byte)).wrapping_mul(0x01000193))
}
