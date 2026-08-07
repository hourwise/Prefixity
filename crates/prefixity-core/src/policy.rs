//! Offline policy simulation.
//!
//! A [`Policy`] describes a *hypothetical* transformation of a trace's block
//! structure. Policies operate on metadata and structure only; they never
//! mutate the source trace (they return index-based decisions), and they are
//! research hypotheses, **not** production recommendations.
//!
//! Phase 0A.1 adds **ordering constraints**:
//!
//! * blocks belong to semantic zones (see [`crate::structure`]);
//! * blocks never move across incompatible zones;
//! * chronological message order is preserved;
//! * required blocks remain pinned in place;
//! * transformations that may affect semantics are labelled
//!   UNSAFE/EXPERIMENTAL and are **not** applied — they are collected in
//!   [`PolicyDecision::unsafe_transformations_deferred`].
//!
//! The `compression` policy name is reserved for future work: compression
//! quality cannot be inferred from token counts, so no compression policy is
//! implemented in Phase 0.
//!
//! Extension point: implement [`Policy`] and register it in
//! [`policy_from_name`] to add a new simulation.

use crate::cost::{compute_cost, CostBreakdown};
use crate::error::PrefixityError;
use crate::model::RequestTrace;
use crate::prefixity_score::{prefixity_score, STABLE_THRESHOLD};
use crate::structure::{zone_of, SemanticZone};
use crate::tokens::block_token_estimate;
use crate::validation::validate_trace;
use std::fmt;

/// A block that a policy decided to remove.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RemovedBlock {
    /// Original position in the trace.
    pub position: usize,
    /// Block ID.
    pub id: String,
    /// Why the policy removed it.
    pub reason: String,
}

/// Safety label for an applied relocation.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum RelocationSafety {
    /// Structurally safe under the current constraints.
    Safe,
    /// Applied but labelled EXPERIMENTAL: reordering may affect semantics
    /// (research hypothesis only, never a live transformation).
    Experimental(String),
}

/// A block that a policy decided to relocate (within its zone).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Relocation {
    /// Block ID.
    pub id: String,
    /// Original position in the trace.
    pub from_position: usize,
    /// New position in the simulated order.
    pub to_position: usize,
    /// Safety label.
    pub safety: RelocationSafety,
}

/// The structural decision produced by a policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDecision {
    /// Indices into `trace.blocks`, in the simulated final order.
    pub retained: Vec<usize>,
    /// Blocks the policy removed (never a `required` block).
    pub removed: Vec<RemovedBlock>,
    /// Blocks relocated by the policy (all labelled; within-zone only).
    pub relocations: Vec<Relocation>,
    /// UNSAFE/EXPERIMENTAL transformations the policy considered but did NOT
    /// apply, with reasons.
    pub unsafe_transformations_deferred: Vec<String>,
    /// Assumptions the policy made.
    pub assumptions: Vec<String>,
    /// Non-fatal findings (e.g. a required block retained despite matching
    /// removal criteria).
    pub warnings: Vec<String>,
}

/// The result of running a policy on one trace.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SimulationResult {
    /// Policy name.
    pub policy: String,
    /// Policy description.
    pub description: String,
    /// Block IDs in the simulated final order.
    pub retained_blocks: Vec<String>,
    /// Blocks relocated by the policy (with safety labels).
    pub relocated_blocks: Vec<Relocation>,
    /// Blocks removed by the policy.
    pub removed_blocks: Vec<RemovedBlock>,
    /// UNSAFE/EXPERIMENTAL transformations the policy did not apply.
    pub unsafe_transformations_deferred: Vec<String>,
    /// Original estimated tokens.
    pub original_tokens: u64,
    /// Simulated estimated tokens.
    pub simulated_tokens: u64,
    /// `simulated_tokens - original_tokens` (negative is a saving).
    pub token_difference: i64,
    /// Original heuristic stable-prefix candidate tokens (NOT observed reuse).
    pub original_stable_prefix_candidate_tokens: u64,
    /// Simulated heuristic stable-prefix candidate tokens (NOT observed reuse).
    pub simulated_stable_prefix_candidate_tokens: u64,
    /// `simulated_candidate - original_candidate`.
    pub stable_prefix_candidate_difference: i64,
    /// Estimated original cost under the supplied profile (labelled
    /// hypothetical cache model).
    pub original_cost: CostBreakdown,
    /// Estimated simulated cost under the supplied profile (labelled
    /// hypothetical cache model).
    pub simulated_cost: CostBreakdown,
    /// `simulated_cost - original_cost` (negative is a saving).
    pub cost_difference: f64,
    /// Assumptions the policy made.
    pub assumptions: Vec<String>,
    /// Non-fatal findings.
    pub warnings: Vec<String>,
}

/// A simulation policy. Must be deterministic and never mutate its input.
pub trait Policy: fmt::Debug + Send + Sync {
    /// Canonical policy name (used by the CLI).
    fn name(&self) -> &'static str;
    /// One-line description of what the policy simulates.
    fn description(&self) -> &'static str;
    /// Produce a structural decision for `trace`.
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError>;
}

/// Policy 1: `baseline` — reproduce the recorded structure without any
/// optimisation. Used as the control for all simulations.
#[derive(Debug, Default)]
pub struct BaselinePolicy;

impl Policy for BaselinePolicy {
    fn name(&self) -> &'static str {
        "baseline"
    }
    fn description(&self) -> &'static str {
        "Reproduce the recorded block structure without optimisation (control)."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        Ok(PolicyDecision {
            retained: (0..trace.blocks.len()).collect(),
            removed: Vec::new(),
            relocations: Vec::new(),
            unsafe_transformations_deferred: Vec::new(),
            assumptions: vec![
                "No transformation is applied; the recorded order is kept.".to_string()
            ],
            warnings: Vec::new(),
        })
    }
}

/// Policy 2: `stable-prefix` — simulate placing historically stable blocks
/// before volatile blocks, **within their semantic zones only**.
///
/// Constraints applied:
///
/// * blocks never move across zones;
/// * chronological `messages` order is preserved;
/// * required blocks are pinned in place;
/// * applied within-zone relocations are labelled EXPERIMENTAL (reordering
///   may affect semantics) and are never a live recommendation;
/// * any transformation the constraints forbid is collected in
///   `unsafe_transformations_deferred`.
///
/// When no safe relocation exists, the policy reports
/// "No safe relocation is available under current structural constraints."
#[derive(Debug, Default)]
pub struct StablePrefixPolicy;

impl Policy for StablePrefixPolicy {
    fn name(&self) -> &'static str {
        "stable-prefix"
    }
    fn description(&self) -> &'static str {
        "Within-zone stable-first ordering; no cross-zone moves, chronological messages preserved, required blocks pinned."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        let mut decision = constrained_stable_first_order(trace);
        decision.assumptions = vec![
            "Blocks never move across semantic zones.".to_string(),
            "Chronological message order is preserved.".to_string(),
            "Required blocks are pinned in place.".to_string(),
            "Within-zone reordering may affect semantics and is EXPERIMENTAL only.".to_string(),
            "Stability is measured by the experimental prefixity score (>= 0.50).".to_string(),
        ];
        if decision.relocations.is_empty() && !decision.unsafe_transformations_deferred.is_empty() {
            decision.warnings.push(
                "No safe relocation is available under current structural constraints.".to_string(),
            );
        }
        Ok(decision)
    }
}

/// Policy 3: `defer-volatile` — simulate excluding blocks explicitly marked
/// `optional` that also score below the stability threshold. Required blocks
/// are never removed.
#[derive(Debug, Default)]
pub struct DeferVolatilePolicy;

impl Policy for DeferVolatilePolicy {
    fn name(&self) -> &'static str {
        "defer-volatile"
    }
    fn description(&self) -> &'static str {
        "Exclude blocks explicitly marked optional that are also volatile (prefixity < 0.50)."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        let mut removed = Vec::new();
        let mut warnings = Vec::new();
        let mut retained = Vec::new();
        for (index, block) in trace.blocks.iter().enumerate() {
            let score = prefixity_score(block).score;
            let matches = block.optional && score < STABLE_THRESHOLD;
            if matches && block.required {
                warnings.push(format!(
                    "required block '{}' matched defer-volatile criteria but was retained",
                    block.id
                ));
                retained.push(index);
            } else if matches {
                removed.push(RemovedBlock {
                    position: index,
                    id: block.id.clone(),
                    reason: "optional and volatile (prefixity below 0.50)".to_string(),
                });
            } else {
                retained.push(index);
            }
        }
        Ok(PolicyDecision {
            retained,
            removed,
            relocations: Vec::new(),
            unsafe_transformations_deferred: Vec::new(),
            assumptions: vec![
                "Only blocks explicitly marked optional are eligible for exclusion.".to_string(),
                "Required blocks are never removed.".to_string(),
            ],
            warnings,
        })
    }
}

/// Policy 4: `prune-stale-tool-output` — simulate removing tool-output
/// blocks explicitly marked stale. Required blocks are never removed.
#[derive(Debug, Default)]
pub struct PruneStaleToolOutputPolicy;

impl Policy for PruneStaleToolOutputPolicy {
    fn name(&self) -> &'static str {
        "prune-stale-tool-output"
    }
    fn description(&self) -> &'static str {
        "Remove tool-output blocks explicitly marked stale."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        let mut removed = Vec::new();
        let mut warnings = Vec::new();
        let mut retained = Vec::new();
        for (index, block) in trace.blocks.iter().enumerate() {
            let is_tool_output = matches!(
                block.source.as_str(),
                "tool_result" | "tool-result" | "tool_output" | "tool-output"
            );
            let matches = block.stale && is_tool_output;
            if matches && block.required {
                warnings.push(format!(
                    "required block '{}' matched prune-stale-tool-output criteria but was retained",
                    block.id
                ));
                retained.push(index);
            } else if matches {
                removed.push(RemovedBlock {
                    position: index,
                    id: block.id.clone(),
                    reason: "stale tool-output block".to_string(),
                });
            } else {
                retained.push(index);
            }
        }
        Ok(PolicyDecision {
            retained,
            removed,
            relocations: Vec::new(),
            unsafe_transformations_deferred: Vec::new(),
            assumptions: vec![
                "Only tool-output blocks explicitly marked stale are eligible for removal."
                    .to_string(),
                "Required blocks are never removed.".to_string(),
            ],
            warnings,
        })
    }
}

/// Policy 5: `combined` — apply only conservative, compatible
/// transformations: first the removals of `defer-volatile` and
/// `prune-stale-tool-output`, then the zone-constrained stable-first ordering
/// of the remaining blocks.
#[derive(Debug, Default)]
pub struct CombinedPolicy;

impl Policy for CombinedPolicy {
    fn name(&self) -> &'static str {
        "combined"
    }
    fn description(&self) -> &'static str {
        "Conservative combination: remove explicitly optional volatile and stale tool-output blocks, then order the remainder stable-first."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        let defer = DeferVolatilePolicy.decide(trace)?;
        let mut removed = defer.removed;
        let mut warnings = defer.warnings;

        // Apply prune criteria on top; a block removed by defer cannot be
        // removed again, but a stale tool-output block that was *not* optional
        // is still eligible.
        for (index, block) in trace.blocks.iter().enumerate() {
            let is_tool_output = matches!(
                block.source.as_str(),
                "tool_result" | "tool-result" | "tool_output" | "tool-output"
            );
            let already_removed = removed.iter().any(|r| r.position == index);
            let matches = block.stale && is_tool_output;
            if matches && !already_removed {
                if block.required {
                    warnings.push(format!(
                        "required block '{}' matched combined criteria but was retained",
                        block.id
                    ));
                } else {
                    removed.push(RemovedBlock {
                        position: index,
                        id: block.id.clone(),
                        reason: "stale tool-output block (combined policy)".to_string(),
                    });
                }
            }
        }

        let removed_positions: Vec<usize> = removed.iter().map(|r| r.position).collect();
        let retained_eligible: Vec<usize> = (0..trace.blocks.len())
            .filter(|i| !removed_positions.contains(i))
            .collect();
        let (retained, relocations, unsafe_transformations_deferred) =
            constrained_stable_first_order_of(trace, &retained_eligible, false);

        Ok(PolicyDecision {
            retained,
            removed,
            relocations,
            unsafe_transformations_deferred,
            assumptions: vec![
                "Only explicitly flagged blocks are removed (optional+volatile, or stale tool output).".to_string(),
                "Required blocks are never removed.".to_string(),
                "Remaining blocks are ordered stable-first within their zones (research hypothesis only).".to_string(),
            ],
            warnings,
        })
    }
}

/// Sort `indices` (original trace positions) by prefixity score descending,
/// ties broken by original position. Deterministic.
fn sort_by_score(trace: &RequestTrace, indices: &mut [usize]) {
    indices.sort_by(|&i, &j| {
        let si = prefixity_score(&trace.blocks[i]).score;
        let sj = prefixity_score(&trace.blocks[j]).score;
        sj.partial_cmp(&si)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(i.cmp(&j))
    });
}

/// Build a zone-constrained stable-first order for `eligible` indices of
/// `trace` (`eligible` must be in original position order).
///
/// Rules: blocks never move across zones, which is guaranteed by reordering
/// **only within maximal contiguous runs of the same zone**; chronological
/// `messages` order is preserved; required blocks are pinned at their
/// original positions within a run. Applied relocations are labelled
/// EXPERIMENTAL. When `report_cross_zone` is set, naive global reorders that
/// would cross zones are reported as deferred unsafe transformations
/// (informational only).
///
/// Returns `(final order, applied relocations, deferred unsafe moves)`.
fn constrained_stable_first_order_of(
    trace: &RequestTrace,
    eligible: &[usize],
    report_cross_zone: bool,
) -> (Vec<usize>, Vec<Relocation>, Vec<String>) {
    // Contiguous runs of the same zone, preserving original order.
    let mut runs: Vec<(SemanticZone, Vec<usize>)> = Vec::new();
    for &index in eligible {
        let zone = zone_of(&trace.blocks[index]);
        if let Some((current_zone, current_indices)) = runs.last_mut() {
            if *current_zone == zone {
                current_indices.push(index);
                continue;
            }
        }
        runs.push((zone, vec![index]));
    }

    let mut retained: Vec<usize> = Vec::with_capacity(eligible.len());
    let mut relocations: Vec<Relocation> = Vec::new();
    let mut deferred: Vec<String> = Vec::new();

    for (zone, indices) in runs {
        if zone.preserves_chronology() {
            // Detect a would-be score reorder and defer it.
            let mut sorted = indices.clone();
            sort_by_score(trace, &mut sorted);
            if sorted != indices {
                deferred.push(format!(
                    "intra-zone reorder in '{}' deferred: chronological message order must be preserved",
                    zone.as_str()
                ));
            }
            retained.extend(indices);
            continue;
        }

        // Sort non-required blocks by score; pin required blocks at their
        // original positions within the run.
        let run_len = indices.len();
        let mut slots: Vec<Option<usize>> = vec![None; run_len];
        for (slot_index, &orig) in indices.iter().enumerate() {
            if trace.blocks[orig].required {
                slots[slot_index] = Some(orig);
            }
        }
        let free_slots: Vec<usize> = (0..run_len).filter(|&k| slots[k].is_none()).collect();
        let mut sorted_non_required: Vec<usize> = indices
            .iter()
            .copied()
            .filter(|&i| !trace.blocks[i].required)
            .collect();
        sort_by_score(trace, &mut sorted_non_required);
        for (slot, index) in free_slots.iter().zip(sorted_non_required) {
            slots[*slot] = Some(index);
        }
        retained.extend(slots.into_iter().map(|s| s.expect("slot filled")));
    }

    // Record applied relocations with EXPERIMENTAL safety labels.
    for (to_position, &original_index) in retained.iter().enumerate() {
        if to_position != original_index {
            relocations.push(Relocation {
                id: trace.blocks[original_index].id.clone(),
                from_position: original_index,
                to_position,
                safety: RelocationSafety::Experimental(
                    "within-zone stable-first reorder may affect semantics (research hypothesis only)"
                        .to_string(),
                ),
            });
        }
    }

    // Report naive global reorders that would cross zones (not applied).
    if report_cross_zone {
        let mut naive: Vec<usize> = eligible.to_vec();
        sort_by_score(trace, &mut naive);
        for (position, &original_index) in naive.iter().enumerate() {
            if position == original_index {
                continue;
            }
            let from_zone = zone_of(&trace.blocks[original_index]);
            let occupant_zone = zone_of(&trace.blocks[position]);
            if from_zone != occupant_zone {
                deferred.push(format!(
                    "cross-zone move deferred: '{}' would move from zone '{}' into the position of a '{}' block; blocks never move across semantic zones",
                    trace.blocks[original_index].id,
                    from_zone.as_str(),
                    occupant_zone.as_str()
                ));
            }
        }
    }

    (retained, relocations, deferred)
}

/// Zone-constrained stable-first order over all blocks of `trace`.
fn constrained_stable_first_order(trace: &RequestTrace) -> PolicyDecision {
    let eligible: Vec<usize> = (0..trace.blocks.len()).collect();
    let (retained, relocations, unsafe_transformations_deferred) =
        constrained_stable_first_order_of(trace, &eligible, true);
    PolicyDecision {
        retained,
        removed: Vec::new(),
        relocations,
        unsafe_transformations_deferred,
        assumptions: Vec::new(),
        warnings: Vec::new(),
    }
}

/// Simulate `policy` on `trace` under `profile`.
///
/// The source trace is only ever read: decisions are index-based and the
/// simulated order is a separate structure, so `trace` is never mutated.
///
/// Costs use a labelled **hypothetical cache model**: stable-prefix
/// candidates are billed at the cache-read price and the remainder as fresh
/// input. This is a research model and is never presented as
/// provider-reported usage.
pub fn simulate_policy(
    trace: &RequestTrace,
    policy: &dyn Policy,
    profile: &crate::model::CostProfile,
) -> Result<SimulationResult, PrefixityError> {
    validate_trace(trace, None)?;
    let decision = policy.decide(trace)?;

    let original_tokens: u64 = trace
        .blocks
        .iter()
        .map(|b| block_token_estimate(b).unwrap_or(0))
        .sum();
    let original_candidate = leading_stable_prefix_candidate_tokens(trace.blocks.iter().collect());
    let original_cost = hypothetical_cost(original_tokens, original_candidate, profile);

    let simulated_tokens: u64 = decision
        .retained
        .iter()
        .map(|&i| block_token_estimate(&trace.blocks[i]).unwrap_or(0))
        .sum();
    let simulated_blocks: Vec<&crate::model::ContextBlock> = decision
        .retained
        .iter()
        .map(|&i| &trace.blocks[i])
        .collect();
    let simulated_candidate = leading_stable_prefix_candidate_tokens(simulated_blocks);
    let simulated_cost = hypothetical_cost(simulated_tokens, simulated_candidate, profile);

    let token_difference = simulated_tokens as i64 - original_tokens as i64;
    let stable_prefix_candidate_difference = simulated_candidate as i64 - original_candidate as i64;
    let cost_difference = simulated_cost.total_cost - original_cost.total_cost;

    Ok(SimulationResult {
        policy: policy.name().to_string(),
        description: policy.description().to_string(),
        retained_blocks: decision
            .retained
            .iter()
            .map(|&i| trace.blocks[i].id.clone())
            .collect(),
        relocated_blocks: decision.relocations,
        removed_blocks: decision.removed,
        unsafe_transformations_deferred: decision.unsafe_transformations_deferred,
        original_tokens,
        simulated_tokens,
        token_difference,
        original_stable_prefix_candidate_tokens: original_candidate,
        simulated_stable_prefix_candidate_tokens: simulated_candidate,
        stable_prefix_candidate_difference,
        original_cost,
        simulated_cost,
        cost_difference,
        assumptions: decision.assumptions,
        warnings: decision.warnings,
    })
}

/// Labelled hypothetical cost model for simulation.
fn hypothetical_cost(
    total: u64,
    candidate: u64,
    profile: &crate::model::CostProfile,
) -> CostBreakdown {
    let fresh = total.saturating_sub(candidate);
    compute_cost(
        total,
        fresh,
        candidate,
        0,
        0,
        "hypothetical cache model (stable-prefix candidates billed at read price; NOT provider-reported)",
        profile,
    )
}

/// Estimated tokens of the longest leading run of stable-scoring blocks
/// (heuristic stable-prefix candidates — NOT observed reuse).
fn leading_stable_prefix_candidate_tokens(blocks: Vec<&crate::model::ContextBlock>) -> u64 {
    let mut total = 0u64;
    for block in blocks {
        let score = prefixity_score(block).score;
        if score >= STABLE_THRESHOLD {
            total = total.saturating_add(block_token_estimate(block).unwrap_or(0));
        } else {
            break;
        }
    }
    total
}

/// Resolve a policy by its CLI name.
pub fn policy_from_name(name: &str) -> Result<Box<dyn Policy>, PrefixityError> {
    match name {
        "baseline" => Ok(Box::new(BaselinePolicy)),
        "stable-prefix" => Ok(Box::new(StablePrefixPolicy)),
        "defer-volatile" => Ok(Box::new(DeferVolatilePolicy)),
        "prune-stale-tool-output" => Ok(Box::new(PruneStaleToolOutputPolicy)),
        "combined" => Ok(Box::new(CombinedPolicy)),
        "compression" => Err(PrefixityError::Reserved {
            what: "policy 'compression' is reserved for future work; compression quality cannot be inferred from token counts, so it is not implemented in Phase 0.".to_string(),
        }),
        other => Err(PrefixityError::PolicyNotFound {
            name: other.to_string(),
        }),
    }
}

/// The names of policies available in Phase 0.
pub fn available_policies() -> &'static [&'static str] {
    &[
        "baseline",
        "stable-prefix",
        "defer-volatile",
        "prune-stale-tool-output",
        "combined",
    ]
}
