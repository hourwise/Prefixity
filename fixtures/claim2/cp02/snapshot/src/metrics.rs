/// Local counters for the read path, with no timing or provider data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReadMetrics {
    pub queries: u64,
    pub segment_loads: u64,
    pub cache_hits: u64,
}

impl ReadMetrics {
    /// Record one successful cache lookup.
    pub fn record_cache_hit(&mut self) {
        self.queries += 1;
        self.cache_hits += 1;
    }

    /// Record a physical segment load for the current query.
    pub fn record_segment_load(&mut self) {
        self.queries += 1;
        self.segment_loads += 1;
    }
}
