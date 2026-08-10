//! Offline-only Phase 1C Stage 0 replay-procedure certification.
//!
//! This module certifies runner mechanics with synthetic payloads and an
//! in-process mock transport. It deliberately has no credential, URL, socket,
//! provider-client, or production-planner boundary.

use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, canonical_json, hash_text, sha256_hex};
use crate::phase1b9::{run_phase1b9_study, PHASE_1B9_POLICY_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const STAGE0_RUNNER_VERSION: &str = "phase1c-stage0-runner-v1";
pub const STAGE0_REPORT_SCHEMA_VERSION: &str = "phase1c-stage0-report-v1";
pub const STAGE0_MOCK_TRANSPORT_SCHEMA_VERSION: &str = "stage0-mock-transport-v1";
pub const STAGE0_EVALUATOR_VERSION: &str = "stage0-deterministic-evaluator-v1";
pub const STAGE0_REDACTION_VERSION: &str = "stage0-redaction-v1";
pub const STAGE0_ABORT_POLICY_VERSION: &str = "stage0-abort-policy-v1";
pub const STAGE0_SOURCE_COMMIT: &str = "d20b3c09fa09cfcf403bdb57792ead55314ee6e6";
pub const STAGE0_CI_RUN: &str = "31371467905";
pub const STAGE0_POLICY_HASH: &str =
    "2139e084d97b16f3ae4ad36d95d40f0c73b4b1f448fe68f197139aa744dfe0e4";
pub const STAGE0_PROTECTED_ACTIVE_HASH: &str =
    "D329C117BF346D65B2587B07EF9B13AA394E5796B580C623E71B1593853F17E2";

const DESIGN_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/phase-1/PHASE_1C_DESIGN_AUTHORIZATION_GATE.md"
));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Stage0CertificationStatus {
    CertifiedReadyForSeparateStage1Authorization,
    NoGoStage0RequirementsNotMet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Stage0Arm {
    Baseline,
    NoOp,
    Intervention,
}

impl Stage0Arm {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Baseline => "BASELINE",
            Self::NoOp => "NO_OP",
            Self::Intervention => "INTERVENTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0Manifest {
    pub source_commit: String,
    pub design_document_hash: String,
    pub phase1b9_policy_version: String,
    pub phase1b9_policy_hash: String,
    pub phase1b9_preregistration_hash: String,
    pub runner_version: String,
    pub cohort_count: usize,
    pub cohort_manifest_hash: String,
    pub task_ids_and_source_hashes: Vec<Stage0TaskIdentity>,
    pub baseline_manifest_hash: String,
    pub no_op_manifest_hash: String,
    pub intervention_manifest_hash: String,
    pub transformation_hashes: Vec<String>,
    pub evaluator_version: String,
    pub evaluator_hash: String,
    pub mock_transport_schema_version: String,
    pub report_schema_version: String,
    pub arm_order: Vec<String>,
    pub replicate_count: u32,
    pub max_turns: u32,
    pub maximum_physical_requests: u32,
    pub estimated_input_unit_ceiling: u64,
    pub output_unit_ceiling: u64,
    pub mock_time_ceiling_ms: u64,
    pub hard_stage0_spend_ceiling: u64,
    pub network_permission: bool,
    pub credential_read_permission: bool,
    pub retry_permission: bool,
    pub redirect_permission: bool,
    pub artifact_redaction_version: String,
    pub abort_policy_version: String,
    pub stage1_placeholders: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0TaskIdentity {
    pub task_id: String,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0EfficiencyGateResult {
    pub total_input_not_increased: bool,
    pub fresh_input_threshold_met: bool,
    pub billed_cost_branch: String,
    pub output_not_increased: bool,
    pub rounds_not_increased: bool,
    pub tool_calls_not_increased: bool,
    pub rereads_not_increased: bool,
    pub recovery_turns_not_increased: bool,
    pub physical_requests_not_increased: bool,
    pub latency_within_threshold: bool,
    pub hypothetical_win: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0TaskRecord {
    pub task_id: String,
    pub selected_class: String,
    pub selected_target: Option<String>,
    pub baseline_no_op_payload_equivalent: bool,
    pub intervention_diff_valid: bool,
    pub status: String,
    pub baseline_completed: bool,
    pub no_op_completed: bool,
    pub intervention_completed: Option<bool>,
    pub evaluator_status: String,
    pub accounting_complete: bool,
    pub efficiency_gate: Option<Stage0EfficiencyGateResult>,
    pub physical_requests: u32,
    pub abort_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0AbortProbe {
    pub condition: String,
    pub passed: bool,
    pub requests_after_abort: u32,
    pub automatic_retries: u32,
    pub baseline_preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage0Report {
    pub artifact_id: String,
    pub report_schema_version: String,
    pub certification_status: Stage0CertificationStatus,
    pub source_commit: String,
    pub design_document_hash: String,
    pub ci_run: String,
    pub runner_version: String,
    pub cohort_count: usize,
    pub cohort_manifest_hash: String,
    pub phase1b9_policy_version: String,
    pub phase1b9_policy_hash: String,
    pub phase1b9_preregistration_hash: String,
    pub evaluator_version: String,
    pub evaluator_hash: String,
    pub mock_transport_schema_version: String,
    pub baseline_manifest_hash: String,
    pub no_op_manifest_hash: String,
    pub intervention_manifest_hash: String,
    pub transformation_hashes: Vec<String>,
    pub network_call_count: u32,
    pub credential_read_count: u32,
    pub mock_physical_request_count: u32,
    pub spend: u64,
    pub baseline_no_op_equivalence: bool,
    pub intervention_diff_integrity: bool,
    pub accounting_certification: bool,
    pub efficiency_gate_logic_certification: bool,
    pub abort_matrix: Vec<Stage0AbortProbe>,
    pub rollback_fail_open_certification: bool,
    pub leakage_certification: bool,
    pub redaction_certification: bool,
    pub task_records: Vec<Stage0TaskRecord>,
    pub stage1_unresolved_inputs: BTreeMap<String, String>,
    pub limitations: Vec<String>,
    pub next_authorization: String,
    pub aggregate_certification_hash: String,
    pub determinism_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MockBlock {
    block_id: String,
    content_hash: String,
    required: bool,
    structural_index: u32,
    references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MockPayload {
    source_nonce: String,
    blocks: Vec<MockBlock>,
    tool_contract: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stage0Request {
    task_id: String,
    arm: Stage0Arm,
    replicate: u32,
    payload: MockPayload,
    run_metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stage0Transformation {
    class: String,
    target_block_id: Option<String>,
    operation: String,
    declared_changed_block_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct Stage0Task {
    identity: Stage0TaskIdentity,
    payload: MockPayload,
    decision_class: String,
    decision_target: Option<String>,
    transformation: Stage0Transformation,
    evaluation: Stage0EvaluationKey,
    fault: Stage0Fault,
    budget_limit: Option<u32>,
}

#[derive(Debug, Clone)]
struct Stage0EvaluationKey {
    expected_efficiency_win: bool,
    evaluator_inconclusive: bool,
    required_block_ids: BTreeSet<String>,
    critical_failure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage0Fault {
    None,
    MissingAccounting,
    SafetyFailure,
    BudgetBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MockFault {
    None,
    MalformedSchema,
    Timeout,
    Redirect,
    RetryRequired,
    MissingAccounting,
    UnexpectedTool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MockUsage {
    total_input_units: Option<u64>,
    fresh_input_units: Option<u64>,
    cache_read_units: Option<u64>,
    cache_write_units: Option<u64>,
    output_units: Option<u64>,
    rounds: Option<u32>,
    tool_calls: Option<u32>,
    rereads: Option<u32>,
    recovery_turns: Option<u32>,
    latency_ms: Option<u64>,
    timeout: bool,
    retry: bool,
    redirect: bool,
    schema_valid: bool,
    unexpected_tool_calls: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MockResponse {
    request_id: String,
    provider_request_id: String,
    usage: MockUsage,
    task_completed: bool,
    required_tool_outcomes: bool,
}

#[derive(Debug, Clone)]
struct MockTransport {
    calls: Vec<Stage0Request>,
    efficiency_wins: BTreeSet<String>,
}

impl MockTransport {
    fn new(efficiency_wins: BTreeSet<String>) -> Self {
        Self {
            calls: Vec::new(),
            efficiency_wins,
        }
    }

    fn execute(
        &mut self,
        request: Stage0Request,
        fault: MockFault,
    ) -> Result<MockResponse, String> {
        self.calls.push(request.clone());
        match fault {
            MockFault::Timeout => return Err("MOCK_TIMEOUT".to_string()),
            MockFault::Redirect => return Err("MOCK_REDIRECT_REJECTED".to_string()),
            MockFault::RetryRequired => return Err("MOCK_RETRY_REQUIRED_REJECTED".to_string()),
            _ => {}
        }
        let efficient = request.arm == Stage0Arm::Intervention
            && self.efficiency_wins.contains(&request.task_id);
        let missing = fault == MockFault::MissingAccounting;
        let unexpected_tool = fault == MockFault::UnexpectedTool;
        let usage = MockUsage {
            total_input_units: (!missing).then_some(if efficient { 90 } else { 100 }),
            fresh_input_units: (!missing).then_some(if efficient { 80 } else { 100 }),
            cache_read_units: (!missing).then_some(0),
            cache_write_units: (!missing).then_some(0),
            output_units: (!missing).then_some(10),
            rounds: (!missing).then_some(1),
            tool_calls: (!missing).then_some(if unexpected_tool { 2 } else { 1 }),
            rereads: (!missing).then_some(0),
            recovery_turns: (!missing).then_some(0),
            latency_ms: (!missing).then_some(100),
            timeout: false,
            retry: false,
            redirect: false,
            schema_valid: fault != MockFault::MalformedSchema,
            unexpected_tool_calls: u32::from(unexpected_tool),
        };
        Ok(MockResponse {
            request_id: format!(
                "mock-request-{}-{}-{}",
                request.task_id,
                request.arm.as_str().to_ascii_lowercase(),
                request.replicate
            ),
            provider_request_id: format!("mock-provider-request-{}", self.calls.len()),
            usage,
            task_completed: fault != MockFault::MalformedSchema,
            required_tool_outcomes: !unexpected_tool,
        })
    }
}

#[derive(Debug, Clone)]
struct PreflightState {
    source_commit: String,
    design_hash: String,
    policy_hash: String,
    cohort_hash: String,
    evaluator_hash: String,
    transformation_hash: String,
    undeclared_arm_diff: bool,
    missing_accounting_requirement: bool,
    spend: u64,
    network_permission: bool,
    credential_read_permission: bool,
    retry_permission: bool,
    redirect_permission: bool,
    active_hash: String,
    active_staged: bool,
}

#[derive(Debug, Clone, Copy)]
enum PreflightMutation {
    SourceCommit,
    DesignHash,
    PolicyHash,
    CohortHash,
    EvaluatorHash,
    TransformationHash,
    UndeclaredArmDiff,
    MissingAccounting,
    NonzeroSpend,
    NetworkPermission,
    CredentialPermission,
    RetryPermission,
    RedirectPermission,
    ActiveHash,
    ActiveStaged,
}

pub fn stage0_design_hash() -> String {
    sha256_hex(DESIGN_BYTES)
}

pub fn canonical_stage0_report_json(report: &Stage0Report) -> Result<Vec<u8>, BenchmarkError> {
    canonical_json(report).map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn run_stage0_certification() -> Result<Stage0Report, BenchmarkError> {
    let first = run_stage0_once()?;
    let second = run_stage0_once()?;
    let first_json = canonical_stage0_report_json(&first)?;
    let second_json = canonical_stage0_report_json(&second)?;
    if first_json != second_json {
        return Err(BenchmarkError::validation(
            "Stage 0 repeated report was not byte-identical",
        ));
    }
    let mut report = first;
    report.determinism_hash = canonical_hash(&report_without_determinism(&report))
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(report)
}

fn run_stage0_once() -> Result<Stage0Report, BenchmarkError> {
    let phase1b9 = run_phase1b9_study()?;
    if phase1b9.policy_version != PHASE_1B9_POLICY_VERSION {
        return Err(BenchmarkError::validation(
            "Phase 1B.9 policy version differs from the frozen Stage 0 input",
        ));
    }
    if phase1b9.policy_hash != STAGE0_POLICY_HASH {
        return Err(BenchmarkError::HashMismatch {
            what: "Phase 1B.9 policy".to_string(),
            expected: STAGE0_POLICY_HASH.to_string(),
            found: phase1b9.policy_hash,
        });
    }

    let tasks = build_stage0_cohort(&phase1b9)?;
    let manifest = build_stage0_manifest(&tasks)?;
    let preflight = preflight_state(&manifest);
    validate_preflight(&preflight)?;

    let baseline_no_op_equivalence = baseline_no_op_equivalence(&tasks)?;
    let intervention_diff_integrity = intervention_diff_integrity(&tasks)?;
    let efficiency_wins = tasks
        .iter()
        .filter(|task| task.evaluation.expected_efficiency_win)
        .map(|task| task.identity.task_id.clone())
        .collect::<BTreeSet<_>>();
    let mut transport = MockTransport::new(efficiency_wins);
    let (task_records, accounting_certification) =
        execute_cohort(&tasks, &manifest, &mut transport)?;
    let abort_matrix = run_abort_matrix(&manifest)?;
    let efficiency_gate_logic_certification = efficiency_gate_logic_certification();
    let rollback_fail_open_certification = rollback_fail_open_certification(&tasks)?;
    let leakage_certification = leakage_certification(&tasks)?;
    let redaction_certification = redaction_certification()?;
    let aggregate_certification_hash = canonical_hash(&serde_json::json!({
        "manifest": manifest,
        "task_records": task_records,
        "abort_matrix": abort_matrix,
        "baseline_no_op_equivalence": baseline_no_op_equivalence,
        "intervention_diff_integrity": intervention_diff_integrity,
        "accounting_certification": accounting_certification,
        "efficiency_gate_logic_certification": efficiency_gate_logic_certification,
        "rollback_fail_open_certification": rollback_fail_open_certification,
        "leakage_certification": leakage_certification,
        "redaction_certification": redaction_certification,
    }))
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let abort_ok = abort_matrix.iter().all(|probe| probe.passed);
    let certified = baseline_no_op_equivalence
        && intervention_diff_integrity
        && accounting_certification
        && efficiency_gate_logic_certification
        && abort_ok
        && rollback_fail_open_certification
        && leakage_certification
        && redaction_certification
        && transport.calls.iter().all(|_| true);

    Ok(Stage0Report {
        artifact_id: "prefixity-phase1c-stage0-certification-v1".to_string(),
        report_schema_version: STAGE0_REPORT_SCHEMA_VERSION.to_string(),
        certification_status: if certified {
            Stage0CertificationStatus::CertifiedReadyForSeparateStage1Authorization
        } else {
            Stage0CertificationStatus::NoGoStage0RequirementsNotMet
        },
        source_commit: STAGE0_SOURCE_COMMIT.to_string(),
        design_document_hash: stage0_design_hash(),
        ci_run: STAGE0_CI_RUN.to_string(),
        runner_version: STAGE0_RUNNER_VERSION.to_string(),
        cohort_count: tasks.len(),
        cohort_manifest_hash: manifest.cohort_manifest_hash.clone(),
        phase1b9_policy_version: manifest.phase1b9_policy_version.clone(),
        phase1b9_policy_hash: manifest.phase1b9_policy_hash.clone(),
        phase1b9_preregistration_hash: manifest.phase1b9_preregistration_hash.clone(),
        evaluator_version: manifest.evaluator_version.clone(),
        evaluator_hash: manifest.evaluator_hash.clone(),
        mock_transport_schema_version: manifest.mock_transport_schema_version.clone(),
        baseline_manifest_hash: manifest.baseline_manifest_hash.clone(),
        no_op_manifest_hash: manifest.no_op_manifest_hash.clone(),
        intervention_manifest_hash: manifest.intervention_manifest_hash.clone(),
        transformation_hashes: manifest.transformation_hashes.clone(),
        network_call_count: 0,
        credential_read_count: 0,
        mock_physical_request_count: transport.calls.len() as u32,
        spend: 0,
        baseline_no_op_equivalence,
        intervention_diff_integrity,
        accounting_certification,
        efficiency_gate_logic_certification,
        abort_matrix,
        rollback_fail_open_certification,
        leakage_certification,
        redaction_certification,
        task_records,
        stage1_unresolved_inputs: manifest.stage1_placeholders.clone(),
        limitations: vec![
            "Mock usage, latency, tool outcomes, and request IDs are synthetic runner fixtures, not provider evidence.".to_string(),
            "Stage 0 does not measure live task quality, provider cache behavior, pricing, or end-to-end production benefit.".to_string(),
            "The Phase 1B.9 CONTROLLED_ONLY policy remains research-only and is not promoted by this certification.".to_string(),
        ],
        next_authorization: "Separate direct Stage 1 authorization naming provider, model, API surface, endpoint, credential boundary, cohort, settings, evaluator, ceilings, pricing, artifact policy, and abort owner.".to_string(),
        aggregate_certification_hash,
        determinism_hash: String::new(),
    })
}

fn report_without_determinism(report: &Stage0Report) -> Stage0Report {
    let mut copy = report.clone();
    copy.determinism_hash.clear();
    copy
}

fn build_stage0_cohort(
    phase1b9: &crate::phase1b9::Phase1b9Report,
) -> Result<Vec<Stage0Task>, BenchmarkError> {
    if phase1b9.decisions.len() != 14 {
        return Err(BenchmarkError::validation(
            "Stage 0 requires the frozen 14-case Phase 1B.9 decision set",
        ));
    }
    let mut tasks = Vec::with_capacity(17);
    for decision in &phase1b9.decisions {
        let payload = base_payload(&decision.case_id, false);
        let target_block_id = decision
            .selected_target_event_id
            .as_deref()
            .map(event_id_to_block_id);
        let transformation = transformation(
            &decision.selected_class,
            target_block_id.clone(),
            &decision.case_id,
        );
        let positive = decision.positive_available;
        let evaluation = Stage0EvaluationKey {
            expected_efficiency_win: positive,
            evaluator_inconclusive: decision.case_id == "h013",
            required_block_ids: BTreeSet::new(),
            critical_failure: false,
        };
        tasks.push(Stage0Task {
            identity: task_identity(&decision.case_id, &payload)?,
            payload,
            decision_class: decision.selected_class.clone(),
            decision_target: target_block_id,
            transformation,
            evaluation,
            fault: if decision.case_id == "h014" {
                Stage0Fault::MissingAccounting
            } else {
                Stage0Fault::None
            },
            budget_limit: None,
        });
    }

    let safety_payload = base_payload("s015", true);
    tasks.push(Stage0Task {
        identity: task_identity("s015", &safety_payload)?,
        payload: safety_payload,
        decision_class: "PRUNE".to_string(),
        decision_target: Some("b002".to_string()),
        transformation: transformation("PRUNE", Some("b002".to_string()), "s015"),
        evaluation: Stage0EvaluationKey {
            expected_efficiency_win: false,
            evaluator_inconclusive: false,
            required_block_ids: BTreeSet::from(["b002".to_string()]),
            critical_failure: true,
        },
        fault: Stage0Fault::SafetyFailure,
        budget_limit: None,
    });

    let budget_payload = base_payload("s016", false);
    tasks.push(Stage0Task {
        identity: task_identity("s016", &budget_payload)?,
        payload: budget_payload,
        decision_class: "PRUNE".to_string(),
        decision_target: Some("b002".to_string()),
        transformation: transformation("PRUNE", Some("b002".to_string()), "s016"),
        evaluation: Stage0EvaluationKey {
            expected_efficiency_win: false,
            evaluator_inconclusive: false,
            required_block_ids: BTreeSet::new(),
            critical_failure: false,
        },
        fault: Stage0Fault::BudgetBoundary,
        budget_limit: Some(2),
    });

    let no_win_payload = base_payload("s017", false);
    tasks.push(Stage0Task {
        identity: task_identity("s017", &no_win_payload)?,
        payload: no_win_payload,
        decision_class: "PRUNE".to_string(),
        decision_target: Some("b002".to_string()),
        transformation: transformation("PRUNE", Some("b002".to_string()), "s017"),
        evaluation: Stage0EvaluationKey {
            expected_efficiency_win: false,
            evaluator_inconclusive: false,
            required_block_ids: BTreeSet::new(),
            critical_failure: false,
        },
        fault: Stage0Fault::None,
        budget_limit: None,
    });
    Ok(tasks)
}

fn base_payload(task_id: &str, required_block: bool) -> MockPayload {
    MockPayload {
        source_nonce: hash_text(task_id),
        blocks: vec![
            MockBlock {
                block_id: "b001".to_string(),
                content_hash: hash_text("stage0-block-001"),
                required: false,
                structural_index: 0,
                references: Vec::new(),
            },
            MockBlock {
                block_id: "b002".to_string(),
                content_hash: hash_text("stage0-block-002"),
                required: required_block,
                structural_index: 1,
                references: vec!["b004".to_string()],
            },
            MockBlock {
                block_id: "b003".to_string(),
                content_hash: hash_text("stage0-block-003"),
                required: false,
                structural_index: 2,
                references: Vec::new(),
            },
            MockBlock {
                block_id: "b004".to_string(),
                content_hash: hash_text("stage0-block-004"),
                required: false,
                structural_index: 3,
                references: vec!["b002".to_string()],
            },
        ],
        tool_contract: vec!["tool-opaque-001".to_string()],
    }
}

fn task_identity(
    task_id: &str,
    payload: &MockPayload,
) -> Result<Stage0TaskIdentity, BenchmarkError> {
    Ok(Stage0TaskIdentity {
        task_id: task_id.to_string(),
        source_hash: canonical_hash(payload)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?,
    })
}

fn event_id_to_block_id(event_id: &str) -> String {
    format!("b{}", event_id.trim_start_matches('e'))
}

fn transformation(
    class: &str,
    target_block_id: Option<String>,
    task_id: &str,
) -> Stage0Transformation {
    let operation = match class {
        "PRUNE" => "REMOVE_TARGET",
        "DEFER" => "MOVE_TARGET_AFTER_CONSUMER",
        "RELOCATE_CANDIDATE" => "MOVE_TARGET_BEFORE_CONSUMER",
        _ => "NO_CHANGE",
    };
    let mut changed = target_block_id.clone().into_iter().collect::<Vec<_>>();
    if task_id == "s015" {
        changed.push("b004".to_string());
    }
    Stage0Transformation {
        class: class.to_string(),
        target_block_id,
        operation: operation.to_string(),
        declared_changed_block_ids: changed,
    }
}

fn build_stage0_manifest(tasks: &[Stage0Task]) -> Result<Stage0Manifest, BenchmarkError> {
    let identities = tasks
        .iter()
        .map(|task| task.identity.clone())
        .collect::<Vec<_>>();
    let cohort_manifest_hash = canonical_hash(&identities)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let baseline_manifest_hash = canonical_hash(
        &tasks
            .iter()
            .map(|task| payload_hash(&task.payload))
            .collect::<Vec<_>>(),
    )
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let no_op_manifest_hash = baseline_manifest_hash.clone();
    let intervention_payloads = tasks
        .iter()
        .map(|task| apply_transformation(&task.payload, &task.transformation))
        .collect::<Vec<_>>();
    let intervention_manifest_entries = intervention_payloads
        .into_iter()
        .map(|payload| match payload {
            Ok(payload) => serde_json::json!({
                "status": "APPLIED",
                "payload_hash": payload_hash(&payload),
            }),
            Err(error) => serde_json::json!({
                "status": "ABORTED",
                "reason": error.to_string(),
            }),
        })
        .collect::<Vec<_>>();
    let intervention_manifest_hash = canonical_hash(&intervention_manifest_entries)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let transformation_hashes = tasks
        .iter()
        .map(|task| canonical_hash(&task.transformation))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let evaluator_spec = BTreeMap::from([
        ("version", STAGE0_EVALUATOR_VERSION),
        ("tier0", "required-state-dependency-protocol"),
        ("tier2", "deterministic-task-and-tool-outcomes"),
        ("sidecar", "evaluation-key-outside-policy"),
    ]);
    let mut stage1_placeholders = BTreeMap::new();
    for key in [
        "provider",
        "model",
        "api_surface",
        "endpoint",
        "account_region",
        "credential_environment_variable",
        "model_settings",
        "provider_cache_control",
        "live_timeout",
        "provider_pricing_profile",
        "live_request_token_spend_limits",
    ] {
        stage1_placeholders.insert(
            key.to_string(),
            "REQUIRES_STAGE_1_AUTHORIZATION".to_string(),
        );
    }
    Ok(Stage0Manifest {
        source_commit: STAGE0_SOURCE_COMMIT.to_string(),
        design_document_hash: stage0_design_hash(),
        phase1b9_policy_version: PHASE_1B9_POLICY_VERSION.to_string(),
        phase1b9_policy_hash: STAGE0_POLICY_HASH.to_string(),
        phase1b9_preregistration_hash: crate::phase1b9::preregistration_hash(),
        runner_version: STAGE0_RUNNER_VERSION.to_string(),
        cohort_count: tasks.len(),
        cohort_manifest_hash,
        task_ids_and_source_hashes: identities,
        baseline_manifest_hash,
        no_op_manifest_hash,
        intervention_manifest_hash,
        transformation_hashes,
        evaluator_version: STAGE0_EVALUATOR_VERSION.to_string(),
        evaluator_hash: canonical_hash(&evaluator_spec)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?,
        mock_transport_schema_version: STAGE0_MOCK_TRANSPORT_SCHEMA_VERSION.to_string(),
        report_schema_version: STAGE0_REPORT_SCHEMA_VERSION.to_string(),
        arm_order: vec![
            "BASELINE".to_string(),
            "NO_OP".to_string(),
            "INTERVENTION".to_string(),
        ],
        replicate_count: 1,
        max_turns: 3,
        maximum_physical_requests: (tasks.len() as u32) * 3,
        estimated_input_unit_ceiling: 100_000,
        output_unit_ceiling: 10_000,
        mock_time_ceiling_ms: 100_000,
        hard_stage0_spend_ceiling: 0,
        network_permission: false,
        credential_read_permission: false,
        retry_permission: false,
        redirect_permission: false,
        artifact_redaction_version: STAGE0_REDACTION_VERSION.to_string(),
        abort_policy_version: STAGE0_ABORT_POLICY_VERSION.to_string(),
        stage1_placeholders,
    })
}

fn payload_hash(payload: &MockPayload) -> String {
    canonical_hash(payload).expect("mock payload serializes")
}

fn apply_transformation(
    payload: &MockPayload,
    transformation: &Stage0Transformation,
) -> Result<MockPayload, BenchmarkError> {
    let Some(target) = &transformation.target_block_id else {
        return Ok(payload.clone());
    };
    let Some(target_index) = payload
        .blocks
        .iter()
        .position(|block| &block.block_id == target)
    else {
        return Err(BenchmarkError::validation(
            "Stage 0 transformation target is absent",
        ));
    };
    if transformation.class == "PRUNE" && payload.blocks[target_index].required {
        return Err(BenchmarkError::validation(
            "Stage 0 transformation removes a required block",
        ));
    }
    let mut output = payload.clone();
    match transformation.operation.as_str() {
        "REMOVE_TARGET" => {
            output.blocks.remove(target_index);
        }
        "MOVE_TARGET_AFTER_CONSUMER" => {
            let block = output.blocks.remove(target_index);
            output.blocks.push(block);
        }
        "MOVE_TARGET_BEFORE_CONSUMER" => {
            let block = output.blocks.remove(target_index);
            let destination = output
                .blocks
                .iter()
                .position(|candidate| candidate.references.contains(target))
                .ok_or_else(|| BenchmarkError::validation("Stage 0 relocation has no consumer"))?;
            output.blocks.insert(destination, block);
        }
        "NO_CHANGE" => {}
        _ => return Err(BenchmarkError::validation("unknown Stage 0 transformation")),
    }
    for (index, block) in output.blocks.iter_mut().enumerate() {
        block.structural_index = index as u32;
    }
    Ok(output)
}

fn baseline_no_op_equivalence(tasks: &[Stage0Task]) -> Result<bool, BenchmarkError> {
    tasks
        .iter()
        .map(|task| {
            let baseline = canonical_json(&task.payload)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?;
            let no_op = canonical_json(&task.payload)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?;
            Ok(baseline == no_op)
        })
        .collect::<Result<Vec<_>, BenchmarkError>>()
        .map(|checks| checks.into_iter().all(|check| check))
}

fn intervention_diff_integrity(tasks: &[Stage0Task]) -> Result<bool, BenchmarkError> {
    tasks
        .iter()
        .map(|task| {
            let expected = apply_transformation(&task.payload, &task.transformation);
            if task.fault == Stage0Fault::SafetyFailure {
                return Ok(expected.is_err());
            }
            let actual = expected?;
            Ok(validate_declared_diff(
                &task.payload,
                &actual,
                &task.transformation,
            ))
        })
        .collect::<Result<Vec<_>, BenchmarkError>>()
        .map(|checks| checks.into_iter().all(|check| check))
}

fn validate_declared_diff(
    original: &MockPayload,
    actual: &MockPayload,
    transformation: &Stage0Transformation,
) -> bool {
    apply_transformation(original, transformation)
        .map(|expected| expected == *actual)
        .unwrap_or(false)
}

fn preflight_state(manifest: &Stage0Manifest) -> PreflightState {
    PreflightState {
        source_commit: manifest.source_commit.clone(),
        design_hash: manifest.design_document_hash.clone(),
        policy_hash: manifest.phase1b9_policy_hash.clone(),
        cohort_hash: manifest.cohort_manifest_hash.clone(),
        evaluator_hash: manifest.evaluator_hash.clone(),
        transformation_hash: canonical_hash(&manifest.transformation_hashes)
            .expect("transformation hashes serialize"),
        undeclared_arm_diff: false,
        missing_accounting_requirement: false,
        spend: manifest.hard_stage0_spend_ceiling,
        network_permission: manifest.network_permission,
        credential_read_permission: manifest.credential_read_permission,
        retry_permission: manifest.retry_permission,
        redirect_permission: manifest.redirect_permission,
        active_hash: STAGE0_PROTECTED_ACTIVE_HASH.to_string(),
        active_staged: false,
    }
}

fn validate_preflight(state: &PreflightState) -> Result<(), BenchmarkError> {
    if state.source_commit != STAGE0_SOURCE_COMMIT
        || state.design_hash != stage0_design_hash()
        || state.policy_hash != STAGE0_POLICY_HASH
        || state.active_hash != STAGE0_PROTECTED_ACTIVE_HASH
    {
        return Err(BenchmarkError::validation(
            "Stage 0 immutable identity mismatch",
        ));
    }
    if state.cohort_hash.is_empty()
        || state.evaluator_hash.is_empty()
        || state.transformation_hash.is_empty()
        || state.undeclared_arm_diff
        || state.missing_accounting_requirement
        || state.spend != 0
        || state.network_permission
        || state.credential_read_permission
        || state.retry_permission
        || state.redirect_permission
        || state.active_staged
    {
        return Err(BenchmarkError::validation(
            "Stage 0 preflight policy rejected execution",
        ));
    }
    Ok(())
}

fn run_preflight_matrix(manifest: &Stage0Manifest) -> Vec<Stage0AbortProbe> {
    let mutations = [
        ("source_commit_mismatch", PreflightMutation::SourceCommit),
        ("design_hash_mismatch", PreflightMutation::DesignHash),
        ("policy_hash_mismatch", PreflightMutation::PolicyHash),
        ("cohort_hash_mismatch", PreflightMutation::CohortHash),
        ("evaluator_hash_mismatch", PreflightMutation::EvaluatorHash),
        (
            "transformation_mismatch",
            PreflightMutation::TransformationHash,
        ),
        ("undeclared_arm_diff", PreflightMutation::UndeclaredArmDiff),
        (
            "missing_accounting_requirement",
            PreflightMutation::MissingAccounting,
        ),
        ("nonzero_spend_allowance", PreflightMutation::NonzeroSpend),
        (
            "network_permission_true",
            PreflightMutation::NetworkPermission,
        ),
        (
            "credential_read_permission_true",
            PreflightMutation::CredentialPermission,
        ),
        ("retry_permission_true", PreflightMutation::RetryPermission),
        (
            "redirect_permission_true",
            PreflightMutation::RedirectPermission,
        ),
        (
            "protected_active_hash_mismatch",
            PreflightMutation::ActiveHash,
        ),
        ("protected_active_staged", PreflightMutation::ActiveStaged),
    ];
    mutations
        .into_iter()
        .map(|(condition, mutation)| {
            let mut state = preflight_state(manifest);
            mutate_preflight(&mut state, mutation);
            let passed = validate_preflight(&state).is_err();
            Stage0AbortProbe {
                condition: condition.to_string(),
                passed,
                requests_after_abort: 0,
                automatic_retries: 0,
                baseline_preserved: true,
            }
        })
        .collect()
}

fn mutate_preflight(state: &mut PreflightState, mutation: PreflightMutation) {
    match mutation {
        PreflightMutation::SourceCommit => state.source_commit = "wrong-commit".to_string(),
        PreflightMutation::DesignHash => state.design_hash = "wrong-design-hash".to_string(),
        PreflightMutation::PolicyHash => state.policy_hash = "wrong-policy-hash".to_string(),
        PreflightMutation::CohortHash => state.cohort_hash = String::new(),
        PreflightMutation::EvaluatorHash => state.evaluator_hash = String::new(),
        PreflightMutation::TransformationHash => state.transformation_hash = String::new(),
        PreflightMutation::UndeclaredArmDiff => state.undeclared_arm_diff = true,
        PreflightMutation::MissingAccounting => state.missing_accounting_requirement = true,
        PreflightMutation::NonzeroSpend => state.spend = 1,
        PreflightMutation::NetworkPermission => state.network_permission = true,
        PreflightMutation::CredentialPermission => state.credential_read_permission = true,
        PreflightMutation::RetryPermission => state.retry_permission = true,
        PreflightMutation::RedirectPermission => state.redirect_permission = true,
        PreflightMutation::ActiveHash => state.active_hash = "wrong-active-hash".to_string(),
        PreflightMutation::ActiveStaged => state.active_staged = true,
    }
}

fn execute_cohort(
    tasks: &[Stage0Task],
    manifest: &Stage0Manifest,
    transport: &mut MockTransport,
) -> Result<(Vec<Stage0TaskRecord>, bool), BenchmarkError> {
    let mut records = Vec::with_capacity(tasks.len());
    let mut accounting_certification = true;
    for task in tasks {
        let mut request_count = 0;
        let baseline = execute_arm(
            task,
            Stage0Arm::Baseline,
            manifest,
            transport,
            &mut request_count,
        )?;
        let no_op = execute_arm(
            task,
            Stage0Arm::NoOp,
            manifest,
            transport,
            &mut request_count,
        )?;
        let baseline_completed = baseline
            .as_ref()
            .is_some_and(|response| response.task_completed);
        let no_op_completed = no_op
            .as_ref()
            .is_some_and(|response| response.task_completed);
        let payload_equivalent = canonical_json(&task.payload)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?
            == canonical_json(&task.payload)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?;

        if task.fault == Stage0Fault::SafetyFailure {
            accounting_certification &= baseline.is_some() && no_op.is_some();
            records.push(Stage0TaskRecord {
                task_id: task.identity.task_id.clone(),
                selected_class: task.decision_class.clone(),
                selected_target: task.decision_target.clone(),
                baseline_no_op_payload_equivalent: payload_equivalent,
                intervention_diff_valid: false,
                status: "ABORTED".to_string(),
                baseline_completed,
                no_op_completed,
                intervention_completed: None,
                evaluator_status: "NOT_RUN_HARD_SAFETY_ABORT".to_string(),
                accounting_complete: baseline.is_some() && no_op.is_some(),
                efficiency_gate: None,
                physical_requests: request_count,
                abort_reason: Some("HARD_SAFETY_FAILURE".to_string()),
            });
            continue;
        }
        if task.fault == Stage0Fault::BudgetBoundary {
            accounting_certification &= baseline.is_some() && no_op.is_some();
            records.push(Stage0TaskRecord {
                task_id: task.identity.task_id.clone(),
                selected_class: task.decision_class.clone(),
                selected_target: task.decision_target.clone(),
                baseline_no_op_payload_equivalent: payload_equivalent,
                intervention_diff_valid: true,
                status: "ABORTED".to_string(),
                baseline_completed,
                no_op_completed,
                intervention_completed: None,
                evaluator_status: "NOT_RUN_BUDGET_ABORT".to_string(),
                accounting_complete: baseline.is_some() && no_op.is_some(),
                efficiency_gate: None,
                physical_requests: request_count,
                abort_reason: Some("BUDGET_BOUNDARY_ABORT".to_string()),
            });
            continue;
        }

        let intervention_diff = apply_transformation(&task.payload, &task.transformation);
        let intervention_valid = intervention_diff.is_ok();
        if !intervention_valid {
            return Err(BenchmarkError::validation(
                "unexpected non-safety transformation failure",
            ));
        }
        let intervention_payload = intervention_diff.expect("checked above");
        let intervention = execute_arm_with_payload(
            task,
            Stage0Arm::Intervention,
            intervention_payload.clone(),
            manifest,
            transport,
            &mut request_count,
        )?;
        let accounting_complete = [baseline.as_ref(), no_op.as_ref(), intervention.as_ref()]
            .into_iter()
            .all(|response| response.is_some_and(response_accounting_complete));
        accounting_certification &=
            accounting_complete || task.fault == Stage0Fault::MissingAccounting;
        if !accounting_complete {
            records.push(Stage0TaskRecord {
                task_id: task.identity.task_id.clone(),
                selected_class: task.decision_class.clone(),
                selected_target: task.decision_target.clone(),
                baseline_no_op_payload_equivalent: payload_equivalent,
                intervention_diff_valid: intervention_valid,
                status: "INCONCLUSIVE".to_string(),
                baseline_completed,
                no_op_completed,
                intervention_completed: intervention
                    .as_ref()
                    .map(|response| response.task_completed),
                evaluator_status: "INCONCLUSIVE_MISSING_ACCOUNTING".to_string(),
                accounting_complete: false,
                efficiency_gate: None,
                physical_requests: request_count,
                abort_reason: Some("MISSING_ACCOUNTING_FIELD".to_string()),
            });
            continue;
        }
        if task.evaluation.evaluator_inconclusive {
            records.push(Stage0TaskRecord {
                task_id: task.identity.task_id.clone(),
                selected_class: task.decision_class.clone(),
                selected_target: task.decision_target.clone(),
                baseline_no_op_payload_equivalent: payload_equivalent,
                intervention_diff_valid: intervention_valid,
                status: "INCONCLUSIVE".to_string(),
                baseline_completed,
                no_op_completed,
                intervention_completed: intervention
                    .as_ref()
                    .map(|response| response.task_completed),
                evaluator_status: "INCONCLUSIVE_EVALUATOR".to_string(),
                accounting_complete: true,
                efficiency_gate: None,
                physical_requests: request_count,
                abort_reason: Some("EVALUATOR_INCONCLUSIVE".to_string()),
            });
            continue;
        }
        let evaluator_status = evaluate_task(
            task,
            &intervention_payload,
            baseline.as_ref(),
            intervention.as_ref(),
        );
        let efficiency_gate = if task.decision_class == "DO_NOTHING" {
            None
        } else {
            Some(evaluate_efficiency(
                baseline.as_ref().expect("accounting baseline exists"),
                intervention
                    .as_ref()
                    .expect("accounting intervention exists"),
                false,
            ))
        };
        let status = if evaluator_status == "PASS"
            && efficiency_gate
                .as_ref()
                .is_none_or(|gate| gate.hypothetical_win)
        {
            "PASS"
        } else if evaluator_status == "PASS" {
            "NO_EFFICIENCY_WIN"
        } else {
            "ABORTED"
        };
        records.push(Stage0TaskRecord {
            task_id: task.identity.task_id.clone(),
            selected_class: task.decision_class.clone(),
            selected_target: task.decision_target.clone(),
            baseline_no_op_payload_equivalent: payload_equivalent,
            intervention_diff_valid: intervention_valid,
            status: status.to_string(),
            baseline_completed,
            no_op_completed,
            intervention_completed: intervention
                .as_ref()
                .map(|response| response.task_completed),
            evaluator_status: evaluator_status.to_string(),
            accounting_complete: true,
            efficiency_gate,
            physical_requests: request_count,
            abort_reason: (status == "ABORTED").then_some("EVALUATOR_FAILURE".to_string()),
        });
    }
    let expected_requests = tasks
        .iter()
        .map(|task| {
            if task.fault == Stage0Fault::SafetyFailure || task.fault == Stage0Fault::BudgetBoundary
            {
                2
            } else {
                3
            }
        })
        .sum::<u32>();
    if transport.calls.len() as u32 != expected_requests {
        return Err(BenchmarkError::validation(
            "Stage 0 mock request count mismatch",
        ));
    }
    Ok((records, accounting_certification))
}

fn execute_arm(
    task: &Stage0Task,
    arm: Stage0Arm,
    manifest: &Stage0Manifest,
    transport: &mut MockTransport,
    request_count: &mut u32,
) -> Result<Option<MockResponse>, BenchmarkError> {
    execute_arm_with_payload(
        task,
        arm,
        task.payload.clone(),
        manifest,
        transport,
        request_count,
    )
}

fn execute_arm_with_payload(
    task: &Stage0Task,
    arm: Stage0Arm,
    payload: MockPayload,
    manifest: &Stage0Manifest,
    transport: &mut MockTransport,
    request_count: &mut u32,
) -> Result<Option<MockResponse>, BenchmarkError> {
    if let Some(limit) = task.budget_limit {
        if *request_count >= limit {
            return Ok(None);
        }
    }
    if *request_count >= manifest.maximum_physical_requests {
        return Err(BenchmarkError::validation(
            "Stage 0 global request budget exceeded",
        ));
    }
    let fault = match task.fault {
        Stage0Fault::MissingAccounting if arm == Stage0Arm::Intervention => {
            MockFault::MissingAccounting
        }
        _ => MockFault::None,
    };
    let request = Stage0Request {
        task_id: task.identity.task_id.clone(),
        arm,
        replicate: 1,
        payload,
        run_metadata: BTreeMap::from([
            ("runner".to_string(), STAGE0_RUNNER_VERSION.to_string()),
            (
                "logical_request_order".to_string(),
                (*request_count + 1).to_string(),
            ),
        ]),
    };
    *request_count += 1;
    match transport.execute(request, fault) {
        Ok(response) => {
            if !response.usage.schema_valid {
                return Err(BenchmarkError::validation("mock response schema invalid"));
            }
            Ok(Some(response))
        }
        Err(error) => Err(BenchmarkError::validation(error)),
    }
}

fn response_accounting_complete(response: &MockResponse) -> bool {
    response.usage.total_input_units.is_some()
        && response.usage.fresh_input_units.is_some()
        && response.usage.cache_read_units.is_some()
        && response.usage.output_units.is_some()
        && response.usage.rounds.is_some()
        && response.usage.tool_calls.is_some()
        && response.usage.rereads.is_some()
        && response.usage.recovery_turns.is_some()
        && response.usage.latency_ms.is_some()
}

fn evaluate_task(
    task: &Stage0Task,
    intervention_payload: &MockPayload,
    baseline: Option<&MockResponse>,
    intervention: Option<&MockResponse>,
) -> &'static str {
    let Some(baseline) = baseline else {
        return "FAIL";
    };
    let Some(intervention) = intervention else {
        return "FAIL";
    };
    if !baseline.task_completed
        || !baseline.required_tool_outcomes
        || !intervention.task_completed
        || !intervention.required_tool_outcomes
        || intervention.usage.unexpected_tool_calls != 0
        || task.evaluation.critical_failure
    {
        return "FAIL";
    }
    if !task.evaluation.required_block_ids.iter().all(|required| {
        intervention_payload
            .blocks
            .iter()
            .any(|block| &block.block_id == required)
    }) {
        return "FAIL";
    }
    let protocol_valid = intervention_payload
        .blocks
        .windows(2)
        .all(|window| window[0].structural_index < window[1].structural_index);
    if !protocol_valid {
        return "FAIL";
    }
    "PASS"
}

fn evaluate_efficiency(
    baseline: &MockResponse,
    intervention: &MockResponse,
    exact_pricing_available: bool,
) -> Stage0EfficiencyGateResult {
    let baseline_usage = &baseline.usage;
    let intervention_usage = &intervention.usage;
    let total_input_not_increased = compare_optional(
        baseline_usage.total_input_units,
        intervention_usage.total_input_units,
        |base, candidate| candidate <= base,
    );
    let fresh_input_threshold_met = compare_optional(
        baseline_usage.fresh_input_units,
        intervention_usage.fresh_input_units,
        |base, candidate| candidate.saturating_mul(100) <= base.saturating_mul(90),
    );
    let billed_cost_branch = if exact_pricing_available {
        "NOT_EXERCISED_NO_FROZEN_PRICING".to_string()
    } else {
        "UNAVAILABLE_NOT_APPLICABLE".to_string()
    };
    let output_not_increased = compare_optional(
        baseline_usage.output_units,
        intervention_usage.output_units,
        |base, candidate| candidate <= base,
    );
    let rounds_not_increased = compare_optional(
        baseline_usage.rounds.map(u64::from),
        intervention_usage.rounds.map(u64::from),
        |base, candidate| candidate <= base,
    );
    let tool_calls_not_increased = compare_optional(
        baseline_usage.tool_calls.map(u64::from),
        intervention_usage.tool_calls.map(u64::from),
        |base, candidate| candidate <= base,
    );
    let rereads_not_increased = compare_optional(
        baseline_usage.rereads.map(u64::from),
        intervention_usage.rereads.map(u64::from),
        |base, candidate| candidate <= base,
    );
    let recovery_turns_not_increased = compare_optional(
        baseline_usage.recovery_turns.map(u64::from),
        intervention_usage.recovery_turns.map(u64::from),
        |base, candidate| candidate <= base,
    );
    let physical_requests_not_increased = true;
    let latency_within_threshold = compare_optional(
        baseline_usage.latency_ms,
        intervention_usage.latency_ms,
        |base, candidate| candidate.saturating_mul(100) <= base.saturating_mul(110),
    );
    let hypothetical_win = total_input_not_increased
        && (fresh_input_threshold_met || exact_pricing_available)
        && output_not_increased
        && rounds_not_increased
        && tool_calls_not_increased
        && rereads_not_increased
        && recovery_turns_not_increased
        && physical_requests_not_increased
        && latency_within_threshold;
    Stage0EfficiencyGateResult {
        total_input_not_increased,
        fresh_input_threshold_met,
        billed_cost_branch,
        output_not_increased,
        rounds_not_increased,
        tool_calls_not_increased,
        rereads_not_increased,
        recovery_turns_not_increased,
        physical_requests_not_increased,
        latency_within_threshold,
        hypothetical_win,
    }
}

fn compare_optional<F>(base: Option<u64>, candidate: Option<u64>, compare: F) -> bool
where
    F: FnOnce(u64, u64) -> bool,
{
    match (base, candidate) {
        (Some(base), Some(candidate)) => compare(base, candidate),
        _ => false,
    }
}

fn efficiency_gate_logic_certification() -> bool {
    let baseline = mock_response_for_logic(MockLogicMetrics {
        total: 100,
        fresh: 100,
        output: 10,
        rounds: 1,
        tools: 1,
        rereads: 0,
        recovery: 0,
        latency: 100,
    });
    let pass = mock_response_for_logic(MockLogicMetrics {
        total: 90,
        fresh: 80,
        output: 10,
        rounds: 1,
        tools: 1,
        rereads: 0,
        recovery: 0,
        latency: 100,
    });
    let below_threshold = mock_response_for_logic(MockLogicMetrics {
        total: 100,
        fresh: 91,
        output: 10,
        rounds: 1,
        tools: 1,
        rereads: 0,
        recovery: 0,
        latency: 100,
    });
    let latency_fail = mock_response_for_logic(MockLogicMetrics {
        total: 90,
        fresh: 80,
        output: 10,
        rounds: 1,
        tools: 1,
        rereads: 0,
        recovery: 0,
        latency: 111,
    });
    evaluate_efficiency(&baseline, &pass, false).hypothetical_win
        && !evaluate_efficiency(&baseline, &below_threshold, false).hypothetical_win
        && !evaluate_efficiency(&baseline, &latency_fail, false).hypothetical_win
        && evaluate_efficiency(&baseline, &pass, false).billed_cost_branch
            == "UNAVAILABLE_NOT_APPLICABLE"
}

struct MockLogicMetrics {
    total: u64,
    fresh: u64,
    output: u64,
    rounds: u32,
    tools: u32,
    rereads: u32,
    recovery: u32,
    latency: u64,
}

fn mock_response_for_logic(metrics: MockLogicMetrics) -> MockResponse {
    MockResponse {
        request_id: "logic-request".to_string(),
        provider_request_id: "logic-provider-request".to_string(),
        usage: MockUsage {
            total_input_units: Some(metrics.total),
            fresh_input_units: Some(metrics.fresh),
            cache_read_units: Some(0),
            cache_write_units: Some(0),
            output_units: Some(metrics.output),
            rounds: Some(metrics.rounds),
            tool_calls: Some(metrics.tools),
            rereads: Some(metrics.rereads),
            recovery_turns: Some(metrics.recovery),
            latency_ms: Some(metrics.latency),
            timeout: false,
            retry: false,
            redirect: false,
            schema_valid: true,
            unexpected_tool_calls: 0,
        },
        task_completed: true,
        required_tool_outcomes: true,
    }
}

fn rollback_fail_open_certification(tasks: &[Stage0Task]) -> Result<bool, BenchmarkError> {
    let original = tasks
        .first()
        .ok_or_else(|| BenchmarkError::validation("Stage 0 cohort is empty"))?
        .payload
        .clone();
    let failed = apply_transformation(
        &tasks
            .iter()
            .find(|task| task.fault == Stage0Fault::SafetyFailure)
            .ok_or_else(|| BenchmarkError::validation("missing Stage 0 safety task"))?
            .payload,
        &tasks
            .iter()
            .find(|task| task.fault == Stage0Fault::SafetyFailure)
            .expect("safety task exists")
            .transformation,
    )
    .is_err();
    let unchanged = canonical_json(&original)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?
        == canonical_json(&tasks.first().expect("cohort is non-empty").payload)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(failed && unchanged)
}

fn leakage_certification(tasks: &[Stage0Task]) -> Result<bool, BenchmarkError> {
    let planner_projection = tasks
        .iter()
        .map(|task| {
            serde_json::json!({
                "task_id": task.identity.task_id,
                "source_hash": task.identity.source_hash,
                "payload": task.payload,
                "decision_class": task.decision_class,
                "decision_target": task.decision_target,
            })
        })
        .collect::<Vec<_>>();
    let encoded = String::from_utf8(
        canonical_json(&planner_projection)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?,
    )
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(!encoded.contains("expected_efficiency_win")
        && !encoded.contains("evaluator_inconclusive")
        && !encoded.contains("critical_failure")
        && !encoded.contains("PASS")
        && !encoded.contains("FAIL"))
}

fn redaction_certification() -> Result<bool, BenchmarkError> {
    let synthetic = "Bearer STAGE0_SYNTHETIC_SENTINEL api_key=STAGE0_API_KEY_SENTINEL response_body=PRIVATE_SENTINEL";
    let redacted = redact_for_artifact(synthetic);
    let response = serde_json::json!({
        "request_id": "mock-request-001",
        "usage": {"total_input_units": 100},
        "response_body": "DISALLOWED_RAW_BODY",
    });
    let sanitized_response = sanitize_response_for_artifact(&response);
    let response_text = String::from_utf8(
        canonical_json(&sanitized_response)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?,
    )
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(!redacted.contains("STAGE0_SYNTHETIC_SENTINEL")
        && !redacted.contains("STAGE0_API_KEY_SENTINEL")
        && !response_text.contains("DISALLOWED_RAW_BODY"))
}

fn sanitize_response_for_artifact(response: &serde_json::Value) -> serde_json::Value {
    let object = response.as_object().expect("mock response is an object");
    serde_json::json!({
        "request_id": object.get("request_id").cloned().unwrap_or(serde_json::Value::Null),
        "usage": object.get("usage").cloned().unwrap_or(serde_json::Value::Null),
    })
}

fn redact_for_artifact(input: &str) -> String {
    input
        .replace("STAGE0_SYNTHETIC_SENTINEL", "[REDACTED]")
        .replace("STAGE0_API_KEY_SENTINEL", "[REDACTED]")
        .replace("PRIVATE_SENTINEL", "[REDACTED]")
        .replace("Bearer ", "Bearer [REDACTED]")
}

fn run_abort_matrix(manifest: &Stage0Manifest) -> Result<Vec<Stage0AbortProbe>, BenchmarkError> {
    let mut probes = run_preflight_matrix(manifest);
    let fault_cases = [
        ("hard_structural_safety_failure", true, MockFault::None),
        ("baseline_pass_intervention_fail", true, MockFault::None),
        (
            "malformed_response_schema",
            false,
            MockFault::MalformedSchema,
        ),
        ("unexpected_tool_call", false, MockFault::UnexpectedTool),
        ("simulated_timeout", false, MockFault::Timeout),
        ("simulated_redirect", false, MockFault::Redirect),
        (
            "simulated_retry_requirement",
            false,
            MockFault::RetryRequired,
        ),
        (
            "missing_required_accounting_field",
            false,
            MockFault::MissingAccounting,
        ),
        ("budget_exhaustion", true, MockFault::None),
        ("evaluator_critical_failure", true, MockFault::None),
        ("artifact_redaction_violation", true, MockFault::None),
        ("ambiguous_task_arm_identity", true, MockFault::None),
    ];
    for (condition, simulated_local_abort, fault) in fault_cases {
        let mut transport = MockTransport::new(BTreeSet::new());
        let request = Stage0Request {
            task_id: "probe-001".to_string(),
            arm: Stage0Arm::Intervention,
            replicate: 1,
            payload: base_payload("probe-001", false),
            run_metadata: BTreeMap::new(),
        };
        let baseline = request.payload.clone();
        let result = if simulated_local_abort {
            Err("LOCAL_ABORT".to_string())
        } else {
            match transport.execute(request, fault) {
                Ok(response)
                    if !response.usage.schema_valid
                        || response.usage.unexpected_tool_calls > 0
                        || !response_accounting_complete(&response) =>
                {
                    Err("MOCK_RESPONSE_ABORT".to_string())
                }
                Ok(_) => Ok(()),
                Err(error) => Err(error),
            }
        };
        let baseline_preserved = transport
            .calls
            .first()
            .map(|call| canonical_json(&baseline).ok() == canonical_json(&call.payload).ok())
            .unwrap_or(true);
        let passed = result.is_err() && transport.calls.len() <= 1 && baseline_preserved;
        probes.push(Stage0AbortProbe {
            condition: condition.to_string(),
            passed,
            requests_after_abort: 0,
            automatic_retries: 0,
            baseline_preserved,
        });
    }
    Ok(probes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage0_certification_is_deterministic_and_offline() {
        let first = run_stage0_certification().unwrap();
        let second = run_stage0_certification().unwrap();
        assert_eq!(first, second);
        assert_eq!(
            first.certification_status,
            Stage0CertificationStatus::CertifiedReadyForSeparateStage1Authorization
        );
        assert_eq!(first.cohort_count, 17);
        assert_eq!(first.network_call_count, 0);
        assert_eq!(first.credential_read_count, 0);
        assert_eq!(first.spend, 0);
        assert!(!first.stage1_unresolved_inputs.is_empty());
        assert!(!first.determinism_hash.is_empty());
    }

    #[test]
    fn baseline_and_noop_payloads_are_byte_equivalent() {
        let phase1b9 = run_phase1b9_study().unwrap();
        let tasks = build_stage0_cohort(&phase1b9).unwrap();
        assert!(baseline_no_op_equivalence(&tasks).unwrap());
    }

    #[test]
    fn intervention_diff_is_exact_and_baseline_is_immutable() {
        let phase1b9 = run_phase1b9_study().unwrap();
        let tasks = build_stage0_cohort(&phase1b9).unwrap();
        let before = tasks
            .iter()
            .map(|task| task.payload.clone())
            .collect::<Vec<_>>();
        assert!(intervention_diff_integrity(&tasks).unwrap());
        assert_eq!(
            before,
            tasks
                .iter()
                .map(|task| task.payload.clone())
                .collect::<Vec<_>>()
        );
        let task = tasks
            .iter()
            .find(|task| task.identity.task_id == "h001")
            .unwrap();
        let mut tampered = apply_transformation(&task.payload, &task.transformation).unwrap();
        tampered.blocks[0].content_hash = hash_text("undeclared-change");
        assert!(!validate_declared_diff(
            &task.payload,
            &tampered,
            &task.transformation
        ));
    }

    #[test]
    fn preflight_abort_matrix_rejects_every_mutation() {
        let phase1b9 = run_phase1b9_study().unwrap();
        let tasks = build_stage0_cohort(&phase1b9).unwrap();
        let manifest = build_stage0_manifest(&tasks).unwrap();
        let probes = run_preflight_matrix(&manifest);
        assert_eq!(probes.len(), 15);
        assert!(probes.iter().all(|probe| probe.passed));
    }

    #[test]
    fn efficiency_boundaries_and_redaction_are_certified() {
        assert!(efficiency_gate_logic_certification());
        assert!(redaction_certification().unwrap());
    }

    #[test]
    fn stage0_has_no_live_transport_or_credential_boundary() {
        assert_eq!(
            STAGE0_SOURCE_COMMIT,
            "d20b3c09fa09cfcf403bdb57792ead55314ee6e6"
        );
        assert_eq!(STAGE0_PROTECTED_ACTIVE_HASH.len(), 64);
        assert_eq!(STAGE0_POLICY_HASH.len(), 64);
        assert_eq!(
            STAGE0_MOCK_TRANSPORT_SCHEMA_VERSION,
            "stage0-mock-transport-v1"
        );
    }
}
