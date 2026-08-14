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
mod phase1c_stage0;
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
    Phase1b9Report, ResearchInterventionClass, ResearchPolicyDecision, PHASE_1B9_POLICY_VERSION,
    PHASE_1B9_SCOPE,
};
pub use phase1c_stage0::{
    canonical_stage0_report_json, run_stage0_certification, stage0_design_hash, Stage0AbortProbe,
    Stage0CertificationStatus, Stage0EfficiencyGateResult, Stage0Manifest, Stage0Report,
    Stage0TaskIdentity, Stage0TaskRecord, STAGE0_ABORT_POLICY_VERSION, STAGE0_EVALUATOR_VERSION,
    STAGE0_MOCK_TRANSPORT_SCHEMA_VERSION, STAGE0_REDACTION_VERSION, STAGE0_REPORT_SCHEMA_VERSION,
    STAGE0_RUNNER_VERSION,
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
