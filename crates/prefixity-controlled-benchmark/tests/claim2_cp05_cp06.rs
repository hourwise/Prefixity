use prefixity_controlled_benchmark::{
    compare_paired_arms, evaluate_claim2_arm, load_claim2_case, project_claim2_trace,
    render_claim2_request, select_claim2_decision, validate_claim2_case, Claim2ArmState,
    Claim2AssetKind, Claim2CaseKind, Claim2ProjectionMode, Claim2SlotStatus,
    Claim2TokenCountStatus, ResearchInterventionClass,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn fixture_root(case_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/claim2")
        .join(case_id.to_ascii_lowercase())
}

fn load(case_id: &str) -> prefixity_controlled_benchmark::LoadedClaim2Case {
    load_claim2_case(&fixture_root(case_id).join("case.json")).unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn asset_bytes(case_id: &str, asset_id: &str) -> Vec<u8> {
    let root = fixture_root(case_id);
    let case = load(case_id);
    let asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == asset_id)
        .unwrap();
    let bytes = fs::read(root.join(&asset.relative_path)).unwrap();
    assert_eq!(sha256(&bytes), asset.sha256);
    bytes
}

fn event(case: &prefixity_controlled_benchmark::LoadedClaim2Case, id: &str) -> Value {
    let event = case
        .manifest()
        .planner_input
        .events
        .iter()
        .find(|event| event.event_id == id)
        .unwrap();
    serde_json::to_value(event).unwrap()
}

fn expected_answer(case_id: &str) -> Value {
    match case_id {
        "CP05" => json!({
            "comparison": "same_bytes_different_state",
            "initial_occurrence_id": "e-catalog-initial",
            "initial_source_revision": "route-catalog-export@R17",
            "initial_world_state_revision": "route-registry@R17",
            "later_occurrence_id": "e-catalog-after-reindex",
            "later_source_revision": "route-catalog-export@R18",
            "later_world_state_revision": "route-registry@R18",
            "operation_id": "rebuild-route-index-064",
            "changed_state_field": "route_index_generation",
            "before": "71",
            "after": "72",
            "should_prune_later_occurrence": false
        }),
        "CP06" => json!({
            "comparison": "same_state_protected_consumer",
            "original_occurrence_id": "e-policy-original",
            "repeated_occurrence_id": "e-audit-repeat",
            "source_revision": "access-policy-export@S84",
            "world_state_revision": "policy-index@S84",
            "receipt_id": "AUD-2081",
            "receipt_consumes_occurrence_id": "e-audit-repeat",
            "should_prune_repeated_occurrence": false
        }),
        _ => panic!("unexpected control case: {case_id}"),
    }
}

fn action_ids(case_id: &str) -> [&'static str; 2] {
    match case_id {
        "CP05" => ["inspect_initial_catalog", "rebuild_route_index_and_export"],
        "CP06" => ["inspect_policy_index", "verify_audit_receipt"],
        _ => panic!("unexpected control case: {case_id}"),
    }
}

fn run_synthetic_arm(
    case: &prefixity_controlled_benchmark::LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    answer: Value,
) -> (
    Claim2ArmState,
    prefixity_controlled_benchmark::Claim2Evaluation,
) {
    let mut arm = Claim2ArmState::new(case, mode);
    let actions = action_ids(case.case_id());
    let outputs = [
        json!({ "action_id": actions[0] }).to_string(),
        json!({ "action_id": actions[1] }).to_string(),
        json!({ "answer": answer }).to_string(),
    ];
    for (index, output) in outputs.into_iter().enumerate() {
        let rendered = render_claim2_request(case, &mut arm).unwrap();
        assert_eq!(rendered.request_slot, index as u8 + 1);
        assert!(!rendered.dispatchable);
        let evaluation = arm.record_output(case, output).unwrap();
        if index < 2 {
            assert!(evaluation.is_none());
        } else {
            assert_eq!(evaluation.unwrap().status, Claim2SlotStatus::Pass);
        }
    }
    let evaluation = evaluate_claim2_arm(case, &arm).unwrap();
    assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
    (arm, evaluation)
}

fn run_bad_final(
    case: &prefixity_controlled_benchmark::LoadedClaim2Case,
    mut answer: Value,
    field: &str,
    replacement: Value,
) -> prefixity_controlled_benchmark::Claim2Evaluation {
    answer[field] = replacement;
    let mut arm = Claim2ArmState::new(case, Claim2ProjectionMode::Baseline);
    let actions = action_ids(case.case_id());
    for action in actions {
        render_claim2_request(case, &mut arm).unwrap();
        assert!(arm
            .record_output(case, json!({ "action_id": action }).to_string())
            .unwrap()
            .is_none());
    }
    render_claim2_request(case, &mut arm).unwrap();
    let failed = arm
        .record_output(case, json!({ "answer": answer }).to_string())
        .unwrap()
        .unwrap();
    assert_eq!(arm.slots()[2].status, Claim2SlotStatus::Fail);
    assert!(!failed.final_answer_passed);
    failed
}

#[test]
fn both_controls_use_the_actual_frozen_policy_and_preserve_all_occurrences() {
    for case_id in ["CP05", "CP06"] {
        let case = load(case_id);
        validate_claim2_case(&case).unwrap();
        assert_eq!(case.manifest().kind, Claim2CaseKind::Control);
        assert_eq!(case.manifest().case_id, case_id);
        assert!(case.manifest().token_proof_inputs.is_none());

        let selected = select_claim2_decision(&case).unwrap();
        assert_eq!(
            selected.decision.class,
            ResearchInterventionClass::DoNothing
        );
        assert_eq!(selected.decision.target_event_id, None);
        assert_eq!(selected.decision.rule, "FAIL_OPEN");
        assert!(selected.candidates.exact_duplicate_prune.is_empty());
        assert!(selected.candidates.explicit_supersession_defer.is_empty());
        assert!(selected.candidates.same_zone_protocol_relocate.is_empty());

        let baseline = project_claim2_trace(&case, Claim2ProjectionMode::Baseline).unwrap();
        let noop = project_claim2_trace(&case, Claim2ProjectionMode::NoOp).unwrap();
        let intervention = project_claim2_trace(&case, Claim2ProjectionMode::Intervention).unwrap();
        assert_eq!(baseline, case.manifest().planner_input);
        assert_eq!(noop, baseline);
        assert_eq!(intervention, baseline);

        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            let arm = Claim2ArmState::new(&case, mode);
            for slot in 1..=3 {
                let rendered = arm.preview_request(&case, slot).unwrap();
                assert_eq!(rendered.request_slot, slot);
                assert!(!rendered.dispatchable);
                assert_eq!(rendered.metrics.omitted_attachment_utf8_bytes, 0);
                assert_eq!(
                    rendered.metrics.token_count_status,
                    Claim2TokenCountStatus::ExactTokenizationRequired
                );
                if slot == 2 {
                    assert_eq!(rendered.metrics.unbound_raw_assistant_slots, vec![1]);
                    assert_eq!(rendered.metrics.unbound_environment_receipt_slots, vec![1]);
                    assert_eq!(rendered.metrics.raw_prior_output_planning_utf8_bytes, 4096);
                } else if slot == 3 {
                    assert_eq!(rendered.metrics.unbound_raw_assistant_slots, vec![1, 2]);
                    assert_eq!(
                        rendered.metrics.unbound_environment_receipt_slots,
                        vec![1, 2]
                    );
                    assert_eq!(rendered.metrics.raw_prior_output_planning_utf8_bytes, 8192);
                }
            }
        }
    }
}

#[test]
fn cp05_pins_equal_catalog_bytes_at_distinct_source_and_world_states() {
    let case = load("CP05");
    let initial_asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "catalog_initial")
        .unwrap();
    let later_asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "catalog_after")
        .unwrap();
    let initial = asset_bytes("CP05", "catalog_initial");
    let later = asset_bytes("CP05", "catalog_after");
    assert_eq!(initial, later);
    assert_eq!(initial_asset.sha256, later_asset.sha256);
    assert_eq!(initial_asset.event_id.as_deref(), Some("e-catalog-initial"));
    assert_eq!(
        later_asset.event_id.as_deref(),
        Some("e-catalog-after-reindex")
    );
    assert_ne!(initial_asset.revision_id, later_asset.revision_id);
    assert_eq!(
        event(&case, "e-catalog-initial")["content_hash"],
        event(&case, "e-catalog-after-reindex")["content_hash"]
    );
    assert_eq!(
        event(&case, "e-catalog-initial")["world_state_revision"],
        "route-registry@R17"
    );
    assert_eq!(
        event(&case, "e-catalog-after-reindex")["world_state_revision"],
        "route-registry@R18"
    );
    assert!(!case
        .manifest()
        .planner_input
        .relations
        .iter()
        .any(|relation| {
            serde_json::to_value(relation).unwrap()["relation_type"] == "same_state_revision"
        }));

    let root = fixture_root("CP05");
    let before_bytes = fs::read(root.join("world/state-before.json")).unwrap();
    let after_bytes = fs::read(root.join("world/state-after.json")).unwrap();
    let before: Value = serde_json::from_slice(&before_bytes).unwrap();
    let after: Value = serde_json::from_slice(&after_bytes).unwrap();
    assert_eq!(before["catalog_artifact_sha256"], initial_asset.sha256);
    assert_eq!(after["catalog_artifact_sha256"], later_asset.sha256);
    assert_eq!(
        before["source_revision"].as_str(),
        initial_asset.revision_id.as_deref()
    );
    assert_eq!(
        after["source_revision"].as_str(),
        later_asset.revision_id.as_deref()
    );
    assert_eq!(
        before["world_state_revision"],
        event(&case, "e-catalog-initial")["world_state_revision"]
    );
    assert_eq!(
        after["world_state_revision"],
        event(&case, "e-catalog-after-reindex")["world_state_revision"]
    );
    assert_eq!(before["route_index_generation"], 71);
    assert_eq!(after["route_index_generation"], 72);
    assert_eq!(
        after["last_route_catalog_operation"],
        "rebuild-route-index-064"
    );
    assert_eq!(
        event(&case, "e-catalog-initial")["provenance"][0]["content_hash"],
        sha256(&before_bytes)
    );
    assert_eq!(
        event(&case, "e-catalog-after-reindex")["provenance"][0]["content_hash"],
        sha256(&after_bytes)
    );
    assert_eq!(
        case.manifest()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "catalog_initial")
            .unwrap()
            .kind,
        Claim2AssetKind::ContextAttachment
    );
    assert_eq!(
        case.manifest()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "receipt_reindex")
            .unwrap()
            .kind,
        Claim2AssetKind::EnvironmentReceipt
    );
    let evaluation_key: Value =
        serde_json::from_slice(&fs::read(root.join("evaluation/key.json")).unwrap()).unwrap();
    let required: BTreeSet<_> = evaluation_key["required_event_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    assert!(required.contains("e-catalog-initial"));
    assert!(required.contains("e-catalog-after-reindex"));
}

#[test]
fn cp06_pins_same_state_repeat_consumed_by_the_audit_receipt() {
    let case = load("CP06");
    let original_asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "policy_original")
        .unwrap();
    let repeated_asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "policy_repeat")
        .unwrap();
    let original = asset_bytes("CP06", "policy_original");
    let repeated = asset_bytes("CP06", "policy_repeat");
    assert_eq!(original, repeated);
    assert_eq!(original_asset.sha256, repeated_asset.sha256);
    assert_eq!(original_asset.revision_id, repeated_asset.revision_id);
    assert_eq!(
        event(&case, "e-policy-original")["world_state_revision"],
        event(&case, "e-audit-repeat")["world_state_revision"]
    );
    let relation = case
        .manifest()
        .planner_input
        .relations
        .iter()
        .find(|relation| relation.relation_id == "same-state-policy-export")
        .unwrap();
    assert_eq!(
        serde_json::to_value(relation).unwrap()["relation_type"],
        "same_state_revision"
    );
    assert_eq!(relation.from_id, "e-policy-original");
    assert_eq!(relation.to_id, "e-audit-repeat");

    let audit_result = event(&case, "e-audit-result");
    assert_eq!(
        audit_result["reference_event_ids"],
        json!(["e-audit-repeat"])
    );
    assert_eq!(
        audit_result["result"]["observation_hash"],
        case.manifest()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "receipt_audit")
            .unwrap()
            .sha256
    );
    let dependency = case
        .manifest()
        .planner_input
        .relations
        .iter()
        .find(|relation| relation.relation_id == "audit-receipt-consumes-repeat")
        .unwrap();
    assert_eq!(
        serde_json::to_value(dependency).unwrap()["relation_type"],
        "depends_on"
    );
    assert_eq!(dependency.from_id, "e-audit-result");
    assert_eq!(dependency.to_id, "e-audit-repeat");

    let receipt = String::from_utf8(asset_bytes("CP06", "receipt_audit")).unwrap();
    assert!(receipt.contains("AUD-2081"));
    assert!(receipt.contains("consumed_attachment_event_id=e-audit-repeat"));
    assert!(receipt.contains("artifact_sha256="));
    let root = fixture_root("CP06");
    let world_bytes = fs::read(root.join("world/policy-state.json")).unwrap();
    let world: Value = serde_json::from_slice(&world_bytes).unwrap();
    assert_eq!(world["artifact_sha256"], original_asset.sha256);
    assert_eq!(world["receipt_requires_occurrence"], "e-audit-repeat");
    assert_eq!(
        event(&case, "e-policy-original")["provenance"][0]["content_hash"],
        sha256(&world_bytes)
    );
    assert_eq!(
        event(&case, "e-audit-repeat")["provenance"][0]["content_hash"],
        sha256(&world_bytes)
    );
    let key: Value =
        serde_json::from_slice(&fs::read(root.join("evaluation/key.json")).unwrap()).unwrap();
    let required: BTreeSet<_> = key["required_event_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert!(required.contains("e-policy-original"));
    assert!(required.contains("e-audit-repeat"));
    assert!(required.contains("e-audit-result"));
    assert!(key["required_relation_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == "audit-receipt-consumes-repeat"));
}

#[test]
fn synthetic_success_fails_for_wrong_state_citation_receipt_action_or_malformed_output() {
    let cp05 = load("CP05");
    let mut wrong_state = expected_answer("CP05");
    wrong_state["later_world_state_revision"] = json!("route-registry@R17");
    assert_eq!(
        run_bad_final(&cp05, wrong_state, "after", json!("71")).status,
        Claim2SlotStatus::Fail
    );
    assert_eq!(
        run_bad_final(
            &cp05,
            expected_answer("CP05"),
            "later_occurrence_id",
            json!("e-catalog-initial")
        )
        .status,
        Claim2SlotStatus::Fail
    );

    let cp06 = load("CP06");
    assert_eq!(
        run_bad_final(
            &cp06,
            expected_answer("CP06"),
            "receipt_id",
            json!("AUD-2080")
        )
        .status,
        Claim2SlotStatus::Fail
    );
    assert_eq!(
        run_bad_final(
            &cp06,
            expected_answer("CP06"),
            "receipt_consumes_occurrence_id",
            json!("e-policy-original")
        )
        .status,
        Claim2SlotStatus::Fail
    );

    let mut wrong_menu_action = Claim2ArmState::new(&cp05, Claim2ProjectionMode::Baseline);
    render_claim2_request(&cp05, &mut wrong_menu_action).unwrap();
    let fail = wrong_menu_action
        .record_output(&cp05, json!({"action_id":"read_health_only"}).to_string())
        .unwrap()
        .unwrap();
    assert_eq!(fail.status, Claim2SlotStatus::Fail);
    assert_eq!(
        wrong_menu_action.slots()[1].status,
        Claim2SlotStatus::NotExecutedAfterFailure
    );
    assert_eq!(
        wrong_menu_action.slots()[2].status,
        Claim2SlotStatus::NotExecutedAfterFailure
    );

    let mut malformed = Claim2ArmState::new(&cp06, Claim2ProjectionMode::Baseline);
    render_claim2_request(&cp06, &mut malformed).unwrap();
    let fail = malformed
        .record_output(&cp06, "not-json".to_string())
        .unwrap()
        .unwrap();
    assert_eq!(fail.status, Claim2SlotStatus::Fail);
    assert_eq!(
        malformed.slots()[1].status,
        Claim2SlotStatus::NotExecutedAfterFailure
    );

    let mut malformed_final = Claim2ArmState::new(&cp06, Claim2ProjectionMode::Baseline);
    for action in action_ids("CP06") {
        render_claim2_request(&cp06, &mut malformed_final).unwrap();
        malformed_final
            .record_output(&cp06, json!({"action_id":action}).to_string())
            .unwrap();
    }
    render_claim2_request(&cp06, &mut malformed_final).unwrap();
    let fail = malformed_final
        .record_output(&cp06, "{\"answer\":".to_string())
        .unwrap()
        .unwrap();
    assert_eq!(fail.status, Claim2SlotStatus::Fail);
    assert_eq!(malformed_final.slots()[2].status, Claim2SlotStatus::Fail);
}

#[test]
fn paired_controls_pass_synthetic_evaluators_and_preserve_exact_arm_projections() {
    for case_id in ["CP05", "CP06"] {
        let case = load(case_id);
        let (mut baseline, baseline_eval) = run_synthetic_arm(
            &case,
            Claim2ProjectionMode::Baseline,
            expected_answer(case_id),
        );
        let (mut noop, noop_eval) =
            run_synthetic_arm(&case, Claim2ProjectionMode::NoOp, expected_answer(case_id));
        let (mut intervention, intervention_eval) = run_synthetic_arm(
            &case,
            Claim2ProjectionMode::Intervention,
            expected_answer(case_id),
        );
        for evaluation in [&baseline_eval, &noop_eval, &intervention_eval] {
            assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
            assert!(evaluation.action_checks_passed);
            assert!(evaluation.intermediate_state_checks_passed);
            assert!(evaluation.receipt_identity_checks_passed);
            assert!(evaluation.required_context_preserved);
            assert!(evaluation.required_relations_preserved);
            assert!(evaluation.critical_events_preserved);
            assert!(evaluation.structurally_complete);
        }
        let comparison = compare_paired_arms(&mut baseline, &mut noop, &mut intervention).unwrap();
        assert_eq!(comparison.status, Claim2SlotStatus::Pass);
        assert!(comparison.baseline_noop_match);
        assert!(comparison.intervention_pre_treatment_match);
    }
}

#[test]
fn public_requests_hide_evaluation_answers_and_static_serialization_is_stable() {
    for case_id in ["CP05", "CP06"] {
        let case = load(case_id);
        let root = fixture_root(case_id);
        let key_bytes = fs::read(root.join(&case.manifest().evaluation_key_path)).unwrap();
        let key: Value = serde_json::from_slice(&key_bytes).unwrap();
        let hidden_answer = serde_json::to_string(&key["expected_final_answer"]).unwrap();
        assert!(!case
            .manifest()
            .assets
            .iter()
            .any(|asset| asset.relative_path == case.manifest().evaluation_key_path));
        assert!(!serde_json::to_string(case.manifest())
            .unwrap()
            .contains(&hidden_answer));

        let reloaded = load(case_id);
        assert_eq!(
            serde_json::to_vec(case.manifest()).unwrap(),
            serde_json::to_vec(reloaded.manifest()).unwrap()
        );
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            for slot in 1..=3 {
                let first = Claim2ArmState::new(&case, mode)
                    .preview_request(&case, slot)
                    .unwrap();
                let second = Claim2ArmState::new(&reloaded, mode)
                    .preview_request(&reloaded, slot)
                    .unwrap();
                assert_eq!(first.request_json, second.request_json);
                assert!(!first.dispatchable);
                assert!(!String::from_utf8(first.request_json.clone())
                    .unwrap()
                    .contains(&hidden_answer));
                assert_eq!(
                    first.metrics.token_count_status,
                    Claim2TokenCountStatus::ExactTokenizationRequired
                );
                if mode == Claim2ProjectionMode::Baseline {
                    println!(
                        "{case_id} BASELINE request{slot}: total_json={} fixed_content={} carried_history={} attachment={} omitted={} wrapper={} json_escape={} max_output_envelope={} raw_output_envelope={} unbound_outputs={:?} unbound_receipts={:?} possible_receipt_options={:?} token_count={:?} dispatchable={}",
                        first.metrics.exact_request_json_bytes,
                        first.metrics.fixed_content_utf8_bytes,
                        first.metrics.carried_assistant_utf8_bytes + first.metrics.carried_environment_utf8_bytes,
                        first.metrics.attachment_utf8_bytes,
                        first.metrics.omitted_attachment_utf8_bytes,
                        first.metrics.structural_wrapper_bytes,
                        first.metrics.json_escape_bytes,
                        first.metrics.assistant_output_planning_bytes,
                        first.metrics.raw_prior_output_planning_utf8_bytes,
                        first.metrics.unbound_raw_assistant_slots,
                        first.metrics.unbound_environment_receipt_slots,
                        first.metrics.possible_environment_receipt_sizes.iter().map(|entry| (entry.request_slot, entry.action_id.as_str(), entry.source_event_id.as_str(), entry.utf8_bytes)).collect::<Vec<_>>(),
                        first.metrics.token_count_status,
                        first.dispatchable
                    );
                }
            }
        }
    }
}
