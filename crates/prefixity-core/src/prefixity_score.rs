//! The experimental "prefixity" score.
//!
//! `prefixity(block)` is an **explainable estimate** of how suitable a
//! context block is for inclusion in a stable, reusable request prefix.
//!
//! This is deliberately **not** a probability, is **not** produced by machine
//! learning, and is **provisional**. Every score carries the reasons and the
//! numeric signals that produced it, so the result can be audited by a human.
//!
//! The heuristic combines:
//!
//! * a baseline score derived from the block's `source` type;
//! * penalties for blocks explicitly marked `optional` / `stale`;
//! * a small adjustment from the observed `lifetime` when recorded;
//! * a note when a block is marked `required` (which policies must honour
//!   regardless of the score).
//!
//! All thresholds and multipliers are documented constants below so that
//! results are reproducible across versions.

use crate::model::ContextBlock;
use std::collections::BTreeMap;

/// Blocks scoring at or above this value are considered "stable" for the
/// theoretical reusable-prefix estimate.
pub const STABLE_THRESHOLD: f64 = 0.5;

/// Score assigned to unknown `source` values: a conservative neutral value.
pub const UNKNOWN_SOURCE_SCORE: f64 = 0.40;

/// Multiplier applied when a block is explicitly marked `optional`.
pub const OPTIONAL_PENALTY: f64 = 0.7;

/// Multiplier applied when a block is explicitly marked `stale`.
pub const STALE_PENALTY: f64 = 0.5;

/// Lifetime (in turns) at or above which a small stability bonus applies.
pub const LONG_LIFETIME_TURNS: u64 = 100;

/// Lifetime (in turns) at or below which a small instability penalty applies.
pub const SHORT_LIFETIME_TURNS: u64 = 5;

/// Multiplier applied for a long observed lifetime.
pub const LONG_LIFETIME_BONUS: f64 = 1.05;

/// Multiplier applied for a short observed lifetime.
pub const SHORT_LIFETIME_PENALTY: f64 = 0.85;

/// The explainable score for one block.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PrefixityScore {
    /// The block this score describes.
    pub block_id: String,
    /// The final score in `[0.0, 1.0]`.
    pub score: f64,
    /// Human-readable reasons, in the order they were applied.
    pub reasons: Vec<String>,
    /// The numeric signals used, keyed by name (for machine inspection).
    pub signals: BTreeMap<String, f64>,
}

/// Baseline score for a known `source` type. Returns `None` for unknown types.
pub fn base_score_for_source(source: &str) -> Option<f64> {
    match source {
        "system" | "system_policy" | "system-policy" => Some(0.99),
        "tool_definition" | "tool_definitions" | "tools" | "tool-definition" => Some(0.96),
        "project_instructions" | "project-instructions" | "instructions" => Some(0.91),
        "conversation" | "conversation_history" | "history" => Some(0.80),
        "repository_map" | "repository-map" | "repo_map" => Some(0.74),
        "file_content" | "file_read" | "file-content" | "file-read" => Some(0.48),
        "git_status" | "git-status" => Some(0.09),
        "tool_result" | "tool-result" | "tool_output" | "tool-output" => Some(0.02),
        "user" | "user_request" | "user-request" => Some(0.00),
        "timestamp" | "time" | "clock" => Some(0.05),
        _ => None,
    }
}

/// Compute the explainable prefixity score for a block.
///
/// The function is pure: identical blocks always produce identical scores.
pub fn prefixity_score(block: &ContextBlock) -> PrefixityScore {
    let mut score;
    let mut reasons = Vec::new();
    let mut signals = BTreeMap::new();

    match base_score_for_source(&block.source) {
        Some(base) => {
            score = base;
            signals.insert("base_source".to_string(), base);
            reasons.push(format!(
                "source '{}' has baseline stability {base:.2}",
                block.source
            ));
        }
        None => {
            score = UNKNOWN_SOURCE_SCORE;
            signals.insert("base_source".to_string(), UNKNOWN_SOURCE_SCORE);
            reasons.push(format!(
                "unknown source '{}'; assigned neutral conservative score {UNKNOWN_SOURCE_SCORE:.2}",
                block.source
            ));
        }
    }

    if block.optional {
        score *= OPTIONAL_PENALTY;
        signals.insert("optional_penalty".to_string(), OPTIONAL_PENALTY);
        reasons.push(format!(
            "block is marked optional: score reduced by {:.0}%",
            (1.0 - OPTIONAL_PENALTY) * 100.0
        ));
    }
    if block.stale {
        score *= STALE_PENALTY;
        signals.insert("stale_penalty".to_string(), STALE_PENALTY);
        reasons.push(format!(
            "block is marked stale: score reduced by {:.0}%",
            (1.0 - STALE_PENALTY) * 100.0
        ));
    }
    match block.lifetime {
        Some(lifetime) => {
            signals.insert("lifetime".to_string(), lifetime as f64);
            if lifetime >= LONG_LIFETIME_TURNS {
                score *= LONG_LIFETIME_BONUS;
                reasons.push(format!(
                    "observed lifetime {lifetime} turns suggests reuse: score boosted 5%"
                ));
            } else if lifetime <= SHORT_LIFETIME_TURNS {
                score *= SHORT_LIFETIME_PENALTY;
                reasons.push(format!(
                    "observed lifetime {lifetime} turns is short: score reduced 15%"
                ));
            } else {
                reasons.push(format!("observed lifetime {lifetime} turns; no adjustment"));
            }
        }
        None => {
            reasons.push("no observed lifetime recorded; no adjustment".to_string());
        }
    }

    score = score.clamp(0.0, 1.0);
    signals.insert("final".to_string(), score);
    if block.required {
        reasons.push(
            "block is marked required: policies must retain it regardless of score".to_string(),
        );
    }

    PrefixityScore {
        block_id: block.id.clone(),
        score,
        reasons,
        signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ContextBlock;
    use std::collections::BTreeMap;

    fn block(
        source: &str,
        optional: bool,
        stale: bool,
        required: bool,
        lifetime: Option<u64>,
    ) -> ContextBlock {
        ContextBlock {
            id: format!("b-{source}"),
            source: source.to_string(),
            position: 0,
            content_hash: "0".repeat(64),
            token_count: Some(1),
            byte_count: 1,
            content: None,
            semantic_zone: None,
            structural_path: None,
            role: None,
            sensitivity: None,
            dependencies: Vec::new(),
            lifetime,
            optional,
            required,
            stale,
            metadata: BTreeMap::new(),
        }
    }

    fn score_of(
        source: &str,
        optional: bool,
        stale: bool,
        required: bool,
        lifetime: Option<u64>,
    ) -> f64 {
        prefixity_score(&block(source, optional, stale, required, lifetime)).score
    }

    #[test]
    fn system_policy_scores_high() {
        assert!((score_of("system_policy", false, false, false, None) - 0.99).abs() < 1e-9);
    }

    #[test]
    fn user_request_scores_zero() {
        assert_eq!(score_of("user_request", false, false, false, None), 0.0);
    }

    #[test]
    fn stale_optional_tool_result_is_volatile() {
        let s = score_of("tool_result", true, true, false, None);
        assert!((s - (0.02 * OPTIONAL_PENALTY * STALE_PENALTY)).abs() < 1e-9);
        assert!(s < STABLE_THRESHOLD);
    }

    #[test]
    fn unknown_source_is_neutral() {
        assert_eq!(
            score_of("mystery_type", false, false, false, None),
            UNKNOWN_SOURCE_SCORE
        );
    }

    #[test]
    fn long_lifetime_boosts() {
        let base = score_of("tool_definition", false, false, false, None);
        let boosted = score_of("tool_definition", false, false, false, Some(500));
        assert!(boosted > base);
    }

    #[test]
    fn short_lifetime_penalises() {
        let base = score_of("tool_definition", false, false, false, None);
        let penalised = score_of("tool_definition", false, false, false, Some(1));
        assert!(penalised < base);
    }

    #[test]
    fn score_is_explainable_and_deterministic() {
        let b = block("tool_result", true, false, false, Some(10));
        let s1 = prefixity_score(&b);
        let s2 = prefixity_score(&b);
        assert_eq!(s1, s2);
        assert!(!s1.reasons.is_empty());
        assert!(s1.reasons.iter().any(|r| r.contains("optional")));
    }

    #[test]
    fn required_blocks_are_noted() {
        let s = prefixity_score(&block("tool_result", true, true, true, None));
        assert!(s.reasons.iter().any(|r| r.contains("required")));
    }
}
