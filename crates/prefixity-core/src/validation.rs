//! Structural validation of [`RequestTrace`] values.
//!
//! Validation enforces the invariants the analysis and simulation functions
//! rely on:
//!
//! * the trace format version is supported;
//! * identifiers and block IDs are present and bounded;
//! * blocks are contiguous and correctly positioned;
//! * block IDs are unique;
//! * content hashes are well-formed and (when content is present) correct;
//! * `byte_count` matches the actual UTF-8 length of `content` when present;
//! * input size limits are respected.
//!
//! Hard problems return [`PrefixityError::Validation`] (or
//! [`PrefixityError::UnsupportedFormatVersion`]); soft problems are collected
//! as warnings in [`ValidationReport`].
//!
//! Raw provider usage is intentionally opaque here: field consistency is the
//! job of the normalizers (see [`crate::usage`]).

use crate::error::PrefixityError;
use crate::hash;
use crate::limits;
use crate::model::{RequestTrace, TRACE_FORMAT_VERSION};
use std::collections::HashSet;
use std::path::Path;

/// The result of a successful validation run. Errors are returned as
/// [`PrefixityError`]; only non-fatal findings appear here.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ValidationReport {
    /// Non-fatal findings (e.g. provider-reported usage inconsistencies or
    /// heuristic token estimates used).
    pub warnings: Vec<String>,
}

/// Validate `trace` structurally.
///
/// `path` is used only for error messages; pass `None` when validating an
/// in-memory value.
pub fn validate_trace(
    trace: &RequestTrace,
    path: Option<&Path>,
) -> Result<ValidationReport, PrefixityError> {
    let path = path.unwrap_or_else(|| Path::new("<in-memory>"));
    let warnings = Vec::new();

    if trace.format_version != TRACE_FORMAT_VERSION {
        return Err(PrefixityError::UnsupportedFormatVersion {
            found: trace.format_version,
            supported: TRACE_FORMAT_VERSION,
        });
    }
    if trace.request_id.trim().is_empty() {
        return Err(PrefixityError::validation(
            path,
            "request_id must not be empty",
        ));
    }
    if trace.provider.trim().is_empty() {
        return Err(PrefixityError::validation(
            path,
            "provider must not be empty",
        ));
    }
    if trace.model.trim().is_empty() {
        return Err(PrefixityError::validation(path, "model must not be empty"));
    }
    if trace.blocks.is_empty() {
        return Err(PrefixityError::validation(
            path,
            "trace must contain at least one context block",
        ));
    }
    if trace.blocks.len() > limits::MAX_BLOCKS {
        return Err(PrefixityError::validation(
            path,
            format!("trace exceeds maximum block count ({})", limits::MAX_BLOCKS),
        ));
    }
    if trace.metadata.len() > limits::MAX_METADATA_ENTRIES {
        return Err(PrefixityError::validation(
            path,
            format!(
                "trace metadata exceeds maximum entry count ({})",
                limits::MAX_METADATA_ENTRIES
            ),
        ));
    }

    let mut seen_ids: HashSet<&str> = HashSet::new();
    for (index, block) in trace.blocks.iter().enumerate() {
        let block_path = format!("block '{}'", block.id);
        if block.position != index {
            return Err(PrefixityError::validation(
                path,
                format!(
                    "{block_path} has position {}, expected {index}",
                    block.position
                ),
            ));
        }
        if block.id.trim().is_empty() {
            return Err(PrefixityError::validation(
                path,
                format!("block[{index}] has an empty id"),
            ));
        }
        if block.id.len() > limits::MAX_BLOCK_ID_BYTES {
            return Err(PrefixityError::validation(
                path,
                format!(
                    "{block_path} id exceeds maximum length ({})",
                    limits::MAX_BLOCK_ID_BYTES
                ),
            ));
        }
        if !seen_ids.insert(block.id.as_str()) {
            return Err(PrefixityError::validation(
                path,
                format!("duplicate block id '{}'", block.id),
            ));
        }
        if !hash::is_valid_sha256_hex(&block.content_hash) {
            return Err(PrefixityError::validation(
                path,
                format!("{block_path} content_hash is not a 64-char hex SHA-256 digest"),
            ));
        }
        if let Some(content) = &block.content {
            if content.len() > limits::MAX_BLOCK_CONTENT_BYTES {
                return Err(PrefixityError::validation(
                    path,
                    format!(
                        "{block_path} content exceeds maximum size ({})",
                        limits::MAX_BLOCK_CONTENT_BYTES
                    ),
                ));
            }
            let computed = hash::hash_content(content);
            if computed != block.content_hash {
                return Err(PrefixityError::validation(
                    path,
                    format!("{block_path} content_hash does not match its content"),
                ));
            }
            let actual_bytes = content.len() as u64;
            if block.byte_count != actual_bytes {
                return Err(PrefixityError::validation(
                    path,
                    format!(
                        "{block_path} byte_count ({}) does not match the UTF-8 length of its content ({actual_bytes})",
                        block.byte_count
                    ),
                ));
            }
        }
        if block.metadata.len() > limits::MAX_METADATA_ENTRIES {
            return Err(PrefixityError::validation(
                path,
                format!(
                    "{block_path} metadata exceeds maximum entry count ({})",
                    limits::MAX_METADATA_ENTRIES
                ),
            ));
        }
        if block.dependencies.len() > limits::MAX_DEPENDENCIES {
            return Err(PrefixityError::validation(
                path,
                format!(
                    "{block_path} dependencies exceeds maximum count ({})",
                    limits::MAX_DEPENDENCIES
                ),
            ));
        }
    }

    Ok(ValidationReport { warnings })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash_content;
    use crate::model::{ContextBlock, RequestTrace};
    use std::collections::BTreeMap;

    fn block(id: &str, position: usize, content: &str) -> ContextBlock {
        ContextBlock {
            id: id.to_string(),
            source: "test".to_string(),
            position,
            content_hash: hash_content(content),
            token_count: Some(10),
            byte_count: content.len() as u64,
            timestamp: None,
            content: Some(content.to_string()),
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

    fn trace(blocks: Vec<ContextBlock>) -> RequestTrace {
        RequestTrace {
            format_version: TRACE_FORMAT_VERSION,
            request_id: "req-1".to_string(),
            session_id: None,
            timestamp: None,
            provider: "synthetic".to_string(),
            model: "synthetic-model".to_string(),
            evidence_schema_version: None,
            blocks,
            usage: None,
            provider_response: None,
            latency: None,
            provenance: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn valid_trace_passes() {
        let t = trace(vec![block("a", 0, "hello"), block("b", 1, "world")]);
        let report = validate_trace(&t, None).unwrap();
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut t = trace(vec![block("a", 0, "hello")]);
        t.format_version = 999;
        let err = validate_trace(&t, None).unwrap_err();
        assert!(matches!(
            err,
            PrefixityError::UnsupportedFormatVersion { found: 999, .. }
        ));
    }

    #[test]
    fn rejects_empty_request_id() {
        let mut t = trace(vec![block("a", 0, "hello")]);
        t.request_id = "   ".to_string();
        assert!(matches!(
            validate_trace(&t, None),
            Err(PrefixityError::Validation { .. })
        ));
    }

    #[test]
    fn rejects_duplicate_block_ids() {
        let t = trace(vec![block("a", 0, "one"), block("a", 1, "two")]);
        let err = validate_trace(&t, None).unwrap_err();
        assert!(err.to_string().contains("duplicate block id"));
    }

    #[test]
    fn rejects_bad_hash_format() {
        let mut b = block("a", 0, "hello");
        b.content_hash = "xyz".to_string();
        let t = trace(vec![b]);
        assert!(matches!(
            validate_trace(&t, None),
            Err(PrefixityError::Validation { .. })
        ));
    }

    #[test]
    fn rejects_hash_mismatch_with_content() {
        let mut b = block("a", 0, "hello");
        b.content_hash = "0".repeat(64);
        let t = trace(vec![b]);
        let err = validate_trace(&t, None).unwrap_err();
        assert!(err.to_string().contains("does not match"));
    }

    #[test]
    fn rejects_position_mismatch() {
        let t = trace(vec![block("a", 1, "hello")]);
        assert!(matches!(
            validate_trace(&t, None),
            Err(PrefixityError::Validation { .. })
        ));
    }

    #[test]
    fn rejects_byte_count_mismatch_with_content() {
        let mut b = block("a", 0, "hello");
        b.byte_count = 999;
        let t = trace(vec![b]);
        let err = validate_trace(&t, None).unwrap_err();
        assert!(err.to_string().contains("byte_count"));
    }

    #[test]
    fn accepts_content_with_matching_byte_count() {
        let t = trace(vec![block("a", 0, "hello")]);
        assert!(validate_trace(&t, None).is_ok());
    }

    #[test]
    fn utf8_byte_count_uses_bytes_not_chars() {
        let content = "héllo wörld"; // multi-byte characters
        let mut b = block("a", 0, content);
        // Correct byte length: content.len() counts UTF-8 bytes.
        b.byte_count = content.len() as u64;
        let t = trace(vec![b]);
        assert!(validate_trace(&t, None).is_ok());
    }
}
