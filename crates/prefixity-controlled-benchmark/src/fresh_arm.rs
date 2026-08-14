//! P0-L6E independent fresh-server-arm experiment preparation.
//!
//! This module is intentionally separate from the P0-L6B five-case runner.
//! It prepares two independently executable two-case arms and never starts,
//! stops, probes, or contacts a runtime by itself.  The caller must finalize
//! the first arm before requesting the operator-controlled restart and the
//! second arm.

use crate::candidate_evaluation::{
    evaluate_candidate, CandidateEvaluation, CandidateEvaluationInput, EnvironmentState,
};
use crate::capability_registry::CapabilityProfile;
use crate::conformance::{
    CaseRelationship, ConformanceCase, ConformanceExperiment, ConformanceRequest,
    ConformanceResult, ExpectedObservationMetadata, ExpectedObservationState, MutationClass,
    RuntimeProfileReference,
};
use crate::diff::RequestDiff;
use crate::error::{BenchmarkError, LivePreparationErrorCode};
use crate::hashing::{canonical_hash, canonical_json};
use crate::layout_planner::LayoutCandidate;
use crate::live_harness::{
    LiveEvidenceState, LiveFailure, LiveRawEvidenceSource, LlamaCppLiveConfig, RawLlamaCppEvidence,
};
use crate::llama_cpp::{
    project_llama_cpp_request_with_generation_limit, validate_llama_cpp_generation_limit,
    LlamaCppConformanceRunner, LlamaCppTransport,
};
use crate::materialization::{build_candidate_experiment_pair, CandidateExperimentPair};
use crate::observation_diagnostics::{
    diagnose_conformance_cache_with_source, CacheDiagnostic, EvidenceSourceClass,
};
use crate::paired_mutation::PairedMutationDefinition;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const FRESH_ARM_SCHEMA_ID: &str = "prefixity.llama-cpp-fresh-arm-paired-mutation";
pub const FRESH_ARM_SCHEMA_VERSION: u32 = 1;
pub const FRESH_ARM_AGGREGATION_SCHEMA_ID: &str = "prefixity.llama-cpp-fresh-arm-aggregation";
pub const FRESH_ARM_AGGREGATION_SCHEMA_VERSION: u32 = 1;
const FRESH_ARM_STEP_COUNT: usize = 2;
const MAX_FRESH_ARM_PROVENANCE: usize = 32;
const MAX_EPOCH_ID_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreshArmKind {
    Control,
    Treatment,
}

impl FreshArmKind {
    fn case_ids(self) -> [&'static str; FRESH_ARM_STEP_COUNT] {
        match self {
            Self::Control => ["A0", "A1"],
            Self::Treatment => ["C0", "C1"],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmStep {
    pub case_id: String,
    pub request: ConformanceRequest,
    pub request_fingerprint: String,
    pub relationship: CaseRelationship,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmDefinition {
    pub schema_id: String,
    pub schema_version: u32,
    pub arm_id: FreshArmKind,
    pub epoch_id: String,
    pub semantic_experiment_id: String,
    pub parent_experiment_id: String,
    pub runtime_config_fingerprint: String,
    pub runtime_profile: RuntimeProfileReference,
    pub fresh_server_for_arm: bool,
    pub steps: Vec<FreshArmStep>,
    pub mutation_request_diff: RequestDiff,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl FreshArmDefinition {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != FRESH_ARM_SCHEMA_ID || self.schema_version != FRESH_ARM_SCHEMA_VERSION
        {
            return Err(fresh_validation("unsupported fresh-arm schema"));
        }
        validate_epoch_id(&self.epoch_id)?;
        validate_hash(
            &self.semantic_experiment_id,
            "fresh-arm semantic experiment ID",
        )?;
        validate_hash(&self.parent_experiment_id, "parent experiment ID")?;
        validate_hash(
            &self.runtime_config_fingerprint,
            "fresh-arm runtime configuration fingerprint",
        )?;
        if !self.fresh_server_for_arm {
            return Err(BenchmarkError::live_harness(
                LivePreparationErrorCode::FreshServerAssertionRequired,
                "each fresh arm requires fresh_server_for_arm=true",
            ));
        }
        if self.steps.len() != FRESH_ARM_STEP_COUNT {
            return Err(fresh_validation("fresh arm must contain exactly two steps"));
        }
        let expected_ids = self.arm_id.case_ids();
        for (step, expected_id) in self.steps.iter().zip(expected_ids) {
            if step.case_id != expected_id
                || step.request_fingerprint != step.request.request_fingerprint()?
            {
                return Err(fresh_validation(
                    "fresh arm request identity is not traceable",
                ));
            }
        }
        let first = &self.steps[0];
        let second = &self.steps[1];
        if first.relationship != CaseRelationship::Baseline
            || second.relationship != CaseRelationship::MutationOf(first.case_id.clone())
            || self.mutation_request_diff.left_request_fingerprint != first.request_fingerprint
            || self.mutation_request_diff.right_request_fingerprint != second.request_fingerprint
        {
            return Err(fresh_validation(
                "fresh arm must represent its exact two-case mutation",
            ));
        }
        validate_provenance(&self.provenance)
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| fresh_validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmExperimentDefinition {
    pub schema_id: String,
    pub schema_version: u32,
    pub semantic_experiment_id: String,
    pub parent_experiment_id: String,
    pub runtime_config_fingerprint: String,
    pub runtime_profile: RuntimeProfileReference,
    pub control: FreshArmDefinition,
    pub treatment: FreshArmDefinition,
    pub candidate: LayoutCandidate,
    pub candidate_pair: CandidateExperimentPair,
    pub no_interference_case: bool,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl FreshArmExperimentDefinition {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != FRESH_ARM_SCHEMA_ID || self.schema_version != FRESH_ARM_SCHEMA_VERSION
        {
            return Err(fresh_validation("unsupported fresh-arm experiment schema"));
        }
        validate_hash(&self.semantic_experiment_id, "fresh-arm semantic ID")?;
        validate_hash(&self.parent_experiment_id, "parent experiment ID")?;
        validate_hash(
            &self.runtime_config_fingerprint,
            "fresh-arm configuration fingerprint",
        )?;
        self.control.validate()?;
        self.treatment.validate()?;
        if self.control.arm_id != FreshArmKind::Control
            || self.treatment.arm_id != FreshArmKind::Treatment
            || self.control.semantic_experiment_id != self.semantic_experiment_id
            || self.treatment.semantic_experiment_id != self.semantic_experiment_id
            || self.control.parent_experiment_id != self.parent_experiment_id
            || self.treatment.parent_experiment_id != self.parent_experiment_id
            || self.control.runtime_config_fingerprint != self.runtime_config_fingerprint
            || self.treatment.runtime_config_fingerprint != self.runtime_config_fingerprint
            || self.control.runtime_profile != self.runtime_profile
            || self.treatment.runtime_profile != self.runtime_profile
            || self.control.epoch_id == self.treatment.epoch_id
            || !self.no_interference_case
        {
            return Err(fresh_validation(
                "fresh-arm control and treatment boundaries are not distinct",
            ));
        }
        self.candidate_pair.validate()?;
        if self.candidate_pair.source_request_fingerprint
            != self.candidate.request_diff.left_request_fingerprint
            || self.candidate_pair.candidate_request_fingerprint
                != self.candidate.request_diff.right_request_fingerprint
            || self.candidate_pair.candidate_fingerprint != self.candidate.layout_fingerprint
        {
            return Err(fresh_validation(
                "fresh-arm candidate or materialization identity does not match",
            ));
        }
        validate_provenance(&self.provenance)
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| fresh_validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmReadinessRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub semantic_experiment_id: String,
    pub runtime_config_fingerprint: String,
    pub network_calls: u32,
    pub generation_limit: u32,
    pub arms: Vec<FreshArmReadiness>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl FreshArmReadinessRecord {
    pub fn validate(
        &self,
        experiment: &FreshArmExperimentDefinition,
    ) -> Result<(), BenchmarkError> {
        experiment.validate()?;
        if self.schema_id != FRESH_ARM_SCHEMA_ID
            || self.schema_version != FRESH_ARM_SCHEMA_VERSION
            || self.semantic_experiment_id != experiment.semantic_experiment_id
            || self.runtime_config_fingerprint != experiment.runtime_config_fingerprint
            || self.network_calls != 0
            || self.generation_limit != 1
            || self.arms
                != vec![
                    readiness(&experiment.control),
                    readiness(&experiment.treatment),
                ]
        {
            return Err(fresh_validation(
                "fresh-arm readiness does not match the prepared design",
            ));
        }
        validate_provenance(&self.provenance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmReadiness {
    pub arm_id: FreshArmKind,
    pub epoch_id: String,
    pub fresh_server_for_arm: bool,
    pub step_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmRunRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub arm_id: FreshArmKind,
    pub epoch_id: String,
    pub semantic_experiment_id: String,
    pub parent_experiment_id: String,
    pub runtime_config_fingerprint: String,
    pub runtime_profile: RuntimeProfileReference,
    pub fresh_server_for_arm: bool,
    pub state: LiveEvidenceState,
    pub expected_steps: usize,
    pub completed_steps: usize,
    pub transport_attempts: u32,
    pub complete_http_responses: u32,
    pub normalized_cases: u32,
    pub request_fingerprints: Vec<String>,
    pub mutation_request_diff: RequestDiff,
    pub raw_evidence: Vec<RawLlamaCppEvidence>,
    pub normalized_result: Option<ConformanceResult>,
    pub failure: Option<LiveFailure>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl FreshArmRunRecord {
    pub fn validate_against(&self, arm: &FreshArmDefinition) -> Result<(), BenchmarkError> {
        arm.validate()?;
        if self.schema_id != FRESH_ARM_SCHEMA_ID
            || self.schema_version != FRESH_ARM_SCHEMA_VERSION
            || self.arm_id != arm.arm_id
            || self.epoch_id != arm.epoch_id
            || self.semantic_experiment_id != arm.semantic_experiment_id
            || self.parent_experiment_id != arm.parent_experiment_id
            || self.runtime_config_fingerprint != arm.runtime_config_fingerprint
            || self.runtime_profile != arm.runtime_profile
            || !self.fresh_server_for_arm
            || self.expected_steps != FRESH_ARM_STEP_COUNT
            || self.completed_steps > self.expected_steps
            || self.request_fingerprints
                != arm
                    .steps
                    .iter()
                    .map(|step| step.request_fingerprint.clone())
                    .collect::<Vec<_>>()
            || self.mutation_request_diff != arm.mutation_request_diff
        {
            return Err(fresh_validation(
                "arm result identity or freshness assertion does not match its arm",
            ));
        }
        if self.transport_attempts < self.complete_http_responses
            || self.complete_http_responses < self.normalized_cases
            || self.raw_evidence.len() > self.expected_steps
        {
            return Err(fresh_validation("arm result accounting is inconsistent"));
        }
        match self.state {
            LiveEvidenceState::Normalized | LiveEvidenceState::Admitted => {
                let result = self.normalized_result.as_ref().ok_or_else(|| {
                    fresh_validation("complete arm result requires normalized evidence")
                })?;
                result.validate()?;
                if self.completed_steps != FRESH_ARM_STEP_COUNT
                    || self.normalized_cases != FRESH_ARM_STEP_COUNT as u32
                    || result.experiment_id != self.semantic_experiment_id
                    || result.runtime_profile != self.runtime_profile
                    || result.cases.len() != FRESH_ARM_STEP_COUNT
                    || result.cases.iter().zip(&arm.steps).any(|(case, step)| {
                        case.case_id != step.case_id
                            || case.request_fingerprint != step.request_fingerprint
                    })
                {
                    return Err(fresh_validation(
                        "normalized arm cases do not match the certified arm",
                    ));
                }
            }
            LiveEvidenceState::Partial | LiveEvidenceState::Failed => {
                if self.normalized_result.is_some() || self.completed_steps == self.expected_steps {
                    return Err(fresh_validation(
                        "partial arm result cannot contain a complete normalized result",
                    ));
                }
            }
            LiveEvidenceState::Prepared | LiveEvidenceState::Executed => {
                return Err(fresh_validation(
                    "arm run record must be finalized or explicitly partial",
                ));
            }
        }
        validate_provenance(&self.provenance)
    }

    pub fn canonical_json(&self, arm: &FreshArmDefinition) -> Result<Vec<u8>, BenchmarkError> {
        self.validate_against(arm)?;
        canonical_json(self).map_err(|error| fresh_validation(error.to_string()))
    }
}

/// Validate and serialize one completed or partial arm independently.  The
/// caller may persist these bytes before requesting the operator restart.
pub fn finalize_fresh_arm_record(
    record: &FreshArmRunRecord,
    arm: &FreshArmDefinition,
) -> Result<Vec<u8>, BenchmarkError> {
    record.canonical_json(arm)
}

/// Persist one finalized arm record without opening a network or runtime
/// boundary.  The second arm is not involved in this operation.
pub fn persist_fresh_arm_record(
    path: &Path,
    record: &FreshArmRunRecord,
    arm: &FreshArmDefinition,
) -> Result<(), BenchmarkError> {
    let bytes = finalize_fresh_arm_record(record, arm)?;
    fs::write(path, bytes).map_err(|source| BenchmarkError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshArmAggregationRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub semantic_experiment_id: String,
    pub runtime_config_fingerprint: String,
    pub control_epoch_id: String,
    pub treatment_epoch_id: String,
    pub control_mutation: CacheDiagnostic,
    pub treatment_mutation: CacheDiagnostic,
    pub candidate_comparison: CacheDiagnostic,
    pub candidate_evaluation: CandidateEvaluation,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl FreshArmAggregationRecord {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != FRESH_ARM_AGGREGATION_SCHEMA_ID
            || self.schema_version != FRESH_ARM_AGGREGATION_SCHEMA_VERSION
            || self.control_epoch_id == self.treatment_epoch_id
        {
            return Err(fresh_validation("invalid fresh-arm aggregation identity"));
        }
        validate_hash(&self.semantic_experiment_id, "aggregation semantic ID")?;
        validate_hash(
            &self.runtime_config_fingerprint,
            "aggregation configuration fingerprint",
        )?;
        self.candidate_evaluation.validate()?;
        validate_provenance(&self.provenance)
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| fresh_validation(error.to_string()))
    }
}

/// Prepare the two independent fresh-server arms.  This function only builds
/// and validates data; it never contacts a socket and never starts or stops a
/// runtime.
pub fn prepare_fresh_arm_experiment(
    paired: &PairedMutationDefinition,
    candidate: LayoutCandidate,
    config: &LlamaCppLiveConfig,
    control_epoch_id: impl Into<String>,
    treatment_epoch_id: impl Into<String>,
    provenance: BTreeMap<String, String>,
) -> Result<FreshArmExperimentDefinition, BenchmarkError> {
    paired.validate()?;
    validate_fresh_config(config)?;
    validate_provenance(&provenance)?;
    let control_epoch_id = control_epoch_id.into();
    let treatment_epoch_id = treatment_epoch_id.into();
    validate_epoch_id(&control_epoch_id)?;
    validate_epoch_id(&treatment_epoch_id)?;
    if control_epoch_id == treatment_epoch_id {
        return Err(fresh_validation(
            "control and treatment epoch IDs must differ",
        ));
    }
    if candidate.request_diff != paired.primary_comparison.request_diff
        || candidate.layout_fingerprint != paired.treatment_mutated.candidate_fingerprint
    {
        return Err(fresh_validation(
            "candidate does not match the existing A1-to-C1 materialization",
        ));
    }
    let candidate_pair = build_candidate_experiment_pair(
        &paired.treatment_mutated,
        "A1",
        "C1",
        BTreeMap::from([("flow".to_string(), "p0-l6e-fresh-arm".to_string())]),
    )?;
    let runtime_config_fingerprint = fresh_arm_config_fingerprint(config)?;
    let semantic_experiment_id = canonical_hash(&json!({
        "schema_id": FRESH_ARM_SCHEMA_ID,
        "schema_version": FRESH_ARM_SCHEMA_VERSION,
        "design": "two-independent-fresh-server-epochs-v1",
        "parent_experiment_id": paired.experiment_id,
        "control_mutation": paired.control_mutation.request_diff,
        "treatment_mutation": paired.treatment_mutation.request_diff,
        "primary_comparison": paired.primary_comparison.request_diff,
        "candidate_pair": candidate_pair.clone(),
        "runtime_profile": paired.runtime_profile,
        "runtime_config_fingerprint": runtime_config_fingerprint,
    }))
    .map_err(|error| fresh_validation(error.to_string()))?;
    let control = build_arm(
        FreshArmKind::Control,
        control_epoch_id,
        &semantic_experiment_id,
        paired,
        &runtime_config_fingerprint,
        &paired.control_mutation.request_diff,
    )?;
    let treatment = build_arm(
        FreshArmKind::Treatment,
        treatment_epoch_id,
        &semantic_experiment_id,
        paired,
        &runtime_config_fingerprint,
        &paired.treatment_mutation.request_diff,
    )?;
    let experiment = FreshArmExperimentDefinition {
        schema_id: FRESH_ARM_SCHEMA_ID.to_string(),
        schema_version: FRESH_ARM_SCHEMA_VERSION,
        semantic_experiment_id,
        parent_experiment_id: paired.experiment_id.clone(),
        runtime_config_fingerprint,
        runtime_profile: paired.runtime_profile.clone(),
        control,
        treatment,
        candidate,
        candidate_pair,
        no_interference_case: true,
        provenance,
    };
    experiment.validate()?;
    Ok(experiment)
}

pub fn fresh_arm_config_fingerprint(config: &LlamaCppLiveConfig) -> Result<String, BenchmarkError> {
    config.validate()?;
    let mut value =
        serde_json::to_value(config).map_err(|error| fresh_validation(error.to_string()))?;
    if let serde_json::Value::Object(object) = &mut value {
        object.remove("execute_live");
        object.remove("fresh_server_for_run");
        object.remove("provenance");
    }
    canonical_hash(&value).map_err(|error| fresh_validation(error.to_string()))
}

pub fn preflight_fresh_arm_experiment(
    experiment: &FreshArmExperimentDefinition,
    config: &LlamaCppLiveConfig,
) -> Result<FreshArmReadinessRecord, BenchmarkError> {
    experiment.validate()?;
    validate_fresh_config(config)?;
    if fresh_arm_config_fingerprint(config)? != experiment.runtime_config_fingerprint {
        return Err(fresh_validation(
            "runtime configuration does not match fresh-arm design",
        ));
    }
    for request in experiment
        .control
        .steps
        .iter()
        .chain(experiment.treatment.steps.iter())
        .map(|step| &step.request)
    {
        let projected = project_llama_cpp_request_with_generation_limit(request, 1)?;
        validate_llama_cpp_generation_limit(&projected, 1)?;
        let bytes =
            serde_json::to_vec(&projected).map_err(|error| fresh_validation(error.to_string()))?;
        if bytes.len() > config.max_context_bytes {
            return Err(BenchmarkError::live_harness(
                LivePreparationErrorCode::ContextLimitRejected,
                "fresh-arm projected request exceeds configured bound",
            ));
        }
    }
    let readiness_record = FreshArmReadinessRecord {
        schema_id: FRESH_ARM_SCHEMA_ID.to_string(),
        schema_version: FRESH_ARM_SCHEMA_VERSION,
        semantic_experiment_id: experiment.semantic_experiment_id.clone(),
        runtime_config_fingerprint: experiment.runtime_config_fingerprint.clone(),
        network_calls: 0,
        generation_limit: 1,
        arms: vec![
            readiness(&experiment.control),
            readiness(&experiment.treatment),
        ],
        provenance: BTreeMap::from([
            ("network".to_string(), "not_contacted".to_string()),
            ("evidence".to_string(), "prepared_only".to_string()),
        ]),
    };
    readiness_record.validate(experiment)?;
    Ok(readiness_record)
}

pub fn execute_fresh_arm<T>(
    experiment: &FreshArmExperimentDefinition,
    arm: FreshArmKind,
    config: &LlamaCppLiveConfig,
    transport: &mut T,
) -> Result<FreshArmRunRecord, BenchmarkError>
where
    T: LlamaCppTransport + LiveRawEvidenceSource,
{
    let _ = preflight_fresh_arm_experiment(experiment, config)?;
    if !config.execute_live {
        return Err(BenchmarkError::live_harness(
            LivePreparationErrorCode::LiveOptInRequired,
            "fresh-arm live execution requires execute_live=true",
        ));
    }
    let arm_definition = match arm {
        FreshArmKind::Control => &experiment.control,
        FreshArmKind::Treatment => &experiment.treatment,
    };
    let conformance = build_arm_conformance_experiment(arm_definition)?;
    let observed_at = config
        .provenance
        .get("observed_at")
        .cloned()
        .unwrap_or_else(|| "caller-supplied-observation-time-required".to_string());
    let mut runner = LlamaCppConformanceRunner::new_live_with_generation_limit(
        transport,
        observed_at,
        arm_definition.runtime_profile.identity.clone(),
        config.generation_limit,
    );
    match conformance.run(&mut runner) {
        Ok(normalized_result) => {
            let raw_evidence = runner.transport().raw_evidence();
            let record = FreshArmRunRecord {
                schema_id: FRESH_ARM_SCHEMA_ID.to_string(),
                schema_version: FRESH_ARM_SCHEMA_VERSION,
                arm_id: arm,
                epoch_id: arm_definition.epoch_id.clone(),
                semantic_experiment_id: experiment.semantic_experiment_id.clone(),
                parent_experiment_id: experiment.parent_experiment_id.clone(),
                runtime_config_fingerprint: experiment.runtime_config_fingerprint.clone(),
                runtime_profile: experiment.runtime_profile.clone(),
                fresh_server_for_arm: true,
                state: LiveEvidenceState::Normalized,
                expected_steps: FRESH_ARM_STEP_COUNT,
                completed_steps: FRESH_ARM_STEP_COUNT,
                transport_attempts: FRESH_ARM_STEP_COUNT as u32,
                complete_http_responses: raw_evidence.len() as u32,
                normalized_cases: normalized_result.cases.len() as u32,
                request_fingerprints: arm_definition
                    .steps
                    .iter()
                    .map(|step| step.request_fingerprint.clone())
                    .collect(),
                mutation_request_diff: arm_definition.mutation_request_diff.clone(),
                raw_evidence,
                normalized_result: Some(normalized_result),
                failure: None,
                provenance: BTreeMap::from([
                    ("flow".to_string(), "p0-l6e-arm-normalized".to_string()),
                    (
                        "evidence".to_string(),
                        "live-loopback-runtime-observation".to_string(),
                    ),
                ]),
            };
            record.validate_against(arm_definition)?;
            Ok(record)
        }
        Err(error) => {
            let raw_evidence = runner.transport().raw_evidence();
            let completed_steps = raw_evidence.len().min(FRESH_ARM_STEP_COUNT);
            let record = FreshArmRunRecord {
                schema_id: FRESH_ARM_SCHEMA_ID.to_string(),
                schema_version: FRESH_ARM_SCHEMA_VERSION,
                arm_id: arm,
                epoch_id: arm_definition.epoch_id.clone(),
                semantic_experiment_id: experiment.semantic_experiment_id.clone(),
                parent_experiment_id: experiment.parent_experiment_id.clone(),
                runtime_config_fingerprint: experiment.runtime_config_fingerprint.clone(),
                runtime_profile: experiment.runtime_profile.clone(),
                fresh_server_for_arm: true,
                state: if completed_steps == 0 {
                    LiveEvidenceState::Failed
                } else {
                    LiveEvidenceState::Partial
                },
                expected_steps: FRESH_ARM_STEP_COUNT,
                completed_steps,
                transport_attempts: completed_steps as u32 + 1,
                complete_http_responses: completed_steps as u32,
                normalized_cases: 0,
                request_fingerprints: arm_definition
                    .steps
                    .iter()
                    .map(|step| step.request_fingerprint.clone())
                    .collect(),
                mutation_request_diff: arm_definition.mutation_request_diff.clone(),
                raw_evidence,
                normalized_result: None,
                failure: Some(LiveFailure {
                    code: "fresh_arm_execution_failed".to_string(),
                    message: error.to_string(),
                }),
                provenance: BTreeMap::from([
                    ("flow".to_string(), "p0-l6e-arm-partial".to_string()),
                    (
                        "evidence".to_string(),
                        "live-loopback-runtime-observation".to_string(),
                    ),
                ]),
            };
            record.validate_against(arm_definition)?;
            Ok(record)
        }
    }
}

/// Combine two independently finalized arms offline.  No request is sent and
/// no runtime state is inspected.  Missing, duplicated, stale, or mismatched
/// arm identity fails closed before P0-L8/P0-L12 evaluation.
pub fn aggregate_fresh_arm_results(
    experiment: &FreshArmExperimentDefinition,
    control: Option<&FreshArmRunRecord>,
    treatment: Option<&FreshArmRunRecord>,
    capability_profile: Option<&CapabilityProfile>,
) -> Result<FreshArmAggregationRecord, BenchmarkError> {
    experiment.validate()?;
    let control = control.ok_or_else(|| fresh_validation("control arm evidence is missing"))?;
    let treatment =
        treatment.ok_or_else(|| fresh_validation("treatment arm evidence is missing"))?;
    control.validate_against(&experiment.control)?;
    treatment.validate_against(&experiment.treatment)?;
    if control.epoch_id == treatment.epoch_id
        || control.runtime_config_fingerprint != experiment.runtime_config_fingerprint
        || treatment.runtime_config_fingerprint != experiment.runtime_config_fingerprint
        || control.runtime_profile != experiment.runtime_profile
        || treatment.runtime_profile != experiment.runtime_profile
    {
        return Err(fresh_validation(
            "fresh-arm results do not share the certified design identity",
        ));
    }
    let control_result = control
        .normalized_result
        .as_ref()
        .ok_or_else(|| fresh_validation("control arm is not complete"))?;
    let treatment_result = treatment
        .normalized_result
        .as_ref()
        .ok_or_else(|| fresh_validation("treatment arm is not complete"))?;
    let profile_id = capability_profile.map(|profile| profile.profile_id.as_str());
    let control_diagnostic = diagnose_conformance_cache_with_source(
        &experiment.control.mutation_request_diff,
        &control_result.cases[0],
        &control_result.cases[1],
        profile_id,
        EvidenceSourceClass::ExperimentallyObservedRuntime,
    );
    let treatment_diagnostic = diagnose_conformance_cache_with_source(
        &experiment.treatment.mutation_request_diff,
        &treatment_result.cases[0],
        &treatment_result.cases[1],
        profile_id,
        EvidenceSourceClass::ExperimentallyObservedRuntime,
    );
    let candidate_diagnostic = diagnose_conformance_cache_with_source(
        &experiment.candidate.request_diff,
        &control_result.cases[1],
        &treatment_result.cases[1],
        profile_id,
        EvidenceSourceClass::ExperimentallyObservedRuntime,
    );
    let candidate_evaluation = evaluate_candidate(CandidateEvaluationInput {
        candidate: &experiment.candidate,
        capability_profile,
        observations: &[
            control_diagnostic.clone(),
            treatment_diagnostic.clone(),
            candidate_diagnostic.clone(),
        ],
        environment: &EnvironmentState::available(),
    })?;
    let record = FreshArmAggregationRecord {
        schema_id: FRESH_ARM_AGGREGATION_SCHEMA_ID.to_string(),
        schema_version: FRESH_ARM_AGGREGATION_SCHEMA_VERSION,
        semantic_experiment_id: experiment.semantic_experiment_id.clone(),
        runtime_config_fingerprint: experiment.runtime_config_fingerprint.clone(),
        control_epoch_id: control.epoch_id.clone(),
        treatment_epoch_id: treatment.epoch_id.clone(),
        control_mutation: control_diagnostic,
        treatment_mutation: treatment_diagnostic,
        candidate_comparison: candidate_diagnostic,
        candidate_evaluation,
        provenance: BTreeMap::from([
            ("flow".to_string(), "p0-l6e-offline-aggregation".to_string()),
            ("network".to_string(), "not_contacted".to_string()),
        ]),
    };
    record.validate()?;
    Ok(record)
}

fn build_arm(
    arm_id: FreshArmKind,
    epoch_id: String,
    semantic_experiment_id: &str,
    paired: &PairedMutationDefinition,
    runtime_config_fingerprint: &str,
    mutation_request_diff: &RequestDiff,
) -> Result<FreshArmDefinition, BenchmarkError> {
    let (requests, relationships) = match arm_id {
        FreshArmKind::Control => (
            [
                paired.control_initial.clone(),
                paired.control_mutated.clone(),
            ],
            [
                CaseRelationship::Baseline,
                CaseRelationship::MutationOf("A0".to_string()),
            ],
        ),
        FreshArmKind::Treatment => (
            [
                paired.treatment_initial.materialized_request.clone(),
                paired.treatment_mutated.materialized_request.clone(),
            ],
            [
                CaseRelationship::Baseline,
                CaseRelationship::MutationOf("C0".to_string()),
            ],
        ),
    };
    let case_ids = arm_id.case_ids();
    let steps = requests
        .into_iter()
        .zip(case_ids)
        .zip(relationships)
        .map(|((request, case_id), relationship)| {
            Ok(FreshArmStep {
                case_id: case_id.to_string(),
                request_fingerprint: request.request_fingerprint()?,
                request,
                relationship,
            })
        })
        .collect::<Result<Vec<_>, BenchmarkError>>()?;
    let arm = FreshArmDefinition {
        schema_id: FRESH_ARM_SCHEMA_ID.to_string(),
        schema_version: FRESH_ARM_SCHEMA_VERSION,
        arm_id,
        epoch_id,
        semantic_experiment_id: semantic_experiment_id.to_string(),
        parent_experiment_id: paired.experiment_id.clone(),
        runtime_config_fingerprint: runtime_config_fingerprint.to_string(),
        runtime_profile: paired.runtime_profile.clone(),
        fresh_server_for_arm: true,
        steps,
        mutation_request_diff: mutation_request_diff.clone(),
        provenance: BTreeMap::from([("flow".to_string(), "p0-l6e-prepared".to_string())]),
    };
    arm.validate()?;
    Ok(arm)
}

fn build_arm_conformance_experiment(
    arm: &FreshArmDefinition,
) -> Result<ConformanceExperiment, BenchmarkError> {
    arm.validate()?;
    let cases = arm
        .steps
        .iter()
        .map(|step| ConformanceCase {
            case_id: step.case_id.clone(),
            mutation: if step.relationship == CaseRelationship::Baseline {
                MutationClass::Baseline
            } else {
                MutationClass::VolatileArtifactContent
            },
            request: step.request.clone(),
            relationship: step.relationship.clone(),
            expected_observation: ExpectedObservationMetadata {
                cache_reuse: ExpectedObservationState::ToBeObserved,
                cache_write: ExpectedObservationState::ToBeObserved,
                notes: "No required direction; preserve bounded cache/prefill accounting."
                    .to_string(),
            },
        })
        .collect();
    let experiment = ConformanceExperiment {
        schema_id: crate::conformance::CONFORMANCE_SCHEMA_ID.to_string(),
        schema_version: crate::conformance::CONFORMANCE_SCHEMA_VERSION,
        experiment_id: arm.semantic_experiment_id.clone(),
        baseline_request: arm.steps[0].request.clone(),
        cases,
        runtime_profile: arm.runtime_profile.clone(),
        metadata: BTreeMap::from([
            (
                "experiment_type".to_string(),
                "p0-l6e-fresh-arm".to_string(),
            ),
            (
                "arm".to_string(),
                format!("{:?}", arm.arm_id).to_lowercase(),
            ),
            ("epoch_id".to_string(), arm.epoch_id.clone()),
        ]),
    };
    experiment.validate()?;
    Ok(experiment)
}

fn readiness(arm: &FreshArmDefinition) -> FreshArmReadiness {
    FreshArmReadiness {
        arm_id: arm.arm_id,
        epoch_id: arm.epoch_id.clone(),
        fresh_server_for_arm: arm.fresh_server_for_arm,
        step_ids: arm.steps.iter().map(|step| step.case_id.clone()).collect(),
    }
}

fn validate_fresh_config(config: &LlamaCppLiveConfig) -> Result<(), BenchmarkError> {
    config.validate()?;
    if !config.fresh_server_for_run {
        return Err(BenchmarkError::live_harness(
            LivePreparationErrorCode::FreshServerAssertionRequired,
            "fresh-arm preparation requires the caller fresh-server assertion",
        ));
    }
    if config.generation_limit != 1
        || config.context_size != 8192
        || config.parallel_slots != 1
        || !config.metrics_enabled
        || config.connect_timeout_ms != 1000
        || config.request_timeout_ms != 600000
    {
        return Err(BenchmarkError::live_harness(
            LivePreparationErrorCode::InvalidConfiguration,
            "fresh-arm configuration must retain the Attempt 006 bounded runtime contract",
        ));
    }
    Ok(())
}

fn validate_epoch_id(value: &str) -> Result<(), BenchmarkError> {
    validate_text(value, "epoch ID")?;
    if value.len() > MAX_EPOCH_ID_BYTES || value.contains(['/', '\\']) {
        return Err(fresh_validation("epoch ID is too long or path-like"));
    }
    Ok(())
}

fn validate_hash(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(fresh_validation(format!(
            "{field} must be a SHA-256 fingerprint"
        )));
    }
    Ok(())
}

fn validate_text(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > 256 {
        return Err(fresh_validation(format!(
            "{field} is empty or exceeds its bound"
        )));
    }
    Ok(())
}

fn validate_provenance(provenance: &BTreeMap<String, String>) -> Result<(), BenchmarkError> {
    if provenance.len() > MAX_FRESH_ARM_PROVENANCE {
        return Err(fresh_validation("fresh-arm provenance exceeds its bound"));
    }
    for (key, value) in provenance {
        validate_text(key, "provenance key")?;
        validate_text(value, "provenance value")?;
    }
    Ok(())
}

fn fresh_validation(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::validation(format!("P0-L6E fresh-arm: {}", message.into()))
}
