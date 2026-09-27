use crate::error::{LookupError, Result};
use crate::index::IndexSnapshot;
use crate::query::LookupQuery;
use crate::record::LedgerRecord;
use crate::store::SegmentStore;
use std::path::{Path, PathBuf};

/// Public lookup facade used by both the library API and CLI.
#[derive(Debug)]
pub struct Library {
    snapshot: IndexSnapshot,
    store: SegmentStore,
}

impl Library {
    /// Open the directory containing the immutable index and segment files.
    pub fn open(index_path: impl Into<PathBuf>) -> Result<Self> {
        let index_path = index_path.into();
        let root = index_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let snapshot = IndexSnapshot::new(root, "a", "z", "segment-0001.tsv");
        Ok(Self {
            snapshot,
            store: SegmentStore::default(),
        })
    }

    /// Find one record by key through the index, store, and cache layers.
    pub fn find_record(&self, query: &LookupQuery) -> Result<Option<LedgerRecord>> {
        let outcome = self.snapshot.lookup(&self.store, query)?;
        Ok(outcome.record)
    }

    /// Return an explicit validation error for a query that cannot be parsed.
    pub fn find_raw(&self, key: impl Into<String>) -> Result<Option<LedgerRecord>> {
        let query = LookupQuery::new(key)?;
        self.find_record(&query)
    }
}
