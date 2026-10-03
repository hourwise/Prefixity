#[path = "../examples/support/claim2_domain_report.rs"]
#[allow(dead_code)]
mod report;

use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, load_claim2_case, Claim2ArmState, Claim2Evaluation,
    Claim2ProjectionMode, Claim2SlotStatus, LoadedClaim2Case,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
struct ReceiptPin {
    event_id: &'static str,
    path: &'static str,
    sha256: &'static str,
    state: &'static str,
}

const RECEIPTS: [ReceiptPin; 12] = [
    ReceiptPin {
        event_id: "e-result1",
        path: "environment/receipt1.txt",
        sha256: "3eaddfcb86e49ad105f6d11f3a900affae2e5ad3d0a10c54f4078ccff86484a8",
        state: "source_loaded",
    },
    ReceiptPin {
        event_id: "e-result2",
        path: "environment/receipt2.txt",
        sha256: "aa90d37f70fafb34c9409a7e025efe4636afb5ba1d3934d777e8aa88078bd7f2",
        state: "retry_regression_failed_on_r41",
    },
    ReceiptPin {
        event_id: "e-result-inventory",
        path: "bodies/receipt1.txt",
        sha256: "e62dea11ce37d189ac0aaa84cb6a1510ebc3845294c10875952ad8c755ee3afa",
        state: "inventory_ready",
    },
    ReceiptPin {
        event_id: "e-result-callsite",
        path: "bodies/receipt2.txt",
        sha256: "4e81fd820ed9740b04bea9111f95317130ceb193b16efb6ba7d8089cf4481777",
        state: "callsites_opened",
    },
    ReceiptPin {
        event_id: "e-result-inventory",
        path: "bodies/receipt1.txt",
        sha256: "df82948814e3851a98709fb122964ba7f3274becb87c2afb0c6c8909411123e9",
        state: "inventory_ready",
    },
    ReceiptPin {
        event_id: "e-result-verify",
        path: "bodies/receipt2.txt",
        sha256: "d832c457e33ba7cc109116f2510b8980b08b09ca3431ad4add0a426d9340ab67",
        state: "snapshot_verified",
    },
    ReceiptPin {
        event_id: "e-result1",
        path: "environment/receipt1.txt",
        sha256: "6ab3eb54f41ecf9ff1ba4a38b147a16c91f6425b073bd98357c7a98252546b69",
        state: "candidate_failed_origin_restored",
    },
    ReceiptPin {
        event_id: "e-result2",
        path: "environment/receipt2.txt",
        sha256: "e68f339375c2fe0c36583226e3e35db64e63407574ac7ec7d44d89b0d34ec874",
        state: "restored_origin_verified",
    },
    ReceiptPin {
        event_id: "e-inspect-result",
        path: "bodies/receipt-inspect.txt",
        sha256: "2b424ac040d25e4eed01bff4ab4407b0ddabd95435811f89ef7dd1e8002dd53a",
        state: "route_registry_r17_inspected",
    },
    ReceiptPin {
        event_id: "e-reindex-result",
        path: "bodies/receipt-reindex.txt",
        sha256: "588976b08d47a5b9d899502a41a85a6d231c7aea002ef85354bb0be76010f747",
        state: "route_registry_r18_reindexed",
    },
    ReceiptPin {
        event_id: "e-policy-inspect-result",
        path: "bodies/receipt-policy-inspect.txt",
        sha256: "c8500e2bdaea68ec5275f0c8f55e1de332fd4433734b88d1b4135afcecc89694",
        state: "policy_index_s84_inspected",
    },
    ReceiptPin {
        event_id: "e-audit-result",
        path: "bodies/receipt-audit-2081.txt",
        sha256: "f1711d4e3232120b0a53306bba96cc3e601ff04d456c6690908b90bd46f476eb",
        state: "audit_receipt_AUD_2081_verified",
    },
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load_cases(repository: &Path) -> Vec<LoadedClaim2Case> {
    (1..=6)
        .map(|number| {
            load_claim2_case(&repository.join(format!("fixtures/claim2/cp{number:02}/case.json")))
                .unwrap()
        })
        .collect()
}

fn expected(case_id: &str, slot: u8) -> &'static report::ExpectedAction {
    report::EXPECTED_ACTIONS
        .iter()
        .find(|point| point.case_id == case_id && point.request_slot == slot)
        .expect("one pinned action per advancing point")
}

fn arm_ready_for_slot(
    case: &LoadedClaim2Case,
    mode: Claim2ProjectionMode,
    target_slot: u8,
) -> Claim2ArmState {
    let mut arm = Claim2ArmState::new(case, mode);
    for slot in 1..target_slot {
        arm.render_next(case).unwrap();
        let raw = canonical_claim2_action_output(expected(case.case_id(), slot).action_id);
        assert!(arm.record_output(case, raw).unwrap().is_none());
    }
    arm.render_next(case).unwrap();
    arm
}

fn unicode_escape_first_character(value: &str) -> String {
    let (index, character) = value.char_indices().next().unwrap();
    format!(
        "{}\\u{:04x}{}",
        &value[..index],
        character as u32,
        &value[index + character.len_utf8()..]
    )
}

fn rejected_variants(
    case: &LoadedClaim2Case,
    slot: u8,
    expected_id: &str,
) -> Vec<(&'static str, String)> {
    let canonical = canonical_claim2_action_output(expected_id);
    let other_menu = case
        .manifest()
        .action_menu
        .iter()
        .find(|entry| entry.action_slot == slot && entry.action_id != expected_id)
        .unwrap();
    let wrong_slot = expected(case.case_id(), if slot == 1 { 2 } else { 1 });
    let wrong_case = report::EXPECTED_ACTIONS
        .iter()
        .find(|point| point.case_id != case.case_id() && point.request_slot == slot)
        .unwrap();
    let id_json = serde_json::to_string(expected_id).unwrap();
    let escaped_id = unicode_escape_first_character(expected_id);
    vec![
        ("leading space", format!(" {canonical}")),
        ("trailing space", format!("{canonical} ")),
        ("trailing newline", format!("{canonical}\n")),
        (
            "leading and trailing JSON whitespace",
            format!("\t{canonical}\r\n"),
        ),
        (
            "internal JSON whitespace",
            format!("{{ \"action_id\" : {id_json} }}"),
        ),
        ("positional array", format!("[{id_json}]")),
        (
            "additional field",
            format!("{{\"action_id\":{id_json},\"x\":1}}"),
        ),
        (
            "duplicate field",
            format!("{{\"action_id\":{id_json},\"action_id\":{id_json}}}"),
        ),
        (
            "Unicode escape alternative",
            format!("{{\"action_id\":\"{escaped_id}\"}}"),
        ),
        ("Markdown fence", format!("```json\n{canonical}\n```")),
        ("UTF-8 BOM", format!("\u{feff}{canonical}")),
        ("malformed JSON", "{\"action_id\":".to_string()),
        (
            "different action from same menu",
            canonical_claim2_action_output(&other_menu.action_id),
        ),
        (
            "action for other request slot",
            canonical_claim2_action_output(wrong_slot.action_id),
        ),
        (
            "action for another case",
            canonical_claim2_action_output(wrong_case.action_id),
        ),
    ]
}

#[test]
fn canonical_advancing_outputs_carry_exactly_through_all_cases_and_arms() {
    let repository = root();
    let cases = load_cases(&repository);
    let modes = [
        Claim2ProjectionMode::Baseline,
        Claim2ProjectionMode::NoOp,
        Claim2ProjectionMode::Intervention,
    ];
    let mut replayed_trajectories = 0;
    for case in &cases {
        for mode in modes {
            // Each arm constructs its own synthetic canonical responses from the frozen IDs.
            let mut arm = Claim2ArmState::new(case, mode);
            let mut responses = Vec::with_capacity(2);
            let mut request = arm.render_next(case).unwrap();
            for slot in 1..=2 {
                assert_eq!(request.request_slot, slot);
                let raw = canonical_claim2_action_output(expected(case.case_id(), slot).action_id);
                let evaluation = arm.record_output(case, raw.clone()).unwrap();
                assert!(evaluation.is_none());
                assert_eq!(
                    arm.slots()[usize::from(slot - 1)].status,
                    Claim2SlotStatus::Pass
                );
                assert_eq!(
                    arm.slots()[usize::from(slot - 1)]
                        .raw_assistant_output
                        .as_deref(),
                    Some(raw.as_str())
                );
                let receipt = arm.environment_receipts().last().unwrap();
                let pin_index =
                    (case.case_id()[2..].parse::<usize>().unwrap() - 1) * 2 + usize::from(slot - 1);
                let pin = RECEIPTS[pin_index];
                assert_eq!(receipt.source_event_id, pin.event_id);
                assert_eq!(receipt.state_after, pin.state);
                assert_eq!(digest(receipt.text.as_bytes()), pin.sha256);

                let next = arm.render_next(case).unwrap();
                assert_eq!(next.request_slot, slot + 1);
                assert!(next
                    .messages
                    .iter()
                    .any(|message| { message.role == "assistant" && message.content == raw }));
                responses.push(raw);
                if slot == 2 {
                    let carried = next
                        .messages
                        .iter()
                        .filter(|message| message.role == "assistant")
                        .map(|message| message.content.as_str())
                        .collect::<Vec<_>>();
                    assert_eq!(
                        carried,
                        responses.iter().map(String::as_str).collect::<Vec<_>>()
                    );
                }
                request = next;
            }
            replayed_trajectories += 1;
        }
    }
    assert_eq!(replayed_trajectories, 18);
}

#[test]
fn every_noncanonical_or_wrong_advancing_spelling_fails_without_later_work() {
    let cases = load_cases(&root());
    let modes = [
        Claim2ProjectionMode::Baseline,
        Claim2ProjectionMode::NoOp,
        Claim2ProjectionMode::Intervention,
    ];
    let mut rejected = 0;
    for case in &cases {
        for slot in 1..=2 {
            let id = expected(case.case_id(), slot).action_id;
            for mode in modes {
                for (label, raw) in rejected_variants(case, slot, id) {
                    let mut arm = arm_ready_for_slot(case, mode, slot);
                    let fail_index = usize::from(slot - 1);
                    let expected_receipt_count = usize::from(slot - 1);
                    let evaluation: Claim2Evaluation = arm
                        .record_output(case, raw.clone())
                        .unwrap()
                        .expect("noncanonical output must terminate with an observed evaluation");
                    assert_eq!(evaluation.status, Claim2SlotStatus::Fail, "{label}");
                    assert_eq!(
                        evaluation.model_status,
                        Some(Claim2SlotStatus::Fail),
                        "{label}"
                    );
                    assert!(evaluation
                        .failure_reasons
                        .contains(&"noncanonical_intermediate_action_output".to_string()));
                    assert_eq!(
                        arm.slots()[fail_index].status,
                        Claim2SlotStatus::Fail,
                        "{label}"
                    );
                    assert_eq!(
                        arm.slots()[fail_index].raw_assistant_output.as_deref(),
                        Some(raw.as_str()),
                        "{label}"
                    );
                    assert!(arm.slots()[fail_index + 1..]
                        .iter()
                        .all(|later| later.status == Claim2SlotStatus::NotExecutedAfterFailure));
                    assert_eq!(
                        arm.environment_receipts().len(),
                        expected_receipt_count,
                        "{label}"
                    );
                    assert!(arm.render_next(case).is_err(), "{label}");
                    rejected += 1;
                }
            }
        }
    }
    assert_eq!(rejected, 12 * 3 * 15);
}

#[test]
fn finite_domain_ledger_and_materialization_successor_are_reproducible_and_pinned() {
    let repository = root();
    let ledger = report::ledger_bytes(&repository).unwrap();
    assert_eq!(
        ledger,
        fs::read(repository.join("fixtures/claim2/advancing-output-domain-v1.json")).unwrap()
    );
    let ledger_value: Value = serde_json::from_slice(&ledger).unwrap();
    let rows = ledger_value["transitions"].as_array().unwrap();
    assert_eq!(rows.len(), 12);
    for (index, row) in rows.iter().enumerate() {
        let expected_action = report::EXPECTED_ACTIONS[index];
        let canonical = format!(r#"{{"action_id":"{}"}}"#, expected_action.action_id);
        assert_eq!(row["case_id"], expected_action.case_id);
        assert_eq!(row["request_slot"], expected_action.request_slot);
        assert_eq!(row["expected_action_id"], expected_action.action_id);
        assert_eq!(row["canonical_raw_response"], canonical);
        assert_eq!(
            row["canonical_response_sha256"],
            digest(canonical.as_bytes())
        );
        assert_eq!(
            canonical_claim2_action_output(expected_action.action_id),
            canonical
        );
        assert_eq!(row["advancing_raw_language_cardinality"], 1);
        assert_eq!(row["arm_scope"].as_array().unwrap().len(), 3);
        let pin = RECEIPTS[index];
        assert_eq!(row["deterministic_receipt_identity"], pin.event_id);
        assert_eq!(
            row["deterministic_receipt_path"],
            format!(
                "fixtures/claim2/{}/{}",
                expected_action.case_id.to_ascii_lowercase(),
                pin.path
            )
        );
        assert_eq!(row["deterministic_receipt_sha256"], pin.sha256);
        assert_eq!(row["next_state"], pin.state);
    }

    let predecessor =
        fs::read(repository.join("fixtures/claim2/materialization-report.json")).unwrap();
    assert_eq!(
        digest(&predecessor),
        "3d9571ddd4e17096b55971040531aac6197b6b9b7bc3d01afefd8992c28f6c65"
    );
    // Version 2 is a frozen six-case historical report. CP07 extends the
    // shared adapter after that report was accepted, so its old source hash
    // must remain a historical pin rather than being silently regenerated.
    // The exact artifact digest preserves every byte and its semantic claims;
    // CP07's versioned case-domain artifact is regenerated and replayed in
    // `claim2_cp07_materialization`.
    let successor =
        fs::read(repository.join("fixtures/claim2/materialization-report-v2.json")).unwrap();
    assert_eq!(
        digest(&successor),
        "30ecb777b52d201765ca7cba83ed9692519e63bb8e318ff515e93280e27b3dab"
    );
    let successor_value: Value = serde_json::from_slice(&successor).unwrap();
    assert_eq!(successor_value["schema_version"], 2);
    assert_eq!(
        successor_value["protocol_provenance"]["implementation_source_sha256_normalized_lf"],
        "b20499f2ac20113deddf8a1c97c0120618ea788f588cf6364e96454648750f06"
    );
    assert_eq!(
        successor_value["protocol_provenance"]["successor_generator_source_sha256_normalized_lf"],
        "4467f4a2777c89ef516f7a27a019d92bec58c83d1614693f791daa04aefc582c"
    );
    assert_eq!(
        successor_value["predecessor"]["sha256"],
        digest(&predecessor)
    );
    assert_eq!(
        successor_value["finite_domain_ledger"]["sha256"],
        digest(&ledger)
    );
    assert_eq!(
        successor_value["rendering_revalidation"]
            ["historical_static_projection_report_reproduces_exactly"],
        true
    );
    assert_eq!(
        successor_value["rendering_revalidation"]["historical_static_request_slots_revalidated"],
        54
    );
    assert_eq!(
        successor_value["rendering_revalidation"]["independently_replayed_arm_trajectories"],
        18
    );
    assert_eq!(
        successor_value["rendering_revalidation"]["canonical_advancing_transition_replays"],
        36
    );
    assert_eq!(
        successor_value["rendering_revalidation"]
            ["request_renderings_slots_1_through_3_revalidated"],
        54
    );
}
