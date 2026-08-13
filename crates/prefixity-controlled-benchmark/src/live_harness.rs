//! P0-L6A preparation boundary for a future llama.cpp live experiment.
//!
//! This module is deliberately inert by default. It validates a certified
//! P0-L13 candidate, constructs a fixed experiment sequence, and provides a
//! dry-run record without opening a socket. A real request is possible only
//! through `execute_live_experiment` with an explicitly deserialized
//! `execute_live: true` configuration and a loopback-only HTTP endpoint.

use crate::conformance::{
    CaseRelationship, ConformanceCase, ConformanceExperiment, ConformanceRequest,
    ConformanceResult, ExpectedObservationMetadata, ExpectedObservationState, MutationClass,
    RuntimeProfileReference,
};
use crate::error::{BenchmarkError, LivePreparationErrorCode};
use crate::hashing::{canonical_hash, canonical_json, sha256_hex};
use crate::llama_cpp::{
    project_llama_cpp_request, LlamaCppConformanceRunner, LlamaCppRequest, LlamaCppResponse,
    LlamaCppTransport,
};
use crate::materialization::{
    build_candidate_experiment_pair, CandidateExperimentPair, MaterializedCandidate,
};
use prefixity_core::observation::Observed;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::Read;
use std::time::{Duration, Instant};

pub const LIVE_HARNESS_SCHEMA_ID: &str = "prefixity.llama-cpp-live-harness";
pub const LIVE_HARNESS_SCHEMA_VERSION: u32 = 1;
pub const LIVE_CONFIG_SCHEMA_ID: &str = "prefixity.llama-cpp-live-config";
pub const LIVE_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const ENVIRONMENT_MANIFEST_SCHEMA_ID: &str = "prefixity.live-environment-manifest";
pub const ENVIRONMENT_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const RAW_EVIDENCE_SCHEMA_ID: &str = "prefixity.llama-cpp-raw-evidence";
pub const RAW_EVIDENCE_SCHEMA_VERSION: u32 = 1;
pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_CONTEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 512;
const MAX_PROVENANCE: usize = 32;
const MAX_RAW_TELEMETRY_BYTES: usize = 16 * 1024;
const SEQUENCE_LENGTH: usize = 7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopbackEndpoint {
    pub url: String,
}

impl LoopbackEndpoint {
    pub fn parse(value: impl Into<String>) -> Result<Self, BenchmarkError> {
        let endpoint = Self { url: value.into() };
        endpoint.validate()?;
        Ok(endpoint)
    }

    pub fn validate(&self) -> Result<(), BenchmarkError> {
        let parsed = reqwest::Url::parse(&self.url).map_err(|error| {
            live_error(
                LivePreparationErrorCode::InvalidEndpoint,
                format!("endpoint URL is invalid: {error}"),
            )
        })?;
        if parsed.scheme() != "http" || parsed.username() != "" || parsed.password().is_some() {
            return Err(live_error(
                LivePreparationErrorCode::InvalidEndpoint,
                "only unauthenticated HTTP endpoints are supported",
            ));
        }
        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(live_error(
                LivePreparationErrorCode::InvalidEndpoint,
                "endpoint must not contain a query or fragment",
            ));
        }
        match parsed.host_str() {
            Some("127.0.0.1" | "::1" | "[::1]" | "localhost") => Ok(()),
            Some(_) => Err(live_error(
                LivePreparationErrorCode::NonLoopbackEndpoint,
                "endpoint host is not an approved loopback literal",
            )),
            None => Err(live_error(
                LivePreparationErrorCode::InvalidEndpoint,
                "endpoint must contain a host",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlamaCppLiveConfig {
    pub schema_id: String,
    pub schema_version: u32,
    pub endpoint: LoopbackEndpoint,
    pub llama_build: String,
    pub model_identity: String,
    pub quantization: Option<String>,
    pub context_size: u64,
    pub threads: Option<u32>,
    pub gpu_offload: Option<String>,
    pub kv_cache: Option<String>,
    pub batch_size: Option<u32>,
    pub generation_limit: u32,
    #[serde(default = "default_parallel_slots")]
    pub parallel_slots: u32,
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub seed: Option<u64>,
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub max_response_bytes: usize,
    pub max_context_bytes: usize,
    pub evidence_location: String,
    #[serde(default)]
    pub execute_live: bool,
    #[serde(default)]
    pub fresh_server_for_run: bool,
    pub runtime_profile: RuntimeProfileReference,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl LlamaCppLiveConfig {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != LIVE_CONFIG_SCHEMA_ID
            || self.schema_version != LIVE_CONFIG_SCHEMA_VERSION
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "unsupported live configuration schema",
            ));
        }
        self.endpoint.validate()?;
        for (field, value) in [
            ("llama_build", &self.llama_build),
            ("model_identity", &self.model_identity),
            ("evidence_location", &self.evidence_location),
        ] {
            bounded_text(field, value)?;
        }
        for (field, value) in [
            ("quantization", &self.quantization),
            ("gpu_offload", &self.gpu_offload),
            ("kv_cache", &self.kv_cache),
        ] {
            if let Some(value) = value {
                bounded_text(field, value)?;
            }
        }
        if self.context_size == 0
            || self.generation_limit == 0
            || self.connect_timeout_ms == 0
            || self.request_timeout_ms == 0
            || self.max_response_bytes == 0
            || self.max_response_bytes > MAX_RESPONSE_BYTES
            || self.max_context_bytes == 0
            || self.max_context_bytes > MAX_CONTEXT_BYTES
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "context, generation, timeout, and evidence bounds must be positive and bounded",
            ));
        }
        if self.threads == Some(0) || self.batch_size == Some(0) {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "threads and batch size must be positive when supplied",
            ));
        }
        if self.parallel_slots == 0 {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "parallel_slots must be positive",
            ));
        }
        for (field, value) in [("temperature", self.temperature), ("top_p", self.top_p)] {
            if let Some(value) = value {
                if !value.is_finite() || value < 0.0 {
                    return Err(live_error(
                        LivePreparationErrorCode::InvalidConfiguration,
                        format!("{field} must be finite and non-negative"),
                    ));
                }
            }
        }
        if self.evidence_location.starts_with('/')
            || self.evidence_location.contains(':')
            || self
                .evidence_location
                .split(['/', '\\'])
                .any(|part| part == "..")
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "evidence location must be a repository-relative path",
            ));
        }
        validate_provenance(&self.provenance)
    }
}

fn default_parallel_slots() -> u32 {
    1
}

fn default_metrics_enabled() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentObservation<T> {
    pub value: Observed<T>,
}

impl<T> EnvironmentObservation<T> {
    pub fn unknown() -> Self {
        Self {
            value: Observed::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveEnvironmentManifest {
    pub schema_id: String,
    pub schema_version: u32,
    pub os: EnvironmentObservation<String>,
    pub llama_build: EnvironmentObservation<String>,
    pub model_identity: EnvironmentObservation<String>,
    pub quantization: EnvironmentObservation<String>,
    pub model_file_size_bytes: EnvironmentObservation<u64>,
    pub context_size: EnvironmentObservation<u64>,
    pub cpu: EnvironmentObservation<String>,
    pub logical_threads: EnvironmentObservation<u32>,
    pub gpu: EnvironmentObservation<String>,
    pub ram_bytes: EnvironmentObservation<u64>,
    pub vram_bytes: EnvironmentObservation<u64>,
    pub launch_configuration: BTreeMap<String, String>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl LiveEnvironmentManifest {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != ENVIRONMENT_MANIFEST_SCHEMA_ID
            || self.schema_version != ENVIRONMENT_MANIFEST_SCHEMA_VERSION
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "unsupported environment manifest schema",
            ));
        }
        validate_map(&self.launch_configuration, "launch configuration", true)?;
        validate_provenance(&self.provenance)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveSequenceRole {
    Control,
    CandidateTreatment,
    Interference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "case_id")]
pub enum LiveSequenceRelation {
    Initial,
    ExactRepeatOf(String),
    ReturnTo(String),
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveSequenceStep {
    pub step_id: String,
    pub role: LiveSequenceRole,
    pub request_fingerprint: String,
    pub relation: LiveSequenceRelation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveExperimentDefinition {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub control_request: ConformanceRequest,
    pub treatment: MaterializedCandidate,
    pub interference_request: ConformanceRequest,
    pub pair: CandidateExperimentPair,
    pub sequence: Vec<LiveSequenceStep>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl LiveExperimentDefinition {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != LIVE_HARNESS_SCHEMA_ID
            || self.schema_version != LIVE_HARNESS_SCHEMA_VERSION
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                "unsupported live harness schema",
            ));
        }
        bounded_text("experiment_id", &self.experiment_id)?;
        self.control_request.validate()?;
        self.treatment.validate().map_err(|error| {
            live_error(
                LivePreparationErrorCode::UnsafeMaterializedCandidate,
                format!("materialized candidate is not certified: {error}"),
            )
        })?;
        self.interference_request.validate()?;
        self.pair.validate()?;
        if self.pair.source_request_fingerprint != self.control_request.request_fingerprint()?
            || self.pair.candidate_request_fingerprint
                != self.treatment.materialized_request_fingerprint
        {
            return Err(live_error(
                LivePreparationErrorCode::UnsafeMaterializedCandidate,
                "control/treatment identities do not match the certified pair",
            ));
        }
        if self.sequence.len() != SEQUENCE_LENGTH {
            return Err(live_error(
                LivePreparationErrorCode::IncompleteSequence,
                "live experiment must contain exactly seven sequence steps",
            ));
        }
        let control = self.control_request.request_fingerprint()?;
        let treatment = self.treatment.materialized_request_fingerprint.clone();
        let interference = self.interference_request.request_fingerprint()?;
        let expected = [
            (
                "A1",
                LiveSequenceRole::Control,
                control.clone(),
                LiveSequenceRelation::Initial,
            ),
            (
                "A2",
                LiveSequenceRole::Control,
                control.clone(),
                LiveSequenceRelation::ExactRepeatOf("A1".to_string()),
            ),
            (
                "C1",
                LiveSequenceRole::CandidateTreatment,
                treatment.clone(),
                LiveSequenceRelation::Initial,
            ),
            (
                "C2",
                LiveSequenceRole::CandidateTreatment,
                treatment.clone(),
                LiveSequenceRelation::ExactRepeatOf("C1".to_string()),
            ),
            (
                "B1",
                LiveSequenceRole::Interference,
                interference,
                LiveSequenceRelation::Independent,
            ),
            (
                "A3",
                LiveSequenceRole::Control,
                control,
                LiveSequenceRelation::ReturnTo("A1".to_string()),
            ),
            (
                "C3",
                LiveSequenceRole::CandidateTreatment,
                treatment,
                LiveSequenceRelation::ReturnTo("C1".to_string()),
            ),
        ];
        for (actual, (step_id, role, fingerprint, relation)) in self.sequence.iter().zip(expected) {
            if actual.step_id != step_id
                || actual.role != role
                || actual.request_fingerprint != fingerprint
                || actual.relation != relation
            {
                return Err(live_error(
                    LivePreparationErrorCode::IncompleteSequence,
                    "live sequence does not match the fixed A1/A2/C1/C2/B1/A3/C3 order",
                ));
            }
        }
        validate_provenance(&self.provenance)
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| BenchmarkError::validation(error.to_string()))
    }
}

pub fn build_live_experiment_definition(
    source_request: &ConformanceRequest,
    materialized: MaterializedCandidate,
    source_case_id: impl Into<String>,
    candidate_case_id: impl Into<String>,
    provenance: BTreeMap<String, String>,
) -> Result<LiveExperimentDefinition, BenchmarkError> {
    source_request.validate()?;
    materialized.validate().map_err(|error| {
        live_error(
            LivePreparationErrorCode::UnsafeMaterializedCandidate,
            format!("materialized candidate is not certified: {error}"),
        )
    })?;
    validate_provenance(&provenance)?;
    if materialized.source_request_fingerprint != source_request.request_fingerprint()? {
        return Err(live_error(
            LivePreparationErrorCode::UnsafeMaterializedCandidate,
            "source request does not match the certified candidate",
        ));
    }
    let pair = build_candidate_experiment_pair(
        &materialized,
        source_case_id,
        candidate_case_id,
        BTreeMap::from([("purpose".to_string(), "p0-l6a-live-preparation".to_string())]),
    )?;
    let marker = &materialized.source_request_fingerprint[..16];
    let mut interference_request = source_request.clone();
    interference_request.context.system_instruction = format!(
        "[interference-marker:{marker}]\n{}",
        source_request.context.system_instruction
    );
    interference_request.validate()?;
    let control_fingerprint = source_request.request_fingerprint()?;
    let treatment_fingerprint = materialized.materialized_request_fingerprint.clone();
    let interference_fingerprint = interference_request.request_fingerprint()?;
    let sequence = vec![
        step(
            "A1",
            LiveSequenceRole::Control,
            &control_fingerprint,
            LiveSequenceRelation::Initial,
        ),
        step(
            "A2",
            LiveSequenceRole::Control,
            &control_fingerprint,
            LiveSequenceRelation::ExactRepeatOf("A1".to_string()),
        ),
        step(
            "C1",
            LiveSequenceRole::CandidateTreatment,
            &treatment_fingerprint,
            LiveSequenceRelation::Initial,
        ),
        step(
            "C2",
            LiveSequenceRole::CandidateTreatment,
            &treatment_fingerprint,
            LiveSequenceRelation::ExactRepeatOf("C1".to_string()),
        ),
        step(
            "B1",
            LiveSequenceRole::Interference,
            &interference_fingerprint,
            LiveSequenceRelation::Independent,
        ),
        step(
            "A3",
            LiveSequenceRole::Control,
            &control_fingerprint,
            LiveSequenceRelation::ReturnTo("A1".to_string()),
        ),
        step(
            "C3",
            LiveSequenceRole::CandidateTreatment,
            &treatment_fingerprint,
            LiveSequenceRelation::ReturnTo("C1".to_string()),
        ),
    ];
    let experiment_id = canonical_hash(&json!({
        "schema_id": LIVE_HARNESS_SCHEMA_ID,
        "source": control_fingerprint,
        "treatment": treatment_fingerprint,
        "interference": interference_fingerprint,
        "pair": pair.pair_fingerprint,
        "sequence": sequence,
    }))
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let definition = LiveExperimentDefinition {
        schema_id: LIVE_HARNESS_SCHEMA_ID.to_string(),
        schema_version: LIVE_HARNESS_SCHEMA_VERSION,
        experiment_id,
        control_request: source_request.clone(),
        treatment: materialized,
        interference_request,
        pair,
        sequence,
        provenance,
    };
    definition.validate()?;
    Ok(definition)
}

fn step(
    step_id: &str,
    role: LiveSequenceRole,
    request_fingerprint: &str,
    relation: LiveSequenceRelation,
) -> LiveSequenceStep {
    LiveSequenceStep {
        step_id: step_id.to_string(),
        role,
        request_fingerprint: request_fingerprint.to_string(),
        relation,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawLlamaCppEvidence {
    pub schema_id: String,
    pub schema_version: u32,
    pub request_identity_fingerprint: String,
    pub response_status: u16,
    pub response_body_bytes: usize,
    pub response_body_fingerprint: String,
    pub elapsed_ms: u64,
    pub raw_telemetry: BTreeMap<String, Value>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl RawLlamaCppEvidence {
    fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != RAW_EVIDENCE_SCHEMA_ID
            || self.schema_version != RAW_EVIDENCE_SCHEMA_VERSION
            || self.response_body_bytes > MAX_RESPONSE_BYTES
            || self.raw_telemetry.len() > 16
        {
            return Err(live_error(
                LivePreparationErrorCode::EvidenceStateMismatch,
                "raw evidence record violates its schema or bounds",
            ));
        }
        validate_provenance(&self.provenance)
    }
}

pub trait LiveRawEvidenceSource {
    fn raw_evidence(&self) -> Vec<RawLlamaCppEvidence>;
}

#[derive(Debug)]
pub struct LoopbackLlamaCppTransport {
    endpoint: LoopbackEndpoint,
    client: Client,
    max_response_bytes: usize,
    raw_evidence: Vec<RawLlamaCppEvidence>,
}

impl LoopbackLlamaCppTransport {
    pub fn new(config: &LlamaCppLiveConfig) -> Result<Self, BenchmarkError> {
        config.validate()?;
        let client = Client::builder()
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .redirect(Policy::none())
            .build()
            .map_err(|error| {
                live_error(
                    LivePreparationErrorCode::InvalidConfiguration,
                    error.to_string(),
                )
            })?;
        Ok(Self {
            endpoint: config.endpoint.clone(),
            client,
            max_response_bytes: config.max_response_bytes,
            raw_evidence: Vec::new(),
        })
    }
}

impl LiveRawEvidenceSource for LoopbackLlamaCppTransport {
    fn raw_evidence(&self) -> Vec<RawLlamaCppEvidence> {
        self.raw_evidence.clone()
    }
}

impl<T: LiveRawEvidenceSource + ?Sized> LiveRawEvidenceSource for &mut T {
    fn raw_evidence(&self) -> Vec<RawLlamaCppEvidence> {
        (**self).raw_evidence()
    }
}

impl LlamaCppTransport for LoopbackLlamaCppTransport {
    fn chat_completion(
        &mut self,
        request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        let body = serde_json::to_vec(request).map_err(|error| {
            live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                error.to_string(),
            )
        })?;
        let request_identity_fingerprint = sha256_hex(&body);
        let started = Instant::now();
        let response = self
            .client
            .post(&self.endpoint.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .map_err(|error| {
                let code = if error.is_timeout() {
                    LivePreparationErrorCode::RequestTimeout
                } else {
                    LivePreparationErrorCode::EndpointUnavailable
                };
                live_error(code, bounded_error(&error.to_string()))
            })?;
        let status = response.status();
        if !status.is_success() {
            return Err(live_error(
                LivePreparationErrorCode::ServerError,
                format!("llama.cpp endpoint returned HTTP {}", status.as_u16()),
            ));
        }
        let response = response;
        let mut bytes = Vec::new();
        response
            .take((self.max_response_bytes + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                live_error(
                    LivePreparationErrorCode::EndpointUnavailable,
                    bounded_error(&error.to_string()),
                )
            })?;
        if bytes.len() > self.max_response_bytes {
            return Err(live_error(
                LivePreparationErrorCode::ResponseTooLarge,
                "llama.cpp response exceeded the configured bound",
            ));
        }
        let body_fingerprint = sha256_hex(&bytes);
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            live_error(
                LivePreparationErrorCode::MalformedResponse,
                bounded_error(&error.to_string()),
            )
        })?;
        let parsed = LlamaCppResponse::from_json(value).map_err(|error| {
            live_error(
                LivePreparationErrorCode::MalformedResponse,
                bounded_error(&error.to_string()),
            )
        })?;
        let raw_telemetry = bounded_telemetry(&parsed.raw_telemetry);
        self.raw_evidence.push(RawLlamaCppEvidence {
            schema_id: RAW_EVIDENCE_SCHEMA_ID.to_string(),
            schema_version: RAW_EVIDENCE_SCHEMA_VERSION,
            request_identity_fingerprint,
            response_status: status.as_u16(),
            response_body_bytes: bytes.len(),
            response_body_fingerprint: body_fingerprint,
            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            raw_telemetry,
            provenance: BTreeMap::from([("transport".to_string(), "loopback-http".to_string())]),
        });
        Ok(parsed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveEvidenceState {
    Prepared,
    Executed,
    Normalized,
    Admitted,
    Partial,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveFailure {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveReadinessRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub state: LiveEvidenceState,
    pub network_calls: u32,
    pub experiment_id: String,
    pub source_request_fingerprint: String,
    pub treatment_request_fingerprint: String,
    pub safety_certificate_fingerprint: String,
    pub runtime_config_fingerprint: String,
    pub endpoint: LoopbackEndpoint,
    pub sequence_step_ids: Vec<String>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveRunRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub state: LiveEvidenceState,
    pub expected_steps: usize,
    pub completed_steps: usize,
    pub raw_evidence: Vec<RawLlamaCppEvidence>,
    pub normalized_result: Option<ConformanceResult>,
    pub failure: Option<LiveFailure>,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl LiveRunRecord {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != LIVE_HARNESS_SCHEMA_ID
            || self.schema_version != LIVE_HARNESS_SCHEMA_VERSION
            || self.expected_steps != SEQUENCE_LENGTH
            || self.completed_steps > self.expected_steps
        {
            return Err(live_error(
                LivePreparationErrorCode::EvidenceStateMismatch,
                "live run record has invalid bounds or schema",
            ));
        }
        for evidence in &self.raw_evidence {
            evidence.validate()?;
        }
        match self.state {
            LiveEvidenceState::Normalized | LiveEvidenceState::Admitted => {
                if self.completed_steps != self.expected_steps || self.normalized_result.is_none() {
                    return Err(live_error(
                        LivePreparationErrorCode::EvidenceStateMismatch,
                        "complete evidence state requires all sequence steps and a normalized result",
                    ));
                }
            }
            LiveEvidenceState::Partial | LiveEvidenceState::Failed => {
                if self.normalized_result.is_some() {
                    return Err(live_error(
                        LivePreparationErrorCode::EvidenceStateMismatch,
                        "partial or failed runs cannot contain a final normalized result",
                    ));
                }
            }
            LiveEvidenceState::Prepared | LiveEvidenceState::Executed => {}
        }
        validate_provenance(&self.provenance)
    }
}

pub fn preflight_live_experiment(
    definition: &LiveExperimentDefinition,
    config: &LlamaCppLiveConfig,
) -> Result<LiveReadinessRecord, BenchmarkError> {
    definition.validate()?;
    config.validate()?;
    for request in [
        &definition.control_request,
        &definition.treatment.materialized_request,
        &definition.interference_request,
    ] {
        let projected = project_llama_cpp_request(request)?;
        let bytes = serde_json::to_vec(&projected)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
        if bytes.len() > config.max_context_bytes {
            return Err(live_error(
                LivePreparationErrorCode::ContextLimitRejected,
                "projected request exceeds the configured context bound",
            ));
        }
    }
    let runtime_config_fingerprint = semantic_config_fingerprint(config, definition)?;
    Ok(LiveReadinessRecord {
        schema_id: LIVE_HARNESS_SCHEMA_ID.to_string(),
        schema_version: LIVE_HARNESS_SCHEMA_VERSION,
        state: LiveEvidenceState::Prepared,
        network_calls: 0,
        experiment_id: definition.experiment_id.clone(),
        source_request_fingerprint: definition.pair.source_request_fingerprint.clone(),
        treatment_request_fingerprint: definition.pair.candidate_request_fingerprint.clone(),
        safety_certificate_fingerprint: definition.pair.safety_certificate_fingerprint.clone(),
        runtime_config_fingerprint,
        endpoint: config.endpoint.clone(),
        sequence_step_ids: definition
            .sequence
            .iter()
            .map(|step| step.step_id.clone())
            .collect(),
        provenance: BTreeMap::from([
            ("network".to_string(), "not_contacted".to_string()),
            ("evidence".to_string(), "prepared_only".to_string()),
        ]),
    })
}

pub fn live_experiment_identity(
    definition: &LiveExperimentDefinition,
    config: &LlamaCppLiveConfig,
) -> Result<String, BenchmarkError> {
    preflight_live_experiment(definition, config)?;
    semantic_config_fingerprint(config, definition)
}

pub fn execute_live_experiment<T>(
    definition: &LiveExperimentDefinition,
    config: &LlamaCppLiveConfig,
    transport: &mut T,
) -> Result<LiveRunRecord, BenchmarkError>
where
    T: LlamaCppTransport + LiveRawEvidenceSource,
{
    let readiness = preflight_live_experiment(definition, config)?;
    if !config.execute_live {
        return Err(live_error(
            LivePreparationErrorCode::LiveOptInRequired,
            "live execution requires explicit execute_live=true",
        ));
    }
    let experiment = conformance_experiment(definition, config, &readiness.experiment_id)?;
    let runtime = config.runtime_profile.identity.clone();
    let observed_at = config
        .provenance
        .get("observed_at")
        .cloned()
        .unwrap_or_else(|| "caller-supplied-observation-time-required".to_string());
    let mut runner = LlamaCppConformanceRunner::new_live(transport, observed_at, runtime);
    match experiment.run(&mut runner) {
        Ok(normalized_result) => {
            let record = LiveRunRecord {
                schema_id: LIVE_HARNESS_SCHEMA_ID.to_string(),
                schema_version: LIVE_HARNESS_SCHEMA_VERSION,
                experiment_id: definition.experiment_id.clone(),
                state: LiveEvidenceState::Normalized,
                expected_steps: SEQUENCE_LENGTH,
                completed_steps: SEQUENCE_LENGTH,
                raw_evidence: runner.transport().raw_evidence(),
                normalized_result: Some(normalized_result),
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
            let raw_evidence = runner.transport().raw_evidence();
            let state = if raw_evidence.is_empty() {
                LiveEvidenceState::Failed
            } else {
                LiveEvidenceState::Partial
            };
            let record = LiveRunRecord {
                schema_id: LIVE_HARNESS_SCHEMA_ID.to_string(),
                schema_version: LIVE_HARNESS_SCHEMA_VERSION,
                experiment_id: definition.experiment_id.clone(),
                state,
                expected_steps: SEQUENCE_LENGTH,
                completed_steps: raw_evidence.len().min(SEQUENCE_LENGTH),
                raw_evidence,
                normalized_result: None,
                failure: Some(LiveFailure {
                    code: failure_code(&error),
                    message: bounded_error(&error.to_string()),
                }),
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

fn conformance_experiment(
    definition: &LiveExperimentDefinition,
    config: &LlamaCppLiveConfig,
    experiment_id: &str,
) -> Result<ConformanceExperiment, BenchmarkError> {
    let expected = |notes: &str| ExpectedObservationMetadata {
        cache_reuse: ExpectedObservationState::ToBeObserved,
        cache_write: ExpectedObservationState::ToBeObserved,
        notes: notes.to_string(),
    };
    let cases = vec![
        ConformanceCase {
            case_id: "A1".to_string(),
            mutation: MutationClass::Baseline,
            request: definition.control_request.clone(),
            relationship: CaseRelationship::Baseline,
            expected_observation: expected("Control observation in fixed sequence."),
        },
        ConformanceCase {
            case_id: "A2".to_string(),
            mutation: MutationClass::ExactRepeat,
            request: definition.control_request.clone(),
            relationship: CaseRelationship::ExactRepeatOf("A1".to_string()),
            expected_observation: expected("Exact control repeat in fixed sequence."),
        },
        ConformanceCase {
            case_id: "C1".to_string(),
            mutation: MutationClass::StableContentBeginning,
            request: definition.treatment.materialized_request.clone(),
            relationship: CaseRelationship::MutationOf("A1".to_string()),
            expected_observation: expected("Certified candidate treatment observation."),
        },
        ConformanceCase {
            case_id: "C2".to_string(),
            mutation: MutationClass::ExactRepeat,
            request: definition.treatment.materialized_request.clone(),
            relationship: CaseRelationship::ExactRepeatOf("C1".to_string()),
            expected_observation: expected("Exact candidate treatment repeat."),
        },
        ConformanceCase {
            case_id: "B1".to_string(),
            mutation: MutationClass::CurrentContentEnd,
            request: definition.interference_request.clone(),
            relationship: CaseRelationship::MutationOf("A1".to_string()),
            expected_observation: expected("Deterministic interference observation."),
        },
        ConformanceCase {
            case_id: "A3".to_string(),
            mutation: MutationClass::StableContentBeginning,
            request: definition.control_request.clone(),
            relationship: CaseRelationship::MutationOf("A1".to_string()),
            expected_observation: expected("Control return observation."),
        },
        ConformanceCase {
            case_id: "C3".to_string(),
            mutation: MutationClass::StableContentBeginning,
            request: definition.treatment.materialized_request.clone(),
            relationship: CaseRelationship::MutationOf("C1".to_string()),
            expected_observation: expected("Candidate treatment return observation."),
        },
    ];
    let mut metadata = config.provenance.clone();
    metadata.insert(
        "harness".to_string(),
        "p0-l6a-prepared-loopback".to_string(),
    );
    Ok(ConformanceExperiment {
        schema_id: crate::conformance::CONFORMANCE_SCHEMA_ID.to_string(),
        schema_version: crate::conformance::CONFORMANCE_SCHEMA_VERSION,
        experiment_id: experiment_id.to_string(),
        baseline_request: definition.control_request.clone(),
        cases,
        runtime_profile: config.runtime_profile.clone(),
        metadata,
    })
}

fn semantic_config_fingerprint(
    config: &LlamaCppLiveConfig,
    definition: &LiveExperimentDefinition,
) -> Result<String, BenchmarkError> {
    let mut config_value = serde_json::to_value(config)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    if let Value::Object(object) = &mut config_value {
        object.remove("execute_live");
        object.remove("provenance");
    }
    let mut definition_value = serde_json::to_value(definition)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    if let Value::Object(object) = &mut definition_value {
        object.remove("provenance");
    }
    canonical_hash(&json!({"definition": definition_value, "config": config_value}))
        .map_err(|error| BenchmarkError::validation(error.to_string()))
}

fn bounded_telemetry(values: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    values
        .iter()
        .filter_map(|(key, value)| {
            let encoded = serde_json::to_vec(value).ok()?;
            if encoded.len() <= MAX_RAW_TELEMETRY_BYTES {
                Some((key.clone(), value.clone()))
            } else {
                Some((
                    key.clone(),
                    Value::String("omitted_due_to_bound".to_string()),
                ))
            }
        })
        .collect()
}

fn failure_code(error: &BenchmarkError) -> String {
    match error {
        BenchmarkError::LiveHarness { code, .. } => match code {
            LivePreparationErrorCode::NormalizationConflict => "normalization_conflict".to_string(),
            LivePreparationErrorCode::ContextLimitRejected => "context_limit_rejected".to_string(),
            LivePreparationErrorCode::RequestTimeout => "request_timeout".to_string(),
            LivePreparationErrorCode::EndpointUnavailable => "endpoint_unavailable".to_string(),
            LivePreparationErrorCode::ServerError => "server_error".to_string(),
            LivePreparationErrorCode::MalformedResponse => "malformed_response".to_string(),
            LivePreparationErrorCode::ResponseTooLarge => "response_too_large".to_string(),
            other => other.to_string(),
        },
        _ => "incomplete_sequence".to_string(),
    }
}

fn live_error(code: LivePreparationErrorCode, message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::live_harness(code, bounded_error(&message.into()))
}

fn bounded_error(value: &str) -> String {
    value.chars().take(MAX_TEXT_BYTES).collect()
}

fn bounded_text(field: &str, value: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_TEXT_BYTES {
        return Err(live_error(
            LivePreparationErrorCode::InvalidConfiguration,
            format!("{field} is empty or exceeds its bound"),
        ));
    }
    Ok(())
}

fn validate_map(
    values: &BTreeMap<String, String>,
    field: &str,
    reject_paths: bool,
) -> Result<(), BenchmarkError> {
    if values.len() > MAX_PROVENANCE {
        return Err(live_error(
            LivePreparationErrorCode::InvalidConfiguration,
            format!("{field} exceeds its bound"),
        ));
    }
    for (key, value) in values {
        bounded_text(&format!("{field} key"), key)?;
        bounded_text(&format!("{field} value"), value)?;
        if reject_paths
            && (value.starts_with('/')
                || value.contains(':')
                || value.split(['/', '\\']).any(|part| part == ".."))
        {
            return Err(live_error(
                LivePreparationErrorCode::InvalidConfiguration,
                format!("{field} contains an unsafe absolute or parent path"),
            ));
        }
    }
    Ok(())
}

fn validate_provenance(values: &BTreeMap<String, String>) -> Result<(), BenchmarkError> {
    validate_map(values, "provenance", false)
}
