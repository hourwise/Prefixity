//! Deterministic token estimation.
//!
//! The preferred source of truth for a block's token count is the recorder-
//! supplied `token_count` field. When that is absent, a documented heuristic
//! is applied to `content` if present. If neither is available, estimation
//! returns `None` and analysis reports a warning rather than guessing.

use crate::model::ContextBlock;

/// Heuristic: approximate one token per this many characters.
///
/// This is a crude, documented stand-in (roughly 4 chars/token for English-
/// heavy text). It is used **only** when `token_count` is absent, and every
/// consumer that uses it must surface that fact so figures are not mistaken
/// for provider-reported counts.
pub const TOKENS_PER_CHAR_DIVISOR: u64 = 4;

/// Estimate the token count of a block, if possible.
///
/// Precedence:
/// 1. `block.token_count` if present;
/// 2. `ceil(chars / TOKENS_PER_CHAR_DIVISOR)` if `content` is present;
/// 3. `None` otherwise.
pub fn block_token_estimate(block: &ContextBlock) -> Option<u64> {
    if let Some(count) = block.token_count {
        return Some(count);
    }
    block.content.as_deref().map(|content| {
        let chars = content.chars().count() as u64;
        if chars == 0 {
            0
        } else {
            chars.div_ceil(TOKENS_PER_CHAR_DIVISOR)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ContextBlock;
    use std::collections::BTreeMap;

    fn block(id: &str, token_count: Option<u64>, content: Option<&str>) -> ContextBlock {
        ContextBlock {
            id: id.to_string(),
            source: "test".to_string(),
            position: 0,
            content_hash: "0".repeat(64),
            token_count,
            byte_count: 0,
            timestamp: None,
            content: content.map(str::to_string),
            semantic_zone: None,
            structural_path: None,
            role: None,
            sensitivity: None,
            dependencies: Vec::new(),
            lifetime: None,
            optional: false,
            required: false,
            stale: false,
            provenance: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn prefers_recorded_token_count() {
        let b = block("a", Some(123), Some("hello world"));
        assert_eq!(block_token_estimate(&b), Some(123));
    }

    #[test]
    fn falls_back_to_heuristic_from_content() {
        let b = block("a", None, Some("abcdefgh")); // 8 chars -> 2 tokens
        assert_eq!(block_token_estimate(&b), Some(2));
        let empty = block("b", None, Some(""));
        assert_eq!(block_token_estimate(&empty), Some(0));
    }

    #[test]
    fn returns_none_without_any_signal() {
        let b = block("a", None, None);
        assert_eq!(block_token_estimate(&b), None);
    }
}
