//! Phase 1B offline intervention decisions.
//!
//! This module is deliberately separate from [`crate::policy`]. The Phase 0
//! policies simulate structural alternatives and report their hypothetical
//! token effects; this module produces an auditable, fail-open recommendation
//! contract. A plan never changes the source trace and is never a live
//! prompt transformation.

use crate::analysis::{analyze_trace, BlockSummary, TraceAnalysis, TraceRef};
use crate::error::PrefixityError;
use crate::model::{ContextBlock, RequestTrace};
use crate::policy::{Policy, RelocationSafety, StablePrefixPolicy};
use crate::prefixity_score::{base_score_for_source, prefixity_score, STABLE_THRESHOLD};
use crate::structure::{zone_of, SemanticZone};
use crate::validation::validate_trace;
use std::collections::{BTreeMap, BTreeSet};

/// Version of the Phase 1B intervention-plan contract.
pub const INTERVENTION_PLAN_CONTRACT_VERSION: u32 = 1;

/// The complete set of Phase 1B intervention classes.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterventionClass {
    /// Retain the block in the recorded context.
    Keep,
    /// Leave an explicitly optional block out until it is needed.
    Defer,
    /// Hypothetically omit a block from a future request.
    Prune,
    /// Hypothetically move a block while preserving structural constraints.
    RelocateCandidate,
    /// Reserved contract class; the baseline never emits it.
    CompressCandidate,
    /// Make no context intervention.
    DoNothing,
}

impl InterventionClass {
    /// Return the stable wire spelling of the class.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Keep => "KEEP",
            Self::Defer => "DEFER",
            Self::Prune => "PRUNE",
            Self::RelocateCandidate => "RELOCATE_CANDIDATE",
            Self::CompressCandidate => "COMPRESS_CANDIDATE",
            Self::DoNothing => "DO_NOTHING",
        }
    }

    /// Return all contract classes in canonical order.
    pub const fn all() -> [Self; 6] {
        [
            Self::Keep,
            Self::Defer,
            Self::Prune,
            Self::RelocateCandidate,
            Self::CompressCandidate,
            Self::DoNothing,
        ]
    }
}

/// Stable reason codes attached to an auditable recommendation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasonCode {
    RequiredBlock,
    ProtocolCriticalBlock,
    CurrentRequest,
    OptionalStaleToolResult,
    OptionalVolatileToolResult,
    DependencyClosureProtected,
    UnknownDependencyEvidence,
    UnknownSafety,
    ChronologyProtected,
    CrossZoneRelocationRejected,
    WithinZoneRelocation,
    StructuralHeuristicNotSufficient,
    NoJustifiedIntervention,
    NoProviderEvidence,
    ProviderEvidenceNotUsedAsSafetyProof,
    NoEconomicEvidence,
    QualityEvidenceAbsent,
    CompressionNotEstablished,
}

impl ReasonCode {
    /// Return the stable wire spelling of the reason code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequiredBlock => "REQUIRED_BLOCK",
            Self::ProtocolCriticalBlock => "PROTOCOL_CRITICAL_BLOCK",
            Self::CurrentRequest => "CURRENT_REQUEST",
            Self::OptionalStaleToolResult => "OPTIONAL_STALE_TOOL_RESULT",
            Self::OptionalVolatileToolResult => "OPTIONAL_VOLATILE_TOOL_RESULT",
            Self::DependencyClosureProtected => "DEPENDENCY_CLOSURE_PROTECTED",
            Self::UnknownDependencyEvidence => "UNKNOWN_DEPENDENCY_EVIDENCE",
            Self::UnknownSafety => "UNKNOWN_SAFETY",
            Self::ChronologyProtected => "CHRONOLOGY_PROTECTED",
            Self::CrossZoneRelocationRejected => "CROSS_ZONE_RELOCATION_REJECTED",
            Self::WithinZoneRelocation => "WITHIN_ZONE_RELOCATION",
            Self::StructuralHeuristicNotSufficient => "STRUCTURAL_HEURISTIC_NOT_SUFFICIENT",
            Self::NoJustifiedIntervention => "NO_JUSTIFIED_INTERVENTION",
            Self::NoProviderEvidence => "NO_PROVIDER_EVIDENCE",
            Self::ProviderEvidenceNotUsedAsSafetyProof => {
                "PROVIDER_EVIDENCE_NOT_USED_AS_SAFETY_PROOF"
            }
            Self::NoEconomicEvidence => "NO_ECONOMIC_EVIDENCE",
            Self::QualityEvidenceAbsent => "QUALITY_EVIDENCE_ABSENT",
            Self::CompressionNotEstablished => "COMPRESSION_NOT_ESTABLISHED",
        }
    }
}

/// Strength of the evidence supporting the recommendation, not a task-quality
/// pass claim. The baseline never treats structural evidence as proof of
/// quality preservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceStrength {
    Unknown,
    Weak,
    Moderate,
    Strong,
}

/// Expected quality risk. Replay and task checks are not part of Phase 1B.0,
/// so every non-retention intervention remains unassessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QualityRisk {
    NoneForRetention,
    Unknown,
}

/// Whether provider cache state may affect the eventual value of a
/// hypothetical intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProviderStateDependence {
    NoneForRetention,
    PotentiallyRelevant,
    Unknown,
}

/// Evidence dimensions are intentionally separate. Empty provider/economic
/// dimensions are represented by an explicit absence note rather than an
/// inferred value.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceSources {
    /// Structural facts and existing heuristic observations.
    pub structural: Vec<String>,
    /// Provider/cache facts from the trace, if any.
    pub provider_cache: Vec<String>,
    /// Economic facts. The baseline has none because it accepts no profile.
    pub economic: Vec<String>,
    /// Quality/replay facts. Phase 1B.0 has no replay evidence.
    pub quality: Vec<String>,
    /// Dependency and closure facts.
    pub dependency: Vec<String>,
}

/// One auditable intervention recommendation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InterventionRecommendation {
    /// Recommendation class.
    pub class: InterventionClass,
    /// Target IDs. Empty only for a trace-level `DO_NOTHING` record.
    pub target_block_ids: Vec<String>,
    /// Deterministic machine-readable reasons.
    pub reason_codes: Vec<ReasonCode>,
    /// Concise human-readable explanation.
    pub explanation: String,
    /// Strength of the available evidence.
    pub evidence_strength: EvidenceStrength,
    /// Evidence separated by dimension.
    pub source_evidence: EvidenceSources,
    /// Direct and transitive dependency IDs relevant to this decision.
    pub relevant_dependencies: Vec<String>,
    /// Expected structural effect if a later phase were to evaluate it.
    pub expected_structural_effect: String,
    /// Expected quality risk before replay/evaluation.
    pub expected_quality_risk: QualityRisk,
    /// Whether provider/cache state may affect the eventual outcome.
    pub provider_state_dependence: ProviderStateDependence,
    /// True when raw provider/cache evidence exists in the source trace.
    pub provider_evidence_present: bool,
    /// True only when economic evidence was supplied to the planner. The
    /// Phase 1B.0 baseline never receives a cost profile.
    pub economic_evidence_present: bool,
    /// Every Phase 1B.0 recommendation is hypothetical.
    pub hypothetical_only: bool,
}

/// Complete deterministic plan for one trace.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InterventionPlan {
    /// Contract version for machine consumers.
    pub contract_version: u32,
    /// Source trace identity; no prompt content is copied into the plan.
    pub trace: TraceRef,
    /// One record per block when an intervention is proposed, otherwise one
    /// trace-level `DO_NOTHING` record.
    pub recommendations: Vec<InterventionRecommendation>,
    /// IDs retained by the hypothetical plan. The source trace is unchanged.
    pub retained_block_ids: Vec<String>,
    /// Fixed safety/integrity statements for this baseline.
    pub planner_notes: Vec<String>,
    /// Always true for this offline contract.
    pub hypothetical_only: bool,
}

struct PlannerContext<'a> {
    trace: &'a RequestTrace,
    analysis: &'a TraceAnalysis,
    dependencies: &'a DependencyGraph,
    provider_evidence_present: bool,
}

struct RecommendationSpec<'a> {
    class: InterventionClass,
    reason_codes: Vec<ReasonCode>,
    evidence_strength: EvidenceStrength,
    quality_risk: QualityRisk,
    provider_state_dependence: ProviderStateDependence,
    explanation: &'a str,
    expected_structural_effect: &'a str,
}

/// Build a conservative Phase 1B.0 plan from one valid trace.
pub fn plan_interventions(trace: &RequestTrace) -> Result<InterventionPlan, PrefixityError> {
    validate_trace(trace, None)?;
    let analysis = analyze_trace(trace, None)?;
    let dependencies = DependencyGraph::new(trace);
    let relocation_decision = StablePrefixPolicy.decide(trace)?;
    let provider_evidence_present = trace.usage.is_some();

    let context = PlannerContext {
        trace,
        analysis: &analysis,
        dependencies: &dependencies,
        provider_evidence_present,
    };
    let mut decisions: Vec<Option<InterventionRecommendation>> = vec![None; trace.blocks.len()];
    for (index, decision) in decisions.iter_mut().enumerate() {
        *decision = destructive_recommendation(&context, index);
    }

    // Relocation is only a hypothetical candidate. Do not combine it with
    // removal/deferral in this first baseline because the Phase 0 simulator's
    // relocation positions are based on the original order.
    let has_destructive = decisions.iter().any(|decision| {
        decision.as_ref().is_some_and(|recommendation| {
            matches!(
                recommendation.class,
                InterventionClass::Prune | InterventionClass::Defer
            )
        })
    });
    if !has_destructive && !dependencies.uncertain {
        for relocation in relocation_decision.relocations {
            let index = relocation.from_position;
            if decisions[index].is_some()
                || !relocation_is_allowed(context.trace, context.dependencies, &relocation)
            {
                continue;
            }
            let summary = &context.analysis.blocks[index];
            decisions[index] = Some(relocation_recommendation(&context, summary, relocation));
        }
    }

    let has_intervention = decisions.iter().any(Option::is_some);
    let recommendations = if has_intervention {
        decisions
            .into_iter()
            .enumerate()
            .map(|(index, recommendation)| {
                recommendation.unwrap_or_else(|| keep_recommendation(&context, index))
            })
            .collect()
    } else {
        vec![do_nothing_recommendation(
            &context,
            &relocation_decision.unsafe_transformations_deferred,
        )]
    };

    let retained_block_ids = trace
        .blocks
        .iter()
        .enumerate()
        .filter(|(index, _)| decisions_retain_index(&recommendations, trace, *index))
        .map(|(_, block)| block.id.clone())
        .collect();

    Ok(InterventionPlan {
        contract_version: INTERVENTION_PLAN_CONTRACT_VERSION,
        trace: TraceRef::from_trace(trace),
        recommendations,
        retained_block_ids,
        planner_notes: vec![
            "Offline only: this plan does not mutate the source trace or a live request."
                .to_string(),
            "Unknown or insufficient safety evidence defaults to retention or DO_NOTHING."
                .to_string(),
            "Phase 1A structural candidates, token size, repetition and non-gold status are not intervention proof."
                .to_string(),
            "COMPRESS_CANDIDATE is part of the contract but is not emitted by this baseline."
                .to_string(),
        ],
        hypothetical_only: true,
    })
}

fn destructive_recommendation(
    context: &PlannerContext<'_>,
    index: usize,
) -> Option<InterventionRecommendation> {
    let block = &context.trace.blocks[index];
    if block.required || is_protocol_critical(block) || is_current_request(block) {
        return None;
    }
    if context.dependencies.uncertain {
        return None;
    }
    let blocking_dependents = context
        .dependencies
        .blocking_dependents(context.trace, index);
    if !blocking_dependents.is_empty() {
        return None;
    }

    let summary = &context.analysis.blocks[index];
    let score = prefixity_score(block).score;
    if block.optional
        && block.stale
        && is_tool_result(block)
        && zone_of(block) != SemanticZone::Messages
    {
        return Some(intervention_recommendation(
            context,
            summary,
            RecommendationSpec {
                class: InterventionClass::Prune,
                reason_codes: vec![ReasonCode::OptionalStaleToolResult],
                evidence_strength: EvidenceStrength::Moderate,
                quality_risk: QualityRisk::Unknown,
                provider_state_dependence: ProviderStateDependence::PotentiallyRelevant,
                explanation: "The recorder explicitly marked this optional tool result stale and no recorded dependency requires it; a later phase may evaluate omission.",
                expected_structural_effect: "Hypothetically omit this block from a future request; retain the source trace unchanged.",
            },
        ));
    }

    // The score is only supporting volatility evidence. Optionality, source
    // type, zone and dependency closure are the safety gate for DEFER.
    if block.optional
        && !block.stale
        && is_tool_result(block)
        && score < STABLE_THRESHOLD
        && zone_of(block) != SemanticZone::Messages
    {
        return Some(intervention_recommendation(
            context,
            summary,
            RecommendationSpec {
                class: InterventionClass::Defer,
                reason_codes: vec![ReasonCode::OptionalVolatileToolResult],
                evidence_strength: EvidenceStrength::Weak,
                quality_risk: QualityRisk::Unknown,
                provider_state_dependence: ProviderStateDependence::PotentiallyRelevant,
                explanation: "The recorder explicitly marked this tool result optional; its existing structural score is only supporting volatility evidence for deferral.",
                expected_structural_effect: "Hypothetically leave this optional block out until requested; do not reorder or mutate the trace.",
            },
        ));
    }

    None
}

fn relocation_is_allowed(
    trace: &RequestTrace,
    dependencies: &DependencyGraph,
    relocation: &crate::policy::Relocation,
) -> bool {
    if !matches!(relocation.safety, RelocationSafety::Experimental(_))
        || relocation.from_position == relocation.to_position
        || relocation.to_position >= trace.blocks.len()
    {
        return false;
    }
    let block = &trace.blocks[relocation.from_position];
    let destination = &trace.blocks[relocation.to_position];
    zone_of(block) == zone_of(destination)
        && zone_of(block) != SemanticZone::Messages
        && base_score_for_source(&block.source).is_some()
        && !block.required
        && !is_protocol_critical(block)
        && !is_current_request(block)
        && !destination.required
        && !is_protocol_critical(destination)
        && !is_current_request(destination)
        && dependencies
            .relevant_for(trace, relocation.from_position)
            .is_empty()
}

fn relocation_recommendation(
    context: &PlannerContext<'_>,
    summary: &BlockSummary,
    relocation: crate::policy::Relocation,
) -> InterventionRecommendation {
    let block = &context.trace.blocks[relocation.from_position];
    let explanation = format!(
        "The existing zone-constrained stable-prefix policy found a hypothetical within-zone move for '{}' from position {} to {}.",
        block.id, relocation.from_position, relocation.to_position
    );
    let expected_structural_effect = format!(
        "Hypothetically move within semantic zone from position {} to {}; chronology and the source trace remain unchanged.",
        relocation.from_position, relocation.to_position
    );
    intervention_recommendation(
        context,
        summary,
        RecommendationSpec {
            class: InterventionClass::RelocateCandidate,
            reason_codes: vec![
                ReasonCode::WithinZoneRelocation,
                ReasonCode::StructuralHeuristicNotSufficient,
            ],
            evidence_strength: EvidenceStrength::Weak,
            quality_risk: QualityRisk::Unknown,
            provider_state_dependence: ProviderStateDependence::PotentiallyRelevant,
            explanation: &explanation,
            expected_structural_effect: &expected_structural_effect,
        },
    )
}

fn intervention_recommendation(
    context: &PlannerContext<'_>,
    summary: &BlockSummary,
    spec: RecommendationSpec<'_>,
) -> InterventionRecommendation {
    let index = summary.position;
    let mut structural = block_structural_evidence(summary);
    structural.push("Safety is based on explicit recorder metadata and structural constraints, not on token size alone.".to_string());
    let mut reasons = spec.reason_codes;
    if context.provider_evidence_present {
        reasons.push(ReasonCode::ProviderEvidenceNotUsedAsSafetyProof);
    } else {
        reasons.push(ReasonCode::NoProviderEvidence);
    }
    reasons.push(ReasonCode::NoEconomicEvidence);
    reasons.push(ReasonCode::QualityEvidenceAbsent);
    deduplicate_reasons(&mut reasons);
    InterventionRecommendation {
        class: spec.class,
        target_block_ids: vec![context.trace.blocks[index].id.clone()],
        reason_codes: reasons,
        explanation: spec.explanation.to_string(),
        evidence_strength: spec.evidence_strength,
        source_evidence: EvidenceSources {
            structural,
            provider_cache: provider_evidence(context.trace),
            economic: vec![
                "No economic profile or economic evidence was supplied to the planner.".to_string(),
            ],
            quality: vec![
                "No replay, task-check or semantic-quality evidence is available in Phase 1B.0."
                    .to_string(),
            ],
            dependency: dependency_evidence(context.trace, context.dependencies, index),
        },
        relevant_dependencies: context.dependencies.relevant_for(context.trace, index),
        expected_structural_effect: spec.expected_structural_effect.to_string(),
        expected_quality_risk: spec.quality_risk,
        provider_state_dependence: spec.provider_state_dependence,
        provider_evidence_present: context.provider_evidence_present,
        economic_evidence_present: false,
        hypothetical_only: true,
    }
}

fn keep_recommendation(context: &PlannerContext<'_>, index: usize) -> InterventionRecommendation {
    let block = &context.trace.blocks[index];
    let mut reasons = Vec::new();
    if block.required {
        reasons.push(ReasonCode::RequiredBlock);
    }
    if is_protocol_critical(block) {
        reasons.push(ReasonCode::ProtocolCriticalBlock);
    }
    if is_current_request(block) {
        reasons.push(ReasonCode::CurrentRequest);
    }
    if context.dependencies.uncertain {
        reasons.push(ReasonCode::UnknownDependencyEvidence);
    } else if !context
        .dependencies
        .relevant_for(context.trace, index)
        .is_empty()
    {
        reasons.push(ReasonCode::DependencyClosureProtected);
    }
    if reasons.is_empty() {
        reasons.push(ReasonCode::UnknownSafety);
    }
    let summary = &context.analysis.blocks[index];
    let mut structural = block_structural_evidence(summary);
    structural.push(
        "No sufficiently justified non-retention intervention was established for this block."
            .to_string(),
    );
    let mut all_reasons = reasons;
    if context.provider_evidence_present {
        all_reasons.push(ReasonCode::ProviderEvidenceNotUsedAsSafetyProof);
    } else {
        all_reasons.push(ReasonCode::NoProviderEvidence);
    }
    all_reasons.push(ReasonCode::NoEconomicEvidence);
    all_reasons.push(ReasonCode::QualityEvidenceAbsent);
    deduplicate_reasons(&mut all_reasons);
    InterventionRecommendation {
        class: InterventionClass::Keep,
        target_block_ids: vec![block.id.clone()],
        reason_codes: all_reasons.clone(),
        explanation: keep_explanation(block, &all_reasons),
        evidence_strength: if block.required
            || is_protocol_critical(block)
            || is_current_request(block)
        {
            EvidenceStrength::Strong
        } else {
            EvidenceStrength::Unknown
        },
        source_evidence: EvidenceSources {
            structural,
            provider_cache: provider_evidence(context.trace),
            economic: vec![
                "No economic profile or economic evidence was supplied to the planner.".to_string(),
            ],
            quality: vec![
                "No replay, task-check or semantic-quality evidence is available in Phase 1B.0."
                    .to_string(),
            ],
            dependency: dependency_evidence(context.trace, context.dependencies, index),
        },
        relevant_dependencies: context.dependencies.relevant_for(context.trace, index),
        expected_structural_effect:
            "Retain the block at its recorded position; apply no structural change.".to_string(),
        expected_quality_risk: QualityRisk::NoneForRetention,
        provider_state_dependence: ProviderStateDependence::NoneForRetention,
        provider_evidence_present: context.provider_evidence_present,
        economic_evidence_present: false,
        hypothetical_only: true,
    }
}

fn do_nothing_recommendation(
    context: &PlannerContext<'_>,
    deferred_relocations: &[String],
) -> InterventionRecommendation {
    let mut reasons = Vec::new();
    for block in &context.trace.blocks {
        if block.required {
            reasons.push(ReasonCode::RequiredBlock);
        }
        if is_protocol_critical(block) {
            reasons.push(ReasonCode::ProtocolCriticalBlock);
        }
        if is_current_request(block) {
            reasons.push(ReasonCode::CurrentRequest);
        }
        if block.optional || block.stale {
            reasons.push(ReasonCode::StructuralHeuristicNotSufficient);
        } else {
            reasons.push(ReasonCode::UnknownSafety);
        }
    }
    if context.dependencies.uncertain {
        reasons.push(ReasonCode::UnknownDependencyEvidence);
    }
    for deferred in deferred_relocations {
        if deferred.contains("chronological") {
            reasons.push(ReasonCode::ChronologyProtected);
        }
        if deferred.contains("cross-zone") {
            reasons.push(ReasonCode::CrossZoneRelocationRejected);
        }
    }
    reasons.push(ReasonCode::NoJustifiedIntervention);
    if context.provider_evidence_present {
        reasons.push(ReasonCode::ProviderEvidenceNotUsedAsSafetyProof);
    } else {
        reasons.push(ReasonCode::NoProviderEvidence);
    }
    reasons.push(ReasonCode::NoEconomicEvidence);
    reasons.push(ReasonCode::QualityEvidenceAbsent);
    deduplicate_reasons(&mut reasons);

    let structural = context
        .analysis
        .blocks
        .iter()
        .map(|summary| {
            format!(
                "block '{}' remains at recorded position {} in semantic zone '{}'",
                summary.id, summary.position, summary.semantic_zone
            )
        })
        .collect();
    InterventionRecommendation {
        class: InterventionClass::DoNothing,
        target_block_ids: Vec::new(),
        reason_codes: reasons,
        explanation: "No sufficiently justified intervention was established; retain the recorded context and order.".to_string(),
        evidence_strength: EvidenceStrength::Unknown,
        source_evidence: EvidenceSources {
            structural,
            provider_cache: provider_evidence(context.trace),
            economic: vec!["No economic profile or economic evidence was supplied to the planner.".to_string()],
            quality: vec!["No replay, task-check or semantic-quality evidence is available in Phase 1B.0.".to_string()],
            dependency: if context.dependencies.uncertain {
                vec!["Dependency closure evidence is insufficient; fail open to retention.".to_string()]
            } else {
                vec!["No dependency-closure violation was observed for a justified intervention.".to_string()]
            },
        },
        relevant_dependencies: Vec::new(),
        expected_structural_effect: "Retain every block and the recorded order; do not mutate the source trace.".to_string(),
        expected_quality_risk: QualityRisk::NoneForRetention,
        provider_state_dependence: ProviderStateDependence::NoneForRetention,
        provider_evidence_present: context.provider_evidence_present,
        economic_evidence_present: false,
        hypothetical_only: true,
    }
}

fn decisions_retain_index(
    recommendations: &[InterventionRecommendation],
    trace: &RequestTrace,
    index: usize,
) -> bool {
    let id = &trace.blocks[index].id;
    recommendations.iter().all(|recommendation| {
        !recommendation
            .target_block_ids
            .iter()
            .any(|target| target == id)
            || matches!(
                recommendation.class,
                InterventionClass::Keep | InterventionClass::RelocateCandidate
            )
    })
}

fn keep_explanation(block: &ContextBlock, reasons: &[ReasonCode]) -> String {
    if block.required {
        return format!("Keep '{}' because the trace explicitly marks it required; no intervention may remove it.", block.id);
    }
    if reasons.contains(&ReasonCode::ProtocolCriticalBlock) {
        return format!("Keep '{}' because its source/zone is treated as protocol-critical by the conservative baseline.", block.id);
    }
    if reasons.contains(&ReasonCode::CurrentRequest) {
        return format!("Keep '{}' because current/user request content is not a destructive-intervention target.", block.id);
    }
    if reasons.contains(&ReasonCode::DependencyClosureProtected) {
        return format!(
            "Keep '{}' because recorded dependency closure would be violated by intervention.",
            block.id
        );
    }
    if reasons.contains(&ReasonCode::UnknownDependencyEvidence) {
        return format!(
            "Keep '{}' because dependency safety evidence is incomplete; the baseline fails open.",
            block.id
        );
    }
    format!(
        "Keep '{}' because available evidence is insufficient for a justified intervention.",
        block.id
    )
}

fn block_structural_evidence(summary: &BlockSummary) -> Vec<String> {
    vec![
        format!(
            "source '{}' at position {}",
            summary.source, summary.position
        ),
        format!("semantic zone '{}'", summary.semantic_zone),
        format!(
            "prefixity {:.2} (heuristic candidate signal only)",
            summary.prefixity
        ),
        format!(
            "optional={} required={} stale={}",
            summary.optional, summary.required, summary.stale
        ),
    ]
}

fn provider_evidence(trace: &RequestTrace) -> Vec<String> {
    match &trace.usage {
        Some(usage) => vec![format!(
            "raw provider usage is present with schema '{}'; it is recorded but not used as a safety proof",
            usage.provider_schema
        )],
        None => vec!["provider/cache evidence is absent from this trace".to_string()],
    }
}

fn dependency_evidence(
    trace: &RequestTrace,
    dependencies: &DependencyGraph,
    index: usize,
) -> Vec<String> {
    if dependencies.uncertain {
        return vec![
            "dependency closure is uncertain because a reference is missing or the graph is cyclic"
                .to_string(),
        ];
    }
    let relevant = dependencies.relevant_for(trace, index);
    if relevant.is_empty() {
        vec!["no recorded direct or transitive dependency blocks this decision".to_string()]
    } else {
        vec![format!("relevant dependency IDs: {}", relevant.join(", "))]
    }
}

fn is_tool_result(block: &ContextBlock) -> bool {
    matches!(
        block.source.as_str(),
        "tool_result" | "tool-result" | "tool_output" | "tool-output"
    )
}

fn is_current_request(block: &ContextBlock) -> bool {
    matches!(
        block.source.as_str(),
        "user_request" | "user-request" | "current_request" | "current-user-request"
    ) || matches!(block.role.as_deref(), Some("user"))
}

fn is_protocol_critical(block: &ContextBlock) -> bool {
    matches!(zone_of(block), SemanticZone::System | SemanticZone::Tools)
        || matches!(block.role.as_deref(), Some("system" | "tool"))
        || matches!(
            block.source.as_str(),
            "system"
                | "system_policy"
                | "system-policy"
                | "system_instruction"
                | "system-instruction"
                | "tool_definition"
                | "tool_definitions"
                | "tool-definition"
                | "tools"
        )
}

fn deduplicate_reasons(reasons: &mut Vec<ReasonCode>) {
    let mut seen = BTreeSet::new();
    reasons.retain(|reason| seen.insert(*reason));
}

/// Dependency information used only for fail-closed planning. The trace
/// format calls dependency references informational; this planner treats
/// missing references and cycles as insufficient safety evidence.
struct DependencyGraph {
    ids: BTreeMap<String, usize>,
    uncertain: bool,
}

impl DependencyGraph {
    fn new(trace: &RequestTrace) -> Self {
        let ids = trace
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.id.clone(), index))
            .collect::<BTreeMap<_, _>>();
        let missing = trace.blocks.iter().any(|block| {
            block
                .dependencies
                .iter()
                .any(|dependency| !ids.contains_key(dependency))
        });
        let mut state = vec![0u8; trace.blocks.len()];
        let cyclic =
            (0..trace.blocks.len()).any(|index| detect_cycle(trace, &ids, index, &mut state));
        Self {
            ids,
            uncertain: missing || cyclic,
        }
    }

    fn relevant_for(&self, trace: &RequestTrace, target: usize) -> Vec<String> {
        let mut relevant = BTreeSet::new();
        for dependency in &trace.blocks[target].dependencies {
            relevant.insert(dependency.clone());
        }
        for index in 0..trace.blocks.len() {
            if index != target && self.depends_on(trace, index, target) {
                relevant.insert(trace.blocks[index].id.clone());
            }
        }
        trace
            .blocks
            .iter()
            .filter(|block| relevant.contains(&block.id))
            .map(|block| block.id.clone())
            .collect()
    }

    fn blocking_dependents(&self, trace: &RequestTrace, target: usize) -> Vec<String> {
        trace
            .blocks
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != target && self.depends_on(trace, *index, target))
            .map(|(_, block)| block.id.clone())
            .collect()
    }

    fn depends_on(&self, trace: &RequestTrace, start: usize, target: usize) -> bool {
        let mut stack = trace.blocks[start]
            .dependencies
            .iter()
            .filter_map(|dependency| self.ids.get(dependency).copied())
            .collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        while let Some(index) = stack.pop() {
            if index == target {
                return true;
            }
            if !seen.insert(index) {
                continue;
            }
            stack.extend(
                trace.blocks[index]
                    .dependencies
                    .iter()
                    .filter_map(|dependency| self.ids.get(dependency).copied()),
            );
        }
        false
    }
}

fn detect_cycle(
    trace: &RequestTrace,
    ids: &BTreeMap<String, usize>,
    index: usize,
    state: &mut [u8],
) -> bool {
    if state[index] == 1 {
        return true;
    }
    if state[index] == 2 {
        return false;
    }
    state[index] = 1;
    for dependency in &trace.blocks[index].dependencies {
        if let Some(&dependency_index) = ids.get(dependency) {
            if detect_cycle(trace, ids, dependency_index, state) {
                return true;
            }
        }
    }
    state[index] = 2;
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash::hash_content;
    use std::collections::BTreeMap;

    fn block(id: &str, source: &str, position: usize) -> ContextBlock {
        ContextBlock {
            id: id.to_string(),
            source: source.to_string(),
            position,
            content_hash: hash_content(id),
            token_count: Some(10),
            byte_count: id.len() as u64,
            timestamp: None,
            content: None,
            semantic_zone: None,
            structural_path: None,
            role: None,
            sensitivity: None,
            dependencies: Vec::new(),
            lifetime: None,
            optional: false,
            required: false,
            stale: false,
            provenance: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    fn trace(blocks: Vec<ContextBlock>) -> RequestTrace {
        RequestTrace {
            format_version: crate::model::TRACE_FORMAT_VERSION,
            request_id: "decision-test".to_string(),
            session_id: None,
            timestamp: None,
            provider: "synthetic".to_string(),
            model: "synthetic-model".to_string(),
            evidence_schema_version: None,
            blocks,
            usage: None,
            provider_response: None,
            latency: None,
            provenance: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn contract_has_exactly_six_classes() {
        assert_eq!(InterventionClass::all().len(), 6);
        assert_eq!(
            InterventionClass::CompressCandidate.as_str(),
            "COMPRESS_CANDIDATE"
        );
    }

    #[test]
    fn missing_dependency_fails_open() {
        let mut candidate = block("candidate", "tool_result", 0);
        candidate.optional = true;
        candidate.stale = true;
        candidate.dependencies.push("not-recorded".to_string());
        let plan = plan_interventions(&trace(vec![candidate])).unwrap();
        assert!(matches!(
            plan.recommendations[0].class,
            InterventionClass::DoNothing
        ));
        assert!(plan.recommendations[0]
            .reason_codes
            .contains(&ReasonCode::UnknownDependencyEvidence));
    }
}
