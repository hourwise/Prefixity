/// Identity for an immutable local repository snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotId {
    pub revision: String,
    pub manifest_fingerprint: u64,
}

impl SnapshotId {
    /// Construct a snapshot identity from the pinned revision and manifest.
    pub fn from_manifest(revision: impl Into<String>, manifest: &[u8]) -> Self {
        Self {
            revision: revision.into(),
            manifest_fingerprint: crate::checksum::fingerprint(manifest),
        }
    }

    /// Compare both revision text and manifest fingerprint.
    pub fn matches(&self, other: &Self) -> bool {
        self == other
    }
}
