use benchmark::{
    canonical_envelope_json, canonical_report_json, evaluate_case, evaluate_envelopes,
    load_envelope, project_planner_evidence, run_benchmark, run_frozen_planner, ActionIdentity,
    EventType, ExecutionStatus, OracleResult, OrderMetadata,
};
use prefixity_controlled_benchmark as benchmark;

fn cases() -> Vec<benchmark::ControlledCase> {
    benchmark::build_seed().unwrap()
}

fn case(id: &str) -> benchmark::ControlledCase {
    cases()
        .into_iter()
        .find(|case| case.scenario_id == id)
        .unwrap()
}

#[test]
fn all_twelve_self_authored_fixtures_load_and_round_trip() {
    let seed = cases();
    assert_eq!(seed.len(), 12);
    for case in seed {
        for envelope in [&case.baseline, &case.intervention] {
            let encoded = canonical_envelope_json(envelope).unwrap();
            let decoded = load_envelope(&encoded).unwrap();
            assert_eq!(&decoded, envelope);
            assert_eq!(canonical_envelope_json(&decoded).unwrap(), encoded);
        }
    }
}

#[test]
fn manifest_hashes_and_report_are_stable() {
    let first = run_benchmark().unwrap();
    let second = run_benchmark().unwrap();
    assert_eq!(first, second);
    assert_eq!(
        canonical_report_json(&first).unwrap(),
        canonical_report_json(&second).unwrap()
    );
    assert_eq!(first.manifest_hashes.len(), 12);
    assert_eq!(first.aggregate_counts.pass, 7);
    assert_eq!(first.aggregate_counts.fail, 5);
    assert_eq!(first.aggregate_counts.invalid_baseline, 0);
    assert_eq!(first.aggregate_counts.inconclusive, 0);
    assert_eq!(first.baseline_count, 12);
    assert_eq!(first.variant_count, 10);
    assert_eq!(first.control_count, 2);
    assert!(!first.aggregate_hash.is_empty());
}

#[test]
fn malformed_schema_id_hash_and_order_are_rejected() {
    let baseline = case("S01_irrelevant_context_removal").baseline;

    let mut bad_schema = baseline.clone();
    bad_schema.schema_id = "wrong-schema".to_string();
    assert!(benchmark::validate_envelope(&bad_schema).is_err());

    let mut bad_hash = baseline.clone();
    bad_hash.trace.planner_input.events[0].content_hash = Some("not-a-hash".to_string());
    assert!(benchmark::validate_envelope(&bad_hash).is_err());

    let mut bad_order = baseline;
    bad_order.trace.planner_input.events[0].sequence_index = 9;
    assert!(benchmark::validate_envelope(&bad_order).is_err());
}

#[test]
fn malformed_pair_identity_and_environment_mismatch_are_rejected() {
    let original = case("S01_irrelevant_context_removal");

    let mut wrong_baseline = original.clone();
    wrong_baseline.intervention.trace.baseline_trace_id = "other:baseline".to_string();
    assert!(benchmark::validate_case(&wrong_baseline).is_err());

    let mut wrong_environment = original;
    wrong_environment.intervention.scenario.environment_revision = "other-world-v1".to_string();
    assert!(benchmark::validate_case(&wrong_environment).is_err());
}

#[test]
fn undeclared_collateral_fixture_change_is_rejected() {
    let mut changed = case("S01_irrelevant_context_removal");
    let mut extra = changed.intervention.trace.planner_input.events[0].clone();
    extra.event_id = "evt-S01-undeclared-extra".to_string();
    extra.context_block_id = Some("ctx-undeclared-extra".to_string());
    extra.sequence_index = changed.intervention.trace.planner_input.events.len() as u32;
    extra.order = Some(OrderMetadata {
        logical_tick: Some(extra.sequence_index),
        source_timestamp: None,
        timestamp_origin: Some(benchmark::TimestampOrigin::DerivedStructural),
    });
    changed.intervention.trace.planner_input.events.push(extra);
    assert!(benchmark::validate_case(&changed).is_err());
}

#[test]
fn action_result_and_authored_relations_round_trip() {
    let fixture = case("S06_dependency_chain_preservation");
    let action_event = fixture
        .baseline
        .trace
        .planner_input
        .events
        .iter()
        .find(|event| {
            event
                .action
                .as_ref()
                .is_some_and(|action| action.action_id == "commit")
        })
        .unwrap();
    assert_eq!(
        action_event.action.as_ref().unwrap().tool_name,
        "commit_change"
    );
    assert!(action_event
        .reference_event_ids
        .contains(&"evt-S06-authorize-result".to_string()));
    let result_event = fixture
        .baseline
        .trace
        .planner_input
        .events
        .iter()
        .find(|event| {
            event
                .result
                .as_ref()
                .is_some_and(|result| result.result_id == "authorize-result")
        })
        .unwrap();
    assert_eq!(
        result_event.result.as_ref().unwrap().originating_action_id,
        "authorize"
    );
    assert!(fixture
        .baseline
        .trace
        .planner_input
        .relations
        .iter()
        .any(|relation| relation.relation_type == benchmark::RelationType::DependsOn));
}

#[test]
fn timestamp_order_does_not_create_staleness_and_repetition_does_not_create_removability() {
    let mut fixture = case("S09_repeated_context_removal");
    fixture.baseline.trace.planner_input.events[0]
        .order
        .as_mut()
        .unwrap()
        .source_timestamp = Some("2000-01-01T00:00:00Z".to_string());
    fixture.baseline.trace.planner_input.events[0]
        .order
        .as_mut()
        .unwrap()
        .timestamp_origin = Some(benchmark::TimestampOrigin::SourceExplicit);
    benchmark::validate_envelope(&fixture.baseline).unwrap();
    let projected = project_planner_evidence(&fixture.baseline).unwrap();
    assert!(projected.trace.planner_input.events[0].order.is_some());
    assert!(!projected.request_trace.blocks[0].stale);
    assert!(!projected.request_trace.blocks[1].optional);
    assert!(!projected.request_trace.blocks[1].stale);
    assert_eq!(
        fixture.manifest.planner_visibility,
        benchmark::PlannerVisibility::EvaluationOnly
    );
}

#[test]
fn evaluation_sidecar_cannot_enter_planner_evidence() {
    let fixture = case("S02_load_bearing_removal_failure");
    let evidence = project_planner_evidence(&fixture.baseline).unwrap();
    let encoded = serde_json::to_string(&evidence).unwrap();
    assert!(!encoded.contains("evaluation_sidecar"));
    assert!(!encoded.contains("oracle_result"));
    assert!(!encoded.contains("expected_quality_risk_category"));
    assert!(!encoded.contains("intervention_manifest"));
}

#[test]
fn planner_projection_and_frozen_planner_are_stable_and_conservative() {
    let fixture = case("S07_safe_context_relocation");
    let first = project_planner_evidence(&fixture.baseline).unwrap();
    let second = project_planner_evidence(&fixture.baseline).unwrap();
    assert_eq!(first, second);
    let first_plan = run_frozen_planner(&first).unwrap();
    let second_plan = run_frozen_planner(&second).unwrap();
    assert_eq!(first_plan, second_plan);
    assert_eq!(first_plan.classes, vec!["DO_NOTHING"]);

    let report = run_benchmark().unwrap();
    assert_eq!(report.planner_runs.len(), 24);
    assert!(report
        .planner_runs
        .iter()
        .all(|run| run.classes == vec!["DO_NOTHING"]));
}

#[test]
fn oracle_reproduces_deterministic_pass_and_fail_cases() {
    let pass = evaluate_case(&case("S01_irrelevant_context_removal")).unwrap();
    assert_eq!(pass.result, OracleResult::Pass);
    let fail = evaluate_case(&case("S02_load_bearing_removal_failure")).unwrap();
    assert_eq!(fail.result, OracleResult::Fail);
    assert!(!fail.collateral_state_keys.is_empty());
}

#[test]
fn oracle_reports_invalid_baseline_and_inconclusive_execution() {
    let original = case("S01_irrelevant_context_removal");

    let mut invalid_baseline = original.baseline.clone();
    let action = invalid_baseline
        .trace
        .planner_input
        .events
        .iter_mut()
        .find(|event| event.event_type == EventType::Action)
        .unwrap();
    action.action = Some(ActionIdentity {
        action_id: "update".to_string(),
        tool_name: "checkout".to_string(),
        argument_hash: action.action.as_ref().unwrap().argument_hash.clone(),
    });
    let invalid = evaluate_envelopes(
        &invalid_baseline,
        &original.intervention,
        &original.manifest,
    )
    .unwrap();
    assert_eq!(invalid.result, OracleResult::InvalidBaseline);

    let mut unresolved_baseline = original.baseline.clone();
    let action = unresolved_baseline
        .trace
        .planner_input
        .events
        .iter_mut()
        .find(|event| event.event_type == EventType::Action)
        .unwrap();
    action.action = Some(ActionIdentity {
        action_id: "update".to_string(),
        tool_name: "unknown_scripted_tool".to_string(),
        argument_hash: action.action.as_ref().unwrap().argument_hash.clone(),
    });
    let unresolved = evaluate_envelopes(
        &unresolved_baseline,
        &original.intervention,
        &original.manifest,
    )
    .unwrap();
    assert_eq!(unresolved.result, OracleResult::Inconclusive);
}

#[test]
fn collateral_state_mutation_prevents_pass() {
    let original = case("S01_irrelevant_context_removal");
    let mut changed = original.intervention.clone();
    let sequence = changed.trace.planner_input.events.len() as u32;
    changed.trace.planner_input.events.push(benchmark::Event {
        event_id: "evt-S01-extra-action".to_string(),
        sequence_index: sequence,
        event_type: EventType::Action,
        actor_role: benchmark::ActorRole::Agent,
        parent_event_ids: Vec::new(),
        reference_event_ids: Vec::new(),
        action: Some(ActionIdentity {
            action_id: "extra".to_string(),
            tool_name: "minimal_task".to_string(),
            argument_hash: Some("0".repeat(64)),
        }),
        result: None,
        context_block_id: None,
        world_state_revision: Some("S01:extra".to_string()),
        order: Some(OrderMetadata {
            logical_tick: Some(sequence),
            source_timestamp: None,
            timestamp_origin: Some(benchmark::TimestampOrigin::DerivedStructural),
        }),
        content_hash: Some("1".repeat(64)),
        provenance: original.baseline.trace.planner_input.events[0]
            .provenance
            .clone(),
    });
    let result = evaluate_envelopes(&original.baseline, &changed, &original.manifest).unwrap();
    assert_eq!(result.result, OracleResult::Fail);
    assert!(result
        .collateral_state_keys
        .iter()
        .any(|key| key == "minimal_task"));
}

#[test]
fn source_fixtures_are_not_mutated_by_evaluation_or_planner() {
    let before = cases();
    let before_json = before
        .iter()
        .map(|case| canonical_envelope_json(&case.baseline).unwrap())
        .collect::<Vec<_>>();
    let _ = run_benchmark().unwrap();
    for (case, expected) in before.iter().zip(before_json) {
        assert_eq!(canonical_envelope_json(&case.baseline).unwrap(), expected);
    }
}

#[test]
fn world_execution_is_deterministic_and_has_no_network_surface() {
    let fixture = case("S03_explicit_supersession_deferral");
    let world = benchmark::ScriptedWorld;
    let first = world.execute(&fixture.baseline).unwrap();
    let second = world.execute(&fixture.baseline).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.status, ExecutionStatus::Complete);
    assert!(first.final_state_hash.is_some());
}

#[test]
fn world_executes_actions_in_validated_trace_order() {
    let fixture = case("S06_dependency_chain_preservation");
    let execution = benchmark::ScriptedWorld.execute(&fixture.baseline).unwrap();
    assert_eq!(
        execution.executed_action_ids,
        vec!["create", "authorize", "commit"]
    );
}

#[test]
fn world_does_not_consume_a_future_result_before_its_producer_runs() {
    let fixture = case("S06_dependency_chain_preservation");
    let execution = benchmark::ScriptedWorld.execute(&fixture.baseline).unwrap();
    let create = execution
        .executed_action_ids
        .iter()
        .position(|action| action == "create")
        .unwrap();
    let authorize = execution
        .executed_action_ids
        .iter()
        .position(|action| action == "authorize")
        .unwrap();
    let commit = execution
        .executed_action_ids
        .iter()
        .position(|action| action == "commit")
        .unwrap();
    assert!(create < authorize && authorize < commit);
}
