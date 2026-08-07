//! Cost arithmetic over externally supplied [`CostProfile`] values, plus a
//! theoretical cache-economics evaluation.
//!
//! Pricing is **data**, never hard-coded fact. Phase 0 only uses synthetic
//! profiles; real profiles belong to a later, audited phase.

use crate::model::CostProfile;

/// One million, the denominator for all per-1M prices.
pub const PER_MILLION: f64 = 1_000_000.0;

/// A breakdown of estimated cost for one request under one profile.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CostBreakdown {
    /// The profile name used.
    pub profile_name: String,
    /// Whether the profile is marked synthetic.
    pub synthetic: bool,
    /// The currency code.
    pub currency: String,
    /// Input tokens (the full request context).
    pub input_tokens: u64,
    /// Cache-read tokens.
    pub cache_read_tokens: u64,
    /// Cache-write tokens.
    pub cache_write_tokens: u64,
    /// Output tokens.
    pub output_tokens: u64,
    /// Fresh (non-cached) input tokens: `input - cache_read - cache_write`.
    pub fresh_tokens: u64,
    /// Cost of the fresh (non-cached) input tokens.
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

/// Compute a cost breakdown from raw token counts.
///
/// Billing model: cached tokens are charged at the cache-read price *instead
/// of* the input price. Fresh tokens are the remainder. Output is charged at
/// the output price.
///
/// ```text
/// fresh = input - cache_read - cache_write   (saturating)
/// cost  = fresh * input_price + cache_read * cache_read_price
///       + cache_write * cache_write_price + output * output_price
/// ```
pub fn compute_cost(
    input: u64,
    cache_read: u64,
    cache_write: u64,
    output: u64,
    profile: &CostProfile,
) -> CostBreakdown {
    let fresh_tokens = input.saturating_sub(cache_read.saturating_add(cache_write));
    let fresh_input_cost = fresh_tokens as f64 / PER_MILLION * profile.input_price_per_1m;
    let cache_read_cost = cache_read as f64 / PER_MILLION * profile.cache_read_price_per_1m;
    let cache_write_cost = cache_write as f64 / PER_MILLION * profile.cache_write_price_per_1m;
    let output_cost = output as f64 / PER_MILLION * profile.output_price_per_1m;
    let total_cost = fresh_input_cost + cache_read_cost + cache_write_cost + output_cost;

    CostBreakdown {
        profile_name: profile.name.clone(),
        synthetic: profile.synthetic,
        currency: profile.currency.clone(),
        input_tokens: input,
        cache_read_tokens: cache_read,
        cache_write_tokens: cache_write,
        output_tokens: output,
        fresh_tokens,
        fresh_input_cost,
        cache_read_cost,
        cache_write_cost,
        output_cost,
        total_cost,
    }
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
        // input 1M, read 1M, write 1M: fresh = 0.
        // read @ 0.10 -> 0.10; write @ 2.50 -> 2.50; output @ 10.00 -> 10.00.
        let cost = compute_cost(1_000_000, 1_000_000, 1_000_000, 1_000_000, &profile);
        assert_eq!(cost.fresh_tokens, 0);
        assert!((cost.fresh_input_cost - 0.0).abs() < 1e-9);
        assert!((cost.cache_read_cost - 0.10).abs() < 1e-9);
        assert!((cost.cache_write_cost - 2.50).abs() < 1e-9);
        assert!((cost.output_cost - 10.00).abs() < 1e-9);
        assert!((cost.total_cost - 12.60).abs() < 1e-9);
    }

    #[test]
    fn fresh_tokens_are_saturating() {
        let profile = default_synthetic_profile();
        let cost = compute_cost(100, 60, 30, 5, &profile);
        assert_eq!(cost.fresh_tokens, 10);
        let cost = compute_cost(100, 200, 0, 0, &profile);
        assert_eq!(cost.fresh_tokens, 0);
    }

    #[test]
    fn cached_tokens_are_not_double_charged() {
        let profile = default_synthetic_profile();
        // 100 tokens, 90 cached: cost = 10 fresh @ 2.50 + 90 read @ 0.10
        let cost = compute_cost(100, 90, 0, 0, &profile);
        let expected = 10.0 / PER_MILLION * 2.50 + 90.0 / PER_MILLION * 0.10;
        assert!((cost.total_cost - expected).abs() < 1e-12);
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
