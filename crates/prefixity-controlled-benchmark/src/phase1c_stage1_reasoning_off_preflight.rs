//! Offline preparation validator for the distinct Phase 1C Stage 1
//! reasoning-off schema smoke.
//!
//! This helper is intentionally not a live adapter. It validates the V2
//! contract, its parent lineage, the matched non-scored fixture, and the
//! unchanged request projection without opening a socket or reading a
//! credential. The existing V1 live adapter remains untouched.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const V2_CONTRACT_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT_V2.json";
pub const V2_CONTRACT_SCHEMA_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT_V2.schema.json";
pub const V2_FIXTURE_PATH: &str = "fixtures/phase1c/phase1c-stage1-schema-smoke-02.json";
pub const V1_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT.json";
pub const V1_FIXTURE_PATH: &str = "fixtures/phase1c/phase1c-stage1-schema-smoke-01.json";
pub const V1_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT.sha256";
pub const V2_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_LOCAL_QWEN_RUNTIME_CONTRACT_V2.sha256";
pub const FORENSIC_REVIEW_PATH: &str =
    "docs/phase-1/PHASE_1C_STAGE1_FAILED_SMOKE_FORENSIC_REVIEW.md";
pub const ORIGINAL_RESPONSE_PATH: &str =
    "experiments/runs/phase1c-stage1-schema-smoke-01/response-body.bin";
pub const V2_EVIDENCE_DIR: &str = "experiments/runs/phase1c-stage1-schema-smoke-02";

const V1_CONTRACT_SHA256: &str = "6d32375c24d51b98d82a8873d6e0f03f1d8c9b59799b843cd3a3d69c05b097b3";
const ORIGINAL_RESPONSE_SHA256: &str =
    "223006f5bed5fec81dae02097e3e723d8c8d3a5997c8ebb116841e95914eb048";
const FORENSIC_REVIEW_SHA256: &str =
    "f9ef96b36ba42da544adaeccafc9e38830e1efa9d7e456d4214c7df3e9a8705c";
const MODEL_ID: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
const ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
const REQUEST_SCHEMA_ID: &str = "prefixity.phase1c.stage1.local-qwen-request-v1";
const OUTPUT_SCHEMA_ID: &str = "prefixity.phase1c.stage1.smoke-v1";
const MAX_INPUT_ESTIMATED_TOKENS: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum RemediationPreflightError {
    #[error("reasoning-off preflight file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reasoning-off preflight JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("reasoning-off preflight validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct RemediationPreflight {
    pub state: String,
    pub experiment_id: String,
    pub parent_experiment_id: String,
    pub parent_contract_sha256: String,
    pub v2_contract_sha256: String,
    pub v1_fixture_sha256: String,
    pub v2_fixture_sha256: String,
    pub request_projection_sha256: String,
    pub wire_request_sha256: String,
    pub evidence_lineage_sha256: String,
    pub forensic_review_sha256: String,
    pub original_response_sha256: String,
    pub model: String,
    pub endpoint: String,
    pub reasoning: String,
    pub input_estimated_tokens: usize,
    pub request_ceiling: u64,
    pub automatic_retries: u64,
    pub evidence_location: String,
    pub scored_task_ids: Vec<String>,
    pub network_calls: u32,
    pub inference_requests: u32,
}

pub fn preflight_stage1_reasoning_off_smoke(
) -> Result<RemediationPreflight, RemediationPreflightError> {
    preflight_stage1_reasoning_off_smoke_with_evidence_guard(true)
}

pub fn validate_stage1_reasoning_off_contract_and_fixture(
) -> Result<RemediationPreflight, RemediationPreflightError> {
    preflight_stage1_reasoning_off_smoke_with_evidence_guard(false)
}

fn preflight_stage1_reasoning_off_smoke_with_evidence_guard(
    require_fresh_evidence_location: bool,
) -> Result<RemediationPreflight, RemediationPreflightError> {
    let v1_contract = read_json(V1_CONTRACT_PATH)?;
    let v2_contract = read_json(V2_CONTRACT_PATH)?;
    let v1_fixture = read_json(V1_FIXTURE_PATH)?;
    let v2_fixture = read_json(V2_FIXTURE_PATH)?;
    let v2_schema = read_json(V2_CONTRACT_SCHEMA_PATH)?;
    let request_schema =
        read_json("docs/phase-1/schemas/phase1c-stage1-local-qwen-request-v1.schema.json")?;
    let output_schema = read_json("docs/phase-1/schemas/phase1c-stage1-smoke-v1.schema.json")?;
    let forensic_bytes = fs::read(workspace_path(FORENSIC_REVIEW_PATH))?;
    let original_response = fs::read(workspace_path(ORIGINAL_RESPONSE_PATH))?;

    validate_v2_schema(&v2_schema)?;
    validate_contract_delta(&v1_contract, &v2_contract)?;
    validate_fixture_pair(&v1_fixture, &v2_fixture)?;
    validate_request_schemas(&request_schema, &output_schema)?;

    let sidecar = fs::read_to_string(workspace_path(V1_CONTRACT_FINGERPRINT_PATH))?;
    if !sidecar.contains(V1_CONTRACT_SHA256) {
        return Err(RemediationPreflightError::Validation(
            "V1 contract fingerprint sidecar does not retain the accepted identity".to_string(),
        ));
    }

    let original_response_sha256 = sha256_hex(&original_response);
    if original_response_sha256 != ORIGINAL_RESPONSE_SHA256 {
        return Err(RemediationPreflightError::Validation(
            "Smoke 01 response hash changed".to_string(),
        ));
    }
    let forensic_review_sha256 = sha256_hex(&forensic_bytes);
    if forensic_review_sha256 != FORENSIC_REVIEW_SHA256 {
        return Err(RemediationPreflightError::Validation(
            "accepted forensic review hash changed".to_string(),
        ));
    }
    if require_fresh_evidence_location && workspace_path(V2_EVIDENCE_DIR).exists() {
        return Err(RemediationPreflightError::Validation(format!(
            "fresh Smoke 02 evidence location already exists: {V2_EVIDENCE_DIR}"
        )));
    }

    let request = build_request(&v2_contract, &v2_fixture)?;
    validate_request(&request, &v2_contract, &v2_fixture)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let input_estimated_tokens = request_bytes.len().saturating_add(3) / 4;
    if input_estimated_tokens > MAX_INPUT_ESTIMATED_TOKENS {
        return Err(RemediationPreflightError::Validation(format!(
            "Smoke 02 input estimate {input_estimated_tokens} exceeds {MAX_INPUT_ESTIMATED_TOKENS}"
        )));
    }

    let v2_contract_sha256 = fingerprint_value(&v2_contract)?;
    let v2_sidecar = fs::read_to_string(workspace_path(V2_CONTRACT_FINGERPRINT_PATH))?;
    if !v2_sidecar.contains(&v2_contract_sha256) {
        return Err(RemediationPreflightError::Validation(
            "V2 contract fingerprint sidecar does not match the canonical contract".to_string(),
        ));
    }
    let v1_fixture_sha256 = fingerprint_value(&v1_fixture)?;
    let v2_fixture_sha256 = fingerprint_value(&v2_fixture)?;
    let request_projection_sha256 = fingerprint_value(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    let evidence_lineage = serde_json::json!({
        "phase": "phase-1c",
        "experiment_id": "phase-1c-stage1-schema-smoke-02",
        "parent_experiment_id": "phase-1c-stage1-schema-smoke-01",
        "parent_contract_sha256": V1_CONTRACT_SHA256,
        "v2_contract_sha256": v2_contract_sha256,
        "forensic_review_sha256": forensic_review_sha256,
        "original_response_sha256": original_response_sha256,
        "remediation": "reasoning-off"
    });

    Ok(RemediationPreflight {
        state: "PREPARED".to_string(),
        experiment_id: "phase-1c-stage1-schema-smoke-02".to_string(),
        parent_experiment_id: "phase-1c-stage1-schema-smoke-01".to_string(),
        parent_contract_sha256: V1_CONTRACT_SHA256.to_string(),
        v2_contract_sha256,
        v1_fixture_sha256,
        v2_fixture_sha256,
        request_projection_sha256,
        wire_request_sha256,
        evidence_lineage_sha256: fingerprint_value(&evidence_lineage)?,
        forensic_review_sha256,
        original_response_sha256,
        model: MODEL_ID.to_string(),
        endpoint: ENDPOINT.to_string(),
        reasoning: "off".to_string(),
        input_estimated_tokens,
        request_ceiling: 1,
        automatic_retries: 0,
        evidence_location: V2_EVIDENCE_DIR.to_string(),
        scored_task_ids: Vec::new(),
        network_calls: 0,
        inference_requests: 0,
    })
}

fn validate_v2_schema(schema: &Value) -> Result<(), RemediationPreflightError> {
    if schema.get("$id")
        != Some(&Value::String(
            "prefixity.phase1c.stage1.local-qwen-runtime-v2".to_string(),
        ))
    {
        return Err(RemediationPreflightError::Validation(
            "V2 contract schema identity is not canonical".to_string(),
        ));
    }
    Ok(())
}

fn validate_contract_delta(v1: &Value, v2: &Value) -> Result<(), RemediationPreflightError> {
    let unchanged_fields = [
        "phase",
        "purpose",
        "engine",
        "server_executable_family",
        "api_surface",
        "endpoint",
        "host",
        "port",
        "model",
        "quantization",
        "context_size",
        "parallel_slots",
        "metrics",
        "credential_policy",
        "generation",
        "timeout_policy",
        "retry_policy",
        "request_ceiling",
        "input_schema_id",
        "output_schema_id",
        "freshness",
        "contamination_rules",
        "prohibited_claims",
        "candidate_source_commit",
        "design_commit",
    ];
    for field in unchanged_fields {
        if v1.get(field) != v2.get(field) {
            return Err(RemediationPreflightError::Validation(format!(
                "V2 changed non-remediation field {field}"
            )));
        }
    }
    ensure_string(
        v2,
        "contract_version",
        "phase1c-stage1-local-qwen-runtime-v2",
    )?;
    ensure_string(v2, "experiment_id", "phase-1c-stage1-schema-smoke-02")?;
    ensure_string(v2, "evidence_location", V2_EVIDENCE_DIR)?;
    ensure_string(v2, "runtime_settings.reasoning", "off")?;
    ensure_string(v2, "parent_contract.path", V1_CONTRACT_PATH)?;
    ensure_string(
        v2,
        "parent_experiment_id",
        "phase-1c-stage1-schema-smoke-01",
    )?;
    ensure_string(v2, "remediation.id", "reasoning-off")?;
    ensure_string(v2, "remediation.changed_runtime_field", "reasoning")?;
    ensure_string(v2, "remediation.changed_runtime_value", "off")?;
    ensure_string(v2, "parent_contract.sha256", V1_CONTRACT_SHA256)?;
    ensure_string(v2, "forensic_basis.review_path", FORENSIC_REVIEW_PATH)?;
    ensure_string(v2, "forensic_basis.review_sha256", FORENSIC_REVIEW_SHA256)?;
    ensure_string(
        v2,
        "forensic_basis.failed_response_sha256",
        ORIGINAL_RESPONSE_SHA256,
    )?;
    ensure_string(
        v2,
        "forensic_basis.primary_classification",
        "THINKING_BUDGET_EXHAUSTION",
    )?;
    ensure_string(
        v2,
        "forensic_basis.secondary_classification",
        "RUNTIME_RESPONSE_SHAPE_INCOMPATIBILITY",
    )?;
    if v2.get("changed_runtime_fields") != Some(&serde_json::json!(["reasoning"])) {
        return Err(RemediationPreflightError::Validation(
            "V2 changed-runtime field set is not exactly reasoning".to_string(),
        ));
    }
    let launch_arguments = v2
        .pointer("/launch/arguments")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            RemediationPreflightError::Validation("V2 launch arguments are absent".to_string())
        })?;
    let expected_arguments = vec![
        "serve",
        "-hf",
        MODEL_ID,
        "-c",
        "8192",
        "-np",
        "1",
        "--metrics",
        "--reasoning",
        "off",
        "--host",
        "127.0.0.1",
        "--port",
        "8080",
    ];
    let actual_arguments: Vec<&str> = launch_arguments.iter().filter_map(Value::as_str).collect();
    if actual_arguments != expected_arguments {
        return Err(RemediationPreflightError::Validation(
            "V2 launch arguments are not the exact reasoning-off command".to_string(),
        ));
    }
    Ok(())
}

fn validate_fixture_pair(v1: &Value, v2: &Value) -> Result<(), RemediationPreflightError> {
    ensure_string(v1, "fixture_id", "phase-1c-stage1-schema-smoke-01")?;
    ensure_string(v2, "fixture_id", "phase-1c-stage1-schema-smoke-02")?;
    if v1.get("fixture_version") != v2.get("fixture_version")
        || v1.get("phase") != v2.get("phase")
        || v1.get("purpose") != v2.get("purpose")
        || v1.get("scored") != v2.get("scored")
        || v1.get("scored_task_ids") != v2.get("scored_task_ids")
        || v1.get("contains_evaluator_answer_material")
            != v2.get("contains_evaluator_answer_material")
        || v1.get("messages") != v2.get("messages")
        || v1.get("expected_payload") != v2.get("expected_payload")
    {
        return Err(RemediationPreflightError::Validation(
            "Smoke 02 fixture differs from Smoke 01 beyond its fresh identity".to_string(),
        ));
    }
    if v2.get("scored") != Some(&Value::Bool(false))
        || v2.get("scored_task_ids") != Some(&Value::Array(Vec::new()))
        || v2.get("contains_evaluator_answer_material") != Some(&Value::Bool(false))
    {
        return Err(RemediationPreflightError::Validation(
            "Smoke 02 fixture is not explicitly non-scored and answer-free".to_string(),
        ));
    }
    Ok(())
}

fn validate_request_schemas(
    request_schema: &Value,
    output_schema: &Value,
) -> Result<(), RemediationPreflightError> {
    if request_schema.get("$id") != Some(&Value::String(REQUEST_SCHEMA_ID.to_string()))
        || output_schema.get("$id") != Some(&Value::String(OUTPUT_SCHEMA_ID.to_string()))
    {
        return Err(RemediationPreflightError::Validation(
            "registered request/output schema identities changed".to_string(),
        ));
    }
    Ok(())
}

fn build_request(contract: &Value, fixture: &Value) -> Result<Value, RemediationPreflightError> {
    let model = contract
        .get("model")
        .cloned()
        .ok_or_else(|| missing("model"))?;
    let messages = fixture
        .get("messages")
        .cloned()
        .ok_or_else(|| missing("messages"))?;
    let temperature = contract
        .pointer("/generation/temperature")
        .cloned()
        .ok_or_else(|| missing("generation.temperature"))?;
    let top_p = contract
        .pointer("/generation/top_p")
        .cloned()
        .ok_or_else(|| missing("generation.top_p"))?;
    let max_tokens = contract
        .pointer("/generation/max_tokens")
        .cloned()
        .ok_or_else(|| missing("generation.max_tokens"))?;
    let stream = contract
        .pointer("/generation/stream")
        .cloned()
        .ok_or_else(|| missing("generation.stream"))?;
    let seed = contract
        .pointer("/generation/seed/value")
        .cloned()
        .ok_or_else(|| missing("generation.seed.value"))?;
    Ok(serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": temperature,
        "top_p": top_p,
        "max_tokens": max_tokens,
        "stream": stream,
        "seed": seed
    }))
}

fn validate_request(
    request: &Value,
    contract: &Value,
    fixture: &Value,
) -> Result<(), RemediationPreflightError> {
    let object = request.as_object().ok_or_else(|| {
        RemediationPreflightError::Validation("request is not an object".to_string())
    })?;
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
    let actual: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    if actual != expected
        || object.get("model") != contract.get("model")
        || object.get("messages") != fixture.get("messages")
        || object.get("temperature") != Some(&serde_json::json!(0))
        || object.get("top_p") != Some(&serde_json::json!(1))
        || object.get("max_tokens") != Some(&serde_json::json!(256))
        || object.get("stream") != Some(&Value::Bool(false))
        || object.get("seed") != Some(&serde_json::json!(1))
    {
        return Err(RemediationPreflightError::Validation(
            "Smoke 02 request projection is not the exact matched request".to_string(),
        ));
    }
    Ok(())
}

fn ensure_string(
    value: &Value,
    path: &str,
    expected: &str,
) -> Result<(), RemediationPreflightError> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_str)
        != Some(expected)
    {
        return Err(RemediationPreflightError::Validation(format!(
            "{path} does not equal {expected}"
        )));
    }
    Ok(())
}

fn missing(path: &str) -> RemediationPreflightError {
    RemediationPreflightError::Validation(format!("missing required field {path}"))
}

fn read_json(path: &str) -> Result<Value, RemediationPreflightError> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn fingerprint_value(value: &Value) -> Result<String, RemediationPreflightError> {
    Ok(sha256_hex(&serde_json::to_vec(&canonical_value(
        value.clone(),
    ))?))
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<String, Value> = object
                .into_iter()
                .map(|(key, value)| (key, canonical_value(value)))
                .collect();
            let mut canonical = serde_json::Map::new();
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

    #[test]
    fn canonical_fingerprint_is_stable_and_array_ordered() {
        let a = serde_json::json!({"b":1,"a":["x","y"]});
        let b = serde_json::json!({"a":["x","y"],"b":1});
        let c = serde_json::json!({"a":["y","x"],"b":1});
        assert_eq!(
            fingerprint_value(&a).unwrap(),
            fingerprint_value(&b).unwrap()
        );
        assert_ne!(
            fingerprint_value(&a).unwrap(),
            fingerprint_value(&c).unwrap()
        );
    }

    #[test]
    fn actual_smoke02_contract_and_fixture_pass_offline_preflight() {
        let report = validate_stage1_reasoning_off_contract_and_fixture().unwrap();
        assert_eq!(report.state, "PREPARED");
        assert_eq!(report.experiment_id, "phase-1c-stage1-schema-smoke-02");
        assert_eq!(report.reasoning, "off");
        assert_eq!(report.input_estimated_tokens, 122);
        assert_eq!(report.request_ceiling, 1);
        assert_eq!(report.automatic_retries, 0);
        assert_eq!(report.network_calls, 0);
        assert_eq!(report.inference_requests, 0);
        assert_eq!(report.scored_task_ids, Vec::<String>::new());
        assert_eq!(report.v1_fixture_sha256.len(), 64);
        assert_eq!(report.v2_contract_sha256.len(), 64);
        assert_eq!(report.evidence_lineage_sha256.len(), 64);
    }
}
