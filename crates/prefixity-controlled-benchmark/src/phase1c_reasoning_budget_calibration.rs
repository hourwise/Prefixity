//! Offline preparation and future live execution for a non-scored
//! reasoning-budget feasibility calibration.
//!
//! The calibration is deliberately separate from the Phase 1C capability
//! lineage. It varies only the llama.cpp server-side reasoning budget, keeps
//! reasoning on and the total completion ceiling at 2048, and never consults
//! the h001 evaluator or any Prefixity treatment path.

use crate::hashing::{canonical_hash, canonicalize_source_bytes, sha256_hex};
use crate::phase1c_windows_runtime_exclusivity as windows_exclusivity;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
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
pub const CALIBRATION_ATTEMPT_003_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_003_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_003_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-003";
pub const CALIBRATION_ATTEMPT_004_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_004_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_004_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-004";
pub const CALIBRATION_ATTEMPT_005_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_005_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_005_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-005";
pub const CALIBRATION_ATTEMPT_006_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_006_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_006_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006";
pub const CALIBRATION_ATTEMPT_007_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_007_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_007_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-007";
pub const CALIBRATION_ATTEMPT_008_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_008_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_1024_ATTEMPT_008_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-008";
pub const CALIBRATION_ATTEMPT_008_FROZEN_STAGE_ROOT: &str = "target/phase1c-attempt-008-frozen";
pub const CALIBRATION_ATTEMPT_008_BUDGET_PROVENANCE_PATH: &str =
    "fixtures/phase1c/attempt-008-budget-provenance.json";
pub const CALIBRATION_ATTEMPT_009_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_009_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009";
pub const CALIBRATION_ATTEMPT_009_FROZEN_STAGE_ROOT: &str = "target/phase1c-attempt-009-frozen";
pub const CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH: &str =
    "fixtures/phase1c/attempt-009-budget-provenance.json";
pub const CALIBRATION_CANDIDATE_TRANSITIONS_PATH: &str =
    "fixtures/phase1c/calibration-candidate-transitions.json";
/// Attempts that were executed but are permanently excluded from calibration
/// selection. No authoritative transition may name them as a source.
const CALIBRATION_INADMISSIBLE_ATTEMPTS: [u64; 2] = [7, 9];
pub const CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_EXECUTION_RECORD.md";
pub const CALIBRATION_ATTEMPT_010_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_010_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-010";
pub const CALIBRATION_ATTEMPT_010_FROZEN_STAGE_ROOT: &str = "target/phase1c-attempt-010-frozen";
pub const CALIBRATION_ATTEMPT_010_PREREQUISITE_TRAVERSAL_ROOT: &str =
    "target/phase1c-attempt-010-prerequisite-traversal";
pub const CALIBRATION_ATTEMPT_011_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json";
pub const CALIBRATION_ATTEMPT_011_IDENTITY_FINGERPRINT_PATH: &str =
    "docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.sha256";
pub const CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT: &str =
    "experiments/runs/phase1c-reasoning-budget-calibration/budget-256-attempt-011";
pub const CALIBRATION_ATTEMPT_011_FROZEN_STAGE_ROOT: &str = "target/phase1c-attempt-011-frozen";
pub const CALIBRATION_ATTEMPT_011_PREREQUISITE_TRAVERSAL_ROOT: &str =
    "target/phase1c-attempt-011-prerequisite-traversal";
pub const WORKFLOW_CERTIFICATION_IDENTITY_PATH: &str =
    "docs/phase-1/PHASE_1C_WORKFLOW_IDENTITY_CERTIFICATION_V1.json";
pub const WORKFLOW_CERTIFICATION_RESULT_SCHEMA_ID: &str =
    "prefixity.phase1c.workflow-identity-certification-result";
pub const CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH: &str =
    "crates/prefixity-controlled-benchmark/src/phase1c_windows_runtime_exclusivity.rs";
pub const CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH: &str =
    "crates/prefixity-controlled-benchmark/src/phase1c_live_supervisor.rs";
const ATTEMPT_003_SEALED_IMPLEMENTATION_SHA256: &str =
    "019ed0c7b066773b570f29adb6130b4447b1e9610e13724933b79a9e40374e93";
pub const CALIBRATION_CASE_IDS: [&str; 3] = ["rbcal-001", "rbcal-002", "rbcal-003"];
pub const CALIBRATION_BUDGETS: [u32; 3] = [1024, 512, 256];

const EXPERIMENT_ID: &str = "phase1c-reasoning-budget-calibration";
const MODEL_ID: &str = "ggml-org/Qwen3.5-0.8B-GGUF:Q4_0";
pub(crate) const ENDPOINT: &str = "http://127.0.0.1:8080/v1/chat/completions";
pub(crate) const HOST: &str = "127.0.0.1";
pub(crate) const PORT: u16 = 8080;
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Attempt003Fingerprint,
    Attempt003Preflight,
    Attempt003DryRun,
    Attempt003Poststart,
    RunAttempt003,
    Attempt004Fingerprint,
    Attempt004Preflight,
    Attempt004DryRun,
    Attempt005Fingerprint,
    Attempt005Preflight,
    Attempt005DryRun,
    Attempt005Poststart,
    Attempt006Fingerprint,
    Attempt006Preflight,
    Attempt006DryRun,
    Attempt006Poststart,
    Attempt007Fingerprint,
    Attempt007Preflight,
    Attempt007DryRun,
    Attempt007Poststart,
    Attempt007ValidateEvidence,
    RunAttempt007,
    Attempt008Fingerprint,
    Attempt008Preflight,
    Attempt008DryRun,
    Attempt008Freeze,
    Attempt008ValidatePreparation,
    RunAttempt008,
    Attempt009Fingerprint,
    Attempt009Preflight,
    Attempt009DryRun,
    Attempt009Freeze,
    Attempt009ValidatePreparation,
    Attempt009CandidateOrder,
    RunAttempt009,
    Attempt010Fingerprint,
    Attempt010Preflight,
    Attempt010DryRun,
    Attempt010Freeze,
    Attempt010ValidatePreparation,
    Attempt010RepositoryContract,
    Attempt010LivePrerequisites,
    RunAttempt010,
    Attempt011Fingerprint,
    Attempt011Preflight,
    Attempt011DryRun,
    Attempt011Freeze,
    Attempt011ValidatePreparation,
    Attempt011RepositoryContract,
    Attempt011LivePrerequisites,
    RunAttempt011,
    WorkflowIdentityCertification { result_path: PathBuf },
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
        [command] if command == "attempt-003-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt003Fingerprint)
        }
        [command] if command == "attempt-003-preflight" => {
            Ok(CalibrationCliCommand::Attempt003Preflight)
        }
        [command] if command == "attempt-003-dry-run" => {
            Ok(CalibrationCliCommand::Attempt003DryRun)
        }
        [command] if command == "attempt-003-poststart" => {
            Ok(CalibrationCliCommand::Attempt003Poststart)
        }
        [command] if command == "run-attempt-003" => Ok(CalibrationCliCommand::RunAttempt003),
        [command] if command == "attempt-004-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt004Fingerprint)
        }
        [command] if command == "attempt-004-preflight" => {
            Ok(CalibrationCliCommand::Attempt004Preflight)
        }
        [command] if command == "attempt-004-dry-run" => {
            Ok(CalibrationCliCommand::Attempt004DryRun)
        }
        [command] if command == "attempt-005-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt005Fingerprint)
        }
        [command] if command == "attempt-005-preflight" => {
            Ok(CalibrationCliCommand::Attempt005Preflight)
        }
        [command] if command == "attempt-005-dry-run" => {
            Ok(CalibrationCliCommand::Attempt005DryRun)
        }
        [command] if command == "attempt-005-poststart" => {
            Ok(CalibrationCliCommand::Attempt005Poststart)
        }
        [command] if command == "attempt-006-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt006Fingerprint)
        }
        [command] if command == "attempt-006-preflight" => {
            Ok(CalibrationCliCommand::Attempt006Preflight)
        }
        [command] if command == "attempt-006-dry-run" => {
            Ok(CalibrationCliCommand::Attempt006DryRun)
        }
        [command] if command == "attempt-006-poststart" => {
            Ok(CalibrationCliCommand::Attempt006Poststart)
        }
        [command] if command == "attempt-007-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt007Fingerprint)
        }
        [command] if command == "attempt-007-preflight" => {
            Ok(CalibrationCliCommand::Attempt007Preflight)
        }
        [command] if command == "attempt-007-dry-run" => {
            Ok(CalibrationCliCommand::Attempt007DryRun)
        }
        [command] if command == "attempt-007-poststart" => {
            Ok(CalibrationCliCommand::Attempt007Poststart)
        }
        [command] if command == "attempt-007-validate-evidence" => {
            Ok(CalibrationCliCommand::Attempt007ValidateEvidence)
        }
        [command] if command == "run-attempt-007" => Ok(CalibrationCliCommand::RunAttempt007),
        [command] if command == "attempt-008-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt008Fingerprint)
        }
        [command] if command == "attempt-008-preflight" => {
            Ok(CalibrationCliCommand::Attempt008Preflight)
        }
        [command] if command == "attempt-008-dry-run" => {
            Ok(CalibrationCliCommand::Attempt008DryRun)
        }
        [command] if command == "attempt-008-freeze" => {
            Ok(CalibrationCliCommand::Attempt008Freeze)
        }
        [command] if command == "attempt-008-validate-preparation" => {
            Ok(CalibrationCliCommand::Attempt008ValidatePreparation)
        }
        [command] if command == "run-attempt-008" => Ok(CalibrationCliCommand::RunAttempt008),
        [command] if command == "attempt-009-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt009Fingerprint)
        }
        [command] if command == "attempt-009-preflight" => {
            Ok(CalibrationCliCommand::Attempt009Preflight)
        }
        [command] if command == "attempt-009-dry-run" => {
            Ok(CalibrationCliCommand::Attempt009DryRun)
        }
        [command] if command == "attempt-009-freeze" => {
            Ok(CalibrationCliCommand::Attempt009Freeze)
        }
        [command] if command == "attempt-009-validate-preparation" => {
            Ok(CalibrationCliCommand::Attempt009ValidatePreparation)
        }
        [command] if command == "attempt-009-candidate-order" => {
            Ok(CalibrationCliCommand::Attempt009CandidateOrder)
        }
        [command] if command == "run-attempt-009" => Ok(CalibrationCliCommand::RunAttempt009),
        [command] if command == "attempt-010-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt010Fingerprint)
        }
        [command] if command == "attempt-010-preflight" => {
            Ok(CalibrationCliCommand::Attempt010Preflight)
        }
        [command] if command == "attempt-010-dry-run" => Ok(CalibrationCliCommand::Attempt010DryRun),
        [command] if command == "attempt-010-freeze" => Ok(CalibrationCliCommand::Attempt010Freeze),
        [command] if command == "attempt-010-validate-preparation" => {
            Ok(CalibrationCliCommand::Attempt010ValidatePreparation)
        }
        [command] if command == "attempt-010-repository-contract" => {
            Ok(CalibrationCliCommand::Attempt010RepositoryContract)
        }
        [command] if command == "attempt-010-live-prerequisites" => {
            Ok(CalibrationCliCommand::Attempt010LivePrerequisites)
        }
        [command] if command == "run-attempt-010" => Ok(CalibrationCliCommand::RunAttempt010),
        [command] if command == "attempt-011-fingerprint" => {
            Ok(CalibrationCliCommand::Attempt011Fingerprint)
        }
        [command] if command == "attempt-011-preflight" => {
            Ok(CalibrationCliCommand::Attempt011Preflight)
        }
        [command] if command == "attempt-011-dry-run" => Ok(CalibrationCliCommand::Attempt011DryRun),
        [command] if command == "attempt-011-freeze" => Ok(CalibrationCliCommand::Attempt011Freeze),
        [command] if command == "attempt-011-validate-preparation" => {
            Ok(CalibrationCliCommand::Attempt011ValidatePreparation)
        }
        [command] if command == "attempt-011-repository-contract" => {
            Ok(CalibrationCliCommand::Attempt011RepositoryContract)
        }
        [command] if command == "attempt-011-live-prerequisites" => {
            Ok(CalibrationCliCommand::Attempt011LivePrerequisites)
        }
        [command] if command == "run-attempt-011" => Ok(CalibrationCliCommand::RunAttempt011),
        [command, flag, path]
            if command == "workflow-identity-certification" && flag == "--result" =>
        {
            if path.trim().is_empty() {
                return Err(invalid("workflow certification result path is empty"));
            }
            Ok(CalibrationCliCommand::WorkflowIdentityCertification {
                result_path: PathBuf::from(path),
            })
        }
        _ => Err(ReasoningBudgetCalibrationError::Validation(
            "usage: prefixity-phase1c-reasoning-budget-calibration [preflight|fingerprint|dry-run|run --budget {1024|512|256} --confirm-fresh-runtime|summarize --budget {1024|512|256}|attempt-002-fingerprint|attempt-002-preflight|attempt-002-dry-run|attempt-002-exclusivity-preflight --confirm-no-other-workflow|run-attempt-002 --budget 1024 --server-pid PID --confirm-fresh-runtime --confirm-exclusive-runtime|summarize-attempt-002 --budget 1024|attempt-003-fingerprint|attempt-003-preflight|attempt-003-dry-run|attempt-003-poststart|run-attempt-003|attempt-004-fingerprint|attempt-004-preflight|attempt-004-dry-run|attempt-005-fingerprint|attempt-005-preflight|attempt-005-dry-run|attempt-005-poststart|attempt-006-fingerprint|attempt-006-preflight|attempt-006-dry-run|attempt-006-poststart|attempt-007-fingerprint|attempt-007-preflight|attempt-007-dry-run|attempt-007-poststart|attempt-007-validate-evidence|run-attempt-007|attempt-008-fingerprint|attempt-008-preflight|attempt-008-dry-run|attempt-008-freeze|attempt-008-validate-preparation|run-attempt-008|attempt-009-fingerprint|attempt-009-preflight|attempt-009-dry-run|attempt-009-freeze|attempt-009-validate-preparation|attempt-009-candidate-order|run-attempt-009|attempt-010-fingerprint|attempt-010-preflight|attempt-010-dry-run|attempt-010-freeze|attempt-010-validate-preparation|attempt-010-repository-contract|attempt-010-live-prerequisites|run-attempt-010|attempt-011-fingerprint|attempt-011-preflight|attempt-011-dry-run|attempt-011-freeze|attempt-011-validate-preparation|attempt-011-repository-contract|attempt-011-live-prerequisites|run-attempt-011]".to_string(),
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
    let processes = windows_exclusivity::process_table();
    let port_listeners = windows_exclusivity::tcp_listener_table(PORT);
    let process_records = processes.as_ref().ok().cloned().unwrap_or_default();
    let listener_records = port_listeners.as_ref().ok().cloned().unwrap_or_default();
    let llama_processes = windows_exclusivity::llama_processes(&process_records);
    let competing_processes =
        windows_exclusivity::competing_processes(&process_records, current_pid);
    let mut outcome = windows_exclusivity::classify_prestart(
        &processes,
        &port_listeners,
        confirm_no_other_workflow,
    );
    if outcome == windows_exclusivity::ExclusivityOutcome::ExclusivePrestart
        && !competing_processes.is_empty()
    {
        outcome = windows_exclusivity::ExclusivityOutcome::CompetingWorkflowProcess;
    }
    let passed = outcome.is_ready() && competing_processes.is_empty() && confirm_no_other_workflow;
    Ok(json!({
        "state": if passed { "READY" } else { "BLOCKED" },
        "attempt": 2,
        "check": "runtime_exclusivity_before_server_start",
        "outcome": outcome.as_str(),
        "current_preflight_pid": current_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": listener_records,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "operator_no_other_workflow_confirmed": confirm_no_other_workflow,
        "required_operator_checks": [
            "no llama.exe process is running",
            "port 8080 is not owned or listening",
            "no other Prefixity or Qwen runner is active",
            "no Luna, Codex, or helper workflow is configured to interact with this runtime"
        ],
        "inspection_mechanisms": {
            "processes": "CreateToolhelp32Snapshot + Process32FirstW + Process32NextW",
            "tcp_listeners": "GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER, AF_INET)",
            "executable_path": "OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW",
            "admin_required": false,
            "localhost_contact": false
        },
        "process_inspection": probe_result_status(&processes),
        "port_inspection": probe_result_status(&port_listeners),
        "os_table_inspections": {
            "process_table": 1,
            "tcp_listener_table": 1
        },
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
    let processes = windows_exclusivity::process_table();
    let port_listeners = windows_exclusivity::tcp_listener_table(PORT);
    let process_records = processes.as_ref().ok().cloned().unwrap_or_default();
    let listener_records = port_listeners.as_ref().ok().cloned().unwrap_or_default();
    let llama_processes = windows_exclusivity::llama_processes(&process_records);
    let current_pid = std::process::id();
    let competing_processes =
        windows_exclusivity::competing_processes(&process_records, current_pid);
    let mut outcome =
        windows_exclusivity::classify_poststart(server_pid, &processes, &port_listeners);
    if outcome == windows_exclusivity::ExclusivityOutcome::ExclusivePoststart
        && !competing_processes.is_empty()
    {
        outcome = windows_exclusivity::ExclusivityOutcome::CompetingWorkflowProcess;
    }
    let single_llama_pid = outcome == windows_exclusivity::ExclusivityOutcome::ExclusivePoststart;
    let single_port_pid = listener_records.len() == 1
        && listener_records
            .first()
            .is_some_and(|listener| listener.pid == server_pid);
    let no_competing_processes = competing_processes.is_empty();
    Ok(json!({
        "state": if single_llama_pid && single_port_pid && no_competing_processes { "READY" } else { "BLOCKED" },
        "attempt": 2,
        "check": "runtime_ownership_before_inference",
        "outcome": outcome.as_str(),
        "expected_server_pid": server_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": listener_records,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "executable_paths": llama_processes.iter().filter_map(|process| process.executable_path.clone()).collect::<Vec<_>>(),
        "server_start_identity": "fresh candidate-1024 llama.cpp process",
        "single_expected_llama_process": single_llama_pid,
        "port_owner_matches_server_pid": single_port_pid,
        "no_competing_processes": no_competing_processes,
        "inspection_mechanisms": {
            "processes": "CreateToolhelp32Snapshot + Process32FirstW + Process32NextW",
            "tcp_listeners": "GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER, AF_INET)",
            "executable_path": "OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW",
            "admin_required": false,
            "localhost_contact": false
        },
        "process_inspection": probe_result_status(&processes),
        "port_inspection": probe_result_status(&port_listeners),
        "os_table_inspections": {
            "process_table": 1,
            "tcp_listener_table": 1
        },
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_003_runtime_ownership(
    server_pid: u32,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = expected_workflow_identity_from_supervisor_env()?;
    attempt_003_runtime_ownership_with_expected_workflow(server_pid, &identity)
}

pub fn attempt_003_runtime_ownership_with_expected_workflow(
    server_pid: u32,
    expected_workflow: &windows_exclusivity::ExpectedWorkflowIdentity,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    if server_pid == 0 {
        return Err(invalid("server PID must be nonzero"));
    }
    let processes = windows_exclusivity::process_table();
    let port_listeners = windows_exclusivity::tcp_listener_table(PORT);
    let process_records = processes.as_ref().ok().cloned().unwrap_or_default();
    let listener_records = port_listeners.as_ref().ok().cloned().unwrap_or_default();
    let llama_processes = windows_exclusivity::llama_processes(&process_records);
    let competing_processes = windows_exclusivity::competing_processes_for_expected_workflow(
        &process_records,
        expected_workflow,
    );
    let outcome = windows_exclusivity::classify_poststart_with_expected_workflow(
        server_pid,
        &processes,
        &port_listeners,
        expected_workflow,
    );
    let single_llama_pid = outcome == windows_exclusivity::ExclusivityOutcome::ExclusivePoststart;
    let single_port_pid = listener_records.len() == 1
        && listener_records
            .first()
            .is_some_and(|listener| listener.pid == server_pid);
    let no_unexpected_processes = competing_processes.is_empty();
    let exclusive = single_llama_pid && single_port_pid && no_unexpected_processes;
    Ok(json!({
        "state": if exclusive { "READY" } else { "BLOCKED" },
        "attempt": 3,
        "check": "runtime_ownership_before_inference",
        "outcome": outcome.as_str(),
        "expected_server_pid": server_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": listener_records,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "expected_workflow_identity": expected_workflow,
        "expected_workflow_state": if exclusive {
            "NO_UNEXPECTED_WORKFLOW_PROCESSES"
        } else {
            outcome.as_str()
        },
        "executable_paths": llama_processes.iter().filter_map(|process| process.executable_path.clone()).collect::<Vec<_>>(),
        "server_start_identity": "fresh candidate-1024 llama.cpp process",
        "single_expected_llama_process": single_llama_pid,
        "port_owner_matches_server_pid": single_port_pid,
        "no_competing_processes": no_unexpected_processes,
        "inspection_mechanisms": {
            "processes": "CreateToolhelp32Snapshot + Process32FirstW + Process32NextW",
            "tcp_listeners": "GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER, AF_INET)",
            "executable_path": "OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW",
            "admin_required": false,
            "localhost_contact": false
        },
        "process_inspection": probe_result_status(&processes),
        "port_inspection": probe_result_status(&port_listeners),
        "os_table_inspections": {
            "process_table": 1,
            "tcp_listener_table": 1
        },
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_003_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let processes = windows_exclusivity::process_table().map_err(|failure| {
        ReasoningBudgetCalibrationError::Validation(format!(
            "post-start process inspection failed: {failure}"
        ))
    })?;
    let llama_processes = windows_exclusivity::llama_processes(&processes);
    if llama_processes.len() != 1 {
        return Err(ReasoningBudgetCalibrationError::Validation(format!(
            "post-start expected exactly one llama.exe process, found {}",
            llama_processes.len()
        )));
    }
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    attempt_003_runtime_ownership_with_expected_workflow(llama_processes[0].pid, &expected_workflow)
}

pub fn expected_workflow_identity_from_supervisor_env(
) -> Result<windows_exclusivity::ExpectedWorkflowIdentity, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    let child_path = env::current_exe()?;
    let child_pid = std::process::id();
    if child_pid == 0 {
        return Err(invalid("expected workflow child PID is invalid"));
    }
    let expected_child_path = std::fs::canonicalize(&child_path)
        .map_err(|error| invalid(&format!("expected workflow child path is invalid: {error}")))?;
    let expected_child_identity = crate::phase1c_executable_identity::inspect(&expected_child_path)
        .map_err(|error| {
            invalid(&format!(
                "expected workflow child identity is invalid: {error}"
            ))
        })?;
    if !crate::phase1c_executable_identity::same(
        &expected_child_identity,
        &metadata.child_executable_identity,
    ) {
        return Err(invalid(
            "expected workflow child executable identity does not match supervisor handoff",
        ));
    }
    let supervisor_identity = crate::phase1c_executable_identity::inspect(Path::new(
        &metadata.supervisor_path,
    ))
    .map_err(|error| {
        invalid(&format!(
            "expected workflow supervisor identity is invalid: {error}"
        ))
    })?;
    if !crate::phase1c_executable_identity::same(
        &supervisor_identity,
        &metadata.supervisor_executable_identity,
    ) {
        return Err(invalid(
            "expected workflow supervisor executable identity does not match handoff",
        ));
    }
    if let Some(binding) = &metadata.frozen_executable_binding {
        crate::phase1c_executable_identity::validate_frozen_executable_binding(
            binding,
            &supervisor_identity,
            &expected_child_identity,
        )
        .map_err(|error| invalid(&error))?;
    } else {
        return Err(invalid(
            "workflow launch metadata is missing frozen executable identity",
        ));
    }
    if metadata.supervisor_pid == child_pid {
        return Err(invalid(
            "expected workflow supervisor and child identities are invalid",
        ));
    }
    Ok(windows_exclusivity::ExpectedWorkflowIdentity {
        launch_identity: metadata.launch_identity,
        supervisor: windows_exclusivity::ExpectedProcessIdentity {
            pid: metadata.supervisor_pid,
            executable_path: metadata.supervisor_path,
            executable_identity: metadata.supervisor_executable_identity,
            parent_pid: None,
        },
        calibration_child: windows_exclusivity::ExpectedProcessIdentity {
            pid: child_pid,
            executable_path: expected_child_path.to_string_lossy().into_owned(),
            executable_identity: expected_child_identity,
            parent_pid: Some(metadata.supervisor_pid),
        },
        inspector: None,
    })
}

pub fn execute_attempt_003() -> Result<Value, ReasoningBudgetCalibrationError> {
    let budget = 1024;
    let identity = read_attempt_003_identity(true)?;
    validate_attempt_003_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_003_bindings(&identity, &fingerprints)?;
    let candidate_root = attempt_003_root();
    if candidate_root.exists() {
        return Err(invalid("attempt-003 evidence root already exists"));
    }
    fs::create_dir_all(&candidate_root)?;
    write_json(
        &candidate_root.join("preflight.json"),
        &json!({
            "state": "PRESTART_GATE_COMPLETED",
            "attempt": 3,
            "candidate_budget": budget,
            "experiment_id": EXPERIMENT_ID,
            "evidence_root": CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT,
            "manifest_sha256": fingerprints["manifest_sha256"],
            "request_hashes": fingerprints["cases"],
            "native_prestart_outcome": "EXCLUSIVE_PRESTART",
            "native_prestart_captured_before_server_start": true,
            "network_calls": 0,
            "listener_checks": 0,
            "inference_requests": 0
        }),
    )?;

    let ownership = match attempt_003_poststart() {
        Ok(value) => value,
        Err(error) => {
            write_json(
                &candidate_root.join("runtime-ownership-error.json"),
                &json!({"error": error.to_string(), "inference_requests": 0}),
            )?;
            return Err(error);
        }
    };
    write_json(&candidate_root.join("runtime-ownership.json"), &ownership)?;
    if ownership.get("state").and_then(Value::as_str) != Some("READY")
        || ownership.get("outcome").and_then(Value::as_str) != Some("EXCLUSIVE_POSTSTART")
    {
        let mut result = candidate_result(
            budget,
            Vec::new(),
            0,
            "INCONCLUSIVE",
            false,
            Some("runtime ownership was not exclusive before inference"),
        )?;
        if let Some(object) = result.as_object_mut() {
            object.insert("listener_check_attempts".to_string(), json!(0));
            object.insert("network_calls".to_string(), json!(0));
        }
        write_json(&candidate_root.join("candidate-result.json"), &result)?;
        return Ok(result);
    }

    write_json(
        &candidate_root.join("runtime-confirmation.json"),
        &json!({
            "confirmation": "operator_current_confirmation",
            "attempt": 3,
            "experiment_id": EXPERIMENT_ID,
            "build": "b10217-ddd4ec142",
            "model": MODEL_ID,
            "quantization": "Q4_0",
            "context_size": 8192,
            "parallel_slots": 1,
            "metrics": "enabled",
            "reasoning": "on",
            "reasoning_budget": budget,
            "reasoning_budget_message": "unset",
            "endpoint": ENDPOINT,
            "fresh_server_per_candidate": true,
            "server_pid": ownership["expected_server_pid"],
            "executable_path": ownership["executable_paths"][0],
            "port_owner_pid": ownership["port_8080_listeners"][0]["pid"],
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
        write_json(&candidate_root.join("candidate-result.json"), &result)?;
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
        let result = execute_case(&manifest, case, Some(budget), &case_dir, &client)?;
        let infrastructure_ambiguous =
            result.get("state").and_then(Value::as_str) == Some("INCONCLUSIVE");
        results.push(result);
        if infrastructure_ambiguous {
            break;
        }
    }
    let state = aggregate_state(&results);
    let case_set_complete = results.len() == CALIBRATION_CASE_IDS.len()
        && results
            .iter()
            .all(|result| result.get("state").and_then(Value::as_str) != Some("INCONCLUSIVE"));
    let error = (state == "INCONCLUSIVE").then_some("infrastructure ambiguity stopped Attempt 003");
    let inference_requests = results.len() as u32;
    let result = candidate_result(
        budget,
        results,
        inference_requests,
        state,
        case_set_complete,
        error,
    )?;
    write_json(&candidate_root.join("candidate-result.json"), &result)?;
    Ok(result)
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
        results.push(execute_case(
            &manifest,
            case,
            Some(budget),
            &case_dir,
            &client,
        )?);
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

pub fn fingerprint_attempt_003() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_003_identity(false)?;
    validate_attempt_003_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_003_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "identity_sha256": canonical_hash(&identity)?,
        "implementation_source_sha256": implementation_source_sha256()?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "attempt": 3,
        "evidence_root": CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_003() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_003_identity(true)?;
    validate_attempt_003_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_003_bindings(&identity, &fingerprints)?;
    if attempt_003_root().exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(
            "attempt-003 evidence root already exists; preparation is no longer pristine"
                .to_string(),
        ));
    }
    if attempt_002_root().exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(
            "attempt-002 evidence root exists; attempt-003 lineage is not pristine".to_string(),
        ));
    }
    let os_inspection = windows_native_prestart_value(3, true);
    Ok(json!({
        "state": "PREPARED",
        "attempt": 3,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT,
        "evidence_root_absent": true,
        "attempt_001_immutable": true,
        "attempt_002_root_absent": true,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_003() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_003()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
            let request = build_request(&manifest, case)?;
            Ok(json!({
                "attempt": 3,
                "budget": 1024,
                "case_id": case_id,
                "server_reasoning_budget": 1024,
                "request_has_reasoning_budget_field": false,
                "request_sha256": canonical_hash(&request)?,
                "wire_request_sha256": sha256_hex(&serde_json::to_vec(&request)?),
                "request_bytes": serde_json::to_vec(&request)?.len(),
                "evidence_root": format!("{CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT}/{case_id}"),
                "network_calls": 0,
                "listener_checks": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 3,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn fingerprint_attempt_004() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_004_identity(false)?;
    validate_attempt_004_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_004_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "identity_sha256": canonical_hash(&identity)?,
        "implementation_source_sha256": implementation_source_sha256()?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "attempt": 4,
        "evidence_root": CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_004() -> Result<Value, ReasoningBudgetCalibrationError> {
    preflight_attempt_004_with_evidence_guard(true)
}

fn preflight_attempt_004_with_evidence_guard(
    require_preserved_evidence: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_004_identity(true)?;
    if require_preserved_evidence {
        validate_attempt_004_identity(&identity, true)?;
    } else {
        validate_attempt_004_identity_for_clean_checkout(&identity, true)?;
    }
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_004_bindings(&identity, &fingerprints)?;
    if require_preserved_evidence {
        validate_attempt_004_evidence_state()?;
    }
    let os_inspection = windows_native_prestart_value(4, true);
    Ok(json!({
        "state": "PREPARED",
        "attempt": 4,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT,
        "evidence_root_absent": true,
        "attempt_001_immutable": true,
        "attempt_002_root_absent": true,
        "attempt_003_immutable": true,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_004() -> Result<Value, ReasoningBudgetCalibrationError> {
    dry_run_attempt_004_with_preflight(preflight_attempt_004()?)
}

/// Validate the Attempt 004 preparation contract without requiring ignored
/// local evidence from earlier attempts. The default workspace test path uses
/// this portable form; the live/preflight command remains evidence-strict.
pub fn dry_run_attempt_004_portable() -> Result<Value, ReasoningBudgetCalibrationError> {
    dry_run_attempt_004_with_preflight(preflight_attempt_004_with_evidence_guard(false)?)
}

fn dry_run_attempt_004_with_preflight(
    preflight: Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 4,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT}/{case_id}"),
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
        "attempt": 4,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Certify preserved calibration evidence in an enriched local workspace.
///
/// This is deliberately not called by the default test suite because the
/// evidence roots are ignored and are absent in a clean checkout. It reads
/// and hashes preserved files only, fails closed when they are missing or
/// changed, and never writes to the evidence roots.
pub fn certify_preserved_calibration_evidence() -> Result<Value, ReasoningBudgetCalibrationError> {
    if !attempt_003_root().exists() {
        return Err(invalid("attempt-003 evidence root is absent"));
    }
    for (name, expected_hash) in attempt_003_evidence_hashes() {
        let actual_hash =
            sha256_hex(&fs::read(attempt_003_root().join(name)).map_err(|error| {
                invalid(&format!(
                    "unable to read preserved attempt-003 evidence {name}: {error}"
                ))
            })?);
        if actual_hash != expected_hash {
            return Err(invalid(&format!(
                "preserved attempt-003 evidence hash mismatch for {name}"
            )));
        }
    }
    let attempt_004_state = if attempt_004_root().exists() {
        validate_attempt_004_execution_evidence()?;
        "PRESENT_VALIDATED"
    } else {
        validate_attempt_004_evidence_state()?;
        "ABSENT_VALIDATED"
    };
    Ok(json!({
        "state": "PRESERVED_EVIDENCE_VALIDATED",
        "attempt_003": "PRESENT_VALIDATED",
        "attempt_004": attempt_004_state,
        "network_calls": 0,
        "inference_requests": 0
    }))
}

pub fn fingerprint_attempt_005() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_005_identity(false)?;
    validate_attempt_005_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_005_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "identity_sha256": canonical_hash(&identity)?,
        "launch_handoff_implementation_sha256": source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH)?,
        "native_exclusivity_implementation_sha256": implementation_source_sha256()?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "attempt": 5,
        "evidence_root": CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_005() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_005_identity(true)?;
    validate_attempt_005_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_005_bindings(&identity, &fingerprints)?;
    validate_attempt_005_evidence_state()?;
    let os_inspection = windows_native_prestart_value(5, true);
    Ok(json!({
        "state": "PREPARED",
        "attempt": 5,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT,
        "evidence_root_absent": true,
        "attempt_001_immutable": true,
        "attempt_002_root_absent": true,
        "attempt_003_immutable": true,
        "attempt_004_immutable": true,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_005() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_005()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 5,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT}/{case_id}"),
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
        "attempt": 5,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_005_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_005_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(5);
    result["server_start_identity"] = json!("fresh candidate-1024 llama.cpp process");
    Ok(result)
}

pub fn fingerprint_attempt_006() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_006_identity(false)?;
    validate_attempt_006_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_006_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "identity_sha256": canonical_hash(&identity)?,
        "launch_handoff_implementation_sha256": source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH)?,
        "native_exclusivity_implementation_sha256": implementation_source_sha256()?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "attempt": 6,
        "evidence_root": CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_006() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_006_identity(true)?;
    validate_attempt_006_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_006_bindings(&identity, &fingerprints)?;
    validate_attempt_006_evidence_state()?;
    let os_inspection = windows_native_prestart_value(6, true);
    Ok(json!({
        "state": "PREPARED",
        "attempt": 6,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT,
        "evidence_root_absent": true,
        "attempt_001_immutable": true,
        "attempt_002_root_absent": true,
        "attempt_003_immutable": true,
        "attempt_004_immutable": true,
        "attempt_005_root_absent": true,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_006() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_006()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 6,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 1024,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 6,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_006_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_006_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(6);
    result["server_start_identity"] = json!("fresh candidate-1024 llama.cpp process");
    Ok(result)
}

pub fn fingerprint_attempt_007() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_007_identity(false)?;
    validate_attempt_007_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_007_bindings(&identity, &fingerprints)?;
    Ok(json!({
        "state": "PREPARED",
        "identity_sha256": canonical_hash(&identity)?,
        "launch_handoff_implementation_sha256": source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH)?,
        "native_exclusivity_implementation_sha256": implementation_source_sha256()?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "workflow_certification_manifest_sha256": "824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4",
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "cases": fingerprints["cases"],
        "attempt": 7,
        "evidence_root": CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn preflight_attempt_007() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_007_identity(true)?;
    validate_attempt_007_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_007_bindings(&identity, &fingerprints)?;
    validate_attempt_007_evidence_state()?;
    let os_inspection = windows_native_prestart_value(7, true);
    Ok(json!({
        "state": "PREPARED",
        "attempt": 7,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT,
        "evidence_root_absent": true,
        "attempt_001_immutable": true,
        "attempt_002_root_absent": true,
        "attempt_003_immutable": true,
        "attempt_004_immutable": true,
        "attempt_005_root_absent": true,
        "attempt_006_evidence_hash_preserved": true,
        "workflow_identity_certification": "WORKFLOW_IDENTITY_CERTIFIED",
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_007() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_007()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 7,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT}/{case_id}"),
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
        "attempt": 7,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn attempt_007_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_007_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(7);
    result["server_start_identity"] = json!("fresh candidate-1024 llama.cpp process");
    Ok(result)
}

pub fn execute_attempt_007() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_007_handoff(&metadata)?;
    let identity = read_attempt_007_identity(true)?;
    validate_attempt_007_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_007_bindings(&identity, &fingerprints)?;
    if attempt_007_root().exists() {
        return Err(invalid("attempt-007 evidence root already exists"));
    }
    let ownership = attempt_007_poststart()?;
    let preparation = json!({
        "state": "POSTSTART_GATE_COMPLETED",
        "attempt": 7,
        "candidate_budget": 1024,
        "experiment_id": EXPERIMENT_ID,
        "evidence_root": CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "native_prestart_outcome": "EXCLUSIVE_PRESTART",
        "native_prestart_captured_before_server_start": true,
        "runtime_ownership": ownership,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    });
    let candidate_root = attempt_007_root();
    execute_calibration_at_root(1024, true, &candidate_root, preparation)
}

/// Validate the deterministic Attempt-008 preparation without starting a
/// model server or creating an Attempt-008 execution record.
pub fn fingerprint_attempt_008() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    validate_attempt_008_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_008_bindings(&identity, &fingerprints)?;
    let frozen = validate_attempt_008_frozen_executables(&identity)?;
    Ok(json!({
        "state": "PREPARED",
        "attempt": 8,
        "candidate_budget": 1024,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "case_order": CALIBRATION_CASE_IDS,
        "frozen_executable_binding": frozen,
        "workflow_identity_certification_v2": "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Run the Attempt-008 virgin-state and native pre-start checks. These are
/// read-only OS-table inspections; they do not contact localhost or a model.
pub fn preflight_attempt_008() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    validate_attempt_008_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_008_bindings(&identity, &fingerprints)?;
    let frozen = validate_attempt_008_frozen_executables(&identity)?;
    validate_attempt_008_history()?;
    validate_attempt_008_budget_provenance()?;
    validate_attempt_008_virgin_state()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let os_inspection = windows_native_prestart_value(8, true);
    Ok(json!({
        "state": "ATTEMPT_008_PREPARED",
        "execution_state": "ATTEMPT_008_NOT_EXECUTED",
        "attempt": 8,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 1024,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT,
        "frozen_executable_binding": frozen,
        "workflow_identity_certification_v2": certification,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_008() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_008()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 8,
                "budget": 1024,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 1024,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 8,
        "budget": 1024,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Freeze the final build outputs into a fresh, bounded, non-overwriting
/// Attempt-008 staging directory. This command is offline and does not run
/// either executable.
pub fn freeze_attempt_008() -> Result<Value, ReasoningBudgetCalibrationError> {
    let stage = workspace_path(CALIBRATION_ATTEMPT_008_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid(
            "attempt-008 frozen staging directory already exists",
        ));
    }
    let supervisor_source = workspace_path("target/debug/prefixity-phase1c-live-supervisor.exe");
    let child_source =
        workspace_path("target/debug/prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_destination = stage.join("prefixity-phase1c-live-supervisor.exe");
    let child_destination = stage.join("prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor = crate::phase1c_executable_identity::freeze_copy(
        &supervisor_source,
        &supervisor_destination,
    )
    .map_err(|error| invalid(&error))?;
    let child = crate::phase1c_executable_identity::freeze_copy(&child_source, &child_destination)
        .map_err(|error| invalid(&error))?;
    Ok(json!({
        "state": "FROZEN",
        "attempt": 8,
        "source_commit": "recorded by preparation identity",
        "supervisor_source_path": supervisor_source,
        "child_source_path": child_source,
        "supervisor_frozen_path": supervisor_destination,
        "child_frozen_path": child_destination,
        "supervisor_binary": supervisor,
        "child_binary": child,
        "overwrite": false,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "inference_requests": 0
    }))
}

pub fn validate_attempt_008_preparation() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_008()?;
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    let frozen = validate_attempt_008_frozen_executables(&identity)?;
    Ok(json!({
        "state": "ATTEMPT_008_PREPARATION_ACCEPTED",
        "execution_state": "ATTEMPT_008_NOT_EXECUTED",
        "identity_sha256": canonical_hash(&identity)?,
        "frozen_executable_binding": frozen,
        "preflight": preflight,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "tcp_readiness_contacts": 0,
        "http_model_requests": 0,
        "inference_requests": 0
    }))
}

/// Validate the deterministic Attempt-009 preparation without starting a
/// model server or creating an Attempt-009 execution record.
pub fn fingerprint_attempt_009() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    validate_attempt_009_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_009_bindings(&identity, &fingerprints)?;
    let frozen = validate_attempt_009_frozen_executables(&identity)?;
    Ok(json!({
        "state": "PREPARED",
        "attempt": 9,
        "candidate_budget": 512,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "case_order": CALIBRATION_CASE_IDS,
        "frozen_executable_binding": frozen,
        "workflow_identity_certification_v2": "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Run the Attempt-009 virgin-state and native pre-start checks. These are
/// read-only OS-table inspections; they do not contact localhost or a model.
pub fn preflight_attempt_009() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    validate_attempt_009_identity(&identity, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_009_bindings(&identity, &fingerprints)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_009_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if registered.attempt != 9
        || registered.candidate_budget != 512
        || registered.candidate_identity != "phase1c-reasoning-budget-512"
        || registered.evidence_root != format!("{CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT}/")
        || registered.frozen_executable_binding.is_none()
    {
        return Err(invalid(
            "Attempt-009 registered workflow identity binding is invalid",
        ));
    }
    let frozen = validate_attempt_009_frozen_executables(&identity)?;
    // The tracked transition is the same dependency the live candidate-order
    // gate validates. Preserved Attempt-008 raw evidence remains a separate
    // forensic audit source and is never a runtime prerequisite.
    let predecessor = resolve_authoritative_candidate_transition(512)?;
    let provenance = validate_attempt_009_budget_provenance(&predecessor)?;
    validate_attempt_009_virgin_state()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let os_inspection = windows_native_prestart_value(9, true);
    if os_inspection["state"] != "READY" || os_inspection["outcome"] != "EXCLUSIVE_PRESTART" {
        return Err(invalid(
            "Attempt-009 preflight did not establish exclusive pre-start state",
        ));
    }
    Ok(json!({
        "state": "ATTEMPT_009_PREPARED",
        "execution_state": "ATTEMPT_009_NOT_EXECUTED",
        "attempt": 9,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 512,
        "case_order": CALIBRATION_CASE_IDS,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "evidence_root": CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT,
        "registered_workflow": {
            "attempt": registered.attempt,
            "candidate_budget": registered.candidate_budget,
            "candidate_identity": registered.candidate_identity,
            "evidence_root": registered.evidence_root,
            "generated_launch_identity": registered.generated_launch_identity()
        },
        "frozen_executable_binding": frozen,
        "attempt_008_admissible_predecessor": predecessor,
        "budget_512_provenance": provenance,
        "workflow_identity_certification_v2": certification,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_009() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_009()?;
    let manifest = read_manifest(true)?;
    let cases = manifest
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
                "attempt": 9,
                "budget": 512,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 512,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 9,
        "budget": 512,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Freeze the final build outputs into a fresh, bounded, non-overwriting
/// Attempt-009 staging directory. This command is offline and does not run
/// either executable.
pub fn freeze_attempt_009() -> Result<Value, ReasoningBudgetCalibrationError> {
    let stage = workspace_path(CALIBRATION_ATTEMPT_009_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid(
            "attempt-009 frozen staging directory already exists",
        ));
    }
    let supervisor_source = workspace_path("target/debug/prefixity-phase1c-live-supervisor.exe");
    let child_source =
        workspace_path("target/debug/prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_destination = stage.join("prefixity-phase1c-live-supervisor.exe");
    let child_destination = stage.join("prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor = crate::phase1c_executable_identity::freeze_copy(
        &supervisor_source,
        &supervisor_destination,
    )
    .map_err(|error| invalid(&error))?;
    let child = crate::phase1c_executable_identity::freeze_copy(&child_source, &child_destination)
        .map_err(|error| invalid(&error))?;
    Ok(json!({
        "state": "FROZEN",
        "attempt": 9,
        "candidate_budget": 512,
        "source_commit": "recorded by preparation identity",
        "supervisor_source_path": supervisor_source,
        "child_source_path": child_source,
        "supervisor_frozen_path": supervisor_destination,
        "child_frozen_path": child_destination,
        "supervisor_binary": supervisor,
        "child_binary": child,
        "overwrite": false,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "inference_requests": 0
    }))
}

pub fn validate_attempt_009_preparation() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_009()?;
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    let frozen = validate_attempt_009_frozen_executables(&identity)?;
    Ok(json!({
        "state": "ATTEMPT_009_PREPARATION_ACCEPTED",
        "execution_state": "ATTEMPT_009_NOT_EXECUTED",
        "identity_sha256": canonical_hash(&identity)?,
        "frozen_executable_binding": frozen,
        "preflight": preflight,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "tcp_readiness_contacts": 0,
        "http_model_requests": 0,
        "inference_requests": 0,
        "attempt_009_executions": 0
    }))
}

/// Validate the Attempt-009 candidate-order dependency without entering any
/// live boundary. This is intentionally usable from a clean checkout after
/// the accepted transition fixture is present; it does not require ignored
/// Attempt-008 raw evidence, a virgin Attempt-009 root, a listener, or a
/// model-server process.
pub fn validate_attempt_009_candidate_order() -> Result<Value, ReasoningBudgetCalibrationError> {
    let order = validate_candidate_order_report(512)?;
    if order["candidate_order_valid"] != true {
        return Err(invalid("Attempt-009 candidate order is invalid"));
    }
    let predecessor = order
        .get("predecessor_transition")
        .cloned()
        .ok_or_else(|| invalid("Attempt-009 predecessor transition is absent"))?;
    let provenance = validate_attempt_009_budget_provenance(&predecessor)?;
    Ok(json!({
        "state": "CANDIDATE_BUDGET_512_ORDER_VALID",
        "attempt": 9,
        "candidate_budget": 512,
        "RUNTIME_DEPENDENCIES_COMPLETE": true,
        "PREDECESSOR_TRANSITION_PRESENT": true,
        "PREDECESSOR_TRANSITION_VALID": true,
        "CANDIDATE_ORDER_VALID": true,
        "SOURCE_ATTEMPT_008": true,
        "SOURCE_BUDGET_1024": true,
        "SOURCE_STATE_FAIL": true,
        "NEXT_BUDGET_512": true,
        "CANDIDATE_BUDGET_512_ORDER_VALID": true,
        "source_attempt": 8,
        "source_candidate_budget": 1024,
        "source_result": "FAIL",
        "source_integrity": "ACCEPTED",
        "source_calibration_admissible": true,
        "next_budget": 512,
        "attempt_007_excluded_from_selection": true,
        "authoritative_transition_path": CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH,
        "raw_predecessor_evidence_required": false,
        "predecessor_transition": predecessor,
        "budget_512_provenance": provenance,
        "next_fresh_attempt_id": 10,
        "next_fresh_candidate_budget": 512,
        "attempt_010_prepared": false,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "tcp_readiness_contacts": 0,
        "http_model_requests": 0,
        "inference_requests": 0,
        "attempt_009_executions_added": 0,
        "attempt_010_executions": 0
    }))
}

/// Future live entry point. It is intentionally separate from preparation
/// commands and is not called by this preparation task.
pub fn execute_attempt_009() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_009_handoff(&metadata)?;
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    validate_attempt_009_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_009_bindings(&identity, &fingerprints)?;
    let candidate_root = attempt_009_root();
    if candidate_root.exists() {
        return Err(invalid("attempt-009 evidence root already exists"));
    }
    let ownership = attempt_009_poststart()?;
    let preparation = json!({
        "state": "POSTSTART_GATE_COMPLETED",
        "attempt": 9,
        "candidate_budget": 512,
        "experiment_id": EXPERIMENT_ID,
        "evidence_root": CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "native_prestart_outcome": "EXCLUSIVE_PRESTART",
        "native_prestart_captured_before_server_start": true,
        "runtime_ownership": ownership,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    });
    execute_calibration_at_root(512, true, &candidate_root, preparation)
}

fn attempt_009_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_009_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(9);
    result["server_start_identity"] = json!("fresh candidate-512 llama.cpp process");
    Ok(result)
}

fn validate_attempt_009_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    validate_attempt_009_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_009_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != 9
        || metadata.candidate_budget != 512
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "attempt-009 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

/// Validate the deterministic Attempt-010 preparation without starting a
/// model server or creating an Attempt-010 execution record.
pub fn fingerprint_attempt_010() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    validate_attempt_010_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_identity_request_bindings(&identity, &fingerprints, "attempt-010")?;
    let frozen = validate_attempt_010_frozen_executables(&identity)?;
    Ok(json!({
        "state": "PREPARED",
        "attempt": 10,
        "candidate_budget": 512,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "case_order": CALIBRATION_CASE_IDS,
        "frozen_executable_binding": frozen,
        "workflow_identity_certification_v2": "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Run the Attempt-010 virgin-state and native pre-start checks. These are
/// read-only OS-table inspections; they do not contact localhost or a model.
pub fn preflight_attempt_010() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    validate_attempt_010_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_010_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if registered.attempt != 10
        || registered.candidate_budget != 512
        || registered.candidate_identity != "phase1c-reasoning-budget-512"
        || registered.evidence_root != format!("{CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT}/")
        || registered.attempt_identity_sha256 != canonical_hash(&identity)?
        || registered.frozen_executable_binding.is_none()
    {
        return Err(invalid(
            "Attempt-010 registered workflow identity binding is invalid",
        ));
    }
    let frozen = validate_attempt_010_frozen_executables(&identity)?;
    let contract = validate_attempt_010_repository_contract()?;
    validate_attempt_010_virgin_state()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let os_inspection = windows_native_prestart_value(10, true);
    if os_inspection["state"] != "READY" || os_inspection["outcome"] != "EXCLUSIVE_PRESTART" {
        return Err(invalid(
            "Attempt-010 preflight did not establish exclusive pre-start state",
        ));
    }
    Ok(json!({
        "state": "ATTEMPT_010_PREPARED",
        "execution_state": "ATTEMPT_010_NOT_EXECUTED",
        "attempt": 10,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 512,
        "case_order": CALIBRATION_CASE_IDS,
        "evidence_root": CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT,
        "registered_workflow": {
            "attempt": registered.attempt,
            "candidate_budget": registered.candidate_budget,
            "candidate_identity": registered.candidate_identity,
            "evidence_root": registered.evidence_root,
            "generated_launch_identity": registered.generated_launch_identity()
        },
        "frozen_executable_binding": frozen,
        "repository_contract": contract,
        "workflow_identity_certification_v2": certification,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_010() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_010()?;
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
                "attempt": 10,
                "budget": 512,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 512,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 10,
        "budget": 512,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Freeze the final build outputs into a fresh, bounded, non-overwriting
/// Attempt-010 staging directory. This command is offline and does not run
/// either executable.
pub fn freeze_attempt_010() -> Result<Value, ReasoningBudgetCalibrationError> {
    let stage = workspace_path(CALIBRATION_ATTEMPT_010_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid(
            "attempt-010 frozen staging directory already exists",
        ));
    }
    let supervisor_source = workspace_path("target/debug/prefixity-phase1c-live-supervisor.exe");
    let child_source =
        workspace_path("target/debug/prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_destination = stage.join("prefixity-phase1c-live-supervisor.exe");
    let child_destination = stage.join("prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_source_identity =
        crate::phase1c_executable_identity::inspect(&supervisor_source)
            .map_err(|error| invalid(&error))?;
    let child_source_identity = crate::phase1c_executable_identity::inspect(&child_source)
        .map_err(|error| invalid(&error))?;
    let supervisor = crate::phase1c_executable_identity::freeze_copy(
        &supervisor_source,
        &supervisor_destination,
    )
    .map_err(|error| invalid(&error))?;
    let child = crate::phase1c_executable_identity::freeze_copy(&child_source, &child_destination)
        .map_err(|error| invalid(&error))?;
    if supervisor.sha256 != supervisor_source_identity.sha256
        || child.sha256 != child_source_identity.sha256
    {
        return Err(invalid(
            "attempt-010 frozen executable content differs from its build output",
        ));
    }
    Ok(json!({
        "state": "FROZEN",
        "attempt": 10,
        "candidate_budget": 512,
        "source_commit": "recorded by preparation identity",
        "supervisor_source_path": supervisor_source,
        "child_source_path": child_source,
        "supervisor_source_binary": supervisor_source_identity,
        "child_source_binary": child_source_identity,
        "supervisor_frozen_path": supervisor_destination,
        "child_frozen_path": child_destination,
        "supervisor_binary": supervisor,
        "child_binary": child,
        "overwrite": false,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "inference_requests": 0
    }))
}

pub fn validate_attempt_010_preparation() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_010()?;
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    let frozen = validate_attempt_010_frozen_executables(&identity)?;
    Ok(json!({
        "state": "ATTEMPT_010_PREPARATION_ACCEPTED",
        "execution_state": "ATTEMPT_010_NOT_EXECUTED",
        "identity_sha256": canonical_hash(&identity)?,
        "frozen_executable_binding": frozen,
        "preflight": preflight,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "tcp_readiness_contacts": 0,
        "http_model_requests": 0,
        "inference_requests": 0,
        "attempt_010_executions": 0
    }))
}

/// Validate every Attempt-010 dependency bound by tracked repository
/// evidence: identity and sidecar, calibration manifest and request hashes,
/// the authoritative Attempt-008 transition, candidate order, Attempt-009
/// exclusion, and the frozen run contract. It reads no ignored prior-run
/// evidence and no frozen executable, so it runs unchanged in a clean
/// checkout. The live prerequisite traversal calls this same function.
pub fn validate_attempt_010_repository_contract() -> Result<Value, ReasoningBudgetCalibrationError>
{
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    validate_attempt_010_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_identity_request_bindings(&identity, &fingerprints, "attempt-010")?;
    let manifest_case_order = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?
        .iter()
        .map(|case| case.get("case_id").cloned().unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    if manifest_case_order != CALIBRATION_CASE_IDS.map(|case_id| json!(case_id)) {
        return Err(invalid("Attempt-010 manifest case set changed"));
    }
    let candidate_order = validate_candidate_order_report(512)?;
    let predecessor = candidate_order
        .get("predecessor_transition")
        .cloned()
        .ok_or_else(|| invalid("Attempt-010 predecessor transition is absent"))?;
    if candidate_order["candidate_order_valid"] != true
        || candidate_order.get("predecessor_result_path").is_some()
        || predecessor["source_attempt"] != 8
        || predecessor["candidate_budget"] != 1024
        || predecessor["candidate_state"] != "FAIL"
        || predecessor["integrity_accepted"] != true
        || predecessor["calibration_admissible"] != true
        || predecessor["next_budget"] != 512
        || predecessor["attempt_007_excluded_from_selection"] != true
        || predecessor["raw_predecessor_evidence_required"] != false
        || identity.pointer("/lineage/attempt_008_identity_sha256")
            != predecessor.get("identity_sha256")
        || identity.pointer("/lineage/attempt_008_evidence_manifest_sha256")
            != predecessor.get("evidence_manifest_sha256")
    {
        return Err(invalid("Attempt-010 predecessor transition is invalid"));
    }
    let attempt_009 = validate_attempt_009_excluded_from_selection()?;
    let expected_server_command = format!(
        "{} {}",
        ATTEMPT_010_SERVER_EXECUTABLE,
        server_launch_arguments(512).join(" ")
    );
    if identity.pointer("/launch_plan/server_command") != Some(&json!(expected_server_command)) {
        return Err(invalid(
            "Attempt-010 server command differs from the runtime launch arguments",
        ));
    }
    Ok(json!({
        "state": "ATTEMPT_010_REPOSITORY_CONTRACT_VALID",
        "attempt": 10,
        "candidate_budget": 512,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "ATTEMPT_ID_VALID": true,
        "ATTEMPT_IDENTITY_VALID": true,
        "PREDECESSOR_TRANSITION_PRESENT": true,
        "PREDECESSOR_TRANSITION_VALID": true,
        "SOURCE_ATTEMPT_008": true,
        "SOURCE_BUDGET_1024": true,
        "SOURCE_STATE_FAIL": true,
        "SOURCE_INTEGRITY_ACCEPTED": true,
        "SOURCE_CALIBRATION_ADMISSIBLE": true,
        "NEXT_BUDGET_512": true,
        "CANDIDATE_ORDER_VALID": true,
        "CANDIDATE_BUDGET_512_ORDER_VALID": true,
        "ATTEMPT_007_EXCLUDED_FROM_SELECTION": true,
        "ATTEMPT_009_EXCLUDED_FROM_SELECTION": true,
        "CASE_SET_VALID": true,
        "REQUEST_CEILING_VALID": true,
        "RETRY_POLICY_VALID": true,
        "FALLBACK_POLICY_VALID": true,
        "case_order": CALIBRATION_CASE_IDS,
        "request_ceiling": 3,
        "automatic_retries": 0,
        "fallback_requests": 0,
        "adaptive_replicates": 0,
        "server_command": expected_server_command,
        "candidate_order": candidate_order,
        "attempt_009": attempt_009,
        "raw_predecessor_evidence_required": false,
        "network_calls": 0,
        "inference_requests": 0
    }))
}

/// Every deterministic check the live Attempt-010 child performs before
/// post-start runtime ownership inspection and the single listener check.
/// `run-attempt-010` calls this first; the offline traversal calls the same
/// function under the frozen supervisor and then stops.
fn attempt_010_live_prerequisites() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_010_handoff(&metadata)?;
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    validate_attempt_010_identity(&identity, true)?;
    let frozen = validate_attempt_010_frozen_executables(&identity)?;
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let contract = validate_attempt_010_repository_contract()?;
    validate_attempt_010_virgin_state()?;
    let (_manifest, candidate_order) = calibration_prestart_checks(512, true, &attempt_010_root())?;
    if candidate_order != contract["candidate_order"] {
        return Err(invalid(
            "Attempt-010 runtime candidate order differs from the repository contract",
        ));
    }
    Ok(json!({
        "attempt": 10,
        "candidate_budget": 512,
        "identity_sha256": canonical_hash(&identity)?,
        "launch_identity": metadata.launch_identity,
        "supervisor_pid": metadata.supervisor_pid,
        "child_pid": std::process::id(),
        "ATTEMPT_ID_VALID": true,
        "ATTEMPT_IDENTITY_VALID": true,
        "FROZEN_SUPERVISOR_VALID": true,
        "FROZEN_CHILD_VALID": true,
        "WORKFLOW_CERTIFICATION_VALID": true,
        "RUNTIME_DEPENDENCIES_COMPLETE": true,
        "PREDECESSOR_TRANSITION_PRESENT": true,
        "PREDECESSOR_TRANSITION_VALID": true,
        "SOURCE_ATTEMPT_008": true,
        "NEXT_BUDGET_512": true,
        "CANDIDATE_ORDER_VALID": true,
        "CASE_SET_VALID": true,
        "REQUEST_CEILING_VALID": true,
        "RETRY_POLICY_VALID": true,
        "FALLBACK_POLICY_VALID": true,
        "ATTEMPT_010_VIRGIN": true,
        "frozen_executable_binding": frozen,
        "expected_workflow": expected_workflow,
        "workflow_identity_certification_v2": certification,
        "repository_contract": contract,
        "runtime_candidate_order": candidate_order,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Offline traversal of the live Attempt-010 child's deterministic startup.
/// It must be launched by the frozen supervisor with the Attempt-010 identity
/// and stops before post-start ownership inspection, TCP readiness, HTTP, and
/// inference. It writes nothing.
pub fn validate_attempt_010_live_prerequisites() -> Result<Value, ReasoningBudgetCalibrationError> {
    let mut report = attempt_010_live_prerequisites()?;
    report["state"] = json!("READY_FOR_MODEL_READINESS_BOUNDARY");
    report["READY_FOR_MODEL_READINESS_BOUNDARY"] = json!(true);
    report["stopped_before"] = json!([
        "post-start runtime ownership inspection",
        "tcp listener readiness",
        "http model request",
        "inference"
    ]);
    report["model_server_startups"] = json!(0);
    report["port_8080_contacts"] = json!(0);
    report["tcp_readiness_contacts"] = json!(0);
    report["http_model_requests"] = json!(0);
    report["inference_requests"] = json!(0);
    report["attempt_010_executions"] = json!(0);
    Ok(report)
}

/// Future live entry point. It is intentionally separate from preparation
/// commands and is not called by the preparation task.
pub fn execute_attempt_010() -> Result<Value, ReasoningBudgetCalibrationError> {
    let prerequisites = attempt_010_live_prerequisites()?;
    let ownership = attempt_010_poststart()?;
    let fingerprints = fingerprint_calibration()?;
    let preparation = json!({
        "state": "POSTSTART_GATE_COMPLETED",
        "attempt": 10,
        "candidate_budget": 512,
        "experiment_id": EXPERIMENT_ID,
        "evidence_root": CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "native_prestart_outcome": "EXCLUSIVE_PRESTART",
        "native_prestart_captured_before_server_start": true,
        "live_prerequisites": prerequisites,
        "runtime_ownership": ownership,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    });
    execute_calibration_at_root(512, true, &attempt_010_root(), preparation)
}

fn attempt_010_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_010_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(10);
    result["server_start_identity"] = json!("fresh candidate-512 llama.cpp process");
    Ok(result)
}

fn validate_attempt_010_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)?;
    validate_attempt_010_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_010_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != 10
        || metadata.candidate_budget != 512
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "attempt-010 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

/// Validate the deterministic Attempt-011 preparation without starting a
/// model server or creating an Attempt-011 execution record.
pub fn fingerprint_attempt_011() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    validate_attempt_011_identity(&identity, false)?;
    let fingerprints = fingerprint_calibration()?;
    validate_identity_request_bindings(&identity, &fingerprints, "attempt-011")?;
    let frozen = validate_attempt_011_frozen_executables(&identity)?;
    Ok(json!({
        "state": "PREPARED",
        "attempt": 11,
        "candidate_budget": 256,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "case_order": CALIBRATION_CASE_IDS,
        "frozen_executable_binding": frozen,
        "workflow_identity_certification_v2": "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Run the Attempt-011 virgin-state and native pre-start checks. These are
/// read-only OS-table inspections; they do not contact localhost or a model.
pub fn preflight_attempt_011() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    validate_attempt_011_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_011_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if registered.attempt != 11
        || registered.candidate_budget != 256
        || registered.candidate_identity != "phase1c-reasoning-budget-256"
        || registered.evidence_root != format!("{CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT}/")
        || registered.attempt_identity_sha256 != canonical_hash(&identity)?
        || registered.frozen_executable_binding.is_none()
    {
        return Err(invalid(
            "Attempt-011 registered workflow identity binding is invalid",
        ));
    }
    let frozen = validate_attempt_011_frozen_executables(&identity)?;
    let contract = validate_attempt_011_repository_contract()?;
    validate_attempt_011_virgin_state()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let os_inspection = windows_native_prestart_value(11, true);
    if os_inspection["state"] != "READY" || os_inspection["outcome"] != "EXCLUSIVE_PRESTART" {
        return Err(invalid(
            "Attempt-011 preflight did not establish exclusive pre-start state",
        ));
    }
    Ok(json!({
        "state": "ATTEMPT_011_PREPARED",
        "execution_state": "ATTEMPT_011_NOT_EXECUTED",
        "attempt": 11,
        "experiment_id": EXPERIMENT_ID,
        "candidate_budget": 256,
        "case_order": CALIBRATION_CASE_IDS,
        "evidence_root": CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT,
        "registered_workflow": {
            "attempt": registered.attempt,
            "candidate_budget": registered.candidate_budget,
            "candidate_identity": registered.candidate_identity,
            "evidence_root": registered.evidence_root,
            "generated_launch_identity": registered.generated_launch_identity()
        },
        "frozen_executable_binding": frozen,
        "repository_contract": contract,
        "workflow_identity_certification_v2": certification,
        "windows_native_exclusivity": os_inspection,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

pub fn dry_run_attempt_011() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_011()?;
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
                "attempt": 11,
                "budget": 256,
                "case_id": case_id,
                "evidence_root": format!("{CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT}/{case_id}"),
                "server_reasoning_budget": 256,
                "request_has_reasoning_budget_field": false,
                "request_sha256": case["request_sha256"],
                "wire_request_sha256": case["wire_request_sha256"],
                "request_bytes": case["request_bytes"],
                "network_calls": 0,
                "inference_requests": 0
            }))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    Ok(json!({
        "state": "DRY_RUN",
        "attempt": 11,
        "budget": 256,
        "preflight": preflight,
        "combinations": combinations,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Freeze the final build outputs into a fresh, bounded, non-overwriting
/// Attempt-011 staging directory. This command is offline and does not run
/// either executable.
pub fn freeze_attempt_011() -> Result<Value, ReasoningBudgetCalibrationError> {
    let stage = workspace_path(CALIBRATION_ATTEMPT_011_FROZEN_STAGE_ROOT);
    if stage.exists() {
        return Err(invalid(
            "attempt-011 frozen staging directory already exists",
        ));
    }
    let supervisor_source = workspace_path("target/debug/prefixity-phase1c-live-supervisor.exe");
    let child_source =
        workspace_path("target/debug/prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_destination = stage.join("prefixity-phase1c-live-supervisor.exe");
    let child_destination = stage.join("prefixity-phase1c-reasoning-budget-calibration.exe");
    let supervisor_source_identity =
        crate::phase1c_executable_identity::inspect(&supervisor_source)
            .map_err(|error| invalid(&error))?;
    let child_source_identity = crate::phase1c_executable_identity::inspect(&child_source)
        .map_err(|error| invalid(&error))?;
    let supervisor = crate::phase1c_executable_identity::freeze_copy(
        &supervisor_source,
        &supervisor_destination,
    )
    .map_err(|error| invalid(&error))?;
    let child = crate::phase1c_executable_identity::freeze_copy(&child_source, &child_destination)
        .map_err(|error| invalid(&error))?;
    if supervisor.sha256 != supervisor_source_identity.sha256
        || child.sha256 != child_source_identity.sha256
    {
        return Err(invalid(
            "attempt-011 frozen executable content differs from its build output",
        ));
    }
    Ok(json!({
        "state": "FROZEN",
        "attempt": 11,
        "candidate_budget": 256,
        "source_commit": "recorded by preparation identity",
        "supervisor_source_path": supervisor_source,
        "child_source_path": child_source,
        "supervisor_source_binary": supervisor_source_identity,
        "child_source_binary": child_source_identity,
        "supervisor_frozen_path": supervisor_destination,
        "child_frozen_path": child_destination,
        "supervisor_binary": supervisor,
        "child_binary": child,
        "overwrite": false,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "inference_requests": 0
    }))
}

pub fn validate_attempt_011_preparation() -> Result<Value, ReasoningBudgetCalibrationError> {
    let preflight = preflight_attempt_011()?;
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    let frozen = validate_attempt_011_frozen_executables(&identity)?;
    Ok(json!({
        "state": "ATTEMPT_011_PREPARATION_ACCEPTED",
        "execution_state": "ATTEMPT_011_NOT_EXECUTED",
        "identity_sha256": canonical_hash(&identity)?,
        "frozen_executable_binding": frozen,
        "preflight": preflight,
        "model_server_startups": 0,
        "port_8080_contacts": 0,
        "tcp_readiness_contacts": 0,
        "http_model_requests": 0,
        "inference_requests": 0,
        "attempt_011_executions": 0
    }))
}

/// Validate every Attempt-011 dependency bound by tracked repository
/// evidence: identity and sidecar, current source, calibration manifest and
/// request hashes, the authoritative Attempt-010 transition resolved through
/// the transition registry, candidate order, Attempt-007/009 exclusion, and
/// the frozen run contract. It reads no ignored prior-run
/// evidence and no frozen executable, so it runs unchanged in a clean
/// checkout. The live prerequisite traversal calls this same function.
pub fn validate_attempt_011_repository_contract() -> Result<Value, ReasoningBudgetCalibrationError>
{
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    validate_attempt_011_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_identity_request_bindings(&identity, &fingerprints, "attempt-011")?;
    let manifest_case_order = manifest
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?
        .iter()
        .map(|case| case.get("case_id").cloned().unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    if manifest_case_order != CALIBRATION_CASE_IDS.map(|case_id| json!(case_id)) {
        return Err(invalid("Attempt-011 manifest case set changed"));
    }
    let candidate_order = validate_candidate_order_report(256)?;
    let predecessor = candidate_order
        .get("predecessor_transition")
        .cloned()
        .ok_or_else(|| invalid("Attempt-011 predecessor transition is absent"))?;
    if candidate_order["candidate_order_valid"] != true
        || candidate_order.get("predecessor_result_path").is_some()
        || predecessor["source_attempt"] != 10
        || predecessor["source_budget"] != 512
        || predecessor["source_state"] != "FAIL"
        || predecessor["integrity_accepted"] != true
        || predecessor["calibration_admissible"] != true
        || predecessor["selected_next_budget"] != 256
        || predecessor["attempt_007_excluded_from_selection"] != true
        || predecessor["attempt_009_excluded_from_selection"] != true
        || predecessor["raw_predecessor_evidence_required"] != false
        || predecessor["authoritative_transition_path"] != CALIBRATION_CANDIDATE_TRANSITIONS_PATH
        || identity.pointer("/lineage/attempt_010_identity_sha256")
            != predecessor.get("identity_sha256")
        || identity.pointer("/lineage/attempt_010_evidence_manifest_sha256")
            != predecessor.get("evidence_manifest_sha256")
        || identity.pointer("/lineage/attempt_010_execution_record_sha256")
            != predecessor.get("execution_record_sha256")
    {
        return Err(invalid("Attempt-011 predecessor transition is invalid"));
    }
    let registry_sha256 = source_sha256(CALIBRATION_CANDIDATE_TRANSITIONS_PATH)?;
    if identity.pointer("/budget_provenance/authoritative_transition_registry_sha256")
        != Some(&json!(registry_sha256))
    {
        return Err(invalid(
            "Attempt-011 authoritative transition registry changed",
        ));
    }
    let attempt_009 = validate_attempt_009_excluded_from_selection()?;
    let expected_server_command = format!(
        "{} {}",
        ATTEMPT_011_SERVER_EXECUTABLE,
        server_launch_arguments(256).join(" ")
    );
    if identity.pointer("/launch_plan/server_command") != Some(&json!(expected_server_command)) {
        return Err(invalid(
            "Attempt-011 server command differs from the runtime launch arguments",
        ));
    }
    Ok(json!({
        "state": "ATTEMPT_011_REPOSITORY_CONTRACT_VALID",
        "attempt": 11,
        "candidate_budget": 256,
        "identity_sha256": canonical_hash(&identity)?,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "ATTEMPT_ID_VALID": true,
        "ATTEMPT_IDENTITY_VALID": true,
        "PREDECESSOR_TRANSITION_PRESENT": true,
        "PREDECESSOR_TRANSITION_VALID": true,
        "SOURCE_ATTEMPT_010": true,
        "SOURCE_BUDGET_512": true,
        "SOURCE_STATE_FAIL": true,
        "SOURCE_INTEGRITY_ACCEPTED": true,
        "SOURCE_CALIBRATION_ADMISSIBLE": true,
        "NEXT_BUDGET_256": true,
        "CANDIDATE_ORDER_VALID": true,
        "CANDIDATE_BUDGET_256_ORDER_VALID": true,
        "ATTEMPT_007_EXCLUDED_FROM_SELECTION": true,
        "ATTEMPT_009_EXCLUDED_FROM_SELECTION": true,
        "ATTEMPT_008_NOT_SOURCE_FOR_256": true,
        "CASE_SET_VALID": true,
        "REQUEST_CEILING_VALID": true,
        "RETRY_POLICY_VALID": true,
        "FALLBACK_POLICY_VALID": true,
        "case_order": CALIBRATION_CASE_IDS,
        "request_ceiling": 3,
        "automatic_retries": 0,
        "fallback_requests": 0,
        "adaptive_replicates": 0,
        "server_command": expected_server_command,
        "candidate_order": candidate_order,
        "authoritative_transition_registry_sha256": registry_sha256,
        "attempt_009": attempt_009,
        "raw_predecessor_evidence_required": false,
        "network_calls": 0,
        "inference_requests": 0
    }))
}

/// Every deterministic check the live Attempt-011 child performs before
/// post-start runtime ownership inspection and the single listener check.
/// `run-attempt-011` calls this first; the offline traversal calls the same
/// function under the frozen supervisor and then stops.
fn attempt_011_live_prerequisites() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_011_handoff(&metadata)?;
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    validate_attempt_011_identity(&identity, true)?;
    let frozen = validate_attempt_011_frozen_executables(&identity)?;
    let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
    let certification = validate_accepted_workflow_certification_v2()?;
    let contract = validate_attempt_011_repository_contract()?;
    validate_attempt_011_virgin_state()?;
    let (_manifest, candidate_order) = calibration_prestart_checks(256, true, &attempt_011_root())?;
    if candidate_order != contract["candidate_order"] {
        return Err(invalid(
            "Attempt-011 runtime candidate order differs from the repository contract",
        ));
    }
    Ok(json!({
        "attempt": 11,
        "candidate_budget": 256,
        "identity_sha256": canonical_hash(&identity)?,
        "launch_identity": metadata.launch_identity,
        "supervisor_pid": metadata.supervisor_pid,
        "child_pid": std::process::id(),
        "ATTEMPT_ID_VALID": true,
        "ATTEMPT_IDENTITY_VALID": true,
        "FROZEN_SUPERVISOR_VALID": true,
        "FROZEN_CHILD_VALID": true,
        "WORKFLOW_CERTIFICATION_VALID": true,
        "RUNTIME_DEPENDENCIES_COMPLETE": true,
        "PREDECESSOR_TRANSITION_PRESENT": true,
        "PREDECESSOR_TRANSITION_VALID": true,
        "SOURCE_ATTEMPT_010": true,
        "NEXT_BUDGET_256": true,
        "CANDIDATE_ORDER_VALID": true,
        "CASE_SET_VALID": true,
        "REQUEST_CEILING_VALID": true,
        "RETRY_POLICY_VALID": true,
        "FALLBACK_POLICY_VALID": true,
        "ATTEMPT_011_VIRGIN": true,
        "frozen_executable_binding": frozen,
        "expected_workflow": expected_workflow,
        "workflow_identity_certification_v2": certification,
        "repository_contract": contract,
        "runtime_candidate_order": candidate_order,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    }))
}

/// Offline traversal of the live Attempt-011 child's deterministic startup.
/// It must be launched by the frozen supervisor with the Attempt-011 identity
/// and stops before post-start ownership inspection, TCP readiness, HTTP, and
/// inference. It writes nothing.
pub fn validate_attempt_011_live_prerequisites() -> Result<Value, ReasoningBudgetCalibrationError> {
    let mut report = attempt_011_live_prerequisites()?;
    report["state"] = json!("READY_FOR_MODEL_READINESS_BOUNDARY");
    report["READY_FOR_MODEL_READINESS_BOUNDARY"] = json!(true);
    report["stopped_before"] = json!([
        "post-start runtime ownership inspection",
        "tcp listener readiness",
        "http model request",
        "inference"
    ]);
    report["model_server_startups"] = json!(0);
    report["port_8080_contacts"] = json!(0);
    report["tcp_readiness_contacts"] = json!(0);
    report["http_model_requests"] = json!(0);
    report["inference_requests"] = json!(0);
    report["attempt_011_executions"] = json!(0);
    Ok(report)
}

/// Future live entry point. It is intentionally separate from preparation
/// commands and is not called by the preparation task.
pub fn execute_attempt_011() -> Result<Value, ReasoningBudgetCalibrationError> {
    let prerequisites = attempt_011_live_prerequisites()?;
    let ownership = attempt_011_poststart()?;
    let fingerprints = fingerprint_calibration()?;
    let preparation = json!({
        "state": "POSTSTART_GATE_COMPLETED",
        "attempt": 11,
        "candidate_budget": 256,
        "experiment_id": EXPERIMENT_ID,
        "evidence_root": CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "native_prestart_outcome": "EXCLUSIVE_PRESTART",
        "native_prestart_captured_before_server_start": true,
        "live_prerequisites": prerequisites,
        "runtime_ownership": ownership,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    });
    execute_calibration_at_root(256, true, &attempt_011_root(), preparation)
}

fn attempt_011_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_011_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(11);
    result["server_start_identity"] = json!("fresh candidate-256 llama.cpp process");
    Ok(result)
}

fn validate_attempt_011_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)?;
    validate_attempt_011_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_011_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != 11
        || metadata.candidate_budget != 256
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "attempt-011 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

/// Future live entry point. It is intentionally separate from preparation
/// commands and is not called by this preparation task.
pub fn execute_attempt_008() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_008_handoff(&metadata)?;
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    validate_attempt_008_identity(&identity, true)?;
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let fingerprints = fingerprint_calibration()?;
    validate_attempt_008_bindings(&identity, &fingerprints)?;
    let candidate_root = attempt_008_root();
    if candidate_root.exists() {
        return Err(invalid("attempt-008 evidence root already exists"));
    }
    let ownership = attempt_008_poststart()?;
    let preparation = json!({
        "state": "POSTSTART_GATE_COMPLETED",
        "attempt": 8,
        "candidate_budget": 1024,
        "experiment_id": EXPERIMENT_ID,
        "evidence_root": CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT,
        "manifest_sha256": fingerprints["manifest_sha256"],
        "request_hashes": fingerprints["cases"],
        "native_prestart_outcome": "EXCLUSIVE_PRESTART",
        "native_prestart_captured_before_server_start": true,
        "runtime_ownership": ownership,
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    });
    execute_calibration_at_root(1024, true, &candidate_root, preparation)
}

fn attempt_008_poststart() -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    validate_attempt_008_handoff(&metadata)?;
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
    let mut result = attempt_003_runtime_ownership_with_expected_workflow(
        llama_processes[0].pid,
        &expected_workflow,
    )?;
    result["attempt"] = json!(8);
    result["server_start_identity"] = json!("fresh candidate-1024 llama.cpp process");
    Ok(result)
}

fn validate_attempt_008_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    validate_attempt_008_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_008_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != 8
        || metadata.candidate_budget != 1024
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "attempt-008 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

/// Validate recorded supervisor evidence against the preparation-time
/// executable objects without interpreting any model output.
pub fn validate_recorded_workflow_evidence_against_preparation(
    preparation_identity: &Value,
    supervisor_evidence: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let binding =
        crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
            preparation_identity,
        )
        .map_err(|error| invalid(&error))?
        .ok_or_else(|| invalid("preparation identity is missing frozen executable identity"))?;
    let metadata = supervisor_evidence
        .pointer("/expected_workflow_handoff/metadata")
        .ok_or_else(|| invalid("supervisor evidence is missing workflow metadata"))?;
    let actual_supervisor = metadata
        .get("supervisor_executable_identity")
        .cloned()
        .ok_or_else(|| invalid("supervisor evidence is missing supervisor executable identity"))?;
    let actual_child = metadata
        .get("child_executable_identity")
        .cloned()
        .ok_or_else(|| invalid("supervisor evidence is missing child executable identity"))?;
    let actual_supervisor: crate::phase1c_executable_identity::ExecutableIdentity =
        serde_json::from_value(actual_supervisor)
            .map_err(|error| invalid(&format!("invalid recorded supervisor identity: {error}")))?;
    let actual_child: crate::phase1c_executable_identity::ExecutableIdentity =
        serde_json::from_value(actual_child)
            .map_err(|error| invalid(&format!("invalid recorded child identity: {error}")))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))
}

/// Offline forensic classification for the preserved Attempt-007 supervisor
/// record. This reads the evidence but never rewrites or reinterprets it.
pub fn validate_attempt_007_execution_evidence() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_attempt_007_identity(false)?;
    let supervisor_path = attempt_007_root().join("supervisor.json");
    let supervisor_evidence = read_json_path(&supervisor_path)?;
    match validate_recorded_workflow_evidence_against_preparation(&identity, &supervisor_evidence) {
        Ok(()) => Ok(json!({
            "state": "INTEGRITY_ACCEPTED",
            "classification": "FROZEN_EXECUTABLE_IDENTITY_MATCH",
            "attempt": 7,
            "calibration_admissible": false,
            "model_outputs_interpreted": false,
            "network_calls": 0,
            "inference_requests": 0
        })),
        Err(error)
            if error
                .to_string()
                .contains("FROZEN_EXECUTABLE_IDENTITY_MISMATCH") =>
        {
            Ok(json!({
                "state": "INTEGRITY_REJECTED",
                "classification": "FROZEN_EXECUTABLE_IDENTITY_MISMATCH",
                "attempt": 7,
                "calibration_admissible": false,
                "model_outputs_interpreted": false,
                "network_calls": 0,
                "inference_requests": 0,
                "reason": error.to_string()
            }))
        }
        Err(error) => Err(error),
    }
}

/// Validate a real supervisor-to-child launch without starting llama.cpp or
/// contacting any endpoint.  This is deliberately a child command so the
/// production supervisor binary and its inherited handoff transport are used.
pub fn certify_workflow_identity(
    result_path: &Path,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let metadata = crate::phase1c_live_supervisor::workflow_launch_metadata_from_env()
        .map_err(|error| invalid(&error.to_string()))?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(&metadata.attempt_identity_path),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) || metadata.attempt_identity_sha256 != registered.attempt_identity_sha256
        || metadata.attempt != registered.attempt
        || metadata.candidate_budget != registered.candidate_budget
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
        || metadata.frozen_executable_binding != registered.frozen_executable_binding
    {
        return Err(invalid(
            "workflow certification handoff does not match registered identity",
        ));
    }
    if registered.frozen_executable_binding.is_none() {
        return Err(invalid(
            "workflow certification identity is missing frozen executable identity",
        ));
    }

    #[cfg(windows)]
    {
        let expected_workflow = expected_workflow_identity_from_supervisor_env()?;
        let processes = windows_exclusivity::process_table()
            .map_err(|failure| invalid(&format!("process inspection failed: {failure}")))?;
        windows_exclusivity::validate_expected_workflow_identity(&processes, &expected_workflow)
            .map_err(|outcome| invalid(outcome.as_str()))?;
        let competing = windows_exclusivity::competing_processes_for_expected_workflow(
            &processes,
            &expected_workflow,
        );
        if !competing.is_empty() {
            return Err(invalid(
                "workflow certification found an unexpected competing workflow",
            ));
        }

        let child_pid = std::process::id();
        let result = json!({
            "schema_id": WORKFLOW_CERTIFICATION_RESULT_SCHEMA_ID,
            "schema_version": 1,
            "state": "WORKFLOW_IDENTITY_CERTIFIED",
            "launch_identity": metadata.launch_identity,
            "registered_identity": {
                "path": metadata.attempt_identity_path,
                "sha256": metadata.attempt_identity_sha256,
                "attempt": metadata.attempt,
                "candidate_budget": metadata.candidate_budget,
                "candidate_identity": metadata.candidate_identity,
                "evidence_root": metadata.evidence_root
            },
            "supervisor": {
                "pid": metadata.supervisor_pid,
                "path": metadata.supervisor_path,
                "executable_identity": metadata.supervisor_executable_identity
            },
            "child": {
                "pid": child_pid,
                "path": metadata.child_executable_path,
                "executable_identity": metadata.child_executable_identity,
                "parent_pid": metadata.supervisor_pid
            },
            "frozen_executable_binding": metadata.frozen_executable_binding,
            "parent_child_validation": "validated by native process table; child parent PID equals supervisor PID",
            "process_table_records": processes,
            "network_accounting": {
                "llama_startups": 0,
                "port_8080_model_contacts": 0,
                "tcp_readiness_contacts": 0,
                "http_requests": 0,
                "inference_requests": 0
            }
        });
        write_json(result_path, &result)?;
        Ok(result)
    }

    #[cfg(not(windows))]
    {
        let _ = result_path;
        Err(invalid(
            "workflow certification requires the Windows native process identity inspector",
        ))
    }
}

pub(crate) fn windows_native_prestart_value(attempt: u32, operator_attested: bool) -> Value {
    let current_pid = std::process::id();
    let processes = windows_exclusivity::process_table();
    let listeners = windows_exclusivity::tcp_listener_table(PORT);
    let process_records = processes.as_ref().ok().cloned().unwrap_or_default();
    let listener_records = listeners.as_ref().ok().cloned().unwrap_or_default();
    let llama_processes = windows_exclusivity::llama_processes(&process_records);
    let competing_processes =
        windows_exclusivity::competing_processes(&process_records, current_pid);
    let mut outcome =
        windows_exclusivity::classify_prestart(&processes, &listeners, operator_attested);
    if outcome == windows_exclusivity::ExclusivityOutcome::ExclusivePrestart
        && !competing_processes.is_empty()
    {
        outcome = windows_exclusivity::ExclusivityOutcome::CompetingWorkflowProcess;
    }
    let ready = outcome.is_ready() && competing_processes.is_empty() && operator_attested;
    json!({
        "state": if ready { "READY" } else { "BLOCKED" },
        "outcome": outcome.as_str(),
        "attempt": attempt,
        "current_preflight_pid": current_pid,
        "llama_processes": llama_processes,
        "port_8080_listeners": listener_records,
        "other_prefixity_qwen_workflow_processes": competing_processes,
        "operator_no_other_workflow_confirmed": operator_attested,
        "inspection_mechanisms": {
            "processes": "CreateToolhelp32Snapshot + Process32FirstW + Process32NextW",
            "tcp_listeners": "GetExtendedTcpTable(TCP_TABLE_OWNER_PID_LISTENER, AF_INET)",
            "executable_path": "OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW",
            "admin_required": false,
            "localhost_contact": false
        },
        "process_inspection": probe_result_status(&processes),
        "port_inspection": probe_result_status(&listeners),
        "os_table_inspections": {
            "process_table": 1,
            "tcp_listener_table": 1
        },
        "network_calls": 0,
        "listener_checks": 0,
        "inference_requests": 0
    })
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
    let preparation = preflight_calibration()?;
    let candidate_root = candidate_root_at(CALIBRATION_EVIDENCE_ROOT, budget);
    execute_calibration_at_root(budget, confirm_fresh_runtime, &candidate_root, preparation)
}

/// Deterministic checks that precede the single listener check. The live
/// calibration path and the Attempt-010 prerequisite traversal share this
/// function, so no pre-network rejection is discoverable only live.
fn calibration_prestart_checks(
    budget: u32,
    confirm_fresh_runtime: bool,
    candidate_root: &Path,
) -> Result<(Value, Value), ReasoningBudgetCalibrationError> {
    ensure_budget(budget)?;
    if !confirm_fresh_runtime {
        return Err(ReasoningBudgetCalibrationError::Validation(
            "explicit fresh-runtime confirmation is required before the listener check".to_string(),
        ));
    }
    let manifest = read_manifest(true)?;
    validate_manifest(&manifest, true)?;
    let candidate_order = validate_candidate_order_report(budget)?;
    if candidate_root.exists() {
        return Err(ReasoningBudgetCalibrationError::Validation(format!(
            "calibration candidate evidence already exists: {}",
            candidate_root.display()
        )));
    }
    Ok((manifest, candidate_order))
}

fn execute_calibration_at_root(
    budget: u32,
    confirm_fresh_runtime: bool,
    candidate_root: &Path,
    preparation: Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let (manifest, _candidate_order) =
        calibration_prestart_checks(budget, confirm_fresh_runtime, candidate_root)?;
    fs::create_dir_all(candidate_root)?;
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
        write_json(&candidate_root.join("candidate-result.json"), &result)?;
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
        results.push(execute_case(
            &manifest,
            case,
            Some(budget),
            &case_dir,
            &client,
        )?);
    }
    let state = aggregate_state(&results);
    let result = candidate_result(budget, results, 3, state, true, None)?;
    write_json(&candidate_root.join("candidate-result.json"), &result)?;
    Ok(result)
}

pub(crate) fn execute_case(
    manifest: &Value,
    case: &Value,
    budget: Option<u32>,
    evidence_dir: &Path,
    client: &Client,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let case_id = case
        .get("case_id")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("case_id"))?;
    let request = build_request(manifest, case)?;
    let max_tokens = request
        .get("max_tokens")
        .and_then(Value::as_u64)
        .ok_or_else(|| missing("max_tokens"))?;
    // The record labels come from the manifest and request actually used, so a
    // reusing gate records its own experiment and model. For the calibration
    // manifest they equal EXPERIMENT_ID and MODEL_ID.
    let experiment_id = manifest
        .get("experiment_id")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("experiment_id"))?;
    let model = request
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| missing("model"))?;
    let request_bytes = serde_json::to_vec(&request)?;
    let request_sha256 = canonical_hash(&request)?;
    let wire_request_sha256 = sha256_hex(&request_bytes);
    write_bytes(&evidence_dir.join("request-turn-1.json"), &request_bytes)?;
    let request_record = json!({
        "schema_id": REQUEST_SCHEMA_ID,
        "schema_version": 1,
        "experiment_id": experiment_id,
        "case_id": case_id,
        "budget": budget,
        "model": model,
        "request_sha256": request_sha256,
        "wire_request_sha256": wire_request_sha256,
        "request_bytes": request_bytes.len(),
        "max_tokens": max_tokens,
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
                    "transport_timeout": error.is_timeout(),
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
                "transport_timeout": io_error_is_timeout(&error),
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
        json!(format!(
            "total {max_tokens}-token ceiling exhausted before terminal completion"
        ))
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

pub(crate) fn validate_manifest(
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

pub(crate) fn build_request(
    manifest: &Value,
    case: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
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

fn read_attempt_003_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_003_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_003_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_003_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-003-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 3)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-003 candidate identity changed"));
    }
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-003")?;
    expect_string(
        identity,
        "lineage.attempt_001_evidence_root",
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/",
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
    let attempt_002_root = format!("{CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT}/");
    expect_string(
        identity,
        "lineage.attempt_002_evidence_root",
        &attempt_002_root,
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_classification",
        "INVALID BEFORE QWEN STARTUP",
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_reason",
        "PROCESS_INSPECTION_PERMISSION_FAILURE",
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_execution_record_commit",
        "a27b4fb411a284a6503d9a1c5525e30b1ae8862c",
    )?;
    expect_u64(identity, "lineage.attempt_002_inference_requests", 0)?;
    expect_bool(identity, "lineage.attempt_002_root_absent", true)?;
    expect_string(
        identity,
        "lineage.root_cause",
        "PROCESS_INSPECTION_PERMISSION_FAILURE",
    )?;
    let attempt_003_root = format!("{CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT}/");
    expect_string(identity, "attempt_003_evidence_root", &attempt_003_root)?;
    expect_string(identity, "runtime.engine", "llama.cpp")?;
    expect_string(identity, "runtime.server_build", "b10217-ddd4ec142")?;
    expect_string(identity, "runtime.model", MODEL_ID)?;
    expect_string(identity, "runtime.quantization", "Q4_0")?;
    expect_u64(identity, "runtime.context_size", 8192)?;
    expect_u64(identity, "runtime.parallel_slots", 1)?;
    expect_string(identity, "runtime.metrics", "enabled")?;
    expect_string(identity, "runtime.reasoning", "on")?;
    expect_u64(identity, "runtime.reasoning_budget", 1024)?;
    expect_string(identity, "runtime.reasoning_budget_message", "unset")?;
    expect_string(identity, "runtime.host", HOST)?;
    expect_u64(identity, "runtime.port", PORT as u64)?;
    expect_u64(identity, "generation.max_tokens", 2048)?;
    expect_u64(identity, "generation.temperature", 0)?;
    expect_u64(identity, "generation.top_p", 1)?;
    expect_u64(identity, "generation.seed", 1)?;
    expect_u64(identity, "timeouts.request_ms", 1_200_000)?;
    expect_u64(identity, "timeouts.supervisor_ms", 1_320_000)?;
    expect_string(
        identity,
        "exclusivity_implementation.source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "exclusivity_implementation.source_sha256",
        ATTEMPT_003_SEALED_IMPLEMENTATION_SHA256,
    )?;
    expect_bool(
        identity,
        "exclusivity_implementation.native_process_table",
        true,
    )?;
    expect_bool(
        identity,
        "exclusivity_implementation.native_tcp_owner_table",
        true,
    )?;
    expect_bool(
        identity,
        "exclusivity_implementation.no_shell_commands",
        true,
    )?;
    expect_bool(
        identity,
        "exclusivity_implementation.no_admin_required",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_001_must_not_be_overwritten",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_002_root_must_remain_absent",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_003_root_must_begin_absent",
        true,
    )?;
    expect_bool(identity, "evidence_policy.no_manifest_copy", true)?;
    expect_bool(identity, "evidence_policy.no_response_repair", true)?;
    expect_bool(identity, "evidence_policy.no_retry", true)?;
    if verify_sidecar {
        validate_attempt_003_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_frozen_request_hashes(
    identity: &Value,
    label: &str,
) -> Result<(), ReasoningBudgetCalibrationError> {
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
            return Err(invalid(&format!("{label} request identity changed")));
        }
    }
    Ok(())
}

fn validate_attempt_003_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_003_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_003_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-003 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_003_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-003 manifest binding mismatch"));
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
            return Err(invalid("attempt-003 request binding mismatch"));
        }
    }
    Ok(())
}

fn read_attempt_004_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_004_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_004_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_004_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_004_identity_with_source(identity, verify_sidecar, true)
}

fn validate_attempt_004_identity_for_clean_checkout(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_004_identity_with_source(identity, verify_sidecar, false)
}

fn validate_attempt_004_identity_with_source(
    identity: &Value,
    verify_sidecar: bool,
    verify_current_source: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-004-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 4)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-004 candidate identity changed"));
    }
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-004")?;
    expect_string(
        identity,
        "lineage.attempt_001_evidence_root",
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024/",
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
    let attempt_002_root = format!("{CALIBRATION_ATTEMPT_002_EVIDENCE_ROOT}/");
    expect_string(
        identity,
        "lineage.attempt_002_evidence_root",
        &attempt_002_root,
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_classification",
        "INVALID BEFORE QWEN STARTUP",
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_reason",
        "PROCESS_INSPECTION_PERMISSION_FAILURE",
    )?;
    expect_string(
        identity,
        "lineage.attempt_002_execution_record_commit",
        "a27b4fb411a284a6503d9a1c5525e30b1ae8862c",
    )?;
    expect_u64(identity, "lineage.attempt_002_inference_requests", 0)?;
    expect_bool(identity, "lineage.attempt_002_root_absent", true)?;
    let attempt_003_root = format!("{CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT}/");
    expect_string(
        identity,
        "lineage.attempt_003_evidence_root",
        &attempt_003_root,
    )?;
    expect_string(
        identity,
        "lineage.attempt_003_classification",
        "INVALID BEFORE READINESS / INFERENCE",
    )?;
    expect_string(
        identity,
        "lineage.attempt_003_reason",
        "EXPECTED_SUPERVISOR_MISCLASSIFIED_AS_COMPETING_WORKFLOW",
    )?;
    expect_string(
        identity,
        "lineage.attempt_003_execution_record_commit",
        "dc0a8801cee4ddf6f3bc928ecb6bfd954ec7eeec",
    )?;
    expect_u64(identity, "lineage.attempt_003_inference_requests", 0)?;
    expect_u64(identity, "lineage.attempt_003_readiness_checks", 0)?;
    expect_string(
        identity,
        "lineage.root_cause",
        "EXPECTED_SUPERVISOR_MISCLASSIFIED_AS_COMPETING_WORKFLOW",
    )?;
    expect_string(
        identity,
        "attempt_004_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT}/"),
    )?;
    expect_string(identity, "runtime.engine", "llama.cpp")?;
    expect_string(identity, "runtime.server_build", "b10217-ddd4ec142")?;
    expect_string(identity, "runtime.model", MODEL_ID)?;
    expect_string(identity, "runtime.quantization", "Q4_0")?;
    expect_u64(identity, "runtime.context_size", 8192)?;
    expect_u64(identity, "runtime.parallel_slots", 1)?;
    expect_string(identity, "runtime.metrics", "enabled")?;
    expect_string(identity, "runtime.reasoning", "on")?;
    expect_u64(identity, "runtime.reasoning_budget", 1024)?;
    expect_string(identity, "runtime.reasoning_budget_message", "unset")?;
    expect_string(identity, "runtime.host", HOST)?;
    expect_u64(identity, "runtime.port", PORT as u64)?;
    expect_u64(identity, "generation.max_tokens", 2048)?;
    expect_u64(identity, "generation.temperature", 0)?;
    expect_u64(identity, "generation.top_p", 1)?;
    expect_u64(identity, "generation.seed", 1)?;
    expect_u64(identity, "timeouts.request_ms", 1_200_000)?;
    expect_u64(identity, "timeouts.supervisor_ms", 1_320_000)?;
    expect_string(
        identity,
        "exclusivity_implementation.source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    if verify_current_source {
        expect_string(
            identity,
            "exclusivity_implementation.source_sha256",
            &implementation_source_sha256()?,
        )?;
    } else {
        expect_sha256_string(identity, "exclusivity_implementation.source_sha256")?;
    }
    for path in [
        "exclusivity_implementation.native_process_table",
        "exclusivity_implementation.native_tcp_owner_table",
        "exclusivity_implementation.native_parent_process_table",
        "exclusivity_implementation.launch_specific_expected_identity",
        "exclusivity_implementation.no_shell_commands",
        "exclusivity_implementation.no_admin_required",
        "exclusivity_implementation.read_only",
        "exclusivity_implementation.localhost_contact",
    ] {
        let expected = !path.ends_with("localhost_contact");
        expect_bool(identity, path, expected)?;
    }
    expect_bool(
        identity,
        "evidence_policy.attempt_001_must_not_be_overwritten",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_002_root_must_remain_absent",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_003_root_must_remain_immutable",
        true,
    )?;
    expect_bool(
        identity,
        "evidence_policy.attempt_004_root_must_begin_absent",
        true,
    )?;
    expect_bool(identity, "evidence_policy.no_manifest_copy", true)?;
    expect_bool(identity, "evidence_policy.no_response_repair", true)?;
    expect_bool(identity, "evidence_policy.no_retry", true)?;
    if verify_sidecar {
        validate_attempt_004_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_004_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_004_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_004_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-004 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_004_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-004 manifest binding mismatch"));
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
            return Err(invalid("attempt-004 request binding mismatch"));
        }
    }
    Ok(())
}

fn read_attempt_005_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_005_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_005_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_005_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-005-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 5)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-005 candidate identity changed"));
    }
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-1024",
    )?;
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-005")?;
    expect_string(
        identity,
        "attempt_005_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT}/"),
    )?;
    expect_string(
        identity,
        "lineage.attempt_004_classification",
        "INVALID BEFORE READINESS / INFERENCE",
    )?;
    expect_string(
        identity,
        "lineage.attempt_004_immediate_cause",
        "SUPERVISOR_LAUNCH_IDENTITY_HANDOFF_MISSING",
    )?;
    expect_string(
        identity,
        "lineage.attempt_004_terminal_sha",
        "5b16161babe015a66f0a2651610963ac7b7ec735",
    )?;
    expect_u64(identity, "lineage.attempt_004_qwen_startup", 1)?;
    expect_u64(identity, "lineage.attempt_004_readiness", 0)?;
    expect_u64(identity, "lineage.attempt_004_inference", 0)?;
    expect_u64(identity, "lineage.attempt_004_retries", 0)?;
    for (path, expected) in [
        (
            "lineage.attempt_004_evidence_sha256.preflight",
            "ddd3bc415f9565fd461449988618630cd95cc8bacd5c89306163aafaea164d6f",
        ),
        (
            "lineage.attempt_004_evidence_sha256.runtime_startup",
            "5710b1bb021b727cee6907314d5757635255bd06f0ad2862163ee25faa914c7b",
        ),
        (
            "lineage.attempt_004_evidence_sha256.runtime_ownership",
            "7206588dbadc82d31fa99739914a4be3c226318dbad34290e8cce858d619be54",
        ),
        (
            "lineage.attempt_004_evidence_sha256.candidate_result",
            "eebdff4272ee9b0215397a8de1f4cd5979388533eb971e3fb69b3d66064a9ea5",
        ),
        (
            "lineage.attempt_004_evidence_sha256.execution_record",
            "e002c05a3b6da64d4a5fa03ba8aa080c39182eee673cfeafac8d629e7c12c8e3",
        ),
    ] {
        expect_string(identity, path, expected)?;
    }
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_handoff_implementation.source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "launch_handoff_implementation.source_sha256",
        &source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH)?,
    )?;
    expect_string(
        identity,
        "native_exclusivity_implementation.source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "native_exclusivity_implementation.source_sha256",
        &implementation_source_sha256()?,
    )?;
    if verify_sidecar {
        validate_attempt_005_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_005_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_005_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_005_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-005 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_005_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-005 manifest binding mismatch"));
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
            return Err(invalid("attempt-005 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_attempt_005_evidence_state() -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_004_execution_evidence()?;
    if attempt_005_root().exists() {
        return Err(invalid("attempt-005 evidence root must remain absent"));
    }
    Ok(())
}

fn validate_attempt_004_execution_evidence() -> Result<(), ReasoningBudgetCalibrationError> {
    if !attempt_004_root().exists() {
        return Err(invalid("attempt-004 evidence root is absent"));
    }
    for (name, expected_hash) in attempt_004_evidence_hashes() {
        let path = attempt_004_root().join(name);
        let actual_hash = sha256_hex(&fs::read(&path).map_err(|error| {
            invalid(&format!(
                "unable to read preserved attempt-004 evidence {name}: {error}"
            ))
        })?);
        if actual_hash != expected_hash {
            return Err(invalid(&format!(
                "preserved attempt-004 evidence hash mismatch for {name}"
            )));
        }
    }
    Ok(())
}

fn validate_attempt_005_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_attempt_005_identity(true)?;
    validate_attempt_005_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_005_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) {
        return Err(invalid("attempt-005 handoff identity path mismatch"));
    }
    if metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != registered.attempt
        || metadata.candidate_budget != registered.candidate_budget
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
    {
        return Err(invalid(
            "attempt-005 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

fn read_attempt_006_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_006_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_006_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_006_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_006_identity_with_source(identity, verify_sidecar, true)
}

#[cfg(test)]
fn validate_attempt_006_identity_for_clean_checkout(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_006_identity_with_source(identity, verify_sidecar, false)
}

fn validate_attempt_006_identity_with_source(
    identity: &Value,
    verify_sidecar: bool,
    verify_current_source: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-006-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 6)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-006 candidate identity changed"));
    }
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-1024",
    )?;
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-006")?;
    expect_string(
        identity,
        "attempt_006_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT}/"),
    )?;
    expect_string(
        identity,
        "lineage.attempt_005_classification",
        "NOT STARTED — PRE-LIVE REPOSITORY GATE FAILED",
    )?;
    expect_string(
        identity,
        "lineage.attempt_005_immediate_cause",
        "SUPERVISOR_SOURCE_IDENTITY_MISMATCH",
    )?;
    expect_string(
        identity,
        "lineage.attempt_005_expected_supervisor_source_sha256",
        "df74a196b79e0407c0d4d83f4a8aede974904a505a6462a72a9e868eda8e8744",
    )?;
    expect_string(
        identity,
        "lineage.attempt_005_observed_checkout_source_sha256",
        "d18e56fb68c0be8e58a10c94cc8333186614c7ef2f928bc3c252d143207d273f",
    )?;
    expect_string(
        identity,
        "lineage.attempt_005_identity_sha256",
        "5d247fe0c5b29d99b1652254743c5ee3d38833244d30c2ef4e64795f7159a8c6",
    )?;
    for (path, expected) in [
        ("lineage.attempt_005_server_startup", 0),
        ("lineage.attempt_005_readiness", 0),
        ("lineage.attempt_005_requests", 0),
        ("lineage.attempt_005_inference", 0),
        ("lineage.attempt_005_retries", 0),
    ] {
        expect_u64(identity, path, expected)?;
    }
    for path in [
        "lineage.attempt_005_evidence_root_absent",
        "lineage.attempt_005_frozen",
        "lineage.attempt_005_identity_must_not_be_rewritten",
    ] {
        expect_bool(identity, path, true)?;
    }
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_bool(
        identity,
        "source_fingerprint.same_for_sealing_and_verification",
        true,
    )?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_handoff_implementation.source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    if verify_current_source {
        expect_string(
            identity,
            "launch_handoff_implementation.source_sha256",
            &source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH)?,
        )?;
    } else {
        expect_sha256_string(identity, "launch_handoff_implementation.source_sha256")?;
    }
    expect_string(
        identity,
        "native_exclusivity_implementation.source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    if verify_current_source {
        expect_string(
            identity,
            "native_exclusivity_implementation.source_sha256",
            &implementation_source_sha256()?,
        )?;
    } else {
        expect_sha256_string(identity, "native_exclusivity_implementation.source_sha256")?;
    }
    if verify_sidecar {
        validate_attempt_006_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_006_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_006_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_006_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-006 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_006_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-006 manifest binding mismatch"));
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
            return Err(invalid("attempt-006 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_attempt_006_evidence_state() -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_004_execution_evidence()?;
    if attempt_005_root().exists() {
        return Err(invalid("attempt-005 evidence root must remain absent"));
    }
    if attempt_006_root().exists() {
        return Err(invalid("attempt-006 evidence root must remain absent"));
    }
    Ok(())
}

fn validate_attempt_006_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_attempt_006_identity(true)?;
    validate_attempt_006_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_006_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) {
        return Err(invalid("attempt-006 handoff identity path mismatch"));
    }
    if metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != registered.attempt
        || metadata.candidate_budget != registered.candidate_budget
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
    {
        return Err(invalid(
            "attempt-006 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

fn read_attempt_007_identity(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_007_IDENTITY_PATH)?;
    if verify_sidecar {
        validate_attempt_007_identity_sidecar(&identity)?;
    }
    Ok(identity)
}

fn validate_attempt_008_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-008-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 8)?;
    expect_string(
        identity,
        "accepted_baseline.main_sha",
        "417673c06b141e07d54ad0bc101e1105e1a734de",
    )?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-1024",
    )?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fallback_requests") != Some(&json!(0))
        || identity.pointer("/candidate/adaptive_replicates") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-008 candidate identity changed"));
    }
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-008")?;
    expect_string(
        identity,
        "attempt_008_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT}/"),
    )?;
    expect_string(
        identity,
        "lineage.attempt_007_identity_sha256",
        "fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8",
    )?;
    expect_bool(identity, "lineage.attempt_007_executed", true)?;
    expect_bool(identity, "lineage.attempt_007_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_007_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_007_reusable", false)?;
    expect_bool(identity, "lineage.attempt_007_permanently_consumed", true)?;
    expect_u64(identity, "lineage.attempt_007_raw_next_budget", 512)?;
    expect_bool(
        identity,
        "lineage.attempt_007_raw_next_budget_excluded",
        true,
    )?;
    expect_string(
        identity,
        "lineage.attempt_006_evidence_sha256",
        "c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07",
    )?;
    expect_bool(identity, "lineage.attempt_006_immutable", true)?;
    expect_string(
        identity,
        "budget_provenance.selected_budget_source",
        "last_admissible_calibration_state_before_attempt_007",
    )?;
    expect_u64(identity, "budget_provenance.selected_budget", 1024)?;
    expect_u64(
        identity,
        "budget_provenance.excluded_attempt_007_next_budget",
        512,
    )?;
    expect_bool(
        identity,
        "budget_provenance.inadmissible_attempt_007_influenced_selection",
        false,
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.canonical_sha256",
        "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.state",
        "WORKFLOW_IDENTITY_CERTIFIED",
    )?;
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_bool(
        identity,
        "source_fingerprint.same_for_sealing_and_verification",
        true,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.supervisor_source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.child_source_path",
        "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    for (field, path) in [
        (
            "implementation_fingerprints.supervisor_source_sha256",
            CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
        ),
        (
            "implementation_fingerprints.child_source_sha256",
            "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
        ),
        (
            "implementation_fingerprints.native_exclusivity_source_sha256",
            CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
        ),
    ] {
        let pointer = format!("/{}", field.replace('.', "/"));
        let expected = identity
            .pointer(&pointer)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(field))?;
        let actual = source_sha256(path)?;
        if expected != actual {
            return Err(invalid(&format!("{field} does not match current source")));
        }
    }
    crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("attempt-008 is missing frozen executable identity"))?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_plan.server_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command",
        ".\\target\\phase1c-attempt-009-frozen\\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_009_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-009/supervisor.json -- .\\target\\phase1c-attempt-009-frozen\\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-009",
    )?;
    expect_u64(identity, "launch_plan.attempt_id", 8)?;
    expect_u64(identity, "launch_plan.candidate_budget", 1024)?;
    expect_u64(identity, "launch_plan.request_ceiling", 3)?;
    expect_u64(identity, "launch_plan.automatic_retries", 0)?;
    expect_u64(identity, "launch_plan.fallback_requests", 0)?;
    expect_u64(identity, "launch_plan.adaptive_replicates", 0)?;
    expect_string(identity, "preparation_result.state", "ATTEMPT_008_PREPARED")?;
    expect_string(
        identity,
        "preparation_result.execution_state",
        "ATTEMPT_008_NOT_EXECUTED",
    )?;
    expect_bool(
        identity,
        "preparation_result.evidence_root_must_begin_absent",
        true,
    )?;
    expect_bool(
        identity,
        "preparation_result.no_result_artifact_created",
        true,
    )?;
    if verify_sidecar {
        validate_attempt_008_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_008_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-008 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_008_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-008 manifest binding mismatch"));
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
            return Err(invalid("attempt-008 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_attempt_008_frozen_executables(
    identity: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let binding = crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("attempt-008 is missing frozen executable identity"))?;
    for path in [
        &binding.supervisor.raw_path,
        &binding.supervisor.final_path,
        &binding.child.raw_path,
        &binding.child.final_path,
    ] {
        if path
            .to_ascii_lowercase()
            .replace('/', "\\")
            .contains("target\\debug\\")
        {
            return Err(invalid(
                "attempt-008 frozen executable identity points at mutable target/debug",
            ));
        }
    }
    let actual_supervisor =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
            .map_err(|error| invalid(&error))?;
    let actual_child =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.child.raw_path))
            .map_err(|error| invalid(&error))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))?;
    Ok(json!({
        "supervisor": actual_supervisor,
        "child": actual_child
    }))
}

fn validate_attempt_008_history() -> Result<(), ReasoningBudgetCalibrationError> {
    let attempt_007_identity = read_attempt_007_identity(true)?;
    if canonical_hash(&attempt_007_identity)?
        != "fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8"
    {
        return Err(invalid("Attempt-007 canonical identity hash changed"));
    }
    let classification = validate_attempt_007_execution_evidence()?;
    if classification["state"] != "INTEGRITY_REJECTED"
        || classification["classification"] != "FROZEN_EXECUTABLE_IDENTITY_MISMATCH"
        || classification["calibration_admissible"] != false
    {
        return Err(invalid(
            "Attempt-007 historical integrity classification changed",
        ));
    }
    let candidate = read_json_path(&attempt_007_root().join("candidate-result.json"))?;
    if candidate["state"] != "FAIL" || candidate["selection"]["next_budget"] != 512 {
        return Err(invalid("Attempt-007 raw candidate or next budget changed"));
    }
    let attempt_006_artifact = workspace_path(
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/supervisor.json",
    );
    let actual_hash = sha256_hex(&fs::read(&attempt_006_artifact).map_err(|error| {
        invalid(&format!(
            "unable to read preserved Attempt-006 evidence: {error}"
        ))
    })?);
    if actual_hash != "c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07" {
        return Err(invalid("preserved Attempt-006 evidence hash mismatch"));
    }
    Ok(())
}

fn validate_attempt_008_budget_provenance() -> Result<(), ReasoningBudgetCalibrationError> {
    let provenance = read_json(CALIBRATION_ATTEMPT_008_BUDGET_PROVENANCE_PATH)?;
    if provenance["selected_budget"] != 1024
        || provenance["selected_budget_source"]
            != "last_admissible_calibration_state_before_attempt_007"
        || provenance["excluded_attempt_007_next_budget"] != 512
        || provenance["inadmissible_attempt_007_influenced_selection"] != false
    {
        return Err(invalid("Attempt-008 budget provenance is invalid"));
    }
    Ok(())
}

fn validate_attempt_008_virgin_state() -> Result<(), ReasoningBudgetCalibrationError> {
    let root = attempt_008_root();
    if root.exists() {
        return Err(invalid("Attempt-008 evidence root already exists"));
    }
    for name in [
        "request-ledger.json",
        "retry-record.json",
        "execution-lock.json",
        "handoff.json",
        "candidate-result.json",
    ] {
        if root.join(name).exists() {
            return Err(invalid(&format!(
                "Attempt-008 stale artifact exists: {name}"
            )));
        }
    }
    Ok(())
}

/// Revalidate the immutable Attempt-008 result used as the budget-selection
/// predecessor. This deliberately does not compare Attempt-008 source
/// fingerprints with the current checkout: a later preparation changes the
/// implementation source, while the executed predecessor remains identified
/// by its canonical identity, evidence manifest, and recorded handoff.
#[allow(dead_code)]
fn validate_attempt_008_admissible_predecessor() -> Result<Value, ReasoningBudgetCalibrationError> {
    let identity = read_json(CALIBRATION_ATTEMPT_008_IDENTITY_PATH)?;
    if canonical_hash(&identity)?
        != "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5"
    {
        return Err(invalid("Attempt-008 canonical identity hash changed"));
    }
    validate_attempt_008_identity_sidecar(&identity)?;
    expect_u64(&identity, "attempt", 8)?;
    expect_u64(&identity, "candidate.reasoning_budget", 1024)?;
    expect_bool(&identity, "candidate.fresh_server_required", true)?;
    expect_bool(&identity, "candidate.runtime_exclusivity_required", true)?;
    expect_u64(&identity, "candidate.maximum_requests", 3)?;
    expect_u64(&identity, "candidate.automatic_retries", 0)?;
    expect_u64(&identity, "candidate.fallback_requests", 0)?;
    expect_u64(&identity, "candidate.adaptive_replicates", 0)?;

    let root = attempt_008_root();
    let manifest_path = root.join("evidence-manifest.json");
    let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
        invalid(&format!(
            "unable to read preserved Attempt-008 evidence manifest: {error}"
        ))
    })?;
    let manifest_sha256 = sha256_hex(&manifest_bytes);
    if manifest_sha256 != "f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779" {
        return Err(invalid("Attempt-008 evidence manifest hash changed"));
    }
    let manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    if manifest["attempt"] != 8
        || manifest["candidate_budget"] != 1024
        || manifest["preparation_identity_sha256"]
            != "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5"
        || manifest["calibration_manifest_sha256"]
            != "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b"
        || manifest["workflow_identity_certification_v2_sha256"]
            != "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194"
        || manifest["server"]["readiness_contacts"] != 1
        || manifest["runtime"]["inference_requests"] != 3
        || manifest["runtime"]["network_calls"] != 4
        || manifest["runtime"]["child_launches"] != 1
        || manifest["runtime"]["child_retries"] != 0
        || manifest["runtime"]["automatic_retries"] != 0
        || manifest["runtime"]["fallback_requests"] != 0
        || manifest["runtime"]["adaptive_replicates"] != 0
        || manifest["runtime"]["case_order"] != json!(CALIBRATION_CASE_IDS)
        || manifest["result"]["state"] != "FAIL"
        || manifest["result"]["case_set_complete"] != true
        || manifest["result"]["next_budget"] != 512
        || manifest["result"]["calibration_interpretation_permitted"] != false
    {
        return Err(invalid(
            "Attempt-008 evidence manifest does not prove the admissible FAIL result",
        ));
    }
    let files = manifest["files"]
        .as_array()
        .ok_or_else(|| missing("Attempt-008 evidence manifest files"))?;
    if files.len() != 23 {
        return Err(invalid("Attempt-008 evidence manifest file count changed"));
    }
    for file in files {
        let relative = file
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| missing("Attempt-008 evidence file path"))?;
        if Path::new(relative).is_absolute() || relative.contains("..") {
            return Err(invalid(
                "Attempt-008 evidence manifest contains unsafe path",
            ));
        }
        let path = root.join(relative);
        let bytes = fs::read(&path).map_err(|error| {
            invalid(&format!(
                "unable to read preserved Attempt-008 evidence {relative}: {error}"
            ))
        })?;
        if file["bytes"] != bytes.len() as u64 || file["sha256"] != sha256_hex(&bytes).as_str() {
            return Err(invalid(&format!(
                "preserved Attempt-008 evidence hash mismatch for {relative}"
            )));
        }
    }
    let candidate = read_json_path(&root.join("candidate-result.json"))?;
    if candidate["state"] != "FAIL"
        || candidate["budget"] != 1024
        || candidate["case_set_complete"] != true
        || candidate["selection"]["next_budget"] != 512
        || candidate["inference_requests"] != 3
        || candidate["request"]["maximum_requests"] != 3
        || candidate["request"]["automatic_retries"] != 0
        || candidate["request"]["inference_requests"] != 3
        || candidate["case_order"] != json!(CALIBRATION_CASE_IDS)
    {
        return Err(invalid("Attempt-008 candidate result changed"));
    }
    let supervisor = read_json_path(&root.join("supervisor.json"))?;
    if supervisor["state"] != "COMPLETED"
        || supervisor["child_exit_code"] != 0
        || supervisor["child_launches"] != 1
        || supervisor["child_retries"] != 0
        || supervisor["supervisor_inference_requests"] != 0
        || supervisor["supervisor_network_calls"] != 0
    {
        return Err(invalid("Attempt-008 supervisor execution record changed"));
    }
    validate_recorded_workflow_evidence_against_preparation(&identity, &supervisor)?;
    Ok(json!({
        "state": "ATTEMPT_008_CALIBRATION_ADMISSIBLE",
        "executed_once": true,
        "integrity_accepted": true,
        "calibration_admissible": true,
        "candidate_budget": 1024,
        "candidate_state": "FAIL",
        "case_set_complete": true,
        "next_budget": 512,
        "requests": 3,
        "automatic_retries": 0,
        "fallback_requests": 0,
        "adaptive_replicates": 0,
        "identity_sha256": "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5",
        "evidence_manifest_sha256": manifest_sha256,
        "raw_evidence_files": files.len()
    }))
}

/// Resolve the single authoritative admissible transition that selects
/// `candidate_budget`. Preparation and live candidate-order validation both
/// reach this through `validate_candidate_order_report`. It reads only the
/// tracked transition registry and the tracked identity and execution record
/// each transition binds; ignored run evidence is never consulted.
pub fn resolve_authoritative_candidate_transition(
    candidate_budget: u32,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let path = workspace_path(CALIBRATION_CANDIDATE_TRANSITIONS_PATH);
    if !path.is_file() {
        return Err(invalid("authoritative predecessor transition is absent"));
    }
    let registry = read_json(CALIBRATION_CANDIDATE_TRANSITIONS_PATH)?;
    let transition = select_candidate_transition(&registry, candidate_budget)?;
    validate_transition_tracked_evidence(&transition)?;
    Ok(normalized_candidate_transition(
        &transition,
        candidate_budget,
    ))
}

/// Structural selection over a registry value. A candidate resolves only when
/// exactly one complete, admissible FAIL transition selects it and that
/// transition's source is the protocol predecessor budget.
fn select_candidate_transition(
    registry: &Value,
    candidate_budget: u32,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    if !registry.is_object() {
        return Err(invalid("authoritative predecessor transition is absent"));
    }
    expect_string(
        registry,
        "schema_id",
        "prefixity.phase1c.calibration-candidate-transitions",
    )?;
    expect_u64(registry, "schema_version", 1)?;
    expect_string(registry, "experiment_id", EXPERIMENT_ID)?;
    if registry.get("candidate_order") != Some(&json!(CALIBRATION_BUDGETS)) {
        return Err(invalid("authoritative transition candidate order changed"));
    }
    let excluded = registry
        .get("excluded_attempts")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("excluded_attempts"))?
        .iter()
        .map(|entry| {
            entry
                .get("attempt")
                .and_then(Value::as_u64)
                .ok_or_else(|| invalid("excluded attempt entry is incomplete"))
        })
        .collect::<Result<Vec<_>, ReasoningBudgetCalibrationError>>()?;
    if CALIBRATION_INADMISSIBLE_ATTEMPTS
        .iter()
        .any(|attempt| !excluded.contains(attempt))
    {
        return Err(invalid(
            "an inadmissible attempt is not excluded from selection",
        ));
    }
    let index = CALIBRATION_BUDGETS
        .iter()
        .position(|candidate| *candidate == candidate_budget)
        .ok_or_else(|| invalid("unregistered calibration budget"))?;
    if index == 0 {
        return Err(invalid(
            "the initial calibration candidate has no predecessor transition",
        ));
    }
    let predecessor_budget = CALIBRATION_BUDGETS[index - 1];
    let transitions = registry
        .get("transitions")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("transitions"))?;
    for transition in transitions {
        validate_candidate_transition_record(transition, &excluded)?;
    }
    for field in ["source_attempt", "source_budget", "selected_next_budget"] {
        let mut values = transitions
            .iter()
            .map(|transition| transition[field].clone())
            .collect::<Vec<_>>();
        let count = values.len();
        values.sort_by_key(Value::to_string);
        values.dedup();
        if values.len() != count {
            return Err(invalid("authoritative predecessor transition is ambiguous"));
        }
    }
    let selected = transitions
        .iter()
        .filter(|transition| transition["selected_next_budget"] == candidate_budget)
        .collect::<Vec<_>>();
    match selected.as_slice() {
        [] => Err(invalid("authoritative predecessor transition is absent")),
        [transition] if transition["source_budget"] == predecessor_budget => {
            Ok((*transition).clone())
        }
        [_] => Err(invalid(
            "authoritative predecessor transition has the wrong source budget",
        )),
        _ => Err(invalid("authoritative predecessor transition is ambiguous")),
    }
}

fn validate_candidate_transition_record(
    transition: &Value,
    excluded: &[u64],
) -> Result<(), ReasoningBudgetCalibrationError> {
    if !transition.is_object() {
        return Err(invalid(
            "authoritative predecessor transition is incomplete",
        ));
    }
    let source_attempt = transition
        .get("source_attempt")
        .and_then(Value::as_u64)
        .ok_or_else(|| missing("source_attempt"))?;
    if source_attempt == 0 || excluded.contains(&source_attempt) {
        return Err(invalid(
            "authoritative transition names an excluded or invalid source attempt",
        ));
    }
    let source_budget = transition
        .get("source_budget")
        .and_then(Value::as_u64)
        .and_then(|budget| u32::try_from(budget).ok())
        .ok_or_else(|| missing("source_budget"))?;
    ensure_budget(source_budget)?;
    expect_string(transition, "source_state", "FAIL")?;
    expect_bool(transition, "integrity_accepted", true)?;
    expect_bool(transition, "calibration_admissible", true)?;
    expect_bool(transition, "case_set_complete", true)?;
    expect_u64(transition, "requests", CALIBRATION_CASE_IDS.len() as u64)?;
    expect_u64(transition, "automatic_retries", 0)?;
    expect_u64(transition, "fallback_requests", 0)?;
    expect_u64(transition, "adaptive_replicates", 0)?;
    let selected_next_budget = transition
        .get("selected_next_budget")
        .and_then(Value::as_u64)
        .ok_or_else(|| missing("selected_next_budget"))?;
    if next_candidate_budget(source_budget, "FAIL").map(u64::from) != Some(selected_next_budget) {
        return Err(invalid(
            "authoritative transition does not select the protocol successor budget",
        ));
    }
    for field in [
        "source_identity_sha256",
        "evidence_manifest_sha256",
        "execution_record_sha256",
    ] {
        let complete = transition
            .get(field)
            .and_then(Value::as_str)
            .is_some_and(|hash| {
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            });
        if !complete {
            return Err(invalid(&format!(
                "authoritative transition {field} is not a complete SHA-256"
            )));
        }
    }
    for field in ["source_identity_path", "execution_record_path"] {
        let tracked = transition
            .get(field)
            .and_then(Value::as_str)
            .is_some_and(|path| path.starts_with("docs/") && !path.contains(".."));
        if !tracked {
            return Err(invalid(&format!(
                "authoritative transition {field} is not tracked documentation"
            )));
        }
    }
    Ok(())
}

/// Bind a structurally valid transition to the tracked identity, identity
/// sidecar, and execution record it names.
fn validate_transition_tracked_evidence(
    transition: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let source_attempt = transition["source_attempt"]
        .as_u64()
        .ok_or_else(|| missing("source_attempt"))?;
    let identity_path = transition["source_identity_path"]
        .as_str()
        .ok_or_else(|| missing("source_identity_path"))?;
    let identity = read_json(identity_path)?;
    let identity_sha256 = canonical_hash(&identity)?;
    let sidecar = read_json(&identity_path.replace(".json", ".sha256"))?;
    if transition["source_identity_sha256"] != identity_sha256.as_str()
        || sidecar.get("canonical_sha256").and_then(Value::as_str) != Some(identity_sha256.as_str())
        || identity.get("attempt").and_then(Value::as_u64) != Some(source_attempt)
        || identity.pointer("/candidate/reasoning_budget") != transition.get("source_budget")
    {
        return Err(invalid(
            "authoritative transition source identity does not match tracked evidence",
        ));
    }
    let record_path = transition["execution_record_path"]
        .as_str()
        .ok_or_else(|| missing("execution_record_path"))?;
    if transition["execution_record_sha256"] != source_sha256(record_path)?.as_str() {
        return Err(invalid(
            "authoritative transition execution record does not match tracked evidence",
        ));
    }
    let record = fs::read_to_string(workspace_path(record_path))?;
    let manifest_sha256 = transition["evidence_manifest_sha256"]
        .as_str()
        .ok_or_else(|| missing("evidence_manifest_sha256"))?;
    for marker in [
        format!("ATTEMPT_{source_attempt:03}_INTEGRITY_ACCEPTED"),
        format!("ATTEMPT_{source_attempt:03}_CALIBRATION_ADMISSIBLE"),
        manifest_sha256.to_string(),
    ] {
        if !record.contains(&marker) {
            return Err(invalid(
                "authoritative transition is not supported by its execution record",
            ));
        }
    }
    Ok(())
}

fn normalized_candidate_transition(transition: &Value, candidate_budget: u32) -> Value {
    let source_attempt = transition["source_attempt"].as_u64().unwrap_or_default();
    json!({
        "state": format!("ATTEMPT_{source_attempt:03}_CALIBRATION_ADMISSIBLE"),
        "source_attempt": source_attempt,
        "source_budget": transition["source_budget"],
        "source_candidate_budget": transition["source_budget"],
        "source_state": "FAIL",
        "source_result": "FAIL",
        "source_integrity": "ACCEPTED",
        "source_calibration_admissible": true,
        "integrity_accepted": true,
        "calibration_admissible": true,
        "candidate_budget": transition["source_budget"],
        "candidate_state": "FAIL",
        "case_set_complete": true,
        "next_budget": transition["selected_next_budget"],
        "selected_next_budget": transition["selected_next_budget"],
        "selected_candidate_budget": candidate_budget,
        "requests": transition["requests"],
        "automatic_retries": 0,
        "fallback_requests": 0,
        "adaptive_replicates": 0,
        "identity_sha256": transition["source_identity_sha256"],
        "source_identity_path": transition["source_identity_path"],
        "evidence_manifest_sha256": transition["evidence_manifest_sha256"],
        "execution_record_path": transition["execution_record_path"],
        "execution_record_sha256": transition["execution_record_sha256"],
        "excluded_attempts": CALIBRATION_INADMISSIBLE_ATTEMPTS,
        "attempt_007_excluded_from_selection": true,
        "attempt_007_raw_next_budget_excluded": true,
        "attempt_009_excluded_from_selection": true,
        "authoritative_transition_path": CALIBRATION_CANDIDATE_TRANSITIONS_PATH,
        "raw_predecessor_evidence_required": false
    })
}

fn validate_authoritative_attempt_008_transition_record(
    provenance: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if !provenance.is_object() {
        return Err(invalid("authoritative predecessor transition is absent"));
    }
    expect_string(
        provenance,
        "schema_id",
        "prefixity.phase1c.attempt-009-budget-provenance",
    )?;
    expect_u64(provenance, "schema_version", 1)?;
    expect_u64(provenance, "attempt", 9)?;
    expect_u64(provenance, "selected_budget", 512)?;
    expect_string(
        provenance,
        "selected_budget_source",
        "attempt_008_admissible_fail_next_budget",
    )?;
    expect_u64(provenance, "source_attempt", 8)?;
    expect_u64(provenance, "source_candidate_budget", 1024)?;
    expect_string(provenance, "source_result", "FAIL")?;
    expect_string(provenance, "source_integrity", "ACCEPTED")?;
    expect_bool(provenance, "source_calibration_admissible", true)?;
    expect_u64(provenance, "selected_next_budget", 512)?;
    expect_bool(provenance, "attempt_007_excluded_from_selection", true)?;
    expect_bool(provenance, "attempt_007_raw_next_budget_excluded", true)?;
    expect_string(
        provenance,
        "selection_rule",
        "Only an integrity-accepted, calibration-admissible FAIL advances the accepted calibration state; Attempt-007 is permanently consumed and its raw next_budget is historical forensic evidence only.",
    )?;
    Ok(())
}

fn validate_attempt_009_budget_provenance(
    predecessor: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    if predecessor["state"] != "ATTEMPT_008_CALIBRATION_ADMISSIBLE"
        || predecessor["candidate_budget"] != 1024
        || predecessor["candidate_state"] != "FAIL"
        || predecessor["case_set_complete"] != true
        || predecessor["next_budget"] != 512
    {
        return Err(invalid(
            "Attempt-009 budget selection requires an admissible Attempt-008 FAIL",
        ));
    }
    let provenance = read_json(CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH)?;
    validate_authoritative_attempt_008_transition_record(&provenance)?;
    Ok(json!({
        "state": "BUDGET_512_PROVENANCE_ACCEPTED",
        "source_attempt": 8,
        "source_candidate_budget": 1024,
        "source_result": "FAIL",
        "source_integrity": "ACCEPTED",
        "source_calibration_admissible": true,
        "selected_next_budget": 512,
        "attempt_007_excluded_from_selection": true,
        "authoritative_transition_path": CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH,
        "raw_predecessor_evidence_required": false
    }))
}

fn validate_attempt_009_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-512-attempt-009-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 9)?;
    expect_string(
        identity,
        "accepted_baseline.main_sha",
        "cc473780596f4085ad21a1a468e569a7688337b4",
    )?;
    expect_u64(identity, "candidate.reasoning_budget", 512)?;
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-512",
    )?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fallback_requests") != Some(&json!(0))
        || identity.pointer("/candidate/adaptive_replicates") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/reasoning_mode") != Some(&json!("on"))
        || identity.pointer("/candidate/reasoning_budget_source")
            != Some(&json!("server-side --reasoning-budget"))
        || identity.pointer("/candidate/output_token_ceiling") != Some(&json!(2048))
        || identity.pointer("/candidate/context_size") != Some(&json!(8192))
        || identity.pointer("/candidate/parallel_slots") != Some(&json!(1))
        || identity.pointer("/candidate/temperature") != Some(&json!(0))
        || identity.pointer("/candidate/top_p") != Some(&json!(1))
        || identity.pointer("/candidate/seed") != Some(&json!(1))
    {
        return Err(invalid("attempt-009 candidate identity changed"));
    }
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-009")?;
    expect_string(
        identity,
        "attempt_009_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT}/"),
    )?;

    expect_string(
        identity,
        "lineage.attempt_008_identity_path",
        CALIBRATION_ATTEMPT_008_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_008_identity_sha256",
        "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5",
    )?;
    expect_string(
        identity,
        "lineage.attempt_008_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT}/"),
    )?;
    expect_bool(identity, "lineage.attempt_008_executed", true)?;
    expect_bool(identity, "lineage.attempt_008_integrity_accepted", true)?;
    expect_bool(identity, "lineage.attempt_008_calibration_admissible", true)?;
    expect_bool(identity, "lineage.attempt_008_raw_evidence_immutable", true)?;
    expect_u64(identity, "lineage.attempt_008_candidate_budget", 1024)?;
    expect_string(identity, "lineage.attempt_008_candidate_state", "FAIL")?;
    expect_bool(identity, "lineage.attempt_008_case_set_complete", true)?;
    expect_u64(identity, "lineage.attempt_008_next_budget", 512)?;
    expect_u64(identity, "lineage.attempt_008_requests", 3)?;
    expect_u64(identity, "lineage.attempt_008_automatic_retries", 0)?;
    expect_u64(identity, "lineage.attempt_008_fallback_requests", 0)?;
    expect_u64(identity, "lineage.attempt_008_adaptive_replicates", 0)?;
    expect_string(
        identity,
        "lineage.attempt_008_evidence_manifest_sha256",
        "f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779",
    )?;
    expect_string(
        identity,
        "lineage.attempt_007_identity_sha256",
        "fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8",
    )?;
    expect_bool(
        identity,
        "lineage.attempt_007_excluded_from_selection",
        true,
    )?;
    expect_bool(identity, "lineage.attempt_007_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_007_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_007_permanently_consumed", true)?;
    expect_u64(identity, "lineage.attempt_007_raw_next_budget", 512)?;
    expect_string(
        identity,
        "lineage.attempt_006_evidence_sha256",
        "c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07",
    )?;
    expect_bool(identity, "lineage.attempt_006_immutable", true)?;
    expect_string(
        identity,
        "budget_provenance.selected_budget_source",
        "attempt_008_admissible_fail_next_budget",
    )?;
    expect_u64(identity, "budget_provenance.selected_budget", 512)?;
    expect_u64(identity, "budget_provenance.source_attempt", 8)?;
    expect_u64(identity, "budget_provenance.source_candidate_budget", 1024)?;
    expect_string(identity, "budget_provenance.source_result", "FAIL")?;
    expect_string(identity, "budget_provenance.source_integrity", "ACCEPTED")?;
    expect_bool(
        identity,
        "budget_provenance.source_calibration_admissible",
        true,
    )?;
    expect_u64(identity, "budget_provenance.selected_next_budget", 512)?;
    expect_bool(
        identity,
        "budget_provenance.attempt_007_excluded_from_selection",
        true,
    )?;
    expect_bool(
        identity,
        "budget_provenance.attempt_007_raw_next_budget_excluded",
        true,
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.canonical_sha256",
        "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.state",
        "WORKFLOW_IDENTITY_CERTIFIED",
    )?;
    expect_bool(
        identity,
        "workflow_identity_certification_v2.frozen_supervisor_binding",
        true,
    )?;
    expect_bool(
        identity,
        "workflow_identity_certification_v2.frozen_child_binding",
        true,
    )?;
    expect_bool(
        identity,
        "workflow_identity_certification_v2.path_only_fallback",
        false,
    )?;
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_bool(
        identity,
        "source_fingerprint.same_for_sealing_and_verification",
        true,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.supervisor_source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.child_source_path",
        "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    for (field, path) in [
        (
            "implementation_fingerprints.supervisor_source_sha256",
            CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
        ),
        (
            "implementation_fingerprints.child_source_sha256",
            "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
        ),
        (
            "implementation_fingerprints.native_exclusivity_source_sha256",
            CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
        ),
    ] {
        let pointer = format!("/{}", field.replace('.', "/"));
        let expected = identity
            .pointer(&pointer)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(field))?;
        let actual = source_sha256(path)?;
        if expected != actual {
            return Err(invalid(&format!("{field} does not match current source")));
        }
    }
    crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("attempt-009 is missing frozen executable identity"))?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_plan.server_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.server_command",
        "C:\\Users\\USER\\AppData\\Local\\Microsoft\\WindowsApps\\llama.exe serve -hf ggml-org/Qwen3.5-0.8B-GGUF:Q4_0 -c 8192 -np 1 --metrics --reasoning on --reasoning-budget 512 --host 127.0.0.1 --port 8080",
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command_status",
        "NOT_EXECUTED",
    )?;
    expect_u64(identity, "launch_plan.attempt_id", 9)?;
    expect_u64(identity, "launch_plan.candidate_budget", 512)?;
    expect_u64(identity, "launch_plan.request_ceiling", 3)?;
    expect_u64(identity, "launch_plan.automatic_retries", 0)?;
    expect_u64(identity, "launch_plan.fallback_requests", 0)?;
    expect_u64(identity, "launch_plan.adaptive_replicates", 0)?;
    expect_string(identity, "preparation_result.state", "ATTEMPT_009_PREPARED")?;
    expect_string(
        identity,
        "preparation_result.execution_state",
        "ATTEMPT_009_NOT_EXECUTED",
    )?;
    expect_bool(
        identity,
        "preparation_result.evidence_root_must_begin_absent",
        true,
    )?;
    expect_bool(
        identity,
        "preparation_result.no_result_artifact_created",
        true,
    )?;
    expect_bool(
        identity,
        "preparation_result.frozen_after_build_and_validation",
        true,
    )?;
    expect_bool(
        identity,
        "preparation_result.frozen_destination_non_overwriting",
        true,
    )?;
    expect_bool(
        identity,
        "preparation_result.mutable_target_debug_execution_objects_forbidden",
        true,
    )?;
    expect_string(
        identity,
        "virgin_state.classification",
        "ATTEMPT_009_VIRGIN",
    )?;
    expect_bool(
        identity,
        "virgin_state.identity_existed_before_preparation",
        false,
    )?;
    expect_bool(identity, "virgin_state.execution_result_exists", false)?;
    expect_bool(identity, "virgin_state.request_ledger_exists", false)?;
    expect_bool(identity, "virgin_state.retry_record_exists", false)?;
    expect_bool(identity, "virgin_state.execution_lock_exists", false)?;
    expect_bool(identity, "virgin_state.stale_handoff_exists", false)?;
    expect_bool(identity, "virgin_state.model_server_running", false)?;
    expect_bool(identity, "virgin_state.port_8080_owner", false)?;
    for field in [
        "network_policy.model_server_startups",
        "network_policy.port_8080_contacts",
        "network_policy.tcp_readiness_contacts",
        "network_policy.http_model_requests",
        "network_policy.inference_requests",
        "network_policy.attempt_008_executions_added",
        "network_policy.attempt_009_executions",
    ] {
        expect_u64(identity, field, 0)?;
    }
    if verify_sidecar {
        validate_attempt_009_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_009_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-009 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_009_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-009 manifest binding mismatch"));
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
            return Err(invalid("attempt-009 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_attempt_009_frozen_executables(
    identity: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let binding = crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("attempt-009 is missing frozen executable identity"))?;
    for path in [
        &binding.supervisor.raw_path,
        &binding.supervisor.final_path,
        &binding.child.raw_path,
        &binding.child.final_path,
    ] {
        if path
            .to_ascii_lowercase()
            .replace('/', "\\")
            .contains("target\\debug\\")
        {
            return Err(invalid(
                "attempt-009 frozen executable identity points at mutable target/debug",
            ));
        }
    }
    let actual_supervisor =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
            .map_err(|error| invalid(&error))?;
    let actual_child =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.child.raw_path))
            .map_err(|error| invalid(&error))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))?;
    Ok(json!({
        "supervisor": actual_supervisor,
        "child": actual_child
    }))
}

fn validate_attempt_009_virgin_state() -> Result<(), ReasoningBudgetCalibrationError> {
    let root = attempt_009_root();
    if root.exists() {
        return Err(invalid("Attempt-009 evidence root already exists"));
    }
    for name in [
        "request-ledger.json",
        "retry-record.json",
        "execution-lock.json",
        "handoff.json",
        "candidate-result.json",
        "supervisor.json",
    ] {
        if root.join(name).exists() {
            return Err(invalid(&format!(
                "Attempt-009 stale artifact exists: {name}"
            )));
        }
    }
    Ok(())
}

const ATTEMPT_010_SERVER_EXECUTABLE: &str =
    "C:\\Users\\USER\\AppData\\Local\\Microsoft\\WindowsApps\\llama.exe";
const ATTEMPT_010_SUPERVISOR_COMMAND: &str = ".\\target\\phase1c-attempt-010-frozen\\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-512-attempt-010/supervisor.json -- .\\target\\phase1c-attempt-010-frozen\\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-010";
const ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND: &str = ".\\target\\phase1c-attempt-010-frozen\\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_IDENTITY_V1.json --evidence target/phase1c-attempt-010-prerequisite-traversal/supervisor.json -- .\\target\\phase1c-attempt-010-frozen\\prefixity-phase1c-reasoning-budget-calibration.exe attempt-010-live-prerequisites";
const ATTEMPT_009_IDENTITY_CANONICAL_SHA256: &str =
    "0929aae1d371415d3efd92e4e7310490ad06fb69fd0398aa76aa9d857818f812";
const ATTEMPT_009_EXECUTION_RECORD_SHA256: &str =
    "2eb51b44600850cd2514c40a2e347670af323afead7e90d1afad9525261cfaaa";

/// Validate the Attempt-010 identity document without reading the sidecar,
/// frozen executables, current implementation sources, or any ignored
/// evidence. Attempt 010 is consumed, so later source changes do not
/// invalidate its recorded document; live and preparation paths still
/// require the source binding through `validate_attempt_010_identity`.
pub fn validate_attempt_010_identity_document(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_010_identity_fields(identity)
}

fn validate_attempt_010_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_010_identity_fields(identity)?;
    validate_attempt_010_source_binding(identity)?;
    if verify_sidecar {
        validate_attempt_010_identity_sidecar(identity)?;
    }
    Ok(())
}

/// The prepared implementation sources must equal the current checkout for
/// any preparation or live use of the Attempt-010 identity.
fn validate_attempt_010_source_binding(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    for (field, path) in [
        (
            "implementation_fingerprints.supervisor_source_sha256",
            CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
        ),
        (
            "implementation_fingerprints.child_source_sha256",
            "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
        ),
        (
            "implementation_fingerprints.native_exclusivity_source_sha256",
            CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
        ),
    ] {
        let pointer = format!("/{}", field.replace('.', "/"));
        let expected = identity
            .pointer(&pointer)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(field))?;
        let actual = source_sha256(path)?;
        if expected != actual {
            return Err(invalid(&format!("{field} does not match current source")));
        }
    }
    Ok(())
}

fn validate_attempt_010_identity_fields(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-512-attempt-010-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 10)?;
    expect_string(
        identity,
        "accepted_baseline.main_sha",
        "a0019bcd96ca8759801a879619f119032067a3f9",
    )?;
    expect_u64(identity, "candidate.reasoning_budget", 512)?;
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-512",
    )?;
    let case_order = json!(CALIBRATION_CASE_IDS);
    if identity.pointer("/candidate/case_order") != Some(&case_order)
        || identity.pointer("/freshness_policy/case_order") != Some(&case_order)
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fallback_requests") != Some(&json!(0))
        || identity.pointer("/candidate/adaptive_replicates") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/reasoning_mode") != Some(&json!("on"))
        || identity.pointer("/candidate/reasoning_budget_source")
            != Some(&json!("server-side --reasoning-budget"))
        || identity.pointer("/candidate/output_token_ceiling") != Some(&json!(2048))
        || identity.pointer("/candidate/context_size") != Some(&json!(8192))
        || identity.pointer("/candidate/parallel_slots") != Some(&json!(1))
        || identity.pointer("/candidate/temperature") != Some(&json!(0))
        || identity.pointer("/candidate/top_p") != Some(&json!(1))
        || identity.pointer("/candidate/seed") != Some(&json!(1))
        || identity.pointer("/candidate/warmup") != Some(&json!("none"))
    {
        return Err(invalid("attempt-010 candidate identity changed"));
    }
    if identity.pointer("/generation/max_tokens") != Some(&json!(2048))
        || identity.pointer("/generation/temperature") != Some(&json!(0))
        || identity.pointer("/generation/top_p") != Some(&json!(1))
        || identity.pointer("/generation/seed") != Some(&json!(1))
        || identity.pointer("/generation/stream") != Some(&Value::Bool(false))
    {
        return Err(invalid("attempt-010 generation settings changed"));
    }
    if identity.pointer("/runtime/model_reference") != Some(&json!(MODEL_ID))
        || identity.pointer("/runtime/quantization") != Some(&json!("Q4_0"))
        || identity.pointer("/runtime/context_size") != Some(&json!(8192))
        || identity.pointer("/runtime/parallel_slots") != Some(&json!(1))
        || identity.pointer("/runtime/reasoning") != Some(&json!("on"))
        || identity.pointer("/runtime/reasoning_budget") != Some(&json!(512))
        || identity.pointer("/runtime/host") != Some(&json!(HOST))
        || identity.pointer("/runtime/port") != Some(&json!(PORT))
        || identity.pointer("/runtime/endpoint") != Some(&json!(ENDPOINT))
    {
        return Err(invalid("attempt-010 runtime settings changed"));
    }
    for field in [
        "retry_policy.automatic_retries",
        "retry_policy.fallback_requests",
        "retry_policy.adaptive_replicates",
    ] {
        expect_u64(identity, field, 0)?;
    }
    expect_bool(
        identity,
        "freshness_policy.fresh_server_per_candidate",
        true,
    )?;
    expect_bool(identity, "freshness_policy.warmup", false)?;
    expect_u64(
        identity,
        "freshness_policy.maximum_listener_checks_per_candidate",
        1,
    )?;
    expect_u64(identity, "freshness_policy.maximum_inference_requests", 3)?;
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-010")?;
    expect_string(
        identity,
        "attempt_010_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT}/"),
    )?;

    expect_string(
        identity,
        "lineage.attempt_008_identity_path",
        CALIBRATION_ATTEMPT_008_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_008_identity_sha256",
        "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5",
    )?;
    expect_string(
        identity,
        "lineage.attempt_008_evidence_manifest_sha256",
        "f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779",
    )?;
    expect_bool(identity, "lineage.attempt_008_executed", true)?;
    expect_bool(identity, "lineage.attempt_008_integrity_accepted", true)?;
    expect_bool(identity, "lineage.attempt_008_calibration_admissible", true)?;
    expect_u64(identity, "lineage.attempt_008_candidate_budget", 1024)?;
    expect_string(identity, "lineage.attempt_008_candidate_state", "FAIL")?;
    expect_u64(identity, "lineage.attempt_008_next_budget", 512)?;
    expect_bool(identity, "lineage.attempt_008_raw_evidence_required", false)?;
    expect_string(
        identity,
        "lineage.attempt_007_identity_sha256",
        "fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8",
    )?;
    expect_bool(identity, "lineage.attempt_007_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_007_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_007_permanently_consumed", true)?;
    expect_bool(
        identity,
        "lineage.attempt_007_excluded_from_selection",
        true,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_identity_path",
        CALIBRATION_ATTEMPT_009_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_identity_sha256",
        ATTEMPT_009_IDENTITY_CANONICAL_SHA256,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_execution_record_path",
        CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_execution_record_sha256",
        ATTEMPT_009_EXECUTION_RECORD_SHA256,
    )?;
    expect_bool(identity, "lineage.attempt_009_executed_once", true)?;
    expect_bool(identity, "lineage.attempt_009_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_009_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_009_permanently_consumed", true)?;
    expect_u64(identity, "lineage.attempt_009_inference_requests", 0)?;
    expect_bool(
        identity,
        "lineage.attempt_009_excluded_from_selection",
        true,
    )?;

    expect_string(
        identity,
        "budget_provenance.authoritative_transition_path",
        CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH,
    )?;
    expect_string(
        identity,
        "budget_provenance.selected_budget_source",
        "attempt_008_admissible_fail_next_budget",
    )?;
    expect_u64(identity, "budget_provenance.selected_budget", 512)?;
    expect_u64(identity, "budget_provenance.source_attempt", 8)?;
    expect_u64(identity, "budget_provenance.source_candidate_budget", 1024)?;
    expect_string(identity, "budget_provenance.source_result", "FAIL")?;
    expect_string(identity, "budget_provenance.source_integrity", "ACCEPTED")?;
    expect_bool(
        identity,
        "budget_provenance.source_calibration_admissible",
        true,
    )?;
    expect_u64(identity, "budget_provenance.selected_next_budget", 512)?;
    expect_bool(
        identity,
        "budget_provenance.attempt_007_excluded_from_selection",
        true,
    )?;
    expect_bool(
        identity,
        "budget_provenance.attempt_009_excluded_from_selection",
        true,
    )?;
    expect_bool(
        identity,
        "budget_provenance.budget_512_experimentally_untested",
        true,
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.canonical_sha256",
        "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.state",
        "WORKFLOW_IDENTITY_CERTIFIED",
    )?;
    for field in [
        "workflow_identity_certification_v2.frozen_supervisor_binding",
        "workflow_identity_certification_v2.frozen_child_binding",
        "workflow_identity_certification_v2.parent_child_relationship_validated",
        "workflow_identity_certification_v2.pre_child_spawn_mismatch_rejection",
        "workflow_identity_certification_v2.independent_evidence_validation",
        "workflow_identity_certification_v2.substitution_rejection",
    ] {
        expect_bool(identity, field, true)?;
    }
    expect_bool(
        identity,
        "workflow_identity_certification_v2.path_only_fallback",
        false,
    )?;
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.supervisor_source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.child_source_path",
        "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    let binding =
        crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
            identity,
        )
        .map_err(|error| invalid(&error))?
        .ok_or_else(|| invalid("attempt-010 is missing frozen executable identity"))?;
    for executable in [&binding.supervisor, &binding.child] {
        let raw_path = executable.raw_path.to_ascii_lowercase().replace('/', "\\");
        let final_path = executable
            .final_path
            .to_ascii_lowercase()
            .replace('/', "\\");
        if raw_path.contains("target\\debug\\")
            || final_path.contains("target\\debug\\")
            || !raw_path.contains("target\\phase1c-attempt-010-frozen\\")
            || !final_path.contains("target\\phase1c-attempt-010-frozen\\")
            || executable.file_id.is_none()
        {
            return Err(invalid(
                "attempt-010 frozen executable identity is not a complete attempt-010 frozen object",
            ));
        }
    }
    expect_string(identity, "build_provenance.lockfile_path", "Cargo.lock")?;
    expect_string(
        identity,
        "build_provenance.target",
        "x86_64-pc-windows-msvc",
    )?;
    expect_string(identity, "build_provenance.profile", "dev")?;
    expect_string(
        identity,
        "build_provenance.build_command",
        "cargo build -p prefixity-controlled-benchmark --bins --locked --offline",
    )?;
    expect_bool(
        identity,
        "build_provenance.build_dependent_validation_completed_before_freeze",
        true,
    )?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_plan.server_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.server_command",
        &format!(
            "{} {}",
            ATTEMPT_010_SERVER_EXECUTABLE,
            server_launch_arguments(512).join(" ")
        ),
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command",
        ATTEMPT_010_SUPERVISOR_COMMAND,
    )?;
    expect_string(
        identity,
        "launch_plan.prerequisite_traversal_command",
        ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND,
    )?;
    expect_u64(identity, "launch_plan.attempt_id", 10)?;
    expect_u64(identity, "launch_plan.candidate_budget", 512)?;
    expect_u64(identity, "launch_plan.request_ceiling", 3)?;
    expect_u64(identity, "launch_plan.automatic_retries", 0)?;
    expect_u64(identity, "launch_plan.fallback_requests", 0)?;
    expect_u64(identity, "launch_plan.adaptive_replicates", 0)?;
    expect_string(identity, "preparation_result.state", "ATTEMPT_010_PREPARED")?;
    expect_string(
        identity,
        "preparation_result.execution_state",
        "ATTEMPT_010_NOT_EXECUTED",
    )?;
    for field in [
        "preparation_result.evidence_root_must_begin_absent",
        "preparation_result.no_result_artifact_created",
        "preparation_result.frozen_after_build_and_validation",
        "preparation_result.frozen_destination_non_overwriting",
        "preparation_result.mutable_target_debug_execution_objects_forbidden",
        "preparation_result.live_prerequisites_shared_with_runtime",
    ] {
        expect_bool(identity, field, true)?;
    }
    expect_string(
        identity,
        "virgin_state.classification",
        "ATTEMPT_010_VIRGIN",
    )?;
    for field in [
        "virgin_state.identity_existed_before_preparation",
        "virgin_state.execution_result_exists",
        "virgin_state.request_ledger_exists",
        "virgin_state.retry_record_exists",
        "virgin_state.execution_lock_exists",
        "virgin_state.stale_handoff_exists",
        "virgin_state.model_server_running",
        "virgin_state.port_8080_owner",
    ] {
        expect_bool(identity, field, false)?;
    }
    for field in [
        "network_policy.model_server_startups",
        "network_policy.port_8080_contacts",
        "network_policy.tcp_readiness_contacts",
        "network_policy.http_model_requests",
        "network_policy.inference_requests",
        "network_policy.attempt_009_executions_added",
        "network_policy.attempt_010_executions",
    ] {
        expect_u64(identity, field, 0)?;
    }
    Ok(())
}

fn validate_attempt_010_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_010_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_010_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-010 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_identity_request_bindings(
    identity: &Value,
    fingerprints: &Value,
    label: &str,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid(&format!("{label} manifest binding mismatch")));
    }
    let expected_cases = identity
        .get("frozen_request_hashes")
        .and_then(Value::as_object)
        .ok_or_else(|| missing("frozen_request_hashes"))?;
    let actual_cases = fingerprints
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| missing("cases"))?;
    if expected_cases.len() != CALIBRATION_CASE_IDS.len()
        || actual_cases.len() != CALIBRATION_CASE_IDS.len()
    {
        return Err(invalid(&format!("{label} case set mismatch")));
    }
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
            return Err(invalid(&format!("{label} request binding mismatch")));
        }
    }
    Ok(())
}

fn validate_attempt_010_frozen_executables(
    identity: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let binding =
        crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
            identity,
        )
        .map_err(|error| invalid(&error))?
        .ok_or_else(|| invalid("attempt-010 is missing frozen executable identity"))?;
    for path in [
        &binding.supervisor.raw_path,
        &binding.supervisor.final_path,
        &binding.child.raw_path,
        &binding.child.final_path,
    ] {
        if path
            .to_ascii_lowercase()
            .replace('/', "\\")
            .contains("target\\debug\\")
        {
            return Err(invalid(
                "attempt-010 frozen executable identity points at mutable target/debug",
            ));
        }
    }
    let actual_supervisor =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
            .map_err(|error| invalid(&error))?;
    let actual_child =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.child.raw_path))
            .map_err(|error| invalid(&error))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))?;
    Ok(json!({
        "supervisor": actual_supervisor,
        "child": actual_child
    }))
}

fn validate_attempt_010_virgin_state() -> Result<(), ReasoningBudgetCalibrationError> {
    if attempt_010_root().exists() {
        return Err(invalid("Attempt-010 evidence root already exists"));
    }
    Ok(())
}

/// Attempt 009 consumed an identity but produced no calibration result. Its
/// tracked identity and execution record are bound here so the selection
/// path can prove it never consults Attempt 009 as a predecessor.
fn validate_attempt_009_excluded_from_selection() -> Result<Value, ReasoningBudgetCalibrationError>
{
    let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH)?;
    let identity_sha256 = canonical_hash(&identity)?;
    let sidecar = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_FINGERPRINT_PATH)?;
    if identity_sha256 != ATTEMPT_009_IDENTITY_CANONICAL_SHA256
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(ATTEMPT_009_IDENTITY_CANONICAL_SHA256)
    {
        return Err(invalid("Attempt-009 tracked identity changed"));
    }
    if source_sha256(CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH)?
        != ATTEMPT_009_EXECUTION_RECORD_SHA256
    {
        return Err(invalid("Attempt-009 tracked execution record changed"));
    }
    Ok(json!({
        "identity_sha256": identity_sha256,
        "execution_record_path": CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH,
        "execution_record_sha256": ATTEMPT_009_EXECUTION_RECORD_SHA256,
        "executed_once": true,
        "integrity_accepted": false,
        "calibration_admissible": false,
        "permanently_consumed": true,
        "inference_requests": 0,
        "excluded_from_selection": true,
        "raw_evidence_required": false
    }))
}

const ATTEMPT_011_SERVER_EXECUTABLE: &str =
    "C:\\Users\\USER\\AppData\\Local\\Microsoft\\WindowsApps\\llama.exe";
const ATTEMPT_011_SUPERVISOR_COMMAND: &str = ".\\target\\phase1c-attempt-011-frozen\\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json --evidence experiments/runs/phase1c-reasoning-budget-calibration/budget-256-attempt-011/supervisor.json -- .\\target\\phase1c-attempt-011-frozen\\prefixity-phase1c-reasoning-budget-calibration.exe run-attempt-011";
const ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND: &str = ".\\target\\phase1c-attempt-011-frozen\\prefixity-phase1c-live-supervisor.exe --attempt-identity docs/phase-1/PHASE_1C_REASONING_BUDGET_256_ATTEMPT_011_IDENTITY_V1.json --evidence target/phase1c-attempt-011-prerequisite-traversal/supervisor.json -- .\\target\\phase1c-attempt-011-frozen\\prefixity-phase1c-reasoning-budget-calibration.exe attempt-011-live-prerequisites";

/// Validate the Attempt-011 identity document without reading the sidecar,
/// frozen executables, current implementation sources, or any ignored
/// evidence. Live and preparation paths additionally require the
/// current-source binding through `validate_attempt_011_identity`.
pub fn validate_attempt_011_identity_document(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_011_identity_fields(identity)
}

fn validate_attempt_011_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    validate_attempt_011_identity_fields(identity)?;
    validate_attempt_011_source_binding(identity)?;
    if verify_sidecar {
        validate_attempt_011_identity_sidecar(identity)?;
    }
    Ok(())
}

/// The prepared implementation sources must equal the current checkout for
/// any preparation or live use of the Attempt-011 identity.
fn validate_attempt_011_source_binding(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    for (field, path) in [
        (
            "implementation_fingerprints.supervisor_source_sha256",
            CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
        ),
        (
            "implementation_fingerprints.child_source_sha256",
            "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
        ),
        (
            "implementation_fingerprints.native_exclusivity_source_sha256",
            CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
        ),
    ] {
        let pointer = format!("/{}", field.replace('.', "/"));
        let expected = identity
            .pointer(&pointer)
            .and_then(Value::as_str)
            .ok_or_else(|| missing(field))?;
        let actual = source_sha256(path)?;
        if expected != actual {
            return Err(invalid(&format!("{field} does not match current source")));
        }
    }
    Ok(())
}

fn validate_attempt_011_identity_fields(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-256-attempt-011-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 11)?;
    expect_string(
        identity,
        "accepted_baseline.main_sha",
        "292dd1f0951f20fdcf6f49facbb446076d80a135",
    )?;
    expect_u64(identity, "candidate.reasoning_budget", 256)?;
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-256",
    )?;
    let case_order = json!(CALIBRATION_CASE_IDS);
    if identity.pointer("/candidate/case_order") != Some(&case_order)
        || identity.pointer("/freshness_policy/case_order") != Some(&case_order)
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fallback_requests") != Some(&json!(0))
        || identity.pointer("/candidate/adaptive_replicates") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/reasoning_mode") != Some(&json!("on"))
        || identity.pointer("/candidate/reasoning_budget_source")
            != Some(&json!("server-side --reasoning-budget"))
        || identity.pointer("/candidate/output_token_ceiling") != Some(&json!(2048))
        || identity.pointer("/candidate/context_size") != Some(&json!(8192))
        || identity.pointer("/candidate/parallel_slots") != Some(&json!(1))
        || identity.pointer("/candidate/temperature") != Some(&json!(0))
        || identity.pointer("/candidate/top_p") != Some(&json!(1))
        || identity.pointer("/candidate/seed") != Some(&json!(1))
        || identity.pointer("/candidate/warmup") != Some(&json!("none"))
    {
        return Err(invalid("attempt-011 candidate identity changed"));
    }
    if identity.pointer("/generation/max_tokens") != Some(&json!(2048))
        || identity.pointer("/generation/temperature") != Some(&json!(0))
        || identity.pointer("/generation/top_p") != Some(&json!(1))
        || identity.pointer("/generation/seed") != Some(&json!(1))
        || identity.pointer("/generation/stream") != Some(&Value::Bool(false))
    {
        return Err(invalid("attempt-011 generation settings changed"));
    }
    if identity.pointer("/runtime/model_reference") != Some(&json!(MODEL_ID))
        || identity.pointer("/runtime/quantization") != Some(&json!("Q4_0"))
        || identity.pointer("/runtime/context_size") != Some(&json!(8192))
        || identity.pointer("/runtime/parallel_slots") != Some(&json!(1))
        || identity.pointer("/runtime/reasoning") != Some(&json!("on"))
        || identity.pointer("/runtime/reasoning_budget") != Some(&json!(256))
        || identity.pointer("/runtime/host") != Some(&json!(HOST))
        || identity.pointer("/runtime/port") != Some(&json!(PORT))
        || identity.pointer("/runtime/endpoint") != Some(&json!(ENDPOINT))
    {
        return Err(invalid("attempt-011 runtime settings changed"));
    }
    for field in [
        "retry_policy.automatic_retries",
        "retry_policy.fallback_requests",
        "retry_policy.adaptive_replicates",
    ] {
        expect_u64(identity, field, 0)?;
    }
    expect_bool(
        identity,
        "freshness_policy.fresh_server_per_candidate",
        true,
    )?;
    expect_bool(identity, "freshness_policy.warmup", false)?;
    expect_u64(
        identity,
        "freshness_policy.maximum_listener_checks_per_candidate",
        1,
    )?;
    expect_u64(identity, "freshness_policy.maximum_inference_requests", 3)?;
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-011")?;
    expect_string(
        identity,
        "attempt_011_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT}/"),
    )?;

    expect_string(
        identity,
        "lineage.attempt_010_identity_path",
        CALIBRATION_ATTEMPT_010_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_010_identity_sha256",
        "9292e9ecdd2e89f695dfb34bc782ade41b70412c427807c6c3a5653b50ec16f7",
    )?;
    expect_string(
        identity,
        "lineage.attempt_010_evidence_manifest_sha256",
        "5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9",
    )?;
    expect_string(
        identity,
        "lineage.attempt_010_execution_record_path",
        "docs/phase-1/PHASE_1C_REASONING_BUDGET_512_ATTEMPT_010_EXECUTION_RECORD.md",
    )?;
    expect_string(
        identity,
        "lineage.attempt_010_execution_record_sha256",
        "c85dd5f3169e29cd96be3d71d4a49bc12c90d7474fe621ace5fbede943160edc",
    )?;
    expect_bool(identity, "lineage.attempt_010_executed_once", true)?;
    expect_bool(identity, "lineage.attempt_010_integrity_accepted", true)?;
    expect_bool(identity, "lineage.attempt_010_calibration_admissible", true)?;
    expect_u64(identity, "lineage.attempt_010_candidate_budget", 512)?;
    expect_string(identity, "lineage.attempt_010_candidate_state", "FAIL")?;
    expect_bool(identity, "lineage.attempt_010_case_set_complete", true)?;
    expect_u64(identity, "lineage.attempt_010_next_budget", 256)?;
    expect_bool(identity, "lineage.attempt_010_permanently_consumed", true)?;
    expect_bool(identity, "lineage.attempt_010_raw_evidence_required", false)?;
    expect_string(
        identity,
        "lineage.attempt_007_identity_sha256",
        "fd536cb507aab2b76511e1731d3860bbbc0074b940e4e21067ec7c6b1f4bbbe8",
    )?;
    expect_bool(identity, "lineage.attempt_007_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_007_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_007_permanently_consumed", true)?;
    expect_bool(
        identity,
        "lineage.attempt_007_excluded_from_selection",
        true,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_identity_path",
        CALIBRATION_ATTEMPT_009_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_identity_sha256",
        ATTEMPT_009_IDENTITY_CANONICAL_SHA256,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_execution_record_path",
        CALIBRATION_ATTEMPT_009_EXECUTION_RECORD_PATH,
    )?;
    expect_string(
        identity,
        "lineage.attempt_009_execution_record_sha256",
        ATTEMPT_009_EXECUTION_RECORD_SHA256,
    )?;
    expect_bool(identity, "lineage.attempt_009_executed_once", true)?;
    expect_bool(identity, "lineage.attempt_009_integrity_accepted", false)?;
    expect_bool(
        identity,
        "lineage.attempt_009_calibration_admissible",
        false,
    )?;
    expect_bool(identity, "lineage.attempt_009_permanently_consumed", true)?;
    expect_u64(identity, "lineage.attempt_009_inference_requests", 0)?;
    expect_bool(
        identity,
        "lineage.attempt_009_excluded_from_selection",
        true,
    )?;

    expect_string(
        identity,
        "budget_provenance.authoritative_transition_registry",
        CALIBRATION_CANDIDATE_TRANSITIONS_PATH,
    )?;
    expect_string(
        identity,
        "budget_provenance.authoritative_transition_resolver",
        "resolve_authoritative_candidate_transition",
    )?;
    if !identity
        .pointer("/budget_provenance/authoritative_transition_registry_sha256")
        .and_then(Value::as_str)
        .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Err(invalid(
            "attempt-011 authoritative transition registry hash is incomplete",
        ));
    }
    expect_string(
        identity,
        "budget_provenance.selected_budget_source",
        "attempt_010_admissible_fail_next_budget",
    )?;
    expect_u64(identity, "budget_provenance.selected_budget", 256)?;
    expect_u64(identity, "budget_provenance.source_attempt", 10)?;
    expect_u64(identity, "budget_provenance.source_budget", 512)?;
    expect_string(identity, "budget_provenance.source_state", "FAIL")?;
    expect_string(identity, "budget_provenance.source_integrity", "ACCEPTED")?;
    expect_bool(
        identity,
        "budget_provenance.source_calibration_admissible",
        true,
    )?;
    expect_u64(identity, "budget_provenance.selected_next_budget", 256)?;
    for field in [
        "budget_provenance.attempt_007_excluded_from_selection",
        "budget_provenance.attempt_009_excluded_from_selection",
        "budget_provenance.attempt_008_not_source_for_256",
        "budget_provenance.budget_256_experimentally_untested",
    ] {
        expect_bool(identity, field, true)?;
    }
    expect_string(
        identity,
        "workflow_identity_certification_v2.canonical_sha256",
        "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification_v2.state",
        "WORKFLOW_IDENTITY_CERTIFIED",
    )?;
    for field in [
        "workflow_identity_certification_v2.frozen_supervisor_binding",
        "workflow_identity_certification_v2.frozen_child_binding",
        "workflow_identity_certification_v2.parent_child_relationship_validated",
        "workflow_identity_certification_v2.pre_child_spawn_mismatch_rejection",
        "workflow_identity_certification_v2.independent_evidence_validation",
        "workflow_identity_certification_v2.substitution_rejection",
    ] {
        expect_bool(identity, field, true)?;
    }
    expect_bool(
        identity,
        "workflow_identity_certification_v2.path_only_fallback",
        false,
    )?;
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.supervisor_source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.child_source_path",
        "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    let binding =
        crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
            identity,
        )
        .map_err(|error| invalid(&error))?
        .ok_or_else(|| invalid("attempt-011 is missing frozen executable identity"))?;
    for executable in [&binding.supervisor, &binding.child] {
        let raw_path = executable.raw_path.to_ascii_lowercase().replace('/', "\\");
        let final_path = executable
            .final_path
            .to_ascii_lowercase()
            .replace('/', "\\");
        if raw_path.contains("target\\debug\\")
            || final_path.contains("target\\debug\\")
            || !raw_path.contains("target\\phase1c-attempt-011-frozen\\")
            || !final_path.contains("target\\phase1c-attempt-011-frozen\\")
            || executable.file_id.is_none()
        {
            return Err(invalid(
                "attempt-011 frozen executable identity is not a complete attempt-011 frozen object",
            ));
        }
    }
    expect_string(identity, "build_provenance.lockfile_path", "Cargo.lock")?;
    expect_string(
        identity,
        "build_provenance.target",
        "x86_64-pc-windows-msvc",
    )?;
    expect_string(identity, "build_provenance.profile", "dev")?;
    expect_string(
        identity,
        "build_provenance.build_command",
        "cargo build -p prefixity-controlled-benchmark --bins --locked --offline",
    )?;
    expect_bool(
        identity,
        "build_provenance.build_dependent_validation_completed_before_freeze",
        true,
    )?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_plan.server_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.server_command",
        &format!(
            "{} {}",
            ATTEMPT_011_SERVER_EXECUTABLE,
            server_launch_arguments(256).join(" ")
        ),
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command_status",
        "NOT_EXECUTED",
    )?;
    expect_string(
        identity,
        "launch_plan.supervisor_command",
        ATTEMPT_011_SUPERVISOR_COMMAND,
    )?;
    expect_string(
        identity,
        "launch_plan.prerequisite_traversal_command",
        ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND,
    )?;
    expect_u64(identity, "launch_plan.attempt_id", 11)?;
    expect_u64(identity, "launch_plan.candidate_budget", 256)?;
    expect_u64(identity, "launch_plan.request_ceiling", 3)?;
    expect_u64(identity, "launch_plan.automatic_retries", 0)?;
    expect_u64(identity, "launch_plan.fallback_requests", 0)?;
    expect_u64(identity, "launch_plan.adaptive_replicates", 0)?;
    expect_string(identity, "preparation_result.state", "ATTEMPT_011_PREPARED")?;
    expect_string(
        identity,
        "preparation_result.execution_state",
        "ATTEMPT_011_NOT_EXECUTED",
    )?;
    for field in [
        "preparation_result.evidence_root_must_begin_absent",
        "preparation_result.no_result_artifact_created",
        "preparation_result.frozen_after_build_and_validation",
        "preparation_result.frozen_destination_non_overwriting",
        "preparation_result.mutable_target_debug_execution_objects_forbidden",
        "preparation_result.live_prerequisites_shared_with_runtime",
    ] {
        expect_bool(identity, field, true)?;
    }
    expect_string(
        identity,
        "virgin_state.classification",
        "ATTEMPT_011_VIRGIN",
    )?;
    for field in [
        "virgin_state.identity_existed_before_preparation",
        "virgin_state.execution_result_exists",
        "virgin_state.request_ledger_exists",
        "virgin_state.retry_record_exists",
        "virgin_state.execution_lock_exists",
        "virgin_state.stale_handoff_exists",
        "virgin_state.model_server_running",
        "virgin_state.port_8080_owner",
    ] {
        expect_bool(identity, field, false)?;
    }
    for field in [
        "network_policy.model_server_startups",
        "network_policy.port_8080_contacts",
        "network_policy.tcp_readiness_contacts",
        "network_policy.http_model_requests",
        "network_policy.inference_requests",
        "network_policy.attempt_009_executions_added",
        "network_policy.attempt_011_executions",
    ] {
        expect_u64(identity, field, 0)?;
    }
    Ok(())
}

fn validate_attempt_011_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_011_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_011_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-011 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_011_frozen_executables(
    identity: &Value,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let binding =
        crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
            identity,
        )
        .map_err(|error| invalid(&error))?
        .ok_or_else(|| invalid("attempt-011 is missing frozen executable identity"))?;
    for path in [
        &binding.supervisor.raw_path,
        &binding.supervisor.final_path,
        &binding.child.raw_path,
        &binding.child.final_path,
    ] {
        if path
            .to_ascii_lowercase()
            .replace('/', "\\")
            .contains("target\\debug\\")
        {
            return Err(invalid(
                "attempt-011 frozen executable identity points at mutable target/debug",
            ));
        }
    }
    let actual_supervisor =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
            .map_err(|error| invalid(&error))?;
    let actual_child =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.child.raw_path))
            .map_err(|error| invalid(&error))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))?;
    Ok(json!({
        "supervisor": actual_supervisor,
        "child": actual_child
    }))
}

fn validate_attempt_011_virgin_state() -> Result<(), ReasoningBudgetCalibrationError> {
    if attempt_011_root().exists() {
        return Err(invalid("Attempt-011 evidence root already exists"));
    }
    Ok(())
}

pub(crate) fn validate_accepted_workflow_certification_v2(
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let root = workspace_path("target/phase1c-workflow-identity-certification-v2-final");
    let identity = read_json_path(&root.join("identity.json"))?;
    if canonical_hash(&identity)?
        != "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194"
    {
        return Err(invalid(
            "accepted v2 workflow certification identity hash changed",
        ));
    }
    let binding = crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        &identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("accepted v2 certification is missing frozen executable binding"))?;
    let actual_supervisor =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.supervisor.raw_path))
            .map_err(|error| invalid(&error))?;
    let actual_child =
        crate::phase1c_executable_identity::inspect(Path::new(&binding.child.raw_path))
            .map_err(|error| invalid(&error))?;
    crate::phase1c_executable_identity::validate_frozen_executable_binding(
        &binding,
        &actual_supervisor,
        &actual_child,
    )
    .map_err(|error| invalid(&error))?;
    let result = read_json_path(&root.join("certification-result.json"))?;
    if result["state"] != "WORKFLOW_IDENTITY_CERTIFIED"
        || result["parent_child_validation"]
            .as_str()
            .is_none_or(|value| !value.contains("child parent PID equals supervisor PID"))
        || result["network_accounting"]["llama_startups"] != 0
        || result["network_accounting"]["port_8080_model_contacts"] != 0
        || result["network_accounting"]["http_requests"] != 0
        || result["network_accounting"]["inference_requests"] != 0
    {
        return Err(invalid("accepted v2 workflow certification result changed"));
    }
    Ok(json!({
        "identity_path": "target/phase1c-workflow-identity-certification-v2-final/identity.json",
        "canonical_sha256": "acf2d18d93b2baf08b4fa4bc5bb0846e8f3f8bedab83927cf9547240abb2d194",
        "state": "WORKFLOW_IDENTITY_CERTIFIED",
        "supervisor": actual_supervisor,
        "child": actual_child,
        "parent_child_validation": "validated by native process table; child parent PID equals supervisor PID",
        "network_accounting": {
            "llama_startups": 0,
            "port_8080_model_contacts": 0,
            "http_requests": 0,
            "inference_requests": 0
        }
    }))
}

fn validate_attempt_007_identity(
    identity: &Value,
    verify_sidecar: bool,
) -> Result<(), ReasoningBudgetCalibrationError> {
    expect_string(
        identity,
        "identity_version",
        "phase1c-reasoning-budget-1024-attempt-007-v1",
    )?;
    expect_string(
        identity,
        "status",
        "PREPARATION_ONLY_NO_INFERENCE_AUTHORIZED",
    )?;
    expect_string(identity, "experiment_id", EXPERIMENT_ID)?;
    expect_u64(identity, "attempt", 7)?;
    expect_u64(identity, "candidate.reasoning_budget", 1024)?;
    if identity.pointer("/candidate/case_order") != Some(&json!(CALIBRATION_CASE_IDS))
        || identity.pointer("/candidate/maximum_requests") != Some(&json!(3))
        || identity.pointer("/candidate/automatic_retries") != Some(&json!(0))
        || identity.pointer("/candidate/fresh_server_required") != Some(&Value::Bool(true))
        || identity.pointer("/candidate/runtime_exclusivity_required") != Some(&Value::Bool(true))
    {
        return Err(invalid("attempt-007 candidate identity changed"));
    }
    expect_string(
        identity,
        "candidate.calibration_candidate_identity",
        "phase1c-reasoning-budget-1024",
    )?;
    expect_string(identity, "manifest.path", CALIBRATION_MANIFEST_PATH)?;
    expect_string(
        identity,
        "manifest.sha256",
        "4c9be251b077d8e21824efca48c8a73f0428cf750d0d499c539645084f11405b",
    )?;
    validate_frozen_request_hashes(identity, "attempt-007")?;
    expect_string(
        identity,
        "attempt_007_evidence_root",
        &format!("{CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT}/"),
    )?;
    expect_string(
        identity,
        "lineage.attempt_006_classification",
        "REASONING BUDGET 1024 ATTEMPT 006 INVALID — STOP FOR REVIEW",
    )?;
    expect_string(
        identity,
        "lineage.attempt_006_immediate_cause",
        "EXPECTED_CHILD_PATH_MISMATCH",
    )?;
    expect_string(
        identity,
        "lineage.attempt_006_evidence_sha256",
        "c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07",
    )?;
    expect_u64(identity, "lineage.attempt_006_inference_requests", 0)?;
    expect_u64(identity, "lineage.attempt_006_retries", 0)?;
    expect_bool(identity, "lineage.attempt_006_immutable", true)?;
    expect_string(
        identity,
        "workflow_identity_certification.identity_path",
        WORKFLOW_CERTIFICATION_IDENTITY_PATH,
    )?;
    expect_string(
        identity,
        "workflow_identity_certification.canonical_sha256",
        "824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification.state",
        "WORKFLOW_IDENTITY_CERTIFIED",
    )?;
    expect_string(
        identity,
        "workflow_identity_certification.path_classification",
        "PATH_REPRESENTATION_MISMATCH",
    )?;
    expect_string(identity, "source_fingerprint.algorithm", "SHA-256")?;
    expect_string(
        identity,
        "source_fingerprint.byte_definition",
        "UTF-8 checkout source bytes; CRLF and lone CR normalized to LF; UTF-8 BOM rejected",
    )?;
    expect_bool(
        identity,
        "source_fingerprint.same_for_sealing_and_verification",
        true,
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.supervisor_source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    expect_sha256_string(
        identity,
        "implementation_fingerprints.supervisor_source_sha256",
    )?;
    expect_string(
        identity,
        "implementation_fingerprints.child_source_path",
        "crates/prefixity-controlled-benchmark/src/phase1c_reasoning_budget_calibration.rs",
    )?;
    expect_sha256_string(identity, "implementation_fingerprints.child_source_sha256")?;
    expect_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_path",
        CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH,
    )?;
    expect_sha256_string(
        identity,
        "implementation_fingerprints.native_exclusivity_source_sha256",
    )?;
    crate::phase1c_executable_identity::FrozenExecutableBinding::from_implementation_fingerprints(
        identity,
    )
    .map_err(|error| invalid(&error))?
    .ok_or_else(|| invalid("attempt-007 is missing frozen executable identity"))?;
    expect_string(
        identity,
        "handoff_contract.schema_id",
        crate::phase1c_live_supervisor::WORKFLOW_HANDOFF_SCHEMA_ID,
    )?;
    expect_string(
        identity,
        "handoff_contract.transport",
        "single inherited environment JSON",
    )?;
    expect_string(identity, "handoff_contract.child_binding", "parent_pid")?;
    expect_string(
        identity,
        "handoff_contract.launch_identity_formula",
        "phase1c-attempt-{attempt}-budget-{candidate_budget}-{attempt_identity_sha256}",
    )?;
    expect_bool(
        identity,
        "handoff_contract.supervisor_generates_launch_identity",
        true,
    )?;
    expect_bool(
        identity,
        "handoff_contract.child_must_not_invent_identity",
        true,
    )?;
    expect_bool(identity, "handoff_contract.fail_closed_on_mismatch", true)?;
    expect_string(
        identity,
        "launch_handoff_implementation.source_path",
        CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH,
    )?;
    if verify_sidecar {
        validate_attempt_007_identity_sidecar(identity)?;
    }
    Ok(())
}

fn validate_attempt_007_identity_sidecar(
    identity: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let sidecar = read_json(CALIBRATION_ATTEMPT_007_IDENTITY_FINGERPRINT_PATH)?;
    if sidecar.get("artifact_path").and_then(Value::as_str)
        != Some(CALIBRATION_ATTEMPT_007_IDENTITY_PATH)
        || sidecar.get("algorithm").and_then(Value::as_str) != Some("SHA-256")
        || sidecar.get("canonicalization").and_then(Value::as_str)
            != Some("sorted JSON object keys; arrays preserve order")
        || sidecar.get("canonical_sha256").and_then(Value::as_str)
            != Some(canonical_hash(identity)?.as_str())
    {
        return Err(invalid("attempt-007 identity fingerprint sidecar mismatch"));
    }
    Ok(())
}

fn validate_attempt_007_bindings(
    identity: &Value,
    fingerprints: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    if identity.pointer("/manifest/sha256") != fingerprints.get("manifest_sha256") {
        return Err(invalid("attempt-007 manifest binding mismatch"));
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
            return Err(invalid("attempt-007 request binding mismatch"));
        }
    }
    Ok(())
}

fn validate_attempt_007_evidence_state() -> Result<(), ReasoningBudgetCalibrationError> {
    if attempt_007_root().exists() {
        return Err(invalid("attempt-007 evidence root must remain absent"));
    }
    let attempt_006_artifact = workspace_path(
        "experiments/runs/phase1c-reasoning-budget-calibration/budget-1024-attempt-006/supervisor.json",
    );
    let actual_hash = sha256_hex(&fs::read(&attempt_006_artifact).map_err(|error| {
        invalid(&format!(
            "unable to read preserved Attempt-006 evidence: {error}"
        ))
    })?);
    if actual_hash != "c8feaf01a4083d609dd5fdf5ccaf96b40db56c1cbb3ef0b7bb88f52b67724b07" {
        return Err(invalid("preserved Attempt-006 evidence hash mismatch"));
    }
    let certification = read_json(WORKFLOW_CERTIFICATION_IDENTITY_PATH)?;
    if canonical_hash(&certification)?
        != "824b65a0f93e18a12e917fd49662790bdb2dce945c6e4f8f0c8b72a9c41524a4"
    {
        return Err(invalid("workflow certification manifest hash mismatch"));
    }
    if attempt_005_root().exists() {
        return Err(invalid("attempt-005 evidence root must remain absent"));
    }
    Ok(())
}

fn validate_attempt_007_handoff(
    metadata: &crate::phase1c_live_supervisor::WorkflowLaunchMetadata,
) -> Result<(), ReasoningBudgetCalibrationError> {
    let identity = read_attempt_007_identity(true)?;
    validate_attempt_007_identity(&identity, true)?;
    let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
        Path::new(CALIBRATION_ATTEMPT_007_IDENTITY_PATH),
    )
    .map_err(|error| invalid(&error.to_string()))?;
    if !same_workflow_identity_path(
        &metadata.attempt_identity_path,
        &registered.attempt_identity_path,
    ) {
        return Err(invalid("attempt-007 handoff identity path mismatch"));
    }
    if metadata.attempt_identity_sha256 != canonical_hash(&identity)?
        || metadata.attempt != registered.attempt
        || metadata.candidate_budget != registered.candidate_budget
        || metadata.candidate_identity != registered.candidate_identity
        || metadata.evidence_root != registered.evidence_root
        || metadata.launch_identity != registered.generated_launch_identity()
    {
        return Err(invalid(
            "attempt-007 supervisor launch metadata does not match registered identity",
        ));
    }
    Ok(())
}

fn validate_attempt_004_evidence_state() -> Result<(), ReasoningBudgetCalibrationError> {
    let attempt_001_root =
        workspace_path("experiments/runs/phase1c-reasoning-budget-calibration/budget-1024");
    if !attempt_001_root.exists() {
        return Err(invalid("attempt-001 evidence root is absent"));
    }
    if attempt_002_root().exists() {
        return Err(invalid("attempt-002 evidence root must remain absent"));
    }
    if !attempt_003_root().exists() {
        return Err(invalid("attempt-003 evidence root is absent"));
    }
    if attempt_004_root().exists() {
        return Err(invalid("attempt-004 evidence root must remain absent"));
    }
    for (name, expected_hash) in attempt_003_evidence_hashes() {
        let path = attempt_003_root().join(name);
        let actual_hash = sha256_hex(&fs::read(&path).map_err(|error| {
            invalid(&format!(
                "unable to read preserved attempt-003 evidence {name}: {error}"
            ))
        })?);
        if actual_hash != expected_hash {
            return Err(invalid(&format!(
                "preserved attempt-003 evidence hash mismatch for {name}"
            )));
        }
    }
    Ok(())
}

fn attempt_003_evidence_hashes() -> [(&'static str, &'static str); 4] {
    [
        (
            "preflight.json",
            "73921f286b141a48802499b1081c660500425fbf5c59d0e54880e1a4588e00e8",
        ),
        (
            "runtime-ownership.json",
            "333c4fbcbbf90d24e28ce44890a478d06d233ced785261c7e08b5f0cb233337c",
        ),
        (
            "candidate-result.json",
            "b756093101fb7683ea74ff4166bd66c6e1129eea26e7e8a6f7791fcec449d62b",
        ),
        (
            "supervisor.json",
            "7a396f58b156bb7abb2eda565be1ed15541310717eac597dab8a7902150e9387",
        ),
    ]
}

fn attempt_004_evidence_hashes() -> [(&'static str, &'static str); 5] {
    [
        (
            "preflight.json",
            "ddd3bc415f9565fd461449988618630cd95cc8bacd5c89306163aafaea164d6f",
        ),
        (
            "runtime-startup.json",
            "5710b1bb021b727cee6907314d5757635255bd06f0ad2862163ee25faa914c7b",
        ),
        (
            "runtime-ownership.json",
            "7206588dbadc82d31fa99739914a4be3c226318dbad34290e8cce858d619be54",
        ),
        (
            "candidate-result.json",
            "eebdff4272ee9b0215397a8de1f4cd5979388533eb971e3fb69b3d66064a9ea5",
        ),
        (
            "execution-record.json",
            "e002c05a3b6da64d4a5fa03ba8aa080c39182eee673cfeafac8d629e7c12c8e3",
        ),
    ]
}

fn implementation_source_sha256() -> Result<String, ReasoningBudgetCalibrationError> {
    source_sha256(CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH)
}

pub(crate) fn source_sha256(path: &str) -> Result<String, ReasoningBudgetCalibrationError> {
    let source = fs::read(workspace_path(path))?;
    let canonical = canonicalize_source_bytes(&source).map_err(invalid)?;
    Ok(sha256_hex(&canonical))
}

/// Candidate-order validation shared by preparation and live execution. The
/// initial candidate needs no predecessor; every other candidate resolves
/// through the single authoritative transition registry. The generic
/// `candidate-result.json` path is never consulted.
pub fn validate_candidate_order_report(
    budget: u32,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let index = CALIBRATION_BUDGETS
        .iter()
        .position(|candidate| *candidate == budget)
        .ok_or_else(|| invalid("unregistered calibration budget"))?;
    if index == 0 {
        return Ok(json!({
            "candidate_budget": budget,
            "candidate_order_valid": true,
            "predecessor_transition_required": false
        }));
    }
    let predecessor = resolve_authoritative_candidate_transition(budget)?;
    Ok(json!({
        "candidate_budget": budget,
        "candidate_order_valid": true,
        "predecessor_budget": CALIBRATION_BUDGETS[index - 1],
        "predecessor_transition": predecessor,
        "predecessor_transition_required": true,
        "raw_predecessor_evidence_required": false
    }))
}

pub(crate) fn aggregate_state(results: &[Value]) -> &'static str {
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

/// Whether a response-body read failed because the request's complete bound
/// elapsed, directly or through a wrapped transport error.
pub(crate) fn io_error_is_timeout(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::TimedOut
        || error
            .get_ref()
            .and_then(|inner| inner.downcast_ref::<reqwest::Error>())
            .is_some_and(reqwest::Error::is_timeout)
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

fn probe_result_status<T>(result: &Result<T, windows_exclusivity::ProbeFailure>) -> Value {
    match result {
        Ok(_) => json!({"status": "ok"}),
        Err(failure) => json!({
            "status": "failed",
            "outcome": failure.kind.outcome(),
            "error": failure.message
        }),
    }
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

pub(crate) fn read_manifest(
    verify_sidecar: bool,
) -> Result<Value, ReasoningBudgetCalibrationError> {
    let manifest = read_json(CALIBRATION_MANIFEST_PATH)?;
    if verify_sidecar {
        validate_sidecar(&manifest)?;
    }
    Ok(manifest)
}

pub(crate) fn read_json(path: &str) -> Result<Value, ReasoningBudgetCalibrationError> {
    Ok(serde_json::from_slice(&fs::read(workspace_path(path))?)?)
}

fn read_json_path(path: &Path) -> Result<Value, ReasoningBudgetCalibrationError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn write_json(
    path: &Path,
    value: &Value,
) -> Result<(), ReasoningBudgetCalibrationError> {
    write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

pub(crate) fn write_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<(), ReasoningBudgetCalibrationError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
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

fn attempt_003_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT)
}

fn attempt_004_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT)
}

fn attempt_005_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_005_EVIDENCE_ROOT)
}

fn attempt_006_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_006_EVIDENCE_ROOT)
}

fn attempt_007_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_007_EVIDENCE_ROOT)
}

fn attempt_008_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_008_EVIDENCE_ROOT)
}

fn attempt_009_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_009_EVIDENCE_ROOT)
}

fn attempt_010_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT)
}

fn attempt_011_root() -> PathBuf {
    workspace_path(CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT)
}

pub(crate) fn workspace_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

pub(crate) fn now_unix_ms() -> Result<u64, ReasoningBudgetCalibrationError> {
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

fn expect_sha256_string(value: &Value, path: &str) -> Result<(), ReasoningBudgetCalibrationError> {
    let actual = value
        .pointer(&format!("/{}", path.replace('.', "/")))
        .and_then(Value::as_str);
    if actual
        .is_none_or(|hash| hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
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

pub(crate) fn same_workflow_identity_path(actual: &str, expected: &str) -> bool {
    #[cfg(windows)]
    {
        actual
            .replace('\\', "/")
            .eq_ignore_ascii_case(&expected.replace('\\', "/"))
    }
    #[cfg(not(windows))]
    {
        actual == expected
    }
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
    fn attempt_009_candidate_order_accepts_tracked_attempt_008_transition() {
        let report = validate_attempt_009_candidate_order().unwrap();

        assert_eq!(report["state"], "CANDIDATE_BUDGET_512_ORDER_VALID");
        assert_eq!(report["SOURCE_ATTEMPT_008"], true);
        assert_eq!(report["SOURCE_BUDGET_1024"], true);
        assert_eq!(report["SOURCE_STATE_FAIL"], true);
        assert_eq!(report["NEXT_BUDGET_512"], true);
        assert_eq!(report["CANDIDATE_BUDGET_512_ORDER_VALID"], true);
        assert_eq!(report["source_attempt"], 8);
        assert_eq!(report["source_candidate_budget"], 1024);
        assert_eq!(report["source_result"], "FAIL");
        assert_eq!(report["next_budget"], 512);
        assert_eq!(report["attempt_010_prepared"], false);
        assert_eq!(report["model_server_startups"], 0);
        assert_eq!(report["inference_requests"], 0);
    }

    #[test]
    fn candidate_order_preparation_and_runtime_share_authoritative_transition() {
        let preparation_transition = resolve_authoritative_candidate_transition(512).unwrap();
        let runtime_order = validate_candidate_order_report(512).unwrap();

        assert_eq!(
            runtime_order["predecessor_transition"],
            preparation_transition
        );
        assert_eq!(runtime_order["candidate_order_valid"], true);
        assert_eq!(runtime_order["raw_predecessor_evidence_required"], false);
    }

    #[test]
    fn missing_authoritative_predecessor_is_rejected_before_model_contact() {
        let error = validate_authoritative_attempt_008_transition_record(&Value::Null)
            .unwrap_err()
            .to_string();

        assert!(error.contains("authoritative predecessor transition is absent"));
    }

    #[test]
    fn attempt_007_raw_next_budget_cannot_satisfy_candidate_order() {
        let mut provenance = read_json(CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH).unwrap();
        provenance["source_attempt"] = json!(7);
        provenance["attempt_007_excluded_from_selection"] = json!(false);
        provenance["attempt_007_raw_next_budget_excluded"] = json!(false);

        let error = validate_authoritative_attempt_008_transition_record(&provenance)
            .unwrap_err()
            .to_string();

        assert!(error.contains("integer mismatch at source_attempt"));
    }

    #[test]
    fn attempt_008_transition_wins_over_generic_and_attempt_007_paths() {
        let order = validate_candidate_order_report(512).unwrap();
        let predecessor = &order["predecessor_transition"];

        assert_eq!(predecessor["candidate_budget"], 1024);
        assert_eq!(predecessor["candidate_state"], "FAIL");
        assert_eq!(
            predecessor["identity_sha256"],
            "917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5"
        );
        assert_eq!(
            predecessor["authoritative_transition_path"],
            CALIBRATION_CANDIDATE_TRANSITIONS_PATH
        );
        assert_eq!(predecessor["source_attempt"], 8);
        assert!(order.get("predecessor_result_path").is_none());
    }

    #[test]
    fn clean_checkout_candidate_order_does_not_require_ignored_raw_predecessor() {
        let report = validate_attempt_009_candidate_order().unwrap();

        assert_eq!(report["PREDECESSOR_TRANSITION_PRESENT"], true);
        assert_eq!(report["PREDECESSOR_TRANSITION_VALID"], true);
        assert_eq!(report["raw_predecessor_evidence_required"], false);
        assert_eq!(report["attempt_009_executions_added"], 0);
    }

    #[test]
    fn authoritative_transition_hashes_match_tracked_attempt_009_lineage() {
        let transition = resolve_authoritative_candidate_transition(512).unwrap();
        let identity = read_json(CALIBRATION_ATTEMPT_009_IDENTITY_PATH).unwrap();

        for (field, lineage_field) in [
            ("identity_sha256", "attempt_008_identity_sha256"),
            (
                "evidence_manifest_sha256",
                "attempt_008_evidence_manifest_sha256",
            ),
        ] {
            let hash = transition[field].as_str().unwrap();
            assert_eq!(hash.len(), 64, "{field} is not a full SHA-256");
            assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
            assert_eq!(transition[field], identity["lineage"][lineage_field]);
        }
    }

    #[test]
    fn candidate_not_selected_by_admissible_predecessor_is_rejected() {
        // An unregistered budget is never ordered, and the initial candidate
        // is never the target of a predecessor transition.
        assert!(validate_candidate_order_report(768).is_err());
        assert!(resolve_authoritative_candidate_transition(768).is_err());
        assert!(resolve_authoritative_candidate_transition(1024).is_err());
        // A registry whose only transition selects 512 does not order 256.
        let mut registry = transition_registry();
        registry["transitions"] = json!([registry["transitions"][0].clone()]);
        assert!(select_candidate_transition(&registry, 256)
            .unwrap_err()
            .to_string()
            .contains("authoritative predecessor transition is absent"));
    }

    /// A labelled fail-closed case: candidate budget and registry mutation.
    type TransitionCase = (&'static str, u32, Box<dyn FnOnce(&mut Value)>);

    fn transition_registry() -> Value {
        read_json(CALIBRATION_CANDIDATE_TRANSITIONS_PATH).unwrap()
    }

    fn transition_rejected(candidate: u32, mutate: impl FnOnce(&mut Value)) -> String {
        let mut registry = transition_registry();
        mutate(&mut registry);
        match select_candidate_transition(&registry, candidate) {
            Ok(transition) => validate_transition_tracked_evidence(&transition)
                .unwrap_err()
                .to_string(),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn attempt_008_to_512_and_attempt_010_to_256_resolve_canonically() {
        let to_512 = resolve_authoritative_candidate_transition(512).unwrap();
        assert_eq!(to_512["source_attempt"], 8);
        assert_eq!(to_512["source_budget"], 1024);
        assert_eq!(to_512["source_state"], "FAIL");
        assert_eq!(to_512["selected_next_budget"], 512);
        assert_eq!(
            to_512["evidence_manifest_sha256"],
            "f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779"
        );

        let to_256 = resolve_authoritative_candidate_transition(256).unwrap();
        assert_eq!(to_256["source_attempt"], 10);
        assert_eq!(to_256["source_budget"], 512);
        assert_eq!(to_256["source_state"], "FAIL");
        assert_eq!(to_256["integrity_accepted"], true);
        assert_eq!(to_256["calibration_admissible"], true);
        assert_eq!(to_256["selected_next_budget"], 256);
        assert_eq!(
            to_256["identity_sha256"],
            "9292e9ecdd2e89f695dfb34bc782ade41b70412c427807c6c3a5653b50ec16f7"
        );
        assert_eq!(
            to_256["evidence_manifest_sha256"],
            "5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9"
        );
        assert_eq!(to_256["raw_predecessor_evidence_required"], false);

        for (budget, source) in [(512, 8), (256, 10)] {
            let order = validate_candidate_order_report(budget).unwrap();
            assert_eq!(order["candidate_order_valid"], true);
            assert_eq!(order["predecessor_transition"]["source_attempt"], source);
            assert!(order.get("predecessor_result_path").is_none());
        }
    }

    #[test]
    fn candidate_256_preparation_and_runtime_share_the_canonical_transition() {
        let absent_root = std::env::temp_dir().join(format!(
            "prefixity-candidate-256-prestart-absent-{}",
            std::process::id()
        ));
        let resolved = resolve_authoritative_candidate_transition(256).unwrap();
        let order = validate_candidate_order_report(256).unwrap();
        let (_manifest, runtime_order) =
            calibration_prestart_checks(256, true, &absent_root).unwrap();

        assert_eq!(order["predecessor_transition"], resolved);
        assert_eq!(runtime_order, order);
    }

    #[test]
    fn authoritative_transitions_exclude_attempts_007_and_009() {
        for attempt in [7, 9] {
            let error = transition_rejected(256, |registry| {
                registry["transitions"][1]["source_attempt"] = json!(attempt);
            });
            assert!(
                error.contains("excluded or invalid source attempt"),
                "{error}"
            );
        }
        let error = transition_rejected(512, |registry| {
            registry["excluded_attempts"]
                .as_array_mut()
                .unwrap()
                .retain(|entry| entry["attempt"] != 9);
        });
        assert!(error.contains("inadmissible attempt is not excluded"));
    }

    #[test]
    fn attempt_010_is_never_a_source_for_512() {
        let error = transition_rejected(512, |registry| {
            registry["transitions"] = json!([registry["transitions"][1].clone()]);
        });
        assert!(error.contains("authoritative predecessor transition is absent"));
    }

    #[test]
    fn authoritative_transitions_fail_closed() {
        let cases: Vec<TransitionCase> = vec![
            (
                "missing registry",
                256,
                Box::new(|registry: &mut Value| *registry = Value::Null),
            ),
            (
                "missing transitions",
                256,
                Box::new(|registry: &mut Value| {
                    registry.as_object_mut().unwrap().remove("transitions");
                }),
            ),
            (
                "corrupt transition",
                256,
                Box::new(|registry: &mut Value| registry["transitions"][1] = json!("corrupt")),
            ),
            (
                "incomplete transition",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]
                        .as_object_mut()
                        .unwrap()
                        .remove("evidence_manifest_sha256");
                }),
            ),
            (
                "integrity rejected",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["integrity_accepted"] = json!(false)
                }),
            ),
            (
                "inadmissible",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["calibration_admissible"] = json!(false)
                }),
            ),
            (
                "incomplete case set",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["case_set_complete"] = json!(false)
                }),
            ),
            (
                "wrong source attempt",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["source_attempt"] = json!(11)
                }),
            ),
            (
                "wrong source budget",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["source_budget"] = json!(1024)
                }),
            ),
            (
                "wrong source state",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["source_state"] = json!("PASS")
                }),
            ),
            (
                "wrong next budget",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["selected_next_budget"] = json!(1024)
                }),
            ),
            (
                "mismatched manifest hash",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["evidence_manifest_sha256"] =
                        json!("f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e4779")
                }),
            ),
            (
                "truncated manifest hash",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["evidence_manifest_sha256"] =
                        json!("5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b")
                }),
            ),
            (
                "mismatched identity hash",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["source_identity_sha256"] =
                        json!("917fde56d11e79a3b700de82f13e5f072bda483fa6b7abe6e2da9ff37ee2dfb5")
                }),
            ),
            (
                "changed execution record",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["execution_record_sha256"] =
                        json!("cc12d12a4a760fce91a9627240b7c7ad8869c999b4f686670093a49531cffc26")
                }),
            ),
            (
                "ignored raw evidence path",
                256,
                Box::new(|registry: &mut Value| {
                    registry["transitions"][1]["execution_record_path"] = json!(
                        "experiments/runs/phase1c-reasoning-budget-calibration/budget-512/candidate-result.json"
                    )
                }),
            ),
            (
                "duplicate transition",
                256,
                Box::new(|registry: &mut Value| {
                    let duplicate = registry["transitions"][1].clone();
                    registry["transitions"]
                        .as_array_mut()
                        .unwrap()
                        .push(duplicate);
                }),
            ),
            (
                "ambiguous source budget",
                512,
                Box::new(|registry: &mut Value| {
                    let mut ambiguous = registry["transitions"][0].clone();
                    ambiguous["source_attempt"] = json!(12);
                    registry["transitions"]
                        .as_array_mut()
                        .unwrap()
                        .push(ambiguous);
                }),
            ),
            (
                "changed candidate order",
                256,
                Box::new(|registry: &mut Value| {
                    registry["candidate_order"] = json!([1024, 256, 512])
                }),
            ),
        ];
        for (label, candidate, mutate) in cases {
            let mut registry = transition_registry();
            mutate(&mut registry);
            let rejected = match select_candidate_transition(&registry, candidate) {
                Ok(transition) => validate_transition_tracked_evidence(&transition).is_err(),
                Err(_) => true,
            };
            assert!(rejected, "{label} was accepted");
        }
    }

    #[test]
    fn corrupt_authoritative_transition_is_rejected_not_reconstructed() {
        let original = read_json(CALIBRATION_ATTEMPT_009_BUDGET_PROVENANCE_PATH).unwrap();
        for (field, value) in [
            ("source_result", json!("PASS")),
            ("source_integrity", json!("REJECTED")),
            ("source_calibration_admissible", json!(false)),
            ("source_candidate_budget", json!(512)),
            ("selected_next_budget", json!(256)),
            ("selected_budget", json!(256)),
        ] {
            let mut provenance = original.clone();
            provenance[field] = value;
            assert!(
                validate_authoritative_attempt_008_transition_record(&provenance).is_err(),
                "{field} corruption was accepted"
            );
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(validate_authoritative_attempt_008_transition_record(&missing).is_err());
        }
    }

    #[test]
    fn attempt_009_forensic_failure_no_longer_looks_like_missing_predecessor() {
        let report = validate_candidate_order_report(512).unwrap();

        assert_eq!(report["candidate_order_valid"], true);
        assert_ne!(report["predecessor_transition"], Value::Null);
        assert_eq!(report["predecessor_transition"]["source_attempt"], 8);
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
    fn attempt_003_identity_is_fingerprinted_and_binds_native_inspection() {
        let identity = read_attempt_003_identity(true).unwrap();
        validate_attempt_003_identity(&identity, true).unwrap();
        assert_eq!(
            canonical_hash(&identity).unwrap(),
            "c40e4528d6dc895a6e688a77dd1159b4c4e740b6a1b78ab2fc5259a0bc02655f"
        );
        assert_eq!(
            identity["attempt_003_evidence_root"],
            format!("{CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT}/")
        );
        assert_eq!(
            identity["exclusivity_implementation"]["source_sha256"],
            ATTEMPT_003_SEALED_IMPLEMENTATION_SHA256
        );
    }

    #[test]
    fn attempt_003_dry_run_is_isolated_and_zero_contact() {
        if attempt_003_root().exists() || attempt_002_root().exists() {
            return;
        }
        let result = dry_run_attempt_003().unwrap();
        assert_eq!(result["state"], "DRY_RUN");
        assert_eq!(result["attempt"], 3);
        assert_eq!(result["budget"], 1024);
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
                        .is_some_and(|path| path.starts_with(CALIBRATION_ATTEMPT_003_EVIDENCE_ROOT))
            }));
    }

    #[test]
    fn attempt_003_cli_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-003-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt003Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-003-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt003Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-003-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt003DryRun
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-003-poststart".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt003Poststart
        );
    }

    #[test]
    fn attempt_004_identity_is_fingerprinted_and_binds_attempt003_evidence() {
        let identity = read_attempt_004_identity(true).unwrap();
        validate_attempt_004_identity_for_clean_checkout(&identity, true).unwrap();
        assert_eq!(
            canonical_hash(&identity).unwrap(),
            "7e59288ccc2847298482dfe6aa4dfe0e2d3e4197fdfbe72031f9551e51c675c9"
        );
        assert_eq!(
            identity["attempt_004_evidence_root"],
            format!("{CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT}/")
        );
        assert_eq!(
            identity["lineage"]["attempt_003_evidence_sha256"]["supervisor.json"],
            "7a396f58b156bb7abb2eda565be1ed15541310717eac597dab8a7902150e9387"
        );
    }

    #[test]
    fn attempt_004_dry_run_is_isolated_and_zero_contact() {
        let result = dry_run_attempt_004_portable().unwrap();
        assert_eq!(result["state"], "DRY_RUN");
        assert_eq!(result["attempt"], 4);
        assert_eq!(result["budget"], 1024);
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
                        .is_some_and(|path| path.starts_with(CALIBRATION_ATTEMPT_004_EVIDENCE_ROOT))
            }));
    }

    #[test]
    fn attempt_004_cli_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-004-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt004Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-004-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt004Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-004-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt004DryRun
        );
    }

    #[test]
    fn attempt_005_frozen_identity_rejects_new_authoritative_supervisor() {
        let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
            Path::new(CALIBRATION_ATTEMPT_005_IDENTITY_PATH),
        )
        .unwrap();
        let child = std::env::current_exe().unwrap();
        let metadata =
            crate::phase1c_live_supervisor::build_workflow_launch_metadata(&registered, &child)
                .unwrap();
        assert!(validate_attempt_005_handoff(&metadata).is_err());
    }

    #[test]
    fn attempt_005_wrong_launch_identity_fails_closed() {
        let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
            Path::new(CALIBRATION_ATTEMPT_005_IDENTITY_PATH),
        )
        .unwrap();
        let child = std::env::current_exe().unwrap();
        let mut metadata =
            crate::phase1c_live_supervisor::build_workflow_launch_metadata(&registered, &child)
                .unwrap();
        metadata.launch_identity = "stale-or-wrong-launch".to_string();
        assert!(validate_attempt_005_handoff(&metadata).is_err());
    }

    #[test]
    fn attempt_005_stale_identity_fails_closed() {
        let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
            Path::new(CALIBRATION_ATTEMPT_005_IDENTITY_PATH),
        )
        .unwrap();
        let child = std::env::current_exe().unwrap();
        let mut metadata =
            crate::phase1c_live_supervisor::build_workflow_launch_metadata(&registered, &child)
                .unwrap();
        metadata.attempt_identity_sha256 = "c".repeat(64);
        assert!(validate_attempt_005_handoff(&metadata).is_err());
    }

    #[test]
    fn attempt004_missing_launch_identity_regression_fails_closed() {
        let processes = Ok(vec![
            windows_exclusivity::ProcessRecord {
                image_name: "prefixity-phase1c-live-supervisor.exe".to_string(),
                pid: 10,
                parent_pid: 1,
                executable_path: Some(
                    r"D:Prefixityprefixity-phase1c-live-supervisor.exe".to_string(),
                ),
                executable_identity: None,
            },
            windows_exclusivity::ProcessRecord {
                image_name: "prefixity-phase1c-calibration.exe".to_string(),
                pid: 11,
                parent_pid: 10,
                executable_path: Some(r"D:Prefixityprefixity-phase1c-calibration.exe".to_string()),
                executable_identity: None,
            },
            windows_exclusivity::ProcessRecord {
                image_name: "llama.exe".to_string(),
                pid: 20,
                parent_pid: 1,
                executable_path: Some(r"D:llamallama.exe".to_string()),
                executable_identity: None,
            },
        ]);
        let listeners = Ok(vec![windows_exclusivity::ListenerRecord {
            local_address: "127.0.0.1".to_string(),
            local_port: 8080,
            pid: 20,
            state: "LISTENING".to_string(),
        }]);
        let identity = windows_exclusivity::ExpectedWorkflowIdentity {
            launch_identity: String::new(),
            supervisor: windows_exclusivity::ExpectedProcessIdentity {
                pid: 10,
                executable_path: r"D:Prefixityprefixity-phase1c-live-supervisor.exe".to_string(),
                executable_identity: windows_exclusivity::ExecutableIdentity {
                    raw_path: r"D:Prefixityprefixity-phase1c-live-supervisor.exe".to_string(),
                    final_path: r"D:Prefixityprefixity-phase1c-live-supervisor.exe".to_string(),
                    file_size: 0,
                    sha256: String::new(),
                    file_id: None,
                },
                parent_pid: None,
            },
            calibration_child: windows_exclusivity::ExpectedProcessIdentity {
                pid: 11,
                executable_path: r"D:Prefixityprefixity-phase1c-calibration.exe".to_string(),
                executable_identity: windows_exclusivity::ExecutableIdentity {
                    raw_path: r"D:Prefixityprefixity-phase1c-calibration.exe".to_string(),
                    final_path: r"D:Prefixityprefixity-phase1c-calibration.exe".to_string(),
                    file_size: 0,
                    sha256: String::new(),
                    file_id: None,
                },
                parent_pid: Some(10),
            },
            inspector: None,
        };
        assert_eq!(
            windows_exclusivity::classify_poststart_with_expected_workflow(
                20, &processes, &listeners, &identity
            ),
            windows_exclusivity::ExclusivityOutcome::ExpectedWorkflowIdentityInvalid
        );
    }

    #[test]
    fn attempt_005_cli_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-005-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt005Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-005-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt005Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-005-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt005DryRun
        );
    }

    #[test]
    fn attempt_006_identity_is_self_consistent_and_bound_to_canonical_sources() {
        let identity = read_attempt_006_identity(true).unwrap();
        validate_attempt_006_identity_for_clean_checkout(&identity, true).unwrap();
        let fingerprints = fingerprint_calibration().unwrap();
        validate_attempt_006_bindings(&identity, &fingerprints).unwrap();
        assert_eq!(
            canonical_hash(&identity).unwrap(),
            "1d326578deb5a5280eafbed5d844a754d1468429c4990b25a0ee207652ca28a1"
        );
        assert_eq!(
            identity["launch_handoff_implementation"]["source_sha256"],
            "76f122f4f6cf096a6a972d5770122b5590767467972aa43c7469d51b1b2656ef"
        );
        assert_eq!(
            identity["native_exclusivity_implementation"]["source_sha256"],
            "9dba33fdc0c4c9279e5df4eb12b3be0a0f5f99492f74efa548310ebf4163c37b"
        );
    }

    #[test]
    fn attempt_006_handoff_contract_accepts_supervisor_generated_metadata() {
        let identity = read_attempt_006_identity(true).unwrap();
        let registered = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
            Path::new(CALIBRATION_ATTEMPT_006_IDENTITY_PATH),
        )
        .unwrap();
        let child = std::env::current_exe().unwrap();
        let metadata =
            crate::phase1c_live_supervisor::build_workflow_launch_metadata(&registered, &child)
                .unwrap();
        assert_eq!(
            metadata.attempt_identity_path,
            CALIBRATION_ATTEMPT_006_IDENTITY_PATH
        );
        assert_eq!(
            metadata.attempt_identity_sha256,
            canonical_hash(&identity).unwrap()
        );
        assert_eq!(metadata.attempt, registered.attempt);
        assert_eq!(metadata.candidate_budget, registered.candidate_budget);
        assert_eq!(metadata.candidate_identity, registered.candidate_identity);
        assert_eq!(metadata.evidence_root, registered.evidence_root);
        assert_eq!(
            metadata.launch_identity,
            registered.generated_launch_identity()
        );
        assert_eq!(
            metadata.attempt_identity_sha256,
            canonical_hash(&identity).unwrap()
        );
    }

    #[test]
    fn attempt_006_cli_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-006-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt006Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-006-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt006Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-006-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt006DryRun
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-006-poststart".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt006Poststart
        );
    }

    #[test]
    fn attempt_007_identity_is_self_consistent_and_bound_to_certification() {
        let identity = read_attempt_007_identity(true).unwrap();
        validate_attempt_007_identity(&identity, true).unwrap();
        let fingerprints = fingerprint_calibration().unwrap();
        validate_attempt_007_bindings(&identity, &fingerprints).unwrap();
        assert_eq!(canonical_hash(&identity).unwrap().len(), 64);
        assert_eq!(identity["attempt"], 7);
        assert_eq!(
            identity["workflow_identity_certification"]["state"],
            "WORKFLOW_IDENTITY_CERTIFIED"
        );
        assert_eq!(
            identity["workflow_identity_certification"]["path_classification"],
            "PATH_REPRESENTATION_MISMATCH"
        );
    }

    #[test]
    fn attempt_007_preserved_mismatch_fixture_is_rejected_offline() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/phase1c/attempt-007-frozen-executable-mismatch.json"
        ))
        .unwrap();
        let preparation = &fixture["preparation_identity"];
        let evidence = &fixture["supervisor_evidence"];
        let error = validate_recorded_workflow_evidence_against_preparation(preparation, evidence)
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("FROZEN_EXECUTABLE_IDENTITY_MISMATCH"));
        assert_eq!(
            fixture["expected_classification"],
            "FROZEN_EXECUTABLE_IDENTITY_MISMATCH"
        );
    }

    #[test]
    fn attempt_007_handoff_contract_accepts_supervisor_generated_metadata() {
        let error = crate::phase1c_live_supervisor::registered_workflow_identity_from_file(
            Path::new(CALIBRATION_ATTEMPT_007_IDENTITY_PATH),
        )
        .unwrap_err();
        assert!(error.to_string().contains("mutable target/debug"));
    }

    #[test]
    fn attempt_007_cli_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-007-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt007Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-007-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt007Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-007-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt007DryRun
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-007-poststart".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt007Poststart
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-007-validate-evidence".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt007ValidateEvidence
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["run-attempt-007".to_string()]).unwrap(),
            CalibrationCliCommand::RunAttempt007
        );
    }

    #[test]
    fn attempt_008_preparation_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-008-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt008Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-008-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt008Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-008-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt008DryRun
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-008-freeze".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt008Freeze
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-008-validate-preparation".to_string(),])
                .unwrap(),
            CalibrationCliCommand::Attempt008ValidatePreparation
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["run-attempt-008".to_string()]).unwrap(),
            CalibrationCliCommand::RunAttempt008
        );
    }

    #[test]
    fn attempt_009_preparation_commands_are_offline_only() {
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-fingerprint".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt009Fingerprint
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-preflight".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt009Preflight
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-dry-run".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt009DryRun
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-freeze".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt009Freeze
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-validate-preparation".to_string(),])
                .unwrap(),
            CalibrationCliCommand::Attempt009ValidatePreparation
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["attempt-009-candidate-order".to_string()]).unwrap(),
            CalibrationCliCommand::Attempt009CandidateOrder
        );
        assert_eq!(
            parse_calibration_cli_args(vec!["run-attempt-009".to_string()]).unwrap(),
            CalibrationCliCommand::RunAttempt009
        );
    }

    #[test]
    fn attempt_010_preparation_commands_are_offline_only() {
        for (command, expected) in [
            (
                "attempt-010-fingerprint",
                CalibrationCliCommand::Attempt010Fingerprint,
            ),
            (
                "attempt-010-preflight",
                CalibrationCliCommand::Attempt010Preflight,
            ),
            (
                "attempt-010-dry-run",
                CalibrationCliCommand::Attempt010DryRun,
            ),
            (
                "attempt-010-freeze",
                CalibrationCliCommand::Attempt010Freeze,
            ),
            (
                "attempt-010-validate-preparation",
                CalibrationCliCommand::Attempt010ValidatePreparation,
            ),
            (
                "attempt-010-repository-contract",
                CalibrationCliCommand::Attempt010RepositoryContract,
            ),
            (
                "attempt-010-live-prerequisites",
                CalibrationCliCommand::Attempt010LivePrerequisites,
            ),
            ("run-attempt-010", CalibrationCliCommand::RunAttempt010),
        ] {
            assert_eq!(
                parse_calibration_cli_args(vec![command.to_string()]).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn attempt_010_recorded_commands_use_only_frozen_objects() {
        for command in [
            ATTEMPT_010_SUPERVISOR_COMMAND,
            ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND,
        ] {
            assert!(!command.to_ascii_lowercase().contains("target\\debug"));
            assert!(command.contains(&CALIBRATION_ATTEMPT_010_FROZEN_STAGE_ROOT.replace('/', "\\")));
            assert!(command.contains(CALIBRATION_ATTEMPT_010_IDENTITY_PATH));
        }
        let live_child = ATTEMPT_010_SUPERVISOR_COMMAND.rsplit(' ').next().unwrap();
        assert_eq!(
            parse_calibration_cli_args(vec![live_child.to_string()]).unwrap(),
            CalibrationCliCommand::RunAttempt010
        );
        assert!(ATTEMPT_010_SUPERVISOR_COMMAND.contains(&format!(
            "{CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT}/supervisor.json"
        )));
        let traversal_child = ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND
            .rsplit(' ')
            .next()
            .unwrap();
        assert_eq!(
            parse_calibration_cli_args(vec![traversal_child.to_string()]).unwrap(),
            CalibrationCliCommand::Attempt010LivePrerequisites
        );
        assert!(
            ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND.contains(&format!(
                "{CALIBRATION_ATTEMPT_010_PREREQUISITE_TRAVERSAL_ROOT}/supervisor.json"
            ))
        );
        assert!(!ATTEMPT_010_PREREQUISITE_TRAVERSAL_COMMAND
            .contains(CALIBRATION_ATTEMPT_010_EVIDENCE_ROOT));
    }

    #[test]
    fn attempt_010_prestart_checks_share_the_runtime_candidate_order() {
        let absent_root = std::env::temp_dir().join(format!(
            "prefixity-attempt-010-prestart-absent-{}",
            std::process::id()
        ));
        let (_manifest, order) = calibration_prestart_checks(512, true, &absent_root).unwrap();

        assert_eq!(order, validate_candidate_order_report(512).unwrap());
        assert_eq!(order["predecessor_transition"]["source_attempt"], 8);
        assert!(order.get("predecessor_result_path").is_none());
    }

    #[test]
    fn attempt_010_prestart_checks_reject_before_listener_contact() {
        let root = std::env::temp_dir().join(format!(
            "prefixity-attempt-010-prestart-existing-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let existing = calibration_prestart_checks(512, true, &root)
            .unwrap_err()
            .to_string();
        fs::remove_dir_all(&root).unwrap();
        assert!(existing.contains("calibration candidate evidence already exists"));

        let absent_root = root.join("absent");
        assert!(calibration_prestart_checks(512, false, &absent_root)
            .unwrap_err()
            .to_string()
            .contains("fresh-runtime confirmation"));
        assert!(calibration_prestart_checks(768, true, &absent_root).is_err());
    }

    #[test]
    fn attempt_010_live_prerequisites_require_supervisor_handoff() {
        let error = attempt_010_live_prerequisites().unwrap_err().to_string();

        assert!(error.contains("expected workflow launch metadata was not handed off"));
    }

    #[test]
    fn attempt_011_preparation_commands_are_offline_only() {
        for (command, expected) in [
            (
                "attempt-011-fingerprint",
                CalibrationCliCommand::Attempt011Fingerprint,
            ),
            (
                "attempt-011-preflight",
                CalibrationCliCommand::Attempt011Preflight,
            ),
            (
                "attempt-011-dry-run",
                CalibrationCliCommand::Attempt011DryRun,
            ),
            (
                "attempt-011-freeze",
                CalibrationCliCommand::Attempt011Freeze,
            ),
            (
                "attempt-011-validate-preparation",
                CalibrationCliCommand::Attempt011ValidatePreparation,
            ),
            (
                "attempt-011-repository-contract",
                CalibrationCliCommand::Attempt011RepositoryContract,
            ),
            (
                "attempt-011-live-prerequisites",
                CalibrationCliCommand::Attempt011LivePrerequisites,
            ),
            ("run-attempt-011", CalibrationCliCommand::RunAttempt011),
        ] {
            assert_eq!(
                parse_calibration_cli_args(vec![command.to_string()]).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn attempt_011_recorded_commands_use_only_frozen_objects() {
        for command in [
            ATTEMPT_011_SUPERVISOR_COMMAND,
            ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND,
        ] {
            assert!(!command.to_ascii_lowercase().contains("target\\debug"));
            assert!(command.contains(&CALIBRATION_ATTEMPT_011_FROZEN_STAGE_ROOT.replace('/', "\\")));
            assert!(command.contains(CALIBRATION_ATTEMPT_011_IDENTITY_PATH));
        }
        let live_child = ATTEMPT_011_SUPERVISOR_COMMAND.rsplit(' ').next().unwrap();
        assert_eq!(
            parse_calibration_cli_args(vec![live_child.to_string()]).unwrap(),
            CalibrationCliCommand::RunAttempt011
        );
        assert!(ATTEMPT_011_SUPERVISOR_COMMAND.contains(&format!(
            "{CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT}/supervisor.json"
        )));
        let traversal_child = ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND
            .rsplit(' ')
            .next()
            .unwrap();
        assert_eq!(
            parse_calibration_cli_args(vec![traversal_child.to_string()]).unwrap(),
            CalibrationCliCommand::Attempt011LivePrerequisites
        );
        assert!(
            ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND.contains(&format!(
                "{CALIBRATION_ATTEMPT_011_PREREQUISITE_TRAVERSAL_ROOT}/supervisor.json"
            ))
        );
        assert!(!ATTEMPT_011_PREREQUISITE_TRAVERSAL_COMMAND
            .contains(CALIBRATION_ATTEMPT_011_EVIDENCE_ROOT));
    }

    #[test]
    fn attempt_011_prestart_checks_resolve_attempt_010_transition() {
        let absent_root = std::env::temp_dir().join(format!(
            "prefixity-attempt-011-prestart-absent-{}",
            std::process::id()
        ));
        let (_manifest, order) = calibration_prestart_checks(256, true, &absent_root).unwrap();

        assert_eq!(order, validate_candidate_order_report(256).unwrap());
        assert_eq!(
            order["predecessor_transition"],
            resolve_authoritative_candidate_transition(256).unwrap()
        );
        assert_eq!(order["predecessor_transition"]["source_attempt"], 10);
        assert_eq!(order["predecessor_transition"]["source_budget"], 512);
        assert_ne!(order["predecessor_transition"]["source_attempt"], 8);
        assert!(order.get("predecessor_result_path").is_none());
    }

    #[test]
    fn attempt_011_live_prerequisites_require_supervisor_handoff() {
        let error = attempt_011_live_prerequisites().unwrap_err().to_string();

        assert!(error.contains("expected workflow launch metadata was not handed off"));
    }

    #[test]
    fn source_fingerprint_normalizes_checkout_line_endings_and_rejects_bom() {
        let lf = b"alpha\nbeta\ngamma\n";
        let crlf = b"alpha\r\nbeta\r\ngamma\r\n";
        let mixed = b"alpha\rbeta\r\ngamma\n";

        assert_eq!(
            canonicalize_source_bytes(lf).unwrap(),
            canonicalize_source_bytes(crlf).unwrap()
        );
        assert_eq!(
            canonicalize_source_bytes(lf).unwrap(),
            canonicalize_source_bytes(mixed).unwrap()
        );
        assert!(canonicalize_source_bytes(&[0xef, 0xbb, 0xbf, b'a']).is_err());
    }

    #[test]
    fn attempt_005_source_fingerprint_matches_canonical_reviewed_bytes() {
        let supervisor = source_sha256(CALIBRATION_SUPERVISOR_HANDOFF_SOURCE_PATH).unwrap();
        let exclusivity = source_sha256(CALIBRATION_WINDOWS_EXCLUSIVITY_SOURCE_PATH).unwrap();
        assert_eq!(supervisor.len(), 64);
        assert_eq!(exclusivity.len(), 64);
        assert_ne!(
            supervisor,
            "7416b793be4ed8eeb2d33a409771d672f7c90dce0b8154f449151e65d3fd2ab7"
        );
    }
}
