use prefixity_controlled_benchmark::{
    canonical_claim2_action_output, load_claim2_case, Claim2ArmState, Claim2ProjectionMode,
    Claim2SlotStatus, LoadedClaim2Case, CLAIM2_CONTEXT_TOKENS, CLAIM2_INPUT_PREFLIGHT_TOKENS,
    CLAIM2_MAX_OUTPUT_TOKENS, CLAIM2_MODEL_LABEL,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::Path;

const CASE_IDS: [&str; 6] = ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"];
const ARM_MODES: [(Claim2ProjectionMode, &str); 3] = [
    (Claim2ProjectionMode::Baseline, "BASELINE"),
    (Claim2ProjectionMode::NoOp, "NO_OP"),
    (Claim2ProjectionMode::Intervention, "INTERVENTION"),
];
const SUCCESSOR_PATH: &str = "fixtures/claim2/materialization-report-v2.json";
const ADVANCING_DOMAIN_PATH: &str = "fixtures/claim2/advancing-output-domain-v1.json";
const MATERIALIZATION_PREDECESSOR_PATH: &str = "fixtures/claim2/materialization-report.json";
const OUTPUT_PATH: &str = "fixtures/claim2/tokenization-request-ledger-v1.json";
const ACCEPTED_SUCCESSOR_SHA256: &str =
    "30ecb777b52d201765ca7cba83ed9692519e63bb8e318ff515e93280e27b3dab";
const ACCEPTED_DOMAIN_SHA256: &str =
    "b4689157548864c819945dd8260478dc0ba6a5811dd96626768f3f5b83a09fc8";
const ACCEPTED_PREDECESSOR_SHA256: &str =
    "3d9571ddd4e17096b55971040531aac6197b6b9b7bc3d01afefd8992c28f6c65";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MessageRecord {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct WireMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Clone)]
struct OutputHistory {
    request_slot: u8,
    action_id: String,
    canonical_response: String,
    canonical_response_sha256: String,
}

#[derive(Clone)]
struct ReceiptHistory {
    request_slot: u8,
    receipt_id: String,
    receipt_path: String,
    receipt_sha256: String,
    state_after: String,
    text: String,
}

struct CanonicalTransition {
    action_id: String,
    response: String,
    response_sha256: String,
    receipt_id: String,
    receipt_path: String,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn arm_name(mode: Claim2ProjectionMode) -> &'static str {
    match mode {
        Claim2ProjectionMode::Baseline => "BASELINE",
        Claim2ProjectionMode::NoOp => "NO_OP",
        Claim2ProjectionMode::Intervention => "INTERVENTION",
    }
}

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn case_manifest_identity(root: &Path, case: &LoadedClaim2Case) -> Result<Value, Box<dyn Error>> {
    let manifest_path = format!(
        "fixtures/claim2/{}/case.json",
        case.case_id().to_ascii_lowercase()
    );
    let manifest_bytes = fs::read(root.join(&manifest_path))?;
    let public_manifest: Value = serde_json::from_slice(&manifest_bytes)?;
    if public_manifest["case_id"] != case.case_id()
        || public_manifest["evaluation_key_sha256"] != case.manifest().evaluation_key_sha256
        || public_manifest["evaluation_key_path"] != case.manifest().evaluation_key_path
    {
        return Err("loaded case and public fixture manifest identity differ".into());
    }
    Ok(json!({
        "case_id": case.case_id(),
        "manifest_path": manifest_path,
        "manifest_sha256": digest(&manifest_bytes),
        "evaluation_key_sha256": case.manifest().evaluation_key_sha256,
        "evaluation_key_path": case.manifest().evaluation_key_path,
    }))
}

fn protocol_transition<'a>(
    domain: &'a Value,
    case_id: &str,
    request_slot: u8,
) -> Result<&'a Value, Box<dyn Error>> {
    domain["transitions"]
        .as_array()
        .ok_or("advancing-domain ledger has no transitions")?
        .iter()
        .find(|row| {
            row["case_id"].as_str() == Some(case_id)
                && row["request_slot"].as_u64() == Some(u64::from(request_slot))
        })
        .ok_or_else(|| "canonical advancing transition is missing".into())
}

fn validate_fixture_identity(
    root: &Path,
    cases: &[LoadedClaim2Case],
    successor: &Value,
    domain: &Value,
) -> Result<(String, String, Vec<Value>), Box<dyn Error>> {
    let successor_bytes = fs::read(root.join(SUCCESSOR_PATH))?;
    let domain_bytes = fs::read(root.join(ADVANCING_DOMAIN_PATH))?;
    let predecessor_bytes = fs::read(root.join(MATERIALIZATION_PREDECESSOR_PATH))?;
    let predecessor: Value = serde_json::from_slice(&predecessor_bytes)?;
    if digest(&successor_bytes) != ACCEPTED_SUCCESSOR_SHA256
        || digest(&domain_bytes) != ACCEPTED_DOMAIN_SHA256
        || digest(&predecessor_bytes) != ACCEPTED_PREDECESSOR_SHA256
        || successor["predecessor"]["path"] != MATERIALIZATION_PREDECESSOR_PATH
        || successor["predecessor"]["sha256"] != ACCEPTED_PREDECESSOR_SHA256
    {
        return Err("accepted Claim-2 fixture ledger hashes changed".into());
    }
    if successor["status"] != "ADVANCING_RAW_HISTORY_DOMAIN_FINITE"
        || successor["finite_domain_ledger"]["path"] != ADVANCING_DOMAIN_PATH
        || successor["finite_domain_ledger"]["sha256"] != ACCEPTED_DOMAIN_SHA256
    {
        return Err(
            "accepted materialization successor identity does not match pinned domain".into(),
        );
    }
    if domain["schema_id"] != "prefixity.phase1c.claim2-advancing-output-domain-ledger"
        || domain["schema_version"] != 1
        || domain["advancing_raw_language_cardinality_per_transition"] != 1
        || domain["transitions"].as_array().map(Vec::len) != Some(12)
    {
        return Err("advancing-output-domain ledger schema or cardinality changed".into());
    }
    let fixture_identities = cases
        .iter()
        .map(|case| case_manifest_identity(root, case))
        .collect::<Result<Vec<_>, _>>()?;
    for (expected, actual) in CASE_IDS.iter().zip(&fixture_identities) {
        if actual["case_id"] != *expected {
            return Err("case fixture order or identity changed".into());
        }
        let predecessor_case = predecessor["cases"]
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["case_id"] == *expected))
            .ok_or("accepted materialization predecessor omits a case")?;
        if actual["manifest_path"] != predecessor_case["manifest_path"]
            || actual["manifest_sha256"] != predecessor_case["manifest_file_sha256"]
            || actual["evaluation_key_sha256"] != predecessor_case["evaluation_key_sha256"]
        {
            return Err(
                "current case fixture identity differs from accepted materialization".into(),
            );
        }
    }
    Ok((
        digest(&successor_bytes),
        digest(&domain_bytes),
        fixture_identities,
    ))
}

fn validate_transition_binding(
    root: &Path,
    case: &LoadedClaim2Case,
    transition: &Value,
    request_slot: u8,
) -> Result<CanonicalTransition, Box<dyn Error>> {
    let expected_action = transition["expected_action_id"]
        .as_str()
        .ok_or("transition has no expected action ID")?;
    let raw = transition["canonical_raw_response"]
        .as_str()
        .ok_or("transition has no canonical raw response")?
        .to_owned();
    let response_hash = transition["canonical_response_sha256"]
        .as_str()
        .ok_or("transition has no canonical response hash")?
        .to_owned();
    if raw != canonical_claim2_action_output(expected_action)
        || digest(raw.as_bytes()) != response_hash
        || transition["advancing_raw_language_cardinality"] != 1
        || transition["arm_scope"] != json!(["BASELINE", "NO_OP", "INTERVENTION"])
    {
        return Err("canonical output identity disagrees with accepted domain ledger".into());
    }
    let manifest_transition = case
        .manifest()
        .action_menu
        .iter()
        .find(|entry| entry.action_slot == request_slot && entry.action_id == expected_action)
        .ok_or("canonical action is absent from the public fixture menu")?;
    let receipt_id = transition["deterministic_receipt_identity"]
        .as_str()
        .ok_or("transition has no pinned receipt ID")?
        .to_owned();
    let receipt_path = transition["deterministic_receipt_path"]
        .as_str()
        .ok_or("transition has no pinned receipt path")?
        .to_owned();
    let receipt_hash = transition["deterministic_receipt_sha256"]
        .as_str()
        .ok_or("transition has no pinned receipt hash")?
        .to_owned();
    let next_state = transition["next_state"]
        .as_str()
        .ok_or("transition has no next state")?
        .to_owned();
    let case_relative_path = format!(
        "fixtures/claim2/{}/{}",
        case.case_id().to_ascii_lowercase(),
        case.manifest()
            .assets
            .iter()
            .find(|asset| asset.asset_id == manifest_transition.result_asset_id)
            .ok_or("transition receipt asset is absent")?
            .relative_path
    );
    let receipt_asset = case
        .manifest()
        .assets
        .iter()
        .find(|asset| asset.asset_id == manifest_transition.result_asset_id)
        .ok_or("transition receipt asset is absent")?;
    let actual_receipt = fs::read(root.join(&receipt_path))?;
    if receipt_id != manifest_transition.result_event_id
        || receipt_path != case_relative_path
        || receipt_hash != receipt_asset.sha256
        || digest(&actual_receipt) != receipt_hash
        || next_state != manifest_transition.state_after
    {
        return Err("pinned receipt identity differs from fixture transition".into());
    }
    Ok(CanonicalTransition {
        action_id: expected_action.to_owned(),
        response: raw,
        response_sha256: response_hash,
        receipt_id,
        receipt_path,
    })
}

fn validate_request_semantics(
    request_json: &[u8],
    messages: &[MessageRecord],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let body: Value = serde_json::from_slice(request_json)?;
    let object = body
        .as_object()
        .ok_or("future counter body must be a JSON object")?;
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
    let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual_fields != expected_fields
        || object.contains_key("chat_template_kwargs")
        || body["model"] != CLAIM2_MODEL_LABEL
        || body["max_tokens"] != CLAIM2_MAX_OUTPUT_TOKENS
        || body["temperature"] != 0
        || body["top_p"] != 1
        || body["seed"] != 1
        || body["stream"] != false
    {
        return Err("renderer body semantics differ from the accepted future request".into());
    }
    let body_messages: Vec<MessageRecord> = serde_json::from_value(body["messages"].clone())?;
    if body_messages != messages {
        return Err("future counter body messages differ from exact renderer output".into());
    }
    let exact_messages_json = serde_json::to_vec(
        &messages
            .iter()
            .map(|message| WireMessage {
                role: &message.role,
                content: &message.content,
            })
            .collect::<Vec<_>>(),
    )?;
    if !request_json
        .windows(exact_messages_json.len())
        .any(|window| window == exact_messages_json)
    {
        return Err(
            "serialized message hash bytes are not the request body's exact messages".into(),
        );
    }
    Ok(exact_messages_json)
}

fn output_history_records(
    history: &[OutputHistory],
    messages: &[MessageRecord],
    case_id: &str,
    arm: &str,
) -> Result<Vec<Value>, Box<dyn Error>> {
    history
        .iter()
        .map(|prior| {
            let matches = messages
                .iter()
                .enumerate()
                .filter(|(_, message)| {
                    message.role == "assistant" && message.content == prior.canonical_response
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err("canonical prior output is not represented once in rendered messages".into());
            }
            let output_id = format!(
                "{case_id}:slot-{}:sha256:{}",
                prior.request_slot, prior.canonical_response_sha256
            );
            Ok(json!({
                "request_slot": prior.request_slot,
                "action_id": prior.action_id,
                "canonical_output_identity": output_id,
                "arm_local_identity": format!("{case_id}/{arm}/slot-{}/sha256:{}", prior.request_slot, prior.canonical_response_sha256),
                "canonical_response": prior.canonical_response,
                "canonical_response_sha256": prior.canonical_response_sha256,
                "message_index": matches[0]
            }))
        })
        .collect()
}

fn receipt_history_records(
    history: &[ReceiptHistory],
    messages: &[MessageRecord],
    case_id: &str,
    arm: &str,
) -> Result<Vec<Value>, Box<dyn Error>> {
    history
        .iter()
        .map(|prior| {
            let matches = messages
                .iter()
                .enumerate()
                .filter(|(_, message)| message.role == "user" && message.content == prior.text)
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err("pinned prior receipt is not represented once in rendered messages".into());
            }
            Ok(json!({
                "request_slot": prior.request_slot,
                "receipt_id": prior.receipt_id,
                "arm_local_identity": format!("{case_id}/{arm}/receipt-{}/sha256:{}", prior.receipt_id, prior.receipt_sha256),
                "receipt_path": prior.receipt_path,
                "receipt_sha256": prior.receipt_sha256,
                "state_after": prior.state_after,
                "message_index": matches[0]
            }))
        })
        .collect()
}

fn logical_request_id(case_id: &str, arm: &str, request_slot: u8) -> String {
    format!("{case_id}/{arm}/slot-{request_slot}")
}

fn expected_identity_groups() -> BTreeSet<BTreeSet<String>> {
    let mut groups = BTreeSet::new();
    for case_id in CASE_IDS {
        for request_slot in [1_u8, 2_u8] {
            groups.insert(
                ARM_MODES
                    .iter()
                    .map(|(_, arm)| logical_request_id(case_id, arm, request_slot))
                    .collect(),
            );
        }
        if matches!(case_id, "CP01" | "CP02" | "CP03" | "CP04") {
            groups.insert(
                ["BASELINE", "NO_OP"]
                    .into_iter()
                    .map(|arm| logical_request_id(case_id, arm, 3))
                    .collect(),
            );
            groups.insert(BTreeSet::from([logical_request_id(
                case_id,
                "INTERVENTION",
                3,
            )]));
        } else {
            groups.insert(
                ARM_MODES
                    .iter()
                    .map(|(_, arm)| logical_request_id(case_id, arm, 3))
                    .collect(),
            );
        }
    }
    groups
}

fn make_request_record(
    case: &LoadedClaim2Case,
    fixture_identity: &Value,
    mode: Claim2ProjectionMode,
    request: &prefixity_controlled_benchmark::Claim2RenderedRequest,
    output_history: &[OutputHistory],
    receipt_history: &[ReceiptHistory],
) -> Result<Value, Box<dyn Error>> {
    if request.dispatchable
        || !request.metrics.unbound_raw_assistant_slots.is_empty()
        || !request.metrics.unbound_environment_receipt_slots.is_empty()
    {
        return Err("realized canonical history contains unbound slots or is dispatchable".into());
    }
    let messages = request
        .messages
        .iter()
        .map(|message| MessageRecord {
            role: message.role.clone(),
            content: message.content.clone(),
        })
        .collect::<Vec<_>>();
    let messages_json = validate_request_semantics(&request.request_json, &messages)?;
    let body = String::from_utf8(request.request_json.clone())?;
    let arm = arm_name(mode);
    Ok(json!({
        "logical_request_id": logical_request_id(case.case_id(), arm, request.request_slot),
        "case_id": case.case_id(),
        "fixture_identity": fixture_identity,
        "arm": arm,
        "request_slot": request.request_slot,
        "canonical_prior_output_identities": output_history_records(output_history, &messages, case.case_id(), arm)?,
        "pinned_prior_receipts": receipt_history_records(receipt_history, &messages, case.case_id(), arm)?,
        "messages": messages,
        "messages_sha256": digest(&messages_json),
        "messages_utf8_byte_length": messages_json.len(),
        "future_token_counter_body": body,
        "request_body_sha256": digest(&request.request_json),
        "request_body_utf8_byte_length": request.request_json.len(),
        "renderer_dispatchable": request.dispatchable,
        "tokenizable": true,
        "tokenizable_meaning": "exact accepted renderer body prepared for a future authorized input_tokens call; no tokenization occurred",
        "live_dispatchable": false,
        "live_dispatchable_meaning": "offline preparation artifact only; this generator has no server or HTTP path",
        "tokenization_status": "NOT_PERFORMED",
        "token_count_status": request.metrics.token_count_status,
        "unbound_raw_assistant_slots": request.metrics.unbound_raw_assistant_slots,
        "unbound_environment_receipt_slots": request.metrics.unbound_environment_receipt_slots
    }))
}

pub fn report_bytes(root: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let cases = CASE_IDS
        .iter()
        .map(|case_id| {
            load_claim2_case(&root.join(format!(
                "fixtures/claim2/{}/case.json",
                case_id.to_ascii_lowercase()
            )))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let successor = read_json(&root.join(SUCCESSOR_PATH))?;
    let domain = read_json(&root.join(ADVANCING_DOMAIN_PATH))?;
    let (successor_sha256, domain_sha256, fixture_identities) =
        validate_fixture_identity(root, &cases, &successor, &domain)?;
    let fixture_by_case = fixture_identities
        .iter()
        .map(|identity| {
            (
                identity["case_id"].as_str().expect("fixture has case ID"),
                identity,
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut requests = Vec::with_capacity(54);
    for case in &cases {
        let fixture_identity = fixture_by_case[case.case_id()];
        for (mode, _) in ARM_MODES {
            let mut state = Claim2ArmState::new(case, mode);
            let mut output_history = Vec::with_capacity(2);
            let mut receipt_history = Vec::with_capacity(2);
            for request_slot in 1..=3 {
                let request = state.render_next(case)?;
                if request.case_id != case.case_id()
                    || request.arm != mode
                    || request.request_slot != request_slot
                {
                    return Err("renderer returned a different logical request identity".into());
                }
                requests.push(make_request_record(
                    case,
                    fixture_identity,
                    mode,
                    &request,
                    &output_history,
                    &receipt_history,
                )?);

                if request_slot < 3 {
                    let transition = protocol_transition(&domain, case.case_id(), request_slot)?;
                    let canonical_transition =
                        validate_transition_binding(root, case, transition, request_slot)?;
                    // Construct a fresh response for this arm; no other arm's output is reused.
                    let arm_local_response = canonical_transition.response.clone();
                    let evaluation = state.record_output(case, arm_local_response.clone())?;
                    if evaluation.is_some()
                        || state.slots()[usize::from(request_slot - 1)].status
                            != Claim2SlotStatus::Pass
                        || state.slots()[usize::from(request_slot - 1)]
                            .raw_assistant_output
                            .as_deref()
                            != Some(arm_local_response.as_str())
                    {
                        return Err("canonical arm-local output did not advance exactly".into());
                    }
                    let receipt = state
                        .environment_receipts()
                        .last()
                        .ok_or("canonical advancing output produced no receipt")?;
                    let receipt_sha256 = digest(receipt.text.as_bytes());
                    if receipt.request_slot != request_slot
                        || receipt.source_event_id != canonical_transition.receipt_id
                        || receipt.state_after
                            != transition["next_state"].as_str().unwrap_or_default()
                        || receipt_sha256 != transition["deterministic_receipt_sha256"]
                    {
                        return Err("arm-local receipt differs from its pinned transition".into());
                    }
                    output_history.push(OutputHistory {
                        request_slot,
                        action_id: canonical_transition.action_id,
                        canonical_response: arm_local_response,
                        canonical_response_sha256: canonical_transition.response_sha256,
                    });
                    receipt_history.push(ReceiptHistory {
                        request_slot,
                        receipt_id: canonical_transition.receipt_id,
                        receipt_path: canonical_transition.receipt_path,
                        receipt_sha256,
                        state_after: receipt.state_after.clone(),
                        text: receipt.text.clone(),
                    });
                }
            }
            if state.slots()[0].status != Claim2SlotStatus::Pass
                || state.slots()[1].status != Claim2SlotStatus::Pass
                || state.slots()[2].status != Claim2SlotStatus::Planned
            {
                // Request 3 is intentionally rendered for tokenization, but no final output is invented.
                return Err("request-three preparation unexpectedly consumed an answer".into());
            }
        }
    }

    let mut groups: BTreeMap<String, (usize, String, Vec<String>)> = BTreeMap::new();
    for request in &requests {
        let hash = request["request_body_sha256"]
            .as_str()
            .ok_or("logical request has no body hash")?
            .to_owned();
        let body = request["future_token_counter_body"]
            .as_str()
            .ok_or("logical request has no exact future body")?;
        let length = request["request_body_utf8_byte_length"]
            .as_u64()
            .ok_or("logical request has no body byte length")? as usize;
        if digest(body.as_bytes()) != hash || body.len() != length {
            return Err("logical request body hash or UTF-8 byte length is inconsistent".into());
        }
        let logical_id = request["logical_request_id"]
            .as_str()
            .ok_or("logical request has no identity")?
            .to_owned();
        match groups.get_mut(&hash) {
            Some((stored_length, stored_body, members)) => {
                if *stored_length != length || stored_body.as_bytes() != body.as_bytes() {
                    return Err("same SHA-256 maps to different exact request bodies".into());
                }
                members.push(logical_id);
            }
            None => {
                groups.insert(hash, (length, body.to_owned(), vec![logical_id]));
            }
        }
    }
    let hash_groups = groups
        .iter()
        .map(|(hash, (length, _, members))| {
            json!({
                "request_body_sha256": hash,
                "request_body_utf8_byte_length": length,
                "logical_request_ids": members
            })
        })
        .collect::<Vec<_>>();
    let observed_groups = groups
        .values()
        .map(|(_, _, members)| members.iter().cloned().collect::<BTreeSet<_>>())
        .collect::<BTreeSet<_>>();
    let expected_groups = expected_identity_groups();
    let identity_structure_matches = observed_groups == expected_groups && groups.len() == 22;
    let unique_bodies_by_slot = (1..=3)
        .map(|slot| {
            let hashes = requests
                .iter()
                .filter(|request| request["request_slot"].as_u64() == Some(slot))
                .map(|request| request["request_body_sha256"].as_str().unwrap_or_default())
                .collect::<BTreeSet<_>>();
            (slot.to_string(), json!(hashes.len()))
        })
        .collect::<Map<String, Value>>();
    let report = json!({
        "schema_id": "prefixity.phase1c.claim2-tokenization-request-ledger",
        "schema_version": 1,
        "classification": "OFFLINE_NON_INFERENCE_TOKENIZATION_PREPARATION",
        "status": if identity_structure_matches { "REQUEST_IDENTITY_STRUCTURE_MATCHES_REVIEW_EXPECTATION" } else { "TOKENIZATION_REQUEST_IDENTITY_MISMATCH" },
        "fixture_identities": {
            "accepted_materialization_successor": { "path": SUCCESSOR_PATH, "sha256": successor_sha256 },
            "advancing_output_domain_ledger": { "path": ADVANCING_DOMAIN_PATH, "sha256": domain_sha256 },
            "accepted_materialization_predecessor": { "path": MATERIALIZATION_PREDECESSOR_PATH, "sha256": ACCEPTED_PREDECESSOR_SHA256 },
            "cases": fixture_identities
        },
        "request_semantics": {
            "token_counter_endpoint": "/v1/chat/completions/input_tokens",
            "model": CLAIM2_MODEL_LABEL,
            "max_tokens": CLAIM2_MAX_OUTPUT_TOKENS,
            "temperature": 0,
            "top_p": 1,
            "seed": 1,
            "stream": false,
            "chat_template_kwargs": "ABSENT",
            "content_type": "TEXT_ONLY",
            "context_tokens": CLAIM2_CONTEXT_TOKENS,
            "input_preflight_limit_tokens": CLAIM2_INPUT_PREFLIGHT_TOKENS
        },
        "offline_boundary": {
            "logical_requests": requests.len(),
            "tokenization_performed": false,
            "server_contacts": 0,
            "inference_requests": 0,
            "no_live_dispatch_interface": true,
            "request_three_final_output_constructed": false
        },
        "deduplication": {
            "deduplication_key": "SHA-256 of exact future_token_counter_body UTF-8 bytes",
            "logical_request_count": requests.len(),
            "unique_request_body_count": groups.len(),
            "duplicate_logical_requests_avoided_if_contacted_later": requests.len().saturating_sub(groups.len()),
            "unique_bodies_by_request_slot": unique_bodies_by_slot,
            "equality_structure_matches_review_expectation": identity_structure_matches,
            "review_expectation": {
                "request_slots_1_and_2": "one exact body per case across BASELINE, NO_OP, INTERVENTION",
                "positive_request_slot_3": "BASELINE equals NO_OP; INTERVENTION differs for CP01-CP04",
                "control_request_slot_3": "BASELINE equals NO_OP equals INTERVENTION for CP05-CP06",
                "unique_request_bodies": 22
            },
            "hash_groups": hash_groups
        },
        "requests": requests
    });
    let mut bytes = serde_json::to_vec_pretty(&report)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn output_path() -> &'static str {
    OUTPUT_PATH
}
