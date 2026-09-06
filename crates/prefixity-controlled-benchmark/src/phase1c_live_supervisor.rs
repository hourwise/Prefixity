//! Experiment-only outer supervisor for the Phase 1C scored runtime.
//!
//! The supervisor launches an already-built child runner. It never starts
//! llama.cpp, opens a socket, sends HTTP, retries, or selects another arm.

use crate::hashing::canonical_hash;
use crate::phase1c_h001::H001Error;
#[cfg(test)]
use crate::phase1c_h001_v2::{parse_v2_cli_args, v2_live_child_args, V2CliCommand};
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
    pub child_binding: String,
    pub child_executable_path: String,
    pub candidate_budget: u32,
    pub candidate_identity: String,
    pub evidence_root: String,
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
    Ok(WorkflowLaunchMetadata {
        schema_id: WORKFLOW_HANDOFF_SCHEMA_ID.to_string(),
        attempt_identity_path: registered.attempt_identity_path.clone(),
        attempt_identity_sha256: registered.attempt_identity_sha256.clone(),
        attempt: registered.attempt,
        launch_identity: registered.generated_launch_identity(),
        supervisor_pid,
        supervisor_path: supervisor_path.to_string_lossy().into_owned(),
        child_binding: "parent_pid".to_string(),
        child_executable_path: child_path.to_string_lossy().into_owned(),
        candidate_budget: registered.candidate_budget,
        candidate_identity: registered.candidate_identity.clone(),
        evidence_root: registered.evidence_root.clone(),
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
        || metadata.child_binding != "parent_pid"
        || metadata.child_executable_path.trim().is_empty()
        || metadata.attempt == 0
        || metadata.candidate_budget == 0
    {
        return Err(H001Error::Validation(
            "expected workflow launch metadata is invalid".to_string(),
        ));
    }
    Ok(metadata)
}

pub fn registered_workflow_identity_from_file(
    path: &Path,
) -> Result<RegisteredWorkflowIdentity, H001Error> {
    let read_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path)
    };
    let identity: Value = serde_json::from_slice(&std::fs::read(read_path)?)?;
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
    let evidence_root = identity
        .get("attempt_005_evidence_root")
        .and_then(Value::as_str)
        .ok_or_else(|| H001Error::Validation("registered evidence root is missing".to_string()))?
        .to_string();
    let binding = RegisteredWorkflowIdentity {
        attempt_identity_path: path.to_string_lossy().into_owned(),
        attempt_identity_sha256: canonical_hash(&identity)?,
        attempt,
        candidate_budget,
        candidate_identity,
        evidence_root,
    };
    binding.validate()?;
    if binding.attempt != 5
        || binding.candidate_budget != 1024
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
    {
        return Err(H001Error::Validation(
            "registered workflow identity is not the Attempt-005 1024 candidate".to_string(),
        ));
    }
    Ok(binding)
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
        let registered = RegisteredWorkflowIdentity {
            attempt_identity_path: "docs/phase-1/attempt-005.json".to_string(),
            attempt_identity_sha256: "a".repeat(64),
            attempt: 5,
            candidate_budget: 1024,
            candidate_identity: "phase1c-reasoning-budget-1024".to_string(),
            evidence_root:
                "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-005/"
                    .to_string(),
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
        };
        assert_eq!(
            registered.generated_launch_identity(),
            format!("phase1c-attempt-5-budget-1024-{}", "b".repeat(64))
        );
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
            std::path::PathBuf::from("sh"),
            vec![
                "-c".to_string(),
                "test -n '$PREFIXITY_PHASE1C_WORKFLOW_HANDOFF'".to_string(),
            ],
        )
    }
}
