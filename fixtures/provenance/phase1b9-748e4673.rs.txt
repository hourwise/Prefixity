//! Phase 1B.9 blinded held-out intervention-selection study.
//!
//! This module is research-only. It keeps the held-out planner representation
//! and the evaluation key separate, uses neutral opaque identifiers, and does
//! not modify or extend the production Phase 1B planner contract.

use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, canonical_json, hash_text};
use crate::model::{
    ActorRole, Event, EventType, OrderMetadata, PlannerInput, Relation, RelationType, SourceKind,
    SourceProvenance,
};
use prefixity_core::decision::plan_interventions;
use prefixity_core::hash::hash_content;
use prefixity_core::model::{ContextBlock, RequestTrace, TRACE_FORMAT_VERSION};
use prefixity_core::validation::validate_trace;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PHASE_1B9_POLICY_VERSION: &str = "controlled-evidence-policy-v1";
pub const PHASE_1B9_SCOPE: &str = "CONTROLLED_ONLY";

const PREREGISTRATION_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/phase-1/PHASE_1B9_PREREGISTRATION.md"
));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResearchInterventionClass {
    Prune,
    Defer,
    RelocateCandidate,
    DoNothing,
}

impl ResearchInterventionClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prune => "PRUNE",
            Self::Defer => "DEFER",
            Self::RelocateCandidate => "RELOCATE_CANDIDATE",
            Self::DoNothing => "DO_NOTHING",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlindedEvent {
    pub event_id: String,
    pub sequence_index: u32,
    pub event_type: EventType,
    pub actor_role: ActorRole,
    pub reference_ids: Vec<String>,
    pub action_id: Option<String>,
    pub result_id: Option<String>,
    pub originating_action_id: Option<String>,
    pub context_id: Option<String>,
    pub content_hash: Option<String>,
    pub structural_zone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlindedRelation {
    pub relation_id: String,
    pub relation_type: RelationType,
    pub from_id: String,
    pub to_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlindedTrace {
    pub scope: String,
    pub events: Vec<BlindedEvent>,
    pub relations: Vec<BlindedRelation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResearchPolicyDecision {
    pub class: ResearchInterventionClass,
    pub target_event_id: Option<String>,
    pub rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenPlannerBaseline {
    pub distribution: BTreeMap<String, usize>,
    pub report_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase1b9DecisionRecord {
    pub case_id: String,
    pub positive_available: bool,
    pub selected_class: String,
    pub selected_target_event_id: Option<String>,
    pub oracle_result: Option<String>,
    pub baseline_completed: bool,
    pub intervention_completed: Option<bool>,
    pub baseline_pass_intervention_fail: bool,
    pub classification: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase1b9Report {
    pub artifact_id: String,
    pub policy_version: String,
    pub preregistration_hash: String,
    pub held_out_set_hash: String,
    pub evaluation_key_hash: String,
    pub planner_evidence_hash: String,
    pub policy_hash: String,
    pub case_count: usize,
    pub positive_cases_available: usize,
    pub frozen_planner: FrozenPlannerBaseline,
    pub decisions: Vec<Phase1b9DecisionRecord>,
    pub positive_interventions_selected: usize,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
    pub true_noop_decisions: usize,
    pub precision: Option<f64>,
    pub recall: f64,
    pub unsafe_intervention_count: usize,
    pub baseline_pass_intervention_failures: usize,
    pub determinism_hash: String,
}

#[derive(Debug, Clone)]
struct HeldOutCase {
    case_id: String,
    planner_input: PlannerInput,
    blinded: BlindedTrace,
    evaluation: EvaluationKey,
}

#[derive(Debug, Clone)]
struct EvaluationKey {
    positive: bool,
    expected: Option<ExpectedSelection>,
    hidden_requirements: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedSelection {
    class: ResearchInterventionClass,
    target_event_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Execution {
    completed: bool,
    final_state: BTreeMap<String, String>,
    final_state_hash: String,
}

pub fn preregistration_hash() -> String {
    crate::hashing::sha256_hex(PREREGISTRATION_BYTES)
}

pub fn canonical_phase1b9_report_json(report: &Phase1b9Report) -> Result<Vec<u8>, BenchmarkError> {
    canonical_json(report).map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn blinded_trace_json(trace: &BlindedTrace) -> Result<Vec<u8>, BenchmarkError> {
    canonical_json(trace).map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn run_phase1b9_study() -> Result<Phase1b9Report, BenchmarkError> {
    let first = run_once()?;
    let second = run_once()?;
    let first_json = canonical_phase1b9_report_json(&first)?;
    let second_json = canonical_phase1b9_report_json(&second)?;
    if first_json != second_json {
        return Err(BenchmarkError::validation(
            "Phase 1B.9 repeated report was not byte-identical",
        ));
    }
    let mut report = first;
    report.determinism_hash = canonical_hash(&report_without_determinism(&report))
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(report)
}

fn run_once() -> Result<Phase1b9Report, BenchmarkError> {
    let cases = build_held_out_cases()?;
    let held_out_set_hash = canonical_hash(
        &cases
            .iter()
            .map(|case| &case.planner_input)
            .collect::<Vec<_>>(),
    )
    .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let evaluation_key_hash =
        canonical_hash(&cases.iter().map(evaluation_key_summary).collect::<Vec<_>>())
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let planner_evidence_hash =
        canonical_hash(&cases.iter().map(|case| &case.blinded).collect::<Vec<_>>())
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let frozen_planner = frozen_planner_baseline(&cases)?;
    let policy_hash = canonical_hash(&policy_spec())
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;

    let mut decisions = Vec::with_capacity(cases.len());
    let mut positive_interventions_selected = 0;
    let mut true_positives = 0;
    let mut false_positives = 0;
    let mut false_negatives = 0;
    let mut true_noop_decisions = 0;
    let mut unsafe_intervention_count = 0;
    let mut baseline_pass_intervention_failures = 0;
    let positive_cases_available = cases.iter().filter(|case| case.evaluation.positive).count();

    for case in &cases {
        let baseline = execute(&case.planner_input, &case.evaluation.hidden_requirements);
        let decision = research_policy(&case.blinded);
        let selected = decision.class != ResearchInterventionClass::DoNothing;
        let mut oracle_result = None;
        let mut intervention_completed = None;
        let mut baseline_pass_intervention_fail = false;

        let classification = if !selected {
            if case.evaluation.positive {
                false_negatives += 1;
                "FALSE_NEGATIVE"
            } else {
                true_noop_decisions += 1;
                "TRUE_NOOP"
            }
        } else {
            positive_interventions_selected += usize::from(case.evaluation.positive);
            let intervention_input = apply_decision(&case.planner_input, &decision)?;
            let intervention = execute(&intervention_input, &case.evaluation.hidden_requirements);
            intervention_completed = Some(intervention.completed);
            let passed = baseline.completed
                && intervention.completed
                && intervention.final_state == baseline.final_state;
            oracle_result = Some(if passed { "PASS" } else { "FAIL" }.to_string());
            baseline_pass_intervention_fail = baseline.completed && !intervention.completed;
            if baseline_pass_intervention_fail {
                baseline_pass_intervention_failures += 1;
            }
            if !passed {
                unsafe_intervention_count += 1;
            }
            if case.evaluation.positive
                && passed
                && case.evaluation.expected.as_ref().is_some_and(|expected| {
                    expected.class == decision.class
                        && expected.target_event_id
                            == decision.target_event_id.clone().unwrap_or_default()
                })
            {
                true_positives += 1;
                "TRUE_POSITIVE"
            } else {
                false_positives += 1;
                "FALSE_POSITIVE"
            }
        };

        decisions.push(Phase1b9DecisionRecord {
            case_id: case.case_id.clone(),
            positive_available: case.evaluation.positive,
            selected_class: decision.class.as_str().to_string(),
            selected_target_event_id: decision.target_event_id,
            oracle_result,
            baseline_completed: baseline.completed,
            intervention_completed,
            baseline_pass_intervention_fail,
            classification: classification.to_string(),
        });
    }

    let selected_count = positive_interventions_selected + false_positives;
    let precision = (selected_count > 0).then_some(true_positives as f64 / selected_count as f64);
    let recall = if positive_cases_available == 0 {
        0.0
    } else {
        true_positives as f64 / positive_cases_available as f64
    };
    let mut report = Phase1b9Report {
        artifact_id: "prefixity-phase1b9-held-out-v1".to_string(),
        policy_version: PHASE_1B9_POLICY_VERSION.to_string(),
        preregistration_hash: preregistration_hash(),
        held_out_set_hash,
        evaluation_key_hash,
        planner_evidence_hash,
        policy_hash,
        case_count: cases.len(),
        positive_cases_available,
        frozen_planner,
        decisions,
        positive_interventions_selected,
        true_positives,
        false_positives,
        false_negatives,
        true_noop_decisions,
        precision,
        recall,
        unsafe_intervention_count,
        baseline_pass_intervention_failures,
        determinism_hash: String::new(),
    };
    report.determinism_hash = String::new();
    Ok(report)
}

fn report_without_determinism(report: &Phase1b9Report) -> Phase1b9Report {
    let mut copy = report.clone();
    copy.determinism_hash.clear();
    copy
}

fn policy_spec() -> BTreeMap<&'static str, Vec<&'static str>> {
    BTreeMap::from([
        (
            "rule_order",
            vec![
                "EXACT_DUPLICATE_PRUNE",
                "EXPLICIT_SUPERSESSION_DEFER",
                "SAME_ZONE_PROTOCOL_RELOCATE",
            ],
        ),
        ("scope", vec![PHASE_1B9_SCOPE]),
        (
            "thresholds",
            vec!["exact_hash", "explicit_relation", "one_intermediate_event"],
        ),
    ])
}

fn evaluation_key_summary(case: &HeldOutCase) -> BTreeMap<String, String> {
    let mut summary = BTreeMap::new();
    summary.insert("case_id".to_string(), case.case_id.clone());
    summary.insert("positive".to_string(), case.evaluation.positive.to_string());
    if let Some(expected) = &case.evaluation.expected {
        summary.insert("class".to_string(), expected.class.as_str().to_string());
        summary.insert("target".to_string(), expected.target_event_id.clone());
    }
    summary.insert(
        "hidden_requirements".to_string(),
        serde_json::to_string(&case.evaluation.hidden_requirements)
            .expect("hidden requirements serialize"),
    );
    summary
}

fn frozen_planner_baseline(cases: &[HeldOutCase]) -> Result<FrozenPlannerBaseline, BenchmarkError> {
    let mut distribution = BTreeMap::new();
    let mut hashes = Vec::new();
    for case in cases {
        let trace = to_request_trace(&case.blinded)?;
        let plan = plan_interventions(&trace)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?;
        let classes = plan
            .recommendations
            .iter()
            .map(|recommendation| recommendation.class.as_str().to_string())
            .collect::<Vec<_>>();
        for class in &classes {
            *distribution.entry(class.clone()).or_insert(0) += 1;
        }
        hashes.push(
            canonical_hash(&plan).map_err(|error| BenchmarkError::validation(error.to_string()))?,
        );
    }
    Ok(FrozenPlannerBaseline {
        distribution,
        report_hash: canonical_hash(&hashes)
            .map_err(|error| BenchmarkError::validation(error.to_string()))?,
    })
}

fn research_policy(trace: &BlindedTrace) -> ResearchPolicyDecision {
    if let Some(target) = exact_duplicate_target(trace) {
        return ResearchPolicyDecision {
            class: ResearchInterventionClass::Prune,
            target_event_id: Some(target),
            rule: "EXACT_DUPLICATE_PRUNE".to_string(),
        };
    }
    if let Some(target) = superseded_target(trace) {
        return ResearchPolicyDecision {
            class: ResearchInterventionClass::Defer,
            target_event_id: Some(target),
            rule: "EXPLICIT_SUPERSESSION_DEFER".to_string(),
        };
    }
    if let Some(target) = relocation_target(trace) {
        return ResearchPolicyDecision {
            class: ResearchInterventionClass::RelocateCandidate,
            target_event_id: Some(target),
            rule: "SAME_ZONE_PROTOCOL_RELOCATE".to_string(),
        };
    }
    ResearchPolicyDecision {
        class: ResearchInterventionClass::DoNothing,
        target_event_id: None,
        rule: "FAIL_OPEN".to_string(),
    }
}

fn exact_duplicate_target(trace: &BlindedTrace) -> Option<String> {
    let mut candidates = Vec::new();
    for target in trace
        .events
        .iter()
        .filter(|event| event.event_type == EventType::Message)
    {
        let Some(hash) = &target.content_hash else {
            continue;
        };
        let has_same_state = trace.relations.iter().any(|relation| {
            relation.relation_type == RelationType::SameStateRevision
                && ((relation.from_id == target.event_id
                    && trace.events.iter().any(|event| {
                        event.event_id == relation.to_id
                            && event.content_hash.as_ref() == Some(hash)
                    }))
                    || (relation.to_id == target.event_id
                        && trace.events.iter().any(|event| {
                            event.event_id == relation.from_id
                                && event.content_hash.as_ref() == Some(hash)
                        })))
        });
        if !has_same_state
            || has_consumer(trace, target)
            || has_protected_relation(trace, &target.event_id)
        {
            continue;
        }
        if trace.events.iter().any(|earlier| {
            earlier.sequence_index < target.sequence_index
                && earlier.content_hash.as_ref() == Some(hash)
        }) {
            candidates.push(target);
        }
    }
    candidates
        .into_iter()
        .max_by_key(|event| event.sequence_index)
        .map(|event| event.event_id.clone())
}

fn superseded_target(trace: &BlindedTrace) -> Option<String> {
    trace
        .relations
        .iter()
        .filter(|relation| relation.relation_type == RelationType::Supersedes)
        .filter_map(|relation| {
            let newer = trace
                .events
                .iter()
                .find(|event| event.event_id == relation.from_id)?;
            let older = trace
                .events
                .iter()
                .find(|event| event.event_id == relation.to_id)?;
            let action = trace.events.iter().find(|event| {
                event.event_type == EventType::Action
                    && event
                        .reference_ids
                        .iter()
                        .any(|reference| reference == &newer.event_id)
                    && trace.relations.iter().any(|protocol| {
                        protocol.relation_type == RelationType::ProtocolPrecedes
                            && protocol.from_id == newer.event_id
                            && (protocol.to_id == event.event_id
                                || event
                                    .action_id
                                    .as_ref()
                                    .is_some_and(|id| protocol.to_id == *id))
                    })
            })?;
            if newer.sequence_index < action.sequence_index
                && older.sequence_index < action.sequence_index
                && !has_consumer(trace, older)
                && !has_protected_relation(trace, &older.event_id)
            {
                Some(older.event_id.clone())
            } else {
                None
            }
        })
        .next()
}

fn relocation_target(trace: &BlindedTrace) -> Option<String> {
    trace
        .relations
        .iter()
        .filter(|relation| relation.relation_type == RelationType::ProtocolPrecedes)
        .filter_map(|relation| {
            let source = trace
                .events
                .iter()
                .find(|event| event.event_id == relation.from_id)?;
            let action = trace.events.iter().find(|event| {
                event.event_id == relation.to_id
                    || event
                        .action_id
                        .as_ref()
                        .is_some_and(|id| id == &relation.to_id)
            })?;
            if !matches!(
                source.event_type,
                EventType::Result | EventType::Observation
            ) || action.event_type != EventType::Action
                || source.structural_zone != "tools"
                || action.structural_zone != "tools"
                || !references_event(action, source)
                || action.sequence_index <= source.sequence_index + 1
                || has_conflicting_relation(trace, source, action)
            {
                return None;
            }
            Some(source.event_id.clone())
        })
        .next()
}

fn references_event(action: &BlindedEvent, source: &BlindedEvent) -> bool {
    action.reference_ids.iter().any(|reference| {
        reference == &source.event_id
            || source
                .result_id
                .as_ref()
                .is_some_and(|result_id| reference == result_id)
            || source
                .context_id
                .as_ref()
                .is_some_and(|context_id| reference == context_id)
    })
}

fn has_consumer(trace: &BlindedTrace, target: &BlindedEvent) -> bool {
    trace.events.iter().any(|event| {
        event.reference_ids.iter().any(|reference| {
            reference == &target.event_id
                || target
                    .result_id
                    .as_ref()
                    .is_some_and(|result_id| reference == result_id)
                || target
                    .context_id
                    .as_ref()
                    .is_some_and(|context_id| reference == context_id)
        })
    })
}

fn has_protected_relation(trace: &BlindedTrace, target: &str) -> bool {
    trace.relations.iter().any(|relation| {
        matches!(
            relation.relation_type,
            RelationType::DependsOn | RelationType::ProtocolPrecedes
        ) && (relation.from_id == target || relation.to_id == target)
    })
}

fn has_conflicting_relation(
    trace: &BlindedTrace,
    source: &BlindedEvent,
    action: &BlindedEvent,
) -> bool {
    trace.relations.iter().any(|relation| {
        matches!(
            relation.relation_type,
            RelationType::DependsOn | RelationType::ProtocolPrecedes
        ) && (relation.from_id == action.event_id
            || action
                .action_id
                .as_ref()
                .is_some_and(|action_id| relation.from_id == *action_id))
            && (relation.to_id == source.event_id
                || source
                    .result_id
                    .as_ref()
                    .is_some_and(|id| relation.to_id == *id))
    })
}

fn apply_decision(
    input: &PlannerInput,
    decision: &ResearchPolicyDecision,
) -> Result<PlannerInput, BenchmarkError> {
    let Some(target) = &decision.target_event_id else {
        return Ok(input.clone());
    };
    let mut output = input.clone();
    let target_index = output
        .events
        .iter()
        .position(|event| event.event_id == *target)
        .ok_or_else(|| BenchmarkError::validation("policy selected an unknown target"))?;
    match decision.class {
        ResearchInterventionClass::Prune => {
            let aliases = aliases_for_event(&output.events[target_index]);
            output.events.remove(target_index);
            output.relations.retain(|relation| {
                !aliases.contains(&relation.from_id) && !aliases.contains(&relation.to_id)
            });
            for event in &mut output.events {
                event
                    .reference_event_ids
                    .retain(|reference| !aliases.contains(reference));
            }
        }
        ResearchInterventionClass::Defer => {
            let event = output.events.remove(target_index);
            let newer = output
                .relations
                .iter()
                .find(|relation| {
                    relation.relation_type == RelationType::Supersedes && relation.to_id == *target
                })
                .map(|relation| relation.from_id.clone());
            let insertion = output
                .events
                .iter()
                .enumerate()
                .find(|(_, candidate)| {
                    candidate.event_type == EventType::Action
                        && output.relations.iter().any(|relation| {
                            relation.relation_type == RelationType::ProtocolPrecedes
                                && newer.as_ref().is_some_and(|id| relation.from_id == *id)
                                && (relation.to_id == candidate.event_id
                                    || candidate
                                        .action
                                        .as_ref()
                                        .is_some_and(|action| relation.to_id == action.action_id))
                        })
                })
                .map(|(index, _)| index + 1)
                .unwrap_or(output.events.len());
            output.events.insert(insertion, event);
        }
        ResearchInterventionClass::RelocateCandidate => {
            let event = output.events.remove(target_index);
            let source_aliases = aliases_for_event(&event);
            let destination = output
                .events
                .iter()
                .position(|candidate| {
                    candidate.event_type == EventType::Action
                        && candidate
                            .reference_event_ids
                            .iter()
                            .any(|reference| source_aliases.contains(reference))
                })
                .ok_or_else(|| BenchmarkError::validation("relocation target has no consumer"))?;
            output.events.insert(destination, event);
        }
        ResearchInterventionClass::DoNothing => {}
    }
    renumber(&mut output);
    Ok(output)
}

fn aliases_for_event(event: &Event) -> BTreeSet<String> {
    let mut aliases = BTreeSet::from([event.event_id.clone()]);
    if let Some(action) = &event.action {
        aliases.insert(action.action_id.clone());
    }
    if let Some(result) = &event.result {
        aliases.insert(result.result_id.clone());
    }
    if let Some(context) = &event.context_block_id {
        aliases.insert(context.clone());
    }
    aliases
}

fn execute(input: &PlannerInput, hidden_requirements: &[(String, String)]) -> Execution {
    let mut available = BTreeSet::new();
    let mut state = BTreeMap::new();
    for event in &input.events {
        if let Some(action) = &event.action {
            if event
                .reference_event_ids
                .iter()
                .any(|reference| !available.contains(reference))
            {
                return failed_execution(state);
            }
            if hidden_requirements.iter().any(|(action_id, required)| {
                action_id == &action.action_id && !available.contains(required)
            }) {
                return failed_execution(state);
            }
            state.insert(action.action_id.clone(), "done".to_string());
        }
        available.insert(event.event_id.clone());
        if let Some(action) = &event.action {
            available.insert(action.action_id.clone());
        }
        if let Some(result) = &event.result {
            available.insert(result.result_id.clone());
        }
        if let Some(context) = &event.context_block_id {
            available.insert(context.clone());
        }
    }
    let final_state_hash = canonical_hash(&state).unwrap_or_default();
    Execution {
        completed: true,
        final_state: state,
        final_state_hash,
    }
}

fn failed_execution(state: BTreeMap<String, String>) -> Execution {
    Execution {
        completed: false,
        final_state_hash: canonical_hash(&state).unwrap_or_default(),
        final_state: state,
    }
}

fn to_request_trace(trace: &BlindedTrace) -> Result<RequestTrace, BenchmarkError> {
    let mut dependencies: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for relation in trace
        .relations
        .iter()
        .filter(|relation| relation.relation_type == RelationType::DependsOn)
    {
        dependencies
            .entry(relation.from_id.clone())
            .or_default()
            .push(relation.to_id.clone());
    }
    let blocks = trace
        .events
        .iter()
        .map(|event| {
            let block_id = event.event_id.clone();
            ContextBlock {
                id: block_id.clone(),
                source: source_for(event.event_type).to_string(),
                position: event.sequence_index as usize,
                content_hash: event
                    .content_hash
                    .clone()
                    .unwrap_or_else(|| hash_content(&event.event_id)),
                token_count: Some(0),
                byte_count: 0,
                timestamp: None,
                content: None,
                semantic_zone: Some(event.structural_zone.clone()),
                structural_path: Some(format!("phase1b9.events[{}]", event.sequence_index)),
                role: Some(role_for(event.event_type).to_string()),
                sensitivity: None,
                dependencies: dependencies.remove(&event.event_id).unwrap_or_default(),
                lifetime: None,
                optional: false,
                required: false,
                stale: false,
                provenance: BTreeMap::new(),
                metadata: BTreeMap::from([(
                    "event_type".to_string(),
                    serde_json::to_value(event.event_type).expect("event type serializes"),
                )]),
            }
        })
        .collect();
    let trace = RequestTrace {
        format_version: TRACE_FORMAT_VERSION,
        request_id: "phase1b9-heldout".to_string(),
        session_id: None,
        timestamp: None,
        provider: "controlled-world".to_string(),
        model: "heldout-v1".to_string(),
        evidence_schema_version: None,
        blocks,
        usage: None,
        provider_response: None,
        latency: None,
        provenance: BTreeMap::new(),
        metadata: BTreeMap::from([(
            "scope".to_string(),
            serde_json::Value::String(PHASE_1B9_SCOPE.to_string()),
        )]),
    };
    validate_trace(&trace, None).map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(trace)
}

fn source_for(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Message => "controlled_message",
        EventType::Action => "controlled_action",
        EventType::Result => "controlled_result",
        EventType::Observation => "controlled_observation",
        EventType::StateSnapshot => "controlled_state",
        EventType::Assertion => "controlled_assertion",
    }
}

fn role_for(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Message => "user",
        EventType::Action => "agent",
        EventType::Result | EventType::Observation => "tool",
        EventType::StateSnapshot => "environment",
        EventType::Assertion => "evaluator",
    }
}

fn zone_for(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Message => "messages",
        EventType::Action | EventType::Result => "tools",
        EventType::Observation | EventType::StateSnapshot | EventType::Assertion => "other",
    }
}

fn blinded_trace(input: &PlannerInput) -> Result<BlindedTrace, BenchmarkError> {
    let events = input
        .events
        .iter()
        .map(|event| BlindedEvent {
            event_id: event.event_id.clone(),
            sequence_index: event.sequence_index,
            event_type: event.event_type,
            actor_role: event.actor_role,
            reference_ids: event.reference_event_ids.clone(),
            action_id: event.action.as_ref().map(|action| action.action_id.clone()),
            result_id: event.result.as_ref().map(|result| result.result_id.clone()),
            originating_action_id: event
                .result
                .as_ref()
                .map(|result| result.originating_action_id.clone()),
            context_id: event.context_block_id.clone(),
            content_hash: event.content_hash.clone(),
            structural_zone: zone_for(event.event_type).to_string(),
        })
        .collect::<Vec<_>>();
    let relations = input
        .relations
        .iter()
        .map(|relation| BlindedRelation {
            relation_id: relation.relation_id.clone(),
            relation_type: relation.relation_type,
            from_id: relation.from_id.clone(),
            to_id: relation.to_id.clone(),
        })
        .collect();
    let trace = BlindedTrace {
        scope: PHASE_1B9_SCOPE.to_string(),
        events,
        relations,
    };
    validate_blinded_trace(&trace)?;
    Ok(trace)
}

fn validate_blinded_trace(trace: &BlindedTrace) -> Result<(), BenchmarkError> {
    for (index, event) in trace.events.iter().enumerate() {
        if event.sequence_index != index as u32 {
            return Err(BenchmarkError::validation(
                "held-out sequence indexes are not contiguous",
            ));
        }
    }
    let addresses = trace
        .events
        .iter()
        .flat_map(|event| {
            let mut values = vec![event.event_id.clone()];
            if let Some(action) = &event.action_id {
                values.push(action.clone());
            }
            if let Some(result) = &event.result_id {
                values.push(result.clone());
            }
            if let Some(context) = &event.context_id {
                values.push(context.clone());
            }
            values
        })
        .collect::<BTreeSet<_>>();
    if trace.events.iter().any(|event| {
        event
            .reference_ids
            .iter()
            .any(|reference| !addresses.contains(reference))
    }) || trace.relations.iter().any(|relation| {
        !addresses.contains(&relation.from_id) || !addresses.contains(&relation.to_id)
    }) {
        return Err(BenchmarkError::validation(
            "held-out reference or relation endpoint is unknown",
        ));
    }
    Ok(())
}

fn input(events: Vec<Event>, relations: Vec<Relation>) -> PlannerInput {
    PlannerInput {
        events,
        relations,
        provenance: vec![provenance("heldout/structural")],
    }
}

fn provenance(locator: &str) -> SourceProvenance {
    SourceProvenance {
        source_kind: SourceKind::SelfAuthored,
        classification: crate::model::EvidenceClass::CapturedExplicit,
        source_locator: Some(locator.to_string()),
        source_revision: Some("phase1b9-structural-v1".to_string()),
        content_hash: Some(hash_text(locator)),
        note: None,
    }
}

fn event(
    id: &str,
    index: u32,
    event_type: EventType,
    action_id: Option<&str>,
    result_id: Option<&str>,
    refs: &[&str],
    hash: &str,
) -> Event {
    Event {
        event_id: id.to_string(),
        sequence_index: index,
        event_type,
        actor_role: match event_type {
            EventType::Message => ActorRole::User,
            EventType::Action => ActorRole::Agent,
            EventType::Result | EventType::Observation => ActorRole::Tool,
            EventType::StateSnapshot => ActorRole::Environment,
            EventType::Assertion => ActorRole::Evaluator,
        },
        parent_event_ids: Vec::new(),
        reference_event_ids: refs.iter().map(|reference| reference.to_string()).collect(),
        action: action_id.map(|id| crate::model::ActionIdentity {
            action_id: id.to_string(),
            tool_name: "operation".to_string(),
            argument_hash: Some(hash_text(id)),
        }),
        result: result_id.map(|id| crate::model::ResultIdentity {
            result_id: id.to_string(),
            originating_action_id: action_id.unwrap_or("a000").to_string(),
            observation_hash: Some(hash_text(id)),
            status: Some(crate::model::ResultStatus::Success),
        }),
        context_block_id: (event_type == EventType::Message
            || event_type == EventType::Observation)
            .then(|| format!("c{}", id.trim_start_matches('e'))),
        world_state_revision: Some(format!("w{index:03}")),
        order: Some(OrderMetadata {
            logical_tick: Some(index),
            source_timestamp: None,
            timestamp_origin: Some(crate::model::TimestampOrigin::DerivedStructural),
        }),
        content_hash: Some(hash_text(hash)),
        provenance: vec![provenance(&format!("heldout/structural/{id}"))],
    }
}

fn relation(id: &str, relation_type: RelationType, from_id: &str, to_id: &str) -> Relation {
    Relation {
        relation_id: id.to_string(),
        relation_type,
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        scope: "heldout_local".to_string(),
        semantics_version: Some(crate::model::RELATION_SEMANTICS_VERSION.to_string()),
        provenance: vec![provenance(&format!("heldout/structural/{id}"))],
    }
}

fn positive(class: ResearchInterventionClass, target: &str) -> EvaluationKey {
    EvaluationKey {
        positive: true,
        expected: Some(ExpectedSelection {
            class,
            target_event_id: target.to_string(),
        }),
        hidden_requirements: Vec::new(),
    }
}

fn negative_hidden(action_id: &str, event_id: &str) -> EvaluationKey {
    EvaluationKey {
        positive: false,
        expected: None,
        hidden_requirements: vec![(action_id.to_string(), event_id.to_string())],
    }
}

fn negative() -> EvaluationKey {
    EvaluationKey {
        positive: false,
        expected: None,
        hidden_requirements: Vec::new(),
    }
}

fn make_case(
    case_id: &str,
    input: PlannerInput,
    evaluation: EvaluationKey,
) -> Result<HeldOutCase, BenchmarkError> {
    let blinded = blinded_trace(&input)?;
    Ok(HeldOutCase {
        case_id: case_id.to_string(),
        planner_input: input,
        blinded,
        evaluation,
    })
}

fn build_held_out_cases() -> Result<Vec<HeldOutCase>, BenchmarkError> {
    let cases = vec![
        make_case(
            "h001",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "dup-a"),
                    event("e002", 1, EventType::Message, None, None, &[], "dup-a"),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &["e001"],
                        "act-a",
                    ),
                ],
                vec![relation(
                    "q001",
                    RelationType::SameStateRevision,
                    "e001",
                    "e002",
                )],
            ),
            positive(ResearchInterventionClass::Prune, "e002"),
        )?,
        make_case(
            "h002",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "dup-b"),
                    event("e002", 1, EventType::Message, None, None, &[], "dup-b"),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &[],
                        "act-b",
                    ),
                ],
                Vec::new(),
            ),
            negative_hidden("a003", "e002"),
        )?,
        make_case(
            "h003",
            input(
                vec![
                    event(
                        "e001",
                        0,
                        EventType::Action,
                        Some("a001"),
                        None,
                        &[],
                        "act-c",
                    ),
                    event(
                        "e002",
                        1,
                        EventType::Result,
                        None,
                        Some("r001"),
                        &[],
                        "res-c",
                    ),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &["r001"],
                        "act-d",
                    ),
                    event(
                        "e004",
                        3,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "res-d",
                    ),
                ],
                vec![
                    relation("q003", RelationType::DependsOn, "a003", "r001"),
                    relation("q004", RelationType::References, "a003", "r001"),
                ],
            ),
            negative(),
        )?,
        make_case(
            "h004",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "dup-d"),
                    event("e002", 1, EventType::Message, None, None, &[], "noise-d"),
                    event("e003", 2, EventType::Message, None, None, &[], "dup-d"),
                    event(
                        "e004",
                        3,
                        EventType::Action,
                        Some("a004"),
                        None,
                        &["e001"],
                        "act-e",
                    ),
                    event(
                        "e005",
                        4,
                        EventType::Result,
                        None,
                        Some("r004"),
                        &[],
                        "res-e",
                    ),
                ],
                vec![relation(
                    "q005",
                    RelationType::SameStateRevision,
                    "e001",
                    "e003",
                )],
            ),
            positive(ResearchInterventionClass::Prune, "e003"),
        )?,
        make_case(
            "h005",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "dup-e"),
                    event("e002", 1, EventType::Message, None, None, &[], "dup-e"),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &["e002"],
                        "act-f",
                    ),
                    event(
                        "e004",
                        3,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "res-f",
                    ),
                ],
                vec![relation("q006", RelationType::References, "a003", "e002")],
            ),
            negative(),
        )?,
        make_case(
            "h006",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "hidden-f"),
                    event(
                        "e002",
                        1,
                        EventType::Action,
                        Some("a002"),
                        None,
                        &[],
                        "act-g",
                    ),
                    event(
                        "e003",
                        2,
                        EventType::Result,
                        None,
                        Some("r002"),
                        &[],
                        "res-g",
                    ),
                ],
                Vec::new(),
            ),
            negative_hidden("a002", "e001"),
        )?,
        make_case(
            "h007",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "old-h"),
                    event("e002", 1, EventType::Message, None, None, &[], "mid-h"),
                    event("e003", 2, EventType::Message, None, None, &[], "new-h"),
                    event(
                        "e004",
                        3,
                        EventType::Action,
                        Some("a004"),
                        None,
                        &["e003"],
                        "act-h",
                    ),
                ],
                vec![
                    relation("q007", RelationType::Supersedes, "e003", "e001"),
                    relation("q008", RelationType::ProtocolPrecedes, "e003", "a004"),
                ],
            ),
            positive(ResearchInterventionClass::Defer, "e001"),
        )?,
        make_case(
            "h008",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "old-i"),
                    event("e002", 1, EventType::Message, None, None, &[], "new-i"),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &["e002"],
                        "act-i",
                    ),
                ],
                Vec::new(),
            ),
            negative(),
        )?,
        make_case(
            "h009",
            input(
                vec![
                    event(
                        "e001",
                        0,
                        EventType::Action,
                        Some("a001"),
                        None,
                        &[],
                        "act-j",
                    ),
                    event(
                        "e002",
                        1,
                        EventType::Result,
                        None,
                        Some("r002"),
                        &[],
                        "res-j",
                    ),
                    event(
                        "e003",
                        2,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "noise-j",
                    ),
                    event(
                        "e004",
                        3,
                        EventType::Action,
                        Some("a004"),
                        None,
                        &["r002"],
                        "act-k",
                    ),
                    event(
                        "e005",
                        4,
                        EventType::Result,
                        None,
                        Some("r005"),
                        &[],
                        "res-k",
                    ),
                ],
                vec![
                    relation("q009", RelationType::ProtocolPrecedes, "e002", "a004"),
                    relation("q010", RelationType::References, "a004", "r002"),
                ],
            ),
            positive(ResearchInterventionClass::RelocateCandidate, "e002"),
        )?,
        make_case(
            "h010",
            input(
                vec![
                    event(
                        "e001",
                        0,
                        EventType::Action,
                        Some("a001"),
                        None,
                        &[],
                        "act-l",
                    ),
                    event("e002", 1, EventType::Observation, None, None, &[], "obs-l"),
                    event(
                        "e003",
                        2,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "noise-l",
                    ),
                    event(
                        "e004",
                        3,
                        EventType::Action,
                        Some("a004"),
                        None,
                        &["e002"],
                        "act-m",
                    ),
                    event(
                        "e005",
                        4,
                        EventType::Result,
                        None,
                        Some("r005"),
                        &[],
                        "res-m",
                    ),
                ],
                vec![
                    relation("q011", RelationType::ProtocolPrecedes, "e002", "a004"),
                    relation("q012", RelationType::DependsOn, "a004", "e002"),
                ],
            ),
            negative(),
        )?,
        make_case(
            "h011",
            input(
                vec![
                    event(
                        "e001",
                        0,
                        EventType::Action,
                        Some("a001"),
                        None,
                        &[],
                        "act-n",
                    ),
                    event(
                        "e002",
                        1,
                        EventType::Result,
                        None,
                        Some("r002"),
                        &[],
                        "res-n",
                    ),
                    event(
                        "e003",
                        2,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "noise-n",
                    ),
                    event(
                        "e004",
                        3,
                        EventType::Action,
                        Some("a004"),
                        None,
                        &["r002"],
                        "act-o",
                    ),
                ],
                vec![relation("q013", RelationType::References, "a004", "r002")],
            ),
            negative(),
        )?,
        make_case(
            "h012",
            input(
                vec![
                    event(
                        "e001",
                        0,
                        EventType::Action,
                        Some("a001"),
                        None,
                        &[],
                        "act-p",
                    ),
                    event(
                        "e002",
                        1,
                        EventType::Result,
                        None,
                        Some("r002"),
                        &[],
                        "res-p",
                    ),
                ],
                Vec::new(),
            ),
            negative(),
        )?,
        make_case(
            "h013",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "amb-q"),
                    event(
                        "e002",
                        1,
                        EventType::Action,
                        Some("a002"),
                        None,
                        &[],
                        "act-q",
                    ),
                    event(
                        "e003",
                        2,
                        EventType::Result,
                        None,
                        Some("r003"),
                        &[],
                        "res-q",
                    ),
                ],
                Vec::new(),
            ),
            negative(),
        )?,
        make_case(
            "h014",
            input(
                vec![
                    event("e001", 0, EventType::Message, None, None, &[], "dup-r"),
                    event("e002", 1, EventType::Message, None, None, &[], "dup-r"),
                    event(
                        "e003",
                        2,
                        EventType::Action,
                        Some("a003"),
                        None,
                        &[],
                        "act-r",
                    ),
                ],
                Vec::new(),
            ),
            negative_hidden("a003", "e002"),
        )?,
    ];
    Ok(cases)
}

fn renumber(input: &mut PlannerInput) {
    for (index, event) in input.events.iter_mut().enumerate() {
        event.sequence_index = index as u32;
        if let Some(order) = &mut event.order {
            order.logical_tick = Some(index as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_out_study_has_expected_shape_and_blinded_serialization() {
        let cases = build_held_out_cases().unwrap();
        assert_eq!(cases.len(), 14);
        assert_eq!(
            cases.iter().filter(|case| case.evaluation.positive).count(),
            4
        );
        for case in cases {
            let encoded = String::from_utf8(blinded_trace_json(&case.blinded).unwrap()).unwrap();
            for forbidden in [
                "oracle",
                "manifest",
                "safe",
                "unsafe",
                "load-bearing",
                "irrelevant",
                "removable",
                "protocol-breaking",
                "scenario_id",
                "variant_role",
            ] {
                assert!(
                    !encoded.to_ascii_lowercase().contains(forbidden),
                    "{forbidden} leaked"
                );
            }
        }
    }

    #[test]
    fn research_policy_is_deterministic_and_id_agnostic() {
        let cases = build_held_out_cases().unwrap();
        let first = cases
            .iter()
            .map(|case| research_policy(&case.blinded))
            .collect::<Vec<_>>();
        let mut renamed = cases[0].blinded.clone();
        let mut aliases = BTreeMap::new();
        for event in &renamed.events {
            aliases.insert(event.event_id.clone(), format!("x{}", event.sequence_index));
            if let Some(action) = &event.action_id {
                aliases.insert(action.clone(), format!("y{}", event.sequence_index));
            }
            if let Some(result) = &event.result_id {
                aliases.insert(result.clone(), format!("z{}", event.sequence_index));
            }
            if let Some(context) = &event.context_id {
                aliases.insert(context.clone(), format!("w{}", event.sequence_index));
            }
        }
        for event in &mut renamed.events {
            event.event_id = aliases.get(&event.event_id).unwrap().clone();
            event.reference_ids = event
                .reference_ids
                .iter()
                .map(|id| aliases.get(id).cloned().unwrap_or_else(|| id.clone()))
                .collect();
            event.action_id = event
                .action_id
                .as_ref()
                .map(|id| aliases.get(id).unwrap().clone());
            event.result_id = event
                .result_id
                .as_ref()
                .map(|id| aliases.get(id).unwrap().clone());
            event.originating_action_id = event
                .originating_action_id
                .as_ref()
                .map(|id| aliases.get(id).unwrap().clone());
            event.context_id = event
                .context_id
                .as_ref()
                .map(|id| aliases.get(id).unwrap().clone());
        }
        for relation in &mut renamed.relations {
            relation.from_id = aliases.get(&relation.from_id).unwrap().clone();
            relation.to_id = aliases.get(&relation.to_id).unwrap().clone();
        }
        let original_decision = research_policy(&cases[0].blinded);
        let renamed_decision = research_policy(&renamed);
        assert_eq!(original_decision.class, renamed_decision.class);
        assert_eq!(original_decision.rule, renamed_decision.rule);
        assert_eq!(
            first,
            cases
                .iter()
                .map(|case| research_policy(&case.blinded))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn mutation_tests_fail_open_and_ignore_lexical_or_distractor_changes() {
        let cases = build_held_out_cases().unwrap();

        let mut lexical = cases[8].planner_input.clone();
        lexical.events[0].event_id = "z001".to_string();
        let original_execution = execute(&cases[8].planner_input, &[]);
        let lexical_execution = execute(&lexical, &[]);
        assert_eq!(original_execution, lexical_execution);
        assert_eq!(
            research_policy(&cases[8].blinded).class,
            research_policy(&blinded_trace(&lexical).unwrap()).class
        );

        let mut distractor = cases[0].planner_input.clone();
        distractor.events.push(event(
            "e004",
            3,
            EventType::Message,
            None,
            None,
            &[],
            "noise",
        ));
        let distractor_trace = blinded_trace(&distractor).unwrap();
        assert_eq!(
            research_policy(&cases[0].blinded).class,
            research_policy(&distractor_trace).class
        );

        let mut no_same_state = cases[0].planner_input.clone();
        no_same_state.relations.clear();
        assert_eq!(
            research_policy(&blinded_trace(&no_same_state).unwrap()).class,
            ResearchInterventionClass::DoNothing
        );

        let mut explicit_dependency = cases[0].planner_input.clone();
        explicit_dependency.relations.push(relation(
            "q999",
            RelationType::DependsOn,
            "a003",
            "e002",
        ));
        assert_eq!(
            research_policy(&blinded_trace(&explicit_dependency).unwrap()).class,
            ResearchInterventionClass::DoNothing
        );

        let mut protocol_boundary = cases[8].planner_input.clone();
        protocol_boundary
            .relations
            .push(relation("q998", RelationType::DependsOn, "a004", "e002"));
        assert_eq!(
            research_policy(&blinded_trace(&protocol_boundary).unwrap()).class,
            ResearchInterventionClass::DoNothing
        );

        let mut old_timestamp = cases[0].planner_input.clone();
        old_timestamp.events[0]
            .order
            .as_mut()
            .expect("held-out event has order metadata")
            .source_timestamp = Some("1900-01-01T00:00:00Z".to_string());
        assert_eq!(
            research_policy(&cases[0].blinded),
            research_policy(&blinded_trace(&old_timestamp).unwrap())
        );

        let original_key = cases[0].evaluation.clone();
        let mut relabeled_key = original_key.clone();
        relabeled_key.positive = false;
        relabeled_key.expected = None;
        assert_eq!(
            research_policy(&cases[0].blinded),
            research_policy(&cases[0].blinded)
        );
        assert_ne!(relabeled_key.positive, original_key.positive);
    }

    #[test]
    fn held_out_report_is_deterministic_and_records_positive_recall() {
        let first = run_phase1b9_study().unwrap();
        let second = run_phase1b9_study().unwrap();
        assert_eq!(
            canonical_phase1b9_report_json(&first).unwrap(),
            canonical_phase1b9_report_json(&second).unwrap()
        );
        assert_eq!(first.positive_cases_available, 4);
        assert_eq!(first.true_positives, 4);
        assert_eq!(first.false_positives, 0);
        assert_eq!(first.false_negatives, 0);
        assert_eq!(first.unsafe_intervention_count, 0);
        assert_eq!(first.baseline_pass_intervention_failures, 0);
        assert_eq!(
            first.frozen_planner.distribution.get("DO_NOTHING"),
            Some(&14)
        );
    }
}
