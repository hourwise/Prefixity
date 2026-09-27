use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, load_claim2_case, Claim2ArmState, Claim2ProjectionMode,
    Claim2SlotStatus, LoadedClaim2Case, CLAIM2_ADVANCING_OUTPUT_PROTOCOL_ID,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::path::Path;

#[path = "claim2_report.rs"]
mod historical_report;

#[derive(Clone, Copy)]
pub struct ExpectedAction {
    pub case_id: &'static str,
    pub request_slot: u8,
    pub action_id: &'static str,
}

pub const EXPECTED_ACTIONS: [ExpectedAction; 12] = [
    ExpectedAction {
        case_id: "CP01",
        request_slot: 1,
        action_id: "inspect_retry_implementation",
    },
    ExpectedAction {
        case_id: "CP01",
        request_slot: 2,
        action_id: "run_retry_regression",
    },
    ExpectedAction {
        case_id: "CP02",
        request_slot: 1,
        action_id: "build_symbol_inventory",
    },
    ExpectedAction {
        case_id: "CP02",
        request_slot: 2,
        action_id: "inspect_resolution_chain",
    },
    ExpectedAction {
        case_id: "CP03",
        request_slot: 1,
        action_id: "collect_dependency_inventory",
    },
    ExpectedAction {
        case_id: "CP03",
        request_slot: 2,
        action_id: "verify_snapshot_and_constraints",
    },
    ExpectedAction {
        case_id: "CP04",
        request_slot: 1,
        action_id: "run_candidate_guardrail_probe",
    },
    ExpectedAction {
        case_id: "CP04",
        request_slot: 2,
        action_id: "verify_restored_state",
    },
    ExpectedAction {
        case_id: "CP05",
        request_slot: 1,
        action_id: "inspect_initial_catalog",
    },
    ExpectedAction {
        case_id: "CP05",
        request_slot: 2,
        action_id: "rebuild_route_index_and_export",
    },
    ExpectedAction {
        case_id: "CP06",
        request_slot: 1,
        action_id: "inspect_policy_index",
    },
    ExpectedAction {
        case_id: "CP06",
        request_slot: 2,
        action_id: "verify_audit_receipt",
    },
];

type DomainEvidence = (Vec<Value>, Vec<Vec<Value>>);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn normalized_source_hash(path: &Path) -> Result<String, Box<dyn Error>> {
    let source = String::from_utf8(fs::read(path)?)?;
    Ok(digest(source.replace("\r\n", "\n").as_bytes()))
}

fn load_cases(root: &Path) -> Result<Vec<LoadedClaim2Case>, Box<dyn Error>> {
    (1..=6)
        .map(|number| {
            load_claim2_case(&root.join(format!("fixtures/claim2/cp{number:02}/case.json")))
                .map_err(Into::into)
        })
        .collect()
}

fn build_domain_rows(
    root: &Path,
    cases: &[LoadedClaim2Case],
) -> Result<DomainEvidence, Box<dyn Error>> {
    let mut ledger_rows = Vec::with_capacity(EXPECTED_ACTIONS.len());
    let mut arm_rows = (0..EXPECTED_ACTIONS.len())
        .map(|_| Vec::with_capacity(3))
        .collect::<Vec<_>>();
    for point in EXPECTED_ACTIONS {
        let case = cases
            .iter()
            .find(|case| case.case_id() == point.case_id)
            .ok_or("expected case is absent")?;
        let transition = case
            .manifest()
            .action_menu
            .iter()
            .find(|entry| {
                entry.action_slot == point.request_slot && entry.action_id == point.action_id
            })
            .ok_or("frozen expected action has no public transition")?;
        let receipt = case
            .manifest()
            .assets
            .iter()
            .find(|asset| asset.asset_id == transition.result_asset_id)
            .ok_or("frozen transition has no receipt asset")?;
        let receipt_path = format!(
            "fixtures/claim2/{}/{}",
            point.case_id.to_ascii_lowercase(),
            receipt.relative_path
        );
        let receipt_bytes = fs::read(root.join(&receipt_path))?;
        if digest(&receipt_bytes) != receipt.sha256 {
            return Err("pinned receipt bytes changed".into());
        }
        let canonical = canonical_claim2_action_output(point.action_id);
        ledger_rows.push(json!({
            "case_id": point.case_id,
            "request_slot": point.request_slot,
            "expected_action_id": point.action_id,
            "canonical_raw_response": canonical,
            "canonical_response_sha256": digest(canonical.as_bytes()),
            "deterministic_receipt_identity": transition.result_event_id,
            "deterministic_receipt_path": receipt_path,
            "deterministic_receipt_sha256": receipt.sha256,
            "next_state": transition.state_after,
            "advancing_raw_language_cardinality": 1,
            "arm_scope": ["BASELINE", "NO_OP", "INTERVENTION"]
        }));
    }

    for case in cases {
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            let mut arm = Claim2ArmState::new(case, mode);
            let mut carried_responses = Vec::with_capacity(2);
            let mut request = arm.render_next(case)?;
            for slot in 1..=2 {
                if request.request_slot != slot {
                    return Err("arm rendered an unexpected request slot".into());
                }
                let request_sha256 = digest(&request.request_json);
                let transition_index = EXPECTED_ACTIONS
                    .iter()
                    .position(|candidate| {
                        candidate.case_id == case.case_id() && candidate.request_slot == slot
                    })
                    .ok_or("missing expected action for arm replay")?;
                let expected = EXPECTED_ACTIONS[transition_index];
                // Construct a new response from the frozen identity for every arm.
                let raw = canonical_claim2_action_output(expected.action_id);
                let evaluation = arm.record_output(case, raw.clone())?;
                if evaluation.is_some()
                    || arm.slots()[usize::from(slot - 1)].status != Claim2SlotStatus::Pass
                    || arm.slots()[usize::from(slot - 1)]
                        .raw_assistant_output
                        .as_deref()
                        != Some(raw.as_str())
                {
                    return Err("canonical arm response did not advance exactly".into());
                }
                let (receipt_identity, receipt_state, receipt_sha256) = {
                    let receipt = arm
                        .environment_receipts()
                        .last()
                        .ok_or("canonical response produced no receipt")?;
                    (
                        receipt.source_event_id.clone(),
                        receipt.state_after.clone(),
                        digest(receipt.text.as_bytes()),
                    )
                };
                let current_transition = case
                    .manifest()
                    .action_menu
                    .iter()
                    .find(|entry| {
                        entry.action_slot == slot && entry.action_id == expected.action_id
                    })
                    .ok_or("canonical expected action has no transition")?;
                if receipt_identity != current_transition.result_event_id
                    || receipt_state != current_transition.state_after
                {
                    return Err("canonical response produced the wrong receipt".into());
                }
                let next_request = arm.render_next(case)?;
                if !next_request
                    .messages
                    .iter()
                    .any(|message| message.role == "assistant" && message.content == raw)
                {
                    return Err("next request did not carry the exact response".into());
                }
                if slot == 2 {
                    let carried = next_request
                        .messages
                        .iter()
                        .filter(|message| message.role == "assistant")
                        .map(|message| message.content.as_str())
                        .collect::<Vec<_>>();
                    let expected_carried = carried_responses
                        .iter()
                        .map(String::as_str)
                        .chain(std::iter::once(raw.as_str()))
                        .collect::<Vec<_>>();
                    if carried != expected_carried {
                        return Err("request 3 did not carry both arm-local responses".into());
                    }
                }
                arm_rows[transition_index].push(json!({
                    "arm": mode,
                    "request_sha256": request_sha256,
                    "next_request_sha256": digest(&next_request.request_json),
                    "canonical_response_sha256": digest(raw.as_bytes()),
                    "receipt_identity": receipt_identity,
                    "receipt_sha256": receipt_sha256,
                    "next_state": receipt_state
                }));
                carried_responses.push(raw);
                request = next_request;
            }
        }
    }
    Ok((ledger_rows, arm_rows))
}

fn encode_ledger(rows: &[Value]) -> Result<Vec<u8>, Box<dyn Error>> {
    let ledger = json!({
        "schema_id": "prefixity.phase1c.claim2-advancing-output-domain-ledger",
        "schema_version": 1,
        "protocol_id": CLAIM2_ADVANCING_OUTPUT_PROTOCOL_ID,
        "classification": "WORKLOAD_PROTOCOL_DOMAIN_CORRECTION",
        "scope": "intermediate action outputs at request slots 1 and 2 only",
        "arm_scope": ["BASELINE", "NO_OP", "INTERVENTION"],
        "advancing_raw_language_cardinality_per_transition": 1,
        "transitions": rows
    });
    let mut bytes = serde_json::to_vec_pretty(&ledger)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn ledger_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let cases = load_cases(root)?;
    let (rows, _) = build_domain_rows(root, &cases)?;
    encode_ledger(&rows)
}

pub fn successor_report_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let prior_path = root.join("fixtures/claim2/materialization-report.json");
    let prior_bytes = fs::read(&prior_path)?;
    let regenerated_prior = historical_report::report_bytes(root)?;
    if regenerated_prior != prior_bytes {
        return Err("historical materialization report no longer reproduces byte-for-byte".into());
    }
    let prior: Value = serde_json::from_slice(&prior_bytes)?;
    let cases = load_cases(root)?;
    let (rows, arm_rows) = build_domain_rows(root, &cases)?;
    let committed_ledger = encode_ledger(&rows)?;
    let ledger_path = root.join("fixtures/claim2/advancing-output-domain-v1.json");
    if fs::read(&ledger_path)? != committed_ledger {
        return Err("checked-in advancing-domain ledger is not reproducible".into());
    }
    let materialization_source =
        root.join("crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs");
    let generator_source =
        root.join("crates/prefixity-controlled-benchmark/examples/support/claim2_domain_report.rs");
    let mode_count = 3;
    let report = json!({
        "schema_id": "prefixity.phase1c.claim2-offline-materialization-successor",
        "schema_version": 2,
        "classification": "WORKLOAD_PROTOCOL_DOMAIN_CORRECTION",
        "status": "ADVANCING_RAW_HISTORY_DOMAIN_FINITE",
        "protocol_id": CLAIM2_ADVANCING_OUTPUT_PROTOCOL_ID,
        "predecessor": {
            "path": "fixtures/claim2/materialization-report.json",
            "schema_version": prior["schema_version"],
            "sha256": digest(&prior_bytes),
            "classification": prior["classification"]
        },
        "protocol_provenance": {
            "implementation_source_path": "crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs",
            "implementation_source_sha256_normalized_lf": normalized_source_hash(&materialization_source)?,
            "successor_generator_source_path": "crates/prefixity-controlled-benchmark/examples/support/claim2_domain_report.rs",
            "successor_generator_source_sha256_normalized_lf": normalized_source_hash(&generator_source)?,
            "canonical_wire_grammar": "UTF-8 bytes exactly equal to compact JSON {\"action_id\":<serde_json string>}, with no BOM, leading/trailing whitespace, terminator, extra field, alternate object spelling, array, or fence"
        },
        "finite_domain_ledger": {
            "path": "fixtures/claim2/advancing-output-domain-v1.json",
            "sha256": digest(&committed_ledger),
            "transition_count": rows.len(),
            "transition_cardinality": 1,
            "arms": mode_count
        },
        "rendering_revalidation": {
            "historical_static_projection_report_reproduces_exactly": true,
            "historical_static_request_slots_revalidated": prior["slots"].as_array().map_or(0, Vec::len),
            "fixture_count": cases.len(),
            "independently_replayed_arm_trajectories": cases.len() * mode_count,
            "canonical_advancing_transition_replays": rows.len() * mode_count,
            "request_renderings_slots_1_through_3_revalidated": cases.len() * mode_count * 3,
            "arm_local_raw_carry_to_later_request": true,
            "receipt_identity_and_state_checks": "PASS"
        },
        "arm_transition_revalidation": rows.iter().zip(arm_rows.iter()).map(|(row, arms)| json!({
            "case_id": row["case_id"],
            "request_slot": row["request_slot"],
            "expected_action_id": row["expected_action_id"],
            "arm_replays": arms
        })).collect::<Vec<_>>(),
        "unchanged_boundaries": [
            "fixture manifests, evaluation keys, task bodies, menus, receipts, attachments, and dependencies",
            "Phase 1B.9 policy, planner, selection, and NO_OP projection",
            "final request-3 output schema and evaluator",
            "accepted 1024-token response envelope; no byte cap added",
            "no inference, tokenizer, or server contact"
        ],
        "next_authorized_task": "NON_INFERENCE_CONTEXT_TOKENIZATION_PASS"
    });
    let mut bytes = serde_json::to_vec_pretty(&report)?;
    bytes.push(b'\n');
    Ok(bytes)
}
