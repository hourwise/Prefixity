//! Provider usage normalisation.
//!
//! Raw provider usage fields have **different meanings per provider** — and
//! per **API surface**. One provider may expose different usage semantics
//! across endpoints (e.g. OpenAI Chat Completions vs a future OpenAI
//! Responses surface), so the provider name alone is insufficient. Schemas
//! are therefore identified by explicit, versioned API-surface identifiers
//! such as `openai-chat-completions-v1`.
//!
//! For example, Anthropic Messages' `input_tokens` is the *uncached*
//! remainder (total is the sum of three input categories), while the
//! synthetic schema's `input_tokens` is the *total* input. A normalizer
//! converts known raw schemas into a provider-independent
//! [`NormalizedUsage`] without inventing values that cannot be derived.
//!
//! Phase 0A.1 implements **deterministic, offline** normalizers only. They
//! make no network calls. Raw data is always preserved (see
//! [`crate::model::RawUsage`]).

use crate::model::RawUsage;
use std::collections::BTreeMap;
use std::fmt;

/// Schema identifier for Prefixity's own synthetic usage shape.
pub const SCHEMA_SYNTHETIC: &str = "synthetic";
/// OpenAI Chat Completions API usage surface, version 1.
pub const SCHEMA_OPENAI_CHAT_COMPLETIONS_V1: &str = "openai-chat-completions-v1";
/// Anthropic Messages API usage surface, version 1.
pub const SCHEMA_ANTHROPIC_MESSAGES_V1: &str = "anthropic-messages-v1";
/// DeepSeek Chat Completions API usage surface, version 1.
pub const SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1: &str = "deepseek-chat-completions-v1";
/// **Reserved** for a future OpenAI Responses adapter. No normalizer is
/// registered for this surface yet; traces carrying it are never interpreted
/// as Chat Completions.
pub const SCHEMA_OPENAI_RESPONSES_V1: &str = "openai-responses-v1";

/// Legacy Phase 0A.1 trace-v2 generic provider name, accepted READ-ONLY as
/// a compatibility alias for [`SCHEMA_OPENAI_CHAT_COMPLETIONS_V1`].
///
/// Historical trace-v2 files written before Phase 0B used the bare provider
/// name as `provider_schema`. These aliases exist **solely** so that existing
/// valid trace-v2 data produced by Phase 0A.1 continues to normalize after
/// the Phase 0B schema change. Writers MUST NOT emit these generic names.
pub const LEGACY_SCHEMA_OPENAI: &str = "openai";
/// Legacy Phase 0A.1 trace-v2 generic provider name, accepted READ-ONLY as
/// a compatibility alias for [`SCHEMA_ANTHROPIC_MESSAGES_V1`].
pub const LEGACY_SCHEMA_ANTHROPIC: &str = "anthropic";
/// Legacy Phase 0A.1 trace-v2 generic provider name, accepted READ-ONLY as
/// a compatibility alias for [`SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1`].
pub const LEGACY_SCHEMA_DEEPSEEK: &str = "deepseek";

/// A provider-independent, normalised view of one request's usage.
///
/// Every field is optional: normalizers never manufacture values that cannot
/// be derived from the raw capture.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct NormalizedUsage {
    /// Total input tokens (all input categories), when derivable.
    pub total_input_tokens: Option<u64>,
    /// Fresh (non-cached) input tokens, when derivable.
    pub fresh_input_tokens: Option<u64>,
    /// Tokens served from the provider's cache, when reported.
    pub cache_read_tokens: Option<u64>,
    /// Tokens written to the provider's cache, when reported.
    pub cache_write_tokens: Option<u64>,
    /// Output tokens, when reported.
    pub output_tokens: Option<u64>,
    /// Which schema produced this normalisation.
    pub normalization_source: String,
    /// Human-readable explanation of how the values were derived.
    pub explanation: String,
}

impl NormalizedUsage {
    fn empty(source: &str) -> Self {
        NormalizedUsage {
            total_input_tokens: None,
            fresh_input_tokens: None,
            cache_read_tokens: None,
            cache_write_tokens: None,
            output_tokens: None,
            normalization_source: source.to_string(),
            explanation: "no usable fields found in raw usage".to_string(),
        }
    }
}

/// A usage-schema adapter: converts a [`RawUsage`] into a [`NormalizedUsage`].
pub trait UsageNormalizer: fmt::Debug + Send + Sync {
    /// Canonical schema name handled by this normalizer.
    fn schema_name(&self) -> &'static str;
    /// Normalise the raw usage, never manufacturing underivable values.
    fn normalize(&self, raw: &RawUsage) -> NormalizedUsage;
}

/// Helper to read a top-level u64 field from a raw usage map.
fn read_u64(raw: &BTreeMap<String, serde_json::Value>, key: &str) -> Option<u64> {
    raw.get(key).and_then(serde_json::Value::as_u64)
}

/// Helper to read a nested u64 field (e.g. `prompt_tokens_details.cached_tokens`).
fn read_nested_u64(
    raw: &BTreeMap<String, serde_json::Value>,
    outer: &str,
    inner: &str,
) -> Option<u64> {
    raw.get(outer)
        .and_then(serde_json::Value::as_object)
        .and_then(|obj| obj.get(inner))
        .and_then(serde_json::Value::as_u64)
}

/// Synthetic schema normalizer.
///
/// Raw fields: `input_tokens` (total input), `cache_read_tokens`,
/// `cache_write_tokens`, `output_tokens`. For this schema, the relationship
/// `fresh = total - read - write` is explicitly supported.
#[derive(Debug, Default)]
pub struct SyntheticNormalizer;

impl UsageNormalizer for SyntheticNormalizer {
    fn schema_name(&self) -> &'static str {
        "synthetic"
    }
    fn normalize(&self, raw: &RawUsage) -> NormalizedUsage {
        let total = read_u64(&raw.raw, "input_tokens");
        let read = read_u64(&raw.raw, "cache_read_tokens");
        let write = read_u64(&raw.raw, "cache_write_tokens");
        let output = read_u64(&raw.raw, "output_tokens");
        let fresh = match (total, read, write) {
            (Some(t), Some(r), Some(w)) => Some(t.saturating_sub(r.saturating_add(w))),
            (Some(t), Some(r), None) => Some(t.saturating_sub(r)),
            _ => None,
        };
        NormalizedUsage {
            total_input_tokens: total,
            fresh_input_tokens: fresh,
            cache_read_tokens: read,
            cache_write_tokens: write,
            output_tokens: output,
            normalization_source: self.schema_name().to_string(),
            explanation: "synthetic schema: input_tokens is total input; fresh = total - read - write is supported by this schema"
                .to_string(),
        }
    }
}

/// Anthropic-shaped schema normalizer.
///
/// Raw fields: `input_tokens` (**uncached** remainder after the cache
/// breakpoint), `cache_read_input_tokens`, `cache_creation_input_tokens`,
/// `output_tokens`. Total input is their sum.
#[derive(Debug, Default)]
pub struct AnthropicNormalizer;

impl UsageNormalizer for AnthropicNormalizer {
    fn schema_name(&self) -> &'static str {
        SCHEMA_ANTHROPIC_MESSAGES_V1
    }
    fn normalize(&self, raw: &RawUsage) -> NormalizedUsage {
        let fresh = read_u64(&raw.raw, "input_tokens");
        let read = read_u64(&raw.raw, "cache_read_input_tokens");
        let write = read_u64(&raw.raw, "cache_creation_input_tokens");
        let output = read_u64(&raw.raw, "output_tokens");
        let total = match (fresh, read, write) {
            (Some(f), Some(r), Some(w)) => Some(f + r + w),
            (Some(f), Some(r), None) => Some(f + r),
            _ => None,
        };
        NormalizedUsage {
            total_input_tokens: total,
            fresh_input_tokens: fresh,
            cache_read_tokens: read,
            cache_write_tokens: write,
            output_tokens: output,
            normalization_source: self.schema_name().to_string(),
            explanation: "anthropic schema: input_tokens is the uncached remainder; total input is the sum of input_tokens + cache_read_input_tokens + cache_creation_input_tokens"
                .to_string(),
        }
    }
}

/// DeepSeek-shaped schema normalizer.
///
/// Raw fields: `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`,
/// `completion_tokens`. Total input is hit + miss. Cache writes are not
/// reported by this schema.
#[derive(Debug, Default)]
pub struct DeepSeekNormalizer;

impl UsageNormalizer for DeepSeekNormalizer {
    fn schema_name(&self) -> &'static str {
        SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1
    }
    fn normalize(&self, raw: &RawUsage) -> NormalizedUsage {
        let read = read_u64(&raw.raw, "prompt_cache_hit_tokens");
        let fresh = read_u64(&raw.raw, "prompt_cache_miss_tokens");
        let output =
            read_u64(&raw.raw, "completion_tokens").or_else(|| read_u64(&raw.raw, "output_tokens"));
        let total = match (read, fresh) {
            (Some(r), Some(f)) => Some(r + f),
            _ => None,
        };
        NormalizedUsage {
            total_input_tokens: total,
            fresh_input_tokens: fresh,
            cache_read_tokens: read,
            cache_write_tokens: None,
            output_tokens: output,
            normalization_source: self.schema_name().to_string(),
            explanation: "deepseek schema: prompt_cache_hit_tokens + prompt_cache_miss_tokens = total input; cache writes are not reported"
                .to_string(),
        }
    }
}

/// OpenAI-shaped schema normalizer.
///
/// Raw fields: `prompt_tokens` (total input),
/// `prompt_tokens_details.cached_tokens` (nested), `completion_tokens`.
/// Cache writes are not reported by this schema.
#[derive(Debug, Default)]
pub struct OpenAiNormalizer;

impl UsageNormalizer for OpenAiNormalizer {
    fn schema_name(&self) -> &'static str {
        SCHEMA_OPENAI_CHAT_COMPLETIONS_V1
    }
    fn normalize(&self, raw: &RawUsage) -> NormalizedUsage {
        let total = read_u64(&raw.raw, "prompt_tokens");
        let read = read_nested_u64(&raw.raw, "prompt_tokens_details", "cached_tokens");
        let output = read_u64(&raw.raw, "completion_tokens");
        let fresh = match (total, read) {
            (Some(t), Some(r)) => Some(t.saturating_sub(r)),
            _ => None,
        };
        NormalizedUsage {
            total_input_tokens: total,
            fresh_input_tokens: fresh,
            cache_read_tokens: read,
            cache_write_tokens: None,
            output_tokens: output,
            normalization_source: self.schema_name().to_string(),
            explanation: "openai schema: prompt_tokens is total input; cached_tokens is nested under prompt_tokens_details; cache writes are not reported"
                .to_string(),
        }
    }
}

/// The usage schemas with an offline normalizer in this version. The reserved
/// [`SCHEMA_OPENAI_RESPONSES_V1`] surface is intentionally **not** listed.
pub fn available_schemas() -> &'static [&'static str] {
    &[
        SCHEMA_SYNTHETIC,
        SCHEMA_OPENAI_CHAT_COMPLETIONS_V1,
        SCHEMA_ANTHROPIC_MESSAGES_V1,
        SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1,
    ]
}

/// Normalise a raw usage capture by dispatching on its `provider_schema`.
///
/// The canonical schemas are explicit versioned API-surface identifiers
/// (`openai-chat-completions-v1`, `anthropic-messages-v1`,
/// `deepseek-chat-completions-v1`, `synthetic`). In addition, the three
/// **legacy Phase 0A.1 generic provider names** (`openai`, `anthropic`,
/// `deepseek`) are accepted READ-ONLY as compatibility aliases for their
/// versioned surfaces, so historical trace-v2 files written before Phase 0B
/// keep normalizing. The aliases never affect what writers emit and are not
/// advertised by [`available_schemas`].
///
/// Unknown schemas — including arbitrary provider names and the reserved
/// `openai-responses-v1` surface — produce an all-`None` [`NormalizedUsage`]
/// with a clear explanation. Values are never manufactured, and an unknown
/// OpenAI schema is never silently interpreted as Chat Completions.
pub fn normalize_usage(raw: &RawUsage) -> NormalizedUsage {
    match raw.provider_schema.as_str() {
        SCHEMA_SYNTHETIC => SyntheticNormalizer.normalize(raw),
        SCHEMA_OPENAI_CHAT_COMPLETIONS_V1 => OpenAiNormalizer.normalize(raw),
        SCHEMA_ANTHROPIC_MESSAGES_V1 => AnthropicNormalizer.normalize(raw),
        SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1 => DeepSeekNormalizer.normalize(raw),
        // Legacy Phase 0A.1 trace-v2 compatibility aliases (read-only): the
        // bare generic provider name meant the same usage surface as its
        // versioned API-surface identifier.
        LEGACY_SCHEMA_OPENAI => {
            normalize_with_legacy_alias(&OpenAiNormalizer, raw, LEGACY_SCHEMA_OPENAI)
        }
        LEGACY_SCHEMA_ANTHROPIC => {
            normalize_with_legacy_alias(&AnthropicNormalizer, raw, LEGACY_SCHEMA_ANTHROPIC)
        }
        LEGACY_SCHEMA_DEEPSEEK => {
            normalize_with_legacy_alias(&DeepSeekNormalizer, raw, LEGACY_SCHEMA_DEEPSEEK)
        }
        SCHEMA_OPENAI_RESPONSES_V1 => {
            let mut usage = NormalizedUsage::empty("unknown-schema");
            usage.explanation = "schema 'openai-responses-v1' is reserved for a later OpenAI \
                Responses adapter; no normalizer is registered in this version and it is NOT \
                interpreted as Chat Completions"
                .to_string();
            usage
        }
        other => {
            let mut usage = NormalizedUsage::empty("unknown-schema");
            usage.explanation = format!(
                "no offline normalizer registered for schema '{other}'; the provider name alone is \
                 not sufficient (the API surface must be identified); values cannot be derived safely"
            );
            usage
        }
    }
}

/// Normalise with `normalizer`, then annotate the explanation to record that
/// a legacy Phase 0A.1 trace-v2 compatibility alias was consumed.
///
/// The `normalization_source` is the canonical versioned API-surface
/// identifier (the normalizer already sets it); only the explanation is
/// augmented so readers can see the historical generic name was used.
fn normalize_with_legacy_alias(
    normalizer: &dyn UsageNormalizer,
    raw: &RawUsage,
    legacy_alias: &str,
) -> NormalizedUsage {
    let mut usage = normalizer.normalize(raw);
    usage.explanation = format!(
        "{} (consumed legacy trace-v2 provider_schema '{legacy_alias}' as a Phase 0A.1 \
         compatibility alias for '{}')",
        usage.explanation,
        normalizer.schema_name()
    );
    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(schema: &str, fields: &[(&str, u64)]) -> RawUsage {
        let mut map = BTreeMap::new();
        for (key, value) in fields {
            map.insert((*key).to_string(), serde_json::json!(value));
        }
        RawUsage {
            provider_schema: schema.to_string(),
            raw: map,
        }
    }

    #[test]
    fn synthetic_normalizes_flat_fields() {
        let usage = SyntheticNormalizer.normalize(&raw(
            "synthetic",
            &[
                ("input_tokens", 1000),
                ("cache_read_tokens", 700),
                ("cache_write_tokens", 100),
                ("output_tokens", 50),
            ],
        ));
        assert_eq!(usage.total_input_tokens, Some(1000));
        assert_eq!(usage.fresh_input_tokens, Some(200));
        assert_eq!(usage.cache_read_tokens, Some(700));
        assert_eq!(usage.cache_write_tokens, Some(100));
        assert_eq!(usage.output_tokens, Some(50));
    }

    #[test]
    fn anthropic_totals_are_the_sum_of_three_categories() {
        let usage = AnthropicNormalizer.normalize(&raw(
            SCHEMA_ANTHROPIC_MESSAGES_V1,
            &[
                ("input_tokens", 500),
                ("cache_read_input_tokens", 4000),
                ("cache_creation_input_tokens", 500),
                ("output_tokens", 120),
            ],
        ));
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(500));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, Some(500));
        assert_eq!(usage.output_tokens, Some(120));
        assert_eq!(usage.normalization_source, SCHEMA_ANTHROPIC_MESSAGES_V1);
    }

    #[test]
    fn deepseek_hit_plus_miss_is_total() {
        let usage = DeepSeekNormalizer.normalize(&raw(
            SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1,
            &[
                ("prompt_cache_hit_tokens", 4000),
                ("prompt_cache_miss_tokens", 1000),
                ("completion_tokens", 120),
            ],
        ));
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(1000));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(
            usage.normalization_source,
            SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1
        );
    }

    #[test]
    fn openai_reads_nested_cached_tokens() {
        let mut map = BTreeMap::new();
        map.insert("prompt_tokens".to_string(), serde_json::json!(5000));
        map.insert(
            "prompt_tokens_details".to_string(),
            serde_json::json!({ "cached_tokens": 4000 }),
        );
        map.insert("completion_tokens".to_string(), serde_json::json!(120));
        let usage = OpenAiNormalizer.normalize(&RawUsage {
            provider_schema: SCHEMA_OPENAI_CHAT_COMPLETIONS_V1.to_string(),
            raw: map,
        });
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(1000));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(
            usage.normalization_source,
            SCHEMA_OPENAI_CHAT_COMPLETIONS_V1
        );
    }

    #[test]
    fn fields_with_different_meanings_are_not_interchangeable() {
        // The SAME numeric fields mean different things per schema. Feeding
        // Anthropic-shaped raw into the synthetic normalizer must NOT produce
        // the Anthropic result.
        let anthropic_raw = raw(
            SCHEMA_ANTHROPIC_MESSAGES_V1,
            &[
                ("input_tokens", 500),
                ("cache_read_input_tokens", 4000),
                ("cache_creation_input_tokens", 500),
            ],
        );
        let as_anthropic = normalize_usage(&anthropic_raw);
        let as_synthetic = SyntheticNormalizer.normalize(&anthropic_raw);

        assert_eq!(as_anthropic.total_input_tokens, Some(5000));
        // Synthetic interprets input_tokens as total: 500.
        assert_eq!(as_synthetic.total_input_tokens, Some(500));
        assert_ne!(
            as_anthropic.total_input_tokens,
            as_synthetic.total_input_tokens
        );
    }

    #[test]
    fn dispatch_by_schema_and_unknown_schema_does_not_manufacture() {
        // Anthropic with only input_tokens cannot derive a total: it would
        // need the read/write categories too. Values are never invented.
        assert_eq!(
            normalize_usage(&raw(SCHEMA_ANTHROPIC_MESSAGES_V1, &[("input_tokens", 5)]))
                .total_input_tokens,
            None
        );
        let unknown = normalize_usage(&raw("mystery-vendor", &[("input_tokens", 5)]));
        assert_eq!(unknown.total_input_tokens, None);
        assert!(unknown.explanation.contains("no offline normalizer"));
    }

    #[test]
    fn generic_and_reserved_schemas_are_not_interpreted() {
        // A TRULY generic/unknown provider name is NOT a valid schema:
        // dispatch must not guess the API surface.
        let generic = normalize_usage(&raw("mystery-vendor", &[("prompt_tokens", 100)]));
        assert_eq!(generic.total_input_tokens, None);
        assert_eq!(generic.normalization_source, "unknown-schema");
        assert!(generic.explanation.contains("no offline normalizer"));

        // The reserved Responses surface must not be read as Chat Completions.
        let reserved = normalize_usage(&raw(SCHEMA_OPENAI_RESPONSES_V1, &[("prompt_tokens", 100)]));
        assert_eq!(reserved.total_input_tokens, None);
        assert!(reserved.explanation.contains("reserved"));
        assert!(reserved
            .explanation
            .contains("NOT interpreted as Chat Completions"));
    }

    #[test]
    fn legacy_v2_generic_openai_alias_normalizes_like_phase_0a1() {
        // The pre-Phase-0B generic provider name "openai" (as emitted by
        // Phase 0A.1 trace-v2 files) must normalize exactly as its old
        // meaning did — OpenAI Chat Completions v1 semantics — while the
        // normalization_source is the canonical versioned identifier.
        let usage = normalize_usage(&raw(
            LEGACY_SCHEMA_OPENAI,
            &[("prompt_tokens", 5000), ("completion_tokens", 120)],
        ));
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.output_tokens, Some(120));
        // Nested cached_tokens is still read for the generic alias.
        let mut map = BTreeMap::new();
        map.insert("prompt_tokens".to_string(), serde_json::json!(5000));
        map.insert(
            "prompt_tokens_details".to_string(),
            serde_json::json!({ "cached_tokens": 4000 }),
        );
        map.insert("completion_tokens".to_string(), serde_json::json!(120));
        let usage = normalize_usage(&RawUsage {
            provider_schema: LEGACY_SCHEMA_OPENAI.to_string(),
            raw: map,
        });
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(1000));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.output_tokens, Some(120));
        assert_eq!(
            usage.normalization_source,
            SCHEMA_OPENAI_CHAT_COMPLETIONS_V1
        );
        assert!(
            usage
                .explanation
                .contains("legacy trace-v2 provider_schema 'openai'"),
            "explanation must record the consumed alias: {}",
            usage.explanation
        );
        assert!(usage
            .explanation
            .contains(SCHEMA_OPENAI_CHAT_COMPLETIONS_V1));
    }

    #[test]
    fn legacy_v2_generic_anthropic_alias_normalizes_like_phase_0a1() {
        // The pre-Phase-0B generic provider name "anthropic" maps to
        // Anthropic Messages v1 semantics (the exact shape of main's old
        // fixture 09-anthropic-usage-semantics.json).
        let usage = normalize_usage(&raw(
            LEGACY_SCHEMA_ANTHROPIC,
            &[
                ("input_tokens", 500),
                ("cache_read_input_tokens", 4000),
                ("cache_creation_input_tokens", 500),
                ("output_tokens", 120),
            ],
        ));
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(500));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, Some(500));
        assert_eq!(usage.output_tokens, Some(120));
        assert_eq!(usage.normalization_source, SCHEMA_ANTHROPIC_MESSAGES_V1);
        assert!(
            usage
                .explanation
                .contains("legacy trace-v2 provider_schema 'anthropic'"),
            "explanation must record the consumed alias: {}",
            usage.explanation
        );
        assert!(usage.explanation.contains(SCHEMA_ANTHROPIC_MESSAGES_V1));
    }

    #[test]
    fn legacy_v2_generic_deepseek_alias_normalizes_like_phase_0a1() {
        // The pre-Phase-0B generic provider name "deepseek" maps to DeepSeek
        // Chat Completions v1 semantics (the exact shape of main's old
        // fixture 10-deepseek-usage-semantics.json).
        let usage = normalize_usage(&raw(
            LEGACY_SCHEMA_DEEPSEEK,
            &[
                ("prompt_cache_hit_tokens", 4000),
                ("prompt_cache_miss_tokens", 1000),
                ("completion_tokens", 120),
            ],
        ));
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(1000));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, None);
        assert_eq!(usage.output_tokens, Some(120));
        assert_eq!(
            usage.normalization_source,
            SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1
        );
        assert!(
            usage
                .explanation
                .contains("legacy trace-v2 provider_schema 'deepseek'"),
            "explanation must record the consumed alias: {}",
            usage.explanation
        );
        assert!(usage
            .explanation
            .contains(SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1));
    }

    #[test]
    fn available_schemas_advertise_only_canonical_versioned_ids() {
        // The compatibility aliases must NOT be advertised: new writers and
        // tooling should only ever see/emit explicit versioned API-surface
        // identifiers.
        let schemas = available_schemas();
        assert!(schemas.contains(&SCHEMA_SYNTHETIC));
        assert!(schemas.contains(&SCHEMA_OPENAI_CHAT_COMPLETIONS_V1));
        assert!(schemas.contains(&SCHEMA_ANTHROPIC_MESSAGES_V1));
        assert!(schemas.contains(&SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1));
        assert!(!schemas.contains(&LEGACY_SCHEMA_OPENAI));
        assert!(!schemas.contains(&LEGACY_SCHEMA_ANTHROPIC));
        assert!(!schemas.contains(&LEGACY_SCHEMA_DEEPSEEK));
        assert!(!schemas.contains(&SCHEMA_OPENAI_RESPONSES_V1));
    }
}
