//! Comparison of two consecutive requests.
//!
//! Deterministically detects where the reusable prefix first diverges,
//! classifies every differing position (changed / added / removed /
//! reordered), and estimates the theoretically reusable tokens. When a cost
//! profile is supplied, a theoretical cache-economics evaluation is included.

use crate::analysis::TraceRef;
use crate::cost::{evaluate_cache_economics, CacheEconomics};
use crate::error::PrefixityError;
use crate::model::RequestTrace;
use crate::tokens::block_token_estimate;
use crate::validation::validate_trace;

/// How a position differs between two traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum DiffKind {
    /// The block at this position is identical in both traces.
    Unchanged,
    /// The block content changed in place.
    Changed,
    /// A block exists in trace B that did not exist at this position in A.
    Added,
    /// A block exists in trace A that does not exist at this position in B.
    Removed,
    /// The same block appears at a different position (an ordering change).
    Reordered,
}

/// The first position at which the two traces diverge.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DivergencePoint {
    /// Position of the first differing block.
    pub position: usize,
    /// The block ID in the earlier trace, if any.
    pub previous_block_id: String,
    /// The block ID in the later trace, if any.
    pub current_block_id: String,
    /// The earlier trace's hash at this position, if any.
    pub previous_hash: String,
    /// The later trace's hash at this position, if any.
    pub current_hash: String,
    /// What kind of divergence this is.
    pub kind: DiffKind,
    /// Human-readable explanation.
    pub explanation: String,
}

/// One differing position between the two traces.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BlockDiff {
    /// Position of the difference.
    pub position: usize,
    /// What kind of difference this is.
    pub kind: DiffKind,
    /// The earlier trace's block ID at this position, if any.
    pub previous_block_id: Option<String>,
    /// The later trace's block ID at this position, if any.
    pub current_block_id: Option<String>,
    /// The earlier trace's hash at this position, if any.
    pub previous_hash: Option<String>,
    /// The later trace's hash at this position, if any.
    pub current_hash: Option<String>,
    /// Human-readable explanation.
    pub explanation: String,
}

/// The full comparison result between two traces.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Comparison {
    /// Identity of the earlier trace.
    pub trace_a: TraceRef,
    /// Identity of the later trace.
    pub trace_b: TraceRef,
    /// Number of blocks in trace A.
    pub blocks_a: usize,
    /// Number of blocks in trace B.
    pub blocks_b: usize,
    /// `true` when both traces are structurally identical.
    pub identical: bool,
    /// The first divergence point, if any.
    pub first_divergence: Option<DivergencePoint>,
    /// Every differing position.
    pub changed_blocks: Vec<BlockDiff>,
    /// Number of positions that matched.
    pub unchanged_block_count: usize,
    /// Estimated tokens of the matching prefix (theoretical cache-read).
    pub reusable_prefix_tokens: u64,
    /// Estimated tokens of trace B from the first divergence onward.
    pub tokens_after_divergence: u64,
    /// Estimated tokens of trace B at differing positions (theoretical
    /// cache-write).
    pub theoretical_cache_write_tokens: u64,
    /// Cache-economics evaluation, when a profile was supplied.
    pub cache_economics: Option<CacheEconomics>,
    /// Non-fatal findings (e.g. provider/model mismatch).
    pub warnings: Vec<String>,
    /// Multi-line, human-readable explanation.
    pub explanation: String,
}

/// Compare trace `b` (later) against trace `a` (earlier).
///
/// Blocks are compared position by position. `profile`, if supplied, adds the
/// `cache_economics` section.
pub fn compare_traces(
    a: &RequestTrace,
    b: &RequestTrace,
    profile: Option<&crate::model::CostProfile>,
) -> Result<Comparison, PrefixityError> {
    let report_a = validate_trace(a, None)?;
    let report_b = validate_trace(b, None)?;
    let mut warnings = report_a.warnings;
    warnings.extend(report_b.warnings);

    if a.provider != b.provider {
        warnings.push(format!(
            "traces use different providers ('{}' vs '{}')",
            a.provider, b.provider
        ));
    }
    if a.model != b.model {
        warnings.push(format!(
            "traces use different models ('{}' vs '{}')",
            a.model, b.model
        ));
    }

    let max_len = a.blocks.len().max(b.blocks.len());
    let mut diffs: Vec<BlockDiff> = Vec::new();
    let mut unchanged_count = 0usize;
    let mut first_divergence: Option<DivergencePoint> = None;
    let mut reusable_prefix_tokens = 0u64;
    let mut identical = a.blocks.len() == b.blocks.len();

    for position in 0..max_len {
        let (kind, explanation) = classify(a, b, position);
        if kind == DiffKind::Unchanged {
            unchanged_count += 1;
            if first_divergence.is_none() {
                if let Some(block) = b.blocks.get(position) {
                    reusable_prefix_tokens = reusable_prefix_tokens
                        .saturating_add(block_token_estimate(block).unwrap_or(0));
                }
            }
        } else {
            identical = false;
            let previous = a.blocks.get(position);
            let current = b.blocks.get(position);
            if first_divergence.is_none() {
                first_divergence = Some(DivergencePoint {
                    position,
                    previous_block_id: previous.map(|x| x.id.clone()).unwrap_or_default(),
                    current_block_id: current.map(|x| x.id.clone()).unwrap_or_default(),
                    previous_hash: previous.map(|x| x.content_hash.clone()).unwrap_or_default(),
                    current_hash: current.map(|x| x.content_hash.clone()).unwrap_or_default(),
                    kind,
                    explanation: explanation.clone(),
                });
            }
            diffs.push(BlockDiff {
                position,
                kind,
                previous_block_id: previous.map(|x| x.id.clone()),
                current_block_id: current.map(|x| x.id.clone()),
                previous_hash: previous.map(|x| x.content_hash.clone()),
                current_hash: current.map(|x| x.content_hash.clone()),
                explanation,
            });
        }
    }

    let tokens_after_divergence = match &first_divergence {
        Some(d) => b.blocks[d.position..]
            .iter()
            .map(|block| block_token_estimate(block).unwrap_or(0))
            .sum(),
        None => 0,
    };

    let theoretical_cache_write_tokens = diffs
        .iter()
        .filter(|d| d.kind != DiffKind::Removed)
        .filter_map(|d| b.blocks.get(d.position))
        .map(|block| block_token_estimate(block).unwrap_or(0))
        .sum();

    let cache_economics = profile.map(|p| {
        let input_b: u64 = b
            .blocks
            .iter()
            .map(|block| block_token_estimate(block).unwrap_or(0))
            .sum();
        evaluate_cache_economics(
            input_b,
            reusable_prefix_tokens,
            theoretical_cache_write_tokens,
            p,
        )
    });

    let explanation = build_explanation(
        a,
        b,
        &first_divergence,
        &diffs,
        reusable_prefix_tokens,
        tokens_after_divergence,
    );

    Ok(Comparison {
        trace_a: TraceRef::from_trace(a),
        trace_b: TraceRef::from_trace(b),
        blocks_a: a.blocks.len(),
        blocks_b: b.blocks.len(),
        identical,
        first_divergence,
        changed_blocks: diffs,
        unchanged_block_count: unchanged_count,
        reusable_prefix_tokens,
        tokens_after_divergence,
        theoretical_cache_write_tokens,
        cache_economics,
        warnings,
        explanation,
    })
}

/// Classify the difference (if any) at `position` between `a` and `b`.
///
/// Reordering is detected by searching the *full* other trace for the same
/// block (by hash), so swaps like `A,B,C -> A,C,B` are classified as
/// reorders at both affected positions. When a hash occurs more than once,
/// the first occurrence is used (acceptable for Phase 0).
fn classify(a: &RequestTrace, b: &RequestTrace, position: usize) -> (DiffKind, String) {
    match (a.blocks.get(position), b.blocks.get(position)) {
        (Some(x), Some(y)) => {
            if x.content_hash == y.content_hash {
                (DiffKind::Unchanged, "identical".to_string())
            } else if let Some(from) = a
                .blocks
                .iter()
                .position(|blk| blk.content_hash == y.content_hash)
            {
                // The current B block exists elsewhere in A: it moved here.
                (
                    DiffKind::Reordered,
                    format!(
                        "block '{}' moved from position {from} to {position} (ordering changed)",
                        y.id
                    ),
                )
            } else if let Some(to) = b
                .blocks
                .iter()
                .position(|blk| blk.content_hash == x.content_hash)
            {
                // The current A block exists elsewhere in B: it moved away.
                (
                    DiffKind::Reordered,
                    format!(
                        "block '{}' moved from position {position} to {to} (ordering changed)",
                        x.id
                    ),
                )
            } else {
                (
                    DiffKind::Changed,
                    format!("content changed in block '{}' at position {position}", y.id),
                )
            }
        }
        (Some(x), None) => (
            DiffKind::Removed,
            format!("block '{}' removed (present in trace A only)", x.id),
        ),
        (None, Some(y)) => (
            DiffKind::Added,
            format!("block '{}' added (present in trace B only)", y.id),
        ),
        (None, None) => unreachable!("position is bounded by max_len"),
    }
}

/// Build the human-readable, deterministic explanation text.
fn build_explanation(
    a: &RequestTrace,
    b: &RequestTrace,
    first: &Option<DivergencePoint>,
    diffs: &[BlockDiff],
    reusable: u64,
    after: u64,
) -> String {
    let mut lines = Vec::new();
    lines.push("Cache divergence:".to_string());
    lines.push(format!("request {} -> {}", a.request_id, b.request_id));
    lines.push(String::new());

    match first {
        Some(d) => {
            lines.push("First changed block:".to_string());
            lines.push(format!("{}[{}]", d.current_block_id, d.position));
            lines.push(String::new());
            lines.push("Previous hash:".to_string());
            lines.push(short_hash(&d.previous_hash));
            lines.push(String::new());
            lines.push("Current hash:".to_string());
            lines.push(short_hash(&d.current_hash));
            lines.push(String::new());
            lines.push("Reusable tokens before divergence:".to_string());
            lines.push(comma(reusable));
            lines.push(String::new());
            lines.push("Tokens after divergence:".to_string());
            lines.push(comma(after));
            lines.push(String::new());
            lines.push("Possible cause:".to_string());
            lines.push(d.explanation.clone());
            lines.push(String::new());
            lines.push("Block diffs:".to_string());
            for diff in diffs {
                lines.push(format!(
                    "  [{}] {:?}  {}",
                    diff.position, diff.kind, diff.explanation
                ));
            }
        }
        None => {
            lines.push("No divergence: the two traces are identical.".to_string());
        }
    }
    lines.join("\n")
}

/// Shorten a hash for display (keeps determinism).
fn short_hash(hash: &str) -> String {
    if hash.len() > 10 {
        format!("{}...", &hash[..8])
    } else {
        hash.to_string()
    }
}

/// Format a count with thousands separators for human output.
fn comma(value: u64) -> String {
    let text = value.to_string();
    let mut out = String::new();
    for (index, c) in text.chars().enumerate() {
        if index > 0 && (text.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash_content;
    use crate::model::{ContextBlock, TRACE_FORMAT_VERSION};
    use std::collections::BTreeMap;

    fn block(id: &str, position: usize, content: &str, tokens: u64) -> ContextBlock {
        ContextBlock {
            id: id.to_string(),
            source: "test".to_string(),
            position,
            content_hash: hash_content(content),
            token_count: Some(tokens),
            byte_count: content.len() as u64,
            content: Some(content.to_string()),
            sensitivity: None,
            dependencies: Vec::new(),
            lifetime: None,
            optional: false,
            required: false,
            stale: false,
            metadata: BTreeMap::new(),
        }
    }

    fn trace(request_id: &str, blocks: Vec<ContextBlock>) -> RequestTrace {
        RequestTrace {
            format_version: TRACE_FORMAT_VERSION,
            request_id: request_id.to_string(),
            session_id: None,
            timestamp: None,
            provider: "synthetic".to_string(),
            model: "synthetic-model".to_string(),
            blocks,
            usage: None,
            latency: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn identical_traces_show_no_divergence() {
        let t1 = trace(
            "a",
            vec![block("x", 0, "one", 10), block("y", 1, "two", 20)],
        );
        let t2 = trace(
            "b",
            vec![block("x", 0, "one", 10), block("y", 1, "two", 20)],
        );
        let c = compare_traces(&t1, &t2, None).unwrap();
        assert!(c.identical);
        assert!(c.first_divergence.is_none());
        assert_eq!(c.reusable_prefix_tokens, 30);
        assert_eq!(c.tokens_after_divergence, 0);
    }

    #[test]
    fn detects_first_divergence_and_reusable_prefix() {
        let t1 = trace(
            "a",
            vec![
                block("x", 0, "one", 10),
                block("y", 1, "two", 20),
                block("z", 2, "three", 30),
            ],
        );
        let t2 = trace(
            "b",
            vec![
                block("x", 0, "one", 10),
                block("y", 1, "CHANGED", 25),
                block("z", 2, "three", 30),
            ],
        );
        let c = compare_traces(&t1, &t2, None).unwrap();
        assert!(!c.identical);
        let d = c.first_divergence.unwrap();
        assert_eq!(d.position, 1);
        assert_eq!(d.kind, DiffKind::Changed);
        assert_eq!(c.reusable_prefix_tokens, 10);
        assert_eq!(c.tokens_after_divergence, 55);
    }

    #[test]
    fn detects_reordering() {
        let t1 = trace(
            "a",
            vec![
                block("x", 0, "one", 10),
                block("a", 1, "A", 5),
                block("b", 2, "B", 6),
            ],
        );
        let t2 = trace(
            "b",
            vec![
                block("x", 0, "one", 10),
                block("b", 1, "B", 6),
                block("a", 2, "A", 5),
            ],
        );
        let c = compare_traces(&t1, &t2, None).unwrap();
        let d = c.first_divergence.unwrap();
        assert_eq!(d.position, 1);
        assert_eq!(d.kind, DiffKind::Reordered);
        assert_eq!(c.reusable_prefix_tokens, 10);
        assert!(d.explanation.contains("moved from position 2 to 1"));
        assert!(c.explanation.contains("ordering changed"));
    }

    #[test]
    fn detects_full_swap_as_reorder_on_both_positions() {
        // A,B,C -> A,C,B: both swapped positions must be Reordered, not Changed.
        let t1 = trace(
            "a",
            vec![
                block("x", 0, "one", 10),
                block("a", 1, "A", 5),
                block("b", 2, "B", 6),
                block("c", 3, "C", 7),
            ],
        );
        let t2 = trace(
            "b",
            vec![
                block("x", 0, "one", 10),
                block("a", 1, "A", 5),
                block("c", 2, "C", 7),
                block("b", 3, "B", 6),
            ],
        );
        let c = compare_traces(&t1, &t2, None).unwrap();
        let reordered: Vec<_> = c
            .changed_blocks
            .iter()
            .filter(|d| d.kind == DiffKind::Reordered)
            .collect();
        assert_eq!(
            reordered.len(),
            2,
            "both swapped positions should be reorders"
        );
        assert!(c.explanation.contains("moved from position 3 to 2"));
        assert!(c.explanation.contains("moved from position 2 to 3"));
    }

    #[test]
    fn detects_added() {
        let t1 = trace("a", vec![block("x", 0, "one", 10)]);
        let t2 = trace(
            "b",
            vec![block("x", 0, "one", 10), block("new", 1, "two", 5)],
        );
        let c = compare_traces(&t1, &t2, None).unwrap();
        assert_eq!(c.first_divergence.unwrap().kind, DiffKind::Added);
    }

    #[test]
    fn detects_removed() {
        // trace B must be valid (>=1 block); simulate removal at position 1
        let t1 = trace("a", vec![block("x", 0, "one", 10), block("y", 1, "two", 5)]);
        let t2 = trace("b", vec![block("x", 0, "one", 10)]);
        let c = compare_traces(&t1, &t2, None).unwrap();
        assert_eq!(c.first_divergence.unwrap().kind, DiffKind::Removed);
    }

    #[test]
    fn economics_section_present_with_profile() {
        let t1 = trace(
            "a",
            vec![block("x", 0, "one", 10), block("y", 1, "two", 20)],
        );
        let t2 = trace(
            "b",
            vec![block("x", 0, "one", 10), block("y", 1, "CHANGED", 20)],
        );
        let profile = crate::cost::default_synthetic_profile();
        let c = compare_traces(&t1, &t2, Some(&profile)).unwrap();
        assert!(c.cache_economics.is_some());
        assert_eq!(c.cache_economics.as_ref().unwrap().reusable_tokens, 10);
    }

    #[test]
    fn explanation_is_deterministic() {
        let t1 = trace("a", vec![block("x", 0, "one", 10)]);
        let t2 = trace("b", vec![block("x", 0, "one", 10)]);
        let c1 = compare_traces(&t1, &t2, None).unwrap();
        let c2 = compare_traces(&t1, &t2, None).unwrap();
        assert_eq!(c1.explanation, c2.explanation);
    }
}
