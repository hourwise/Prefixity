//! Checkpoint metadata retained across an interrupted segment rebuild.

use segment_format::SegmentFrame;

/// Recovery cursor for the last committed immutable segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryMarker {
    pub offset: u64,
    pub segment: String,
}

impl RecoveryMarker {
    /// Create a marker after a validated segment frame.
    pub fn after_frame(offset: u64, segment: impl Into<String>, frame: &SegmentFrame) -> Option<Self> {
        frame.is_readable().then(|| Self { offset, segment: segment.into() })
    }

    /// Create a marker used when a checkpoint names a segment directly.
    pub fn new(offset: u64, segment: impl Into<String>) -> Self {
        Self { offset, segment: segment.into() }
    }
}
