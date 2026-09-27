/// Snapshot metadata for one immutable segment generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentManifest {
    pub generation: u64,
    pub first_key: String,
    pub last_key: String,
    pub row_count: usize,
}

impl SegmentManifest {
    /// Validate the manifest's key bounds and row count before opening data.
    pub fn validate(&self) -> bool {
        !self.first_key.is_empty()
            && self.first_key <= self.last_key
            && self.row_count > 0
    }

    /// Return the immutable segment file name for this generation.
    pub fn segment_name(&self) -> String {
        format!("segment-{:04}.tsv", self.generation)
    }
}
