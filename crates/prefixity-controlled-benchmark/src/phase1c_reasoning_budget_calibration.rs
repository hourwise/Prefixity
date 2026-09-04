//! Offline preparation and future live execution for a non-scored
//! reasoning-budget feasibility calibration.
//!
//! The calibration is deliberately separate from the Phase 1C capability
//! lineage. It varies only the llama.cpp server-side reasoning budget, keeps
//! reasoning on and the total completion ceiling at 2048, and never consults
//! the h001 evaluator or any Prefixity treatment path.

use crate::hashing::{canonical_hash, sha256_hex};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const CALIBRATION_MANIFEST_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.json";
pub const CALIBRATION_MANIFEST_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_CALIBRATION_MANIFEST_V1.sha256";
pub const CALIBRATION_EVIDENCE_ROOT: &str = "experiments/runs/phase1c-reasoning-budget-calibration";
pub const CALIBRATION_ATTEMPT_002_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_002_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-002";
pub const CALIBRATION_CASE_IDS: [&str; 3] = ["rbcal-001", "rbcal-002", "rbcal-003"];
pub const CALIBRATION_BUDGETS: [u32; 3] = [1024, 512, 256];

const EXPERIMENT_ID: &str = "phase1c-reasoning-budget-calibration";
const MODEL_ID: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
const ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
const HOST: &str = "127.0.0.1";
const PORT: u16 = 8080;
const MAX_TURNS: u32 = 1;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const REQUEST_SCHEMA_ID: &str = "prefixity.phase1c.reasoning-budget-calibration-request";
const RESPONSE_SCHEMA_ID: &str = "prefixity.phase1c.reasoning-budget-calibration-response";
const CANDIDATE_RESULT_SCHEMA_ID: &str =
    "prefixity.phase1c.reasoning-budget-calibration-candidate-result";
const REASONING_DIAGNOSTIC_SCHEMA_ID: &str =
    "prefixity.phase1c.reasoning-budget-calibration-reasoning-diagnostic";

#[derive(Debug, thiserror::Error)]
pub enum ReasoningBudgetCalibrationError {
    #[error("reasoning-budget calibration file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("reasoning-budget calibration JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("reasoning-budget calibration validation failed: {0}")]
    Validation(String),
    #[error("reasoning-budget calibration transport failed: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalibrationCliCommand {
    Preflight,
    Fingerprint,
    DryRun,
    Run { budget: u32 },
    Summarize { budget: u32 },
    Attempt002Fingerprint,
    Attempt002Preflight,
    Attempt002DryRun,
    Attempt002ExclusivityPreflight,
    RunAttempt002 { budget: u32, server_pid: u32 },
    SummarizeAttempt002 { budget: u32 },
}

pub fn parse_calibration_cli_args<I>(
    args: I,
) -> Result<CalibrationCliCommand, ReasoningBudgetCalibrationError>
where
    I: IntoIterator<Item = String>,
{
    let args = args.into_iter().collect::<Vec<_>>();
    match args.as_slice() {
        [command] if command == "preflight" => Ok(CalibrationCliCommand::Preflight),
        [command] if command == "fingerprint" => Ok(CalibrationCliCommand::Fingerprint),
        [command] if command == "dry-run" => Ok(CalibrationCliCommand::DryRun),
        [command, flag, value, confirm]
            if command == "run"
                && flag == "--budget"
                && confirm == "--confirm-fresh-runtime" =>
        {
            Ok(CalibrationCliCommand::Run {
                budget: parse_budget(value)?,
            })
        }
        [command, flag, value] if command == "summarize" && flag == "--budget" => {
            Ok(CalibrationCliCommand::Summarize {
                budget: parse_budget(value)?,
            })
        }
        [command] if command == "attempt-002-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt002Fingerprint)
        }
        [command] if command == "attempt-002-preflight" => {
            Ok(CalibrationCliCommand::Attempt002Preflight)
        }
        [command] if command == "attempt-002-dry-run" => {
            Ok(CalibrationCliCommand::Attempt002DryRun)
        }
        [command, confirm]
            if command == "attempt-002-exclusivity-preflight"
                && confirm == "--confirm-no-other-workflow" =>
        {
            Ok(CalibrationCliCommand::Attempt002ExclusivityPreflight)
        }
        [command, budget_flag, budget, pid_flag, pid, fresh, exclusive]
            if command == "run-attempt-002"
                && budget_flag == "--budget"
                && pid_flag == "--server-pid"
                && fresh == "--confirm-fresh-runtime"
                && exclusive == "--confirm-exclusive-runtime" =>
        {
            let budget = parse_budget(budget)?;
            if budget != 1024 {
                return Err(invalid("attempt-002 is registered only for budget 1024"));
            }
            Ok(CalibrationCliCommand::RunAttempt002 {
                budget,
                server_pid: parse_server_pid(pid)?,
            })
        }
        [command, flag, value]
            if command == "summarize-attempt-002" && flag == "--budget" =>
        {
            let budget = parse_budget(value)?;
            if budget != 1024 {
                return Err(invalid("attempt-002 is registered only for budget 1024"));
            }
            Ok(CalibrationCliCommand::SummarizeAttempt002 { budget })
        }
        _ => Err(ReasoningBudgetCalibrationError::Validation(
            "usage: prefixity-phase1c-reasoning-budget-calibration [preflight|fingerprint|dry-run|run --budget {1024|512|256} --confirm-fresh-runtime|summarize --budget {1024|512|256}|attempt-002-fingerprint|attempt-002-preflight|attempt-002-dry-run|attempt-002-exclusivity-preflight --confirm-no-other-workflow|run-attempt-002 --budget 1024 --server-pid PID --confirm-fresh-runtime --confirm-exclusive-runtime|summarize-attempt-002 --budget 1024]".to_string(),
        )),
    }
}

pub fn fingerprint_calibration() -> Result<Value, ReasoningBudgetCalibrationError> {
    let manifest = read_manifest(false)?;
    validate_manifest(&manifest, false)?;
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let request_fingerprints = CALIBRATION_CASE_IDS
        .iter()
        .map(|case_id| {
            let case = cases
                .iter()
                .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
                .ok_or_else(|| missing(case_id))?;
            let request = build_request(&manifest, case)?;
            Ok(json!({
                "case_id": case_id,
                "request_sha256": canonical_hash(&request)?,
                "wire_request_sha256": sha256_hex(&serde_json::to_vec(&request)?),
                "request_bytes": serde_json::to_vec(&request)?.len()
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "manifest_sha256": canonical_hash(&manifest)?,
        "candidate_budgets": CALIBRATION_BUDGETS,
        "candidate_order": CALIBRATION_BUDGETS,
        "cases": request_fingerprints,
        "network_calls": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_calibration() -> Result<Value, ReasoningBudgetCalibrationError> {
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    if workspace_path(CALIBRATION_EVIDENCE_ROOT).exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(
            "calibration evidence root already exists; preparation is no longer pristine"
                .to_string(),
        ));
    }
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let case_projections = CALIBRATION_CASE_IDS
        .iter()
        .map(|case_id| {
            let case = cases
                .iter()
                .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
                .ok_or_else(|| missing(case_id))?;
            let request = build_request(&manifest, case)?;
            Ok(json!({
                "case_id": case_id,
                "request_sha256": canonical_hash(&request)?,
                "wire_request_sha256": sha256_hex(&serde_json::to_vec(&request)?),
                "request_bytes": serde_json::to_vec(&request)?.len()
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "PREPARED",
        "experiment_id": EXPERIMENT_ID,
        "purpose": "NON-SCORED RUNTIME FEASIBILITY CALIBRATION",
        "manifest_sha256": canonical_hash(&manifest)?,
        "model": MODEL_ID,
        "endpoint": ENDPOINT,
        "reasoning": "on",
        "candidate_budgets": CALIBRATION_BUDGETS,
        "candidate_order": CALIBRATION_BUDGETS,
        "case_order": CALIBRATION_CASE_IDS,
        "case_projections": case_projections,
        "maximum_inference_requests": 9,
        "maximum_requests_per_candidate": 3,
        "network_calls": 0,
        "listener_checks": 0,
        "credential_reads": 0,
        "inference_requests": 0,
        "evidence_root": CALIBRATION_EVIDENCE_ROOT
    }))
}

pub fn dry_run_calibration() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_calibration()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let mut combinations = Vec::new();
    for budget in CALIBRATION_BUDGETS {
        for case_id in CALIBRATION_CASE_IDS {
            let case = cases
                .iter()
                .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
                .ok_or_else(|| missing(case_id))?;
            let request = build_request(&manifest, case)?;
            combinations.push(json!({
                "budget": budget,
                "case_id": case_id,
                "server_reasoning_budget": budget,
                "request_has_reasoning_budget_field": false,
                "request_sha256": canonical_hash(&request)?,
                "wire_request_sha256": sha256_hex(&serde_json::to_vec(&request)?),
                "request_bytes": serde_json::to_vec(&request)?.len(),
                "network_calls": 0,
                "inference_requests": 0
            }));
        }
    }
    Ok(json!({
        "state": "DRY_RUN",
        "experiment_id": EXPERIMENT_ID,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "inference_requests": 0
    }))
}

pub fn fingerprint_attempt_002() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_002_identity(false)?;
    validate_attempt_002_identity(&identity, false)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_002_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "state": "PREPARED",
        "attempt": 2,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "candidate_budget": 1024,
        "candidate_order": CALIBRATION_BUDGETS,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_002() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_002_identity(true)?;
    validate_attempt_002_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_002_bindings(&identity, &fingerprints)?;
    if workspace_path(CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT).exists() {
        return Err(invalid(
            "attempt-002 evidence root already exists; preparation is not pristine",
        ));
    }
    Ok(json!({
        "state": "PREPARED",
        "attempt": 2,
        "experiment_id": EXPERIMENT_ID,
        "purpose": "NON-SCORED RUNTIME FEASIBILITY CALIBRATION",
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "model": MODEL_ID,
        "candidate_budget": 1024,
        "candidate_order": CALIBRATION_BUDGETS,
        "case_order": CALIBRATION_CASE_IDS,
        "case_projections": fingerprints["cases"],
        "maximum_requests": 3,
        "automatic_retries": 0,
        "fresh_server_required": true,
        "runtime_exclusivity_required": true,
        "runtime_exclusivity_preflight": "required-before-server-start",
        "root_cause": "ROOT CAUSE NOT ESTABLISHED",
        "network_calls": 0,
        "listener_checks": 0,
        "credential_reads": 0,
        "inference_requests": 0,
        "evidence_root": CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT,
        "attempt_001_evidence_root": "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/"
    }))
}

pub fn dry_run_attempt_002() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_002()?;
    let fingerprints = fingerprint_calibration()?;
    let cases = fingerprints
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let combinations = CALIBRATION_CASE_IDS
        .iter()
        .map(|case_id| {
            let case = cases
                .iter()
                .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
                .ok_or_else(|| missing(case_id))?;
            Ok(json!({
                "attempt": 2,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 1024,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "listener_checks": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 2,
        "experiment_id": EXPERIMENT_ID,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_002_exclusivity_preflight(
    confirm_no_other_workflow: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let current_pid = std::process::id();
    let processes = tasklist_processes()?;
    let llama_processes = processes
        .iter()
        .filter(|process| is_llama_process(&process.image_name))
        .cloned()
        .collect::<Vec<_>>();
    let competing_processes = processes
        .iter()
        .filter(|process| process.pid != current_pid && is_competing_process(&process.image_name))
        .cloned()
        .collect::<Vec<_>>();
    let port_listeners = netstat_port_listeners(PORT)?;
    let passed = llama_processes.is_empty()
        && port_listeners.is_empty()
        && competing_processes.is_empty()
        && confirm_no_other_workflow;
    Ok(json!({
        "state": if passed { "READY" } else { "BLOCKED" },
        "attempt": 2,
        "check": "runtime_exclusivity_before_server_start",
        "current_preflight_pid": current_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": port_listeners,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "operator_no_other_workflow_confirmed": confirm_no_other_workflow,
        "required_operator_checks": [
            "no llama.exe process is running",
            "port 8080 is not owned or listening",
            "no other Prefixity or Qwen runner is active",
            "no Luna, Codex, or helper workflow is configured to interact with this runtime"
        ],
        "inspection_commands": [
            "tasklist /FO CSV /NH",
            "netstat -ano -p tcp"
        ],
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_002_runtime_ownership(
    server_pid: u32,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    if server_pid == 0 {
        return Err(invalid("server PID must be nonzero"));
    }
    let processes = tasklist_processes()?;
    let llama_processes = processes
        .iter()
        .filter(|process| is_llama_process(&process.image_name))
        .cloned()
        .collect::<Vec<_>>();
    let current_pid = std::process::id();
    let competing_processes = processes
        .iter()
        .filter(|process| process.pid != current_pid && is_competing_process(&process.image_name))
        .cloned()
        .collect::<Vec<_>>();
    let port_listeners = netstat_port_listeners(PORT)?;
    let single_llama_pid = llama_processes.len() == 1
        && llama_processes
            .first()
            .is_some_and(|process| process.pid == server_pid);
    let single_port_pid = port_listeners.len() == 1
        && port_listeners
            .first()
            .is_some_and(|listener| listener.pid == server_pid);
    let no_competing_processes = competing_processes.is_empty();
    Ok(json!({
        "state": if single_llama_pid && single_port_pid && no_competing_processes { "READY" } else { "BLOCKED" },
        "attempt": 2,
        "check": "runtime_ownership_before_inference",
        "expected_server_pid": server_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": port_listeners,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "executable_path": "C:\\Users\\USER\\AppData\\Local\\Microsoft\\WindowsApps\\llama.exe",
        "server_start_identity": "fresh candidate-1024 llama.cpp process",
        "single_expected_llama_process": single_llama_pid,
        "port_owner_matches_server_pid": single_port_pid,
        "no_competing_processes": no_competing_processes,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn execute_attempt_002(
    budget: u32,
    server_pid: u32,
    confirm_fresh_runtime: bool,
    confirm_exclusive_runtime: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    if budget != 1024 {
        return Err(invalid("attempt-002 is registered only for budget 1024"));
    }
    if !confirm_fresh_runtime || !confirm_exclusive_runtime {
        return Err(invalid(
            "attempt-002 requires fresh-runtime and exclusive-runtime confirmations",
        ));
    }
    let identity = read_attempt_002_identity(true)?;
    validate_attempt_002_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let preparation = preflight_attempt_002()?;
    let candidate_root = attempt_002_root();
    if candidate_root.exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(format!(
            "attempt-002 candidate evidence already exists: {}",
            candidate_root.display()
        )));
    }
    fs::create_dir_all(&candidate_root)?;
    write_json(&candidate_root.join("preflight.json"), &preparation)?;
    write_json(
        &candidate_root.join("runtime-confirmation.json"),
        &json!({
            "confirmation": "operator_current_confirmation",
            "attempt": 2,
            "experiment_id": EXPERIMENT_ID,
            "model": MODEL_ID,
            "quantization": "Q4_0",
            "context_size": 8192,
            "parallel_slots": 1,
            "metrics": "enabled",
            "reasoning": "on",
            "reasoning_budget": budget,
            "endpoint": ENDPOINT,
            "fresh_server_per_candidate": true,
            "runtime_exclusivity_confirmed": confirm_exclusive_runtime,
            "server_pid": server_pid,
            "zero_inference_since_startup": true,
            "no_warmup": true,
            "no_manual_request": true,
            "no_browser_or_endpoint_contact": true,
            "server_launch_arguments": server_launch_arguments(budget),
            "recorded_at_unix_ms": now_unix_ms()?
        }),
    )?;

    let ownership = attempt_002_runtime_ownership(server_pid)?;
    write_json(&candidate_root.join("runtime-ownership.json"), &ownership)?;
    if ownership.get("state").and_then(Value::as_str) != Some("READY") {
        let result = candidate_result(
            budget,
            Vec::new(),
            0,
            "INCONCLUSIVE",
            false,
            Some("runtime ownership was not exclusive before inference"),
        )?;
        write_json(&attempt_002_result_path(), &result)?;
        return Ok(result);
    }

    let readiness_started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], PORT)),
        Duration::from_millis(1000),
    );
    let readiness_elapsed_ms = readiness_started.elapsed().as_millis() as u64;
    if let Err(error) = readiness {
        let readiness_record = json!({
            "check": "tcp_listener_connect",
            "host": HOST,
            "port": PORT,
            "listener_check_attempts": 1,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
            "passed": false,
            "error": error.to_string()
        });
        write_json(&candidate_root.join("readiness.json"), &readiness_record)?;
        let result = candidate_result(
            budget,
            Vec::new(),
            0,
            "INCONCLUSIVE",
            false,
            Some("single non-inference listener check failed"),
        )?;
        write_json(&attempt_002_result_path(), &result)?;
        return Ok(result);
    }
    write_json(
        &candidate_root.join("readiness.json"),
        &json!({
            "check": "tcp_listener_connect",
            "host": HOST,
            "port": PORT,
            "listener_check_attempts": 1,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
            "passed": true
        }),
    )?;

    let client = Client::builder()
        .connect_timeout(Duration::from_millis(1000))
        .timeout(Duration::from_millis(1_200_000))
        .redirect(Policy::none())
        .build()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let mut results = Vec::new();
    for case_id in CALIBRATION_CASE_IDS {
        let case = cases
            .iter()
            .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
            .ok_or_else(|| missing(case_id))?;
        let case_dir = candidate_root.join(case_id);
        fs::create_dir_all(&case_dir)?;
        results.push(execute_case(&manifest, case, budget, &case_dir, &client)?);
    }
    let state = aggregate_state(&results);
    let result = candidate_result(budget, results, 3, state, true, None)?;
    write_json(&attempt_002_result_path(), &result)?;
    Ok(result)
}

pub fn summarize_attempt_002_budget(budget: u32) -> Result<Value, ReasoningBudgetCalibrationError> {
    if budget != 1024 {
        return Err(invalid("attempt-002 is registered only for budget 1024"));
    }
    read_json_path(&attempt_002_result_path())
}

pub fn summarize_calibration_budget(budget: u32) -> Result<Value, ReasoningBudgetCalibrationError> {
    ensure_budget(budget)?;
    read_json_path(&candidate_result_path(budget))
}

pub fn next_candidate_budget(budget: u32, candidate_status: &str) -> Option<u32> {
    if candidate_status == "PASS" {
        return None;
    }
    CALIBRATION_BUDGETS
        .iter()
        .position(|candidate| *candidate == budget)
        .and_then(|index| CALIBRATION_BUDGETS.get(index + 1).copied())
}

pub fn execute_calibration(
    budget: u32,
    confirm_fresh_runtime: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    ensure_budget(budget)?;
    if !confirm_fresh_runtime {
        return Err(ReasoningBudgetCalibrationError::Validation(
            "explicit fresh-runtime confirmation is required before the listener check".to_string(),
        ));
    }
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    validate_candidate_order(budget)?;
    let preparation = preflight_calibration()?;
    let candidate_root = candidate_root(budget);
    if candidate_root.exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(format!(
            "calibration candidate evidence already exists: {}",
            candidate_root.display()
        )));
    }
    fs::create_dir_all(&candidate_root)?;
    write_json(&candidate_root.join("preflight.json"), &preparation)?;
    write_json(
        &candidate_root.join("runtime-confirmation.json"),
        &json!({
            "confirmation": "operator_current_confirmation",
            "experiment_id": EXPERIMENT_ID,
            "model": MODEL_ID,
            "quantization": "Q4_0",
            "context_size": 8192,
            "parallel_slots": 1,
            "metrics": "enabled",
            "reasoning": "on",
            "reasoning_budget": budget,
            "endpoint": ENDPOINT,
            "fresh_server_per_candidate": true,
            "zero_inference_since_startup": true,
            "no_warmup": true,
            "no_manual_request": true,
            "no_browser_or_endpoint_contact": true,
            "server_launch_arguments": server_launch_arguments(budget),
            "recorded_at_unix_ms": now_unix_ms()?
        }),
    )?;

    let readiness_started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], PORT)),
        Duration::from_millis(1000),
    );
    let readiness_elapsed_ms = readiness_started.elapsed().as_millis() as u64;
    if let Err(error) = readiness {
        let readiness_record = json!({
            "check": "tcp_listener_connect",
            "host": HOST,
            "port": PORT,
            "listener_check_attempts": 1,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
            "passed": false,
            "error": error.to_string()
        });
        write_json(&candidate_root.join("readiness.json"), &readiness_record)?;
        let result = candidate_result(
            budget,
            Vec::new(),
            0,
            "INCONCLUSIVE",
            false,
            Some("single non-inference listener check failed"),
        )?;
        write_json(&candidate_result_path(budget), &result)?;
        return Ok(result);
    }
    write_json(
        &candidate_root.join("readiness.json"),
        &json!({
            "check": "tcp_listener_connect",
            "host": HOST,
            "port": PORT,
            "listener_check_attempts": 1,
            "network_calls": 1,
            "inference_requests": 0,
            "elapsed_ms": readiness_elapsed_ms,
            "passed": true
        }),
    )?;

    let client = Client::builder()
        .connect_timeout(Duration::from_millis(1000))
        .timeout(Duration::from_millis(1_200_000))
        .redirect(Policy::none())
        .build()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    let mut results = Vec::new();
    for case_id in CALIBRATION_CASE_IDS {
        let case = cases
            .iter()
            .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
            .ok_or_else(|| missing(case_id))?;
        let case_dir = candidate_root.join(case_id);
        fs::create_dir_all(&case_dir)?;
        results.push(execute_case(&manifest, case, budget, &case_dir, &client)?);
    }
    let state = aggregate_state(&results);
    let result = candidate_result(budget, results, 3, state, true, None)?;
    write_json(&candidate_result_path(budget), &result)?;
    Ok(result)
}

fn execute_case(
    manifest: &Value,
    case: &Value,
    budget: u32,
    evidence_dir: &Path,
    client: &Client,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let case_id = case
        .get("case_id")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("case_id"))?;
    let request = build_request(manifest, case)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let request_sha256 = canonical_hash(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    write_bytes(&evidence_dir.join("request-turn-1.json"), &request_bytes)?;
    let request_record = json!({
        "schema_id": REQUEST_SCHEMA_ID,
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "case_id": case_id,
        "budget": budget,
        "model": MODEL_ID,
        "request_sha256": request_sha256,
        "wire_request_sha256": wire_request_sha256,
        "request_bytes": request_bytes.len(),
        "max_tokens": 2048,
        "temperature": 0,
        "top_p": 1,
        "seed": 1,
        "stream": false,
        "reasoning_budget_in_request": false
    });

    let request_started = Instant::now();
    let response = client
        .post(ENDPOINT)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_bytes)
        .send();
    let transport_elapsed_ms = request_started.elapsed().as_millis() as u64;
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            write_json(&evidence_dir.join("request-record.json"), &request_record)?;
            write_json(
                &evidence_dir.join("reasoning-turn-1.json"),
                &json!({
                    "schema_id": REASONING_DIAGNOSTIC_SCHEMA_ID,
                    "schema_version": 1,
                    "case_id": case_id,
                    "budget": budget,
                    "reasoning_content_present": null,
                    "reasoning_content": null,
                    "reasoning_is_scored": false
                }),
            )?;
            let result = json!({
                "schema_id": RESPONSE_SCHEMA_ID,
                "schema_version": 1,
                "case_id": case_id,
                "budget": budget,
                "state": "INCONCLUSIVE",
                "request": request_record,
                "response": {
                    "http_status": null,
                    "complete": false,
                    "response_body_bytes": null,
                    "response_body_sha256": null,
                    "response_body_file": null,
                    "transport_elapsed_ms": transport_elapsed_ms,
                    "reasoning_content_present": null,
                    "finish_reason": null,
                    "prompt_tokens": null,
                    "completion_tokens": null,
                    "cached_tokens": null
                },
                "validation": {
                    "transport_ambiguous": true,
                    "response_json_parsed": false,
                    "terminal_final_content": false,
                    "structural_response_valid": false,
                    "reasoning_content_is_diagnostic_only": true,
                    "error": format!("request dispatch/completion is ambiguous: {error}")
                },
                "reasoning_diagnostic": "reasoning-turn-1.json"
            });
            write_json(&evidence_dir.join("case-result.json"), &result)?;
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
    let complete = response_body.len() <= MAX_RESPONSE_BYTES && body_result.is_ok();
    let response_body_sha256 = sha256_hex(&response_body);
    write_json(&evidence_dir.join("request-record.json"), &request_record)?;
    write_bytes(&evidence_dir.join("response-turn-1.bin"), &response_body)?;
    if let Err(error) = body_result {
        let result = json!({
            "schema_id": RESPONSE_SCHEMA_ID,
            "schema_version": 1,
            "case_id": case_id,
            "budget": budget,
            "state": "INCONCLUSIVE",
            "request": request_record,
            "response": {
                "http_status": status,
                "complete": false,
                "response_body_bytes": response_body.len(),
                "response_body_sha256": response_body_sha256,
                "response_body_file": "response-turn-1.bin",
                "safe_headers": safe_headers,
                "transport_elapsed_ms": transport_elapsed_ms,
                "reasoning_content_present": null,
                "finish_reason": null,
                "prompt_tokens": null,
                "completion_tokens": null,
                "cached_tokens": null
            },
            "validation": {
                "transport_ambiguous": true,
                "response_json_parsed": false,
                "terminal_final_content": false,
                "structural_response_valid": false,
                "reasoning_content_is_diagnostic_only": true,
                "error": format!("response body read is ambiguous: {error}")
            },
            "reasoning_diagnostic": "reasoning-turn-1.json"
        });
        write_json(
            &evidence_dir.join("reasoning-turn-1.json"),
            &json!({
                "schema_id": REASONING_DIAGNOSTIC_SCHEMA_ID,
                "schema_version": 1,
                "case_id": case_id,
                "budget": budget,
                "reasoning_content_present": null,
                "reasoning_content": null,
                "reasoning_is_scored": false
            }),
        )?;
        write_json(&evidence_dir.join("case-result.json"), &result)?;
        return Ok(result);
    }

    let parsed = if complete && status == 200 {
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
    let reasoning_content_present = reasoning_content.is_some();
    let tool_calls = message.and_then(|value| value.get("tool_calls")).cloned();
    let tool_call_present = tool_calls
        .as_ref()
        .is_some_and(|value| value.as_array().is_some_and(|values| !values.is_empty()));
    let finish_reason = parsed
        .as_ref()
        .and_then(|value| value.pointer("/choices/0/finish_reason"))
        .cloned();
    let finish_reason_text = finish_reason.as_ref().and_then(Value::as_str);
    let terminal_final_content = final_content
        .as_deref()
        .is_some_and(|content| !content.trim().is_empty())
        && !tool_call_present
        && finish_reason_text.is_some_and(|reason| reason != "length");
    let expected = case
        .pointer("/expected_response/exact_object")
        .ok_or_else(|| missing("expected_response.exact_object"))?;
    let structural_response_valid = terminal_final_content
        && final_content
            .as_deref()
            .is_some_and(|content| structurally_matches(content, expected));
    let state = if complete && status == 200 && structural_response_valid {
        "PASS"
    } else {
        "FAIL"
    };
    let usage = parsed
        .as_ref()
        .and_then(|value| value.get("usage"))
        .cloned();
    let timings = parsed
        .as_ref()
        .and_then(|value| value.get("timings"))
        .cloned();
    let prompt_tokens = usage
        .as_ref()
        .and_then(|value| value.get("prompt_tokens"))
        .cloned();
    let completion_tokens = usage
        .as_ref()
        .and_then(|value| value.get("completion_tokens"))
        .cloned();
    let cached_tokens = usage
        .as_ref()
        .and_then(|value| value.pointer("/prompt_tokens_details/cached_tokens"))
        .cloned();
    let error = if state == "PASS" {
        Value::Null
    } else if !complete {
        json!("response exceeded the bounded body size")
    } else if status != 200 {
        json!(format!("unexpected HTTP status {status}"))
    } else if parsed.is_none() {
        json!("response body was not valid JSON")
    } else if finish_reason_text == Some("length") {
        json!("total 2048-token ceiling exhausted before terminal completion")
    } else if final_content.is_none() {
        json!("terminal assistant content was absent or not a string")
    } else if tool_call_present {
        json!("tool call emitted under the no-tool structural contract")
    } else {
        json!("terminal content failed the exact structural response schema")
    };
    write_json(
        &evidence_dir.join("reasoning-turn-1.json"),
        &json!({
            "schema_id": REASONING_DIAGNOSTIC_SCHEMA_ID,
            "schema_version": 1,
            "case_id": case_id,
            "budget": budget,
            "reasoning_content_present": reasoning_content_present,
            "reasoning_content": reasoning_content,
            "reasoning_content_bytes": message
                .and_then(|value| value.get("reasoning_content"))
                .and_then(Value::as_str)
                .map(str::len),
            "reasoning_is_scored": false
        }),
    )?;
    write_json(
        &evidence_dir.join("normalized-response.json"),
        &json!({
            "schema_id": RESPONSE_SCHEMA_ID,
            "schema_version": 1,
            "case_id": case_id,
            "budget": budget,
            "final_content": final_content,
            "tool_calls": tool_calls,
            "finish_reason": finish_reason,
            "usage": usage,
            "timings": timings,
            "response_json_parsed": parsed.is_some(),
            "terminal_final_content": terminal_final_content,
            "structural_response_valid": structural_response_valid,
            "reasoning_content_present": reasoning_content_present,
            "reasoning_is_scored": false
        }),
    )?;
    let result = json!({
        "schema_id": RESPONSE_SCHEMA_ID,
        "schema_version": 1,
        "case_id": case_id,
        "budget": budget,
        "state": state,
        "request": request_record,
        "response": {
            "http_status": status,
            "complete": complete,
            "response_body_bytes": response_body.len(),
            "response_body_sha256": response_body_sha256,
            "response_body_file": "response-turn-1.bin",
            "safe_headers": safe_headers,
            "transport_elapsed_ms": transport_elapsed_ms,
            "reasoning_content_present": reasoning_content_present,
            "finish_reason": finish_reason,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "cached_tokens": cached_tokens
        },
        "validation": {
            "transport_ambiguous": false,
            "response_json_parsed": parsed.is_some(),
            "terminal_final_content": terminal_final_content,
            "structural_response_valid": structural_response_valid,
            "reasoning_content_is_diagnostic_only": true,
            "no_tool_call": !tool_call_present,
            "error": error
        },
        "normalized_response": "normalized-response.json",
        "reasoning_diagnostic": "reasoning-turn-1.json"
    });
    write_json(&evidence_dir.join("case-result.json"), &result)?;
    Ok(result)
}

fn validate_manifest(
    manifest: &Value,
    verify_recorded_hashes: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        manifest,
        "manifest_version",
        "phase1c-reasoning-budget-calibration-v1",
    )?;
    expect_string(
        manifest,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(manifest, "experiment_id", EXPERIMENT_ID)?;
    expect_string(
        manifest,
        "purpose",
        "NON-SCORED RUNTIME FEASIBILITY CALIBRATION",
    )?;
    if manifest.get("capability_claims_permitted") != Some(&Value::Bool(false)) {
        return Err(invalid("capability claims must be disabled"));
    }
    expect_string(manifest, "runtime.engine", "llama.cpp")?;
    expect_string(
        manifest,
        "runtime.api_surface",
        "llama.cpp-openai-compatible-chat-completions-v1",
    )?;
    expect_string(manifest, "runtime.model", MODEL_ID)?;
    expect_string(manifest, "runtime.quantization", "Q4_0")?;
    expect_u64(manifest, "runtime.context_size", 8192)?;
    expect_u64(manifest, "runtime.parallel_slots", 1)?;
    expect_string(manifest, "runtime.metrics", "enabled")?;
    expect_string(manifest, "runtime.reasoning", "on")?;
    expect_string(manifest, "runtime.server_build", "b10217-ddd4ec142")?;
    expect_string(
        manifest,
        "runtime.reasoning_budget_support.flag_syntax",
        "--reasoning-budget N",
    )?;
    expect_string(
        manifest,
        "runtime.reasoning_budget_support.message_flag_syntax",
        "--reasoning-budget-message MESSAGE",
    )?;
    if manifest.pointer("/runtime/reasoning_budget_support/semantics/-1")
        != Some(&Value::String("unrestricted".to_string()))
        || manifest.pointer("/runtime/reasoning_budget_support/semantics/0")
            != Some(&Value::String("immediate end".to_string()))
        || manifest.pointer("/runtime/reasoning_budget_support/semantics/N>0")
            != Some(&Value::String("token budget for thinking".to_string()))
    {
        return Err(invalid(
            "installed reasoning-budget semantics are not recorded exactly",
        ));
    }
    if manifest.pointer("/runtime/reasoning_budget_support/server_side") != Some(&Value::Bool(true))
        || manifest.pointer("/runtime/reasoning_budget_support/preserves_reasoning_on")
            != Some(&Value::Bool(true))
    {
        return Err(invalid(
            "reasoning budget is not bound as server-side reasoning-on configuration",
        ));
    }
    expect_u64(manifest, "generation.temperature", 0)?;
    expect_u64(manifest, "generation.top_p", 1)?;
    expect_u64(manifest, "generation.max_tokens", 2048)?;
    expect_bool(manifest, "generation.stream", false)?;
    expect_u64(manifest, "generation.seed", 1)?;
    expect_u64(manifest, "timeout_policy.connect_timeout_ms", 1000)?;
    expect_u64(
        manifest,
        "timeout_policy.complete_request_timeout_ms",
        1_200_000,
    )?;
    expect_u64(manifest, "timeout_policy.supervisor_timeout_ms", 1_320_000)?;
    expect_u64(manifest, "retry_policy.automatic_retries", 0)?;
    expect_u64(manifest, "retry_policy.fallback_requests", 0)?;
    expect_u64(manifest, "retry_policy.adaptive_replicates", 0)?;
    if manifest.get("candidate_budgets") != Some(&json!([1024, 512, 256]))
        || manifest.get("candidate_order") != Some(&json!([1024, 512, 256]))
        || manifest.get("maximum_inference_requests") != Some(&json!(9))
        || manifest.get("maximum_requests_per_candidate") != Some(&json!(3))
    {
        return Err(invalid("candidate budget order or request bound changed"));
    }
    if manifest.pointer("/freshness_policy/fresh_server_per_candidate") != Some(&Value::Bool(true))
        || manifest.pointer("/freshness_policy/warmup") != Some(&Value::Bool(false))
        || manifest.pointer("/freshness_policy/maximum_non_inference_listener_checks_per_candidate")
            != Some(&json!(1))
    {
        return Err(invalid("fresh-runtime policy changed"));
    }
    let cases = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    if cases.len() != CALIBRATION_CASE_IDS.len() {
        return Err(invalid("calibration case count is not exactly three"));
    }
    for case_id in CALIBRATION_CASE_IDS {
        let case = cases
            .iter()
            .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
            .ok_or_else(|| missing(case_id))?;
        validate_case(manifest, case, verify_recorded_hashes)?;
    }
    if verify_recorded_hashes {
        validate_sidecar(manifest)?;
    }
    Ok(())
}

fn validate_case(
    manifest: &Value,
    case: &Value,
    verify_recorded_hashes: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let case_id = case
        .get("case_id")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("case_id"))?;
    if !CALIBRATION_CASE_IDS.contains(&case_id) {
        return Err(invalid("unknown calibration case"));
    }
    let messages = case
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("messages"))?;
    if messages.len() != 2
        || messages[0].get("role").and_then(Value::as_str) != Some("system")
        || messages[1].get("role").and_then(Value::as_str) != Some("user")
    {
        return Err(invalid(
            "calibration case must contain one system and one user message",
        ));
    }
    let serialized = serde_json::to_string(case)?.to_ascii_lowercase();
    for forbidden in [
        "h001",
        "e001",
        "e002",
        "a003",
        "evaluator",
        "expected_final_answer",
    ] {
        if serialized.contains(forbidden) {
            return Err(invalid("calibration case contains scored-lineage material"));
        }
    }
    let expected = case
        .pointer("/expected_response/exact_object")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("expected_response.exact_object"))?;
    let required_keys = case
        .pointer("/expected_response/required_keys")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("expected_response.required_keys"))?;
    if expected.len() != 3 || required_keys.len() != 3 {
        return Err(invalid(
            "calibration structural response must contain exactly three fields",
        ));
    }
    let request = build_request(manifest, case)?;
    let request_bytes = serde_json::to_vec(&request)?;
    let request_sha256 = canonical_hash(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    if verify_recorded_hashes {
        expect_string(case, "request_sha256", &request_sha256)?;
        expect_string(case, "wire_request_sha256", &wire_request_sha256)?;
        expect_u64(case, "request_bytes", request_bytes.len() as u64)?;
    }
    if case.get("evidence_path").and_then(Value::as_str)
        != Some(&format!(
            "{CALIBRATION_EVIDENCE_ROOT}/budget-{{budget}}/{case_id}"
        ))
    {
        return Err(invalid("calibration evidence path is not canonical"));
    }
    Ok(())
}

fn build_request(manifest: &Value, case: &Value) -> Result<Value, ReasoningBudgetCalibrationError> {
    Ok(json!({
        "model": manifest.pointer("/runtime/model").cloned().ok_or_else(|| missing("runtime.model"))?,
        "messages": case.get("messages").cloned().ok_or_else(|| missing("messages"))?,
        "temperature": manifest.pointer("/generation/temperature").cloned().ok_or_else(|| missing("generation.temperature"))?,
        "top_p": manifest.pointer("/generation/top_p").cloned().ok_or_else(|| missing("generation.top_p"))?,
        "max_tokens": manifest.pointer("/generation/max_tokens").cloned().ok_or_else(|| missing("generation.max_tokens"))?,
        "stream": manifest.pointer("/generation/stream").cloned().ok_or_else(|| missing("generation.stream"))?,
        "seed": manifest.pointer("/generation/seed").cloned().ok_or_else(|| missing("generation.seed"))?
    }))
}

fn validate_sidecar(manifest: &Value) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_MANIFEST_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str) != Some(CALIBRATION_MANIFEST_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(manifest)?.as_str())
    {
        return Err(invalid("calibration manifest fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn read_attempt_002_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_002_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_002_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_002_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-002-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 2)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-002 candidate identity changed"));
    }
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    let frozen_hashes = identity
        .get("frozen_request_hashes")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("frozen_request_hashes"))?;
    for (case_id, expected_hash) in [
        (
            "rbcal-001",
            "f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461",
        ),
        (
            "rbcal-002",
            "e9cb29143ed1be27ce5c5b27bda4daa546ff63825189b170b37083624534c1b3",
        ),
        (
            "rbcal-003",
            "2c9839a9482080b3d03fa89d908c63d442d35ec64e142d5c573d276e801aec7e",
        ),
    ] {
        if frozen_hashes.get(case_id).and_then(Value::as_str) != Some(expected_hash) {
            return Err(invalid("attempt-002 request identity changed"));
        }
    }
    expect_string(
        identity,
        "lineage.attempt_001_evidence_root",
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/",
    )?;
    expect_string(
        identity,
        "lineage.attempt_001_request_sha256",
        "f32863dfb1da27c00a61d54986d4984569c87e9636cf5c6263c69906cb336461",
    )?;
    expect_string(
        identity,
        "lineage.attempt_001_classification",
        "INVALID / AMBIGUOUS",
    )?;
    expect_string(
        identity,
        "lineage.attempt_001_reason",
        "LLAMA_RUNTIME_FAILURE_WITH_UNCERTAIN_REQUEST_COMPLETION",
    )?;
    expect_u64(identity, "lineage.attempt_001_request_attempts", 1)?;
    expect_bool(identity, "lineage.attempt_001_immutable", true)?;
    expect_string(identity, "lineage.root_cause", "ROOT CAUSE NOT ESTABLISHED")?;
    expect_string(
        identity,
        "attempt_002_evidence_root",
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-002/",
    )?;
    expect_string(identity, "runtime.engine", "llama.cpp")?;
    expect_string(identity, "runtime.server_build", "b10217-ddd4ec142")?;
    expect_string(identity, "runtime.model", MODEL_ID)?;
    expect_string(identity, "runtime.quantization", "Q4_0")?;
    expect_u64(identity, "runtime.context_size", 8192)?;
    expect_u64(identity, "runtime.parallel_slots", 1)?;
    expect_string(identity, "runtime.metrics", "enabled")?;
    expect_string(identity, "runtime.reasoning", "on")?;
    expect_string(identity, "runtime.reasoning_budget_message", "unset")?;
    expect_string(identity, "runtime.host", HOST)?;
    expect_u64(identity, "runtime.port", PORT as u64)?;
    expect_bool(
        identity,
        "runtime_exclusivity.post_start_pid_record_required",
        true,
    )?;
    expect_bool(
        identity,
        "runtime_exclusivity.single_llama_pid_required",
        true,
    )?;
    expect_bool(
        identity,
        "runtime_exclusivity.port_owner_must_match_pid",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_002_root_must_begin_absent",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_001_must_not_be_overwritten",
        true,
    )?;
    expect_bool(identity, "evidence_policy.no_manifest_copy", true)?;
    expect_bool(identity, "evidence_policy.no_response_repair", true)?;
    expect_bool(identity, "evidence_policy.no_retry", true)?;
    if verify_sidecar {
        validate_attempt_002_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_002_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_002_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_002_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-002 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_002_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-002 manifest binding mismatch"));
    }
    let expected_cases = identity
        .get("frozen_request_hashes")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("frozen_request_hashes"))?;
    let actual_cases = fingerprints
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    for case_id in CALIBRATION_CASE_IDS {
        let expected = expected_cases
            .get(case_id)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(case_id))?;
        let actual = actual_cases
            .iter()
            .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
            .and_then(|case| case.get("request_sha256"))
            .and_then(Value::as_str);
        if actual != Some(expected) {
            return Err(invalid("attempt-002 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_candidate_order(budget: u32) -> Result<(), ReasoningBudgetCalibrationError> {
    if budget == CALIBRATION_BUDGETS[0] {
        return Ok(());
    }
    let index = CALIBRATION_BUDGETS
        .iter()
        .position(|candidate| *candidate == budget)
        .ok_or_else(|| invalid("unregistered calibration budget"))?;
    let previous = CALIBRATION_BUDGETS[index - 1];
    let previous_result = candidate_result_path(previous);
    if !previous_result.exists() {
        return Err(invalid(
            "candidate order is not satisfied: prior candidate result is absent",
        ));
    }
    let previous_value = read_json_path(&previous_result)?;
    if previous_value.get("case_set_complete") != Some(&Value::Bool(true)) {
        return Err(invalid(
            "candidate order is not satisfied: prior case set is incomplete",
        ));
    }
    if previous_value.get("state").and_then(Value::as_str) == Some("PASS") {
        return Err(invalid(
            "candidate order stopped after a passing prior candidate",
        ));
    }
    Ok(())
}

fn aggregate_state(results: &[Value]) -> &'static str {
    if results
        .iter()
        .all(|result| result.get("state").and_then(Value::as_str) == Some("PASS"))
    {
        "PASS"
    } else if results
        .iter()
        .any(|result| result.get("state").and_then(Value::as_str) == Some("INCONCLUSIVE"))
    {
        "INCONCLUSIVE"
    } else {
        "FAIL"
    }
}

fn candidate_result(
    budget: u32,
    results: Vec<Value>,
    inference_requests: u32,
    state: &str,
    case_set_complete: bool,
    error: Option<&str>,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let case_summaries = results
        .iter()
        .map(|result| {
            json!({
                "case_id": result.get("case_id"),
                "state": result.get("state"),
                "request_sha256": result.pointer("/request/request_sha256"),
                "response_sha256": result.pointer("/response/response_body_sha256"),
                "http_status": result.pointer("/response/http_status"),
                "prompt_tokens": result.pointer("/response/prompt_tokens"),
                "completion_tokens": result.pointer("/response/completion_tokens"),
                "cached_tokens": result.pointer("/response/cached_tokens"),
                "reasoning_content_present": result.pointer("/response/reasoning_content_present"),
                "finish_reason": result.pointer("/response/finish_reason"),
                "response_body_bytes": result.pointer("/response/response_body_bytes"),
                "transport_elapsed_ms": result.pointer("/response/transport_elapsed_ms"),
                "terminal_final_content": result.pointer("/validation/terminal_final_content"),
                "structural_response_valid": result.pointer("/validation/structural_response_valid"),
                "reasoning_diagnostic": result.get("reasoning_diagnostic")
            })
        })
        .collect::<Vec<_>>();
    let next_budget = if case_set_complete {
        next_candidate_budget(budget, state)
    } else {
        None
    };
    Ok(json!({
        "schema_id": CANDIDATE_RESULT_SCHEMA_ID,
        "schema_version": 1,
        "experiment_id": EXPERIMENT_ID,
        "purpose": "NON-SCORED RUNTIME FEASIBILITY CALIBRATION",
        "budget": budget,
        "candidate_order": CALIBRATION_BUDGETS,
        "case_order": CALIBRATION_CASE_IDS,
        "max_turns": MAX_TURNS,
        "state": state,
        "case_set_complete": case_set_complete,
        "cases": case_summaries,
        "request": {
            "maximum_requests": 3,
            "inference_requests": inference_requests,
            "automatic_retries": 0,
            "fallback_requests": 0
        },
        "selection": {
            "next_budget": next_budget,
            "selected_budget": if state == "PASS" { json!(budget) } else { Value::Null },
            "terminal_classification": if budget == 256 && state != "PASS" {
                json!("REASONING-ON / 2048-TOKEN SCORED CONFIGURATION NOT FEASIBLE")
            } else {
                Value::Null
            }
        },
        "network_calls": if inference_requests > 0 { 4 } else { 1 },
        "listener_check_attempts": 1,
        "inference_requests": inference_requests,
        "automatic_retries": 0,
        "reasoning_content_is_diagnostic_only": true,
        "capability_claims_permitted": false,
        "error": error
    }))
}

fn structurally_matches(content: &str, expected: &Value) -> bool {
    let trimmed = content.trim();
    if trimmed.starts_with("```") || trimmed.ends_with("```") {
        return false;
    }
    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return false;
    };
    value.as_object().is_some_and(|object| object.len() == 3) && value == *expected
}

fn server_launch_arguments(budget: u32) -> Vec<String> {
    vec![
        "serve".to_string(),
        "-hf".to_string(),
        MODEL_ID.to_string(),
        "-c".to_string(),
        "8192".to_string(),
        "-np".to_string(),
        "1".to_string(),
        "--metrics".to_string(),
        "--reasoning".to_string(),
        "on".to_string(),
        "--reasoning-budget".to_string(),
        budget.to_string(),
        "--host".to_string(),
        HOST.to_string(),
        "--port".to_string(),
        PORT.to_string(),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct ProcessObservation {
    image_name: String,
    pid: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct ListenerObservation {
    local_address: String,
    pid: u32,
    state: String,
}

const MAX_OS_INSPECTION_OUTPUT_BYTES: usize = 64 * 1024;

fn tasklist_processes() -> Result<Vec<ProcessObservation>, ReasoningBudgetCalibrationError> {
    let output = bounded_os_command("tasklist", &["/FO", "CSV", "/NH"])?;
    Ok(String::from_utf8_lossy(&output)
        .lines()
        .filter_map(parse_tasklist_row)
        .collect())
}

fn netstat_port_listeners(
    port: u16,
) -> Result<Vec<ListenerObservation>, ReasoningBudgetCalibrationError> {
    let output = bounded_os_command("netstat", &["-ano", "-p", "tcp"])?;
    Ok(String::from_utf8_lossy(&output)
        .lines()
        .filter_map(|line| parse_netstat_listener(line, port))
        .collect())
}

fn bounded_os_command(
    program: &str,
    args: &[&str],
) -> Result<Vec<u8>, ReasoningBudgetCalibrationError> {
    let output = Command::new(program).args(args).output().map_err(|error| {
        invalid(&format!(
            "bounded OS inspection command {program} failed to start: {error}"
        ))
    })?;
    if !output.status.success() {
        return Err(invalid(&format!(
            "bounded OS inspection command {program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    if output.stdout.len() > MAX_OS_INSPECTION_OUTPUT_BYTES {
        return Err(invalid(&format!(
            "bounded OS inspection command {program} exceeded output limit"
        )));
    }
    Ok(output.stdout)
}

fn parse_tasklist_row(line: &str) -> Option<ProcessObservation> {
    let fields = line
        .trim()
        .strip_prefix('"')?
        .strip_suffix('"')?
        .split("\",\"")
        .collect::<Vec<_>>();
    Some(ProcessObservation {
        image_name: fields.first()?.to_string(),
        pid: fields.get(1)?.parse().ok()?,
    })
}

fn parse_netstat_listener(line: &str, port: u16) -> Option<ListenerObservation> {
    let fields = line.split_whitespace().collect::<Vec<_>>();
    if fields.len() < 5 || !fields[0].eq_ignore_ascii_case("TCP") {
        return None;
    }
    let local_address = fields[1];
    let local_port = local_address.rsplit_once(':')?.1.parse::<u16>().ok()?;
    if local_port != port || !fields[3].eq_ignore_ascii_case("LISTENING") {
        return None;
    }
    Some(ListenerObservation {
        local_address: local_address.to_string(),
        pid: fields[4].parse().ok()?,
        state: fields[3].to_string(),
    })
}

fn is_llama_process(image_name: &str) -> bool {
    let image_name = image_name.to_ascii_lowercase();
    image_name == "llama.exe" || image_name.ends_with("\\llama.exe")
}

fn is_competing_process(image_name: &str) -> bool {
    let image_name = image_name.to_ascii_lowercase();
    ["prefixity", "qwen", "luna", "codex"]
        .iter()
        .any(|marker| image_name.contains(marker))
}

fn safe_headers(headers: &reqwest::header::HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            Some((name.as_str().to_string(), value.to_str().ok()?.to_string()))
        })
        .collect()
}

fn ensure_budget(budget: u32) -> Result<(), ReasoningBudgetCalibrationError> {
    if CALIBRATION_BUDGETS.contains(&budget) {
        Ok(())
    } else {
        Err(invalid(
            "budget is not one of the pre-registered candidates",
        ))
    }
}

fn parse_budget(value: &str) -> Result<u32, ReasoningBudgetCalibrationError> {
    let budget = value
        .parse::<u32>()
        .map_err(|_| invalid("budget must be an integer candidate"))?;
    ensure_budget(budget)?;
    Ok(budget)
}

fn parse_server_pid(value: &str) -> Result<u32, ReasoningBudgetCalibrationError> {
    let pid = value
        .parse::<u32>()
        .map_err(|_| invalid("server PID must be an integer"))?;
    if pid == 0 {
        return Err(invalid("server PID must be nonzero"));
    }
    Ok(pid)
}

fn read_manifest(verify_sidecar: bool) -> Result<Value, ReasoningBudgetCalibrationError> {
    let manifest = read_json(CALIBRATION_MANIFEST_PATH)?;
    if verify_sidecar {
        validate_sidecar(&manifest)?;
    }
    Ok(manifest)
}

fn read_json(path: &str) -> Result<Value, ReasoningBudgetCalibrationError> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn read_json_path(path: &Path) -> Result<Value, ReasoningBudgetCalibrationError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: &Path, value: &Value) -> Result<(), ReasoningBudgetCalibrationError> {
    write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), ReasoningBudgetCalibrationError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn candidate_root(budget: u32) -> PathBuf {
    candidate_root_at(CALIBRATION_EVIDENCE_ROOT, budget)
}

fn candidate_result_path(budget: u32) -> PathBuf {
    candidate_result_path_at(CALIBRATION_EVIDENCE_ROOT, budget)
}

fn candidate_root_at(evidence_root: &str, budget: u32) -> PathBuf {
    workspace_path(&format!("{evidence_root}/budget-{budget}"))
}

fn candidate_result_path_at(evidence_root: &str, budget: u32) -> PathBuf {
    candidate_root_at(evidence_root, budget).join("candidate-result.json")
}

fn attempt_002_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT)
}

fn attempt_002_result_path() -> PathBuf {
    attempt_002_root().join("candidate-result.json")
}

fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn now_unix_ms() -> Result<u64, ReasoningBudgetCalibrationError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| invalid(&format!("system clock before UNIX epoch: {error}")))?
        .as_millis() as u64)
}

fn expect_string(
    value: &Value,
    path: &str,
    expected: &str,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_str)
        != Some(expected)
    {
        return Err(invalid(&format!("value mismatch at {path}")));
    }
    Ok(())
}

fn expect_u64(
    value: &Value,
    path: &str,
    expected: u64,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_u64)
        != Some(expected)
    {
        return Err(invalid(&format!("integer mismatch at {path}")));
    }
    Ok(())
}

fn expect_bool(
    value: &Value,
    path: &str,
    expected: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_bool)
        != Some(expected)
    {
        return Err(invalid(&format!("boolean mismatch at {path}")));
    }
    Ok(())
}

fn missing(path: &str) -> ReasoningBudgetCalibrationError {
    invalid(&format!("missing {path}"))
}

fn invalid(message: &str) -> ReasoningBudgetCalibrationError {
    ReasoningBudgetCalibrationError::Validation(message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_independent_and_has_three_cases() {
        let manifest = read_manifest(false).unwrap();
        validate_manifest(&manifest, true).unwrap();
        assert_eq!(manifest["cases"].as_array().unwrap().len(), 3);
        let cases = serde_json::to_string(&manifest["cases"])
            .unwrap()
            .to_ascii_lowercase();
        assert!(!cases.contains("h001"));
    }

    #[test]
    fn dry_run_covers_all_nine_combinations_with_zero_contact() {
        if workspace_path(CALIBRATION_EVIDENCE_ROOT).exists() {
            // The live candidate freezes the evidence root; preparation-only
            // dry-run assertions are intentionally not rerun over it.
            return;
        }
        let result = dry_run_calibration().unwrap();
        assert_eq!(result["state"], "DRY_RUN");
        assert_eq!(result["combinations"].as_array().unwrap().len(), 9);
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["inference_requests"], 0);
        assert!(result["combinations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| {
                entry["network_calls"] == 0
                    && entry["inference_requests"] == 0
                    && entry["request_has_reasoning_budget_field"] == false
            }));
    }

    #[test]
    fn candidate_order_is_descending_and_fixed() {
        assert_eq!(CALIBRATION_BUDGETS, [1024, 512, 256]);
        assert_eq!(next_candidate_budget(1024, "FAIL"), Some(512));
        assert_eq!(next_candidate_budget(512, "INCONCLUSIVE"), Some(256));
        assert_eq!(next_candidate_budget(1024, "PASS"), None);
        assert_eq!(next_candidate_budget(256, "FAIL"), None);
    }

    #[test]
    fn canonical_run_parser_accepts_only_registered_budget() {
        assert_eq!(
            parse_calibration_cli_args(vec![
                "run".to_string(),
                "--budget".to_string(),
                "1024".to_string(),
                "--confirm-fresh-runtime".to_string()
            ])
            .unwrap(),
            CalibrationCliCommand::Run { budget: 1024 }
        );
        assert!(parse_calibration_cli_args(vec![
            "run".to_string(),
            "--budget".to_string(),
            "128".to_string(),
            "--confirm-fresh-runtime".to_string()
        ])
        .is_err());
    }

    #[test]
    fn malformed_run_confirmation_fails_closed() {
        assert!(parse_calibration_cli_args(vec![
            "run".to_string(),
            "--confirm-fresh-runtime".to_string(),
            "--budget".to_string(),
            "1024".to_string()
        ])
        .is_err());
        assert!(parse_calibration_cli_args(vec![
            "run".to_string(),
            "--budget".to_string(),
            "1024".to_string()
        ])
        .is_err());
    }

    #[test]
    fn request_projection_has_no_budget_field_and_dry_run_is_zero_contact() {
        let manifest = read_manifest(false).unwrap();
        let case = &manifest["cases"][0];
        let request = build_request(&manifest, case).unwrap();
        assert!(request.get("reasoning_budget").is_none());
        assert_eq!(request["max_tokens"], 2048);
    }

    #[test]
    fn server_launch_adds_only_the_registered_budget_setting() {
        let args = server_launch_arguments(512);
        assert_eq!(
            args[8..12].to_vec(),
            vec![
                "--reasoning".to_string(),
                "on".to_string(),
                "--reasoning-budget".to_string(),
                "512".to_string()
            ]
        );
        assert!(!args.contains(&"--reasoning-budget-message".to_string()));
    }

    #[test]
    fn selection_does_not_choose_an_unregistered_budget() {
        assert!(parse_budget("2048").is_err());
        assert!(parse_budget("0").is_err());
        assert!(parse_budget("-1").is_err());
    }

    #[test]
    fn attempt_002_identity_is_fingerprinted_and_bound_to_attempt_001() {
        let identity = read_attempt_002_identity(true).unwrap();
        validate_attempt_002_identity(&identity, true).unwrap();
        assert_eq!(
            canonical_hash(&identity).unwrap(),
            "7d9dd05ed5c855f02dc5b37a70e7cac257af03ce5dc87686acf1e64590f37610"
        );
        assert_eq!(
            identity["attempt_002_evidence_root"],
            format!("{CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT}/")
        );
        assert_ne!(
            identity["attempt_002_evidence_root"],
            CALIBRATION_EVIDENCE_ROOT
        );
    }

    #[test]
    fn attempt_002_dry_run_is_isolated_and_zero_contact() {
        if attempt_002_root().exists() {
            // A live attempt freezes this root; preparation-only assertions
            // are not rerun over preserved execution evidence.
            return;
        }
        let result = dry_run_attempt_002().unwrap();
        assert_eq!(result["state"], "DRY_RUN");
        assert_eq!(result["attempt"], 2);
        assert_eq!(result["combinations"].as_array().unwrap().len(), 3);
        assert_eq!(result["network_calls"], 0);
        assert_eq!(result["listener_checks"], 0);
        assert_eq!(result["inference_requests"], 0);
        assert!(result["combinations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|case| {
                case["budget"] == 1024
                    && case["request_has_reasoning_budget_field"] == false
                    && case["network_calls"] == 0
                    && case["listener_checks"] == 0
                    && case["inference_requests"] == 0
                    && case["evidence_root"]
                        .as_str()
                        .is_some_and(|path| path.starts_with(CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT))
            }));
    }

    #[test]
    fn attempt_002_cli_requires_exclusive_confirmation_and_pid() {
        assert_eq!(
            parse_calibration_cli_args(vec![
                "attempt-002-exclusivity-preflight".to_string(),
                "--confirm-no-other-workflow".to_string()
            ])
            .unwrap(),
            CalibrationCliCommand::Attempt002ExclusivityPreflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec![
                "run-attempt-002".to_string(),
                "--budget".to_string(),
                "1024".to_string(),
                "--server-pid".to_string(),
                "14588".to_string(),
                "--confirm-fresh-runtime".to_string(),
                "--confirm-exclusive-runtime".to_string()
            ])
            .unwrap(),
            CalibrationCliCommand::RunAttempt002 {
                budget: 1024,
                server_pid: 14588
            }
        );
        assert!(parse_calibration_cli_args(vec![
            "run-attempt-002".to_string(),
            "--budget".to_string(),
            "1024".to_string(),
            "--server-pid".to_string(),
            "0".to_string(),
            "--confirm-fresh-runtime".to_string(),
            "--confirm-exclusive-runtime".to_string()
        ])
        .is_err());
    }

    #[test]
    fn bounded_runtime_inspection_parses_only_listeners_and_process_identity() {
        assert_eq!(
            parse_tasklist_row("\"llama.exe\",\"14588\",\"Console\",\"1\",\"10,000 K\""),
            Some(ProcessObservation {
                image_name: "llama.exe".to_string(),
                pid: 14588
            })
        );
        assert_eq!(
            parse_netstat_listener(
                "  TCP    127.0.0.1:8080    0.0.0.0:0    LISTENING    14588",
                8080
            ),
            Some(ListenerObservation {
                local_address: "127.0.0.1:8080".to_string(),
                pid: 14588,
                state: "LISTENING".to_string()
            })
        );
        assert!(is_llama_process("llama.exe"));
        assert!(is_competing_process("prefixity-helper.exe"));
        assert!(!is_competing_process("cargo.exe"));
    }
}
