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
    ///
    /// The value is validated **locally, before any request is built**: it
    /// must be non-empty, have no leading/trailing whitespace, and contain
    /// no ASCII/control characters unsuitable for an HTTP header value. A
    /// bad value is rejected here (never silently trimmed or altered), so it
    /// cannot later fail inside HTTP-header construction with a generic
    /// transport error. No provider-specific prefix (e.g. `sk-`) is
    /// enforced: credential formats may change and differ by provider.
    pub fn from_env(env_var: &'static str) -> Result<Credentials, LiveError> {
        let value = std::env::var(env_var).map_err(|_| LiveError::MissingCredential { env_var })?;
        if value.trim().is_empty() {
            return Err(LiveError::MissingCredential { env_var });
        }
        if value != value.trim() {
            return Err(LiveError::InvalidCredential {
                env_var,
                reason: "leading/trailing whitespace is not allowed",
            });
        }
        if contains_control_characters(&value) {
            return Err(LiveError::InvalidCredential {
                env_var,
                reason: "control characters are not allowed",
            });
        }
        Ok(Credentials {
            key: value,
            env_var,
        })
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

/// Whether `value` contains an ASCII/control character unsuitable for an
/// HTTP header value: C0 controls (0x00–0x1F) and DEL (0x7F). Header values
/// must be visible characters; a copied credential containing a newline or
/// other control character must be rejected here rather than failing later
/// inside header construction.
fn contains_control_characters(value: &str) -> bool {
    value.chars().any(|c| {
        let code = c as u32;
        code < 0x20 || code == 0x7F
    })
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

    #[test]
    fn valid_credential_is_accepted_unchanged() {
        std::env::set_var("PREFIXITY_TEST_KEY_4", "sk-abc123");
        let creds = Credentials::from_env("PREFIXITY_TEST_KEY_4").unwrap();
        assert_eq!(creds.raw(), "sk-abc123");
        assert_eq!(creds.env_var(), "PREFIXITY_TEST_KEY_4");
        std::env::remove_var("PREFIXITY_TEST_KEY_4");
    }

    #[test]
    fn leading_whitespace_credential_is_rejected() {
        std::env::set_var("PREFIXITY_TEST_KEY_5", "  sk-abc123");
        let err = Credentials::from_env("PREFIXITY_TEST_KEY_5").unwrap_err();
        assert!(matches!(
            err,
            LiveError::InvalidCredential {
                reason: "leading/trailing whitespace is not allowed",
                ..
            }
        ));
        // The error names the variable and reason, never the value.
        let text = err.to_string();
        assert!(text.contains("PREFIXITY_TEST_KEY_5"));
        assert!(!text.contains("sk-abc123"));
        std::env::remove_var("PREFIXITY_TEST_KEY_5");
    }

    #[test]
    fn trailing_whitespace_and_newline_credential_is_rejected() {
        for bad in ["sk-abc123  ", "sk-abc123\n", "sk-abc123\r\n"] {
            std::env::set_var("PREFIXITY_TEST_KEY_6", bad);
            let err = Credentials::from_env("PREFIXITY_TEST_KEY_6").unwrap_err();
            assert!(
                matches!(
                    err,
                    LiveError::InvalidCredential {
                        reason: "leading/trailing whitespace is not allowed",
                        ..
                    }
                ),
                "value {bad:?} must be rejected for trailing whitespace"
            );
            assert!(!err.to_string().contains("sk-abc123"));
            std::env::remove_var("PREFIXITY_TEST_KEY_6");
        }
    }

    #[test]
    fn embedded_control_character_credential_is_rejected() {
        for bad in ["sk-ab\tc123", "sk-ab\u{0001}c123", "sk-abc\x7f"] {
            std::env::set_var("PREFIXITY_TEST_KEY_7", bad);
            let err = Credentials::from_env("PREFIXITY_TEST_KEY_7").unwrap_err();
            assert!(
                matches!(
                    err,
                    LiveError::InvalidCredential {
                        reason: "control characters are not allowed",
                        ..
                    }
                ),
                "value {bad:?} must be rejected for control characters"
            );
            assert!(!err.to_string().contains("c123"));
            std::env::remove_var("PREFIXITY_TEST_KEY_7");
        }
    }

    #[test]
    fn credential_errors_and_debug_never_contain_the_value() {
        // A value that trips validation must not appear in the error, and the
        // accepted credential's Debug output must remain redacted.
        let secret = "sk-very-secret-material-987654321";
        std::env::set_var("PREFIXITY_TEST_KEY_8", format!(" {secret}"));
        let err = Credentials::from_env("PREFIXITY_TEST_KEY_8").unwrap_err();
        let err_text = format!("{err:?}") + &err.to_string();
        assert!(!err_text.contains("very-secret"));
        assert!(!err_text.contains("987654321"));
        std::env::remove_var("PREFIXITY_TEST_KEY_8");

        std::env::set_var("PREFIXITY_TEST_KEY_9", secret);
        let creds = Credentials::from_env("PREFIXITY_TEST_KEY_9").unwrap();
        let debug = format!("{creds:?}");
        assert!(!debug.contains("very-secret"));
        assert!(!debug.contains("987654321"));
        assert!(debug.contains("[REDACTED]"));
        std::env::remove_var("PREFIXITY_TEST_KEY_9");
    }
}
