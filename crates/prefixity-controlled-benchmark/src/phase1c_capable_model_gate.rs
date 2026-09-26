//! Phase 1C capable measuring-model feasibility gate: one bounded competence
//! gate for the local Qwen3.5-9B Q4_K_M instrument with reasoning off.
//!
//! The gate tests the measuring model, not Prefixity. Everything
//! gate-specific lives in [`CapableModelGateSpec`], derived deterministically
//! from the tracked capable-model runtime contract, the frozen calibration
//! manifest, and the frozen h001 artifacts; the gate identity must carry
//! exactly that spec. Stage A sends the three frozen `rbcal` structural probes
//! to the unchanged calibration evaluator; only if all three pass, Stage B
//! sends one h001 BASELINE trajectory (the frozen no-tool contract terminates
//! on the first response) to the unchanged Stage 0 evaluator. Every request is
//! counted with `/v1/chat/completions/input_tokens` and guarded before
//! dispatch. The V3 types, deadline derivation, and live helpers are reused;
//! there is no per-attempt validator family.

use crate::hashing::canonical_hash;
use crate::phase1c_executable_identity as executable_identity;
use crate::phase1c_h001::{self as h001, H001Arm, H001Error, H001TurnInput};
use crate::phase1c_reasoning_budget_calibration::{
    build_request, execute_case, expected_workflow_identity_from_supervisor_env, now_unix_ms,
    read_json, same_workflow_identity_path, validate_accepted_workflow_certification_v2,
    windows_native_prestart_value, workspace_path, write_bytes, write_json,
    ReasoningBudgetCalibrationError, CALIBRATION_CASE_IDS, ENDPOINT, HOST, PORT,
};
use crate::phase1c_v3_feasibility::{
    compare_file_identity, context_guard, count_input_tokens, expect, field, invalid, number,
    poststart_ownership, text, tracked_manifest, validate_frozen_executables,
    validate_launch_environment, validate_source_binding, validate_supervisor_deadline, CaseSpec,
    ContextDecision, DeadlinesSpec, FileIdentitySpec, LimitsSpec, INCONCLUSIVE_CONTEXT_BOUND,
    V3_LLAMA_EXECUTABLE_FILE_ID,
};
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, ReasoningBudgetCalibrationError>;

pub const LOCAL_9B_CONTRACT_PATH: &str =
    "docs/phase-1/PHASE_1C_CAPABLE_MODEL_RUNTIME_CONTRACT_LOCAL_9B_V1.json";
pub const LOCAL_9B_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_CAPABLE_MODEL_RUNTIME_CONTRACT_LOCAL_9B_V1.sha256";
pub const LOCAL_9B_GATE_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_IDENTITY_V1.json";
pub const LOCAL_9B_GATE_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_LOCAL_9B_FEASIBILITY_GATE_IDENTITY_V1.sha256";
pub const LOCAL_9B_GATE_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-capable-model-local-9b/feasibility-gate";
pub const LOCAL_9B_GATE_FROZEN_STAGE_ROOT: &str = "target/phase1c-local-9b-feasibility-frozen";
pub const LOCAL_9B_GATE_TRAVERSAL_ROOT: &str =
    "target/phase1c-local-9b-feasibility-prerequisite-traversal";
pub const LOCAL_9B_GATE_ID: &str = "phase1c-local-9b-feasibility-gate";
pub const LOCAL_9B_EXPERIMENT_ID: &str = "phase-1c-capable-model-local-9b";
pub const LOCAL_9B_MAX_TOKENS: u64 = 1024;
pub const LOCAL_9B_CONTEXT_TOKENS: u64 = 8192;
/// Upstream LFS SHA-256 of the selected artifact, equal to the local file.
pub const LOCAL_9B_GGUF_SHA256: &str =
    "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13";
pub const LOCAL_9B_TOKEN_COUNT_ENDPOINT: &str =
    "http://127.0.0.1:8080/v1/chat/completions/input_tokens";

pub const LOCAL_9B_FEASIBILITY_PASSED: &str = "LOCAL_9B_FEASIBILITY_PASSED";
pub const LOCAL_9B_FEASIBILITY_FAILED: &str = "LOCAL_9B_FEASIBILITY_FAILED";
pub const LOCAL_9B_GATE_PRE_INFERENCE_FAILURE: &str = "GATE_PRE_INFERENCE_FAILURE";

const CHILD_BINARY: &str = "prefixity-phase1c-capable-model-gate.exe";
const SUPERVISOR_BINARY: &str = "prefixity-phase1c-live-supervisor.exe";
const STRUCTURAL_REQUESTS: u32 = 3;
const TASK_TURN_CEILING: u32 = 3;

/// Implementation sources whose current bytes must equal the prepared ones.
pub const LOCAL_9B_BOUND_SOURCES: [&str; 9] = [
    "crates/prefixity-controlled-benchmark/src/phase1c_capable_model_gate.rs",
    "crates/prefixity-controlled-benchmark/src/bin/phase1c_capable_model_gate.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_v3_feasibility.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_h001.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_live_supervisor.rs",
    "crates/prefixity-controlled-benchmark/src/bin/phase1c_live_supervisor.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_windows_runtime_exclusivity.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_executable_identity.rs",
];

/// Request-body fields that could change reasoning or template semantics per
/// request in the frozen b10217 server (`oaicompat_chat_params_parse`).
pub const FORBIDDEN_REQUEST_FIELDS: [&str; 6] = [
    "chat_template_kwargs",
    "reasoning_effort",
    "reasoning_format",
    "reasoning_budget_tokens",
    "thinking_budget_tokens",
    "reasoning_budget_message",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSourceSpec {
    pub repository: String,
    pub revision: String,
    pub filename: String,
    pub upstream_sha256: String,
    pub license: String,
    pub base_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReasoningSpec {
    pub mode: String,
    pub launch_flag: Vec<String>,
    pub forbidden_request_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatorSpec {
    pub structural_manifest_sha256: String,
    pub task_evaluator_version: String,
    pub task_evaluator_sha256: String,
    pub task_tool_contract_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskCaseSpec {
    pub task_id: String,
    pub arm: String,
    pub request_sha256: String,
    pub turn_ceiling: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageLimitsSpec {
    pub structural_requests: u32,
    pub task_turn_ceiling: u32,
    pub task_stage_requires_all_structural_pass: bool,
}

/// The single data structure that parameterizes the capable-model gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapableModelGateSpec {
    pub gate_id: String,
    pub experiment_id: String,
    pub contract_path: String,
    pub contract_sha256: String,
    pub runtime_version: String,
    pub runtime_executable: FileIdentitySpec,
    pub model_file: FileIdentitySpec,
    pub model_source: ModelSourceSpec,
    pub model_label: String,
    pub launch_arguments: Vec<String>,
    pub forbidden_launch_arguments: Vec<String>,
    pub forbidden_environment_prefix: String,
    pub reasoning: ReasoningSpec,
    pub structural_cases: Vec<CaseSpec>,
    pub task_case: TaskCaseSpec,
    pub evaluators: EvaluatorSpec,
    pub stages: StageLimitsSpec,
    pub limits: LimitsSpec,
    pub deadlines: DeadlinesSpec,
    pub evidence_root: String,
    pub token_count_endpoint: String,
    pub inference_endpoint: String,
}

/// The exact launch arguments registered for a model path.
pub fn local_9b_launch_arguments(model_path: &str) -> Vec<String> {
    [
        "serve",
        "-m",
        model_path,
        "-c",
        "8192",
        "-np",
        "1",
        "--metrics",
        "--reasoning",
        "off",
        "--offline",
        "--host",
        HOST,
        "--port",
        "8080",
    ]
    .iter()
    .map(|argument| argument.to_string())
    .collect()
}

/// The registered gate contact limits.
fn gate_limits() -> LimitsSpec {
    LimitsSpec {
        context_tokens: LOCAL_9B_CONTEXT_TOKENS,
        max_tokens: LOCAL_9B_MAX_TOKENS,
        readiness_contacts: 1,
        token_count_contacts: STRUCTURAL_REQUESTS + TASK_TURN_CEILING,
        inference_requests: STRUCTURAL_REQUESTS + TASK_TURN_CEILING,
        retry_requests: 0,
        fallback_requests: 0,
        adaptive_replicates: 0,
        warmup_requests: 0,
    }
}

fn contract_deadlines(contract: &Value) -> Result<DeadlinesSpec> {
    let policy = |key: &str| number(contract, &format!("/timeout_policy/{key}"));
    Ok(DeadlinesSpec {
        connect_timeout_ms: policy("connect_timeout_ms")?,
        readiness_timeout_ms: policy("readiness_timeout_ms")?,
        token_count_request_timeout_ms: policy("token_count_request_timeout_ms")?,
        inference_request_timeout_ms: policy("inference_request_timeout_ms")?,
        non_request_margin_ms: policy("non_request_margin_ms")?,
        supervisor_deadline_ms: policy("supervisor_deadline_ms")?,
    })
}

fn string_list(value: &Value, pointer: &str) -> Result<Vec<String>> {
    field(value, pointer)?
        .as_array()
        .ok_or_else(|| invalid(&format!("value {pointer} is not an array")))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| invalid(&format!("value {pointer} holds a non-string")))
        })
        .collect()
}

/// Validate the tracked capable-model runtime contract against the accepted
/// design (`PHASE_1C_CAPABLE_MODEL_DESIGN_DECISION.md`).
pub fn validate_local_9b_contract(contract: &Value) -> Result<()> {
    expect(
        contract,
        "/contract_version",
        json!("phase1c-capable-model-runtime-local-9b-v1"),
    )?;
    expect(contract, "/experiment_id", json!(LOCAL_9B_EXPERIMENT_ID))?;
    expect(
        contract,
        "/status",
        json!("PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED"),
    )?;
    expect(
        contract,
        "/runtime/executable/windows_file_id",
        json!(V3_LLAMA_EXECUTABLE_FILE_ID),
    )?;
    expect(
        contract,
        "/runtime/executable/sha256",
        json!("cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617"),
    )?;
    expect(
        contract,
        "/runtime/executable/version",
        json!("b10217-ddd4ec142"),
    )?;
    expect(
        contract,
        "/runtime/model/sha256",
        json!(LOCAL_9B_GGUF_SHA256),
    )?;
    expect(
        contract,
        "/runtime/model/upstream_sha256",
        json!(LOCAL_9B_GGUF_SHA256),
    )?;
    expect(contract, "/runtime/model/quantization", json!("Q4_K_M"))?;
    expect(contract, "/runtime/model/architecture", json!("qwen35"))?;
    expect(
        contract,
        "/runtime/context_size",
        json!(LOCAL_9B_CONTEXT_TOKENS),
    )?;
    expect(contract, "/runtime/parallel_slots", json!(1))?;
    expect(contract, "/runtime/metrics", json!("enabled"))?;
    expect(contract, "/runtime/reasoning", json!("off"))?;
    expect(
        contract,
        "/runtime/reasoning_mechanism/launch_flag",
        json!(["--reasoning", "off"]),
    )?;
    expect(
        contract,
        "/runtime/reasoning_mechanism/forbidden_request_fields",
        json!(FORBIDDEN_REQUEST_FIELDS),
    )?;
    expect(contract, "/runtime/host", json!(HOST))?;
    expect(contract, "/runtime/port", json!(PORT))?;
    expect(contract, "/runtime/endpoint", json!(ENDPOINT))?;
    expect(
        contract,
        "/runtime/forbidden_environment_prefix",
        json!("LLAMA_ARG_"),
    )?;
    expect(
        contract,
        "/generation/max_tokens",
        json!(LOCAL_9B_MAX_TOKENS),
    )?;
    expect(contract, "/generation/temperature", json!(0))?;
    expect(contract, "/generation/top_p", json!(1))?;
    expect(contract, "/generation/seed", json!(1))?;
    expect(contract, "/generation/stream", json!(false))?;
    expect(
        contract,
        "/generation/additional_sampling_parameters",
        json!({}),
    )?;
    expect(
        contract,
        "/context_budget/context_tokens",
        json!(LOCAL_9B_CONTEXT_TOKENS),
    )?;
    expect(
        contract,
        "/context_budget/output_ceiling_tokens",
        json!(LOCAL_9B_MAX_TOKENS),
    )?;
    expect(
        contract,
        "/context_budget/token_counter_is_inference",
        json!(false),
    )?;
    expect(
        contract,
        "/gate/structural_requests",
        json!(STRUCTURAL_REQUESTS),
    )?;
    expect(
        contract,
        "/gate/task_turn_ceiling",
        json!(TASK_TURN_CEILING),
    )?;
    expect(contract, "/gate/max_inference_requests", json!(6))?;
    for key in [
        "automatic_retries",
        "fallback_requests",
        "adaptive_replicates",
        "warmup_requests",
    ] {
        expect(contract, &format!("/retry_policy/{key}"), json!(0))?;
    }
    let model_path = text(contract, "/runtime/model/path")?;
    if !model_path
        .to_ascii_uppercase()
        .starts_with("D:\\PREFIXITY-LAB\\MODELS\\")
    {
        return Err(invalid(
            "capable-model GGUF must be stored under D:\\Prefixity-Lab\\models\\",
        ));
    }
    let launch = field(contract, "/runtime/launch_arguments")?;
    if launch != &json!(local_9b_launch_arguments(&model_path)) {
        return Err(invalid("capable-model contract launch arguments changed"));
    }
    let forbidden = string_list(contract, "/runtime/forbidden_launch_arguments")?;
    for required in [
        "--reasoning-budget",
        "--chat-template-kwargs",
        "-hf",
        "--hf-repo",
        "--mmproj",
    ] {
        if !forbidden.iter().any(|argument| argument == required) {
            return Err(invalid(&format!(
                "capable-model contract does not forbid {required}"
            )));
        }
    }
    let launch_arguments = string_list(contract, "/runtime/launch_arguments")?;
    if launch_arguments
        .iter()
        .any(|argument| forbidden.contains(argument))
    {
        return Err(invalid(
            "capable-model launch arguments include a forbidden argument",
        ));
    }
    validate_supervisor_deadline(&gate_limits(), &contract_deadlines(contract)?)?;
    Ok(())
}

/// Reject any request field that could re-enable reasoning or alter template
/// semantics per request.
pub fn validate_reasoning_neutral_request(request: &Value) -> Result<()> {
    let object = request
        .as_object()
        .ok_or_else(|| invalid("capable-model request is not a JSON object"))?;
    if let Some(key) = FORBIDDEN_REQUEST_FIELDS
        .iter()
        .find(|key| object.contains_key(**key))
    {
        return Err(invalid(&format!(
            "capable-model request carries forbidden reasoning field {key}"
        )));
    }
    Ok(())
}

/// The calibration manifest re-labelled for this gate: experiment id, model
/// label, and output ceiling. Case material is unchanged.
pub fn gate_calibration_manifest(manifest: &Value, model_label: &str) -> Result<Value> {
    let mut gate_manifest = manifest.clone();
    for (pointer, value) in [
        ("/experiment_id", json!(LOCAL_9B_EXPERIMENT_ID)),
        ("/runtime/model", json!(model_label)),
        ("/generation/max_tokens", json!(LOCAL_9B_MAX_TOKENS)),
    ] {
        *gate_manifest
            .pointer_mut(pointer)
            .ok_or_else(|| invalid(&format!("calibration manifest {pointer} is missing")))? = value;
    }
    Ok(gate_manifest)
}

/// The three frozen structural probes as gate request bodies.
pub fn structural_requests(gate_manifest: &Value) -> Result<Vec<(String, Value, Value)>> {
    let cases = gate_manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("manifest cases are missing"))?;
    CALIBRATION_CASE_IDS
        .iter()
        .map(|case_id| {
            let case = cases
                .iter()
                .find(|case| case.get("case_id").and_then(Value::as_str) == Some(case_id))
                .ok_or_else(|| invalid(&format!("manifest case {case_id} is missing")))?;
            let request = build_request(gate_manifest, case)?;
            validate_reasoning_neutral_request(&request)?;
            Ok((case_id.to_string(), case.clone(), request))
        })
        .collect()
}

fn h001_error(error: H001Error) -> ReasoningBudgetCalibrationError {
    invalid(&error.to_string())
}

/// The frozen h001 BASELINE request with only the model label and output
/// ceiling substituted.
pub fn task_request(model_label: &str) -> Result<Value> {
    let dry_run = h001::dry_run_h001(H001Arm::Baseline).map_err(h001_error)?;
    let mut request = dry_run
        .get("model_visible_request")
        .cloned()
        .ok_or_else(|| invalid("h001 dry run omitted the model request"))?;
    for (key, value) in [
        ("model", json!(model_label)),
        ("max_tokens", json!(LOCAL_9B_MAX_TOKENS)),
    ] {
        *request
            .get_mut(key)
            .ok_or_else(|| invalid(&format!("h001 request {key} is missing")))? = value;
    }
    validate_reasoning_neutral_request(&request)?;
    Ok(request)
}

fn file_identity_spec(contract: &Value, pointer: &str) -> Result<FileIdentitySpec> {
    Ok(FileIdentitySpec {
        path: text(contract, &format!("{pointer}/path"))?,
        sha256: text(contract, &format!("{pointer}/sha256"))?,
        file_size: number(contract, &format!("{pointer}/file_size"))?,
        windows_file_id: text(contract, &format!("{pointer}/windows_file_id"))?,
    })
}

/// Derive the gate spec from the tracked contract, calibration manifest, and
/// frozen h001 artifacts. A loaded gate identity must carry exactly this spec.
pub fn derive_local_9b_gate_spec(
    contract: &Value,
    manifest: &Value,
) -> Result<CapableModelGateSpec> {
    validate_local_9b_contract(contract)?;
    let model_label = text(contract, "/runtime/model/label")?;
    let gate_manifest = gate_calibration_manifest(manifest, &model_label)?;
    let structural_cases = structural_requests(&gate_manifest)?
        .iter()
        .map(|(case_id, _, request)| {
            Ok(CaseSpec {
                case_id: case_id.clone(),
                request_sha256: canonical_hash(request)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let task = task_request(&model_label)?;
    let evaluator = read_json(h001::H001_EVALUATOR_PATH)?;
    let tool_contract = read_json(h001::H001_TOOL_CONTRACT_PATH)?;
    Ok(CapableModelGateSpec {
        gate_id: LOCAL_9B_GATE_ID.to_string(),
        experiment_id: LOCAL_9B_EXPERIMENT_ID.to_string(),
        contract_path: LOCAL_9B_CONTRACT_PATH.to_string(),
        contract_sha256: canonical_hash(contract)?,
        runtime_version: text(contract, "/runtime/executable/version")?,
        runtime_executable: file_identity_spec(contract, "/runtime/executable")?,
        model_file: file_identity_spec(contract, "/runtime/model")?,
        model_source: ModelSourceSpec {
            repository: text(contract, "/runtime/model/repository")?,
            revision: text(contract, "/runtime/model/revision")?,
            filename: text(contract, "/runtime/model/filename")?,
            upstream_sha256: text(contract, "/runtime/model/upstream_sha256")?,
            license: text(contract, "/runtime/model/license")?,
            base_model: text(contract, "/runtime/model/base_model")?,
        },
        model_label,
        launch_arguments: string_list(contract, "/runtime/launch_arguments")?,
        forbidden_launch_arguments: string_list(contract, "/runtime/forbidden_launch_arguments")?,
        forbidden_environment_prefix: text(contract, "/runtime/forbidden_environment_prefix")?,
        reasoning: ReasoningSpec {
            mode: text(contract, "/runtime/reasoning")?,
            launch_flag: string_list(contract, "/runtime/reasoning_mechanism/launch_flag")?,
            forbidden_request_fields: string_list(
                contract,
                "/runtime/reasoning_mechanism/forbidden_request_fields",
            )?,
        },
        structural_cases,
        task_case: TaskCaseSpec {
            task_id: "h001".to_string(),
            arm: H001Arm::Baseline.as_str().to_string(),
            request_sha256: canonical_hash(&task)?,
            turn_ceiling: TASK_TURN_CEILING,
        },
        evaluators: EvaluatorSpec {
            structural_manifest_sha256: canonical_hash(manifest)?,
            task_evaluator_version: text(&evaluator, "/evaluator_version")?,
            task_evaluator_sha256: canonical_hash(&evaluator)?,
            task_tool_contract_version: text(&tool_contract, "/tool_contract_version")?,
        },
        stages: StageLimitsSpec {
            structural_requests: STRUCTURAL_REQUESTS,
            task_turn_ceiling: TASK_TURN_CEILING,
            task_stage_requires_all_structural_pass: true,
        },
        limits: gate_limits(),
        deadlines: contract_deadlines(contract)?,
        evidence_root: LOCAL_9B_GATE_EVIDENCE_ROOT.to_string(),
        token_count_endpoint: LOCAL_9B_TOKEN_COUNT_ENDPOINT.to_string(),
        inference_endpoint: ENDPOINT.to_string(),
    })
}

/// A spec is valid only when it equals the contract-derived spec.
pub fn validate_local_9b_gate_spec(
    spec: &CapableModelGateSpec,
    contract: &Value,
    manifest: &Value,
) -> Result<()> {
    validate_supervisor_deadline(&spec.limits, &spec.deadlines)?;
    if spec != &derive_local_9b_gate_spec(contract, manifest)? {
        return Err(invalid(
            "local-9B gate spec differs from the contract-derived spec",
        ));
    }
    Ok(())
}

/// Stage B runs only when all three structural probes passed.
pub fn task_stage_permitted(structural_states: &[&str]) -> bool {
    structural_states.len() == STRUCTURAL_REQUESTS as usize
        && structural_states.iter().all(|state| *state == "PASS")
}

/// The pre-registered stopping rule: pass only when all three structural
/// probes and the h001 BASELINE trajectory pass with no context bound.
pub fn classify_local_9b_gate(
    context_bound: bool,
    structural_states: &[&str],
    task_result: Option<&str>,
) -> &'static str {
    if !context_bound && task_stage_permitted(structural_states) && task_result == Some("PASS") {
        LOCAL_9B_FEASIBILITY_PASSED
    } else {
        LOCAL_9B_FEASIBILITY_FAILED
    }
}

/// What a gate classification permits next.
pub fn local_9b_gate_permission(classification: &str) -> &'static str {
    match classification {
        LOCAL_9B_FEASIBILITY_PASSED => "PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY",
        LOCAL_9B_FEASIBILITY_FAILED => "CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW",
        _ => "REPLACEMENT_IDENTITY_REVIEW_ONLY_IF_ZERO_INFERENCE",
    }
}

/// Split a Windows command line with the `CommandLineToArgvW` rules: the
/// program name ends at the closing quote or first whitespace; later
/// arguments honour quotes and backslash escaping before quotes.
pub fn split_windows_command_line(command_line: &str) -> Vec<String> {
    let chars = command_line.chars().collect::<Vec<_>>();
    let mut arguments = Vec::new();
    let mut index = 0;
    let mut program = String::new();
    if chars.first() == Some(&'"') {
        index = 1;
        while index < chars.len() && chars[index] != '"' {
            program.push(chars[index]);
            index += 1;
        }
        index += 1;
    } else {
        while index < chars.len() && !chars[index].is_whitespace() {
            program.push(chars[index]);
            index += 1;
        }
    }
    arguments.push(program);
    loop {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }
        let mut argument = String::new();
        let mut quoted = false;
        while index < chars.len() && (quoted || !chars[index].is_whitespace()) {
            let mut backslashes = 0;
            while index < chars.len() && chars[index] == '\\' {
                backslashes += 1;
                index += 1;
            }
            if index < chars.len() && chars[index] == '"' {
                argument.extend(std::iter::repeat_n('\\', backslashes / 2));
                if backslashes % 2 == 1 {
                    argument.push('"');
                } else {
                    quoted = !quoted;
                }
                index += 1;
            } else {
                argument.extend(std::iter::repeat_n('\\', backslashes));
                if index < chars.len() && (quoted || !chars[index].is_whitespace()) {
                    argument.push(chars[index]);
                    index += 1;
                }
            }
        }
        arguments.push(argument);
    }
    arguments
}

/// The server's argument vector must be exactly the registered executable
/// followed by exactly the registered launch arguments. Reasoning mode is
/// server-wide in b10217, so a missing or different `--reasoning off` fails.
pub fn validate_server_arguments(arguments: &[String], spec: &CapableModelGateSpec) -> Result<()> {
    let Some((program, rest)) = arguments.split_first() else {
        return Err(invalid("server command line is empty"));
    };
    if !program.eq_ignore_ascii_case(&spec.runtime_executable.path) {
        return Err(invalid(
            "server command line names an unregistered executable",
        ));
    }
    if rest != spec.launch_arguments.as_slice() {
        return Err(invalid(
            "server launch arguments differ from the registered launch arguments",
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn process_command_line(pid: u32) -> Result<String> {
    use std::ptr::null_mut;
    use windows_sys::Wdk::System::Threading::{
        NtQueryInformationProcess, ProcessCommandLineInformation,
    };
    use windows_sys::Win32::Foundation::{CloseHandle, UNICODE_STRING};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return Err(invalid(&format!(
            "cannot open server process {pid} for inspection"
        )));
    }
    let result = (|| {
        let mut needed = 0u32;
        unsafe {
            NtQueryInformationProcess(
                handle,
                ProcessCommandLineInformation,
                null_mut(),
                0,
                &mut needed,
            )
        };
        if needed == 0 {
            return Err(invalid("server command-line length query failed"));
        }
        // u64 storage keeps the UNICODE_STRING header aligned.
        let mut buffer = vec![0u64; (needed as usize).div_ceil(8)];
        let status = unsafe {
            NtQueryInformationProcess(
                handle,
                ProcessCommandLineInformation,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        };
        if status < 0 {
            return Err(invalid(&format!(
                "server command-line query failed with NTSTATUS {status:#x}"
            )));
        }
        let header = unsafe { &*(buffer.as_ptr() as *const UNICODE_STRING) };
        if header.Buffer.is_null() {
            return Ok(String::new());
        }
        let units =
            unsafe { std::slice::from_raw_parts(header.Buffer, (header.Length / 2) as usize) };
        Ok(String::from_utf16_lossy(units))
    })();
    unsafe { CloseHandle(handle) };
    result
}

#[cfg(not(windows))]
fn process_command_line(_pid: u32) -> Result<String> {
    Err(invalid("server command-line inspection requires Windows"))
}

/// Read and validate the verified server process's command line.
fn verify_server_command_line(pid: u32, spec: &CapableModelGateSpec) -> Result<Value> {
    let command_line = process_command_line(pid)?;
    let arguments = split_windows_command_line(&command_line);
    validate_server_arguments(&arguments, spec)?;
    Ok(json!({
        "server_pid": pid,
        "mechanism": "NtQueryInformationProcess(ProcessCommandLineInformation)",
        "arguments": arguments,
        "matches_registered_launch_arguments": true,
        "reasoning_flag": spec.reasoning.launch_flag
    }))
}

/// Inspect the frozen llama.cpp executable and GGUF model on disk.
pub fn validate_runtime_objects(spec: &CapableModelGateSpec) -> Result<Value> {
    let executable = executable_identity::inspect(Path::new(&spec.runtime_executable.path))
        .map_err(|error| invalid(&error))?;
    compare_file_identity(
        "llama.cpp executable",
        &spec.runtime_executable,
        &executable,
    )?;
    let model = executable_identity::inspect(Path::new(&spec.model_file.path))
        .map_err(|error| invalid(&error))?;
    compare_file_identity("GGUF model", &spec.model_file, &model)?;
    Ok(json!({ "executable": executable, "model": model }))
}

/// The offline plan: no readiness, network, token-count, or inference call.
pub fn dry_run_plan(spec: &CapableModelGateSpec) -> Value {
    json!({
        "state": "LOCAL_9B_FEASIBILITY_GATE_DRY_RUN",
        "gate_id": spec.gate_id,
        "model_label": spec.model_label,
        "reasoning": spec.reasoning,
        "structural_cases": spec.structural_cases,
        "task_case": spec.task_case,
        "stages": spec.stages,
        "limits": spec.limits,
        "deadlines": spec.deadlines,
        "readiness_contacts": 0,
        "network_calls": 0,
        "token_count_contacts": 0,
        "inference_requests": 0,
        "retry_requests": 0,
        "fallback_requests": 0,
        "adaptive_replicates": 0,
        "warmup_requests": 0
    })
}

fn tracked_contract() -> Result<Value> {
    let contract = read_json(LOCAL_9B_CONTRACT_PATH)?;
    let sidecar = read_json(LOCAL_9B_CONTRACT_FINGERPRINT_PATH)?;
    if sidecar.get("canonical_sha256").and_then(Value::as_str)
        != Some(canonical_hash(&contract)?.as_str())
    {
        return Err(invalid(
            "capable-model contract fingerprint sidecar mismatch",
        ));
    }
    Ok(contract)
}

/// Offline dry run: contract, derived spec, requests, and plan. With
/// `inspect_runtime_objects`, the frozen executable and GGUF are hashed.
pub fn dry_run_local_9b_gate(inspect_runtime_objects: bool) -> Result<Value> {
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    let spec = derive_local_9b_gate_spec(&contract, &manifest)?;
    let runtime_objects = if inspect_runtime_objects {
        validate_runtime_objects(&spec)?
    } else {
        Value::Null
    };
    let mut plan = dry_run_plan(&spec);
    plan["derived_spec"] = serde_json::to_value(&spec)?;
    plan["runtime_objects"] = runtime_objects;
    plan["identity_present"] = json!(workspace_path(LOCAL_9B_GATE_IDENTITY_PATH).is_file());
    Ok(plan)
}

/// Freeze the final build outputs into the non-overwriting stage.
pub fn freeze_local_9b_gate() -> Result<Value> {
    let stage = workspace_path(LOCAL_9B_GATE_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid("local-9B frozen staging directory already exists"));
    }
    let mut frozen = serde_json::Map::new();
    for (label, binary) in [("supervisor", SUPERVISOR_BINARY), ("child", CHILD_BINARY)] {
        let source = workspace_path(&format!("target/debug/{binary}"));
        let source_identity =
            executable_identity::inspect(&source).map_err(|error| invalid(&error))?;
        let copy = executable_identity::freeze_copy(&source, &stage.join(binary))
            .map_err(|error| invalid(&error))?;
        if copy.sha256 != source_identity.sha256 {
            return Err(invalid(
                "local-9B frozen executable differs from its build output",
            ));
        }
        frozen.insert(format!("{label}_source_binary"), json!(source_identity));
        frozen.insert(format!("{label}_binary"), json!(copy));
    }
    frozen.insert("state".to_string(), json!("FROZEN"));
    frozen.insert("overwrite".to_string(), json!(false));
    frozen.insert("inference_requests".to_string(), json!(0));
    Ok(Value::Object(frozen))
}

/// Load the gate identity, its sidecar, and its spec.
pub fn load_gate_identity() -> Result<(Value, CapableModelGateSpec)> {
    let identity = read_json(LOCAL_9B_GATE_IDENTITY_PATH)?;
    let sidecar = read_json(LOCAL_9B_GATE_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str) != Some(LOCAL_9B_GATE_IDENTITY_PATH)
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(&identity)?.as_str())
    {
        return Err(invalid(
            "local-9B gate identity fingerprint sidecar mismatch",
        ));
    }
    validate_gate_identity_document(&identity)?;
    let spec: CapableModelGateSpec = serde_json::from_value(field(&identity, "/spec")?.clone())?;
    Ok((identity, spec))
}

/// Document-level checks that do not read other files.
pub fn validate_gate_identity_document(identity: &Value) -> Result<()> {
    if !text(identity, "/identity_version")?
        .starts_with(crate::phase1c_live_supervisor::LOCAL_9B_FEASIBILITY_GATE_IDENTITY_PREFIX)
    {
        return Err(invalid("local-9B gate identity version is invalid"));
    }
    expect(
        identity,
        "/status",
        json!("PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED"),
    )?;
    if !matches!(number(identity, "/gate_identity_number")?, 1 | 2) {
        return Err(invalid("local-9B gate identity number must be 1 or 2"));
    }
    let sources = field(identity, "/implementation_sources")?
        .as_array()
        .ok_or_else(|| invalid("local-9B implementation sources are missing"))?;
    for required in LOCAL_9B_BOUND_SOURCES {
        if !sources
            .iter()
            .any(|source| source.get("path").and_then(Value::as_str) == Some(required))
        {
            return Err(invalid(&format!(
                "local-9B identity does not bind source {required}"
            )));
        }
    }
    let binding =
        executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(identity)
            .map_err(|error| invalid(&error))?
            .ok_or_else(|| invalid("local-9B identity is missing frozen executable identity"))?;
    for object in [&binding.supervisor, &binding.child] {
        let raw = object.raw_path.to_ascii_lowercase().replace('/', "\\");
        let stage = LOCAL_9B_GATE_FROZEN_STAGE_ROOT.replace('/', "\\");
        if raw.contains("target\\debug\\") || !raw.contains(&stage) || object.file_id.is_none() {
            return Err(invalid(
                "local-9B frozen executable is not a complete staged object",
            ));
        }
    }
    for key in [
        "model_server_startups",
        "port_8080_contacts",
        "readiness_contacts",
        "token_count_contacts",
        "inference_requests",
        "gate_executions",
    ] {
        expect(identity, &format!("/network_policy/{key}"), json!(0))?;
    }
    Ok(())
}

/// Offline preflight before the operator starts the server.
pub fn preflight_local_9b_gate() -> Result<Value> {
    let (identity, spec) = load_gate_identity()?;
    validate_source_binding(&identity)?;
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    validate_local_9b_gate_spec(&spec, &contract, &manifest)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(LOCAL_9B_GATE_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    let frozen = validate_frozen_executables(&identity)?;
    let runtime_objects = validate_runtime_objects(&spec)?;
    validate_launch_environment(std::env::vars(), &spec.forbidden_environment_prefix)?;
    let certification = validate_accepted_workflow_certification_v2()?;
    if workspace_path(&spec.evidence_root).exists() {
        return Err(invalid("local-9B gate evidence root already exists"));
    }
    let os_inspection = windows_native_prestart_value(0, true);
    if os_inspection["state"] != "READY" || os_inspection["outcome"] != "EXCLUSIVE_PRESTART" {
        return Err(invalid(
            "local-9B preflight did not establish exclusive pre-start state",
        ));
    }
    Ok(json!({
        "state": "LOCAL_9B_FEASIBILITY_GATE_PREPARED",
        "execution_state": "LOCAL_9B_FEASIBILITY_GATE_NOT_EXECUTED",
        "identity_sha256": canonical_hash(&identity)?,
        "registered_launch_identity": registered.generated_launch_identity(),
        "frozen_executable_binding": frozen,
        "runtime_objects": runtime_objects,
        "workflow_identity_certification_v2": certification,
        "windows_native_exclusivity": os_inspection,
        "plan": dry_run_plan(&spec),
        "network_calls": 0,
        "inference_requests": 0
    }))
}

struct GatePrerequisites {
    identity_sha256: String,
    spec: CapableModelGateSpec,
    gate_manifest: Value,
    structural: Vec<(String, Value, Value)>,
    task: Value,
    report: Value,
}

/// Every deterministic check the live gate performs before post-start
/// ownership inspection, readiness, token counting, or inference. The live
/// entry point and the offline traversal both call this function.
fn gate_prerequisites() -> Result<GatePrerequisites> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    let (identity, spec) = load_gate_identity()?;
    let identity_sha256 = canonical_hash(&identity)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(LOCAL_9B_GATE_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != identity_sha256
        || metadata.attempt != registered.attempt
        || metadata.candidate_budget != registered.candidate_budget
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "local-9B supervisor launch metadata does not match the registered gate identity",
        ));
    }
    validate_source_binding(&identity)?;
    let frozen = validate_frozen_executables(&identity)?;
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    validate_local_9b_gate_spec(&spec, &contract, &manifest)?;
    let runtime_objects = validate_runtime_objects(&spec)?;
    validate_launch_environment(std::env::vars(), &spec.forbidden_environment_prefix)?;
    let gate_manifest = gate_calibration_manifest(&manifest, &spec.model_label)?;
    let structural = structural_requests(&gate_manifest)?;
    for ((case_id, _, request), registered_case) in structural.iter().zip(&spec.structural_cases) {
        if case_id != &registered_case.case_id
            || canonical_hash(request)? != registered_case.request_sha256
        {
            return Err(invalid(
                "local-9B structural request differs from the registration",
            ));
        }
    }
    let task = task_request(&spec.model_label)?;
    if canonical_hash(&task)? != spec.task_case.request_sha256 {
        return Err(invalid(
            "local-9B task request differs from the registration",
        ));
    }
    if workspace_path(&spec.evidence_root).exists() {
        return Err(invalid("local-9B gate evidence root already exists"));
    }
    let report = json!({
        "gate_id": spec.gate_id,
        "identity_sha256": identity_sha256,
        "launch_identity": metadata.launch_identity,
        "supervisor_pid": metadata.supervisor_pid,
        "child_pid": std::process::id(),
        "GATE_IDENTITY_VALID": true,
        "SOURCE_BINDING_VALID": true,
        "FROZEN_SUPERVISOR_VALID": true,
        "FROZEN_CHILD_VALID": true,
        "WORKFLOW_CERTIFICATION_VALID": true,
        "CONTRACT_VALID": true,
        "SPEC_MATCHES_CONTRACT": true,
        "RUNTIME_EXECUTABLE_VALID": true,
        "GGUF_MODEL_VALID": true,
        "LAUNCH_ENVIRONMENT_VALID": true,
        "REQUESTS_MATCH_REGISTRATION": true,
        "REQUESTS_REASONING_NEUTRAL": true,
        "GATE_EVIDENCE_ROOT_ABSENT": true,
        "RUNTIME_DEPENDENCIES_COMPLETE": true,
        "frozen_executable_binding": frozen,
        "runtime_objects": runtime_objects,
        "expected_workflow": expected_workflow,
        "workflow_identity_certification_v2": certification,
        "network_calls": 0,
        "token_count_contacts": 0,
        "inference_requests": 0
    });
    Ok(GatePrerequisites {
        identity_sha256,
        spec,
        gate_manifest,
        structural,
        task,
        report,
    })
}

/// Offline traversal under the frozen supervisor; stops before post-start
/// ownership inspection, server command-line verification, readiness, token
/// counting, and inference.
pub fn validate_local_9b_live_prerequisites() -> Result<Value> {
    let mut report = gate_prerequisites()?.report;
    report["state"] = json!("READY_FOR_MODEL_READINESS_BOUNDARY");
    report["READY_FOR_MODEL_READINESS_BOUNDARY"] = json!(true);
    report["stopped_before"] = json!([
        "post-start runtime ownership inspection",
        "server command-line verification",
        "tcp listener readiness",
        "token-count request",
        "inference"
    ]);
    report["model_server_startups"] = json!(0);
    report["readiness_contacts"] = json!(0);
    Ok(report)
}

#[derive(Default)]
struct Accounting {
    structural_token_counts: u32,
    task_token_counts: u32,
    structural_inference: u32,
    task_inference: u32,
}

impl Accounting {
    fn inference(&self) -> u32 {
        self.structural_inference + self.task_inference
    }

    fn value(&self) -> Value {
        json!({
            "readiness_contacts": 1,
            "token_count_contacts": {
                "structural": self.structural_token_counts,
                "task_level": self.task_token_counts,
                "total": self.structural_token_counts + self.task_token_counts
            },
            "inference_requests": {
                "structural": self.structural_inference,
                "task_level": self.task_inference,
                "total": self.inference()
            },
            "retry_requests": 0,
            "fallback_requests": 0,
            "adaptive_replicates": 0,
            "warmup_requests": 0
        })
    }
}

fn gate_result(
    prerequisites: &GatePrerequisites,
    classification: &str,
    detail: Value,
    accounting: &Accounting,
) -> Result<Value> {
    Ok(json!({
        "schema_id": "prefixity.phase1c.local-9b-feasibility-gate-result",
        "schema_version": 1,
        "gate_id": prerequisites.spec.gate_id,
        "identity_sha256": prerequisites.identity_sha256,
        "classification": classification,
        "permits": local_9b_gate_permission(classification),
        "detail": detail,
        "accounting": accounting.value(),
        "recorded_at_unix_ms": now_unix_ms()?
    }))
}

fn seal_result(root: &Path, result: &Value) -> Result<()> {
    write_json(&root.join("gate-result.json"), result)?;
    write_bytes(
        &root.join("gate-result.sha256"),
        format!("{}  gate-result.json\n", canonical_hash(result)?).as_bytes(),
    )
}

/// A token-count transport or HTTP failure: a pre-inference failure while no
/// inference has been dispatched, otherwise a consumed failing gate.
fn token_count_failure(
    prerequisites: &GatePrerequisites,
    root: &Path,
    accounting: &Accounting,
    case_id: &str,
    error: &ReasoningBudgetCalibrationError,
    counts: &[Value],
) -> Result<Value> {
    let (classification, failure_class) = if accounting.inference() == 0 {
        (
            LOCAL_9B_GATE_PRE_INFERENCE_FAILURE,
            "TOKEN_COUNT_FAILED_BEFORE_INFERENCE",
        )
    } else {
        (LOCAL_9B_FEASIBILITY_FAILED, "INFRASTRUCTURE_AFTER_DISPATCH")
    };
    write_json(&root.join("token-counts.json"), &json!(counts))?;
    let result = gate_result(
        prerequisites,
        classification,
        json!({
            "failure_class": failure_class,
            "case_id": case_id,
            "error": error.to_string(),
            "token_counts": counts
        }),
        accounting,
    )?;
    seal_result(root, &result)?;
    Ok(result)
}

/// Live entry point. It is not called by preparation.
pub fn execute_local_9b_feasibility_gate() -> Result<Value> {
    let prerequisites = gate_prerequisites()?;
    let ownership = poststart_ownership()?;
    let server_pid = ownership
        .get("expected_server_pid")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("post-start ownership omitted the server PID"))?
        as u32;
    let spec = &prerequisites.spec;
    let server_command_line = verify_server_command_line(server_pid, spec)?;
    let root = workspace_path(&spec.evidence_root);
    if root.exists() {
        return Err(invalid("local-9B gate evidence root already exists"));
    }
    fs::create_dir_all(&root)?;
    write_json(
        &root.join("preflight.json"),
        &json!({
            "live_prerequisites": prerequisites.report,
            "runtime_ownership": ownership,
            "server_command_line": server_command_line
        }),
    )?;

    let mut accounting = Accounting::default();
    let started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], PORT)),
        Duration::from_millis(spec.deadlines.readiness_timeout_ms),
    );
    let readiness_elapsed_ms = started.elapsed().as_millis() as u64;
    write_json(
        &root.join("readiness.json"),
        &json!({
            "check": "tcp_listener_connect",
            "listener_check_attempts": 1,
            "timeout_ms": spec.deadlines.readiness_timeout_ms,
            "elapsed_ms": readiness_elapsed_ms,
            "passed": readiness.is_ok(),
            "error": readiness.as_ref().err().map(ToString::to_string),
            "inference_requests": 0
        }),
    )?;
    if readiness.is_err() {
        let result = gate_result(
            &prerequisites,
            LOCAL_9B_GATE_PRE_INFERENCE_FAILURE,
            json!({ "failure_class": "READINESS_FAILED" }),
            &accounting,
        )?;
        seal_result(&root, &result)?;
        return Ok(result);
    }

    // The client default is the inference bound; token counting overrides it
    // per request with its own bound.
    let client = Client::builder()
        .connect_timeout(Duration::from_millis(spec.deadlines.connect_timeout_ms))
        .timeout(Duration::from_millis(
            spec.deadlines.inference_request_timeout_ms,
        ))
        .redirect(Policy::none())
        .build()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;
    let token_timeout = Duration::from_millis(spec.deadlines.token_count_request_timeout_ms);
    let mut counts = Vec::new();

    // Stage A: structural probes, each counted and guarded before dispatch.
    let mut structural_results = Vec::new();
    let mut context_bound = false;
    for (case_id, case, request) in &prerequisites.structural {
        let (input_tokens, record) =
            match count_input_tokens(&client, &spec.token_count_endpoint, token_timeout, request) {
                Ok(counted) => counted,
                Err(error) => {
                    accounting.structural_token_counts += 1;
                    return token_count_failure(
                        &prerequisites,
                        &root,
                        &accounting,
                        case_id,
                        &error,
                        &counts,
                    );
                }
            };
        accounting.structural_token_counts += 1;
        let fits = context_guard(input_tokens, &spec.limits) == ContextDecision::Fits;
        counts.push(json!({
            "stage": "structural",
            "case_id": case_id,
            "input_tokens": input_tokens,
            "fits": fits,
            "record": record
        }));
        if !fits {
            context_bound = true;
            break;
        }
        let case_dir = root.join(case_id);
        fs::create_dir_all(&case_dir)?;
        accounting.structural_inference += 1;
        let outcome = execute_case(&prerequisites.gate_manifest, case, None, &case_dir, &client)?;
        let passed = outcome.get("state").and_then(Value::as_str) == Some("PASS");
        structural_results.push(outcome);
        if !passed {
            break;
        }
    }
    let structural_states = structural_results
        .iter()
        .map(|result| {
            result
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("INCONCLUSIVE")
        })
        .collect::<Vec<_>>();

    // Stage B: one h001 BASELINE trajectory, only after three structural passes.
    let mut task_result = None;
    let mut task_detail = Value::Null;
    if !context_bound && task_stage_permitted(&structural_states) {
        let request = &prerequisites.task;
        let (input_tokens, record) =
            match count_input_tokens(&client, &spec.token_count_endpoint, token_timeout, request) {
                Ok(counted) => counted,
                Err(error) => {
                    accounting.task_token_counts += 1;
                    return token_count_failure(
                        &prerequisites,
                        &root,
                        &accounting,
                        "h001",
                        &error,
                        &counts,
                    );
                }
            };
        accounting.task_token_counts += 1;
        let fits = context_guard(input_tokens, &spec.limits) == ContextDecision::Fits;
        counts.push(json!({
            "stage": "task_level",
            "case_id": "h001",
            "input_tokens": input_tokens,
            "fits": fits,
            "record": record
        }));
        if fits {
            let task_dir = root.join("h001-baseline");
            fs::create_dir_all(&task_dir)?;
            accounting.task_inference += 1;
            let arm_result = h001::execute_h001_turn(H001TurnInput {
                arm: H001Arm::Baseline,
                request,
                evidence_dir: &task_dir,
                experiment_id: &spec.experiment_id,
                readiness_elapsed_ms,
                timeouts: || {
                    Ok((
                        Duration::from_millis(spec.deadlines.connect_timeout_ms),
                        Duration::from_millis(spec.deadlines.inference_request_timeout_ms),
                    ))
                },
            })
            .map_err(h001_error)?;
            let scored =
                h001::score_h001_arm_at(H001Arm::Baseline, task_dir).map_err(h001_error)?;
            task_result = scored
                .get("result")
                .and_then(Value::as_str)
                .map(str::to_string);
            task_detail = json!({
                "arm_state": arm_result.get("state"),
                "evaluator_result": task_result,
                "http_status": arm_result.pointer("/response/http_status"),
                "validation": arm_result.get("validation")
            });
        } else {
            context_bound = true;
        }
    }
    write_json(&root.join("token-counts.json"), &json!(counts))?;

    let classification =
        classify_local_9b_gate(context_bound, &structural_states, task_result.as_deref());
    let result = gate_result(
        &prerequisites,
        classification,
        json!({
            "context_result": if context_bound { json!(INCONCLUSIVE_CONTEXT_BOUND) } else { Value::Null },
            "structural_states": structural_states,
            "task_stage_executed": accounting.task_inference > 0,
            "task_result": task_result,
            "task_detail": task_detail,
            "token_counts": counts
        }),
        &accounting,
    )?;
    seal_result(&root, &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1c_executable_identity::ExecutableIdentity;

    fn contract() -> Value {
        read_json(LOCAL_9B_CONTRACT_PATH).unwrap()
    }

    fn manifest() -> Value {
        tracked_manifest().unwrap()
    }

    fn spec() -> CapableModelGateSpec {
        derive_local_9b_gate_spec(&contract(), &manifest()).unwrap()
    }

    fn identity(sha256: &str, file_size: u64, file_id: Option<&str>) -> ExecutableIdentity {
        ExecutableIdentity {
            raw_path: "object".to_string(),
            final_path: "object".to_string(),
            file_size,
            sha256: sha256.to_string(),
            file_id: file_id.map(str::to_string),
        }
    }

    #[test]
    fn tracked_local_9b_contract_is_exact_and_sealed() {
        validate_local_9b_contract(&contract()).unwrap();
        tracked_contract().unwrap();
        let spec = spec();
        assert_eq!(spec.limits.max_tokens, 1024);
        assert_eq!(spec.limits.context_tokens, 8192);
        assert_eq!(spec.limits.inference_requests, 6);
        assert_eq!(spec.limits.token_count_contacts, 6);
        assert_eq!(spec.limits.readiness_contacts, 1);
        assert_eq!(spec.model_file.sha256, LOCAL_9B_GGUF_SHA256);
        assert_eq!(spec.model_source.upstream_sha256, LOCAL_9B_GGUF_SHA256);
        assert_eq!(
            spec.runtime_executable.windows_file_id,
            V3_LLAMA_EXECUTABLE_FILE_ID
        );
        assert_eq!(spec.reasoning.mode, "off");
        assert_eq!(spec.reasoning.launch_flag, ["--reasoning", "off"]);
        assert_eq!(spec.stages.structural_requests, 3);
        assert_eq!(spec.stages.task_turn_ceiling, 3);
        assert!(spec.stages.task_stage_requires_all_structural_pass);
        assert_eq!(spec.task_case.task_id, "h001");
        assert_eq!(spec.task_case.arm, "BASELINE");
    }

    #[test]
    fn derived_deadline_covers_every_contact_and_rejects_independent_values() {
        let spec = spec();
        let d = &spec.deadlines;
        assert_eq!(
            d.supervisor_deadline_ms,
            d.readiness_timeout_ms
                + 6 * d.token_count_request_timeout_ms
                + 6 * d.inference_request_timeout_ms
                + d.non_request_margin_ms
        );
        let mut independent = contract();
        independent["timeout_policy"]["supervisor_deadline_ms"] =
            json!(d.supervisor_deadline_ms + 1);
        assert!(validate_local_9b_contract(&independent).is_err());
        let mut v3_inference = contract();
        v3_inference["timeout_policy"]["inference_request_timeout_ms"] = json!(2_400_000);
        assert!(validate_local_9b_contract(&v3_inference).is_err());
        let mut changed = spec.clone();
        changed.deadlines.non_request_margin_ms += 1;
        assert!(validate_local_9b_gate_spec(&changed, &contract(), &manifest()).is_err());
    }

    #[test]
    fn reasoning_is_off_and_cannot_be_reenabled() {
        let spec = spec();
        let launch = &spec.launch_arguments;
        let flag = launch
            .iter()
            .position(|argument| argument == "--reasoning")
            .unwrap();
        assert_eq!(launch[flag + 1], "off");
        assert!(!launch
            .iter()
            .any(|argument| argument.contains("reasoning-budget")));
        assert!(launch.contains(&"-m".to_string()));
        assert!(launch.contains(&"--offline".to_string()));

        let mut on = contract();
        on["runtime"]["reasoning"] = json!("on");
        assert!(validate_local_9b_contract(&on).is_err());
        let mut on_flag = contract();
        on_flag["runtime"]["launch_arguments"][9] = json!("on");
        assert!(validate_local_9b_contract(&on_flag).is_err());
        let mut kwargs = contract();
        let arguments = kwargs["runtime"]["launch_arguments"]
            .as_array_mut()
            .unwrap();
        arguments.push(json!("--chat-template-kwargs"));
        arguments.push(json!("{\"enable_thinking\":true}"));
        assert!(validate_local_9b_contract(&kwargs).is_err());
        let mut hf = contract();
        hf["runtime"]["launch_arguments"][1] = json!("-hf");
        assert!(validate_local_9b_contract(&hf).is_err());
    }

    #[test]
    fn request_level_reasoning_overrides_are_rejected() {
        let (_, _, request) = structural_requests(
            &gate_calibration_manifest(&manifest(), &spec().model_label).unwrap(),
        )
        .unwrap()
        .remove(0);
        validate_reasoning_neutral_request(&request).unwrap();
        for key in FORBIDDEN_REQUEST_FIELDS {
            let mut overridden = request.clone();
            overridden[key] = json!({ "enable_thinking": true });
            assert!(
                validate_reasoning_neutral_request(&overridden).is_err(),
                "{key}"
            );
        }
    }

    #[test]
    fn gate_requests_differ_from_frozen_requests_only_in_model_and_ceiling() {
        let spec = spec();
        let manifest = manifest();
        let gate_manifest = gate_calibration_manifest(&manifest, &spec.model_label).unwrap();
        let requests = structural_requests(&gate_manifest).unwrap();
        assert_eq!(requests.len(), 3);
        for (case_id, case, request) in &requests {
            let mut original = build_request(&manifest, case).unwrap();
            assert_eq!(original["model"], "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0");
            original["model"] = json!(spec.model_label);
            original["max_tokens"] = json!(1024);
            assert_eq!(&original, request, "{case_id}");
        }
        let task = task_request(&spec.model_label).unwrap();
        let mut frozen =
            h001::dry_run_h001(H001Arm::Baseline).unwrap()["model_visible_request"].clone();
        assert_eq!(frozen["model"], "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0");
        frozen["model"] = json!(spec.model_label);
        frozen["max_tokens"] = json!(1024);
        assert_eq!(frozen, task);
        assert_eq!(
            canonical_hash(&task).unwrap(),
            spec.task_case.request_sha256
        );
    }

    #[test]
    fn wrong_executable_or_model_identity_is_rejected() {
        let spec = spec();
        let exe = &spec.runtime_executable;
        let model = &spec.model_file;
        compare_file_identity(
            "model",
            model,
            &identity(&model.sha256, model.file_size, Some(&model.windows_file_id)),
        )
        .unwrap();
        assert!(compare_file_identity(
            "model",
            model,
            &identity(&exe.sha256, model.file_size, Some(&model.windows_file_id))
        )
        .is_err());
        assert!(compare_file_identity(
            "model",
            model,
            &identity(
                &model.sha256,
                model.file_size,
                Some("volume=c4c93b54;index=0000000000000001")
            )
        )
        .is_err());
        let mut changed = contract();
        changed["runtime"]["model"]["sha256"] = json!("0".repeat(64));
        assert!(validate_local_9b_contract(&changed).is_err());
        let mut elsewhere = contract();
        elsewhere["runtime"]["model"]["path"] =
            json!("C:\\models\\Qwen3.5-9B\\Qwen3.5-9B-Q4_K_M.gguf");
        assert!(validate_local_9b_contract(&elsewhere).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn registered_gguf_is_bound_to_its_containing_volume() {
        use crate::phase1c_executable_identity::{containing_volume_serial, file_id_volume};
        let spec = spec();
        let model = Path::new(&spec.model_file.path);
        if model.is_file() {
            assert_eq!(
                file_id_volume(&spec.model_file.windows_file_id),
                Some(containing_volume_serial(model).unwrap())
            );
            assert!(spec.model_file.path.starts_with("D:\\"));
        } else {
            eprintln!("frozen 9B GGUF absent; containing-volume check skipped");
        }
    }

    #[test]
    fn server_command_line_must_match_registered_arguments() {
        let spec = spec();
        let exact = format!(
            "\"{}\" {}",
            spec.runtime_executable.path,
            spec.launch_arguments.join(" ")
        );
        let arguments = split_windows_command_line(&exact);
        validate_server_arguments(&arguments, &spec).unwrap();
        let unquoted = format!(
            "{} {}",
            spec.runtime_executable.path,
            spec.launch_arguments.join("  ")
        );
        validate_server_arguments(&split_windows_command_line(&unquoted), &spec).unwrap();

        for (label, command_line) in [
            (
                "reasoning on",
                exact.replace("--reasoning off", "--reasoning on"),
            ),
            ("reasoning absent", exact.replace(" --reasoning off", "")),
            ("budget added", format!("{exact} --reasoning-budget 256")),
            (
                "kwargs added",
                format!("{exact} --chat-template-kwargs {{\"enable_thinking\":true}}"),
            ),
            (
                "other model",
                exact.replace("Qwen3.5-9B-Q4_K_M", "Qwen3.5-4B-Q4_K_M"),
            ),
            (
                "other executable",
                exact.replace("llama.exe", "llama-server.exe"),
            ),
        ] {
            assert!(
                validate_server_arguments(&split_windows_command_line(&command_line), &spec)
                    .is_err(),
                "{label}"
            );
        }
        assert_eq!(
            split_windows_command_line(r#""C:\a b\x.exe" "q r" s\"t u\\\"v"#),
            [r"C:\a b\x.exe", "q r", "s\"t", "u\\\"v"]
        );
    }

    #[test]
    fn task_stage_runs_only_after_three_structural_passes() {
        assert!(task_stage_permitted(&["PASS", "PASS", "PASS"]));
        for states in [
            vec!["PASS", "PASS"],
            vec!["FAIL"],
            vec!["PASS", "FAIL"],
            vec!["PASS", "PASS", "FAIL"],
            vec!["PASS", "PASS", "INCONCLUSIVE"],
        ] {
            assert!(!task_stage_permitted(&states), "{states:?}");
        }
    }

    #[test]
    fn only_three_structural_passes_and_a_task_pass_pass_the_gate() {
        let passed = classify_local_9b_gate(false, &["PASS", "PASS", "PASS"], Some("PASS"));
        assert_eq!(passed, LOCAL_9B_FEASIBILITY_PASSED);
        assert_eq!(
            local_9b_gate_permission(passed),
            "PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY"
        );
        for (context_bound, structural, task) in [
            (true, vec!["PASS", "PASS", "PASS"], Some("PASS")),
            (false, vec!["PASS", "PASS", "PASS"], Some("FAIL")),
            (false, vec!["PASS", "PASS", "PASS"], Some("INCONCLUSIVE")),
            (false, vec!["PASS", "PASS", "PASS"], None),
            (false, vec!["PASS", "FAIL"], None),
            (false, vec!["PASS", "PASS", "FAIL"], Some("PASS")),
        ] {
            let classification = classify_local_9b_gate(context_bound, &structural, task);
            assert_eq!(classification, LOCAL_9B_FEASIBILITY_FAILED);
            assert_eq!(
                local_9b_gate_permission(classification),
                "CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW"
            );
        }
        assert_eq!(
            local_9b_gate_permission(LOCAL_9B_GATE_PRE_INFERENCE_FAILURE),
            "REPLACEMENT_IDENTITY_REVIEW_ONLY_IF_ZERO_INFERENCE"
        );
    }

    #[test]
    fn dry_run_makes_no_contacts() {
        let plan = dry_run_plan(&spec());
        for key in [
            "readiness_contacts",
            "network_calls",
            "token_count_contacts",
            "inference_requests",
            "retry_requests",
            "fallback_requests",
            "adaptive_replicates",
            "warmup_requests",
        ] {
            assert_eq!(plan[key], 0, "{key}");
        }
    }

    #[test]
    fn gate_identity_must_bind_every_source() {
        let sources = LOCAL_9B_BOUND_SOURCES
            .iter()
            .map(|path| {
                json!({
                    "path": path,
                    "sha256": crate::phase1c_reasoning_budget_calibration::source_sha256(path).unwrap()
                })
            })
            .collect::<Vec<_>>();
        let staged = |name: &str| {
            json!({
                "raw_path": format!("D:/frozen/{LOCAL_9B_GATE_FROZEN_STAGE_ROOT}/{name}"),
                "final_path": format!("D:/frozen/{LOCAL_9B_GATE_FROZEN_STAGE_ROOT}/{name}"),
                "file_size": 1,
                "sha256": "a".repeat(64),
                "file_id": "volume=ba2f80f4;index=0000000000000001"
            })
        };
        let identity = json!({
            "identity_version": "phase1c-local-9b-feasibility-gate-v1",
            "status": "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
            "gate_identity_number": 1,
            "implementation_sources": sources,
            "implementation_fingerprints": {
                "supervisor_binary": staged(SUPERVISOR_BINARY),
                "child_binary": staged(CHILD_BINARY)
            },
            "network_policy": {
                "model_server_startups": 0,
                "port_8080_contacts": 0,
                "readiness_contacts": 0,
                "token_count_contacts": 0,
                "inference_requests": 0,
                "gate_executions": 0
            }
        });
        validate_gate_identity_document(&identity).unwrap();
        validate_source_binding(&identity).unwrap();
        for required in LOCAL_9B_BOUND_SOURCES {
            let mut unbound = identity.clone();
            unbound["implementation_sources"]
                .as_array_mut()
                .unwrap()
                .retain(|source| source["path"] != required);
            assert!(
                validate_gate_identity_document(&unbound).is_err(),
                "{required}"
            );
        }
        let mut v3_stage = identity.clone();
        v3_stage["implementation_fingerprints"]["child_binary"]["raw_path"] =
            json!("D:/frozen/target/phase1c-v3-feasibility-frozen/child.exe");
        assert!(validate_gate_identity_document(&v3_stage).is_err());
    }

    #[test]
    fn consumed_v3_gate_identity_fails_closed_after_source_changes() {
        // The V3 gate executed once and is consumed. Its identity binds the
        // sources this gate generalized, so it must no longer validate.
        let (identity, _) = crate::phase1c_v3_feasibility::load_gate_identity().unwrap();
        let error = validate_source_binding(&identity).unwrap_err().to_string();
        assert!(error.contains("does not match current source"), "{error}");
    }

    #[test]
    fn live_prerequisites_require_supervisor_handoff() {
        let error = validate_local_9b_live_prerequisites()
            .unwrap_err()
            .to_string();
        assert!(error.contains("expected workflow launch metadata was not handed off"));
    }
}
