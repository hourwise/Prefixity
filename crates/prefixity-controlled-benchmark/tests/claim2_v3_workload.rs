#[path = "../examples/support/claim2_v3_workload.rs"]
mod workload;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn ledger() -> Value {
    let root = root();
    let regenerated = workload::ledger_bytes(&root).unwrap();
    assert_eq!(
        regenerated,
        fs::read(root.join(workload::LEDGER_PATH)).unwrap()
    );
    serde_json::from_slice(&regenerated).unwrap()
}

#[test]
fn cohort_finite_domain_reproduces_twelve_single_spellings() {
    let root = root();
    let bytes = workload::domain_bytes(&root).unwrap();
    assert_eq!(bytes, fs::read(root.join(workload::DOMAIN_PATH)).unwrap());
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let points = value["transitions"].as_array().unwrap();
    assert_eq!(points.len(), 12);
    for point in points {
        let raw = point["canonical_raw_response"].as_str().unwrap();
        let expected = format!(
            "{{\"action_id\":\"{}\"}}",
            point["expected_action_id"].as_str().unwrap()
        );
        assert_eq!(raw, expected);
        assert_eq!(digest(raw.as_bytes()), point["canonical_response_sha256"]);
        assert_eq!(point["advancing_raw_language_cardinality"], 1);
    }
}

#[test]
fn complete_cohort_ledger_reproduces_exact_rendered_bytes_and_inheritance() {
    let value = ledger();
    assert_eq!(
        value["cohort_order"],
        serde_json::json!(["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"])
    );
    assert_eq!(value["deduplication"]["logical_request_count"], 54);
    assert_eq!(
        value["deduplication"]["inherited_v1_unique_request_hashes"],
        14
    );
    assert_eq!(value["deduplication"]["inherited_v1_logical_requests"], 36);
    assert_eq!(value["offline_boundary"]["server_contacts"], 0);
    assert_eq!(value["offline_boundary"]["inference_requests"], 0);
    assert_eq!(value["offline_boundary"]["tokenization_performed"], false);

    let rows = value["requests"].as_array().unwrap();
    let mut ids = BTreeSet::new();
    let mut groups: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for row in rows {
        let id = row["logical_request_id"].as_str().unwrap();
        assert!(ids.insert(id));
        let body = row["future_token_counter_body"].as_str().unwrap();
        let hash = row["request_body_sha256"].as_str().unwrap();
        assert_eq!(digest(body.as_bytes()), hash, "{id}");
        assert_eq!(
            body.len() as u64,
            row["request_body_utf8_byte_length"],
            "{id}"
        );
        let parsed: Value = serde_json::from_str(body).unwrap();
        assert_eq!(parsed["messages"], row["messages"], "{id}");
        assert!(parsed.get("chat_template_kwargs").is_none());
        assert_eq!(row["renderer_dispatchable"], false);
        assert_eq!(row["live_dispatchable"], false);
        assert_eq!(row["unbound_raw_assistant_slots"], serde_json::json!([]));
        assert_eq!(
            row["unbound_environment_receipt_slots"],
            serde_json::json!([])
        );
        for prior in row["canonical_prior_output_identities"].as_array().unwrap() {
            let raw = prior["canonical_response"].as_str().unwrap();
            assert_eq!(digest(raw.as_bytes()), prior["canonical_response_sha256"]);
            assert!(row["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|message| { message["role"] == "assistant" && message["content"] == raw }));
        }
        groups.entry(hash.to_owned()).or_default().push(row);
    }
    assert_eq!(ids.len(), 54);
    assert_eq!(
        value["deduplication"]["unique_request_body_count"],
        groups.len()
    );
    assert_eq!(
        value["deduplication"]["new_unique_request_hashes_requiring_tokenization"],
        groups.len() - 14
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT")
            .count(),
        36
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["evidence_classification"] == "NEW_TOKEN_COUNT_REQUIRED")
            .count(),
        18
    );
    for members in groups.values() {
        let first = members[0]["future_token_counter_body"].as_str().unwrap();
        assert!(members
            .iter()
            .all(|row| row["future_token_counter_body"] == first));
    }
}

#[test]
fn new_cases_have_one_slot_three_omission_and_retained_controls_are_exact() {
    let value = ledger();
    let rows = value["requests"].as_array().unwrap();
    for case_id in ["CP09", "CP10"] {
        let for_id = |arm: &str, slot: u64| {
            rows.iter()
                .find(|row| {
                    row["case_id"] == case_id && row["arm"] == arm && row["request_slot"] == slot
                })
                .unwrap()
        };
        for slot in 1..=2 {
            assert_eq!(
                for_id("BASELINE", slot)["future_token_counter_body"],
                for_id("NO_OP", slot)["future_token_counter_body"]
            );
            assert_eq!(
                for_id("BASELINE", slot)["future_token_counter_body"],
                for_id("INTERVENTION", slot)["future_token_counter_body"]
            );
        }
        assert_eq!(
            for_id("BASELINE", 3)["future_token_counter_body"],
            for_id("NO_OP", 3)["future_token_counter_body"]
        );
        assert_ne!(
            for_id("BASELINE", 3)["future_token_counter_body"],
            for_id("INTERVENTION", 3)["future_token_counter_body"]
        );
        assert_eq!(
            for_id("BASELINE", 3)["policy_decision"]["rule"],
            "EXACT_DUPLICATE_PRUNE"
        );
        assert_eq!(
            for_id("BASELINE", 3)["policy_decision"]["eligible_prune_candidates"],
            1
        );
        assert_eq!(
            for_id("BASELINE", 3)["policy_decision"]["eligible_defer_candidates"],
            0
        );
        assert_eq!(
            for_id("BASELINE", 3)["policy_decision"]["eligible_relocate_candidates"],
            0
        );
        assert_eq!(
            for_id("BASELINE", 3)["canonical_prior_output_identities"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            for_id("BASELINE", 3)["pinned_prior_receipts"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
    for case_id in ["CP05", "CP06"] {
        for slot in 1..=3 {
            let hashes = rows
                .iter()
                .filter(|row| row["case_id"] == case_id && row["request_slot"] == slot)
                .map(|row| row["request_body_sha256"].as_str().unwrap())
                .collect::<BTreeSet<_>>();
            assert_eq!(hashes.len(), 1);
        }
    }
}

#[test]
fn versioned_successor_binds_exact_ledger_and_historical_predecessor() {
    let root = root();
    let regenerated = workload::successor_bytes(&root).unwrap();
    assert_eq!(
        regenerated,
        fs::read(root.join(workload::SUCCESSOR_PATH)).unwrap()
    );
    let report: Value = serde_json::from_slice(&regenerated).unwrap();
    assert_eq!(report["schema_version"], 4);
    assert_eq!(
        report["historical_predecessor"]["sha256"],
        "a24c37cb879b658736d06680425d4e0dec036cf4ff85ef63eba1206ed7ff4732"
    );
    assert_eq!(
        report["request_ledger"]["sha256"],
        digest(&fs::read(root.join(workload::LEDGER_PATH)).unwrap())
    );
    assert_eq!(report["rendering_revalidation"]["arm_trajectories"], 18);
    assert_eq!(report["rendering_revalidation"]["request_renderings"], 54);
    assert_eq!(
        report["rendering_revalidation"]["canonical_advancing_transitions"],
        36
    );
}
