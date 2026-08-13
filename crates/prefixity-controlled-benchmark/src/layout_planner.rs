//! Conservative, provider-neutral context-layout candidate planning.
//!
//! P0-L11 proposes only bounded reorders of existing context artifacts when
//! explicit movement permission, ordering, and trust information make the
//! proposal safe to describe. It does not apply a candidate, execute a
//! rewritten request, or make a cache or performance claim.

use crate::conformance::{ConformanceRequest, ContextArtifactInput};
use crate::context_stability::{
    analyze_context_stability, ContextRole, ContextSegmentAnalysis, ContextStabilityAnalysis,
    ContextStabilityInputs,
};
use crate::diff::{request_diff, CacheImpactAssessment, RequestDiff};
use crate::error::BenchmarkError;
use crate::hashing::canonical_hash;
use prefixity_core::observation::{ArtifactStability, Observed, TrustLevel};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const CONTEXT_LAYOUT_PLAN_SCHEMA_ID: &str = "prefixity.context-layout-plan";
pub const CONTEXT_LAYOUT_PLAN_SCHEMA_VERSION: u32 = 1;
pub const MAX_LAYOUT_CANDIDATES: usize = 8;
pub const MAX_LAYOUT_REJECTIONS: usize = 32;
pub const MAX_LAYOUT_CONSTRAINTS: usize = 256;
pub const MAX_LAYOUT_PROVENANCE: usize = 16;
pub const MAX_LAYOUT_TEXT_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreserveOrderReason {
    SemanticDependency,
    Chronology,
    SourceAuthority,
    ToolCallResult,
    Continuation,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrderingConstraint {
    MustPrecede {
        before: String,
        after: String,
    },
    MustFollow {
        segment: String,
        after: String,
    },
    FixedPosition {
        segment: String,
        position: usize,
    },
    PreserveRelativeOrder {
        segments: Vec<String>,
        reason: PreserveOrderReason,
    },
    MovableWithinCompatibleRegion {
        segment: String,
        region: String,
    },
    Unknown {
        segment: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LayoutPlanningConstraints {
    #[serde(default)]
    pub constraints: Vec<OrderingConstraint>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl LayoutPlanningConstraints {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.constraints.len() > MAX_LAYOUT_CONSTRAINTS {
            return Err(validation("layout planning constraints exceed their bound"));
        }
        validate_provenance(&self.provenance, "constraint")?;
        for constraint in &self.constraints {
            match constraint {
                OrderingConstraint::MustPrecede { before, after } => {
                    validate_segment_name(before, "must_precede.before")?;
                    validate_segment_name(after, "must_precede.after")?;
                }
                OrderingConstraint::MustFollow { segment, after } => {
                    validate_segment_name(segment, "must_follow.segment")?;
                    validate_segment_name(after, "must_follow.after")?;
                }
                OrderingConstraint::FixedPosition { segment, .. } => {
                    validate_segment_name(segment, "fixed_position.segment")?;
                }
                OrderingConstraint::PreserveRelativeOrder { segments, .. } => {
                    if segments.len() < 2 {
                        return Err(validation(
                            "preserve_relative_order requires at least two segments",
                        ));
                    }
                    for segment in segments {
                        validate_segment_name(segment, "preserve_relative_order.segment")?;
                    }
                }
                OrderingConstraint::MovableWithinCompatibleRegion { segment, region } => {
                    validate_segment_name(segment, "movable.segment")?;
                    validate_text(region, "movable.region")?;
                }
                OrderingConstraint::Unknown { segment, reason } => {
                    validate_segment_name(segment, "unknown.segment")?;
                    validate_text(reason, "unknown.reason")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutSegmentReference {
    pub source_position: usize,
    pub structural_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    pub role: ContextRole,
    pub content_fingerprint: String,
    /// Fingerprint of the complete P0-L2 metadata record when metadata was
    /// supplied to P0-L10. Layout identity intentionally excludes this
    /// value, while P0-L13 uses it to prove metadata conservation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutTransformationKind {
    AdjacentSwap,
    RegionLocalMove,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutTransformation {
    pub kind: LayoutTransformationKind,
    pub moved_segments: Vec<String>,
    pub from_positions: Vec<usize>,
    pub to_positions: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateSafetyStatus {
    OrderingSafeUnderDeclaredConstraints,
    Rejected,
    UnknownNotProvable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanningReason {
    RemovesStabilityInversion,
    IncreasesStabilityAlignedLeadingRegion,
    PreservesDeclaredOrdering,
    TrustBoundaryPreserved,
    NoRuntimeEvidenceAttached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    OrderingConstraint,
    SemanticDependency,
    TrustBoundary,
    UnknownMoveSafety,
    FixedSegment,
    WouldAlterChronology,
    NoStructuralBenefit,
    DuplicateCandidate,
    UnsupportedSegmentRegion,
    CandidateLimitReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedLayoutCandidate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempted_layout_fingerprint: Option<String>,
    pub transformation: LayoutTransformation,
    pub status: CandidateSafetyStatus,
    pub reasons: Vec<RejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutStructuralMetrics {
    pub inversion_count: usize,
    pub stability_aligned_leading_segments: usize,
    pub unknown_boundary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralLayoutEffect {
    pub source: LayoutStructuralMetrics,
    pub candidate: LayoutStructuralMetrics,
    pub moved_segment_count: usize,
    pub changed_relative_relationships: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutCandidate {
    pub candidate_id: String,
    pub layout_fingerprint: String,
    pub ordered_segments: Vec<LayoutSegmentReference>,
    pub transformations: Vec<LayoutTransformation>,
    pub resulting_analysis: ContextStabilityAnalysis,
    pub safety: CandidateSafetyStatus,
    pub reasons: Vec<PlanningReason>,
    pub structural_effect: StructuralLayoutEffect,
    pub request_diff: RequestDiff,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextLayoutPlan {
    pub schema_id: String,
    pub schema_version: u32,
    pub source_request_fingerprint: String,
    pub source_context_fingerprint: String,
    pub source_stability_analysis: ContextStabilityAnalysis,
    pub candidates: Vec<LayoutCandidate>,
    pub rejected_candidates: Vec<RejectedLayoutCandidate>,
    pub constraints: LayoutPlanningConstraints,
    pub provenance: BTreeMap<String, String>,
}

impl ContextLayoutPlan {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CONTEXT_LAYOUT_PLAN_SCHEMA_ID
            || self.schema_version != CONTEXT_LAYOUT_PLAN_SCHEMA_VERSION
        {
            return Err(validation("unsupported context layout plan schema"));
        }
        validate_hash(
            &self.source_request_fingerprint,
            "source request fingerprint",
        )?;
        validate_hash(
            &self.source_context_fingerprint,
            "source context fingerprint",
        )?;
        self.source_stability_analysis.validate()?;
        self.constraints.validate()?;
        validate_constraint_references(&self.source_stability_analysis, &self.constraints)?;
        if self.candidates.len() > MAX_LAYOUT_CANDIDATES
            || self.rejected_candidates.len() > MAX_LAYOUT_REJECTIONS
        {
            return Err(validation("context layout plan exceeds a candidate bound"));
        }
        validate_provenance(&self.provenance, "plan")?;

        let mut candidate_ids = BTreeSet::new();
        let mut layout_fingerprints = BTreeSet::new();
        for candidate in &self.candidates {
            if candidate.safety != CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints {
                return Err(validation("a proposed candidate must be ordering-safe"));
            }
            if !candidate_ids.insert(&candidate.candidate_id)
                || !layout_fingerprints.insert(&candidate.layout_fingerprint)
            {
                return Err(validation("context layout candidates must be unique"));
            }
            validate_hash(
                &candidate.layout_fingerprint,
                "candidate layout fingerprint",
            )?;
            validate_segment_references(
                &candidate.ordered_segments,
                self.source_stability_analysis.segments.len(),
            )?;
            candidate.resulting_analysis.validate()?;
            if candidate.request_diff.left_request_fingerprint != self.source_request_fingerprint
                || candidate.request_diff.prefix_diff.cache_impact != CacheImpactAssessment::Unknown
                || candidate.request_diff.envelope_diff.cache_impact
                    != CacheImpactAssessment::Unknown
                || candidate.request_diff.cache_impact != CacheImpactAssessment::Unknown
            {
                return Err(validation(
                    "candidate request diff must preserve unknown cache impact",
                ));
            }
        }
        for rejected in &self.rejected_candidates {
            if rejected.status == CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints {
                return Err(validation("a rejected candidate cannot be marked safe"));
            }
            if rejected.reasons.is_empty() {
                return Err(validation("rejected candidate must have a reason"));
            }
            validate_transformation(&rejected.transformation)?;
            if let Some(fingerprint) = &rejected.attempted_layout_fingerprint {
                validate_hash(fingerprint, "attempted layout fingerprint")?;
            }
        }
        Ok(())
    }
}

pub fn plan_context_layout(
    request: &ConformanceRequest,
    stability_inputs: &ContextStabilityInputs,
    source_analysis: &ContextStabilityAnalysis,
    constraints: &LayoutPlanningConstraints,
) -> Result<ContextLayoutPlan, BenchmarkError> {
    request.validate()?;
    stability_inputs.validate()?;
    constraints.validate()?;
    let expected_analysis = analyze_context_stability(request, stability_inputs)?;
    if &expected_analysis != source_analysis {
        return Err(validation(
            "source stability analysis does not match request and metadata",
        ));
    }
    validate_source_shape(request, source_analysis)?;
    validate_constraint_references(source_analysis, constraints)?;

    let source_request_fingerprint = request.request_fingerprint()?;
    let source_context_fingerprint = request.context_fingerprint()?;
    let source_metrics = layout_metrics(source_analysis);
    let source_order: Vec<usize> = (0..source_analysis.segments.len()).collect();
    let mut candidates = Vec::new();
    let mut rejected_candidates = Vec::new();
    let mut seen_layouts = BTreeSet::new();

    for boundary in source_analysis.boundaries.iter().filter(|boundary| {
        boundary.direction == crate::context_stability::BoundaryDirection::TowardMoreStable
    }) {
        for kind in [
            LayoutTransformationKind::AdjacentSwap,
            LayoutTransformationKind::RegionLocalMove,
        ] {
            let order = proposed_order(source_analysis, boundary, kind)?;
            let transformation =
                transformation_for_order(source_analysis, &source_order, &order, kind)?;
            let attempted_layout_fingerprint =
                layout_fingerprint_for_order(source_analysis, &order)?;

            if candidates.len() >= MAX_LAYOUT_CANDIDATES {
                push_rejection(
                    &mut rejected_candidates,
                    RejectedLayoutCandidate {
                        attempted_layout_fingerprint: Some(attempted_layout_fingerprint),
                        transformation,
                        status: CandidateSafetyStatus::Rejected,
                        reasons: vec![RejectionReason::CandidateLimitReached],
                    },
                );
                break;
            }

            let reasons = safety_reasons(source_analysis, &order, constraints)?;
            if !reasons.is_empty() {
                let status = if reasons.contains(&RejectionReason::UnknownMoveSafety) {
                    CandidateSafetyStatus::UnknownNotProvable
                } else {
                    CandidateSafetyStatus::Rejected
                };
                push_rejection(
                    &mut rejected_candidates,
                    RejectedLayoutCandidate {
                        attempted_layout_fingerprint: Some(attempted_layout_fingerprint),
                        transformation,
                        status,
                        reasons,
                    },
                );
                continue;
            }

            let candidate_request = reordered_request(request, source_analysis, &order)?;
            let candidate_analysis =
                analyze_context_stability(&candidate_request, stability_inputs)?;
            let effect = structural_effect(
                &source_metrics,
                &layout_metrics(&candidate_analysis),
                &transformation,
                source_analysis,
                &order,
            );
            if effect.candidate.inversion_count >= effect.source.inversion_count
                && effect.candidate.stability_aligned_leading_segments
                    <= effect.source.stability_aligned_leading_segments
            {
                push_rejection(
                    &mut rejected_candidates,
                    RejectedLayoutCandidate {
                        attempted_layout_fingerprint: Some(attempted_layout_fingerprint),
                        transformation,
                        status: CandidateSafetyStatus::Rejected,
                        reasons: vec![RejectionReason::NoStructuralBenefit],
                    },
                );
                continue;
            }

            if !seen_layouts.insert(attempted_layout_fingerprint.clone()) {
                push_rejection(
                    &mut rejected_candidates,
                    RejectedLayoutCandidate {
                        attempted_layout_fingerprint: Some(attempted_layout_fingerprint),
                        transformation,
                        status: CandidateSafetyStatus::Rejected,
                        reasons: vec![RejectionReason::DuplicateCandidate],
                    },
                );
                continue;
            }

            let diff = request_diff(request, &candidate_request)?;
            let reasons = planning_reasons(&effect);
            candidates.push(LayoutCandidate {
                candidate_id: format!("layout-{attempted_layout_fingerprint}"),
                layout_fingerprint: attempted_layout_fingerprint,
                ordered_segments: ordered_references(source_analysis, &order, stability_inputs)?,
                transformations: vec![transformation],
                resulting_analysis: candidate_analysis,
                safety: CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints,
                reasons,
                structural_effect: effect,
                request_diff: diff,
            });
        }
    }

    candidates.sort_by(|left, right| candidate_sort_key(left).cmp(&candidate_sort_key(right)));
    let plan = ContextLayoutPlan {
        schema_id: CONTEXT_LAYOUT_PLAN_SCHEMA_ID.to_string(),
        schema_version: CONTEXT_LAYOUT_PLAN_SCHEMA_VERSION,
        source_request_fingerprint,
        source_context_fingerprint,
        source_stability_analysis: source_analysis.clone(),
        candidates,
        rejected_candidates,
        constraints: constraints.clone(),
        provenance: BTreeMap::from([
            (
                "source".to_string(),
                "p0-l10-analysis-and-declared-layout-constraints".to_string(),
            ),
            ("runtime_evidence".to_string(), "not_observed".to_string()),
            ("application".to_string(), "proposal_only".to_string()),
        ]),
    };
    plan.validate()?;
    Ok(plan)
}

pub fn plan_request_layout(
    request: &ConformanceRequest,
    stability_inputs: &ContextStabilityInputs,
    constraints: &LayoutPlanningConstraints,
) -> Result<ContextLayoutPlan, BenchmarkError> {
    let analysis = analyze_context_stability(request, stability_inputs)?;
    plan_context_layout(request, stability_inputs, &analysis, constraints)
}

fn proposed_order(
    analysis: &ContextStabilityAnalysis,
    boundary: &crate::context_stability::StabilityBoundary,
    kind: LayoutTransformationKind,
) -> Result<Vec<usize>, BenchmarkError> {
    let mut order: Vec<usize> = (0..analysis.segments.len()).collect();
    let left = boundary.left_segment;
    let right = boundary.right_segment;
    if kind == LayoutTransformationKind::AdjacentSwap {
        order.swap(left, right);
        return Ok(order);
    }
    if !is_artifact_position(analysis, left) || !is_artifact_position(analysis, right) {
        order.swap(left, right);
        return Ok(order);
    }
    let right_rank = stability_rank(&analysis.segments[right].stability)
        .ok_or_else(|| validation("region move requires known right stability"))?;
    let mut start = left;
    while start > 0
        && is_artifact_position(analysis, start - 1)
        && stability_rank(&analysis.segments[start - 1].stability)
            .is_some_and(|rank| rank > right_rank)
    {
        start -= 1;
    }
    let moved = order.remove(right);
    order.insert(start, moved);
    Ok(order)
}

fn safety_reasons(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
    constraints: &LayoutPlanningConstraints,
) -> Result<Vec<RejectionReason>, BenchmarkError> {
    let changed = changed_positions(order);
    let mut reasons = BTreeSet::new();
    if changed.is_empty() {
        reasons.insert(RejectionReason::NoStructuralBenefit);
    }
    if changed
        .iter()
        .any(|position| !is_artifact_position(analysis, *position))
    {
        reasons.insert(RejectionReason::UnsupportedSegmentRegion);
    }
    if changed
        .iter()
        .any(|position| analysis.segments[*position].stability == ArtifactStability::Unknown)
    {
        reasons.insert(RejectionReason::UnknownMoveSafety);
    }

    let regions: Vec<&str> = changed
        .iter()
        .filter_map(|position| {
            constraints.constraints.iter().find_map(|constraint| {
                if let OrderingConstraint::MovableWithinCompatibleRegion { segment, region } =
                    constraint
                {
                    (*segment == segment_name(&analysis.segments[*position]))
                        .then_some(region.as_str())
                } else {
                    None
                }
            })
        })
        .collect();
    if regions.len() != changed.len() {
        reasons.insert(RejectionReason::UnknownMoveSafety);
    }
    if !regions.is_empty() && regions.iter().any(|region| *region != regions[0]) {
        reasons.insert(RejectionReason::OrderingConstraint);
    }

    let original_positions =
        positions_by_name(analysis, &(0..analysis.segments.len()).collect::<Vec<_>>());
    let candidate_positions = positions_by_name(analysis, order);
    for constraint in &constraints.constraints {
        match constraint {
            OrderingConstraint::FixedPosition { segment, position } => {
                if candidate_positions.get(segment).copied() != Some(*position)
                    && original_positions.get(segment).copied() == Some(*position)
                {
                    reasons.insert(RejectionReason::FixedSegment);
                }
            }
            OrderingConstraint::MustPrecede { before, after } => {
                if order_relation(&candidate_positions, before, after) != Some(true) {
                    reasons.insert(RejectionReason::OrderingConstraint);
                }
            }
            OrderingConstraint::MustFollow { segment, after } => {
                if order_relation(&candidate_positions, after, segment) != Some(true) {
                    reasons.insert(RejectionReason::OrderingConstraint);
                }
            }
            OrderingConstraint::PreserveRelativeOrder { segments, reason } => {
                if relative_order_changed(&original_positions, &candidate_positions, segments) {
                    reasons.insert(match reason {
                        PreserveOrderReason::Chronology => RejectionReason::WouldAlterChronology,
                        _ => RejectionReason::SemanticDependency,
                    });
                }
            }
            OrderingConstraint::Unknown { segment, .. } => {
                if changed
                    .iter()
                    .any(|position| *segment == segment_name(&analysis.segments[*position]))
                {
                    reasons.insert(RejectionReason::UnknownMoveSafety);
                }
            }
            OrderingConstraint::MovableWithinCompatibleRegion { .. } => {}
        }
    }

    for (left_name, right_name) in changed_relative_pairs(analysis, order) {
        let left = analysis
            .segments
            .iter()
            .find(|segment| segment_name(segment) == left_name)
            .ok_or_else(|| validation("changed layout segment is missing"))?;
        let right = analysis
            .segments
            .iter()
            .find(|segment| segment_name(segment) == right_name)
            .ok_or_else(|| validation("changed layout segment is missing"))?;
        let (earlier, later) =
            if candidate_positions[left_name.as_str()] < candidate_positions[right_name.as_str()] {
                (left, right)
            } else {
                (right, left)
            };
        let earlier_trust = trust_rank(&earlier.trust);
        let later_trust = trust_rank(&later.trust);
        match (earlier_trust, later_trust) {
            (Some(earlier_rank), Some(later_rank)) if earlier_rank <= later_rank => {}
            (Some(_), Some(_)) => {
                reasons.insert(RejectionReason::TrustBoundary);
            }
            _ => {
                reasons.insert(RejectionReason::UnknownMoveSafety);
            }
        }
    }
    Ok(reasons.into_iter().collect())
}

pub(crate) fn reordered_request(
    request: &ConformanceRequest,
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> Result<ConformanceRequest, BenchmarkError> {
    if order.len() != analysis.segments.len()
        || order.iter().enumerate().any(|(position, source)| {
            *source != position && !is_artifact_position(analysis, position)
        })
    {
        return Err(validation(
            "candidate reordering crosses a fixed request-context slot",
        ));
    }
    let mut by_id = BTreeMap::new();
    for artifact in &request.context.artifacts {
        by_id.insert(artifact.artifact_id.clone(), artifact.clone());
    }
    let mut candidate = request.clone();
    candidate.context.artifacts = order
        .iter()
        .filter(|position| is_artifact_position(analysis, **position))
        .map(|position| {
            let id = analysis.segments[*position]
                .component_id
                .as_ref()
                .ok_or_else(|| validation("artifact segment has no component identity"))?;
            by_id
                .get(id)
                .cloned()
                .ok_or_else(|| validation("candidate artifact is absent from request"))
        })
        .collect::<Result<Vec<ContextArtifactInput>, BenchmarkError>>()?;
    candidate.validate()?;
    Ok(candidate)
}

fn validate_source_shape(
    request: &ConformanceRequest,
    analysis: &ContextStabilityAnalysis,
) -> Result<(), BenchmarkError> {
    let expected = 2 + request.context.artifacts.len() + request.context.tools.len();
    if analysis.segments.len() != expected {
        return Err(validation(
            "stability analysis does not cover request structure",
        ));
    }
    if analysis.segments[0].role != ContextRole::SystemInstruction
        || analysis.segments[0].structural_path != "context.system_instruction"
    {
        return Err(validation(
            "stability analysis system segment is inconsistent",
        ));
    }
    for (index, artifact) in request.context.artifacts.iter().enumerate() {
        let segment = &analysis.segments[index + 1];
        if segment.role != ContextRole::ContextArtifact
            || segment.component_id.as_deref() != Some(artifact.artifact_id.as_str())
            || segment.structural_path != format!("context.artifacts[{}]", artifact.artifact_id)
        {
            return Err(validation(
                "stability analysis artifact segment is inconsistent",
            ));
        }
    }
    let user_position = request.context.artifacts.len() + 1;
    if analysis.segments[user_position].role != ContextRole::CurrentUserTask
        || analysis.segments[user_position].structural_path != "context.current_user"
    {
        return Err(validation(
            "stability analysis user segment is inconsistent",
        ));
    }
    for (offset, tool) in request.context.tools.iter().enumerate() {
        let segment = &analysis.segments[user_position + 1 + offset];
        if segment.role != ContextRole::ToolDefinition
            || segment.component_id.as_deref() != Some(tool.name.as_str())
            || segment.structural_path != format!("context.tools[{}]", tool.name)
        {
            return Err(validation(
                "stability analysis tool segment is inconsistent",
            ));
        }
    }
    Ok(())
}

fn validate_constraint_references(
    analysis: &ContextStabilityAnalysis,
    constraints: &LayoutPlanningConstraints,
) -> Result<(), BenchmarkError> {
    let known: BTreeSet<String> = analysis.segments.iter().map(segment_name).collect();
    for constraint in &constraints.constraints {
        let references = constraint_references(constraint);
        if references
            .iter()
            .any(|reference| !known.contains(*reference))
        {
            return Err(validation(
                "layout constraint references an unknown segment",
            ));
        }
    }
    Ok(())
}

fn constraint_references(constraint: &OrderingConstraint) -> Vec<&String> {
    match constraint {
        OrderingConstraint::MustPrecede { before, after } => vec![before, after],
        OrderingConstraint::MustFollow { segment, after } => vec![segment, after],
        OrderingConstraint::FixedPosition { segment, .. } => vec![segment],
        OrderingConstraint::PreserveRelativeOrder { segments, .. } => segments.iter().collect(),
        OrderingConstraint::MovableWithinCompatibleRegion { segment, .. } => vec![segment],
        OrderingConstraint::Unknown { segment, .. } => vec![segment],
    }
}

fn transformation_for_order(
    analysis: &ContextStabilityAnalysis,
    source_order: &[usize],
    order: &[usize],
    kind: LayoutTransformationKind,
) -> Result<LayoutTransformation, BenchmarkError> {
    let mut moved_segments = Vec::new();
    let mut from_positions = Vec::new();
    let mut to_positions = Vec::new();
    let positions: BTreeMap<usize, usize> = order
        .iter()
        .enumerate()
        .map(|(new_position, source_position)| (*source_position, new_position))
        .collect();
    for source_position in source_order {
        let new_position = positions
            .get(source_position)
            .copied()
            .ok_or_else(|| validation("candidate layout is not a permutation"))?;
        if *source_position != new_position {
            moved_segments.push(segment_name(&analysis.segments[*source_position]));
            from_positions.push(*source_position);
            to_positions.push(new_position);
        }
    }
    Ok(LayoutTransformation {
        kind,
        moved_segments,
        from_positions,
        to_positions,
    })
}

fn ordered_references(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
    inputs: &ContextStabilityInputs,
) -> Result<Vec<LayoutSegmentReference>, BenchmarkError> {
    order
        .iter()
        .map(
            |position| -> Result<LayoutSegmentReference, BenchmarkError> {
                let segment = &analysis.segments[*position];
                Ok(LayoutSegmentReference {
                    source_position: *position,
                    structural_path: segment.structural_path.clone(),
                    component_id: segment.component_id.clone(),
                    role: segment.role,
                    content_fingerprint: segment.content_fingerprint.clone(),
                    metadata_fingerprint: metadata_fingerprint_for_segment(segment, inputs)?,
                })
            },
        )
        .collect()
}

pub(crate) fn metadata_fingerprint_for_segment(
    segment: &ContextSegmentAnalysis,
    inputs: &ContextStabilityInputs,
) -> Result<Option<String>, BenchmarkError> {
    let metadata = match segment.role {
        ContextRole::SystemInstruction => inputs.system_instruction.as_ref(),
        ContextRole::ContextArtifact => segment
            .component_id
            .as_ref()
            .and_then(|id| inputs.artifacts.get(id)),
        ContextRole::CurrentUserTask => inputs.current_user_task.as_ref(),
        ContextRole::ToolDefinition => segment
            .component_id
            .as_ref()
            .and_then(|name| inputs.tools.get(name)),
    };
    metadata
        .map(|value| canonical_hash(value).map_err(|error| validation(error.to_string())))
        .transpose()
}

#[derive(Debug, Serialize)]
struct LayoutIdentity<'a> {
    segments: Vec<LayoutIdentitySegment<'a>>,
}

#[derive(Debug, Serialize)]
struct LayoutIdentitySegment<'a> {
    role: ContextRole,
    component_id: &'a Option<String>,
    content_fingerprint: &'a str,
}

pub(crate) fn layout_fingerprint_for_order(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> Result<String, BenchmarkError> {
    if order.len() != analysis.segments.len() {
        return Err(validation("candidate layout has an invalid segment count"));
    }
    let identity = LayoutIdentity {
        segments: order
            .iter()
            .map(|position| {
                let segment = &analysis.segments[*position];
                LayoutIdentitySegment {
                    role: segment.role,
                    component_id: &segment.component_id,
                    content_fingerprint: &segment.content_fingerprint,
                }
            })
            .collect(),
    };
    canonical_hash(&identity).map_err(|error| validation(error.to_string()))
}

pub(crate) fn layout_metrics(analysis: &ContextStabilityAnalysis) -> LayoutStructuralMetrics {
    LayoutStructuralMetrics {
        inversion_count: analysis
            .findings
            .iter()
            .filter(|finding| {
                finding.kind == crate::context_stability::StabilityFindingKind::StabilityInversion
            })
            .count(),
        stability_aligned_leading_segments: analysis.leading_region.segment_count,
        unknown_boundary_count: analysis
            .boundaries
            .iter()
            .filter(|boundary| {
                boundary.classification == crate::context_stability::BoundaryClassification::Unknown
            })
            .count(),
    }
}

fn structural_effect(
    source: &LayoutStructuralMetrics,
    candidate: &LayoutStructuralMetrics,
    transformation: &LayoutTransformation,
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> StructuralLayoutEffect {
    StructuralLayoutEffect {
        source: source.clone(),
        candidate: candidate.clone(),
        moved_segment_count: transformation.moved_segments.len(),
        changed_relative_relationships: changed_relative_relationship_count(analysis, order),
    }
}

fn planning_reasons(effect: &StructuralLayoutEffect) -> Vec<PlanningReason> {
    let mut reasons = Vec::new();
    if effect.candidate.inversion_count < effect.source.inversion_count {
        reasons.push(PlanningReason::RemovesStabilityInversion);
    }
    if effect.candidate.stability_aligned_leading_segments
        > effect.source.stability_aligned_leading_segments
    {
        reasons.push(PlanningReason::IncreasesStabilityAlignedLeadingRegion);
    }
    reasons.extend([
        PlanningReason::PreservesDeclaredOrdering,
        PlanningReason::TrustBoundaryPreserved,
        PlanningReason::NoRuntimeEvidenceAttached,
    ]);
    reasons
}

fn changed_relative_relationship_count(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> usize {
    changed_relative_pairs(analysis, order).len()
}

fn changed_relative_pairs(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> Vec<(String, String)> {
    let original: Vec<String> = (0..analysis.segments.len())
        .map(|position| segment_name(&analysis.segments[position]))
        .collect();
    let candidate: Vec<String> = order
        .iter()
        .map(|position| segment_name(&analysis.segments[*position]))
        .collect();
    let original_positions: BTreeMap<&str, usize> = original
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect();
    let candidate_positions: BTreeMap<&str, usize> = candidate
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect();
    let mut changed = Vec::new();
    for left in &original {
        for right in &original {
            if left >= right {
                continue;
            }
            let old_relation =
                original_positions[left.as_str()] < original_positions[right.as_str()];
            let new_relation =
                candidate_positions[left.as_str()] < candidate_positions[right.as_str()];
            if old_relation != new_relation {
                changed.push((left.clone(), right.clone()));
            }
        }
    }
    changed
}

fn positions_by_name(
    analysis: &ContextStabilityAnalysis,
    order: &[usize],
) -> BTreeMap<String, usize> {
    order
        .iter()
        .enumerate()
        .map(|(position, source_position)| {
            (segment_name(&analysis.segments[*source_position]), position)
        })
        .collect()
}

fn changed_positions(order: &[usize]) -> Vec<usize> {
    let positions: BTreeMap<usize, usize> = order
        .iter()
        .enumerate()
        .map(|(new_position, source_position)| (*source_position, new_position))
        .collect();
    positions
        .iter()
        .filter_map(|(source_position, new_position)| {
            (*source_position != *new_position).then_some(*source_position)
        })
        .collect()
}

fn relative_order_changed(
    original: &BTreeMap<String, usize>,
    candidate: &BTreeMap<String, usize>,
    segments: &[String],
) -> bool {
    segments.windows(2).any(|pair| {
        original[&pair[0]] < original[&pair[1]] && candidate[&pair[0]] >= candidate[&pair[1]]
    })
}

fn order_relation(positions: &BTreeMap<String, usize>, before: &str, after: &str) -> Option<bool> {
    Some(positions.get(before)? < positions.get(after)?)
}

fn is_artifact_position(analysis: &ContextStabilityAnalysis, position: usize) -> bool {
    analysis
        .segments
        .get(position)
        .is_some_and(|segment| segment.role == ContextRole::ContextArtifact)
}

fn segment_name(segment: &ContextSegmentAnalysis) -> String {
    segment.structural_path.clone()
}

fn stability_rank(stability: &ArtifactStability) -> Option<u8> {
    match stability {
        ArtifactStability::Immutable => Some(0),
        ArtifactStability::Stable => Some(1),
        ArtifactStability::AppendOnly => Some(2),
        ArtifactStability::Volatile => Some(3),
        ArtifactStability::Unknown => None,
    }
}

fn trust_rank(trust: &Observed<TrustLevel>) -> Option<u8> {
    match trust {
        Observed::Known(TrustLevel::Trusted) => Some(0),
        Observed::Known(TrustLevel::Mixed) => Some(1),
        Observed::Known(TrustLevel::Untrusted) => Some(2),
        Observed::Known(TrustLevel::Unknown) | Observed::Unknown | Observed::NotObserved => None,
    }
}

fn candidate_sort_key(candidate: &LayoutCandidate) -> (usize, usize, usize, usize, usize, &str) {
    (
        candidate.structural_effect.candidate.inversion_count,
        usize::MAX
            - candidate
                .structural_effect
                .candidate
                .stability_aligned_leading_segments,
        candidate.structural_effect.candidate.unknown_boundary_count,
        candidate.structural_effect.moved_segment_count,
        candidate.structural_effect.changed_relative_relationships,
        &candidate.layout_fingerprint,
    )
}

fn validate_segment_references(
    references: &[LayoutSegmentReference],
    expected_len: usize,
) -> Result<(), BenchmarkError> {
    if references.len() != expected_len {
        return Err(validation(
            "candidate does not reference every source segment",
        ));
    }
    let mut positions = BTreeSet::new();
    for reference in references {
        if !positions.insert(reference.source_position) || reference.source_position >= expected_len
        {
            return Err(validation(
                "candidate segment references are not a permutation",
            ));
        }
        validate_text(&reference.structural_path, "candidate structural path")?;
        validate_hash(
            &reference.content_fingerprint,
            "candidate segment fingerprint",
        )?;
        if let Some(fingerprint) = &reference.metadata_fingerprint {
            validate_hash(fingerprint, "candidate segment metadata fingerprint")?;
        }
    }
    Ok(())
}

fn validate_transformation(transformation: &LayoutTransformation) -> Result<(), BenchmarkError> {
    if transformation.moved_segments.len() != transformation.from_positions.len()
        || transformation.from_positions.len() != transformation.to_positions.len()
    {
        return Err(validation(
            "layout transformation positions are inconsistent",
        ));
    }
    for segment in &transformation.moved_segments {
        validate_segment_name(segment, "transformation.segment")?;
    }
    Ok(())
}

fn push_rejection(rejected: &mut Vec<RejectedLayoutCandidate>, candidate: RejectedLayoutCandidate) {
    if rejected.len() < MAX_LAYOUT_REJECTIONS {
        rejected.push(candidate);
    }
}

fn validate_provenance(
    provenance: &BTreeMap<String, String>,
    field: &str,
) -> Result<(), BenchmarkError> {
    if provenance.len() > MAX_LAYOUT_PROVENANCE {
        return Err(validation(format!("{field} provenance exceeds its bound")));
    }
    for (key, value) in provenance {
        validate_text(key, &format!("{field} provenance key"))?;
        validate_text(value, &format!("{field} provenance value"))?;
    }
    Ok(())
}

fn validate_segment_name(value: &str, field: &str) -> Result<(), BenchmarkError> {
    validate_text(value, field)
}

fn validate_hash(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(validation(format!("{field} must be a SHA-256 fingerprint")));
    }
    Ok(())
}

fn validate_text(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_LAYOUT_TEXT_BYTES {
        return Err(validation(format!(
            "{field} must be non-empty and at most {MAX_LAYOUT_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validation(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::validation(message)
}
