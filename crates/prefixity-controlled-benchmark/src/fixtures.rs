//! Self-authored, deterministic twelve-case seed.

use crate::error::BenchmarkError;
use crate::hashing::{hash_text, sha256_hex};
use crate::loader::{manifest_hash, validate_case};
use crate::model::*;

pub fn build_seed() -> Result<Vec<ControlledCase>, BenchmarkError> {
    let cases = vec![
        s01_irrelevant_removal()?,
        s02_load_bearing_result()?,
        s03_supersession_defer()?,
        s04_result_needed_later()?,
        s05_result_not_needed()?,
        s06_dependency_chain()?,
        s07_safe_relocation()?,
        s08_protocol_relocation()?,
        s09_exact_repeat()?,
        s10_load_bearing_repeat()?,
        s11_noop_control()?,
        s12_ambiguous_control()?,
    ];
    for case in &cases {
        validate_case(case)?;
    }
    Ok(cases)
}

fn s01_irrelevant_removal() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message(
                "S01",
                "irrelevant",
                "ctx_irrelevant",
                0,
                "unrelated-reference",
            ),
            action("S01", "update", "update_profile", 1, Vec::new()),
            result("S01", "update-result", "update", 2),
        ],
        vec![produces("S01", "update", "update-result")],
    );
    let intervention = remove_events(&baseline, &["evt-S01-irrelevant"]);
    case_with(
        "S01_irrelevant_context_removal",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S01-irrelevant"],
        "Remove only the unrelated context event.",
        "No referenced event or state transition changes.",
        QualityRiskCategory::Low,
    )
}

fn s02_load_bearing_result() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            action("S02", "inventory", "check_inventory", 0, Vec::new()),
            result("S02", "inventory-result", "inventory", 1),
            action(
                "S02",
                "checkout",
                "checkout",
                2,
                vec!["evt-S02-inventory-result"],
            ),
            result("S02", "checkout-result", "checkout", 3),
        ],
        vec![
            produces("S02", "inventory", "inventory-result"),
            produces("S02", "checkout", "checkout-result"),
            references("S02", "checkout", "inventory-result"),
            depends_on("S02", "checkout", "inventory-result"),
        ],
    );
    let intervention = remove_events(&baseline, &["evt-S02-inventory-result"]);
    case_with(
        "S02_load_bearing_removal_failure",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S02-inventory-result"],
        "Remove the inventory result from checkout context.",
        "Checkout loses its explicit inventory result input.",
        QualityRiskCategory::High,
    )
}

fn s03_supersession_defer() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message("S03", "policy-old", "ctx_policy_v1", 0, "policy-v1"),
            message("S03", "policy-new", "ctx_policy_v2", 1, "policy-v2"),
            action(
                "S03",
                "apply-policy",
                "apply_policy",
                2,
                vec!["evt-S03-policy-new"],
            ),
            result("S03", "policy-result", "apply-policy", 3),
        ],
        vec![
            supersedes("S03", "evt-S03-policy-new", "evt-S03-policy-old"),
            protocol_precedes("S03", "evt-S03-policy-new", "apply-policy"),
        ],
    );
    let intervention = relocate_before(&baseline, "evt-S03-policy-old", "evt-S03-policy-result");
    case_with(
        "S03_explicit_supersession_deferral",
        baseline,
        intervention,
        InterventionClass::Defer,
        vec!["evt-S03-policy-old"],
        "Move explicitly superseded policy v1 behind the bounded action boundary.",
        "Current policy v2 remains before apply_policy.",
        QualityRiskCategory::LowIfBoundaryPreserved,
    )
}

fn s04_result_needed_later() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            action("S04", "create", "create_record", 0, Vec::new()),
            result("S04", "created-result", "create", 1),
            action(
                "S04",
                "update",
                "update_record",
                2,
                vec!["evt-S04-created-result"],
            ),
            result("S04", "update-result", "update", 3),
        ],
        vec![
            produces("S04", "create", "created-result"),
            produces("S04", "update", "update-result"),
            references("S04", "update", "created-result"),
            depends_on("S04", "update", "created-result"),
        ],
    );
    let intervention = remove_events(&baseline, &["evt-S04-created-result"]);
    case_with(
        "S04_action_result_needed_later",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S04-created-result"],
        "Remove the generated identifier result from the later update.",
        "The later action loses its explicit generated-identifier input.",
        QualityRiskCategory::High,
    )
}

fn s05_result_not_needed() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            action("S05", "audit", "write_audit", 0, Vec::new()),
            result("S05", "audit-result", "audit", 1),
            action("S05", "update", "update_profile", 2, Vec::new()),
            result("S05", "update-result", "update", 3),
        ],
        vec![
            produces("S05", "audit", "audit-result"),
            produces("S05", "update", "update-result"),
        ],
    );
    let intervention = remove_events(&baseline, &["evt-S05-audit-result"]);
    case_with(
        "S05_action_result_not_needed",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S05-audit-result"],
        "Remove an unreferenced audit result with no state effect.",
        "An unreferenced result is absent while task state is unchanged.",
        QualityRiskCategory::Low,
    )
}

fn s06_dependency_chain() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            action("S06", "create", "create_record", 0, Vec::new()),
            result("S06", "create-result", "create", 1),
            action(
                "S06",
                "authorize",
                "authorize_change",
                2,
                vec!["evt-S06-create-result"],
            ),
            result("S06", "authorize-result", "authorize", 3),
            action(
                "S06",
                "commit",
                "commit_change",
                4,
                vec!["evt-S06-authorize-result"],
            ),
            result("S06", "commit-result", "commit", 5),
        ],
        vec![
            produces("S06", "create", "create-result"),
            produces("S06", "authorize", "authorize-result"),
            produces("S06", "commit", "commit-result"),
            references("S06", "authorize", "create-result"),
            references("S06", "commit", "authorize-result"),
            depends_on("S06", "authorize", "create-result"),
            depends_on("S06", "commit", "authorize-result"),
            protocol_precedes("S06", "authorize", "commit"),
        ],
    );
    let intervention = remove_events(&baseline, &["evt-S06-authorize-result"]);
    case_with(
        "S06_dependency_chain_preservation",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S06-authorize-result"],
        "Remove the middle authorization result from the commit chain.",
        "The explicit commit dependency chain is broken.",
        QualityRiskCategory::High,
    )
}

fn s07_safe_relocation() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message("S07", "reference", "ctx_reference", 0, "stable-reference"),
            message("S07", "other", "ctx_other", 1, "other-context"),
            action(
                "S07",
                "execute",
                "execute_with_reference",
                2,
                vec!["evt-S07-reference"],
            ),
            result("S07", "execute-result", "execute", 3),
        ],
        vec![
            protocol_precedes("S07", "evt-S07-reference", "execute"),
            same_state("S07", "evt-S07-reference", "evt-S07-other"),
        ],
    );
    let intervention = relocate_before(&baseline, "evt-S07-reference", "evt-S07-execute");
    case_with(
        "S07_safe_context_relocation",
        baseline,
        intervention,
        InterventionClass::Relocate,
        vec!["evt-S07-reference"],
        "Move the stable reference within the safe pre-action zone.",
        "Only the target position changes; protocol order and state revision hold.",
        QualityRiskCategory::LowIfOrderConstraintsHold,
    )
}

fn s08_protocol_relocation() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message("S08", "handshake", "ctx_handshake", 0, "protocol-handshake"),
            action(
                "S08",
                "execute",
                "execute_with_handshake",
                1,
                vec!["evt-S08-handshake"],
            ),
            result("S08", "execute-result", "execute", 2),
        ],
        vec![
            protocol_precedes("S08", "evt-S08-handshake", "execute"),
            depends_on("S08", "execute", "evt-S08-handshake"),
        ],
    );
    let intervention = relocate_after(&baseline, "evt-S08-handshake", "evt-S08-execute");
    case_with(
        "S08_protocol_breaking_relocation",
        baseline,
        intervention,
        InterventionClass::Relocate,
        vec!["evt-S08-handshake"],
        "Move the handshake after the action that requires it.",
        "The explicit protocol boundary is broken.",
        QualityRiskCategory::High,
    )
}

fn s09_exact_repeat() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message(
                "S09",
                "reference-1",
                "ctx_reference_1",
                0,
                "immutable-repeat",
            ),
            message(
                "S09",
                "reference-2",
                "ctx_reference_2",
                1,
                "immutable-repeat",
            ),
            action(
                "S09",
                "execute",
                "execute_with_reference",
                2,
                vec!["evt-S09-reference-1"],
            ),
            result("S09", "execute-result", "execute", 3),
        ],
        vec![same_state(
            "S09",
            "evt-S09-reference-1",
            "evt-S09-reference-2",
        )],
    );
    let intervention = remove_events(&baseline, &["evt-S09-reference-2"]);
    case_with(
        "S09_repeated_context_removal",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S09-reference-2"],
        "Remove the exact repeated immutable context block.",
        "The first reference remains and the state predicate is unchanged.",
        QualityRiskCategory::LowIfOrderConstraintsHold,
    )
}

fn s10_load_bearing_repeat() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message(
                "S10",
                "repeat-1",
                "ctx_repeated_1",
                0,
                "similar-instruction",
            ),
            message(
                "S10",
                "repeat-2",
                "ctx_repeated_2",
                1,
                "similar-instruction",
            ),
            action(
                "S10",
                "execute",
                "execute_load_bearing_repeat",
                2,
                vec!["evt-S10-repeat-2"],
            ),
            result("S10", "execute-result", "execute", 3),
        ],
        vec![references("S10", "execute", "evt-S10-repeat-2")],
    );
    let intervention = remove_events(&baseline, &["evt-S10-repeat-2"]);
    case_with(
        "S10_repeated_but_load_bearing",
        baseline,
        intervention,
        InterventionClass::Remove,
        vec!["evt-S10-repeat-2"],
        "Remove superficially repeated context that has a later explicit reference.",
        "The later operation loses its load-bearing reference.",
        QualityRiskCategory::High,
    )
}

fn s11_noop_control() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            action("S11", "minimal", "minimal_task", 0, Vec::new()),
            result("S11", "minimal-result", "minimal", 1),
        ],
        vec![produces("S11", "minimal", "minimal-result")],
    );
    let intervention = baseline.clone();
    case_with(
        "S11_already_efficient_noop",
        baseline,
        intervention,
        InterventionClass::NoChange,
        vec!["evt-S11-minimal"],
        "No transformation; retain the already-efficient context.",
        "No structural change and no claimed savings.",
        QualityRiskCategory::None,
    )
}

fn s12_ambiguous_control() -> Result<ControlledCase, BenchmarkError> {
    let baseline = input(
        vec![
            message(
                "S12",
                "observation-1",
                "ctx_observation_1",
                0,
                "ambiguous-repeat",
            ),
            message(
                "S12",
                "observation-2",
                "ctx_observation_2",
                1,
                "ambiguous-repeat",
            ),
            action("S12", "finish", "finish_ambiguous_task", 2, Vec::new()),
            result("S12", "finish-result", "finish", 3),
        ],
        vec![produces("S12", "finish", "finish-result")],
    );
    let intervention = baseline.clone();
    case_with(
        "S12_ambiguous_evidence",
        baseline,
        intervention,
        InterventionClass::NoChange,
        vec!["evt-S12-observation-2"],
        "No mutation is admitted while dependency/removability evidence is absent.",
        "No planner-visible change; ambiguity remains absent evidence.",
        QualityRiskCategory::Unknown,
    )
}

#[allow(clippy::too_many_arguments)]
fn case_with(
    scenario_id: &str,
    baseline_input: PlannerInput,
    intervention_input: PlannerInput,
    intervention_class: InterventionClass,
    target_event_ids: Vec<&str>,
    exact_transformation: &str,
    expected_structural_effect: &str,
    expected_quality_risk_category: QualityRiskCategory,
) -> Result<ControlledCase, BenchmarkError> {
    let baseline_trace_id = format!("{scenario_id}:baseline");
    let intervention_is_control = intervention_class == InterventionClass::NoChange;
    let intervention_trace_id = format!(
        "{scenario_id}:{}",
        if intervention_is_control {
            "control"
        } else {
            "variant"
        }
    );
    let manifest = InterventionManifest {
        manifest_id: format!("{scenario_id}:manifest"),
        baseline_trace_id: baseline_trace_id.clone(),
        variant_trace_id: intervention_trace_id.clone(),
        target_event_ids: target_event_ids.into_iter().map(str::to_string).collect(),
        intervention_class,
        exact_transformation: exact_transformation.to_string(),
        reason: format!("bounded self-authored case {scenario_id}"),
        planner_visibility: PlannerVisibility::EvaluationOnly,
        expected_structural_effect: expected_structural_effect.to_string(),
        expected_quality_risk_category,
    };
    let scenario = scenario_identity(scenario_id);
    let baseline = envelope(
        scenario.clone(),
        baseline_trace_id.clone(),
        VariantRole::Baseline,
        baseline_trace_id.clone(),
        baseline_input,
        manifest.clone(),
    );
    let intervention = envelope(
        scenario,
        intervention_trace_id.clone(),
        if intervention_is_control {
            VariantRole::Control
        } else {
            VariantRole::Variant
        },
        baseline_trace_id,
        intervention_input,
        manifest.clone(),
    );
    let manifest_hash = manifest_hash(&manifest)?;
    let case = ControlledCase {
        scenario_id: scenario_id.to_string(),
        baseline,
        intervention,
        manifest,
        manifest_hash,
    };
    validate_case(&case)?;
    Ok(case)
}

fn envelope(
    scenario: ScenarioIdentity,
    trace_id: String,
    variant_role: VariantRole,
    baseline_trace_id: String,
    planner_input: PlannerInput,
    manifest: InterventionManifest,
) -> ControlledEnvelope {
    let quality_evaluation_id = format!("{}:quality", scenario.scenario_id);
    ControlledEnvelope {
        schema_id: SCHEMA_ID.to_string(),
        schema_version: SCHEMA_VERSION,
        benchmark_id: BENCHMARK_ID.to_string(),
        scenario,
        trace: TraceEnvelope {
            trace_id,
            variant_role,
            baseline_trace_id,
            planner_input,
        },
        evaluation_sidecar: EvaluationSidecar {
            intervention_manifest_ref: manifest.manifest_id.clone(),
            intervention_manifest: manifest,
            quality_evaluation_ids: vec![quality_evaluation_id],
            oracle_result: None,
            evaluation_content_hash: None,
        },
    }
}

fn scenario_identity(scenario_id: &str) -> ScenarioIdentity {
    let seed = scenario_id.as_bytes().iter().fold(0u32, |value, byte| {
        value.wrapping_mul(33).wrapping_add(*byte as u32)
    });
    ScenarioIdentity {
        scenario_id: scenario_id.to_string(),
        scenario_version: "v1".to_string(),
        task_revision: TASK_REVISION.to_string(),
        environment_revision: ENVIRONMENT_REVISION.to_string(),
        initial_state_id: format!("{scenario_id}:initial"),
        fixed_seed: seed,
        provenance: vec![SourceProvenance {
            source_kind: SourceKind::SelfAuthored,
            classification: EvidenceClass::CapturedExplicit,
            source_locator: Some(format!("seed/{scenario_id}")),
            source_revision: Some("self-authored-seed-v1".to_string()),
            content_hash: Some(hash_text(scenario_id)),
            note: None,
        }],
    }
}

fn input(events: Vec<Event>, relations: Vec<Relation>) -> PlannerInput {
    PlannerInput {
        events: renumber(events),
        relations,
        provenance: vec![SourceProvenance {
            source_kind: SourceKind::SelfAuthored,
            classification: EvidenceClass::CapturedExplicit,
            source_locator: Some("self-authored/planner-input".to_string()),
            source_revision: Some("self-authored-seed-v1".to_string()),
            content_hash: Some(hash_text("self-authored/planner-input")),
            note: None,
        }],
    }
}

fn message(
    scenario: &str,
    suffix: &str,
    context_id: &str,
    sequence_index: u32,
    content_label: &str,
) -> Event {
    event(
        scenario,
        format!("evt-{scenario}-{suffix}"),
        sequence_index,
        EventType::Message,
        None,
        None,
        Vec::new(),
        Some(context_id.to_string()),
        Some(content_label),
    )
}

fn action(
    scenario: &str,
    suffix: &str,
    tool_name: &str,
    sequence_index: u32,
    references: Vec<&str>,
) -> Event {
    event(
        scenario,
        format!("evt-{scenario}-{suffix}"),
        sequence_index,
        EventType::Action,
        Some(ActionIdentity {
            action_id: suffix.to_string(),
            tool_name: tool_name.to_string(),
            argument_hash: Some(hash_text(&format!("args:{scenario}:{suffix}"))),
        }),
        None,
        references.into_iter().map(str::to_string).collect(),
        None,
        None,
    )
}

fn result(scenario: &str, suffix: &str, originating_action_id: &str, sequence_index: u32) -> Event {
    event(
        scenario,
        format!("evt-{scenario}-{suffix}"),
        sequence_index,
        EventType::Result,
        None,
        Some(ResultIdentity {
            result_id: suffix.to_string(),
            originating_action_id: originating_action_id.to_string(),
            observation_hash: Some(hash_text(&format!("observation:{scenario}:{suffix}"))),
            status: Some(ResultStatus::Success),
        }),
        Vec::new(),
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn event(
    scenario: &str,
    event_id: String,
    sequence_index: u32,
    event_type: EventType,
    action_identity: Option<ActionIdentity>,
    result_identity: Option<ResultIdentity>,
    reference_event_ids: Vec<String>,
    context_block_id: Option<String>,
    content_label: Option<&str>,
) -> Event {
    let label = content_label.unwrap_or(&event_id);
    Event {
        event_id: event_id.clone(),
        sequence_index,
        event_type,
        actor_role: match event_type {
            EventType::Message => ActorRole::User,
            EventType::Action => ActorRole::Agent,
            EventType::Result | EventType::Observation => ActorRole::Tool,
            EventType::StateSnapshot => ActorRole::Environment,
            EventType::Assertion => ActorRole::Evaluator,
        },
        parent_event_ids: Vec::new(),
        reference_event_ids,
        action: action_identity,
        result: result_identity,
        context_block_id,
        world_state_revision: Some(format!("{scenario}:state-{}", sequence_index)),
        order: Some(OrderMetadata {
            logical_tick: Some(sequence_index),
            source_timestamp: None,
            timestamp_origin: Some(TimestampOrigin::DerivedStructural),
        }),
        content_hash: Some(hash_text(label)),
        provenance: vec![SourceProvenance {
            source_kind: SourceKind::SelfAuthored,
            classification: EvidenceClass::CapturedExplicit,
            source_locator: Some(format!("seed/{scenario}/{event_id}")),
            source_revision: Some("self-authored-seed-v1".to_string()),
            content_hash: Some(hash_text(&event_id)),
            note: None,
        }],
    }
}

fn produces(scenario: &str, action_id: &str, result_id: &str) -> Relation {
    relation(scenario, RelationType::Produces, action_id, result_id)
}

fn references(scenario: &str, action_id: &str, result_id: &str) -> Relation {
    relation(scenario, RelationType::References, action_id, result_id)
}

fn depends_on(scenario: &str, action_id: &str, result_id: &str) -> Relation {
    relation(scenario, RelationType::DependsOn, action_id, result_id)
}

fn supersedes(scenario: &str, newer: &str, older: &str) -> Relation {
    relation(scenario, RelationType::Supersedes, newer, older)
}

fn protocol_precedes(scenario: &str, before: &str, after: &str) -> Relation {
    relation(scenario, RelationType::ProtocolPrecedes, before, after)
}

fn same_state(scenario: &str, left: &str, right: &str) -> Relation {
    relation(scenario, RelationType::SameStateRevision, left, right)
}

fn relation(scenario: &str, relation_type: RelationType, from_id: &str, to_id: &str) -> Relation {
    Relation {
        relation_id: format!(
            "rel-{scenario}-{}-{from_id}-{to_id}",
            serde_json::to_string(&relation_type)
                .expect("relation type serializes")
                .trim_matches('"')
        ),
        relation_type,
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        scope: "scenario_local".to_string(),
        semantics_version: Some(RELATION_SEMANTICS_VERSION.to_string()),
        provenance: vec![SourceProvenance {
            source_kind: SourceKind::SelfAuthored,
            classification: EvidenceClass::CapturedExplicit,
            source_locator: Some(format!("seed/{scenario}/relation")),
            source_revision: Some("self-authored-seed-v1".to_string()),
            content_hash: Some(hash_text(&format!("{from_id}:{to_id}"))),
            note: None,
        }],
    }
}

fn remove_events(input: &PlannerInput, targets: &[&str]) -> PlannerInput {
    let target_set = targets
        .iter()
        .map(|target| (*target).to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let mut result = input.clone();
    let mut target_aliases = target_set.clone();
    for event in &result.events {
        if target_set.contains(event.event_id.as_str()) {
            if let Some(context_id) = &event.context_block_id {
                target_aliases.insert(context_id.clone());
            }
            if let Some(action) = &event.action {
                target_aliases.insert(action.action_id.clone());
            }
            if let Some(result_identity) = &event.result {
                target_aliases.insert(result_identity.result_id.clone());
            }
        }
    }
    result
        .events
        .retain(|event| !target_set.contains(event.event_id.as_str()));
    for event in &mut result.events {
        event
            .parent_event_ids
            .retain(|id| !target_set.contains(id.as_str()));
        event
            .reference_event_ids
            .retain(|id| !target_set.contains(id.as_str()));
    }
    result.relations.retain(|relation| {
        !target_aliases.contains(relation.from_id.as_str())
            && !target_aliases.contains(relation.to_id.as_str())
    });
    renumber_input(result)
}

fn relocate_before(input: &PlannerInput, target: &str, before: &str) -> PlannerInput {
    let mut result = input.clone();
    let target_index = result
        .events
        .iter()
        .position(|event| event.event_id == target)
        .expect("fixture target exists");
    let event = result.events.remove(target_index);
    let before_index = result
        .events
        .iter()
        .position(|candidate| candidate.event_id == before)
        .expect("fixture insertion point exists");
    result.events.insert(before_index, event);
    renumber_input(result)
}

fn relocate_after(input: &PlannerInput, target: &str, after: &str) -> PlannerInput {
    let mut result = input.clone();
    let target_index = result
        .events
        .iter()
        .position(|event| event.event_id == target)
        .expect("fixture target exists");
    let event = result.events.remove(target_index);
    let after_index = result
        .events
        .iter()
        .position(|candidate| candidate.event_id == after)
        .expect("fixture insertion point exists");
    result.events.insert(after_index + 1, event);
    renumber_input(result)
}

fn renumber_input(mut input: PlannerInput) -> PlannerInput {
    input.events = renumber(input.events);
    input
}

fn renumber(mut events: Vec<Event>) -> Vec<Event> {
    for (index, event) in events.iter_mut().enumerate() {
        event.sequence_index = index as u32;
        if let Some(order) = &mut event.order {
            order.logical_tick = Some(index as u32);
        }
    }
    events
}

#[allow(dead_code)]
pub fn seed_source_hash() -> String {
    let mut ids = build_seed()
        .expect("approved self-authored seed must build")
        .into_iter()
        .map(|case| case.scenario_id)
        .collect::<Vec<_>>();
    ids.sort();
    sha256_hex(ids.join("\n").as_bytes())
}
