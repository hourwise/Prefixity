//! Isolated Phase 1C Stage 1 local-Qwen schema-smoke adapter.
//!
//! This module is intentionally separate from `prefixity-live` and from all
//! Prefixity planning/candidate behavior. It sends one ordinary
//! OpenAI-compatible llama.cpp request only when the caller supplies an
//! explicit fresh-runtime confirmation flag. The offline preflight performs
//! no socket, credential, model, or provider work.

use crate::live_harness::LoopbackEndpoint;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const STAGE1_CONTRACT_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT.json";
pub const STAGE1_FIXTURE_PATH: &str = "fixtures/phase1c/phase1c-stage1-schema-smoke-01.json";
pub const STAGE1_REQUEST_SCHEMA_PATH: &str =
    "docs/phase-1/schemas/phase1c-stage1-local-qwen-request-v1.schema.json";
pub const STAGE1_OUTPUT_SCHEMA_PATH: &str =
    "docs/phase-1/schemas/phase1c-stage1-smoke-v1.schema.json";
pub const STAGE1_EVIDENCE_DIR: &str = "experiments/runs/phase1c-stage1-schema-smoke-01";

const CONTRACT_VERSION: &str = "phase1c-stage1-local-qwen-runtime-v1";
const REQUEST_SCHEMA_ID: &str = "prefixity.phase1c.stage1.local-qwen-request-v1";
const OUTPUT_SCHEMA_ID: &str = "prefixity.phase1c.stage1.smoke-v1";
const EVIDENCE_SCHEMA_ID: &str = "prefixity.phase1c.stage1.local-qwen-evidence";
const EVIDENCE_SCHEMA_VERSION: u32 = 1;
const MODEL_ID: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
const ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_INPUT_ESTIMATED_TOKENS: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum Stage1Error {
    #[error("stage1 file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("stage1 JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("stage1 validation failed: {0}")]
    Validation(String),
    #[error("stage1 live transport failed: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeContract {
    contract_version: String,
    phase: String,
    purpose: String,
    engine: String,
    server_executable_family: Vec<String>,
    api_surface: String,
    endpoint: String,
    host: String,
    port: u16,
    model: String,
    quantization: String,
    context_size: u64,
    parallel_slots: u32,
    metrics: String,
    credential_policy: CredentialPolicy,
    generation: GenerationContract,
    timeout_policy: TimeoutPolicy,
    retry_policy: RetryPolicy,
    request_ceiling: u32,
    input_schema_id: String,
    output_schema_id: String,
    freshness: FreshnessContract,
    contamination_rules: Vec<String>,
    permitted_claims: Vec<String>,
    prohibited_claims: Vec<String>,
    candidate_source_commit: String,
    design_commit: String,
    evidence_location: String,
    fingerprint: FingerprintContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CredentialPolicy {
    required: bool,
    authorization_header: String,
    account: String,
    region: String,
    pricing_spend_profile: String,
    provider_api_key_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenerationContract {
    temperature: f64,
    top_p: f64,
    max_tokens: u32,
    stream: bool,
    seed: SeedContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SeedContract {
    value: Option<u64>,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeoutPolicy {
    connect_timeout_ms: u64,
    complete_request_timeout_ms: u64,
    supervisor_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetryPolicy {
    automatic_retries: u32,
    fallback_requests: u32,
    adaptive_replicates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FreshnessContract {
    required: bool,
    zero_inference_requests_since_startup: bool,
    maximum_non_inference_listener_checks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FingerprintContract {
    algorithm: String,
    canonicalization: String,
    recorded_in: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SmokeFixture {
    fixture_id: String,
    fixture_version: u32,
    phase: String,
    purpose: String,
    scored: bool,
    scored_task_ids: Vec<String>,
    contains_evaluator_answer_material: bool,
    messages: Vec<FixtureMessage>,
    expected_payload: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
struct SmokeRequest {
    model: String,
    messages: Vec<FixtureMessage>,
    temperature: f64,
    top_p: f64,
    max_tokens: u32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stage1Preflight {
    pub state: String,
    pub experiment_id: String,
    pub contract_sha256: String,
    pub fixture_sha256: String,
    pub request_schema_sha256: String,
    pub output_schema_sha256: String,
    pub request_projection_sha256: String,
    pub wire_request_sha256: String,
    pub model: String,
    pub endpoint: String,
    pub input_estimated_tokens: usize,
    pub request_ceiling: u32,
    pub automatic_retries: u32,
    pub evidence_location: String,
    pub scored_task_ids: Vec<String>,
    pub network_calls: u32,
    pub inference_requests: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stage1RunRecord {
    pub schema_id: String,
    pub schema_version: u32,
    pub experiment_id: String,
    pub purpose: String,
    pub state: String,
    pub runtime_configuration_identity: String,
    pub fixture_id: String,
    pub fixture_sha256: String,
    pub request: RequestAccounting,
    pub response: ResponseAccounting,
    pub normalized_payload: Option<Value>,
    pub validation: ValidationRecord,
    pub lineage: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestAccounting {
    pub authorized_ceiling: u32,
    pub transport_attempts: u32,
    pub inference_requests: u32,
    pub automatic_retries: u32,
    pub request_fingerprint: String,
    pub request_file: String,
    pub input_estimated_tokens: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseAccounting {
    pub complete: bool,
    pub http_status: Option<u16>,
    pub response_body_bytes: Option<usize>,
    pub response_body_sha256: Option<String>,
    pub response_body_file: Option<String>,
    pub safe_headers: BTreeMap<String, String>,
    pub assistant_content: Option<String>,
    pub usage: Option<Value>,
    pub timings: Option<Value>,
    pub transport_elapsed_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationRecord {
    pub response_json_parsed: bool,
    pub assistant_content_extracted: bool,
    pub semantic_payload_parsed: bool,
    pub required_fields_validated: bool,
    pub normalized_under_schema: bool,
    pub no_scored_task_consumed: bool,
    pub error: Option<String>,
}

pub fn preflight_stage1_smoke() -> Result<Stage1Preflight, Stage1Error> {
    let contract_bytes = fs::read(STAGE1_CONTRACT_PATH)?;
    let fixture_bytes = fs::read(STAGE1_FIXTURE_PATH)?;
    let request_schema_bytes = fs::read(STAGE1_REQUEST_SCHEMA_PATH)?;
    let output_schema_bytes = fs::read(STAGE1_OUTPUT_SCHEMA_PATH)?;

    let contract_value: Value = serde_json::from_slice(&contract_bytes)?;
    let fixture_value: Value = serde_json::from_slice(&fixture_bytes)?;
    let request_schema: Value = serde_json::from_slice(&request_schema_bytes)?;
    let output_schema: Value = serde_json::from_slice(&output_schema_bytes)?;
    let contract: RuntimeContract = serde_json::from_value(contract_value.clone())?;
    let fixture: SmokeFixture = serde_json::from_value(fixture_value.clone())?;

    validate_contract(&contract)?;
    validate_fixture(&fixture)?;
    validate_schemas(&request_schema, &output_schema)?;

    let request = build_request(&contract, &fixture)?;
    let request_value = serde_json::to_value(&request)?;
    validate_request(&request_value, &contract, &fixture)?;
    let wire_request = serde_json::to_vec(&request)?;
    let input_estimated_tokens = estimate_input_tokens(&request_value)?;
    if input_estimated_tokens > MAX_INPUT_ESTIMATED_TOKENS {
        return Err(Stage1Error::Validation(format!(
            "serialized smoke input estimate {input_estimated_tokens} exceeds {MAX_INPUT_ESTIMATED_TOKENS}"
        )));
    }

    if Path::new(STAGE1_EVIDENCE_DIR).exists() {
        return Err(Stage1Error::Validation(format!(
            "fresh evidence location already exists: {STAGE1_EVIDENCE_DIR}"
        )));
    }

    Ok(Stage1Preflight {
        state: "PREPARED".to_string(),
        experiment_id: fixture.fixture_id,
        contract_sha256: fingerprint_value(&contract_value)?,
        fixture_sha256: fingerprint_value(&fixture_value)?,
        request_schema_sha256: fingerprint_value(&request_schema)?,
        output_schema_sha256: fingerprint_value(&output_schema)?,
        request_projection_sha256: fingerprint_value(&request_value)?,
        wire_request_sha256: sha256_hex(&wire_request),
        model: contract.model,
        endpoint: contract.endpoint,
        input_estimated_tokens,
        request_ceiling: contract.request_ceiling,
        automatic_retries: contract.retry_policy.automatic_retries,
        evidence_location: STAGE1_EVIDENCE_DIR.to_string(),
        scored_task_ids: fixture.scored_task_ids,
        network_calls: 0,
        inference_requests: 0,
    })
}

pub fn execute_stage1_smoke(confirm_fresh_runtime: bool) -> Result<Stage1RunRecord, Stage1Error> {
    if !confirm_fresh_runtime {
        return Err(Stage1Error::Validation(
            "explicit fresh-runtime confirmation is required before the listener check".to_string(),
        ));
    }

    let preflight = preflight_stage1_smoke()?;
    let contract_value: Value = serde_json::from_slice(&fs::read(STAGE1_CONTRACT_PATH)?)?;
    let fixture_value: Value = serde_json::from_slice(&fs::read(STAGE1_FIXTURE_PATH)?)?;
    let contract: RuntimeContract = serde_json::from_value(contract_value)?;
    let fixture: SmokeFixture = serde_json::from_value(fixture_value)?;
    let request = build_request(&contract, &fixture)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let request_fingerprint = sha256_hex(&request_bytes);

    let endpoint = LoopbackEndpoint::parse(contract.endpoint.clone())
        .map_err(|error| Stage1Error::Validation(error.to_string()))?;
    let address = SocketAddr::from(([127, 0, 0, 1], contract.port));
    let readiness_started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &address,
        Duration::from_millis(contract.timeout_policy.connect_timeout_ms),
    );
    let readiness_elapsed_ms = readiness_started.elapsed().as_millis() as u64;
    if let Err(error) = readiness {
        return Err(Stage1Error::Transport(format!(
            "single non-inference listener check failed after {readiness_elapsed_ms}ms: {error}"
        )));
    }

    let evidence_dir = PathBuf::from(STAGE1_EVIDENCE_DIR);
    fs::create_dir(&evidence_dir)?;
    write_json(&evidence_dir.join("preflight.json"), &preflight)?;
    write_bytes(&evidence_dir.join("request.json"), &request_bytes)?;
    write_json(
        &evidence_dir.join("readiness.json"),
        &serde_json::json!({
            "check": "tcp_listener_connect",
            "host": contract.host,
            "port": contract.port,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
        }),
    )?;

    let client = Client::builder()
        .connect_timeout(Duration::from_millis(
            contract.timeout_policy.connect_timeout_ms,
        ))
        .timeout(Duration::from_millis(
            contract.timeout_policy.complete_request_timeout_ms,
        ))
        .redirect(Policy::none())
        .build()
        .map_err(|error| Stage1Error::Transport(error.to_string()))?;

    let request_started = Instant::now();
    let response = client
        .post(&endpoint.url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_bytes.clone())
        .send();
    let transport_elapsed_ms = request_started.elapsed().as_millis() as u64;

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let record = failed_record(
                &preflight,
                request_fingerprint,
                request_bytes.len(),
                transport_elapsed_ms,
                format!("request dispatch/completion is ambiguous: {error}"),
            );
            write_json(&evidence_dir.join("stage1-result.json"), &record)?;
            return Ok(record);
        }
    };

    let status = response.status().as_u16();
    let safe_headers = safe_headers(response.headers());
    let mut response = response;
    let mut response_body = Vec::new();
    response
        .by_ref()
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response_body)?;
    let complete = response_body.len() <= MAX_RESPONSE_BYTES;
    let response_body_sha256 = sha256_hex(&response_body);
    write_bytes(&evidence_dir.join("response-body.bin"), &response_body)?;

    let parsed = if complete && (200..300).contains(&status) {
        serde_json::from_slice::<Value>(&response_body).ok()
    } else {
        None
    };
    let usage = parsed
        .as_ref()
        .and_then(|value| value.get("usage").cloned());
    let timings = parsed
        .as_ref()
        .and_then(|value| value.get("timings").cloned());
    let assistant_content = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/message/content"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let normalized_payload = assistant_content
        .as_deref()
        .and_then(|content| normalize_payload(content, &fixture.expected_payload).ok());
    let success = complete
        && (200..300).contains(&status)
        && parsed.is_some()
        && assistant_content.is_some()
        && normalized_payload.is_some();
    let error = if success {
        None
    } else {
        Some(if !complete {
            "response exceeded the bounded body size".to_string()
        } else if !(200..300).contains(&status) {
            format!("unexpected HTTP status {status}")
        } else if parsed.is_none() {
            "response body was not valid JSON".to_string()
        } else if assistant_content.is_none() {
            "choices[0].message.content was absent or not a string".to_string()
        } else {
            "assistant content failed the exact semantic smoke schema".to_string()
        })
    };

    let record = Stage1RunRecord {
        schema_id: EVIDENCE_SCHEMA_ID.to_string(),
        schema_version: EVIDENCE_SCHEMA_VERSION,
        experiment_id: preflight.experiment_id.clone(),
        purpose: "schema_smoke_only".to_string(),
        state: if success { "PASSED" } else { "FAILED" }.to_string(),
        runtime_configuration_identity: preflight.contract_sha256.clone(),
        fixture_id: preflight.experiment_id.clone(),
        fixture_sha256: preflight.fixture_sha256.clone(),
        request: RequestAccounting {
            authorized_ceiling: preflight.request_ceiling,
            transport_attempts: 1,
            inference_requests: 1,
            automatic_retries: 0,
            request_fingerprint,
            request_file: "request.json".to_string(),
            input_estimated_tokens: preflight.input_estimated_tokens,
        },
        response: ResponseAccounting {
            complete,
            http_status: Some(status),
            response_body_bytes: Some(response_body.len()),
            response_body_sha256: Some(response_body_sha256),
            response_body_file: Some("response-body.bin".to_string()),
            safe_headers,
            assistant_content,
            usage,
            timings,
            transport_elapsed_ms: Some(transport_elapsed_ms),
        },
        normalized_payload: normalized_payload.clone(),
        validation: ValidationRecord {
            response_json_parsed: parsed.is_some(),
            assistant_content_extracted: normalized_payload.is_some()
                || error
                    .as_deref()
                    .is_some_and(|message| !message.contains("content was absent")),
            semantic_payload_parsed: normalized_payload.is_some(),
            required_fields_validated: normalized_payload.is_some(),
            normalized_under_schema: normalized_payload.is_some(),
            no_scored_task_consumed: true,
            error,
        },
        lineage: BTreeMap::from([
            ("phase".to_string(), "phase-1c".to_string()),
            ("runtime_engine".to_string(), "llama.cpp".to_string()),
            ("api_surface".to_string(), contract.api_surface),
            (
                "candidate_source_commit".to_string(),
                "748e4673e8454d2ac3e27cefabee9259992038aa".to_string(),
            ),
            (
                "evidence_class".to_string(),
                "schema_pipeline_only".to_string(),
            ),
        ]),
    };
    write_json(&evidence_dir.join("stage1-result.json"), &record)?;
    Ok(record)
}

fn failed_record(
    preflight: &Stage1Preflight,
    request_fingerprint: String,
    request_bytes: usize,
    transport_elapsed_ms: u64,
    error: String,
) -> Stage1RunRecord {
    Stage1RunRecord {
        schema_id: EVIDENCE_SCHEMA_ID.to_string(),
        schema_version: EVIDENCE_SCHEMA_VERSION,
        experiment_id: preflight.experiment_id.clone(),
        purpose: "schema_smoke_only".to_string(),
        state: "AMBIGUOUS".to_string(),
        runtime_configuration_identity: preflight.contract_sha256.clone(),
        fixture_id: preflight.experiment_id.clone(),
        fixture_sha256: preflight.fixture_sha256.clone(),
        request: RequestAccounting {
            authorized_ceiling: preflight.request_ceiling,
            transport_attempts: 1,
            inference_requests: 1,
            automatic_retries: 0,
            request_fingerprint,
            request_file: "request.json".to_string(),
            input_estimated_tokens: preflight.input_estimated_tokens,
        },
        response: ResponseAccounting {
            complete: false,
            http_status: None,
            response_body_bytes: None,
            response_body_sha256: None,
            response_body_file: None,
            safe_headers: BTreeMap::new(),
            assistant_content: None,
            usage: None,
            timings: None,
            transport_elapsed_ms: Some(transport_elapsed_ms),
        },
        normalized_payload: None,
        validation: ValidationRecord {
            response_json_parsed: false,
            assistant_content_extracted: false,
            semantic_payload_parsed: false,
            required_fields_validated: false,
            normalized_under_schema: false,
            no_scored_task_consumed: true,
            error: Some(error),
        },
        lineage: BTreeMap::from([
            ("phase".to_string(), "phase-1c".to_string()),
            ("runtime_engine".to_string(), "llama.cpp".to_string()),
            (
                "evidence_class".to_string(),
                "schema_pipeline_only".to_string(),
            ),
            ("request_body_bytes".to_string(), request_bytes.to_string()),
        ]),
    }
}

fn validate_contract(contract: &RuntimeContract) -> Result<(), Stage1Error> {
    if contract.contract_version != CONTRACT_VERSION
        || contract.phase != "phase-1c"
        || contract.purpose != "schema_smoke_only"
        || contract.engine != "llama.cpp"
        || contract.api_surface != "llama.cpp-openai-compatible-chat-completions-v1"
        || contract.endpoint != ENDPOINT
        || contract.host != "127.0.0.1"
        || contract.port != 8080
        || contract.model != MODEL_ID
        || contract.quantization != "Q4_0"
        || contract.context_size != 8192
        || contract.parallel_slots != 1
        || contract.metrics != "enabled"
        || contract.input_schema_id != REQUEST_SCHEMA_ID
        || contract.output_schema_id != OUTPUT_SCHEMA_ID
        || contract.request_ceiling != 1
    {
        return Err(Stage1Error::Validation(
            "runtime contract does not match the authorized local-Qwen selection".to_string(),
        ));
    }
    if contract.credential_policy.required
        || contract.credential_policy.authorization_header != "none"
        || contract.credential_policy.provider_api_key_read
        || contract.generation.temperature != 0.0
        || contract.generation.top_p != 1.0
        || contract.generation.max_tokens != 256
        || contract.generation.stream
        || contract.generation.seed.value != Some(1)
        || contract.generation.seed.status != "applied"
        || contract.timeout_policy.connect_timeout_ms != 1000
        || contract.timeout_policy.complete_request_timeout_ms != 600_000
        || contract.timeout_policy.supervisor_timeout_ms
            <= contract.timeout_policy.complete_request_timeout_ms
        || contract.retry_policy.automatic_retries != 0
        || contract.retry_policy.fallback_requests != 0
        || contract.retry_policy.adaptive_replicates != 0
        || !contract.freshness.required
        || !contract.freshness.zero_inference_requests_since_startup
        || contract.freshness.maximum_non_inference_listener_checks != 1
        || contract.candidate_source_commit != "748e4673e8454d2ac3e27cefabee9259992038aa"
        || contract.design_commit != "36f960579e55c7ad48e54e4cb2670cc55cd1ef3e"
        || contract.evidence_location != STAGE1_EVIDENCE_DIR
    {
        return Err(Stage1Error::Validation(
            "runtime contract violates a safety, generation, identity, timeout, retry, or freshness bound".to_string(),
        ));
    }
    if contract.server_executable_family.is_empty()
        || contract.contamination_rules.is_empty()
        || contract.permitted_claims.is_empty()
        || contract.prohibited_claims.is_empty()
        || contract.fingerprint.algorithm != "SHA-256"
    {
        return Err(Stage1Error::Validation(
            "runtime contract is missing required lineage, claim, contamination, or fingerprint fields".to_string(),
        ));
    }
    Ok(())
}

fn validate_fixture(fixture: &SmokeFixture) -> Result<(), Stage1Error> {
    if fixture.fixture_id != "phase-1c-stage1-schema-smoke-01"
        || fixture.fixture_version != 1
        || fixture.phase != "phase-1c"
        || fixture.purpose != "schema_smoke_only"
        || fixture.scored
        || !fixture.scored_task_ids.is_empty()
        || fixture.contains_evaluator_answer_material
        || fixture.messages.len() != 2
        || fixture.messages[0].role != "system"
        || fixture.messages[1].role != "user"
    {
        return Err(Stage1Error::Validation(
            "smoke fixture is not a dedicated non-scored two-message fixture".to_string(),
        ));
    }
    let expected = BTreeMap::from([
        (
            "schema_version".to_string(),
            "phase1c-stage1-smoke-v1".to_string(),
        ),
        ("status".to_string(), "ok".to_string()),
        ("marker".to_string(), "PREFIXITY_PHASE1C_STAGE1".to_string()),
    ]);
    if fixture.expected_payload != expected {
        return Err(Stage1Error::Validation(
            "smoke fixture expected payload differs from the authorized values".to_string(),
        ));
    }
    let forbidden = [
        "h001", "h002", "h003", "h004", "h005", "h006", "h007", "h008", "h009", "h010", "h011",
        "h012",
    ];
    if fixture
        .messages
        .iter()
        .any(|message| forbidden.iter().any(|id| message.content.contains(id)))
    {
        return Err(Stage1Error::Validation(
            "smoke fixture contains a scored-cohort task identifier".to_string(),
        ));
    }
    Ok(())
}

fn validate_schemas(request: &Value, output: &Value) -> Result<(), Stage1Error> {
    if request.get("$id") != Some(&Value::String(REQUEST_SCHEMA_ID.to_string()))
        || output.get("$id") != Some(&Value::String(OUTPUT_SCHEMA_ID.to_string()))
        || output.get("additionalProperties") != Some(&Value::Bool(false))
    {
        return Err(Stage1Error::Validation(
            "tracked Stage 1 request/output schema identities are not canonical".to_string(),
        ));
    }
    Ok(())
}

fn build_request(
    contract: &RuntimeContract,
    fixture: &SmokeFixture,
) -> Result<SmokeRequest, Stage1Error> {
    if contract.generation.seed.status != "applied" {
        return Err(Stage1Error::Validation(
            "the authorized installed runtime contract must apply seed=1".to_string(),
        ));
    }
    Ok(SmokeRequest {
        model: contract.model.clone(),
        messages: fixture.messages.clone(),
        temperature: contract.generation.temperature,
        top_p: contract.generation.top_p,
        max_tokens: contract.generation.max_tokens,
        stream: contract.generation.stream,
        seed: contract.generation.seed.value,
    })
}

fn validate_request(
    request: &Value,
    contract: &RuntimeContract,
    fixture: &SmokeFixture,
) -> Result<(), Stage1Error> {
    let object = request
        .as_object()
        .ok_or_else(|| Stage1Error::Validation("projected request is not an object".to_string()))?;
    let expected_keys: BTreeSet<&str> = [
        "model",
        "messages",
        "temperature",
        "top_p",
        "max_tokens",
        "stream",
        "seed",
    ]
    .into_iter()
    .collect();
    let actual_keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    if actual_keys != expected_keys
        || object.get("model") != Some(&Value::String(contract.model.clone()))
        || object.get("temperature") != Some(&serde_json::json!(0.0))
        || object.get("top_p") != Some(&serde_json::json!(1.0))
        || object.get("max_tokens") != Some(&serde_json::json!(256))
        || object.get("stream") != Some(&Value::Bool(false))
        || object.get("seed") != Some(&serde_json::json!(1))
        || object.get("messages") != Some(&serde_json::to_value(&fixture.messages)?)
    {
        return Err(Stage1Error::Validation(
            "projected request does not match the exact Stage 1 request contract".to_string(),
        ));
    }
    Ok(())
}

fn estimate_input_tokens(request: &Value) -> Result<usize, Stage1Error> {
    let bytes = serde_json::to_vec(request)?.len();
    Ok(bytes.saturating_add(3) / 4)
}

fn normalize_payload(
    content: &str,
    expected: &BTreeMap<String, String>,
) -> Result<Value, Stage1Error> {
    let value: Value = serde_json::from_str(content.trim())?;
    let object = value.as_object().ok_or_else(|| {
        Stage1Error::Validation("assistant content is not a JSON object".to_string())
    })?;
    if object.len() != expected.len()
        || object
            .iter()
            .any(|(key, value)| expected.get(key) != value.as_str().map(str::to_string).as_ref())
    {
        return Err(Stage1Error::Validation(
            "assistant JSON does not contain exactly the required semantic fields".to_string(),
        ));
    }
    Ok(Value::Object(object.clone()))
}

fn safe_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    [
        "content-type",
        "content-length",
        "x-request-id",
        "request-id",
    ]
    .into_iter()
    .filter_map(|name| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| (name.to_string(), value.to_string()))
    })
    .collect()
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), Stage1Error> {
    write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), Stage1Error> {
    fs::write(path, bytes).map_err(Stage1Error::Io)
}

fn fingerprint_value(value: &Value) -> Result<String, Stage1Error> {
    Ok(sha256_hex(&canonical_json(value)?))
}

fn canonical_json(value: &Value) -> Result<Vec<u8>, Stage1Error> {
    let canonical = canonical_value(value.clone());
    Ok(serde_json::to_vec(&canonical)?)
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<String, Value> = object
                .into_iter()
                .map(|(key, value)| (key, canonical_value(value)))
                .collect();
            let mut canonical = Map::new();
            for (key, value) in sorted {
                canonical.insert(key, value);
            }
            Value::Object(canonical)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonical_value).collect()),
        scalar => scalar,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> SmokeFixture {
        SmokeFixture {
            fixture_id: "phase-1c-stage1-schema-smoke-01".to_string(),
            fixture_version: 1,
            phase: "phase-1c".to_string(),
            purpose: "schema_smoke_only".to_string(),
            scored: false,
            scored_task_ids: Vec::new(),
            contains_evaluator_answer_material: false,
            messages: vec![
                FixtureMessage {
                    role: "system".to_string(),
                    content: "system".to_string(),
                },
                FixtureMessage {
                    role: "user".to_string(),
                    content: "user".to_string(),
                },
            ],
            expected_payload: BTreeMap::from([
                (
                    "schema_version".to_string(),
                    "phase1c-stage1-smoke-v1".to_string(),
                ),
                ("status".to_string(), "ok".to_string()),
                ("marker".to_string(), "PREFIXITY_PHASE1C_STAGE1".to_string()),
            ]),
        }
    }

    fn contract() -> RuntimeContract {
        RuntimeContract {
            contract_version: CONTRACT_VERSION.to_string(),
            phase: "phase-1c".to_string(),
            purpose: "schema_smoke_only".to_string(),
            engine: "llama.cpp".to_string(),
            server_executable_family: vec!["llama".to_string(), "llama-server".to_string()],
            api_surface: "llama.cpp-openai-compatible-chat-completions-v1".to_string(),
            endpoint: ENDPOINT.to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            model: MODEL_ID.to_string(),
            quantization: "Q4_0".to_string(),
            context_size: 8192,
            parallel_slots: 1,
            metrics: "enabled".to_string(),
            credential_policy: CredentialPolicy {
                required: false,
                authorization_header: "none".to_string(),
                account: "not applicable".to_string(),
                region: "not applicable".to_string(),
                pricing_spend_profile: "local runtime — no provider spend".to_string(),
                provider_api_key_read: false,
            },
            generation: GenerationContract {
                temperature: 0.0,
                top_p: 1.0,
                max_tokens: 256,
                stream: false,
                seed: SeedContract {
                    value: Some(1),
                    status: "applied".to_string(),
                },
            },
            timeout_policy: TimeoutPolicy {
                connect_timeout_ms: 1000,
                complete_request_timeout_ms: 600_000,
                supervisor_timeout_ms: 660_000,
            },
            retry_policy: RetryPolicy {
                automatic_retries: 0,
                fallback_requests: 0,
                adaptive_replicates: 0,
            },
            request_ceiling: 1,
            input_schema_id: REQUEST_SCHEMA_ID.to_string(),
            output_schema_id: OUTPUT_SCHEMA_ID.to_string(),
            freshness: FreshnessContract {
                required: true,
                zero_inference_requests_since_startup: true,
                maximum_non_inference_listener_checks: 1,
            },
            contamination_rules: vec!["no scored cohort material".to_string()],
            permitted_claims: vec!["schema pipeline validated".to_string()],
            prohibited_claims: vec!["capability improvement".to_string()],
            candidate_source_commit: "748e4673e8454d2ac3e27cefabee9259992038aa".to_string(),
            design_commit: "36f960579e55c7ad48e54e4cb2670cc55cd1ef3e".to_string(),
            evidence_location: STAGE1_EVIDENCE_DIR.to_string(),
            fingerprint: FingerprintContract {
                algorithm: "SHA-256".to_string(),
                canonicalization: "sorted JSON object keys; arrays preserve order".to_string(),
                recorded_in: "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT.sha256"
                    .to_string(),
            },
        }
    }

    #[test]
    fn request_projection_has_exact_generation_fields_and_seed() {
        let request = build_request(&contract(), &fixture()).unwrap();
        let value = serde_json::to_value(request).unwrap();
        let keys: BTreeSet<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let expected: BTreeSet<&str> = [
            "model",
            "messages",
            "temperature",
            "top_p",
            "max_tokens",
            "stream",
            "seed",
        ]
        .into_iter()
        .collect();
        assert_eq!(keys, expected);
        assert_eq!(value["seed"], serde_json::json!(1));
        assert_eq!(value["max_tokens"], serde_json::json!(256));
    }

    #[test]
    fn payload_validator_rejects_fences_prose_and_extra_fields() {
        let expected = fixture().expected_payload;
        let good = serde_json::to_string(&expected).unwrap();
        assert!(normalize_payload(&good, &expected).is_ok());
        assert!(normalize_payload(&format!("```json\n{good}\n```"), &expected).is_err());
        assert!(normalize_payload(&format!("answer: {good}"), &expected).is_err());
        assert!(normalize_payload(
            r#"{"schema_version":"phase1c-stage1-smoke-v1","status":"ok","marker":"PREFIXITY_PHASE1C_STAGE1","extra":"x"}"#,
            &expected
        )
        .is_err());
    }

    #[test]
    fn contract_validation_keeps_local_no_credential_and_zero_retry_boundary() {
        let value = contract();
        validate_contract(&value).unwrap();
        let mut invalid = value;
        invalid.retry_policy.automatic_retries = 1;
        assert!(validate_contract(&invalid).is_err());
    }

    #[test]
    fn canonical_fingerprint_is_key_order_independent_and_array_ordered() {
        let a: Value = serde_json::json!({"b":1,"a":["x","y"]});
        let b: Value = serde_json::json!({"a":["x","y"],"b":1});
        let c: Value = serde_json::json!({"a":["y","x"],"b":1});
        assert_eq!(
            fingerprint_value(&a).unwrap(),
            fingerprint_value(&b).unwrap()
        );
        assert_ne!(
            fingerprint_value(&a).unwrap(),
            fingerprint_value(&c).unwrap()
        );
    }
}
