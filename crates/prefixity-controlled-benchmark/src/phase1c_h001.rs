//! Isolated Phase 1C h001 task materialization and live-arm executor.
//!
//! This module is experiment-only. It loads the frozen h001 artifacts, checks
//! their identities before any socket is opened, and executes at most one
//! operator-selected arm. It never starts or restarts llama.cpp, retries a
//! request, advances to another arm, or uses evaluator-only data in a model
//! request.

use crate::hashing::{canonical_hash, canonical_json, sha256_hex};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const H001_TASK_PATH: &str = "docs/phase-1/PHASE_1C_H001_SCORED_TASK_V1.json";
pub const H001_SOURCE_MANIFEST_PATH: &str =
    "docs/phase-1/PHASE_1C_H001_SCORED_SOURCE_MANIFEST_V1.json";
pub const H001_REQUIRED_STATE_PATH: &str = "docs/phase-1/PHASE_1C_H001_REQUIRED_STATE_V1.json";
pub const H001_TOOL_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_H001_TOOL_CONTRACT_V1.json";
pub const H001_EVALUATOR_PATH: &str = "docs/phase-1/PHASE_1C_H001_EVALUATOR_V1.json";
pub const H001_ARM_PROJECTIONS_PATH: &str = "docs/phase-1/PHASE_1C_H001_ARM_PROJECTIONS_V1.json";
pub const H001_MANIFEST_PATH: &str = "docs/phase-1/PHASE_1C_H001_SCORED_TASK_MANIFEST_V1.json";
pub const SCORED_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.json";
pub const SCORED_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.sha256";
pub const SCORED_PILOT_MANIFEST_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.json";
pub const SCORED_PILOT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.sha256";
pub const H001_EVIDENCE_ROOT: &str = "experiments/runs/phase1c-scored-capability-v1/h001";

const EXPERIMENT_ID: &str = "phase-1c-scored-capability-v1";
const TASK_ID: &str = "h001";
const TASK_VERSION: &str = "phase1c-h001-scored-task-v1";
const MODEL: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;
const MAX_TURNS: u32 = 3;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum H001Error {
    #[error("h001 file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("h001 JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("h001 validation failed: {0}")]
    Validation(String),
    #[error("h001 live transport failed: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H001Arm {
    Baseline,
    NoOp,
    Intervention,
}

impl H001Arm {
    pub fn parse(value: &str) -> Result<Self, H001Error> {
        match value {
            "BASELINE" => Ok(Self::Baseline),
            "NO_OP" => Ok(Self::NoOp),
            "INTERVENTION" => Ok(Self::Intervention),
            _ => Err(H001Error::Validation(format!(
                "unknown arm {value}; expected BASELINE, NO_OP, or INTERVENTION"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Baseline => "BASELINE",
            Self::NoOp => "NO_OP",
            Self::Intervention => "INTERVENTION",
        }
    }
}

#[derive(Debug, Clone)]
struct H001Artifacts {
    contract: Value,
    task: Value,
    source_manifest: Value,
    required_state: Value,
    tool_contract: Value,
    evaluator: Value,
    arms: Value,
    manifest: Value,
    pilot_manifest: Value,
}

pub fn preflight_h001() -> Result<Value, H001Error> {
    let artifacts = load_artifacts()?;
    validate_artifacts(&artifacts)?;
    let prompt_sha256 = prompt_sha256(&artifacts.task)?;
    let arm_details = arm_details(&artifacts.arms)?;
    let arm_projection_sha256 = arm_details
        .iter()
        .map(|(arm, request, source_ids)| {
            Ok(json!({
                "arm": arm,
                "request_sha256": canonical_hash(request)?,
                "source_ids": source_ids,
            }))
        })
        .collect::<Result<Vec<_>, H001Error>>()?;
    Ok(json!({
        "state": "PREPARED",
        "experiment_id": EXPERIMENT_ID,
        "task_id": TASK_ID,
        "task_version": TASK_VERSION,
        "runtime_contract_sha256": canonical_hash(&artifacts.contract)?,
        "task_artifact_sha256": canonical_hash(&artifacts.task)?,
        "source_manifest_sha256": canonical_hash(&artifacts.source_manifest)?,
        "required_state_sha256": canonical_hash(&artifacts.required_state)?,
        "tool_contract_sha256": canonical_hash(&artifacts.tool_contract)?,
        "evaluator_sha256": canonical_hash(&artifacts.evaluator)?,
        "arm_materialization_sha256": canonical_hash(&artifacts.arms)?,
        "task_prompt_sha256": prompt_sha256,
        "arm_projection_sha256": arm_projection_sha256,
        "max_turns": MAX_TURNS,
        "replicate": 1,
        "network_calls": 0,
        "credential_reads": 0,
        "inference_requests": 0,
        "evidence_root": H001_EVIDENCE_ROOT,
    }))
}

/// Print preparation fingerprints without opening a socket. This command is
/// intentionally independent of manifest bindings so the checked-in hashes
/// can be filled once after the artifacts are authored.
pub fn fingerprint_h001() -> Result<Value, H001Error> {
    let artifacts = load_artifacts()?;
    let arm_fingerprints = arm_details(&artifacts.arms)?
        .into_iter()
        .map(|(arm, request, source_ids)| {
            Ok(json!({
                "arm": arm,
                "request_sha256": canonical_hash(&request)?,
                "source_ids": source_ids,
            }))
        })
        .collect::<Result<Vec<_>, H001Error>>()?;
    let source_fingerprints = artifacts
        .source_manifest
        .get("source_records")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("source_manifest.source_records"))?
        .iter()
        .map(|record| {
            let source_id = record
                .get("source_id")
                .and_then(Value::as_str)
                .ok_or_else(|| missing("source_id"))?;
            let exact_content = record
                .get("exact_content")
                .and_then(Value::as_str)
                .ok_or_else(|| missing("exact_content"))?;
            Ok(json!({
                "source_id": source_id,
                "declared_sha256": record.get("sha256"),
                "actual_sha256": sha256_hex(exact_content.as_bytes()),
            }))
        })
        .collect::<Result<Vec<_>, H001Error>>()?;
    Ok(json!({
        "task_artifact_sha256": canonical_hash(&artifacts.task)?,
        "source_manifest_sha256": canonical_hash(&artifacts.source_manifest)?,
        "required_state_sha256": canonical_hash(&artifacts.required_state)?,
        "tool_contract_sha256": canonical_hash(&artifacts.tool_contract)?,
        "evaluator_sha256": canonical_hash(&artifacts.evaluator)?,
        "arm_materialization_sha256": canonical_hash(&artifacts.arms)?,
        "pilot_manifest_sha256": canonical_hash(&artifacts.pilot_manifest)?,
        "task_manifest_sha256": canonical_hash(&artifacts.manifest)?,
        "task_prompt_sha256": prompt_sha256(&artifacts.task)?,
        "arm_projection_sha256": arm_fingerprints,
        "source_fingerprints": source_fingerprints,
    }))
}

pub fn dry_run_h001(arm: H001Arm) -> Result<Value, H001Error> {
    let preflight = preflight_h001()?;
    let artifacts = load_artifacts()?;
    let request = request_for_arm(&artifacts.arms, arm)?;
    Ok(json!({
        "state": "DRY_RUN",
        "preflight": preflight,
        "arm": arm.as_str(),
        "request_sha256": canonical_hash(&request)?,
        "request_bytes": serde_json::to_vec(&request)?.len(),
        "model_visible_request": request,
        "network_calls": 0,
        "inference_requests": 0,
    }))
}

pub fn execute_h001_arm(arm: H001Arm, confirm_fresh_runtime: bool) -> Result<Value, H001Error> {
    if !confirm_fresh_runtime {
        return Err(H001Error::Validation(
            "explicit fresh-runtime confirmation is required before the listener check".to_string(),
        ));
    }

    let preflight = preflight_h001()?;
    let artifacts = load_artifacts()?;
    let request = request_for_arm(&artifacts.arms, arm)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let request_sha256 = canonical_hash(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    let evidence_dir = arm_evidence_dir(arm);
    if evidence_dir.exists() {
        return Err(H001Error::Validation(format!(
            "h001 arm evidence already exists: {}",
            evidence_dir.display()
        )));
    }

    let readiness_started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], PORT)),
        Duration::from_millis(contract_u64(
            &artifacts.contract,
            "timeout_policy.connect_timeout_ms",
        )?),
    );
    let readiness_elapsed_ms = readiness_started.elapsed().as_millis() as u64;
    if let Err(error) = readiness {
        return Err(H001Error::Transport(format!(
            "single non-inference listener check failed after {readiness_elapsed_ms}ms: {error}"
        )));
    }

    fs::create_dir_all(&evidence_dir)?;
    write_json(&evidence_dir.join("preflight.json"), &preflight)?;
    write_json(
        &evidence_dir.join("runtime-confirmation.json"),
        &json!({
            "confirmation": "operator_current_confirmation",
            "contract_identity": preflight.get("runtime_contract_sha256"),
            "model": MODEL,
            "quantization": "Q4_0",
            "context_size": 8192,
            "parallel_slots": 1,
            "metrics": "enabled",
            "reasoning": "on",
            "endpoint": "http://127.0.0.1:8080",
            "zero_inference_since_startup": true,
            "no_warmup": true,
            "no_manual_request": true,
            "no_browser_or_endpoint_contact": true,
            "recorded_at_unix_ms": now_unix_ms()?,
        }),
    )?;
    write_json(
        &evidence_dir.join("readiness.json"),
        &json!({
            "check": "tcp_listener_connect",
            "host": HOST,
            "port": PORT,
            "listener_check_attempts": 1,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
        }),
    )?;
    write_bytes(&evidence_dir.join("request-turn-1.json"), &request_bytes)?;

    let client = Client::builder()
        .connect_timeout(Duration::from_millis(contract_u64(
            &artifacts.contract,
            "timeout_policy.connect_timeout_ms",
        )?))
        .timeout(Duration::from_millis(contract_u64(
            &artifacts.contract,
            "timeout_policy.complete_request_timeout_ms",
        )?))
        .redirect(Policy::none())
        .build()
        .map_err(|error| H001Error::Transport(error.to_string()))?;
    let request_started = Instant::now();
    let response = client
        .post("http://127.0.0.1:8080/v1/chat/completions")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_bytes.clone())
        .send();
    let transport_elapsed_ms = request_started.elapsed().as_millis() as u64;

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let result = ambiguous_result(
                arm,
                &request_sha256,
                &wire_request_sha256,
                request_bytes.len(),
                readiness_elapsed_ms,
                transport_elapsed_ms,
                format!("request dispatch/completion is ambiguous: {error}"),
            );
            persist_ambiguous(&evidence_dir, &result)?;
            return Ok(result);
        }
    };

    let status = response.status().as_u16();
    let safe_headers = safe_headers(response.headers());
    let mut response = response;
    let mut response_body = Vec::new();
    let body_result = response
        .by_ref()
        .take((MAX_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response_body);
    let response_body_sha256 = sha256_hex(&response_body);
    write_bytes(&evidence_dir.join("response-turn-1.bin"), &response_body)?;
    if let Err(error) = body_result {
        let result = ambiguous_result(
            arm,
            &request_sha256,
            &wire_request_sha256,
            request_bytes.len(),
            readiness_elapsed_ms,
            transport_elapsed_ms,
            format!("response body read is ambiguous: {error}"),
        );
        persist_ambiguous(&evidence_dir, &result)?;
        return Ok(result);
    }

    let complete = response_body.len() <= MAX_RESPONSE_BYTES;
    let parsed = if complete && (200..300).contains(&status) {
        serde_json::from_slice::<Value>(&response_body).ok()
    } else {
        None
    };
    let message = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/message"));
    let final_content = message
        .and_then(|value| value.get("content"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let reasoning_content = message
        .and_then(|value| value.get("reasoning_content"))
        .cloned();
    let tool_calls = message.and_then(|value| value.get("tool_calls")).cloned();
    let finish_reason = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/finish_reason"))
        .cloned();
    let tool_call_present = tool_calls
        .as_ref()
        .is_some_and(|value| value.as_array().is_some_and(|values| !values.is_empty()));
    let length_terminated = finish_reason
        .as_ref()
        .is_some_and(|value| value == "length");
    let terminal_final_content =
        final_content.is_some() && !tool_call_present && !length_terminated;
    let state = if terminal_final_content && (200..300).contains(&status) && parsed.is_some() {
        "COMPLETE"
    } else {
        "INCONCLUSIVE"
    };
    let error = if state == "COMPLETE" {
        None
    } else if !complete {
        Some("response exceeded the bounded body size".to_string())
    } else if !(200..300).contains(&status) {
        Some(format!("unexpected HTTP status {status}"))
    } else if parsed.is_none() {
        Some("response body was not valid JSON".to_string())
    } else if tool_call_present {
        Some("tool call emitted under the frozen no-tool contract".to_string())
    } else if length_terminated || final_content.is_none() {
        Some("terminal final content was unavailable before the registered ceiling".to_string())
    } else {
        Some("response contract was incomplete".to_string())
    };
    let normalized = json!({
        "schema_id": "prefixity.phase1c.h001.normalized-turn",
        "schema_version": 1,
        "task_id": TASK_ID,
        "arm": arm.as_str(),
        "replicate": 1,
        "turn": 1,
        "final_content": final_content,
        "reasoning_content_diagnostic": reasoning_content,
        "tool_calls": tool_calls,
        "finish_reason": finish_reason,
        "usage": parsed.as_ref().and_then(|value| value.get("usage")).cloned(),
        "timings": parsed.as_ref().and_then(|value| value.get("timings")).cloned(),
        "response_json_parsed": parsed.is_some(),
        "terminal_final_content": terminal_final_content,
        "reasoning_is_scored": false,
    });
    write_json(&evidence_dir.join("normalized-turn-1.json"), &normalized)?;
    let trajectory = json!({
        "task_id": TASK_ID,
        "arm": arm.as_str(),
        "replicate": 1,
        "completed_turns": 1,
        "max_turns": MAX_TURNS,
        "state": state,
        "terminal_final_content": terminal_final_content,
        "tool_calls": tool_call_present,
        "recovery_turns": 0,
        "automatic_retries": 0,
        "next_arm": null,
    });
    write_json(&evidence_dir.join("trajectory-state.json"), &trajectory)?;
    let result = json!({
        "schema_id": "prefixity.phase1c.h001.arm-result",
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "task_id": TASK_ID,
        "arm": arm.as_str(),
        "replicate": 1,
        "state": state,
        "request": {
            "turns": 1,
            "max_turns": MAX_TURNS,
            "transport_attempts": 1,
            "inference_requests": 1,
            "automatic_retries": 0,
            "request_sha256": request_sha256,
            "wire_request_sha256": wire_request_sha256,
            "request_file": "request-turn-1.json"
        },
        "readiness": {
            "listener_check_attempts": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms
        },
        "response": {
            "complete": complete,
            "http_status": status,
            "response_body_bytes": response_body.len(),
            "response_body_sha256": response_body_sha256,
            "response_body_file": "response-turn-1.bin",
            "safe_headers": safe_headers,
            "transport_elapsed_ms": transport_elapsed_ms
        },
        "validation": {
            "response_json_parsed": parsed.is_some(),
            "final_content_available": final_content.is_some(),
            "terminal_final_content": terminal_final_content,
            "reasoning_content_is_diagnostic_only": true,
            "no_tool_call": !tool_call_present,
            "error": error
        },
        "evaluator_input": "normalized-turn-1.json.final_content",
        "reasoning_diagnostic": "normalized-turn-1.json.reasoning_content_diagnostic",
        "next_arm": null
    });
    write_json(&evidence_dir.join("arm-result.json"), &result)?;
    Ok(result)
}

pub fn score_h001_arm(arm: H001Arm) -> Result<Value, H001Error> {
    let artifacts = load_artifacts()?;
    validate_artifacts(&artifacts)?;
    let evidence_dir = arm_evidence_dir(arm);
    let arm_result = read_json(&evidence_dir.join("arm-result.json"))?;
    let normalized = read_json(&evidence_dir.join("normalized-turn-1.json"))?;
    if arm_result.get("arm").and_then(Value::as_str) != Some(arm.as_str()) {
        return Err(H001Error::Validation(
            "arm-result identity mismatch".to_string(),
        ));
    }
    let result_state = arm_result
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("");
    let final_content = normalized.get("final_content").and_then(Value::as_str);
    let expected = artifacts
        .evaluator
        .pointer("/expected_final_answer")
        .ok_or_else(|| missing("evaluator.expected_final_answer"))?;
    let parsed_final =
        final_content.and_then(|content| serde_json::from_str::<Value>(content.trim()).ok());
    let task_success = parsed_final.as_ref() == Some(expected);
    let source_ids = arm_source_ids(&artifacts.arms, arm)?;
    let required_ids = artifacts
        .required_state
        .pointer("/minimum_terminal_state/required_context_source_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            missing("required_state.minimum_terminal_state.required_context_source_ids")
        })?;
    let required_context_recall = required_ids
        .iter()
        .filter(|required| source_ids.iter().any(|source| source == *required))
        .count() as f64
        / required_ids.len() as f64;
    let evaluator_result = if result_state != "COMPLETE" || final_content.is_none() {
        "INCONCLUSIVE"
    } else if task_success {
        "PASS"
    } else {
        "FAIL"
    };
    let scored = json!({
        "schema_id": "prefixity.phase1c.h001.evaluator-result",
        "schema_version": 1,
        "evaluator_version": "stage0-deterministic-evaluator-v1",
        "task_id": TASK_ID,
        "arm": arm.as_str(),
        "replicate": 1,
        "result": evaluator_result,
        "metrics": {
            "task_success": task_success,
            "required_context_recall": required_context_recall,
            "dependency_protocol_validity": task_success && normalized.get("tool_calls").is_none_or(Value::is_null),
            "critical_regressions": if task_success { Vec::<String>::new() } else { vec!["final answer did not match the evaluator key".to_string()] },
            "recovery_turns": 0,
            "evaluator_complete": result_state == "COMPLETE" && final_content.is_some()
        },
        "final_answer_source": "normalized-turn-1.json.final_content",
        "reasoning_source": "normalized-turn-1.json.reasoning_content_diagnostic",
        "reasoning_scored": false,
        "expected_answer_identity": canonical_hash(expected)?,
    });
    write_json(&evidence_dir.join("evaluator-result.json"), &scored)?;
    Ok(scored)
}

fn load_artifacts() -> Result<H001Artifacts, H001Error> {
    Ok(H001Artifacts {
        contract: read_repo_json(SCORED_CONTRACT_PATH)?,
        task: read_repo_json(H001_TASK_PATH)?,
        source_manifest: read_repo_json(H001_SOURCE_MANIFEST_PATH)?,
        required_state: read_repo_json(H001_REQUIRED_STATE_PATH)?,
        tool_contract: read_repo_json(H001_TOOL_CONTRACT_PATH)?,
        evaluator: read_repo_json(H001_EVALUATOR_PATH)?,
        arms: read_repo_json(H001_ARM_PROJECTIONS_PATH)?,
        manifest: read_repo_json(H001_MANIFEST_PATH)?,
        pilot_manifest: read_repo_json(SCORED_PILOT_MANIFEST_PATH)?,
    })
}

fn validate_artifacts(artifacts: &H001Artifacts) -> Result<(), H001Error> {
    expect_string(
        &artifacts.contract,
        "contract_version",
        "phase1c-scored-runtime-local-qwen-v1",
    )?;
    expect_string(&artifacts.contract, "model", MODEL)?;
    expect_u64(&artifacts.contract, "context_size", 8192)?;
    expect_u64(&artifacts.contract, "parallel_slots", 1)?;
    expect_string(&artifacts.contract, "metrics", "enabled")?;
    expect_string(&artifacts.contract, "runtime_settings.reasoning", "on")?;
    expect_u64(&artifacts.contract, "generation.max_tokens", 2048)?;
    expect_u64(&artifacts.contract, "generation.seed.value", 1)?;
    expect_u64(
        &artifacts.contract,
        "request_ceiling.per_arm_replicate_max_model_turns",
        3,
    )?;
    expect_u64(&artifacts.contract, "retry_policy.automatic_retries", 0)?;
    expect_u64(&artifacts.contract, "retry_policy.fallback_requests", 0)?;
    expect_u64(&artifacts.contract, "retry_policy.adaptive_replicates", 0)?;
    expect_u64(
        &artifacts.contract,
        "freshness_cache_isolation.maximum_non_inference_listener_checks_per_server",
        1,
    )?;
    expect_string(
        &artifacts.contract,
        "freshness_cache_isolation.listener_check",
        "bounded TCP listener check only; no HTTP readiness probe",
    )?;

    expect_string(&artifacts.task, "task_id", TASK_ID)?;
    expect_string(&artifacts.task, "task_version", TASK_VERSION)?;
    expect_string(&artifacts.source_manifest, "task_id", TASK_ID)?;
    expect_string(&artifacts.required_state, "task_id", TASK_ID)?;
    expect_string(&artifacts.tool_contract, "task_id", TASK_ID)?;
    expect_string(&artifacts.evaluator, "task_id", TASK_ID)?;
    expect_string(&artifacts.arms, "task_id", TASK_ID)?;
    expect_string(&artifacts.manifest, "task_id", TASK_ID)?;
    expect_string(&artifacts.manifest, "task_version", TASK_VERSION)?;
    expect_string(&artifacts.manifest, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(&artifacts.manifest, "max_turns", 3)?;
    expect_u64(&artifacts.manifest, "replicate", 1)?;
    expect_string(
        &artifacts.manifest,
        "arm_order",
        "BASELINE -> NO_OP -> INTERVENTION",
    )?;
    expect_u64(&artifacts.manifest, "retry_policy.automatic_retries", 0)?;

    let contract_sha = canonical_hash(&artifacts.contract)?;
    let contract_sidecar = read_repo_json(SCORED_CONTRACT_FINGERPRINT_PATH)?;
    if contract_sidecar
        .get("canonical_sha256")
        .and_then(Value::as_str)
        != Some(contract_sha.as_str())
    {
        return Err(H001Error::Validation(
            "scored runtime contract fingerprint mismatch".to_string(),
        ));
    }
    let scored_design = crate::phase1c_scored_design::validate_scored_design()
        .map_err(|error| H001Error::Validation(error.to_string()))?;
    if scored_design.contract_schema_sha256
        != "ad42524c6fb2f75056391754cfedd3d84658d797199429f0b68827fdb33d80fb"
    {
        return Err(H001Error::Validation(
            "scored runtime contract schema fingerprint mismatch".to_string(),
        ));
    }
    let pilot_sha = canonical_hash(&artifacts.pilot_manifest)?;
    let pilot_sidecar = read_repo_json(SCORED_PILOT_FINGERPRINT_PATH)?;
    if pilot_sidecar
        .get("canonical_sha256")
        .and_then(Value::as_str)
        != Some(pilot_sha.as_str())
    {
        return Err(H001Error::Validation(
            "scored pilot manifest fingerprint mismatch".to_string(),
        ));
    }
    expect_string(
        &artifacts.pilot_manifest,
        "manifest_version",
        "phase1c-scored-pilot-manifest-v1",
    )?;
    expect_string(
        &artifacts.pilot_manifest,
        "status",
        "PREREGISTERED_DESIGN_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    let pilot_cases = artifacts
        .pilot_manifest
        .pointer("/cohort/pilot_case_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("pilot_manifest.cohort.pilot_case_ids"))?;
    if !pilot_cases
        .iter()
        .any(|case| case.as_str() == Some(TASK_ID))
    {
        return Err(H001Error::Validation(
            "h001 is not in the frozen pilot case set".to_string(),
        ));
    }
    let pilot_arms = artifacts
        .pilot_manifest
        .get("arms")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("pilot_manifest.arms"))?;
    if pilot_arms != &vec![json!("BASELINE"), json!("NO_OP"), json!("INTERVENTION")] {
        return Err(H001Error::Validation(
            "pilot arm set does not match h001 authorization".to_string(),
        ));
    }
    let source_code = fs::read(workspace_path(
        "crates/prefixity-controlled-benchmark/src/phase1b9.rs",
    ))?;
    let expected_source_file_sha = artifacts
        .source_manifest
        .pointer("/source_revision/source_file_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("source_manifest.source_revision.source_file_sha256"))?;
    if sha256_hex(&source_code) != expected_source_file_sha {
        return Err(H001Error::Validation(
            "h001 source revision file hash mismatch".to_string(),
        ));
    }

    let manifest_bindings = [
        ("task_artifact_sha256", &artifacts.task),
        ("source_manifest_sha256", &artifacts.source_manifest),
        ("required_state_sha256", &artifacts.required_state),
        ("tool_contract_sha256", &artifacts.tool_contract),
        ("evaluator_sha256", &artifacts.evaluator),
        ("arm_materialization_sha256", &artifacts.arms),
    ];
    for (field, artifact) in manifest_bindings {
        let expected = artifacts
            .manifest
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(&format!("manifest.{field}")))?;
        let actual = canonical_hash(artifact)?;
        if expected != actual {
            return Err(H001Error::Validation(format!("manifest.{field} mismatch")));
        }
    }
    for (artifact_path, sidecar_path, artifact) in [
        (
            H001_TASK_PATH,
            "docs/phase-1/PHASE_1C_H001_SCORED_TASK_V1.sha256",
            &artifacts.task,
        ),
        (
            H001_SOURCE_MANIFEST_PATH,
            "docs/phase-1/PHASE_1C_H001_SCORED_SOURCE_MANIFEST_V1.sha256",
            &artifacts.source_manifest,
        ),
        (
            H001_REQUIRED_STATE_PATH,
            "docs/phase-1/PHASE_1C_H001_REQUIRED_STATE_V1.sha256",
            &artifacts.required_state,
        ),
        (
            H001_TOOL_CONTRACT_PATH,
            "docs/phase-1/PHASE_1C_H001_TOOL_CONTRACT_V1.sha256",
            &artifacts.tool_contract,
        ),
        (
            H001_EVALUATOR_PATH,
            "docs/phase-1/PHASE_1C_H001_EVALUATOR_V1.sha256",
            &artifacts.evaluator,
        ),
        (
            H001_ARM_PROJECTIONS_PATH,
            "docs/phase-1/PHASE_1C_H001_ARM_PROJECTIONS_V1.sha256",
            &artifacts.arms,
        ),
        (
            H001_MANIFEST_PATH,
            "docs/phase-1/PHASE_1C_H001_SCORED_TASK_MANIFEST_V1.sha256",
            &artifacts.manifest,
        ),
    ] {
        validate_sidecar(artifact_path, sidecar_path, artifact)?;
    }
    if artifacts
        .manifest
        .get("runtime_contract_sha256")
        .and_then(Value::as_str)
        != Some(contract_sha.as_str())
    {
        return Err(H001Error::Validation(
            "manifest runtime contract binding mismatch".to_string(),
        ));
    }
    if artifacts
        .manifest
        .get("pilot_manifest_sha256")
        .and_then(Value::as_str)
        != Some(pilot_sha.as_str())
    {
        return Err(H001Error::Validation(
            "h001 manifest pilot binding mismatch".to_string(),
        ));
    }

    validate_source_records(&artifacts.source_manifest)?;
    validate_task(&artifacts.task, &artifacts.source_manifest)?;
    validate_hidden_state(
        &artifacts.required_state,
        &artifacts.evaluator,
        &artifacts.task,
    )?;
    validate_tool_contract(&artifacts.tool_contract)?;
    let prompt_sha = prompt_sha256(&artifacts.task)?;
    if artifacts
        .manifest
        .get("task_prompt_sha256")
        .and_then(Value::as_str)
        != Some(prompt_sha.as_str())
    {
        return Err(H001Error::Validation(
            "manifest task prompt fingerprint mismatch".to_string(),
        ));
    }
    validate_arms(artifacts, &prompt_sha)?;
    Ok(())
}

fn validate_source_records(source_manifest: &Value) -> Result<(), H001Error> {
    let records = source_manifest
        .get("source_records")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("source_manifest.source_records"))?;
    let mut ids = BTreeMap::new();
    for record in records {
        let id = record
            .get("source_id")
            .and_then(Value::as_str)
            .ok_or_else(|| missing("source_id"))?;
        let content = record
            .get("exact_content")
            .and_then(Value::as_str)
            .ok_or_else(|| missing("exact_content"))?;
        let expected = record
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| missing("sha256"))?;
        if sha256_hex(content.as_bytes()) != expected {
            return Err(H001Error::Validation(format!(
                "source hash mismatch for {id}"
            )));
        }
        if ids.insert(id, record).is_some() {
            return Err(H001Error::Validation(format!("duplicate source id {id}")));
        }
    }
    if records.len() != 7 {
        return Err(H001Error::Validation(
            "h001 source record count is not 7".to_string(),
        ));
    }
    Ok(())
}

fn validate_task(task: &Value, source_manifest: &Value) -> Result<(), H001Error> {
    let sources = source_manifest
        .get("source_records")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("source_records"))?;
    let model_ids = task
        .get("source_ids_order")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("task.source_ids_order"))?;
    let hidden_ids = task
        .get("evaluator_only_source_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("task.evaluator_only_source_ids"))?;
    if hidden_ids.iter().any(|id| model_ids.contains(id)) {
        return Err(H001Error::Validation(
            "evaluator-only source leaked into task source order".to_string(),
        ));
    }
    for id in model_ids.iter().chain(hidden_ids.iter()) {
        if !sources
            .iter()
            .any(|record| record.get("source_id") == Some(id))
        {
            return Err(H001Error::Validation(
                "task references unknown source id".to_string(),
            ));
        }
    }
    let messages = task
        .pointer("/model_visible/messages")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("task.model_visible.messages"))?;
    if messages.len() != 2
        || messages[0].get("role").and_then(Value::as_str) != Some("system")
        || messages[1].get("role").and_then(Value::as_str) != Some("user")
    {
        return Err(H001Error::Validation(
            "h001 must have exactly one system and one user message".to_string(),
        ));
    }
    let hidden_relation = sources
        .iter()
        .find(|record| {
            record.get("source_id").and_then(Value::as_str) == Some("h001-relation-q001")
        })
        .ok_or_else(|| missing("h001-relation-q001"))?;
    let user_content = messages[1]
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("task user content"))?;
    if user_content.contains(
        hidden_relation
            .get("exact_content")
            .and_then(Value::as_str)
            .unwrap_or(""),
    ) {
        return Err(H001Error::Validation(
            "evaluator-only relation content leaked into task prompt".to_string(),
        ));
    }
    Ok(())
}

fn validate_hidden_state(
    required_state: &Value,
    evaluator: &Value,
    task: &Value,
) -> Result<(), H001Error> {
    if required_state.get("visibility").and_then(Value::as_str) != Some("evaluator_only")
        || evaluator.get("visibility").and_then(Value::as_str) != Some("evaluator_only")
    {
        return Err(H001Error::Validation(
            "required state and evaluator must be evaluator-only".to_string(),
        ));
    }
    let expected = evaluator
        .pointer("/expected_final_answer")
        .ok_or_else(|| missing("evaluator.expected_final_answer"))?;
    let messages = task
        .pointer("/model_visible/messages")
        .ok_or_else(|| missing("task messages"))?;
    if messages.to_string().contains(&expected.to_string()) {
        return Err(H001Error::Validation(
            "evaluator answer object leaked into model-visible messages".to_string(),
        ));
    }
    Ok(())
}

fn validate_tool_contract(tool_contract: &Value) -> Result<(), H001Error> {
    let tools = tool_contract
        .get("visible_tools")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("tool_contract.visible_tools"))?;
    if !tools.is_empty()
        || tool_contract
            .get("same_across_arms")
            .and_then(Value::as_bool)
            != Some(true)
    {
        return Err(H001Error::Validation(
            "h001 tool contract is not the frozen no-tool contract".to_string(),
        ));
    }
    Ok(())
}

fn validate_arms(artifacts: &H001Artifacts, prompt_sha: &str) -> Result<(), H001Error> {
    let details = arm_details(&artifacts.arms)?;
    if details.len() != 3 {
        return Err(H001Error::Validation(
            "h001 must materialize exactly three arms".to_string(),
        ));
    }
    let baseline = details
        .iter()
        .find(|(arm, _, _)| arm == "BASELINE")
        .ok_or_else(|| missing("BASELINE arm"))?;
    let no_op = details
        .iter()
        .find(|(arm, _, _)| arm == "NO_OP")
        .ok_or_else(|| missing("NO_OP arm"))?;
    let intervention = details
        .iter()
        .find(|(arm, _, _)| arm == "INTERVENTION")
        .ok_or_else(|| missing("INTERVENTION arm"))?;
    if canonical_json(&baseline.1)? != canonical_json(&no_op.1)? {
        return Err(H001Error::Validation(
            "BASELINE and NO_OP requests are not byte-equivalent by canonical identity".to_string(),
        ));
    }
    if canonical_json(&baseline.1)? == canonical_json(&intervention.1)? {
        return Err(H001Error::Validation(
            "INTERVENTION request did not differ from BASELINE".to_string(),
        ));
    }
    for (arm, request, _) in &details {
        expect_string(request, "model", MODEL)?;
        expect_u64(request, "temperature", 0)?;
        expect_u64(request, "top_p", 1)?;
        expect_u64(request, "max_tokens", 2048)?;
        expect_bool(request, "stream", false)?;
        expect_u64(request, "seed", 1)?;
        if request.get("tools").is_some() || request.to_string().contains("reasoning_content") {
            return Err(H001Error::Validation(
                "model request contains an undeclared tool or reasoning field".to_string(),
            ));
        }
        let messages = request
            .get("messages")
            .and_then(Value::as_array)
            .ok_or_else(|| missing("request.messages"))?;
        if messages.len() != 2
            || messages[0].get("role").and_then(Value::as_str) != Some("system")
            || messages[1].get("role").and_then(Value::as_str) != Some("user")
        {
            return Err(H001Error::Validation(
                "arm request message shape mismatch".to_string(),
            ));
        }
        if arm == "BASELINE" && canonical_hash(messages)? != prompt_sha {
            return Err(H001Error::Validation(
                "BASELINE prompt fingerprint mismatch".to_string(),
            ));
        }
    }
    if intervention
        .2
        .iter()
        .any(|source| source == "h001-event-e002-content")
    {
        return Err(H001Error::Validation(
            "INTERVENTION retained the frozen duplicate target".to_string(),
        ));
    }
    let expected = artifacts
        .manifest
        .get("arm_projection_sha256")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("manifest.arm_projection_sha256"))?;
    for (arm, request, _) in details {
        let expected_hash = expected
            .get(&arm)
            .and_then(Value::as_str)
            .ok_or_else(|| missing("arm projection hash"))?;
        if expected_hash != canonical_hash(&request)? {
            return Err(H001Error::Validation(format!(
                "projection identity mismatch for {arm}"
            )));
        }
    }
    Ok(())
}

fn arm_details(arms: &Value) -> Result<Vec<(String, Value, Vec<String>)>, H001Error> {
    arms.get("arms")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("arms.arms"))?
        .iter()
        .map(|arm| {
            let name = arm
                .get("arm")
                .and_then(Value::as_str)
                .ok_or_else(|| missing("arm"))?
                .to_string();
            let request = arm
                .get("request")
                .cloned()
                .ok_or_else(|| missing("arm.request"))?;
            let source_ids = arm
                .get("source_ids")
                .and_then(Value::as_array)
                .ok_or_else(|| missing("arm.source_ids"))?
                .iter()
                .map(|id| {
                    id.as_str()
                        .map(str::to_string)
                        .ok_or_else(|| missing("source id"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((name, request, source_ids))
        })
        .collect()
}

fn request_for_arm(arms: &Value, requested: H001Arm) -> Result<Value, H001Error> {
    let name = requested.as_str();
    arms.get("arms")
        .and_then(Value::as_array)
        .and_then(|values| {
            values
                .iter()
                .find(|arm| arm.get("arm").and_then(Value::as_str) == Some(name))
        })
        .and_then(|arm| arm.get("request"))
        .cloned()
        .ok_or_else(|| H001Error::Validation(format!("missing arm projection for {name}")))
}

fn arm_source_ids(arms: &Value, requested: H001Arm) -> Result<Vec<String>, H001Error> {
    let name = requested.as_str();
    arms.get("arms")
        .and_then(Value::as_array)
        .and_then(|values| {
            values
                .iter()
                .find(|arm| arm.get("arm").and_then(Value::as_str) == Some(name))
        })
        .and_then(|arm| arm.get("source_ids"))
        .and_then(Value::as_array)
        .ok_or_else(|| H001Error::Validation(format!("missing source ids for {name}")))?
        .iter()
        .map(|id| {
            id.as_str()
                .map(str::to_string)
                .ok_or_else(|| missing("source id"))
        })
        .collect()
}

fn prompt_sha256(task: &Value) -> Result<String, H001Error> {
    let messages = task
        .pointer("/model_visible/messages")
        .ok_or_else(|| missing("task.model_visible.messages"))?;
    Ok(canonical_hash(messages)?)
}

fn ambiguous_result(
    arm: H001Arm,
    request_sha256: &str,
    wire_request_sha256: &str,
    request_bytes: usize,
    readiness_elapsed_ms: u64,
    transport_elapsed_ms: u64,
    error: String,
) -> Value {
    json!({
        "schema_id": "prefixity.phase1c.h001.arm-result",
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "task_id": TASK_ID,
        "arm": arm.as_str(),
        "replicate": 1,
        "state": "AMBIGUOUS",
        "request": {
            "turns": 0,
            "max_turns": MAX_TURNS,
            "transport_attempts": 1,
            "inference_requests": 1,
            "automatic_retries": 0,
            "request_sha256": request_sha256,
            "wire_request_sha256": wire_request_sha256,
            "request_body_bytes": request_bytes,
            "request_file": "request-turn-1.json"
        },
        "readiness": {"listener_check_attempts": 1, "inference_requests": 0, "elapsed_ms": readiness_elapsed_ms},
        "response": {"complete": false, "http_status": null, "response_body_bytes": null, "response_body_sha256": null, "response_body_file": null, "transport_elapsed_ms": transport_elapsed_ms},
        "validation": {"response_json_parsed": false, "final_content_available": false, "terminal_final_content": false, "reasoning_diagnostic_only": true, "error": error},
        "next_arm": null
    })
}

fn persist_ambiguous(evidence_dir: &Path, result: &Value) -> Result<(), H001Error> {
    write_json(
        &evidence_dir.join("trajectory-state.json"),
        &json!({"task_id": TASK_ID, "state": "AMBIGUOUS", "completed_turns": 0, "max_turns": MAX_TURNS, "next_arm": null}),
    )?;
    write_json(&evidence_dir.join("arm-result.json"), result)
}

fn arm_evidence_dir(arm: H001Arm) -> PathBuf {
    workspace_path(H001_EVIDENCE_ROOT)
        .join("replicate-1")
        .join(arm.as_str().to_ascii_lowercase())
}

fn safe_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_string(), value.to_string()))
        })
        .collect()
}

fn read_json(path: &Path) -> Result<Value, H001Error> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn read_repo_json(path: &str) -> Result<Value, H001Error> {
    read_json(&workspace_path(path))
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn validate_sidecar(
    artifact_path: &str,
    sidecar_path: &str,
    artifact: &Value,
) -> Result<(), H001Error> {
    let sidecar = read_repo_json(sidecar_path)?;
    if sidecar.get("artifact_path").and_then(Value::as_str) != Some(artifact_path)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(artifact)?.as_str())
    {
        return Err(H001Error::Validation(format!(
            "artifact fingerprint sidecar mismatch for {artifact_path}"
        )));
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<(), H001Error> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), H001Error> {
    fs::write(path, bytes)?;
    Ok(())
}

fn contract_value<'a>(contract: &'a Value, path: &str) -> Result<&'a Value, H001Error> {
    contract
        .pointer(&format!("/{}", path.replace('.', "/")))
        .ok_or_else(|| missing(path))
}

fn contract_u64(contract: &Value, path: &str) -> Result<u64, H001Error> {
    contract_value(contract, path)?
        .as_u64()
        .ok_or_else(|| H001Error::Validation(format!("{path} is not an integer")))
}

fn expect_string(value: &Value, path: &str, expected: &str) -> Result<(), H001Error> {
    if contract_value(value, path)?.as_str() != Some(expected) {
        return Err(H001Error::Validation(format!("{path} is not {expected}")));
    }
    Ok(())
}

fn expect_u64(value: &Value, path: &str, expected: u64) -> Result<(), H001Error> {
    if contract_value(value, path)?.as_u64() != Some(expected) {
        return Err(H001Error::Validation(format!("{path} is not {expected}")));
    }
    Ok(())
}

fn expect_bool(value: &Value, path: &str, expected: bool) -> Result<(), H001Error> {
    if contract_value(value, path)?.as_bool() != Some(expected) {
        return Err(H001Error::Validation(format!("{path} is not {expected}")));
    }
    Ok(())
}

fn missing(path: &str) -> H001Error {
    H001Error::Validation(format!("missing {path}"))
}

fn now_unix_ms() -> Result<u128, H001Error> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| H001Error::Validation(format!("system clock before Unix epoch: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h001_preflight_is_offline_and_complete() {
        let result = preflight_h001().unwrap();
        assert_eq!(result["state"], "PREPARED");
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["inference_requests"], 0);
        assert_eq!(result["arm_projection_sha256"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn dry_run_never_contacts_transport() {
        let result = dry_run_h001(H001Arm::Baseline).unwrap();
        assert_eq!(result["state"], "DRY_RUN");
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["inference_requests"], 0);
    }

    #[test]
    fn baseline_and_noop_are_identical_but_intervention_prunes_only_duplicate() {
        let artifacts = load_artifacts().unwrap();
        validate_artifacts(&artifacts).unwrap();
        let baseline = request_for_arm(&artifacts.arms, H001Arm::Baseline).unwrap();
        let no_op = request_for_arm(&artifacts.arms, H001Arm::NoOp).unwrap();
        let intervention = request_for_arm(&artifacts.arms, H001Arm::Intervention).unwrap();
        assert_eq!(
            canonical_json(&baseline).unwrap(),
            canonical_json(&no_op).unwrap()
        );
        assert_ne!(
            canonical_json(&baseline).unwrap(),
            canonical_json(&intervention).unwrap()
        );
        assert!(intervention.to_string().contains("e001"));
        assert!(!intervention.to_string().contains("e002"));
    }

    #[test]
    fn evaluator_key_and_reasoning_are_separate_from_prompt_and_score() {
        let artifacts = load_artifacts().unwrap();
        validate_artifacts(&artifacts).unwrap();
        let expected = artifacts
            .evaluator
            .pointer("/expected_final_answer")
            .unwrap()
            .clone();
        let final_content = serde_json::to_string(&expected).unwrap();
        assert!(!artifacts
            .task
            .pointer("/model_visible/messages")
            .unwrap()
            .to_string()
            .contains(&expected.to_string()));
        assert_eq!(expected["required_event_id"], "e001");
        assert!(!final_content.is_empty());
    }
}
