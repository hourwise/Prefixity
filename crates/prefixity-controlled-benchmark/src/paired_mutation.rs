//! P0-L6B paired mutation preparation.
//!
//! This is an additive experiment path beside P0-L6A. It prepares a vanilla
//! control mutation and the same mutation under an independently certified
//! P0-L13 layout. It never opens a socket, starts a runtime, or asserts a
//! positive outcome.

use crate::candidate_evaluation::{evaluate_candidate, CandidateEvaluationInput};
use crate::conformance::{
    CaseRelationship, ConformanceCase, ConformanceExperiment, ConformanceRequest,
    ConformanceResult, ExpectedObservationMetadata, ExpectedObservationState, MutationClass,
    RuntimeProfileReference,
};
use crate::context_stability::{analyze_context_stability, ContextStabilityInputs};
use crate::diff::{request_diff, RequestDiff};
use crate::error::{BenchmarkError, LivePreparationErrorCode};
use crate::hashing::{canonical_hash, canonical_json};
use crate::layout_planner::{plan_request_layout, LayoutPlanningConstraints, OrderingConstraint};
use crate::live_harness::{
    LiveEvidenceState, LiveRawEvidenceSource, LlamaCppLiveConfig, RawLlamaCppEvidence,
};
use crate::llama_cpp::{project_llama_cpp_request, LlamaCppConformanceRunner, LlamaCppTransport};
use crate::materialization::{materialize_candidate, MaterializedCandidate};
use prefixity_core::model::{EvidenceOrigin, EvidenceProvenance};
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, ContextArtifact, Observed,
    TrustLevel, CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PAIRED_MUTATION_SCHEMA_ID: &str = "prefixity.llama-cpp-paired-mutation";
pub const PAIRED_MUTATION_SCHEMA_VERSION: u32 = 1;
const PAIRED_SEQUENCE_LENGTH: usize = 5;
const MAX_WORKLOAD_BYTES: usize = 256 * 1024;
const TARGET_WORKLOAD_BYTES: usize = 12 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairedMutationSequenceRole {
    Control,
    CandidateTreatment,
    Interference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "case_id")]
pub enum PairedMutationSequenceRelation {
    Initial,
    VolatileMutationOf(String),
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedMutationSequenceStep {
    pub step_id: String,
    pub role: PairedMutationSequenceRole,
    pub request_fingerprint: String,
    pub relation: PairedMutationSequenceRelation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairedComparisonKind {
    ControlMutation,
    TreatmentMutation,
    PrimaryMutationOutcome,
    ControlLayout,
    TreatmentLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedMutationComparison {
    pub comparison_id: String,
    pub kind: PairedComparisonKind,
    pub left_request_fingerprint: String,
    pub right_request_fingerprint: String,
    pub request_diff: RequestDiff,
    pub interpretation: String,
    pub cache_outcome: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairedOutcomeExpectation {
    NoRequiredDirection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedWorkloadSummary {
    pub target_bytes: usize,
    pub control_initial_bytes: usize,
    pub control_mutated_bytes: usize,
    pub treatment_initial_bytes: usize,
    pub treatment_mutated_bytes: usize,
    pub context_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedMutationDefinition {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub control_initial: ConformanceRequest,
    pub control_mutated: ConformanceRequest,
    pub interference: ConformanceRequest,
    pub treatment_initial: MaterializedCandidate,
    pub treatment_mutated: MaterializedCandidate,
    pub control_mutation: PairedMutationComparison,
    pub treatment_mutation: PairedMutationComparison,
    pub control_layout: PairedMutationComparison,
    pub treatment_layout: PairedMutationComparison,
    pub primary_comparison: PairedMutationComparison,
    pub sequence: Vec<PairedMutationSequenceStep>,
    pub workload: PairedWorkloadSummary,
    pub control_initial_inversion_count: usize,
    pub control_mutated_inversion_count: usize,
    pub treatment_initial_inversion_count: usize,
    pub treatment_mutated_inversion_count: usize,
    pub control_initial_leading_segments: usize,
    pub treatment_initial_leading_segments: usize,
    pub expected_outcome: PairedOutcomeExpectation,
    pub fresh_server_for_run: bool,
    pub runtime_profile: RuntimeProfileReference,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl PairedMutationDefinition {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != PAIRED_MUTATION_SCHEMA_ID
            || self.schema_version != PAIRED_MUTATION_SCHEMA_VERSION
        {
            return Err(paired_error("unsupported paired mutation schema"));
        }
        for request in [
            &self.control_initial,
            &self.control_mutated,
            &self.interference,
            &self.treatment_initial.materialized_request,
            &self.treatment_mutated.materialized_request,
        ] {
            request.validate()?;
            let projected = project_llama_cpp_request(request)?;
            let bytes = serde_json::to_vec(&projected)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?;
            if bytes.len() > MAX_WORKLOAD_BYTES {
                return Err(paired_error("paired projected workload exceeds its bound"));
            }
        }
        self.treatment_initial
            .validate()
            .map_err(|error| paired_error(format!("C0 is not independently certified: {error}")))?;
        self.treatment_mutated
            .validate()
            .map_err(|error| paired_error(format!("C1 is not independently certified: {error}")))?;
        if !self.fresh_server_for_run {
            return Err(paired_error(
                "paired run requires fresh_server_for_run=true",
            ));
        }
        if self.expected_outcome != PairedOutcomeExpectation::NoRequiredDirection {
            return Err(paired_error(
                "paired outcome must not require a positive direction",
            ));
        }
        validate_request_diff(&self.control_mutation.request_diff, false)?;
        validate_request_diff(&self.treatment_mutation.request_diff, false)?;
        validate_request_diff(&self.control_layout.request_diff, true)?;
        validate_request_diff(&self.treatment_layout.request_diff, true)?;
        if self.primary_comparison.request_diff.envelope_diff.changes
            != self.control_mutation.request_diff.envelope_diff.changes
        {
            return Err(paired_error(
                "primary comparison must preserve the neutral envelope",
            ));
        }
        let control_initial = self.control_initial.request_fingerprint()?;
        let control_mutated = self.control_mutated.request_fingerprint()?;
        let treatment_initial = self
            .treatment_initial
            .materialized_request_fingerprint
            .clone();
        let treatment_mutated = self
            .treatment_mutated
            .materialized_request_fingerprint
            .clone();
        let interference = self.interference.request_fingerprint()?;
        let expected = [
            (
                "A0",
                PairedMutationSequenceRole::Control,
                control_initial.clone(),
                PairedMutationSequenceRelation::Initial,
            ),
            (
                "A1",
                PairedMutationSequenceRole::Control,
                control_mutated.clone(),
                PairedMutationSequenceRelation::VolatileMutationOf("A0".to_string()),
            ),
            (
                "B1",
                PairedMutationSequenceRole::Interference,
                interference,
                PairedMutationSequenceRelation::Independent,
            ),
            (
                "C0",
                PairedMutationSequenceRole::CandidateTreatment,
                treatment_initial.clone(),
                PairedMutationSequenceRelation::Initial,
            ),
            (
                "C1",
                PairedMutationSequenceRole::CandidateTreatment,
                treatment_mutated.clone(),
                PairedMutationSequenceRelation::VolatileMutationOf("C0".to_string()),
            ),
        ];
        if self.sequence.len() != PAIRED_SEQUENCE_LENGTH
            || self
                .sequence
                .iter()
                .zip(expected)
                .any(|(actual, expected)| {
                    actual.step_id != expected.0
                        || actual.role != expected.1
                        || actual.request_fingerprint != expected.2
                        || actual.relation != expected.3
                })
        {
            return Err(paired_error(
                "paired sequence must be exactly A0/A1/B1/C0/C1",
            ));
        }
        if self.control_mutation.left_request_fingerprint != control_initial
            || self.control_mutation.right_request_fingerprint != control_mutated
            || self.treatment_mutation.left_request_fingerprint
                != self.treatment_initial.materialized_request_fingerprint
            || self.treatment_mutation.right_request_fingerprint
                != self.treatment_mutated.materialized_request_fingerprint
        {
            return Err(paired_error("paired mutation identities are not traceable"));
        }
        if self.control_layout.left_request_fingerprint
            != self.control_initial.request_fingerprint()?
            || self.control_layout.right_request_fingerprint != treatment_initial
            || self.treatment_layout.left_request_fingerprint
                != self.control_mutated.request_fingerprint()?
            || self.treatment_layout.right_request_fingerprint != treatment_mutated
        {
            return Err(paired_error("paired layout identities are not traceable"));
        }
        if self.workload.context_limit != 8192
            || self.workload.control_initial_bytes == 0
            || self.workload.control_initial_bytes > MAX_WORKLOAD_BYTES
        {
            return Err(paired_error(
                "paired workload is outside its bounded context contract",
            ));
        }
        if self.treatment_initial_inversion_count > self.control_initial_inversion_count
            || self.treatment_mutated_inversion_count > self.control_mutated_inversion_count
            || self.treatment_initial_leading_segments < self.control_initial_leading_segments
        {
            return Err(paired_error(
                "P0-L10 treatment metrics do not represent an inversion reduction",
            ));
        }
        validate_metadata(&self.provenance)
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| BenchmarkError::validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedMutationSeed {
    pub control_initial: ConformanceRequest,
    pub control_mutated: ConformanceRequest,
    pub interference: ConformanceRequest,
    pub stability_inputs: ContextStabilityInputs,
    pub constraints: LayoutPlanningConstraints,
    pub workload: PairedWorkloadSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedReadinessRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub state: LiveEvidenceState,
    pub network_calls: u32,
    pub experiment_id: String,
    pub sequence_step_ids: Vec<String>,
    pub fresh_server_for_run: bool,
    pub runtime_config_fingerprint: String,
    pub comparisons: Vec<String>,
    pub primary_outcome: String,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedMutationRunRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub state: LiveEvidenceState,
    pub expected_steps: usize,
    pub completed_steps: usize,
    pub raw_evidence: Vec<RawLlamaCppEvidence>,
    pub normalized_result: Option<ConformanceResult>,
    pub failure: Option<String>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl PairedMutationRunRecord {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != PAIRED_MUTATION_SCHEMA_ID
            || self.schema_version != PAIRED_MUTATION_SCHEMA_VERSION
            || self.expected_steps != PAIRED_SEQUENCE_LENGTH
            || self.completed_steps > self.expected_steps
        {
            return Err(paired_error(
                "paired run record has invalid bounds or schema",
            ));
        }
        match self.state {
            LiveEvidenceState::Normalized | LiveEvidenceState::Admitted => {
                if self.completed_steps != self.expected_steps || self.normalized_result.is_none() {
                    return Err(paired_error(
                        "complete paired state requires five steps and a normalized result",
                    ));
                }
            }
            LiveEvidenceState::Partial | LiveEvidenceState::Failed => {
                if self.normalized_result.is_some() {
                    return Err(paired_error(
                        "partial or failed paired runs cannot contain a final result",
                    ));
                }
            }
            LiveEvidenceState::Prepared | LiveEvidenceState::Executed => {}
        }
        validate_metadata(&self.provenance)
    }
}

pub fn build_synthetic_paired_mutation_seed() -> Result<PairedMutationSeed, BenchmarkError> {
    let stable_a = bounded_material("stable-a", "A");
    let stable_b = bounded_material("stable-b", "B");
    let volatile_v0 = bounded_material("volatile-v0", "V0");
    let volatile_v1 = bounded_material("volatile-v1", "V1");
    let context_prefix = format!(
        "Synthetic paired-mutation workload. {}",
        "stable context ".repeat(170)
    );
    let make_request = |volatile: &str| ConformanceRequest {
        context: crate::conformance::RequestContext {
            system_instruction: "Synthetic semantically independent cache experiment.".to_string(),
            artifacts: vec![
                ContextArtifactInput {
                    artifact_id: "stable-a".to_string(),
                    content: format!("{context_prefix}{stable_a}"),
                },
                ContextArtifactInput {
                    artifact_id: "volatile-v".to_string(),
                    content: format!("{context_prefix}{volatile}"),
                },
                ContextArtifactInput {
                    artifact_id: "stable-b".to_string(),
                    content: format!("{context_prefix}{stable_b}"),
                },
            ],
            user_content: "Return the bounded synthetic result.".to_string(),
            tools: Vec::new(),
        },
        envelope: crate::conformance::RequestEnvelope {
            model: "fixture-model".to_string(),
            reasoning: None,
            response_format: None,
        },
    };
    let control_initial = make_request(&volatile_v0);
    let control_mutated = make_request(&volatile_v1);
    let mut interference = control_initial.clone();
    interference.context.system_instruction =
        "Synthetic deterministic interference request.".to_string();
    let metadata = |stability: ArtifactStability, id: &str| ContextArtifact {
        schema_version: CONTEXT_ARTIFACT_SCHEMA_VERSION,
        artifact_id: id.to_string(),
        origin_id: format!("origin-{id}"),
        content_source_id: Observed::Known(format!("synthetic-source-{id}")),
        content_hash: Observed::Unknown,
        revision: Observed::Known("v1".to_string()),
        artifact_type: ArtifactType::SourceFile,
        stability,
        lifecycle: ArtifactLifecycle::PersistentVersioned,
        sizes: ArtifactSizes {
            byte_size: Observed::Known(12),
            ..Default::default()
        },
        cache: Default::default(),
        trust: Observed::Known(TrustLevel::Trusted),
        provenance: BTreeMap::from([(
            "source".to_string(),
            EvidenceProvenance {
                origin: EvidenceOrigin::SourceExplicit,
                ..EvidenceProvenance::default()
            },
        )]),
        metadata: BTreeMap::new(),
    };
    let inputs = ContextStabilityInputs {
        artifacts: BTreeMap::from([
            (
                "stable-a".to_string(),
                metadata(ArtifactStability::Stable, "stable-a"),
            ),
            (
                "volatile-v".to_string(),
                metadata(ArtifactStability::Volatile, "volatile-v"),
            ),
            (
                "stable-b".to_string(),
                metadata(ArtifactStability::Stable, "stable-b"),
            ),
        ]),
        provenance: BTreeMap::from([(
            "classification".to_string(),
            "explicit-metadata".to_string(),
        )]),
        ..Default::default()
    };
    let constraints = LayoutPlanningConstraints {
        constraints: vec![
            OrderingConstraint::MovableWithinCompatibleRegion {
                segment: "context.artifacts[stable-b]".to_string(),
                region: "synthetic-independent-artifacts".to_string(),
            },
            OrderingConstraint::MovableWithinCompatibleRegion {
                segment: "context.artifacts[volatile-v]".to_string(),
                region: "synthetic-independent-artifacts".to_string(),
            },
        ],
        provenance: BTreeMap::from([(
            "permission".to_string(),
            "stable-b-explicitly-movable".to_string(),
        )]),
    };
    let workload = PairedWorkloadSummary {
        target_bytes: TARGET_WORKLOAD_BYTES,
        control_initial_bytes: serde_json::to_vec(&control_initial).unwrap().len(),
        control_mutated_bytes: serde_json::to_vec(&control_mutated).unwrap().len(),
        treatment_initial_bytes: 0,
        treatment_mutated_bytes: 0,
        context_limit: 8192,
    };
    Ok(PairedMutationSeed {
        control_initial,
        control_mutated,
        interference,
        stability_inputs: inputs,
        constraints,
        workload,
    })
}

pub fn prepare_paired_mutation_experiment(
    seed: PairedMutationSeed,
    runtime_profile: RuntimeProfileReference,
    fresh_server_for_run: bool,
    provenance: BTreeMap<String, String>,
) -> Result<PairedMutationDefinition, BenchmarkError> {
    seed.control_initial.validate()?;
    seed.control_mutated.validate()?;
    seed.stability_inputs.validate()?;
    let control_analysis =
        analyze_context_stability(&seed.control_initial, &seed.stability_inputs)?;
    let mutated_analysis =
        analyze_context_stability(&seed.control_mutated, &seed.stability_inputs)?;
    let plan_a0 = plan_request_layout(
        &seed.control_initial,
        &seed.stability_inputs,
        &seed.constraints,
    )?;
    let plan_a1 = plan_request_layout(
        &seed.control_mutated,
        &seed.stability_inputs,
        &seed.constraints,
    )?;
    let candidate_a0 = plan_a0
        .candidates
        .first()
        .ok_or_else(|| paired_error("P0-L11 produced no safe A0 candidate"))?
        .clone();
    let candidate_a1 = plan_a1
        .candidates
        .first()
        .ok_or_else(|| paired_error("P0-L11 produced no safe A1 candidate"))?
        .clone();
    let eval_a0 = evaluate_candidate(CandidateEvaluationInput {
        candidate: &candidate_a0,
        capability_profile: None,
        observations: &[],
        environment: &crate::candidate_evaluation::EnvironmentState::available(),
    })?;
    let eval_a1 = evaluate_candidate(CandidateEvaluationInput {
        candidate: &candidate_a1,
        capability_profile: None,
        observations: &[],
        environment: &crate::candidate_evaluation::EnvironmentState::available(),
    })?;
    let c0 = materialize_candidate(
        &seed.control_initial,
        &plan_a0,
        &candidate_a0,
        &eval_a0,
        &seed.stability_inputs,
        BTreeMap::from([("case".to_string(), "C0".to_string())]),
    )?;
    let c1 = materialize_candidate(
        &seed.control_mutated,
        &plan_a1,
        &candidate_a1,
        &eval_a1,
        &seed.stability_inputs,
        BTreeMap::from([("case".to_string(), "C1".to_string())]),
    )?;
    let control_mutation = comparison(
        "A0-to-A1",
        PairedComparisonKind::ControlMutation,
        &seed.control_initial,
        &seed.control_mutated,
        false,
    )?;
    validate_volatile_mutation(&seed.control_initial, &seed.control_mutated)?;
    let treatment_mutation = comparison(
        "C0-to-C1",
        PairedComparisonKind::TreatmentMutation,
        &c0.materialized_request,
        &c1.materialized_request,
        false,
    )?;
    validate_volatile_mutation(&c0.materialized_request, &c1.materialized_request)?;
    let control_layout = comparison(
        "A0-to-C0",
        PairedComparisonKind::ControlLayout,
        &seed.control_initial,
        &c0.materialized_request,
        true,
    )?;
    let treatment_layout = comparison(
        "A1-to-C1",
        PairedComparisonKind::TreatmentLayout,
        &seed.control_mutated,
        &c1.materialized_request,
        true,
    )?;
    let primary_comparison = PairedMutationComparison {
        comparison_id: "A1-vs-C1".to_string(),
        kind: PairedComparisonKind::PrimaryMutationOutcome,
        left_request_fingerprint: seed.control_mutated.request_fingerprint()?,
        right_request_fingerprint: c1.materialized_request_fingerprint.clone(),
        request_diff: request_diff(&seed.control_mutated, &c1.materialized_request)?,
        interpretation:
            "Equal logical information with deliberately different layout; not causal proof."
                .to_string(),
        cache_outcome: "not_observed; no required direction".to_string(),
    };
    let mut interference = seed.interference.clone();
    interference.context.system_instruction = format!(
        "[paired-interference:{}] {}",
        &seed.control_initial.request_fingerprint()?[..16],
        interference.context.system_instruction
    );
    let sequence = vec![
        paired_step(
            "A0",
            PairedMutationSequenceRole::Control,
            &seed.control_initial,
            PairedMutationSequenceRelation::Initial,
        )?,
        paired_step(
            "A1",
            PairedMutationSequenceRole::Control,
            &seed.control_mutated,
            PairedMutationSequenceRelation::VolatileMutationOf("A0".to_string()),
        )?,
        paired_step(
            "B1",
            PairedMutationSequenceRole::Interference,
            &interference,
            PairedMutationSequenceRelation::Independent,
        )?,
        paired_step(
            "C0",
            PairedMutationSequenceRole::CandidateTreatment,
            &c0.materialized_request,
            PairedMutationSequenceRelation::Initial,
        )?,
        paired_step(
            "C1",
            PairedMutationSequenceRole::CandidateTreatment,
            &c1.materialized_request,
            PairedMutationSequenceRelation::VolatileMutationOf("C0".to_string()),
        )?,
    ];
    let mut workload = seed.workload;
    workload.treatment_initial_bytes = serde_json::to_vec(&c0.materialized_request).unwrap().len();
    workload.treatment_mutated_bytes = serde_json::to_vec(&c1.materialized_request).unwrap().len();
    let treatment_initial_analysis =
        analyze_context_stability(&c0.materialized_request, &seed.stability_inputs)?;
    let treatment_mutated_analysis =
        analyze_context_stability(&c1.materialized_request, &seed.stability_inputs)?;
    let experiment_id = canonical_hash(&json!({"schema_id": PAIRED_MUTATION_SCHEMA_ID, "control_mutation": control_mutation, "treatment_mutation": treatment_mutation, "control_layout": control_layout, "treatment_layout": treatment_layout, "sequence": sequence, "runtime_profile": runtime_profile})) .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let definition = PairedMutationDefinition {
        schema_id: PAIRED_MUTATION_SCHEMA_ID.to_string(),
        schema_version: PAIRED_MUTATION_SCHEMA_VERSION,
        experiment_id,
        control_initial: seed.control_initial,
        control_mutated: seed.control_mutated,
        interference,
        treatment_initial: c0,
        treatment_mutated: c1,
        control_mutation,
        treatment_mutation,
        control_layout,
        treatment_layout,
        primary_comparison,
        sequence,
        workload,
        control_initial_inversion_count: inversion_count(&control_analysis),
        control_mutated_inversion_count: inversion_count(&mutated_analysis),
        treatment_initial_inversion_count: inversion_count(&treatment_initial_analysis),
        treatment_mutated_inversion_count: inversion_count(&treatment_mutated_analysis),
        control_initial_leading_segments: control_analysis.leading_region.segment_count,
        treatment_initial_leading_segments: treatment_initial_analysis.leading_region.segment_count,
        expected_outcome: PairedOutcomeExpectation::NoRequiredDirection,
        fresh_server_for_run,
        runtime_profile,
        provenance,
    };
    definition.validate()?;
    Ok(definition)
}

pub fn preflight_paired_mutation_experiment(
    definition: &PairedMutationDefinition,
    config: &LlamaCppLiveConfig,
) -> Result<PairedReadinessRecord, BenchmarkError> {
    definition.validate()?;
    let _experiment = build_paired_mutation_conformance_experiment(definition)?;
    if !config.fresh_server_for_run {
        return Err(BenchmarkError::live_harness(
            LivePreparationErrorCode::FreshServerAssertionRequired,
            "caller must assert fresh_server_for_run=true",
        ));
    }
    config.validate()?;
    if config.context_size != 8192 || config.parallel_slots != 1 || !config.metrics_enabled {
        return Err(paired_error(
            "P0-L6B runtime compatibility requires context 8192, one slot, and metrics enabled",
        ));
    }
    for request in [
        &definition.control_initial,
        &definition.control_mutated,
        &definition.interference,
        &definition.treatment_initial.materialized_request,
        &definition.treatment_mutated.materialized_request,
    ] {
        let projected = project_llama_cpp_request(request)?;
        let bytes = serde_json::to_vec(&projected)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
        if bytes.len() > config.max_context_bytes {
            return Err(BenchmarkError::live_harness(
                LivePreparationErrorCode::ContextLimitRejected,
                "paired projected request exceeds configured bound",
            ));
        }
    }
    let mut config_value = serde_json::to_value(config)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    if let serde_json::Value::Object(object) = &mut config_value {
        object.remove("execute_live");
        object.remove("provenance");
    }
    let runtime_config_fingerprint = canonical_hash(&json!({
        "definition": definition.experiment_id,
        "config": config_value,
    }))
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(PairedReadinessRecord {
        schema_id: PAIRED_MUTATION_SCHEMA_ID.to_string(),
        schema_version: PAIRED_MUTATION_SCHEMA_VERSION,
        state: LiveEvidenceState::Prepared,
        network_calls: 0,
        experiment_id: definition.experiment_id.clone(),
        sequence_step_ids: definition
            .sequence
            .iter()
            .map(|step| step.step_id.clone())
            .collect(),
        fresh_server_for_run: true,
        runtime_config_fingerprint,
        comparisons: vec![
            "A0-to-A1".to_string(),
            "C0-to-C1".to_string(),
            "A1-vs-C1".to_string(),
        ],
        primary_outcome: "cache/prefill accounting; no required direction".to_string(),
        provenance: BTreeMap::from([
            ("network".to_string(), "not_contacted".to_string()),
            ("evidence".to_string(), "prepared_only".to_string()),
        ]),
    })
}

pub fn live_paired_mutation_identity(
    definition: &PairedMutationDefinition,
    config: &LlamaCppLiveConfig,
) -> Result<String, BenchmarkError> {
    preflight_paired_mutation_experiment(definition, config)?;
    canonical_hash(&json!({"definition": definition, "config": config}))
        .map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn execute_paired_mutation_experiment<T: LlamaCppTransport + LiveRawEvidenceSource>(
    definition: &PairedMutationDefinition,
    config: &LlamaCppLiveConfig,
    transport: &mut T,
) -> Result<PairedMutationRunRecord, BenchmarkError> {
    let _ = preflight_paired_mutation_experiment(definition, config)?;
    if !config.execute_live {
        return Err(BenchmarkError::live_harness(
            LivePreparationErrorCode::LiveOptInRequired,
            "paired live execution requires execute_live=true",
        ));
    }
    let experiment = build_paired_mutation_conformance_experiment(definition)?;
    let observed_at = config
        .provenance
        .get("observed_at")
        .cloned()
        .unwrap_or_else(|| "caller-supplied-observation-time-required".to_string());
    let mut runner = LlamaCppConformanceRunner::new_live(
        transport,
        observed_at,
        definition.runtime_profile.identity.clone(),
    );
    match experiment.run(&mut runner) {
        Ok(result) => {
            let record = PairedMutationRunRecord {
                schema_id: PAIRED_MUTATION_SCHEMA_ID.to_string(),
                schema_version: PAIRED_MUTATION_SCHEMA_VERSION,
                experiment_id: definition.experiment_id.clone(),
                state: LiveEvidenceState::Normalized,
                expected_steps: PAIRED_SEQUENCE_LENGTH,
                completed_steps: PAIRED_SEQUENCE_LENGTH,
                raw_evidence: runner.transport().raw_evidence(),
                normalized_result: Some(result),
                failure: None,
                provenance: BTreeMap::from([(
                    "flow".to_string(),
                    "p0-l5-to-normalized-result".to_string(),
                )]),
            };
            record.validate()?;
            Ok(record)
        }
        Err(error) => {
            let raw = runner.transport().raw_evidence();
            let record = PairedMutationRunRecord {
                schema_id: PAIRED_MUTATION_SCHEMA_ID.to_string(),
                schema_version: PAIRED_MUTATION_SCHEMA_VERSION,
                experiment_id: definition.experiment_id.clone(),
                state: if raw.is_empty() {
                    LiveEvidenceState::Failed
                } else {
                    LiveEvidenceState::Partial
                },
                expected_steps: PAIRED_SEQUENCE_LENGTH,
                completed_steps: raw.len().min(PAIRED_SEQUENCE_LENGTH),
                raw_evidence: raw,
                normalized_result: None,
                failure: Some(error.to_string()),
                provenance: BTreeMap::from([(
                    "flow".to_string(),
                    "aborted-on-first-failure".to_string(),
                )]),
            };
            record.validate()?;
            Ok(record)
        }
    }
}

/// Construct and validate the one generic conformance experiment used by both
/// paired preflight and execution. A0 is the sole generic baseline; C0 is a
/// certified artifact-order mutation derived from A0 rather than a second
/// baseline.
pub fn build_paired_mutation_conformance_experiment(
    definition: &PairedMutationDefinition,
) -> Result<ConformanceExperiment, BenchmarkError> {
    definition.validate()?;
    let cases = definition
        .sequence
        .iter()
        .map(|step| ConformanceCase {
            case_id: step.step_id.clone(),
            mutation: match step.step_id.as_str() {
                "A0" => MutationClass::Baseline,
                "A1" | "C1" => MutationClass::VolatileArtifactContent,
                "B1" => MutationClass::CurrentContentEnd,
                "C0" => MutationClass::ArtifactOrder,
                _ => unreachable!(),
            },
            request: match step.step_id.as_str() {
                "A0" => definition.control_initial.clone(),
                "A1" => definition.control_mutated.clone(),
                "B1" => definition.interference.clone(),
                "C0" => definition.treatment_initial.materialized_request.clone(),
                "C1" => definition.treatment_mutated.materialized_request.clone(),
                _ => unreachable!(),
            },
            relationship: match step.step_id.as_str() {
                "A0" => CaseRelationship::Baseline,
                "A1" => CaseRelationship::MutationOf("A0".to_string()),
                "B1" => CaseRelationship::MutationOf("A0".to_string()),
                "C0" => CaseRelationship::MutationOf("A0".to_string()),
                "C1" => CaseRelationship::MutationOf("C0".to_string()),
                _ => unreachable!(),
            },
            expected_observation: ExpectedObservationMetadata {
                cache_reuse: ExpectedObservationState::ToBeObserved,
                cache_write: ExpectedObservationState::ToBeObserved,
                notes:
                    "No expected direction; record cache/prefill accounting and descriptive timing."
                        .to_string(),
            },
        })
        .collect::<Vec<_>>();
    let experiment = ConformanceExperiment {
        schema_id: crate::conformance::CONFORMANCE_SCHEMA_ID.to_string(),
        schema_version: crate::conformance::CONFORMANCE_SCHEMA_VERSION,
        experiment_id: definition.experiment_id.clone(),
        baseline_request: definition.control_initial.clone(),
        cases,
        runtime_profile: definition.runtime_profile.clone(),
        metadata: BTreeMap::from([
            (
                "experiment_type".to_string(),
                "p0-l6b-paired-mutation".to_string(),
            ),
            ("baseline_case".to_string(), "A0".to_string()),
        ]),
    };
    experiment.validate()?;
    Ok(experiment)
}

fn comparison(
    id: &str,
    kind: PairedComparisonKind,
    left: &ConformanceRequest,
    right: &ConformanceRequest,
    reorder_only: bool,
) -> Result<PairedMutationComparison, BenchmarkError> {
    let diff = request_diff(left, right)?;
    validate_request_diff(&diff, reorder_only)?;
    Ok(PairedMutationComparison {
        comparison_id: id.to_string(),
        kind,
        left_request_fingerprint: diff.left_request_fingerprint.clone(),
        right_request_fingerprint: diff.right_request_fingerprint.clone(),
        request_diff: diff,
        interpretation: if reorder_only {
            "Authorized artifact reordering only; no causal interpretation."
        } else {
            "Exact designated volatile mutation only; no expected direction."
        }
        .to_string(),
        cache_outcome: "not_observed".to_string(),
    })
}

fn paired_step(
    id: &str,
    role: PairedMutationSequenceRole,
    request: &ConformanceRequest,
    relation: PairedMutationSequenceRelation,
) -> Result<PairedMutationSequenceStep, BenchmarkError> {
    Ok(PairedMutationSequenceStep {
        step_id: id.to_string(),
        role,
        request_fingerprint: request.request_fingerprint()?,
        relation,
    })
}

fn validate_request_diff(diff: &RequestDiff, reorder_only: bool) -> Result<(), BenchmarkError> {
    if !diff.envelope_diff.changes.is_empty() {
        return Err(paired_error("paired request diff changed the envelope"));
    }
    if reorder_only {
        if diff.prefix_diff.changes.len() != 1
            || diff.prefix_diff.changes[0].category != crate::ChangeCategory::ArtifactOrderChanged
            || !diff.prefix_diff.changes[0].order_changed
            || diff.prefix_diff.changes[0].content_changed
        {
            return Err(paired_error(
                "paired layout diff is not authorized artifact reordering only",
            ));
        }
    } else if diff.prefix_diff.changes.len() != 1
        || diff.prefix_diff.changes[0].category != crate::ChangeCategory::ArtifactContentChanged
        || diff.prefix_diff.changes[0].order_changed
        || diff.prefix_diff.changes[0].presence_changed
    {
        return Err(paired_error(
            "paired mutation diff must contain exactly one artifact content change",
        ));
    }
    Ok(())
}

fn validate_volatile_mutation(
    left: &ConformanceRequest,
    right: &ConformanceRequest,
) -> Result<(), BenchmarkError> {
    if left.context.system_instruction != right.context.system_instruction
        || left.context.user_content != right.context.user_content
        || left.context.tools != right.context.tools
        || left.envelope != right.envelope
    {
        return Err(paired_error(
            "volatile mutation changed a non-volatile request dimension",
        ));
    }
    let left_by_id = left
        .context
        .artifacts
        .iter()
        .map(|artifact| (artifact.artifact_id.as_str(), artifact.content.as_str()))
        .collect::<BTreeMap<_, _>>();
    let right_by_id = right
        .context
        .artifacts
        .iter()
        .map(|artifact| (artifact.artifact_id.as_str(), artifact.content.as_str()))
        .collect::<BTreeMap<_, _>>();
    if left_by_id.len() != right_by_id.len() || left_by_id.keys().ne(right_by_id.keys()) {
        return Err(paired_error(
            "volatile mutation changed artifact membership",
        ));
    }
    let changed = left_by_id
        .iter()
        .filter(|(id, content)| right_by_id.get(*id) != Some(*content))
        .count();
    if changed != 1 || left_by_id.get("volatile-v") == right_by_id.get("volatile-v") {
        return Err(paired_error(
            "volatile mutation must change only the designated volatile artifact",
        ));
    }
    Ok(())
}

fn bounded_material(id: &str, value: &str) -> String {
    format!("{id}:{value}:{}", "bounded synthetic reference ".repeat(8))
}

fn inversion_count(analysis: &crate::context_stability::ContextStabilityAnalysis) -> usize {
    analysis
        .findings
        .iter()
        .filter(|finding| {
            finding.kind == crate::context_stability::StabilityFindingKind::StabilityInversion
        })
        .count()
}

fn paired_error(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::live_harness(LivePreparationErrorCode::InvalidConfiguration, message)
}
fn validate_metadata(values: &BTreeMap<String, String>) -> Result<(), BenchmarkError> {
    if values.len() > 32 {
        return Err(paired_error("paired provenance exceeds its bound"));
    }
    Ok(())
}

use crate::conformance::ContextArtifactInput;
use serde_json::json;
