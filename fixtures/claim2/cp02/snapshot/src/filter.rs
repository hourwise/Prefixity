/// A bounded prefix filter applied before a segment read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPredicate {
    prefix: String,
}

impl RecordPredicate {
    /// Build a prefix predicate for one local query.
    pub fn starts_with(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }

    /// Test whether a key starts with the configured prefix.
    pub fn matches(&self, key: &str) -> bool {
        key.starts_with(&self.prefix)
    }
}
