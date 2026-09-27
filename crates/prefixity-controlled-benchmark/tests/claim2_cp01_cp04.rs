use prefixity_controlled_benchmark::{
    compare_paired_arms, evaluate_claim2_arm, load_claim2_case, project_claim2_trace,
    render_claim2_request, select_claim2_decision, Claim2ArmState, Claim2AssetKind,
    Claim2Evaluation, Claim2ProjectionMode, Claim2SlotStatus, ResearchInterventionClass,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_root(case_id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/claim2")
        .join(case_id.to_ascii_lowercase())
}

fn load(case_id: &str) -> prefixity_controlled_benchmark::LoadedClaim2Case {
    load_claim2_case(&fixture_root(case_id).join("case.json")).unwrap()
}

fn repeated_event(case_id: &str) -> &'static str {
    match case_id {
        "CP01" => "e-repeat",
        "CP04" => "e-recovery-copy",
        _ => panic!("unowned case"),
    }
}

fn expected_actions(case_id: &str) -> (&'static str, &'static str) {
    match case_id {
        "CP01" => ("inspect_retry_implementation", "run_retry_regression"),
        "CP04" => ("run_candidate_guardrail_probe", "verify_restored_state"),
        _ => panic!("unowned case"),
    }
}

fn expected_answer(case_id: &str) -> Value {
    match case_id {
        "CP01" => json!({
            "file": "src/retry.py",
            "symbol": "dispatch_with_retry",
            "root_cause": "range(1, policy.max_attempts) excludes the configured final attempt",
            "configured_max_attempts": 4,
            "observed_attempts": 3,
            "minimal_fix": "range(1, policy.max_attempts + 1)",
            "backoff_unit": "milliseconds"
        }),
        "CP04" => json!({
            "selected_branch": "stable",
            "verified_source_revision": "R41",
            "account_scope_required": true,
            "rejected_candidate_branch": "quick-route-v42",
            "invariant": "request tenant must match credential tenant when account scope is required"
        }),
        _ => panic!("unowned case"),
    }
}

fn output_action(action_id: &str) -> String {
    json!({ "action_id": action_id }).to_string()
}

fn output_answer(answer: Value) -> String {
    json!({ "answer": answer }).to_string()
}

fn asset_path(case_id: &str, asset_id: &str) -> PathBuf {
    let case = load(case_id);
    let descriptor = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == asset_id)
        .unwrap();
    fixture_root(case_id).join(&descriptor.relative_path)
}

fn asset_text(case_id: &str, asset_id: &str) -> String {
    fs::read_to_string(asset_path(case_id, asset_id)).unwrap()
}

fn response_fixture(case_id: &str, slot: u8) -> String {
    let actions = expected_actions(case_id);
    match slot {
        1 => output_action(actions.0),
        2 => output_action(actions.1),
        3 => output_answer(expected_answer(case_id)),
        _ => panic!("only three request slots exist"),
    }
}

fn run_synthetic_arm(
    case: &prefixity_controlled_benchmark::LoadedClaim2Case,
    mode: Claim2ProjectionMode,
) -> (
    Claim2ArmState,
    Claim2Evaluation,
    Vec<prefixity_controlled_benchmark::Claim2RenderedRequest>,
) {
    let mut arm = Claim2ArmState::new(case, mode);
    let mut requests = Vec::new();
    for slot in 1..=3 {
        let request = render_claim2_request(case, &mut arm).unwrap();
        assert_eq!(request.request_slot, slot);
        assert!(!request.dispatchable);
        requests.push(request);
        let output = response_fixture(case.case_id(), slot);
        let evaluation = arm.record_output(case, output).unwrap();
        if slot < 3 {
            assert!(evaluation.is_none());
        } else {
            let evaluation = evaluation.expect("request 3 evaluates the synthetic answer");
            assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
            return (arm, evaluation, requests);
        }
    }
    unreachable!("three requests always return a final evaluation")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn temporary_case_copy(case_id: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let target = std::env::temp_dir().join(format!(
        "prefixity-claim2-{case_id}-{}-{nonce}",
        std::process::id()
    ));
    copy_tree(&fixture_root(case_id), &target);
    target
}

fn event_map(manifest: &Value) -> BTreeMap<String, Value> {
    manifest["planner_input"]["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|event| {
            (
                event["event_id"].as_str().unwrap().to_string(),
                event.clone(),
            )
        })
        .collect()
}

fn count_occurrences(text: &str, needle: &str) -> usize {
    assert!(!needle.is_empty());
    text.match_indices(needle).count()
}

#[test]
fn cp01_cp04_are_policy_selected_single_target_natural_duplicates() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        let target_id = repeated_event(case_id);
        let selection = select_claim2_decision(&case).unwrap();
        assert_eq!(selection.decision.class, ResearchInterventionClass::Prune);
        assert_eq!(
            selection.decision.rule, "EXACT_DUPLICATE_PRUNE",
            "{case_id}"
        );
        assert_eq!(
            selection.decision.target_event_id.as_deref(),
            Some(target_id)
        );
        assert_eq!(selection.candidates.exact_duplicate_prune.len(), 1);
        let candidate = &selection.candidates.exact_duplicate_prune[0];
        assert_eq!(candidate.target_event_id, target_id);
        assert_eq!(candidate.matching_earlier_event_ids, ["e-source"]);
        assert_eq!(candidate.rule, "EXACT_DUPLICATE_PRUNE");
        assert!(selection.candidates.explicit_supersession_defer.is_empty());
        assert!(selection.candidates.same_zone_protocol_relocate.is_empty());

        let manifest = serde_json::to_value(case.manifest()).unwrap();
        let events = event_map(&manifest);
        let mut aliases = BTreeSet::new();
        for event in manifest["planner_input"]["events"].as_array().unwrap() {
            aliases.insert(event["event_id"].as_str().unwrap().to_string());
            if let Some(action_id) = event["action"]["action_id"].as_str() {
                assert!(
                    aliases.insert(action_id.to_string()),
                    "{case_id}: action alias"
                );
            }
            if let Some(result_id) = event["result"]["result_id"].as_str() {
                assert!(
                    aliases.insert(result_id.to_string()),
                    "{case_id}: result alias"
                );
            }
            if let Some(context_id) = event["context_block_id"].as_str() {
                assert!(
                    aliases.insert(context_id.to_string()),
                    "{case_id}: context alias"
                );
            }
        }
        for event in manifest["planner_input"]["events"].as_array().unwrap() {
            let current_sequence = event["sequence_index"].as_u64().unwrap();
            for field in ["parent_event_ids", "reference_event_ids"] {
                if let Some(references) = event.get(field).and_then(Value::as_array) {
                    for reference in references {
                        let reference = reference.as_str().unwrap();
                        assert!(
                            aliases.contains(reference),
                            "{case_id}: unresolved {field} {reference}"
                        );
                        if field == "parent_event_ids" {
                            assert!(
                                events[reference]["sequence_index"].as_u64().unwrap()
                                    < current_sequence,
                                "{case_id}: parent edges must point backward"
                            );
                        }
                    }
                }
            }
        }
        for relation in manifest["planner_input"]["relations"].as_array().unwrap() {
            assert!(aliases.contains(relation["from_id"].as_str().unwrap()));
            assert!(aliases.contains(relation["to_id"].as_str().unwrap()));
        }
        assert_eq!(
            case.manifest()
                .action_menu
                .iter()
                .map(|action| action.action_slot)
                .collect::<Vec<_>>(),
            [1, 1, 2, 2]
        );
        for transition in &case.manifest().action_menu {
            let action_event = &events[&transition.action_event_id];
            let result_event = &events[&transition.result_event_id];
            assert_eq!(action_event["action"]["action_id"], transition.action_id);
            assert_eq!(
                result_event["result"]["originating_action_id"],
                transition.action_id
            );
            let receipt_asset = case
                .manifest()
                .assets
                .iter()
                .find(|asset| asset.asset_id == transition.result_asset_id)
                .unwrap();
            assert_eq!(receipt_asset.kind, Claim2AssetKind::EnvironmentReceipt);
            assert_eq!(
                receipt_asset.event_id.as_deref(),
                Some(transition.result_event_id.as_str())
            );
            let receipt_bytes =
                fs::read(fixture_root(case_id).join(&receipt_asset.relative_path)).unwrap();
            assert_eq!(digest(&receipt_bytes), receipt_asset.sha256);
            assert_eq!(
                result_event["result"]["observation_hash"].as_str(),
                Some(receipt_asset.sha256.as_str())
            );
            assert!(!transition.state_after.is_empty());
        }
        let original = &events["e-source"];
        let repeated = &events[target_id];
        assert_eq!(original["event_type"], "message");
        assert_eq!(repeated["event_type"], "message");
        assert_eq!(original["actor_role"], "user");
        assert_eq!(repeated["actor_role"], "user");
        assert_eq!(
            original["content_hash"], repeated["content_hash"],
            "{case_id} attachment bytes must be exactly equal"
        );
        assert_eq!(
            original["world_state_revision"], repeated["world_state_revision"],
            "{case_id} must use the same explicit state revision"
        );
        let original_asset = case
            .manifest()
            .assets
            .iter()
            .find(|asset| asset.event_id.as_deref() == Some("e-source"))
            .unwrap();
        let repeated_asset = case
            .manifest()
            .assets
            .iter()
            .find(|asset| asset.event_id.as_deref() == Some(target_id))
            .unwrap();
        assert_eq!(original_asset.kind, Claim2AssetKind::ContextAttachment);
        assert_eq!(repeated_asset.kind, Claim2AssetKind::ContextAttachment);
        assert_eq!(original_asset.revision_id, repeated_asset.revision_id);
        let original_bytes =
            fs::read(fixture_root(case_id).join(&original_asset.relative_path)).unwrap();
        let repeated_bytes =
            fs::read(fixture_root(case_id).join(&repeated_asset.relative_path)).unwrap();
        assert_eq!(original_bytes, repeated_bytes);
        assert_eq!(digest(&original_bytes), original_asset.sha256);

        // The later attachment has no event-reference consumer and no
        // protected dependency/protocol relation. Its exporter provenance is
        // distinct from the Result receipt which records the workflow step.
        for event in manifest["planner_input"]["events"].as_array().unwrap() {
            assert!(!event
                .get("reference_event_ids")
                .and_then(Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(target_id))));
            assert!(!event
                .get("reference_event_ids")
                .and_then(Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some("ctx-repeat"))));
        }
        let relations = manifest["planner_input"]["relations"].as_array().unwrap();
        assert!(relations
            .iter()
            .filter(|relation| {
                relation["from_id"].as_str() == Some(target_id)
                    || relation["to_id"].as_str() == Some(target_id)
            })
            .all(|relation| relation["relation_type"] == "same_state_revision"));
        assert!(!relations.iter().any(|relation| {
            matches!(
                relation["relation_type"].as_str(),
                Some("depends_on" | "protocol_precedes")
            ) && (relation["from_id"].as_str() == Some(target_id)
                || relation["to_id"].as_str() == Some(target_id))
        }));
        assert!(repeated["parent_event_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|parent| parent.as_str() == Some("e-result2")));

        let templates = manifest["request_templates"].as_array().unwrap();
        for template in templates
            .iter()
            .filter(|template| template["request_slot"].as_u64().unwrap() < 3)
        {
            assert!(!template["messages"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|message| message["parts"].as_array().unwrap())
                .any(|part| { part["kind"] == "event_body" && part["event_id"] == target_id }));
        }
        let request_three = templates
            .iter()
            .find(|template| template["request_slot"] == 3)
            .unwrap();
        assert!(request_three["messages"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|message| message["parts"].as_array().unwrap())
            .any(|part| part["kind"] == "event_body" && part["event_id"] == target_id));
    }
}

#[test]
fn cp01_and_cp04_pinned_evidence_support_their_hidden_oracles() {
    let retry_source = fs::read_to_string(fixture_root("CP01").join("source/retry.py")).unwrap();
    let retry_config =
        fs::read_to_string(fixture_root("CP01").join("source/dispatcher.toml")).unwrap();
    let retry_loader =
        fs::read_to_string(fixture_root("CP01").join("source/policy_loader.py")).unwrap();
    let retry_test = fs::read_to_string(fixture_root("CP01").join("source/test_retry.py")).unwrap();
    assert!(retry_source.contains("for attempt in range(1, policy.max_attempts)"));
    assert!(retry_config.contains("max_attempts = 4"));
    assert!(retry_config.contains("backoff_ms = 125"));
    assert!(retry_loader.contains("max_attempts=int(values[\"max_attempts\"])"));
    assert!(retry_test.contains("statuses = iter([503, 503, 503, 200])"));
    let source_export = asset_text("CP01", "source_original");
    assert!(source_export.contains(&retry_source));
    assert!(source_export.contains(&retry_config));
    assert!(source_export.contains(&retry_loader));
    let calls_from_frozen_range = (1..4).count();
    let calls_from_inclusive_fix = (1..=4).count();
    assert_eq!(calls_from_frozen_range, 3);
    assert_eq!(calls_from_inclusive_fix, 4);
    let retry_receipt = asset_text("CP01", "receipt2");
    assert!(retry_receipt.contains("Observed transport calls: 3"));
    assert!(retry_receipt.contains("max_attempts=4"));
    assert!(retry_receipt.contains("125 milliseconds is 0.125 seconds"));
    assert!(retry_receipt.contains("native Message occurrence e-repeat"));

    let branch_config =
        fs::read_to_string(fixture_root("CP04").join("source/runtime.toml")).unwrap();
    let branch_rules =
        fs::read_to_string(fixture_root("CP04").join("source/branch_rules.py")).unwrap();
    let candidate_config =
        fs::read_to_string(fixture_root("CP04").join("source/quick-route-v42.toml")).unwrap();
    let cp04 = load("CP04");
    let candidate_asset = cp04
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == "candidate_branch_state")
        .unwrap();
    assert_eq!(candidate_asset.kind, Claim2AssetKind::PromptText);
    assert_eq!(candidate_asset.revision_id.as_deref(), Some("cp04-r42"));
    let candidate_bytes =
        fs::read(fixture_root("CP04").join(&candidate_asset.relative_path)).unwrap();
    assert_eq!(digest(&candidate_bytes), candidate_asset.sha256);
    assert!(
        !serde_json::to_value(cp04.manifest()).unwrap()["request_templates"]
            .to_string()
            .contains("candidate_branch_state")
    );
    let recovery_code =
        fs::read_to_string(fixture_root("CP04").join("source/recovery.py")).unwrap();
    let recovery_policy =
        fs::read_to_string(fixture_root("CP04").join("source/branch-policy.toml")).unwrap();
    assert!(branch_config.contains("active_release_channel = \"stable\""));
    assert!(branch_config.contains("source_revision = \"R41\""));
    assert!(branch_config.contains("account_scope_required = true"));
    assert!(branch_rules.contains("request_tenant == token_tenant"));
    assert!(recovery_code.contains("receipt.source_revision == expected_revision"));
    assert!(recovery_policy.contains("restore_origin_on_probe_failure = true"));
    assert!(candidate_config.contains("source_revision = \"R42\""));
    assert!(candidate_config.contains("account_scope_required = false"));
    let branch_export = asset_text("CP04", "source_original");
    assert!(branch_export.contains(&branch_config));
    assert!(branch_export.contains(&branch_rules));
    assert!(!branch_export.contains(&candidate_config));
    let failed_probe = asset_text("CP04", "receipt1");
    let restored_state = asset_text("CP04", "receipt2");
    assert!(failed_probe.contains("account_scope_required from true to false"));
    assert!(failed_probe.contains("RESTORE-CP04-R41"));
    assert!(restored_state.contains("branch=stable, source_revision=R41, dirty=false"));
    assert!(restored_state.contains("account_scope_required=true"));
    assert!(restored_state.contains("candidate quick-route-v42 failed"));
    assert!(restored_state.contains("native Message occurrence e-recovery-copy"));
}

#[test]
fn arm_projection_removes_only_the_selected_message_and_preserves_dependencies() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        let target = repeated_event(case_id);
        let baseline = project_claim2_trace(&case, Claim2ProjectionMode::Baseline).unwrap();
        let noop = project_claim2_trace(&case, Claim2ProjectionMode::NoOp).unwrap();
        let intervention = project_claim2_trace(&case, Claim2ProjectionMode::Intervention).unwrap();
        assert_eq!(
            baseline, noop,
            "{case_id}: forced NO_OP must preserve trace bytes"
        );

        let baseline_value = serde_json::to_value(&baseline).unwrap();
        let intervention_value = serde_json::to_value(&intervention).unwrap();
        let mut baseline_events = baseline_value["events"].as_array().unwrap().clone();
        let mut intervention_events = intervention_value["events"].as_array().unwrap().clone();
        baseline_events.retain(|event| event["event_id"].as_str() != Some(target));
        intervention_events.iter_mut().for_each(|event| {
            event.as_object_mut().unwrap().remove("sequence_index");
        });
        for event in &mut baseline_events {
            event.as_object_mut().unwrap().remove("sequence_index");
        }
        assert_eq!(baseline_events, intervention_events);
        assert!(intervention_value["events"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_id"].as_str() != Some(target)));

        let mut baseline_relations = baseline_value["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|relation| relation["relation_id"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        let intervention_relations = intervention_value["relations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|relation| relation["relation_id"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        baseline_relations.remove("same-state-source-repeat");
        assert_eq!(baseline_relations, intervention_relations);
        for required in ["produces-1-1", "produces-2-1"] {
            assert!(intervention_relations.contains(required));
        }
    }
}

#[test]
fn synthetic_three_request_trajectories_pass_all_three_arms_and_equality_gates() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        let (mut baseline, baseline_eval, baseline_requests) =
            run_synthetic_arm(&case, Claim2ProjectionMode::Baseline);
        let (mut noop, noop_eval, noop_requests) =
            run_synthetic_arm(&case, Claim2ProjectionMode::NoOp);
        let (mut intervention, intervention_eval, intervention_requests) =
            run_synthetic_arm(&case, Claim2ProjectionMode::Intervention);
        for evaluation in [&baseline_eval, &noop_eval, &intervention_eval] {
            assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
            assert!(evaluation.action_checks_passed);
            assert!(evaluation.intermediate_state_checks_passed);
            assert!(evaluation.receipt_identity_checks_passed);
            assert!(evaluation.final_answer_passed);
            assert!(evaluation.required_context_preserved);
            assert!(evaluation.required_relations_preserved);
            assert!(evaluation.critical_events_preserved);
            assert!(evaluation.structurally_complete);
        }
        assert!(baseline
            .slots()
            .iter()
            .all(|slot| slot.status == Claim2SlotStatus::Pass));
        assert!(noop
            .slots()
            .iter()
            .all(|slot| slot.status == Claim2SlotStatus::Pass));
        assert!(intervention
            .slots()
            .iter()
            .all(|slot| slot.status == Claim2SlotStatus::Pass));
        assert_eq!(
            baseline_requests[0].request_json,
            noop_requests[0].request_json
        );
        assert_eq!(
            baseline_requests[1].request_json,
            noop_requests[1].request_json
        );
        assert_eq!(
            baseline_requests[2].request_json,
            noop_requests[2].request_json
        );
        assert_eq!(
            baseline_requests[0].request_json,
            intervention_requests[0].request_json
        );
        assert_eq!(
            baseline_requests[1].request_json,
            intervention_requests[1].request_json
        );
        assert_ne!(
            baseline_requests[2].request_json,
            intervention_requests[2].request_json
        );
        assert_eq!(
            intervention_requests[2]
                .metrics
                .omitted_attachment_utf8_bytes,
            fs::metadata(asset_path(case_id, "source_repeat"))
                .unwrap()
                .len() as usize
        );
        assert!(baseline_requests[2].metrics.omitted_attachment_utf8_bytes == 0);
        assert_eq!(
            baseline_requests[2].metrics.carried_assistant_utf8_bytes,
            baseline_requests[2]
                .messages
                .iter()
                .filter(|message| message.role == "assistant")
                .map(|message| message.content.len())
                .sum::<usize>()
        );
        assert!(baseline_requests[1].messages.iter().any(|message| {
            message.role == "assistant" && message.content == response_fixture(case_id, 1)
        }));
        for source_result in ["e-result1", "e-result2"] {
            assert!(baseline_requests[2]
                .messages
                .iter()
                .any(|message| message.source_event_id.as_deref() == Some(source_result)));
        }
        let repeated_body = asset_text(case_id, "source_repeat");
        let baseline_final_body = baseline_requests[2]
            .messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<String>();
        let intervention_final_body = intervention_requests[2]
            .messages
            .iter()
            .map(|message| message.content.as_str())
            .collect::<String>();
        assert_eq!(count_occurrences(&baseline_final_body, &repeated_body), 2);
        assert_eq!(
            count_occurrences(&intervention_final_body, &repeated_body),
            1
        );

        let comparison = compare_paired_arms(&mut baseline, &mut noop, &mut intervention).unwrap();
        assert!(comparison.baseline_noop_match);
        assert!(comparison.intervention_pre_treatment_match);
        assert_eq!(comparison.status, Claim2SlotStatus::Pass);
        assert!(render_claim2_request(&case, &mut baseline).is_err());
        assert!(
            evaluate_claim2_arm(&case, &intervention)
                .unwrap()
                .final_answer_passed
        );
    }
}

#[test]
fn malformed_wrong_action_wrong_final_fact_and_broken_graphs_fail_closed() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        let expected = expected_actions(case_id);
        let alternative = case
            .manifest()
            .action_menu
            .iter()
            .find(|action| action.action_slot == 1 && action.action_id != expected.0)
            .unwrap();

        let mut malformed = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut malformed).unwrap();
        let malformed_result = malformed
            .record_output(&case, "{\"action_id\":".to_string())
            .unwrap()
            .unwrap();
        assert_eq!(malformed_result.status, Claim2SlotStatus::Fail);
        assert_eq!(
            malformed.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );

        let mut unknown = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut unknown).unwrap();
        let unknown_result = unknown
            .record_output(&case, output_action("not_in_the_frozen_menu"))
            .unwrap()
            .unwrap();
        assert_eq!(unknown_result.status, Claim2SlotStatus::Fail);

        let mut wrong_action = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut wrong_action).unwrap();
        let wrong_action_result = wrong_action
            .record_output(&case, output_action(&alternative.action_id))
            .unwrap()
            .unwrap();
        assert_eq!(wrong_action_result.status, Claim2SlotStatus::Fail);
        assert_eq!(wrong_action.slots()[0].status, Claim2SlotStatus::Fail);
        assert_eq!(
            wrong_action.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        assert_eq!(
            wrong_action.slots()[2].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );

        let mut wrong_answer = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut wrong_answer).unwrap();
        wrong_answer
            .record_output(&case, output_action(expected.0))
            .unwrap();
        render_claim2_request(&case, &mut wrong_answer).unwrap();
        wrong_answer
            .record_output(&case, output_action(expected.1))
            .unwrap();
        render_claim2_request(&case, &mut wrong_answer).unwrap();
        let mut wrong_final = expected_answer(case_id);
        match case_id {
            "CP01" => wrong_final["observed_attempts"] = json!(4),
            "CP04" => wrong_final["account_scope_required"] = json!(false),
            _ => unreachable!(),
        }
        let wrong_answer_result = wrong_answer
            .record_output(&case, output_answer(wrong_final))
            .unwrap()
            .unwrap();
        assert_eq!(wrong_answer_result.status, Claim2SlotStatus::Fail);
        assert!(!wrong_answer_result.final_answer_passed);
        assert!(wrong_answer_result.required_relations_preserved);

        for missing_relation in ["same-state-source-repeat", "produces-1-1"] {
            let temporary_dir = temporary_case_copy(case_id);
            let case_path = temporary_dir.join("case.json");
            let mut manifest: Value =
                serde_json::from_slice(&fs::read(&case_path).unwrap()).unwrap();
            manifest["planner_input"]["relations"]
                .as_array_mut()
                .unwrap()
                .retain(|relation| relation["relation_id"] != missing_relation);
            fs::write(&case_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
            assert!(
                load_claim2_case(&case_path).is_err(),
                "{case_id}: missing {missing_relation} must fail fixture admission"
            );
            fs::remove_dir_all(&temporary_dir).unwrap();
        }
    }
}

#[test]
fn evaluation_key_is_external_and_pinned_assets_are_portable_utf8() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        let root = fixture_root(case_id);
        let public_manifest = serde_json::to_value(case.manifest()).unwrap();
        let public_json = public_manifest.to_string();
        assert!(public_manifest["evaluation_key_path"] == "evaluation/key.json");
        assert!(!public_json.contains("expected_action_ids"));
        assert!(!public_json.contains("expected_final_answer"));
        assert!(!public_json.contains("required_event_ids"));
        assert!(!public_json.contains("critical_event_ids"));
        let key_path = root.join("evaluation/key.json");
        let key_bytes = fs::read(&key_path).unwrap();
        assert_eq!(digest(&key_bytes), case.manifest().evaluation_key_sha256);
        assert!(case.manifest().assets.iter().all(|asset| {
            asset.relative_path != "evaluation/key.json"
                && asset.sha256 != case.manifest().evaluation_key_sha256
        }));
        assert!(!key_bytes.starts_with(&[0xef, 0xbb, 0xbf]));
        assert!(!key_bytes.contains(&b'\r'));

        for asset in &case.manifest().assets {
            let bytes = fs::read(root.join(&asset.relative_path)).unwrap();
            assert_eq!(digest(&bytes), asset.sha256, "{}", asset.asset_id);
            assert!(
                !bytes.starts_with(&[0xef, 0xbb, 0xbf]),
                "{}",
                asset.asset_id
            );
            assert!(!bytes.contains(&b'\r'), "{}", asset.asset_id);
            assert!(std::str::from_utf8(&bytes).is_ok(), "{}", asset.asset_id);
        }
        let first_manifest_serialization = serde_json::to_vec(case.manifest()).unwrap();
        let second_manifest_serialization = serde_json::to_vec(case.manifest()).unwrap();
        assert_eq!(first_manifest_serialization, second_manifest_serialization);
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            for request_slot in 1..=3 {
                let first = Claim2ArmState::new(&case, mode)
                    .preview_request(&case, request_slot)
                    .unwrap();
                let second = Claim2ArmState::new(&case, mode)
                    .preview_request(&case, request_slot)
                    .unwrap();
                assert_eq!(first.request_json, second.request_json);
                assert_eq!(
                    first.request_json.len(),
                    first.metrics.exact_request_json_bytes
                );
                assert!(!first.dispatchable);
            }
        }
    }
}

#[test]
fn offline_byte_precheck_separates_exact_request_one_from_unbound_skeletons() {
    for case_id in ["CP01", "CP04"] {
        let case = load(case_id);
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            for request_slot in 1..=3 {
                let request = Claim2ArmState::new(&case, mode)
                    .preview_request(&case, request_slot)
                    .unwrap();
                let metrics = &request.metrics;
                assert!(!request.dispatchable);
                assert_eq!(metrics.exact_request_json_bytes, request.request_json.len());
                assert_eq!(metrics.token_count_status,
                    prefixity_controlled_benchmark::Claim2TokenCountStatus::ExactTokenizationRequired);
                if request_slot == 1 {
                    assert!(metrics.unbound_raw_assistant_slots.is_empty());
                    assert!(metrics.unbound_environment_receipt_slots.is_empty());
                    assert_eq!(metrics.raw_prior_output_planning_utf8_bytes, 0);
                } else {
                    assert_eq!(
                        metrics.unbound_raw_assistant_slots,
                        (1..request_slot).collect::<Vec<_>>()
                    );
                    assert_eq!(
                        metrics.unbound_environment_receipt_slots,
                        (1..request_slot).collect::<Vec<_>>()
                    );
                    assert_eq!(
                        metrics.raw_prior_output_planning_utf8_bytes,
                        usize::from(request_slot - 1)
                            * case.manifest().assistant_output_planning_bytes
                    );
                    assert!(!metrics.possible_environment_receipt_sizes.is_empty());
                }
                if request_slot == 3 && mode == Claim2ProjectionMode::Intervention {
                    assert_eq!(
                        metrics.omitted_attachment_utf8_bytes,
                        fs::metadata(asset_path(case_id, "source_repeat"))
                            .unwrap()
                            .len() as usize
                    );
                }
                println!(
                    "[claim2-offline-bytes] {case_id} {mode:?} request={} dispatchable={} body_bytes={} fixed={} attachment={} omitted={} carried_assistant={} carried_environment={} wrapper={} json_escape={} assistant_envelope={} unbound_assistant={:?} unbound_receipts={:?} possible_receipts={:?}",
                    request_slot,
                    request.dispatchable,
                    metrics.exact_request_json_bytes,
                    metrics.fixed_content_utf8_bytes,
                    metrics.attachment_utf8_bytes,
                    metrics.omitted_attachment_utf8_bytes,
                    metrics.carried_assistant_utf8_bytes,
                    metrics.carried_environment_utf8_bytes,
                    metrics.structural_wrapper_bytes,
                    metrics.json_escape_bytes,
                    metrics.raw_prior_output_planning_utf8_bytes,
                    metrics.unbound_raw_assistant_slots,
                    metrics.unbound_environment_receipt_slots,
                    metrics.possible_environment_receipt_sizes,
                );
            }
        }
    }
}
