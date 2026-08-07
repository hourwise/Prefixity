//! Provider usage normalisation.
//!
//! Raw provider usage fields have **different meanings per provider**. For
//! example, Anthropic's `input_tokens` is the *uncached* remainder (total is
//! the sum of three input categories), while the synthetic schema's
//! `input_tokens` is the *total* input. A normalizer converts known raw
//! schemas into a provider-independent [`NormalizedUsage`] without inventing
//! values that cannot be derived.
//!
//! Phase 0A.1 implements **deterministic, offline** normalizers only. They
//! make no network calls. Raw data is always preserved (see
//! [`crate::model::RawUsage`]).

use crate::model::RawUsage;
use std::collections::BTreeMap;
use std::fmt;

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
        "anthropic"
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
        "deepseek"
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
        "openai"
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

/// The names of the usage schemas with an offline normalizer in Phase 0A.1.
pub fn available_schemas() -> &'static [&'static str] {
    &["synthetic", "anthropic", "deepseek", "openai"]
}

/// Normalise a raw usage capture by dispatching on its `provider_schema`.
///
/// Unknown schemas produce an all-`None` [`NormalizedUsage`] with a clear
/// explanation — values are never manufactured for unhandled schemas.
pub fn normalize_usage(raw: &RawUsage) -> NormalizedUsage {
    match raw.provider_schema.as_str() {
        "synthetic" => SyntheticNormalizer.normalize(raw),
        "anthropic" => AnthropicNormalizer.normalize(raw),
        "deepseek" => DeepSeekNormalizer.normalize(raw),
        "openai" => OpenAiNormalizer.normalize(raw),
        other => {
            let mut usage = NormalizedUsage::empty("unknown-schema");
            usage.explanation = format!(
                "no offline normalizer registered for schema '{other}'; values cannot be derived safely"
            );
            usage
        }
    }
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
            "anthropic",
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
    }

    #[test]
    fn deepseek_hit_plus_miss_is_total() {
        let usage = DeepSeekNormalizer.normalize(&raw(
            "deepseek",
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
            provider_schema: "openai".to_string(),
            raw: map,
        });
        assert_eq!(usage.total_input_tokens, Some(5000));
        assert_eq!(usage.fresh_input_tokens, Some(1000));
        assert_eq!(usage.cache_read_tokens, Some(4000));
        assert_eq!(usage.cache_write_tokens, None);
    }

    #[test]
    fn fields_with_different_meanings_are_not_interchangeable() {
        // The SAME numeric fields mean different things per schema. Feeding
        // Anthropic-shaped raw into the synthetic normalizer must NOT produce
        // the Anthropic result.
        let anthropic_raw = raw(
            "anthropic",
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
            normalize_usage(&raw("anthropic", &[("input_tokens", 5)])).total_input_tokens,
            None
        );
        let unknown = normalize_usage(&raw("mystery-vendor", &[("input_tokens", 5)]));
        assert_eq!(unknown.total_input_tokens, None);
        assert!(unknown.explanation.contains("no offline normalizer"));
    }
}
