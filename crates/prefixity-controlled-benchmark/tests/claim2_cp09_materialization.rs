#[path = "../examples/support/claim2_cp09_fixture.rs"]
mod fixture;

use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, compare_paired_arms, evaluate_claim2_arm, load_claim2_case,
    render_claim2_request, select_claim2_decision, Claim2ArmState, Claim2ProjectionMode,
    Claim2RenderedRequest, Claim2SlotStatus, LoadedClaim2Case, RelationType,
    ResearchInterventionClass,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_root() -> PathBuf {
    repository_root().join("fixtures/claim2/cp09")
}

fn load_cp09() -> LoadedClaim2Case {
    load_claim2_case(&fixture_root().join("case.json")).unwrap()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse(path: &str) -> Value {
    serde_json::from_slice(&fs::read(fixture_root().join(path)).unwrap()).unwrap()
}

fn hidden_answer() -> Value {
    parse("evaluation/key.json")["expected_final_answer"].clone()
}

fn independently_derived_answer() -> Value {
    let manifest = parse("source/deployment-cohort-manifest-v1.json");
    let state = parse("source/frozen-release-state-v1.json");
    let gates = manifest["required_gates"].as_array().unwrap();
    let actual_gates = state["gate_results"].as_array().unwrap();
    let ordered_gate_ids = gates
        .iter()
        .map(|gate| gate["gate_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(ordered_gate_ids.len(), 6);
    assert_eq!(actual_gates.len(), ordered_gate_ids.len());
    for (required, actual) in gates.iter().zip(actual_gates) {
        assert_eq!(required["gate_id"], actual["gate_id"]);
        assert_eq!(actual["status"], "PASS");
    }

    let release = &manifest["release"];
    let build = &state["build"];
    assert_eq!(release["release_id"], state["release_id"]);
    assert_eq!(release["source_commit"], build["source_commit"]);
    assert_eq!(
        release["source_tree_digest"],
        state["source"]["source_tree_sha256"]
    );
    assert_eq!(release["build_attestation_id"], build["attestation_id"]);
    assert_eq!(release["build_digest"], build["build_digest"]);
    assert_eq!(release["image_digest"], build["subject_image_digest"]);
    assert_eq!(build["signature_status"], "VALID_SYNTHETIC_ATTESTATION");

    let canary = &state["canary_observation"];
    let limits = &manifest["canary_limits"];
    let canary_cohort = manifest["rollout_cohorts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["cohort_id"] == canary["cohort_id"])
        .unwrap();
    assert_eq!(canary["traffic_share_basis_points"], 1000);
    assert_eq!(
        canary["traffic_share_basis_points"],
        canary_cohort["traffic_share_basis_points"]
    );
    assert!(
        canary["availability_basis_points"].as_u64().unwrap()
            >= limits["availability_minimum_basis_points"]
                .as_u64()
                .unwrap()
    );
    assert!(
        canary["error_rate_basis_points"].as_u64().unwrap()
            <= limits["error_rate_maximum_basis_points"].as_u64().unwrap()
    );
    assert!(
        canary["p95_latency_milliseconds"].as_u64().unwrap()
            <= limits["p95_latency_maximum_milliseconds"].as_u64().unwrap()
    );
    assert_eq!(canary["observation_minutes"], limits["observation_minutes"]);

    let cohort = &state["requested_cohort"];
    let declared_cohort = manifest["rollout_cohorts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|candidate| candidate["cohort_id"] == cohort["cohort_id"])
        .unwrap();
    assert_eq!(declared_cohort["environment"], "production");
    assert_eq!(
        declared_cohort["deployment_unit_ids"],
        cohort["deployment_unit_ids"]
    );
    assert_eq!(cohort["target_id"], "production-wave-eu1");

    let rollback = &state["rollback_artifact"];
    assert_eq!(rollback["retrieval_status"], "RETRIEVABLE_READ_ONLY");
    assert_eq!(
        state["gate_results"][5]["rollback_artifact_id"],
        rollback["artifact_id"]
    );
    json!({
        "decision":"PROMOTE",
        "release_id":release["release_id"],
        "build_digest":release["build_digest"],
        "image_digest":release["image_digest"],
        "governing_gate_ids":ordered_gate_ids,
        "environment_cohort":{"environment":cohort["environment"],"cohort_id":cohort["cohort_id"],"deployment_unit_ids":cohort["deployment_unit_ids"]},
        "required_rollback_artifact_id":rollback["artifact_id"],
        "reason_code":"ALL_REQUIRED_GATES_PASS_ROLLBACK_READY"
    })
}

fn execute_history(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    answer: &Value,
) -> (Claim2ArmState, Vec<Claim2RenderedRequest>) {
    let mut arm = Claim2ArmState::new(case, mode);
    let mut requests = Vec::new();
    for (slot, action_id) in ["verify_build_provenance", "verify_required_rollout_gates"]
        .into_iter()
        .enumerate()
    {
        let request = render_claim2_request(case, &mut arm).unwrap();
        assert_eq!(request.request_slot, slot as u8 + 1);
        assert!(!request.dispatchable);
        requests.push(request);
        assert!(arm
            .record_output(case, canonical_claim2_action_output(action_id))
            .unwrap()
            .is_none());
    }
    let request3 = render_claim2_request(case, &mut arm).unwrap();
    assert_eq!(request3.request_slot, 3);
    assert!(!request3.dispatchable);
    requests.push(request3);
    let output = json!({"answer":answer}).to_string();
    let evaluation = arm
        .record_output(case, output)
        .unwrap()
        .expect("request 3 reaches the exact deterministic evaluator");
    assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
    assert_eq!(evaluation.model_status, Some(Claim2SlotStatus::Pass));
    assert!(evaluation.action_checks_passed);
    assert!(evaluation.intermediate_state_checks_passed);
    assert!(evaluation.receipt_identity_checks_passed);
    assert!(evaluation.final_answer_passed);
    assert!(evaluation.final_answer_shape_valid);
    assert!(evaluation.required_context_preserved);
    assert!(evaluation.required_relations_preserved);
    assert!(evaluation.critical_events_preserved);
    assert!(evaluation.structurally_complete);
    (arm, requests)
}

#[test]
fn deterministic_source_generator_reproduces_every_cp09_fixture_byte() {
    let root = fixture_root();
    let generated = fixture::generated_files_for_test();
    assert_eq!(generated.len(), 20);
    for (relative, expected) in generated {
        let tracked = fs::read(root.join(&relative))
            .unwrap_or_else(|error| panic!("missing generated CP09 file {relative}: {error}"));
        assert_eq!(tracked, expected, "generated bytes differ at {relative}");
    }
}

#[test]
fn frozen_policy_selects_only_the_ordinary_manifest_reattachment() {
    let case = load_cp09();
    assert_eq!(case.case_id(), "CP09");
    let manifest = case.manifest();
    let selection = select_claim2_decision(&case).unwrap();
    assert_eq!(selection.decision.class, ResearchInterventionClass::Prune);
    assert_eq!(selection.decision.rule, "EXACT_DUPLICATE_PRUNE");
    assert_eq!(
        selection.decision.target_event_id.as_deref(),
        Some("e-release-manifest-final-reattachment")
    );
    assert_eq!(selection.candidates.exact_duplicate_prune.len(), 1);
    assert!(selection.candidates.explicit_supersession_defer.is_empty());
    assert!(selection.candidates.same_zone_protocol_relocate.is_empty());

    let target = manifest
        .planner_input
        .events
        .iter()
        .find(|event| event.event_id == "e-release-manifest-final-reattachment")
        .unwrap();
    let original = manifest
        .planner_input
        .events
        .iter()
        .find(|event| event.event_id == "e-release-manifest-original")
        .unwrap();
    assert_eq!(
        target.event_type,
        prefixity_controlled_benchmark::EventType::Message
    );
    assert_eq!(
        target.actor_role,
        prefixity_controlled_benchmark::ActorRole::User
    );
    assert_eq!(target.content_hash, original.content_hash);
    assert_eq!(target.world_state_revision, original.world_state_revision);
    assert_eq!(target.sequence_index, 10);

    let original_asset = manifest
        .assets
        .iter()
        .find(|asset| asset.event_id.as_deref() == Some("e-release-manifest-original"))
        .unwrap();
    let final_asset = manifest
        .assets
        .iter()
        .find(|asset| asset.event_id.as_deref() == Some("e-release-manifest-final-reattachment"))
        .unwrap();
    assert_eq!(original_asset.sha256, final_asset.sha256);
    assert_eq!(original_asset.revision_id, final_asset.revision_id);
    let original_bytes =
        fs::read(fixture_root().join("bodies/deployment-cohort-manifest-original.json")).unwrap();
    let final_bytes =
        fs::read(fixture_root().join("bodies/deployment-cohort-manifest-final-packet.json"))
            .unwrap();
    let source_bytes =
        fs::read(fixture_root().join("source/deployment-cohort-manifest-v1.json")).unwrap();
    assert_eq!(original_bytes, final_bytes);
    assert_eq!(original_bytes, source_bytes);
    assert_eq!(
        target.content_hash.as_deref(),
        Some(digest(&original_bytes).as_str())
    );

    let target_context = target.context_block_id.as_deref().unwrap();
    let consumer_count = manifest
        .planner_input
        .events
        .iter()
        .filter(|event| {
            event
                .parent_event_ids
                .iter()
                .chain(&event.reference_event_ids)
                .any(|reference| reference == &target.event_id || reference == target_context)
        })
        .count();
    let protected_count = manifest
        .planner_input
        .relations
        .iter()
        .filter(|relation| {
            matches!(
                relation.relation_type,
                RelationType::DependsOn | RelationType::ProtocolPrecedes
            ) && (relation.from_id == target.event_id || relation.to_id == target.event_id)
        })
        .count();
    assert_eq!(consumer_count, 0);
    assert_eq!(protected_count, 0);
    let same_state = manifest
        .planner_input
        .relations
        .iter()
        .find(|relation| relation.relation_id == "same-state-manifest-reattachment")
        .unwrap();
    assert_eq!(same_state.from_id, "e-release-manifest-original");
    assert_eq!(same_state.to_id, "e-release-manifest-final-reattachment");
    assert_eq!(same_state.relation_type, RelationType::SameStateRevision);

    let recipe = parse("source/change-board-packet-recipe-v1.json");
    assert_eq!(
        recipe["ordinary_trigger"],
        "production_promotion_review_opened"
    );
    assert_eq!(recipe["benchmark_condition_used"], false);
    assert_eq!(
        recipe["attachment_behavior"],
        "emit_one_native_complete_manifest_message_for_every_release_promotion_board_packet"
    );
    assert_eq!(
        recipe["manifest_resolution"]["body_sha256"],
        final_asset.sha256
    );
    let workflow = fs::read_to_string(fixture_root().join("workflow.md")).unwrap();
    assert!(workflow.contains("irrespective of"));
    assert!(workflow.contains("document size or any benchmark arm"));

    let final_template = &manifest.request_templates[2];
    assert!(final_template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-release-manifest-final-reattachment")));
    assert!(!manifest.request_templates[..2].iter().any(|template| template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-release-manifest-final-reattachment"))));
}

#[test]
fn manifest_is_complete_state_is_frozen_and_all_dependencies_close() {
    let case = load_cp09();
    let manifest = case.manifest();
    let public = parse("source/deployment-cohort-manifest-v1.json");
    let state_bytes = fs::read(fixture_root().join("source/frozen-release-state-v1.json")).unwrap();
    let state: Value = serde_json::from_slice(&state_bytes).unwrap();
    assert_eq!(public["synthetic_only"], true);
    assert_eq!(public["record_state"], "immutable_governing_record");
    assert_eq!(
        public["completeness_contract"]["filler_entries_permitted"],
        false
    );
    assert_eq!(
        public["completeness_contract"]["cohort_membership_is_closed_world"],
        true
    );
    assert_eq!(
        public["bounded_deployment_units"].as_array().unwrap().len(),
        4
    );
    assert_eq!(public["environment_targets"].as_array().unwrap().len(), 3);
    assert_eq!(public["rollout_cohorts"].as_array().unwrap().len(), 3);
    assert_eq!(public["required_gates"].as_array().unwrap().len(), 6);
    assert_eq!(
        public["rollback_by_production_target"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(state["synthetic_only"], true);
    assert_eq!(state["record_state"], "frozen_read_only");
    assert_eq!(state["state_integrity"]["mutations_during_review"], 0);
    assert_eq!(state["state_integrity"]["production_contacted"], false);
    assert_eq!(
        public["world_state_revision"],
        state["world_state_revision"]
    );
    assert_eq!(public["release"]["release_id"], state["release_id"]);
    assert_eq!(
        public["release"]["image_digest"],
        state["build"]["subject_image_digest"]
    );
    assert_eq!(
        public["release"]["build_digest"],
        state["build"]["build_digest"]
    );
    assert_eq!(
        state["manifest_sha256"],
        digest(
            &fs::read(fixture_root().join("source/deployment-cohort-manifest-v1.json")).unwrap()
        )
    );

    let event_ids = manifest
        .planner_input
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    let index_by_id = manifest
        .planner_input
        .events
        .iter()
        .map(|event| (event.event_id.as_str(), event.sequence_index))
        .collect::<BTreeMap<_, _>>();
    for event in &manifest.planner_input.events {
        for reference in event
            .parent_event_ids
            .iter()
            .chain(&event.reference_event_ids)
        {
            assert!(
                event_ids.contains(reference.as_str()),
                "unresolved event reference {reference}"
            );
            assert!(
                index_by_id[reference.as_str()] < event.sequence_index,
                "dependency is not earlier than consumer: {reference} -> {}",
                event.event_id
            );
        }
    }
    for (index, action_id, event_id, result_id, path) in [
        (
            0,
            "verify_build_provenance",
            "e-build-provenance-result",
            "r-build-provenance-044",
            "bodies/receipt-build-provenance.txt",
        ),
        (
            1,
            "inspect_release_cohort_membership",
            "e-cohort-inspection-result",
            "r-cohort-inspection-044",
            "bodies/receipt-cohort-inspection.txt",
        ),
        (
            2,
            "verify_required_rollout_gates",
            "e-rollout-gates-result",
            "r-rollout-gates-044",
            "bodies/receipt-rollout-gates.txt",
        ),
        (
            3,
            "verify_rollback_artifact",
            "e-rollback-verification-result",
            "r-rollback-verification-044",
            "bodies/receipt-rollback-verification.txt",
        ),
    ] {
        let action = manifest
            .planner_input
            .events
            .iter()
            .find(|event| {
                event
                    .action
                    .as_ref()
                    .is_some_and(|action| action.action_id == action_id)
            })
            .unwrap();
        let result = manifest
            .planner_input
            .events
            .iter()
            .find(|event| event.event_id == event_id)
            .unwrap();
        assert_eq!(result.result.as_ref().unwrap().result_id, result_id);
        assert_eq!(
            result.result.as_ref().unwrap().originating_action_id,
            action_id
        );
        let body = fs::read(fixture_root().join(path)).unwrap();
        assert_eq!(
            result.result.as_ref().unwrap().observation_hash.as_deref(),
            Some(digest(&body).as_str())
        );
        let relation = manifest
            .planner_input
            .relations
            .iter()
            .find(|relation| relation.to_id == result_id)
            .unwrap();
        assert_eq!(relation.from_id, action_id);
        let descriptor = manifest
            .assets
            .iter()
            .find(|asset| asset.event_id.as_deref() == Some(event_id))
            .unwrap();
        assert_eq!(descriptor.relative_path, path);
        assert_eq!(descriptor.sha256, digest(&body));
        assert!(manifest
            .action_menu
            .iter()
            .any(|item| item.action_id == action_id
                && item.result_event_id == event_id
                && item.action_slot == if index < 2 { 1 } else { 2 }));
        assert!(action.sequence_index < result.sequence_index);
        assert_eq!(
            result.world_state_revision.as_deref(),
            Some("release-promotion@REL-SYN-2026-10-03-04-v1")
        );
    }

    let key_bytes = fs::read(fixture_root().join("evaluation/key.json")).unwrap();
    assert_eq!(digest(&key_bytes), manifest.evaluation_key_sha256);
    assert!(manifest
        .assets
        .iter()
        .all(|asset| asset.sha256 != manifest.evaluation_key_sha256));
    assert!(manifest
        .assets
        .iter()
        .all(|asset| asset.relative_path != manifest.evaluation_key_path));
    let visible = manifest
        .assets
        .iter()
        .map(|asset| {
            fs::read_to_string(fixture_root().join(&asset.relative_path)).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let key: Value = serde_json::from_slice(&key_bytes).unwrap();
    assert!(!visible.contains(&key["expected_final_answer"].to_string()));
    assert_eq!(key["required_event_ids"].as_array().unwrap().len(), 4);
    assert_eq!(key["critical_event_ids"].as_array().unwrap().len(), 4);
}

#[test]
fn deterministic_release_evaluator_matches_the_independently_derived_key() {
    let expected = independently_derived_answer();
    assert_eq!(expected, hidden_answer());
    assert_eq!(expected["decision"], "PROMOTE");
    assert_eq!(expected["governing_gate_ids"].as_array().unwrap().len(), 6);
    assert_eq!(
        expected["environment_cohort"]["deployment_unit_ids"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        expected["required_rollback_artifact_id"],
        "RB-SYN-REL-2026-10-03-04-PREV3"
    );
}

#[test]
fn all_three_canonical_offline_arms_pass_with_one_slot_three_manifest_prune() {
    let case = load_cp09();
    let answer = independently_derived_answer();
    let (mut baseline, baseline_requests) =
        execute_history(&case, Claim2ProjectionMode::Baseline, &answer);
    let (mut no_op, no_op_requests) = execute_history(&case, Claim2ProjectionMode::NoOp, &answer);
    let (mut intervention, intervention_requests) =
        execute_history(&case, Claim2ProjectionMode::Intervention, &answer);
    let baseline_bodies = baseline_requests
        .iter()
        .map(|request| &request.request_json)
        .collect::<Vec<_>>();
    let no_op_bodies = no_op_requests
        .iter()
        .map(|request| &request.request_json)
        .collect::<Vec<_>>();
    let intervention_bodies = intervention_requests
        .iter()
        .map(|request| &request.request_json)
        .collect::<Vec<_>>();
    assert_eq!(baseline_bodies, no_op_bodies);
    assert_eq!(&baseline_bodies[..2], &intervention_bodies[..2]);
    assert_ne!(baseline_bodies[2], intervention_bodies[2]);
    assert_eq!(
        compare_paired_arms(&mut baseline, &mut no_op, &mut intervention)
            .unwrap()
            .status,
        Claim2SlotStatus::Pass
    );
    for arm in [&baseline, &no_op, &intervention] {
        assert_eq!(
            evaluate_claim2_arm(&case, arm).unwrap().status,
            Claim2SlotStatus::Pass
        );
        assert_eq!(arm.slots().len(), 3);
        assert!(arm
            .slots()
            .iter()
            .all(|slot| slot.status == Claim2SlotStatus::Pass));
        assert_eq!(arm.environment_receipts().len(), 2);
    }
    let manifest_body = String::from_utf8(
        fs::read(fixture_root().join("source/deployment-cohort-manifest-v1.json")).unwrap(),
    )
    .unwrap();
    let baseline_content = serde_json::from_slice::<Value>(baseline_bodies[2]).unwrap()["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["content"].as_str().unwrap_or_default())
        .collect::<String>();
    let intervention_content = serde_json::from_slice::<Value>(intervention_bodies[2]).unwrap()
        ["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["content"].as_str().unwrap_or_default())
        .collect::<String>();
    assert_eq!(baseline_content.matches(&manifest_body).count(), 2);
    assert_eq!(intervention_content.matches(&manifest_body).count(), 1);
}

#[test]
fn canonical_advancing_domain_pins_two_exact_outputs_and_receipts() {
    let case = load_cp09();
    let domain: Value = parse("advancing-output-domain-v3.json");
    assert_eq!(domain["schema_version"], 3);
    assert_eq!(domain["case_id"], "CP09");
    assert_eq!(domain["offline_only"], true);
    assert_eq!(domain["points"].as_array().unwrap().len(), 2);
    for (index, action_id) in ["verify_build_provenance", "verify_required_rollout_gates"]
        .into_iter()
        .enumerate()
    {
        let point = &domain["points"][index];
        let raw = canonical_claim2_action_output(action_id);
        assert_eq!(point["request_slot"], index + 1);
        assert_eq!(point["expected_action_id"], action_id);
        assert_eq!(point["canonical_raw_utf8"], raw);
        assert_eq!(point["canonical_raw_sha256"], digest(raw.as_bytes()));
        assert_eq!(point["raw_language_cardinality"], 1);
        assert!(!raw.ends_with('\n'));
        let receipt =
            fs::read(fixture_root().join(point["receipt_path"].as_str().unwrap())).unwrap();
        assert_eq!(point["receipt_sha256"], digest(&receipt));
        let transition = case
            .manifest()
            .action_menu
            .iter()
            .find(|item| item.action_slot == (index + 1) as u8 && item.action_id == action_id)
            .unwrap();
        assert_eq!(point["receipt_event_id"], transition.result_event_id);
        assert_eq!(point["state_after"], transition.state_after);
    }
}

#[test]
fn malformed_action_output_stops_without_a_later_request() {
    let case = load_cp09();
    let canonical = canonical_claim2_action_output("verify_build_provenance");
    let invalid_outputs = [
        format!(" {canonical}"),
        format!("{canonical}\n"),
        "{\"action_id\": \"verify_build_provenance\"}".to_string(),
        "{\"action_id\":\"verify_build_provenance\",\"extra\":true}".to_string(),
    ];
    for raw in invalid_outputs {
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        render_claim2_request(&case, &mut arm).unwrap();
        let evaluation = arm.record_output(&case, raw.clone()).unwrap().unwrap();
        assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
        assert_eq!(
            arm.slots()[0].raw_assistant_output.as_deref(),
            Some(raw.as_str())
        );
        assert_eq!(
            arm.slots()[1].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        assert_eq!(
            arm.slots()[2].status,
            Claim2SlotStatus::NotExecutedAfterFailure
        );
        assert!(render_claim2_request(&case, &mut arm).is_err());
    }
}

#[test]
fn exact_structured_evaluator_rejects_mutated_release_identity_or_gate_set() {
    let case = load_cp09();
    for mutation in ["release_id", "governing_gate_ids"] {
        let mut answer = independently_derived_answer();
        if mutation == "release_id" {
            answer["release_id"] = json!("REL-SYN-OTHER");
        } else {
            answer["governing_gate_ids"][0] = json!("G-UNKNOWN-00");
        }
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        for action in ["verify_build_provenance", "verify_required_rollout_gates"] {
            render_claim2_request(&case, &mut arm).unwrap();
            assert!(arm
                .record_output(&case, canonical_claim2_action_output(action))
                .unwrap()
                .is_none());
        }
        render_claim2_request(&case, &mut arm).unwrap();
        let result = arm
            .record_output(&case, json!({"answer":answer}).to_string())
            .unwrap()
            .unwrap();
        assert_eq!(result.status, Claim2SlotStatus::Fail);
        assert!(result.final_answer_shape_valid);
        assert!(!result.final_answer_passed);
    }
}
