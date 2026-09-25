//! Clean-checkout proof for authoritative calibration candidate order.
//!
//! These tests read only tracked repository evidence: the transition
//! registry and the identities and execution records it binds. A fresh CI
//! clone has no ignored run evidence, so passing here shows that candidate
//! order never depends on it.

use prefixity_controlled_benchmark::{
    resolve_authoritative_candidate_transition, validate_candidate_order_report,
    CALIBRATION_CANDIDATE_TRANSITIONS_PATH,
};

#[test]
fn clean_checkout_candidate_512_order_is_valid() {
    let order = validate_candidate_order_report(512).unwrap();
    let predecessor = &order["predecessor_transition"];

    assert_eq!(order["candidate_order_valid"], true);
    assert_eq!(predecessor["source_attempt"], 8);
    assert_eq!(predecessor["source_budget"], 1024);
    assert_eq!(predecessor["source_state"], "FAIL");
    assert_eq!(predecessor["selected_next_budget"], 512);
    assert_eq!(
        predecessor["authoritative_transition_path"],
        CALIBRATION_CANDIDATE_TRANSITIONS_PATH
    );
    assert_eq!(predecessor["raw_predecessor_evidence_required"], false);
    assert!(order.get("predecessor_result_path").is_none());
}

#[test]
fn clean_checkout_candidate_256_order_is_valid() {
    let order = validate_candidate_order_report(256).unwrap();
    let predecessor = &order["predecessor_transition"];

    assert_eq!(order["candidate_order_valid"], true);
    assert_eq!(predecessor["source_attempt"], 10);
    assert_eq!(predecessor["source_budget"], 512);
    assert_eq!(predecessor["source_state"], "FAIL");
    assert_eq!(predecessor["integrity_accepted"], true);
    assert_eq!(predecessor["calibration_admissible"], true);
    assert_eq!(predecessor["selected_next_budget"], 256);
    assert_eq!(
        predecessor["evidence_manifest_sha256"],
        "5673e55b381d1f5171c38bc9f1721b3105adf829ece4ec029cbcca4db150c4b9"
    );
    assert_eq!(predecessor["raw_predecessor_evidence_required"], false);
    assert!(order.get("predecessor_result_path").is_none());
}

#[test]
fn clean_checkout_resolution_excludes_attempts_007_and_009() {
    for budget in [512, 256] {
        let transition = resolve_authoritative_candidate_transition(budget).unwrap();
        assert_ne!(transition["source_attempt"], 7);
        assert_ne!(transition["source_attempt"], 9);
        assert_eq!(transition["attempt_007_excluded_from_selection"], true);
        assert_eq!(transition["attempt_009_excluded_from_selection"], true);
    }
}

#[test]
fn initial_and_unregistered_candidates_have_no_predecessor_transition() {
    let initial = validate_candidate_order_report(1024).unwrap();
    assert_eq!(initial["predecessor_transition_required"], false);
    assert!(resolve_authoritative_candidate_transition(1024).is_err());
    assert!(validate_candidate_order_report(128).is_err());
}
