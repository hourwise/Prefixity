use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

#[path = "claim2_v2_token_evidence_inheritance.rs"]
#[allow(dead_code)]
mod v2_inheritance;
#[path = "claim2_v2_workload.rs"]
#[allow(dead_code)]
mod v2_workload;

pub const OUTPUT_PATH: &str = "fixtures/claim2/token-evidence-inheritance-map-v3.json";
const V1_LEDGER_PATH: &str = "fixtures/claim2/tokenization-request-ledger-v1.json";
const V1_LEDGER_SHA256: &str = "739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45";
const V2_LEDGER_PATH: &str = "fixtures/claim2/workload-request-ledger-v2.json";
const V2_LEDGER_SHA256: &str = "a539c33355827ef574912ee72e235bf6301bd0acdc6666fa30effebed242a5eb";
const V2_MAP_PATH: &str = "fixtures/claim2/token-evidence-inheritance-map-v2.json";
const V2_MAP_SHA256: &str = "2d33c1bd253f586d0363620ae14379ee2de49e58429598eff8b5b1c780bd268d";
const V2_SUCCESSOR_PATH: &str = "fixtures/claim2/materialization-report-v3.json";
const V2_SUCCESSOR_SHA256: &str =
    "a24c37cb879b658736d06680425d4e0dec036cf4ff85ef63eba1206ed7ff4732";
const V3_LEDGER_PATH: &str = "fixtures/claim2/workload-request-ledger-v3.json";
const RETAINED_CASES: [&str; 4] = ["CP02", "CP03", "CP05", "CP06"];
const V3_COHORT_ORDER: [&str; 6] = ["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"];
const V2_COHORT_ORDER: [&str; 6] = ["CP02", "CP03", "CP07", "CP08", "CP05", "CP06"];
const ARMS: [&str; 3] = ["BASELINE", "NO_OP", "INTERVENTION"];
type Measurement = (u64, usize, BTreeSet<String>, BTreeSet<String>);
type Measurements = BTreeMap<String, Measurement>;
type GroupResult = (Vec<Value>, Vec<Value>, usize, usize, usize);

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fail(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn verify_sha256(name: &str, bytes: &[u8], expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = digest(bytes);
    if actual != expected {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} SHA-256 changed (expected {expected}, found {actual})"
        ))
        .into());
    }
    Ok(())
}

fn parse_json(bytes: &[u8], name: &str) -> Result<Value, Box<dyn Error>> {
    serde_json::from_slice(bytes).map_err(|error| {
        fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: invalid {name}: {error}"
        ))
        .into()
    })
}

fn required_str<'a>(value: &'a Value, key: &str, name: &str) -> Result<&'a str, Box<dyn Error>> {
    value[key].as_str().ok_or_else(|| {
        fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing {name}.{key}"
        ))
        .into()
    })
}

fn row_index<'a>(
    ledger: &'a Value,
    name: &str,
) -> Result<BTreeMap<String, &'a Value>, Box<dyn Error>> {
    let rows = ledger["requests"].as_array().ok_or_else(|| {
        fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} has no request array"
        ))
    })?;
    let mut indexed = BTreeMap::new();
    for row in rows {
        let id = required_str(row, "logical_request_id", name)?.to_owned();
        if indexed.insert(id.clone(), row).is_some() {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: duplicate {name} logical request {id}"
            ))
            .into());
        }
    }
    Ok(indexed)
}

fn retained_count(rows: &BTreeMap<String, &Value>) -> usize {
    rows.values()
        .filter(|row| {
            row["case_id"]
                .as_str()
                .is_some_and(|case_id| RETAINED_CASES.contains(&case_id))
        })
        .count()
}

fn expected_logical_ids(cohort: &[&str]) -> Vec<String> {
    let mut ids = Vec::with_capacity(cohort.len() * ARMS.len() * 3);
    for case_id in cohort {
        for arm in ARMS {
            for slot in 1..=3 {
                ids.push(format!("{case_id}/{arm}/slot-{slot}"));
            }
        }
    }
    ids
}

fn verify_order(
    ledger: &Value,
    expected_cohort: &[&str],
    name: &str,
) -> Result<(), Box<dyn Error>> {
    if ledger["cohort_order"] != json!(expected_cohort) {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} cohort order changed"
        ))
        .into());
    }
    let rows = ledger["requests"].as_array().ok_or_else(|| {
        fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} has no request array"
        ))
    })?;
    let ids = rows
        .iter()
        .map(|row| required_str(row, "logical_request_id", name).map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    if ids != expected_logical_ids(expected_cohort) {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} request order or coverage changed"
        ))
        .into());
    }
    Ok(())
}

fn compare_exact_rendering(
    source: &Value,
    target: &Value,
    logical_id: &str,
) -> Result<(), Box<dyn Error>> {
    for field in [
        "case_id",
        "arm",
        "request_slot",
        "logical_request_id",
        "messages",
        "messages_sha256",
        "messages_utf8_byte_length",
        "future_token_counter_body",
        "request_body_sha256",
        "request_body_utf8_byte_length",
        "canonical_prior_output_identities",
        "pinned_prior_receipts",
        "fixture_identity",
    ] {
        if source[field] != target[field] {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained rendered request drift for {logical_id} at {field}"
            ))
            .into());
        }
    }
    let body = required_str(source, "future_token_counter_body", "source request")?;
    let hash = required_str(source, "request_body_sha256", "source request")?;
    if digest(body.as_bytes()) != hash
        || source["request_body_utf8_byte_length"].as_u64() != Some(body.len() as u64)
        || target["request_body_utf8_byte_length"].as_u64() != Some(body.len() as u64)
    {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained body bytes/hash/length disagree for {logical_id}"
        ))
        .into());
    }
    Ok(())
}

fn compare_retained_rows(
    source: &Value,
    target: &Value,
    source_name: &str,
    target_name: &str,
    expected_count: usize,
) -> Result<Vec<String>, Box<dyn Error>> {
    let source_rows = row_index(source, source_name)?;
    let target_rows = row_index(target, target_name)?;
    let target_retained = retained_count(&target_rows);
    if target_retained != expected_count {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {target_name} retained request count is {target_retained}, expected {expected_count} from the sealed V1 map"
        ))
        .into());
    }
    let mut ids = Vec::with_capacity(expected_count);
    for (logical_id, source_row) in source_rows {
        if !source_row["case_id"]
            .as_str()
            .is_some_and(|case_id| RETAINED_CASES.contains(&case_id))
        {
            continue;
        }
        let target_row = target_rows.get(&logical_id).ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {target_name} omits retained request {logical_id}"
            ))
        })?;
        compare_exact_rendering(source_row, target_row, &logical_id)?;
        ids.push(logical_id);
    }
    if ids.len() != expected_count {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {source_name} retained request count is {}, expected {expected_count} from the sealed V1 map",
            ids.len(),
        ))
        .into());
    }
    Ok(ids)
}

fn verify_ledger_bodies(ledger: &Value, name: &str) -> Result<(), Box<dyn Error>> {
    let rows = ledger["requests"].as_array().ok_or_else(|| {
        fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} has no request array"
        ))
    })?;
    if rows.len() != 54 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} has {} requests, expected 54",
            rows.len()
        ))
        .into());
    }
    for row in rows {
        let id = required_str(row, "logical_request_id", name)?;
        let body = required_str(row, "future_token_counter_body", name)?;
        let hash = required_str(row, "request_body_sha256", name)?;
        if digest(body.as_bytes()) != hash
            || row["request_body_utf8_byte_length"].as_u64() != Some(body.len() as u64)
        {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} request body failed hash/length validation for {id}"
            ))
            .into());
        }
    }
    Ok(())
}

fn v1_measurements(v2_map: &Value) -> Result<Measurements, Box<dyn Error>> {
    let rows = v2_map["inherited_unique_request_hashes"]
        .as_array()
        .ok_or_else(|| {
            fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V2 inheritance map has no V1 count mapping")
        })?;
    let mut measurements = BTreeMap::new();
    for row in rows {
        let hash = required_str(row, "request_body_sha256", "V2 inherited count")?.to_owned();
        let input_tokens = row["input_tokens"].as_u64().ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing V1 count for {hash}"
            ))
        })?;
        let byte_length = row["request_body_utf8_byte_length"]
            .as_u64()
            .ok_or_else(|| {
                fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing body length for {hash}"
                ))
            })? as usize;
        let v1_ids = row["v1_logical_request_ids"]
            .as_array()
            .ok_or_else(|| {
                fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing V1 IDs for {hash}"
                ))
            })?
            .iter()
            .map(|id| {
                id.as_str().map(str::to_owned).ok_or_else(|| {
                    fail(format!(
                        "V1_TOKEN_EVIDENCE_NOT_REUSABLE: invalid V1 ID for {hash}"
                    ))
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let sequence_hashes = row["message_sequence_sha256_values"]
            .as_array()
            .ok_or_else(|| {
                fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing message hashes for {hash}"
                ))
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_owned).ok_or_else(|| {
                    fail(format!(
                        "V1_TOKEN_EVIDENCE_NOT_REUSABLE: invalid message hash for {hash}"
                    ))
                })
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        if row["measurement_label"] != "V1_MEASUREMENT_REUSED_IN_V2"
            || row["v1_logical_request_ids"] != row["v2_logical_request_ids"]
            || v1_ids.is_empty()
            || sequence_hashes.is_empty()
            || measurements
                .insert(
                    hash.clone(),
                    (input_tokens, byte_length, v1_ids, sequence_hashes),
                )
                .is_some()
        {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: malformed or duplicate V2 inheritance entry for {hash}"
            ))
            .into());
        }
    }
    Ok(measurements)
}

fn build_groups(
    v3_ledger: &Value,
    measurements: &Measurements,
    expected_retained_logical: usize,
) -> Result<GroupResult, Box<dyn Error>> {
    let rows = v3_ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V3 ledger has no request array"))?;
    let mut groups: BTreeMap<String, (String, usize, BTreeSet<String>, Vec<String>)> =
        BTreeMap::new();
    let mut inherited_logical = 0usize;
    for row in rows {
        let id = required_str(row, "logical_request_id", "V3 request")?.to_owned();
        let case_id = required_str(row, "case_id", "V3 request")?;
        let body = required_str(row, "future_token_counter_body", "V3 request")?.to_owned();
        let hash = required_str(row, "request_body_sha256", "V3 request")?.to_owned();
        let body_length = row["request_body_utf8_byte_length"]
            .as_u64()
            .ok_or_else(|| {
                fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing V3 body length for {id}"
                ))
            })? as usize;
        let message_hash = required_str(row, "messages_sha256", "V3 request")?.to_owned();
        let measured = measurements.get(&hash);
        if let Some((tokens, measured_length, _, measured_messages)) = measured {
            if body_length != *measured_length
                || digest(body.as_bytes()) != hash
                || !measured_messages.contains(&message_hash)
                || row["evidence_classification"] != "INHERITED_V1_TOKEN_COUNT"
                || row["inherited_v1_input_tokens"].as_u64() != Some(*tokens)
            {
                return Err(fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: V3 evidence classification or exact V1 identity differs for {id}"
                ))
                .into());
            }
            inherited_logical += 1;
        } else if row["evidence_classification"] != "NEW_TOKEN_COUNT_REQUIRED"
            || !row["inherited_v1_input_tokens"].is_null()
        {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: unmeasured V3 body {id} has an inherited classification"
            ))
            .into());
        }
        match groups.get_mut(&hash) {
            Some((existing_body, existing_length, message_hashes, ids)) => {
                if existing_body.as_bytes() != body.as_bytes() || *existing_length != body_length {
                    return Err(fail(format!(
                        "V1_TOKEN_EVIDENCE_NOT_REUSABLE: duplicate body hash has different bytes for {id}"
                    ))
                    .into());
                }
                message_hashes.insert(message_hash);
                ids.push(id);
            }
            None => {
                groups.insert(
                    hash,
                    (body, body_length, BTreeSet::from([message_hash]), vec![id]),
                );
            }
        }
        if RETAINED_CASES.contains(&case_id) && measured.is_none() {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained V3 request has no V1 measurement: {case_id}"
            ))
            .into());
        }
    }

    let mut inherited = Vec::new();
    let mut new_hashes = Vec::new();
    let mut retained_hashes = BTreeSet::new();
    let mut retained_logical = 0usize;
    for (hash, (_, body_length, message_hashes, ids)) in groups {
        if let Some((input_tokens, measured_length, v1_ids, measured_messages)) =
            measurements.get(&hash)
        {
            if *measured_length != body_length || *measured_messages != message_hashes {
                return Err(fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: exact V1 message/body identity differs for {hash}"
                ))
                .into());
            }
            for id in &ids {
                if RETAINED_CASES
                    .iter()
                    .any(|case_id| id.starts_with(&format!("{case_id}/")))
                {
                    retained_logical += 1;
                    retained_hashes.insert(hash.clone());
                }
            }
            inherited.push(json!({
                "request_body_sha256": hash,
                "request_body_utf8_byte_length": body_length,
                "message_sequence_sha256_values": message_hashes,
                "input_tokens": input_tokens,
                "measurement_label": "V1_MEASUREMENT_REUSED_IN_V3",
                "v1_logical_request_ids": v1_ids,
                "v2_logical_request_ids": v1_ids,
                "v3_logical_request_ids": ids
            }));
        } else {
            new_hashes.push(json!({
                "request_body_sha256": hash,
                "request_body_utf8_byte_length": body_length,
                "message_sequence_sha256_values": message_hashes,
                "v3_logical_request_ids": ids
            }));
        }
    }
    let unique_hashes = inherited.len() + new_hashes.len();
    if retained_logical != expected_retained_logical
        || retained_hashes.len() != measurements.len()
        || inherited_logical < retained_logical
    {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: V3 retained join covered {retained_logical} logical requests and {} unique hashes; sealed proof covers {expected_retained_logical} and {}",
            retained_hashes.len(), measurements.len()
        ))
        .into());
    }
    Ok((
        inherited,
        new_hashes,
        inherited_logical,
        unique_hashes,
        retained_logical,
    ))
}

pub fn map_bytes(
    root: &Path,
    regenerated_v1_ledger: &[u8],
    v3_ledger_bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn Error>> {
    verify_sha256("V1 request ledger", regenerated_v1_ledger, V1_LEDGER_SHA256)?;
    let committed_v1 = fs::read(root.join(V1_LEDGER_PATH))?;
    if regenerated_v1_ledger != committed_v1 {
        return Err(fail(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: current retained fixtures do not reproduce the sealed V1 request ledger",
        )
        .into());
    }

    let regenerated_v2_successor = v2_workload::successor_bytes(root)?;
    verify_sha256(
        "V2 materialization successor",
        &regenerated_v2_successor,
        V2_SUCCESSOR_SHA256,
    )?;
    let committed_v2_successor = fs::read(root.join(V2_SUCCESSOR_PATH))?;
    if regenerated_v2_successor != committed_v2_successor {
        return Err(fail(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: accepted V2 successor no longer regenerates exactly",
        )
        .into());
    }
    let v2_ledger_bytes = fs::read(root.join(V2_LEDGER_PATH))?;
    verify_sha256("V2 request ledger", &v2_ledger_bytes, V2_LEDGER_SHA256)?;
    let v2_map_bytes = fs::read(root.join(V2_MAP_PATH))?;
    verify_sha256(
        "V2 token evidence inheritance map",
        &v2_map_bytes,
        V2_MAP_SHA256,
    )?;

    let v3_ledger_path = root.join(V3_LEDGER_PATH);
    let committed_v3_ledger = fs::read(&v3_ledger_path)?;
    if v3_ledger_bytes != committed_v3_ledger {
        return Err(fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: supplied V3 ledger differs from the tracked candidate ledger").into());
    }

    let v1_value = parse_json(regenerated_v1_ledger, "V1 request ledger")?;
    let v2_value = parse_json(&v2_ledger_bytes, "V2 request ledger")?;
    let v3_value = parse_json(v3_ledger_bytes, "V3 request ledger")?;
    verify_order(&v2_value, &V2_COHORT_ORDER, "V2 request ledger")?;
    verify_order(&v3_value, &V3_COHORT_ORDER, "V3 request ledger")?;
    verify_ledger_bodies(&v2_value, "V2 request ledger")?;
    verify_ledger_bodies(&v3_value, "V3 request ledger")?;

    let v2_map = parse_json(&v2_map_bytes, "V2 inheritance map")?;
    let measurements = v1_measurements(&v2_map)?;
    let expected_retained_logical = v2_map["scope"]["logical_requests_covered"]
        .as_u64()
        .ok_or_else(|| {
            fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: sealed V1 map omits retained logical coverage")
        })? as usize;
    let expected_retained_hashes = v2_map["scope"]["unique_exact_request_hashes_covered"]
        .as_u64()
        .ok_or_else(|| {
            fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: sealed V1 map omits retained unique coverage")
        })? as usize;
    if measurements.len() != expected_retained_hashes {
        return Err(fail(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: V2 map count roster differs from its sealed coverage summary",
        )
        .into());
    }
    compare_retained_rows(
        &v1_value,
        &v2_value,
        "V1 request ledger",
        "V2 request ledger",
        expected_retained_logical,
    )?;
    compare_retained_rows(
        &v1_value,
        &v3_value,
        "V1 request ledger",
        "V3 request ledger",
        expected_retained_logical,
    )?;
    compare_retained_rows(
        &v2_value,
        &v3_value,
        "V2 request ledger",
        "V3 request ledger",
        expected_retained_logical,
    )?;

    let retained_rows = row_index(&v3_value, "V3 request ledger")?;
    let retained_hashes = retained_rows
        .values()
        .filter(|row| {
            row["case_id"]
                .as_str()
                .is_some_and(|id| RETAINED_CASES.contains(&id))
        })
        .map(|row| {
            required_str(row, "request_body_sha256", "V3 retained request").map(str::to_owned)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if retained_hashes.len() != expected_retained_hashes {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained V3 body hashes ({}) differ from the sealed V1 count roster ({})",
            retained_hashes.len(), measurements.len()
        ))
        .into());
    }
    let (inherited, new_hashes, inherited_logical, unique_hash_count, retained_logical) =
        build_groups(&v3_value, &measurements, expected_retained_logical)?;
    let raw_evidence = v2_map["v1_bindings"].clone();
    if raw_evidence.is_null() {
        return Err(
            fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V2 inheritance map omits V1 seals").into(),
        );
    }
    let map = json!({
        "schema_id": "prefixity.phase1c.claim2-v3-token-evidence-inheritance-map",
        "schema_version": 3,
        "status": "RETAINED_V1_REQUEST_IDENTITIES_VERIFIED_FOR_V3",
        "scope": {
            "cohort_order": V3_COHORT_ORDER,
            "retained_cases_covered": RETAINED_CASES,
            "new_cases_not_yet_materialized": [],
            "complete_v3_request_ledger_present": true,
            "v3_request_ledger_path": V3_LEDGER_PATH,
            "v3_request_ledger_sha256": digest(v3_ledger_bytes),
            "logical_requests_covered": v3_value["requests"].as_array().map(Vec::len),
            "retained_logical_requests_verified": retained_logical,
            "retained_unique_request_hashes_verified": expected_retained_hashes,
            "inherited_v1_logical_requests": inherited_logical,
            "inherited_v1_unique_request_hashes": inherited.len(),
            "new_logical_requests": v3_value["requests"].as_array().map(Vec::len).unwrap_or_default() - inherited_logical,
            "new_unique_request_hashes_requiring_tokenization": new_hashes.len(),
            "unique_request_hashes": unique_hash_count,
            "tokenization_performed_by_this_generator": false,
            "inference_performed_by_this_generator": false
        },
        "historical_v1_bindings": raw_evidence,
        "historical_v2_bindings": {
            "request_ledger_path": V2_LEDGER_PATH,
            "request_ledger_sha256": digest(&v2_ledger_bytes),
            "inheritance_map_path": V2_MAP_PATH,
            "inheritance_map_sha256": digest(&v2_map_bytes),
            "materialization_successor_path": V2_SUCCESSOR_PATH,
            "materialization_successor_sha256": digest(&committed_v2_successor)
        },
        "reuse_conditions": [
            "exact request-body UTF-8 bytes and SHA-256 match a V1 measured body",
            "the exact rendered message sequence, tokenizer/model, llama build, reasoning setting, template behavior, endpoint, and relevant runtime conditions match the sealed V1 identity",
            "all bound V1 identity, raw-evidence, interpreted-result, and V2 inheritance seals verify"
        ],
        "inherited_unique_request_hashes": inherited,
        "new_unique_request_hashes": new_hashes
    });
    let mut bytes = serde_json::to_vec_pretty(&map)?;
    bytes.push(b'\n');
    Ok(bytes)
}
