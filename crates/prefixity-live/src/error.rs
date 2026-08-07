//! Structured errors for `prefixity-live`.
//!
//! Errors never contain credentials. Network errors carry a sanitized
//! message only; HTTP errors carry a status code only (provider response
//! bodies are never embedded, because a body could in principle echo a
//! credential).

use std::path::PathBuf;

/// The unified error type for the live validation harness.
#[derive(Debug, thiserror::Error)]
pub enum LiveError {
    /// The required environment variable is not set (or empty).
    #[error("missing credential: environment variable {env_var} is not set")]
    MissingCredential {
        /// The name of the missing environment variable.
        env_var: &'static str,
    },

    /// A transport-level failure (connect, TLS, etc.). Sanitized message.
    #[error("network error: {message}")]
    Network {
        /// Sanitized description, never containing credentials.
        message: String,
    },

    /// The provider returned a non-success HTTP status. Body is never
    /// included (see module docs).
    #[error("HTTP error: provider returned status {status}")]
    HttpStatus {
        /// The HTTP status code.
        status: u16,
    },

    /// The request timed out (no automatic retry follows).
    #[error("request timed out")]
    Timeout,

    /// The provider response could not be parsed.
    #[error("invalid provider response: {message}")]
    InvalidResponse {
        /// Description of the parse failure.
        message: String,
    },

    /// The provider's usage fields do not fit the registered normalizer.
    #[error("provider usage schema mismatch: {message}")]
    SchemaMismatch {
        /// Description of the mismatch.
        message: String,
    },

    /// An experiment guard refused to proceed (request limit, token ceiling).
    #[error("experiment guard refused: {message}")]
    Guard {
        /// Why the guard refused.
        message: String,
    },

    /// Invalid user-supplied argument.
    #[error("invalid argument: {message}")]
    InvalidArgument {
        /// Description of the problem.
        message: String,
    },

    /// Artifact writing failure.
    #[error("artifact I/O error: {path}: {message}")]
    Artifact {
        /// The path that could not be written.
        path: PathBuf,
        /// Underlying description.
        message: String,
    },
}

impl LiveError {
    /// Build a [`LiveError::InvalidArgument`].
    pub fn argument(message: impl Into<String>) -> Self {
        LiveError::InvalidArgument {
            message: message.into(),
        }
    }

    /// Build a [`LiveError::Guard`].
    pub fn guard(message: impl Into<String>) -> Self {
        LiveError::Guard {
            message: message.into(),
        }
    }
}
