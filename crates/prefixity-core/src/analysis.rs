//! Single-trace analysis.
//!
//! Given one valid trace, produce deterministic accounting: per-block
//! summaries with prefixity scores, **heuristic** stable-prefix candidates,
//! provider usage normalisation and reconciliation, fresh-context
//! attribution, an optional cost breakdown, and a conservative
//! recommendation.
//!
//! IMPORTANT (Phase 0A.1): a single isolated trace **cannot prove that any
//! tokens were reused**. All figures derived from the prefixity score here
//! are labelled "candidate"/"heuristic" — never "reusable" or "cache-read".
//! Observed reuse requires a trace-to-trace comparison (see [`crate::compare`]);
//! provider-reported reuse requires normalized provider usage (see
//! [`crate::usage`]).

use crate::cost::{compute_cost, compute_cost_normalized, CostBreakdown};
use crate::error::PrefixityError;
use crate::model::{RawUsage, RequestTrace};
use crate::prefixity_score::{prefixity_score, PrefixityScore, STABLE_THRESHOLD};
use crate::structure::structural_fingerprint;
use crate::tokens::block_token_estimate;
use crate::usage::{normalize_usage, NormalizedUsage};
use crate::validation::validate_trace;

/// Lightweight identity of a trace, used in analysis and comparison output.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TraceRef {
    /// The trace's request ID.
    pub request_id: String,
    /// The optional session ID.
    pub session_id: Option<String>,
    /// The provider identifier.
    pub provider: String,
    /// The model identifier.
    pub model: String,
}

impl TraceRef {
    /// Build a [`TraceRef`] from a trace.
    pub fn from_trace(trace: &RequestTrace) -> Self {
        TraceRef {
            request_id: trace.request_id.clone(),
            session_id: trace.session_id.clone(),
            provider: trace.provider.clone(),
            model: trace.model.clone(),
        }
    }
}

/// Per-block summary produced by analysis.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BlockSummary {
    /// Position within the trace.
    pub position: usize,
    /// Block ID.
    pub id: String,
    /// Block source/type.
    pub source: String,
    /// Semantic zone (explicit, or `other` when absent).
    pub semantic_zone: String,
    /// Structural fingerprint used for prefix comparison.
    pub structural_fingerprint: String,
    /// Estimated tokens (may use the documented heuristic).
    pub tokens: u64,
    /// Recorded byte count.
    pub bytes: u64,
    /// Whether the block is marked optional.
    pub optional: bool,
    /// Whether the block is marked required.
    pub required: bool,
    /// Whether the block is marked stale.
    pub stale: bool,
    /// The experimental prefixity score.
    pub prefixity: f64,
    /// `true` when the score is at or above [`STABLE_THRESHOLD`] (a
    /// stable-prefix *candidate* — not observed reuse).
    pub stable: bool,
}

/// One contributor to fresh (non-cached) context, ordered by token weight.
///
/// This is a *heuristic* attribution based on the prefixity score; it does
/// not claim provider cache behavior.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FreshBlockContribution {
    /// Position within the trace.
    pub position: usize,
    /// Block ID.
    pub id: String,
    /// Block source/type.
    pub source: String,
    /// Estimated tokens.
    pub tokens: u64,
    /// Whether the block is considered a stable-prefix candidate.
    pub stable: bool,
    /// Why this block is a fresh-input driver (heuristic).
    pub explanation: String,
}

/// Reconciliation between provider-reported usage and Prefixity's heuristic
/// stable-prefix candidates. Per the source-of-truth principles, reported
/// values outrank candidates when both exist, and a single trace can never
/// prove reuse.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UsageReconciliation {
    /// Normalized provider-reported total input tokens, if any.
    pub reported_total_input_tokens: Option<u64>,
    /// Normalized provider-reported fresh input tokens, if any.
    pub reported_fresh_input_tokens: Option<u64>,
    /// Normalized provider-reported cache-read tokens, if any.
    pub reported_cache_read_tokens: Option<u64>,
    /// Normalized provider-reported cache-write tokens, if any.
    pub reported_cache_write_tokens: Option<u64>,
    /// Normalized provider-reported output tokens, if any.
    pub reported_output_tokens: Option<u64>,
    /// Which schema produced the normalized figures.
    pub normalization_source: String,
    /// Sum of estimated tokens of blocks scoring at/above the threshold
    /// (HEURISTIC stable-prefix candidates).
    pub stable_prefix_candidate_tokens: u64,
    /// Sum of estimated tokens of volatile blocks.
    pub volatile_tokens: u64,
    /// Longest leading run of stable-scoring blocks (HEURISTIC candidate
    /// prefix — NOT observed reuse).
    pub leading_stable_prefix_candidate_tokens: u64,
    /// Human-readable note about how the two views relate.
    pub note: String,
}

/// The full analysis result for a single trace.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TraceAnalysis {
    /// Identity of the analysed trace.
    pub trace: TraceRef,
    /// Trace format version.
    pub format_version: u32,
    /// Number of context blocks.
    pub block_count: usize,
    /// Sum of estimated tokens across all blocks.
    pub total_estimated_tokens: u64,
    /// `true` if any block required the heuristic token estimate.
    pub used_heuristic_token_estimates: bool,
    /// Sum of recorded byte counts.
    pub total_bytes: u64,
    /// Raw provider usage, preserved verbatim, if captured.
    pub raw_usage: Option<RawUsage>,
    /// Normalized provider usage, if raw usage was captured and a schema
    /// normalizer could derive values.
    pub normalized_usage: Option<NormalizedUsage>,
    /// Reconciliation of provider-reported vs heuristic candidate figures.
    pub reconciliation: Option<UsageReconciliation>,
    /// Estimated tokens of blocks scoring at/above the stable threshold
    /// (HEURISTIC candidates — NOT observed reuse).
    pub stable_prefix_candidate_tokens: u64,
    /// Estimated tokens of blocks scoring below the stable threshold.
    pub volatile_tokens: u64,
    /// Longest leading run of stable-scoring blocks (HEURISTIC candidate
    /// prefix — NOT observed reuse).
    pub leading_stable_prefix_candidate_tokens: u64,
    /// Per-block summaries in position order.
    pub blocks: Vec<BlockSummary>,
    /// Volatile blocks ordered by estimated tokens (fresh-input drivers).
    pub top_fresh_blocks: Vec<FreshBlockContribution>,
    /// Explainable prefixity scores for every block.
    pub prefixity_scores: Vec<PrefixityScore>,
    /// Estimated cost under the supplied profile, if one was given.
    pub cost: Option<CostBreakdown>,
    /// Non-fatal findings.
    pub warnings: Vec<String>,
    /// Conservative, deterministic recommendation text.
    pub recommendation: String,
}

/// Analyse a single trace.
///
/// The trace is validated first; an invalid trace is an error. `profile`, if
/// supplied, produces the `cost` section.
///
/// All stable-prefix figures are **heuristic candidates**. Without provider
/// usage, cost bills every input token as fresh input — candidates are never
/// silently billed at cache-read prices. With raw provider usage, cost uses
/// the normalized categories.
pub fn analyze_trace(
    trace: &RequestTrace,
    profile: Option<&crate::model::CostProfile>,
) -> Result<TraceAnalysis, PrefixityError> {
    let mut report = validate_trace(trace, None)?;
    let mut warnings = std::mem::take(&mut report.warnings);

    let mut blocks = Vec::with_capacity(trace.blocks.len());
    let mut prefixity_scores = Vec::with_capacity(trace.blocks.len());
    let mut total_estimated_tokens = 0u64;
    let mut total_bytes = 0u64;
    let mut used_heuristic_token_estimates = false;
    let mut stable_tokens = 0u64;
    let mut volatile_tokens = 0u64;
    let mut leading_run_stable = true;
    let mut leading_candidate_tokens = 0u64;

    for block in &trace.blocks {
        let estimate = block_token_estimate(block);
        let tokens = estimate.unwrap_or(0);
        match (estimate, block.token_count) {
            (None, _) => warnings.push(format!(
                "block '{}' has no token_count and no content; cannot estimate tokens",
                block.id
            )),
            (Some(_), None) => {
                used_heuristic_token_estimates = true;
                warnings.push(format!(
                    "block '{}' used heuristic token estimate (chars/4) because token_count is absent",
                    block.id
                ));
            }
            (Some(_), Some(_)) => {}
        }

        let score = prefixity_score(block);
        let stable = score.score >= STABLE_THRESHOLD;
        total_estimated_tokens = total_estimated_tokens.saturating_add(tokens);
        total_bytes = total_bytes.saturating_add(block.byte_count);
        if stable {
            stable_tokens = stable_tokens.saturating_add(tokens);
        } else {
            volatile_tokens = volatile_tokens.saturating_add(tokens);
        }
        if leading_run_stable {
            if stable {
                leading_candidate_tokens = leading_candidate_tokens.saturating_add(tokens);
            } else {
                leading_run_stable = false;
            }
        }

        blocks.push(BlockSummary {
            position: block.position,
            id: block.id.clone(),
            source: block.source.clone(),
            semantic_zone: crate::structure::zone_of(block).as_str().to_string(),
            structural_fingerprint: structural_fingerprint(block),
            tokens,
            bytes: block.byte_count,
            optional: block.optional,
            required: block.required,
            stale: block.stale,
            prefixity: score.score,
            stable,
        });
        prefixity_scores.push(score);
    }

    let mut top_fresh_blocks: Vec<FreshBlockContribution> = blocks
        .iter()
        .filter(|b| !b.stable)
        .map(|b| FreshBlockContribution {
            position: b.position,
            id: b.id.clone(),
            source: b.source.clone(),
            tokens: b.tokens,
            stable: b.stable,
            explanation: format!(
                "volatile block (prefixity {:.2} < threshold {:.2}; heuristic)",
                b.prefixity, STABLE_THRESHOLD
            ),
        })
        .collect();
    top_fresh_blocks.sort_by(|x, y| y.tokens.cmp(&x.tokens).then(x.position.cmp(&y.position)));

    let raw_usage = trace.usage.clone();
    let normalized_usage = raw_usage.as_ref().map(normalize_usage);
    if let Some(normalized) = &normalized_usage {
        if normalized.normalization_source == "unknown-schema" {
            warnings.push(normalized.explanation.clone());
        }
        if let (Some(total), Some(read)) =
            (normalized.total_input_tokens, normalized.cache_read_tokens)
        {
            if read > total {
                warnings.push(format!(
                    "normalized usage inconsistent: cache_read ({read}) > total input ({total})"
                ));
            }
        }
    }

    let reconciliation = normalized_usage
        .as_ref()
        .map(|n| reconcile_usage(n, stable_tokens, volatile_tokens, leading_candidate_tokens));

    let cost = profile.map(|p| {
        cost_for_analysis(
            total_estimated_tokens,
            leading_candidate_tokens,
            normalized_usage.as_ref(),
            p,
        )
    });

    let recommendation = build_recommendation(
        trace,
        stable_tokens,
        volatile_tokens,
        total_estimated_tokens,
    );

    Ok(TraceAnalysis {
        trace: TraceRef::from_trace(trace),
        format_version: trace.format_version,
        block_count: trace.blocks.len(),
        total_estimated_tokens,
        used_heuristic_token_estimates,
        total_bytes,
        raw_usage,
        normalized_usage,
        reconciliation,
        stable_prefix_candidate_tokens: stable_tokens,
        volatile_tokens,
        leading_stable_prefix_candidate_tokens: leading_candidate_tokens,
        blocks,
        top_fresh_blocks,
        prefixity_scores,
        cost,
        warnings,
        recommendation,
    })
}

/// Build the reconciliation note and figures.
fn reconcile_usage(
    normalized: &NormalizedUsage,
    stable_tokens: u64,
    volatile_tokens: u64,
    leading_candidate_tokens: u64,
) -> UsageReconciliation {
    let note = match normalized.cache_read_tokens {
        Some(read) => format!(
            "provider reported {read} cache-read tokens (normalized from schema '{}'). \
             A single trace cannot prove reuse: the {leading_candidate_tokens} leading stable-prefix \
             candidate tokens are heuristic only. Per source-of-truth principle 7, provider-reported \
             values outrank candidates when determining what actually happened.",
            normalized.normalization_source
        ),
        None => format!(
            "provider reported no cache-read figure. A single trace cannot prove reuse: the \
             {leading_candidate_tokens} leading stable-prefix candidate tokens are heuristic only."
        ),
    };
    UsageReconciliation {
        reported_total_input_tokens: normalized.total_input_tokens,
        reported_fresh_input_tokens: normalized.fresh_input_tokens,
        reported_cache_read_tokens: normalized.cache_read_tokens,
        reported_cache_write_tokens: normalized.cache_write_tokens,
        reported_output_tokens: normalized.output_tokens,
        normalization_source: normalized.normalization_source.clone(),
        stable_prefix_candidate_tokens: stable_tokens,
        volatile_tokens,
        leading_stable_prefix_candidate_tokens: leading_candidate_tokens,
        note,
    }
}

/// Choose the token figures used for the cost breakdown.
///
/// With normalized provider usage, explicit normalized categories are billed
/// (never the `total - read - write` assumption). Without provider usage, all
/// input is billed as fresh input — stable-prefix candidates are **not**
/// billed at cache-read prices because a single trace cannot prove reuse.
fn cost_for_analysis(
    total_estimated_tokens: u64,
    _leading_candidate_tokens: u64,
    normalized: Option<&NormalizedUsage>,
    profile: &crate::model::CostProfile,
) -> CostBreakdown {
    match normalized {
        Some(n) => compute_cost_normalized(n, profile),
        None => compute_cost(
            total_estimated_tokens,
            total_estimated_tokens,
            0,
            0,
            0,
            "no provider usage; all input billed as fresh input (candidates are NOT billed at cache-read prices)",
            profile,
        ),
    }
}

/// Build a conservative, deterministic recommendation.
fn build_recommendation(
    trace: &RequestTrace,
    stable_tokens: u64,
    volatile_tokens: u64,
    total_tokens: u64,
) -> String {
    let has_removable = trace
        .blocks
        .iter()
        .any(|b| (b.optional || b.stale) && !b.required);
    let has_required_flagged = trace
        .blocks
        .iter()
        .any(|b| (b.optional || b.stale) && b.required);
    let volatile_fraction = if total_tokens > 0 {
        volatile_tokens as f64 / total_tokens as f64
    } else {
        0.0
    };

    let mut parts: Vec<String> = Vec::new();
    if !has_removable && !has_required_flagged && volatile_fraction < 0.10 {
        parts.push(format!(
            "no structural change recommended: layout is already effectively stable-first \
             ({stable_tokens} estimated stable-prefix candidate tokens)"
        ));
    } else {
        if has_removable {
            parts.push(
                "optional/stale blocks are present; consider OFFLINE policy simulation (defer-volatile / prune-stale-tool-output). Do not modify live requests."
                    .to_string(),
            );
        }
        if has_required_flagged {
            parts.push(
                "some optional/stale blocks are marked required and must never be removed"
                    .to_string(),
            );
        }
        if volatile_fraction >= 0.10 {
            parts.push(format!(
                "{:.1}% of input tokens are estimated volatile; consider OFFLINE stable-prefix simulation",
                volatile_fraction * 100.0
            ));
        }
    }
    parts.push(
        "single-trace analysis cannot prove cache reuse; all stable-prefix figures are heuristic candidates"
            .to_string(),
    );
    parts.join("; ")
}
