//! Offline validator for the persisted Phase 1C Stage 1 reasoning-off smoke.
//!
//! This helper reads only the completed Smoke 02 evidence directory. It never
//! opens a socket and never contacts the model endpoint.

use crate::phase1c_stage1_reasoning_off_preflight::{
    validate_stage1_reasoning_off_contract_and_fixture, V2_CONTRACT_FINGERPRINT_PATH,
    V2_CONTRACT_PATH, V2_EVIDENCE_DIR, V2_FIXTURE_PATH,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const EXPERIMENT_ID: &str = "phase-1c-stage1-schema-smoke-02";
const V2_CONTRACT_SHA256: &str = "b57462a22c91649e9e629b22d65214cdea7a47a38570506256c0b945467fe882";
const V2_FIXTURE_SHA256: &str = "1ce4ec0232d49fc2941f432b8a1067fa5fa165ce3dc4727312f79190f0643c78";
const REQUEST_SHA256: &str = "49cbcae3c82924542434d5e8f8f92994096d9ce032b99e475a89cdcdc8805b94";
const RESPONSE_SHA256: &str = "20514424872eaee321405697287816fcb02d16f5a4be5e1f51aa5ee3c860b978";
const EXPECTED_MARKER: &str = "PREFIXITY_PHASE1C_STAGE1";

#[derive(Debug, thiserror::Error)]
pub enum ReasoningOffPostrunError {
    #[error("reasoning-off post-run file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reasoning-off post-run JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("reasoning-off post-run validation failed: {0}")]
    Validation(String),
}

pub fn validate_stage1_reasoning_off_evidence() -> Result<Value, ReasoningOffPostrunError> {
    let contract_preflight = validate_stage1_reasoning_off_contract_and_fixture()
        .map_err(|error| ReasoningOffPostrunError::Validation(error.to_string()))?;
    if contract_preflight.v2_contract_sha256 != V2_CONTRACT_SHA256
        || contract_preflight.v2_fixture_sha256 != V2_FIXTURE_SHA256
        || contract_preflight.request_projection_sha256 != REQUEST_SHA256
    {
        return Err(invalid(
            "offline V2 contract/fixture preflight identity changed",
        ));
    }
    let contract = read_json(V2_CONTRACT_PATH)?;
    let fixture = read_json(V2_FIXTURE_PATH)?;
    let evidence_dir = workspace_path(V2_EVIDENCE_DIR);
    let preflight = read_json_from(&evidence_dir.join("preflight.json"))?;
    let readiness = read_json_from(&evidence_dir.join("readiness.json"))?;
    let result = read_json_from(&evidence_dir.join("stage1-result.json"))?;
    let request_bytes = fs::read(evidence_dir.join("request.json"))?;
    let response_body = fs::read(evidence_dir.join("response-body.bin"))?;

    validate_contract(&contract)?;
    validate_fixture(&fixture)?;
    validate_preflight(&preflight)?;
    validate_readiness(&readiness)?;
    if sha256_hex(&request_bytes) != REQUEST_SHA256 {
        return Err(ReasoningOffPostrunError::Validation(
            "persisted request fingerprint changed".to_string(),
        ));
    }
    validate_result(&result, &fixture, &response_body)?;

    Ok(serde_json::json!({
        "state": "VALIDATED",
        "experiment_id": EXPERIMENT_ID,
        "contract_sha256": V2_CONTRACT_SHA256,
        "fixture_sha256": V2_FIXTURE_SHA256,
        "request_sha256": REQUEST_SHA256,
        "response_body_sha256": RESPONSE_SHA256,
        "response_body_bytes": response_body.len(),
        "assistant_content_bytes": result.pointer("/response/assistant_content_bytes"),
        "assistant_content_characters": result.pointer("/response/assistant_content_characters"),
        "reasoning_content_present": false,
        "http_status": 200,
        "finish_reason": "stop",
        "normalized_payload": fixture.get("expected_payload"),
        "listener_check_attempts": 1,
        "inference_requests": 1,
        "automatic_retries": 0,
        "network_calls_after_run": 0,
        "scored_task_ids": []
    }))
}

fn validate_contract(contract: &Value) -> Result<(), ReasoningOffPostrunError> {
    if fingerprint_value(contract)? != V2_CONTRACT_SHA256
        || contract.get("experiment_id") != Some(&Value::String(EXPERIMENT_ID.to_string()))
        || contract.pointer("/runtime_settings/reasoning")
            != Some(&Value::String("off".to_string()))
        || contract.get("model")
            != Some(&Value::String(
                "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0".to_string(),
            ))
        || contract.get("quantization") != Some(&Value::String("Q4_0".to_string()))
        || contract.get("context_size") != Some(&serde_json::json!(8192))
        || contract.get("parallel_slots") != Some(&serde_json::json!(1))
        || contract.get("metrics") != Some(&Value::String("enabled".to_string()))
        || contract.get("request_ceiling") != Some(&serde_json::json!(1))
        || contract.pointer("/retry_policy/automatic_retries") != Some(&serde_json::json!(0))
    {
        return Err(ReasoningOffPostrunError::Validation(
            "V2 contract identity or runtime bounds changed".to_string(),
        ));
    }
    let sidecar = fs::read_to_string(workspace_path(V2_CONTRACT_FINGERPRINT_PATH))?;
    if !sidecar.contains(V2_CONTRACT_SHA256) {
        return Err(ReasoningOffPostrunError::Validation(
            "V2 contract fingerprint sidecar does not match".to_string(),
        ));
    }
    Ok(())
}

fn validate_fixture(fixture: &Value) -> Result<(), ReasoningOffPostrunError> {
    if fingerprint_value(fixture)? != V2_FIXTURE_SHA256
        || fixture.get("fixture_id") != Some(&Value::String(EXPERIMENT_ID.to_string()))
        || fixture.get("scored") != Some(&Value::Bool(false))
        || fixture.get("scored_task_ids") != Some(&Value::Array(Vec::new()))
        || fixture.get("contains_evaluator_answer_material") != Some(&Value::Bool(false))
        || fixture.pointer("/expected_payload/marker")
            != Some(&Value::String(EXPECTED_MARKER.to_string()))
    {
        return Err(ReasoningOffPostrunError::Validation(
            "Smoke 02 fixture identity or non-scored boundary changed".to_string(),
        ));
    }
    Ok(())
}

fn validate_preflight(preflight: &Value) -> Result<(), ReasoningOffPostrunError> {
    if preflight.get("state") != Some(&Value::String("PREPARED".to_string()))
        || preflight.get("experiment_id") != Some(&Value::String(EXPERIMENT_ID.to_string()))
        || preflight.get("v2_contract_sha256")
            != Some(&Value::String(V2_CONTRACT_SHA256.to_string()))
        || preflight.get("v2_fixture_sha256") != Some(&Value::String(V2_FIXTURE_SHA256.to_string()))
        || preflight.get("request_projection_sha256")
            != Some(&Value::String(REQUEST_SHA256.to_string()))
        || preflight.get("wire_request_sha256") != Some(&Value::String(REQUEST_SHA256.to_string()))
        || preflight.get("network_calls") != Some(&serde_json::json!(0))
        || preflight.get("inference_requests") != Some(&serde_json::json!(0))
    {
        return Err(ReasoningOffPostrunError::Validation(
            "persisted preflight identity or zero-contact accounting changed".to_string(),
        ));
    }
    Ok(())
}

fn validate_readiness(readiness: &Value) -> Result<(), ReasoningOffPostrunError> {
    if readiness.get("check") != Some(&Value::String("tcp_listener_connect".to_string()))
        || readiness.get("network_calls") != Some(&serde_json::json!(1))
        || readiness.get("inference_requests") != Some(&serde_json::json!(0))
        || readiness
            .get("elapsed_ms")
            .and_then(Value::as_u64)
            .is_none()
    {
        return Err(ReasoningOffPostrunError::Validation(
            "persisted listener readiness record is not the single permitted check".to_string(),
        ));
    }
    Ok(())
}

fn validate_result(
    result: &Value,
    fixture: &Value,
    response_body: &[u8],
) -> Result<(), ReasoningOffPostrunError> {
    if result.get("state") != Some(&Value::String("PASSED".to_string()))
        || result.get("experiment_id") != Some(&Value::String(EXPERIMENT_ID.to_string()))
        || result.get("fixture_id") != Some(&Value::String(EXPERIMENT_ID.to_string()))
        || result.pointer("/request/authorized_ceiling") != Some(&serde_json::json!(1))
        || result.pointer("/request/transport_attempts") != Some(&serde_json::json!(1))
        || result.pointer("/request/inference_requests") != Some(&serde_json::json!(1))
        || result.pointer("/request/automatic_retries") != Some(&serde_json::json!(0))
        || result.pointer("/request/request_projection_sha256")
            != Some(&Value::String(REQUEST_SHA256.to_string()))
        || result.pointer("/request/wire_request_sha256")
            != Some(&Value::String(REQUEST_SHA256.to_string()))
        || result.pointer("/response/complete") != Some(&Value::Bool(true))
        || result.pointer("/response/http_status") != Some(&serde_json::json!(200))
        || result.pointer("/response/response_body_bytes")
            != Some(&serde_json::json!(response_body.len()))
        || result.pointer("/response/response_body_sha256")
            != Some(&Value::String(RESPONSE_SHA256.to_string()))
        || result.pointer("/response/reasoning_content_present") != Some(&Value::Bool(false))
        || result.pointer("/response/reasoning_content") != Some(&Value::Null)
        || result.pointer("/response/finish_reason") != Some(&Value::String("stop".to_string()))
        || result.pointer("/validation/no_scored_task_consumed") != Some(&Value::Bool(true))
        || result.pointer("/validation/normalized_under_schema") != Some(&Value::Bool(true))
        || result.pointer("/validation/error") != Some(&Value::Null)
    {
        return Err(ReasoningOffPostrunError::Validation(
            "persisted Smoke 02 accounting or validation state is not a pass".to_string(),
        ));
    }
    if sha256_hex(response_body) != RESPONSE_SHA256 {
        return Err(ReasoningOffPostrunError::Validation(
            "persisted Smoke 02 response body fingerprint changed".to_string(),
        ));
    }
    let raw: Value = serde_json::from_slice(response_body)?;
    if raw
        .pointer("/choices/0/message/reasoning_content")
        .is_some()
    {
        return Err(ReasoningOffPostrunError::Validation(
            "reasoning_content appeared in the persisted response under reasoning=off".to_string(),
        ));
    }
    let content = raw
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("raw response content is absent or not a string"))?;
    if Some(content)
        != result
            .pointer("/response/assistant_content")
            .and_then(Value::as_str)
    {
        return Err(invalid(
            "persisted assistant content differs from raw response",
        ));
    }
    let normalized: Value = serde_json::from_str(content.trim())?;
    if normalized
        != *fixture
            .get("expected_payload")
            .ok_or_else(|| invalid("fixture expected payload is absent"))?
    {
        return Err(invalid(
            "persisted assistant content is not the exact expected payload",
        ));
    }
    if raw.get("usage") != result.pointer("/response/usage")
        || raw.get("timings") != result.pointer("/response/timings")
        || raw.pointer("/choices/0/finish_reason") != result.pointer("/response/finish_reason")
    {
        return Err(invalid(
            "persisted response telemetry differs from raw response",
        ));
    }
    Ok(())
}

fn invalid(message: &str) -> ReasoningOffPostrunError {
    ReasoningOffPostrunError::Validation(message.to_string())
}

fn read_json(path: &str) -> Result<Value, ReasoningOffPostrunError> {
    read_json_from(&workspace_path(path))
}

fn read_json_from(path: &Path) -> Result<Value, ReasoningOffPostrunError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn fingerprint_value(value: &Value) -> Result<String, ReasoningOffPostrunError> {
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
