//! Local immutable snapshot cache with an explicit protocol reader floor.

use std::collections::BTreeMap;
use snapshot_index::IndexManifest;
use wire_schema::RecordEnvelope;

/// The minimum protocol revision accepted by this cache engine.
pub const MIN_WIRE_REVISION: u16 = 3;

/// A bounded key-to-envelope cache for one immutable project snapshot.
#[derive(Debug, Default)]
pub struct SnapshotCache {
    entries: BTreeMap<String, RecordEnvelope>,
    manifest: Option<IndexManifest>,
}

impl SnapshotCache {
    /// Insert an envelope only when its wire revision is supported.
    pub fn insert(&mut self, envelope: RecordEnvelope) -> Result<(), &'static str> {
        if envelope.wire_revision < MIN_WIRE_REVISION {
            return Err("unsupported wire revision");
        }
        self.entries.insert(envelope.key.clone(), envelope);
        Ok(())
    }

    /// Read one envelope by its stable key.
    pub fn get(&self, key: &str) -> Option<&RecordEnvelope> {
        self.entries.get(key)
    }

    /// Bind the cache to the immutable index revision it serves.
    pub fn bind_manifest(&mut self, manifest: IndexManifest) {
        self.manifest = Some(manifest);
    }

    /// Return the bound snapshot generation when one has been installed.
    pub fn generation(&self) -> Option<u64> {
        self.manifest.as_ref().map(|manifest| manifest.generation)
    }
}
