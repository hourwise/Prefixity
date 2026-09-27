use crate::error::Result;
use crate::query::LookupQuery;
use crate::record::LookupOutcome;
use crate::store::SegmentStore;
use std::path::{Path, PathBuf};

/// Immutable mapping from a key range to one segment file.
#[derive(Debug, Clone)]
pub(crate) struct IndexSnapshot {
    root: PathBuf,
    first_key: String,
    last_key: String,
    segment_name: String,
}

impl IndexSnapshot {
    pub(crate) fn new(
        root: PathBuf,
        first_key: impl Into<String>,
        last_key: impl Into<String>,
        segment_name: impl Into<String>,
    ) -> Self {
        Self {
            root,
            first_key: first_key.into(),
            last_key: last_key.into(),
            segment_name: segment_name.into(),
        }
    }

    /// Resolve a query to the segment and use the store to read its records.
    pub(crate) fn lookup(
        &self,
        store: &SegmentStore,
        query: &LookupQuery,
    ) -> Result<LookupOutcome> {
        let segment_path = self.segment_path();
        let records = store.read_segment(&segment_path)?;
        let record = if query.key() < self.first_key.as_str()
            || query.key() > self.last_key.as_str()
        {
            None
        } else {
            records.iter().find(|record| record.key == query.key()).cloned()
        };
        Ok(LookupOutcome {
            segment_name: self.segment_name.clone(),
            record,
        })
    }

    fn segment_path(&self) -> PathBuf {
        self.root.join(&self.segment_name)
    }

    #[allow(dead_code)]
    fn index_path(root: &Path) -> PathBuf {
        root.join("index.meta")
    }
}
