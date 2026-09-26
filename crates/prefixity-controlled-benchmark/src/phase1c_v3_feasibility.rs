//! Phase 1C V3 feasibility gate: one final bounded falsification gate for
//! the current Qwen3.5-0.8B instrument.
//!
//! Everything gate-specific lives in one data structure, [`V3GateSpec`],
//! carried by the tracked gate identity and derived deterministically from
//! the tracked V3 runtime contract and calibration manifest. Validators are
//! generic functions of that spec; there is no per-attempt validator family.
//! The live entry point and the offline prerequisite traversal share
//! [`gate_prerequisites`]. Token counts come only from the frozen server's
//! non-inference `/v1/chat/completions/input_tokens` endpoint.

use crate::hashing::canonical_hash;
use crate::phase1c_executable_identity::{self as executable_identity, ExecutableIdentity};
use crate::phase1c_reasoning_budget_calibration::{
    aggregate_state, attempt_003_runtime_ownership_with_expected_workflow, build_request,
    execute_case, expected_workflow_identity_from_supervisor_env, now_unix_ms, read_json,
    read_manifest, same_workflow_identity_path, source_sha256,
    validate_accepted_workflow_certification_v2, validate_manifest, windows_native_prestart_value,
    workspace_path, write_bytes, write_json, ReasoningBudgetCalibrationError, CALIBRATION_CASE_IDS,
    ENDPOINT, HOST, PORT,
};
use crate::phase1c_windows_runtime_exclusivity as windows_exclusivity;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, ReasoningBudgetCalibrationError>;

pub const V3_CONTRACT_PATH: &str = "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V3.json";
pub const V3_CONTRACT_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_SCORED_RUNTIME_CONTRACT_V3.sha256";
pub const V3_GATE_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_IDENTITY_V1.json";
pub const V3_GATE_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_V3_FEASIBILITY_GATE_IDENTITY_V1.sha256";
pub const V3_GATE_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-scored-capability-v3/feasibility-gate";
pub const V3_GATE_FROZEN_STAGE_ROOT: &str = "target/phase1c-v3-feasibility-frozen";
pub const V3_GATE_TRAVERSAL_ROOT: &str = "target/phase1c-v3-feasibility-prerequisite-traversal";
pub const V3_GATE_ID: &str = "phase1c-v3-feasibility-gate";
pub const V3_MAX_TOKENS: u64 = 4096;
pub const V3_CONTEXT_TOKENS: u64 = 8192;
/// Windows file identity of the frozen llama.cpp executable on C:, as the
/// repository `inspect()` reports it from the opened handle. Attempts 009-011
/// recorded `volume=ba2f80f4` (the D: serial) for this file; that historical
/// value is rejected here.
pub const V3_LLAMA_EXECUTABLE_FILE_ID: &str = "volume=c4c93b54;index=00060000001ea970";
pub const V3_TOKEN_COUNT_ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions/input_tokens";

pub const V3_FEASIBILITY_PASSED: &str = "V3_FEASIBILITY_PASSED";
pub const CURRENT_QWEN_SCORED_PATH_CLOSED: &str = "CURRENT_QWEN_SCORED_PATH_CLOSED";
pub const INCONCLUSIVE_CONTEXT_BOUND: &str = "INCONCLUSIVE_CONTEXT_BOUND";
pub const GATE_PRE_INFERENCE_FAILURE: &str = "GATE_PRE_INFERENCE_FAILURE";

const V3_CHILD_BINARY: &str = "prefixity-phase1c-v3-feasibility.exe";
const V3_SUPERVISOR_BINARY: &str = "prefixity-phase1c-live-supervisor.exe";
/// Implementation sources whose current bytes must equal the prepared ones.
/// The executable-identity source is bound because V3 depends directly on
/// its stable Windows file identity and its validation semantics.
pub const V3_BOUND_SOURCES: [&str; 7] = [
    "crates/prefixity-controlled-benchmark/src/phase1c_v3_feasibility.rs",
    "crates/prefixity-controlled-benchmark/src/bin/phase1c_v3_feasibility.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_live_supervisor.rs",
    "crates/prefixity-controlled-benchmark/src/bin/phase1c_live_supervisor.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_windows_runtime_exclusivity.rs",
    "crates/prefixity-controlled-benchmark/src/phase1c_executable_identity.rs",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileIdentitySpec {
    pub path: String,
    pub sha256: String,
    pub file_size: u64,
    pub windows_file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSpec {
    pub case_id: String,
    pub request_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimitsSpec {
    pub context_tokens: u64,
    pub max_tokens: u64,
    pub readiness_contacts: u32,
    pub token_count_contacts: u32,
    pub inference_requests: u32,
    pub retry_requests: u32,
    pub fallback_requests: u32,
    pub adaptive_replicates: u32,
    pub warmup_requests: u32,
}

/// Every child-lifecycle bound. Each network contact class has its own
/// complete-request bound; none inherits the generation timeout. The
/// supervisor deadline is not an independent value: it must equal
/// [`derive_supervisor_deadline_ms`] of the other components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeadlinesSpec {
    /// TCP connect bound inside each HTTP request's complete bound.
    pub connect_timeout_ms: u64,
    /// Complete bound of one readiness contact (the listener connect).
    pub readiness_timeout_ms: u64,
    /// Complete bound of one `/v1/chat/completions/input_tokens` request.
    pub token_count_request_timeout_ms: u64,
    /// Complete bound of one `/v1/chat/completions` inference request.
    pub inference_request_timeout_ms: u64,
    /// Explicit bound for all non-request child work: process start,
    /// prerequisite hashing, ownership inspection, evidence writes, exit.
    pub non_request_margin_ms: u64,
    /// Derived; see [`derive_supervisor_deadline_ms`].
    pub supervisor_deadline_ms: u64,
}

/// The complete child-lifecycle bound:
///
/// ```text
/// readiness_contacts   * readiness_timeout_ms
/// + token_count_contacts * token_count_request_timeout_ms
/// + inference_requests   * inference_request_timeout_ms
/// + non_request_margin_ms
/// ```
///
/// Every component must be positive and the connect bound must fit inside
/// every contact bound. Overflow is rejected rather than saturated.
pub fn derive_supervisor_deadline_ms(
    limits: &LimitsSpec,
    deadlines: &DeadlinesSpec,
) -> Result<u64> {
    let components = [
        ("connect_timeout_ms", deadlines.connect_timeout_ms),
        ("readiness_timeout_ms", deadlines.readiness_timeout_ms),
        (
            "token_count_request_timeout_ms",
            deadlines.token_count_request_timeout_ms,
        ),
        (
            "inference_request_timeout_ms",
            deadlines.inference_request_timeout_ms,
        ),
        ("non_request_margin_ms", deadlines.non_request_margin_ms),
    ];
    if let Some((name, _)) = components.iter().find(|(_, value)| *value == 0) {
        return Err(invalid(&format!(
            "V3 deadline component {name} must be positive"
        )));
    }
    if deadlines.connect_timeout_ms
        > deadlines
            .readiness_timeout_ms
            .min(deadlines.token_count_request_timeout_ms)
            .min(deadlines.inference_request_timeout_ms)
    {
        return Err(invalid(
            "V3 connect timeout exceeds a complete contact bound",
        ));
    }
    let overflow = || invalid("V3 supervisor deadline derivation overflows");
    let term = |count: u32, timeout_ms: u64| u64::from(count).checked_mul(timeout_ms);
    [
        term(limits.readiness_contacts, deadlines.readiness_timeout_ms),
        term(
            limits.token_count_contacts,
            deadlines.token_count_request_timeout_ms,
        ),
        term(
            limits.inference_requests,
            deadlines.inference_request_timeout_ms,
        ),
        Some(deadlines.non_request_margin_ms),
    ]
    .into_iter()
    .try_fold(0_u64, |total, term| total.checked_add(term?))
    .ok_or_else(overflow)
}

/// Validate the recorded supervisor deadline against its components and
/// return it. A value that differs from the derivation in either direction is
/// rejected, so no identity can supply an independent supervisor timeout.
pub fn validate_supervisor_deadline(limits: &LimitsSpec, deadlines: &DeadlinesSpec) -> Result<u64> {
    let derived = derive_supervisor_deadline_ms(limits, deadlines)?;
    if deadlines.supervisor_deadline_ms != derived {
        return Err(invalid(&format!(
            "V3 supervisor deadline {} differs from the derived child-lifecycle bound {derived}",
            deadlines.supervisor_deadline_ms
        )));
    }
    Ok(derived)
}

/// The single data structure that parameterizes the V3 feasibility gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct V3GateSpec {
    pub gate_id: String,
    pub contract_path: String,
    pub contract_sha256: String,
    pub runtime_version: String,
    pub runtime_executable: FileIdentitySpec,
    pub model_file: FileIdentitySpec,
    pub launch_arguments: Vec<String>,
    pub forbidden_launch_arguments: Vec<String>,
    pub forbidden_environment_prefix: String,
    pub cases: Vec<CaseSpec>,
    pub limits: LimitsSpec,
    pub deadlines: DeadlinesSpec,
    pub evidence_root: String,
    pub token_count_endpoint: String,
    pub inference_endpoint: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextDecision {
    Fits,
    Bound,
}

pub(crate) fn invalid(message: &str) -> ReasoningBudgetCalibrationError {
    ReasoningBudgetCalibrationError::Validation(message.to_string())
}

pub(crate) fn field<'a>(value: &'a Value, pointer: &str) -> Result<&'a Value> {
    value
        .pointer(pointer)
        .ok_or_else(|| invalid(&format!("V3 value is missing {pointer}")))
}

pub(crate) fn text(value: &Value, pointer: &str) -> Result<String> {
    field(value, pointer)?
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| invalid(&format!("V3 value {pointer} is not a string")))
}

pub(crate) fn number(value: &Value, pointer: &str) -> Result<u64> {
    field(value, pointer)?
        .as_u64()
        .ok_or_else(|| invalid(&format!("V3 value {pointer} is not an unsigned integer")))
}

pub(crate) fn expect(value: &Value, pointer: &str, expected: Value) -> Result<()> {
    if field(value, pointer)? != &expected {
        return Err(invalid(&format!("V3 contract value {pointer} changed")));
    }
    Ok(())
}

/// The exact launch arguments registered for a model path.
pub fn v3_launch_arguments(model_path: &str) -> Vec<String> {
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
        "on",
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

/// Validate the tracked V3 runtime contract against the accepted design.
pub fn validate_v3_contract(contract: &Value) -> Result<()> {
    expect(
        contract,
        "/contract_version",
        json!("phase1c-scored-runtime-local-qwen-v3"),
    )?;
    expect(
        contract,
        "/status",
        json!("PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED"),
    )?;
    expect(contract, "/runtime/reasoning", json!("on"))?;
    expect(contract, "/runtime/reasoning_budget_flag", json!("ABSENT"))?;
    expect(contract, "/runtime/context_size", json!(V3_CONTEXT_TOKENS))?;
    expect(contract, "/runtime/parallel_slots", json!(1))?;
    expect(contract, "/runtime/metrics", json!("enabled"))?;
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
        "/runtime/executable/windows_file_id",
        json!(V3_LLAMA_EXECUTABLE_FILE_ID),
    )?;
    expect(contract, "/generation/max_tokens", json!(V3_MAX_TOKENS))?;
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
        json!(V3_CONTEXT_TOKENS),
    )?;
    expect(
        contract,
        "/context_budget/output_ceiling_tokens",
        json!(V3_MAX_TOKENS),
    )?;
    expect(
        contract,
        "/context_budget/token_counter_is_inference",
        json!(false),
    )?;
    expect(contract, "/timeout_policy/connect_timeout_ms", json!(1000))?;
    expect(
        contract,
        "/timeout_policy/readiness_timeout_ms",
        json!(1000),
    )?;
    expect(
        contract,
        "/timeout_policy/token_count_request_timeout_ms",
        json!(60_000),
    )?;
    expect(
        contract,
        "/timeout_policy/inference_request_timeout_ms",
        json!(2_400_000),
    )?;
    expect(
        contract,
        "/timeout_policy/non_request_margin_ms",
        json!(120_000),
    )?;
    if field(contract, "/timeout_policy/complete_request_timeout_ms").is_ok()
        || field(contract, "/timeout_policy/supervisor_timeout_ms").is_ok()
    {
        return Err(invalid(
            "V3 contract carries a superseded single-request timeout field",
        ));
    }
    validate_supervisor_deadline(&gate_limits(), &contract_deadlines(contract)?)?;
    for key in [
        "automatic_retries",
        "fallback_requests",
        "adaptive_replicates",
        "warmup_requests",
    ] {
        expect(contract, &format!("/retry_policy/{key}"), json!(0))?;
    }
    let model_path = text(contract, "/runtime/model/path")?;
    let launch = field(contract, "/runtime/launch_arguments")?;
    if launch != &json!(v3_launch_arguments(&model_path)) {
        return Err(invalid("V3 contract launch arguments changed"));
    }
    let forbidden = field(contract, "/runtime/forbidden_launch_arguments")?
        .as_array()
        .ok_or_else(|| invalid("V3 forbidden launch arguments are missing"))?;
    for required in ["--reasoning-budget", "-hf", "--hf-repo"] {
        if !forbidden.contains(&json!(required)) {
            return Err(invalid(&format!("V3 contract does not forbid {required}")));
        }
    }
    let launch_arguments = launch.as_array().into_iter().flatten();
    for argument in launch_arguments {
        if forbidden.contains(argument) {
            return Err(invalid("V3 launch arguments include a forbidden argument"));
        }
    }
    Ok(())
}

/// The registered gate contact limits.
fn gate_limits() -> LimitsSpec {
    LimitsSpec {
        context_tokens: V3_CONTEXT_TOKENS,
        max_tokens: V3_MAX_TOKENS,
        readiness_contacts: 1,
        token_count_contacts: 3,
        inference_requests: 3,
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

fn file_identity_spec(contract: &Value, pointer: &str) -> Result<FileIdentitySpec> {
    Ok(FileIdentitySpec {
        path: text(contract, &format!("{pointer}/path"))?,
        sha256: text(contract, &format!("{pointer}/sha256"))?,
        file_size: number(contract, &format!("{pointer}/file_size"))?,
        windows_file_id: text(contract, &format!("{pointer}/windows_file_id"))?,
    })
}

/// The three frozen calibration cases rendered as V3 request bodies. Only
/// `max_tokens` differs from the calibration requests.
pub fn gate_requests(manifest: &Value, max_tokens: u64) -> Result<Vec<(String, Value, Value)>> {
    let mut v3_manifest = manifest.clone();
    *v3_manifest
        .pointer_mut("/generation/max_tokens")
        .ok_or_else(|| invalid("manifest generation.max_tokens is missing"))? = json!(max_tokens);
    let cases = manifest
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
            let request = build_request(&v3_manifest, case)?;
            Ok((case_id.to_string(), case.clone(), request))
        })
        .collect()
}

/// Derive the gate spec from the tracked contract and calibration manifest.
/// A loaded gate identity must carry exactly this spec.
pub fn derive_gate_spec(contract: &Value, manifest: &Value) -> Result<V3GateSpec> {
    validate_v3_contract(contract)?;
    let cases = gate_requests(manifest, V3_MAX_TOKENS)?
        .iter()
        .map(|(case_id, _, request)| {
            Ok(CaseSpec {
                case_id: case_id.clone(),
                request_sha256: canonical_hash(request)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let string_list = |pointer: &str| -> Result<Vec<String>> {
        field(contract, pointer)?
            .as_array()
            .ok_or_else(|| invalid(&format!("V3 value {pointer} is not an array")))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| invalid(&format!("V3 value {pointer} holds a non-string")))
            })
            .collect()
    };
    Ok(V3GateSpec {
        gate_id: V3_GATE_ID.to_string(),
        contract_path: V3_CONTRACT_PATH.to_string(),
        contract_sha256: canonical_hash(contract)?,
        runtime_version: text(contract, "/runtime/executable/version")?,
        runtime_executable: file_identity_spec(contract, "/runtime/executable")?,
        model_file: file_identity_spec(contract, "/runtime/model")?,
        launch_arguments: string_list("/runtime/launch_arguments")?,
        forbidden_launch_arguments: string_list("/runtime/forbidden_launch_arguments")?,
        forbidden_environment_prefix: text(contract, "/runtime/forbidden_environment_prefix")?,
        cases,
        limits: gate_limits(),
        deadlines: contract_deadlines(contract)?,
        evidence_root: V3_GATE_EVIDENCE_ROOT.to_string(),
        token_count_endpoint: V3_TOKEN_COUNT_ENDPOINT.to_string(),
        inference_endpoint: ENDPOINT.to_string(),
    })
}

/// A spec is valid only when it equals the contract-derived spec, so no
/// ceiling, budget, case, request, or limit can be selected independently.
pub fn validate_gate_spec(spec: &V3GateSpec, contract: &Value, manifest: &Value) -> Result<()> {
    validate_supervisor_deadline(&spec.limits, &spec.deadlines)?;
    let derived = derive_gate_spec(contract, manifest)?;
    if spec != &derived {
        return Err(invalid(
            "V3 gate spec differs from the contract-derived spec",
        ));
    }
    Ok(())
}

/// Compare a registered file identity with an inspected object.
pub fn compare_file_identity(
    label: &str,
    expected: &FileIdentitySpec,
    actual: &ExecutableIdentity,
) -> Result<()> {
    if actual.sha256 != expected.sha256
        || actual.file_size != expected.file_size
        || actual.file_id.as_deref() != Some(expected.windows_file_id.as_str())
    {
        return Err(invalid(&format!(
            "V3 {label} identity differs from the registered frozen object"
        )));
    }
    Ok(())
}

/// Inspect the frozen llama.cpp executable and GGUF model on disk.
pub fn validate_runtime_objects(spec: &V3GateSpec) -> Result<Value> {
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

/// Reject any environment variable that could inject llama.cpp arguments.
pub fn validate_launch_environment<I>(variables: I, forbidden_prefix: &str) -> Result<()>
where
    I: IntoIterator<Item = (String, String)>,
{
    let injected = variables
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| name.to_ascii_uppercase().starts_with(forbidden_prefix))
        .collect::<Vec<_>>();
    if !injected.is_empty() {
        return Err(invalid(&format!(
            "V3 launch environment contains forbidden variables: {}",
            injected.join(", ")
        )));
    }
    Ok(())
}

/// `input_tokens + max_tokens <= context_tokens`, without overflow.
pub fn context_guard(input_tokens: u64, limits: &LimitsSpec) -> ContextDecision {
    match input_tokens.checked_add(limits.max_tokens) {
        Some(total) if total <= limits.context_tokens => ContextDecision::Fits,
        _ => ContextDecision::Bound,
    }
}

/// Dispatch only when the context guard admits the request.
pub fn guarded_dispatch<T>(
    input_tokens: u64,
    limits: &LimitsSpec,
    dispatch: impl FnOnce() -> T,
) -> std::result::Result<T, &'static str> {
    match context_guard(input_tokens, limits) {
        ContextDecision::Fits => Ok(dispatch()),
        ContextDecision::Bound => Err(INCONCLUSIVE_CONTEXT_BOUND),
    }
}

/// Parse the llama.cpp `input_tokens` response.
pub fn parse_input_tokens(body: &[u8]) -> Result<u64> {
    let value: Value = serde_json::from_slice(body)?;
    if value.get("object").and_then(Value::as_str) != Some("response.input_tokens") {
        return Err(invalid(
            "token-count response object is not response.input_tokens",
        ));
    }
    value
        .get("input_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid("token-count response has no unsigned input_tokens"))
}

/// The pre-registered stopping rule. Only three passing cases with no
/// context-bound result pass the gate; every other outcome closes the path.
pub fn classify_gate(context_bound: bool, case_states: &[&str]) -> &'static str {
    if !context_bound
        && case_states.len() == CALIBRATION_CASE_IDS.len()
        && case_states.iter().all(|state| *state == "PASS")
    {
        V3_FEASIBILITY_PASSED
    } else {
        CURRENT_QWEN_SCORED_PATH_CLOSED
    }
}

/// What a gate classification permits next.
pub fn gate_permission(classification: &str) -> &'static str {
    if classification == V3_FEASIBILITY_PASSED {
        "V3_PILOT_PREPARATION_ONLY"
    } else {
        "DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY"
    }
}

/// The offline plan: no readiness, network, token-count, or inference call.
pub fn dry_run_plan(spec: &V3GateSpec) -> Value {
    json!({
        "state": "V3_FEASIBILITY_GATE_DRY_RUN",
        "gate_id": spec.gate_id,
        "cases": spec.cases,
        "max_tokens": spec.limits.max_tokens,
        "context_tokens": spec.limits.context_tokens,
        "planned_readiness_contacts": spec.limits.readiness_contacts,
        "planned_token_count_contacts": spec.limits.token_count_contacts,
        "planned_inference_requests": spec.limits.inference_requests,
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
    let contract = read_json(V3_CONTRACT_PATH)?;
    let sidecar = read_json(V3_CONTRACT_FINGERPRINT_PATH)?;
    if sidecar.get("canonical_sha256").and_then(Value::as_str)
        != Some(canonical_hash(&contract)?.as_str())
    {
        return Err(invalid("V3 contract fingerprint sidecar mismatch"));
    }
    Ok(contract)
}

pub(crate) fn tracked_manifest() -> Result<Value> {
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    Ok(manifest)
}

/// Offline dry run: contract, derived spec, requests, and plan. With
/// `inspect_runtime_objects`, the frozen executable and GGUF are hashed.
pub fn dry_run_v3_feasibility_gate(inspect_runtime_objects: bool) -> Result<Value> {
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    let spec = derive_gate_spec(&contract, &manifest)?;
    let runtime_objects = if inspect_runtime_objects {
        validate_runtime_objects(&spec)?
    } else {
        Value::Null
    };
    let mut plan = dry_run_plan(&spec);
    plan["derived_spec"] = serde_json::to_value(&spec)?;
    plan["runtime_objects"] = runtime_objects;
    plan["identity_present"] = json!(workspace_path(V3_GATE_IDENTITY_PATH).is_file());
    Ok(plan)
}

/// Freeze the final build outputs into the non-overwriting V3 stage.
pub fn freeze_v3_feasibility_gate() -> Result<Value> {
    let stage = workspace_path(V3_GATE_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid("V3 frozen staging directory already exists"));
    }
    let mut frozen = serde_json::Map::new();
    for (label, binary) in [
        ("supervisor", V3_SUPERVISOR_BINARY),
        ("child", V3_CHILD_BINARY),
    ] {
        let source = workspace_path(&format!("target/debug/{binary}"));
        let source_identity =
            executable_identity::inspect(&source).map_err(|error| invalid(&error))?;
        let copy = executable_identity::freeze_copy(&source, &stage.join(binary))
            .map_err(|error| invalid(&error))?;
        if copy.sha256 != source_identity.sha256 {
            return Err(invalid(
                "V3 frozen executable differs from its build output",
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
pub fn load_gate_identity() -> Result<(Value, V3GateSpec)> {
    let identity = read_json(V3_GATE_IDENTITY_PATH)?;
    let sidecar = read_json(V3_GATE_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str) != Some(V3_GATE_IDENTITY_PATH)
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(&identity)?.as_str())
    {
        return Err(invalid("V3 gate identity fingerprint sidecar mismatch"));
    }
    validate_gate_identity_document(&identity)?;
    let spec: V3GateSpec = serde_json::from_value(field(&identity, "/spec")?.clone())?;
    Ok((identity, spec))
}

/// Document-level checks that do not read other files.
pub fn validate_gate_identity_document(identity: &Value) -> Result<()> {
    if !text(identity, "/identity_version")?.starts_with("phase1c-v3-feasibility-gate-") {
        return Err(invalid("V3 gate identity version is invalid"));
    }
    expect(
        identity,
        "/status",
        json!("PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED"),
    )?;
    if !matches!(number(identity, "/gate_identity_number")?, 1 | 2) {
        return Err(invalid("V3 gate identity number must be 1 or 2"));
    }
    let sources = field(identity, "/implementation_sources")?
        .as_array()
        .ok_or_else(|| invalid("V3 implementation sources are missing"))?;
    for required in V3_BOUND_SOURCES {
        if !sources
            .iter()
            .any(|source| source.get("path").and_then(Value::as_str) == Some(required))
        {
            return Err(invalid(&format!(
                "V3 identity does not bind source {required}"
            )));
        }
    }
    let binding =
        executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(identity)
            .map_err(|error| invalid(&error))?
            .ok_or_else(|| invalid("V3 identity is missing frozen executable identity"))?;
    for object in [&binding.supervisor, &binding.child] {
        let raw = object.raw_path.to_ascii_lowercase().replace('/', "\\");
        let stage = V3_GATE_FROZEN_STAGE_ROOT.replace('/', "\\");
        if raw.contains("target\\debug\\") || !raw.contains(&stage) || object.file_id.is_none() {
            return Err(invalid(
                "V3 frozen executable is not a complete staged object",
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

pub(crate) fn validate_source_binding(identity: &Value) -> Result<()> {
    for source in field(identity, "/implementation_sources")?
        .as_array()
        .into_iter()
        .flatten()
    {
        let path = text(source, "/path")?;
        if text(source, "/sha256")? != source_sha256(&path)? {
            return Err(invalid(&format!(
                "V3 source {path} does not match current source"
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_frozen_executables(identity: &Value) -> Result<Value> {
    let binding =
        executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(identity)
            .map_err(|error| invalid(&error))?
            .ok_or_else(|| invalid("V3 identity is missing frozen executable identity"))?;
    let supervisor = executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
        .map_err(|error| invalid(&error))?;
    let child = executable_identity::inspect(Path::new(&binding.child.raw_path))
        .map_err(|error| invalid(&error))?;
    executable_identity::validate_frozen_executable_binding(&binding, &supervisor, &child)
        .map_err(|error| invalid(&error))?;
    Ok(json!({ "supervisor": supervisor, "child": child }))
}

/// Offline preflight before the operator starts the server.
pub fn preflight_v3_feasibility_gate() -> Result<Value> {
    let (identity, spec) = load_gate_identity()?;
    validate_source_binding(&identity)?;
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    validate_gate_spec(&spec, &contract, &manifest)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(V3_GATE_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    let frozen = validate_frozen_executables(&identity)?;
    let runtime_objects = validate_runtime_objects(&spec)?;
    validate_launch_environment(std::env::vars(), &spec.forbidden_environment_prefix)?;
    let certification = validate_accepted_workflow_certification_v2()?;
    if workspace_path(&spec.evidence_root).exists() {
        return Err(invalid("V3 gate evidence root already exists"));
    }
    let os_inspection = windows_native_prestart_value(0, true);
    if os_inspection["state"] != "READY" || os_inspection["outcome"] != "EXCLUSIVE_PRESTART" {
        return Err(invalid(
            "V3 preflight did not establish exclusive pre-start state",
        ));
    }
    Ok(json!({
        "state": "V3_FEASIBILITY_GATE_PREPARED",
        "execution_state": "V3_FEASIBILITY_GATE_NOT_EXECUTED",
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
    spec: V3GateSpec,
    manifest: Value,
    requests: Vec<(String, Value, Value)>,
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
        Path::new(V3_GATE_IDENTITY_PATH),
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
            "V3 supervisor launch metadata does not match the registered gate identity",
        ));
    }
    validate_source_binding(&identity)?;
    let frozen = validate_frozen_executables(&identity)?;
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let contract = tracked_contract()?;
    let manifest = tracked_manifest()?;
    validate_gate_spec(&spec, &contract, &manifest)?;
    let runtime_objects = validate_runtime_objects(&spec)?;
    validate_launch_environment(std::env::vars(), &spec.forbidden_environment_prefix)?;
    let requests = gate_requests(&manifest, spec.limits.max_tokens)?;
    for ((case_id, _, request), registered_case) in requests.iter().zip(&spec.cases) {
        if case_id != &registered_case.case_id
            || canonical_hash(request)? != registered_case.request_sha256
        {
            return Err(invalid(
                "V3 gate request differs from the registered request",
            ));
        }
    }
    if workspace_path(&spec.evidence_root).exists() {
        return Err(invalid("V3 gate evidence root already exists"));
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
        manifest,
        requests,
        report,
    })
}

/// Offline traversal under the frozen supervisor; stops before post-start
/// ownership inspection, readiness, token counting, and inference.
pub fn validate_v3_live_prerequisites() -> Result<Value> {
    let mut report = gate_prerequisites()?.report;
    report["state"] = json!("READY_FOR_MODEL_READINESS_BOUNDARY");
    report["READY_FOR_MODEL_READINESS_BOUNDARY"] = json!(true);
    report["stopped_before"] = json!([
        "post-start runtime ownership inspection",
        "tcp listener readiness",
        "token-count request",
        "inference"
    ]);
    report["model_server_startups"] = json!(0);
    report["readiness_contacts"] = json!(0);
    Ok(report)
}

pub(crate) fn poststart_ownership() -> Result<Value> {
    let processes = windows_exclusivity::process_table()
        .map_err(|failure| invalid(&format!("post-start process inspection failed: {failure}")))?;
    let llama_processes = windows_exclusivity::llama_processes(&processes);
    if llama_processes.len() != 1 {
        return Err(invalid(&format!(
            "post-start expected exactly one llama.exe process, found {}",
            llama_processes.len()
        )));
    }
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    attempt_003_runtime_ownership_with_expected_workflow(llama_processes[0].pid, &expected_workflow)
}

pub(crate) fn count_input_tokens(
    client: &Client,
    endpoint: &str,
    timeout: Duration,
    request: &Value,
) -> Result<(u64, Value)> {
    let request_bytes = serde_json::to_vec(request)?;
    let started = Instant::now();
    let response = client
        .post(endpoint)
        .timeout(timeout)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_bytes)
        .send()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;
    let status = response.status().as_u16();
    let body = response
        .bytes()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;
    if status != 200 {
        return Err(invalid(&format!(
            "token-count endpoint returned HTTP {status}"
        )));
    }
    let input_tokens = parse_input_tokens(&body)?;
    Ok((
        input_tokens,
        json!({
            "http_status": status,
            "timeout_ms": timeout.as_millis() as u64,
            "elapsed_ms": started.elapsed().as_millis() as u64,
            "response_sha256": crate::hashing::sha256_hex(&body),
            "input_tokens": input_tokens
        }),
    ))
}

fn gate_result(
    prerequisites: &GatePrerequisites,
    classification: &str,
    detail: Value,
    token_count_contacts: usize,
    inference_requests: usize,
) -> Result<Value> {
    Ok(json!({
        "schema_id": "prefixity.phase1c.v3-feasibility-gate-result",
        "schema_version": 1,
        "gate_id": prerequisites.spec.gate_id,
        "identity_sha256": prerequisites.identity_sha256,
        "classification": classification,
        "permits": gate_permission(classification),
        "detail": detail,
        "accounting": {
            "readiness_contacts": 1,
            "token_count_contacts": token_count_contacts,
            "inference_requests": inference_requests,
            "retry_requests": 0,
            "fallback_requests": 0,
            "adaptive_replicates": 0,
            "warmup_requests": 0
        },
        "recorded_at_unix_ms": now_unix_ms()?
    }))
}

/// Live entry point. It is not called by preparation.
pub fn execute_v3_feasibility_gate() -> Result<Value> {
    let prerequisites = gate_prerequisites()?;
    let ownership = poststart_ownership()?;
    let spec = &prerequisites.spec;
    let root = workspace_path(&spec.evidence_root);
    if root.exists() {
        return Err(invalid("V3 gate evidence root already exists"));
    }
    fs::create_dir_all(&root)?;
    write_json(
        &root.join("preflight.json"),
        &json!({
            "live_prerequisites": prerequisites.report,
            "runtime_ownership": ownership
        }),
    )?;

    let started = Instant::now();
    let readiness = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], PORT)),
        Duration::from_millis(spec.deadlines.readiness_timeout_ms),
    );
    let readiness_record = json!({
        "check": "tcp_listener_connect",
        "listener_check_attempts": 1,
        "timeout_ms": spec.deadlines.readiness_timeout_ms,
        "elapsed_ms": started.elapsed().as_millis() as u64,
        "passed": readiness.is_ok(),
        "error": readiness.as_ref().err().map(ToString::to_string),
        "inference_requests": 0
    });
    write_json(&root.join("readiness.json"), &readiness_record)?;
    if readiness.is_err() {
        let result = gate_result(
            &prerequisites,
            GATE_PRE_INFERENCE_FAILURE,
            json!({ "reason": "readiness check failed" }),
            0,
            0,
        )?;
        write_json(&root.join("gate-result.json"), &result)?;
        return Ok(result);
    }

    // The client default is the inference bound; token counting overrides it
    // per request with its own bound and never inherits the generation bound.
    let client = Client::builder()
        .connect_timeout(Duration::from_millis(spec.deadlines.connect_timeout_ms))
        .timeout(Duration::from_millis(
            spec.deadlines.inference_request_timeout_ms,
        ))
        .redirect(Policy::none())
        .build()
        .map_err(|error| ReasoningBudgetCalibrationError::Transport(error.to_string()))?;

    let mut counts = Vec::new();
    for (case_id, _, request) in &prerequisites.requests {
        match count_input_tokens(
            &client,
            &spec.token_count_endpoint,
            Duration::from_millis(spec.deadlines.token_count_request_timeout_ms),
            request,
        ) {
            Ok((input_tokens, record)) => counts.push(json!({
                "case_id": case_id,
                "record": record,
                "fits": context_guard(input_tokens, &spec.limits) == ContextDecision::Fits,
                "input_tokens": input_tokens
            })),
            Err(error) => {
                write_json(&root.join("token-counts.json"), &json!(counts))?;
                let result = gate_result(
                    &prerequisites,
                    GATE_PRE_INFERENCE_FAILURE,
                    json!({ "reason": "token-count request failed", "case_id": case_id, "error": error.to_string() }),
                    counts.len() + 1,
                    0,
                )?;
                write_json(&root.join("gate-result.json"), &result)?;
                return Ok(result);
            }
        }
    }
    write_json(&root.join("token-counts.json"), &json!(counts))?;
    if counts.iter().any(|count| count["fits"] != true) {
        let result = gate_result(
            &prerequisites,
            CURRENT_QWEN_SCORED_PATH_CLOSED,
            json!({ "context_result": INCONCLUSIVE_CONTEXT_BOUND, "token_counts": counts }),
            counts.len(),
            0,
        )?;
        write_json(&root.join("gate-result.json"), &result)?;
        return Ok(result);
    }

    let mut v3_manifest = prerequisites.manifest.clone();
    v3_manifest["generation"]["max_tokens"] = json!(spec.limits.max_tokens);
    let mut results = Vec::new();
    for ((case_id, case, _), count) in prerequisites.requests.iter().zip(&counts) {
        let input_tokens = count["input_tokens"].as_u64().unwrap_or(u64::MAX);
        let case_dir = root.join(case_id);
        fs::create_dir_all(&case_dir)?;
        let outcome = guarded_dispatch(input_tokens, &spec.limits, || {
            execute_case(&v3_manifest, case, None, &case_dir, &client)
        })
        .map_err(invalid)??;
        results.push(outcome);
    }
    let states = results
        .iter()
        .map(|result| {
            result
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("INCONCLUSIVE")
        })
        .collect::<Vec<_>>();
    let classification = classify_gate(false, &states);
    let result = gate_result(
        &prerequisites,
        classification,
        json!({
            "aggregate_state": aggregate_state(&results),
            "case_states": states,
            "token_counts": counts
        }),
        counts.len(),
        results.len(),
    )?;
    write_json(&root.join("gate-result.json"), &result)?;
    write_bytes(
        &root.join("gate-result.sha256"),
        format!("{}  gate-result.json\n", canonical_hash(&result)?).as_bytes(),
    )?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> Value {
        read_json(V3_CONTRACT_PATH).unwrap()
    }

    fn manifest() -> Value {
        tracked_manifest().unwrap()
    }

    fn spec() -> V3GateSpec {
        derive_gate_spec(&contract(), &manifest()).unwrap()
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
    fn tracked_v3_contract_is_exact_and_sealed() {
        validate_v3_contract(&contract()).unwrap();
        tracked_contract().unwrap();
        let spec = spec();
        assert_eq!(spec.limits.max_tokens, 4096);
        assert_eq!(spec.limits.context_tokens, 8192);
        assert_eq!(spec.deadlines.connect_timeout_ms, 1_000);
        assert_eq!(spec.deadlines.readiness_timeout_ms, 1_000);
        assert_eq!(spec.deadlines.token_count_request_timeout_ms, 60_000);
        assert_eq!(spec.deadlines.inference_request_timeout_ms, 2_400_000);
        assert_eq!(spec.deadlines.non_request_margin_ms, 120_000);
        // 1 * 1000 + 3 * 60000 + 3 * 2400000 + 120000
        assert_eq!(spec.deadlines.supervisor_deadline_ms, 7_501_000);
        assert_eq!(
            spec.model_file.sha256,
            "57d1997790d1744fba5b40a7317df71ea5e2acee28c47e78f0cce39c0703f8cf"
        );
    }

    #[test]
    fn v3_executable_identity_is_bound_to_its_containing_volume() {
        use crate::phase1c_executable_identity::file_id_volume;

        let spec = spec();
        assert_eq!(
            spec.runtime_executable.windows_file_id,
            V3_LLAMA_EXECUTABLE_FILE_ID
        );
        assert_eq!(
            file_id_volume(&spec.runtime_executable.windows_file_id),
            Some(0xc4c9_3b54)
        );
        assert!(spec.runtime_executable.path.starts_with("C:\\"));
        assert!(spec.model_file.path.starts_with("D:\\"));
        assert_ne!(
            file_id_volume(&spec.runtime_executable.windows_file_id),
            file_id_volume(&spec.model_file.windows_file_id)
        );

        // The historical Attempt 009-011 value is rejected.
        let mut historical = contract();
        historical["runtime"]["executable"]["windows_file_id"] =
            json!("volume=ba2f80f4;index=00060000001ea970");
        assert!(validate_v3_contract(&historical).is_err());
        assert!(derive_gate_spec(&historical, &manifest()).is_err());

        // On a host that holds the frozen objects, the recorded volume must
        // be the volume that actually contains each object, and the helper
        // must reproduce the recorded executable identity exactly. The model
        // is not hashed here; only its containing volume is compared.
        #[cfg(windows)]
        {
            use crate::phase1c_executable_identity::containing_volume_serial;
            let executable = Path::new(&spec.runtime_executable.path);
            let model = Path::new(&spec.model_file.path);
            if executable.is_file() && model.is_file() {
                assert_eq!(
                    file_id_volume(&spec.runtime_executable.windows_file_id),
                    Some(containing_volume_serial(executable).unwrap())
                );
                assert_eq!(
                    file_id_volume(&spec.model_file.windows_file_id),
                    Some(containing_volume_serial(model).unwrap())
                );
                let inspected = executable_identity::inspect(executable).unwrap();
                compare_file_identity("llama.cpp executable", &spec.runtime_executable, &inspected)
                    .unwrap();
            } else {
                eprintln!("frozen V3 runtime objects absent; containing-volume check skipped");
            }
        }
    }

    #[test]
    fn gate_identity_must_bind_every_v3_source_including_executable_identity() {
        let executable_identity_source =
            "crates/prefixity-controlled-benchmark/src/phase1c_executable_identity.rs";
        assert!(V3_BOUND_SOURCES.contains(&executable_identity_source));
        let sources = V3_BOUND_SOURCES
            .iter()
            .map(|path| json!({ "path": path, "sha256": source_sha256(path).unwrap() }))
            .collect::<Vec<_>>();
        let staged = |name: &str| {
            json!({
                "raw_path": format!("D:/frozen/{V3_GATE_FROZEN_STAGE_ROOT}/{name}"),
                "final_path": format!("D:/frozen/{V3_GATE_FROZEN_STAGE_ROOT}/{name}"),
                "file_size": 1,
                "sha256": "a".repeat(64),
                "file_id": "volume=ba2f80f4;index=0000000000000001"
            })
        };
        let identity = json!({
            "identity_version": "phase1c-v3-feasibility-gate-v1",
            "status": "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
            "gate_identity_number": 1,
            "implementation_sources": sources,
            "implementation_fingerprints": {
                "supervisor_binary": staged(V3_SUPERVISOR_BINARY),
                "child_binary": staged(V3_CHILD_BINARY)
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

        let mut unbound = identity.clone();
        unbound["implementation_sources"]
            .as_array_mut()
            .unwrap()
            .retain(|source| source["path"] != executable_identity_source);
        let error = validate_gate_identity_document(&unbound)
            .unwrap_err()
            .to_string();
        assert!(error.contains(executable_identity_source), "{error}");

        let mut stale = identity.clone();
        for source in stale["implementation_sources"].as_array_mut().unwrap() {
            if source["path"] == executable_identity_source {
                source["sha256"] = json!("0".repeat(64));
            }
        }
        assert!(validate_source_binding(&stale).is_err());
    }

    #[test]
    fn every_deadline_component_changes_the_derived_supervisor_bound() {
        let spec = spec();
        let base = derive_supervisor_deadline_ms(&spec.limits, &spec.deadlines).unwrap();
        assert_eq!(base, 7_501_000);
        let deadline_changes: [fn(&mut DeadlinesSpec); 4] = [
            |d| d.readiness_timeout_ms += 1,
            |d| d.token_count_request_timeout_ms += 1,
            |d| d.inference_request_timeout_ms += 1,
            |d| d.non_request_margin_ms += 1,
        ];
        for change in deadline_changes {
            let mut deadlines = spec.deadlines.clone();
            change(&mut deadlines);
            assert!(derive_supervisor_deadline_ms(&spec.limits, &deadlines).unwrap() > base);
        }
        let limit_changes: [fn(&mut LimitsSpec); 3] = [
            |l| l.readiness_contacts += 1,
            |l| l.token_count_contacts += 1,
            |l| l.inference_requests += 1,
        ];
        for change in limit_changes {
            let mut limits = spec.limits.clone();
            change(&mut limits);
            assert!(derive_supervisor_deadline_ms(&limits, &spec.deadlines).unwrap() > base);
        }
    }

    #[test]
    fn supervisor_bound_is_never_shorter_than_the_child_maximum() {
        let spec = spec();
        let (limits, deadlines) = (&spec.limits, &spec.deadlines);
        let child_request_maximum = u64::from(limits.readiness_contacts)
            * deadlines.readiness_timeout_ms
            + u64::from(limits.token_count_contacts) * deadlines.token_count_request_timeout_ms
            + u64::from(limits.inference_requests) * deadlines.inference_request_timeout_ms;
        let child_maximum = child_request_maximum + deadlines.non_request_margin_ms;
        assert_eq!(child_request_maximum, 7_381_000);
        assert_eq!(
            validate_supervisor_deadline(limits, deadlines).unwrap(),
            child_maximum
        );
        assert!(deadlines.supervisor_deadline_ms >= child_maximum);

        let mut shorter = deadlines.clone();
        shorter.supervisor_deadline_ms = child_maximum - 1;
        assert!(validate_supervisor_deadline(limits, &shorter).is_err());
        // The rejected single-request deadline no longer fits.
        shorter.supervisor_deadline_ms = 2_520_000;
        assert!(validate_supervisor_deadline(limits, &shorter).is_err());

        let mut no_margin = deadlines.clone();
        no_margin.non_request_margin_ms = 0;
        assert!(derive_supervisor_deadline_ms(limits, &no_margin).is_err());
        let mut slow_connect = deadlines.clone();
        slow_connect.connect_timeout_ms = deadlines.readiness_timeout_ms + 1;
        assert!(derive_supervisor_deadline_ms(limits, &slow_connect).is_err());
        let mut overflow = deadlines.clone();
        overflow.inference_request_timeout_ms = u64::MAX;
        assert!(derive_supervisor_deadline_ms(limits, &overflow).is_err());
    }

    #[test]
    fn independent_supervisor_timeout_is_rejected() {
        let contract = contract();
        let manifest = manifest();
        let spec = spec();

        let mut longer = spec.clone();
        longer.deadlines.supervisor_deadline_ms += 1;
        assert!(validate_gate_spec(&longer, &contract, &manifest).is_err());

        // Consistent with its own components but not with the contract.
        let mut rederived = spec.clone();
        rederived.deadlines.token_count_request_timeout_ms = 2_400_000;
        rederived.deadlines.supervisor_deadline_ms =
            derive_supervisor_deadline_ms(&rederived.limits, &rederived.deadlines).unwrap();
        assert!(validate_gate_spec(&rederived, &contract, &manifest).is_err());

        let mut contract_deadline = contract.clone();
        contract_deadline["timeout_policy"]["supervisor_deadline_ms"] = json!(2_520_000);
        assert!(validate_v3_contract(&contract_deadline).is_err());
        assert!(derive_gate_spec(&contract_deadline, &manifest).is_err());

        let mut legacy = contract.clone();
        legacy["timeout_policy"]["supervisor_timeout_ms"] = json!(7_501_000);
        assert!(validate_v3_contract(&legacy).is_err());

        let mut identity_spec = serde_json::to_value(&spec).unwrap();
        identity_spec["deadlines"]["supervisor_timeout_ms"] = json!(7_501_000);
        assert!(serde_json::from_value::<V3GateSpec>(identity_spec).is_err());
    }

    #[test]
    fn reasoning_budget_flag_is_absent_and_cannot_be_added() {
        let spec = spec();
        assert!(!spec
            .launch_arguments
            .iter()
            .any(|argument| argument.contains("reasoning-budget")));
        assert!(spec.launch_arguments.contains(&"-m".to_string()));
        assert!(!spec.launch_arguments.contains(&"-hf".to_string()));

        let mut flagged = contract();
        flagged["runtime"]["reasoning_budget_flag"] = json!("PRESENT");
        assert!(validate_v3_contract(&flagged).is_err());

        let mut budget = contract();
        let arguments = budget["runtime"]["launch_arguments"]
            .as_array_mut()
            .unwrap();
        arguments.push(json!("--reasoning-budget"));
        arguments.push(json!("256"));
        assert!(validate_v3_contract(&budget).is_err());

        let mut hf = contract();
        hf["runtime"]["launch_arguments"][1] = json!("-hf");
        assert!(validate_v3_contract(&hf).is_err());
    }

    #[test]
    fn wrong_executable_or_model_identity_is_rejected() {
        let spec = spec();
        let exe = &spec.runtime_executable;
        let model = &spec.model_file;
        compare_file_identity(
            "exe",
            exe,
            &identity(&exe.sha256, exe.file_size, Some(&exe.windows_file_id)),
        )
        .unwrap();
        assert!(compare_file_identity(
            "exe",
            exe,
            &identity(&model.sha256, exe.file_size, Some(&exe.windows_file_id))
        )
        .is_err());
        assert!(compare_file_identity(
            "model",
            model,
            &identity(&exe.sha256, model.file_size, Some(&model.windows_file_id))
        )
        .is_err());
        assert!(compare_file_identity(
            "model",
            model,
            &identity(&model.sha256, model.file_size, Some("volume=0;index=1"))
        )
        .is_err());
        assert!(compare_file_identity(
            "model",
            model,
            &identity(&model.sha256, model.file_size, None)
        )
        .is_err());
    }

    #[test]
    fn changed_model_bytes_are_rejected() {
        let directory =
            std::env::temp_dir().join(format!("prefixity-v3-gguf-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("model.gguf");
        fs::write(&path, b"GGUF frozen bytes").unwrap();
        let frozen = executable_identity::inspect(&path).unwrap();
        let expected = FileIdentitySpec {
            path: path.to_string_lossy().into_owned(),
            sha256: frozen.sha256.clone(),
            file_size: frozen.file_size,
            windows_file_id: frozen.file_id.clone().unwrap_or_default(),
        };
        fs::write(&path, b"GGUF changed bytes").unwrap();
        let changed = executable_identity::inspect(&path).unwrap();
        assert!(compare_file_identity("model", &expected, &changed).is_err());
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn changed_request_or_ceiling_is_rejected() {
        let contract = contract();
        let manifest = manifest();
        let spec = spec();
        validate_gate_spec(&spec, &contract, &manifest).unwrap();

        let mut request = spec.clone();
        request.cases[1].request_sha256 = "0".repeat(64);
        assert!(validate_gate_spec(&request, &contract, &manifest).is_err());

        let mut ceiling = spec.clone();
        ceiling.limits.max_tokens = 8192;
        assert!(validate_gate_spec(&ceiling, &contract, &manifest).is_err());

        let mut contract_ceiling = contract.clone();
        contract_ceiling["generation"]["max_tokens"] = json!(2048);
        assert!(derive_gate_spec(&contract_ceiling, &manifest).is_err());

        let mut launch_ceiling = spec.clone();
        launch_ceiling.launch_arguments[4] = "16384".to_string();
        assert!(validate_gate_spec(&launch_ceiling, &contract, &manifest).is_err());
    }

    #[test]
    fn gate_requests_differ_from_calibration_only_in_max_tokens() {
        let manifest = manifest();
        let requests = gate_requests(&manifest, V3_MAX_TOKENS).unwrap();
        assert_eq!(requests.len(), 3);
        for (case_id, case, request) in &requests {
            let mut original = build_request(&manifest, case).unwrap();
            assert_eq!(original["max_tokens"], 2048);
            assert_eq!(request["max_tokens"], 4096);
            original["max_tokens"] = json!(4096);
            assert_eq!(&original, request, "{case_id}");
            assert!(request.get("reasoning_budget").is_none());
        }
    }

    #[test]
    fn context_guard_admits_fitting_and_rejects_overflow_before_transport() {
        let limits = spec().limits;
        assert_eq!(context_guard(339, &limits), ContextDecision::Fits);
        assert_eq!(context_guard(4096, &limits), ContextDecision::Fits);
        assert_eq!(context_guard(4097, &limits), ContextDecision::Bound);
        assert_eq!(context_guard(u64::MAX, &limits), ContextDecision::Bound);

        let mut dispatched = false;
        assert_eq!(
            guarded_dispatch(4097, &limits, || dispatched = true),
            Err(INCONCLUSIVE_CONTEXT_BOUND)
        );
        assert!(!dispatched);
        guarded_dispatch(339, &limits, || dispatched = true).unwrap();
        assert!(dispatched);
    }

    #[test]
    fn token_count_response_is_parsed_strictly() {
        assert_eq!(
            parse_input_tokens(br#"{"input_tokens":339,"object":"response.input_tokens"}"#)
                .unwrap(),
            339
        );
        assert!(parse_input_tokens(br#"{"input_tokens":339}"#).is_err());
        assert!(
            parse_input_tokens(br#"{"input_tokens":-1,"object":"response.input_tokens"}"#).is_err()
        );
        assert!(parse_input_tokens(b"not json").is_err());
    }

    #[test]
    fn dry_run_makes_no_contacts_and_plans_exactly_three_requests() {
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
        assert_eq!(plan["planned_readiness_contacts"], 1);
        assert_eq!(plan["planned_token_count_contacts"], 3);
        assert_eq!(plan["planned_inference_requests"], 3);
        assert_eq!(plan["cases"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn retry_fallback_and_warmup_remain_zero() {
        let limits = spec().limits;
        assert_eq!(limits.retry_requests, 0);
        assert_eq!(limits.fallback_requests, 0);
        assert_eq!(limits.adaptive_replicates, 0);
        assert_eq!(limits.warmup_requests, 0);
        let mut retrying = contract();
        retrying["retry_policy"]["automatic_retries"] = json!(1);
        assert!(validate_v3_contract(&retrying).is_err());
    }

    #[test]
    fn launch_environment_rejects_injected_llama_arguments() {
        let clean = vec![("LLAMA_CACHE".to_string(), "D:\\models".to_string())];
        validate_launch_environment(clean, "LLAMA_ARG_").unwrap();
        let injected = vec![("LLAMA_ARG_REASONING_BUDGET".to_string(), "256".to_string())];
        assert!(validate_launch_environment(injected, "LLAMA_ARG_").is_err());
    }

    #[test]
    fn gate_pass_permits_pilot_preparation_only() {
        let classification = classify_gate(false, &["PASS", "PASS", "PASS"]);
        assert_eq!(classification, V3_FEASIBILITY_PASSED);
        assert_eq!(gate_permission(classification), "V3_PILOT_PREPARATION_ONLY");
    }

    #[test]
    fn any_model_output_failure_closes_the_current_qwen_path() {
        for states in [
            vec!["FAIL", "PASS", "PASS"],
            vec!["PASS", "INCONCLUSIVE", "PASS"],
            vec!["PASS", "PASS", "FAIL"],
            vec!["PASS", "PASS"],
        ] {
            let classification = classify_gate(false, &states);
            assert_eq!(
                classification, CURRENT_QWEN_SCORED_PATH_CLOSED,
                "{states:?}"
            );
            assert_eq!(
                gate_permission(classification),
                "DIFFERENT_CAPABLE_MODEL_DESIGN_REVIEW_ONLY"
            );
        }
        assert_eq!(
            classify_gate(true, &["PASS", "PASS", "PASS"]),
            CURRENT_QWEN_SCORED_PATH_CLOSED
        );
    }

    #[test]
    fn live_prerequisites_require_supervisor_handoff() {
        let error = validate_v3_live_prerequisites().unwrap_err().to_string();
        assert!(error.contains("expected workflow launch metadata was not handed off"));
    }
}
