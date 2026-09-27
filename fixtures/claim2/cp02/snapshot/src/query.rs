use crate::error::{LookupError, Result};

/// A validated key lookup request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupQuery {
    key: String,
}

impl LookupQuery {
    /// Reject empty keys and the tab delimiter used by the segment format.
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        if key.is_empty() {
            return Err(LookupError::EmptyKey);
        }
        if key.contains('\t') || key.contains('\n') {
            return Err(LookupError::InvalidKey);
        }
        Ok(Self { key })
    }

    /// Return the normalized key exactly as supplied after validation.
    pub fn key(&self) -> &str {
        &self.key
    }
}
