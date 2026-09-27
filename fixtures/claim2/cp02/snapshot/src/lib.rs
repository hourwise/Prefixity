//! Read-only lookups over an immutable, locally indexed segment set.

mod cache;
mod error;
mod index;
mod query;
mod record;
mod service;
mod store;

pub mod bloom;
pub mod builder;
pub mod checksum;
pub mod compaction;
pub mod filter;
pub mod manifest;
pub mod metrics;
pub mod pagination;
pub mod routing;
pub mod snapshot;
pub mod transaction;
pub mod version;

pub use error::{LookupError, Result};
pub use query::LookupQuery;
pub use record::{LookupOutcome, LedgerRecord};
pub use service::Library;

/// Build a library from a pinned index and its segment directory.
pub fn open(index_path: impl Into<std::path::PathBuf>) -> Result<Library> {
    Library::open(index_path)
}
