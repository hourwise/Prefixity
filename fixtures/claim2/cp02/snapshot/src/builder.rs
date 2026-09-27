use crate::record::LedgerRecord;

/// Collects records and emits a stable sorted segment representation.
#[derive(Debug, Default)]
pub struct IndexBuilder {
    records: Vec<LedgerRecord>,
}

impl IndexBuilder {
    /// Add one record to the pending segment.
    pub fn add_record(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.records.push(LedgerRecord { key: key.into(), value: value.into() });
    }

    /// Sort records by key and serialize one tab-delimited segment body.
    pub fn finish(mut self) -> Vec<u8> {
        self.records.sort_by(|left, right| left.key.cmp(&right.key));
        self.records
            .iter()
            .map(|record| format!("{}\t{}\n", record.key, record.value))
            .collect::<String>()
            .into_bytes()
    }
}
