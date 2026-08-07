//! Safe credential acquisition.
//!
//! Credentials are read **only** from environment variables, never from CLI
//! arguments or files. They are held in a [`Credentials`] value whose
//! `Debug` output is redacted, and they are never serialized, logged, or
//! included in artifacts or errors.

use crate::error::LiveError;
use std::fmt;

/// The environment variable names supported in Phase 0B.
pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
pub const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
pub const DEEPSEEK_API_KEY: &str = "DEEPSEEK_API_KEY";

/// A credential read from the environment. `Debug` is redacted.
pub struct Credentials {
    key: String,
    env_var: &'static str,
}

impl Credentials {
    /// Read a credential from the named environment variable.
    pub fn from_env(env_var: &'static str) -> Result<Credentials, LiveError> {
        match std::env::var(env_var) {
            Ok(value) if !value.trim().is_empty() => Ok(Credentials {
                key: value,
                env_var,
            }),
            _ => Err(LiveError::MissingCredential { env_var }),
        }
    }

    /// The environment variable this credential came from.
    pub fn env_var(&self) -> &'static str {
        self.env_var
    }

    /// The raw credential value (used only to build the Authorization header
    /// immediately before the request; never persisted or logged).
    pub(crate) fn raw(&self) -> &str {
        &self.key
    }
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Credentials {{ env_var: {}, key: [REDACTED] }}",
            self.env_var
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_debug_is_redacted() {
        std::env::set_var("PREFIXITY_TEST_KEY_1", "sk-super-secret-value");
        let creds = Credentials::from_env("PREFIXITY_TEST_KEY_1").unwrap();
        let debug = format!("{creds:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("sk-super-secret-value"));
        assert!(!debug.contains("super-secret"));
        std::env::remove_var("PREFIXITY_TEST_KEY_1");
    }

    #[test]
    fn missing_credential_is_reported() {
        std::env::remove_var("PREFIXITY_TEST_KEY_2");
        let err = Credentials::from_env("PREFIXITY_TEST_KEY_2").unwrap_err();
        assert!(matches!(err, LiveError::MissingCredential { .. }));
        assert!(err.to_string().contains("PREFIXITY_TEST_KEY_2"));
    }

    #[test]
    fn empty_credential_is_missing() {
        std::env::set_var("PREFIXITY_TEST_KEY_3", "   ");
        let err = Credentials::from_env("PREFIXITY_TEST_KEY_3").unwrap_err();
        assert!(matches!(err, LiveError::MissingCredential { .. }));
        std::env::remove_var("PREFIXITY_TEST_KEY_3");
    }
}
