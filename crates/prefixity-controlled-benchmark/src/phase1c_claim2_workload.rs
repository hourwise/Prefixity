//! Offline materialization primitives for the Phase 1C Claim-2 workload.
//!
//! This module does not call a model or provider. Case fixtures bind pinned
//! local bodies to the frozen Phase 1B.9 evidence policy, render exact request
//! bodies when raw arm history exists, and expose a deterministic state machine
//! for the later separately authorized runner.

use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, sha256_hex};
use crate::model::{ActorRole, EventType, PlannerInput, RelationType, SourceKind};
use crate::phase1b9::{
    self, BlindedTrace, ResearchInterventionClass, ResearchPolicyCandidates, ResearchPolicyDecision,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const CLAIM2_CASE_SCHEMA_ID: &str = "prefixity.phase1c.claim2-workload-case";
pub const CLAIM2_CASE_SCHEMA_VERSION: u32 = 1;
pub const CLAIM2_MODEL_LABEL: &str = "lmstudio-community/Qwen3.5-9B-GGUF:Q4_K_M";
pub const CLAIM2_CONTEXT_TOKENS: u32 = 8192;
pub const CLAIM2_MAX_OUTPUT_TOKENS: u32 = 1024;
pub const CLAIM2_INPUT_PREFLIGHT_TOKENS: u32 = 6000;
pub const CLAIM2_ADVANCING_OUTPUT_PROTOCOL_ID: &str =
    "prefixity.phase1c.claim2-canonical-advancing-output.v1";

/// Return the sole canonical UTF-8 wire representation for an advancing action.
/// The output is compact JSON with one `action_id` field and no terminator.
pub fn canonical_claim2_action_output(action_id: &str) -> String {
    let encoded_action_id =
        serde_json::to_string(action_id).expect("serializing a string cannot fail");
    format!("{{\"action_id\":{encoded_action_id}}}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Claim2CaseKind {
    Positive,
    Control,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Claim2AssetKind {
    PromptText,
    ContextAttachment,
    EnvironmentReceipt,
    NativeExportBody,
}

/// An exact body pinned as a local file. The loader checks its bytes and path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2PinnedAsset {
    pub asset_id: String,
    pub relative_path: String,
    pub sha256: String,
    pub kind: Claim2AssetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Claim2PromptRole {
    System,
    User,
    Assistant,
}

/// Dynamic prompt parts are limited to pinned text, natural event bodies, and
/// this arm's own prior outputs. A result remains a separate receipt message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Claim2PromptPart {
    Asset { asset_id: String },
    EventBody { event_id: String },
    PriorAssistantOutput { request_slot: u8 },
    PriorEnvironmentReceipt { request_slot: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2PromptMessage {
    pub role: Claim2PromptRole,
    pub parts: Vec<Claim2PromptPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2RequestTemplate {
    pub request_slot: u8,
    pub messages: Vec<Claim2PromptMessage>,
}

/// One deterministic environment transition for an action in the finite menu.
/// Result event IDs are tied to their originating action and receipt asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2ActionTransition {
    pub action_slot: u8,
    pub action_id: String,
    pub action_event_id: String,
    pub result_event_id: String,
    pub result_asset_id: String,
    pub state_after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2TokenProofInputs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d3_low: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b3_high: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dsum_low: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum_b_high: Option<u64>,
}

/// Public fixture manifest. Evaluation keys are read separately and never
/// serialized with the public case or passed to policy selection/rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2CaseManifest {
    pub schema_id: String,
    pub schema_version: u32,
    pub case_id: String,
    pub kind: Claim2CaseKind,
    pub planner_input: PlannerInput,
    pub assets: Vec<Claim2PinnedAsset>,
    pub action_menu: Vec<Claim2ActionTransition>,
    pub request_templates: Vec<Claim2RequestTemplate>,
    pub evaluation_key_path: String,
    pub evaluation_key_sha256: String,
    /// Planning estimate only: never an output cap or a tokenizer upper bound.
    pub assistant_output_planning_bytes: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_proof_inputs: Option<Claim2TokenProofInputs>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2EvaluationKey {
    pub expected_action_ids: Vec<String>,
    pub expected_states_after_action: Vec<String>,
    pub expected_result_event_ids: Vec<String>,
    pub expected_final_answer: Value,
    pub required_event_ids: Vec<String>,
    pub required_relation_ids: Vec<String>,
    pub critical_event_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct LoadedAsset {
    descriptor: Claim2PinnedAsset,
    canonical_path: PathBuf,
    text: String,
    bytes: Vec<u8>,
}

/// Deliberately not serde serializable: this holds the separate evaluation key.
#[derive(Debug, Clone)]
pub struct LoadedClaim2Case {
    manifest: Claim2CaseManifest,
    assets: BTreeMap<String, LoadedAsset>,
    evaluation_key: Claim2EvaluationKey,
    blinded_trace: BlindedTrace,
    fingerprint: String,
}

impl LoadedClaim2Case {
    pub fn manifest(&self) -> &Claim2CaseManifest {
        &self.manifest
    }

    pub fn case_id(&self) -> &str {
        &self.manifest.case_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Claim2Selection {
    pub decision: ResearchPolicyDecision,
    pub candidates: ResearchPolicyCandidates,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Claim2ProjectionMode {
    Baseline,
    NoOp,
    Intervention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim2ProjectedTrace {
    pub mode: Claim2ProjectionMode,
    pub actual_selection: Claim2Selection,
    pub planner_input: PlannerInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Claim2SlotStatus {
    Planned,
    Pass,
    Fail,
    Inconclusive,
    NotExecutedAfterFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2SlotRecord {
    pub request_slot: u8,
    pub status: Claim2SlotStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_assistant_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Claim2EnvironmentReceipt {
    pub request_slot: u8,
    pub source_event_id: String,
    pub text: String,
    pub state_after: String,
}

/// An arm-local state machine. It accepts exactly three outputs in order and
/// cannot copy outputs from another arm or re-issue a rendered request.
#[derive(Debug)]
pub struct Claim2ArmState {
    case_id: String,
    case_fingerprint: String,
    mode: Claim2ProjectionMode,
    slots: Vec<Claim2SlotRecord>,
    assistant_outputs: Vec<String>,
    environment_outputs: Vec<Claim2EnvironmentReceipt>,
    selected_actions: Vec<String>,
    selected_action_event_ids: Vec<String>,
    rendered_request_json: Vec<Vec<u8>>,
    rendered_messages: Vec<Vec<Claim2RenderedMessage>>,
    output_schema_valid: Vec<bool>,
    terminal_assessment: Option<Claim2Evaluation>,
    next_request_slot: u8,
    awaiting_output: bool,
    failed: bool,
    finished: bool,
    procedural_inconclusive: bool,
}

impl Claim2ArmState {
    pub fn new(case: &LoadedClaim2Case, mode: Claim2ProjectionMode) -> Self {
        Self {
            case_id: case.manifest.case_id.clone(),
            case_fingerprint: case.fingerprint.clone(),
            mode,
            slots: (1..=3)
                .map(|request_slot| Claim2SlotRecord {
                    request_slot,
                    status: Claim2SlotStatus::Planned,
                    raw_assistant_output: None,
                })
                .collect(),
            assistant_outputs: Vec::new(),
            environment_outputs: Vec::new(),
            selected_actions: Vec::new(),
            selected_action_event_ids: Vec::new(),
            rendered_request_json: Vec::new(),
            rendered_messages: Vec::new(),
            output_schema_valid: Vec::new(),
            terminal_assessment: None,
            next_request_slot: 1,
            awaiting_output: false,
            failed: false,
            finished: false,
            procedural_inconclusive: false,
        }
    }

    pub fn slots(&self) -> &[Claim2SlotRecord] {
        &self.slots
    }

    pub fn mode(&self) -> Claim2ProjectionMode {
        self.mode
    }

    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    pub fn environment_receipts(&self) -> &[Claim2EnvironmentReceipt] {
        &self.environment_outputs
    }

    /// Return a static, no-output request projection for measurement. This
    /// preview cannot accept or borrow another arm's history.
    pub fn preview_request(
        &self,
        case: &LoadedClaim2Case,
        request_slot: u8,
    ) -> Result<Claim2RenderedRequest, BenchmarkError> {
        self.check_case(case)?;
        render_projection(case, self.mode, request_slot, &[], &[])
    }

    pub fn render_next(
        &mut self,
        case: &LoadedClaim2Case,
    ) -> Result<Claim2RenderedRequest, BenchmarkError> {
        self.check_case(case)?;
        if self.finished || self.failed || self.next_request_slot > 3 {
            return Err(BenchmarkError::validation(
                "arm has no executable request slot",
            ));
        }
        if self.awaiting_output {
            return Err(BenchmarkError::validation(
                "request slot was already rendered and awaits one output",
            ));
        }
        let request_slot = self.next_request_slot;
        let rendered = render_projection(
            case,
            self.mode,
            request_slot,
            &self.assistant_outputs,
            &self.environment_outputs,
        )?;
        self.rendered_request_json
            .push(rendered.request_json.clone());
        self.rendered_messages.push(rendered.messages.clone());
        self.awaiting_output = true;
        Ok(rendered)
    }

    /// Store one raw model response; this is only an offline transition API.
    pub fn record_output(
        &mut self,
        case: &LoadedClaim2Case,
        raw: String,
    ) -> Result<Option<Claim2Evaluation>, BenchmarkError> {
        self.check_case(case)?;
        if self.finished || !self.awaiting_output || self.next_request_slot > 3 {
            return Err(BenchmarkError::validation(
                "output is repeated, out of order, or the arm is finished",
            ));
        }
        self.awaiting_output = false;
        let slot = self.next_request_slot;
        let index = usize::from(slot - 1);
        self.slots[index].raw_assistant_output = Some(raw.clone());
        self.assistant_outputs.push(raw.clone());
        self.output_schema_valid.push(false);
        if slot < 3 {
            let expected_action =
                case.evaluation_key.expected_action_ids[usize::from(slot - 1)].as_str();
            if raw != canonical_claim2_action_output(expected_action) {
                self.fail_from(slot);
                let evaluation = failed_evaluation(case, "noncanonical_intermediate_action_output");
                self.terminal_assessment = Some(evaluation.clone());
                return Ok(Some(evaluation));
            }
            let action_output: Claim2ActionOutput = match serde_json::from_str(&raw) {
                Ok(output) => output,
                Err(_) => {
                    self.fail_from(slot);
                    let evaluation =
                        failed_evaluation(case, "malformed_or_out_of_schema_action_output");
                    self.terminal_assessment = Some(evaluation.clone());
                    return Ok(Some(evaluation));
                }
            };
            self.output_schema_valid[index] = true;
            let Some(transition) = case.manifest.action_menu.iter().find(|entry| {
                entry.action_slot == slot && entry.action_id == action_output.action_id
            }) else {
                self.fail_from(slot);
                let evaluation = failed_evaluation(case, "unknown_or_out_of_menu_action");
                self.terminal_assessment = Some(evaluation.clone());
                return Ok(Some(evaluation));
            };
            let expected_state =
                case.evaluation_key.expected_states_after_action[usize::from(slot - 1)].as_str();
            if transition.action_id != expected_action || transition.state_after != expected_state {
                self.fail_from(slot);
                let evaluation =
                    failed_evaluation(case, "intermediate_action_or_state_oracle_failed");
                self.terminal_assessment = Some(evaluation.clone());
                return Ok(Some(evaluation));
            }
            let result_asset = case
                .assets
                .get(&transition.result_asset_id)
                .ok_or_else(|| {
                    BenchmarkError::validation("action transition references missing result asset")
                })?;
            self.selected_actions.push(transition.action_id.clone());
            self.selected_action_event_ids
                .push(transition.action_event_id.clone());
            self.environment_outputs.push(Claim2EnvironmentReceipt {
                request_slot: slot,
                source_event_id: transition.result_event_id.clone(),
                text: result_asset.text.clone(),
                state_after: transition.state_after.clone(),
            });
            self.slots[index].status = Claim2SlotStatus::Pass;
            self.next_request_slot += 1;
            return Ok(None);
        }

        let final_output: Claim2FinalOutput = match serde_json::from_str(&raw) {
            Ok(output) => output,
            Err(_) => {
                self.fail_from(slot);
                let evaluation = failed_evaluation(case, "malformed_or_out_of_schema_final_output");
                self.terminal_assessment = Some(evaluation.clone());
                return Ok(Some(evaluation));
            }
        };
        self.output_schema_valid[index] = answer_shape_matches(
            &case.evaluation_key.expected_final_answer,
            &final_output.answer,
        );
        if !self.output_schema_valid[index] {
            self.fail_from(slot);
            let evaluation = failed_evaluation(case, "malformed_or_out_of_schema_final_output");
            self.terminal_assessment = Some(evaluation.clone());
            return Ok(Some(evaluation));
        }
        self.slots[index].status = Claim2SlotStatus::Pass;
        self.finished = true;
        self.next_request_slot += 1;
        let evaluation = evaluate_answer(case, self, &final_output.answer)?;
        if evaluation.status == Claim2SlotStatus::Fail {
            self.slots[index].status = Claim2SlotStatus::Fail;
        }
        self.terminal_assessment = Some(evaluation.clone());
        Ok(Some(evaluation))
    }

    pub fn mark_inconclusive(&mut self) -> Result<(), BenchmarkError> {
        if self.finished || self.failed || self.next_request_slot > 3 || !self.awaiting_output {
            return Err(BenchmarkError::validation(
                "inconclusive status requires one rendered but unresolved request",
            ));
        }
        self.slots[usize::from(self.next_request_slot - 1)].status = Claim2SlotStatus::Inconclusive;
        for slot in self
            .slots
            .iter_mut()
            .skip(usize::from(self.next_request_slot))
        {
            slot.status = Claim2SlotStatus::NotExecutedAfterFailure;
        }
        self.finished = true;
        self.awaiting_output = false;
        Ok(())
    }

    fn check_case(&self, case: &LoadedClaim2Case) -> Result<(), BenchmarkError> {
        if self.case_id != case.manifest.case_id || self.case_fingerprint != case.fingerprint {
            return Err(BenchmarkError::validation(
                "arm state belongs to another case",
            ));
        }
        Ok(())
    }

    fn fail_from(&mut self, slot: u8) {
        self.failed = true;
        self.finished = true;
        self.slots[usize::from(slot - 1)].status = Claim2SlotStatus::Fail;
        for remaining in self.slots.iter_mut().skip(usize::from(slot)) {
            remaining.status = Claim2SlotStatus::NotExecutedAfterFailure;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2Evaluation {
    pub case_id: String,
    pub status: Claim2SlotStatus,
    /// Task/model result independent of paired-arm procedural integrity.
    /// `None` means no terminal model assessment was possible.
    pub model_status: Option<Claim2SlotStatus>,
    pub action_checks_passed: bool,
    pub intermediate_state_checks_passed: bool,
    pub receipt_identity_checks_passed: bool,
    pub final_answer_passed: bool,
    pub final_answer_shape_valid: bool,
    pub required_context_available: usize,
    pub required_context_total: usize,
    pub required_context_recall: Option<f64>,
    pub required_context_preserved: bool,
    pub required_relations_preserved: bool,
    pub critical_events_preserved: bool,
    pub structurally_complete: bool,
    pub failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2RequestMetrics {
    pub fixed_content_utf8_bytes: usize,
    pub carried_assistant_utf8_bytes: usize,
    pub carried_environment_utf8_bytes: usize,
    pub attachment_utf8_bytes: usize,
    pub omitted_attachment_utf8_bytes: usize,
    pub structural_wrapper_bytes: usize,
    pub json_escape_bytes: usize,
    pub exact_request_json_bytes: usize,
    pub assistant_output_planning_bytes: usize,
    pub raw_prior_output_planning_utf8_bytes: usize,
    pub unbound_raw_assistant_slots: Vec<u8>,
    pub unbound_environment_receipt_slots: Vec<u8>,
    pub possible_environment_receipt_sizes: Vec<Claim2ReceiptSize>,
    pub token_count_status: Claim2TokenCountStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2ReceiptSize {
    pub request_slot: u8,
    pub action_id: String,
    pub source_event_id: String,
    pub utf8_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Claim2TokenCountStatus {
    ExactTokenizationRequired,
    WithinPreflightLimit,
    OverPreflightLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2RenderedRequest {
    pub case_id: String,
    pub arm: Claim2ProjectionMode,
    pub request_slot: u8,
    pub dispatchable: bool,
    pub messages: Vec<Claim2RenderedMessage>,
    pub request_json: Vec<u8>,
    pub metrics: Claim2RequestMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2RenderedMessage {
    pub role: String,
    pub content: String,
    /// Message event bodies actually rendered in this message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_event_ids: Vec<String>,
    /// Tool/result receipts actually rendered in this message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_receipt_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2PairComparison {
    pub baseline_noop_match: bool,
    pub intervention_pre_treatment_match: bool,
    pub status: Claim2SlotStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct Claim2WireRequest<'a> {
    model: &'static str,
    messages: &'a [Claim2WireMessage<'a>],
    max_tokens: u32,
    temperature: u8,
    top_p: u8,
    seed: u8,
    stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct Claim2WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim2PrecheckSlot {
    pub case_id: String,
    pub arm: Claim2ProjectionMode,
    pub request_slot: u8,
    pub metrics: Claim2RequestMetrics,
    pub dispatchable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Claim2TokenGuardResult {
    ExactTokenizationRequired,
    WithinLimit,
    OverLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Claim2MaterialityResult {
    ExactTokenizationRequired,
    Qualifies,
    DoesNotQualify,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim2ActionOutput {
    action_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim2FinalOutput {
    answer: Value,
}

/// Load the public manifest and separate evaluation key, validate pinned files,
/// provenance, closed-world graph, and exact frozen policy admission.
pub fn load_case(case_json_path: &Path) -> Result<LoadedClaim2Case, BenchmarkError> {
    let root = case_json_path
        .parent()
        .ok_or_else(|| BenchmarkError::validation("case manifest has no parent directory"))?
        .canonicalize()
        .map_err(|source| BenchmarkError::Io {
            path: case_json_path.to_path_buf(),
            source,
        })?;
    let manifest_path = case_json_path
        .canonicalize()
        .map_err(|source| BenchmarkError::Io {
            path: case_json_path.to_path_buf(),
            source,
        })?;
    if !manifest_path.starts_with(&root) {
        return Err(BenchmarkError::validation(
            "case manifest symlink escapes its case directory",
        ));
    }
    let manifest_bytes = read_file(&manifest_path)?;
    let manifest: Claim2CaseManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|source| BenchmarkError::InvalidJson {
            path: manifest_path.clone(),
            source,
        })?;
    let mut assets = BTreeMap::new();
    for descriptor in &manifest.assets {
        validate_asset_id(&descriptor.asset_id)?;
        if assets.contains_key(&descriptor.asset_id) {
            return Err(BenchmarkError::validation("duplicate pinned asset ID"));
        }
        let path = resolve_fixture_path(&root, &descriptor.relative_path)?;
        let bytes = read_file(&path)?;
        verify_sha256(&descriptor.asset_id, &descriptor.sha256, &bytes)?;
        let text = String::from_utf8(bytes.clone()).map_err(|_| {
            BenchmarkError::validation(format!("pinned asset {} is not UTF-8", descriptor.asset_id))
        })?;
        assets.insert(
            descriptor.asset_id.clone(),
            LoadedAsset {
                descriptor: descriptor.clone(),
                canonical_path: path,
                text,
                bytes,
            },
        );
    }
    let eval_path = resolve_fixture_path(&root, &manifest.evaluation_key_path)?;
    if assets
        .values()
        .any(|asset| asset.canonical_path == eval_path)
        || assets
            .values()
            .any(|asset| asset.descriptor.sha256 == manifest.evaluation_key_sha256)
    {
        return Err(BenchmarkError::validation(
            "evaluation key path or exact bytes must not be included among model-visible pinned assets",
        ));
    }
    let eval_bytes = read_file(&eval_path)?;
    verify_sha256(
        "evaluation key",
        &manifest.evaluation_key_sha256,
        &eval_bytes,
    )?;
    let evaluation_key: Claim2EvaluationKey =
        serde_json::from_slice(&eval_bytes).map_err(|source| BenchmarkError::InvalidJson {
            path: eval_path,
            source,
        })?;
    let blinded_trace = validate_manifest(&manifest, &assets, &evaluation_key)?;
    let fingerprint =
        canonical_hash(&manifest).map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let loaded = LoadedClaim2Case {
        manifest,
        assets,
        evaluation_key,
        blinded_trace,
        fingerprint,
    };
    let selection = select(&loaded)?;
    validate_expected_policy(&loaded, &selection)?;
    Ok(loaded)
}

/// Re-check the public case and return a fresh blinded representation.
pub fn validate_case(case: &LoadedClaim2Case) -> Result<(), BenchmarkError> {
    let blinded = validate_manifest(&case.manifest, &case.assets, &case.evaluation_key)?;
    if blinded != case.blinded_trace {
        return Err(BenchmarkError::validation(
            "loaded case blinded trace changed",
        ));
    }
    let selection = select(case)?;
    validate_expected_policy(case, &selection)
}

/// Use the exact Phase 1B.9 rule helpers and expose their evidence, without
/// ranking candidates by size or applying an alternate selector.
pub fn select(case: &LoadedClaim2Case) -> Result<Claim2Selection, BenchmarkError> {
    let blinded = phase1b9::blinded_trace(&case.manifest.planner_input)?;
    let candidates = phase1b9::research_policy_candidates(&blinded);
    let decision = phase1b9::research_policy(&blinded);
    Ok(Claim2Selection {
        decision,
        candidates,
    })
}

/// Identity BASELINE, forced DO_NOTHING NO_OP, or the exact selected frozen
/// decision for INTERVENTION. Selection is still run for all three modes.
pub fn apply(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
) -> Result<Claim2ProjectedTrace, BenchmarkError> {
    let actual_selection = select(case)?;
    let decision = match mode {
        Claim2ProjectionMode::Baseline => ResearchPolicyDecision {
            class: ResearchInterventionClass::DoNothing,
            target_event_id: None,
            rule: "BASELINE_IDENTITY".to_string(),
        },
        Claim2ProjectionMode::NoOp => ResearchPolicyDecision {
            class: ResearchInterventionClass::DoNothing,
            target_event_id: None,
            rule: "FORCED_NO_OP".to_string(),
        },
        Claim2ProjectionMode::Intervention => actual_selection.decision.clone(),
    };
    let planner_input = match mode {
        Claim2ProjectionMode::Baseline => case.manifest.planner_input.clone(),
        _ => phase1b9::apply_decision(&case.manifest.planner_input, &decision)?,
    };
    Ok(Claim2ProjectedTrace {
        mode,
        actual_selection,
        planner_input,
    })
}

/// Alias for the shared API name used by fixture authors and case workers.
pub fn project_trace(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
) -> Result<PlannerInput, BenchmarkError> {
    Ok(apply(case, mode)?.planner_input)
}

/// Render the arm's next exact compact request using only its own history.
pub fn render(
    case: &LoadedClaim2Case,
    arm: &mut Claim2ArmState,
) -> Result<Claim2RenderedRequest, BenchmarkError> {
    arm.render_next(case)
}

/// Internal renderer also serves no-output previews and prechecks; callers
/// cannot use it to attach another arm's raw history.
fn render_projection(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    request_slot: u8,
    prior_assistant_outputs: &[String],
    prior_environment_outputs: &[Claim2EnvironmentReceipt],
) -> Result<Claim2RenderedRequest, BenchmarkError> {
    if !(1..=3).contains(&request_slot) {
        return Err(BenchmarkError::validation(
            "request_slot must be one of the three frozen slots",
        ));
    }
    let expected_outputs = usize::from(request_slot - 1);
    if prior_assistant_outputs.len() > expected_outputs
        || prior_environment_outputs.len() > expected_outputs
    {
        return Err(BenchmarkError::validation(
            "render received future or cross-slot history",
        ));
    }
    let template = case
        .manifest
        .request_templates
        .iter()
        .find(|template| template.request_slot == request_slot)
        .ok_or_else(|| BenchmarkError::validation("missing request template"))?;
    let effective_mode = if mode == Claim2ProjectionMode::Intervention && request_slot < 3 {
        Claim2ProjectionMode::Baseline
    } else {
        mode
    };
    let projection = apply(case, effective_mode)?;
    let visible_events = projection
        .planner_input
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    let original_visible_events = case
        .manifest
        .planner_input
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut unbound_assistant = BTreeSet::new();
    let mut unbound_environment = BTreeSet::new();
    let mut metrics = Claim2RequestMetrics {
        fixed_content_utf8_bytes: 0,
        carried_assistant_utf8_bytes: 0,
        carried_environment_utf8_bytes: 0,
        attachment_utf8_bytes: 0,
        omitted_attachment_utf8_bytes: 0,
        structural_wrapper_bytes: 0,
        json_escape_bytes: 0,
        exact_request_json_bytes: 0,
        assistant_output_planning_bytes: case.manifest.assistant_output_planning_bytes,
        raw_prior_output_planning_utf8_bytes: 0,
        unbound_raw_assistant_slots: Vec::new(),
        unbound_environment_receipt_slots: Vec::new(),
        possible_environment_receipt_sizes: Vec::new(),
        token_count_status: Claim2TokenCountStatus::ExactTokenizationRequired,
    };
    let mut messages = Vec::with_capacity(template.messages.len());
    for message in &template.messages {
        let role = match message.role {
            Claim2PromptRole::System => "system",
            Claim2PromptRole::User => "user",
            Claim2PromptRole::Assistant => "assistant",
        };
        let mut content = String::new();
        let mut source_event_id = None;
        let mut source_event_ids = Vec::new();
        let mut source_receipt_event_ids = Vec::new();
        for part in &message.parts {
            match part {
                Claim2PromptPart::Asset { asset_id } => {
                    let asset = case.assets.get(asset_id).ok_or_else(|| {
                        BenchmarkError::validation(format!("unknown prompt asset {asset_id}"))
                    })?;
                    content.push_str(&asset.text);
                    metrics.fixed_content_utf8_bytes += asset.bytes.len();
                }
                Claim2PromptPart::EventBody { event_id } => {
                    let asset = case
                        .assets
                        .values()
                        .find(|asset| asset.descriptor.event_id.as_deref() == Some(event_id))
                        .ok_or_else(|| {
                            BenchmarkError::validation(format!(
                                "no pinned event body for {event_id}"
                            ))
                        })?;
                    if visible_events.contains(event_id.as_str()) {
                        content.push_str(&asset.text);
                        metrics.attachment_utf8_bytes += asset.bytes.len();
                        source_event_ids.push(event_id.clone());
                    } else if original_visible_events.contains(event_id.as_str()) {
                        metrics.omitted_attachment_utf8_bytes += asset.bytes.len();
                    }
                }
                Claim2PromptPart::PriorAssistantOutput { request_slot } => {
                    let Some(output) = prior_assistant_outputs.get(usize::from(*request_slot - 1))
                    else {
                        unbound_assistant.insert(*request_slot);
                        continue;
                    };
                    content.push_str(output);
                    metrics.carried_assistant_utf8_bytes += output.len();
                }
                Claim2PromptPart::PriorEnvironmentReceipt { request_slot } => {
                    let Some(output) = prior_environment_outputs
                        .iter()
                        .find(|output| output.request_slot == *request_slot)
                    else {
                        unbound_environment.insert(*request_slot);
                        continue;
                    };
                    if source_event_id
                        .replace(output.source_event_id.clone())
                        .is_some()
                    {
                        return Err(BenchmarkError::validation(
                            "one prompt message cannot collapse multiple environment receipts",
                        ));
                    }
                    source_receipt_event_ids.push(output.source_event_id.clone());
                    content.push_str(&output.text);
                    metrics.carried_environment_utf8_bytes += output.text.len();
                }
            }
        }
        messages.push(Claim2RenderedMessage {
            role: role.to_string(),
            content,
            source_event_ids,
            source_receipt_event_ids,
            source_event_id,
        });
    }
    metrics.unbound_raw_assistant_slots = unbound_assistant.into_iter().collect();
    metrics.unbound_environment_receipt_slots = unbound_environment.into_iter().collect();
    metrics.raw_prior_output_planning_utf8_bytes = metrics
        .unbound_raw_assistant_slots
        .len()
        .saturating_mul(case.manifest.assistant_output_planning_bytes);
    for request_slot in &metrics.unbound_environment_receipt_slots {
        for transition in case
            .manifest
            .action_menu
            .iter()
            .filter(|transition| transition.action_slot == *request_slot)
        {
            let asset = case
                .assets
                .get(&transition.result_asset_id)
                .expect("validated action menu binds each receipt asset");
            metrics
                .possible_environment_receipt_sizes
                .push(Claim2ReceiptSize {
                    request_slot: *request_slot,
                    action_id: transition.action_id.clone(),
                    source_event_id: transition.result_event_id.clone(),
                    utf8_bytes: asset.bytes.len(),
                });
        }
    }
    let wire_messages = messages
        .iter()
        .map(|message| Claim2WireMessage {
            role: &message.role,
            content: &message.content,
        })
        .collect::<Vec<_>>();
    let request = Claim2WireRequest {
        model: CLAIM2_MODEL_LABEL,
        messages: &wire_messages,
        max_tokens: CLAIM2_MAX_OUTPUT_TOKENS,
        temperature: 0,
        top_p: 1,
        seed: 1,
        stream: false,
    };
    let request_json = serde_json::to_vec(&request)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let empty_messages = messages
        .iter()
        .map(|message| Claim2WireMessage {
            role: &message.role,
            content: "",
        })
        .collect::<Vec<_>>();
    let empty_request = Claim2WireRequest {
        messages: &empty_messages,
        ..request
    };
    let wrapper_bytes = serde_json::to_vec(&empty_request)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?
        .len();
    let content_bytes = messages
        .iter()
        .map(|message| message.content.len())
        .sum::<usize>();
    metrics.structural_wrapper_bytes = wrapper_bytes;
    metrics.exact_request_json_bytes = request_json.len();
    metrics.json_escape_bytes = request_json
        .len()
        .saturating_sub(wrapper_bytes + content_bytes);
    // No token counts are available in this offline stage. Byte facts never
    // stand in for a count, and this crate deliberately has no dispatch path.
    metrics.token_count_status = Claim2TokenCountStatus::ExactTokenizationRequired;
    let dispatchable = false;
    Ok(Claim2RenderedRequest {
        case_id: case.manifest.case_id.clone(),
        arm: mode,
        request_slot,
        dispatchable,
        messages,
        request_json,
        metrics,
    })
}

/// Emit nine static request skeletons per case, or exactly 54 across CP01–06.
/// Missing raw histories are intentionally left unbound and non-dispatchable.
pub fn precheck_cohort(
    cases: &[LoadedClaim2Case],
) -> Result<Vec<Claim2PrecheckSlot>, BenchmarkError> {
    let expected = ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"];
    if cases.len() != expected.len()
        || cases
            .iter()
            .map(|case| case.manifest.case_id.as_str())
            .collect::<Vec<_>>()
            != expected
    {
        return Err(BenchmarkError::validation(
            "Claim-2 precheck requires CP01 through CP06 in frozen order",
        ));
    }
    let mut slots = Vec::with_capacity(54);
    for case in cases {
        validate_case(case)?;
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            for request_slot in 1..=3 {
                let rendered =
                    Claim2ArmState::new(case, mode).preview_request(case, request_slot)?;
                slots.push(Claim2PrecheckSlot {
                    case_id: case.manifest.case_id.clone(),
                    arm: mode,
                    request_slot,
                    metrics: rendered.metrics,
                    dispatchable: rendered.dispatchable,
                });
            }
        }
    }
    Ok(slots)
}

pub fn token_guard(input_tokens: Option<u64>) -> Claim2TokenGuardResult {
    let Some(input_tokens) = input_tokens else {
        return Claim2TokenGuardResult::ExactTokenizationRequired;
    };
    if input_tokens > u64::from(CLAIM2_INPUT_PREFLIGHT_TOKENS)
        || input_tokens
            .checked_add(u64::from(CLAIM2_MAX_OUTPUT_TOKENS))
            .is_none_or(|sum| sum > u64::from(CLAIM2_CONTEXT_TOKENS))
    {
        Claim2TokenGuardResult::OverLimit
    } else {
        Claim2TokenGuardResult::WithinLimit
    }
}

pub fn materiality_result(inputs: &Claim2TokenProofInputs) -> Claim2MaterialityResult {
    let (Some(d3), Some(b3), Some(dsum), Some(sum_b)) = (
        inputs.d3_low,
        inputs.b3_high,
        inputs.dsum_low,
        inputs.sum_b_high,
    ) else {
        return Claim2MaterialityResult::ExactTokenizationRequired;
    };
    if b3 == 0 || sum_b == 0 || d3 > b3 || dsum > sum_b {
        return Claim2MaterialityResult::DoesNotQualify;
    }
    let final_fraction_holds = u128::from(d3) * 100 >= u128::from(b3) * 20;
    let cumulative_fraction_holds = u128::from(dsum) * 100 >= u128::from(sum_b) * 8;
    if d3 >= 800 && final_fraction_holds && cumulative_fraction_holds {
        Claim2MaterialityResult::Qualifies
    } else {
        Claim2MaterialityResult::DoesNotQualify
    }
}

/// Checks BASELINE/NO_OP request equality and the intervention's untouched
/// request 1/2 outputs and bodies. No output is copied or normalized.
pub fn compare_paired_arms(
    baseline: &mut Claim2ArmState,
    no_op: &mut Claim2ArmState,
    intervention: &mut Claim2ArmState,
) -> Result<Claim2PairComparison, BenchmarkError> {
    if baseline.case_id != no_op.case_id || baseline.case_id != intervention.case_id {
        return Err(BenchmarkError::validation(
            "paired arm case identities differ",
        ));
    }
    if baseline.case_fingerprint != no_op.case_fingerprint
        || baseline.case_fingerprint != intervention.case_fingerprint
    {
        return Err(BenchmarkError::validation(
            "paired arm states must belong to the same pinned case revision",
        ));
    }
    if baseline.mode != Claim2ProjectionMode::Baseline
        || no_op.mode != Claim2ProjectionMode::NoOp
        || intervention.mode != Claim2ProjectionMode::Intervention
    {
        return Err(BenchmarkError::validation(
            "paired arms have incorrect mode labels",
        ));
    }
    let baseline_noop_match = arm_has_three_requests_and_outputs(baseline)
        && arm_has_three_requests_and_outputs(no_op)
        && baseline.assistant_outputs == no_op.assistant_outputs
        && baseline.rendered_request_json == no_op.rendered_request_json;
    let intervention_pre_treatment_match = baseline.rendered_request_json.len() >= 2
        && baseline.assistant_outputs.len() >= 2
        && intervention.rendered_request_json.len() >= 2
        && intervention.assistant_outputs.len() >= 2
        && (0..2).all(|index| {
            baseline.assistant_outputs.get(index) == intervention.assistant_outputs.get(index)
                && baseline.rendered_request_json.get(index)
                    == intervention.rendered_request_json.get(index)
        });
    if !baseline_noop_match {
        baseline.procedural_inconclusive = true;
        no_op.procedural_inconclusive = true;
    }
    if !intervention_pre_treatment_match {
        intervention.procedural_inconclusive = true;
    }
    let all_trajectories_complete = arm_has_three_requests_and_outputs(baseline)
        && arm_has_three_requests_and_outputs(no_op)
        && arm_has_three_requests_and_outputs(intervention);
    if !all_trajectories_complete {
        baseline.procedural_inconclusive = true;
        no_op.procedural_inconclusive = true;
        intervention.procedural_inconclusive = true;
    }
    Ok(Claim2PairComparison {
        baseline_noop_match,
        intervention_pre_treatment_match,
        status: if all_trajectories_complete
            && baseline_noop_match
            && intervention_pre_treatment_match
        {
            Claim2SlotStatus::Pass
        } else {
            Claim2SlotStatus::Inconclusive
        },
    })
}

fn arm_has_three_requests_and_outputs(arm: &Claim2ArmState) -> bool {
    arm.finished
        && !arm.awaiting_output
        && arm.rendered_request_json.len() == 3
        && arm.rendered_messages.len() == 3
        && arm.assistant_outputs.len() == 3
        && arm
            .slots
            .iter()
            .all(|slot| slot.raw_assistant_output.is_some())
}

pub fn evaluate(
    case: &LoadedClaim2Case,
    arm: &Claim2ArmState,
) -> Result<Claim2Evaluation, BenchmarkError> {
    if arm.case_id != case.manifest.case_id || arm.case_fingerprint != case.fingerprint {
        return Err(BenchmarkError::validation(
            "arm state belongs to another case",
        ));
    }
    if !arm.finished {
        return Err(BenchmarkError::validation(
            "evaluation requires a terminal arm state",
        ));
    }
    let mut evaluation = arm
        .terminal_assessment
        .clone()
        .unwrap_or_else(|| inconclusive_evaluation(case, arm));
    if arm.procedural_inconclusive {
        evaluation.status = Claim2SlotStatus::Inconclusive;
        if !evaluation
            .failure_reasons
            .iter()
            .any(|reason| reason == "procedural_pre_treatment_divergence")
        {
            evaluation
                .failure_reasons
                .push("procedural_pre_treatment_divergence".to_string());
        }
    }
    Ok(evaluation)
}

fn evaluate_answer(
    case: &LoadedClaim2Case,
    arm: &Claim2ArmState,
    answer: &Value,
) -> Result<Claim2Evaluation, BenchmarkError> {
    let key = &case.evaluation_key;
    let projected = apply(case, arm.mode)?;
    let action_checks_passed = arm.selected_actions == key.expected_action_ids;
    let observed_states = arm
        .environment_outputs
        .iter()
        .map(|output| output.state_after.clone())
        .collect::<Vec<_>>();
    let intermediate_state_checks_passed = observed_states == key.expected_states_after_action;
    let observed_receipts = arm
        .environment_outputs
        .iter()
        .map(|output| output.source_event_id.clone())
        .collect::<Vec<_>>();
    let receipt_identity_checks_passed = observed_receipts == key.expected_result_event_ids;
    let final_answer_passed = answer == &key.expected_final_answer;
    let available_events = required_events_available(case, arm, &projected.planner_input);
    let required_context_available = key
        .required_event_ids
        .iter()
        .filter(|event_id| available_events.contains(event_id.as_str()))
        .count();
    let required_context_total = key.required_event_ids.len();
    let required_context_recall = (required_context_total > 0)
        .then_some(required_context_available as f64 / required_context_total as f64);
    let required_context_preserved = required_context_available == required_context_total;
    let visible_relations = projected
        .planner_input
        .relations
        .iter()
        .map(|relation| relation.relation_id.as_str())
        .collect::<BTreeSet<_>>();
    let required_relations_preserved = key
        .required_relation_ids
        .iter()
        .all(|relation_id| visible_relations.contains(relation_id.as_str()));
    let critical_events_preserved = key
        .critical_event_ids
        .iter()
        .all(|event_id| available_events.contains(event_id.as_str()));
    let structurally_complete = arm.assistant_outputs.len() == 3
        && arm.environment_outputs.len() == 2
        && arm.rendered_request_json.len() == 3
        && arm.rendered_messages.len() == 3
        && arm.output_schema_valid.len() == 3
        && arm.output_schema_valid.iter().all(|valid| *valid);
    let mut failure_reasons = Vec::new();
    if !action_checks_passed {
        failure_reasons.push("action_mismatch".to_string());
    }
    if !intermediate_state_checks_passed {
        failure_reasons.push("intermediate_state_mismatch".to_string());
    }
    if !receipt_identity_checks_passed {
        failure_reasons.push("environment_receipt_identity_mismatch".to_string());
    }
    if !final_answer_passed {
        failure_reasons.push("final_answer_mismatch".to_string());
    }
    if !required_context_preserved {
        failure_reasons.push("required_context_missing".to_string());
    }
    if !required_relations_preserved {
        failure_reasons.push("required_dependency_missing".to_string());
    }
    if !critical_events_preserved {
        failure_reasons.push("critical_event_missing".to_string());
    }
    let passed = failure_reasons.is_empty() && structurally_complete;
    if !structurally_complete {
        failure_reasons.push("incomplete_arm".to_string());
    }
    if arm.procedural_inconclusive {
        failure_reasons.push("procedural_pre_treatment_divergence".to_string());
    }
    let model_status = if passed {
        Claim2SlotStatus::Pass
    } else {
        Claim2SlotStatus::Fail
    };
    Ok(Claim2Evaluation {
        case_id: case.manifest.case_id.clone(),
        status: if arm.procedural_inconclusive {
            Claim2SlotStatus::Inconclusive
        } else {
            model_status.clone()
        },
        model_status: Some(model_status),
        action_checks_passed,
        intermediate_state_checks_passed,
        receipt_identity_checks_passed,
        final_answer_passed,
        final_answer_shape_valid: answer_shape_matches(&key.expected_final_answer, answer),
        required_context_available,
        required_context_total,
        required_context_recall,
        required_context_preserved,
        required_relations_preserved,
        critical_events_preserved,
        structurally_complete,
        failure_reasons,
    })
}

fn failed_evaluation(case: &LoadedClaim2Case, reason: &str) -> Claim2Evaluation {
    Claim2Evaluation {
        case_id: case.manifest.case_id.clone(),
        status: Claim2SlotStatus::Fail,
        model_status: Some(Claim2SlotStatus::Fail),
        action_checks_passed: false,
        intermediate_state_checks_passed: false,
        receipt_identity_checks_passed: false,
        final_answer_passed: false,
        final_answer_shape_valid: false,
        required_context_available: 0,
        required_context_total: case.evaluation_key.required_event_ids.len(),
        required_context_recall: None,
        required_context_preserved: false,
        required_relations_preserved: false,
        critical_events_preserved: false,
        structurally_complete: false,
        failure_reasons: vec![reason.to_string()],
    }
}

fn inconclusive_evaluation(case: &LoadedClaim2Case, arm: &Claim2ArmState) -> Claim2Evaluation {
    let mut evaluation = failed_evaluation(case, "execution_inconclusive");
    evaluation.status = Claim2SlotStatus::Inconclusive;
    evaluation.model_status = None;
    evaluation.failure_reasons.clear();
    evaluation
        .failure_reasons
        .push("execution_inconclusive".to_string());
    if arm.rendered_messages.len() < 3 {
        evaluation.required_context_recall = None;
    }
    evaluation
}

fn answer_shape_matches(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(_), Value::Bool(_)) => true,
        (Value::Number(_), Value::Number(_)) => true,
        (Value::String(_), Value::String(_)) => true,
        (Value::Array(expected), Value::Array(actual)) => {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .zip(actual)
                    .all(|(expected, actual)| answer_shape_matches(expected, actual))
        }
        (Value::Object(expected), Value::Object(actual)) => {
            expected.len() == actual.len()
                && expected.iter().all(|(key, expected_value)| {
                    actual.get(key).is_some_and(|actual_value| {
                        answer_shape_matches(expected_value, actual_value)
                    })
                })
        }
        _ => false,
    }
}

fn required_events_available(
    case: &LoadedClaim2Case,
    arm: &Claim2ArmState,
    projected: &PlannerInput,
) -> BTreeSet<String> {
    let mut available = BTreeSet::new();
    let projected_ids = projected
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    let Some(final_messages) = arm.rendered_messages.get(2) else {
        return available;
    };
    for message in final_messages {
        for event_id in &message.source_event_ids {
            if projected_ids.contains(event_id.as_str())
                && case.manifest.planner_input.events.iter().any(|event| {
                    event.event_id == *event_id && event.event_type == EventType::Message
                })
            {
                available.insert(event_id.clone());
            }
        }
        for event_id in &message.source_receipt_event_ids {
            if arm
                .environment_outputs
                .iter()
                .any(|receipt| receipt.source_event_id == *event_id)
                && case.manifest.planner_input.events.iter().any(|event| {
                    event.event_id == *event_id && event.event_type == EventType::Result
                })
            {
                available.insert(event_id.clone());
            }
        }
    }
    for event_id in &arm.selected_action_event_ids {
        if projected_ids.contains(event_id.as_str())
            && case
                .manifest
                .planner_input
                .events
                .iter()
                .any(|event| event.event_id == *event_id && event.event_type == EventType::Action)
        {
            available.insert(event_id.clone());
        }
    }
    available
}

fn validate_manifest(
    manifest: &Claim2CaseManifest,
    assets: &BTreeMap<String, LoadedAsset>,
    key: &Claim2EvaluationKey,
) -> Result<BlindedTrace, BenchmarkError> {
    if manifest.schema_id != CLAIM2_CASE_SCHEMA_ID
        || manifest.schema_version != CLAIM2_CASE_SCHEMA_VERSION
        || !matches!(
            manifest.case_id.as_str(),
            "CP01" | "CP02" | "CP03" | "CP04" | "CP05" | "CP06"
        )
    {
        return Err(BenchmarkError::validation(
            "unsupported Claim-2 case schema or identity",
        ));
    }
    let expected_kind = match manifest.case_id.as_str() {
        "CP01" | "CP02" | "CP03" | "CP04" => Claim2CaseKind::Positive,
        "CP05" | "CP06" => Claim2CaseKind::Control,
        _ => unreachable!("case ID was constrained above"),
    };
    if manifest.kind != expected_kind {
        return Err(BenchmarkError::validation(
            "case ID does not match the frozen positive/control stratum",
        ));
    }
    if manifest.assistant_output_planning_bytes == 0 {
        return Err(BenchmarkError::validation(
            "assistant output planning byte estimate must be explicit and positive",
        ));
    }
    if manifest.request_templates.len() != 3
        || manifest
            .request_templates
            .iter()
            .map(|template| template.request_slot)
            .collect::<Vec<_>>()
            != [1, 2, 3]
    {
        return Err(BenchmarkError::validation(
            "fixture must declare exactly request slots 1, 2, and 3",
        ));
    }
    if manifest.action_menu.is_empty()
        || manifest.action_menu.len() > 64
        || manifest
            .action_menu
            .iter()
            .all(|action| action.action_slot != 1)
        || manifest
            .action_menu
            .iter()
            .all(|action| action.action_slot != 2)
        || manifest
            .action_menu
            .iter()
            .any(|action| !(1..=2).contains(&action.action_slot))
    {
        return Err(BenchmarkError::validation(
            "action menu must have at least one finite transition at both action slots",
        ));
    }
    let mut action_ids = BTreeSet::new();
    let mut action_event_ids = BTreeSet::new();
    let mut result_event_ids = BTreeSet::new();
    for transition in &manifest.action_menu {
        if transition.action_id.is_empty()
            || !action_ids.insert(transition.action_id.as_str())
            || !action_event_ids.insert(transition.action_event_id.as_str())
            || !result_event_ids.insert(transition.result_event_id.as_str())
        {
            return Err(BenchmarkError::validation(
                "finite action transitions must have unique action and event identities",
            ));
        }
    }
    validate_assets_and_trace(manifest, assets)?;
    validate_templates(manifest, assets)?;
    validate_key(manifest, key)?;
    let trace = phase1b9::blinded_trace(&manifest.planner_input)?;
    Ok(trace)
}

fn validate_expected_policy(
    case: &LoadedClaim2Case,
    selection: &Claim2Selection,
) -> Result<(), BenchmarkError> {
    match case.manifest.kind {
        Claim2CaseKind::Positive => {
            if selection.decision.class != ResearchInterventionClass::Prune
                || selection.decision.target_event_id.is_none()
                || selection.decision.rule != "EXACT_DUPLICATE_PRUNE"
                || selection.candidates.exact_duplicate_prune.len() != 1
                || !selection.candidates.explicit_supersession_defer.is_empty()
                || !selection.candidates.same_zone_protocol_relocate.is_empty()
            {
                return Err(BenchmarkError::validation(
                    "positive case is ineligible under the unchanged controlled policy",
                ));
            }
            let target_id = selection
                .decision
                .target_event_id
                .as_deref()
                .expect("positive admission requires one selected target");
            let final_template = manifest_template(&case.manifest, 3)?;
            if !final_template.messages.iter().flat_map(|message| &message.parts).any(|part| {
                matches!(part, Claim2PromptPart::EventBody { event_id } if event_id == target_id)
            }) {
                return Err(BenchmarkError::validation("selected positive attachment must occur in request 3's natural rendered context"));
            }
            for request_slot in [1, 2] {
                if manifest_template(&case.manifest, request_slot)?
                    .messages
                    .iter()
                    .flat_map(|message| &message.parts)
                    .any(|part| matches!(part, Claim2PromptPart::EventBody { event_id } if event_id == target_id))
                {
                    return Err(BenchmarkError::validation(
                        "the one intervention target must first occur in request 3's context",
                    ));
                }
            }
            if case
                .evaluation_key
                .required_event_ids
                .iter()
                .chain(case.evaluation_key.critical_event_ids.iter())
                .any(|event_id| event_id == target_id)
                || selection
                    .candidates
                    .exact_duplicate_prune
                    .iter()
                    .filter(|candidate| candidate.target_event_id == target_id)
                    .flat_map(|candidate| &candidate.evidence_relation_ids)
                    .any(|relation_id| {
                        case.evaluation_key
                            .required_relation_ids
                            .contains(relation_id)
                    })
            {
                return Err(BenchmarkError::validation(
                    "evaluation key cannot require the removed target or its same-state edge",
                ));
            }
        }
        Claim2CaseKind::Control => {
            if selection.decision.class != ResearchInterventionClass::DoNothing
                || selection.decision.target_event_id.is_some()
                || !selection.candidates.exact_duplicate_prune.is_empty()
                || !selection.candidates.explicit_supersession_defer.is_empty()
                || !selection.candidates.same_zone_protocol_relocate.is_empty()
            {
                return Err(BenchmarkError::validation(
                    "control is not a natural DO_NOTHING under the unchanged controlled policy",
                ));
            }
        }
    }
    Ok(())
}

fn manifest_template(
    manifest: &Claim2CaseManifest,
    request_slot: u8,
) -> Result<&Claim2RequestTemplate, BenchmarkError> {
    manifest
        .request_templates
        .iter()
        .find(|template| template.request_slot == request_slot)
        .ok_or_else(|| BenchmarkError::validation("missing request template"))
}

fn validate_assets_and_trace(
    manifest: &Claim2CaseManifest,
    assets: &BTreeMap<String, LoadedAsset>,
) -> Result<(), BenchmarkError> {
    let input = &manifest.planner_input;
    if input.events.is_empty() {
        return Err(BenchmarkError::validation(
            "planner input must contain at least one event",
        ));
    }
    if input.events.len() > crate::model::MAX_EVENTS
        || input.relations.len() > crate::model::MAX_RELATIONS
        || input.provenance.len() > crate::model::MAX_PROVENANCE
    {
        return Err(BenchmarkError::validation(
            "planner input exceeds the bounded event, relation, or provenance schema",
        ));
    }
    if input.provenance.is_empty()
        || input.provenance.iter().any(|provenance| {
            matches!(
                provenance.classification,
                crate::model::EvidenceClass::EvaluationOnly
                    | crate::model::EvidenceClass::InferredUnsafe
            ) || matches!(provenance.source_kind, SourceKind::PublicDesignReference)
        })
    {
        return Err(BenchmarkError::validation(
            "planner input provenance is absent or contains evaluation/design-only evidence",
        ));
    }
    let mut alias_owners = BTreeMap::<String, String>::new();
    let mut event_by_id = BTreeMap::new();
    for (index, event) in input.events.iter().enumerate() {
        if event.sequence_index as usize != index || event.event_id.is_empty() {
            return Err(BenchmarkError::validation(
                "event sequence must be contiguous and event IDs nonempty",
            ));
        }
        event_by_id.insert(event.event_id.as_str(), event);
        if matches!(event.event_type, EventType::Message) && event.actor_role != ActorRole::User {
            return Err(BenchmarkError::validation(
                "native context messages must retain their user-origin event role",
            ));
        }
        if event.event_type == EventType::Action && event.actor_role != ActorRole::Agent {
            return Err(BenchmarkError::validation(
                "action events must retain the agent role",
            ));
        }
        if event.event_type == EventType::Result && event.actor_role != ActorRole::Tool {
            return Err(BenchmarkError::validation(
                "result receipts must retain the tool role",
            ));
        }
        if event.event_type == EventType::Action && event.action.is_none()
            || event.event_type != EventType::Action && event.action.is_some()
        {
            return Err(BenchmarkError::validation(
                "action identity must appear only on Action events",
            ));
        }
        if event.event_type == EventType::Result && event.result.is_none()
            || event.event_type != EventType::Result && event.result.is_some()
        {
            return Err(BenchmarkError::validation(
                "result identity must appear only on Result events in this workload",
            ));
        }
        for alias in std::iter::once(event.event_id.as_str())
            .chain(
                event
                    .action
                    .as_ref()
                    .map(|action| action.action_id.as_str()),
            )
            .chain(
                event
                    .result
                    .as_ref()
                    .map(|result| result.result_id.as_str()),
            )
            .chain(event.context_block_id.as_deref())
        {
            if alias_owners
                .insert(alias.to_string(), event.event_id.clone())
                .is_some()
            {
                return Err(BenchmarkError::validation(
                    "event, action, result, and context aliases must be globally unique",
                ));
            }
        }
        if event.provenance.is_empty()
            || event.provenance.iter().any(|provenance| {
                matches!(
                    provenance.classification,
                    crate::model::EvidenceClass::EvaluationOnly
                        | crate::model::EvidenceClass::InferredUnsafe
                ) || matches!(provenance.source_kind, SourceKind::PublicDesignReference)
            })
        {
            return Err(BenchmarkError::validation("planner-visible event provenance is absent or leaks evaluation/design-only evidence"));
        }
        if event
            .parent_event_ids
            .iter()
            .chain(event.reference_event_ids.iter())
            .any(|id| {
                !event_by_id.contains_key(id.as_str())
                    && !input.events.iter().any(|other| other.event_id == *id)
            })
        {
            return Err(BenchmarkError::validation(
                "parent/reference must resolve to a closed-world event ID",
            ));
        }
    }
    for event in &input.events {
        if let Some(result) = &event.result {
            let producer = input
                .events
                .iter()
                .find(|producer| {
                    producer
                        .action
                        .as_ref()
                        .is_some_and(|action| action.action_id == result.originating_action_id)
                })
                .ok_or_else(|| BenchmarkError::validation("result origin action is missing"))?;
            let links = input
                .relations
                .iter()
                .filter(|relation| {
                    relation.relation_type == RelationType::Produces
                        && relation.from_id == result.originating_action_id
                        && relation.to_id == result.result_id
                })
                .count();
            if producer.event_type != EventType::Action || links != 1 {
                return Err(BenchmarkError::validation(
                    "result must have exactly one matching producer link",
                ));
            }
        }
        if event.event_type == EventType::Message {
            let body_assets = assets
                .values()
                .filter(|asset| asset.descriptor.event_id.as_deref() == Some(&event.event_id))
                .collect::<Vec<_>>();
            if body_assets.len() != 1
                || event.content_hash.as_deref() != Some(sha256_hex(&body_assets[0].bytes).as_str())
            {
                return Err(BenchmarkError::validation(format!(
                    "message {} must bind one byte-matching pinned body",
                    event.event_id
                )));
            }
            if !matches!(
                body_assets[0].descriptor.kind,
                Claim2AssetKind::ContextAttachment | Claim2AssetKind::NativeExportBody
            ) {
                return Err(BenchmarkError::validation(
                    "message event bodies must retain their native context-attachment channel",
                ));
            }
            if body_assets[0]
                .descriptor
                .revision_id
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err(BenchmarkError::validation(
                    "native message body must pin its source revision separately from world-state revision",
                ));
            }
        }
        if event.event_type == EventType::Result {
            let receipt_assets = assets
                .values()
                .filter(|asset| asset.descriptor.event_id.as_deref() == Some(&event.event_id))
                .collect::<Vec<_>>();
            if receipt_assets.len() != 1
                || receipt_assets[0].descriptor.kind != Claim2AssetKind::EnvironmentReceipt
                || event
                    .result
                    .as_ref()
                    .and_then(|result| result.observation_hash.as_deref())
                    != Some(sha256_hex(&receipt_assets[0].bytes).as_str())
            {
                return Err(BenchmarkError::validation(format!(
                    "result {} must bind one byte-matching pinned environment receipt",
                    event.event_id
                )));
            }
        }
    }
    for asset in assets.values() {
        if let Some(event_id) = &asset.descriptor.event_id {
            let event = event_by_id.get(event_id.as_str()).ok_or_else(|| {
                BenchmarkError::validation("pinned asset refers to unknown event")
            })?;
            match asset.descriptor.kind {
                Claim2AssetKind::ContextAttachment | Claim2AssetKind::NativeExportBody
                    if event.event_type != EventType::Message =>
                {
                    return Err(BenchmarkError::validation(
                        "attachment assets can bind only to native message events",
                    ));
                }
                Claim2AssetKind::EnvironmentReceipt if event.event_type != EventType::Result => {
                    return Err(BenchmarkError::validation(
                        "environment receipt must stay bound to a Result event",
                    ));
                }
                Claim2AssetKind::PromptText => {
                    return Err(BenchmarkError::validation(
                        "prompt-text assets cannot be attached to structural events",
                    ));
                }
                _ => {}
            }
            if asset
                .descriptor
                .revision_id
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err(BenchmarkError::validation(
                    "event-bound body/receipt assets must pin a revision identity",
                ));
            }
        }
    }
    let mut relation_ids = BTreeSet::new();
    let mut graph = BTreeMap::<String, Vec<String>>::new();
    let mut same_state_neighbors = BTreeMap::<String, Vec<String>>::new();
    for event in &input.events {
        for dependency in event
            .parent_event_ids
            .iter()
            .chain(event.reference_event_ids.iter())
        {
            let referenced = event_by_id.get(dependency.as_str()).ok_or_else(|| {
                BenchmarkError::validation("closed-world parent/reference endpoint is unknown")
            })?;
            if referenced.sequence_index >= event.sequence_index {
                return Err(BenchmarkError::validation(
                    "parent/reference dependencies must point to prior events",
                ));
            }
            graph
                .entry(event.event_id.clone())
                .or_default()
                .push(dependency.clone());
        }
    }
    for relation in &input.relations {
        if relation.relation_id.is_empty() || !relation_ids.insert(relation.relation_id.as_str()) {
            return Err(BenchmarkError::validation(
                "relation IDs must be nonempty and unique",
            ));
        }
        let from = alias_owners
            .get(&relation.from_id)
            .ok_or_else(|| BenchmarkError::validation("relation source alias is unknown"))?;
        let to = alias_owners
            .get(&relation.to_id)
            .ok_or_else(|| BenchmarkError::validation("relation target alias is unknown"))?;
        if relation.provenance.is_empty()
            || relation.provenance.iter().any(|p| {
                matches!(
                    p.classification,
                    crate::model::EvidenceClass::EvaluationOnly
                        | crate::model::EvidenceClass::InferredUnsafe
                ) || matches!(p.source_kind, SourceKind::PublicDesignReference)
            })
        {
            return Err(BenchmarkError::validation(
                "relation provenance is absent or evaluation-only",
            ));
        }
        if relation.scope != "scenario_local"
            || relation.semantics_version.as_deref()
                != Some(crate::model::RELATION_SEMANTICS_VERSION)
        {
            return Err(BenchmarkError::validation(
                "relation uses an unsupported scope or semantics revision",
            ));
        }
        match relation.relation_type {
            RelationType::Produces => {
                let from_event = event_by_id
                    .get(from.as_str())
                    .expect("alias owner is event");
                let to_event = event_by_id.get(to.as_str()).expect("alias owner is event");
                if from_event
                    .action
                    .as_ref()
                    .map(|action| action.action_id.as_str())
                    != Some(relation.from_id.as_str())
                    || to_event
                        .result
                        .as_ref()
                        .map(|result| result.result_id.as_str())
                        != Some(relation.to_id.as_str())
                    || to_event
                        .result
                        .as_ref()
                        .map(|result| result.originating_action_id.as_str())
                        != Some(relation.from_id.as_str())
                {
                    return Err(BenchmarkError::validation(
                        "producer relation does not agree with action/result identity",
                    ));
                }
            }
            RelationType::SameStateRevision => {
                let left = event_by_id
                    .get(from.as_str())
                    .expect("alias owner is event");
                let right = event_by_id.get(to.as_str()).expect("alias owner is event");
                if relation.from_id != left.event_id
                    || relation.to_id != right.event_id
                    || left.event_type != EventType::Message
                    || right.event_type != EventType::Message
                    || left.content_hash.is_none()
                    || left.content_hash != right.content_hash
                    || left.world_state_revision.is_none()
                    || left.world_state_revision != right.world_state_revision
                    || left.sequence_index >= right.sequence_index
                {
                    return Err(BenchmarkError::validation("same-state relation must join equal native Message bodies at the same explicit revision"));
                }
                same_state_neighbors
                    .entry(left.event_id.clone())
                    .or_default()
                    .push(right.event_id.clone());
                same_state_neighbors
                    .entry(right.event_id.clone())
                    .or_default()
                    .push(left.event_id.clone());
            }
            _ => {
                graph.entry(from.clone()).or_default().push(to.clone());
            }
        }
    }
    validate_acyclic(&graph)?;
    for event in &input.events {
        for id in event
            .parent_event_ids
            .iter()
            .chain(event.reference_event_ids.iter())
        {
            if !event_by_id.contains_key(id.as_str()) {
                return Err(BenchmarkError::validation(
                    "closed-world parent/reference endpoint is unknown",
                ));
            }
        }
    }
    for transition in &manifest.action_menu {
        if !(1..=2).contains(&transition.action_slot) || transition.state_after.is_empty() {
            return Err(BenchmarkError::validation(
                "action transition has an invalid slot or empty state",
            ));
        }
        let action_event = event_by_id
            .get(transition.action_event_id.as_str())
            .ok_or_else(|| BenchmarkError::validation("transition action event missing"))?;
        let result_event = event_by_id
            .get(transition.result_event_id.as_str())
            .ok_or_else(|| BenchmarkError::validation("transition result event missing"))?;
        let result_asset = assets
            .get(&transition.result_asset_id)
            .ok_or_else(|| BenchmarkError::validation("transition result asset missing"))?;
        if action_event.event_type != EventType::Action
            || action_event
                .action
                .as_ref()
                .map(|action| action.action_id.as_str())
                != Some(transition.action_id.as_str())
            || result_event.event_type != EventType::Result
            || result_event
                .result
                .as_ref()
                .map(|result| result.originating_action_id.as_str())
                != Some(transition.action_id.as_str())
            || result_asset.descriptor.event_id.as_deref()
                != Some(transition.result_event_id.as_str())
            || result_event
                .result
                .as_ref()
                .and_then(|result| result.observation_hash.as_deref())
                != Some(sha256_hex(&result_asset.bytes).as_str())
        {
            return Err(BenchmarkError::validation(
                "action transition conflicts with its frozen action/result identities",
            ));
        }
    }
    if !same_state_neighbors.is_empty() {
        for neighbors in same_state_neighbors.values() {
            if neighbors.is_empty() {
                return Err(BenchmarkError::validation(
                    "same-state relation must be an explicit pair",
                ));
            }
        }
    }
    Ok(())
}

fn validate_templates(
    manifest: &Claim2CaseManifest,
    assets: &BTreeMap<String, LoadedAsset>,
) -> Result<(), BenchmarkError> {
    for (index, template) in manifest.request_templates.iter().enumerate() {
        if index > 0 {
            let previous = &manifest.request_templates[index - 1].messages;
            if template.messages.len() < previous.len()
                || template.messages[..previous.len()] != previous[..]
            {
                return Err(BenchmarkError::validation(
                    "each request must preserve the complete prior prompt as an unchanged prefix",
                ));
            }
        }
        if template.request_slot != index as u8 + 1
            || template.messages.len() < 2
            || template.messages.first().map(|message| message.role)
                != Some(Claim2PromptRole::System)
            || template.messages.last().map(|message| message.role) != Some(Claim2PromptRole::User)
        {
            return Err(BenchmarkError::validation(
                "request template must start with system and end with user",
            ));
        }
        let mut prior_assistants = BTreeSet::new();
        let mut prior_receipts = BTreeSet::new();
        let mut event_bodies = BTreeSet::new();
        for message in &template.messages {
            for part in &message.parts {
                match part {
                    Claim2PromptPart::Asset { asset_id } => {
                        let asset = assets.get(asset_id).ok_or_else(|| {
                            BenchmarkError::validation("request references unknown asset")
                        })?;
                        if asset.descriptor.kind != Claim2AssetKind::PromptText {
                            return Err(BenchmarkError::validation("receipts and attachments must use event/history parts, not prompt text assets"));
                        }
                    }
                    Claim2PromptPart::EventBody { event_id } => {
                        let event = manifest
                            .planner_input
                            .events
                            .iter()
                            .find(|event| &event.event_id == event_id)
                            .ok_or_else(|| {
                                BenchmarkError::validation(
                                    "request references unknown context event",
                                )
                            })?;
                        if event.event_type != EventType::Message {
                            return Err(BenchmarkError::validation("event body rendering is reserved for native Message attachments; Result receipts keep their own channel"));
                        }
                        if !event_bodies.insert(event_id.as_str()) {
                            return Err(BenchmarkError::validation(
                                "a native Message occurrence can appear only once per request",
                            ));
                        }
                    }
                    Claim2PromptPart::PriorAssistantOutput { request_slot } => {
                        if *request_slot == 0
                            || *request_slot >= template.request_slot
                            || message.role != Claim2PromptRole::Assistant
                            || message.parts.len() != 1
                        {
                            return Err(BenchmarkError::validation("raw assistant history must use assistant role and refer only to prior slots"));
                        }
                        if !prior_assistants.insert(*request_slot) {
                            return Err(BenchmarkError::validation(
                                "each raw assistant output can be carried only once per request",
                            ));
                        }
                    }
                    Claim2PromptPart::PriorEnvironmentReceipt { request_slot } => {
                        if *request_slot == 0
                            || *request_slot >= template.request_slot
                            || message.role != Claim2PromptRole::User
                            || message.parts.len() != 1
                        {
                            return Err(BenchmarkError::validation("environment receipts must use user role and refer only to prior slots"));
                        }
                        if !prior_receipts.insert(*request_slot) {
                            return Err(BenchmarkError::validation(
                                "each environment receipt can be carried only once per request",
                            ));
                        }
                    }
                }
            }
        }
        let expected_prior_slots = (1..template.request_slot).collect::<BTreeSet<_>>();
        if prior_assistants != expected_prior_slots || prior_receipts != expected_prior_slots {
            return Err(BenchmarkError::validation("each request must carry this arm's complete prior assistant and environment history"));
        }
    }
    Ok(())
}

fn validate_key(
    manifest: &Claim2CaseManifest,
    key: &Claim2EvaluationKey,
) -> Result<(), BenchmarkError> {
    if key.expected_action_ids.len() != 2
        || key.expected_states_after_action.len() != 2
        || key.expected_result_event_ids.len() != 2
    {
        return Err(BenchmarkError::validation(
            "evaluation key must describe exactly two action transitions",
        ));
    }
    for slot in 1..=2 {
        if !manifest.action_menu.iter().any(|transition| {
            transition.action_slot == slot
                && transition.action_id == key.expected_action_ids[usize::from(slot - 1)]
                && transition.result_event_id
                    == key.expected_result_event_ids[usize::from(slot - 1)]
        }) {
            return Err(BenchmarkError::validation(
                "expected action/result pair is outside the finite public action menu",
            ));
        }
    }
    let event_ids = manifest
        .planner_input
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    if key
        .required_event_ids
        .iter()
        .chain(key.critical_event_ids.iter())
        .any(|id| !event_ids.contains(id.as_str()))
    {
        return Err(BenchmarkError::validation(
            "evaluation key names an event outside the frozen case",
        ));
    }
    let relation_ids = manifest
        .planner_input
        .relations
        .iter()
        .map(|relation| relation.relation_id.as_str())
        .collect::<BTreeSet<_>>();
    if key
        .required_relation_ids
        .iter()
        .any(|id| !relation_ids.contains(id.as_str()))
    {
        return Err(BenchmarkError::validation(
            "evaluation key names a relation outside the frozen case",
        ));
    }
    Ok(())
}

fn validate_acyclic(graph: &BTreeMap<String, Vec<String>>) -> Result<(), BenchmarkError> {
    let mut indegree = BTreeMap::<String, usize>::new();
    for (source, targets) in graph {
        indegree.entry(source.clone()).or_default();
        for target in targets {
            *indegree.entry(target.clone()).or_default() += 1;
        }
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<_>>();
    let mut visited = 0;
    while let Some(node) = ready.pop_front() {
        visited += 1;
        for target in graph.get(&node).into_iter().flatten() {
            let degree = indegree.get_mut(target).expect("graph target has indegree");
            *degree -= 1;
            if *degree == 0 {
                ready.push_back(target.clone());
            }
        }
    }
    if visited != indegree.len() {
        return Err(BenchmarkError::validation(
            "dependency/protocol/producer graph contains a cycle",
        ));
    }
    Ok(())
}

fn resolve_fixture_path(root: &Path, relative: &str) -> Result<PathBuf, BenchmarkError> {
    let relative_path = Path::new(relative);
    if relative_path.as_os_str().is_empty()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BenchmarkError::validation(
            "fixture asset path must be relative and contain no dot or parent components",
        ));
    }
    let resolved =
        root.join(relative_path)
            .canonicalize()
            .map_err(|source| BenchmarkError::Io {
                path: root.join(relative_path),
                source,
            })?;
    if !resolved.starts_with(root) {
        return Err(BenchmarkError::validation(
            "fixture path escapes its case directory",
        ));
    }
    Ok(resolved)
}

fn read_file(path: &Path) -> Result<Vec<u8>, BenchmarkError> {
    fs::read(path).map_err(|source| BenchmarkError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn verify_sha256(what: &str, expected: &str, bytes: &[u8]) -> Result<(), BenchmarkError> {
    let found = sha256_hex(bytes);
    if expected != found {
        return Err(BenchmarkError::HashMismatch {
            what: what.to_string(),
            expected: expected.to_string(),
            found,
        });
    }
    Ok(())
}

fn validate_asset_id(id: &str) -> Result<(), BenchmarkError> {
    if id.is_empty()
        || id.len() > 128
        || id
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || b"-_.".contains(&byte)))
    {
        return Err(BenchmarkError::validation(
            "asset ID must be a bounded ASCII identifier",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ActionIdentity, EvidenceClass, Relation, SourceProvenance, TimestampOrigin,
        RELATION_SEMANTICS_VERSION,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn provenance(locator: &str, revision: &str) -> SourceProvenance {
        SourceProvenance {
            source_kind: SourceKind::SelfAuthored,
            classification: EvidenceClass::CapturedExplicit,
            source_locator: Some(locator.to_string()),
            source_revision: Some(revision.to_string()),
            content_hash: Some(sha256_hex(locator.as_bytes())),
            note: None,
        }
    }

    // Synthetic trace rows keep all event fields visible together in the test.
    #[allow(clippy::too_many_arguments)]
    fn event(
        event_id: &str,
        sequence_index: u32,
        event_type: EventType,
        actor_role: ActorRole,
        references: Vec<String>,
        action: Option<ActionIdentity>,
        result: Option<crate::model::ResultIdentity>,
        context_block_id: Option<&str>,
        content_hash: Option<String>,
        world_state_revision: Option<&str>,
    ) -> crate::model::Event {
        crate::model::Event {
            event_id: event_id.to_string(),
            sequence_index,
            event_type,
            actor_role,
            parent_event_ids: Vec::new(),
            reference_event_ids: references,
            action,
            result,
            context_block_id: context_block_id.map(str::to_string),
            world_state_revision: world_state_revision.map(str::to_string),
            order: Some(crate::model::OrderMetadata {
                logical_tick: Some(sequence_index),
                source_timestamp: None,
                timestamp_origin: Some(TimestampOrigin::DerivedStructural),
            }),
            content_hash,
            provenance: vec![provenance(&format!("case/e{sequence_index}"), "test-rev-1")],
        }
    }

    fn relation(
        relation_id: &str,
        relation_type: RelationType,
        from_id: &str,
        to_id: &str,
    ) -> Relation {
        Relation {
            relation_id: relation_id.to_string(),
            relation_type,
            from_id: from_id.to_string(),
            to_id: to_id.to_string(),
            scope: "scenario_local".to_string(),
            semantics_version: Some(RELATION_SEMANTICS_VERSION.to_string()),
            provenance: vec![provenance(&format!("case/{relation_id}"), "test-rev-1")],
        }
    }

    // Test fixture writer mirrors the independently pinned asset fields.
    #[allow(clippy::too_many_arguments)]
    fn write_asset(
        root: &Path,
        assets: &mut Vec<Claim2PinnedAsset>,
        asset_id: &str,
        relative_path: &str,
        kind: Claim2AssetKind,
        event_id: Option<&str>,
        revision_id: Option<&str>,
        text: &str,
    ) {
        let path = root.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text.as_bytes()).unwrap();
        assets.push(Claim2PinnedAsset {
            asset_id: asset_id.to_string(),
            relative_path: relative_path.to_string(),
            sha256: sha256_hex(text.as_bytes()),
            kind,
            event_id: event_id.map(str::to_string),
            revision_id: revision_id.map(str::to_string),
        });
    }

    fn test_case_dir() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "prefixity-claim2-test-{}-{nonce}",
            std::process::id()
        ))
    }

    fn synthetic_case() -> (PathBuf, LoadedClaim2Case) {
        let root = test_case_dir();
        fs::create_dir_all(&root).unwrap();
        let task = "Inspect the named source, verify its recorded revision, and report the exact verified state.";
        let source = "Source R-41: stable exporter branch; current state is S-41.\n";
        let receipt1 = "Inspection receipt: occurrence src-41 loaded from revision R-41.\n";
        let receipt1_alt = "Inspection receipt: inspection was skipped.\n";
        let receipt2 = "Verification receipt: revision R-41 is current at state S-41.\n";
        let receipt2_alt = "Verification receipt: current revision could not be verified.\n";
        let duplicate = source;
        let mut assets = Vec::new();
        write_asset(
            &root,
            &mut assets,
            "task",
            "bodies/task.txt",
            Claim2AssetKind::ContextAttachment,
            Some("e-task"),
            Some("task-r1"),
            task,
        );
        write_asset(
            &root,
            &mut assets,
            "source",
            "bodies/source.txt",
            Claim2AssetKind::NativeExportBody,
            Some("e-source"),
            Some("rev-41"),
            source,
        );
        write_asset(
            &root,
            &mut assets,
            "receipt1",
            "bodies/receipt1.txt",
            Claim2AssetKind::EnvironmentReceipt,
            Some("e-result1"),
            Some("env-r1"),
            receipt1,
        );
        write_asset(
            &root,
            &mut assets,
            "receipt1_alt",
            "bodies/receipt1_alt.txt",
            Claim2AssetKind::EnvironmentReceipt,
            Some("e-result1-alt"),
            Some("env-r1-alt"),
            receipt1_alt,
        );
        write_asset(
            &root,
            &mut assets,
            "receipt2",
            "bodies/receipt2.txt",
            Claim2AssetKind::EnvironmentReceipt,
            Some("e-result2"),
            Some("env-r2"),
            receipt2,
        );
        write_asset(
            &root,
            &mut assets,
            "receipt2_alt",
            "bodies/receipt2_alt.txt",
            Claim2AssetKind::EnvironmentReceipt,
            Some("e-result2-alt"),
            Some("env-r2-alt"),
            receipt2_alt,
        );
        write_asset(
            &root,
            &mut assets,
            "duplicate",
            "bodies/duplicate.txt",
            Claim2AssetKind::ContextAttachment,
            Some("e-duplicate"),
            Some("rev-41"),
            duplicate,
        );
        write_asset(
            &root,
            &mut assets,
            "system",
            "prompt/system.txt",
            Claim2AssetKind::PromptText,
            None,
            None,
            "Return one JSON object matching the current step's output schema.\n",
        );
        write_asset(&root, &mut assets, "menu", "prompt/menu.txt", Claim2AssetKind::PromptText, None, None, "Slot 1 actions: inspect_source or skip_inspection. Slot 2 actions: verify_revision or accept_unverified.\n");
        write_asset(
            &root,
            &mut assets,
            "verify",
            "prompt/verify.txt",
            Claim2AssetKind::PromptText,
            None,
            None,
            "Choose the verification action using the receipt and cite the source occurrence.\n",
        );
        write_asset(
            &root,
            &mut assets,
            "final",
            "prompt/final.txt",
            Claim2AssetKind::PromptText,
            None,
            None,
            "Report source_id, revision, and status as JSON.\n",
        );

        let mut events = vec![
            event(
                "e-task",
                0,
                EventType::Message,
                ActorRole::User,
                vec![],
                None,
                None,
                Some("ctx-task"),
                Some(sha256_hex(task.as_bytes())),
                Some("task-state"),
            ),
            event(
                "e-source",
                1,
                EventType::Message,
                ActorRole::User,
                vec![],
                None,
                None,
                Some("ctx-source"),
                Some(sha256_hex(source.as_bytes())),
                Some("S-41"),
            ),
        ];
        let actions = [
            (
                "inspect_source",
                "e-action1",
                "inspect_source",
                "e-result1",
                "r-result1",
                receipt1,
                "source_loaded",
            ),
            (
                "skip_inspection",
                "e-action1-alt",
                "skip_inspection",
                "e-result1-alt",
                "r-result1-alt",
                receipt1_alt,
                "inspection_skipped",
            ),
            (
                "verify_revision",
                "e-action2",
                "verify_revision",
                "e-result2",
                "r-result2",
                receipt2,
                "revision_verified",
            ),
            (
                "accept_unverified",
                "e-action2-alt",
                "accept_unverified",
                "e-result2-alt",
                "r-result2-alt",
                receipt2_alt,
                "revision_unverified",
            ),
        ];
        for (
            offset,
            (_menu, action_event_id, action_id, result_event_id, result_id, _receipt, _state),
        ) in actions.iter().enumerate()
        {
            let sequence = 2 + (offset as u32 * 2);
            let refs = if offset < 2 {
                vec!["e-task".to_string()]
            } else {
                vec![if offset == 2 {
                    "e-result1"
                } else {
                    "e-result1-alt"
                }
                .to_string()]
            };
            events.push(event(
                action_event_id,
                sequence,
                EventType::Action,
                ActorRole::Agent,
                refs,
                Some(ActionIdentity {
                    action_id: (*action_id).to_string(),
                    tool_name: (*action_id).to_string(),
                    argument_hash: Some(sha256_hex(action_id.as_bytes())),
                }),
                None,
                None,
                None,
                Some("S-41"),
            ));
            let asset_id = match offset {
                0 => "receipt1",
                1 => "receipt1_alt",
                2 => "receipt2",
                _ => "receipt2_alt",
            };
            let text = actions[offset].5;
            events.push(event(
                result_event_id,
                sequence + 1,
                EventType::Result,
                ActorRole::Tool,
                vec![],
                None,
                Some(crate::model::ResultIdentity {
                    result_id: result_id.to_string(),
                    originating_action_id: action_id.to_string(),
                    observation_hash: Some(sha256_hex(text.as_bytes())),
                    status: Some(crate::model::ResultStatus::Success),
                }),
                None,
                None,
                Some("S-41"),
            ));
            assert!(assets.iter().any(|asset| asset.asset_id == asset_id));
        }
        events.push(event(
            "e-duplicate",
            10,
            EventType::Message,
            ActorRole::User,
            vec![],
            None,
            None,
            Some("ctx-duplicate"),
            Some(sha256_hex(duplicate.as_bytes())),
            Some("S-41"),
        ));
        let mut relations = Vec::new();
        for (relation_id, action_id, result_id) in [
            ("produce1", "inspect_source", "r-result1"),
            ("produce1-alt", "skip_inspection", "r-result1-alt"),
            ("produce2", "verify_revision", "r-result2"),
            ("produce2-alt", "accept_unverified", "r-result2-alt"),
        ] {
            relations.push(relation(
                relation_id,
                RelationType::Produces,
                action_id,
                result_id,
            ));
        }
        relations.push(relation(
            "same-state-source-duplicate",
            RelationType::SameStateRevision,
            "e-source",
            "e-duplicate",
        ));

        let actions = vec![
            Claim2ActionTransition {
                action_slot: 1,
                action_id: "inspect_source".to_string(),
                action_event_id: "e-action1".to_string(),
                result_event_id: "e-result1".to_string(),
                result_asset_id: "receipt1".to_string(),
                state_after: "source_loaded".to_string(),
            },
            Claim2ActionTransition {
                action_slot: 1,
                action_id: "skip_inspection".to_string(),
                action_event_id: "e-action1-alt".to_string(),
                result_event_id: "e-result1-alt".to_string(),
                result_asset_id: "receipt1_alt".to_string(),
                state_after: "inspection_skipped".to_string(),
            },
            Claim2ActionTransition {
                action_slot: 2,
                action_id: "verify_revision".to_string(),
                action_event_id: "e-action2".to_string(),
                result_event_id: "e-result2".to_string(),
                result_asset_id: "receipt2".to_string(),
                state_after: "revision_verified".to_string(),
            },
            Claim2ActionTransition {
                action_slot: 2,
                action_id: "accept_unverified".to_string(),
                action_event_id: "e-action2-alt".to_string(),
                result_event_id: "e-result2-alt".to_string(),
                result_asset_id: "receipt2_alt".to_string(),
                state_after: "revision_unverified".to_string(),
            },
        ];
        let msg = |role, parts| Claim2PromptMessage { role, parts };
        let asset = |asset_id: &str| Claim2PromptPart::Asset {
            asset_id: asset_id.to_string(),
        };
        let body = |event_id: &str| Claim2PromptPart::EventBody {
            event_id: event_id.to_string(),
        };
        let assistant = |request_slot| Claim2PromptPart::PriorAssistantOutput { request_slot };
        let receipt = |request_slot| Claim2PromptPart::PriorEnvironmentReceipt { request_slot };
        let templates = vec![
            Claim2RequestTemplate {
                request_slot: 1,
                messages: vec![
                    msg(Claim2PromptRole::System, vec![asset("system")]),
                    msg(
                        Claim2PromptRole::User,
                        vec![body("e-task"), body("e-source"), asset("menu")],
                    ),
                ],
            },
            Claim2RequestTemplate {
                request_slot: 2,
                messages: vec![
                    msg(Claim2PromptRole::System, vec![asset("system")]),
                    msg(
                        Claim2PromptRole::User,
                        vec![body("e-task"), body("e-source"), asset("menu")],
                    ),
                    msg(Claim2PromptRole::Assistant, vec![assistant(1)]),
                    msg(Claim2PromptRole::User, vec![receipt(1)]),
                    msg(Claim2PromptRole::User, vec![asset("verify")]),
                ],
            },
            Claim2RequestTemplate {
                request_slot: 3,
                messages: vec![
                    msg(Claim2PromptRole::System, vec![asset("system")]),
                    msg(
                        Claim2PromptRole::User,
                        vec![body("e-task"), body("e-source"), asset("menu")],
                    ),
                    msg(Claim2PromptRole::Assistant, vec![assistant(1)]),
                    msg(Claim2PromptRole::User, vec![receipt(1)]),
                    msg(Claim2PromptRole::User, vec![asset("verify")]),
                    msg(Claim2PromptRole::Assistant, vec![assistant(2)]),
                    msg(Claim2PromptRole::User, vec![receipt(2)]),
                    msg(
                        Claim2PromptRole::User,
                        vec![body("e-duplicate"), asset("final")],
                    ),
                ],
            },
        ];
        let key = Claim2EvaluationKey {
            expected_action_ids: vec!["inspect_source".to_string(), "verify_revision".to_string()],
            expected_states_after_action: vec![
                "source_loaded".to_string(),
                "revision_verified".to_string(),
            ],
            expected_result_event_ids: vec!["e-result1".to_string(), "e-result2".to_string()],
            expected_final_answer: serde_json::json!({"source_id":"src-41", "revision":"R-41", "status":"verified"}),
            required_event_ids: vec![
                "e-task".to_string(),
                "e-source".to_string(),
                "e-result1".to_string(),
                "e-result2".to_string(),
            ],
            required_relation_ids: vec!["produce1".to_string(), "produce2".to_string()],
            critical_event_ids: vec![
                "e-task".to_string(),
                "e-source".to_string(),
                "e-result1".to_string(),
                "e-result2".to_string(),
            ],
        };
        let key_bytes = serde_json::to_vec(&key).unwrap();
        fs::write(root.join("evaluation.json"), &key_bytes).unwrap();
        let manifest = Claim2CaseManifest {
            schema_id: CLAIM2_CASE_SCHEMA_ID.to_string(),
            schema_version: CLAIM2_CASE_SCHEMA_VERSION,
            case_id: "CP01".to_string(),
            kind: Claim2CaseKind::Positive,
            planner_input: PlannerInput {
                events,
                relations,
                provenance: vec![provenance("case/trace", "test-rev-1")],
            },
            assets,
            action_menu: actions,
            request_templates: templates,
            evaluation_key_path: "evaluation.json".to_string(),
            evaluation_key_sha256: sha256_hex(&key_bytes),
            assistant_output_planning_bytes: 2048,
            token_proof_inputs: None,
        };
        fs::write(
            root.join("case.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let case = load_case(&root.join("case.json")).unwrap();
        (root, case)
    }

    fn run_good_arm(
        case: &LoadedClaim2Case,
        mode: Claim2ProjectionMode,
    ) -> (Claim2ArmState, Claim2Evaluation) {
        let mut arm = Claim2ArmState::new(case, mode);
        arm.render_next(case).unwrap();
        assert!(arm
            .record_output(case, r#"{"action_id":"inspect_source"}"#.to_string())
            .unwrap()
            .is_none());
        arm.render_next(case).unwrap();
        assert!(arm
            .record_output(case, r#"{"action_id":"verify_revision"}"#.to_string())
            .unwrap()
            .is_none());
        arm.render_next(case).unwrap();
        let result = arm
            .record_output(
                case,
                r#"{"answer":{"source_id":"src-41","revision":"R-41","status":"verified"}}"#
                    .to_string(),
            )
            .unwrap()
            .unwrap();
        (arm, result)
    }

    fn run_arm_with_answer(
        case: &LoadedClaim2Case,
        answer: Value,
    ) -> (Claim2ArmState, Claim2Evaluation) {
        let mut arm = Claim2ArmState::new(case, Claim2ProjectionMode::Baseline);
        for action_id in ["inspect_source", "verify_revision"] {
            arm.render_next(case).unwrap();
            assert!(arm
                .record_output(
                    case,
                    serde_json::json!({"action_id": action_id}).to_string()
                )
                .unwrap()
                .is_none());
        }
        arm.render_next(case).unwrap();
        let evaluation = arm
            .record_output(case, serde_json::json!({"answer": answer}).to_string())
            .unwrap()
            .unwrap();
        (arm, evaluation)
    }

    fn replace_rendered_slot_json(arm: &mut Claim2ArmState, slot_index: usize) {
        let wire_messages = arm.rendered_messages[slot_index]
            .iter()
            .map(|message| Claim2WireMessage {
                role: &message.role,
                content: &message.content,
            })
            .collect::<Vec<_>>();
        let request = Claim2WireRequest {
            model: CLAIM2_MODEL_LABEL,
            messages: &wire_messages,
            max_tokens: CLAIM2_MAX_OUTPUT_TOKENS,
            temperature: 0,
            top_p: 1,
            seed: 1,
            stream: false,
        };
        arm.rendered_request_json[slot_index] = serde_json::to_vec(&request).unwrap();
    }

    fn complete_two_actions_and_render_final(case: &LoadedClaim2Case) -> Claim2ArmState {
        let mut arm = Claim2ArmState::new(case, Claim2ProjectionMode::Baseline);
        for action_id in ["inspect_source", "verify_revision"] {
            arm.render_next(case).unwrap();
            assert!(arm
                .record_output(
                    case,
                    serde_json::json!({"action_id": action_id}).to_string()
                )
                .unwrap()
                .is_none());
        }
        arm.render_next(case).unwrap();
        arm
    }

    #[test]
    fn intermediate_output_is_exact_while_final_schema_still_accepts_whitespace() {
        let (root, case) = synthetic_case();
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        arm.render_next(&case).unwrap();
        let noncanonical = format!(
            "{}{}",
            " ".repeat(case.manifest.assistant_output_planning_bytes + 1),
            canonical_claim2_action_output("inspect_source")
        );
        let failure = arm
            .record_output(&case, noncanonical.clone())
            .unwrap()
            .unwrap();
        assert_eq!(failure.status, Claim2SlotStatus::Fail);
        assert_eq!(
            arm.slots()[0].raw_assistant_output.as_deref(),
            Some(noncanonical.as_str())
        );
        assert_eq!(arm.slots()[0].status, Claim2SlotStatus::Fail);
        assert_eq!(
            arm.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        assert!(arm.environment_receipts().is_empty());

        let mut final_arm = complete_two_actions_and_render_final(&case);
        let final_raw = format!(
            "{}{}",
            " ".repeat(case.manifest.assistant_output_planning_bytes + 1),
            serde_json::json!({"answer": case.evaluation_key.expected_final_answer.clone()})
        );
        assert!(final_raw.len() > case.manifest.assistant_output_planning_bytes);
        let final_result = final_arm
            .record_output(&case, final_raw.clone())
            .unwrap()
            .unwrap();
        assert_eq!(final_result.status, Claim2SlotStatus::Pass);
        assert_eq!(
            final_arm.slots()[2].raw_assistant_output.as_deref(),
            Some(final_raw.as_str())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn canonical_action_encoder_has_a_pinned_compact_json_shape() {
        for (action_id, expected) in [
            ("inspect_source", r#"{"action_id":"inspect_source"}"#),
            (
                "run_retry_regression",
                r#"{"action_id":"run_retry_regression"}"#,
            ),
            ("é", r#"{"action_id":"é"}"#),
            ("a\nb", r#"{"action_id":"a\nb"}"#),
        ] {
            let encoded = canonical_claim2_action_output(action_id);
            assert_eq!(encoded, expected);
            assert!(!encoded.starts_with('\u{feff}'));
            assert!(!encoded.ends_with(' '));
            assert!(!encoded.ends_with('\n'));
        }
    }

    #[test]
    fn historical_phase1b9_report_remains_byte_exact() {
        let actual =
            phase1b9::canonical_phase1b9_report_json(&phase1b9::run_phase1b9_study().unwrap())
                .unwrap();
        let report: phase1b9::Phase1b9Report = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/phase-1/PHASE_1B9_HELD_OUT_REPORT.json"
        )))
        .unwrap();
        assert_eq!(
            actual,
            phase1b9::canonical_phase1b9_report_json(&report).unwrap()
        );
    }

    #[test]
    fn synthetic_case_uses_frozen_policy_and_exact_three_request_adapter() {
        let (root, case) = synthetic_case();
        let selected = select(&case).unwrap();
        assert_eq!(selected.decision.class, ResearchInterventionClass::Prune);
        assert_eq!(
            selected.decision.target_event_id.as_deref(),
            Some("e-duplicate")
        );
        assert_eq!(selected.candidates.exact_duplicate_prune.len(), 1);
        assert_eq!(
            selected.candidates.exact_duplicate_prune[0].evidence_relation_ids,
            vec!["same-state-source-duplicate"]
        );
        let baseline = project_trace(&case, Claim2ProjectionMode::Baseline).unwrap();
        let noop = project_trace(&case, Claim2ProjectionMode::NoOp).unwrap();
        let intervention = project_trace(&case, Claim2ProjectionMode::Intervention).unwrap();
        assert_eq!(baseline, noop);
        assert!(baseline
            .events
            .iter()
            .any(|event| event.event_id == "e-duplicate"));
        assert!(!intervention
            .events
            .iter()
            .any(|event| event.event_id == "e-duplicate"));
        assert!(intervention
            .events
            .iter()
            .any(|event| event.event_type == EventType::Result));

        let skeleton = Claim2ArmState::new(&case, Claim2ProjectionMode::Intervention)
            .preview_request(&case, 3)
            .unwrap();
        assert!(!skeleton.dispatchable);
        assert_eq!(skeleton.metrics.unbound_raw_assistant_slots, vec![1, 2]);
        assert_eq!(
            skeleton.metrics.unbound_environment_receipt_slots,
            vec![1, 2]
        );
        assert_eq!(skeleton.metrics.raw_prior_output_planning_utf8_bytes, 4096);
        assert_eq!(skeleton.metrics.possible_environment_receipt_sizes.len(), 4);
        assert_eq!(
            skeleton.metrics.omitted_attachment_utf8_bytes,
            "Source R-41: stable exporter branch; current state is S-41.\n".len()
        );
        let wire: Value = serde_json::from_slice(
            &Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline)
                .preview_request(&case, 1)
                .unwrap()
                .request_json,
        )
        .unwrap();
        assert_eq!(wire["model"], CLAIM2_MODEL_LABEL);
        assert_eq!(wire["max_tokens"], CLAIM2_MAX_OUTPUT_TOKENS);
        assert_eq!(wire["temperature"], 0);
        assert_eq!(wire["top_p"], 1);
        assert_eq!(wire["seed"], 1);
        assert_eq!(wire["stream"], false);
        for forbidden in [
            "chat_template_kwargs",
            "reasoning_effort",
            "reasoning_format",
            "reasoning_budget_tokens",
        ] {
            assert!(wire.get(forbidden).is_none());
        }
        assert!(!String::from_utf8(skeleton.request_json)
            .unwrap()
            .contains("src-41\",\"revision\":\"R-41\",\"status\":\"verified"));

        let (mut baseline_state, baseline_eval) =
            run_good_arm(&case, Claim2ProjectionMode::Baseline);
        let (mut noop_state, noop_eval) = run_good_arm(&case, Claim2ProjectionMode::NoOp);
        let (mut intervention_state, intervention_eval) =
            run_good_arm(&case, Claim2ProjectionMode::Intervention);
        assert_eq!(baseline_eval.status, Claim2SlotStatus::Pass);
        assert_eq!(noop_eval.status, Claim2SlotStatus::Pass);
        assert_eq!(intervention_eval.status, Claim2SlotStatus::Pass);
        assert_eq!(
            evaluate(&case, &baseline_state).unwrap().status,
            Claim2SlotStatus::Pass
        );
        let comparison = compare_paired_arms(
            &mut baseline_state,
            &mut noop_state,
            &mut intervention_state,
        )
        .unwrap();
        assert_eq!(comparison.status, Claim2SlotStatus::Pass);
        assert!(comparison.baseline_noop_match);
        assert!(comparison.intervention_pre_treatment_match);
        assert_eq!(
            baseline_state
                .slots()
                .iter()
                .map(|slot| slot.raw_assistant_output.as_deref())
                .collect::<Vec<_>>(),
            noop_state
                .slots()
                .iter()
                .map(|slot| slot.raw_assistant_output.as_deref())
                .collect::<Vec<_>>()
        );
        assert_eq!(baseline_state.slots().len(), 3);
        assert_ne!(
            baseline_state.rendered_request_json[2],
            intervention_state.rendered_request_json[2]
        );
        assert!(intervention_eval.required_relations_preserved);
        assert!(Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline)
            .rendered_request_json
            .is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn same_typed_wrong_final_facts_remain_complete_and_reevaluate_stably() {
        let (root, case) = synthetic_case();
        let mut wrong_answer = case.evaluation_key.expected_final_answer.clone();
        wrong_answer["status"] = Value::String("unverified".to_string());
        let (arm, first) = run_arm_with_answer(&case, wrong_answer);
        assert_eq!(first.status, Claim2SlotStatus::Fail);
        assert_eq!(first.model_status, Some(Claim2SlotStatus::Fail));
        assert!(!first.final_answer_passed);
        assert!(first.final_answer_shape_valid);
        assert!(first.structurally_complete);
        assert_eq!(first.required_context_available, 4);
        assert_eq!(first.required_context_total, 4);
        assert_eq!(first.required_context_recall, Some(1.0));
        assert!(first
            .failure_reasons
            .contains(&"final_answer_mismatch".to_string()));
        assert!(!first
            .failure_reasons
            .contains(&"incomplete_arm".to_string()));
        assert_eq!(evaluate(&case, &arm).unwrap(), first);
        assert_eq!(arm.slots()[2].status, Claim2SlotStatus::Fail);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn wrong_final_schema_is_a_model_failure_and_never_structurally_complete() {
        let (root, case) = synthetic_case();
        let mut wrong_shape = case.evaluation_key.expected_final_answer.clone();
        wrong_shape["revision"] = Value::Number(41.into());
        let (arm, evaluation) = run_arm_with_answer(&case, wrong_shape);
        assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
        assert_eq!(evaluation.model_status, Some(Claim2SlotStatus::Fail));
        assert!(!evaluation.final_answer_shape_valid);
        assert!(!evaluation.structurally_complete);
        assert_eq!(arm.slots()[2].status, Claim2SlotStatus::Fail);
        assert_eq!(evaluate(&case, &arm).unwrap(), evaluation);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn required_context_counts_only_the_actual_final_rendered_message_and_receipt_view() {
        let (root, case) = synthetic_case();

        let mut message_missing = complete_two_actions_and_render_final(&case);
        let source_text = case
            .assets
            .values()
            .find(|asset| asset.descriptor.event_id.as_deref() == Some("e-source"))
            .unwrap()
            .text
            .clone();
        let source_message = message_missing.rendered_messages[2]
            .iter_mut()
            .find(|message| message.source_event_ids.iter().any(|id| id == "e-source"))
            .unwrap();
        assert!(source_message.content.contains(&source_text));
        source_message.content = source_message.content.replace(&source_text, "");
        source_message
            .source_event_ids
            .retain(|id| id != "e-source");
        replace_rendered_slot_json(&mut message_missing, 2);
        let missing_message_eval = message_missing
            .record_output(
                &case,
                serde_json::json!({"answer": case.evaluation_key.expected_final_answer})
                    .to_string(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(missing_message_eval.status, Claim2SlotStatus::Fail);
        assert!(missing_message_eval.structurally_complete);
        assert_eq!(missing_message_eval.required_context_available, 3);
        assert_eq!(missing_message_eval.required_context_recall, Some(0.75));
        assert!(!missing_message_eval.required_context_preserved);
        assert!(missing_message_eval
            .failure_reasons
            .contains(&"required_context_missing".to_string()));

        let mut receipt_missing = complete_two_actions_and_render_final(&case);
        let receipt_message = receipt_missing.rendered_messages[2]
            .iter_mut()
            .find(|message| {
                message
                    .source_receipt_event_ids
                    .iter()
                    .any(|id| id == "e-result2")
            })
            .unwrap();
        receipt_message.content.clear();
        receipt_message.source_event_id = None;
        receipt_message.source_receipt_event_ids.clear();
        replace_rendered_slot_json(&mut receipt_missing, 2);
        let missing_receipt_eval = receipt_missing
            .record_output(
                &case,
                serde_json::json!({"answer": case.evaluation_key.expected_final_answer})
                    .to_string(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(missing_receipt_eval.status, Claim2SlotStatus::Fail);
        assert!(missing_receipt_eval.structurally_complete);
        assert_eq!(missing_receipt_eval.required_context_available, 3);
        assert_eq!(missing_receipt_eval.required_context_recall, Some(0.75));
        assert!(!missing_receipt_eval.required_context_preserved);
        assert!(missing_receipt_eval
            .failure_reasons
            .contains(&"required_context_missing".to_string()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn template_tampering_cannot_rewrite_prior_messages_or_drop_required_receipts() {
        let (root, case) = synthetic_case();
        let mut omitted_prior_message = case.manifest.clone();
        omitted_prior_message.request_templates[1].messages[1]
            .parts
            .retain(|part| !matches!(part, Claim2PromptPart::EventBody { event_id } if event_id == "e-source"));
        assert!(validate_templates(&omitted_prior_message, &case.assets).is_err());

        let mut edited_prior_message = case.manifest.clone();
        edited_prior_message.request_templates[1].messages[1].parts[0] =
            Claim2PromptPart::EventBody {
                event_id: "e-duplicate".to_string(),
            };
        assert!(validate_templates(&edited_prior_message, &case.assets).is_err());

        let mut reordered_prior_messages = case.manifest.clone();
        reordered_prior_messages.request_templates[2]
            .messages
            .swap(0, 1);
        assert!(validate_templates(&reordered_prior_messages, &case.assets).is_err());

        let mut missing_receipt = case.manifest.clone();
        missing_receipt.request_templates[2]
            .messages
            .retain(|message| {
                !matches!(
                    message.parts.as_slice(),
                    [Claim2PromptPart::PriorEnvironmentReceipt { request_slot: 2 }]
                )
            });
        assert!(validate_templates(&missing_receipt, &case.assets).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn incomplete_or_divergent_pairs_never_pass_and_keep_task_failure_evidence() {
        let (root, case) = synthetic_case();

        let mut fresh_baseline = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        let mut fresh_noop = Claim2ArmState::new(&case, Claim2ProjectionMode::NoOp);
        let mut fresh_intervention = Claim2ArmState::new(&case, Claim2ProjectionMode::Intervention);
        let fresh = compare_paired_arms(
            &mut fresh_baseline,
            &mut fresh_noop,
            &mut fresh_intervention,
        )
        .unwrap();
        assert_eq!(fresh.status, Claim2SlotStatus::Inconclusive);
        assert!(!fresh.baseline_noop_match);
        assert!(!fresh.intervention_pre_treatment_match);

        let mut partial_baseline = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        let mut partial_noop = Claim2ArmState::new(&case, Claim2ProjectionMode::NoOp);
        let mut partial_intervention =
            Claim2ArmState::new(&case, Claim2ProjectionMode::Intervention);
        for arm in [
            &mut partial_baseline,
            &mut partial_noop,
            &mut partial_intervention,
        ] {
            arm.render_next(&case).unwrap();
            arm.record_output(&case, r#"{"action_id":"inspect_source"}"#.to_string())
                .unwrap();
        }
        let partial = compare_paired_arms(
            &mut partial_baseline,
            &mut partial_noop,
            &mut partial_intervention,
        )
        .unwrap();
        assert_eq!(partial.status, Claim2SlotStatus::Inconclusive);
        assert!(!partial.baseline_noop_match);
        assert!(!partial.intervention_pre_treatment_match);

        let (mut baseline, _) = run_good_arm(&case, Claim2ProjectionMode::Baseline);
        let (mut noop, _) = run_good_arm(&case, Claim2ProjectionMode::NoOp);
        let mut intervention = Claim2ArmState::new(&case, Claim2ProjectionMode::Intervention);
        intervention.render_next(&case).unwrap();
        let task_failure = intervention
            .record_output(&case, r#"{"action_id":"skip_inspection"}"#.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(task_failure.model_status, Some(Claim2SlotStatus::Fail));
        let comparison = compare_paired_arms(&mut baseline, &mut noop, &mut intervention).unwrap();
        assert_eq!(comparison.status, Claim2SlotStatus::Inconclusive);
        let after_pair = evaluate(&case, &intervention).unwrap();
        assert_eq!(after_pair.status, Claim2SlotStatus::Inconclusive);
        assert_eq!(after_pair.model_status, Some(Claim2SlotStatus::Fail));
        assert_eq!(intervention.slots()[0].status, Claim2SlotStatus::Fail);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn arm_machine_fails_fast_and_rejects_repeat_or_fourth_request() {
        let (root, case) = synthetic_case();
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        arm.render_next(&case).unwrap();
        assert!(arm.render_next(&case).is_err());
        let failed = arm
            .record_output(&case, r#"{"action_id":"skip_inspection"}"#.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(failed.status, Claim2SlotStatus::Fail);
        assert_eq!(arm.slots()[0].status, Claim2SlotStatus::Fail);
        assert_eq!(
            arm.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        assert!(arm.render_next(&case).is_err());
        assert!(arm.record_output(&case, "{}".to_string()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pre_treatment_arm_divergence_is_recorded_as_inconclusive() {
        let (root, case) = synthetic_case();
        let (mut baseline, _) = run_good_arm(&case, Claim2ProjectionMode::Baseline);
        let (mut noop, _) = run_good_arm(&case, Claim2ProjectionMode::NoOp);
        let mut intervention = Claim2ArmState::new(&case, Claim2ProjectionMode::Intervention);
        intervention.render_next(&case).unwrap();
        let failed = intervention
            .record_output(&case, r#"{"action_id":"skip_inspection"}"#.to_string())
            .unwrap()
            .unwrap();
        assert_eq!(failed.status, Claim2SlotStatus::Fail);
        let comparison = compare_paired_arms(&mut baseline, &mut noop, &mut intervention).unwrap();
        assert_eq!(comparison.status, Claim2SlotStatus::Inconclusive);
        assert!(comparison.baseline_noop_match);
        assert!(!comparison.intervention_pre_treatment_match);
        assert_eq!(
            intervention.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dependency_cycles_and_unpinned_mutations_fail_closed() {
        let (root, case) = synthetic_case();
        let mut cyclic = case.manifest.clone();
        cyclic.planner_input.relations.push(relation(
            "cycle",
            RelationType::DependsOn,
            "e-task",
            "e-action1",
        ));
        let assets = case.assets.clone();
        assert!(validate_manifest(&cyclic, &assets, &case.evaluation_key).is_err());
        let mut ambiguous_alias = case.manifest.clone();
        ambiguous_alias.planner_input.events[1].context_block_id = Some("e-task".to_string());
        assert!(validate_manifest(&ambiguous_alias, &assets, &case.evaluation_key).is_err());
        let mut missing_producer = case.manifest.clone();
        missing_producer
            .planner_input
            .relations
            .retain(|relation| relation.relation_id != "produce1");
        assert!(validate_manifest(&missing_producer, &assets, &case.evaluation_key).is_err());
        let asset_path = root.join("bodies/source.txt");
        fs::write(&asset_path, "changed bytes\n").unwrap();
        assert!(load_case(&root.join("case.json")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn different_state_and_protected_consumer_controls_stay_natural_noop() {
        let (root, positive) = synthetic_case();

        let mut cp05 = positive.manifest.clone();
        cp05.case_id = "CP05".to_string();
        cp05.kind = Claim2CaseKind::Control;
        cp05.planner_input
            .events
            .iter_mut()
            .find(|event| event.event_id == "e-duplicate")
            .unwrap()
            .world_state_revision = Some("S-42".to_string());
        cp05.planner_input
            .relations
            .retain(|relation| relation.relation_type != RelationType::SameStateRevision);
        fs::write(
            root.join("case.json"),
            serde_json::to_vec_pretty(&cp05).unwrap(),
        )
        .unwrap();
        let cp05 = load_case(&root.join("case.json")).unwrap();
        let selected = select(&cp05).unwrap();
        assert_eq!(
            selected.decision.class,
            ResearchInterventionClass::DoNothing
        );
        assert!(selected.candidates.exact_duplicate_prune.is_empty());
        assert_eq!(
            project_trace(&cp05, Claim2ProjectionMode::Baseline).unwrap(),
            project_trace(&cp05, Claim2ProjectionMode::Intervention).unwrap()
        );

        let mut cp06 = positive.manifest.clone();
        cp06.case_id = "CP06".to_string();
        cp06.kind = Claim2CaseKind::Control;
        let audit_receipt =
            "Audit receipt: duplicate occurrence remains linked to the audit checkpoint.\n";
        let mut new_assets = cp06.assets.clone();
        write_asset(
            &root,
            &mut new_assets,
            "audit_receipt",
            "bodies/audit_receipt.txt",
            Claim2AssetKind::EnvironmentReceipt,
            Some("e-audit-result"),
            Some("audit-rev-1"),
            audit_receipt,
        );
        cp06.assets = new_assets;
        cp06.planner_input.events.push(event(
            "e-audit-action",
            11,
            EventType::Action,
            ActorRole::Agent,
            vec![],
            Some(ActionIdentity {
                action_id: "audit_duplicate".to_string(),
                tool_name: "audit_duplicate".to_string(),
                argument_hash: Some(sha256_hex(b"audit_duplicate")),
            }),
            None,
            None,
            None,
            Some("S-41"),
        ));
        cp06.planner_input.events.push(event(
            "e-audit-result",
            12,
            EventType::Result,
            ActorRole::Tool,
            vec![],
            None,
            Some(crate::model::ResultIdentity {
                result_id: "r-audit-result".to_string(),
                originating_action_id: "audit_duplicate".to_string(),
                observation_hash: Some(sha256_hex(audit_receipt.as_bytes())),
                status: Some(crate::model::ResultStatus::Success),
            }),
            None,
            None,
            Some("S-41"),
        ));
        cp06.planner_input.relations.push(relation(
            "produce-audit",
            RelationType::Produces,
            "audit_duplicate",
            "r-audit-result",
        ));
        cp06.planner_input.relations.push(relation(
            "duplicate-protected-before-audit",
            RelationType::ProtocolPrecedes,
            "e-duplicate",
            "audit_duplicate",
        ));
        fs::write(
            root.join("case.json"),
            serde_json::to_vec_pretty(&cp06).unwrap(),
        )
        .unwrap();
        let cp06 = load_case(&root.join("case.json")).unwrap();
        let selected = select(&cp06).unwrap();
        assert_eq!(
            selected.decision.class,
            ResearchInterventionClass::DoNothing
        );
        assert!(selected.candidates.exact_duplicate_prune.is_empty());
        assert_eq!(
            project_trace(&cp06, Claim2ProjectionMode::Baseline).unwrap(),
            project_trace(&cp06, Claim2ProjectionMode::Intervention).unwrap()
        );
        let mut cohort = vec![positive.clone()];
        for case_id in ["CP02", "CP03", "CP04"] {
            let mut positive_peer = positive.clone();
            positive_peer.manifest.case_id = case_id.to_string();
            positive_peer.fingerprint = canonical_hash(&positive_peer.manifest).unwrap();
            cohort.push(positive_peer);
        }
        cohort.push(cp05);
        cohort.push(cp06);
        let slots = precheck_cohort(&cohort).unwrap();
        assert_eq!(slots.len(), 54);
        assert!(slots.iter().all(|slot| !slot.dispatchable));
        assert!(slots.iter().all(|slot| {
            slot.metrics.token_count_status == Claim2TokenCountStatus::ExactTokenizationRequired
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn token_guards_and_materiality_require_token_evidence() {
        assert_eq!(
            token_guard(None),
            Claim2TokenGuardResult::ExactTokenizationRequired
        );
        assert_eq!(token_guard(Some(5500)), Claim2TokenGuardResult::WithinLimit);
        assert_eq!(token_guard(Some(6001)), Claim2TokenGuardResult::OverLimit);
        assert_eq!(
            token_guard(Some(u64::MAX)),
            Claim2TokenGuardResult::OverLimit
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: None,
                b3_high: None,
                dsum_low: None,
                sum_b_high: None
            }),
            Claim2MaterialityResult::ExactTokenizationRequired
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4000),
                dsum_low: Some(800),
                sum_b_high: Some(10000)
            }),
            Claim2MaterialityResult::Qualifies
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(799),
                b3_high: Some(4000),
                dsum_low: Some(1000),
                sum_b_high: Some(10000),
            }),
            Claim2MaterialityResult::DoesNotQualify
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4000),
                dsum_low: Some(799),
                sum_b_high: Some(10000),
            }),
            Claim2MaterialityResult::DoesNotQualify
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4001),
                dsum_low: Some(800),
                sum_b_high: Some(10000),
            }),
            Claim2MaterialityResult::DoesNotQualify
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4000),
                dsum_low: Some(801),
                sum_b_high: Some(10000),
            }),
            Claim2MaterialityResult::Qualifies
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(u64::MAX),
                b3_high: Some(u64::MAX),
                dsum_low: Some(u64::MAX),
                sum_b_high: Some(u64::MAX),
            }),
            Claim2MaterialityResult::Qualifies
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4000),
                dsum_low: Some(10_001),
                sum_b_high: Some(10_000),
            }),
            Claim2MaterialityResult::DoesNotQualify
        );
        assert_eq!(
            materiality_result(&Claim2TokenProofInputs {
                d3_low: Some(800),
                b3_high: Some(4000),
                dsum_low: None,
                sum_b_high: Some(10_000),
            }),
            Claim2MaterialityResult::ExactTokenizationRequired
        );
    }
}
