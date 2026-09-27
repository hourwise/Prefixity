/// One decoded record from a segment file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRecord {
    pub key: String,
    pub value: String,
}

/// The local index result before the service layer converts it to an option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupOutcome {
    pub segment_name: String,
    pub record: Option<LedgerRecord>,
}
