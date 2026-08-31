//! Experiment-only outer supervisor for the Phase 1C scored runtime.
//!
//! The supervisor launches an already-built child runner. It never starts
//! llama.cpp, opens a socket, sends HTTP, retries, or selects another arm.

use crate::phase1c_h001::H001Error;
#[cfg(test)]
use crate::phase1c_h001_v2::{parse_v2_cli_args, v2_live_child_args, V2CliCommand};
use serde_json::{json, Value};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub const PRODUCTION_SUPERVISOR_TIMEOUT_MS: u64 = 1_320_000;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

pub fn run_supervised(
    program: &Path,
    args: &[String],
    deadline: Duration,
) -> Result<Value, H001Error> {
    if deadline.is_zero() {
        return Err(H001Error::Validation(
            "supervisor deadline must be positive".to_string(),
        ));
    }

    let started = Instant::now();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .spawn()?;
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
                state,
                status.code(),
                false,
            ));
        }
        if started.elapsed() >= deadline {
            child.kill()?;
            let status = child.wait()?;
            return Ok(supervisor_result(
                program,
                args,
                deadline,
                "SUPERVISOR_TIMEOUT",
                status.code(),
                true,
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
    state: &str,
    child_exit_code: Option<i32>,
    child_terminated: bool,
) -> Value {
    json!({
        "schema_id": "prefixity.phase1c.live-supervisor-result",
        "schema_version": 1,
        "state": state,
        "production_deadline_ms": PRODUCTION_SUPERVISOR_TIMEOUT_MS,
        "applied_deadline_ms": deadline.as_millis(),
        "child_program": program.to_string_lossy(),
        "child_args": args,
        "child_launches": 1,
        "child_retries": 0,
        "child_exit_code": child_exit_code,
        "child_terminated": child_terminated,
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
}
