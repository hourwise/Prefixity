//! Single-trace analysis.
//!
//! Given one valid trace, produce deterministic accounting: per-block
//! summaries with prefixity scores, theoretical stable/volatile split,
//! provider-reported usage reconciliation, fresh-context attribution, an
//! optional cost breakdown, and a conservative recommendation.

use crate::cost::{compute_cost, CostBreakdown};
use crate::error::PrefixityError;
use crate::model::{ProviderUsage, RequestTrace};
use crate::prefixity_score::{prefixity_score, PrefixityScore, STABLE_THRESHOLD};
use crate::tokens::block_token_estimate;
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
    /// `true` when the score is at or above [`STABLE_THRESHOLD`].
    pub stable: bool,
}

/// One contributor to fresh (non-cached) context, ordered by token weight.
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
    /// Whether the block is considered stable.
    pub stable: bool,
    /// Why this block is a fresh-input driver.
    pub explanation: String,
}

/// Reconciliation between provider-reported usage and Prefixity's
/// theoretical estimates. Per the source-of-truth principles, reported
/// values outrank theoretical ones when both exist.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct UsageReconciliation {
    /// Provider-reported input tokens, if any.
    pub reported_input_tokens: Option<u64>,
    /// Provider-reported cache-read tokens, if any.
    pub reported_cache_read_tokens: Option<u64>,
    /// Provider-reported cache-write tokens, if any.
    pub reported_cache_write_tokens: Option<u64>,
    /// Provider-reported output tokens, if any.
    pub reported_output_tokens: Option<u64>,
    /// Fresh tokens derived from provider-reported figures, if derivable.
    pub reported_fresh_tokens: Option<u64>,
    /// Sum of estimated tokens of stable blocks.
    pub theoretical_stable_tokens: u64,
    /// Sum of estimated tokens of volatile blocks.
    pub theoretical_volatile_tokens: u64,
    /// Longest leading run of stable blocks (theoretical reusable prefix).
    pub theoretical_reusable_prefix_tokens: u64,
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
    /// Provider-reported usage, if present.
    pub usage: Option<ProviderUsage>,
    /// Reconciliation of reported vs theoretical figures, if usage is present.
    pub reconciliation: Option<UsageReconciliation>,
    /// Sum of estimated tokens of stable blocks.
    pub theoretical_stable_tokens: u64,
    /// Sum of estimated tokens of volatile blocks.
    pub theoretical_volatile_tokens: u64,
    /// Longest leading run of stable blocks (theoretical reusable prefix).
    pub theoretical_reusable_prefix_tokens: u64,
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
/// supplied, produces the `cost` section. If the trace carries provider
/// usage, reported figures are used for the cost breakdown; otherwise
/// theoretical estimates are used.
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
    let mut reusable_prefix_tokens = 0u64;

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
                reusable_prefix_tokens = reusable_prefix_tokens.saturating_add(tokens);
            } else {
                leading_run_stable = false;
            }
        }

        blocks.push(BlockSummary {
            position: block.position,
            id: block.id.clone(),
            source: block.source.clone(),
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
                "volatile block (prefixity {:.2} < threshold {:.2})",
                b.prefixity, STABLE_THRESHOLD
            ),
        })
        .collect();
    top_fresh_blocks.sort_by(|x, y| y.tokens.cmp(&x.tokens).then(x.position.cmp(&y.position)));

    let usage = trace.usage.clone();
    let reconciliation = usage
        .as_ref()
        .map(|u| reconcile_usage(u, stable_tokens, volatile_tokens, reusable_prefix_tokens));

    let cost = profile.map(|p| {
        cost_for_analysis(
            total_estimated_tokens,
            reusable_prefix_tokens,
            usage.as_ref(),
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
        usage,
        reconciliation,
        theoretical_stable_tokens: stable_tokens,
        theoretical_volatile_tokens: volatile_tokens,
        theoretical_reusable_prefix_tokens: reusable_prefix_tokens,
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
    usage: &ProviderUsage,
    stable_tokens: u64,
    volatile_tokens: u64,
    reusable_prefix_tokens: u64,
) -> UsageReconciliation {
    let reported_fresh_tokens = match (
        usage.input_tokens,
        usage.cache_read_tokens,
        usage.cache_write_tokens,
    ) {
        (Some(input), Some(read), Some(write)) => Some(input.saturating_sub(read + write)),
        (Some(input), Some(read), None) => Some(input.saturating_sub(read)),
        _ => None,
    };
    let note = match (usage.cache_read_tokens, usage.input_tokens) {
        (Some(read), Some(input)) if input > 0 => {
            let fraction = read as f64 / input as f64 * 100.0;
            format!(
                "provider reported {read} cache-read tokens ({fraction:.1}% of input). \
                 Per source-of-truth principle 7, provider-reported usage outranks Prefixity's \
                 theoretical estimate of {reusable_prefix_tokens} reusable-prefix tokens."
            )
        }
        _ => format!(
            "provider reported no cache-read figure; Prefixity's theoretical estimate of \
             {reusable_prefix_tokens} reusable-prefix tokens stands."
        ),
    };
    UsageReconciliation {
        reported_input_tokens: usage.input_tokens,
        reported_cache_read_tokens: usage.cache_read_tokens,
        reported_cache_write_tokens: usage.cache_write_tokens,
        reported_output_tokens: usage.output_tokens,
        reported_fresh_tokens,
        theoretical_stable_tokens: stable_tokens,
        theoretical_volatile_tokens: volatile_tokens,
        theoretical_reusable_prefix_tokens: reusable_prefix_tokens,
        note,
    }
}

/// Choose the token figures used for the cost breakdown: provider-reported
/// when available, otherwise theoretical.
fn cost_for_analysis(
    total_estimated_tokens: u64,
    reusable_prefix_tokens: u64,
    usage: Option<&ProviderUsage>,
    profile: &crate::model::CostProfile,
) -> CostBreakdown {
    match usage {
        Some(u) => compute_cost(
            u.input_tokens.unwrap_or(total_estimated_tokens),
            u.cache_read_tokens.unwrap_or(reusable_prefix_tokens),
            u.cache_write_tokens.unwrap_or(0),
            u.output_tokens.unwrap_or(0),
            profile,
        ),
        None => compute_cost(
            total_estimated_tokens,
            reusable_prefix_tokens,
            0,
            0,
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
             ({stable_tokens} estimated stable tokens)"
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
    parts.join("; ")
}
