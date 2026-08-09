//! Provider-neutral Phase 1B.7 data types.
//!
//! The types in this crate deliberately do not extend `RequestTrace`. A
//! controlled envelope contains a structural planner input and a separate
//! evaluation sidecar; callers must explicitly project the former before
//! invoking the frozen Phase 1B planner.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_ID: &str = "prefixity.controlled-benchmark";
pub const SCHEMA_VERSION: u32 = 1;
pub const RELATION_SEMANTICS_VERSION: &str = "controlled-benchmark-relations-v1";
pub const ORACLE_VERSION: &str = "prefixity-scripted-oracle-v1";
pub const BENCHMARK_ID: &str = "prefixity-controlled-seed-v1";
pub const TASK_REVISION: &str = "self-authored-task-v1";
pub const ENVIRONMENT_REVISION: &str = "prefixity-scripted-world-v1";

pub const MAX_ID_BYTES: usize = 128;
pub const MAX_TEXT_BYTES: usize = 512;
pub const MAX_EVENTS: usize = 512;
pub const MAX_RELATIONS: usize = 1024;
pub const MAX_PROVENANCE: usize = 512;

/// Evidence classification preserved by the approved Phase 1B.6 design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceClass {
    CapturedExplicit,
    DerivedStructural,
    EvaluationOnly,
    InferredUnsafe,
    Absent,
}

/// Provenance source kind. Public design references are metadata only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    SelfAuthored,
    PinnedPublicMetadata,
    PublicDesignReference,
}

/// Bounded provenance attached to structural evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceProvenance {
    pub source_kind: SourceKind,
    pub classification: EvidenceClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_locator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Task/environment identity shared by every member of a pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioIdentity {
    pub scenario_id: String,
    pub scenario_version: String,
    pub task_revision: String,
    pub environment_revision: String,
    pub initial_state_id: String,
    pub fixed_seed: u32,
    pub provenance: Vec<SourceProvenance>,
}

/// Baseline, intervention variant, or unchanged control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantRole {
    Baseline,
    Variant,
    Control,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Message,
    Action,
    Result,
    Observation,
    StateSnapshot,
    Assertion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorRole {
    System,
    User,
    Agent,
    Tool,
    Environment,
    Evaluator,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionIdentity {
    pub action_id: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argument_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultIdentity {
    pub result_id: String,
    pub originating_action_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<ResultStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Success,
    Failure,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logical_tick: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp_origin: Option<TimestampOrigin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimestampOrigin {
    SourceExplicit,
    DerivedStructural,
}

/// One planner-visible structural event. Raw content is intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub event_id: String,
    pub sequence_index: u32,
    pub event_type: EventType,
    pub actor_role: ActorRole,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_event_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ResultIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_block_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_state_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<OrderMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    pub provenance: Vec<SourceProvenance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Produces,
    References,
    DependsOn,
    Supersedes,
    Invalidates,
    ProtocolPrecedes,
    SameStateRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub relation_id: String,
    pub relation_type: RelationType,
    pub from_id: String,
    pub to_id: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics_version: Option<String>,
    pub provenance: Vec<SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerInput {
    pub events: Vec<Event>,
    pub relations: Vec<Relation>,
    pub provenance: Vec<SourceProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceEnvelope {
    pub trace_id: String,
    pub variant_role: VariantRole,
    pub baseline_trace_id: String,
    pub planner_input: PlannerInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionClass {
    Remove,
    Defer,
    Relocate,
    Compress,
    NoChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannerVisibility {
    PlannerVisible,
    EvaluationOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityRiskCategory {
    None,
    Low,
    LowIfBoundaryPreserved,
    LowIfOrderConstraintsHold,
    High,
    Unknown,
}

/// Evaluation-only pairing and expected-risk metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterventionManifest {
    pub manifest_id: String,
    pub baseline_trace_id: String,
    pub variant_trace_id: String,
    pub target_event_ids: Vec<String>,
    pub intervention_class: InterventionClass,
    pub exact_transformation: String,
    pub reason: String,
    pub planner_visibility: PlannerVisibility,
    pub expected_structural_effect: String,
    pub expected_quality_risk_category: QualityRiskCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationSidecar {
    pub intervention_manifest_ref: String,
    pub intervention_manifest: InterventionManifest,
    pub quality_evaluation_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle_result: Option<OracleResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_content_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OracleResult {
    Pass,
    Fail,
    InvalidBaseline,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlledEnvelope {
    pub schema_id: String,
    pub schema_version: u32,
    pub benchmark_id: String,
    pub scenario: ScenarioIdentity,
    pub trace: TraceEnvelope,
    pub evaluation_sidecar: EvaluationSidecar,
}

/// A validated baseline/variant or baseline/control pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlledCase {
    pub scenario_id: String,
    pub baseline: ControlledEnvelope,
    pub intervention: ControlledEnvelope,
    pub manifest: InterventionManifest,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlannerEvidence {
    pub benchmark_id: String,
    pub scenario: ScenarioIdentity,
    pub trace: TraceEnvelope,
    /// Existing production planner input, projected without the sidecar.
    pub request_trace: prefixity_core::model::RequestTrace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvaluationRecord {
    pub scenario_id: String,
    pub manifest_id: String,
    pub baseline_trace_id: String,
    pub intervention_trace_id: String,
    pub result: OracleResult,
    pub baseline_completed: bool,
    pub intervention_completed: bool,
    pub baseline_final_state_hash: Option<String>,
    pub intervention_final_state_hash: Option<String>,
    pub collateral_state_keys: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlannerRun {
    pub scenario_id: String,
    pub trace_id: String,
    pub classes: Vec<String>,
    pub plan_json_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AggregateCounts {
    pub pass: usize,
    pub fail: usize,
    pub invalid_baseline: usize,
    pub inconclusive: usize,
}

impl AggregateCounts {
    pub fn record(&mut self, result: OracleResult) {
        match result {
            OracleResult::Pass => self.pass += 1,
            OracleResult::Fail => self.fail += 1,
            OracleResult::InvalidBaseline => self.invalid_baseline += 1,
            OracleResult::Inconclusive => self.inconclusive += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BenchmarkReport {
    pub artifact_id: String,
    pub schema_id: String,
    pub schema_version: u32,
    pub oracle_version: String,
    pub scenario_count: usize,
    pub baseline_count: usize,
    pub variant_count: usize,
    pub control_count: usize,
    pub manifest_hashes: BTreeMap<String, String>,
    pub aggregate_hash: String,
    pub evaluations: Vec<EvaluationRecord>,
    pub aggregate_counts: AggregateCounts,
    pub planner_runs: Vec<PlannerRun>,
}
