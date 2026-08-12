//! Offline llama-server request projection and response observation.
//!
//! This module deliberately stops at a typed protocol boundary. It has no
//! socket, HTTP, model-loading, or inference implementation. P0-L6 may later
//! provide a loopback transport that implements `LlamaCppTransport`.

use crate::conformance::{
    ConformanceCase, ConformanceCaseResult, ConformanceRequest, ConformanceRunner,
    RuntimeProfileReference,
};
use crate::error::BenchmarkError;
use crate::hashing::hash_text;
use prefixity_core::observation::{
    ArtifactReference, CacheObservation, ContextIdentity, Observed, ResourceUsage, RuntimeIdentity,
    TimingObservation, TokenAccounting, TokenCount, CACHE_OBSERVATION_SCHEMA_VERSION,
};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::{BTreeMap, VecDeque};

pub const LLAMA_CPP_PROTOCOL_ID: &str = "llama.cpp-openai-chat-v1";
pub const LLAMA_CPP_ADAPTER_VERSION: &str = "1";
const MAX_ERROR_BYTES: usize = 512;

/// A llama-server chat-completions request with intentional message/tool order.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LlamaCppRequest {
    pub model: String,
    pub messages: Vec<LlamaCppMessage>,
    pub tools: Vec<LlamaCppTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_format: Option<LlamaCppResponseFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LlamaCppMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LlamaCppTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: LlamaCppFunction,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LlamaCppFunction {
    pub name: String,
    pub description: String,
    pub parameters: LlamaCppJsonObject,
}

/// An ordered object used by the request projection. Its fields serialize as
/// a normal JSON object while retaining the neutral request's intentional
/// order for structural comparison and tests.
#[derive(Debug, Clone, PartialEq)]
pub struct LlamaCppJsonObject {
    pub fields: Vec<(String, Value)>,
}

impl Serialize for LlamaCppJsonObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = serializer.serialize_map(Some(self.fields.len()))?;
        for (name, value) in &self.fields {
            object.serialize_entry(name, value)?;
        }
        object.end()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlamaCppResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: LlamaCppJsonObject },
}

/// Native llama-server timing fields used by this adapter. Other response
/// fields are intentionally ignored by normalization and retained only when
/// they are part of the bounded raw telemetry map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlamaCppTimings {
    #[serde(default)]
    pub cache_n: Option<f64>,
    #[serde(default)]
    pub prompt_n: Option<f64>,
    #[serde(default)]
    pub prompt_ms: Option<f64>,
    #[serde(default)]
    pub predicted_n: Option<f64>,
    #[serde(default)]
    pub predicted_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlamaCppPromptTokenDetails {
    #[serde(default)]
    pub cached_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LlamaCppUsage {
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    #[serde(default)]
    pub completion_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub prompt_tokens_details: Option<LlamaCppPromptTokenDetails>,
}

/// Parsed response data plus bounded native values retained for debugging.
#[derive(Debug, Clone, PartialEq)]
pub struct LlamaCppResponse {
    pub timings: Option<LlamaCppTimings>,
    pub usage: Option<LlamaCppUsage>,
    pub raw_telemetry: BTreeMap<String, Value>,
}

impl LlamaCppResponse {
    /// Parse only the documented response fields needed by P0-L5. Unknown
    /// response fields, including generated choices, are not retained.
    pub fn from_json(value: Value) -> Result<Self, BenchmarkError> {
        let wire: LlamaCppWireResponse = serde_json::from_value(value.clone())
            .map_err(|error| validation(format!("malformed llama.cpp response: {error}")))?;
        let mut raw_telemetry = BTreeMap::new();
        if let Some(timings) = value.get("timings") {
            raw_telemetry.insert("llama_cpp_timings".to_string(), timings.clone());
        }
        if let Some(usage) = value.get("usage") {
            raw_telemetry.insert("llama_cpp_usage".to_string(), usage.clone());
        }
        Ok(Self {
            timings: wire.timings,
            usage: wire.usage,
            raw_telemetry,
        })
    }
}

#[derive(Debug, Deserialize)]
struct LlamaCppWireResponse {
    #[serde(default)]
    timings: Option<LlamaCppTimings>,
    #[serde(default)]
    usage: Option<LlamaCppUsage>,
}

/// Project a neutral request without changing represented content or order.
pub fn project_llama_cpp_request(
    request: &ConformanceRequest,
) -> Result<LlamaCppRequest, BenchmarkError> {
    request.validate()?;
    if request.envelope.reasoning.is_some() {
        return Err(validation(
            "llama.cpp does not have a faithful neutral reasoning-setting projection",
        ));
    }

    let mut messages = Vec::with_capacity(2 + request.context.artifacts.len());
    messages.push(LlamaCppMessage {
        role: "system".to_string(),
        content: request.context.system_instruction.clone(),
    });
    for artifact in &request.context.artifacts {
        messages.push(LlamaCppMessage {
            role: "user".to_string(),
            content: artifact.content.clone(),
        });
    }
    messages.push(LlamaCppMessage {
        role: "user".to_string(),
        content: request.context.user_content.clone(),
    });

    let tools = request
        .context
        .tools
        .iter()
        .map(|tool| LlamaCppTool {
            tool_type: "function".to_string(),
            function: LlamaCppFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: LlamaCppJsonObject {
                    fields: tool
                        .parameters
                        .fields
                        .iter()
                        .map(|field| (field.name.clone(), field.value.clone()))
                        .collect(),
                },
            },
        })
        .collect();

    let response_format = request
        .envelope
        .response_format
        .as_ref()
        .map(|format| match format {
            crate::conformance::ResponseFormat::Text => LlamaCppResponseFormat::Text,
            crate::conformance::ResponseFormat::JsonObject => LlamaCppResponseFormat::JsonObject,
            crate::conformance::ResponseFormat::JsonSchema(schema) => {
                LlamaCppResponseFormat::JsonSchema {
                    json_schema: LlamaCppJsonObject {
                        fields: schema
                            .fields
                            .iter()
                            .map(|field| (field.name.clone(), field.value.clone()))
                            .collect(),
                    },
                }
            }
        });

    Ok(LlamaCppRequest {
        model: request.envelope.model.clone(),
        messages,
        tools,
        response_format,
    })
}

/// Normalize a parsed llama-server response into the existing P0-L2 contract.
pub fn normalize_llama_cpp_response(
    response: &LlamaCppResponse,
    request: &ConformanceRequest,
    request_fingerprint: String,
    observation_id: String,
    observed_at: String,
    runtime: RuntimeIdentity,
) -> Result<CacheObservation, BenchmarkError> {
    let timings = response.timings.as_ref();
    let usage = response.usage.as_ref();
    let native_cached = timings
        .and_then(|timing| timing.cache_n)
        .map(|value| exact_count("timings.cache_n", value))
        .transpose()?;
    let compatibility_cached = usage
        .and_then(|usage| usage.prompt_tokens_details.as_ref())
        .and_then(|details| details.cached_tokens);
    ensure_agree("cached prompt tokens", native_cached, compatibility_cached)?;
    let cached = native_cached.or(compatibility_cached);

    let fresh = timings
        .and_then(|timing| timing.prompt_n)
        .map(|value| exact_count("timings.prompt_n", value))
        .transpose()?;
    if let (Some(total), Some(cached), Some(fresh)) =
        (usage.and_then(|usage| usage.prompt_tokens), cached, fresh)
    {
        let derived_total = cached
            .checked_add(fresh)
            .ok_or_else(|| validation("llama.cpp prompt token accounting overflow"))?;
        if total != derived_total {
            return Err(validation(
                "conflicting llama.cpp prompt token totals between timings and usage",
            ));
        }
    }

    let native_output = timings
        .and_then(|timing| timing.predicted_n)
        .map(|value| exact_count("timings.predicted_n", value))
        .transpose()?;
    let compatibility_output = usage.and_then(|usage| usage.completion_tokens);
    ensure_agree("output tokens", native_output, compatibility_output)?;
    let output = native_output.or(compatibility_output);

    let accounting = TokenAccounting {
        transmitted_input_tokens: usage
            .and_then(|usage| usage.prompt_tokens)
            .map(|value| Observed::Known(scoped_count(value, &runtime)))
            .unwrap_or(Observed::NotObserved),
        provider_cached_tokens: cached
            .map(|value| Observed::Known(scoped_count(value, &runtime)))
            .unwrap_or(Observed::NotObserved),
        fresh_prefill_tokens: fresh
            .map(|value| Observed::Known(scoped_count(value, &runtime)))
            .unwrap_or(Observed::NotObserved),
        output_tokens: output
            .map(|value| Observed::Known(scoped_count(value, &runtime)))
            .unwrap_or(Observed::NotObserved),
        ..TokenAccounting::default()
    };

    let mut timing = TimingObservation::default();
    timing.prefill_duration_ms = timings
        .and_then(|timing| timing.prompt_ms)
        .map(|value| duration_ms("timings.prompt_ms", value))
        .transpose()?
        .map(Observed::Known)
        .unwrap_or(Observed::NotObserved);
    timing.generation_duration_ms = timings
        .and_then(|timing| timing.predicted_ms)
        .map(|value| duration_ms("timings.predicted_ms", value))
        .transpose()?
        .map(Observed::Known)
        .unwrap_or(Observed::NotObserved);

    let mut raw_telemetry = response.raw_telemetry.clone();
    raw_telemetry.insert(
        "adapter".to_string(),
        Value::String(LLAMA_CPP_PROTOCOL_ID.to_string()),
    );
    let observation = CacheObservation {
        schema_version: CACHE_OBSERVATION_SCHEMA_VERSION,
        observation_id,
        observed_at,
        runtime,
        context: ContextIdentity {
            artifacts: artifact_references(request),
            serialized_request_identity: Observed::Known(request_fingerprint),
            reusable_prefix_identity: Observed::Unknown,
        },
        accounting,
        timing,
        resources: ResourceUsage::default(),
        cache: prefixity_core::observation::CacheBehavior::default(),
        outcome: prefixity_core::observation::ObservationOutcome::default(),
        raw_telemetry,
    };
    observation.validate().map_err(|error| {
        validation(format!(
            "normalized llama.cpp observation is invalid: {error}"
        ))
    })?;
    Ok(observation)
}

/// Narrow transport boundary for a future loopback llama-server transport.
pub trait LlamaCppTransport {
    fn chat_completion(
        &mut self,
        request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError>;
}

/// Deterministic fake transport used by P0-L5 tests only.
#[derive(Debug, Default)]
pub struct FakeLlamaCppTransport {
    responses: VecDeque<Result<LlamaCppResponse, String>>,
    requests: Vec<LlamaCppRequest>,
}

impl FakeLlamaCppTransport {
    pub fn new(responses: Vec<Result<LlamaCppResponse, String>>) -> Self {
        Self {
            responses: responses.into(),
            requests: Vec::new(),
        }
    }

    pub fn from_json(values: Vec<Result<Value, String>>) -> Result<Self, BenchmarkError> {
        let mut responses = Vec::with_capacity(values.len());
        for value in values {
            responses.push(match value {
                Ok(value) => Ok(LlamaCppResponse::from_json(value)?),
                Err(error) => Err(error),
            });
        }
        Ok(Self::new(responses))
    }

    pub fn requests(&self) -> &[LlamaCppRequest] {
        &self.requests
    }
}

impl LlamaCppTransport for FakeLlamaCppTransport {
    fn chat_completion(
        &mut self,
        request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        self.requests.push(request.clone());
        match self.responses.pop_front() {
            Some(Ok(response)) => Ok(response),
            Some(Err(error)) => Err(validation(format!(
                "llama.cpp fake transport failure: {}",
                bound_error(&error)
            ))),
            None => Err(validation(
                "llama.cpp fake transport response queue exhausted",
            )),
        }
    }
}

/// P0-L4 runner integration for a llama-server-shaped fake transport.
#[derive(Debug)]
pub struct LlamaCppConformanceRunner<T> {
    transport: T,
    observed_at: String,
    runtime: RuntimeIdentity,
}

impl<T> LlamaCppConformanceRunner<T> {
    pub fn new(transport: T, observed_at: impl Into<String>, runtime: RuntimeIdentity) -> Self {
        Self {
            transport,
            observed_at: observed_at.into(),
            runtime,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: LlamaCppTransport> ConformanceRunner for LlamaCppConformanceRunner<T> {
    fn execute(
        &mut self,
        experiment_id: &str,
        runtime_profile: &RuntimeProfileReference,
        case: &ConformanceCase,
    ) -> Result<ConformanceCaseResult, BenchmarkError> {
        if runtime_profile.identity != self.runtime {
            return Err(case_error(
                experiment_id,
                case,
                "runner runtime does not match experiment profile",
            ));
        }
        let request = project_llama_cpp_request(&case.request).map_err(|error| {
            case_error(
                experiment_id,
                case,
                format!("request is not representable: {error}"),
            )
        })?;
        let request_fingerprint = case.request.request_fingerprint()?;
        let context_fingerprint = case.request.context_fingerprint()?;
        let response = self.transport.chat_completion(&request).map_err(|error| {
            case_error(experiment_id, case, format!("transport failed: {error}"))
        })?;
        let observation_id = format!(
            "observation-{}",
            &hash_text(&format!("{experiment_id}:{}", case.case_id))[..24]
        );
        let observation = normalize_llama_cpp_response(
            &response,
            &case.request,
            request_fingerprint.clone(),
            observation_id,
            self.observed_at.clone(),
            self.runtime.clone(),
        )
        .map_err(|error| {
            case_error(
                experiment_id,
                case,
                format!("response normalization failed: {error}"),
            )
        })?;
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

    fn provenance(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("runner".to_string(), "llama.cpp-adapter".to_string()),
            (
                "evidence".to_string(),
                "synthetic-protocol-validation-only".to_string(),
            ),
        ])
    }
}

fn artifact_references(request: &ConformanceRequest) -> Vec<ArtifactReference> {
    request
        .context
        .artifacts
        .iter()
        .map(|artifact| ArtifactReference {
            artifact_id: artifact.artifact_id.clone(),
            content_hash: Observed::Known(prefixity_core::hash::hash_content(&artifact.content)),
        })
        .collect()
}

fn scoped_count(count: u64, runtime: &RuntimeIdentity) -> TokenCount {
    TokenCount {
        count,
        provider: runtime.provider.clone(),
        model: runtime.model.clone(),
        tokenizer: Observed::Unknown,
    }
}

fn exact_count(field: &str, value: f64) -> Result<u64, BenchmarkError> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > u64::MAX as f64 {
        return Err(validation(format!(
            "{field} must be a non-negative integer"
        )));
    }
    Ok(value as u64)
}

fn duration_ms(field: &str, value: f64) -> Result<u64, BenchmarkError> {
    if !value.is_finite() || value < 0.0 || value > u64::MAX as f64 {
        return Err(validation(format!(
            "{field} must be a finite non-negative number"
        )));
    }
    Ok(value.round() as u64)
}

fn ensure_agree(
    field: &str,
    native: Option<u64>,
    compatibility: Option<u64>,
) -> Result<(), BenchmarkError> {
    if let (Some(native), Some(compatibility)) = (native, compatibility) {
        if native != compatibility {
            return Err(validation(format!(
                "conflicting llama.cpp {field}: native={native}, usage={compatibility}"
            )));
        }
    }
    Ok(())
}

fn case_error(
    experiment_id: &str,
    case: &ConformanceCase,
    message: impl Into<String>,
) -> BenchmarkError {
    validation(format!(
        "llama.cpp case failure experiment={experiment_id} case={} mutation={:?}: {}",
        case.case_id,
        case.mutation,
        bound_error(&message.into())
    ))
}

fn bound_error(value: &str) -> String {
    value.chars().take(MAX_ERROR_BYTES).collect()
}

fn validation(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::validation(message)
}
