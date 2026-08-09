//! Structural / wire identity for context blocks.
//!
//! `content_hash` identifies block *content* but is insufficient for provider
//! prefix comparison: two blocks can contain identical text yet occupy
//! different semantic positions (system vs user vs tool definition vs tool
//! result). This module adds:
//!
//! * [`SemanticZone`] — a coarse provider-independent zone;
//! * [`structural_fingerprint`] — a deterministic fingerprint derived from
//!   the block's structural identity and content hash.
//!
//! The fingerprint is a **Prefixity structural fingerprint**, not a guarantee
//! of a provider's hidden tokenizer or serializer. It is used by [`crate::compare`]
//! for prefix equality instead of content hash alone; `content_hash` remains
//! the content-level identity.
//!
//! Without any structural identity on a block, the fingerprint falls back to
//! the content hash (so traces recorded without structural metadata still
//! compare content-identically).

use crate::hash::sha256_hex;
use crate::model::ContextBlock;

/// Coarse provider-independent semantic zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticZone {
    /// Tool definitions / schemas.
    Tools,
    /// System instructions and policies.
    System,
    /// Chronologically ordered conversation messages (including user,
    /// assistant, and tool-result turns).
    Messages,
    /// Anything else, including unzoned blocks.
    Other,
}

impl SemanticZone {
    /// Map a zone string to a [`SemanticZone`]. Unknown values map to
    /// [`SemanticZone::Other`].
    pub fn from_name(zone: &str) -> SemanticZone {
        match zone {
            "tools" => SemanticZone::Tools,
            "system" => SemanticZone::System,
            "messages" => SemanticZone::Messages,
            _ => SemanticZone::Other,
        }
    }

    /// Canonical string form of the zone.
    pub fn as_str(&self) -> &'static str {
        match self {
            SemanticZone::Tools => "tools",
            SemanticZone::System => "system",
            SemanticZone::Messages => "messages",
            SemanticZone::Other => "other",
        }
    }

    /// Whether reordering blocks within this zone is ever considered safe.
    /// Chronological zones (`messages`) must never be reordered.
    pub fn preserves_chronology(&self) -> bool {
        matches!(self, SemanticZone::Messages)
    }
}

/// The zone a block belongs to. Explicit `semantic_zone` wins; absent blocks
/// are treated as [`SemanticZone::Other`].
pub fn zone_of(block: &ContextBlock) -> SemanticZone {
    match &block.semantic_zone {
        Some(zone) => SemanticZone::from_name(zone),
        None => SemanticZone::Other,
    }
}

/// Compute the structural fingerprint of a block.
///
/// The fingerprint is the SHA-256 of the canonical concatenation of zone,
/// role, structural path and content hash. Blocks without any structural
/// identity fall back to the content hash itself.
pub fn structural_fingerprint(block: &ContextBlock) -> String {
    let zone = block.semantic_zone.as_deref().unwrap_or("");
    let role = block.role.as_deref().unwrap_or("");
    let path = block.structural_path.as_deref().unwrap_or("");
    if zone.is_empty() && role.is_empty() && path.is_empty() {
        return block.content_hash.clone();
    }
    sha256_hex(format!("{zone}|{role}|{path}|{}", block.content_hash).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash_content;
    use std::collections::BTreeMap;

    fn block(
        id: &str,
        content: &str,
        zone: Option<&str>,
        role: Option<&str>,
        path: Option<&str>,
    ) -> ContextBlock {
        ContextBlock {
            id: id.to_string(),
            source: "test".to_string(),
            position: 0,
            content_hash: hash_content(content),
            token_count: Some(1),
            byte_count: content.len() as u64,
            timestamp: None,
            content: Some(content.to_string()),
            semantic_zone: zone.map(str::to_string),
            structural_path: path.map(str::to_string),
            role: role.map(str::to_string),
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
    fn same_text_same_structure_matches() {
        let a = block(
            "a",
            "identical text",
            Some("system"),
            Some("system"),
            Some("system[0]"),
        );
        let b = block(
            "b",
            "identical text",
            Some("system"),
            Some("system"),
            Some("system[0]"),
        );
        assert_eq!(structural_fingerprint(&a), structural_fingerprint(&b));
    }

    #[test]
    fn same_text_different_role_differs() {
        let a = block(
            "a",
            "identical text",
            Some("messages"),
            Some("user"),
            Some("messages[0]"),
        );
        let b = block(
            "b",
            "identical text",
            Some("messages"),
            Some("assistant"),
            Some("messages[1]"),
        );
        assert_ne!(structural_fingerprint(&a), structural_fingerprint(&b));
    }

    #[test]
    fn same_text_different_zone_differs() {
        let a = block(
            "a",
            "identical text",
            Some("system"),
            Some("system"),
            Some("system[0]"),
        );
        let b = block(
            "b",
            "identical text",
            Some("tools"),
            Some("system"),
            Some("tools[0]"),
        );
        assert_ne!(structural_fingerprint(&a), structural_fingerprint(&b));
    }

    #[test]
    fn same_text_different_path_differs() {
        let a = block(
            "a",
            "identical text",
            Some("messages"),
            Some("user"),
            Some("messages[0]"),
        );
        let b = block(
            "b",
            "identical text",
            Some("messages"),
            Some("user"),
            Some("messages[1]"),
        );
        assert_ne!(structural_fingerprint(&a), structural_fingerprint(&b));
    }

    #[test]
    fn no_structural_identity_falls_back_to_content_hash() {
        let a = block("a", "plain", None, None, None);
        assert_eq!(structural_fingerprint(&a), a.content_hash);
    }

    #[test]
    fn zone_mapping_and_chronology() {
        assert_eq!(SemanticZone::from_name("tools"), SemanticZone::Tools);
        assert_eq!(SemanticZone::from_name("messages"), SemanticZone::Messages);
        assert_eq!(SemanticZone::from_name("bogus"), SemanticZone::Other);
        assert!(SemanticZone::Messages.preserves_chronology());
        assert!(!SemanticZone::Tools.preserves_chronology());
    }
}
