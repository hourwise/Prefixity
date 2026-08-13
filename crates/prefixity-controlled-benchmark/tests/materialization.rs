use prefixity_controlled_benchmark::{
    build_candidate_experiment_pair, evaluate_candidate, materialize_candidate, BenchmarkError,
    CandidateEvaluation, CandidateEvaluationInput, CandidateSafetyStatus, ConformanceRequest,
    ContextArtifactInput, ContextStabilityInputs, EnvironmentState, LayoutPlanningConstraints,
    MaterializationErrorCode, OrderedJsonObject, OrderingConstraint, RequestContext,
    RequestEnvelope, ToolDefinition,
};
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, ContextArtifact, Observed,
    TrustLevel, CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
use std::collections::BTreeMap;

fn request(ids: &[&str]) -> ConformanceRequest {
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
            tools: vec![
                ToolDefinition {
                    name: "read_file".to_string(),
                    description: "read a bounded file".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
                ToolDefinition {
                    name: "list_files".to_string(),
                    description: "list bounded files".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
            ],
        },
        envelope: RequestEnvelope {
            model: "fixture-model".to_string(),
            reasoning: None,
            response_format: None,
        },
    }
}

fn metadata(id: &str, stability: ArtifactStability) -> ContextArtifact {
    ContextArtifact {
        schema_version: CONTEXT_ARTIFACT_SCHEMA_VERSION,
        artifact_id: id.to_string(),
        origin_id: format!("origin-{id}"),
        content_source_id: Observed::Known(format!("source-{id}")),
        content_hash: Observed::Unknown,
        revision: Observed::Known("v1".to_string()),
        artifact_type: ArtifactType::SourceFile,
        stability,
        lifecycle: ArtifactLifecycle::PersistentVersioned,
        sizes: ArtifactSizes {
            byte_size: Observed::Known(10),
            ..ArtifactSizes::default()
        },
        cache: Default::default(),
        trust: Observed::Known(TrustLevel::Trusted),
        provenance: BTreeMap::new(),
        metadata: BTreeMap::new(),
    }
}

fn fixture() -> (
    ConformanceRequest,
    ContextStabilityInputs,
    prefixity_controlled_benchmark::ContextLayoutPlan,
) {
    let ids = ["a", "b", "c"];
    let request = request(&ids);
    let inputs = ContextStabilityInputs {
        artifacts: BTreeMap::from([
            ("a".to_string(), metadata("a", ArtifactStability::Stable)),
            ("b".to_string(), metadata("b", ArtifactStability::Volatile)),
            ("c".to_string(), metadata("c", ArtifactStability::Stable)),
        ]),
        ..ContextStabilityInputs::default()
    };
    let constraints = LayoutPlanningConstraints {
        constraints: ids
            .iter()
            .map(|id| OrderingConstraint::MovableWithinCompatibleRegion {
                segment: format!("context.artifacts[{id}]"),
                region: "artifact-sequence".to_string(),
            })
            .collect(),
        ..LayoutPlanningConstraints::default()
    };
    let plan = prefixity_controlled_benchmark::plan_request_layout(&request, &inputs, &constraints)
        .expect("fixture plan should be safe");
    (request, inputs, plan)
}

fn evaluation(candidate: &prefixity_controlled_benchmark::LayoutCandidate) -> CandidateEvaluation {
    evaluate_candidate(CandidateEvaluationInput {
        candidate,
        capability_profile: None,
        observations: &[],
        environment: &EnvironmentState::available(),
    })
    .expect("fixture evaluation should be valid")
}

fn error_code(error: BenchmarkError) -> MaterializationErrorCode {
    match error {
        BenchmarkError::Materialization { code, .. } => code,
        other => panic!("expected typed materialization error, got {other:?}"),
    }
}

fn materialize_fixture() -> (
    ConformanceRequest,
    ContextStabilityInputs,
    prefixity_controlled_benchmark::ContextLayoutPlan,
    prefixity_controlled_benchmark::MaterializedCandidate,
) {
    let (request, inputs, plan) = fixture();
    let candidate = plan.candidates[0].clone();
    let evaluation = evaluation(&candidate);
    let materialized = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &evaluation,
        &inputs,
        BTreeMap::from([("case".to_string(), "offline-fixture".to_string())]),
    )
    .expect("safe candidate should materialize");
    (request, inputs, plan, materialized)
}

#[test]
fn valid_reorder_is_certified_and_preserves_control_treatment_boundary() {
    let (request, _, plan, materialized) = materialize_fixture();
    assert_eq!(
        materialized.materialized_request.context.artifacts,
        vec![
            ContextArtifactInput {
                artifact_id: "a".to_string(),
                content: "artifact-a".to_string(),
            },
            ContextArtifactInput {
                artifact_id: "c".to_string(),
                content: "artifact-c".to_string(),
            },
            ContextArtifactInput {
                artifact_id: "b".to_string(),
                content: "artifact-b".to_string(),
            },
        ]
    );
    assert_eq!(
        materialized.safety_certificate.certification_status,
        prefixity_controlled_benchmark::CertificationStatus::CertifiedForExperimentMaterialization
    );
    assert_eq!(
        materialized
            .safety_certificate
            .request_diff_reference
            .source_request_fingerprint,
        request.request_fingerprint().unwrap()
    );
    assert!(materialized
        .safety_certificate
        .invariant_results
        .iter()
        .all(|result| result.passed));
    let pair = build_candidate_experiment_pair(
        &materialized,
        "control-source",
        "candidate-treatment",
        BTreeMap::from([("study".to_string(), "future-experiment".to_string())]),
    )
    .unwrap();
    assert_eq!(
        pair.source_request_fingerprint,
        request.request_fingerprint().unwrap()
    );
    assert_eq!(
        pair.candidate_request_fingerprint,
        materialized.materialized_request_fingerprint
    );
    assert_ne!(pair.source_case_id, pair.candidate_case_id);
    let serialized = serde_json::to_string(&pair).unwrap();
    assert!(!serialized.contains("optimized"));
    assert!(!serialized.contains("runtime_result"));
    assert_eq!(
        plan.candidates[0].layout_fingerprint,
        materialized.candidate_fingerprint
    );
}

#[test]
fn stale_source_fails_closed() {
    let (mut request, inputs, plan, _) = materialize_fixture();
    request.context.user_content.push_str(" changed");
    let candidate = plan.candidates[0].clone();
    let error = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &evaluation(&candidate),
        &inputs,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert_eq!(
        error_code(error),
        MaterializationErrorCode::StaleSourceRequest
    );
}

fn assert_source_drift_is_stale(mutator: fn(&mut ConformanceRequest)) {
    let (mut request, inputs, plan, _) = materialize_fixture();
    mutator(&mut request);
    let candidate = plan.candidates[0].clone();
    let error = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &evaluation(&candidate),
        &inputs,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert_eq!(
        error_code(error),
        MaterializationErrorCode::StaleSourceRequest
    );
}

#[test]
fn fixed_content_tools_and_envelope_drift_is_stale() {
    assert_source_drift_is_stale(|request| {
        request.context.system_instruction.push_str(" changed");
    });
    assert_source_drift_is_stale(|request| {
        request.context.user_content.push_str(" changed");
    });
    assert_source_drift_is_stale(|request| {
        request.context.tools[0].description.push_str(" changed");
    });
    assert_source_drift_is_stale(|request| {
        request.context.tools.reverse();
    });
    assert_source_drift_is_stale(|request| {
        request.envelope.model.push_str("-changed");
    });
}

#[test]
fn candidate_identity_content_and_metadata_changes_fail_with_bounded_codes() {
    let (request, inputs, plan, _) = materialize_fixture();

    let mut content = plan.candidates[0].clone();
    content.ordered_segments[2].content_fingerprint = "0".repeat(64);
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &plan,
                &content,
                &evaluation(&content),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::ArtifactContentMismatch
    );

    let mut metadata_change = plan.candidates[0].clone();
    metadata_change.ordered_segments[2].metadata_fingerprint = Some("0".repeat(64));
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &plan,
                &metadata_change,
                &evaluation(&metadata_change),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::TrustProvenanceMismatch
    );

    let mut omitted = plan.clone();
    omitted.candidates[0].ordered_segments.pop();
    let candidate = omitted.candidates[0].clone();
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &omitted,
                &candidate,
                &evaluation(&candidate),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::ArtifactMissing
    );

    let mut duplicated = plan.clone();
    duplicated.candidates[0].ordered_segments[2] =
        duplicated.candidates[0].ordered_segments[1].clone();
    let candidate = duplicated.candidates[0].clone();
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &duplicated,
                &candidate,
                &evaluation(&candidate),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::ArtifactDuplicated
    );
}

#[test]
fn unsafe_candidate_cannot_materialize_even_with_favourable_synthetic_evidence_shape() {
    let (request, inputs, mut plan, _) = materialize_fixture();
    plan.candidates[0].safety = CandidateSafetyStatus::Rejected;
    let candidate = plan.candidates[0].clone();
    let error = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &evaluation(&candidate),
        &inputs,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert_eq!(
        error_code(error),
        MaterializationErrorCode::CandidateSafetyRejected
    );
}

#[test]
fn planned_diff_and_structural_reanalysis_mismatches_are_rejected() {
    let (request, inputs, mut plan, _) = materialize_fixture();
    plan.candidates[0].request_diff =
        prefixity_controlled_benchmark::request_diff(&request, &request).unwrap();
    let candidate = plan.candidates[0].clone();
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &plan,
                &candidate,
                &evaluation(&candidate),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::PlannedActualDiffMismatch
    );

    let (request, inputs, mut plan, _) = materialize_fixture();
    plan.candidates[0]
        .resulting_analysis
        .leading_region
        .segment_count += 1;
    let candidate = plan.candidates[0].clone();
    assert_eq!(
        error_code(
            materialize_candidate(
                &request,
                &plan,
                &candidate,
                &evaluation(&candidate),
                &inputs,
                BTreeMap::new(),
            )
            .unwrap_err()
        ),
        MaterializationErrorCode::StructuralReanalysisMismatch
    );
}

#[test]
fn stale_evaluation_and_unsupported_shape_fail_without_fallback() {
    let (request, inputs, plan, _) = materialize_fixture();
    let candidate = plan.candidates[0].clone();
    let mut stale_evaluation = evaluation(&candidate);
    stale_evaluation.candidate.layout_fingerprint = "0".repeat(64);
    let error = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &stale_evaluation,
        &inputs,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert_eq!(
        error_code(error),
        MaterializationErrorCode::EvaluationMismatch
    );

    let mut unsupported = plan.clone();
    unsupported.candidates[0].transformations.clear();
    let candidate = unsupported.candidates[0].clone();
    let error = materialize_candidate(
        &request,
        &unsupported,
        &candidate,
        &evaluation(&candidate),
        &inputs,
        BTreeMap::new(),
    )
    .unwrap_err();
    assert_eq!(
        error_code(error),
        MaterializationErrorCode::UnsupportedTransformation
    );
}

#[test]
fn certificate_and_pair_are_deterministic_and_claim_no_runtime_result() {
    let (_, _, _, first) = materialize_fixture();
    let (_, _, _, second) = materialize_fixture();
    assert_eq!(
        first.canonical_json().unwrap(),
        second.canonical_json().unwrap()
    );
    assert_eq!(
        first.safety_certificate.certification_status,
        prefixity_controlled_benchmark::CertificationStatus::CertifiedForExperimentMaterialization
    );
    let first_pair = build_candidate_experiment_pair(
        &first,
        "control",
        "treatment",
        BTreeMap::from([("source".to_string(), "test".to_string())]),
    )
    .unwrap();
    let second_pair = build_candidate_experiment_pair(
        &second,
        "control",
        "treatment",
        BTreeMap::from([("source".to_string(), "test".to_string())]),
    )
    .unwrap();
    assert_eq!(first_pair, second_pair);
    assert!(!serde_json::to_string(&first).unwrap().contains("cache_hit"));
    assert!(!serde_json::to_string(&first)
        .unwrap()
        .contains("production_safe"));
}

#[test]
fn source_plan_and_evaluation_remain_unchanged() {
    let (request, inputs, plan) = fixture();
    let candidate = plan.candidates[0].clone();
    let evaluation = evaluation(&candidate);
    let request_before = request.clone();
    let plan_before = plan.clone();
    let evaluation_before = evaluation.clone();
    let _ = materialize_candidate(
        &request,
        &plan,
        &candidate,
        &evaluation,
        &inputs,
        BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(request, request_before);
    assert_eq!(plan, plan_before);
    assert_eq!(evaluation, evaluation_before);
}
