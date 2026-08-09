use crate::error::BenchmarkError;
use crate::hashing::canonical_hash;
use crate::loader::validate_envelope;
use crate::model::{
    ControlledEnvelope, EventType, PlannerEvidence, PlannerRun, RelationType, SourceProvenance,
};
use prefixity_core::decision::plan_interventions;
use prefixity_core::hash::hash_content;
use prefixity_core::model::{
    ContextBlock, EvidenceOrigin, EvidenceProvenance, RequestTrace, SourceLocator,
    TRACE_FORMAT_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn project_planner_evidence(
    envelope: &ControlledEnvelope,
) -> Result<PlannerEvidence, BenchmarkError> {
    validate_envelope(envelope)?;
    let request_trace = project_request_trace(envelope)?;
    Ok(PlannerEvidence {
        benchmark_id: envelope.benchmark_id.clone(),
        scenario: envelope.scenario.clone(),
        trace: envelope.trace.clone(),
        request_trace,
    })
}

pub fn run_frozen_planner(evidence: &PlannerEvidence) -> Result<PlannerRun, BenchmarkError> {
    let plan = plan_interventions(&evidence.request_trace)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let plan_json = serde_json::to_value(&plan)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let plan_json_hash = canonical_hash(&plan_json)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(PlannerRun {
        scenario_id: evidence.scenario.scenario_id.clone(),
        trace_id: evidence.trace.trace_id.clone(),
        classes: plan
            .recommendations
            .iter()
            .map(|recommendation| recommendation.class.as_str().to_string())
            .collect(),
        plan_json_hash,
    })
}

fn project_request_trace(envelope: &ControlledEnvelope) -> Result<RequestTrace, BenchmarkError> {
    let input = &envelope.trace.planner_input;
    let mut address_to_event = BTreeMap::new();
    let mut action_to_event = BTreeMap::new();
    let mut result_to_event = BTreeMap::new();
    for event in &input.events {
        let block_id = block_id(&event.event_id);
        address_to_event.insert(event.event_id.clone(), block_id.clone());
        if let Some(context_id) = &event.context_block_id {
            address_to_event.insert(context_id.clone(), block_id.clone());
        }
        if let Some(action) = &event.action {
            action_to_event.insert(action.action_id.clone(), event.event_id.clone());
            address_to_event.insert(action.action_id.clone(), block_id.clone());
        }
        if let Some(result) = &event.result {
            result_to_event.insert(result.result_id.clone(), event.event_id.clone());
            address_to_event.insert(result.result_id.clone(), block_id);
        }
    }

    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for relation in &input.relations {
        if relation.relation_type != RelationType::DependsOn {
            continue;
        }
        let Some(from_event) =
            resolve_event_id(&relation.from_id, &action_to_event, &result_to_event)
        else {
            continue;
        };
        let Some(target_block) = address_to_event.get(&relation.to_id) else {
            continue;
        };
        dependencies
            .entry(block_id(&from_event))
            .or_default()
            .insert(target_block.clone());
    }

    let blocks = input
        .events
        .iter()
        .map(|event| {
            let id = block_id(&event.event_id);
            let mut metadata = BTreeMap::new();
            metadata.insert(
                "controlled_event_id".to_string(),
                serde_json::Value::String(event.event_id.clone()),
            );
            metadata.insert(
                "controlled_event_type".to_string(),
                serde_json::to_value(event.event_type).expect("event type is serializable"),
            );
            if let Some(action) = &event.action {
                metadata.insert(
                    "action_id".to_string(),
                    serde_json::Value::String(action.action_id.clone()),
                );
                metadata.insert(
                    "tool_name".to_string(),
                    serde_json::Value::String(action.tool_name.clone()),
                );
            }
            if let Some(result) = &event.result {
                metadata.insert(
                    "result_id".to_string(),
                    serde_json::Value::String(result.result_id.clone()),
                );
                metadata.insert(
                    "originating_action_id".to_string(),
                    serde_json::Value::String(result.originating_action_id.clone()),
                );
            }
            let provenance = convert_provenance(&event.provenance);
            ContextBlock {
                id,
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
                semantic_zone: Some(zone_for(event.event_type).to_string()),
                structural_path: Some(format!("controlled.events[{}]", event.sequence_index)),
                role: Some(role_for(event.event_type).to_string()),
                sensitivity: None,
                dependencies: dependencies
                    .remove(&block_id(&event.event_id))
                    .unwrap_or_default()
                    .into_iter()
                    .collect(),
                lifetime: None,
                // The controlled benchmark never feeds expected safety labels
                // into the frozen planner projection.
                optional: false,
                required: false,
                stale: false,
                provenance,
                metadata,
            }
        })
        .collect();

    let mut trace_provenance = BTreeMap::new();
    for (index, provenance) in input.provenance.iter().enumerate() {
        trace_provenance.insert(
            format!("planner_input_{index}"),
            convert_one_provenance(provenance),
        );
    }
    let mut trace_metadata = BTreeMap::new();
    trace_metadata.insert(
        "controlled_benchmark_id".to_string(),
        serde_json::Value::String(envelope.benchmark_id.clone()),
    );
    trace_metadata.insert(
        "scenario_id".to_string(),
        serde_json::Value::String(envelope.scenario.scenario_id.clone()),
    );
    trace_metadata.insert(
        "variant_role".to_string(),
        serde_json::to_value(envelope.trace.variant_role).expect("variant role is serializable"),
    );

    let trace = RequestTrace {
        format_version: TRACE_FORMAT_VERSION,
        request_id: envelope.trace.trace_id.clone(),
        session_id: Some(envelope.scenario.scenario_id.clone()),
        timestamp: None,
        provider: "scripted-world".to_string(),
        model: "controlled-benchmark-v1".to_string(),
        evidence_schema_version: None,
        blocks,
        usage: None,
        provider_response: None,
        latency: None,
        provenance: trace_provenance,
        metadata: trace_metadata,
    };
    prefixity_core::validation::validate_trace(&trace, None)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    Ok(trace)
}

fn convert_provenance(entries: &[SourceProvenance]) -> BTreeMap<String, EvidenceProvenance> {
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| (format!("source_{index}"), convert_one_provenance(entry)))
        .collect()
}

fn convert_one_provenance(entry: &SourceProvenance) -> EvidenceProvenance {
    EvidenceProvenance {
        origin: match entry.classification {
            crate::model::EvidenceClass::CapturedExplicit => EvidenceOrigin::SourceExplicit,
            crate::model::EvidenceClass::DerivedStructural => EvidenceOrigin::DerivedStructural,
            _ => EvidenceOrigin::Unknown,
        },
        source_locator: entry.source_locator.as_ref().map(|locator| SourceLocator {
            upstream_field_path: Some(locator.clone()),
            ..SourceLocator::default()
        }),
        derivation_rule: entry.note.clone(),
        derivation_inputs: Vec::new(),
        evaluation_only: false,
    }
}

fn resolve_event_id(
    id: &str,
    action_to_event: &BTreeMap<String, String>,
    result_to_event: &BTreeMap<String, String>,
) -> Option<String> {
    action_to_event
        .get(id)
        .or_else(|| result_to_event.get(id))
        .cloned()
}

fn block_id(event_id: &str) -> String {
    format!("controlled-{event_id}")
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

fn zone_for(event_type: EventType) -> &'static str {
    match event_type {
        EventType::Message => "messages",
        EventType::Action | EventType::Result => "tools",
        EventType::Observation | EventType::StateSnapshot | EventType::Assertion => "other",
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

#[allow(dead_code)]
fn _stable_projection_hash(evidence: &PlannerEvidence) -> Result<String, BenchmarkError> {
    canonical_hash(evidence).map_err(|error| BenchmarkError::validation(error.to_string()))
}
