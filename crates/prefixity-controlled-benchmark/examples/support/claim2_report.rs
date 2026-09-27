use prefixity_controlled_benchmark::{
    load_claim2_case, precheck_claim2_cohort, project_claim2_trace, select_claim2_decision,
    Claim2ArmState, Claim2ProjectionMode,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::path::Path;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Offline evidence only. No fabricated assistant output is supplied to the renderer.
pub fn build_report(root: &Path) -> Result<Value, Box<dyn Error>> {
    let cases = (1..=6)
        .map(|n| load_claim2_case(&root.join(format!("fixtures/claim2/cp{n:02}/case.json"))))
        .collect::<Result<Vec<_>, _>>()?;
    let precheck = precheck_claim2_cohort(&cases)?;
    let mut case_records = Vec::new();
    let mut slots = Vec::new();
    for case in &cases {
        let manifest = case.manifest();
        let relative = format!(
            "fixtures/claim2/{}/case.json",
            case.case_id().to_lowercase()
        );
        let selection = select_claim2_decision(case)?;
        let baseline = project_claim2_trace(case, Claim2ProjectionMode::Baseline)?;
        let noop = project_claim2_trace(case, Claim2ProjectionMode::NoOp)?;
        let intervention = project_claim2_trace(case, Claim2ProjectionMode::Intervention)?;
        assert_eq!(baseline, noop);
        let removed_events = baseline
            .events
            .iter()
            .filter(|e| !intervention.events.iter().any(|i| i.event_id == e.event_id))
            .map(|e| &e.event_id)
            .collect::<Vec<_>>();
        let removed_relations = baseline
            .relations
            .iter()
            .filter(|r| {
                !intervention
                    .relations
                    .iter()
                    .any(|i| i.relation_id == r.relation_id)
            })
            .map(|r| &r.relation_id)
            .collect::<Vec<_>>();
        case_records.push(json!({
            "case_id": case.case_id(), "kind": manifest.kind,
            "manifest_path": relative,
            "manifest_file_sha256": digest(&fs::read(root.join(&relative))?),
            "trace_json_sha256": digest(&serde_json::to_vec(&manifest.planner_input)?),
            "evaluation_key_sha256": manifest.evaluation_key_sha256,
            "selection": selection,
            "removed_event_ids": removed_events, "removed_relation_ids": removed_relations,
            "baseline_noop_trace_equal": baseline == noop,
            "token_proof_inputs": { "D3_low": null, "B3_high": null, "Dsum_low": null, "sum_B_high": null },
            "token_bound_status": "EXACT_TOKENIZATION_REQUIRED"
        }));
        for mode in [
            Claim2ProjectionMode::Baseline,
            Claim2ProjectionMode::NoOp,
            Claim2ProjectionMode::Intervention,
        ] {
            let arm = Claim2ArmState::new(case, mode);
            for slot in 1..=3 {
                let request = arm.preview_request(case, slot)?;
                let message_bytes = request
                    .messages
                    .iter()
                    .enumerate()
                    .map(|(index, message)| {
                        json!({
                            "index": index, "role": message.role,
                            "content_utf8_bytes": message.content.len(),
                            "content_sha256": digest(message.content.as_bytes()),
                            "source_event_ids": message.source_event_ids,
                            "source_receipt_event_ids": message.source_receipt_event_ids
                        })
                    })
                    .collect::<Vec<_>>();
                slots.push(json!({
                    "case_id": case.case_id(), "arm": mode, "request_slot": slot,
                    "dispatchable": request.dispatchable,
                    "fully_bound_static_request": slot == 1,
                    "metrics": request.metrics,
                    "messages": message_bytes,
                    "static_skeleton_sha256": digest(&request.request_json),
                    "static_skeleton_json": String::from_utf8(request.request_json)?
                }));
            }
        }
    }
    assert_eq!(precheck.len(), 54);
    assert_eq!(slots.len(), 54);
    Ok(json!({
        "schema_id": "prefixity.phase1c.claim2-offline-materialization-report",
        "schema_version": 1,
        "starting_sha": "6b7e21ddd64b2b11e7e264e42d924d1eedc85dee",
        "planned_inference_slots": 54, "executed_inference_slots": 0,
        "classification": "CLAIM_2_WORKLOAD_MATERIALIZATION_READY_FOR_TOKENIZATION_REVIEW",
        "history_status": "UNBOUND_ARM_LOCAL_OUTPUTS_AND_RECEIPTS_IN_SLOTS_2_AND_3",
        "byte_estimates_are_output_caps": false,
        "conservative_prior_output_token_envelope_per_response": 1024,
        "conservative_prior_output_byte_bound": null,
        "cases": case_records, "slots": slots
    }))
}

pub fn report_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(&build_report(root)?)?;
    bytes.push(b'\n');
    Ok(bytes)
}
