//! Stable record envelope shared by the local audit tools.

/// Wire revision understood by the 2.x protocol series.
pub const WIRE_REVISION: u16 = 4;

/// The envelope written to a segment by the workspace producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEnvelope {
    pub key: String,
    pub payload: String,
    pub wire_revision: u16,
}

impl RecordEnvelope {
    /// Build an envelope using the current protocol revision.
    pub fn new(key: impl Into<String>, payload: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            payload: payload.into(),
            wire_revision: WIRE_REVISION,
        }
    }

    /// Return whether a reader can decode this envelope revision.
    pub fn is_supported_by(reader_revision: u16) -> bool {
        reader_revision >= WIRE_REVISION
    }
}
