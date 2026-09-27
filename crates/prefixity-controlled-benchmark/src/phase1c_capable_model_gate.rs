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
/// An infrastructure or integrity failure after at least one inference
/// request was dispatched. It consumes the identity but establishes neither a
/// pass nor model inadequacy; see [`LOCAL_9B_INCONCLUSIVE_TERMINAL_FLAGS`].
pub const LOCAL_9B_FEASIBILITY_INCONCLUSIVE: &str = "LOCAL_9B_FEASIBILITY_INCONCLUSIVE";
pub const LOCAL_9B_INCONCLUSIVE_TERMINAL_FLAGS: [&str; 4] = [
    "LOCAL_9B_GATE_IDENTITY_CONSUMED",
    "INFRASTRUCTURE_AFTER_DISPATCH",
    "NO_REPLACEMENT_AUTHORIZED",
    "DESIGN_REVIEW_REQUIRED",
];

const CHILD_BINARY: &str = "prefixity-phase1c-capable-model-gate.exe";
const SUPERVISOR_BINARY: &str = "prefixity-phase1c-live-supervisor.exe";
/// Stage A: the three frozen structural probes.
const STRUCTURAL_INFERENCE_LIMIT: u32 = 3;
/// Stage B: the frozen h001 no-tool contract terminates on its first
/// response, so the task stage can legitimately issue exactly one request.
const TASK_INFERENCE_LIMIT: u32 = 1;

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
    pub request_chat_template_kwargs: String,
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
    pub inference_limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageLimitsSpec {
    pub structural_inference_limit: u32,
    pub task_inference_limit: u32,
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
    pub multimodal_projector: String,
    pub request_content: String,
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
        token_count_contacts: STRUCTURAL_INFERENCE_LIMIT + TASK_INFERENCE_LIMIT,
        inference_requests: STRUCTURAL_INFERENCE_LIMIT + TASK_INFERENCE_LIMIT,
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
        "/gate/structural_inference_limit",
        json!(STRUCTURAL_INFERENCE_LIMIT),
    )?;
    expect(
        contract,
        "/gate/task_inference_limit",
        json!(TASK_INFERENCE_LIMIT),
    )?;
    expect(contract, "/gate/max_inference_requests", json!(4))?;
    expect(contract, "/gate/max_token_count_contacts", json!(4))?;
    expect(
        contract,
        "/gate/inconclusive_state",
        json!(LOCAL_9B_FEASIBILITY_INCONCLUSIVE),
    )?;
    expect(
        contract,
        "/gate/inconclusive_flags",
        json!(LOCAL_9B_INCONCLUSIVE_TERMINAL_FLAGS),
    )?;
    expect(contract, "/runtime/multimodal_projector", json!("ABSENT"))?;
    expect(contract, "/runtime/request_content", json!("text-only"))?;
    expect(
        contract,
        "/runtime/reasoning_mechanism/request_chat_template_kwargs",
        json!("ABSENT"),
    )?;
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
    parse_server_options(&string_list(contract, "/runtime/launch_arguments")?)?;
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

/// Registered requests are reasoning-neutral and text-only: no field that
/// could re-enable reasoning or alter template semantics per request (in
/// particular `chat_template_kwargs` must be absent, not merely non-thinking),
/// and every message content is a plain string, so no image, audio, or other
/// multimodal part can be sent to this text-only runtime.
pub fn validate_registered_request(request: &Value) -> Result<()> {
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
    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .filter(|messages| !messages.is_empty())
        .ok_or_else(|| invalid("capable-model request has no messages"))?;
    if messages
        .iter()
        .any(|message| !message.get("content").is_some_and(Value::is_string))
    {
        return Err(invalid(
            "capable-model request content is not text-only string content",
        ));
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
            validate_registered_request(&request)?;
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
    validate_registered_request(&request)?;
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
        multimodal_projector: text(contract, "/runtime/multimodal_projector")?,
        request_content: text(contract, "/runtime/request_content")?,
        reasoning: ReasoningSpec {
            mode: text(contract, "/runtime/reasoning")?,
            launch_flag: string_list(contract, "/runtime/reasoning_mechanism/launch_flag")?,
            request_chat_template_kwargs: text(
                contract,
                "/runtime/reasoning_mechanism/request_chat_template_kwargs",
            )?,
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
            inference_limit: TASK_INFERENCE_LIMIT,
        },
        evaluators: EvaluatorSpec {
            structural_manifest_sha256: canonical_hash(manifest)?,
            task_evaluator_version: text(&evaluator, "/evaluator_version")?,
            task_evaluator_sha256: canonical_hash(&evaluator)?,
            task_tool_contract_version: text(&tool_contract, "/tool_contract_version")?,
        },
        stages: StageLimitsSpec {
            structural_inference_limit: STRUCTURAL_INFERENCE_LIMIT,
            task_inference_limit: TASK_INFERENCE_LIMIT,
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

/// The classified result of one dispatched inference request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestOutcome {
    Pass,
    /// Evidence attributable to the model or the scored request path.
    ModelFailure(&'static str),
    /// The request was dispatched but transport or the server did not yield
    /// a classifiable model result.
    InfrastructureAfterDispatch(&'static str),
}

impl RequestOutcome {
    fn value(self) -> Value {
        match self {
            Self::Pass => json!({ "outcome": "PASS" }),
            Self::ModelFailure(reason) => json!({ "outcome": "MODEL_FAILURE", "reason": reason }),
            Self::InfrastructureAfterDispatch(reason) => {
                json!({ "outcome": "INFRASTRUCTURE_AFTER_DISPATCH", "reason": reason })
            }
        }
    }
}

/// The frozen evaluator's verdict on a completed response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    /// No acceptable terminal answer (no content, tool call, or equivalent).
    NoAcceptableAnswer,
}

/// Facts of one dispatched request, taken from its persisted record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchObservation<'a> {
    pub transport_completed: bool,
    pub transport_timeout: bool,
    pub http_status: Option<u64>,
    pub body_complete: bool,
    pub json_parsed: bool,
    pub finish_reason: Option<&'a str>,
    pub verdict: Verdict,
}

/// Classify one dispatched request. An inference timeout after dispatch,
/// `length`, an evaluator FAIL, or no acceptable answer is a model-side
/// failure; any other transport failure, a non-200 status, or an unusable
/// response body is infrastructure after dispatch.
pub fn classify_dispatch(observation: &DispatchObservation<'_>) -> RequestOutcome {
    if !observation.transport_completed {
        return if observation.transport_timeout {
            RequestOutcome::ModelFailure("INFERENCE_TIMEOUT_AFTER_DISPATCH")
        } else {
            RequestOutcome::InfrastructureAfterDispatch("TRANSPORT_FAILED_AFTER_DISPATCH")
        };
    }
    if observation.http_status != Some(200) {
        return RequestOutcome::InfrastructureAfterDispatch("HTTP_STATUS_NOT_200");
    }
    if !observation.body_complete || !observation.json_parsed {
        return RequestOutcome::InfrastructureAfterDispatch("RESPONSE_BODY_UNUSABLE");
    }
    if observation.finish_reason == Some("length") {
        return RequestOutcome::ModelFailure("LENGTH");
    }
    match observation.verdict {
        Verdict::Pass => RequestOutcome::Pass,
        Verdict::Fail => RequestOutcome::ModelFailure("EVALUATOR_FAIL"),
        Verdict::NoAcceptableAnswer => {
            RequestOutcome::ModelFailure("NO_ACCEPTABLE_TERMINAL_ANSWER")
        }
    }
}

/// Observation of a structural probe from its calibration `case-result`.
pub fn structural_observation(case_result: &Value) -> DispatchObservation<'_> {
    let validation = &case_result["validation"];
    let response = &case_result["response"];
    DispatchObservation {
        transport_completed: validation["transport_ambiguous"] != true,
        transport_timeout: validation["transport_timeout"] == true,
        http_status: response["http_status"].as_u64(),
        body_complete: response["complete"] == true,
        json_parsed: validation["response_json_parsed"] == true,
        finish_reason: response["finish_reason"].as_str(),
        verdict: match case_result["state"].as_str() {
            Some("PASS") => Verdict::Pass,
            Some("FAIL") => Verdict::Fail,
            _ => Verdict::NoAcceptableAnswer,
        },
    }
}

/// Observation of the h001 request from its arm result, normalized turn,
/// and Stage 0 evaluator result (absent when transport was ambiguous).
pub fn task_observation<'a>(
    arm_result: &'a Value,
    normalized: Option<&'a Value>,
    evaluator_result: Option<&str>,
) -> DispatchObservation<'a> {
    DispatchObservation {
        transport_completed: arm_result["state"] != "AMBIGUOUS",
        transport_timeout: arm_result["validation"]["transport_timeout"] == true,
        http_status: arm_result["response"]["http_status"].as_u64(),
        body_complete: arm_result["response"]["complete"] == true,
        json_parsed: arm_result["validation"]["response_json_parsed"] == true,
        finish_reason: normalized.and_then(|turn| turn["finish_reason"].as_str()),
        verdict: match evaluator_result {
            Some("PASS") => Verdict::Pass,
            Some("FAIL") => Verdict::Fail,
            _ => Verdict::NoAcceptableAnswer,
        },
    }
}

/// Stage B runs only when all three structural probes passed.
pub fn task_stage_permitted(outcomes: &[RequestOutcome]) -> bool {
    outcomes.len() == STRUCTURAL_INFERENCE_LIMIT as usize
        && outcomes
            .iter()
            .all(|outcome| *outcome == RequestOutcome::Pass)
}

/// The pre-registered terminal classification. Dispatched requests are read
/// in order and the first non-pass decides: a model-side failure fails the
/// gate; infrastructure after dispatch is inconclusive. A token-count,
/// readiness, or executor failure is a pre-inference failure only while no
/// inference was dispatched, otherwise inconclusive. A context bound fails the
/// gate. Only the complete registered path of four passes passes.
pub fn classify_local_9b_gate(
    inference_requests: u32,
    infrastructure_failure: bool,
    context_bound: bool,
    outcomes: &[RequestOutcome],
) -> &'static str {
    if let Some(first) = outcomes
        .iter()
        .find(|outcome| **outcome != RequestOutcome::Pass)
    {
        return match first {
            RequestOutcome::ModelFailure(_) => LOCAL_9B_FEASIBILITY_FAILED,
            _ => LOCAL_9B_FEASIBILITY_INCONCLUSIVE,
        };
    }
    let registered_path = (STRUCTURAL_INFERENCE_LIMIT + TASK_INFERENCE_LIMIT) as usize;
    if !infrastructure_failure && !context_bound && outcomes.len() == registered_path {
        return LOCAL_9B_FEASIBILITY_PASSED;
    }
    if context_bound && !infrastructure_failure {
        return LOCAL_9B_FEASIBILITY_FAILED;
    }
    if inference_requests == 0 {
        LOCAL_9B_GATE_PRE_INFERENCE_FAILURE
    } else {
        LOCAL_9B_FEASIBILITY_INCONCLUSIVE
    }
}

/// What a gate classification permits next.
pub fn local_9b_gate_permission(classification: &str) -> &'static str {
    match classification {
        LOCAL_9B_FEASIBILITY_PASSED => "PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY",
        LOCAL_9B_FEASIBILITY_FAILED => "CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW",
        LOCAL_9B_GATE_PRE_INFERENCE_FAILURE => "REPLACEMENT_IDENTITY_REVIEW_ONLY_IF_ZERO_INFERENCE",
        _ => "DESIGN_REVIEW_REQUIRED",
    }
}

/// Terminal semantics recorded with every gate result.
pub fn terminal_state(classification: &str, inference_requests: u32) -> Value {
    let consumed = inference_requests > 0;
    let inconclusive = classification == LOCAL_9B_FEASIBILITY_INCONCLUSIVE;
    json!({
        "identity_consumed": consumed,
        "replacement_identity_eligible":
            classification == LOCAL_9B_GATE_PRE_INFERENCE_FAILURE && !consumed,
        "capability_result_established": matches!(
            classification,
            LOCAL_9B_FEASIBILITY_PASSED | LOCAL_9B_FEASIBILITY_FAILED
        ),
        "model_inadequacy_established": classification == LOCAL_9B_FEASIBILITY_FAILED,
        "cloud_gpu_review_authorized": classification == LOCAL_9B_FEASIBILITY_FAILED,
        "flags": if inconclusive {
            json!(LOCAL_9B_INCONCLUSIVE_TERMINAL_FLAGS)
        } else {
            json!([])
        }
    })
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

/// Options the registered server command may carry, each exactly once.
/// Anything else, including aliases such as `-rea` or `--model`, fails closed.
const SERVER_VALUE_OPTIONS: [&str; 6] = ["-m", "-c", "-np", "--reasoning", "--host", "--port"];
const SERVER_FLAG_OPTIONS: [&str; 2] = ["--metrics", "--offline"];

/// Parse server launch arguments into their effective options. The first
/// argument must be `serve`; every option must be registered and appear
/// exactly once, so a later duplicate cannot change effective semantics.
pub fn parse_server_options(
    arguments: &[String],
) -> Result<std::collections::BTreeMap<String, Option<String>>> {
    let Some((subcommand, rest)) = arguments.split_first() else {
        return Err(invalid("server launch arguments are empty"));
    };
    if subcommand != "serve" {
        return Err(invalid("server launch subcommand is not serve"));
    }
    let mut options = std::collections::BTreeMap::new();
    let mut index = 0;
    while index < rest.len() {
        let option = rest[index].as_str();
        let value = if SERVER_VALUE_OPTIONS.contains(&option) {
            index += 1;
            Some(
                rest.get(index)
                    .ok_or_else(|| invalid(&format!("server option {option} has no value")))?
                    .clone(),
            )
        } else if SERVER_FLAG_OPTIONS.contains(&option) {
            None
        } else {
            return Err(invalid(&format!(
                "server launch argument {option} is not registered"
            )));
        };
        if options.insert(option.to_string(), value).is_some() {
            return Err(invalid(&format!(
                "server launch argument {option} appears more than once"
            )));
        }
        index += 1;
    }
    Ok(options)
}

/// The server's argument vector must name the registered executable and
/// carry exactly the registered effective options: the frozen `-m` GGUF,
/// `-c 8192`, `-np 1`, `--metrics`, one `--reasoning off`, `--offline`,
/// `--host 127.0.0.1`, and `--port 8080`, and nothing else. Reasoning mode
/// is server-wide in b10217, so any other reasoning setting fails.
pub fn validate_server_arguments(arguments: &[String], spec: &CapableModelGateSpec) -> Result<()> {
    let Some((program, rest)) = arguments.split_first() else {
        return Err(invalid("server command line is empty"));
    };
    if !program.eq_ignore_ascii_case(&spec.runtime_executable.path) {
        return Err(invalid(
            "server command line names an unregistered executable",
        ));
    }
    if parse_server_options(rest)? != parse_server_options(&spec.launch_arguments)? {
        return Err(invalid(
            "server effective options differ from the registered launch arguments",
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
        "terminal": terminal_state(classification, accounting.inference()),
        "detail": detail,
        "accounting": accounting.value(),
        "recorded_at_unix_ms": now_unix_ms()?
    }))
}

/// Refuse a contact that would exceed its registered stage capacity.
fn ensure_capacity(used: u32, limit: u32, contact: &str) -> Result<()> {
    if used >= limit {
        return Err(invalid(&format!(
            "local-9B {contact} would exceed its registered limit of {limit}"
        )));
    }
    Ok(())
}

fn seal_result(root: &Path, result: &Value) -> Result<()> {
    write_json(&root.join("gate-result.json"), result)?;
    write_bytes(
        &root.join("gate-result.sha256"),
        format!("{}  gate-result.json\n", canonical_hash(result)?).as_bytes(),
    )
}

/// A non-request infrastructure failure: before any inference it is a
/// pre-inference failure; after dispatch it is recorded for the inconclusive
/// classification.
fn infrastructure_record(
    accounting: &Accounting,
    stage: &str,
    case_id: &str,
    contact: &str,
    error: &dyn std::fmt::Display,
) -> Value {
    let when = if accounting.inference() == 0 {
        "BEFORE_INFERENCE"
    } else {
        "AFTER_DISPATCH"
    };
    json!({
        "failure_class": format!("{contact}_FAILED_{when}"),
        "stage": stage,
        "case_id": case_id,
        "error": error.to_string()
    })
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
    let mut outcomes = Vec::new();
    let mut requests = Vec::new();
    let mut context_bound = false;
    let mut infrastructure = None;

    'stages: {
        // Stage A: structural probes, each counted and guarded before dispatch.
        for (case_id, case, request) in &prerequisites.structural {
            ensure_capacity(
                accounting.structural_token_counts,
                spec.stages.structural_inference_limit,
                "structural token count",
            )?;
            accounting.structural_token_counts += 1;
            let (input_tokens, record) = match count_input_tokens(
                &client,
                &spec.token_count_endpoint,
                token_timeout,
                request,
            ) {
                Ok(counted) => counted,
                Err(error) => {
                    infrastructure = Some(infrastructure_record(
                        &accounting,
                        "structural",
                        case_id,
                        "TOKEN_COUNT",
                        &error,
                    ));
                    break 'stages;
                }
            };
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
                break 'stages;
            }
            let case_dir = root.join(case_id);
            fs::create_dir_all(&case_dir)?;
            ensure_capacity(
                accounting.structural_inference,
                spec.stages.structural_inference_limit,
                "structural inference request",
            )?;
            accounting.structural_inference += 1;
            let case_result =
                match execute_case(&prerequisites.gate_manifest, case, None, &case_dir, &client) {
                    Ok(case_result) => case_result,
                    Err(error) => {
                        infrastructure = Some(infrastructure_record(
                            &accounting,
                            "structural",
                            case_id,
                            "EXECUTOR",
                            &error,
                        ));
                        break 'stages;
                    }
                };
            let outcome = classify_dispatch(&structural_observation(&case_result));
            requests.push(json!({
                "stage": "structural",
                "case_id": case_id,
                "evaluator_state": case_result.get("state"),
                "classification": outcome.value()
            }));
            outcomes.push(outcome);
            if outcome != RequestOutcome::Pass {
                break 'stages;
            }
        }

        // Stage B: one h001 BASELINE request, only after three structural passes.
        if !task_stage_permitted(&outcomes) {
            break 'stages;
        }
        let request = &prerequisites.task;
        ensure_capacity(
            accounting.task_token_counts,
            spec.stages.task_inference_limit,
            "task token count",
        )?;
        accounting.task_token_counts += 1;
        let (input_tokens, record) =
            match count_input_tokens(&client, &spec.token_count_endpoint, token_timeout, request) {
                Ok(counted) => counted,
                Err(error) => {
                    infrastructure = Some(infrastructure_record(
                        &accounting,
                        "task_level",
                        "h001",
                        "TOKEN_COUNT",
                        &error,
                    ));
                    break 'stages;
                }
            };
        let fits = context_guard(input_tokens, &spec.limits) == ContextDecision::Fits;
        counts.push(json!({
            "stage": "task_level",
            "case_id": "h001",
            "input_tokens": input_tokens,
            "fits": fits,
            "record": record
        }));
        if !fits {
            context_bound = true;
            break 'stages;
        }
        let task_dir = root.join("h001-baseline");
        fs::create_dir_all(&task_dir)?;
        ensure_capacity(
            accounting.task_inference,
            spec.stages.task_inference_limit,
            "task inference request",
        )?;
        accounting.task_inference += 1;
        let turn = h001::execute_h001_turn(H001TurnInput {
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
        });
        let arm_result = match turn {
            Ok(arm_result) => arm_result,
            Err(error) => {
                infrastructure = Some(infrastructure_record(
                    &accounting,
                    "task_level",
                    "h001",
                    "EXECUTOR",
                    &error,
                ));
                break 'stages;
            }
        };
        // An ambiguous transport leaves no normalized turn, so it is not scored.
        let scored = if arm_result["state"] == "AMBIGUOUS" {
            None
        } else {
            let evaluation = fs::read(task_dir.join("normalized-turn-1.json"))
                .map_err(H001Error::from)
                .and_then(|bytes| Ok(serde_json::from_slice::<Value>(&bytes)?))
                .and_then(|normalized| {
                    Ok((
                        normalized,
                        h001::score_h001_arm_at(H001Arm::Baseline, task_dir.clone())?,
                    ))
                });
            match evaluation {
                Ok(scored) => Some(scored),
                Err(error) => {
                    infrastructure = Some(infrastructure_record(
                        &accounting,
                        "task_level",
                        "h001",
                        "EXECUTOR",
                        &error,
                    ));
                    break 'stages;
                }
            }
        };
        let evaluator_result = scored
            .as_ref()
            .and_then(|(_, evaluation)| evaluation["result"].as_str());
        let outcome = classify_dispatch(&task_observation(
            &arm_result,
            scored.as_ref().map(|(normalized, _)| normalized),
            evaluator_result,
        ));
        requests.push(json!({
            "stage": "task_level",
            "case_id": "h001",
            "arm_state": arm_result.get("state"),
            "evaluator_result": evaluator_result,
            "http_status": arm_result.pointer("/response/http_status"),
            "classification": outcome.value()
        }));
        outcomes.push(outcome);
    }
    write_json(&root.join("token-counts.json"), &json!(counts))?;

    let classification = classify_local_9b_gate(
        accounting.inference(),
        infrastructure.is_some(),
        context_bound,
        &outcomes,
    );
    let result = gate_result(
        &prerequisites,
        classification,
        json!({
            "context_result": if context_bound { json!(INCONCLUSIVE_CONTEXT_BOUND) } else { Value::Null },
            "infrastructure": infrastructure,
            "requests": requests,
            "task_stage_executed": accounting.task_inference > 0,
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
        assert_eq!(spec.limits.readiness_contacts, 1);
        // REGISTERED_MAX_INFERENCE_REQUESTS = 3 structural + 1 frozen h001 BASELINE.
        assert_eq!(spec.limits.inference_requests, 4);
        assert_eq!(spec.limits.token_count_contacts, 4);
        assert_eq!(spec.stages.structural_inference_limit, 3);
        assert_eq!(spec.stages.task_inference_limit, 1);
        assert_eq!(spec.task_case.inference_limit, 1);
        assert!(spec.stages.task_stage_requires_all_structural_pass);
        assert_eq!(spec.model_file.sha256, LOCAL_9B_GGUF_SHA256);
        assert_eq!(spec.model_source.upstream_sha256, LOCAL_9B_GGUF_SHA256);
        assert_eq!(
            spec.runtime_executable.windows_file_id,
            V3_LLAMA_EXECUTABLE_FILE_ID
        );
        assert_eq!(spec.reasoning.mode, "off");
        assert_eq!(spec.reasoning.launch_flag, ["--reasoning", "off"]);
        assert_eq!(spec.reasoning.request_chat_template_kwargs, "ABSENT");
        assert_eq!(spec.multimodal_projector, "ABSENT");
        assert_eq!(spec.request_content, "text-only");
        assert_eq!(spec.task_case.task_id, "h001");
        assert_eq!(spec.task_case.arm, "BASELINE");

        for (pointer, value) in [
            ("/gate/max_inference_requests", json!(6)),
            ("/gate/max_token_count_contacts", json!(6)),
            ("/gate/task_inference_limit", json!(3)),
            (
                "/runtime/multimodal_projector",
                json!("mmproj-Qwen3.5-9B-BF16.gguf"),
            ),
            ("/runtime/request_content", json!("multimodal")),
            (
                "/gate/inconclusive_state",
                json!("LOCAL_9B_FEASIBILITY_FAILED"),
            ),
            (
                "/gate/inconclusive_flags",
                json!(["DESIGN_REVIEW_REQUIRED"]),
            ),
            (
                "/runtime/reasoning_mechanism/request_chat_template_kwargs",
                json!({ "enable_thinking": false }),
            ),
        ] {
            let mut changed = contract();
            *changed.pointer_mut(pointer).unwrap() = value;
            assert!(validate_local_9b_contract(&changed).is_err(), "{pointer}");
        }
    }

    #[test]
    fn derived_deadline_covers_every_contact_and_rejects_independent_values() {
        let spec = spec();
        let d = &spec.deadlines;
        let stages = &spec.stages;
        let structural = u64::from(stages.structural_inference_limit);
        let task = u64::from(stages.task_inference_limit);
        let derived = d.readiness_timeout_ms
            + structural * d.token_count_request_timeout_ms
            + task * d.token_count_request_timeout_ms
            + structural * d.inference_request_timeout_ms
            + task * d.inference_request_timeout_ms
            + d.non_request_margin_ms;
        // 1*1000 + 3*60000 + 1*60000 + 3*3540000 + 1*3540000 + 1200000
        assert_eq!(derived, 15_601_000);
        assert_eq!(d.supervisor_deadline_ms, derived);

        let mut six_request_capacity = contract();
        six_request_capacity["timeout_policy"]["supervisor_deadline_ms"] = json!(22_801_000);
        assert!(validate_local_9b_contract(&six_request_capacity).is_err());
        let mut independent = contract();
        independent["timeout_policy"]["supervisor_deadline_ms"] = json!(derived + 1);
        assert!(validate_local_9b_contract(&independent).is_err());
        let mut v3_inference = contract();
        v3_inference["timeout_policy"]["inference_request_timeout_ms"] = json!(2_400_000);
        assert!(validate_local_9b_contract(&v3_inference).is_err());
        let mut changed = spec.clone();
        changed.deadlines.non_request_margin_ms += 1;
        assert!(validate_local_9b_gate_spec(&changed, &contract(), &manifest()).is_err());
        let mut widened = spec.clone();
        widened.limits.inference_requests = 6;
        widened.limits.token_count_contacts = 6;
        widened.deadlines.supervisor_deadline_ms = 22_801_000;
        assert!(validate_local_9b_gate_spec(&widened, &contract(), &manifest()).is_err());
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
        let spec = spec();
        let (_, _, request) = structural_requests(
            &gate_calibration_manifest(&manifest(), &spec.model_label).unwrap(),
        )
        .unwrap()
        .remove(0);
        let task = task_request(&spec.model_label).unwrap();
        for registered in [&request, &task] {
            validate_registered_request(registered).unwrap();
            assert!(registered.get("chat_template_kwargs").is_none());

            // chat_template_kwargs is rejected whatever it carries, including
            // an explicit thinking-disabled value.
            for kwargs in [
                json!({ "enable_thinking": true }),
                json!({ "enable_thinking": false }),
                json!({}),
                json!({ "other": 1 }),
            ] {
                let mut overridden = registered.clone();
                overridden["chat_template_kwargs"] = kwargs.clone();
                assert!(
                    validate_registered_request(&overridden).is_err(),
                    "{kwargs}"
                );
                assert_ne!(
                    canonical_hash(&overridden).unwrap(),
                    canonical_hash(registered).unwrap()
                );
            }
            for key in FORBIDDEN_REQUEST_FIELDS {
                let mut overridden = registered.clone();
                overridden[key] = json!("none");
                assert!(validate_registered_request(&overridden).is_err(), "{key}");
            }
        }
        // Registered hashes bind the absence: a request carrying
        // chat_template_kwargs cannot match the registered hash.
        let mut enabled = task.clone();
        enabled["chat_template_kwargs"] = json!({ "enable_thinking": true });
        assert_ne!(
            canonical_hash(&enabled).unwrap(),
            spec.task_case.request_sha256
        );
    }

    #[test]
    fn requests_are_text_only() {
        let task = task_request(&spec().model_label).unwrap();
        let mut image = task.clone();
        image["messages"][0]["content"] = json!([
            { "type": "text", "text": "x" },
            { "type": "image_url", "image_url": { "url": "data:image/png;base64,AA==" } }
        ]);
        assert!(validate_registered_request(&image).is_err());
        let mut parts = task.clone();
        parts["messages"][0]["content"] = json!([{ "type": "text", "text": "x" }]);
        assert!(validate_registered_request(&parts).is_err());
        let mut empty = task.clone();
        empty["messages"] = json!([]);
        assert!(validate_registered_request(&empty).is_err());
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
        let exe = &spec.runtime_executable.path;
        let model = &spec.model_file.path;
        let build = |arguments: &str| format!("\"{exe}\" serve {arguments}");
        let accepted = format!(
            "-m {model} -c 8192 -np 1 --metrics --reasoning off --offline --host 127.0.0.1 --port 8080"
        );
        validate_server_arguments(&split_windows_command_line(&build(&accepted)), &spec).unwrap();
        // Order does not change effective semantics; unquoted program is accepted.
        let reordered = format!(
            "--port 8080 --host 127.0.0.1 --offline --reasoning off --metrics -np 1 -c 8192 -m {model}"
        );
        validate_server_arguments(&split_windows_command_line(&build(&reordered)), &spec).unwrap();
        validate_server_arguments(
            &split_windows_command_line(&format!("{exe} serve {accepted}")),
            &spec,
        )
        .unwrap();

        let rejected = |label: &str, arguments: String| {
            assert!(
                validate_server_arguments(&split_windows_command_line(&build(&arguments)), &spec)
                    .is_err(),
                "{label}"
            );
        };
        rejected(
            "reasoning omitted",
            accepted.replace(" --reasoning off", ""),
        );
        rejected(
            "reasoning on",
            accepted.replace("--reasoning off", "--reasoning on"),
        );
        rejected(
            "reasoning auto",
            accepted.replace("--reasoning off", "--reasoning auto"),
        );
        rejected(
            "duplicate reasoning off",
            format!("{accepted} --reasoning off"),
        );
        rejected(
            "conflicting reasoning",
            format!("{accepted} --reasoning on"),
        );
        rejected(
            "reasoning alias",
            accepted.replace("--reasoning off", "-rea off"),
        );
        rejected(
            "reasoning equals form",
            accepted.replace("--reasoning off", "--reasoning=off"),
        );
        rejected(
            "reasoning budget",
            format!("{accepted} --reasoning-budget 256"),
        );
        rejected(
            "chat template kwargs",
            format!("{accepted} --chat-template-kwargs {{\"enable_thinking\":false}}"),
        );
        rejected(
            "hugging face",
            accepted.replace(
                &format!("-m {model}"),
                "-hf lmstudio-community/Qwen3.5-9B-GGUF:Q4_K_M",
            ),
        );
        rejected(
            "mmproj",
            format!("{accepted} --mmproj D:\\Prefixity-Lab\\models\\Qwen3.5-9B\\mmproj-Qwen3.5-9B-BF16.gguf"),
        );
        rejected("mm alias", format!("{accepted} -mm D:\\x.gguf"));
        rejected(
            "other model path",
            accepted.replace(
                model.as_str(),
                "D:\\Prefixity-Lab\\models\\Qwen3.5-9B\\copy.gguf",
            ),
        );
        rejected("model alias", accepted.replace("-m ", "--model "));
        rejected("duplicate model", format!("{accepted} -m {model}"));
        rejected("context 4096", accepted.replace("-c 8192", "-c 4096"));
        rejected(
            "context alias",
            accepted.replace("-c 8192", "--ctx-size 8192"),
        );
        rejected("slots 2", accepted.replace("-np 1", "-np 2"));
        rejected("metrics omitted", accepted.replace(" --metrics", ""));
        rejected("offline omitted", accepted.replace(" --offline", ""));
        rejected(
            "host 0.0.0.0",
            accepted.replace("--host 127.0.0.1", "--host 0.0.0.0"),
        );
        rejected("port 8081", accepted.replace("--port 8080", "--port 8081"));
        rejected("value missing", accepted.replace(" --port 8080", " --port"));
        rejected("unknown flag", format!("{accepted} --jinja"));
        assert!(validate_server_arguments(
            &split_windows_command_line(&format!("\"{exe}\" cli {accepted}")),
            &spec
        )
        .is_err());
        assert!(validate_server_arguments(
            &split_windows_command_line(&build(&accepted).replace("llama.exe", "llama-server.exe")),
            &spec
        )
        .is_err());

        assert_eq!(
            split_windows_command_line(r#""C:\a b\x.exe" "q r" s\"t u\\\"v"#),
            [r"C:\a b\x.exe", "q r", "s\"t", "u\\\"v"]
        );
    }

    use RequestOutcome::{InfrastructureAfterDispatch, ModelFailure, Pass};

    fn completed(finish_reason: &str, verdict: Verdict) -> DispatchObservation<'_> {
        DispatchObservation {
            transport_completed: true,
            transport_timeout: false,
            http_status: Some(200),
            body_complete: true,
            json_parsed: true,
            finish_reason: Some(finish_reason),
            verdict,
        }
    }

    const PASSES: [RequestOutcome; 3] = [Pass, Pass, Pass];

    #[test]
    fn zero_inference_pre_inference_failure_keeps_the_replacement_rule() {
        // Readiness or first token-count failure: nothing dispatched.
        let classification = classify_local_9b_gate(0, true, false, &[]);
        assert_eq!(classification, LOCAL_9B_GATE_PRE_INFERENCE_FAILURE);
        assert_eq!(
            local_9b_gate_permission(classification),
            "REPLACEMENT_IDENTITY_REVIEW_ONLY_IF_ZERO_INFERENCE"
        );
        let terminal = terminal_state(classification, 0);
        assert_eq!(terminal["identity_consumed"], false);
        assert_eq!(terminal["replacement_identity_eligible"], true);
        assert_eq!(terminal["capability_result_established"], false);
        assert_eq!(terminal["cloud_gpu_review_authorized"], false);
    }

    #[test]
    fn structural_model_failure_after_dispatch_fails_the_gate() {
        let outcome = classify_dispatch(&completed("stop", Verdict::Fail));
        assert_eq!(outcome, ModelFailure("EVALUATOR_FAIL"));
        let no_answer = classify_dispatch(&completed("stop", Verdict::NoAcceptableAnswer));
        assert_eq!(no_answer, ModelFailure("NO_ACCEPTABLE_TERMINAL_ANSWER"));
        for outcomes in [vec![outcome], vec![Pass, no_answer]] {
            let classification =
                classify_local_9b_gate(outcomes.len() as u32, false, false, &outcomes);
            assert_eq!(classification, LOCAL_9B_FEASIBILITY_FAILED);
            assert_eq!(
                local_9b_gate_permission(classification),
                "CLOUD_GPU_CAPABLE_MODEL_DESIGN_REVIEW"
            );
            let terminal = terminal_state(classification, outcomes.len() as u32);
            assert_eq!(terminal["identity_consumed"], true);
            assert_eq!(terminal["model_inadequacy_established"], true);
            assert_eq!(terminal["cloud_gpu_review_authorized"], true);
        }
        assert!(!task_stage_permitted(&[Pass, outcome]));
    }

    #[test]
    fn length_after_dispatch_fails_the_gate() {
        // Length decides even if the evaluator verdict were otherwise.
        for verdict in [Verdict::Fail, Verdict::NoAcceptableAnswer, Verdict::Pass] {
            assert_eq!(
                classify_dispatch(&completed("length", verdict)),
                ModelFailure("LENGTH")
            );
        }
        let structural = [ModelFailure("LENGTH")];
        assert_eq!(
            classify_local_9b_gate(1, false, false, &structural),
            LOCAL_9B_FEASIBILITY_FAILED
        );
        let task = [Pass, Pass, Pass, ModelFailure("LENGTH")];
        assert_eq!(
            classify_local_9b_gate(4, false, false, &task),
            LOCAL_9B_FEASIBILITY_FAILED
        );
        // An inference timeout after dispatch is model-side.
        let timeout = DispatchObservation {
            transport_completed: false,
            transport_timeout: true,
            http_status: None,
            body_complete: false,
            json_parsed: false,
            finish_reason: None,
            verdict: Verdict::NoAcceptableAnswer,
        };
        assert_eq!(
            classify_dispatch(&timeout),
            ModelFailure("INFERENCE_TIMEOUT_AFTER_DISPATCH")
        );
    }

    #[test]
    fn h001_evaluator_model_failure_fails_the_gate() {
        let arm_result = json!({
            "state": "COMPLETE",
            "response": { "http_status": 200, "complete": true },
            "validation": { "response_json_parsed": true }
        });
        let normalized = json!({ "finish_reason": "stop" });
        let outcome = classify_dispatch(&task_observation(
            &arm_result,
            Some(&normalized),
            Some("FAIL"),
        ));
        assert_eq!(outcome, ModelFailure("EVALUATOR_FAIL"));
        assert_eq!(
            classify_local_9b_gate(4, false, false, &[Pass, Pass, Pass, outcome]),
            LOCAL_9B_FEASIBILITY_FAILED
        );
        // No terminal answer (for example a tool call) is also model-side.
        let incomplete = json!({
            "state": "INCONCLUSIVE",
            "response": { "http_status": 200, "complete": true },
            "validation": { "response_json_parsed": true }
        });
        assert_eq!(
            classify_dispatch(&task_observation(
                &incomplete,
                Some(&normalized),
                Some("INCONCLUSIVE")
            )),
            ModelFailure("NO_ACCEPTABLE_TERMINAL_ANSWER")
        );
    }

    #[test]
    fn infrastructure_after_structural_inference_is_inconclusive_and_consumed() {
        // Three structural passes dispatched; the h001 token count then fails
        // before h001 inference.
        let classification = classify_local_9b_gate(3, true, false, &PASSES);
        assert_eq!(classification, LOCAL_9B_FEASIBILITY_INCONCLUSIVE);
        assert_ne!(classification, LOCAL_9B_FEASIBILITY_FAILED);
        assert_ne!(classification, LOCAL_9B_FEASIBILITY_PASSED);
        assert_eq!(
            local_9b_gate_permission(classification),
            "DESIGN_REVIEW_REQUIRED"
        );
        let terminal = terminal_state(classification, 3);
        assert_eq!(terminal["identity_consumed"], true);
        assert_eq!(terminal["replacement_identity_eligible"], false);
        assert_eq!(terminal["capability_result_established"], false);
        assert_eq!(terminal["model_inadequacy_established"], false);
        assert_eq!(terminal["cloud_gpu_review_authorized"], false);
        assert_eq!(
            terminal["flags"],
            json!(LOCAL_9B_INCONCLUSIVE_TERMINAL_FLAGS)
        );
        // Also after only one structural request.
        assert_eq!(
            classify_local_9b_gate(1, true, false, &[Pass]),
            LOCAL_9B_FEASIBILITY_INCONCLUSIVE
        );
        let record = infrastructure_record(
            &Accounting {
                structural_token_counts: 3,
                task_token_counts: 1,
                structural_inference: 3,
                task_inference: 0,
            },
            "task_level",
            "h001",
            "TOKEN_COUNT",
            &"connection refused",
        );
        assert_eq!(record["failure_class"], "TOKEN_COUNT_FAILED_AFTER_DISPATCH");
        let before = infrastructure_record(
            &Accounting::default(),
            "structural",
            "rbcal-001",
            "TOKEN_COUNT",
            &"connection refused",
        );
        assert_eq!(
            before["failure_class"],
            "TOKEN_COUNT_FAILED_BEFORE_INFERENCE"
        );
    }

    #[test]
    fn transport_failure_of_a_dispatched_request_is_inconclusive_and_consumed() {
        let reset = DispatchObservation {
            transport_completed: false,
            transport_timeout: false,
            http_status: None,
            body_complete: false,
            json_parsed: false,
            finish_reason: None,
            verdict: Verdict::NoAcceptableAnswer,
        };
        let server_error = DispatchObservation {
            http_status: Some(503),
            ..completed("stop", Verdict::Fail)
        };
        let unusable = DispatchObservation {
            json_parsed: false,
            ..completed("stop", Verdict::Fail)
        };
        let truncated = DispatchObservation {
            body_complete: false,
            ..completed("stop", Verdict::Fail)
        };
        for (observation, reason) in [
            (&reset, "TRANSPORT_FAILED_AFTER_DISPATCH"),
            (&server_error, "HTTP_STATUS_NOT_200"),
            (&unusable, "RESPONSE_BODY_UNUSABLE"),
            (&truncated, "RESPONSE_BODY_UNUSABLE"),
        ] {
            let outcome = classify_dispatch(observation);
            assert_eq!(outcome, InfrastructureAfterDispatch(reason));
            for outcomes in [vec![outcome], vec![Pass, Pass, Pass, outcome]] {
                let classification =
                    classify_local_9b_gate(outcomes.len() as u32, false, false, &outcomes);
                assert_eq!(
                    classification, LOCAL_9B_FEASIBILITY_INCONCLUSIVE,
                    "{reason}"
                );
                let terminal = terminal_state(classification, outcomes.len() as u32);
                assert_eq!(terminal["identity_consumed"], true);
                assert_eq!(terminal["replacement_identity_eligible"], false);
            }
        }
        // The persisted records map onto the same classes.
        let structural_reset = json!({
            "state": "INCONCLUSIVE",
            "response": { "http_status": null, "complete": false },
            "validation": { "transport_ambiguous": true, "transport_timeout": false, "response_json_parsed": false }
        });
        assert_eq!(
            classify_dispatch(&structural_observation(&structural_reset)),
            InfrastructureAfterDispatch("TRANSPORT_FAILED_AFTER_DISPATCH")
        );
        let structural_timeout = json!({
            "state": "INCONCLUSIVE",
            "response": { "http_status": null, "complete": false },
            "validation": { "transport_ambiguous": true, "transport_timeout": true, "response_json_parsed": false }
        });
        assert_eq!(
            classify_dispatch(&structural_observation(&structural_timeout)),
            ModelFailure("INFERENCE_TIMEOUT_AFTER_DISPATCH")
        );
        let h001_ambiguous = json!({
            "state": "AMBIGUOUS",
            "response": { "http_status": null, "complete": false },
            "validation": { "response_json_parsed": false, "transport_timeout": false }
        });
        assert_eq!(
            classify_dispatch(&task_observation(&h001_ambiguous, None, None)),
            InfrastructureAfterDispatch("TRANSPORT_FAILED_AFTER_DISPATCH")
        );
        let structural_503 = json!({
            "state": "FAIL",
            "response": { "http_status": 503, "complete": true, "finish_reason": null },
            "validation": { "transport_ambiguous": false, "response_json_parsed": false }
        });
        assert_eq!(
            classify_dispatch(&structural_observation(&structural_503)),
            InfrastructureAfterDispatch("HTTP_STATUS_NOT_200")
        );
    }

    #[test]
    fn only_the_complete_registered_path_passes() {
        let pass = classify_dispatch(&completed("stop", Verdict::Pass));
        assert_eq!(pass, Pass);
        let full = [Pass, Pass, Pass, Pass];
        let classification = classify_local_9b_gate(4, false, false, &full);
        assert_eq!(classification, LOCAL_9B_FEASIBILITY_PASSED);
        assert_eq!(
            local_9b_gate_permission(classification),
            "PREFIXITY_PILOT_CONTEXT_ADEQUACY_REVIEW_ONLY"
        );
        assert_eq!(
            terminal_state(classification, 4)["capability_result_established"],
            true
        );
        assert!(task_stage_permitted(&PASSES));

        // Every incomplete, bounded, or disrupted path is not a pass.
        for (inference, infrastructure, context_bound, outcomes) in [
            (3, false, false, PASSES.to_vec()),
            (3, false, true, PASSES.to_vec()),
            (4, true, false, full.to_vec()),
            (2, false, false, vec![Pass, Pass]),
            (0, false, true, vec![]),
            (3, true, false, PASSES.to_vec()),
        ] {
            assert_ne!(
                classify_local_9b_gate(inference, infrastructure, context_bound, &outcomes),
                LOCAL_9B_FEASIBILITY_PASSED,
                "{inference} {infrastructure} {context_bound} {outcomes:?}"
            );
        }
        // A context bound under the registered rule fails the gate.
        assert_eq!(
            classify_local_9b_gate(3, false, true, &PASSES),
            LOCAL_9B_FEASIBILITY_FAILED
        );
        assert_eq!(
            classify_local_9b_gate(0, false, true, &[]),
            LOCAL_9B_FEASIBILITY_FAILED
        );
        for incomplete in [vec![Pass], vec![Pass, Pass]] {
            assert!(!task_stage_permitted(&incomplete));
        }
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
