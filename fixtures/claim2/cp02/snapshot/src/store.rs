use crate::cache::IndexCache;
use crate::error::{LookupError, Result};
use crate::record::LedgerRecord;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Reads tab-delimited segment files through the shared immutable cache.
#[derive(Debug, Default)]
pub(crate) struct SegmentStore {
    cache: IndexCache,
}

impl SegmentStore {
    /// Decode a segment, reusing the cached representation when available.
    pub(crate) fn read_segment(&self, path: &Path) -> Result<Arc<Vec<LedgerRecord>>> {
        let owned_path: PathBuf = path.to_path_buf();
        self.cache.get_or_load(owned_path.clone(), || {
            let bytes = fs::read(&owned_path)
                .map_err(|_| LookupError::MissingSegment(owned_path.display().to_string()))?;
            decode_segment(&owned_path, &bytes)
        })
    }
}

fn decode_segment(path: &Path, bytes: &[u8]) -> Result<Vec<LedgerRecord>> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| LookupError::CorruptSegment(path.display().to_string()))?;
    text.lines()
        .map(|line| {
            let (key, value) = line
                .split_once('\t')
                .ok_or_else(|| LookupError::CorruptSegment(path.display().to_string()))?;
            Ok(LedgerRecord {
                key: key.to_owned(),
                value: value.to_owned(),
            })
        })
        .collect()
}
