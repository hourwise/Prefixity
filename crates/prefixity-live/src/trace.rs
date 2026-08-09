//! Conversion of one captured live request into a Prefixity trace (format v2).
//!
//! Block content is **not** stored in the trace (hashes and structural
//! identity suffice, per the Phase 0 privacy stance); the deterministic
//! synthetic content can be regenerated from the manifest's seed and
//! scenario.

use crate::content::estimate_tokens;
use crate::providers::LiveProvider;
use prefixity_core::hash::sha256_hex;
use prefixity_core::model::{
    ContextBlock, LatencyInfo, RawUsage, RequestTrace, TRACE_FORMAT_VERSION,
};
use std::collections::BTreeMap;

/// Everything captured for one live request.
#[derive(Debug, Clone)]
pub struct RequestRecord {
    /// 1-based turn number.
    pub turn: usize,
    /// Header block content.
    pub header: String,
    /// Large synthetic prefix (stable core) block content.
    pub prefix: String,
    /// Late mutable suffix block content, if any (`late-divergence` only).
    pub suffix: Option<String>,
    /// Per-turn tail instruction content.
    pub tail: String,
    /// Raw provider usage (schema + verbatim fields).
    pub raw_usage: RawUsage,
    /// Provider request id if captured (safe).
    pub provider_request_id: Option<String>,
    /// HTTP status.
    pub http_status: u16,
    /// Start time (ISO-8601 UTC).
    pub started_at: String,
    /// Time until response headers were received.
    pub time_to_headers_ms: u64,
    /// Time until the first body byte arrived (approximate), if measurable.
    pub time_to_first_body_byte_ms: Option<u64>,
    /// Total response time.
    pub total_ms: u64,
}

fn block(
    position: usize,
    id: &str,
    source: &str,
    zone: &str,
    role: &str,
    path: &str,
    content: &str,
) -> ContextBlock {
    ContextBlock {
        id: id.to_string(),
        source: source.to_string(),
        position,
        content_hash: sha256_hex(content.as_bytes()),
        token_count: Some(estimate_tokens(content)),
        byte_count: content.len() as u64,
        timestamp: None,
        content: None,
        semantic_zone: Some(zone.to_string()),
        structural_path: Some(path.to_string()),
        role: Some(role.to_string()),
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

/// Build a format-v2 [`RequestTrace`] from a captured live request.
pub fn build_trace(
    provider: &dyn LiveProvider,
    model: &str,
    experiment_id: &str,
    record: &RequestRecord,
) -> RequestTrace {
    let mut blocks = vec![
        block(
            0,
            "prefix-header",
            "timestamp",
            "system",
            "system",
            provider.header_structural_path(),
            &record.header,
        ),
        block(
            1,
            "synthetic-prefix",
            "system_policy",
            "system",
            "system",
            provider.prefix_structural_path(),
            &record.prefix,
        ),
    ];
    if let Some(suffix) = &record.suffix {
        blocks.push(block(
            blocks.len(),
            "late-suffix",
            "system_policy",
            "system",
            "system",
            provider.suffix_structural_path(),
            suffix,
        ));
    }
    blocks.push(block(
        blocks.len(),
        "tail",
        "user_request",
        "messages",
        "user",
        provider.tail_structural_path(record.suffix.is_some()),
        &record.tail,
    ));

    let mut metadata = BTreeMap::new();
    metadata.insert(
        "http_status".to_string(),
        serde_json::json!(record.http_status),
    );
    if let Some(request_id) = &record.provider_request_id {
        metadata.insert(
            "provider_request_id".to_string(),
            serde_json::json!(request_id),
        );
    }

    RequestTrace {
        format_version: TRACE_FORMAT_VERSION,
        request_id: format!("{experiment_id}-turn-{}", record.turn),
        session_id: Some(experiment_id.to_string()),
        timestamp: Some(record.started_at.clone()),
        provider: provider.provider_id().to_string(),
        model: model.to_string(),
        evidence_schema_version: None,
        blocks,
        usage: Some(record.raw_usage.clone()),
        provider_response: None,
        latency: Some(LatencyInfo {
            time_to_first_token_ms: None,
            total_ms: Some(record.total_ms),
            provider_raw: BTreeMap::new(),
        }),
        provenance: BTreeMap::new(),
        metadata,
    }
}
