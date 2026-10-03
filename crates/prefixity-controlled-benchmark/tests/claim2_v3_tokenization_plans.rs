#[path = "../examples/support/claim2_v3_tokenization_plans.rs"]
mod plans;

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn v3_hybrid_and_full_fresh_plans_reproduce_from_the_frozen_renderer() {
    let root = repository_root();
    let (hybrid, full) = plans::plan_pair(&root).unwrap();
    assert_eq!(hybrid, fs::read(root.join(plans::HYBRID_PATH)).unwrap());
    assert_eq!(full, fs::read(root.join(plans::FULL_FRESH_PATH)).unwrap());

    let hybrid_plan: Value = serde_json::from_slice(&hybrid).unwrap();
    let full_plan: Value = serde_json::from_slice(&full).unwrap();
    for (plan, mode, fresh, inherited) in [
        (&hybrid_plan, "HYBRID_8_NEW", 8, 14),
        (&full_plan, "FULL_FRESH_22", 22, 0),
    ] {
        assert_eq!(
            plan["schema_id"],
            "prefixity.phase1c.claim2-tokenization-v3-contact-plan"
        );
        assert_eq!(plan["schema_version"], 3);
        assert_eq!(plan["status"], "FROZEN_OFFLINE_NO_CONTACTS");
        assert_eq!(plan["mode"], mode);
        assert_eq!(
            plan["source_ledger"]["sha256"],
            "406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2"
        );
        assert_eq!(
            plan["source_inheritance_map"]["sha256"],
            "6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2"
        );
        assert_eq!(plan["logical_request_count"], 54);
        assert_eq!(plan["total_unique_body_count"], 22);
        assert_eq!(plan["inherited_unique_body_count"], inherited);
        assert_eq!(plan["fresh_unique_body_count"], fresh);
        assert_eq!(plan["maximum_readiness_contacts"], 1);
        assert_eq!(plan["maximum_token_count_contacts"], fresh);
        assert_eq!(plan["inference_allowance"], 0);
        assert_eq!(
            plan["endpoint_allowlist"]["readiness"],
            serde_json::json!({"method":"GET","path":"/health"})
        );
        assert_eq!(
            plan["endpoint_allowlist"]["token_count"],
            serde_json::json!({"method":"POST","path":"/v1/chat/completions/input_tokens"})
        );
    }
}

#[test]
fn every_plan_entry_resolves_to_exact_v3_ledger_bodies_in_hash_order() {
    let root = repository_root();
    let (hybrid, full) = plans::plan_pair(&root).unwrap();
    let hybrid: Value = serde_json::from_slice(&hybrid).unwrap();
    let full: Value = serde_json::from_slice(&full).unwrap();
    let ledger: Value = serde_json::from_slice(
        &fs::read(root.join("fixtures/claim2/workload-request-ledger-v3.json")).unwrap(),
    )
    .unwrap();
    let rows = ledger["requests"].as_array().unwrap();
    let mut body_by_hash = BTreeMap::new();
    for row in rows {
        body_by_hash
            .entry(row["request_body_sha256"].as_str().unwrap().to_owned())
            .or_insert_with(|| row);
    }

    for plan in [&hybrid, &full] {
        let entries = plan["entries"].as_array().unwrap();
        let hashes = entries
            .iter()
            .map(|entry| entry["request_body_sha256"].as_str().unwrap())
            .collect::<Vec<_>>();
        let mut sorted = hashes.clone();
        sorted.sort_unstable();
        assert_eq!(hashes, sorted);
        for entry in entries {
            let hash = entry["request_body_sha256"].as_str().unwrap();
            let ledger_row = body_by_hash[hash];
            let body = ledger_row["future_token_counter_body"].as_str().unwrap();
            assert_eq!(entry["exact_request_body"], body);
            assert_eq!(entry["request_body_utf8_byte_length"], body.len());
            assert_eq!(digest(body.as_bytes()), hash);
            assert_eq!(entry["case_id"], ledger_row["case_id"]);
            assert_eq!(entry["request_slot"], ledger_row["request_slot"]);
            let mut ids = rows
                .iter()
                .filter(|row| row["request_body_sha256"] == hash)
                .map(|row| row["logical_request_id"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            ids.sort();
            assert_eq!(entry["logical_request_ids"], serde_json::json!(ids));
            assert_eq!(
                entry["representative_logical_request_id"],
                ids.first().unwrap().as_str()
            );
        }
    }
}

#[test]
fn arbitrary_or_drifted_ledger_and_map_bytes_fail_closed() {
    let root = repository_root();
    let ledger_bytes = fs::read(root.join(plans::LEDGER_PATH)).unwrap();
    let map_bytes = fs::read(root.join(plans::MAP_PATH)).unwrap();

    let mut altered_ledger: Value = serde_json::from_slice(&ledger_bytes).unwrap();
    let row = altered_ledger["requests"]
        .as_array_mut()
        .unwrap()
        .first_mut()
        .unwrap();
    row["future_token_counter_body"] = Value::String(format!(
        "{} ",
        row["future_token_counter_body"].as_str().unwrap()
    ));
    let changed_ledger = serde_json::to_vec_pretty(&altered_ledger).unwrap();
    assert!(
        plans::plan_pair_from_sources(&root, &changed_ledger, &map_bytes)
            .unwrap_err()
            .to_string()
            .contains("accepted renderer or frozen SHA-256")
    );

    let mut changed_map = map_bytes;
    changed_map[0] ^= 1;
    assert!(
        plans::plan_pair_from_sources(&root, &ledger_bytes, &changed_map)
            .unwrap_err()
            .to_string()
            .contains("frozen SHA-256 or tracked bytes")
    );
}
