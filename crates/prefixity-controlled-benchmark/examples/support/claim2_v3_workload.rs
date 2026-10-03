use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, load_claim2_case, select_claim2_decision, Claim2ArmState,
    Claim2ProjectionMode, Claim2SlotStatus, LoadedClaim2Case, ResearchInterventionClass,
    CLAIM2_CONTEXT_TOKENS, CLAIM2_INPUT_PREFLIGHT_TOKENS, CLAIM2_MAX_OUTPUT_TOKENS,
    CLAIM2_MODEL_LABEL,
};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::Path;

#[path = "claim2_v2_token_evidence_inheritance.rs"]
#[allow(dead_code)]
mod inheritance;
#[path = "claim2_tokenization_report.rs"]
#[allow(dead_code)]
mod v1_renderer;

const CASE_IDS: [&str; 6] = ["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"];
const ARMS: [(Claim2ProjectionMode, &str); 3] = [
    (Claim2ProjectionMode::Baseline, "BASELINE"),
    (Claim2ProjectionMode::NoOp, "NO_OP"),
    (Claim2ProjectionMode::Intervention, "INTERVENTION"),
];
pub const LEDGER_PATH: &str = "fixtures/claim2/workload-request-ledger-v3.json";
pub const SUCCESSOR_PATH: &str = "fixtures/claim2/materialization-report-v4.json";
pub const DOMAIN_PATH: &str = "fixtures/claim2/advancing-output-domain-v3.json";
const V1_LEDGER_PATH: &str = "fixtures/claim2/tokenization-request-ledger-v1.json";
const V1_RESULT_PATH: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json";
const V1_SUCCESSOR_PATH: &str = "fixtures/claim2/materialization-report-v2.json";
const V1_SUCCESSOR_SHA: &str = "30ecb777b52d201765ca7cba83ed9692519e63bb8e318ff515e93280e27b3dab";
const V1_DOMAIN_PATH: &str = "fixtures/claim2/advancing-output-domain-v1.json";
const V1_DOMAIN_SHA: &str = "b4689157548864c819945dd8260478dc0ba6a5811dd96626768f3f5b83a09fc8";
const V2_LEDGER_PATH: &str = "fixtures/claim2/workload-request-ledger-v2.json";
const V2_LEDGER_SHA: &str = "a539c33355827ef574912ee72e235bf6301bd0acdc6666fa30effebed242a5eb";
const V2_SUCCESSOR_PATH: &str = "fixtures/claim2/materialization-report-v3.json";
const V2_SUCCESSOR_SHA: &str = "a24c37cb879b658736d06680425d4e0dec036cf4ff85ef63eba1206ed7ff4732";
const V2_DOMAIN_PATH: &str = "fixtures/claim2/advancing-output-domain-v2.json";
const V2_DOMAIN_SHA: &str = "d1439be00db6ff7a63e2646124008c05af6be715172ad64427c99f929aebf1c9";
const V3_CONTRACT_PATH: &str = "fixtures/claim2/workload-contract-v3.md";
const V3_INHERITANCE_PATH: &str = "fixtures/claim2/token-evidence-inheritance-map-v3.json";

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn read_json(root: &Path, path: &str) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(root.join(path))?)?)
}

fn encode(value: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn case_identity(root: &Path, case: &LoadedClaim2Case) -> Result<Value, Box<dyn Error>> {
    let path = format!(
        "fixtures/claim2/{}/case.json",
        case.case_id().to_ascii_lowercase()
    );
    let bytes = fs::read(root.join(&path))?;
    Ok(json!({
        "case_id": case.case_id(),
        "manifest_path": path,
        "manifest_sha256": digest(&bytes),
        "evaluation_key_sha256": case.manifest().evaluation_key_sha256,
        "evaluation_key_path": case.manifest().evaluation_key_path
    }))
}

fn policy_decision(case: &LoadedClaim2Case) -> Result<Value, Box<dyn Error>> {
    let selection = select_claim2_decision(case)?;
    let positive = case.case_id() == "CP02"
        || case.case_id() == "CP03"
        || case.case_id() == "CP09"
        || case.case_id() == "CP10";
    if positive {
        if selection.decision.class != ResearchInterventionClass::Prune
            || selection.decision.rule != "EXACT_DUPLICATE_PRUNE"
            || selection.candidates.exact_duplicate_prune.len() != 1
            || !selection.candidates.explicit_supersession_defer.is_empty()
            || !selection.candidates.same_zone_protocol_relocate.is_empty()
        {
            return Err(format!("{} lacks its sole frozen PRUNE target", case.case_id()).into());
        }
    } else if selection.decision.class != ResearchInterventionClass::DoNothing
        || !selection.candidates.exact_duplicate_prune.is_empty()
    {
        return Err(format!("{} is not an exact zero-mutation control", case.case_id()).into());
    }
    Ok(json!({
        "policy_id": "controlled-evidence-policy-v1",
        "class": if positive { "PRUNE" } else { "DO_NOTHING" },
        "rule": selection.decision.rule,
        "target_event_id": selection.decision.target_event_id,
        "eligible_prune_candidates": selection.candidates.exact_duplicate_prune.len(),
        "eligible_defer_candidates": selection.candidates.explicit_supersession_defer.len(),
        "eligible_relocate_candidates": selection.candidates.same_zone_protocol_relocate.len()
    }))
}

fn verify_history(
    root: &Path,
    case: &LoadedClaim2Case,
    slot: u8,
    point: &Value,
) -> Result<(String, String), Box<dyn Error>> {
    if point["request_slot"] != slot || point["raw_language_cardinality"] != 1 {
        return Err("new-case advancing point has wrong slot/cardinality".into());
    }
    let action = point["expected_action_id"]
        .as_str()
        .ok_or("missing action ID")?;
    let raw = point["canonical_raw_utf8"]
        .as_str()
        .ok_or("missing raw output")?;
    if raw != canonical_claim2_action_output(action)
        || digest(raw.as_bytes()) != point["canonical_raw_sha256"]
    {
        return Err("new-case advancing raw output is noncanonical".into());
    }
    let transition = case
        .manifest()
        .action_menu
        .iter()
        .find(|entry| entry.action_slot == slot && entry.action_id == action)
        .ok_or("canonical action not in menu")?;
    let receipt_path = point["receipt_path"]
        .as_str()
        .ok_or("missing receipt path")?;
    let receipt = fs::read(root.join(format!(
        "fixtures/claim2/{}/{}",
        case.case_id().to_ascii_lowercase(),
        receipt_path
    )))?;
    if digest(&receipt) != point["receipt_sha256"]
        || point["receipt_event_id"] != transition.result_event_id
        || point["state_after"] != transition.state_after
    {
        return Err("new-case receipt does not match canonical transition".into());
    }
    Ok((raw.to_owned(), receipt_path.to_owned()))
}

fn render_new_case(
    root: &Path,
    case: &LoadedClaim2Case,
    identity: &Value,
    decision: &Value,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let domain_path = format!(
        "fixtures/claim2/{}/advancing-output-domain-v3.json",
        case.case_id().to_ascii_lowercase()
    );
    let domain = read_json(root, &domain_path)?;
    if domain["case_id"] != case.case_id() || domain["points"].as_array().map(Vec::len) != Some(2) {
        return Err("new-case advancing domain is incomplete".into());
    }
    let points = domain["points"]
        .as_array()
        .ok_or("missing advancing points")?;
    let mut rows = Vec::with_capacity(9);
    for (mode, arm_name) in ARMS {
        let mut state = Claim2ArmState::new(case, mode);
        let mut prior_outputs = Vec::new();
        let mut prior_receipts = Vec::new();
        for slot in 1..=3 {
            let rendered = state.render_next(case)?;
            if rendered.request_slot != slot
                || rendered.arm != mode
                || rendered.case_id != case.case_id()
                || rendered.dispatchable
                || !rendered.metrics.unbound_raw_assistant_slots.is_empty()
                || !rendered
                    .metrics
                    .unbound_environment_receipt_slots
                    .is_empty()
            {
                return Err("new-case renderer identity or canonical history failed".into());
            }
            let messages = rendered
                .messages
                .iter()
                .map(|message| {
                    json!({
                        "role": message.role, "content": message.content
                    })
                })
                .collect::<Vec<_>>();
            let wire = rendered
                .messages
                .iter()
                .map(|message| WireMessage {
                    role: &message.role,
                    content: &message.content,
                })
                .collect::<Vec<_>>();
            let message_bytes = serde_json::to_vec(&wire)?;
            let body = String::from_utf8(rendered.request_json.clone())?;
            let body_value: Value = serde_json::from_slice(&rendered.request_json)?;
            let expected_fields = [
                "model",
                "messages",
                "max_tokens",
                "temperature",
                "top_p",
                "seed",
                "stream",
            ]
            .into_iter()
            .collect::<BTreeSet<_>>();
            let actual_fields = body_value
                .as_object()
                .ok_or("new-case body is not an object")?
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            if actual_fields != expected_fields
                || body_value["messages"] != json!(messages)
                || body_value["model"] != CLAIM2_MODEL_LABEL
                || body_value["max_tokens"] != CLAIM2_MAX_OUTPUT_TOKENS
                || body_value["temperature"] != 0
                || body_value["top_p"] != 1
                || body_value["seed"] != 1
                || body_value["stream"] != false
                || !rendered
                    .request_json
                    .windows(message_bytes.len())
                    .any(|window| window == message_bytes)
            {
                return Err("new-case wire semantics differ from accepted request".into());
            }
            rows.push(json!({
                "logical_request_id": format!("{}/{arm_name}/slot-{slot}", case.case_id()),
                "case_id": case.case_id(),
                "arm": arm_name,
                "request_slot": slot,
                "fixture_identity": identity,
                "messages": messages,
                "messages_sha256": digest(&message_bytes),
                "messages_utf8_byte_length": message_bytes.len(),
                "future_token_counter_body": body,
                "request_body_sha256": digest(&rendered.request_json),
                "request_body_utf8_byte_length": rendered.request_json.len(),
                "canonical_prior_output_identities": prior_outputs,
                "pinned_prior_receipts": prior_receipts,
                "policy_decision": decision,
                "renderer_dispatchable": false,
                "tokenizable": true,
                "live_dispatchable": false,
                "tokenization_status": "NOT_PERFORMED",
                "token_count_status": rendered.metrics.token_count_status,
                "unbound_raw_assistant_slots": rendered.metrics.unbound_raw_assistant_slots,
                "unbound_environment_receipt_slots": rendered.metrics.unbound_environment_receipt_slots
            }));
            if slot < 3 {
                let point = &points[usize::from(slot - 1)];
                let (raw, receipt_path) = verify_history(root, case, slot, point)?;
                if state.record_output(case, raw.clone())?.is_some()
                    || state.slots()[usize::from(slot - 1)].status != Claim2SlotStatus::Pass
                {
                    return Err("new-case canonical action failed to advance".into());
                }
                let receipt = state
                    .environment_receipts()
                    .last()
                    .ok_or("no receipt after action")?;
                if digest(receipt.text.as_bytes()) != point["receipt_sha256"]
                    || receipt.source_event_id != point["receipt_event_id"]
                {
                    return Err("new-case arm-local receipt changed".into());
                }
                prior_outputs.push(json!({
                    "request_slot": slot,
                    "action_id": point["expected_action_id"],
                    "canonical_response": raw,
                    "canonical_response_sha256": point["canonical_raw_sha256"],
                    "arm_local_identity": format!("{}/{arm_name}/slot-{slot}/sha256:{}", case.case_id(), point["canonical_raw_sha256"].as_str().unwrap_or_default())
                }));
                prior_receipts.push(json!({
                    "request_slot": slot,
                    "receipt_id": point["receipt_event_id"],
                    "receipt_path": format!("fixtures/claim2/{}/{}", case.case_id().to_ascii_lowercase(), receipt_path),
                    "receipt_sha256": point["receipt_sha256"],
                    "state_after": point["state_after"]
                }));
            }
        }
        if state.slots()[2].status != Claim2SlotStatus::Planned {
            return Err("offline ledger unexpectedly completed request 3".into());
        }
    }
    Ok(rows)
}

pub fn domain_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let historical_bytes = fs::read(root.join(V2_DOMAIN_PATH))?;
    if digest(&historical_bytes) != V2_DOMAIN_SHA {
        return Err("historical advancing-output domain changed".into());
    }
    let historical: Value = serde_json::from_slice(&historical_bytes)?;
    let mut transitions = Vec::with_capacity(12);
    for case_id in CASE_IDS {
        if case_id == "CP09" || case_id == "CP10" {
            let path = format!(
                "fixtures/claim2/{}/advancing-output-domain-v3.json",
                case_id.to_ascii_lowercase()
            );
            let domain = read_json(root, &path)?;
            let case = load_claim2_case(&root.join(format!(
                "fixtures/claim2/{}/case.json",
                case_id.to_ascii_lowercase()
            )))?;
            for (index, point) in domain["points"]
                .as_array()
                .ok_or("new case lacks domain points")?
                .iter()
                .enumerate()
            {
                let slot = index as u8 + 1;
                verify_history(root, &case, slot, point)?;
                transitions.push(json!({
                    "case_id": case_id, "request_slot": slot,
                    "expected_action_id": point["expected_action_id"],
                    "canonical_raw_response": point["canonical_raw_utf8"],
                    "canonical_response_sha256": point["canonical_raw_sha256"],
                    "deterministic_receipt_identity": point["receipt_event_id"],
                    "deterministic_receipt_path": format!("fixtures/claim2/{}/{}", case_id.to_ascii_lowercase(), point["receipt_path"].as_str().ok_or("receipt path missing")?),
                    "deterministic_receipt_sha256": point["receipt_sha256"],
                    "next_state": point["state_after"],
                    "advancing_raw_language_cardinality": 1,
                    "arm_scope": ["BASELINE", "NO_OP", "INTERVENTION"]
                }));
            }
        } else {
            for slot in 1..=2 {
                let point = historical["transitions"]
                    .as_array()
                    .ok_or("V2 domain transitions missing")?
                    .iter()
                    .find(|point| point["case_id"] == case_id && point["request_slot"] == slot)
                    .ok_or("retained V2 advancing point missing")?;
                transitions.push(point.clone());
            }
        }
    }
    if transitions.len() != 12 {
        return Err("V3 finite domain does not have 12 points".into());
    }
    encode(&json!({
        "schema_id": "prefixity.phase1c.claim2-v3-advancing-output-domain-ledger",
        "schema_version": 3,
        "protocol_id": "prefixity.phase1c.claim2-canonical-advancing-output.v1",
        "cohort_order": CASE_IDS,
        "advancing_raw_language_cardinality_per_transition": 1,
        "arm_scope": ["BASELINE", "NO_OP", "INTERVENTION"],
        "historical_domain": {"path": V2_DOMAIN_PATH, "sha256": V2_DOMAIN_SHA},
        "transitions": transitions
    }))
}

pub fn ledger_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let domain = domain_bytes(root)?;
    if fs::read(root.join(DOMAIN_PATH))? != domain {
        return Err("checked-in V3 finite domain differs from regeneration".into());
    }
    let v1_regenerated = v1_renderer::report_bytes(root)?;
    let map_regenerated = inheritance::map_bytes(root, &v1_regenerated)?;
    let map_bytes = fs::read(root.join(inheritance::OUTPUT_PATH))?;
    if map_regenerated != map_bytes {
        return Err("V1_TOKEN_EVIDENCE_NOT_REUSABLE: inheritance map drift".into());
    }
    let v1: Value = serde_json::from_slice(&v1_regenerated)?;
    let result = read_json(root, V1_RESULT_PATH)?;
    let counts = result["authoritative_unique_counts"]
        .as_array()
        .ok_or("V1 counts missing")?
        .iter()
        .map(|row| {
            (
                row["request_body_sha256"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                row["input_tokens"].as_u64().unwrap_or_default(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if counts.len() != 22 || counts.values().any(|count| *count == 0) {
        return Err("sealed V1 counts are incomplete".into());
    }
    let v1_rows = v1["requests"]
        .as_array()
        .ok_or("V1 requests missing")?
        .iter()
        .map(|row| {
            (
                row["logical_request_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                row.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let v2_bytes = fs::read(root.join(V2_LEDGER_PATH))?;
    if digest(&v2_bytes) != V2_LEDGER_SHA {
        return Err("historical V2 request ledger changed".into());
    }
    let v2: Value = serde_json::from_slice(&v2_bytes)?;
    let v2_rows = v2["requests"]
        .as_array()
        .ok_or("V2 requests missing")?
        .iter()
        .map(|row| {
            (
                row["logical_request_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                row,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let v1_bodies = v1_rows
        .values()
        .map(|row| {
            (
                row["request_body_sha256"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                row["future_token_counter_body"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::with_capacity(54);
    let mut identities = Vec::with_capacity(6);
    let mut decisions = Vec::with_capacity(6);
    let mut domains = Vec::with_capacity(2);
    for case_id in CASE_IDS {
        let case = load_claim2_case(&root.join(format!(
            "fixtures/claim2/{}/case.json",
            case_id.to_ascii_lowercase()
        )))?;
        let identity = case_identity(root, &case)?;
        let decision = policy_decision(&case)?;
        if case_id == "CP09" || case_id == "CP10" {
            let domain_path = format!(
                "fixtures/claim2/{}/advancing-output-domain-v3.json",
                case_id.to_ascii_lowercase()
            );
            domains.push(json!({"case_id": case_id, "path": domain_path, "sha256": digest(&fs::read(root.join(&domain_path))?)}));
            rows.extend(render_new_case(root, &case, &identity, &decision)?);
        } else {
            for (_, arm_name) in ARMS {
                for slot in 1..=3 {
                    let id = format!("{case_id}/{arm_name}/slot-{slot}");
                    let mut row = v1_rows
                        .get(&id)
                        .ok_or("retained V1 request missing")?
                        .clone();
                    if row["fixture_identity"] != identity {
                        return Err(
                            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained fixture changed".into()
                        );
                    }
                    let v2_row = v2_rows.get(&id).ok_or("retained V2 request missing")?;
                    for field in [
                        "future_token_counter_body",
                        "request_body_sha256",
                        "messages",
                        "messages_sha256",
                        "canonical_prior_output_identities",
                        "pinned_prior_receipts",
                    ] {
                        if row[field] != v2_row[field] {
                            return Err(
                                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained V2 request drift".into(),
                            );
                        }
                    }
                    row["policy_decision"] = decision.clone();
                    rows.push(row);
                }
            }
        }
        identities.push(identity);
        decisions.push(json!({"case_id": case_id, "decision": decision}));
    }
    if rows.len() != 54 {
        return Err("V3 cohort did not render 54 requests".into());
    }
    let mut groups: BTreeMap<String, (String, Vec<String>)> = BTreeMap::new();
    let mut inherited_logical = 0_usize;
    for row in &mut rows {
        let hash = row["request_body_sha256"]
            .as_str()
            .ok_or("request hash missing")?
            .to_owned();
        let body = row["future_token_counter_body"]
            .as_str()
            .ok_or("exact body missing")?
            .to_owned();
        let length = row["request_body_utf8_byte_length"]
            .as_u64()
            .ok_or("body length missing")? as usize;
        if digest(body.as_bytes()) != hash || body.len() != length {
            return Err("V3 request body/hash/byte length mismatch".into());
        }
        let id = row["logical_request_id"]
            .as_str()
            .ok_or("request ID missing")?
            .to_owned();
        let is_new_case = row["case_id"] == "CP09" || row["case_id"] == "CP10";
        if let Some((old_body, members)) = groups.get_mut(&hash) {
            if old_body.as_bytes() != body.as_bytes() {
                return Err("SHA collision or body drift".into());
            }
            members.push(id);
        } else {
            groups.insert(hash.clone(), (body.clone(), vec![id]));
        }
        if let (Some(v1_body), Some(count)) = (v1_bodies.get(&hash), counts.get(&hash)) {
            if is_new_case {
                return Err("new case unexpectedly collides with V1 token evidence".into());
            }
            if v1_body.as_bytes() != body.as_bytes() {
                return Err("V1_TOKEN_EVIDENCE_NOT_REUSABLE: hash matches but bytes differ".into());
            }
            row["evidence_classification"] = json!("INHERITED_V1_TOKEN_COUNT");
            row["inherited_v1_input_tokens"] = json!(count);
            inherited_logical += 1;
        } else {
            if !is_new_case {
                return Err("V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained request lacks count".into());
            }
            row["evidence_classification"] = json!("NEW_TOKEN_COUNT_REQUIRED");
            row["inherited_v1_input_tokens"] = Value::Null;
        }
    }
    let inherited_hashes = groups
        .keys()
        .filter(|hash| counts.contains_key(*hash))
        .count();
    let new_hashes = groups.len() - inherited_hashes;
    if inherited_logical != 36 || inherited_hashes != 14 {
        return Err(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained scope differs from accepted proof".into(),
        );
    }
    for case_id in ["CP05", "CP06"] {
        for slot in 1..=3 {
            let bodies = rows
                .iter()
                .filter(|row| row["case_id"] == case_id && row["request_slot"] == slot)
                .map(|row| row["request_body_sha256"].as_str().unwrap_or_default())
                .collect::<BTreeSet<_>>();
            if bodies.len() != 1 {
                return Err("control arms diverged".into());
            }
        }
    }
    let group_rows = groups.iter().map(|(hash, (body, ids))| json!({
        "request_body_sha256": hash,
        "request_body_utf8_byte_length": body.len(),
        "logical_request_ids": ids,
        "evidence_classification": if counts.contains_key(hash) { "INHERITED_V1_TOKEN_COUNT" } else { "NEW_TOKEN_COUNT_REQUIRED" }
    })).collect::<Vec<_>>();
    let v1_successor = fs::read(root.join(V1_SUCCESSOR_PATH))?;
    let v1_domain = fs::read(root.join(V1_DOMAIN_PATH))?;
    if digest(&v1_successor) != V1_SUCCESSOR_SHA || digest(&v1_domain) != V1_DOMAIN_SHA {
        return Err("historical V1 materialization evidence changed".into());
    }
    for (path, sha) in [
        (V2_LEDGER_PATH, V2_LEDGER_SHA),
        (V2_SUCCESSOR_PATH, V2_SUCCESSOR_SHA),
        (V2_DOMAIN_PATH, V2_DOMAIN_SHA),
    ] {
        if digest(&fs::read(root.join(path))?) != sha {
            return Err("historical V2 materialization evidence changed".into());
        }
    }
    encode(&json!({
        "schema_id": "prefixity.phase1c.claim2-v3-workload-request-ledger",
        "schema_version": 3,
        "classification": "OFFLINE_V3_WORKLOAD_MATERIALIZATION",
        "status": "EXACT_REQUEST_BODIES_FROZEN_UNCOUNTED_FOR_NEW_CASES",
        "cohort_order": CASE_IDS,
        "case_roles": ["positive", "positive", "positive", "positive", "control", "control"],
        "fixture_identities": identities,
        "policy_decisions": decisions,
        "new_case_advancing_domains": domains,
        "v3_advancing_domain": {"path": DOMAIN_PATH, "sha256": digest(&domain)},
        "historical_bindings": {
            "v1_request_ledger": {"path": V1_LEDGER_PATH, "sha256": digest(&v1_regenerated)},
            "v1_materialization_successor": {"path": V1_SUCCESSOR_PATH, "sha256": V1_SUCCESSOR_SHA},
            "v1_advancing_domain": {"path": V1_DOMAIN_PATH, "sha256": V1_DOMAIN_SHA},
            "v2_inheritance_map": {"path": inheritance::OUTPUT_PATH, "sha256": digest(&map_bytes)},
            "v2_request_ledger": {"path": V2_LEDGER_PATH, "sha256": V2_LEDGER_SHA},
            "v2_materialization_successor": {"path": V2_SUCCESSOR_PATH, "sha256": V2_SUCCESSOR_SHA},
            "v2_advancing_domain": {"path": V2_DOMAIN_PATH, "sha256": V2_DOMAIN_SHA},
            "v1_tokenization_identity": "claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91",
            "v1_identity_canonical_sha256": "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16",
            "v1_raw_evidence_sha256": "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264",
            "v1_result_canonical_sha256": "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040"
        },
        "request_semantics": {
            "model": CLAIM2_MODEL_LABEL, "max_tokens": CLAIM2_MAX_OUTPUT_TOKENS,
            "temperature": 0, "top_p": 1, "seed": 1, "stream": false,
            "chat_template_kwargs": "ABSENT", "context_tokens": CLAIM2_CONTEXT_TOKENS,
            "input_preflight_limit_tokens": CLAIM2_INPUT_PREFLIGHT_TOKENS
        },
        "offline_boundary": {"logical_requests": 54, "arm_trajectories": 18,
            "tokenization_performed": false, "server_contacts": 0, "inference_requests": 0,
            "request_three_final_output_constructed": false},
        "deduplication": {
            "logical_request_count": rows.len(),
            "unique_request_body_count": groups.len(),
            "inherited_v1_unique_request_hashes": inherited_hashes,
            "inherited_v1_logical_requests": inherited_logical,
            "new_unique_request_hashes_requiring_tokenization": new_hashes,
            "new_logical_requests": rows.len() - inherited_logical,
            "hash_groups": group_rows
        },
        "requests": rows
    }))
}

pub fn successor_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let ledger = ledger_bytes(root)?;
    if fs::read(root.join(LEDGER_PATH))? != ledger {
        return Err("checked-in V3 request ledger differs from regeneration".into());
    }
    let value: Value = serde_json::from_slice(&ledger)?;
    let source_path = "crates/prefixity-controlled-benchmark/src/phase1c_claim2_workload.rs";
    let generator_path =
        "crates/prefixity-controlled-benchmark/examples/support/claim2_v3_workload.rs";
    let source = fs::read(root.join(source_path))?;
    let generator = fs::read(root.join(generator_path))?;
    let normalize = |bytes: &[u8]| String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    encode(&json!({
        "schema_id": "prefixity.phase1c.claim2-v3-offline-materialization-successor",
        "schema_version": 4,
        "classification": "OFFLINE_V3_WORKLOAD_MATERIALIZATION",
        "status": "CLAIM_2_WORKLOAD_V3_MATERIALIZED_UNTOKENIZED",
        "historical_predecessor": {"path": V2_SUCCESSOR_PATH, "sha256": V2_SUCCESSOR_SHA},
        "request_ledger": {"path": LEDGER_PATH, "sha256": digest(&ledger),
            "logical_requests": 54, "unique_bodies": value["deduplication"]["unique_request_body_count"]},
        "inheritance_map": {"path": V3_INHERITANCE_PATH, "sha256": digest(&fs::read(root.join(V3_INHERITANCE_PATH))?)},
        "new_case_advancing_domains": value["new_case_advancing_domains"],
        "v3_advancing_domain": value["v3_advancing_domain"],
        "workload_contract": {"path": V3_CONTRACT_PATH, "sha256": digest(&fs::read(root.join(V3_CONTRACT_PATH))?)},
        "case_order": CASE_IDS,
        "fixture_identities": value["fixture_identities"],
        "policy_decisions": value["policy_decisions"],
        "rendering_revalidation": {"cases": 6, "arm_trajectories": 18,
            "request_renderings": 54, "canonical_advancing_transitions": 36,
            "inherited_v1_logical_requests": value["deduplication"]["inherited_v1_logical_requests"],
            "new_logical_requests": value["deduplication"]["new_logical_requests"]},
        "source_provenance": {
            "renderer_path": source_path, "renderer_sha256_normalized_lf": digest(normalize(&source).as_bytes()),
            "generator_path": generator_path, "generator_sha256_normalized_lf": digest(normalize(&generator).as_bytes())
        },
        "offline_boundary": value["offline_boundary"],
        "next_authorized_task": "CLAIM_2_WORKLOAD_V3_TOKENIZATION_PREPARATION"
    }))
}
