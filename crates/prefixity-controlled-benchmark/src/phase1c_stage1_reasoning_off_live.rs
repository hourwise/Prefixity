//! Single-use live executor for the Phase 1C Stage 1 reasoning-off smoke.
//!
//! This is an experiment-only adapter. It performs one bounded TCP listener
//! check and one HTTP request after explicit fresh-runtime confirmation. It
//! does not retry, warm up, repair, or contact any scored capability path.

use crate::live_harness::LoopbackEndpoint;
use crate::phase1c_stage1_reasoning_off_preflight::{
    preflight_stage1_reasoning_off_smoke, V2_CONTRACT_PATH, V2_EVIDENCE_DIR, V2_FIXTURE_PATH,
};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const EXPERIMENT_ID: &str = "phase-1c-stage1-schema-smoke-02";
const EVIDENCE_SCHEMA_ID: &str = "prefixity.phase1c.stage1.local-qwen-evidence";
const EVIDENCE_SCHEMA_VERSION: u32 = 1;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_INPUT_ESTIMATED_TOKENS: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum ReasoningOffLiveError {
    #[error("reasoning-off live file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reasoning-off live JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("reasoning-off live validation failed: {0}")]
    Validation(String),
    #[error("reasoning-off live transport failed: {0}")]
    Transport(String),
}

pub fn execute_stage1_reasoning_off_smoke(
    confirm_fresh_runtime: bool,
) -> Result<Value, ReasoningOffLiveError> {
    if !confirm_fresh_runtime {
        return Err(ReasoningOffLiveError::Validation(
            "explicit fresh-runtime confirmation is required before the listener check".to_string(),
        ));
    }

    let preflight = preflight_stage1_reasoning_off_smoke()
        .map_err(|error| ReasoningOffLiveError::Validation(error.to_string()))?;
    if preflight.input_estimated_tokens > MAX_INPUT_ESTIMATED_TOKENS {
        return Err(ReasoningOffLiveError::Validation(format!(
            "Smoke 02 input estimate {} exceeds {MAX_INPUT_ESTIMATED_TOKENS}",
            preflight.input_estimated_tokens
        )));
    }
    let contract = read_json(V2_CONTRACT_PATH)?;
    let fixture = read_json(V2_FIXTURE_PATH)?;
    let request = build_request(&contract, &fixture)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    if wire_request_sha256 != preflight.wire_request_sha256 {
        return Err(ReasoningOffLiveError::Validation(
            "live request bytes differ from the preflight projection".to_string(),
        ));
    }
    let endpoint_text = contract_string(&contract, "endpoint")?;
    let endpoint = LoopbackEndpoint::parse(endpoint_text)
        .map_err(|error| ReasoningOffLiveError::Validation(error.to_string()))?;
    let host = contract_string(&contract, "host")?;
    let port = contract
        .get("port")
        .and_then(Value::as_u64)
        .ok_or_else(|| missing("port"))?;
    if host != "127.0.0.1" || port != 8080 {
        return Err(ReasoningOffLiveError::Validation(
            "V2 live endpoint is not the authorized loopback address".to_string(),
        ));
    }

    let readiness_started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port as u16)),
        Duration::from_millis(contract_u64(
            &contract,
            "timeout_policy.connect_timeout_ms",
        )?),
    );
    let readiness_elapsed_ms = readiness_started.elapsed().as_millis() as u64;
    if let Err(error) = readiness {
        return Err(ReasoningOffLiveError::Transport(format!(
            "single non-inference listener check failed after {readiness_elapsed_ms}ms: {error}"
        )));
    }

    let evidence_dir = workspace_path(V2_EVIDENCE_DIR);
    fs::create_dir(&evidence_dir)?;
    write_json(
        &evidence_dir.join("preflight.json"),
        &serde_json::to_value(&preflight)?,
    )?;
    write_bytes(&evidence_dir.join("request.json"), &request_bytes)?;
    write_json(
        &evidence_dir.join("readiness.json"),
        &serde_json::json!({
            "check": "tcp_listener_connect",
            "host": host,
            "port": port,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
        }),
    )?;

    let client = Client::builder()
        .connect_timeout(Duration::from_millis(contract_u64(
            &contract,
            "timeout_policy.connect_timeout_ms",
        )?))
        .timeout(Duration::from_millis(contract_u64(
            &contract,
            "timeout_policy.complete_request_timeout_ms",
        )?))
        .redirect(Policy::none())
        .build()
        .map_err(|error| ReasoningOffLiveError::Transport(error.to_string()))?;

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
            let record = ambiguous_record(
                &preflight,
                &wire_request_sha256,
                request_bytes.len(),
                readiness_elapsed_ms,
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
    let message = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/message"));
    let assistant_content = message
        .and_then(|value| value.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let reasoning_content = message
        .and_then(|value| value.get("reasoning_content"))
        .cloned();
    let reasoning_content_present = reasoning_content.is_some();
    let normalized_candidate = match assistant_content.as_deref() {
        Some(content) => normalize_payload(content, &fixture)?,
        None => None,
    };
    let normalized_payload = if reasoning_content_present {
        None
    } else {
        normalized_candidate.clone()
    };
    let success = complete
        && (200..300).contains(&status)
        && parsed.is_some()
        && assistant_content.is_some()
        && normalized_payload.is_some()
        && !reasoning_content_present;
    let error = if success {
        None
    } else if reasoning_content_present {
        Some("unexpected message.reasoning_content under reasoning=off".to_string())
    } else if !complete {
        Some("response exceeded the bounded body size".to_string())
    } else if !(200..300).contains(&status) {
        Some(format!("unexpected HTTP status {status}"))
    } else if parsed.is_none() {
        Some("response body was not valid JSON".to_string())
    } else if assistant_content.is_none() {
        Some("choices[0].message.content was absent or not a string".to_string())
    } else if normalized_payload.is_none() {
        Some("assistant content failed the exact semantic smoke schema".to_string())
    } else {
        Some("Smoke 02 response contract failed".to_string())
    };
    let usage = parsed
        .as_ref()
        .and_then(|value| value.get("usage"))
        .cloned();
    let timings = parsed
        .as_ref()
        .and_then(|value| value.get("timings"))
        .cloned();
    let finish_reason = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/finish_reason"))
        .cloned();
    let record = serde_json::json!({
        "schema_id": EVIDENCE_SCHEMA_ID,
        "schema_version": EVIDENCE_SCHEMA_VERSION,
        "experiment_id": EXPERIMENT_ID,
        "purpose": "schema_smoke_only",
        "state": if success { "PASSED" } else { "FAILED" },
        "runtime_configuration_identity": preflight.v2_contract_sha256,
        "fixture_id": EXPERIMENT_ID,
        "fixture_sha256": preflight.v2_fixture_sha256,
        "request": {
            "authorized_ceiling": preflight.request_ceiling,
            "transport_attempts": 1,
            "inference_requests": 1,
            "automatic_retries": 0,
            "request_projection_sha256": preflight.request_projection_sha256,
            "wire_request_sha256": wire_request_sha256,
            "request_file": "request.json",
            "input_estimated_tokens": preflight.input_estimated_tokens
        },
        "readiness": {
            "listener_check_attempts": 1,
            "elapsed_ms": readiness_elapsed_ms,
            "inference_requests": 0
        },
        "response": {
            "complete": complete,
            "http_status": status,
            "response_body_bytes": response_body.len(),
            "response_body_sha256": response_body_sha256,
            "response_body_file": "response-body.bin",
            "safe_headers": safe_headers,
            "assistant_content": assistant_content,
            "assistant_content_bytes": assistant_content.as_ref().map(String::len),
            "assistant_content_characters": assistant_content
                .as_ref()
                .map(|content| content.chars().count()),
            "reasoning_content_present": reasoning_content_present,
            "reasoning_content": reasoning_content,
            "reasoning_content_bytes": reasoning_content_bytes(message),
            "reasoning_content_characters": reasoning_content_characters(message),
            "finish_reason": finish_reason,
            "usage": usage,
            "timings": timings,
            "transport_elapsed_ms": transport_elapsed_ms
        },
        "normalized_payload": normalized_payload,
        "validation": {
            "response_json_parsed": parsed.is_some(),
            "assistant_content_extracted": assistant_content.is_some(),
            "semantic_payload_parsed": normalized_candidate.is_some(),
            "required_fields_validated": normalized_payload.is_some(),
            "normalized_under_schema": normalized_payload.is_some(),
            "reasoning_content_absent_under_reasoning_off": !reasoning_content_present,
            "no_scored_task_consumed": true,
            "error": error
        },
        "lineage": {
            "phase": "phase-1c",
            "runtime_engine": "llama.cpp",
            "runtime_setting_reasoning": "off",
            "api_surface": contract_string(&contract, "api_surface")?,
            "parent_experiment_id": "phase-1c-stage1-schema-smoke-01",
            "parent_response_sha256": preflight.original_response_sha256,
            "forensic_review_sha256": preflight.forensic_review_sha256,
            "evidence_class": "schema_pipeline_only"
        }
    });
    write_json(&evidence_dir.join("stage1-result.json"), &record)?;
    Ok(record)
}

fn ambiguous_record(
    preflight: &crate::phase1c_stage1_reasoning_off_preflight::RemediationPreflight,
    wire_request_sha256: &str,
    request_bytes: usize,
    readiness_elapsed_ms: u64,
    transport_elapsed_ms: u64,
    error: String,
) -> Value {
    serde_json::json!({
        "schema_id": EVIDENCE_SCHEMA_ID,
        "schema_version": EVIDENCE_SCHEMA_VERSION,
        "experiment_id": EXPERIMENT_ID,
        "purpose": "schema_smoke_only",
        "state": "AMBIGUOUS",
        "runtime_configuration_identity": preflight.v2_contract_sha256,
        "fixture_id": EXPERIMENT_ID,
        "fixture_sha256": preflight.v2_fixture_sha256,
        "request": {
            "authorized_ceiling": preflight.request_ceiling,
            "transport_attempts": 1,
            "inference_requests": 1,
            "automatic_retries": 0,
            "request_projection_sha256": preflight.request_projection_sha256,
            "wire_request_sha256": wire_request_sha256,
            "request_file": "request.json",
            "input_estimated_tokens": preflight.input_estimated_tokens,
            "request_body_bytes": request_bytes
        },
        "readiness": {
            "listener_check_attempts": 1,
            "elapsed_ms": readiness_elapsed_ms,
            "inference_requests": 0
        },
        "response": {
            "complete": false,
            "http_status": null,
            "response_body_bytes": null,
            "response_body_sha256": null,
            "response_body_file": null,
            "safe_headers": {},
            "assistant_content": null,
            "reasoning_content_present": null,
            "reasoning_content": null,
            "finish_reason": null,
            "usage": null,
            "timings": null,
            "transport_elapsed_ms": transport_elapsed_ms
        },
        "normalized_payload": null,
        "validation": {
            "response_json_parsed": false,
            "assistant_content_extracted": false,
            "semantic_payload_parsed": false,
            "required_fields_validated": false,
            "normalized_under_schema": false,
            "reasoning_content_absent_under_reasoning_off": null,
            "no_scored_task_consumed": true,
            "error": error
        }
    })
}

fn build_request(contract: &Value, fixture: &Value) -> Result<Value, ReasoningOffLiveError> {
    Ok(serde_json::json!({
        "model": contract.get("model").cloned().ok_or_else(|| missing("model"))?,
        "messages": fixture.get("messages").cloned().ok_or_else(|| missing("messages"))?,
        "temperature": contract_value(contract, "generation.temperature")?,
        "top_p": contract_value(contract, "generation.top_p")?,
        "max_tokens": contract_value(contract, "generation.max_tokens")?,
        "stream": contract_value(contract, "generation.stream")?,
        "seed": contract_value(contract, "generation.seed.value")?
    }))
}

fn normalize_payload(
    content: &str,
    fixture: &Value,
) -> Result<Option<Value>, ReasoningOffLiveError> {
    let trimmed = content.trim();
    if trimmed.starts_with("```") || trimmed.ends_with("```") {
        return Ok(None);
    }
    let value: Value = match serde_json::from_str(trimmed) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    let expected = fixture
        .get("expected_payload")
        .ok_or_else(|| missing("expected_payload"))?;
    if value.as_object().is_some() && value == *expected {
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn reasoning_content_bytes(message: Option<&Value>) -> Option<usize> {
    message
        .and_then(|value| value.get("reasoning_content"))
        .and_then(Value::as_str)
        .map(str::len)
}

fn reasoning_content_characters(message: Option<&Value>) -> Option<usize> {
    message
        .and_then(|value| value.get("reasoning_content"))
        .and_then(Value::as_str)
        .map(|value| value.chars().count())
}

fn contract_string(contract: &Value, path: &str) -> Result<String, ReasoningOffLiveError> {
    contract_value(contract, path)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| ReasoningOffLiveError::Validation(format!("{path} is not a string")))
}

fn contract_u64(contract: &Value, path: &str) -> Result<u64, ReasoningOffLiveError> {
    contract_value(contract, path)?
        .as_u64()
        .ok_or_else(|| ReasoningOffLiveError::Validation(format!("{path} is not an integer")))
}

fn contract_value<'a>(contract: &'a Value, path: &str) -> Result<&'a Value, ReasoningOffLiveError> {
    contract
        .pointer(&format!("/{}", path.replace('.', "/")))
        .ok_or_else(|| missing(path))
}

fn missing(path: &str) -> ReasoningOffLiveError {
    ReasoningOffLiveError::Validation(format!("missing required field {path}"))
}

fn read_json(path: &str) -> Result<Value, ReasoningOffLiveError> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
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

fn write_json(path: &Path, value: &Value) -> Result<(), ReasoningOffLiveError> {
    write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), ReasoningOffLiveError> {
    fs::write(path, bytes)?;
    Ok(())
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
    fn payload_normalization_is_exact_and_non_repairing() {
        let fixture = serde_json::json!({
            "expected_payload": {
                "schema_version": "phase1c-stage1-smoke-v1",
                "status": "ok",
                "marker": "PREFIXITY_PHASE1C_STAGE1"
            }
        });
        assert!(normalize_payload(
            r#"{"schema_version":"phase1c-stage1-smoke-v1","status":"ok","marker":"PREFIXITY_PHASE1C_STAGE1"}"#,
            &fixture
        )
        .unwrap()
        .is_some());
        assert!(normalize_payload("```json\n{}\n```", &fixture)
            .unwrap()
            .is_none());
        assert!(normalize_payload("prose", &fixture).unwrap().is_none());
    }
}
