#[path = "../examples/support/claim2_tokenization_report.rs"]
mod report;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ledger_path(root: &std::path::Path) -> PathBuf {
    root.join(report::output_path())
}

fn ledger_value(root: &std::path::Path) -> Value {
    serde_json::from_slice(&fs::read(ledger_path(root)).unwrap()).unwrap()
}

#[test]
fn frozen_ledger_reproduces_all_54_requests_byte_for_byte() {
    let root = repository_root();
    let regenerated = report::report_bytes(&root).unwrap();
    assert_eq!(regenerated, fs::read(ledger_path(&root)).unwrap());
    let ledger: Value = serde_json::from_slice(&regenerated).unwrap();
    assert_eq!(ledger["offline_boundary"]["logical_requests"], 54);
    assert_eq!(ledger["requests"].as_array().unwrap().len(), 54);
}

#[test]
fn all_requests_pin_exact_messages_body_hashes_settings_and_dispatch_boundary() {
    let root = repository_root();
    let ledger = ledger_value(&root);
    let requests = ledger["requests"].as_array().unwrap();
    assert_eq!(requests.len(), 54);
    let mut identities = BTreeSet::new();
    let mut observed = BTreeMap::new();

    for request in requests {
        let request_id = request["logical_request_id"].as_str().unwrap();
        assert!(
            identities.insert(request_id.to_owned()),
            "duplicate {request_id}"
        );
        let case_id = request["case_id"].as_str().unwrap();
        let arm = request["arm"].as_str().unwrap();
        let slot = request["request_slot"].as_u64().unwrap();
        let expected_id = format!("{case_id}/{arm}/slot-{slot}");
        assert_eq!(request_id, expected_id);
        assert!(matches!(arm, "BASELINE" | "NO_OP" | "INTERVENTION"));

        let messages: Vec<Message> = serde_json::from_value(request["messages"].clone()).unwrap();
        assert!(!messages.is_empty());
        let exact_messages_bytes = serde_json::to_vec(
            &messages
                .iter()
                .map(|message| WireMessage {
                    role: &message.role,
                    content: &message.content,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        assert_eq!(request["messages_sha256"], digest(&exact_messages_bytes));
        assert_eq!(
            request["messages_utf8_byte_length"],
            exact_messages_bytes.len()
        );

        let body_text = request["future_token_counter_body"].as_str().unwrap();
        let body_bytes = body_text.as_bytes();
        assert_eq!(request["request_body_sha256"], digest(body_bytes));
        assert_eq!(request["request_body_utf8_byte_length"], body_bytes.len());
        let body: Value = serde_json::from_slice(body_bytes).unwrap();
        let object = body.as_object().unwrap();
        let fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
        assert_eq!(
            fields,
            BTreeSet::from([
                "model",
                "messages",
                "max_tokens",
                "temperature",
                "top_p",
                "seed",
                "stream"
            ])
        );
        assert_eq!(body["model"], ledger["request_semantics"]["model"]);
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["temperature"], 0);
        assert_eq!(body["top_p"], 1);
        assert_eq!(body["seed"], 1);
        assert_eq!(body["stream"], false);
        assert!(object.get("chat_template_kwargs").is_none());
        assert_eq!(body["messages"], request["messages"]);
        assert!(body_bytes
            .windows(exact_messages_bytes.len())
            .any(|window| window == exact_messages_bytes));

        assert_eq!(request["renderer_dispatchable"], false);
        assert_eq!(request["tokenizable"], true);
        assert_eq!(request["live_dispatchable"], false);
        assert_eq!(request["tokenization_status"], "NOT_PERFORMED");
        assert_eq!(request["token_count_status"], "EXACT_TOKENIZATION_REQUIRED");
        assert_eq!(request["unbound_raw_assistant_slots"], json!([]));
        assert_eq!(request["unbound_environment_receipt_slots"], json!([]));
        assert_eq!(
            request["canonical_prior_output_identities"]
                .as_array()
                .unwrap()
                .len() as u64,
            slot - 1
        );
        assert_eq!(
            request["pinned_prior_receipts"].as_array().unwrap().len() as u64,
            slot - 1
        );
        observed.insert((case_id.to_owned(), arm.to_owned(), slot), request);
    }
    assert_eq!(identities.len(), 54);
    for case in ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"] {
        for arm in ["BASELINE", "NO_OP", "INTERVENTION"] {
            for slot in 1..=3 {
                assert!(observed.contains_key(&(case.to_owned(), arm.to_owned(), slot)));
            }
        }
    }
}

#[test]
fn canonical_histories_carry_arm_local_outputs_and_pinned_receipts() {
    let root = repository_root();
    let ledger = ledger_value(&root);
    let advancing_domain: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/claim2/advancing-output-domain-v1.json")).unwrap(),
    )
    .unwrap();
    let transitions = advancing_domain["transitions"].as_array().unwrap();

    for request in ledger["requests"].as_array().unwrap() {
        let case_id = request["case_id"].as_str().unwrap();
        let arm = request["arm"].as_str().unwrap();
        let slot = request["request_slot"].as_u64().unwrap();
        let messages: Vec<Message> = serde_json::from_value(request["messages"].clone()).unwrap();
        for prior_slot in 1..slot {
            let transition = transitions
                .iter()
                .find(|row| {
                    row["case_id"] == case_id && row["request_slot"].as_u64() == Some(prior_slot)
                })
                .unwrap();
            let output = request["canonical_prior_output_identities"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["request_slot"].as_u64() == Some(prior_slot))
                .unwrap();
            assert_eq!(output["action_id"], transition["expected_action_id"]);
            assert_eq!(
                output["canonical_response"],
                transition["canonical_raw_response"]
            );
            assert_eq!(
                output["canonical_response_sha256"],
                transition["canonical_response_sha256"]
            );
            assert_eq!(
                output["arm_local_identity"]
                    .as_str()
                    .unwrap()
                    .split('/')
                    .nth(1),
                Some(arm)
            );
            let message_index = output["message_index"].as_u64().unwrap() as usize;
            assert_eq!(messages[message_index].role, "assistant");
            assert_eq!(
                messages[message_index].content,
                transition["canonical_raw_response"]
            );

            let receipt = request["pinned_prior_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["request_slot"].as_u64() == Some(prior_slot))
                .unwrap();
            assert_eq!(
                receipt["receipt_id"],
                transition["deterministic_receipt_identity"]
            );
            assert_eq!(
                receipt["receipt_path"],
                transition["deterministic_receipt_path"]
            );
            assert_eq!(
                receipt["receipt_sha256"],
                transition["deterministic_receipt_sha256"]
            );
            assert_eq!(
                receipt["arm_local_identity"]
                    .as_str()
                    .unwrap()
                    .split('/')
                    .nth(1),
                Some(arm)
            );
            let receipt_bytes =
                fs::read(root.join(receipt["receipt_path"].as_str().unwrap())).unwrap();
            assert_eq!(digest(&receipt_bytes), receipt["receipt_sha256"]);
            let receipt_text = String::from_utf8(receipt_bytes).unwrap();
            let receipt_message_index = receipt["message_index"].as_u64().unwrap() as usize;
            assert_eq!(messages[receipt_message_index].role, "user");
            assert_eq!(messages[receipt_message_index].content, receipt_text);
        }
    }
}

#[test]
fn fixture_identity_is_pinned_without_exposing_hidden_keys_and_hash_groups_are_observed() {
    let root = repository_root();
    let ledger_bytes = fs::read(ledger_path(&root)).unwrap();
    let ledger: Value = serde_json::from_slice(&ledger_bytes).unwrap();
    assert_eq!(
        ledger["fixture_identities"]["accepted_materialization_successor"]["sha256"],
        digest(&fs::read(root.join("fixtures/claim2/materialization-report-v2.json")).unwrap())
    );
    assert_eq!(
        ledger["fixture_identities"]["advancing_output_domain_ledger"]["sha256"],
        digest(&fs::read(root.join("fixtures/claim2/advancing-output-domain-v1.json")).unwrap())
    );
    assert_eq!(
        ledger["fixture_identities"]["accepted_materialization_predecessor"]["sha256"],
        "3d9571ddd4e17096b55971040531aac6197b6b9b7bc3d01afefd8992c28f6c65"
    );
    let case_identities = ledger["fixture_identities"]["cases"].as_array().unwrap();
    assert_eq!(case_identities.len(), 6);
    for identity in case_identities {
        let manifest_bytes =
            fs::read(root.join(identity["manifest_path"].as_str().unwrap())).unwrap();
        assert_eq!(digest(&manifest_bytes), identity["manifest_sha256"]);
        let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
        assert_eq!(
            manifest["evaluation_key_sha256"],
            identity["evaluation_key_sha256"]
        );
        let key_bytes = fs::read(
            root.join("fixtures/claim2")
                .join(identity["case_id"].as_str().unwrap().to_ascii_lowercase())
                .join(manifest["evaluation_key_path"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(digest(&key_bytes), identity["evaluation_key_sha256"]);
        assert!(!ledger_bytes
            .windows(key_bytes.len())
            .any(|window| window == key_bytes));
    }
    fn hidden_answer_key_present(value: &Value) -> bool {
        match value {
            Value::Object(object) => object.iter().any(|(key, child)| {
                matches!(
                    key.as_str(),
                    "evaluation_key"
                        | "expected_action_ids"
                        | "expected_result_event_ids"
                        | "expected_states"
                        | "expected_final_answer"
                        | "required_event_ids"
                        | "required_relation_ids"
                        | "critical_event_ids"
                ) || hidden_answer_key_present(child)
            }),
            Value::Array(items) => items.iter().any(hidden_answer_key_present),
            _ => false,
        }
    }
    assert!(!hidden_answer_key_present(&ledger));

    assert_eq!(ledger["deduplication"]["logical_request_count"], 54);
    assert_eq!(ledger["deduplication"]["unique_request_body_count"], 22);
    assert_eq!(
        ledger["deduplication"]["duplicate_logical_requests_avoided_if_contacted_later"],
        32
    );
    assert_eq!(
        ledger["deduplication"]["equality_structure_matches_review_expectation"],
        true
    );
    assert_eq!(
        ledger["deduplication"]["unique_bodies_by_request_slot"]["1"],
        6
    );
    assert_eq!(
        ledger["deduplication"]["unique_bodies_by_request_slot"]["2"],
        6
    );
    assert_eq!(
        ledger["deduplication"]["unique_bodies_by_request_slot"]["3"],
        10
    );
    let groups = ledger["deduplication"]["hash_groups"].as_array().unwrap();
    assert_eq!(groups.len(), 22);
    let mut members = BTreeSet::new();
    let mut recorded_groups = BTreeMap::new();
    for group in groups {
        let hash = group["request_body_sha256"].as_str().unwrap();
        let mut member_ids = BTreeSet::new();
        for id in group["logical_request_ids"].as_array().unwrap() {
            let id = id.as_str().unwrap().to_owned();
            assert!(members.insert(id.clone()));
            member_ids.insert(id);
        }
        recorded_groups.insert(hash.to_owned(), member_ids);
    }
    assert_eq!(members.len(), 54);

    let mut observed_groups = BTreeMap::<String, BTreeSet<String>>::new();
    let mut body_by_hash = BTreeMap::<String, String>::new();
    for request in ledger["requests"].as_array().unwrap() {
        let body = request["future_token_counter_body"].as_str().unwrap();
        let hash = digest(body.as_bytes());
        assert_eq!(request["request_body_sha256"], hash);
        if let Some(previous) = body_by_hash.insert(hash.clone(), body.to_owned()) {
            assert_eq!(previous.as_bytes(), body.as_bytes());
        }
        observed_groups
            .entry(hash)
            .or_default()
            .insert(request["logical_request_id"].as_str().unwrap().to_owned());
    }
    assert_eq!(recorded_groups, observed_groups);

    let mut expected_groups = BTreeSet::new();
    for case_id in ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"] {
        for slot in [1, 2] {
            expected_groups.insert(
                ["BASELINE", "NO_OP", "INTERVENTION"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-{slot}"))
                    .collect::<BTreeSet<_>>(),
            );
        }
        if matches!(case_id, "CP01" | "CP02" | "CP03" | "CP04") {
            expected_groups.insert(
                ["BASELINE", "NO_OP"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-3"))
                    .collect(),
            );
            expected_groups.insert(BTreeSet::from([format!("{case_id}/INTERVENTION/slot-3")]));
        } else {
            expected_groups.insert(
                ["BASELINE", "NO_OP", "INTERVENTION"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-3"))
                    .collect(),
            );
        }
    }
    assert_eq!(
        observed_groups.values().cloned().collect::<BTreeSet<_>>(),
        expected_groups
    );
}
