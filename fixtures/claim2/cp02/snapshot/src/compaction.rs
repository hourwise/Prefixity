/// A deterministic set of adjacent segments eligible for compaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionPlan {
    pub input_segments: Vec<String>,
    pub output_segment: String,
}

/// Plan a merge when at least two immutable segments are supplied.
pub fn plan_compaction(segments: &[String], generation: u64) -> Option<CompactionPlan> {
    if segments.len() < 2 {
        return None;
    }
    Some(CompactionPlan {
        input_segments: segments.to_vec(),
        output_segment: format!("segment-{generation:04}.tsv"),
    })
}
