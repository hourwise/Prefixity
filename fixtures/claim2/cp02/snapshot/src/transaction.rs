/// A single segment write staged before its manifest is published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentWriteTxn {
    pub generation: u64,
    pub records: usize,
}

impl SegmentWriteTxn {
    /// Create a pending transaction for a known record count.
    pub fn begin(generation: u64, records: usize) -> Option<Self> {
        (records > 0).then_some(Self { generation, records })
    }

    /// Return the stable output name that can be committed atomically.
    pub fn commit_name(&self) -> String {
        format!("segment-{:04}.tsv", self.generation)
    }
}
