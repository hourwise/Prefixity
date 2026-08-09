use crate::error::BenchmarkError;
use crate::hashing::{canonical_hash, canonical_json};
use crate::model::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

pub fn load_envelope(bytes: &[u8]) -> Result<ControlledEnvelope, BenchmarkError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(BenchmarkError::validation(format!(
            "controlled envelope exceeds {} bytes",
            MAX_FILE_BYTES
        )));
    }
    let envelope: ControlledEnvelope =
        serde_json::from_slice(bytes).map_err(|source| BenchmarkError::InvalidJson {
            path: "<bytes>".into(),
            source,
        })?;
    validate_envelope(&envelope)?;
    Ok(envelope)
}

pub fn load_envelope_from_path(path: &Path) -> Result<ControlledEnvelope, BenchmarkError> {
    let bytes = std::fs::read(path).map_err(|source| BenchmarkError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    load_envelope(&bytes).map_err(|error| match error {
        BenchmarkError::InvalidJson { source, .. } => BenchmarkError::InvalidJson {
            path: path.to_path_buf(),
            source,
        },
        BenchmarkError::Validation { message, .. } => BenchmarkError::Validation {
            path: path.display().to_string(),
            message,
        },
        other => other,
    })
}

pub fn validate_envelope(envelope: &ControlledEnvelope) -> Result<(), BenchmarkError> {
    if envelope.schema_id != SCHEMA_ID {
        return Err(BenchmarkError::validation(format!(
            "schema_id must be {SCHEMA_ID}"
        )));
    }
    if envelope.schema_version != SCHEMA_VERSION {
        return Err(BenchmarkError::validation(format!(
            "schema_version must be {SCHEMA_VERSION}"
        )));
    }
    validate_id(&envelope.benchmark_id, "benchmark_id")?;
    validate_scenario(&envelope.scenario)?;
    validate_trace(&envelope.trace)?;
    validate_sidecar(&envelope.evaluation_sidecar, &envelope.trace)?;
    Ok(())
}

pub fn validate_case(case: &ControlledCase) -> Result<(), BenchmarkError> {
    validate_envelope(&case.baseline)?;
    validate_envelope(&case.intervention)?;

    if case.scenario_id != case.baseline.scenario.scenario_id
        || case.scenario_id != case.intervention.scenario.scenario_id
    {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "case scenario ID does not match both envelopes",
        ));
    }
    if case.baseline.benchmark_id != case.intervention.benchmark_id
        || case.baseline.benchmark_id != BENCHMARK_ID
    {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "benchmark identity differs or is not the approved seed",
        ));
    }
    if case.baseline.scenario != case.intervention.scenario {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "task, environment, initial-state, seed, or provenance identity differs",
        ));
    }
    if case.baseline.trace.variant_role != VariantRole::Baseline {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "baseline trace does not have baseline role",
        ));
    }
    if case.baseline.trace.baseline_trace_id != case.baseline.trace.trace_id
        || case.intervention.trace.baseline_trace_id != case.baseline.trace.trace_id
    {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "baseline_trace_id does not bind both traces to the baseline",
        ));
    }
    if case.manifest != case.baseline.evaluation_sidecar.intervention_manifest
        || case.manifest != case.intervention.evaluation_sidecar.intervention_manifest
    {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "pair manifest differs between case and sidecars",
        ));
    }
    if case.manifest.baseline_trace_id != case.baseline.trace.trace_id
        || case.manifest.variant_trace_id != case.intervention.trace.trace_id
    {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "manifest trace IDs do not match the pair",
        ));
    }
    let expected_manifest_hash = manifest_hash(&case.manifest)?;
    if case.manifest_hash != expected_manifest_hash {
        return Err(BenchmarkError::HashMismatch {
            what: format!("{} intervention manifest", case.scenario_id),
            expected: expected_manifest_hash,
            found: case.manifest_hash.clone(),
        });
    }

    let expected_role = match case.manifest.intervention_class {
        InterventionClass::NoChange => VariantRole::Control,
        _ => VariantRole::Variant,
    };
    if case.intervention.trace.variant_role != expected_role {
        return Err(BenchmarkError::pair(
            &case.scenario_id,
            "intervention role does not match intervention class",
        ));
    }

    validate_pair_shape(
        &case.scenario_id,
        &case.baseline.trace.planner_input,
        &case.intervention.trace.planner_input,
        &case.manifest,
    )
}

pub fn manifest_hash(manifest: &InterventionManifest) -> Result<String, BenchmarkError> {
    canonical_hash(manifest).map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn envelope_hash(envelope: &ControlledEnvelope) -> Result<String, BenchmarkError> {
    canonical_hash(envelope).map_err(|error| BenchmarkError::validation(error.to_string()))
}

pub fn canonical_envelope_json(envelope: &ControlledEnvelope) -> Result<Vec<u8>, BenchmarkError> {
    canonical_json(envelope).map_err(|error| BenchmarkError::validation(error.to_string()))
}

fn validate_scenario(scenario: &ScenarioIdentity) -> Result<(), BenchmarkError> {
    validate_id(&scenario.scenario_id, "scenario.scenario_id")?;
    if !is_version(&scenario.scenario_version) {
        return Err(BenchmarkError::validation(
            "scenario.scenario_version must be v followed by digits",
        ));
    }
    validate_text(&scenario.task_revision, "scenario.task_revision")?;
    validate_text(
        &scenario.environment_revision,
        "scenario.environment_revision",
    )?;
    validate_id(&scenario.initial_state_id, "scenario.initial_state_id")?;
    if scenario.provenance.len() > 16 {
        return Err(BenchmarkError::validation(
            "scenario.provenance exceeds its bound",
        ));
    }
    validate_provenance(&scenario.provenance, "scenario.provenance", false)
}

fn validate_trace(trace: &TraceEnvelope) -> Result<(), BenchmarkError> {
    validate_id(&trace.trace_id, "trace.trace_id")?;
    validate_id(&trace.baseline_trace_id, "trace.baseline_trace_id")?;
    validate_planner_input(&trace.planner_input)
}

fn validate_planner_input(input: &PlannerInput) -> Result<(), BenchmarkError> {
    if input.events.is_empty() || input.events.len() > MAX_EVENTS {
        return Err(BenchmarkError::validation(format!(
            "planner_input.events must contain 1..={MAX_EVENTS} events"
        )));
    }
    if input.relations.len() > MAX_RELATIONS {
        return Err(BenchmarkError::validation(format!(
            "planner_input.relations exceeds {MAX_RELATIONS}"
        )));
    }
    if input.provenance.len() > MAX_PROVENANCE {
        return Err(BenchmarkError::validation(format!(
            "planner_input.provenance exceeds {MAX_PROVENANCE}"
        )));
    }
    validate_provenance(&input.provenance, "planner_input.provenance", true)?;

    let mut event_ids = BTreeSet::new();
    let mut action_ids = BTreeMap::new();
    let mut result_ids = BTreeMap::new();
    let mut addressable_ids = BTreeSet::new();
    for (index, event) in input.events.iter().enumerate() {
        validate_id(&event.event_id, "event.event_id")?;
        if event.sequence_index != index as u32 {
            return Err(BenchmarkError::validation(format!(
                "event {} has sequence_index {}, expected {}",
                event.event_id, event.sequence_index, index
            )));
        }
        if !event_ids.insert(event.event_id.clone()) {
            return Err(BenchmarkError::validation(format!(
                "duplicate event ID {}",
                event.event_id
            )));
        }
        addressable_ids.insert(event.event_id.clone());
        if let Some(context_id) = &event.context_block_id {
            validate_id(context_id, "event.context_block_id")?;
            addressable_ids.insert(context_id.clone());
        }
        validate_id_list(&event.parent_event_ids, "event.parent_event_ids", 32)?;
        validate_id_list(&event.reference_event_ids, "event.reference_event_ids", 32)?;
        if let Some(action) = &event.action {
            validate_id(&action.action_id, "action.action_id")?;
            validate_text(&action.tool_name, "action.tool_name")?;
            if let Some(hash) = &action.argument_hash {
                validate_hash(hash, "action.argument_hash")?;
            }
            if event.event_type != EventType::Action {
                return Err(BenchmarkError::validation(format!(
                    "action identity on {} requires event_type=action",
                    event.event_id
                )));
            }
            if action_ids
                .insert(action.action_id.clone(), event.event_id.clone())
                .is_some()
            {
                return Err(BenchmarkError::validation(format!(
                    "duplicate action ID {}",
                    action.action_id
                )));
            }
            addressable_ids.insert(action.action_id.clone());
        }
        if let Some(result) = &event.result {
            validate_id(&result.result_id, "result.result_id")?;
            validate_id(
                &result.originating_action_id,
                "result.originating_action_id",
            )?;
            if let Some(hash) = &result.observation_hash {
                validate_hash(hash, "result.observation_hash")?;
            }
            if event.event_type != EventType::Result && event.event_type != EventType::Observation {
                return Err(BenchmarkError::validation(format!(
                    "result identity on {} requires event_type=result or observation",
                    event.event_id
                )));
            }
            if result_ids
                .insert(result.result_id.clone(), event.event_id.clone())
                .is_some()
            {
                return Err(BenchmarkError::validation(format!(
                    "duplicate result ID {}",
                    result.result_id
                )));
            }
            addressable_ids.insert(result.result_id.clone());
        }
        if let Some(hash) = &event.content_hash {
            validate_hash(hash, "event.content_hash")?;
        }
        if let Some(order) = &event.order {
            if order.source_timestamp.is_some()
                && order.timestamp_origin != Some(TimestampOrigin::SourceExplicit)
            {
                return Err(BenchmarkError::validation(format!(
                    "source timestamp on {} must be source_explicit",
                    event.event_id
                )));
            }
            if order.timestamp_origin == Some(TimestampOrigin::DerivedStructural)
                && order.source_timestamp.is_some()
            {
                return Err(BenchmarkError::validation(format!(
                    "derived order on {} cannot carry a source timestamp",
                    event.event_id
                )));
            }
        }
        validate_provenance(
            &event.provenance,
            &format!("event {} provenance", event.event_id),
            true,
        )?;
    }

    for event in &input.events {
        for id in event
            .parent_event_ids
            .iter()
            .chain(event.reference_event_ids.iter())
        {
            if !event_ids.contains(id) {
                return Err(BenchmarkError::validation(format!(
                    "event {} references unknown event {}",
                    event.event_id, id
                )));
            }
        }
        if let Some(result) = &event.result {
            if !action_ids.contains_key(&result.originating_action_id) {
                return Err(BenchmarkError::validation(format!(
                    "result {} originates from unknown action {}",
                    result.result_id, result.originating_action_id
                )));
            }
        }
    }

    let mut relation_ids = BTreeSet::new();
    for relation in &input.relations {
        validate_id(&relation.relation_id, "relation.relation_id")?;
        if !relation_ids.insert(relation.relation_id.clone()) {
            return Err(BenchmarkError::validation(format!(
                "duplicate relation ID {}",
                relation.relation_id
            )));
        }
        if relation.scope != "scenario_local"
            || relation.semantics_version.as_deref() != Some(RELATION_SEMANTICS_VERSION)
        {
            return Err(BenchmarkError::validation(format!(
                "relation {} has unsupported scope or semantics version",
                relation.relation_id
            )));
        }
        if !addressable_ids.contains(&relation.from_id)
            || !addressable_ids.contains(&relation.to_id)
        {
            return Err(BenchmarkError::validation(format!(
                "relation {} has an unknown endpoint",
                relation.relation_id
            )));
        }
        validate_provenance(
            &relation.provenance,
            &format!("relation {} provenance", relation.relation_id),
            true,
        )?;
        if relation.relation_type == RelationType::Produces {
            let Some(action_event_id) = action_ids.get(&relation.from_id) else {
                return Err(BenchmarkError::validation(format!(
                    "produces relation {} does not originate from an action",
                    relation.relation_id
                )));
            };
            let Some(result_event_id) = result_ids.get(&relation.to_id) else {
                return Err(BenchmarkError::validation(format!(
                    "produces relation {} does not target a result",
                    relation.relation_id
                )));
            };
            let action_event = input
                .events
                .iter()
                .find(|event| &event.event_id == action_event_id)
                .expect("action event index is built from input events");
            let result_event = input
                .events
                .iter()
                .find(|event| &event.event_id == result_event_id)
                .expect("result event index is built from input events");
            if result_event
                .result
                .as_ref()
                .map(|result| result.originating_action_id.as_str())
                != action_event
                    .action
                    .as_ref()
                    .map(|action| action.action_id.as_str())
            {
                return Err(BenchmarkError::validation(format!(
                    "produces relation {} disagrees with result origin",
                    relation.relation_id
                )));
            }
        }
    }
    Ok(())
}

fn validate_sidecar(
    sidecar: &EvaluationSidecar,
    trace: &TraceEnvelope,
) -> Result<(), BenchmarkError> {
    validate_id(
        &sidecar.intervention_manifest_ref,
        "evaluation_sidecar.intervention_manifest_ref",
    )?;
    if sidecar.intervention_manifest_ref != sidecar.intervention_manifest.manifest_id {
        return Err(BenchmarkError::validation(
            "intervention_manifest_ref does not match manifest_id",
        ));
    }
    validate_manifest(&sidecar.intervention_manifest)?;
    if sidecar.quality_evaluation_ids.len() > 8 {
        return Err(BenchmarkError::validation(
            "quality_evaluation_ids exceeds its bound",
        ));
    }
    validate_id_list(&sidecar.quality_evaluation_ids, "quality_evaluation_ids", 8)?;
    if let Some(hash) = &sidecar.evaluation_content_hash {
        validate_hash(hash, "evaluation_content_hash")?;
    }
    if sidecar.intervention_manifest.baseline_trace_id != trace.baseline_trace_id
        && trace.variant_role == VariantRole::Baseline
    {
        return Err(BenchmarkError::validation(
            "baseline sidecar manifest does not point at the trace baseline",
        ));
    }
    Ok(())
}

fn validate_manifest(manifest: &InterventionManifest) -> Result<(), BenchmarkError> {
    validate_id(&manifest.manifest_id, "manifest_id")?;
    validate_id(&manifest.baseline_trace_id, "manifest.baseline_trace_id")?;
    validate_id(&manifest.variant_trace_id, "manifest.variant_trace_id")?;
    if manifest.target_event_ids.is_empty() || manifest.target_event_ids.len() > 32 {
        return Err(BenchmarkError::validation(
            "manifest.target_event_ids must contain 1..=32 IDs",
        ));
    }
    validate_id_list(&manifest.target_event_ids, "manifest.target_event_ids", 32)?;
    validate_text(
        &manifest.exact_transformation,
        "manifest.exact_transformation",
    )?;
    validate_text(&manifest.reason, "manifest.reason")?;
    validate_text(
        &manifest.expected_structural_effect,
        "manifest.expected_structural_effect",
    )?;
    Ok(())
}

fn validate_provenance(
    entries: &[SourceProvenance],
    field: &str,
    planner_visible: bool,
) -> Result<(), BenchmarkError> {
    for entry in entries {
        if let Some(locator) = &entry.source_locator {
            validate_text(locator, &format!("{field}.source_locator"))?;
        }
        if let Some(revision) = &entry.source_revision {
            validate_text(revision, &format!("{field}.source_revision"))?;
        }
        if let Some(hash) = &entry.content_hash {
            validate_hash(hash, &format!("{field}.content_hash"))?;
        }
        if let Some(note) = &entry.note {
            validate_text(note, &format!("{field}.note"))?;
        }
        if planner_visible && entry.classification == EvidenceClass::EvaluationOnly {
            return Err(BenchmarkError::validation(format!(
                "evaluation-only provenance cannot enter {field}"
            )));
        }
    }
    Ok(())
}

fn validate_id_list(ids: &[String], field: &str, max: usize) -> Result<(), BenchmarkError> {
    if ids.len() > max {
        return Err(BenchmarkError::validation(format!(
            "{field} exceeds maximum length {max}"
        )));
    }
    let mut seen = BTreeSet::new();
    for id in ids {
        validate_id(id, field)?;
        if !seen.insert(id) {
            return Err(BenchmarkError::validation(format!(
                "{field} contains duplicate ID {id}"
            )));
        }
    }
    Ok(())
}

fn validate_id(id: &str, field: &str) -> Result<(), BenchmarkError> {
    if id.len() > MAX_ID_BYTES
        || id.is_empty()
        || !id.as_bytes()[0].is_ascii_alphanumeric()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'))
    {
        return Err(BenchmarkError::validation(format!(
            "{field} contains malformed ID"
        )));
    }
    Ok(())
}

fn validate_text(text: &str, field: &str) -> Result<(), BenchmarkError> {
    if text.is_empty() || text.len() > MAX_TEXT_BYTES {
        return Err(BenchmarkError::validation(format!(
            "{field} must be non-empty and at most {MAX_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}

fn validate_hash(hash: &str, field: &str) -> Result<(), BenchmarkError> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(BenchmarkError::validation(format!(
            "{field} must be lowercase SHA-256"
        )));
    }
    Ok(())
}

fn is_version(version: &str) -> bool {
    version.len() > 1
        && version.starts_with('v')
        && version[1..].bytes().all(|b| b.is_ascii_digit())
}

fn validate_pair_shape(
    scenario_id: &str,
    baseline: &PlannerInput,
    intervention: &PlannerInput,
    manifest: &InterventionManifest,
) -> Result<(), BenchmarkError> {
    let baseline_events = index_events(baseline);
    let intervention_events = index_events(intervention);
    let target_ids: BTreeSet<&str> = manifest
        .target_event_ids
        .iter()
        .map(String::as_str)
        .collect();

    match manifest.intervention_class {
        InterventionClass::Remove => {
            for target in &target_ids {
                if !baseline_events.contains_key(*target) {
                    return Err(BenchmarkError::pair(
                        scenario_id,
                        format!("removal target {target} is absent from baseline"),
                    ));
                }
                if intervention_events.contains_key(*target) {
                    return Err(BenchmarkError::pair(
                        scenario_id,
                        format!("removal target {target} remains in intervention"),
                    ));
                }
            }
            if baseline_events.len() != intervention_events.len() + target_ids.len() {
                return Err(BenchmarkError::pair(
                    scenario_id,
                    "removal changed more event identities than declared",
                ));
            }
        }
        InterventionClass::Defer | InterventionClass::Relocate | InterventionClass::NoChange => {
            if baseline_events.keys().collect::<BTreeSet<_>>()
                != intervention_events.keys().collect::<BTreeSet<_>>()
            {
                return Err(BenchmarkError::pair(
                    scenario_id,
                    "non-removal intervention changed event identities",
                ));
            }
        }
        InterventionClass::Compress => {
            return Err(BenchmarkError::pair(
                scenario_id,
                "compression is not implemented in the approved seed",
            ));
        }
    }

    let baseline_order = event_order(baseline);
    let intervention_order = event_order(intervention);
    let target_aliases = target_aliases(baseline, &target_ids);
    let baseline_common = baseline_order
        .iter()
        .filter(|id| !target_ids.contains(id.as_str()))
        .collect::<Vec<_>>();
    let intervention_common = intervention_order
        .iter()
        .filter(|id| !target_ids.contains(id.as_str()))
        .collect::<Vec<_>>();
    if baseline_common != intervention_common {
        return Err(BenchmarkError::pair(
            scenario_id,
            "relative order of non-target events changed",
        ));
    }

    for event_id in &baseline_common {
        let base = baseline_events
            .get(event_id.as_str())
            .expect("indexed event");
        let changed = intervention_events
            .get(event_id.as_str())
            .expect("common event exists");
        if !same_event_except_order_and_target_refs(base, changed, &target_ids) {
            return Err(BenchmarkError::pair(
                scenario_id,
                format!("undeclared mutation in common event {event_id}"),
            ));
        }
    }

    let baseline_relations = scrub_relations(&baseline.relations, &target_aliases);
    let intervention_relations = scrub_relations(&intervention.relations, &target_aliases);
    if baseline_relations != intervention_relations
        && manifest.intervention_class != InterventionClass::Remove
    {
        return Err(BenchmarkError::pair(
            scenario_id,
            "undeclared relation mutation",
        ));
    }
    if manifest.intervention_class == InterventionClass::NoChange
        && baseline_order != intervention_order
    {
        return Err(BenchmarkError::pair(
            scenario_id,
            "no-change control reordered events",
        ));
    }
    if matches!(
        manifest.intervention_class,
        InterventionClass::Defer | InterventionClass::Relocate
    ) {
        for target in &target_ids {
            if baseline_events.get(*target).map(|e| e.sequence_index)
                == intervention_events.get(*target).map(|e| e.sequence_index)
            {
                return Err(BenchmarkError::pair(
                    scenario_id,
                    format!("relocation target {target} did not move"),
                ));
            }
        }
    }
    Ok(())
}

fn index_events(input: &PlannerInput) -> BTreeMap<String, Event> {
    input
        .events
        .iter()
        .cloned()
        .map(|event| (event.event_id.clone(), event))
        .collect()
}

fn event_order(input: &PlannerInput) -> Vec<String> {
    input
        .events
        .iter()
        .map(|event| event.event_id.clone())
        .collect()
}

fn same_event_except_order_and_target_refs(
    baseline: &Event,
    intervention: &Event,
    target_ids: &BTreeSet<&str>,
) -> bool {
    let mut left = baseline.clone();
    let mut right = intervention.clone();
    left.sequence_index = 0;
    right.sequence_index = 0;
    left.order = None;
    right.order = None;
    left.parent_event_ids
        .retain(|id| !target_ids.contains(id.as_str()));
    right
        .parent_event_ids
        .retain(|id| !target_ids.contains(id.as_str()));
    left.reference_event_ids
        .retain(|id| !target_ids.contains(id.as_str()));
    right
        .reference_event_ids
        .retain(|id| !target_ids.contains(id.as_str()));
    left == right
}

fn scrub_relations(relations: &[Relation], target_ids: &BTreeSet<String>) -> Vec<Relation> {
    relations
        .iter()
        .filter(|relation| {
            !target_ids.contains(relation.from_id.as_str())
                && !target_ids.contains(relation.to_id.as_str())
        })
        .cloned()
        .collect()
}

fn target_aliases(input: &PlannerInput, target_ids: &BTreeSet<&str>) -> BTreeSet<String> {
    let mut aliases = target_ids
        .iter()
        .map(|id| (*id).to_string())
        .collect::<BTreeSet<String>>();
    for event in &input.events {
        if target_ids.contains(event.event_id.as_str()) {
            aliases.insert(event.event_id.clone());
            if let Some(context_id) = &event.context_block_id {
                aliases.insert(context_id.clone());
            }
            if let Some(action) = &event.action {
                aliases.insert(action.action_id.clone());
            }
            if let Some(result) = &event.result {
                aliases.insert(result.result_id.clone());
            }
        }
    }
    aliases
}
