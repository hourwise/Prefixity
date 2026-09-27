#[path = "../examples/support/claim2_report.rs"]
mod report;

#[test]
fn historical_task_provenance_does_not_reauthorize_consumed_gate() {
    let error = prefixity_controlled_benchmark::preflight_local_9b_gate()
        .unwrap_err()
        .to_string();
    assert!(error.contains("does not match current source"), "{error}");
}

#[test]
fn complete_offline_cohort_matches_committed_evidence() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generated = report::report_bytes(&root).unwrap();
    let committed =
        std::fs::read(root.join("fixtures/claim2/materialization-report.json")).unwrap();
    assert_eq!(
        generated, committed,
        "regenerate only after reviewing fixture changes"
    );
    let value: serde_json::Value = serde_json::from_slice(&generated).unwrap();
    let slots = value["slots"].as_array().unwrap();
    for group in slots.chunks_exact(9) {
        for slot in 0..3 {
            assert_eq!(
                group[slot]["static_skeleton_json"],
                group[slot + 3]["static_skeleton_json"]
            );
            assert_eq!(
                group[slot]["static_skeleton_sha256"],
                group[slot + 3]["static_skeleton_sha256"]
            );
            if slot < 2 || group[slot]["case_id"] == "CP05" || group[slot]["case_id"] == "CP06" {
                assert_eq!(
                    group[slot]["static_skeleton_json"],
                    group[slot + 6]["static_skeleton_json"]
                );
            }
        }
    }
    assert!(slots.iter().all(|slot| slot["dispatchable"] == false));
}
