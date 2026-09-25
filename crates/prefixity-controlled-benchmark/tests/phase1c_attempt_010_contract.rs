//! Clean-checkout proof for the prepared Attempt-010 contract.
//!
//! These tests read only tracked repository evidence. They do not require
//! ignored prior-run evidence, frozen executables, a supervisor, a listener,
//! or a model server, so a fresh CI clone exercises the same repository
//! contract that the live Attempt-010 child validates before model contact.

use prefixity_controlled_benchmark::{
    validate_attempt_010_identity_document, validate_attempt_010_repository_contract,
    CALIBRATION_ATTEMPT_010_IDENTITY_PATH,
};
use serde_json::{json, Value};
use std::path::Path;

fn identity() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(CALIBRATION_ATTEMPT_010_IDENTITY_PATH);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn rejected(mutate: impl FnOnce(&mut Value)) -> bool {
    let mut identity = identity();
    mutate(&mut identity);
    validate_attempt_010_identity_document(&identity).is_err()
}

#[test]
fn clean_checkout_attempt_010_repository_contract_is_valid() {
    let contract = validate_attempt_010_repository_contract().unwrap();

    assert_eq!(contract["state"], "ATTEMPT_010_REPOSITORY_CONTRACT_VALID");
    for flag in [
        "ATTEMPT_ID_VALID",
        "ATTEMPT_IDENTITY_VALID",
        "PREDECESSOR_TRANSITION_PRESENT",
        "PREDECESSOR_TRANSITION_VALID",
        "SOURCE_ATTEMPT_008",
        "SOURCE_BUDGET_1024",
        "SOURCE_STATE_FAIL",
        "SOURCE_INTEGRITY_ACCEPTED",
        "SOURCE_CALIBRATION_ADMISSIBLE",
        "NEXT_BUDGET_512",
        "CANDIDATE_ORDER_VALID",
        "CANDIDATE_BUDGET_512_ORDER_VALID",
        "ATTEMPT_007_EXCLUDED_FROM_SELECTION",
        "ATTEMPT_009_EXCLUDED_FROM_SELECTION",
        "CASE_SET_VALID",
        "REQUEST_CEILING_VALID",
        "RETRY_POLICY_VALID",
        "FALLBACK_POLICY_VALID",
    ] {
        assert_eq!(contract[flag], true, "{flag}");
    }
    assert_eq!(contract["raw_predecessor_evidence_required"], false);
    assert_eq!(
        contract["candidate_order"]["predecessor_transition"]["source_attempt"],
        8
    );
    assert!(contract["candidate_order"]
        .get("predecessor_result_path")
        .is_none());
    assert_eq!(contract["network_calls"], 0);
    assert_eq!(contract["inference_requests"], 0);
}

#[test]
fn attempt_010_identity_document_is_accepted_unchanged() {
    validate_attempt_010_identity_document(&identity()).unwrap();
}

#[test]
fn attempt_010_identity_rejects_contract_changes() {
    assert!(rejected(|id| id["attempt"] = json!(9)));
    assert!(rejected(|id| id["candidate"]["reasoning_budget"] = json!(256)));
    assert!(rejected(|id| id["runtime"]["reasoning_budget"] = json!(1024)));
    assert!(rejected(|id| id["candidate"]["maximum_requests"] = json!(4)));
    assert!(rejected(|id| id["retry_policy"]["automatic_retries"] = json!(1)));
    assert!(rejected(|id| id["retry_policy"]["fallback_requests"] = json!(1)));
    assert!(rejected(|id| id["generation"]["temperature"] = json!(1)));
    assert!(rejected(|id| id["candidate"]["case_order"] =
        json!(["rbcal-003", "rbcal-002", "rbcal-001"])));
    assert!(rejected(|id| id["launch_plan"]["server_command"] = json!(id["launch_plan"]
        ["server_command"]
        .as_str()
        .unwrap()
        .replace("--reasoning-budget 512", "--reasoning-budget 1024"))));
}

#[test]
fn attempt_010_identity_rejects_inadmissible_predecessors() {
    assert!(rejected(|id| id["budget_provenance"]["source_attempt"] = json!(7)));
    assert!(rejected(|id| id["budget_provenance"]["source_attempt"] = json!(9)));
    assert!(rejected(
        |id| id["lineage"]["attempt_007_excluded_from_selection"] = json!(false)
    ));
    assert!(rejected(
        |id| id["lineage"]["attempt_009_excluded_from_selection"] = json!(false)
    ));
    assert!(rejected(
        |id| id["lineage"]["attempt_009_calibration_admissible"] = json!(true)
    ));
    assert!(rejected(|id| id["lineage"]["attempt_008_evidence_manifest_sha256"] =
        json!("f20c4ce0149070e3ca1bc167f4400d71b88fe0bd7adac41851169ba8540e477")));
}

#[test]
fn attempt_010_identity_rejects_missing_partial_or_mutable_executables() {
    assert!(rejected(|id| {
        id["implementation_fingerprints"]
            .as_object_mut()
            .unwrap()
            .remove("child_binary");
    }));
    assert!(rejected(|id| {
        id["implementation_fingerprints"]
            .as_object_mut()
            .unwrap()
            .remove("supervisor_binary");
        id["implementation_fingerprints"]
            .as_object_mut()
            .unwrap()
            .remove("child_binary");
    }));
    assert!(rejected(|id| {
        id["implementation_fingerprints"]["child_binary"]
            .as_object_mut()
            .unwrap()
            .remove("windows_file_id");
    }));
    assert!(rejected(|id| {
        id["implementation_fingerprints"]["supervisor_binary"]
            .as_object_mut()
            .unwrap()
            .remove("sha256");
    }));
    assert!(rejected(|id| {
        id["implementation_fingerprints"]["child_binary"]["raw_path"] = json!(
            "D:\\Users\\fleur\\Prefixity\\target\\debug\\prefixity-phase1c-reasoning-budget-calibration.exe"
        )
    }));
    assert!(rejected(|id| {
        id["implementation_fingerprints"]["supervisor_binary"]["final_path"] = json!(
            "\\\\?\\D:\\Users\\fleur\\Prefixity\\target\\phase1c-attempt-009-frozen\\prefixity-phase1c-live-supervisor.exe"
        )
    }));
    assert!(rejected(|id| id["launch_plan"]["supervisor_command"] = json!(id["launch_plan"]
        ["supervisor_command"]
        .as_str()
        .unwrap()
        .replace("phase1c-attempt-010-frozen", "debug"))));
}
