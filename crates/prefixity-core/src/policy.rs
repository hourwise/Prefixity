//! Offline policy simulation.
//!
//! A [`Policy`] describes a *hypothetical* transformation of a trace's block
//! structure. Policies operate on metadata and structure only; they never
//! mutate the source trace (they return index-based decisions), and they are
//! research hypotheses, **not** production recommendations.
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

/// A block that a policy decided to relocate.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Relocation {
    /// Block ID.
    pub id: String,
    /// Original position in the trace.
    pub from_position: usize,
    /// New position in the simulated order.
    pub to_position: usize,
}

/// The structural decision produced by a policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyDecision {
    /// Indices into `trace.blocks`, in the simulated final order.
    pub retained: Vec<usize>,
    /// Blocks the policy removed (never a `required` block).
    pub removed: Vec<RemovedBlock>,
    /// Blocks whose position changed relative to the original order.
    pub relocations: Vec<Relocation>,
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
    /// Blocks relocated by the policy.
    pub relocated_blocks: Vec<Relocation>,
    /// Blocks removed by the policy.
    pub removed_blocks: Vec<RemovedBlock>,
    /// Original estimated tokens.
    pub original_tokens: u64,
    /// Simulated estimated tokens.
    pub simulated_tokens: u64,
    /// `simulated_tokens - original_tokens` (negative is a saving).
    pub token_difference: i64,
    /// Original theoretical reusable-prefix tokens.
    pub original_reusable_prefix_tokens: u64,
    /// Simulated theoretical reusable-prefix tokens.
    pub simulated_reusable_prefix_tokens: u64,
    /// `simulated_reusable - original_reusable`.
    pub reusable_prefix_difference: i64,
    /// Estimated original cost under the supplied profile.
    pub original_cost: CostBreakdown,
    /// Estimated simulated cost under the supplied profile.
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
            assumptions: vec![
                "No transformation is applied; the recorded order is kept.".to_string()
            ],
            warnings: Vec::new(),
        })
    }
}

/// Policy 2: `stable-prefix` — simulate placing historically stable blocks
/// before volatile blocks (deterministic sort by prefixity score, ties
/// broken by original position). All blocks are retained.
#[derive(Debug, Default)]
pub struct StablePrefixPolicy;

impl Policy for StablePrefixPolicy {
    fn name(&self) -> &'static str {
        "stable-prefix"
    }
    fn description(&self) -> &'static str {
        "Place historically stable blocks before volatile blocks; no blocks are removed."
    }
    fn decide(&self, trace: &RequestTrace) -> Result<PolicyDecision, PrefixityError> {
        let mut indices: Vec<usize> = (0..trace.blocks.len()).collect();
        indices.sort_by(|&i, &j| {
            let si = prefixity_score(&trace.blocks[i]).score;
            let sj = prefixity_score(&trace.blocks[j]).score;
            sj.partial_cmp(&si)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(i.cmp(&j))
        });
        let relocations = compute_relocations(&indices);
        Ok(PolicyDecision {
            retained: indices,
            removed: Vec::new(),
            relocations,
            assumptions: vec![
                "Relocating blocks does not change their semantics (research hypothesis only)."
                    .to_string(),
                "Stability is measured by the experimental prefixity score (>= 0.50).".to_string(),
            ],
            warnings: Vec::new(),
        })
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
/// `prune-stale-tool-output`, then the stable-first ordering of the
/// remaining blocks.
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
        let mut retained: Vec<usize> = (0..trace.blocks.len())
            .filter(|i| !removed_positions.contains(i))
            .collect();
        retained.sort_by(|&i, &j| {
            let si = prefixity_score(&trace.blocks[i]).score;
            let sj = prefixity_score(&trace.blocks[j]).score;
            sj.partial_cmp(&si)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(i.cmp(&j))
        });
        let relocations = compute_relocations(&retained);

        Ok(PolicyDecision {
            retained,
            removed,
            relocations,
            assumptions: vec![
                "Only explicitly flagged blocks are removed (optional+volatile, or stale tool output).".to_string(),
                "Required blocks are never removed.".to_string(),
                "Remaining blocks are ordered stable-first (research hypothesis only).".to_string(),
            ],
            warnings,
        })
    }
}

/// Compute relocations between the original index order and the retained
/// order (indices are original trace positions).
fn compute_relocations(retained: &[usize]) -> Vec<Relocation> {
    // `retained` is in final order; `retained[i]` is the original index now
    // at final position `i`. A block is relocated if final position !=
    // original position.
    let mut relocations = Vec::new();
    for (to_position, &original_index) in retained.iter().enumerate() {
        if to_position != original_index {
            relocations.push(Relocation {
                id: String::new(), // filled by caller context; see simulate_policy
                from_position: original_index,
                to_position,
            });
        }
    }
    relocations
}

/// Simulate `policy` on `trace` under `profile`.
///
/// The source trace is only ever read: decisions are index-based and the
/// simulated order is a separate structure, so `trace` is never mutated.
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
    let original_reusable = leading_stable_prefix_tokens(trace.blocks.iter().collect());
    let original_cost = compute_cost(original_tokens, original_reusable, 0, 0, profile);

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
    let simulated_reusable = leading_stable_prefix_tokens(simulated_blocks);
    let simulated_cost = compute_cost(simulated_tokens, simulated_reusable, 0, 0, profile);

    let token_difference = simulated_tokens as i64 - original_tokens as i64;
    let reusable_prefix_difference = simulated_reusable as i64 - original_reusable as i64;
    let cost_difference = simulated_cost.total_cost - original_cost.total_cost;

    // Attach block IDs to relocations.
    let relocated_blocks: Vec<Relocation> = decision
        .relocations
        .into_iter()
        .map(|r| Relocation {
            id: trace.blocks[r.from_position].id.clone(),
            from_position: r.from_position,
            to_position: r.to_position,
        })
        .collect();

    Ok(SimulationResult {
        policy: policy.name().to_string(),
        description: policy.description().to_string(),
        retained_blocks: decision
            .retained
            .iter()
            .map(|&i| trace.blocks[i].id.clone())
            .collect(),
        relocated_blocks,
        removed_blocks: decision.removed,
        original_tokens,
        simulated_tokens,
        token_difference,
        original_reusable_prefix_tokens: original_reusable,
        simulated_reusable_prefix_tokens: simulated_reusable,
        reusable_prefix_difference,
        original_cost,
        simulated_cost,
        cost_difference,
        assumptions: decision.assumptions,
        warnings: decision.warnings,
    })
}

/// Estimated tokens of the longest leading run of stable blocks.
fn leading_stable_prefix_tokens(blocks: Vec<&crate::model::ContextBlock>) -> u64 {
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
