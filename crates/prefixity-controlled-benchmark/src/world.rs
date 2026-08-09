//! Minimal deterministic world for the twelve self-authored scenarios.

use crate::error::BenchmarkError;
use crate::hashing::canonical_hash;
use crate::loader::validate_envelope;
use crate::model::{ControlledEnvelope, Event, RelationType};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Complete,
    TaskFailure,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldExecution {
    pub status: ExecutionStatus,
    pub completed: bool,
    pub final_state: BTreeMap<String, String>,
    pub final_state_hash: Option<String>,
    pub note: String,
}

pub struct ScriptedWorld;

impl ScriptedWorld {
    pub fn execute(&self, envelope: &ControlledEnvelope) -> Result<WorldExecution, BenchmarkError> {
        validate_envelope(envelope)?;
        let scenario_id = &envelope.scenario.scenario_id;
        let events = envelope
            .trace
            .planner_input
            .events
            .iter()
            .map(|event| (event.event_id.clone(), event.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut actions = BTreeMap::new();
        for event in events.values() {
            if let Some(action) = &event.action {
                actions.insert(action.action_id.clone(), event.clone());
            }
        }
        let mut state = initial_state(envelope);

        if let Some(failure) =
            relation_failure(&envelope.trace.planner_input.relations, &events, &actions)
        {
            return Ok(task_failure(state, failure));
        }

        for event in events.values() {
            let Some(action) = &event.action else {
                continue;
            };
            if let Some(failure) = reference_order_failure(event, &events) {
                return Ok(task_failure(state, failure));
            }
            match execute_action(
                scenario_id,
                event,
                action.tool_name.as_str(),
                &events,
                &mut state,
            ) {
                ActionResult::Continue => {}
                ActionResult::TaskFailure(note) => return Ok(task_failure(state, note)),
                ActionResult::Unresolved(note) => return Ok(unresolved(state, note)),
            }
        }

        let final_state_hash =
            Some(
                canonical_hash(&state).map_err(|error| BenchmarkError::World {
                    scenario_id: scenario_id.clone(),
                    message: error.to_string(),
                })?,
            );
        Ok(WorldExecution {
            status: ExecutionStatus::Complete,
            completed: true,
            final_state: state,
            final_state_hash,
            note: "all scripted actions completed".to_string(),
        })
    }
}

enum ActionResult {
    Continue,
    TaskFailure(String),
    Unresolved(String),
}

fn execute_action(
    scenario_id: &str,
    event: &Event,
    tool_name: &str,
    events: &BTreeMap<String, Event>,
    state: &mut BTreeMap<String, String>,
) -> ActionResult {
    let has_reference = |expected: &str| {
        event
            .reference_event_ids
            .iter()
            .any(|reference| reference == expected && events.contains_key(reference))
    };
    match tool_name {
        "update_profile" => {
            state.insert("profile_updated".to_string(), "done".to_string());
            ActionResult::Continue
        }
        "check_inventory" => {
            state.insert("inventory_checked".to_string(), "ready".to_string());
            ActionResult::Continue
        }
        "checkout" => {
            if !has_reference("evt-S02-inventory-result") {
                ActionResult::TaskFailure("checkout lacks the inventory result".to_string())
            } else {
                state.insert("checkout".to_string(), "complete".to_string());
                ActionResult::Continue
            }
        }
        "apply_policy" => {
            if !has_reference("evt-S03-policy-new") {
                ActionResult::TaskFailure(
                    "current policy v2 is absent before apply_policy".to_string(),
                )
            } else {
                state.insert("policy_version".to_string(), "v2".to_string());
                ActionResult::Continue
            }
        }
        "create_record" => {
            state.insert("record_created".to_string(), "yes".to_string());
            if scenario_id == "S06_dependency_chain_preservation" {
                state.insert("chain_created".to_string(), "yes".to_string());
            }
            ActionResult::Continue
        }
        "update_record" => {
            if !has_reference("evt-S04-created-result") {
                ActionResult::TaskFailure(
                    "update_record lacks the generated identifier".to_string(),
                )
            } else {
                state.insert("record_updated".to_string(), "yes".to_string());
                ActionResult::Continue
            }
        }
        "write_audit" => ActionResult::Continue,
        "authorize_change" => {
            if !has_reference("evt-S06-create-result") {
                ActionResult::TaskFailure("authorization lacks the create result".to_string())
            } else {
                state.insert("authorized".to_string(), "yes".to_string());
                ActionResult::Continue
            }
        }
        "commit_change" => {
            if !has_reference("evt-S06-authorize-result") {
                ActionResult::TaskFailure("commit lacks the authorization result".to_string())
            } else if state.get("authorized") != Some(&"yes".to_string()) {
                ActionResult::TaskFailure("commit occurred without authorization".to_string())
            } else {
                state.insert("committed".to_string(), "yes".to_string());
                ActionResult::Continue
            }
        }
        "execute_with_reference" => {
            if event.reference_event_ids.is_empty() {
                ActionResult::TaskFailure("reference-dependent action has no reference".to_string())
            } else {
                state.insert("reference_execution".to_string(), "complete".to_string());
                ActionResult::Continue
            }
        }
        "execute_with_handshake" => {
            if !has_reference("evt-S08-handshake") {
                ActionResult::TaskFailure("handshake is absent or not available".to_string())
            } else {
                state.insert("handshake_execution".to_string(), "complete".to_string());
                ActionResult::Continue
            }
        }
        "execute_load_bearing_repeat" => {
            if !has_reference("evt-S10-repeat-2") {
                ActionResult::TaskFailure("load-bearing repeated context is absent".to_string())
            } else {
                state.insert("load_bearing_execution".to_string(), "complete".to_string());
                ActionResult::Continue
            }
        }
        "minimal_task" => {
            state.insert("minimal_task".to_string(), "complete".to_string());
            ActionResult::Continue
        }
        "finish_ambiguous_task" => {
            state.insert("ambiguous_task".to_string(), "complete".to_string());
            ActionResult::Continue
        }
        other => ActionResult::Unresolved(format!("unknown scripted tool {other}")),
    }
}

fn relation_failure(
    relations: &[crate::model::Relation],
    events: &BTreeMap<String, Event>,
    actions: &BTreeMap<String, Event>,
) -> Option<String> {
    for relation in relations {
        if relation.relation_type != RelationType::DependsOn
            && relation.relation_type != RelationType::ProtocolPrecedes
        {
            continue;
        }
        let from = resolve_address(&relation.from_id, events, actions)?;
        let to = resolve_address(&relation.to_id, events, actions)?;
        let invalid_order = match relation.relation_type {
            RelationType::DependsOn => from.sequence_index <= to.sequence_index,
            RelationType::ProtocolPrecedes => from.sequence_index >= to.sequence_index,
            _ => false,
        };
        if invalid_order {
            return Some(format!(
                "{} relation order is invalid: {} is not before {}",
                relation.relation_id, relation.from_id, relation.to_id
            ));
        }
    }
    None
}

fn reference_order_failure(event: &Event, events: &BTreeMap<String, Event>) -> Option<String> {
    for reference in &event.reference_event_ids {
        let referenced = events.get(reference)?;
        if referenced.sequence_index >= event.sequence_index {
            return Some(format!(
                "event {} references {} at a later order position",
                event.event_id, reference
            ));
        }
    }
    None
}

fn resolve_address<'a>(
    address: &str,
    events: &'a BTreeMap<String, Event>,
    actions: &'a BTreeMap<String, Event>,
) -> Option<&'a Event> {
    events
        .get(address)
        .or_else(|| actions.get(address))
        .or_else(|| {
            events.values().find(|event| {
                event
                    .result
                    .as_ref()
                    .is_some_and(|result| result.result_id == address)
                    || event
                        .context_block_id
                        .as_ref()
                        .is_some_and(|context| context == address)
            })
        })
}

fn initial_state(envelope: &ControlledEnvelope) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "scenario_id".to_string(),
            envelope.scenario.scenario_id.clone(),
        ),
        (
            "initial_state_id".to_string(),
            envelope.scenario.initial_state_id.clone(),
        ),
        (
            "task_revision".to_string(),
            envelope.scenario.task_revision.clone(),
        ),
    ])
}

fn task_failure(state: BTreeMap<String, String>, note: String) -> WorldExecution {
    WorldExecution {
        status: ExecutionStatus::TaskFailure,
        completed: false,
        final_state: state,
        final_state_hash: None,
        note,
    }
}

fn unresolved(state: BTreeMap<String, String>, note: String) -> WorldExecution {
    WorldExecution {
        status: ExecutionStatus::Unresolved,
        completed: false,
        final_state: state,
        final_state_hash: None,
        note,
    }
}

#[allow(dead_code)]
fn _stable_state_keys(state: &BTreeMap<String, String>) -> BTreeSet<String> {
    state.keys().cloned().collect()
}
