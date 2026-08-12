//! Provider/runtime-neutral cache-conformance experiment foundation.
//!
//! This module extends the existing isolated controlled-benchmark crate with
//! an experiment structure for later cache-behaviour observations. It only
//! constructs deterministic request variants and executes them through an
//! in-process mock transport. It does not infer cache outcomes or contact a
//! runtime/provider.

use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, canonical_json, hash_text};
use prefixity_core::observation::{
    ArtifactReference, CacheBehavior, CacheObservation, ContextIdentity, ObservationOutcome,
    Observed, ResourceUsage, RuntimeIdentity, TimingObservation, TokenAccounting,
    CACHE_OBSERVATION_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const CONFORMANCE_SCHEMA_ID: &str = "prefixity.cache-conformance";
pub const CONFORMANCE_SCHEMA_VERSION: u32 = 1;
pub const CONFORMANCE_RESULT_SCHEMA_ID: &str = "prefixity.cache-conformance-result";
pub const CONFORMANCE_RESULT_SCHEMA_VERSION: u32 = 1;
pub const MOCK_TRANSPORT_ID: &str = "prefixity.mock-conformance-transport";
pub const MOCK_TRANSPORT_VERSION: &str = "1";

const MAX_ID_BYTES: usize = 256;
const MAX_TEXT_BYTES: usize = 64 * 1024;
const MAX_CASES: usize = 128;
const MAX_TOOLS: usize = 64;
const MAX_CONTEXT_ARTIFACTS: usize = 128;
const MAX_JSON_FIELDS: usize = 128;
const MAX_METADATA_FIELDS: usize = 64;

/// A controlled mutation class. The class describes what was changed, not
/// what a runtime is expected to do with that change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationClass {
    Baseline,
    ExactRepeat,
    StableContentBeginning,
    CurrentContentEnd,
    WhitespaceOnly,
    JsonFieldOrder,
    ToolDefinitionOrder,
    OptionalToolField,
    ToolDefinitionChange,
    ModelIdentifier,
    ReasoningSetting,
    ResponseFormat,
}

/// The relationship that lets a case be compared with a baseline or an
/// earlier case without relying on vector position alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "case_id", rename_all = "snake_case")]
pub enum CaseRelationship {
    Baseline,
    ExactRepeatOf(String),
    MutationOf(String),
}

/// Expected result metadata remains deliberately non-committal at P0-L4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedObservationState {
    Unknown,
    ToBeObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedObservationMetadata {
    pub cache_reuse: ExpectedObservationState,
    pub cache_write: ExpectedObservationState,
    pub notes: String,
}

/// A JSON field whose vector position is intentional and therefore observable
/// by the request fingerprint. Nested JSON values remain canonicalized by the
/// existing benchmark hashing helper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonField {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderedJsonObject {
    pub fields: Vec<JsonField>,
}

impl OrderedJsonObject {
    pub fn new(fields: Vec<JsonField>) -> Self {
        Self { fields }
    }

    pub fn validate(&self, field: &str) -> Result<(), BenchmarkError> {
        if self.fields.len() > MAX_JSON_FIELDS {
            return Err(validation(format!(
                "{field} exceeds {MAX_JSON_FIELDS} fields"
            )));
        }
        let mut names = BTreeSet::new();
        for entry in &self.fields {
            validate_id(&format!("{field}.name"), &entry.name)?;
            if !names.insert(&entry.name) {
                return Err(validation(format!(
                    "{field} contains duplicate field names"
                )));
            }
        }
        Ok(())
    }

    pub fn with_field(&self, name: &str, value: Value) -> Result<Self, BenchmarkError> {
        let mut fields = self.fields.clone();
        if let Some(existing) = fields.iter_mut().find(|field| field.name == name) {
            existing.value = value;
        } else {
            fields.push(JsonField {
                name: name.to_string(),
                value,
            });
        }
        let object = Self { fields };
        object.validate("ordered_object")?;
        Ok(object)
    }

    pub fn reordered(&self) -> Self {
        let mut fields = self.fields.clone();
        fields.reverse();
        Self { fields }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextArtifactInput {
    pub artifact_id: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: OrderedJsonObject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestContext {
    pub system_instruction: String,
    pub artifacts: Vec<ContextArtifactInput>,
    pub user_content: String,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelope {
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningSetting>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningSetting {
    Disabled,
    Enabled,
    Level(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "schema", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema(OrderedJsonObject),
}

/// The smallest neutral request representation needed by this harness.
/// Context is intentionally separate from the request envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceRequest {
    pub context: RequestContext,
    pub envelope: RequestEnvelope,
}

impl ConformanceRequest {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        validate_text(
            "context.system_instruction",
            &self.context.system_instruction,
        )?;
        validate_text("context.user_content", &self.context.user_content)?;
        if self.context.artifacts.len() > MAX_CONTEXT_ARTIFACTS {
            return Err(validation("context.artifacts exceeds its bound"));
        }
        let mut artifact_ids = BTreeSet::new();
        for artifact in &self.context.artifacts {
            validate_id("context.artifact_id", &artifact.artifact_id)?;
            validate_text("context.artifact.content", &artifact.content)?;
            if !artifact_ids.insert(&artifact.artifact_id) {
                return Err(validation("context contains duplicate artifact IDs"));
            }
        }
        if self.context.tools.len() > MAX_TOOLS {
            return Err(validation("context.tools exceeds its bound"));
        }
        let mut tool_names = BTreeSet::new();
        for tool in &self.context.tools {
            validate_id("context.tool.name", &tool.name)?;
            validate_text("context.tool.description", &tool.description)?;
            tool.parameters.validate("context.tool.parameters")?;
            if !tool_names.insert(&tool.name) {
                return Err(validation("context contains duplicate tool names"));
            }
        }
        validate_id("envelope.model", &self.envelope.model)?;
        if let Some(ReasoningSetting::Level(level)) = &self.envelope.reasoning {
            validate_id("envelope.reasoning.level", level)?;
        }
        if let Some(ResponseFormat::JsonSchema(schema)) = &self.envelope.response_format {
            schema.validate("envelope.response_format.schema")?;
        }
        Ok(())
    }

    pub fn request_fingerprint(&self) -> Result<String, BenchmarkError> {
        self.validate()?;
        canonical_hash(self).map_err(|error| validation(error.to_string()))
    }

    pub fn context_fingerprint(&self) -> Result<String, BenchmarkError> {
        self.validate()?;
        canonical_hash(&self.context).map_err(|error| validation(error.to_string()))
    }

    pub(crate) fn artifact_references(&self) -> Vec<ArtifactReference> {
        self.context
            .artifacts
            .iter()
            .map(|artifact| ArtifactReference {
                artifact_id: artifact.artifact_id.clone(),
                content_hash: Observed::Known(hash_text(&artifact.content)),
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileReference {
    pub profile_id: String,
    pub identity: RuntimeIdentity,
}

impl RuntimeProfileReference {
    fn validate(&self) -> Result<(), BenchmarkError> {
        validate_id("runtime_profile.profile_id", &self.profile_id)?;
        validate_id("runtime_profile.identity.backend", &self.identity.backend)?;
        for (field, value) in [
            ("provider", &self.identity.provider),
            ("model", &self.identity.model),
            ("protocol", &self.identity.protocol),
            ("runtime", &self.identity.runtime),
            ("runtime_version", &self.identity.runtime_version),
        ] {
            if let Observed::Known(value) = value {
                validate_id(&format!("runtime_profile.identity.{field}"), value)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceCase {
    pub case_id: String,
    pub mutation: MutationClass,
    pub request: ConformanceRequest,
    pub relationship: CaseRelationship,
    pub expected_observation: ExpectedObservationMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceExperiment {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub baseline_request: ConformanceRequest,
    pub cases: Vec<ConformanceCase>,
    pub runtime_profile: RuntimeProfileReference,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl ConformanceExperiment {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CONFORMANCE_SCHEMA_ID {
            return Err(validation("unsupported conformance schema_id"));
        }
        if self.schema_version != CONFORMANCE_SCHEMA_VERSION {
            return Err(validation("unsupported conformance schema_version"));
        }
        validate_id("experiment_id", &self.experiment_id)?;
        self.baseline_request.validate()?;
        self.runtime_profile.validate()?;
        validate_metadata(&self.metadata)?;
        if self.cases.is_empty() || self.cases.len() > MAX_CASES {
            return Err(validation(
                "experiment must contain a bounded non-empty case list",
            ));
        }

        let mut case_ids = BTreeSet::new();
        let baseline_fingerprint = self.baseline_request.request_fingerprint()?;
        let mut baseline_case_count = 0;
        for case in &self.cases {
            validate_id("case_id", &case.case_id)?;
            if !case_ids.insert(&case.case_id) {
                return Err(validation("experiment contains duplicate case IDs"));
            }
            case.request.validate()?;
            if case.expected_observation.notes.trim().is_empty() {
                return Err(validation("expected observation notes must not be empty"));
            }
            match &case.relationship {
                CaseRelationship::Baseline => {
                    baseline_case_count += 1;
                    if case.mutation != MutationClass::Baseline {
                        return Err(validation(
                            "baseline relationship requires baseline mutation",
                        ));
                    }
                    if case.request.request_fingerprint()? != baseline_fingerprint {
                        return Err(validation("baseline case does not match baseline_request"));
                    }
                }
                CaseRelationship::ExactRepeatOf(target) => {
                    if case.mutation != MutationClass::ExactRepeat {
                        return Err(validation("exact-repeat relationship has wrong mutation"));
                    }
                    if target == &case.case_id {
                        return Err(validation("case cannot repeat itself"));
                    }
                }
                CaseRelationship::MutationOf(target) => {
                    if case.mutation == MutationClass::Baseline
                        || case.mutation == MutationClass::ExactRepeat
                    {
                        return Err(validation("mutation relationship has wrong mutation class"));
                    }
                    if target == &case.case_id {
                        return Err(validation("case cannot mutate itself"));
                    }
                }
            }
        }
        if baseline_case_count != 1 {
            return Err(validation(
                "experiment must contain exactly one baseline case",
            ));
        }
        for case in &self.cases {
            match &case.relationship {
                CaseRelationship::Baseline => {}
                CaseRelationship::ExactRepeatOf(target) => {
                    let target_case = self
                        .cases
                        .iter()
                        .find(|candidate| candidate.case_id == *target)
                        .ok_or_else(|| validation("exact-repeat target case does not exist"))?;
                    if case.request.request_fingerprint()?
                        != target_case.request.request_fingerprint()?
                    {
                        return Err(validation("exact-repeat request differs from its target"));
                    }
                }
                CaseRelationship::MutationOf(target) => {
                    if !case_ids.contains(target) {
                        return Err(validation("mutation target case does not exist"));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| validation(error.to_string()))
    }

    pub fn run<R: ConformanceRunner>(
        &self,
        runner: &mut R,
    ) -> Result<ConformanceResult, BenchmarkError> {
        self.validate()?;
        let mut results = Vec::with_capacity(self.cases.len());
        for case in &self.cases {
            results.push(runner.execute(&self.experiment_id, &self.runtime_profile, case)?);
        }
        Ok(ConformanceResult {
            schema_id: CONFORMANCE_RESULT_SCHEMA_ID.to_string(),
            schema_version: CONFORMANCE_RESULT_SCHEMA_VERSION,
            experiment_id: self.experiment_id.clone(),
            runtime_profile: self.runtime_profile.clone(),
            cases: results,
            status: CompletionStatus::Complete,
            provenance: runner.provenance(),
        })
    }
}

/// One case-level association between a conformance case and the existing
/// neutral CacheObservation contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceCaseResult {
    pub experiment_id: String,
    pub case_id: String,
    pub mutation: MutationClass,
    pub relationship: CaseRelationship,
    pub request_fingerprint: String,
    pub context_fingerprint: String,
    pub observation: CacheObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionStatus {
    Complete,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceResult {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub runtime_profile: RuntimeProfileReference,
    pub cases: Vec<ConformanceCaseResult>,
    pub status: CompletionStatus,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl ConformanceResult {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CONFORMANCE_RESULT_SCHEMA_ID
            || self.schema_version != CONFORMANCE_RESULT_SCHEMA_VERSION
        {
            return Err(validation("unsupported conformance result schema"));
        }
        validate_id("result.experiment_id", &self.experiment_id)?;
        self.runtime_profile.validate()?;
        validate_metadata(&self.provenance)?;
        if self.cases.is_empty() || self.cases.len() > MAX_CASES {
            return Err(validation(
                "result must contain a bounded non-empty case list",
            ));
        }
        let mut ids = BTreeSet::new();
        for case in &self.cases {
            if case.experiment_id != self.experiment_id {
                return Err(validation(
                    "case result experiment_id does not match result",
                ));
            }
            if !ids.insert(&case.case_id) {
                return Err(validation("result contains duplicate case IDs"));
            }
            validate_id("result.case_id", &case.case_id)?;
            validate_id("result.request_fingerprint", &case.request_fingerprint)?;
            validate_id("result.context_fingerprint", &case.context_fingerprint)?;
            case.observation.validate().map_err(|error| {
                validation(format!("case {} observation: {error}", case.case_id))
            })?;
            if case.observation.context.serialized_request_identity
                != Observed::Known(case.request_fingerprint.clone())
            {
                return Err(validation("observation request identity is not traceable"));
            }
            if case.observation.runtime != self.runtime_profile.identity {
                return Err(validation(
                    "observation runtime does not match result profile",
                ));
            }
        }
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        canonical_json(self).map_err(|error| validation(error.to_string()))
    }
}

/// Narrow transport boundary for future runtime adapters.
pub trait ConformanceRunner {
    fn execute(
        &mut self,
        experiment_id: &str,
        runtime_profile: &RuntimeProfileReference,
        case: &ConformanceCase,
    ) -> Result<ConformanceCaseResult, BenchmarkError>;

    fn provenance(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("runner".to_string(), MOCK_TRANSPORT_ID.to_string()),
            ("cache_metrics".to_string(), "not_observed".to_string()),
        ])
    }
}

/// Deterministic, in-process runner. It records identity and explicit absence
/// but never fabricates cache-hit, token, latency, or quality values.
#[derive(Debug, Clone)]
pub struct MockConformanceRunner {
    observed_at: String,
    runtime: RuntimeIdentity,
}

impl MockConformanceRunner {
    pub fn new(observed_at: impl Into<String>, runtime: RuntimeIdentity) -> Self {
        Self {
            observed_at: observed_at.into(),
            runtime,
        }
    }
}

impl ConformanceRunner for MockConformanceRunner {
    fn execute(
        &mut self,
        experiment_id: &str,
        runtime_profile: &RuntimeProfileReference,
        case: &ConformanceCase,
    ) -> Result<ConformanceCaseResult, BenchmarkError> {
        case.request.validate()?;
        let request_fingerprint = case.request.request_fingerprint()?;
        let context_fingerprint = case.request.context_fingerprint()?;
        let observation_id = format!(
            "observation-{}",
            &hash_text(&format!("{experiment_id}:{}", case.case_id))[..24]
        );
        let observation = CacheObservation {
            schema_version: CACHE_OBSERVATION_SCHEMA_VERSION,
            observation_id,
            observed_at: self.observed_at.clone(),
            runtime: self.runtime.clone(),
            context: ContextIdentity {
                artifacts: case.request.artifact_references(),
                serialized_request_identity: Observed::Known(request_fingerprint.clone()),
                reusable_prefix_identity: Observed::Unknown,
            },
            raw_telemetry: BTreeMap::from([
                (
                    "transport".to_string(),
                    Value::String(MOCK_TRANSPORT_ID.to_string()),
                ),
                (
                    "cache_metrics".to_string(),
                    Value::String("not_observed".to_string()),
                ),
            ]),
            accounting: TokenAccounting::default(),
            timing: TimingObservation::default(),
            resources: ResourceUsage::default(),
            cache: CacheBehavior::default(),
            outcome: ObservationOutcome::default(),
        };
        observation.validate().map_err(|error| {
            validation(format!("mock observation for {}: {error}", case.case_id))
        })?;
        if runtime_profile.identity.backend != self.runtime.backend {
            return Err(validation(
                "runner runtime does not match experiment profile",
            ));
        }
        Ok(ConformanceCaseResult {
            experiment_id: experiment_id.to_string(),
            case_id: case.case_id.clone(),
            mutation: case.mutation,
            relationship: case.relationship.clone(),
            request_fingerprint,
            context_fingerprint,
            observation,
        })
    }
}

fn validation(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::validation(message)
}

fn validate_id(field: &str, value: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_ID_BYTES {
        return Err(validation(format!(
            "{field} is empty or exceeds {MAX_ID_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_text(field: &str, value: &str) -> Result<(), BenchmarkError> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(validation(format!(
            "{field} exceeds {MAX_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_metadata(values: &BTreeMap<String, String>) -> Result<(), BenchmarkError> {
    if values.len() > MAX_METADATA_FIELDS {
        return Err(validation("metadata exceeds its field bound"));
    }
    for (key, value) in values {
        validate_id("metadata key", key)?;
        validate_text("metadata value", value)?;
    }
    Ok(())
}
