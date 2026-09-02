//! Offline preparation and future execution binding for the h001 timeout-only
//! V2 attempt. V1 remains the default path; this module selects only the V2
//! contract, lineage, and evidence root.

use crate::hashing::{canonical_hash, canonical_json};
use crate::phase1c_h001::{
    arm_evidence_dir_at, dry_run_h001, execute_h001_arm_with_spec, preflight_h001,
    score_h001_arm_at, H001Arm, H001Error, SCORED_CONTRACT_PATH, SCORED_PILOT_MANIFEST_PATH,
};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const V2_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V2.json";
pub const V2_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V2.sha256";
pub const V2_PILOT_MANIFEST_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V2.json";
pub const V2_PILOT_FINGERPRINT_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V2.sha256";
pub const V2_BASELINE_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_H001_V2_BASELINE_IDENTITY_V1.json";
pub const V2_BASELINE_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_H001_V2_BASELINE_IDENTITY_V1.sha256";
pub const V2_EVIDENCE_ROOT: &str = "experiments/runs/phase1c-scored-capability-v2";
pub const V2_H001_EVIDENCE_ROOT: &str = "experiments/runs/phase1c-scored-capability-v2/h001";
pub const V2_H001_ATTEMPT_002_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_IDENTITY_V1.json";
pub const V2_H001_ATTEMPT_002_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_IDENTITY_V1.sha256";
pub const V2_H001_ATTEMPT_002_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline-attempt-002";
pub const V2_LIVE_CONFIRMATION_FLAG: &str = "--confirm-fresh-runtime";

const V2_EXPERIMENT_ID: &str = "phase-1c-scored-capability-v2";
const V2_TASK_ID: &str = "h001";
const V1_CONTRACT_SHA256: &str = "2f05440eb66d195d5362b9ad3a5dfc1708c4b8902cf8cd5774d26076c9aa1520";
const V1_PILOT_MANIFEST_SHA256: &str =
    "201c21d7472dcf274bb912488f9334ac264bbba4968262e4e9fa4939a2597552";
const V1_FORENSIC_REVIEW_SHA256: &str =
    "e0645929c951a842a0b63dc79f1425983d8a734a58e393cf96f2a639306d5e3a";
const V1_BASELINE_REQUEST_SHA256: &str =
    "26bc77415683d81c9f3af4e556151d8abab775b48dd5f4632ed6caba1ad25a2a";
const V1_H001_MANIFEST_SHA256: &str =
    "a107f741a632b0865717b1a81412fbf2dc464e11edb3c90376fe8bd90946b8d8";
const V1_TASK_SHA256: &str = "4597b25cac114899d7e52ff67e7fb2297442c954cb10bffba2aab94cc20ef7c8";
const V1_SOURCE_MANIFEST_SHA256: &str =
    "da397d5ebaa4bbbc48f7f84cb1a0c3f6480d20c7cadfad96231b596b5074cc4a";
const V1_REQUIRED_STATE_SHA256: &str =
    "459f5e093e540e146179399b3263e168642b2044578db59239badc9bfb5e0b8a";
const V1_TOOL_CONTRACT_SHA256: &str =
    "1105e16039acc4d9f1db6d784f6a5eb0653d4601117aeefcc8686338df7552a9";
const V1_EVALUATOR_SHA256: &str =
    "5fc19f47bd36e8625535e63939caebbb0e9ca1a458bb41b357de9d3f6d1f7fe3";
const V1_ARM_MATERIALIZATION_SHA256: &str =
    "1e09aaf51d1b93bddadb54e198b987481490083405a0de8d480b18393cb81416";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V2CliCommand {
    Preflight,
    Fingerprint,
    DryRun,
    Score,
    RunBaseline,
}

pub fn parse_v2_cli_args<I>(args: I) -> Result<V2CliCommand, H001Error>
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "preflight" => Ok(V2CliCommand::Preflight),
        [command] if command == "fingerprint" => Ok(V2CliCommand::Fingerprint),
        [command] if command == "dry-run" => Ok(V2CliCommand::DryRun),
        [command] if command == "score" => Ok(V2CliCommand::Score),
        [command, flag] if command == "run" && flag == V2_LIVE_CONFIRMATION_FLAG => {
            Ok(V2CliCommand::RunBaseline)
        }
        _ => Err(H001Error::Validation(
            "usage: prefixity-phase1c-h001-v2 [preflight|fingerprint|dry-run|score|run --confirm-fresh-runtime]".to_string(),
        )),
    }
}

pub fn v2_live_child_args() -> Vec<String> {
    vec!["run".to_string(), V2_LIVE_CONFIRMATION_FLAG.to_string()]
}

pub fn fingerprint_v2() -> Result<Value, H001Error> {
    let contract = read_repo_json(V2_CONTRACT_PATH)?;
    let manifest = read_repo_json(V2_PILOT_MANIFEST_PATH)?;
    let identity = read_repo_json(V2_BASELINE_IDENTITY_PATH)?;
    let attempt_002_identity = read_repo_json(V2_H001_ATTEMPT_002_IDENTITY_PATH)?;
    let request = baseline_request()?;
    Ok(json!({
        "contract_sha256": canonical_hash(&contract)?,
        "pilot_manifest_sha256": canonical_hash(&manifest)?,
        "baseline_identity_sha256": canonical_hash(&identity)?,
        "attempt_002_identity_sha256": canonical_hash(&attempt_002_identity)?,
        "baseline_request_sha256": canonical_hash(&request)?,
        "baseline_request_bytes": serde_json::to_vec(&request)?.len(),
        "v1_baseline_request_sha256": V1_BASELINE_REQUEST_SHA256,
        "v1_contract_sha256": V1_CONTRACT_SHA256,
        "v1_pilot_manifest_sha256": V1_PILOT_MANIFEST_SHA256,
        "timeout_policy": contract.get("timeout_policy"),
        "network_calls": 0,
        "credential_reads": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_v2() -> Result<Value, H001Error> {
    let v1_preflight = preflight_h001()?;
    let v1_contract = read_repo_json(SCORED_CONTRACT_PATH)?;
    let v1_manifest = read_repo_json(SCORED_PILOT_MANIFEST_PATH)?;
    let v2_contract = read_repo_json(V2_CONTRACT_PATH)?;
    let v2_manifest = read_repo_json(V2_PILOT_MANIFEST_PATH)?;
    let identity = read_repo_json(V2_BASELINE_IDENTITY_PATH)?;
    validate_v2_contract(&v1_contract, &v2_contract)?;
    validate_v2_manifest(&v1_manifest, &v2_manifest, &v2_contract)?;
    validate_v2_identity(&identity, &v2_contract, &v2_manifest, &v1_preflight)?;
    validate_sidecar(V2_CONTRACT_PATH, V2_CONTRACT_FINGERPRINT_PATH, &v2_contract)?;
    validate_sidecar(
        V2_PILOT_MANIFEST_PATH,
        V2_PILOT_FINGERPRINT_PATH,
        &v2_manifest,
    )?;
    validate_sidecar(
        V2_BASELINE_IDENTITY_PATH,
        V2_BASELINE_IDENTITY_FINGERPRINT_PATH,
        &identity,
    )?;
    let request = baseline_request()?;
    if canonical_hash(&request)? != V1_BASELINE_REQUEST_SHA256 {
        return Err(H001Error::Validation(
            "V2 BASELINE request differs from the frozen V1 projection".to_string(),
        ));
    }
    Ok(json!({
        "state": "PREPARED",
        "experiment_id": V2_EXPERIMENT_ID,
        "task_id": V2_TASK_ID,
        "contract_sha256": canonical_hash(&v2_contract)?,
        "pilot_manifest_sha256": canonical_hash(&v2_manifest)?,
        "baseline_identity_sha256": canonical_hash(&identity)?,
        "baseline_request_sha256": canonical_hash(&request)?,
        "baseline_request_bytes": serde_json::to_vec(&request)?.len(),
        "v1_baseline_request_sha256": V1_BASELINE_REQUEST_SHA256,
        "v2_evidence_root": V2_H001_EVIDENCE_ROOT,
        "max_turns": 3,
        "replicate": 1,
        "network_calls": 0,
        "credential_reads": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_v2_attempt_002() -> Result<Value, H001Error> {
    let _ = preflight_v2()?;
    let contract = read_repo_json(V2_CONTRACT_PATH)?;
    let manifest = read_repo_json(V2_PILOT_MANIFEST_PATH)?;
    let identity = read_repo_json(V2_H001_ATTEMPT_002_IDENTITY_PATH)?;
    validate_v2_attempt_002_identity(&identity, &contract, &manifest)?;
    validate_sidecar(
        V2_H001_ATTEMPT_002_IDENTITY_PATH,
        V2_H001_ATTEMPT_002_FINGERPRINT_PATH,
        &identity,
    )?;
    if workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT).exists() {
        return Err(H001Error::Validation(
            "V2 h001 BASELINE attempt-002 evidence already exists".to_string(),
        ));
    }
    let request = baseline_request()?;
    Ok(json!({
        "state": "PREPARED",
        "experiment_id": V2_EXPERIMENT_ID,
        "task_id": V2_TASK_ID,
        "launch_attempt": 2,
        "contract_sha256": canonical_hash(&contract)?,
        "pilot_manifest_sha256": canonical_hash(&manifest)?,
        "attempt_002_identity_sha256": canonical_hash(&identity)?,
        "baseline_request_sha256": canonical_hash(&request)?,
        "baseline_request_bytes": serde_json::to_vec(&request)?.len(),
        "evidence_root": V2_H001_ATTEMPT_002_EVIDENCE_ROOT,
        "max_turns": 3,
        "replicate": 1,
        "network_calls": 0,
        "listener_checks": 0,
        "credential_reads": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_v2_attempt_002() -> Result<Value, H001Error> {
    let preflight = preflight_v2_attempt_002()?;
    let request = baseline_request()?;
    Ok(json!({
        "state": "DRY_RUN",
        "arm": "BASELINE",
        "launch_attempt": 2,
        "preflight": preflight,
        "model_visible_request": request,
        "request_sha256": canonical_hash(&request)?,
        "request_bytes": serde_json::to_vec(&request)?.len(),
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_v2() -> Result<Value, H001Error> {
    let preflight = preflight_v2()?;
    let request = baseline_request()?;
    Ok(json!({
        "state": "DRY_RUN",
        "arm": "BASELINE",
        "preflight": preflight,
        "model_visible_request": request,
        "request_sha256": canonical_hash(&request)?,
        "request_bytes": serde_json::to_vec(&request)?.len(),
        "network_calls": 0,
        "inference_requests": 0
    }))
}

pub fn execute_v2_baseline(confirm_fresh_runtime: bool) -> Result<Value, H001Error> {
    let preflight = preflight_v2_attempt_002()?;
    let contract = read_repo_json(V2_CONTRACT_PATH)?;
    let request = baseline_request()?;
    let evidence_root = workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT);
    execute_h001_arm_with_spec(
        H001Arm::Baseline,
        confirm_fresh_runtime,
        &preflight,
        &contract,
        &request,
        arm_evidence_dir_at(&evidence_root, H001Arm::Baseline),
        V2_EXPERIMENT_ID,
    )
}

pub fn score_v2_baseline() -> Result<Value, H001Error> {
    let _ = validate_v2_attempt_002_identity_files(false)?;
    score_h001_arm_at(
        H001Arm::Baseline,
        arm_evidence_dir_at(
            &workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT),
            H001Arm::Baseline,
        ),
    )
}

fn baseline_request() -> Result<Value, H001Error> {
    let result = dry_run_h001(H001Arm::Baseline)?;
    result
        .get("model_visible_request")
        .cloned()
        .ok_or_else(|| H001Error::Validation("V1 dry-run omitted model request".to_string()))
}

fn validate_v2_contract(v1: &Value, v2: &Value) -> Result<(), H001Error> {
    if canonical_hash(v1)? != V1_CONTRACT_SHA256 {
        return Err(H001Error::Validation(
            "V1 scored contract fingerprint changed".to_string(),
        ));
    }
    expect_string(
        v2,
        "contract_version",
        "phase1c-scored-runtime-local-qwen-v2",
    )?;
    expect_string(v2, "experiment_id", V2_EXPERIMENT_ID)?;
    expect_string(v2, "status", "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED")?;
    expect_string(v2, "model", "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0")?;
    expect_u64(v2, "context_size", 8192)?;
    expect_u64(v2, "parallel_slots", 1)?;
    expect_string(v2, "metrics", "enabled")?;
    expect_string(v2, "runtime_settings.reasoning", "on")?;
    expect_u64(v2, "generation.max_tokens", 2048)?;
    expect_u64(v2, "generation.seed.value", 1)?;
    expect_u64(v2, "generation.temperature", 0)?;
    expect_u64(v2, "generation.top_p", 1)?;
    expect_bool(v2, "generation.stream", false)?;
    expect_u64(v2, "timeout_policy.connect_timeout_ms", 1000)?;
    expect_u64(v2, "timeout_policy.complete_request_timeout_ms", 1_200_000)?;
    expect_u64(v2, "timeout_policy.supervisor_timeout_ms", 1_320_000)?;
    expect_u64(v2, "retry_policy.automatic_retries", 0)?;
    expect_u64(v2, "retry_policy.fallback_requests", 0)?;
    expect_u64(v2, "retry_policy.adaptive_replicates", 0)?;
    expect_bool(
        v2,
        "freshness_cache_isolation.fresh_server_process_per_arm_replicate",
        true,
    )?;
    expect_bool(
        v2,
        "freshness_cache_isolation.zero_inference_requests_since_startup",
        true,
    )?;
    expect_bool(v2, "freshness_cache_isolation.warmup", false)?;
    expect_bool(v2, "reasoning_isolation.fed_to_later_turns", false)?;
    expect_bool(v2, "reasoning_isolation.fed_to_later_arms", false)?;
    expect_bool(v2, "reasoning_isolation.fed_to_evaluator", false)?;
    expect_string(
        v2,
        "parent_lineage.parent_contract_path",
        SCORED_CONTRACT_PATH,
    )?;
    expect_string(
        v2,
        "parent_lineage.parent_contract_sha256",
        V1_CONTRACT_SHA256,
    )?;
    expect_string(
        v2,
        "parent_lineage.timeout_forensic_review_path",
        "docs/phase-1/PHASE_1C_H001_BASELINE_TIMEOUT_FORENSIC_REVIEW.md",
    )?;
    expect_string(
        v2,
        "parent_lineage.timeout_forensic_review_sha256",
        V1_FORENSIC_REVIEW_SHA256,
    )?;
    expect_string(
        v2,
        "parent_lineage.v1_baseline_evidence_root",
        "experiments/runs/phase1c-scored-capability-v1/h001/replicate-1/baseline",
    )?;
    expect_string(
        v2,
        "parent_lineage.v1_baseline_status",
        "AMBIGUOUS / INCONCLUSIVE",
    )?;
    expect_u64(v2, "parent_lineage.v1_baseline_request_count", 1)?;

    let mut normalized = v2.clone();
    normalized["contract_version"] = v1["contract_version"].clone();
    normalized["experiment_id"] = v1["experiment_id"].clone();
    normalized["status"] = v1["status"].clone();
    normalized["timeout_policy"] = v1["timeout_policy"].clone();
    normalized["evidence_persistence"]["design_evidence_location"] =
        v1["evidence_persistence"]["design_evidence_location"].clone();
    normalized["fingerprint"]["recorded_in"] = v1["fingerprint"]["recorded_in"].clone();
    normalized
        .as_object_mut()
        .ok_or_else(|| H001Error::Validation("V2 contract is not an object".to_string()))?
        .remove("parent_lineage");
    if canonical_json(&normalized)? != canonical_json(v1)? {
        return Err(H001Error::Validation(
            "V2 contract changed fields beyond timeout and lineage metadata".to_string(),
        ));
    }
    Ok(())
}

fn validate_v2_manifest(v1: &Value, v2: &Value, contract: &Value) -> Result<(), H001Error> {
    if canonical_hash(v1)? != V1_PILOT_MANIFEST_SHA256 {
        return Err(H001Error::Validation(
            "V1 pilot manifest fingerprint changed".to_string(),
        ));
    }
    expect_string(v2, "manifest_version", "phase1c-scored-pilot-manifest-v2")?;
    expect_string(v2, "experiment_id", V2_EXPERIMENT_ID)?;
    expect_string(v2, "status", "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED")?;
    expect_string(v2, "runtime_contract_path", V2_CONTRACT_PATH)?;
    if v2.get("runtime_contract_sha256").and_then(Value::as_str)
        != Some(canonical_hash(contract)?.as_str())
    {
        return Err(H001Error::Validation(
            "V2 pilot manifest contract binding mismatch".to_string(),
        ));
    }
    if v2.pointer("/cohort/pilot_case_ids") != v1.pointer("/cohort/pilot_case_ids")
        || v2.get("arms") != v1.get("arms")
        || v2.get("arm_order") != v1.get("arm_order")
        || v2.get("replicates") != v1.get("replicates")
        || v2.get("max_model_turns_per_arm_replicate")
            != v1.get("max_model_turns_per_arm_replicate")
        || v2.get("max_model_inference_requests") != v1.get("max_model_inference_requests")
    {
        return Err(H001Error::Validation(
            "V2 pilot manifest changed frozen pilot semantics".to_string(),
        ));
    }
    expect_string(
        v2,
        "parent_lineage.parent_manifest_path",
        "docs/phase-1/PHASE_1C_SCORED_PILOT_MANIFEST_V1.json",
    )?;
    expect_string(
        v2,
        "parent_lineage.parent_manifest_sha256",
        V1_PILOT_MANIFEST_SHA256,
    )?;
    let mut normalized = v2.clone();
    normalized["manifest_version"] = v1["manifest_version"].clone();
    normalized["experiment_id"] = v1["experiment_id"].clone();
    normalized["status"] = v1["status"].clone();
    normalized["runtime_contract_path"] = v1["runtime_contract_path"].clone();
    normalized["runtime_contract_sha256"] = v1["runtime_contract_sha256"].clone();
    normalized["fingerprint"]["recorded_in"] = v1["fingerprint"]["recorded_in"].clone();
    normalized
        .as_object_mut()
        .ok_or_else(|| H001Error::Validation("V2 pilot manifest is not an object".to_string()))?
        .remove("parent_lineage");
    if canonical_json(&normalized)? != canonical_json(v1)? {
        return Err(H001Error::Validation(
            "V2 pilot manifest changed fields beyond identity metadata".to_string(),
        ));
    }
    Ok(())
}

fn validate_v2_identity(
    identity: &Value,
    contract: &Value,
    manifest: &Value,
    v1_preflight: &Value,
) -> Result<(), H001Error> {
    validate_v2_identity_bindings(
        identity,
        contract,
        manifest,
        v1_preflight,
        "phase1c-h001-v2-baseline-execution-identity-v1",
        "experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline",
    )
}

fn validate_v2_attempt_002_identity(
    identity: &Value,
    contract: &Value,
    manifest: &Value,
) -> Result<(), H001Error> {
    let v1_preflight = preflight_h001()?;
    validate_v2_identity_bindings(
        identity,
        contract,
        manifest,
        &v1_preflight,
        "phase1c-h001-v2-baseline-attempt-002-execution-identity-v1",
        V2_H001_ATTEMPT_002_EVIDENCE_ROOT,
    )?;
    expect_u64(identity, "launch_attempt", 2)?;
    expect_string(
        identity,
        "corrected_cli_identity",
        "prefixity-phase1c-h001-v2 run --confirm-fresh-runtime",
    )?;
    expect_string(
        identity,
        "h001_manifest_path",
        "docs/phase-1/PHASE_1C_H001_SCORED_TASK_MANIFEST_V1.json",
    )?;
    expect_string(identity, "h001_manifest_sha256", V1_H001_MANIFEST_SHA256)?;
    expect_string(
        identity,
        "lineage.predecessor_attempt_id",
        "h001 V2 BASELINE launch attempt 001",
    )?;
    expect_string(
        identity,
        "lineage.predecessor_supervisor_result_path",
        "experiments/runs/phase1c-scored-capability-v2/h001/replicate-1/baseline/supervisor-result.json",
    )?;
    expect_string(
        identity,
        "lineage.predecessor_supervisor_result_sha256",
        "bf65a2b4e9a326354566df8852d4d05e858f263e628cd46e046cb8d2f0492e5",
    )?;
    expect_string(
        identity,
        "lineage.root_cause",
        "EXPERIMENT_RUNNER_CLI_ARGUMENT_PARSING_DEFECT",
    )?;
    Ok(())
}

fn validate_v2_attempt_002_identity_files(require_absent: bool) -> Result<Value, H001Error> {
    let v1_preflight = preflight_h001()?;
    let contract = read_repo_json(V2_CONTRACT_PATH)?;
    let manifest = read_repo_json(V2_PILOT_MANIFEST_PATH)?;
    let identity = read_repo_json(V2_H001_ATTEMPT_002_IDENTITY_PATH)?;
    let v1_contract = read_repo_json(SCORED_CONTRACT_PATH)?;
    let v1_manifest = read_repo_json(SCORED_PILOT_MANIFEST_PATH)?;
    validate_v2_contract(&v1_contract, &contract)?;
    validate_v2_manifest(&v1_manifest, &manifest, &contract)?;
    validate_v2_attempt_002_identity(&identity, &contract, &manifest)?;
    validate_sidecar(
        V2_H001_ATTEMPT_002_IDENTITY_PATH,
        V2_H001_ATTEMPT_002_FINGERPRINT_PATH,
        &identity,
    )?;
    if require_absent && workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT).exists() {
        return Err(H001Error::Validation(
            "V2 h001 BASELINE attempt-002 evidence already exists".to_string(),
        ));
    }
    Ok(json!({
        "contract_sha256": canonical_hash(&contract)?,
        "pilot_manifest_sha256": canonical_hash(&manifest)?,
        "attempt_002_identity_sha256": canonical_hash(&identity)?,
        "baseline_projection_sha256": V1_BASELINE_REQUEST_SHA256,
        "v1_preflight": v1_preflight
    }))
}

fn validate_v2_identity_bindings(
    identity: &Value,
    contract: &Value,
    manifest: &Value,
    v1_preflight: &Value,
    identity_version: &str,
    evidence_root: &str,
) -> Result<(), H001Error> {
    expect_string(identity, "identity_version", identity_version)?;
    expect_string(identity, "experiment_id", V2_EXPERIMENT_ID)?;
    expect_string(identity, "task_id", V2_TASK_ID)?;
    expect_string(identity, "arm", "BASELINE")?;
    expect_u64(identity, "replicate", 1)?;
    expect_u64(identity, "max_turns", 3)?;
    expect_u64(identity, "max_inference_requests", 3)?;
    expect_u64(identity, "automatic_retries", 0)?;
    expect_string(identity, "runtime_contract_path", V2_CONTRACT_PATH)?;
    expect_string(
        identity,
        "runtime_contract_sha256",
        canonical_hash(contract)?.as_str(),
    )?;
    expect_string(identity, "pilot_manifest_path", V2_PILOT_MANIFEST_PATH)?;
    expect_string(
        identity,
        "pilot_manifest_sha256",
        canonical_hash(manifest)?.as_str(),
    )?;
    expect_string(
        identity,
        "baseline_projection_sha256",
        V1_BASELINE_REQUEST_SHA256,
    )?;
    expect_string(identity, "task_artifact_sha256", V1_TASK_SHA256)?;
    expect_string(
        identity,
        "source_manifest_sha256",
        V1_SOURCE_MANIFEST_SHA256,
    )?;
    expect_string(identity, "required_state_sha256", V1_REQUIRED_STATE_SHA256)?;
    expect_string(identity, "tool_contract_sha256", V1_TOOL_CONTRACT_SHA256)?;
    expect_string(identity, "evaluator_sha256", V1_EVALUATOR_SHA256)?;
    expect_string(
        identity,
        "arm_materialization_sha256",
        V1_ARM_MATERIALIZATION_SHA256,
    )?;
    expect_string(identity, "evidence_root", evidence_root)?;
    expect_string(identity, "status", "READY_FOR_SEPARATE_LIVE_AUTHORIZATION")?;
    expect_string(
        identity,
        "lineage.v1_baseline_evidence_root",
        "experiments/runs/phase1c-scored-capability-v1/h001/replicate-1/baseline",
    )?;
    expect_string(
        identity,
        "lineage.v1_baseline_status",
        "AMBIGUOUS / INCONCLUSIVE",
    )?;
    expect_u64(identity, "lineage.v1_baseline_request_count", 1)?;
    if v1_preflight
        .get("arm_projection_sha256")
        .and_then(Value::as_array)
        .and_then(|values| {
            values.iter().find(|value| {
                value.get("arm").and_then(Value::as_str) == Some("BASELINE")
                    && value.get("request_sha256").and_then(Value::as_str)
                        == Some(V1_BASELINE_REQUEST_SHA256)
            })
        })
        .is_none()
    {
        return Err(H001Error::Validation(
            "V1 preflight no longer exposes the frozen BASELINE projection".to_string(),
        ));
    }
    Ok(())
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
            "V2 artifact fingerprint sidecar mismatch for {artifact_path}"
        )));
    }
    Ok(())
}

fn read_repo_json(path: &str) -> Result<Value, H001Error> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn expect_string(value: &Value, path: &str, expected: &str) -> Result<(), H001Error> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_str)
        != Some(expected)
    {
        return Err(H001Error::Validation(format!(
            "V2 value mismatch at {path}"
        )));
    }
    Ok(())
}

fn expect_u64(value: &Value, path: &str, expected: u64) -> Result<(), H001Error> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_u64)
        != Some(expected)
    {
        return Err(H001Error::Validation(format!(
            "V2 value mismatch at {path}"
        )));
    }
    Ok(())
}

fn expect_bool(value: &Value, path: &str, expected: bool) -> Result<(), H001Error> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_bool)
        != Some(expected)
    {
        return Err(H001Error::Validation(format!(
            "V2 value mismatch at {path}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_preflight_is_offline_and_ready() {
        let result = preflight_v2().unwrap();
        assert_eq!(result["state"], "PREPARED");
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["credential_reads"], 0);
        assert_eq!(result["inference_requests"], 0);
    }

    #[test]
    fn v1_and_v2_baseline_requests_are_identical() {
        let v1 = baseline_request().unwrap();
        let v2 = dry_run_v2().unwrap()["model_visible_request"].clone();
        assert_eq!(canonical_json(&v1).unwrap(), canonical_json(&v2).unwrap());
        assert_eq!(canonical_hash(&v2).unwrap(), V1_BASELINE_REQUEST_SHA256);
    }

    #[test]
    fn v2_only_changes_timeout_and_lineage_metadata() {
        let v1 = read_repo_json(SCORED_CONTRACT_PATH).unwrap();
        let v2 = read_repo_json(V2_CONTRACT_PATH).unwrap();
        validate_v2_contract(&v1, &v2).unwrap();
        assert_eq!(
            v2["timeout_policy"]["complete_request_timeout_ms"],
            1_200_000
        );
        assert_eq!(v2["timeout_policy"]["supervisor_timeout_ms"], 1_320_000);
    }

    #[test]
    fn v2_manifest_preserves_frozen_pilot_semantics() {
        let v1 = read_repo_json(SCORED_PILOT_MANIFEST_PATH).unwrap();
        let v2 = read_repo_json(V2_PILOT_MANIFEST_PATH).unwrap();
        let contract = read_repo_json(V2_CONTRACT_PATH).unwrap();
        validate_v2_manifest(&v1, &v2, &contract).unwrap();
        assert_eq!(
            v2["cohort"]["pilot_case_ids"],
            json!(["h001", "h004", "h006", "h007", "h009", "h010"])
        );
    }

    #[test]
    fn v2_dry_run_has_no_transport_and_preserves_attempt_lineage() {
        let result = dry_run_v2().unwrap();
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["inference_requests"], 0);
        assert!(workspace_path(V2_H001_EVIDENCE_ROOT).exists());
    }

    #[test]
    fn canonical_live_invocation_parses_as_v2_baseline() {
        assert_eq!(
            parse_v2_cli_args(v2_live_child_args()).unwrap(),
            V2CliCommand::RunBaseline
        );
    }

    #[test]
    fn missing_confirmation_fails_closed() {
        assert!(parse_v2_cli_args(vec!["run".to_string()]).is_err());
    }

    #[test]
    fn invalid_confirmation_placement_fails_closed() {
        assert!(parse_v2_cli_args(vec![
            V2_LIVE_CONFIRMATION_FLAG.to_string(),
            "run".to_string()
        ])
        .is_err());
        assert!(parse_v2_cli_args(vec![
            "run".to_string(),
            V2_LIVE_CONFIRMATION_FLAG.to_string(),
            "unexpected".to_string()
        ])
        .is_err());
    }

    #[test]
    fn attempt_002_preflight_is_offline_and_respects_evidence_lifecycle() {
        if workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT).exists() {
            let error = preflight_v2_attempt_002().unwrap_err();
            assert!(error
                .to_string()
                .contains("attempt-002 evidence already exists"));
        } else {
            let result = preflight_v2_attempt_002().unwrap();
            assert_eq!(result["state"], "PREPARED");
            assert_eq!(result["launch_attempt"], 2);
            assert_eq!(result["listener_checks"], 0);
            assert_eq!(result["network_calls"], 0);
            assert_eq!(result["inference_requests"], 0);
        }
    }

    #[test]
    fn attempt_002_dry_run_preserves_request_and_has_no_transport() {
        if workspace_path(V2_H001_ATTEMPT_002_EVIDENCE_ROOT).exists() {
            let error = dry_run_v2_attempt_002().unwrap_err();
            assert!(error
                .to_string()
                .contains("attempt-002 evidence already exists"));
        } else {
            let result = dry_run_v2_attempt_002().unwrap();
            assert_eq!(result["state"], "DRY_RUN");
            assert_eq!(result["request_bytes"], 1310);
            assert_eq!(result["request_sha256"], V1_BASELINE_REQUEST_SHA256);
            assert_eq!(result["network_calls"], 0);
            assert_eq!(result["listener_checks"], 0);
            assert_eq!(result["inference_requests"], 0);
        }
    }
}
