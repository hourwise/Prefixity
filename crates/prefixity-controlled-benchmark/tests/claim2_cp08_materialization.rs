#[path = "../examples/support/claim2_cp08_fixture.rs"]
mod fixture;

use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, compare_paired_arms, evaluate_claim2_arm, load_claim2_case,
    render_claim2_request, select_claim2_decision, Claim2ArmState, Claim2ProjectionMode,
    Claim2RenderedRequest, Claim2SlotStatus, LoadedClaim2Case, RelationType,
    ResearchInterventionClass,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixture_root() -> PathBuf {
    repository_root().join("fixtures/claim2/cp08")
}

fn load_cp08() -> LoadedClaim2Case {
    load_claim2_case(&fixture_root().join("case.json")).unwrap()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse(path: &str) -> Value {
    serde_json::from_slice(&fs::read(fixture_root().join(path)).unwrap()).unwrap()
}

fn minutes_since_midnight(timestamp: &str) -> i64 {
    let clock = timestamp.split('T').nth(1).unwrap().trim_end_matches('Z');
    let mut parts = clock.split(':').map(|part| part.parse::<i64>().unwrap());
    let hours = parts.next().unwrap();
    let minutes = parts.next().unwrap();
    let seconds = parts.next().unwrap();
    assert_eq!(seconds, 0);
    hours * 60 + minutes
}

fn final_answer() -> Value {
    parse("evaluation/key.json")["expected_final_answer"].clone()
}

fn execute_synthetic_history(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    answer: &Value,
) -> (Claim2ArmState, Vec<Claim2RenderedRequest>) {
    let mut arm = Claim2ArmState::new(case, mode);
    let mut requests = Vec::new();
    for (slot, action_id) in [
        "inspect_lot_logger_state",
        "calculate_calibration_and_exposure",
    ]
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
    let raw_final = serde_json::json!({ "answer": answer }).to_string();
    let evaluation = arm
        .record_output(case, raw_final)
        .unwrap()
        .expect("request 3 reaches deterministic evaluation");
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

fn independently_calculated_answer() -> Value {
    let lot = parse("source/lot-snapshot-LOT-SYN-CC08-117-v1.json");
    let logger = parse("source/logger-record-LOGGER-SYN-14-v1.json");
    let calibration = parse("source/calibration-CAL-SYN-14-2026-v1.json");
    let sop_text =
        fs::read_to_string(fixture_root().join("source/CC-SOP-04-revision-3.2.1.md")).unwrap();
    assert!(sop_text.contains("Document: CC-SOP-04\nRevision: 3.2.1"));
    assert!(sop_text.contains("no greater than 150 tenths of C-minute"));

    let source_date = lot["shipment"]["receipt_end"].as_str().unwrap();
    let cert = &calibration["as_found"];
    let correction = cert["reference_celsius_tenths"].as_i64().unwrap()
        - cert["indicated_celsius_tenths"].as_i64().unwrap();
    let error = correction.abs();
    let calibration_valid = calibration["logger_serial"] == lot["logger"]["logger_serial"]
        && calibration["calibration_id"] == lot["logger"]["calibration_id"]
        && calibration["issued_at"].as_str().unwrap()
            < lot["shipment"]["receipt_start"].as_str().unwrap()
        && calibration["valid_through"].as_str().unwrap() >= &source_date[..10]
        && !cert["adjustment_before_as_found"].as_bool().unwrap()
        && error
            <= cert["maximum_allowed_error_celsius_tenths"]
                .as_i64()
                .unwrap();
    assert!(calibration_valid);

    let interval_minutes = logger["sampling_interval_minutes"].as_i64().unwrap();
    assert_eq!(
        interval_minutes,
        lot["logger"]["required_interval_minutes"].as_i64().unwrap()
    );
    let mut total = 0_i64;
    let mut hot = 0_i64;
    let mut cold = 0_i64;
    let mut corrected_rows = Vec::new();
    let mut compliant = 0_i64;
    let mut high_count = 0_i64;
    let mut low_count = 0_i64;
    let rows = logger["intervals"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0]["start"], lot["shipment"]["receipt_start"]);
    assert_eq!(rows.last().unwrap()["end"], lot["shipment"]["receipt_end"]);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(
            minutes_since_midnight(row["end"].as_str().unwrap())
                - minutes_since_midnight(row["start"].as_str().unwrap()),
            interval_minutes
        );
        if index > 0 {
            assert_eq!(rows[index - 1]["end"], row["start"]);
        }
        let minimum = row["minimum_indicated_celsius_tenths"].as_i64().unwrap() + correction;
        let maximum = row["maximum_indicated_celsius_tenths"].as_i64().unwrap() + correction;
        let hot_excess = (maximum - 80).max(0);
        let cold_excess = (20 - minimum).max(0);
        let row_hot = hot_excess * interval_minutes;
        let row_cold = cold_excess * interval_minutes;
        hot += row_hot;
        cold += row_cold;
        total += (row_hot + row_cold > 0) as i64 * interval_minutes;
        let status = if row_hot > 0 {
            high_count += 1;
            "HIGH_EXCURSION"
        } else if row_cold > 0 {
            low_count += 1;
            "LOW_EXCURSION"
        } else {
            compliant += 1;
            "COMPLIANT"
        };
        corrected_rows.push(json!({
            "start":row["start"],"end":row["end"],
            "minimum_celsius_tenths":minimum,"maximum_celsius_tenths":maximum,"status":status
        }));
    }
    let disposition = if !calibration_valid || total > 30 || hot + cold > 150 {
        "ESCALATE"
    } else if total > 0 {
        "HOLD"
    } else {
        "RELEASE"
    };
    let reason = match disposition {
        "HOLD" => "One corrected high excursion lasted 15 minutes and totals 75 tenths of C-minute. Identity, calibration, and logger continuity pass; the exposure is within CC-08 review boundaries, so segregate the lot on HOLD pending quality review before use.",
        "RELEASE" => "All corrected intervals are inside the inclusive storage band and cumulative exposure is zero.",
        _ => "Calibration or source evidence failed, or cumulative exposure exceeded the escalation limits; retain HOLD and escalate to the quality lead.",
    };
    let rule_ids = match disposition {
        "HOLD" => vec!["CC-04", "CC-05", "CC-06", "CC-08"],
        "RELEASE" => vec!["CC-04", "CC-05", "CC-06", "CC-07"],
        _ => vec!["CC-04", "CC-05", "CC-06", "CC-09"],
    };
    json!({
        "disposition":disposition,
        "governing_rule_ids":rule_ids,
        "time_temperature_calculation":{
            "storage_range_celsius_tenths":{"minimum_inclusive":20,"maximum_inclusive":80},
            "interval_minutes":interval_minutes,"corrected_interval_extrema":corrected_rows,
            "compliant_interval_count":compliant,"high_excursion_interval_count":high_count,"low_excursion_interval_count":low_count
        },
        "cumulative_exposure_calculation":{"high_minutes":if high_count>0 {interval_minutes*high_count} else {0},"low_minutes":if low_count>0 {interval_minutes*low_count} else {0},"total_excursion_minutes":total,"high_degree_minutes_celsius_tenths":hot,"low_degree_minutes_celsius_tenths":cold,"total_degree_minutes_celsius_tenths":hot+cold},
        "calibration":{"status":if calibration_valid {"VALID"} else {"INVALID"},"calibration_id":calibration["calibration_id"],"reference_celsius_tenths":cert["reference_celsius_tenths"],"indicated_celsius_tenths":cert["indicated_celsius_tenths"],"correction_celsius_tenths":correction,"absolute_error_celsius_tenths":error,"maximum_allowed_error_celsius_tenths":cert["maximum_allowed_error_celsius_tenths"],"valid_through":calibration["valid_through"],"applied_correction_celsius_tenths":correction},
        "required_hold_or_escalation_reason":reason
    })
}

#[test]
fn source_generator_reproduces_every_cp08_fixture_byte() {
    let root = fixture_root();
    let generated = fixture::generated_files_for_test();
    assert!(generated.len() >= 20);
    for (relative, expected) in generated {
        let tracked = fs::read(root.join(&relative))
            .unwrap_or_else(|error| panic!("missing generated CP08 file {relative}: {error}"));
        assert_eq!(tracked, expected, "generated bytes differ at {relative}");
    }
}

#[test]
fn unchanged_policy_selects_the_only_natural_sop_reattachment() {
    let case = load_cp08();
    assert_eq!(case.case_id(), "CP08");
    let manifest = case.manifest();
    let selection = select_claim2_decision(&case).unwrap();
    assert_eq!(selection.decision.class, ResearchInterventionClass::Prune);
    assert_eq!(selection.decision.rule, "EXACT_DUPLICATE_PRUNE");
    assert_eq!(
        selection.decision.target_event_id.as_deref(),
        Some("e-sop-final-reattachment")
    );
    assert_eq!(selection.candidates.exact_duplicate_prune.len(), 1);
    assert!(selection.candidates.explicit_supersession_defer.is_empty());
    assert!(selection.candidates.same_zone_protocol_relocate.is_empty());

    let events = &manifest.planner_input.events;
    let target = events
        .iter()
        .find(|event| event.event_id == "e-sop-final-reattachment")
        .unwrap();
    let original = events
        .iter()
        .find(|event| event.event_id == "e-sop-original")
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
    assert_eq!(target.sequence_index, 12);

    let original_asset = manifest
        .assets
        .iter()
        .find(|asset| asset.event_id.as_deref() == Some("e-sop-original"))
        .unwrap();
    let final_asset = manifest
        .assets
        .iter()
        .find(|asset| asset.event_id.as_deref() == Some("e-sop-final-reattachment"))
        .unwrap();
    assert_eq!(original_asset.sha256, final_asset.sha256);
    assert_eq!(original_asset.revision_id, final_asset.revision_id);
    let original_bytes = fs::read(fixture_root().join("bodies/CC-SOP-04-original.md")).unwrap();
    let final_bytes = fs::read(fixture_root().join("bodies/CC-SOP-04-final-packet.md")).unwrap();
    let source_bytes = fs::read(fixture_root().join("source/CC-SOP-04-revision-3.2.1.md")).unwrap();
    assert_eq!(original_bytes, final_bytes);
    assert_eq!(original_bytes, source_bytes);
    assert_eq!(
        target.content_hash.as_deref(),
        Some(digest(&original_bytes).as_str())
    );

    let consumers = events
        .iter()
        .filter(|event| {
            event
                .parent_event_ids
                .iter()
                .chain(&event.reference_event_ids)
                .any(|reference| {
                    reference == &target.event_id
                        || reference == target.context_block_id.as_ref().unwrap()
                })
        })
        .count();
    let protected_relations = manifest
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
    assert_eq!(consumers, 0);
    assert_eq!(protected_relations, 0);
    let same_state = manifest
        .planner_input
        .relations
        .iter()
        .find(|relation| relation.relation_id == "same-state-sop-reattachment")
        .unwrap();
    assert_eq!(same_state.from_id, "e-sop-original");
    assert_eq!(same_state.to_id, "e-sop-final-reattachment");
    assert_eq!(same_state.relation_type, RelationType::SameStateRevision);

    let recipe = parse("source/quality-review-packet-recipe-v1.json");
    assert_eq!(recipe["ordinary_trigger"], "lot_disposition_review_opened");
    assert_eq!(recipe["benchmark_condition_used"], false);
    assert_eq!(
        recipe["attachment_behavior"],
        "emit_one_native_sop_message_for_every_final_lot_review"
    );
    assert_eq!(
        recipe["procedure_resolution"]["body_sha256"],
        final_asset.sha256
    );
    let workflow = fs::read_to_string(fixture_root().join("workflow.md")).unwrap();
    assert!(workflow.contains("routine controlled-record packet"));
    assert!(workflow.contains("does not inspect document size or use a benchmark condition"));

    let final_template = &manifest.request_templates[2];
    assert!(final_template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-sop-final-reattachment")));
    assert!(!manifest.request_templates[..2].iter().any(|template| template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-sop-final-reattachment"))));
    assert!(
        original_bytes.len() > 6_000,
        "the SOP should have a meaningful byte margin over the failed V1 positive bodies"
    );
}

#[test]
fn frozen_sources_receipts_provenance_and_dependency_bindings_close() {
    let case = load_cp08();
    let manifest = case.manifest();
    let event_ids = manifest
        .planner_input
        .events
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
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
        }
    }
    let lot_bytes =
        fs::read(fixture_root().join("source/lot-snapshot-LOT-SYN-CC08-117-v1.json")).unwrap();
    let logger_bytes =
        fs::read(fixture_root().join("source/logger-record-LOGGER-SYN-14-v1.json")).unwrap();
    let calibration_bytes =
        fs::read(fixture_root().join("source/calibration-CAL-SYN-14-2026-v1.json")).unwrap();
    assert_eq!(
        lot_bytes,
        fs::read(fixture_root().join("bodies/lot-snapshot-LOT-SYN-CC08-117-v1.json")).unwrap()
    );
    assert_eq!(
        logger_bytes,
        fs::read(fixture_root().join("bodies/logger-record-LOGGER-SYN-14-v1.json")).unwrap()
    );
    assert_eq!(
        calibration_bytes,
        fs::read(fixture_root().join("bodies/calibration-CAL-SYN-14-2026-v1.json")).unwrap()
    );
    let lot: Value = serde_json::from_slice(&lot_bytes).unwrap();
    let logger: Value = serde_json::from_slice(&logger_bytes).unwrap();
    let calibration: Value = serde_json::from_slice(&calibration_bytes).unwrap();
    assert_eq!(lot["synthetic_only"], true);
    assert_eq!(lot["record_state"], "frozen_read_only");
    assert_eq!(logger["synthetic_only"], true);
    assert_eq!(logger["export_state"], "immutable_captured_record");
    assert_eq!(calibration["synthetic_only"], true);
    assert_eq!(
        calibration["certificate_state"],
        "immutable_captured_record"
    );
    assert_eq!(lot["logger"]["logger_record_sha256"], digest(&logger_bytes));
    assert_eq!(
        lot["logger"]["calibration_record_sha256"],
        digest(&calibration_bytes)
    );
    assert_eq!(
        lot["procedure"]["body_sha256"],
        digest(&fs::read(fixture_root().join("source/CC-SOP-04-revision-3.2.1.md")).unwrap())
    );

    for (action_id, result_id, relation_id, receipt_path, index) in [
        (
            "inspect_lot_logger_state",
            "r-inspect-CC08-117",
            "produce-inspection",
            "bodies/receipt-inspection.txt",
            0,
        ),
        (
            "verify_receiving_record_completeness",
            "r-completeness-CC08-117",
            "produce-completeness",
            "bodies/receipt-completeness.txt",
            1,
        ),
        (
            "calculate_calibration_and_exposure",
            "r-exposure-CC08-117",
            "produce-exposure-check",
            "bodies/receipt-exposure-check.txt",
            2,
        ),
        (
            "verify_calibration_certificate",
            "r-calibration-CC08-117",
            "produce-calibration-check",
            "bodies/receipt-calibration-check.txt",
            3,
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
                    .is_some_and(|identity| identity.action_id == action_id)
            })
            .unwrap();
        let result = manifest
            .planner_input
            .events
            .iter()
            .find(|event| {
                event
                    .result
                    .as_ref()
                    .is_some_and(|identity| identity.result_id == result_id)
            })
            .unwrap();
        let link = manifest
            .planner_input
            .relations
            .iter()
            .find(|relation| relation.relation_id == relation_id)
            .unwrap();
        let receipt = fs::read(fixture_root().join(receipt_path)).unwrap();
        assert_eq!(link.from_id, action_id);
        assert_eq!(link.to_id, result_id);
        assert_eq!(
            result.result.as_ref().unwrap().originating_action_id,
            action_id
        );
        assert_eq!(
            result.result.as_ref().unwrap().observation_hash.as_deref(),
            Some(digest(&receipt).as_str())
        );
        assert!(action.sequence_index < result.sequence_index);
        assert_eq!(
            result.world_state_revision,
            Some("cold-chain-disposition@LOT-SYN-CC08-117-v1".to_string())
        );
        let descriptor = manifest
            .assets
            .iter()
            .find(|asset| asset.event_id.as_deref() == Some(result.event_id.as_str()))
            .unwrap();
        assert_eq!(descriptor.sha256, digest(&receipt));
        assert_eq!(descriptor.relative_path, receipt_path);
        assert!(manifest
            .action_menu
            .iter()
            .any(|transition| transition.action_id == action_id
                && transition.result_event_id == result.event_id
                && transition.result_asset_id == descriptor.asset_id
                && transition.action_slot == if index < 2 { 1 } else { 2 }));
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
        .filter_map(|asset| fs::read_to_string(fixture_root().join(&asset.relative_path)).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let key: Value = serde_json::from_slice(&key_bytes).unwrap();
    assert!(!visible.contains(&key["expected_final_answer"].to_string()));
    assert_eq!(key["required_event_ids"].as_array().unwrap().len(), 6);
}

#[test]
fn exact_calibration_temperature_and_cumulative_exposure_arithmetic_matches_the_hidden_key() {
    let expected = independently_calculated_answer();
    assert_eq!(expected, final_answer());
    assert_eq!(expected["disposition"], "HOLD");
    assert_eq!(
        expected["time_temperature_calculation"]["high_excursion_interval_count"],
        1
    );
    assert_eq!(
        expected["cumulative_exposure_calculation"]["total_excursion_minutes"],
        15
    );
    assert_eq!(
        expected["cumulative_exposure_calculation"]["total_degree_minutes_celsius_tenths"],
        75
    );
    assert_eq!(expected["calibration"]["status"], "VALID");
    assert!(expected["required_hold_or_escalation_reason"]
        .as_str()
        .unwrap()
        .contains("HOLD pending quality review"));
}

#[test]
fn all_three_canonical_offline_arms_pass_with_a_single_slot_three_sop_omission() {
    let case = load_cp08();
    let answer = final_answer();
    let (mut baseline, baseline_requests) =
        execute_synthetic_history(&case, Claim2ProjectionMode::Baseline, &answer);
    let (mut no_op, no_op_requests) =
        execute_synthetic_history(&case, Claim2ProjectionMode::NoOp, &answer);
    let (mut intervention, intervention_requests) =
        execute_synthetic_history(&case, Claim2ProjectionMode::Intervention, &answer);
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
    let comparison = compare_paired_arms(&mut baseline, &mut no_op, &mut intervention).unwrap();
    assert_eq!(comparison.status, Claim2SlotStatus::Pass);
    assert!(comparison.baseline_noop_match);
    assert!(comparison.intervention_pre_treatment_match);
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
    let source =
        fs::read_to_string(fixture_root().join("source/CC-SOP-04-revision-3.2.1.md")).unwrap();
    let request3_baseline: Value = serde_json::from_slice(baseline_bodies[2]).unwrap();
    let request3_intervention: Value = serde_json::from_slice(intervention_bodies[2]).unwrap();
    let baseline_content = request3_baseline["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["content"].as_str().unwrap_or_default())
        .collect::<String>();
    let intervention_content = request3_intervention["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["content"].as_str().unwrap_or_default())
        .collect::<String>();
    assert_eq!(baseline_content.matches(&source).count(), 2);
    assert_eq!(intervention_content.matches(&source).count(), 1);
}

#[test]
fn canonical_advancing_domain_pins_the_two_exact_outputs_and_receipts() {
    let case = load_cp08();
    let root = fixture_root();
    let domain: Value =
        serde_json::from_slice(&fs::read(root.join("advancing-output-domain-v2.json")).unwrap())
            .unwrap();
    assert_eq!(domain["case_id"], "CP08");
    assert_eq!(domain["offline_only"], true);
    let points = domain["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
    for (index, action_id) in [
        "inspect_lot_logger_state",
        "calculate_calibration_and_exposure",
    ]
    .into_iter()
    .enumerate()
    {
        let point = &points[index];
        let raw = canonical_claim2_action_output(action_id);
        assert_eq!(point["request_slot"], index + 1);
        assert_eq!(point["expected_action_id"], action_id);
        assert_eq!(point["canonical_raw_utf8"], raw);
        assert_eq!(point["canonical_raw_sha256"], digest(raw.as_bytes()));
        assert_eq!(point["raw_language_cardinality"], 1);
        assert!(!raw.ends_with('\n'));
        let receipt = fs::read(root.join(point["receipt_path"].as_str().unwrap())).unwrap();
        assert_eq!(point["receipt_sha256"], digest(&receipt));
        let transition = case
            .manifest()
            .action_menu
            .iter()
            .find(|transition| {
                transition.action_slot == (index + 1) as u8 && transition.action_id == action_id
            })
            .unwrap();
        assert_eq!(point["receipt_event_id"], transition.result_event_id);
        assert_eq!(point["state_after"], transition.state_after);
    }
}

#[test]
fn noncanonical_advancing_spelling_fails_without_running_later_requests() {
    let case = load_cp08();
    let canonical = canonical_claim2_action_output("inspect_lot_logger_state");
    for raw in [
        format!(" {canonical}"),
        format!("{canonical}\n"),
        "{\"action_id\": \"inspect_lot_logger_state\"}".to_string(),
        "{\"action_id\":\"inspect_lot_logger_state\",\"extra\":true}".to_string(),
    ] {
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
fn exact_structured_evaluator_rejects_mutated_exposure_or_calibration_facts() {
    let case = load_cp08();
    for pointer in [
        "/cumulative_exposure_calculation/total_degree_minutes_celsius_tenths",
        "/calibration/absolute_error_celsius_tenths",
    ] {
        let mut answer = final_answer();
        let current = answer.pointer(pointer).unwrap().as_i64().unwrap();
        answer
            .pointer_mut(pointer)
            .unwrap()
            .clone_from(&json!(current + 1));
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        for action_id in [
            "inspect_lot_logger_state",
            "calculate_calibration_and_exposure",
        ] {
            render_claim2_request(&case, &mut arm).unwrap();
            assert!(arm
                .record_output(&case, canonical_claim2_action_output(action_id))
                .unwrap()
                .is_none());
        }
        render_claim2_request(&case, &mut arm).unwrap();
        let output = serde_json::json!({"answer":answer}).to_string();
        let evaluation = arm.record_output(&case, output).unwrap().unwrap();
        assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
        assert!(evaluation.final_answer_shape_valid);
        assert!(!evaluation.final_answer_passed);
    }
}
