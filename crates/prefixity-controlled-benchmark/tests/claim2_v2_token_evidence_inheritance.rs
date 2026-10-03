#[path = "../examples/support/claim2_v2_token_evidence_inheritance.rs"]
mod inheritance;
#[path = "../examples/support/claim2_tokenization_report.rs"]
mod report;

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn map_value(root: &std::path::Path) -> Value {
    let regenerated = report::report_bytes(root).unwrap();
    let bytes = inheritance::map_bytes(root, &regenerated).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[test]
fn retained_case_map_reproduces_exactly_and_covers_14_hashes_for_36_requests() {
    let root = repository_root();
    assert_eq!(
        report::output_path(),
        "fixtures/claim2/tokenization-request-ledger-v1.json"
    );
    let regenerated = report::report_bytes(&root).unwrap();
    let expected = fs::read(root.join(inheritance::OUTPUT_PATH)).unwrap();
    let actual = inheritance::map_bytes(&root, &regenerated).unwrap();
    assert_eq!(actual, expected);

    let map: Value = serde_json::from_slice(&actual).unwrap();
    assert_eq!(map["schema_version"], 2);
    assert_eq!(
        map["scope"]["cohort_order"],
        serde_json::json!(["CP02", "CP03", "CP07", "CP08", "CP05", "CP06"])
    );
    assert_eq!(map["scope"]["logical_requests_covered"], 36);
    assert_eq!(map["scope"]["unique_exact_request_hashes_covered"], 14);
    assert_eq!(map["scope"]["complete_v2_request_ledger_present"], false);
    assert_eq!(
        map["inherited_unique_request_hashes"]
            .as_array()
            .unwrap()
            .len(),
        14
    );

    let mut all_ids = Vec::new();
    let mut counts_by_id = std::collections::BTreeMap::new();
    for row in map["inherited_unique_request_hashes"].as_array().unwrap() {
        assert_eq!(row["measurement_label"], "V1_MEASUREMENT_REUSED_IN_V2");
        assert!(row["input_tokens"].as_u64().unwrap_or_default() > 0);
        assert_eq!(row["v1_logical_request_ids"], row["v2_logical_request_ids"]);
        for id in row["v2_logical_request_ids"].as_array().unwrap() {
            let id = id.as_str().unwrap().to_owned();
            counts_by_id.insert(id.clone(), row["input_tokens"].as_u64().unwrap());
            all_ids.push(id);
        }
    }
    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), 36);
    assert!(all_ids
        .iter()
        .all(|id| ["CP02", "CP03", "CP05", "CP06"].contains(&id.split('/').next().unwrap())));
    for (case_id, expected) in [
        ("CP02", [241, 1440, 4078]),
        ("CP03", [249, 1305, 2488]),
        ("CP05", [1390, 1608, 3101]),
        ("CP06", [1443, 1655, 3167]),
    ] {
        for (slot, count) in expected.into_iter().enumerate() {
            for arm in ["BASELINE", "NO_OP", "INTERVENTION"] {
                let id = format!("{case_id}/{arm}/slot-{}", slot + 1);
                let expected_count = if case_id == "CP02" && slot == 2 && arm == "INTERVENTION" {
                    3093
                } else if case_id == "CP03" && slot == 2 && arm == "INTERVENTION" {
                    1590
                } else {
                    count
                };
                assert_eq!(counts_by_id[&id], expected_count, "{id}");
            }
        }
    }
}

#[test]
fn any_retained_request_body_drift_fails_closed_as_non_reusable() {
    let root = repository_root();
    let regenerated = report::report_bytes(&root).unwrap();
    let mut changed: Value = serde_json::from_slice(&regenerated).unwrap();
    let retained = changed["requests"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|request| request["logical_request_id"] == "CP02/BASELINE/slot-1")
        .unwrap();
    retained["future_token_counter_body"] = Value::String(format!(
        "{} ",
        retained["future_token_counter_body"].as_str().unwrap()
    ));
    let changed_bytes = serde_json::to_vec_pretty(&changed).unwrap();
    let error = inheritance::map_bytes(&root, &changed_bytes).unwrap_err();
    assert!(error.to_string().contains("V1_TOKEN_EVIDENCE_NOT_REUSABLE"));
}

#[test]
fn inherited_counts_bind_to_the_immutable_failed_v1_result_and_seals() {
    let root = repository_root();
    let map = map_value(&root);
    assert_eq!(
        map["v1_bindings"]["tokenization_identity"],
        "claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91"
    );
    assert_eq!(
        map["v1_bindings"]["identity_canonical_sha256"],
        "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16"
    );
    assert_eq!(
        map["v1_bindings"]["raw_evidence_sha256"],
        "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264"
    );
    assert_eq!(
        map["v1_bindings"]["interpreted_result_canonical_sha256"],
        "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040"
    );
    assert_eq!(
        map["v1_bindings"]["terminal_classification"],
        "WORKLOAD_TOKEN_ADMISSION_FAILED"
    );
    assert_eq!(
        map["v1_bindings"]["instrument_identity"]["runtime_contract"]["reasoning"],
        "off"
    );
    assert_eq!(
        map["v1_bindings"]["instrument_identity"]["runtime_contract"]["chat_template_kwargs"],
        "ABSENT"
    );
}
