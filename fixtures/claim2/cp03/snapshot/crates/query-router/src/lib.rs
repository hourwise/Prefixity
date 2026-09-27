//! Protocol-aware selection of a local snapshot query path.

use cache_engine::SnapshotCache;
use snapshot_index::IndexManifest;
use wire_schema::RecordEnvelope;

/// Frozen query-route compatibility decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutePlan {
    pub reader_revision: u16,
}

impl RoutePlan {
    /// Construct a route for one negotiated wire revision.
    pub fn for_protocol(reader_revision: u16) -> Self {
        Self { reader_revision }
    }

    /// Accept only a record whose wire revision is supported by the route.
    pub fn accepts(&self, wire_revision: u16) -> bool {
        wire_revision <= self.reader_revision
            && RecordEnvelope::is_supported_by(self.reader_revision)
    }

    /// Bind one index manifest to the cache selected by this route.
    pub fn bind(&self, cache: &mut SnapshotCache, manifest: IndexManifest) {
        cache.bind_manifest(manifest);
    }
}
