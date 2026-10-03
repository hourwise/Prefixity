#[path = "../examples/support/claim2_cp10_fixture.rs"]
mod fixture;

use prefixity_controlled_benchmark::{
    apply_claim2_decision, canonical_claim2_action_output, compare_paired_arms,
    evaluate_claim2_arm, load_claim2_case, render_claim2_request, select_claim2_decision,
    Claim2ArmState, Claim2ProjectionMode, Claim2RenderedRequest, Claim2SlotStatus,
    LoadedClaim2Case, RelationType, ResearchInterventionClass,
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
    repository_root().join("fixtures/claim2/cp10")
}

fn load_cp10() -> LoadedClaim2Case {
    load_claim2_case(&fixture_root().join("case.json")).unwrap()
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn parse(relative: &str) -> Value {
    serde_json::from_slice(&fs::read(fixture_root().join(relative)).unwrap()).unwrap()
}

fn final_answer() -> Value {
    parse("evaluation/key.json")["expected_final_answer"].clone()
}

fn run_arm(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    answer: &Value,
) -> (Claim2ArmState, Vec<Claim2RenderedRequest>) {
    let mut arm = Claim2ArmState::new(case, mode);
    let mut requests = Vec::new();
    for (slot, action_id) in [
        "inspect_observation_catalog",
        "inspect_quality_and_coverage",
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
    let request = render_claim2_request(case, &mut arm).unwrap();
    assert_eq!(request.request_slot, 3);
    assert!(!request.dispatchable);
    requests.push(request);
    let output = json!({"answer":answer}).to_string();
    let evaluation = arm
        .record_output(case, output)
        .unwrap()
        .expect("third offline output reaches exact evaluator");
    assert_eq!(evaluation.status, Claim2SlotStatus::Pass);
    assert_eq!(evaluation.model_status, Some(Claim2SlotStatus::Pass));
    assert!(evaluation.action_checks_passed);
    assert!(evaluation.intermediate_state_checks_passed);
    assert!(evaluation.receipt_identity_checks_passed);
    assert!(evaluation.final_answer_passed);
    assert!(evaluation.required_context_preserved);
    assert!(evaluation.required_relations_preserved);
    assert!(evaluation.critical_events_preserved);
    assert!(evaluation.structurally_complete);
    (arm, requests)
}

fn normalize_rfc3339_to_utc(timestamp: &str) -> String {
    let (date, time_with_zone) = timestamp.split_once('T').unwrap();
    let (clock, offset_minutes) = if let Some(clock) = time_with_zone.strip_suffix('Z') {
        (clock, 0_i32)
    } else {
        let split_at = time_with_zone
            .char_indices()
            .skip(1)
            .find_map(|(index, ch)| matches!(ch, '+' | '-').then_some(index))
            .unwrap();
        let (clock, offset) = time_with_zone.split_at(split_at);
        let sign = if offset.starts_with('+') { 1 } else { -1 };
        let mut pieces = offset[1..].split(':');
        let hours: i32 = pieces.next().unwrap().parse().unwrap();
        let minutes: i32 = pieces.next().unwrap().parse().unwrap();
        assert!(pieces.next().is_none());
        (clock, sign * (hours * 60 + minutes))
    };
    let mut clock = clock.split(':').map(|piece| piece.parse::<i32>().unwrap());
    let hour = clock.next().unwrap();
    let minute = clock.next().unwrap();
    let second = clock.next().unwrap();
    assert_eq!(second, 0);
    let utc_minutes = (hour * 60 + minute - offset_minutes).rem_euclid(1440);
    // All CP10 timestamps remain on this source UTC date after normalization.
    assert!((7..=9).contains(&(utc_minutes / 60)));
    format!("{date}T{:02}:{:02}:00Z", utc_minutes / 60, utc_minutes % 60)
}

fn independently_calculated_answer() -> Value {
    let mut groups = BTreeMap::<(String, String), Vec<(String, i64)>>::new();
    let mut flagged = Vec::new();
    let mut all_source_ids = Vec::new();
    for index in 1..=3 {
        let shard: Value = parse(&format!("source/shard-OBS-SYN-CP10-S0{index}-v1.json"));
        all_source_ids.push(shard["shard_id"].as_str().unwrap().to_string());
        for observation in shard["observations"].as_array().unwrap() {
            let id = observation["observation_id"].as_str().unwrap();
            let instrument = observation["instrument_id"].as_str().unwrap();
            let normalized = normalize_rfc3339_to_utc(observation["observed_at"].as_str().unwrap());
            match observation["value_celsius_tenths"].as_i64() {
                None => flagged.push(json!({
                    "observation_id":id,"normalized_utc":normalized,"quality_flag":observation["quality_flag"],
                    "rule_id":"QF-01","disposition":"EXCLUDED_NULL_MEASUREMENT"
                })),
                Some(_) if observation["quality_flag"] != "VALID" => flagged.push(json!({
                    "observation_id":id,"normalized_utc":normalized,"quality_flag":observation["quality_flag"],
                    "rule_id":"QF-02","disposition":"EXCLUDED_NON_VALID_SOURCE_QUALITY"
                })),
                Some(value) => groups
                    .entry((instrument.to_string(), normalized[..10].to_string()))
                    .or_default()
                    .push((id.to_string(), value)),
            }
        }
    }
    flagged.sort_by(|a, b| {
        a["observation_id"]
            .as_str()
            .cmp(&b["observation_id"].as_str())
    });
    all_source_ids.sort();
    let rows = groups
        .into_iter()
        .map(|((instrument_id, utc_date), mut values)| {
            values.sort_by(|a, b| a.0.cmp(&b.0));
            let count = values.len() as i64;
            let sum = values.iter().map(|(_, value)| *value).sum::<i64>();
            let mean = (sum * 2 + count) / (2 * count);
            let minimum = values.iter().map(|(_, value)| *value).min().unwrap();
            let maximum = values.iter().map(|(_, value)| *value).max().unwrap();
            json!({
                "instrument_id":instrument_id,"utc_date":utc_date,"unit":"0.1 degree Celsius",
                "valid_observation_count":count,"mean_value_celsius_tenths":mean,
                "minimum_value_celsius_tenths":minimum,"maximum_value_celsius_tenths":maximum,
                "source_observation_ids":values.iter().map(|(id,_)| id.clone()).collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    json!({
        "publication_status":"PUBLISHED_WITH_FLAGS","dataset_id":"OBS-SYN-CP10","output_rows":rows,
        "flagged_observations":flagged,"source_ids":all_source_ids,"quality_rule_ids":["QF-01","QF-02"],
        "publication_reason":"Publish ten valid observations in two instrument-day rows. Exclude the one SUSPECT measurement under QF-02 and report the one null measurement under QF-01; preserve both flags in the publication record."
    })
}

#[test]
fn source_generator_reproduces_every_cp10_fixture_byte() {
    let root = fixture_root();
    let generated = fixture::generated_files_for_test();
    assert!(generated.len() >= 20);
    for (relative, expected) in generated {
        let tracked = fs::read(root.join(&relative))
            .unwrap_or_else(|error| panic!("missing generated CP10 file {relative}: {error}"));
        assert_eq!(tracked, expected, "generated bytes differ at {relative}");
    }
}

#[test]
fn unchanged_policy_selects_only_the_natural_specification_reattachment() {
    let case = load_cp10();
    assert_eq!(case.case_id(), "CP10");
    let manifest = case.manifest();
    let selection = select_claim2_decision(&case).unwrap();
    assert_eq!(selection.decision.class, ResearchInterventionClass::Prune);
    assert_eq!(selection.decision.rule, "EXACT_DUPLICATE_PRUNE");
    assert_eq!(
        selection.decision.target_event_id.as_deref(),
        Some("e-spec-final-reattachment")
    );
    assert_eq!(selection.candidates.exact_duplicate_prune.len(), 1);
    assert!(selection.candidates.explicit_supersession_defer.is_empty());
    assert!(selection.candidates.same_zone_protocol_relocate.is_empty());

    let events = &manifest.planner_input.events;
    let target = events
        .iter()
        .find(|event| event.event_id == "e-spec-final-reattachment")
        .unwrap();
    let original = events
        .iter()
        .find(|event| event.event_id == "e-spec-original")
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

    let spec_source =
        fs::read(fixture_root().join("source/field-dictionary-aggregation-v1.json")).unwrap();
    let original_bytes =
        fs::read(fixture_root().join("bodies/field-dictionary-original.json")).unwrap();
    let final_bytes =
        fs::read(fixture_root().join("bodies/field-dictionary-final-packet.json")).unwrap();
    assert_eq!(spec_source, original_bytes);
    assert_eq!(original_bytes, final_bytes);
    assert_eq!(
        target.content_hash.as_deref(),
        Some(digest(&spec_source).as_str())
    );
    assert!(target
        .provenance
        .iter()
        .any(|source| source.source_revision.as_deref()
            == Some("field-dictionary-aggregation@1.0.0")));

    let consumers = events
        .iter()
        .filter(|event| {
            event
                .parent_event_ids
                .iter()
                .chain(&event.reference_event_ids)
                .any(|reference| {
                    reference == &target.event_id
                        || Some(reference.as_str()) == target.context_block_id.as_deref()
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
        .find(|relation| relation.relation_id == "same-state-spec-reattachment")
        .unwrap();
    assert_eq!(same_state.from_id, "e-spec-original");
    assert_eq!(same_state.to_id, "e-spec-final-reattachment");
    assert_eq!(same_state.relation_type, RelationType::SameStateRevision);

    let recipe = parse("source/publication-packet-recipe-v1.json");
    assert_eq!(
        recipe["ordinary_trigger"],
        "publication_packet_opened_for_final_rollup"
    );
    assert_eq!(recipe["benchmark_condition_used"], false);
    assert_eq!(recipe["attachment_behavior"], "attach_the_effective_field_dictionary_and_aggregation_specification_as_one_native_message_to_every_final_publication_packet");
    assert_eq!(
        recipe["specification_resolution"]["sha256"],
        digest(&spec_source)
    );
    assert_eq!(recipe["specification_resolution"]["transformation"], "none");
    let workflow = fs::read_to_string(fixture_root().join("workflow.md")).unwrap();
    assert!(workflow.contains("regardless of document size, arm, or benchmark state"));
    assert!(workflow.contains("does not transform or summarize that object"));
    let final_template = &manifest.request_templates[2];
    assert!(final_template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-spec-final-reattachment")));
    assert!(!manifest.request_templates[..2].iter().any(|template| template.messages.iter().flat_map(|message| &message.parts).any(|part| matches!(part, prefixity_controlled_benchmark::Claim2PromptPart::EventBody { event_id } if event_id == "e-spec-final-reattachment"))));
}

#[test]
fn frozen_collection_provenance_receipts_and_dependency_graph_close() {
    let case = load_cp10();
    let manifest = case.manifest();
    let by_id = manifest
        .planner_input
        .events
        .iter()
        .map(|event| (event.event_id.as_str(), event))
        .collect::<BTreeMap<_, _>>();
    for event in &manifest.planner_input.events {
        for reference in event
            .parent_event_ids
            .iter()
            .chain(&event.reference_event_ids)
        {
            let dependency = by_id
                .get(reference.as_str())
                .unwrap_or_else(|| panic!("unresolved event reference {reference}"));
            assert!(
                dependency.sequence_index < event.sequence_index,
                "non-acyclic dependency {reference} -> {}",
                event.event_id
            );
        }
    }
    let spec = parse("source/field-dictionary-aggregation-v1.json");
    assert_eq!(spec["synthetic_only"], true);
    assert_eq!(spec["collection"]["shard_count"], 3);
    assert_eq!(spec["collection"]["expected_observation_count"], 12);
    assert_eq!(spec["fields"].as_array().unwrap().len(), 5);
    assert_eq!(spec["quality_rules"].as_array().unwrap().len(), 2);
    let specified_fields = spec["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|field| field["field_id"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    let source_fields = parse("source/shard-OBS-SYN-CP10-S01-v1.json")["observations"][0]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(specified_fields, source_fields);
    assert!(spec["fields"]
        .as_array()
        .unwrap()
        .iter()
        .any(|field| field["field_id"] == "observed_at"
            && field["timezone_rule"]
                == "offset is mandatory; normalize the represented instant to UTC before grouping or flag reporting"));
    assert_eq!(
        spec["aggregation"]["mean"],
        "sum integer tenths divided by count, rounded to nearest integer with exact halves away from zero (round-half-up for nonnegative observations)"
    );
    let collection = parse("source/collection-manifest-v1.json");
    assert_eq!(collection["closed_world"], true);
    assert_eq!(collection["shards"].as_array().unwrap().len(), 3);
    assert_eq!(collection["total_observations"], 12);

    for index in 1..=3 {
        let source_path = format!("source/shard-OBS-SYN-CP10-S0{index}-v1.json");
        let body_path = format!("bodies/shard-OBS-SYN-CP10-S0{index}-v1.json");
        let source = fs::read(fixture_root().join(&source_path)).unwrap();
        assert_eq!(source, fs::read(fixture_root().join(body_path)).unwrap());
        let shard: Value = serde_json::from_slice(&source).unwrap();
        assert_eq!(shard["synthetic_only"], true);
        assert_eq!(shard["record_state"], "immutable_frozen");
        assert_eq!(shard["observations"].as_array().unwrap().len(), 4);
        let descriptor = collection["shards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["shard_id"] == shard["shard_id"])
            .unwrap();
        assert_eq!(descriptor["sha256"], digest(&source));
    }

    for (action_id, result_id, relation_id, receipt_path, index) in [
        (
            "inspect_observation_catalog",
            "r-catalog-OBS-SYN-CP10",
            "produce-source-catalog",
            "bodies/receipt-source-catalog.json",
            0,
        ),
        (
            "verify_shard_inventory",
            "r-inventory-OBS-SYN-CP10",
            "produce-shard-inventory",
            "bodies/receipt-shard-inventory.json",
            1,
        ),
        (
            "inspect_quality_and_coverage",
            "r-quality-OBS-SYN-CP10",
            "produce-quality-gaps",
            "bodies/receipt-quality-gaps.json",
            2,
        ),
        (
            "verify_timezone_and_units",
            "r-timezone-OBS-SYN-CP10",
            "produce-timezone-units",
            "bodies/receipt-timezone-units.json",
            3,
        ),
    ] {
        let action_event = manifest
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
        let result_event = manifest
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
        let relation = manifest
            .planner_input
            .relations
            .iter()
            .find(|relation| relation.relation_id == relation_id)
            .unwrap();
        let receipt = fs::read(fixture_root().join(receipt_path)).unwrap();
        assert_eq!(relation.from_id, action_id);
        assert_eq!(relation.to_id, result_id);
        assert_eq!(
            result_event.result.as_ref().unwrap().originating_action_id,
            action_id
        );
        assert_eq!(
            result_event
                .result
                .as_ref()
                .unwrap()
                .observation_hash
                .as_deref(),
            Some(digest(&receipt).as_str())
        );
        assert!(action_event.sequence_index < result_event.sequence_index);
        assert_eq!(
            result_event.world_state_revision.as_deref(),
            Some("observation-publication@OBS-SYN-CP10-v1")
        );
        let descriptor = manifest
            .assets
            .iter()
            .find(|asset| asset.event_id.as_deref() == Some(result_event.event_id.as_str()))
            .unwrap();
        assert_eq!(descriptor.sha256, digest(&receipt));
        assert_eq!(descriptor.relative_path, receipt_path);
        let transition = manifest
            .action_menu
            .iter()
            .find(|transition| transition.action_id == action_id)
            .unwrap();
        assert_eq!(transition.result_event_id, result_event.event_id);
        assert_eq!(transition.result_asset_id, descriptor.asset_id);
        assert_eq!(transition.action_slot, if index < 2 { 1 } else { 2 });
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
    assert_eq!(key["critical_event_ids"].as_array().unwrap().len(), 5);
}

#[test]
fn exact_timezone_quality_and_integer_aggregation_matches_the_hidden_answer() {
    let expected = independently_calculated_answer();
    assert_eq!(expected, final_answer());
    assert_eq!(expected["publication_status"], "PUBLISHED_WITH_FLAGS");
    assert_eq!(expected["output_rows"].as_array().unwrap().len(), 2);
    assert_eq!(expected["output_rows"][0]["mean_value_celsius_tenths"], 215);
    assert_eq!(expected["output_rows"][0]["valid_observation_count"], 4);
    assert_eq!(expected["output_rows"][1]["mean_value_celsius_tenths"], 201);
    assert_eq!(expected["output_rows"][1]["valid_observation_count"], 6);
    assert_eq!(
        expected["flagged_observations"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        expected["flagged_observations"][0]["normalized_utc"],
        "2026-01-14T07:30:00Z"
    );
}

#[test]
fn all_three_canonical_offline_arms_pass_with_one_slot_three_specification_omission() {
    let case = load_cp10();
    let answer = final_answer();
    let (mut baseline, baseline_requests) = run_arm(&case, Claim2ProjectionMode::Baseline, &answer);
    let (mut no_op, no_op_requests) = run_arm(&case, Claim2ProjectionMode::NoOp, &answer);
    let (mut intervention, intervention_requests) =
        run_arm(&case, Claim2ProjectionMode::Intervention, &answer);
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

    let intervention_projection =
        apply_claim2_decision(&case, Claim2ProjectionMode::Intervention).unwrap();
    assert_eq!(
        intervention_projection
            .actual_selection
            .decision
            .target_event_id
            .as_deref(),
        Some("e-spec-final-reattachment")
    );
    assert!(intervention_projection
        .planner_input
        .events
        .iter()
        .any(|event| event.event_id == "e-spec-original"));
    assert!(!intervention_projection
        .planner_input
        .events
        .iter()
        .any(|event| event.event_id == "e-spec-final-reattachment"));
    let baseline_request3: Value = serde_json::from_slice(baseline_bodies[2]).unwrap();
    let intervention_request3: Value = serde_json::from_slice(intervention_bodies[2]).unwrap();
    let baseline_content = baseline_request3["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["content"].as_str().unwrap_or_default())
        .collect::<String>();
    let intervention_content = intervention_request3["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|message| message["content"].as_str().unwrap_or_default())
        .collect::<String>();
    let spec =
        fs::read_to_string(fixture_root().join("source/field-dictionary-aggregation-v1.json"))
            .unwrap();
    assert_eq!(baseline_content.matches(&spec).count(), 2);
    assert_eq!(intervention_content.matches(&spec).count(), 1);
}

#[test]
fn v3_advancing_domain_pins_exact_output_and_receipt_bytes() {
    let case = load_cp10();
    let domain = parse("advancing-output-domain-v3.json");
    assert_eq!(domain["case_id"], "CP10");
    assert_eq!(domain["schema_version"], 3);
    assert_eq!(domain["offline_only"], true);
    let points = domain["points"].as_array().unwrap();
    assert_eq!(points.len(), 2);
    for (index, action_id) in [
        "inspect_observation_catalog",
        "inspect_quality_and_coverage",
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
        let receipt =
            fs::read(fixture_root().join(point["receipt_path"].as_str().unwrap())).unwrap();
        assert_eq!(point["receipt_sha256"], digest(&receipt));
        let transition = case
            .manifest()
            .action_menu
            .iter()
            .find(|transition| {
                transition.action_slot == index as u8 + 1 && transition.action_id == action_id
            })
            .unwrap();
        assert_eq!(point["receipt_event_id"], transition.result_event_id);
        assert_eq!(point["state_after"], transition.state_after);
    }
}

#[test]
fn exact_final_evaluator_rejects_changed_aggregate_or_quality_flag() {
    let case = load_cp10();
    for pointer in [
        "/output_rows/0/mean_value_celsius_tenths",
        "/flagged_observations/0/rule_id",
    ] {
        let mut answer = final_answer();
        if pointer.ends_with("rule_id") {
            answer
                .pointer_mut(pointer)
                .unwrap()
                .clone_from(&json!("QF-01"));
        } else {
            let value = answer.pointer(pointer).unwrap().as_i64().unwrap();
            answer
                .pointer_mut(pointer)
                .unwrap()
                .clone_from(&json!(value + 1));
        }
        let mut arm = Claim2ArmState::new(&case, Claim2ProjectionMode::Baseline);
        for action_id in [
            "inspect_observation_catalog",
            "inspect_quality_and_coverage",
        ] {
            render_claim2_request(&case, &mut arm).unwrap();
            assert!(arm
                .record_output(&case, canonical_claim2_action_output(action_id))
                .unwrap()
                .is_none());
        }
        render_claim2_request(&case, &mut arm).unwrap();
        let raw = json!({"answer":answer}).to_string();
        let evaluation = arm.record_output(&case, raw).unwrap().unwrap();
        assert_eq!(evaluation.status, Claim2SlotStatus::Fail);
        assert!(evaluation.final_answer_shape_valid);
        assert!(!evaluation.final_answer_passed);
    }
}
