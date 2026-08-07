//! Structured error types for `prefixity-core`.
//!
//! All fallible public functions return [`PrefixityError`]. Functions that
//! analyse user-supplied traces never panic on malformed input; they return
//! a descriptive error instead.

use std::path::PathBuf;

/// The unified error type for Prefixity analysis and simulation.
#[derive(Debug, thiserror::Error)]
pub enum PrefixityError {
    /// Filesystem error while reading a file.
    #[error("I/O error reading {path}: {source}")]
    Io {
        /// The path that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// The file exists but is larger than the configured safety limit.
    #[error("file {path} exceeds maximum supported size ({limit} bytes)")]
    FileTooLarge {
        /// The path of the oversized file.
        path: PathBuf,
        /// The configured limit in bytes.
        limit: u64,
    },

    /// The file could not be parsed as the expected JSON structure.
    #[error("invalid JSON in {path}: {source}")]
    InvalidJson {
        /// The path that failed to parse.
        path: PathBuf,
        /// The underlying JSON error.
        source: serde_json::Error,
    },

    /// The trace failed structural validation.
    #[error("trace validation failed for {path}: {message}")]
    Validation {
        /// The path of the offending file (or `<in-memory>`).
        path: PathBuf,
        /// A human-readable description of the problem.
        message: String,
    },

    /// The trace uses a format version this build cannot read.
    #[error("unsupported trace format version {found} (supported: {supported})")]
    UnsupportedFormatVersion {
        /// The version found in the file.
        found: u32,
        /// The version supported by this build.
        supported: u32,
    },

    /// The requested policy name is unknown.
    #[error("unknown policy '{name}'. available policies: baseline, stable-prefix, defer-volatile, prune-stale-tool-output, combined")]
    PolicyNotFound {
        /// The unknown policy name.
        name: String,
    },

    /// The requested feature is reserved for a later phase.
    #[error("{what}")]
    Reserved {
        /// A human-readable description of the reserved feature.
        what: String,
    },

    /// The supplied cost profile failed validation.
    #[error("invalid cost profile {path}: {message}")]
    InvalidCostProfile {
        /// The path of the offending profile.
        path: PathBuf,
        /// A human-readable description of the problem.
        message: String,
    },

    /// The comparison of two traces could not be performed.
    #[error("comparison error: {message}")]
    Comparison {
        /// A human-readable description of the problem.
        message: String,
    },
}

impl PrefixityError {
    /// Build a [`PrefixityError::Validation`] value.
    pub fn validation(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        PrefixityError::Validation {
            path: path.into(),
            message: message.into(),
        }
    }
}
