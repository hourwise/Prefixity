//! Experiment-only outer supervisor for the Phase 1C scored runtime.
//!
//! The supervisor launches an already-built child runner. It never starts
//! llama.cpp, opens a socket, sends HTTP, retries, or selects another arm.

use crate::hashing::canonical_hash;
use crate::phase1c_executable_identity::{
    inspect as inspect_executable, validate_frozen_executable_binding, ExecutableIdentity,
    FrozenExecutableBinding,
};
use crate::phase1c_h001::H001Error;
#[cfg(test)]
use crate::phase1c_h001_v2::{parse_v2_cli_args, v2_live_child_args, V2CliCommand};
use crate::phase1c_v3_feasibility::{validate_supervisor_deadline, DeadlinesSpec, LimitsSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const PRODUCTION_SUPERVISOR_TIMEOUT_MS: u64 = 1_320_000;
pub const WORKFLOW_HANDOFF_ENV: &str = "PREFIXITY_PHASE1C_WORKFLOW_HANDOFF";
pub const WORKFLOW_HANDOFF_SCHEMA_ID: &str = "prefixity.phase1c.workflow-launch-metadata";
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredWorkflowIdentity {
    pub attempt_identity_path: String,
    pub attempt_identity_sha256: String,
    pub attempt: u32,
    pub candidate_budget: u32,
    pub candidate_identity: String,
    pub evidence_root: String,
    /// Complete preparation-time binding. Historical identities may omit it,
    /// but the live registered-workflow path rejects that omission.
    pub frozen_executable_binding: Option<FrozenExecutableBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowLaunchMetadata {
    pub schema_id: String,
    pub attempt_identity_path: String,
    pub attempt_identity_sha256: String,
    pub attempt: u32,
    pub launch_identity: String,
    pub supervisor_pid: u32,
    pub supervisor_path: String,
    pub supervisor_executable_identity: ExecutableIdentity,
    pub child_binding: String,
    pub child_executable_path: String,
    pub child_executable_identity: ExecutableIdentity,
    pub candidate_budget: u32,
    pub candidate_identity: String,
    pub evidence_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_executable_binding: Option<FrozenExecutableBinding>,
}

impl RegisteredWorkflowIdentity {
    pub fn validate(&self) -> Result<(), H001Error> {
        if self.attempt_identity_path.trim().is_empty()
            || self.attempt_identity_sha256.trim().is_empty()
            || self.candidate_identity.trim().is_empty()
            || self.evidence_root.trim().is_empty()
            || self.attempt == 0
            || self.candidate_budget == 0
        {
            return Err(H001Error::Validation(
                "registered workflow identity is incomplete".to_string(),
            ));
        }
        Ok(())
    }

    pub fn generated_launch_identity(&self) -> String {
        format!(
            "phase1c-attempt-{}-budget-{}-{}",
            self.attempt, self.candidate_budget, self.attempt_identity_sha256
        )
    }
}

pub fn build_workflow_launch_metadata(
    registered: &RegisteredWorkflowIdentity,
    child_program: &Path,
) -> Result<WorkflowLaunchMetadata, H001Error> {
    registered.validate()?;
    let supervisor_pid = std::process::id();
    if supervisor_pid == 0 {
        return Err(H001Error::Validation(
            "supervisor PID must be nonzero".to_string(),
        ));
    }
    let supervisor_path = std::env::current_exe()?;
    let child_path = std::fs::canonicalize(child_program)?;
    let supervisor_executable_identity =
        inspect_executable(&supervisor_path).map_err(H001Error::Validation)?;
    let child_executable_identity =
        inspect_executable(&child_path).map_err(H001Error::Validation)?;
    if let Some(expected) = &registered.frozen_executable_binding {
        validate_frozen_executable_binding(
            expected,
            &supervisor_executable_identity,
            &child_executable_identity,
        )
        .map_err(H001Error::Validation)?;
    }
    Ok(WorkflowLaunchMetadata {
        schema_id: WORKFLOW_HANDOFF_SCHEMA_ID.to_string(),
        attempt_identity_path: registered.attempt_identity_path.clone(),
        attempt_identity_sha256: registered.attempt_identity_sha256.clone(),
        attempt: registered.attempt,
        launch_identity: registered.generated_launch_identity(),
        supervisor_pid,
        supervisor_path: supervisor_path.to_string_lossy().into_owned(),
        supervisor_executable_identity,
        child_binding: "parent_pid".to_string(),
        child_executable_path: child_path.to_string_lossy().into_owned(),
        child_executable_identity,
        candidate_budget: registered.candidate_budget,
        candidate_identity: registered.candidate_identity.clone(),
        evidence_root: registered.evidence_root.clone(),
        frozen_executable_binding: registered.frozen_executable_binding.clone(),
    })
}

pub fn workflow_launch_metadata_from_env() -> Result<WorkflowLaunchMetadata, H001Error> {
    let raw = std::env::var(WORKFLOW_HANDOFF_ENV).map_err(|_| {
        H001Error::Validation("expected workflow launch metadata was not handed off".to_string())
    })?;
    let metadata: WorkflowLaunchMetadata = serde_json::from_str(&raw)?;
    if metadata.schema_id != WORKFLOW_HANDOFF_SCHEMA_ID
        || metadata.launch_identity.trim().is_empty()
        || metadata.supervisor_pid == 0
        || metadata.supervisor_path.trim().is_empty()
        || metadata
            .supervisor_executable_identity
            .raw_path
            .trim()
            .is_empty()
        || metadata
            .supervisor_executable_identity
            .final_path
            .trim()
            .is_empty()
        || metadata.child_binding != "parent_pid"
        || metadata.child_executable_path.trim().is_empty()
        || metadata
            .child_executable_identity
            .raw_path
            .trim()
            .is_empty()
        || metadata
            .child_executable_identity
            .final_path
            .trim()
            .is_empty()
        || metadata.attempt == 0
        || metadata.candidate_budget == 0
    {
        return Err(H001Error::Validation(
            "expected workflow launch metadata is invalid".to_string(),
        ));
    }
    if let Some(binding) = &metadata.frozen_executable_binding {
        if binding.supervisor.raw_path.trim().is_empty()
            || binding.supervisor.final_path.trim().is_empty()
            || binding.child.raw_path.trim().is_empty()
            || binding.child.final_path.trim().is_empty()
        {
            return Err(H001Error::Validation(
                "frozen executable binding in workflow launch metadata is invalid".to_string(),
            ));
        }
    }
    Ok(metadata)
}

pub fn registered_workflow_identity_from_file(
    path: &Path,
) -> Result<RegisteredWorkflowIdentity, H001Error> {
    let identity = read_registered_identity(path)?;
    if let Some(kind) = gate_identity_kind(&identity) {
        return registered_gate_identity(kind, path, &identity);
    }
    let attempt = identity
        .pointer("/attempt")
        .and_then(Value::as_u64)
        .ok_or_else(|| H001Error::Validation("registered attempt is missing".to_string()))?
        as u32;
    let candidate_budget = identity
        .pointer("/candidate/reasoning_budget")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            H001Error::Validation("registered candidate budget is missing".to_string())
        })? as u32;
    let candidate_identity = identity
        .pointer("/candidate/calibration_candidate_identity")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            H001Error::Validation("registered candidate identity is missing".to_string())
        })?
        .to_string();
    let evidence_root_key = format!("attempt_{attempt:03}_evidence_root");
    let certification_identity = identity
        .get("identity_version")
        .and_then(Value::as_str)
        .is_some_and(|version| version.starts_with("phase1c-workflow-identity-certification-"));
    let evidence_root_key = if certification_identity {
        "certification_evidence_root".to_string()
    } else {
        evidence_root_key
    };
    let evidence_root = identity
        .get(&evidence_root_key)
        .and_then(Value::as_str)
        .ok_or_else(|| H001Error::Validation("registered evidence root is missing".to_string()))?
        .to_string();
    let frozen_executable_binding = registered_frozen_binding(&identity)?;
    if let Some(binding) = &frozen_executable_binding {
        reject_mutable_frozen_binding(binding)?;
    }
    let binding = RegisteredWorkflowIdentity {
        attempt_identity_path: path.to_string_lossy().into_owned(),
        attempt_identity_sha256: canonical_hash(&identity)?,
        attempt,
        candidate_budget,
        candidate_identity,
        evidence_root,
        frozen_executable_binding,
    };
    binding.validate()?;
    let expected_budget = if certification_identity {
        1024
    } else {
        match binding.attempt {
            5..=8 => 1024,
            9 | 10 => 512,
            11 => 256,
            _ => 0,
        }
    };
    if (!matches!(binding.attempt, 5..=11) && !certification_identity)
        || binding.candidate_budget != expected_budget
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
    {
        return Err(H001Error::Validation(
            "registered workflow identity is not an Attempt-005/006/007/008 1024, Attempt-009/010 512, or Attempt-011 256 candidate"
                .to_string(),
        ));
    }
    Ok(binding)
}

/// Identity-version prefix of the Phase 1C V3 feasibility gate. The gate is
/// not a calibration attempt: its registered `attempt` is the gate identity
/// number and its `candidate_budget` carries the output ceiling, because the
/// V3 runtime has no reasoning-budget flag.
pub const V3_FEASIBILITY_GATE_IDENTITY_PREFIX: &str = "phase1c-v3-feasibility-gate-";

/// Identity-version prefix of the Phase 1C local-9B capable-model
/// feasibility gate. Registration follows the V3 gate convention.
pub const LOCAL_9B_FEASIBILITY_GATE_IDENTITY_PREFIX: &str = "phase1c-local-9b-feasibility-gate-";

/// A registered spec-driven gate identity kind: its identity prefix, gate id,
/// output ceiling, and inference-request limit.
struct GateIdentityKind {
    label: &'static str,
    prefix: &'static str,
    gate_id: &'static str,
    max_tokens: u32,
    inference_requests: u64,
}

const GATE_IDENTITY_KINDS: [GateIdentityKind; 2] = [
    GateIdentityKind {
        label: "V3 feasibility gate",
        prefix: V3_FEASIBILITY_GATE_IDENTITY_PREFIX,
        gate_id: "phase1c-v3-feasibility-gate",
        max_tokens: 4096,
        inference_requests: 3,
    },
    GateIdentityKind {
        label: "local-9B feasibility gate",
        prefix: LOCAL_9B_FEASIBILITY_GATE_IDENTITY_PREFIX,
        gate_id: "phase1c-local-9b-feasibility-gate",
        max_tokens: 1024,
        // 3 structural probes + 1 frozen h001 BASELINE request.
        inference_requests: 4,
    },
];

/// Supervisor deadline for a registered identity. For a gate identity it is
/// the complete child-lifecycle bound re-derived from the bound spec's limits
/// and deadline components; a recorded deadline that differs from that
/// derivation is rejected. Every other identity keeps the existing production
/// deadline.
pub fn registered_supervisor_deadline_ms(path: &Path) -> Result<u64, H001Error> {
    let identity = read_registered_identity(path)?;
    match gate_identity_kind(&identity) {
        Some(kind) => gate_supervisor_deadline_ms(kind, &identity),
        None => Ok(PRODUCTION_SUPERVISOR_TIMEOUT_MS),
    }
}

fn gate_supervisor_deadline_ms(
    kind: &GateIdentityKind,
    identity: &Value,
) -> Result<u64, H001Error> {
    let component = |pointer: &str| {
        identity.pointer(pointer).cloned().ok_or_else(|| {
            H001Error::Validation(format!("{} identity is missing {pointer}", kind.label))
        })
    };
    let limits: LimitsSpec = serde_json::from_value(component("/spec/limits")?)?;
    let deadlines: DeadlinesSpec = serde_json::from_value(component("/spec/deadlines")?)?;
    validate_supervisor_deadline(&limits, &deadlines)
        .map_err(|error| H001Error::Validation(error.to_string()))
}

/// Read a registered identity; relative paths resolve from the workspace root.
fn read_registered_identity(path: &Path) -> Result<Value, H001Error> {
    let read_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    };
    Ok(serde_json::from_slice(&std::fs::read(read_path)?)?)
}

fn gate_identity_kind(identity: &Value) -> Option<&'static GateIdentityKind> {
    let version = identity.get("identity_version").and_then(Value::as_str)?;
    GATE_IDENTITY_KINDS
        .iter()
        .find(|kind| version.starts_with(kind.prefix))
}

fn reject_mutable_frozen_binding(binding: &FrozenExecutableBinding) -> Result<(), H001Error> {
    if [
        &binding.supervisor.raw_path,
        &binding.supervisor.final_path,
        &binding.child.raw_path,
        &binding.child.final_path,
    ]
    .into_iter()
    .any(|path| mutable_target_debug_path(path))
    {
        return Err(H001Error::Validation(
            "frozen executable identity points at mutable target/debug; stage executables before registration"
                .to_string(),
        ));
    }
    Ok(())
}

fn registered_frozen_binding(
    identity: &Value,
) -> Result<Option<FrozenExecutableBinding>, H001Error> {
    FrozenExecutableBinding::from_implementation_fingerprints(identity)
        .map_err(H001Error::Validation)
}

fn registered_gate_identity(
    kind: &GateIdentityKind,
    path: &Path,
    identity: &Value,
) -> Result<RegisteredWorkflowIdentity, H001Error> {
    let label = kind.label;
    let field = |pointer: &str| {
        identity
            .pointer(pointer)
            .ok_or_else(|| H001Error::Validation(format!("{label} identity is missing {pointer}")))
    };
    let gate_number = field("/gate_identity_number")?
        .as_u64()
        .filter(|number| matches!(number, 1 | 2))
        .ok_or_else(|| H001Error::Validation(format!("{label} identity number must be 1 or 2")))?
        as u32;
    if field("/spec/limits/max_tokens")? != &json!(kind.max_tokens)
        || field("/spec/limits/inference_requests")? != &json!(kind.inference_requests)
        || field("/spec/limits/retry_requests")? != &json!(0)
    {
        return Err(H001Error::Validation(format!(
            "{label} identity limits are not the registered gate limits"
        )));
    }
    gate_supervisor_deadline_ms(kind, identity)?;
    let candidate_identity = field("/spec/gate_id")?
        .as_str()
        .filter(|value| *value == kind.gate_id)
        .ok_or_else(|| H001Error::Validation(format!("{label} id is invalid")))?
        .to_string();
    let evidence_root = field("/spec/evidence_root")?
        .as_str()
        .map(|root| format!("{}/", root.trim_end_matches('/')))
        .ok_or_else(|| H001Error::Validation(format!("{label} evidence root is invalid")))?;
    let frozen_executable_binding = registered_frozen_binding(identity)?;
    let Some(binding) = &frozen_executable_binding else {
        return Err(H001Error::Validation(format!(
            "{label} identity is missing frozen executable identity"
        )));
    };
    reject_mutable_frozen_binding(binding)?;
    let registered = RegisteredWorkflowIdentity {
        attempt_identity_path: path.to_string_lossy().into_owned(),
        attempt_identity_sha256: canonical_hash(identity)?,
        attempt: gate_number,
        candidate_budget: kind.max_tokens,
        candidate_identity,
        evidence_root,
        frozen_executable_binding,
    };
    registered.validate()?;
    Ok(registered)
}

fn mutable_target_debug_path(path: &str) -> bool {
    path.to_ascii_lowercase()
        .replace('/', "\\")
        .contains("target\\debug\\")
}

pub fn run_supervised(
    program: &Path,
    args: &[String],
    deadline: Duration,
) -> Result<Value, H001Error> {
    run_supervised_internal(program, args, deadline, None)
}

/// Launch a child with identity metadata generated by this supervisor.
///
/// The registered attempt identity is the only caller input that selects the
/// launch identity. The supervisor generates the launch identity, its PID and
/// executable path, while the child binds its own PID and executable path from
/// its current process identity. The serialized metadata is the sole handoff
/// transport and is not an authority to exempt an arbitrary PID.
pub fn run_supervised_with_registered_workflow_identity(
    program: &Path,
    args: &[String],
    deadline: Duration,
    registered: &RegisteredWorkflowIdentity,
) -> Result<Value, H001Error> {
    if registered.frozen_executable_binding.is_none() {
        return Err(H001Error::Validation(
            "registered workflow identity is missing frozen executable identity".to_string(),
        ));
    }
    let handoff = build_workflow_launch_metadata(registered, program)?;
    run_supervised_internal(program, args, deadline, Some(handoff))
}

struct ChildOutcome<'a> {
    state: &'a str,
    child_exit_code: Option<i32>,
    child_terminated: bool,
    child_pid: u32,
}

fn run_supervised_internal(
    program: &Path,
    args: &[String],
    deadline: Duration,
    handoff: Option<WorkflowLaunchMetadata>,
) -> Result<Value, H001Error> {
    if deadline.is_zero() {
        return Err(H001Error::Validation(
            "supervisor deadline must be positive".to_string(),
        ));
    }

    let started = Instant::now();
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    if let Some(handoff) = &handoff {
        command.env(WORKFLOW_HANDOFF_ENV, serde_json::to_string(handoff)?);
    }
    let mut child = command.spawn()?;
    let child_pid = child.id();
    loop {
        if let Some(status) = child.try_wait()? {
            let state = if status.success() {
                "COMPLETED"
            } else {
                "CHILD_FAILED"
            };
            return Ok(supervisor_result(
                program,
                args,
                deadline,
                ChildOutcome {
                    state,
                    child_exit_code: status.code(),
                    child_terminated: false,
                    child_pid,
                },
                handoff.as_ref(),
            ));
        }
        if started.elapsed() >= deadline {
            child.kill()?;
            let status = child.wait()?;
            return Ok(supervisor_result(
                program,
                args,
                deadline,
                ChildOutcome {
                    state: "SUPERVISOR_TIMEOUT",
                    child_exit_code: status.code(),
                    child_terminated: true,
                    child_pid,
                },
                handoff.as_ref(),
            ));
        }
        thread::sleep(POLL_INTERVAL);
    }
}

pub fn persist_supervisor_result(path: &Path, result: &Value) -> Result<(), H001Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(result)?)?;
    Ok(())
}

fn supervisor_result(
    program: &Path,
    args: &[String],
    deadline: Duration,
    outcome: ChildOutcome<'_>,
    handoff: Option<&WorkflowLaunchMetadata>,
) -> Value {
    json!({
        "schema_id": "prefixity.phase1c.live-supervisor-result",
        "schema_version": 1,
        "state": outcome.state,
        "production_deadline_ms": PRODUCTION_SUPERVISOR_TIMEOUT_MS,
        "applied_deadline_ms": deadline.as_millis(),
        "child_program": program.to_string_lossy(),
        "child_args": args,
        "child_launches": 1,
        "child_retries": 0,
        "child_exit_code": outcome.child_exit_code,
        "child_pid": outcome.child_pid,
        "child_terminated": outcome.child_terminated,
        "expected_workflow_handoff": handoff.map(|handoff| json!({
            "metadata": handoff,
            "child_pid": outcome.child_pid,
            "transport": "single supervisor-generated serialized environment metadata"
        })),
        "supervisor_network_calls": 0,
        "supervisor_inference_requests": 0,
        "supervisor_retries": 0,
        "termination_policy": "kill child and wait once at outer deadline",
        "arm_advance": "forbidden"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_completion_is_classified_without_retry() {
        let (program, args) = successful_child();
        let result = run_supervised(&program, &args, Duration::from_millis(500)).unwrap();
        assert_eq!(result["state"], "COMPLETED");
        assert_eq!(result["child_retries"], 0);
        assert_eq!(result["child_terminated"], false);
    }

    #[test]
    fn child_failure_is_preserved_without_retry() {
        let (program, args) = failing_child();
        let result = run_supervised(&program, &args, Duration::from_millis(500)).unwrap();
        assert_eq!(result["state"], "CHILD_FAILED");
        assert_eq!(result["child_exit_code"], 7);
        assert_eq!(result["child_retries"], 0);
    }

    #[test]
    fn supervisor_timeout_terminates_child_without_retry() {
        let (program, args) = sleeping_child();
        let result = run_supervised(&program, &args, Duration::from_millis(50)).unwrap();
        assert_eq!(result["state"], "SUPERVISOR_TIMEOUT");
        assert_eq!(result["child_terminated"], true);
        assert_eq!(result["child_retries"], 0);
    }

    #[test]
    fn expected_workflow_handoff_serializes_supervisor_generated_identity() {
        let (program, args) = handoff_child();
        let expected_supervisor = inspect_executable(&std::env::current_exe().unwrap()).unwrap();
        let expected_child = inspect_executable(&program).unwrap();
        let registered = RegisteredWorkflowIdentity {
            attempt_identity_path: "docs/phase-1/attempt-005.json".to_string(),
            attempt_identity_sha256: "a".repeat(64),
            attempt: 5,
            candidate_budget: 1024,
            candidate_identity: "phase1c-reasoning-budget-1024".to_string(),
            evidence_root:
                "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-005/"
                    .to_string(),
            frozen_executable_binding: Some(FrozenExecutableBinding {
                supervisor: expected_supervisor,
                child: expected_child,
            }),
        };
        let result = run_supervised_with_registered_workflow_identity(
            &program,
            &args,
            Duration::from_millis(500),
            &registered,
        )
        .unwrap();
        assert_eq!(result["state"], "COMPLETED");
        assert_eq!(
            result["expected_workflow_handoff"]["metadata"]["launch_identity"],
            registered.generated_launch_identity()
        );
        assert_eq!(
            result["expected_workflow_handoff"]["metadata"]["supervisor_pid"],
            std::process::id()
        );
        assert!(result["expected_workflow_handoff"]["child_pid"]
            .as_u64()
            .is_some_and(|pid| pid > 0));
        assert!(
            result["expected_workflow_handoff"]["metadata"]["supervisor_path"]
                .as_str()
                .is_some_and(|path| !path.is_empty())
        );
        assert_eq!(
            result["expected_workflow_handoff"]["metadata"]["child_binding"],
            "parent_pid"
        );
        assert!(
            result["expected_workflow_handoff"]["metadata"]["child_executable_path"]
                .as_str()
                .is_some_and(|path| !path.is_empty())
        );
    }

    #[test]
    fn launch_identity_is_derived_from_registered_attempt_identity() {
        let registered = RegisteredWorkflowIdentity {
            attempt_identity_path: "attempt-005.json".to_string(),
            attempt_identity_sha256: "b".repeat(64),
            attempt: 5,
            candidate_budget: 1024,
            candidate_identity: "phase1c-reasoning-budget-1024".to_string(),
            evidence_root: "attempt-005/".to_string(),
            frozen_executable_binding: None,
        };
        assert_eq!(
            registered.generated_launch_identity(),
            format!("phase1c-attempt-5-budget-1024-{}", "b".repeat(64))
        );
    }

    #[test]
    fn registered_workflow_without_frozen_identity_is_rejected_before_spawn() {
        let (program, args) = successful_child();
        let registered = RegisteredWorkflowIdentity {
            attempt_identity_path: "attempt-005.json".to_string(),
            attempt_identity_sha256: "c".repeat(64),
            attempt: 5,
            candidate_budget: 1024,
            candidate_identity: "phase1c-reasoning-budget-1024".to_string(),
            evidence_root: "attempt-005/".to_string(),
            frozen_executable_binding: None,
        };
        let error = run_supervised_with_registered_workflow_identity(
            &program,
            &args,
            Duration::from_millis(500),
            &registered,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("missing frozen executable identity"));
    }

    #[test]
    fn attempt_010_registers_only_as_a_512_candidate() {
        let directory = std::env::temp_dir().join(format!(
            "prefixity-attempt-010-registration-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let register = |attempt: u32, budget: u32| {
            let path = directory.join(format!("attempt-{attempt}-{budget}.json"));
            let identity = json!({
                "attempt": attempt,
                "candidate": {
                    "reasoning_budget": budget,
                    "calibration_candidate_identity": format!("phase1c-reasoning-budget-{budget}"),
                    "maximum_requests": 3,
                    "automatic_retries": 0
                },
                format!("attempt_{attempt:03}_evidence_root"): "budget-512-attempt-010/"
            });
            std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
            registered_workflow_identity_from_file(&path)
        };

        let accepted = register(10, 512).unwrap();
        assert_eq!(accepted.attempt, 10);
        assert_eq!(accepted.candidate_budget, 512);
        assert!(register(10, 1024).is_err());
        assert!(register(11, 512).is_err());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn attempt_011_registers_only_as_a_256_candidate() {
        let directory = std::env::temp_dir().join(format!(
            "prefixity-attempt-011-registration-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let register = |attempt: u32, budget: u32| {
            let path = directory.join(format!("attempt-{attempt}-{budget}.json"));
            let identity = json!({
                "attempt": attempt,
                "candidate": {
                    "reasoning_budget": budget,
                    "calibration_candidate_identity": format!("phase1c-reasoning-budget-{budget}"),
                    "maximum_requests": 3,
                    "automatic_retries": 0
                },
                format!("attempt_{attempt:03}_evidence_root"): "budget-256-attempt-011/"
            });
            std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
            registered_workflow_identity_from_file(&path)
        };

        let accepted = register(11, 256).unwrap();
        assert_eq!(accepted.attempt, 11);
        assert_eq!(accepted.candidate_budget, 256);
        assert!(register(11, 512).is_err());
        assert!(register(12, 256).is_err());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn registration_rejects_mutable_target_debug_binding_for_every_identity_kind() {
        let directory = std::env::temp_dir().join(format!(
            "prefixity-mutable-binding-registration-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let object = |stage: &str, name: &str| {
            json!({
                "raw_path": format!("D:/frozen/target/{stage}/{name}"),
                "final_path": format!("D:/frozen/target/{stage}/{name}"),
                "file_size": 1,
                "sha256": "a".repeat(64),
                "file_id": "volume=1;index=1"
            })
        };
        let fingerprints = |stage: &str| {
            json!({
                "supervisor_binary": object(stage, "supervisor.exe"),
                "child_binary": object(stage, "child.exe")
            })
        };
        let write = |name: &str, mut identity: Value, stage: &str| {
            identity["implementation_fingerprints"] = fingerprints(stage);
            let path = directory.join(name);
            std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
            path
        };
        let historical = json!({
            "attempt": 11,
            "candidate": {
                "reasoning_budget": 256,
                "calibration_candidate_identity": "phase1c-reasoning-budget-256",
                "maximum_requests": 3,
                "automatic_retries": 0
            },
            "attempt_011_evidence_root": "budget-256-attempt-011/"
        });
        let gate = json!({
            "identity_version": "phase1c-v3-feasibility-gate-v1",
            "gate_identity_number": 1,
            "spec": {
                "gate_id": "phase1c-v3-feasibility-gate",
                "evidence_root": "experiments/runs/phase1c-scored-capability-v3/feasibility-gate",
                "limits": v3_gate_limits(4096),
                "deadlines": v3_gate_deadlines()
            }
        });
        for (label, identity) in [("historical", historical), ("gate", gate)] {
            let staged = write(
                &format!("{label}-staged.json"),
                identity.clone(),
                "phase1c-v3-feasibility-frozen",
            );
            registered_workflow_identity_from_file(&staged).unwrap();
            let mutable = write(&format!("{label}-debug.json"), identity, "debug");
            let error = registered_workflow_identity_from_file(&mutable)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("points at mutable target/debug"),
                "{label}: {error}"
            );
        }
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn v3_feasibility_gate_identity_registers_only_with_gate_limits() {
        let directory = std::env::temp_dir().join(format!(
            "prefixity-v3-gate-registration-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        // Registration parses the binding only; staged paths need not exist.
        let staged = |name: &str| {
            json!({
                "raw_path": format!("D:/frozen/target/phase1c-v3-feasibility-frozen/{name}"),
                "final_path": format!("D:/frozen/target/phase1c-v3-feasibility-frozen/{name}"),
                "file_size": 1,
                "sha256": "a".repeat(64),
                "file_id": "volume=1;index=1"
            })
        };
        let executable = staged("supervisor.exe");
        let child = staged("child.exe");
        let write_with = |name: &str,
                          gate: u64,
                          max_tokens: u64,
                          with_binding: bool,
                          deadlines: Value| {
            let mut identity = json!({
                "identity_version": "phase1c-v3-feasibility-gate-v1",
                "gate_identity_number": gate,
                "spec": {
                    "gate_id": "phase1c-v3-feasibility-gate",
                    "evidence_root": "experiments/runs/phase1c-scored-capability-v3/feasibility-gate",
                    "limits": v3_gate_limits(max_tokens),
                    "deadlines": deadlines
                }
            });
            if with_binding {
                identity["implementation_fingerprints"] = json!({
                    "supervisor_binary": executable,
                    "child_binary": child
                });
            }
            let path = directory.join(name);
            std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
            path
        };
        let write = |name: &str, gate: u64, max_tokens: u64, with_binding: bool| {
            write_with(name, gate, max_tokens, with_binding, v3_gate_deadlines())
        };

        let accepted = write("gate.json", 1, 4096, true);
        let registered = registered_workflow_identity_from_file(&accepted).unwrap();
        assert_eq!(registered.attempt, 1);
        assert_eq!(registered.candidate_budget, 4096);
        assert_eq!(
            registered.evidence_root,
            "experiments/runs/phase1c-scored-capability-v3/feasibility-gate/"
        );
        assert_eq!(
            registered_supervisor_deadline_ms(&accepted).unwrap(),
            7_501_000
        );
        assert!(
            registered_workflow_identity_from_file(&write("third.json", 3, 4096, true)).is_err()
        );
        assert!(
            registered_workflow_identity_from_file(&write("ceiling.json", 1, 2048, true)).is_err()
        );
        assert!(
            registered_workflow_identity_from_file(&write("unbound.json", 1, 4096, false)).is_err()
        );

        // The supervisor re-derives the deadline from the bound components.
        let mut slower = v3_gate_deadlines();
        slower["inference_request_timeout_ms"] = json!(2_400_001);
        slower["supervisor_deadline_ms"] = json!(7_501_003);
        let slower = write_with("slower.json", 1, 4096, true, slower);
        assert_eq!(
            registered_supervisor_deadline_ms(&slower).unwrap(),
            7_501_003
        );

        // An independent supervisor timeout is rejected by both the deadline
        // reader and registration, whether shorter or longer.
        for (name, deadline) in [("short.json", 2_520_000), ("long.json", 7_501_001)] {
            let mut independent = v3_gate_deadlines();
            independent["supervisor_deadline_ms"] = json!(deadline);
            let path = write_with(name, 1, 4096, true, independent);
            assert!(registered_supervisor_deadline_ms(&path).is_err(), "{name}");
            assert!(
                registered_workflow_identity_from_file(&path).is_err(),
                "{name}"
            );
        }
        let mut legacy = v3_gate_deadlines();
        legacy["supervisor_timeout_ms"] = json!(2_520_000);
        let legacy = write_with("legacy.json", 1, 4096, true, legacy);
        assert!(registered_supervisor_deadline_ms(&legacy).is_err());
        let missing = write_with(
            "missing.json",
            1,
            4096,
            true,
            json!({"supervisor_deadline_ms": 7_501_000}),
        );
        assert!(registered_supervisor_deadline_ms(&missing).is_err());
        std::fs::remove_dir_all(&directory).unwrap();
    }

    fn v3_gate_limits(max_tokens: u64) -> Value {
        json!({
            "context_tokens": 8192,
            "max_tokens": max_tokens,
            "readiness_contacts": 1,
            "token_count_contacts": 3,
            "inference_requests": 3,
            "retry_requests": 0,
            "fallback_requests": 0,
            "adaptive_replicates": 0,
            "warmup_requests": 0
        })
    }

    fn v3_gate_deadlines() -> Value {
        json!({
            "connect_timeout_ms": 1_000,
            "readiness_timeout_ms": 1_000,
            "token_count_request_timeout_ms": 60_000,
            "inference_request_timeout_ms": 2_400_000,
            "non_request_margin_ms": 120_000,
            "supervisor_deadline_ms": 7_501_000
        })
    }

    #[test]
    fn local_9b_gate_identity_registers_only_with_its_own_limits() {
        let directory = std::env::temp_dir().join(format!(
            "prefixity-local-9b-gate-registration-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let staged = |name: &str| {
            json!({
                "raw_path": format!("D:/frozen/target/phase1c-local-9b-feasibility-frozen/{name}"),
                "final_path": format!("D:/frozen/target/phase1c-local-9b-feasibility-frozen/{name}"),
                "file_size": 1,
                "sha256": "a".repeat(64),
                "file_id": "volume=1;index=1"
            })
        };
        let limits = |max_tokens: u64, requests: u64| {
            json!({
                "context_tokens": 8192,
                "max_tokens": max_tokens,
                "readiness_contacts": 1,
                "token_count_contacts": requests,
                "inference_requests": requests,
                "retry_requests": 0,
                "fallback_requests": 0,
                "adaptive_replicates": 0,
                "warmup_requests": 0
            })
        };
        let deadlines = |supervisor_deadline_ms: u64| {
            json!({
                "connect_timeout_ms": 1_000,
                "readiness_timeout_ms": 1_000,
                "token_count_request_timeout_ms": 60_000,
                "inference_request_timeout_ms": 3_540_000,
                "non_request_margin_ms": 1_200_000,
                "supervisor_deadline_ms": supervisor_deadline_ms
            })
        };
        let write = |name: &str, gate_id: &str, limits: Value, deadlines: Value| {
            let identity = json!({
                "identity_version": "phase1c-local-9b-feasibility-gate-v1",
                "gate_identity_number": 1,
                "spec": {
                    "gate_id": gate_id,
                    "evidence_root": "experiments/runs/phase1c-capable-model-local-9b/feasibility-gate",
                    "limits": limits,
                    "deadlines": deadlines
                },
                "implementation_fingerprints": {
                    "supervisor_binary": staged("supervisor.exe"),
                    "child_binary": staged("child.exe")
                }
            });
            let path = directory.join(name);
            std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
            path
        };
        // 1*1000 + 3*60000 + 1*60000 + 3*3540000 + 1*3540000 + 1200000
        let derived = 15_601_000;
        let gate = "phase1c-local-9b-feasibility-gate";
        let accepted = write("gate.json", gate, limits(1024, 4), deadlines(derived));
        let registered = registered_workflow_identity_from_file(&accepted).unwrap();
        assert_eq!(registered.attempt, 1);
        assert_eq!(registered.candidate_budget, 1024);
        assert_eq!(registered.candidate_identity, gate);
        assert_eq!(
            registered.evidence_root,
            "experiments/runs/phase1c-capable-model-local-9b/feasibility-gate/"
        );
        assert_eq!(
            registered_supervisor_deadline_ms(&accepted).unwrap(),
            derived
        );

        // The earlier six-request capacity is rejected even with its own
        // consistent derived deadline.
        for (name, gate_id, limits, deadline) in [
            ("six-requests.json", gate, limits(1024, 6), 22_801_000),
            ("v3-limits.json", gate, limits(4096, 3), derived),
            (
                "v3-gate-id.json",
                "phase1c-v3-feasibility-gate",
                limits(1024, 4),
                derived,
            ),
            ("independent.json", gate, limits(1024, 4), derived + 1),
        ] {
            let path = write(name, gate_id, limits, deadlines(deadline));
            assert!(
                registered_workflow_identity_from_file(&path).is_err(),
                "{name}"
            );
        }
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn historical_identity_kinds_keep_the_production_deadline() {
        let historical = [
            "docs/phase-1/PHASE_1C_H001_V2_BASELINE_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_H001_V2_BASELINE_ATTEMPT_002_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_002_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json",
            "docs/phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json",
        ];
        for path in historical {
            assert_eq!(
                registered_supervisor_deadline_ms(Path::new(path)).unwrap(),
                1_320_000,
                "{path}"
            );
        }
        assert_eq!(PRODUCTION_SUPERVISOR_TIMEOUT_MS, 1_320_000);

        // A non-V3 identity cannot opt into a different deadline by carrying
        // V3-shaped deadline fields.
        let directory = std::env::temp_dir().join(format!(
            "prefixity-historical-deadline-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("attempt.json");
        let identity = json!({
            "identity_version": "phase1c-reasoning-budget-256-attempt-012-v1",
            "spec": {"limits": v3_gate_limits(4096), "deadlines": v3_gate_deadlines()}
        });
        std::fs::write(&path, serde_json::to_vec(&identity).unwrap()).unwrap();
        assert_eq!(registered_supervisor_deadline_ms(&path).unwrap(), 1_320_000);
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn registered_v2_child_args_are_accepted_by_child_cli() {
        assert_eq!(
            parse_v2_cli_args(v2_live_child_args()).unwrap(),
            V2CliCommand::RunBaseline
        );
    }

    #[cfg(windows)]
    fn successful_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from(std::env::var("COMSPEC").unwrap()),
            vec!["/D".to_string(), "/C".to_string(), "exit 0".to_string()],
        )
    }

    #[cfg(windows)]
    fn failing_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from(std::env::var("COMSPEC").unwrap()),
            vec!["/D".to_string(), "/C".to_string(), "exit 7".to_string()],
        )
    }

    #[cfg(windows)]
    fn sleeping_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from("powershell.exe"),
            vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 2".to_string(),
            ],
        )
    }

    #[cfg(windows)]
    fn handoff_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from(std::env::var("COMSPEC").unwrap()),
            vec![
                "/D".to_string(),
                "/C".to_string(),
                format!("if defined {} (exit 0) else (exit 9)", WORKFLOW_HANDOFF_ENV),
            ],
        )
    }

    #[cfg(not(windows))]
    fn successful_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from("sh"),
            vec!["-c".to_string(), "exit 0".to_string()],
        )
    }

    #[cfg(not(windows))]
    fn failing_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from("sh"),
            vec!["-c".to_string(), "exit 7".to_string()],
        )
    }

    #[cfg(not(windows))]
    fn sleeping_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from("sh"),
            vec!["-c".to_string(), "sleep 2".to_string()],
        )
    }

    #[cfg(not(windows))]
    fn handoff_child() -> (std::path::PathBuf, Vec<String>) {
        (
            std::path::PathBuf::from("/bin/sh"),
            vec![
                "-c".to_string(),
                "test -n \"$PREFIXITY_PHASE1C_WORKFLOW_HANDOFF\"".to_string(),
            ],
        )
    }
}
