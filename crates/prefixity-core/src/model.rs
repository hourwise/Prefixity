//! Core data model for the versioned Prefixity trace format (Phase 0A).
//!
//! See `docs/phase-0/TRACE_FORMAT.md` for the normative description of the
//! on-disk format. The structs here mirror that document.
//!
//! Phase 0A.1 correction: the model distinguishes three concepts that must
//! never be conflated:
//!
//! * **prefixity score** — an experimental deterministic estimate of whether
//!   a block looks suitable for stable-prefix placement (see
//!   [`crate::prefixity_score`]);
//! * **observed prefix reuse** — exact structural prefix equality between
//!   consecutive recorded requests (see [`crate::compare`]);
//! * **provider-reported cache reuse** — tokens the provider explicitly
//!   reports, captured as raw usage and normalised by [`crate::usage`].
//!
//! A single isolated trace cannot prove that any tokens were reused.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The trace format version this crate reads and writes.
///
/// Bumped to 2 in Phase 0A.1: `usage` became [`RawUsage`] (schema + verbatim
/// fields) and [`ContextBlock`] gained structural identity fields.
pub const TRACE_FORMAT_VERSION: u32 = 2;

/// The provider-profile format version this crate reads.
pub const PROVIDER_PROFILE_FORMAT_VERSION: u32 = 1;

/// A single recorded LLM/agent request and its context.
///
/// Traces are the unit of analysis. A trace must carry a valid
/// `format_version`, a non-empty `request_id`, a non-empty `provider` and
/// `model`, and at least one [`ContextBlock`] (see [`crate::validation`]).
///
/// The format is intentionally lossy with respect to prompt content:
/// `ContextBlock` stores hashes and metadata, and only optionally the raw
/// content. This supports the Phase 0 privacy stance of avoiding complete
/// prompt retention wherever hashes and metadata suffice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestTrace {
    /// Trace format version (see [`TRACE_FORMAT_VERSION`]).
    pub format_version: u32,
    /// Unique identifier for this request within the recording session.
    pub request_id: String,
    /// Optional session identifier grouping multiple requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Optional timestamp. Kept as an opaque string: it is metadata, is never
    /// parsed, and must never influence hashing or analysis decisions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Provider identifier (for example `synthetic`, `openai`, `anthropic`).
    pub provider: String,
    /// Model identifier (for example `synthetic-model`).
    pub model: String,
    /// Ordered context blocks exactly as sent to the model.
    pub blocks: Vec<ContextBlock>,
    /// Provider-specific raw usage, preserved verbatim. Normalisation is a
    /// separate step (see [`crate::usage`]); the raw capture is never
    /// reinterpreted in place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<RawUsage>,
    /// Optional latency information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency: Option<LatencyInfo>,
    /// Free-form metadata. Must remain JSON so that unknown future fields
    /// round-trip losslessly.
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// One ordered piece of the context sent to the model.
///
/// `position` must equal the index of the block in `RequestTrace::blocks`
/// (contiguous, starting at 0) — this is enforced by validation.
///
/// `content_hash` is a SHA-256 hex digest and identifies the *content*. It is
/// **not** sufficient for provider prefix comparison: two blocks may contain
/// identical text but occupy different semantic positions. The structural
/// identity fields (`semantic_zone`, `structural_path`, `role`) combined with
/// `content_hash` form the structural fingerprint used for prefix comparison
/// (see [`crate::structure`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextBlock {
    /// Stable unique ID within the trace.
    pub id: String,
    /// Source/type of the block (e.g. `system_policy`, `tool_definition`,
    /// `tool_result`, `user_request`). Unknown values are accepted and scored
    /// conservatively; see [`crate::prefixity_score`].
    pub source: String,
    /// Ordering position within the trace (must equal the array index).
    pub position: usize,
    /// SHA-256 hex digest of the block content (64 lowercase hex chars).
    pub content_hash: String,
    /// Token count if known. If absent, analysis falls back to a documented
    /// heuristic when `content` is present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_count: Option<u64>,
    /// Byte count of the block content. When `content` is present, validation
    /// requires this to equal the UTF-8 byte length of `content`.
    pub byte_count: u64,
    /// Optional actual content. Phase 0 fixtures generally omit this and keep
    /// hashes only, to avoid storing complete prompt content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Provider-independent semantic zone, e.g. `tools`, `system`,
    /// `messages`, `other`. Used by zone-aware policies and the structural
    /// fingerprint. Absent blocks are treated as zone `other`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_zone: Option<String>,
    /// Structural path within the wire message, e.g. `tools[3]`,
    /// `system[0]`, `messages[7].content[1]`. Informational and part of the
    /// structural fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structural_path: Option<String>,
    /// Semantic role/type where applicable, e.g. `system`, `user`,
    /// `assistant`, `tool`, `tool_result`. Part of the structural fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Optional sensitivity classification (e.g. `public`, `confidential`).
    /// Informational only in Phase 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<String>,
    /// IDs of other blocks this block depends on (informational; supports
    /// future dependency-aware scoring).
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Optional observed lifetime/age in turns, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifetime: Option<u64>,
    /// Explicitly marked as removable/optional by the recorder. Policies may
    /// remove optional volatile blocks; never a `required` block.
    #[serde(default)]
    pub optional: bool,
    /// Explicitly marked as required. **Policies must never remove a required
    /// block**, regardless of size or other flags.
    #[serde(default)]
    pub required: bool,
    /// Explicitly marked as stale (e.g. superseded tool output).
    #[serde(default)]
    pub stale: bool,
    /// Free-form block metadata.
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Provider-specific raw usage for one request, preserved verbatim.
///
/// The `provider_schema` names the semantics of the `raw` fields as an
/// **explicit versioned API-surface identifier**, e.g.
/// `openai-chat-completions-v1`, `anthropic-messages-v1`,
/// `deepseek-chat-completions-v1`, or `synthetic`. The provider name alone is
/// insufficient: one provider may expose different usage semantics across
/// endpoints. The `raw` map is opaque to the rest of the model — it is never
/// reinterpreted in place. A normalizer (see [`crate::usage`]) converts known
/// schemas into a [`crate::usage::NormalizedUsage`].
///
/// Provider field semantics are **not interchangeable**. For example,
/// Anthropic's `input_tokens` means *uncached* tokens, while the synthetic
/// schema's `input_tokens` means *total* input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RawUsage {
    /// Which versioned API-surface schema the `raw` fields follow.
    #[serde(default)]
    pub provider_schema: String,
    /// Provider-specific raw usage fields, preserved verbatim.
    #[serde(default)]
    pub raw: BTreeMap<String, serde_json::Value>,
}

/// Optional latency information for one request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LatencyInfo {
    /// Time to first token, in milliseconds, if measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_to_first_token_ms: Option<u64>,
    /// Total request latency, in milliseconds, if measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_ms: Option<u64>,
    /// Provider-specific raw latency fields preserved verbatim.
    #[serde(default)]
    pub provider_raw: BTreeMap<String, serde_json::Value>,
}

/// An externally supplied provider cost profile.
///
/// Phase 0 deliberately does **not** hard-code claimed current provider
/// pricing. Profiles are data, loaded from `provider-profiles/`, and must be
/// marked `synthetic` unless a live phase supplies audited figures. All
/// prices are per one million tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostProfile {
    /// Profile name (e.g. `synthetic-example`).
    pub name: String,
    /// Profile format version (see [`PROVIDER_PROFILE_FORMAT_VERSION`]).
    #[serde(default)]
    pub version: u32,
    /// Must be `true` for every example profile in this repository. Set to
    /// `false` only by an audited, externally supplied profile in a later
    /// phase.
    pub synthetic: bool,
    /// Currency code (e.g. `USD`).
    pub currency: String,
    /// Price per 1M input (non-cached) tokens.
    pub input_price_per_1m: f64,
    /// Price per 1M cache-read tokens.
    pub cache_read_price_per_1m: f64,
    /// Price per 1M cache-write tokens. Use `0.0` for providers that do not
    /// charge separately for cache writes.
    pub cache_write_price_per_1m: f64,
    /// Price per 1M output tokens. Use `0.0` if output cost is out of scope.
    pub output_price_per_1m: f64,
    /// Human-readable notes. Every example profile must state it is SYNTHETIC.
    #[serde(default)]
    pub notes: String,
}
