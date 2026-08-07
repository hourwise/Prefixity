//! Cost arithmetic over externally supplied [`CostProfile`] values, plus a
//! theoretical cache-economics evaluation.
//!
//! Pricing is **data**, never hard-coded fact. Phase 0 only uses synthetic
//! profiles; real profiles belong to a later, audited phase.
//!
//! Billing operates on **explicit normalized categories**. The
//! `fresh = total - read - write` relationship is only applied where the
//! caller can support it (e.g. the synthetic usage schema or a labelled
//! hypothetical model); it is never silently assumed for provider-normalized
//! usage.

use crate::model::{CostProfile, PROVIDER_PROFILE_FORMAT_VERSION};
use crate::usage::NormalizedUsage;

/// One million, the denominator for all per-1M prices.
pub const PER_MILLION: f64 = 1_000_000.0;

/// A breakdown of estimated cost for one request under one profile.
///
/// Input is split into explicit categories: `fresh_input_tokens` (billed at
/// the input price) and `cache_read_tokens` / `cache_write_tokens`. The
/// `fresh_input_derivation` records how the fresh category was obtained so a
/// reader can tell explicit normalized values from derived/heuristic ones.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CostBreakdown {
    /// The profile name used.
    pub profile_name: String,
    /// Whether the profile is marked synthetic.
    pub synthetic: bool,
    /// The currency code.
    pub currency: String,
    /// Total input tokens.
    pub total_input_tokens: u64,
    /// Fresh (non-cached) input tokens, billed at the input price.
    pub fresh_input_tokens: u64,
    /// Cache-read tokens.
    pub cache_read_tokens: u64,
    /// Cache-write tokens.
    pub cache_write_tokens: u64,
    /// Output tokens.
    pub output_tokens: u64,
    /// How the fresh category was derived (explicit, derived, or heuristic).
    pub fresh_input_derivation: String,
    /// Cost of the fresh input tokens.
    pub fresh_input_cost: f64,
    /// Cost of cache reads.
    pub cache_read_cost: f64,
    /// Cost of cache writes.
    pub cache_write_cost: f64,
    /// Cost of output tokens.
    pub output_cost: f64,
    /// Total estimated cost.
    pub total_cost: f64,
}

/// Compute a cost breakdown from explicit token categories.
///
/// `fresh_input` is taken as given — this function never derives it. Callers
/// that need the `total - read - write` relationship must establish that it
/// is valid first (see [`derive_fresh_input`]) and pass the result in with an
/// honest `fresh_input_derivation` string.
///
/// ```text
/// cost = fresh_input * input_price + cache_read * cache_read_price
///      + cache_write * cache_write_price + output * output_price
/// ```
pub fn compute_cost(
    total_input: u64,
    fresh_input: u64,
    cache_read: u64,
    cache_write: u64,
    output: u64,
    fresh_input_derivation: &str,
    profile: &CostProfile,
) -> CostBreakdown {
    let fresh_input_cost = fresh_input as f64 / PER_MILLION * profile.input_price_per_1m;
    let cache_read_cost = cache_read as f64 / PER_MILLION * profile.cache_read_price_per_1m;
    let cache_write_cost = cache_write as f64 / PER_MILLION * profile.cache_write_price_per_1m;
    let output_cost = output as f64 / PER_MILLION * profile.output_price_per_1m;
    let total_cost = fresh_input_cost + cache_read_cost + cache_write_cost + output_cost;

    CostBreakdown {
        profile_name: profile.name.clone(),
        synthetic: profile.synthetic,
        currency: profile.currency.clone(),
        total_input_tokens: total_input,
        fresh_input_tokens: fresh_input,
        cache_read_tokens: cache_read,
        cache_write_tokens: cache_write,
        output_tokens: output,
        fresh_input_derivation: fresh_input_derivation.to_string(),
        fresh_input_cost,
        cache_read_cost,
        cache_write_cost,
        output_cost,
        total_cost,
    }
}

/// Compute the fresh-input category with the saturating relationship
/// `fresh = total - read - write`. Only use this where that relationship is
/// explicitly supported (e.g. the synthetic usage schema, or a labelled
/// hypothetical model); never for opaque provider-normalized usage.
pub fn derive_fresh_input(total: u64, cache_read: u64, cache_write: u64) -> u64 {
    total.saturating_sub(cache_read.saturating_add(cache_write))
}

/// Compute cost directly from a [`NormalizedUsage`].
///
/// Only values the normalizer could derive are used; missing categories are
/// billed as zero and the derivation string names the schema.
pub fn compute_cost_normalized(
    normalized: &NormalizedUsage,
    profile: &CostProfile,
) -> CostBreakdown {
    let total = normalized.total_input_tokens.unwrap_or(0);
    let fresh = normalized.fresh_input_tokens.unwrap_or(0);
    let read = normalized.cache_read_tokens.unwrap_or(0);
    let write = normalized.cache_write_tokens.unwrap_or(0);
    let output = normalized.output_tokens.unwrap_or(0);
    compute_cost(
        total,
        fresh,
        read,
        write,
        output,
        &format!(
            "explicit provider-normalized usage (schema: {})",
            normalized.normalization_source
        ),
        profile,
    )
}

/// Format a cost value with six decimal places (stable across runs).
pub fn format_cost(value: f64) -> String {
    format!("{value:.6}")
}

/// Theoretical evaluation of whether provider caching is worthwhile for a
/// request given a profile.
///
/// Model (documented, deliberately simple):
///
/// * **no-cache cost**: every input token is billed at the input price;
/// * **with-cache cost**: the reusable prefix is billed at the cache-read
///   price, the changed tokens are billed at the cache-write price, and any
///   remaining fresh tokens are billed at the input price.
///
/// This is a research estimate used to demonstrate that cache economics
/// depend on provider pricing data — not a provider guarantee.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CacheEconomics {
    /// Total input tokens of the request being evaluated.
    pub input_tokens: u64,
    /// Tokens of the reusable prefix (theoretically cache-readable).
    pub reusable_tokens: u64,
    /// Tokens that changed and would need to be (re)written to cache.
    pub changed_tokens: u64,
    /// Remaining fresh tokens after reusable and changed portions.
    pub fresh_tokens: u64,
    /// Estimated cost if no provider cache is used.
    pub cost_no_cache: f64,
    /// Estimated cost if the provider cache is used.
    pub cost_with_cache: f64,
    /// Whether the cache is theoretically worthwhile under this profile.
    pub cache_worthwhile: bool,
    /// Human-readable explanation of the numbers above.
    pub explanation: String,
}

/// Evaluate whether provider caching is worthwhile for the given token
/// counts under `profile`.
pub fn evaluate_cache_economics(
    input_tokens: u64,
    reusable_tokens: u64,
    changed_tokens: u64,
    profile: &CostProfile,
) -> CacheEconomics {
    let fresh_tokens = input_tokens.saturating_sub(reusable_tokens.saturating_add(changed_tokens));
    let cost_no_cache = input_tokens as f64 / PER_MILLION * profile.input_price_per_1m;
    let cost_with_cache = reusable_tokens as f64 / PER_MILLION * profile.cache_read_price_per_1m
        + changed_tokens as f64 / PER_MILLION * profile.cache_write_price_per_1m
        + fresh_tokens as f64 / PER_MILLION * profile.input_price_per_1m;
    let worthwhile = cost_with_cache + 1e-12 < cost_no_cache;

    let explanation = format!(
        "no-cache cost ${:.6} vs with-cache cost ${:.6} \
         (read {} @ {:.3}/1M, write {} @ {:.3}/1M, fresh {} @ {:.3}/1M). \
         Caching is {} under profile '{}'.",
        cost_no_cache,
        cost_with_cache,
        reusable_tokens,
        profile.cache_read_price_per_1m,
        changed_tokens,
        profile.cache_write_price_per_1m,
        fresh_tokens,
        profile.input_price_per_1m,
        if worthwhile {
            "theoretically worthwhile"
        } else {
            "NOT worthwhile"
        },
        profile.name,
    );

    CacheEconomics {
        input_tokens,
        reusable_tokens,
        changed_tokens,
        fresh_tokens,
        cost_no_cache,
        cost_with_cache,
        cache_worthwhile: worthwhile,
        explanation,
    }
}

/// Validate a cost profile's fields. Returns a human-readable error message
/// on failure.
pub fn validate_cost_profile(profile: &CostProfile) -> Result<(), String> {
    if profile.version != PROVIDER_PROFILE_FORMAT_VERSION {
        return Err(format!(
            "unsupported profile version {} (supported: {})",
            profile.version, PROVIDER_PROFILE_FORMAT_VERSION
        ));
    }
    if profile.name.trim().is_empty() {
        return Err("profile name must not be empty".to_string());
    }
    if profile.currency.trim().is_empty() {
        return Err("currency must not be empty".to_string());
    }
    for (label, value) in [
        ("input_price_per_1m", profile.input_price_per_1m),
        ("cache_read_price_per_1m", profile.cache_read_price_per_1m),
        ("cache_write_price_per_1m", profile.cache_write_price_per_1m),
        ("output_price_per_1m", profile.output_price_per_1m),
    ] {
        if !value.is_finite() || value < 0.0 {
            return Err(format!("{label} must be a finite non-negative number"));
        }
    }
    Ok(())
}

/// The built-in synthetic default profile.
///
/// Used by the CLI when no `--provider-profile` is supplied. It is a clearly
/// labelled example for deterministic tests and is **not** real pricing.
pub fn default_synthetic_profile() -> CostProfile {
    CostProfile {
        name: "synthetic-example".to_string(),
        version: 1,
        synthetic: true,
        currency: "USD".to_string(),
        input_price_per_1m: 2.50,
        cache_read_price_per_1m: 0.10,
        cache_write_price_per_1m: 2.50,
        output_price_per_1m: 10.00,
        notes: "SYNTHETIC example profile for deterministic tests. NOT real provider pricing."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_arithmetic_is_exact() {
        let profile = default_synthetic_profile();
        // total 1M, fresh 1M, read 1M, write 1M, output 1M.
        // fresh @ 2.50 -> 2.50; read @ 0.10 -> 0.10; write @ 2.50 -> 2.50; output @ 10.00 -> 10.00.
        let cost = compute_cost(
            1_000_000, 1_000_000, 1_000_000, 1_000_000, 1_000_000, "test", &profile,
        );
        assert_eq!(cost.total_input_tokens, 1_000_000);
        assert_eq!(cost.fresh_input_tokens, 1_000_000);
        assert!((cost.fresh_input_cost - 2.50).abs() < 1e-9);
        assert!((cost.cache_read_cost - 0.10).abs() < 1e-9);
        assert!((cost.cache_write_cost - 2.50).abs() < 1e-9);
        assert!((cost.output_cost - 10.00).abs() < 1e-9);
        assert!((cost.total_cost - 15.10).abs() < 1e-9);
    }

    #[test]
    fn derive_fresh_is_saturating() {
        assert_eq!(derive_fresh_input(100, 60, 30), 10);
        assert_eq!(derive_fresh_input(100, 200, 0), 0);
        assert_eq!(derive_fresh_input(100, 60, 0), 40);
    }

    #[test]
    fn cached_tokens_are_not_double_charged() {
        let profile = default_synthetic_profile();
        // 100 total, 90 cached: cost = 10 fresh @ 2.50 + 90 read @ 0.10
        let cost = compute_cost(100, 10, 90, 0, 0, "test", &profile);
        let expected = 10.0 / PER_MILLION * 2.50 + 90.0 / PER_MILLION * 0.10;
        assert!((cost.total_cost - expected).abs() < 1e-12);
        assert_eq!(cost.fresh_input_derivation, "test");
    }

    #[test]
    fn normalized_usage_is_billed_explicitly() {
        use crate::usage::NormalizedUsage;
        let profile = default_synthetic_profile();
        let normalized = NormalizedUsage {
            total_input_tokens: Some(1000),
            fresh_input_tokens: Some(200),
            cache_read_tokens: Some(700),
            cache_write_tokens: Some(100),
            output_tokens: Some(50),
            normalization_source: "synthetic".to_string(),
            explanation: "test".to_string(),
        };
        let cost = compute_cost_normalized(&normalized, &profile);
        assert_eq!(cost.total_input_tokens, 1000);
        assert_eq!(cost.fresh_input_tokens, 200);
        assert!(cost.fresh_input_derivation.contains("synthetic"));
    }

    #[test]
    fn profile_version_is_checked() {
        let mut profile = default_synthetic_profile();
        profile.version = 99;
        assert!(validate_cost_profile(&profile)
            .unwrap_err()
            .contains("unsupported profile version"));
    }

    #[test]
    fn cache_is_worthwhile_when_reuse_is_large_and_cheap() {
        let profile = default_synthetic_profile();
        let e = evaluate_cache_economics(10_000, 9_500, 500, &profile);
        assert!(e.cache_worthwhile);
    }

    #[test]
    fn cache_is_not_worthwhile_when_writes_are_expensive() {
        let mut profile = default_synthetic_profile();
        profile.cache_write_price_per_1m = 10.00;
        let e = evaluate_cache_economics(10_000, 500, 9_500, &profile);
        assert!(!e.cache_worthwhile);
    }

    #[test]
    fn deterministic_format() {
        assert_eq!(format_cost(0.1234567), "0.123457");
    }
}
