use std::fmt::{Display, Formatter};

/// Errors returned while validating a query or reading a pinned segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupError {
    EmptyKey,
    InvalidKey,
    MissingSegment(String),
    CorruptSegment(String),
}

impl Display for LookupError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("lookup key is empty"),
            Self::InvalidKey => formatter.write_str("lookup key contains a delimiter"),
            Self::MissingSegment(path) => write!(formatter, "segment is missing: {path}"),
            Self::CorruptSegment(path) => write!(formatter, "segment is corrupt: {path}"),
        }
    }
}

impl std::error::Error for LookupError {}

/// Result type used by the library's local I/O path.
pub type Result<T> = std::result::Result<T, LookupError>;
