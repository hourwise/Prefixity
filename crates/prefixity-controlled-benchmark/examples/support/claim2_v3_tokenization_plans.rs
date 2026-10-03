use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

#[path = "claim2_v3_workload.rs"]
#[allow(dead_code)]
mod v3_workload;

pub const LEDGER_PATH: &str = "fixtures/claim2/workload-request-ledger-v3.json";
pub const MAP_PATH: &str = "fixtures/claim2/token-evidence-inheritance-map-v3.json";
pub const HYBRID_PATH: &str = "fixtures/claim2/tokenization-contact-plan-v3-hybrid.json";
pub const FULL_FRESH_PATH: &str = "fixtures/claim2/tokenization-contact-plan-v3-full-fresh.json";
const LEDGER_SHA256: &str = "406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2";
const MAP_SHA256: &str = "6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2";
const V3_CASE_ORDER: [&str; 6] = ["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"];
const ARMS: [&str; 3] = ["BASELINE", "NO_OP", "INTERVENTION"];
const REQUEST_FIELDS: [&str; 7] = [
    "model",
    "messages",
    "max_tokens",
    "temperature",
    "top_p",
    "seed",
    "stream",
];

#[derive(Clone)]
struct RequestGroup {
    body: String,
    case_id: String,
    request_slot: u64,
    logical_request_ids: Vec<String>,
    arms: BTreeSet<String>,
    message_hashes: BTreeSet<String>,
}

type RequestGroups = BTreeMap<String, RequestGroup>;
type InheritedCounts = BTreeMap<String, u64>;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fail(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn parse(bytes: &[u8], name: &str) -> Result<Value, Box<dyn Error>> {
    serde_json::from_slice(bytes).map_err(|error| fail(format!("invalid {name}: {error}")).into())
}

fn required_str<'a>(value: &'a Value, key: &str, label: &str) -> Result<&'a str, Box<dyn Error>> {
    value[key]
        .as_str()
        .ok_or_else(|| fail(format!("missing {label}.{key}")).into())
}

fn verify_source_files(
    root: &Path,
    supplied_ledger: &[u8],
    supplied_map: &[u8],
) -> Result<(), Box<dyn Error>> {
    let regenerated_ledger = v3_workload::ledger_bytes(root)?;
    if digest(&regenerated_ledger) != LEDGER_SHA256
        || regenerated_ledger != supplied_ledger
        || fs::read(root.join(LEDGER_PATH))? != regenerated_ledger
    {
        return Err(fail(
            "V3 contact plan source ledger differs from the accepted renderer or frozen SHA-256",
        )
        .into());
    }
    if digest(supplied_map) != MAP_SHA256 || fs::read(root.join(MAP_PATH))? != supplied_map {
        return Err(fail(
            "V3 contact plan inheritance map differs from the frozen SHA-256 or tracked bytes",
        )
        .into());
    }
    Ok(())
}

fn expected_logical_ids() -> Vec<String> {
    let mut ids = Vec::with_capacity(54);
    for case_id in V3_CASE_ORDER {
        for arm in ARMS {
            for slot in 1..=3 {
                ids.push(format!("{case_id}/{arm}/slot-{slot}"));
            }
        }
    }
    ids
}

fn verify_map_header(map: &Value) -> Result<(), Box<dyn Error>> {
    if map["schema_id"] != "prefixity.phase1c.claim2-v3-token-evidence-inheritance-map"
        || map["schema_version"] != 3
        || map["status"] != "RETAINED_V1_REQUEST_IDENTITIES_VERIFIED_FOR_V3"
        || map["scope"]["cohort_order"] != json!(V3_CASE_ORDER)
        || map["scope"]["v3_request_ledger_sha256"] != LEDGER_SHA256
        || map["scope"]["logical_requests_covered"] != 54
        || map["scope"]["retained_logical_requests_verified"] != 36
        || map["scope"]["inherited_v1_logical_requests"] != 36
        || map["scope"]["inherited_v1_unique_request_hashes"] != 14
        || map["scope"]["new_logical_requests"] != 18
        || map["scope"]["new_unique_request_hashes_requiring_tokenization"] != 8
        || map["scope"]["unique_request_hashes"] != 22
        || map["scope"]["tokenization_performed_by_this_generator"] != false
        || map["scope"]["inference_performed_by_this_generator"] != false
        || map["historical_v1_bindings"]["identity_canonical_sha256"]
            != "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16"
        || map["historical_v1_bindings"]["raw_evidence_sha256"]
            != "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264"
        || map["historical_v1_bindings"]["interpreted_result_canonical_sha256"]
            != "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040"
    {
        return Err(fail(
            "V3 inheritance map identity, evidence, or zero-contact boundary changed",
        )
        .into());
    }
    Ok(())
}

fn validate_body(row: &Value, semantics: &Value) -> Result<(String, String), Box<dyn Error>> {
    let id = required_str(row, "logical_request_id", "request row")?;
    let body = required_str(row, "future_token_counter_body", "request row")?.to_owned();
    let hash = required_str(row, "request_body_sha256", "request row")?.to_owned();
    let slot = row["request_slot"]
        .as_u64()
        .ok_or_else(|| fail(format!("invalid request slot for {id}")))?;
    let case_id = required_str(row, "case_id", "request row")?;
    let arm = required_str(row, "arm", "request row")?;
    if !(1..=3).contains(&slot)
        || !V3_CASE_ORDER.contains(&case_id)
        || !ARMS.contains(&arm)
        || id != format!("{case_id}/{arm}/slot-{slot}")
        || digest(body.as_bytes()) != hash
        || row["request_body_utf8_byte_length"].as_u64() != Some(body.len() as u64)
        || row["messages_utf8_byte_length"].as_u64().is_none()
        || row["tokenization_status"] != "NOT_PERFORMED"
        || row["live_dispatchable"] != false
        || row["renderer_dispatchable"] != false
        || row["evidence_classification"].as_str().is_none()
    {
        return Err(fail(format!(
            "request identity or offline contract failed for {id}"
        ))
        .into());
    }
    let parsed = parse(body.as_bytes(), "exact rendered request body")?;
    let object = parsed
        .as_object()
        .ok_or_else(|| fail(format!("request body is not an object for {id}")))?;
    let actual_fields = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected_fields = REQUEST_FIELDS.into_iter().collect::<BTreeSet<_>>();
    if actual_fields != expected_fields
        || parsed["messages"] != row["messages"]
        || parsed["model"] != semantics["model"]
        || parsed["max_tokens"] != semantics["max_tokens"]
        || parsed["temperature"] != semantics["temperature"]
        || parsed["top_p"] != semantics["top_p"]
        || parsed["seed"] != semantics["seed"]
        || parsed["stream"] != semantics["stream"]
        || semantics["chat_template_kwargs"] != "ABSENT"
    {
        return Err(fail(format!(
            "request bytes or frozen settings do not match the ledger for {id}"
        ))
        .into());
    }
    Ok((body, hash))
}

fn request_groups(
    ledger: &Value,
    map: &Value,
) -> Result<(RequestGroups, InheritedCounts), Box<dyn Error>> {
    verify_map_header(map)?;
    if ledger["schema_id"] != "prefixity.phase1c.claim2-v3-workload-request-ledger"
        || ledger["schema_version"] != 3
        || ledger["cohort_order"] != json!(V3_CASE_ORDER)
        || ledger["offline_boundary"]["logical_requests"] != 54
        || ledger["offline_boundary"]["tokenization_performed"] != false
        || ledger["offline_boundary"]["server_contacts"] != 0
        || ledger["offline_boundary"]["inference_requests"] != 0
    {
        return Err(fail("V3 request ledger header or offline boundary changed").into());
    }
    let requests = ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("V3 request ledger has no requests"))?;
    let expected_ids = expected_logical_ids();
    if requests.len() != expected_ids.len() {
        return Err(fail("V3 request ledger does not contain exactly 54 logical requests").into());
    }
    let mut groups = BTreeMap::<String, RequestGroup>::new();
    let mut ids = BTreeSet::new();
    for (row, expected_id) in requests.iter().zip(expected_ids) {
        let id = required_str(row, "logical_request_id", "request row")?;
        if id != expected_id || !ids.insert(id.to_owned()) {
            return Err(fail("V3 request order, roster, or uniqueness changed").into());
        }
        let (body, hash) = validate_body(row, &ledger["request_semantics"])?;
        let case_id = required_str(row, "case_id", "request row")?.to_owned();
        let arm = required_str(row, "arm", "request row")?.to_owned();
        let slot = row["request_slot"]
            .as_u64()
            .ok_or_else(|| fail("invalid request slot"))?;
        let message_hash = required_str(row, "messages_sha256", "request row")?.to_owned();
        match groups.get_mut(&hash) {
            Some(group) => {
                if group.body.as_bytes() != body.as_bytes()
                    || group.case_id != case_id
                    || group.request_slot != slot
                {
                    return Err(fail(format!(
                        "request body hash group has conflicting bytes or location for {id}"
                    ))
                    .into());
                }
                group.arms.insert(arm);
                group.message_hashes.insert(message_hash);
                group.logical_request_ids.push(id.to_owned());
            }
            None => {
                groups.insert(
                    hash,
                    RequestGroup {
                        body,
                        case_id,
                        request_slot: slot,
                        logical_request_ids: vec![id.to_owned()],
                        arms: BTreeSet::from([arm]),
                        message_hashes: BTreeSet::from([message_hash]),
                    },
                );
            }
        }
    }
    if ids.len() != 54 || groups.len() != 22 {
        return Err(fail(format!(
            "V3 request ledger has {} unique bodies; expected 22 from its accepted materialization",
            groups.len()
        ))
        .into());
    }
    for group in groups.values_mut() {
        group.logical_request_ids.sort();
    }
    let inherited_counts = verify_map_groups(&groups, map)?;
    for row in requests {
        let id = required_str(row, "logical_request_id", "request row")?;
        let hash = required_str(row, "request_body_sha256", "request row")?;
        if let Some(input_tokens) = inherited_counts.get(hash) {
            if row["evidence_classification"] != "INHERITED_V1_TOKEN_COUNT"
                || row["inherited_v1_input_tokens"].as_u64() != Some(*input_tokens)
            {
                return Err(fail(format!(
                    "V1 inherited count classification changed for {id}"
                ))
                .into());
            }
        } else if row["evidence_classification"] != "NEW_TOKEN_COUNT_REQUIRED"
            || !row["inherited_v1_input_tokens"].is_null()
        {
            return Err(fail(format!("new V3 request classification changed for {id}")).into());
        }
    }
    Ok((groups, inherited_counts))
}

fn map_ids(row: &Value, key: &str) -> Result<BTreeSet<String>, Box<dyn Error>> {
    row[key]
        .as_array()
        .ok_or_else(|| fail(format!("inheritance map is missing {key}")))?
        .iter()
        .map(|id| {
            id.as_str()
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("inheritance map has invalid {key}")).into())
        })
        .collect()
}

fn verify_map_groups(
    groups: &BTreeMap<String, RequestGroup>,
    map: &Value,
) -> Result<BTreeMap<String, u64>, Box<dyn Error>> {
    let mut inherited = BTreeMap::new();
    for row in map["inherited_unique_request_hashes"]
        .as_array()
        .ok_or_else(|| fail("V3 inheritance map has no inherited bodies"))?
    {
        let hash = required_str(row, "request_body_sha256", "inherited map row")?.to_owned();
        let ids = map_ids(row, "v3_logical_request_ids")?;
        let input_tokens = row["input_tokens"]
            .as_u64()
            .ok_or_else(|| fail("V3 inherited count is missing"))?;
        if row["measurement_label"] != "V1_MEASUREMENT_REUSED_IN_V3"
            || row["request_body_utf8_byte_length"].as_u64().is_none()
            || inherited.insert(hash, (ids, row, input_tokens)).is_some()
        {
            return Err(fail("V3 inherited request map has invalid or duplicate evidence").into());
        }
    }
    let mut new = BTreeMap::new();
    for row in map["new_unique_request_hashes"]
        .as_array()
        .ok_or_else(|| fail("V3 inheritance map has no new-body list"))?
    {
        let hash = required_str(row, "request_body_sha256", "new map row")?.to_owned();
        let ids = map_ids(row, "v3_logical_request_ids")?;
        if row["request_body_utf8_byte_length"].as_u64().is_none()
            || !row.get("input_tokens").is_none()
            || new.insert(hash, (ids, row)).is_some()
        {
            return Err(fail("V3 new request map has invalid or duplicate evidence").into());
        }
    }
    let mapped_hashes = inherited
        .keys()
        .chain(new.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    if mapped_hashes.len() != groups.len()
        || mapped_hashes.iter().ne(groups.keys())
        || inherited.len() != 14
        || new.len() != 8
    {
        return Err(
            fail("V3 inheritance map does not cover the exact frozen request hash groups").into(),
        );
    }
    for (hash, group) in groups {
        let (ids, row) = if let Some((ids, row, _)) = inherited.get(hash) {
            (ids, *row)
        } else if let Some((ids, row)) = new.get(hash) {
            (ids, *row)
        } else {
            return Err(fail(format!("V3 inheritance map omits body {hash}")).into());
        };
        let expected_ids = group
            .logical_request_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let message_hashes = row["message_sequence_sha256_values"]
            .as_array()
            .ok_or_else(|| fail(format!("V3 map omits message identity for {hash}")))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail(format!("V3 map has invalid message identity for {hash}")))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if ids != &expected_ids
            || row["request_body_utf8_byte_length"].as_u64() != Some(group.body.len() as u64)
            || message_hashes != group.message_hashes
        {
            return Err(fail(format!(
                "V3 inheritance map identity differs from rendered body {hash}"
            ))
            .into());
        }
    }
    Ok(inherited
        .into_iter()
        .map(|(hash, (_, _, input_tokens))| (hash, input_tokens))
        .collect())
}

fn equality_class(group: &RequestGroup) -> Result<&'static str, Box<dyn Error>> {
    if group.arms
        == BTreeSet::from([
            "BASELINE".to_owned(),
            "NO_OP".to_owned(),
            "INTERVENTION".to_owned(),
        ])
    {
        Ok("ALL_ARMS")
    } else if group.arms == BTreeSet::from(["BASELINE".to_owned(), "NO_OP".to_owned()]) {
        Ok("BASELINE_EQUALS_NO_OP")
    } else if group.arms == BTreeSet::from(["INTERVENTION".to_owned()]) {
        Ok("INTERVENTION_ONLY")
    } else {
        Err(fail(format!(
            "unsupported treatment-equality class: {:?}",
            group.arms
        ))
        .into())
    }
}

fn plan_value(
    mode: &str,
    groups: &BTreeMap<String, RequestGroup>,
    inherited_hashes: &BTreeMap<String, u64>,
    ledger_hash: &str,
    map_hash: &str,
) -> Result<Value, Box<dyn Error>> {
    let selected = groups
        .iter()
        .filter(|(hash, _)| mode == "FULL_FRESH_22" || !inherited_hashes.contains_key(*hash))
        .collect::<Vec<_>>();
    let expected_count = match mode {
        "HYBRID_8_NEW" => 8,
        "FULL_FRESH_22" => 22,
        _ => return Err(fail("unknown V3 tokenization plan mode").into()),
    };
    if selected.len() != expected_count {
        return Err(fail(format!(
            "{mode} plan has {} entries, expected {expected_count}",
            selected.len()
        ))
        .into());
    }
    let mut entries = Vec::with_capacity(selected.len());
    for (hash, group) in selected {
        entries.push(json!({
            "request_body_sha256": hash,
            "request_body_utf8_byte_length": group.body.len(),
            "representative_logical_request_id": group.logical_request_ids.first(),
            "logical_request_ids": group.logical_request_ids,
            "case_id": group.case_id,
            "request_slot": group.request_slot,
            "treatment_equality_class": equality_class(group)?,
            "exact_request_body": group.body
        }));
    }
    let inherited_count = if mode == "HYBRID_8_NEW" { 14 } else { 0 };
    Ok(json!({
        "schema_id": "prefixity.phase1c.claim2-tokenization-v3-contact-plan",
        "schema_version": 3,
        "status": "FROZEN_OFFLINE_NO_CONTACTS",
        "mode": mode,
        "source_ledger": {"path": LEDGER_PATH, "sha256": ledger_hash},
        "source_inheritance_map": {"path": MAP_PATH, "sha256": map_hash},
        "logical_request_count": 54,
        "total_unique_body_count": 22,
        "inherited_unique_body_count": inherited_count,
        "fresh_unique_body_count": entries.len(),
        "maximum_readiness_contacts": 1,
        "maximum_token_count_contacts": entries.len(),
        "inference_allowance": 0,
        "ordering": "request_body_sha256 ascending, lowercase ASCII hex",
        "endpoint_allowlist": {
            "readiness": {"method": "GET", "path": "/health"},
            "token_count": {"method": "POST", "path": "/v1/chat/completions/input_tokens"}
        },
        "entries": entries
    }))
}

fn encode_plan(value: &Value) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn plan_pair_from_sources(
    root: &Path,
    ledger_bytes: &[u8],
    map_bytes: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    verify_source_files(root, ledger_bytes, map_bytes)?;
    let ledger = parse(ledger_bytes, "frozen V3 workload ledger")?;
    let map = parse(map_bytes, "frozen V3 inheritance map")?;
    let (groups, inherited_hashes) = request_groups(&ledger, &map)?;
    let ledger_hash = digest(ledger_bytes);
    let map_hash = digest(map_bytes);
    let hybrid = encode_plan(&plan_value(
        "HYBRID_8_NEW",
        &groups,
        &inherited_hashes,
        &ledger_hash,
        &map_hash,
    )?)?;
    let full = encode_plan(&plan_value(
        "FULL_FRESH_22",
        &groups,
        &inherited_hashes,
        &ledger_hash,
        &map_hash,
    )?)?;
    Ok((hybrid, full))
}

pub fn plan_pair(root: &Path) -> Result<(Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let ledger = v3_workload::ledger_bytes(root)?;
    let map = fs::read(root.join(MAP_PATH))?;
    plan_pair_from_sources(root, &ledger, &map)
}
