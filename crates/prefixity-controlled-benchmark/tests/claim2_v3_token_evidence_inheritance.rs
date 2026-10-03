// Historical generators are deliberately loaded together to audit V3 reuse.
#![allow(clippy::duplicate_mod)]

#[path = "../examples/support/claim2_v3_token_evidence_inheritance.rs"]
mod inheritance;
#[path = "../examples/support/claim2_tokenization_report.rs"]
#[allow(dead_code)]
mod v1_renderer;

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn regenerated_map(root: &std::path::Path) -> Vec<u8> {
    let v1_ledger = v1_renderer::report_bytes(root).unwrap();
    let v3_ledger = fs::read(root.join("fixtures/claim2/workload-request-ledger-v3.json")).unwrap();
    inheritance::map_bytes(root, &v1_ledger, &v3_ledger).unwrap()
}

#[test]
fn exact_v3_inheritance_map_reproduces_and_joins_the_sealed_v1_measurements() {
    let root = repository_root();
    let actual = regenerated_map(&root);
    let expected = fs::read(root.join(inheritance::OUTPUT_PATH)).unwrap();
    assert_eq!(actual, expected);

    let map: Value = serde_json::from_slice(&actual).unwrap();
    assert_eq!(map["schema_version"], 3);
    assert_eq!(
        map["scope"]["cohort_order"],
        serde_json::json!(["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"])
    );
    assert_eq!(map["scope"]["logical_requests_covered"], 54);
    assert_eq!(map["scope"]["retained_logical_requests_verified"], 36);
    assert_eq!(map["scope"]["retained_unique_request_hashes_verified"], 14);
    assert_eq!(
        map["scope"]["tokenization_performed_by_this_generator"],
        false
    );
    assert_eq!(map["scope"]["inference_performed_by_this_generator"], false);
    assert_eq!(
        map["historical_v1_bindings"]["identity_canonical_sha256"],
        "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16"
    );
    assert_eq!(
        map["historical_v1_bindings"]["raw_evidence_sha256"],
        "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264"
    );
    assert_eq!(
        map["historical_v1_bindings"]["interpreted_result_canonical_sha256"],
        "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040"
    );
    let ledger: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/claim2/workload-request-ledger-v3.json")).unwrap(),
    )
    .unwrap();
    let inherited_rows = ledger["requests"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT")
        .count();
    let new_rows = ledger["requests"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["evidence_classification"] == "NEW_TOKEN_COUNT_REQUIRED")
        .count();
    let inherited_hashes = map["inherited_unique_request_hashes"]
        .as_array()
        .unwrap()
        .len();
    let new_hashes = map["new_unique_request_hashes"].as_array().unwrap().len();
    assert_eq!(
        map["scope"]["inherited_v1_logical_requests"],
        inherited_rows
    );
    assert_eq!(map["scope"]["new_logical_requests"], new_rows);
    assert_eq!(
        map["scope"]["inherited_v1_unique_request_hashes"],
        inherited_hashes
    );
    assert_eq!(
        map["scope"]["new_unique_request_hashes_requiring_tokenization"],
        new_hashes
    );
    assert_eq!(
        map["scope"]["unique_request_hashes"],
        inherited_hashes + new_hashes
    );
    assert!(inherited_rows >= 36);
}

#[test]
fn retained_v3_body_drift_fails_closed_before_inheritance_is_emitted() {
    let root = repository_root();
    let v1_ledger = v1_renderer::report_bytes(&root).unwrap();
    let v3_ledger = fs::read(root.join("fixtures/claim2/workload-request-ledger-v3.json")).unwrap();
    let mut changed: Value = serde_json::from_slice(&v3_ledger).unwrap();
    let retained = changed["requests"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["logical_request_id"] == "CP02/BASELINE/slot-1")
        .unwrap();
    retained["future_token_counter_body"] = Value::String(format!(
        "{} ",
        retained["future_token_counter_body"].as_str().unwrap()
    ));
    let changed_bytes = serde_json::to_vec_pretty(&changed).unwrap();
    let error = inheritance::map_bytes(&root, &v1_ledger, &changed_bytes).unwrap_err();
    assert!(error.to_string().contains("V1_TOKEN_EVIDENCE_NOT_REUSABLE"));
}

#[test]
fn changed_v1_ledger_is_rejected_even_when_supplied_directly() {
    let root = repository_root();
    let mut v1_ledger = v1_renderer::report_bytes(&root).unwrap();
    v1_ledger[0] ^= 1;
    let error = inheritance::map_bytes(&root, &v1_ledger, &[]).unwrap_err();
    assert!(error.to_string().contains("V1_TOKEN_EVIDENCE_NOT_REUSABLE"));
}
