use prefixity_controlled_benchmark::{
    analyze_context_stability, plan_context_layout, plan_request_layout, BoundaryClassification,
    CacheImpactAssessment, CandidateSafetyStatus, ConformanceRequest, ContextArtifactInput,
    ContextRole, ContextStabilityInputs, LayoutPlanningConstraints, OrderedJsonObject,
    OrderingConstraint, PreserveOrderReason, RejectionReason, RequestContext, RequestEnvelope,
    ToolDefinition, MAX_LAYOUT_CANDIDATES, MAX_LAYOUT_REJECTIONS,
};
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, ContextArtifact, Observed,
    TrustLevel, CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

fn request(ids: &[&str], with_tool: bool) -> ConformanceRequest {
    ConformanceRequest {
        context: RequestContext {
            system_instruction: "system instruction".to_string(),
            artifacts: ids
                .iter()
                .map(|id| ContextArtifactInput {
                    artifact_id: (*id).to_string(),
                    content: format!("artifact-{id}"),
                })
                .collect(),
            user_content: "current user task".to_string(),
            tools: if with_tool {
                vec![ToolDefinition {
                    name: "tool".to_string(),
                    description: "tool description".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                }]
            } else {
                Vec::new()
            },
        },
        envelope: RequestEnvelope {
            model: "fixture-model".to_string(),
            reasoning: None,
            response_format: None,
        },
    }
}

fn metadata(id: &str, stability: ArtifactStability, trust: TrustLevel) -> ContextArtifact {
    ContextArtifact {
        schema_version: CONTEXT_ARTIFACT_SCHEMA_VERSION,
        artifact_id: id.to_string(),
        origin_id: format!("origin-{id}"),
        content_source_id: Observed::Known(format!("source-{id}")),
        content_hash: Observed::Unknown,
        revision: Observed::Known("v1".to_string()),
        artifact_type: ArtifactType::Text,
        stability,
        lifecycle: ArtifactLifecycle::PersistentVersioned,
        sizes: ArtifactSizes {
            byte_size: Observed::Known(10),
            ..ArtifactSizes::default()
        },
        cache: Default::default(),
        trust: Observed::Known(trust),
        provenance: BTreeMap::new(),
        metadata: BTreeMap::new(),
    }
}

fn inputs(ids: &[&str], stabilities: &[ArtifactStability]) -> ContextStabilityInputs {
    let artifacts = ids
        .iter()
        .zip(stabilities)
        .map(|(id, stability)| {
            (
                (*id).to_string(),
                metadata(id, stability.clone(), TrustLevel::Trusted),
            )
        })
        .collect();
    ContextStabilityInputs {
        artifacts,
        ..ContextStabilityInputs::default()
    }
}

fn movable(ids: &[&str]) -> LayoutPlanningConstraints {
    LayoutPlanningConstraints {
        constraints: ids
            .iter()
            .map(|id| OrderingConstraint::MovableWithinCompatibleRegion {
                segment: format!("context.artifacts[{id}]"),
                region: "artifact-sequence".to_string(),
            })
            .collect(),
        provenance: BTreeMap::from([(
            "source".to_string(),
            "explicit-test-constraints".to_string(),
        )]),
    }
}

fn adjacent_fixture() -> (
    ConformanceRequest,
    ContextStabilityInputs,
    LayoutPlanningConstraints,
) {
    let ids = ["a", "b", "c"];
    (
        request(&ids, false),
        inputs(
            &ids,
            &[
                ArtifactStability::Stable,
                ArtifactStability::Volatile,
                ArtifactStability::Stable,
            ],
        ),
        movable(&ids),
    )
}

fn plan_fixture() -> prefixity_controlled_benchmark::ContextLayoutPlan {
    let (request, inputs, constraints) = adjacent_fixture();
    plan_request_layout(&request, &inputs, &constraints).unwrap()
}

#[test]
fn safe_adjacent_inversion_produces_one_candidate() {
    let plan = plan_fixture();
    assert_eq!(plan.candidates.len(), 1);
    assert_eq!(
        plan.candidates[0].safety,
        CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints
    );
    assert!(plan.candidates[0]
        .reasons
        .contains(&prefixity_controlled_benchmark::PlanningReason::RemovesStabilityInversion));
}

#[test]
fn candidate_reduces_inversion_and_improves_leading_region() {
    let plan = plan_fixture();
    let effect = &plan.candidates[0].structural_effect;
    assert!(effect.candidate.inversion_count < effect.source.inversion_count);
    assert!(
        effect.candidate.stability_aligned_leading_segments
            > effect.source.stability_aligned_leading_segments
    );
}

#[test]
fn source_request_is_never_mutated() {
    let (request, inputs, constraints) = adjacent_fixture();
    let original = request.clone();
    let _ = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert_eq!(request, original);
}

#[test]
fn fixed_segments_never_move() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints
        .constraints
        .push(OrderingConstraint::FixedPosition {
            segment: "context.artifacts[c]".to_string(),
            position: 3,
        });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan
        .rejected_candidates
        .iter()
        .any(|candidate| { candidate.reasons.contains(&RejectionReason::FixedSegment) }));
}

#[test]
fn explicit_relative_order_blocks_the_move() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints
        .constraints
        .push(OrderingConstraint::PreserveRelativeOrder {
            segments: vec![
                "context.artifacts[b]".to_string(),
                "context.artifacts[c]".to_string(),
            ],
            reason: PreserveOrderReason::SemanticDependency,
        });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::SemanticDependency)
    }));
}

#[test]
fn chronology_constraint_blocks_the_move() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints
        .constraints
        .push(OrderingConstraint::PreserveRelativeOrder {
            segments: vec![
                "context.artifacts[b]".to_string(),
                "context.artifacts[c]".to_string(),
            ],
            reason: PreserveOrderReason::Chronology,
        });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::WouldAlterChronology)
    }));
}

#[test]
fn must_precede_constraint_is_preserved() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints
        .constraints
        .push(OrderingConstraint::MustPrecede {
            before: "context.artifacts[b]".to_string(),
            after: "context.artifacts[c]".to_string(),
        });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::OrderingConstraint)
    }));
}

#[test]
fn must_follow_constraint_is_preserved() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints
        .constraints
        .push(OrderingConstraint::MustFollow {
            segment: "context.artifacts[c]".to_string(),
            after: "context.artifacts[b]".to_string(),
        });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
}

#[test]
fn missing_movement_permission_is_unknown_not_optimistic() {
    let (request, inputs, _) = adjacent_fixture();
    let plan =
        plan_request_layout(&request, &inputs, &LayoutPlanningConstraints::default()).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate.status == CandidateSafetyStatus::UnknownNotProvable
            && candidate
                .reasons
                .contains(&RejectionReason::UnknownMoveSafety)
    }));
}

#[test]
fn unknown_constraint_blocks_the_move() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints.constraints.push(OrderingConstraint::Unknown {
        segment: "context.artifacts[c]".to_string(),
        reason: "semantic commutativity not established".to_string(),
    });
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan
        .rejected_candidates
        .iter()
        .any(|candidate| { candidate.status == CandidateSafetyStatus::UnknownNotProvable }));
}

#[test]
fn trust_boundary_blocks_untrusted_promotion() {
    let request = request(&["a", "b", "c"], false);
    let mut stability_inputs = inputs(
        &["a", "b", "c"],
        &[
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
        ],
    );
    stability_inputs.artifacts.get_mut("c").unwrap().trust = Observed::Known(TrustLevel::Untrusted);
    let constraints = movable(&["a", "b", "c"]);
    let plan = plan_request_layout(&request, &stability_inputs, &constraints).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan
        .rejected_candidates
        .iter()
        .any(|candidate| { candidate.reasons.contains(&RejectionReason::TrustBoundary) }));
}

#[test]
fn unknown_trust_is_not_treated_as_safe() {
    let request = request(&["a", "b", "c"], false);
    let mut stability_inputs = inputs(
        &["a", "b", "c"],
        &[
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
        ],
    );
    stability_inputs.artifacts.get_mut("c").unwrap().trust = Observed::Unknown;
    let plan =
        plan_request_layout(&request, &stability_inputs, &movable(&["a", "b", "c"])).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::UnknownMoveSafety)
    }));
}

#[test]
fn unknown_stability_is_never_guessed() {
    let ids = ["a", "b", "c"];
    let request = request(&ids, false);
    let inputs = inputs(
        &ids,
        &[
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Unknown,
        ],
    );
    let analysis = analyze_context_stability(&request, &inputs).unwrap();
    assert_eq!(
        analysis.boundaries[2].classification,
        BoundaryClassification::Unknown
    );
    let plan = plan_context_layout(&request, &inputs, &analysis, &movable(&ids)).unwrap();
    assert!(plan.candidates.is_empty());
}

#[test]
fn unsupported_cross_slot_move_is_rejected() {
    let request = request(&["a"], true);
    let mut inputs = inputs(&["a"], &[ArtifactStability::Stable]);
    inputs.tools.insert(
        "tool".to_string(),
        metadata("tool", ArtifactStability::Stable, TrustLevel::Trusted),
    );
    let analysis = analyze_context_stability(&request, &inputs).unwrap();
    assert!(analysis.boundaries.iter().any(|boundary| boundary.direction
        == prefixity_controlled_benchmark::BoundaryDirection::TowardMoreStable));
    let plan = plan_context_layout(
        &request,
        &inputs,
        &analysis,
        &LayoutPlanningConstraints::default(),
    )
    .unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::UnsupportedSegmentRegion)
    }));
}

#[test]
fn already_aligned_context_has_no_gratuitous_candidate() {
    let ids = ["a", "b", "c", "d"];
    let request = request(&ids, false);
    let inputs = inputs(
        &ids,
        &[
            ArtifactStability::Stable,
            ArtifactStability::Stable,
            ArtifactStability::AppendOnly,
            ArtifactStability::Volatile,
        ],
    );
    let plan = plan_request_layout(&request, &inputs, &movable(&ids)).unwrap();
    assert!(plan.candidates.is_empty());
    assert!(plan.rejected_candidates.is_empty());
}

#[test]
fn transient_stable_lifecycle_is_carried_without_heuristic_rewrite() {
    let (request, mut inputs, constraints) = adjacent_fixture();
    inputs.artifacts.get_mut("c").unwrap().lifecycle = ArtifactLifecycle::Transient;
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert_eq!(
        plan.candidates[0].resulting_analysis.segments[2].lifecycle,
        ArtifactLifecycle::Transient
    );
}

#[test]
fn multiple_safe_candidates_are_deterministic() {
    let ids = ["a", "b", "c", "d", "e"];
    let request = request(&ids, false);
    let inputs = inputs(
        &ids,
        &[
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
        ],
    );
    let constraints = movable(&ids);
    let first = plan_request_layout(&request, &inputs, &constraints).unwrap();
    let second = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.candidates.len(), 2);
    assert!(
        first.candidates[0]
            .structural_effect
            .candidate
            .stability_aligned_leading_segments
            >= first.candidates[1]
                .structural_effect
                .candidate
                .stability_aligned_leading_segments
    );
}

#[test]
fn equivalent_transformation_paths_are_deduplicated() {
    let plan = plan_fixture();
    assert_eq!(
        plan.candidates
            .iter()
            .map(|candidate| candidate.layout_fingerprint.clone())
            .collect::<BTreeSet<_>>()
            .len(),
        plan.candidates.len()
    );
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::DuplicateCandidate)
    }));
}

#[test]
fn candidate_fingerprints_are_deterministic() {
    let first = plan_fixture();
    let second = plan_fixture();
    assert_eq!(
        first.candidates[0].layout_fingerprint,
        second.candidates[0].layout_fingerprint
    );
    assert_eq!(
        first.candidates[0].candidate_id,
        second.candidates[0].candidate_id
    );
}

#[test]
fn different_safe_layouts_have_different_fingerprints() {
    let ids = ["a", "b", "c", "d", "e"];
    let plan = plan_request_layout(
        &request(&ids, false),
        &inputs(
            &ids,
            &[
                ArtifactStability::Stable,
                ArtifactStability::Volatile,
                ArtifactStability::Stable,
                ArtifactStability::Volatile,
                ArtifactStability::Stable,
            ],
        ),
        &movable(&ids),
    )
    .unwrap();
    assert_ne!(
        plan.candidates[0].layout_fingerprint,
        plan.candidates[1].layout_fingerprint
    );
}

#[test]
fn candidate_order_is_a_permutation_of_source_segments() {
    let plan = plan_fixture();
    let positions: BTreeSet<_> = plan.candidates[0]
        .ordered_segments
        .iter()
        .map(|segment| segment.source_position)
        .collect();
    assert_eq!(positions, (0..5).collect());
}

#[test]
fn candidate_diff_reports_exact_artifact_reorder() {
    let plan = plan_fixture();
    let diff = &plan.candidates[0].request_diff;
    assert_eq!(
        diff.interpretation.context,
        prefixity_controlled_benchmark::DiffState::Changed
    );
    assert!(diff.prefix_diff.changes.iter().any(|change| change.category
        == prefixity_controlled_benchmark::ChangeCategory::ArtifactOrderChanged));
    assert_eq!(diff.cache_impact, CacheImpactAssessment::Unknown);
}

#[test]
fn all_candidate_diff_layers_keep_cache_impact_unknown() {
    let plan = plan_fixture();
    let candidate = &plan.candidates[0];
    assert_eq!(
        candidate.request_diff.prefix_diff.cache_impact,
        CacheImpactAssessment::Unknown
    );
    assert_eq!(
        candidate.request_diff.envelope_diff.cache_impact,
        CacheImpactAssessment::Unknown
    );
    assert_eq!(
        candidate.request_diff.cache_impact,
        CacheImpactAssessment::Unknown
    );
}

#[test]
fn candidate_is_reanalysed_through_p0_l10() {
    let plan = plan_fixture();
    let candidate = &plan.candidates[0];
    assert_eq!(
        candidate.resulting_analysis.schema_id,
        "prefixity.context-stability-analysis"
    );
    assert_eq!(candidate.resulting_analysis.findings.len(), 0);
    assert!(
        candidate.resulting_analysis.leading_region.segment_count
            > plan.source_stability_analysis.leading_region.segment_count
    );
}

#[test]
fn unknown_boundaries_continue_to_limit_source_conclusions() {
    let ids = ["a", "b", "c"];
    let request = request(&ids, false);
    let inputs = inputs(
        &ids,
        &[
            ArtifactStability::Stable,
            ArtifactStability::Unknown,
            ArtifactStability::Stable,
        ],
    );
    let plan = plan_request_layout(&request, &inputs, &movable(&ids)).unwrap();
    assert_eq!(
        plan.source_stability_analysis.leading_region.limit,
        prefixity_controlled_benchmark::LeadingRegionLimit::LimitedByUnknown
    );
    assert!(plan.candidates.is_empty());
}

#[test]
fn repeated_plan_serialization_is_identical() {
    let first = serde_json::to_vec(&plan_fixture()).unwrap();
    let second = serde_json::to_vec(&plan_fixture()).unwrap();
    assert_eq!(first, second);
}

#[test]
fn large_source_content_is_not_copied_into_plan() {
    let ids = ["a", "b", "c"];
    let mut request = request(&ids, false);
    request.context.artifacts[2].content = "unique-layout-content-".repeat(200);
    let inputs = inputs(
        &ids,
        &[
            ArtifactStability::Stable,
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
        ],
    );
    let plan = plan_request_layout(&request, &inputs, &movable(&ids)).unwrap();
    let bytes = serde_json::to_vec(&plan).unwrap();
    assert!(!String::from_utf8(bytes)
        .unwrap()
        .contains(&"unique-layout-content-".repeat(200)));
}

#[test]
fn plan_has_no_synthetic_cache_telemetry_or_performance_field() {
    let serialized = serde_json::to_string(&plan_fixture()).unwrap();
    assert!(!serialized.contains("cache_hit"));
    assert!(!serialized.contains("latency"));
    assert!(!serialized.contains("token_savings"));
    assert!(serialized.contains("\"runtime_evidence\":\"not_observed\""));
}

#[test]
fn plan_does_not_expose_a_candidate_request_for_automatic_application() {
    let serialized = serde_json::to_string(&plan_fixture()).unwrap();
    assert!(!serialized.contains("candidate_request"));
    assert!(!serialized.contains("\"apply\""));
}

#[test]
fn source_and_candidate_request_fingerprints_are_distinct() {
    let plan = plan_fixture();
    assert_ne!(
        plan.source_context_fingerprint,
        plan.candidates[0].resulting_analysis.context_fingerprint
    );
    assert_eq!(
        plan.candidates[0].request_diff.left_request_fingerprint,
        plan.source_request_fingerprint
    );
}

#[test]
fn plan_validation_rejects_stale_source_analysis() {
    let (request, inputs, constraints) = adjacent_fixture();
    let mut analysis = analyze_context_stability(&request, &inputs).unwrap();
    analysis.schema_version += 1;
    assert!(plan_context_layout(&request, &inputs, &analysis, &constraints).is_err());
}

#[test]
fn plan_validation_rejects_unknown_constraint_reference() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    constraints.constraints.push(OrderingConstraint::Unknown {
        segment: "context.artifacts[missing]".to_string(),
        reason: "not established".to_string(),
    });
    assert!(plan_request_layout(&request, &inputs, &constraints).is_err());
}

#[test]
fn candidate_bound_is_deterministic() {
    let ids: Vec<String> = (0..20).map(|index| format!("a{index}")).collect();
    let id_refs: Vec<&str> = ids.iter().map(String::as_str).collect();
    let stabilities: Vec<_> = (0..20)
        .map(|index| {
            if index % 2 == 0 {
                ArtifactStability::Volatile
            } else {
                ArtifactStability::Stable
            }
        })
        .collect();
    let plan = plan_request_layout(
        &request(&id_refs, false),
        &inputs(&id_refs, &stabilities),
        &movable(&id_refs),
    )
    .unwrap();
    assert!(plan.candidates.len() <= MAX_LAYOUT_CANDIDATES);
    assert!(plan.rejected_candidates.len() <= MAX_LAYOUT_REJECTIONS);
    assert!(plan.rejected_candidates.iter().any(|candidate| {
        candidate
            .reasons
            .contains(&RejectionReason::CandidateLimitReached)
    }));
}

#[test]
fn rejected_records_are_bounded_and_content_free() {
    let (request, inputs, _) = adjacent_fixture();
    let mut constraints = LayoutPlanningConstraints::default();
    for index in 0..100 {
        constraints.constraints.push(OrderingConstraint::Unknown {
            segment: "context.artifacts[b]".to_string(),
            reason: format!("reason-{index}"),
        });
    }
    let plan = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert!(plan.rejected_candidates.len() <= MAX_LAYOUT_REJECTIONS);
    let serialized = serde_json::to_string(&plan).unwrap();
    assert!(!serialized.contains("artifact-b"));
}

#[test]
fn explicit_artifact_ordering_is_the_only_reordered_sequence() {
    let plan = plan_fixture();
    let candidate = &plan.candidates[0];
    assert_eq!(
        candidate.ordered_segments[0].role,
        ContextRole::SystemInstruction
    );
    assert_eq!(
        candidate.ordered_segments[4].role,
        ContextRole::CurrentUserTask
    );
    assert!(
        candidate
            .ordered_segments
            .iter()
            .filter(|segment| segment.role == ContextRole::ToolDefinition)
            .count()
            == 0
    );
}

#[test]
fn candidate_safety_wording_is_scoped_to_declared_constraints() {
    let plan = plan_fixture();
    assert_eq!(
        plan.candidates[0].safety,
        CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints
    );
    let serialized = serde_json::to_string(&plan).unwrap();
    assert!(serialized.contains("ordering_safe_under_declared_constraints"));
    assert!(!serialized.contains("semantically_identical"));
}

#[test]
fn lifecycle_does_not_create_a_candidate_without_move_permission() {
    let (request, mut inputs, _) = adjacent_fixture();
    inputs.artifacts.get_mut("c").unwrap().lifecycle = ArtifactLifecycle::Transient;
    let plan =
        plan_request_layout(&request, &inputs, &LayoutPlanningConstraints::default()).unwrap();
    assert!(plan.candidates.is_empty());
}

#[test]
fn candidate_plan_is_independent_of_provenance_for_layout_identity() {
    let (request, inputs, mut constraints) = adjacent_fixture();
    let first = plan_request_layout(&request, &inputs, &constraints).unwrap();
    constraints
        .provenance
        .insert("ingested_at".to_string(), "different-run".to_string());
    let second = plan_request_layout(&request, &inputs, &constraints).unwrap();
    assert_eq!(
        first.candidates[0].layout_fingerprint,
        second.candidates[0].layout_fingerprint
    );
}

#[test]
fn constraints_round_trip_deterministically() {
    let (_, _, constraints) = adjacent_fixture();
    let json = serde_json::to_vec(&constraints).unwrap();
    let decoded: LayoutPlanningConstraints = serde_json::from_slice(&json).unwrap();
    assert_eq!(decoded, constraints);
}

#[test]
fn plan_round_trip_preserves_rejections_and_candidates() {
    let plan = plan_fixture();
    let json = serde_json::to_vec(&plan).unwrap();
    let decoded: prefixity_controlled_benchmark::ContextLayoutPlan =
        serde_json::from_slice(&json).unwrap();
    assert_eq!(decoded, plan);
    decoded.validate().unwrap();
}
