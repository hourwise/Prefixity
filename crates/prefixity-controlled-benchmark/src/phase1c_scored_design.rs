//! Offline validator for the Phase 1C scored-runtime design.
//!
//! This module only reads checked-in contract/manifest artifacts. It does not
//! open a socket, start a runtime, read credentials, or inspect live evidence.
//! It exists to make the design boundary auditable without recreating the
//! Defender-triggering inline forensic workflow.

use crate::hashing::canonical_hash;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const SCORED_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.json";
pub const SCORED_CONTRACT_SCHEMA_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.schema.json";
pub const SCORED_PILOT_MANIFEST_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.json";
pub const SCORED_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V1.sha256";
pub const SCORED_PILOT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.sha256";

const EXPECTED_CONTRACT_VERSION: &str = "phase1c-scored-runtime-local-qwen-v1";
const EXPECTED_EXPERIMENT_ID: &str = "phase-1c-scored-capability-v1";
const EXPECTED_MODEL: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
const EXPECTED_ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
const EXPECTED_PILOT_CASES: [&str; 6] = ["h001", "h004", "h006", "h007", "h009", "h010"];
const EXPECTED_ARMS: [&str; 3] = ["BASELINE", "NO_OP", "INTERVENTION"];

#[derive(Debug, thiserror::Error)]
pub enum ScoredDesignError {
    #[error("scored design file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("scored design JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("scored design validation failed: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoredDesignFingerprint {
    pub state: String,
    pub contract_sha256: String,
    pub contract_schema_sha256: String,
    pub pilot_manifest_sha256: String,
    pub pilot_case_count: usize,
    pub pilot_arm_count: usize,
    pub pilot_replicate_count: usize,
    pub pilot_max_model_inference_requests: u64,
    pub full_cohort_max_model_inference_requests: u64,
    pub network_calls: u32,
    pub credential_reads: u32,
    pub inference_requests: u32,
}

pub fn fingerprint_scored_design() -> Result<ScoredDesignFingerprint, ScoredDesignError> {
    let contract = read_json(SCORED_CONTRACT_PATH)?;
    let schema = read_json(SCORED_CONTRACT_SCHEMA_PATH)?;
    let manifest = read_json(SCORED_PILOT_MANIFEST_PATH)?;
    let (pilot_case_count, pilot_arm_count, pilot_replicate_count, pilot_max, full_max) =
        manifest_counts(&manifest)?;
    Ok(ScoredDesignFingerprint {
        state: "FINGERPRINTED_OFFLINE".to_string(),
        contract_sha256: canonical_hash(&contract)?,
        contract_schema_sha256: canonical_hash(&schema)?,
        pilot_manifest_sha256: canonical_hash(&manifest)?,
        pilot_case_count,
        pilot_arm_count,
        pilot_replicate_count,
        pilot_max_model_inference_requests: pilot_max,
        full_cohort_max_model_inference_requests: full_max,
        network_calls: 0,
        credential_reads: 0,
        inference_requests: 0,
    })
}

pub fn validate_scored_design() -> Result<ScoredDesignFingerprint, ScoredDesignError> {
    let contract = read_json(SCORED_CONTRACT_PATH)?;
    let schema = read_json(SCORED_CONTRACT_SCHEMA_PATH)?;
    let manifest = read_json(SCORED_PILOT_MANIFEST_PATH)?;
    validate_contract(&contract)?;
    validate_schema(&schema)?;
    validate_manifest(&manifest)?;

    let contract_sha256 = canonical_hash(&contract)?;
    let schema_sha256 = canonical_hash(&schema)?;
    let manifest_sha256 = canonical_hash(&manifest)?;
    let manifest_contract_sha256 = string_at(&manifest, "runtime_contract_sha256")?;
    if manifest_contract_sha256 != contract_sha256 {
        return Err(ScoredDesignError::Validation(
            "pilot manifest does not bind the scored runtime contract hash".to_string(),
        ));
    }
    validate_sidecar(
        SCORED_CONTRACT_FINGERPRINT_PATH,
        SCORED_CONTRACT_PATH,
        &contract_sha256,
    )?;
    validate_sidecar(
        SCORED_PILOT_FINGERPRINT_PATH,
        SCORED_PILOT_MANIFEST_PATH,
        &manifest_sha256,
    )?;

    let (pilot_case_count, pilot_arm_count, pilot_replicate_count, pilot_max, full_max) =
        manifest_counts(&manifest)?;
    Ok(ScoredDesignFingerprint {
        state: "VALIDATED_OFFLINE".to_string(),
        contract_sha256,
        contract_schema_sha256: schema_sha256,
        pilot_manifest_sha256: manifest_sha256,
        pilot_case_count,
        pilot_arm_count,
        pilot_replicate_count,
        pilot_max_model_inference_requests: pilot_max,
        full_cohort_max_model_inference_requests: full_max,
        network_calls: 0,
        credential_reads: 0,
        inference_requests: 0,
    })
}

fn validate_contract(value: &Value) -> Result<(), ScoredDesignError> {
    expect_string(value, "contract_version", EXPECTED_CONTRACT_VERSION)?;
    expect_string(value, "experiment_id", EXPECTED_EXPERIMENT_ID)?;
    expect_string(value, "status", "DESIGN_ONLY_NO_INFERENCE_AUTHORIZED")?;
    expect_string(value, "engine", "llama.cpp")?;
    expect_string(value, "runtime", "llama-server")?;
    expect_string(
        value,
        "api_surface",
        "llama.cpp-openai-compatible-chat-completions-v1",
    )?;
    expect_string(value, "endpoint", EXPECTED_ENDPOINT)?;
    expect_string(value, "host", "127.0.0.1")?;
    expect_u64(value, "port", 8080)?;
    expect_string(value, "model", EXPECTED_MODEL)?;
    expect_string(value, "quantization", "Q4_0")?;
    expect_u64(value, "context_size", 8192)?;
    expect_u64(value, "parallel_slots", 1)?;
    expect_string(value, "metrics", "enabled")?;
    expect_string(value, "runtime_settings.reasoning", "on")?;
    expect_bool(
        value,
        "runtime_settings.same_for_all_arms_and_replicates",
        true,
    )?;
    expect_u64(value, "generation.max_tokens", 2048)?;
    expect_number(value, "generation.temperature", 0.0)?;
    expect_number(value, "generation.top_p", 1.0)?;
    expect_bool(value, "generation.stream", false)?;
    expect_u64(value, "generation.seed.value", 1)?;
    expect_string(value, "generation.seed.status", "must_be_applied_or_abort")?;
    expect_u64(value, "timeout_policy.connect_timeout_ms", 1000)?;
    expect_u64(value, "timeout_policy.complete_request_timeout_ms", 600000)?;
    expect_u64(value, "timeout_policy.supervisor_timeout_ms", 660000)?;
    expect_u64(value, "retry_policy.automatic_retries", 0)?;
    expect_u64(value, "retry_policy.fallback_requests", 0)?;
    expect_u64(value, "retry_policy.adaptive_replicates", 0)?;
    expect_u64(
        value,
        "request_ceiling.per_arm_replicate_max_model_turns",
        3,
    )?;
    expect_u64(
        value,
        "request_ceiling.pilot_max_model_inference_requests",
        54,
    )?;
    expect_u64(
        value,
        "request_ceiling.full_cohort_planned_max_model_inference_requests",
        216,
    )?;
    expect_bool(
        value,
        "freshness_cache_isolation.fresh_server_process_per_arm_replicate",
        true,
    )?;
    expect_bool(
        value,
        "freshness_cache_isolation.zero_inference_requests_since_startup",
        true,
    )?;
    expect_bool(value, "freshness_cache_isolation.warmup", false)?;
    expect_string(
        value,
        "response_extraction.final_content_json_pointer",
        "/choices/0/message/content",
    )?;
    expect_string(
        value,
        "response_extraction.reasoning_content_json_pointer",
        "/choices/0/message/reasoning_content",
    )?;
    expect_bool(
        value,
        "response_extraction.final_content_required_for_terminal_turn",
        true,
    )?;
    for field in [
        "fed_to_later_turns",
        "fed_to_later_arms",
        "fed_to_prefixity_treatment",
        "fed_to_evaluator",
        "affects_task_identity",
        "affects_arm_identity",
        "affects_treatment_identity",
        "affects_expected_answer_or_hidden_labels",
    ] {
        expect_bool(value, &format!("reasoning_isolation.{field}"), false)?;
    }
    expect_bool(
        value,
        "reasoning_isolation.retained_separately_from_final_answer",
        true,
    )?;
    expect_bool(value, "credential_policy.required", false)?;
    expect_bool(value, "credential_policy.provider_api_key_read", false)?;
    expect_bool(value, "candidate.production_behavior_change", false)?;
    expect_string(
        value,
        "scoring_boundary.evaluator_version",
        "stage0-deterministic-evaluator-v1",
    )?;
    expect_string(value, "scoring_boundary.reasoning_score", "none")?;
    Ok(())
}

fn validate_schema(value: &Value) -> Result<(), ScoredDesignError> {
    expect_string(value, "$id", "prefixity.phase1c.scored-runtime-contract-v1")?;
    expect_string(
        value,
        "title",
        "Prefixity Phase 1C scored runtime contract v1",
    )?;
    expect_string(value, "type", "object")?;
    for field in [
        "contract_version",
        "experiment_id",
        "status",
        "engine",
        "model",
        "quantization",
        "context_size",
        "parallel_slots",
        "metrics",
        "runtime_settings",
        "generation",
        "timeout_policy",
        "retry_policy",
        "freshness_cache_isolation",
        "response_extraction",
        "reasoning_isolation",
        "scoring_boundary",
        "arm_matching_requirements",
        "candidate",
        "cohort_inputs",
    ] {
        required_contains(value, field)?;
    }
    Ok(())
}

fn validate_manifest(value: &Value) -> Result<(), ScoredDesignError> {
    expect_string(
        value,
        "manifest_version",
        "phase1c-scored-pilot-manifest-v1",
    )?;
    expect_string(value, "experiment_id", EXPECTED_EXPERIMENT_ID)?;
    expect_string(
        value,
        "status",
        "PREREGISTERED_DESIGN_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(value, "runtime_contract_path", SCORED_CONTRACT_PATH)?;
    if string_at(value, "runtime_contract_sha256")? == "__FILLED_BY_OFFLINE_VALIDATOR__" {
        return Err(ScoredDesignError::Validation(
            "pilot manifest still contains the contract hash placeholder".to_string(),
        ));
    }
    expect_string(
        value,
        "candidate_source_commit",
        "748e4673e8454d2ac3e27cefabee9259992038aa",
    )?;
    expect_string(
        value,
        "design_base_commit",
        "fea5b829a7c80bd3ad5feb23318309fecaf7a3ad",
    )?;
    let pilot_cases = array_strings_at(value, "cohort.pilot_case_ids")?;
    let expected_pilot_cases = EXPECTED_PILOT_CASES
        .iter()
        .map(|case_id| case_id.to_string())
        .collect::<Vec<_>>();
    if pilot_cases != expected_pilot_cases {
        return Err(ScoredDesignError::Validation(format!(
            "pilot case IDs differ from the preregistered order: {pilot_cases:?}"
        )));
    }
    let eligible = array_strings_at(value, "cohort.eligible_population")?;
    let expected_eligible = [
        "h001", "h002", "h003", "h004", "h005", "h006", "h007", "h008", "h009", "h010", "h011",
        "h012",
    ]
    .iter()
    .map(|case_id| case_id.to_string())
    .collect::<Vec<_>>();
    if eligible != expected_eligible {
        return Err(ScoredDesignError::Validation(
            "eligible population is not the frozen h001-h012 cohort".to_string(),
        ));
    }
    let arms = array_strings_at(value, "arms")?;
    let expected_arms = EXPECTED_ARMS
        .iter()
        .map(|arm| arm.to_string())
        .collect::<Vec<_>>();
    if arms != expected_arms {
        return Err(ScoredDesignError::Validation(
            "arm order is not BASELINE, NO_OP, INTERVENTION".to_string(),
        ));
    }
    let replicates = array_u64_at(value, "replicates")?;
    if replicates != vec![1] {
        return Err(ScoredDesignError::Validation(
            "pilot replicate policy is not exactly replicate 1".to_string(),
        ));
    }
    expect_u64(value, "max_model_turns_per_arm_replicate", 3)?;
    expect_u64(value, "max_model_inference_requests", 54)?;
    let execution_order = value_at(value, "execution_order")?
        .as_array()
        .ok_or_else(|| invalid_type("execution_order", "array"))?;
    if execution_order.len() != EXPECTED_PILOT_CASES.len() * EXPECTED_ARMS.len() {
        return Err(ScoredDesignError::Validation(
            "execution order does not contain exactly 18 arm boundaries".to_string(),
        ));
    }
    for (index, expected_case) in EXPECTED_PILOT_CASES.iter().enumerate() {
        for (arm_index, expected_arm) in EXPECTED_ARMS.iter().enumerate() {
            let item = &execution_order[index * EXPECTED_ARMS.len() + arm_index];
            expect_string(item, "case_id", expected_case)?;
            expect_u64(item, "replicate", 1)?;
            expect_string(item, "arm", expected_arm)?;
        }
    }
    expect_bool(
        value,
        "session_isolation.fresh_server_process_per_arm_replicate",
        true,
    )?;
    expect_bool(
        value,
        "session_isolation.zero_inference_since_startup",
        true,
    )?;
    expect_bool(value, "session_isolation.no_http_probe_or_warmup", true)?;
    expect_bool(value, "session_isolation.no_cache_carryover", true)?;
    expect_bool(value, "scoring.evaluation_key_sidecar_only", true)?;
    expect_bool(value, "scoring.baseline_no_op_equivalence_required", true)?;
    expect_bool(value, "scoring.intervention_diff_validator_required", true)?;
    expect_bool(
        value,
        "pilot_acceptance.does_not_authorize_full_cohort",
        true,
    )?;
    expect_bool(
        value,
        "pilot_acceptance.scaling_requires_separate_authorization",
        true,
    )?;
    expect_u64(
        value,
        "full_programme_boundary.maximum_model_inference_requests",
        216,
    )?;
    Ok(())
}

fn manifest_counts(value: &Value) -> Result<(usize, usize, usize, u64, u64), ScoredDesignError> {
    let cases = array_strings_at(value, "cohort.pilot_case_ids")?;
    let arms = array_strings_at(value, "arms")?;
    let replicates = array_u64_at(value, "replicates")?;
    let pilot_max = value_at(value, "max_model_inference_requests")?
        .as_u64()
        .ok_or_else(|| invalid_type("max_model_inference_requests", "unsigned integer"))?;
    let full_max = value_at(
        value,
        "full_programme_boundary.maximum_model_inference_requests",
    )?
    .as_u64()
    .ok_or_else(|| {
        invalid_type(
            "full_programme_boundary.maximum_model_inference_requests",
            "unsigned integer",
        )
    })?;
    Ok((
        cases.len(),
        arms.len(),
        replicates.len(),
        pilot_max,
        full_max,
    ))
}

fn validate_sidecar(
    sidecar_path: &str,
    artifact_path: &str,
    expected_hash: &str,
) -> Result<(), ScoredDesignError> {
    let sidecar = read_json(sidecar_path)?;
    expect_string(&sidecar, "artifact_path", artifact_path)?;
    expect_string(&sidecar, "algorithm", "SHA-256")?;
    expect_string(
        &sidecar,
        "canonicalization",
        "sorted JSON object keys; arrays preserve order",
    )?;
    expect_string(&sidecar, "canonical_sha256", expected_hash)?;
    if expected_hash.len() != 64
        || !expected_hash
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(ScoredDesignError::Validation(format!(
            "invalid canonical SHA-256 for {artifact_path}"
        )));
    }
    Ok(())
}

fn read_json(path: &str) -> Result<Value, ScoredDesignError> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn value_at<'a>(value: &'a Value, path: &str) -> Result<&'a Value, ScoredDesignError> {
    path.split('.').try_fold(value, |current, segment| {
        current
            .get(segment)
            .ok_or_else(|| ScoredDesignError::Validation(format!("missing required field {path}")))
    })
}

fn string_at(value: &Value, path: &str) -> Result<String, ScoredDesignError> {
    Ok(value_at(value, path)?
        .as_str()
        .ok_or_else(|| invalid_type(path, "string"))?
        .to_string())
}

fn array_strings_at(value: &Value, path: &str) -> Result<Vec<String>, ScoredDesignError> {
    let array = value_at(value, path)?
        .as_array()
        .ok_or_else(|| invalid_type(path, "array"))?;
    array
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| invalid_type(path, "array of strings"))
        })
        .collect()
}

fn array_u64_at(value: &Value, path: &str) -> Result<Vec<u64>, ScoredDesignError> {
    let array = value_at(value, path)?
        .as_array()
        .ok_or_else(|| invalid_type(path, "array"))?;
    array
        .iter()
        .map(|item| {
            item.as_u64()
                .ok_or_else(|| invalid_type(path, "array of unsigned integers"))
        })
        .collect()
}

fn expect_string(value: &Value, path: &str, expected: &str) -> Result<(), ScoredDesignError> {
    let actual = string_at(value, path)?;
    if actual != expected {
        return Err(ScoredDesignError::Validation(format!(
            "{path} expected {expected:?}, got {actual:?}"
        )));
    }
    Ok(())
}

fn expect_u64(value: &Value, path: &str, expected: u64) -> Result<(), ScoredDesignError> {
    let actual = value_at(value, path)?
        .as_u64()
        .ok_or_else(|| invalid_type(path, "unsigned integer"))?;
    if actual != expected {
        return Err(ScoredDesignError::Validation(format!(
            "{path} expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn expect_number(value: &Value, path: &str, expected: f64) -> Result<(), ScoredDesignError> {
    let actual = value_at(value, path)?
        .as_f64()
        .ok_or_else(|| invalid_type(path, "number"))?;
    if (actual - expected).abs() > f64::EPSILON {
        return Err(ScoredDesignError::Validation(format!(
            "{path} expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn expect_bool(value: &Value, path: &str, expected: bool) -> Result<(), ScoredDesignError> {
    let actual = value_at(value, path)?
        .as_bool()
        .ok_or_else(|| invalid_type(path, "boolean"))?;
    if actual != expected {
        return Err(ScoredDesignError::Validation(format!(
            "{path} expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn required_contains(value: &Value, expected: &str) -> Result<(), ScoredDesignError> {
    let required = value_at(value, "required")?
        .as_array()
        .ok_or_else(|| invalid_type("required", "array"))?;
    if !required.iter().any(|item| item.as_str() == Some(expected)) {
        return Err(ScoredDesignError::Validation(format!(
            "schema required list is missing {expected}"
        )));
    }
    Ok(())
}

fn invalid_type(path: &str, expected: &str) -> ScoredDesignError {
    ScoredDesignError::Validation(format!("{path} must be {expected}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn checked_in_scored_design_validates_without_live_calls() {
        let report = validate_scored_design().expect("scored design should validate offline");
        assert_eq!(report.state, "VALIDATED_OFFLINE");
        assert_eq!(report.pilot_case_count, 6);
        assert_eq!(report.pilot_arm_count, 3);
        assert_eq!(report.pilot_replicate_count, 1);
        assert_eq!(report.pilot_max_model_inference_requests, 54);
        assert_eq!(report.full_cohort_max_model_inference_requests, 216);
        assert_eq!(report.network_calls, 0);
        assert_eq!(report.credential_reads, 0);
        assert_eq!(report.inference_requests, 0);
    }

    #[test]
    fn pilot_cases_are_unique_and_within_eligible_population() {
        let manifest = read_json(SCORED_PILOT_MANIFEST_PATH).unwrap();
        let cases = array_strings_at(&manifest, "cohort.pilot_case_ids").unwrap();
        let unique = cases.iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), cases.len());
        let eligible = array_strings_at(&manifest, "cohort.eligible_population").unwrap();
        assert!(cases.iter().all(|case_id| eligible.contains(case_id)));
    }
}
