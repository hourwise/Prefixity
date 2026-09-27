use crate::record::LedgerRecord;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Small process-local cache for immutable segment records.
#[derive(Debug, Default)]
pub(crate) struct IndexCache {
    entries: Mutex<BTreeMap<PathBuf, Arc<Vec<LedgerRecord>>>>,
}

impl IndexCache {
    /// Load one segment once and return the shared decoded record list.
    pub(crate) fn get_or_load<F>(
        &self,
        path: PathBuf,
        load: F,
    ) -> crate::error::Result<Arc<Vec<LedgerRecord>>>
    where
        F: FnOnce() -> crate::error::Result<Vec<LedgerRecord>>,
    {
        if let Some(records) = self.entries.lock().expect("cache mutex").get(&path).cloned() {
            return Ok(records);
        }
        let records = Arc::new(load()?);
        self.entries
            .lock()
            .expect("cache mutex")
            .insert(path, records.clone());
        Ok(records)
    }

    /// Remove a single immutable segment from the process-local cache.
    pub(crate) fn evict(&self, path: &std::path::Path) {
        self.entries.lock().expect("cache mutex").remove(path);
    }
}
