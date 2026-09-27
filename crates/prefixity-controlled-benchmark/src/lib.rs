//! Isolated Phase 1B.7 controlled benchmark implementation.
//!
//! This crate is offline-only. It owns the controlled envelope, deterministic
//! self-authored seed, scripted world, evaluation oracle, and a one-way
//! planner-visible projection. It does not alter `RequestTrace`, planner
//! eligibility, CodeTraceBench, or live-provider code.

mod candidate_evaluation;
mod capability_registry;
mod conformance;
mod context_stability;
mod diff;
mod error;
mod external_artifact_admission;
mod fixtures;
mod fresh_arm;
mod hashing;
mod layout_planner;
mod live_harness;
mod llama_cpp;
mod loader;
mod materialization;
mod model;
mod observation_diagnostics;
mod oracle;
mod paired_mutation;
mod phase1b9;
mod phase1c_capable_model_gate;
mod phase1c_claim2_workload;
mod phase1c_executable_identity;
mod phase1c_h001;
mod phase1c_h001_v2;
mod phase1c_live_supervisor;
mod phase1c_reasoning_budget_calibration;
mod phase1c_scored_design;
mod phase1c_stage0;
mod phase1c_stage1_local_qwen;
mod phase1c_stage1_reasoning_off_live;
mod phase1c_stage1_reasoning_off_postrun;
mod phase1c_stage1_reasoning_off_preflight;
mod phase1c_v3_feasibility;
mod phase1c_windows_runtime_exclusivity;
mod planner;
mod world;

pub use candidate_evaluation::{
    evaluate_candidate, CandidateEvaluation, CandidateEvaluationInput, CandidateHypothesis,
    CandidateReference, CapabilityAssessment, CapabilityGateAssessment, ClaimPermission,
    ClaimPermissions, DesignReadiness, EnvironmentReadiness, EnvironmentState,
    EvaluationProvenance, EvidenceBlocker, EvidenceState as CandidateEvidenceState,
    ExecutionReadiness, ExperimentReadiness, NextAction, ObservationEvidence, ObservationRelevance,
    ObservationRelevanceReason, RuntimeProfileReference as EvaluationRuntimeProfileReference,
    StructuralAssessment, CANDIDATE_EVALUATION_SCHEMA_ID, CANDIDATE_EVALUATION_SCHEMA_VERSION,
    CANDIDATE_EVALUATOR_VERSION, MAX_EVALUATION_BLOCKERS, MAX_EVALUATION_OBSERVATIONS,
    MAX_EVALUATION_PROVENANCE,
};
pub use capability_registry::{
    load_approved_capability_registry, load_capability_registry_from_paths, CapabilityCell,
    CapabilityGap, CapabilityKey, CapabilityMatrix, CapabilityMatrixRow, CapabilityProfile,
    CapabilityQuery, CapabilityRegistry, CapabilityState, ProfileGap, RegistryEvidenceOrigin,
    ResearchGapReport, APPROVED_CAPABILITY_FIXTURE_PATHS, CAPABILITY_REGISTRY_SCHEMA_ID,
    CAPABILITY_REGISTRY_SCHEMA_VERSION, MAX_REGISTRY_PROFILES, MAX_REGISTRY_PROVENANCE,
    MAX_REGISTRY_TEXT_BYTES,
};
pub use conformance::{
    CaseRelationship, CompletionStatus, ConformanceCase, ConformanceCaseResult,
    ConformanceExperiment, ConformanceRequest, ConformanceResult, ConformanceRunner,
    ContextArtifactInput, ExpectedObservationMetadata, ExpectedObservationState, JsonField,
    MockConformanceRunner, MutationClass, OrderedJsonObject, ReasoningSetting, RequestContext,
    RequestEnvelope, ResponseFormat, RuntimeProfileReference, ToolDefinition,
    CONFORMANCE_RESULT_SCHEMA_ID, CONFORMANCE_RESULT_SCHEMA_VERSION, CONFORMANCE_SCHEMA_ID,
    CONFORMANCE_SCHEMA_VERSION, MOCK_TRANSPORT_ID, MOCK_TRANSPORT_VERSION,
};
pub use context_stability::{
    analyze_context_stability, analyze_request_stability, BoundaryClassification,
    BoundaryDirection, ClassificationSource, ContextRole, ContextSegmentAnalysis,
    ContextStabilityAnalysis, ContextStabilityInputs, LeadingRegionLimit, SizeSource,
    StabilityAlignedLeadingRegion, StabilityBoundary, StabilityFinding, StabilityFindingKind,
    StabilitySummary, StructuralRoleDefault, StructuralRoleDefaults, CONTEXT_STABILITY_SCHEMA_ID,
    CONTEXT_STABILITY_SCHEMA_VERSION, MAX_STABILITY_BOUNDARIES, MAX_STABILITY_FINDINGS,
    MAX_STABILITY_PROVENANCE, MAX_STABILITY_SEGMENTS, MAX_STABILITY_TEXT_BYTES,
};
pub use diff::{
    envelope_diff, prefix_diff, request_diff, CacheImpactAssessment, ChangeCategory,
    CommonPrefixMeasurement, DiffChange, DiffState, EnvelopeChange, EnvelopeDiff, EnvelopeField,
    PrefixDiff, RequestDiff, RequestDiffInterpretation, TextCommonPrefix, ValueSummary,
    ENVELOPE_DIFF_SCHEMA_ID, ENVELOPE_DIFF_SCHEMA_VERSION, PREFIX_DIFF_SCHEMA_ID,
    PREFIX_DIFF_SCHEMA_VERSION, REQUEST_DIFF_SCHEMA_ID, REQUEST_DIFF_SCHEMA_VERSION,
};
pub use error::{BenchmarkError, LivePreparationErrorCode, MaterializationErrorCode};
pub use external_artifact_admission::{
    canonical_manifest_json, derive_admission, parse_manifest_json, validate_manifest,
    AdmissionDecision, AdmissionDecisionReport, AdmissionError, AdmissionReason,
    AdmissionReasonCode, AdmissionValidationError, AdmissionWarning, AdmissionWarningCode,
    ArtifactContentEvidence, ArtifactKind, ContentSufficiency, EvidenceRecord, EvidenceReference,
    EvidenceState, ExecutionRequirement, ExecutionRequirements, ExternalArtifactAdmissionManifest,
    ExternalArtifactAdmissionManifestV1, GitRetention, GitRetentionPolicy, GoldIndependence,
    GoldIndependenceEvidence, JoinAmbiguity, JoinClassification, JoinKeyKind, MaterialEvidence,
    MaterialPresence, OperationEvidence, ParentProjectIdentity, PermissionBasis,
    PermissionEvidence, PresenceStatus, PublicAccessibility, RequestedUse, RevisionKind,
    StableJoinEvidence, ThirdPartyMaterialEvidence, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID,
    EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION, MAX_MANIFEST_BYTES,
};
pub use fixtures::build_seed;
pub use fresh_arm::{
    aggregate_fresh_arm_results, execute_fresh_arm, finalize_fresh_arm_record,
    fresh_arm_config_fingerprint, persist_fresh_arm_record, preflight_fresh_arm_experiment,
    prepare_fresh_arm_experiment, FreshArmAggregationRecord, FreshArmDefinition,
    FreshArmExperimentDefinition, FreshArmKind, FreshArmReadiness, FreshArmReadinessRecord,
    FreshArmRunRecord, FreshArmStep, FRESH_ARM_AGGREGATION_SCHEMA_ID,
    FRESH_ARM_AGGREGATION_SCHEMA_VERSION, FRESH_ARM_SCHEMA_ID, FRESH_ARM_SCHEMA_VERSION,
};
pub use layout_planner::{
    plan_context_layout, plan_request_layout, CandidateSafetyStatus, ContextLayoutPlan,
    LayoutCandidate, LayoutPlanningConstraints, LayoutSegmentReference, LayoutStructuralMetrics,
    LayoutTransformation, LayoutTransformationKind, OrderingConstraint, PlanningReason,
    PreserveOrderReason, RejectedLayoutCandidate, RejectionReason, StructuralLayoutEffect,
    CONTEXT_LAYOUT_PLAN_SCHEMA_ID, CONTEXT_LAYOUT_PLAN_SCHEMA_VERSION, MAX_LAYOUT_CANDIDATES,
    MAX_LAYOUT_CONSTRAINTS, MAX_LAYOUT_PROVENANCE, MAX_LAYOUT_REJECTIONS, MAX_LAYOUT_TEXT_BYTES,
};
pub use live_harness::{
    build_live_experiment_definition, execute_live_experiment, live_experiment_identity,
    preflight_live_experiment, EnvironmentObservation, LiveEnvironmentManifest, LiveEvidenceState,
    LiveExperimentDefinition, LiveFailure, LiveRawEvidenceSource, LiveReadinessRecord,
    LiveRunRecord, LiveSequenceRelation, LiveSequenceRole, LiveSequenceStep, LlamaCppLiveConfig,
    LoopbackEndpoint, LoopbackLlamaCppTransport, RawLlamaCppEvidence,
    ENVIRONMENT_MANIFEST_SCHEMA_ID, ENVIRONMENT_MANIFEST_SCHEMA_VERSION, LIVE_CONFIG_SCHEMA_ID,
    LIVE_CONFIG_SCHEMA_VERSION, LIVE_HARNESS_SCHEMA_ID, LIVE_HARNESS_SCHEMA_VERSION,
    RAW_EVIDENCE_SCHEMA_ID, RAW_EVIDENCE_SCHEMA_VERSION,
};
pub use llama_cpp::{
    normalize_llama_cpp_response, project_llama_cpp_request,
    project_llama_cpp_request_with_generation_limit, validate_llama_cpp_generation_limit,
    FakeLlamaCppTransport, LlamaCppConformanceRunner, LlamaCppFunction, LlamaCppJsonObject,
    LlamaCppMessage, LlamaCppPromptTokenDetails, LlamaCppRequest, LlamaCppResponse,
    LlamaCppResponseFormat, LlamaCppTimings, LlamaCppTool, LlamaCppTransport, LlamaCppUsage,
    LLAMA_CPP_ADAPTER_VERSION, LLAMA_CPP_PROTOCOL_ID,
};
pub use loader::{
    canonical_envelope_json, envelope_hash, load_envelope, load_envelope_from_path, manifest_hash,
    validate_case, validate_envelope,
};
pub use materialization::{
    build_candidate_experiment_pair, materialize_candidate, CandidateExperimentPair,
    CertificationStatus, InvariantResult, MaterializationInvariant,
    MaterializationSafetyCertificate, MaterializedCandidate, RequestDiffReference,
    EXPERIMENT_PAIR_SCHEMA_ID, EXPERIMENT_PAIR_SCHEMA_VERSION, MATERIALIZATION_SCHEMA_ID,
    MATERIALIZATION_SCHEMA_VERSION, MAX_EXPERIMENT_CASE_ID_BYTES, MAX_MATERIALIZATION_PROVENANCE,
    SAFETY_CERTIFICATE_SCHEMA_ID, SAFETY_CERTIFICATE_SCHEMA_VERSION,
};
pub use model::{
    ActionIdentity, ActorRole, AggregateCounts, BenchmarkReport, ControlledCase,
    ControlledEnvelope, EvaluationRecord, Event, EventType, EvidenceClass, InterventionClass,
    InterventionManifest, OracleResult, OrderMetadata, PlannerEvidence, PlannerInput, PlannerRun,
    PlannerVisibility, QualityRiskCategory, Relation, RelationType, ScenarioIdentity, SourceKind,
    SourceProvenance, TimestampOrigin, TraceEnvelope, VariantRole, BENCHMARK_ID,
    ENVIRONMENT_REVISION, ORACLE_VERSION, RELATION_SEMANTICS_VERSION, SCHEMA_ID, SCHEMA_VERSION,
    TASK_REVISION,
};
pub use observation_diagnostics::{
    compare_conformance_cases, compare_observations, diagnose_cache, diagnose_conformance_cache,
    diagnose_conformance_cache_with_source, CacheDiagnostic, CacheRegressionAssessment,
    CausalityStatus, ComparabilityLevel, ComparabilityReason, ComparabilityReport, DerivedMetrics,
    DerivedRatio, DiagnosticMetric, EvidenceAssociation, EvidenceSourceClass, EvidenceStatement,
    IdentityComparison, IdentityMatch, MetricDirection, NumericMetricDelta, ObservationComparison,
    ObservationReference, RequestObservationAlignment, ResourceDeltas, RuntimeIdentityReference,
    TimingDeltas, TokenDeltas, TokenMetricDelta, TokenMetricName, CACHE_DIAGNOSTIC_SCHEMA_ID,
    CACHE_DIAGNOSTIC_SCHEMA_VERSION, OBSERVATION_COMPARISON_SCHEMA_ID,
    OBSERVATION_COMPARISON_SCHEMA_VERSION,
};
pub use oracle::{evaluate_case, evaluate_envelopes};
pub use paired_mutation::{
    build_paired_mutation_conformance_experiment, build_synthetic_paired_mutation_seed,
    execute_paired_mutation_experiment, live_paired_mutation_identity,
    preflight_paired_mutation_experiment, prepare_paired_mutation_experiment, PairedComparisonKind,
    PairedMutationComparison, PairedMutationDefinition, PairedMutationRunRecord,
    PairedMutationSeed, PairedMutationSequenceRelation, PairedMutationSequenceRole,
    PairedMutationSequenceStep, PairedOutcomeExpectation, PairedReadinessRecord,
    PairedWorkloadSummary, PAIRED_MUTATION_SCHEMA_ID, PAIRED_MUTATION_SCHEMA_VERSION,
};
pub use phase1b9::{
    blinded_trace_json, canonical_phase1b9_report_json, preregistration_hash, run_phase1b9_study,
    BlindedEvent, BlindedRelation, BlindedTrace, FrozenPlannerBaseline, Phase1b9DecisionRecord,
    Phase1b9Report, ResearchInterventionClass, ResearchPolicyCandidate, ResearchPolicyCandidates,
    ResearchPolicyDecision, PHASE_1B9_POLICY_VERSION, PHASE_1B9_SCOPE,
};
pub use phase1c_capable_model_gate::{
    classify_local_9b_gate, derive_local_9b_gate_spec, dry_run_local_9b_gate,
    execute_local_9b_feasibility_gate, freeze_local_9b_gate, local_9b_gate_permission,
    preflight_local_9b_gate, validate_local_9b_contract, validate_local_9b_live_prerequisites,
    CapableModelGateSpec, LOCAL_9B_BOUND_SOURCES, LOCAL_9B_CONTRACT_PATH,
    LOCAL_9B_FEASIBILITY_FAILED, LOCAL_9B_FEASIBILITY_PASSED, LOCAL_9B_GATE_EVIDENCE_ROOT,
    LOCAL_9B_GATE_FROZEN_STAGE_ROOT, LOCAL_9B_GATE_IDENTITY_PATH, LOCAL_9B_GATE_TRAVERSAL_ROOT,
};
pub use phase1c_claim2_workload::{
    apply as apply_claim2_decision, canonical_claim2_action_output, compare_paired_arms,
    evaluate as evaluate_claim2_arm, load_case as load_claim2_case, materiality_result,
    precheck_cohort as precheck_claim2_cohort, project_trace as project_claim2_trace,
    render as render_claim2_request, select as select_claim2_decision,
    token_guard as claim2_token_guard, validate_case as validate_claim2_case,
    Claim2ActionTransition, Claim2ArmState, Claim2AssetKind, Claim2CaseKind, Claim2CaseManifest,
    Claim2EnvironmentReceipt, Claim2Evaluation, Claim2EvaluationKey, Claim2MaterialityResult,
    Claim2PairComparison, Claim2PinnedAsset, Claim2PrecheckSlot, Claim2ProjectedTrace,
    Claim2ProjectionMode, Claim2PromptMessage, Claim2PromptPart, Claim2PromptRole,
    Claim2ReceiptSize, Claim2RenderedMessage, Claim2RenderedRequest, Claim2RequestMetrics,
    Claim2RequestTemplate, Claim2Selection, Claim2SlotRecord, Claim2SlotStatus,
    Claim2TokenCountStatus, Claim2TokenGuardResult, Claim2TokenProofInputs, LoadedClaim2Case,
    CLAIM2_ADVANCING_OUTPUT_PROTOCOL_ID, CLAIM2_CASE_SCHEMA_ID, CLAIM2_CASE_SCHEMA_VERSION,
    CLAIM2_CONTEXT_TOKENS, CLAIM2_INPUT_PREFLIGHT_TOKENS, CLAIM2_MAX_OUTPUT_TOKENS,
    CLAIM2_MODEL_LABEL,
};
pub use phase1c_executable_identity::{
    freeze_copy, inspect as inspect_executable_identity, validate_frozen_executable_binding,
    ExecutableIdentity, FrozenExecutableBinding,
};
pub use phase1c_h001::{
    dry_run_h001, execute_h001_arm, fingerprint_h001, preflight_h001, score_h001_arm, H001Arm,
    H001Error,
};
pub use phase1c_h001_v2::{
    dry_run_v2, dry_run_v2_attempt_002, execute_v2_baseline, fingerprint_v2, parse_v2_cli_args,
    preflight_v2, preflight_v2_attempt_002, score_v2_baseline, v2_live_child_args, V2CliCommand,
    V2_BASELINE_IDENTITY_FINGERPRINT_PATH, V2_BASELINE_IDENTITY_PATH, V2_CONTRACT_FINGERPRINT_PATH,
    V2_CONTRACT_PATH, V2_EVIDENCE_ROOT, V2_H001_ATTEMPT_002_EVIDENCE_ROOT,
    V2_H001_ATTEMPT_002_FINGERPRINT_PATH, V2_H001_ATTEMPT_002_IDENTITY_PATH, V2_H001_EVIDENCE_ROOT,
    V2_LIVE_CONFIRMATION_FLAG, V2_PILOT_FINGERPRINT_PATH, V2_PILOT_MANIFEST_PATH,
};
pub use phase1c_live_supervisor::LOCAL_9B_FEASIBILITY_GATE_IDENTITY_PREFIX;
pub use phase1c_live_supervisor::{
    build_workflow_launch_metadata, persist_supervisor_result,
    registered_workflow_identity_from_file, run_supervised,
    run_supervised_with_registered_workflow_identity, workflow_launch_metadata_from_env,
    RegisteredWorkflowIdentity, WorkflowLaunchMetadata, PRODUCTION_SUPERVISOR_TIMEOUT_MS,
    WORKFLOW_HANDOFF_ENV, WORKFLOW_HANDOFF_SCHEMA_ID,
};
pub use phase1c_live_supervisor::{
    registered_supervisor_deadline_ms, V3_FEASIBILITY_GATE_IDENTITY_PREFIX,
};
pub use phase1c_reasoning_budget_calibration::{
    attempt_002_exclusivity_preflight, attempt_002_runtime_ownership, attempt_003_poststart,
    attempt_003_runtime_ownership, attempt_003_runtime_ownership_with_expected_workflow,
    attempt_005_poststart, attempt_006_poststart, attempt_007_poststart,
    certify_preserved_calibration_evidence, certify_workflow_identity, dry_run_attempt_002,
    dry_run_attempt_003, dry_run_attempt_004, dry_run_attempt_004_portable, dry_run_attempt_005,
    dry_run_attempt_006, dry_run_attempt_007, dry_run_attempt_008, dry_run_attempt_009,
    dry_run_attempt_010, dry_run_attempt_011, dry_run_calibration, execute_attempt_002,
    execute_attempt_003, execute_attempt_007, execute_attempt_008, execute_attempt_009,
    execute_attempt_010, execute_attempt_011, execute_calibration,
    expected_workflow_identity_from_supervisor_env, fingerprint_attempt_002,
    fingerprint_attempt_003, fingerprint_attempt_004, fingerprint_attempt_005,
    fingerprint_attempt_006, fingerprint_attempt_007, fingerprint_attempt_008,
    fingerprint_attempt_009, fingerprint_attempt_010, fingerprint_attempt_011,
    fingerprint_calibration, freeze_attempt_008, freeze_attempt_009, freeze_attempt_010,
    freeze_attempt_011, next_candidate_budget, parse_calibration_cli_args, preflight_attempt_002,
    preflight_attempt_003, preflight_attempt_004, preflight_attempt_005, preflight_attempt_006,
    preflight_attempt_007, preflight_attempt_008, preflight_attempt_009, preflight_attempt_010,
    preflight_attempt_011, preflight_calibration, resolve_authoritative_candidate_transition,
    summarize_attempt_002_budget, summarize_calibration_budget,
    validate_attempt_007_execution_evidence, validate_attempt_008_preparation,
    validate_attempt_009_candidate_order, validate_attempt_009_preparation,
    validate_attempt_010_identity_document, validate_attempt_010_live_prerequisites,
    validate_attempt_010_preparation, validate_attempt_010_repository_contract,
    validate_attempt_011_identity_document, validate_attempt_011_live_prerequisites,
    validate_attempt_011_preparation, validate_attempt_011_repository_contract,
    validate_candidate_order_report, validate_recorded_workflow_evidence_against_preparation,
    CalibrationCliCommand, ReasoningBudgetCalibrationError, CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT,
    CALIBRATION_ATTEMPT_002_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_002_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT, CALIBRATION_ATTEMPT_003_IDENTITY_FINGERPRINT_PATH,
    CALIBRATION_ATTEMPT_003_IDENTITY_PATH, CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT,
    CALIBRATION_ATTEMPT_004_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_004_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT, CALIBRATION_ATTEMPT_005_IDENTITY_FINGERPRINT_PATH,
    CALIBRATION_ATTEMPT_005_IDENTITY_PATH, CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT,
    CALIBRATION_ATTEMPT_006_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_006_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT, CALIBRATION_ATTEMPT_007_IDENTITY_FINGERPRINT_PATH,
    CALIBRATION_ATTEMPT_007_IDENTITY_PATH, CALIBRATION_ATTEMPT_008_BUDGET_PROVENANCE_PATH,
    CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT, CALIBRATION_ATTEMPT_008_FROZEN_STAGE_ROOT,
    CALIBRATION_ATTEMPT_008_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_008_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH, CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT,
    CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH, CALIBRATION_ATTEMPT_009_FROZEN_STAGE_ROOT,
    CALIBRATION_ATTEMPT_009_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_009_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT, CALIBRATION_ATTEMPT_010_FROZEN_STAGE_ROOT,
    CALIBRATION_ATTEMPT_010_IDENTITY_FINGERPRINT_PATH, CALIBRATION_ATTEMPT_010_IDENTITY_PATH,
    CALIBRATION_ATTEMPT_010_PREREQUISITE_TRAVERSAL_ROOT, CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT,
    CALIBRATION_ATTEMPT_011_FROZEN_STAGE_ROOT, CALIBRATION_ATTEMPT_011_IDENTITY_FINGERPRINT_PATH,
    CALIBRATION_ATTEMPT_011_IDENTITY_PATH, CALIBRATION_ATTEMPT_011_PREREQUISITE_TRAVERSAL_ROOT,
    CALIBRATION_BUDGETS, CALIBRATION_CANDIDATE_TRANSITIONS_PATH, CALIBRATION_CASE_IDS,
    CALIBRATION_EVIDENCE_ROOT, CALIBRATION_MANIFEST_FINGERPRINT_PATH, CALIBRATION_MANIFEST_PATH,
    CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH, CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    WORKFLOW_CERTIFICATION_IDENTITY_PATH, WORKFLOW_CERTIFICATION_RESULT_SCHEMA_ID,
};
pub use phase1c_scored_design::{
    fingerprint_scored_design, validate_scored_design, ScoredDesignError, ScoredDesignFingerprint,
    SCORED_CONTRACT_FINGERPRINT_PATH, SCORED_CONTRACT_PATH, SCORED_CONTRACT_SCHEMA_PATH,
    SCORED_PILOT_FINGERPRINT_PATH, SCORED_PILOT_MANIFEST_PATH,
};
pub use phase1c_stage0::{
    canonical_stage0_report_json, run_stage0_certification, stage0_design_hash, Stage0AbortProbe,
    Stage0CertificationStatus, Stage0EfficiencyGateResult, Stage0Manifest, Stage0Report,
    Stage0TaskIdentity, Stage0TaskRecord, STAGE0_ABORT_POLICY_VERSION, STAGE0_EVALUATOR_VERSION,
    STAGE0_MOCK_TRANSPORT_SCHEMA_VERSION, STAGE0_REDACTION_VERSION, STAGE0_REPORT_SCHEMA_VERSION,
    STAGE0_RUNNER_VERSION,
};
pub use phase1c_stage1_local_qwen::{
    execute_stage1_smoke, preflight_stage1_smoke, Stage1Error, Stage1Preflight, Stage1RunRecord,
    STAGE1_CONTRACT_PATH, STAGE1_EVIDENCE_DIR, STAGE1_FIXTURE_PATH, STAGE1_OUTPUT_SCHEMA_PATH,
    STAGE1_REQUEST_SCHEMA_PATH,
};
pub use phase1c_stage1_reasoning_off_live::{
    execute_stage1_reasoning_off_smoke, ReasoningOffLiveError,
};
pub use phase1c_stage1_reasoning_off_postrun::{
    validate_stage1_reasoning_off_evidence, ReasoningOffPostrunError,
};
pub use phase1c_stage1_reasoning_off_preflight::{
    certify_stage1_reasoning_off_preserved_evidence, preflight_stage1_reasoning_off_smoke,
    validate_stage1_reasoning_off_contract_and_fixture, RemediationPreflight,
    RemediationPreflightError,
};
pub use phase1c_v3_feasibility::{
    classify_gate, context_guard, derive_gate_spec, dry_run_v3_feasibility_gate,
    execute_v3_feasibility_gate, freeze_v3_feasibility_gate, gate_permission,
    preflight_v3_feasibility_gate, validate_gate_identity_document, validate_gate_spec,
    validate_v3_contract, validate_v3_live_prerequisites, ContextDecision, V3GateSpec,
    CURRENT_QWEN_SCORED_PATH_CLOSED, INCONCLUSIVE_CONTEXT_BOUND, V3_CONTRACT_PATH,
    V3_FEASIBILITY_PASSED, V3_GATE_EVIDENCE_ROOT, V3_GATE_FROZEN_STAGE_ROOT,
    V3_GATE_IDENTITY_FINGERPRINT_PATH, V3_GATE_IDENTITY_PATH, V3_GATE_TRAVERSAL_ROOT,
    V3_MAX_TOKENS,
};
pub use planner::{project_planner_evidence, run_frozen_planner};
pub use world::{ExecutionStatus, ScriptedWorld, WorldExecution};

/// Build, evaluate, and run the frozen planner over every self-authored pair.
///
/// The evaluation sidecar is consumed only by the oracle. The planner runs
/// are created from `PlannerEvidence` projections before any evaluation result
/// is produced or consulted.
pub fn run_benchmark() -> Result<BenchmarkReport, BenchmarkError> {
    let cases = build_seed()?;
    let mut evaluations = Vec::with_capacity(cases.len());
    let mut planner_runs = Vec::with_capacity(cases.len() * 2);
    let mut manifest_hashes = std::collections::BTreeMap::new();
    let mut baseline_count = 0;
    let mut variant_count = 0;
    let mut control_count = 0;

    for case in &cases {
        manifest_hashes.insert(case.scenario_id.clone(), case.manifest_hash.clone());
        baseline_count += 1;
        match case.intervention.trace.variant_role {
            VariantRole::Variant => variant_count += 1,
            VariantRole::Control => control_count += 1,
            VariantRole::Baseline => {
                return Err(BenchmarkError::pair(
                    &case.scenario_id,
                    "intervention unexpectedly has baseline role",
                ));
            }
        }

        let baseline_evidence = project_planner_evidence(&case.baseline)?;
        let intervention_evidence = project_planner_evidence(&case.intervention)?;
        planner_runs.push(run_frozen_planner(&baseline_evidence)?);
        planner_runs.push(run_frozen_planner(&intervention_evidence)?);

        evaluations.push(evaluate_case(case)?);
    }

    let aggregate_input = serde_json::json!({
        "artifact_id": BENCHMARK_ID,
        "schema_id": SCHEMA_ID,
        "schema_version": SCHEMA_VERSION,
        "oracle_version": ORACLE_VERSION,
        "manifest_hashes": manifest_hashes,
        "evaluations": evaluations,
        "planner_runs": planner_runs,
    });
    let aggregate_hash = hashing::canonical_hash(&aggregate_input)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let mut aggregate_counts = AggregateCounts {
        pass: 0,
        fail: 0,
        invalid_baseline: 0,
        inconclusive: 0,
    };
    for evaluation in &evaluations {
        aggregate_counts.record(evaluation.result);
    }

    Ok(BenchmarkReport {
        artifact_id: BENCHMARK_ID.to_string(),
        schema_id: SCHEMA_ID.to_string(),
        schema_version: SCHEMA_VERSION,
        oracle_version: ORACLE_VERSION.to_string(),
        scenario_count: cases.len(),
        baseline_count,
        variant_count,
        control_count,
        manifest_hashes,
        aggregate_hash,
        evaluations,
        aggregate_counts,
        planner_runs,
    })
}

pub fn canonical_report_json(report: &BenchmarkReport) -> Result<Vec<u8>, BenchmarkError> {
    hashing::canonical_json(report).map_err(|error| BenchmarkError::validation(error.to_string()))
}
