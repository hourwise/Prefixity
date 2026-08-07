//! Bounded input limits used to keep Phase 0 handling safe against
//! pathologically large or deeply nested trace files.
//!
//! These limits are deliberately generous for real workloads but finite, so
//! that a single malicious trace cannot exhaust memory or hang analysis.

/// Maximum size of any trace or profile file read by the CLI, in bytes.
pub const MAX_TRACE_FILE_BYTES: u64 = 256 * 1024 * 1024; // 256 MiB

/// Maximum number of context blocks in a single trace.
pub const MAX_BLOCKS: usize = 100_000;

/// Maximum length of a single block's content, in bytes.
pub const MAX_BLOCK_CONTENT_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

/// Maximum length of a block ID, in bytes.
pub const MAX_BLOCK_ID_BYTES: usize = 1024;

/// Maximum number of entries in a trace/block `metadata` map.
pub const MAX_METADATA_ENTRIES: usize = 10_000;

/// Maximum number of entries in a block's `dependencies` list.
pub const MAX_DEPENDENCIES: usize = 10_000;
