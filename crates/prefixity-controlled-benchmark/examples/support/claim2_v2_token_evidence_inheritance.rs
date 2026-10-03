use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

pub const OUTPUT_PATH: &str = "fixtures/claim2/token-evidence-inheritance-map-v2.json";

const RETAINED_CASES: [&str; 4] = ["CP02", "CP03", "CP05", "CP06"];
const V1_LEDGER_PATH: &str = "fixtures/claim2/tokenization-request-ledger-v1.json";
const V1_LEDGER_SHA256: &str = "739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45";
const V1_PLAN_PATH: &str = "fixtures/claim2/tokenization-contact-plan-v1.json";
const V1_PLAN_SHA256: &str = "6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6";
const V1_IDENTITY_PATH: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V1.json";
const V1_IDENTITY_SHA256: &str = "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16";
const V1_IDENTITY_FILE_SHA256: &str =
    "4696629b5e0f2ed87f80bd3c193bc457d5e374f570df258b35892e54372be902";
const V1_IDENTITY_SEAL_PATH: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V1.sha256";
const V1_IDENTITY_SEAL_FILE_SHA256: &str =
    "a0b247048f4d58f7dd5c1af8cd9a1a472e875156444273e2ea1724b3bb609102";
const V1_RAW_EVIDENCE_PATH: &str = "claim2-tokenization-pass-evidence-v1.json";
const V1_RAW_EVIDENCE_SHA256: &str =
    "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264";
const V1_RESULT_PATH: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json";
const V1_RESULT_SEAL: &str = "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040";
const V1_RESULT_FILE_SHA256: &str =
    "727b577ce044d11c9b69cca33f7e3d3b23eab9785875d7445dda028b9a2b1377";
const V1_RESULT_SEAL_PATH: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.sha256";
const V1_RESULT_SEAL_FILE_SHA256: &str =
    "f92b8620cfb61612f281f98940fe7c15aa11444fb079edcec571d25a8573d881";
const V1_IDENTITY_ID: &str =
    "claim2-tokenization-v1-a5a6b896555db8296318f38010b7120dad8ad191e1329f030e0f738f31b90b91";

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let sorted: BTreeMap<String, Value> = object
                .into_iter()
                .map(|(key, value)| (key, canonical_value(value)))
                .collect();
            let mut canonical = Map::new();
            for (key, value) in sorted {
                canonical.insert(key, value);
            }
            Value::Object(canonical)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonical_value).collect()),
        scalar => scalar,
    }
}

fn canonical_json(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&canonical_value(value.clone()))
}

fn fail(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

fn verify_pinned_bytes(
    name: &str,
    bytes: &[u8],
    expected_sha256: &str,
) -> Result<(), Box<dyn Error>> {
    let actual = digest(bytes);
    if actual != expected_sha256 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} SHA-256 changed (expected {expected_sha256}, found {actual})"
        ))
        .into());
    }
    Ok(())
}

fn read_pinned(
    root: &Path,
    name: &str,
    path: &str,
    expected_sha256: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = fs::read(root.join(path))?;
    verify_pinned_bytes(name, &bytes, expected_sha256)?;
    Ok(bytes)
}

fn read_verified_sidecar(
    root: &Path,
    name: &str,
    path: &str,
    expected_file_sha256: &str,
    expected_content_sha256: &str,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let sidecar_bytes = fs::read(root.join(path))?;
    verify_pinned_bytes(name, &sidecar_bytes, expected_file_sha256)?;
    let sidecar = std::str::from_utf8(&sidecar_bytes)?;
    let found = sidecar
        .split_whitespace()
        .next()
        .ok_or_else(|| fail(format!("V1_TOKEN_EVIDENCE_NOT_REUSABLE: empty {name} seal")))?;
    if found != expected_content_sha256 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: {name} seal sidecar changed"
        ))
        .into());
    }
    Ok(sidecar_bytes)
}

fn json(bytes: &[u8], what: &str) -> Result<Value, Box<dyn Error>> {
    serde_json::from_slice(bytes)
        .map_err(|error| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: invalid {what}: {error}"
            ))
        })
        .map_err(Into::into)
}

fn as_str<'a>(value: &'a Value, key: &str, what: &str) -> Result<&'a str, Box<dyn Error>> {
    value[key]
        .as_str()
        .ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: missing {what}.{key}"
            ))
        })
        .map_err(Into::into)
}

pub fn map_bytes(root: &Path, regenerated_v1_ledger: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    verify_pinned_bytes("V1 request ledger", regenerated_v1_ledger, V1_LEDGER_SHA256)?;
    let committed_ledger = fs::read(root.join(V1_LEDGER_PATH))?;
    if regenerated_v1_ledger != committed_ledger {
        return Err(fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: current retained fixtures no longer reproduce the sealed V1 request ledger byte-for-byte").into());
    }

    let contact_plan = read_pinned(
        root,
        "V1 tokenization contact plan",
        V1_PLAN_PATH,
        V1_PLAN_SHA256,
    )?;
    let identity_bytes = read_pinned(
        root,
        "V1 tokenization identity file",
        V1_IDENTITY_PATH,
        V1_IDENTITY_FILE_SHA256,
    )?;
    let identity_value = json(&identity_bytes, "V1 tokenization identity")?;
    verify_pinned_bytes(
        "V1 tokenization identity canonical seal",
        &canonical_json(&identity_value)?,
        V1_IDENTITY_SHA256,
    )?;
    let identity_seal_sidecar = read_verified_sidecar(
        root,
        "V1 tokenization identity",
        V1_IDENTITY_SEAL_PATH,
        V1_IDENTITY_SEAL_FILE_SHA256,
        V1_IDENTITY_SHA256,
    )?;
    let raw_evidence = read_pinned(
        root,
        "V1 raw tokenization evidence",
        V1_RAW_EVIDENCE_PATH,
        V1_RAW_EVIDENCE_SHA256,
    )?;
    let result_bytes = read_pinned(
        root,
        "V1 interpreted tokenization result file",
        V1_RESULT_PATH,
        V1_RESULT_FILE_SHA256,
    )?;
    let result_value = json(&result_bytes, "interpreted V1 tokenization result")?;
    let result_canonical = canonical_json(&result_value)?;
    verify_pinned_bytes(
        "V1 interpreted-result seal",
        &result_canonical,
        V1_RESULT_SEAL,
    )?;
    let result_seal_sidecar = read_verified_sidecar(
        root,
        "V1 interpreted-result",
        V1_RESULT_SEAL_PATH,
        V1_RESULT_SEAL_FILE_SHA256,
        V1_RESULT_SEAL,
    )?;

    let ledger = json(regenerated_v1_ledger, "regenerated V1 request ledger")?;
    let identity = identity_value;
    if as_str(&identity, "experiment_id", "identity")? != V1_IDENTITY_ID
        || as_str(
            &identity["bindings"]["request_ledger"],
            "sha256",
            "identity.bindings.request_ledger",
        )? != V1_LEDGER_SHA256
        || as_str(&result_value, "terminal_classification", "result")?
            != "WORKLOAD_TOKEN_ADMISSION_FAILED"
        || as_str(
            &result_value["provenance"],
            "tokenization_experiment_id",
            "result.provenance",
        )? != V1_IDENTITY_ID
        || as_str(
            &result_value["provenance"],
            "tokenization_identity_canonical_sha256",
            "result.provenance",
        )? != V1_IDENTITY_SHA256
        || as_str(
            &result_value["provenance"],
            "raw_evidence_sha256",
            "result.provenance",
        )? != V1_RAW_EVIDENCE_SHA256
        || as_str(
            &result_value["provenance"],
            "request_ledger_sha256",
            "result.provenance",
        )? != V1_LEDGER_SHA256
        || as_str(
            &result_value["provenance"],
            "contact_plan_sha256",
            "result.provenance",
        )? != V1_PLAN_SHA256
    {
        return Err(fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V1 identity/result bindings do not match the accepted evidence").into());
    }

    let requests = ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V1 ledger has no request array"))?;
    let identity_requests = identity["bindings"]["logical_requests"]
        .as_array()
        .ok_or_else(|| {
            fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: V1 identity has no logical request bindings")
        })?;
    let identity_by_id = identity_requests
        .iter()
        .map(|request| {
            Ok((
                as_str(request, "logical_request_id", "identity request")?.to_owned(),
                request,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?;

    let mut selected = Vec::new();
    for request in requests {
        let case_id = as_str(request, "case_id", "V1 request")?;
        let request_id = as_str(request, "logical_request_id", "V1 request")?;
        let body = as_str(request, "future_token_counter_body", "V1 request")?;
        let body_hash = as_str(request, "request_body_sha256", "V1 request")?;
        if digest(body.as_bytes()) != body_hash
            || request["request_body_utf8_byte_length"].as_u64() != Some(body.len() as u64)
        {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: exact V1 request bytes failed validation for {request_id}"
            ))
            .into());
        }
        let identity_request = identity_by_id.get(request_id).ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: V1 identity omits {request_id}"
            ))
        })?;
        for field in [
            "case_id",
            "arm",
            "request_slot",
            "messages_sha256",
            "request_body_sha256",
            "request_body_utf8_byte_length",
        ] {
            if request[field] != identity_request[field] {
                return Err(fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: request identity mismatch for {request_id} at {field}"
                ))
                .into());
            }
        }
        if RETAINED_CASES.contains(&case_id) {
            selected.push(request);
        }
    }
    if selected.len() != 36 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: expected 36 retained logical requests, found {}",
            selected.len()
        ))
        .into());
    }

    let unique_counts = result_value["authoritative_unique_counts"]
        .as_array()
        .ok_or_else(|| fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: result has no unique counts"))?;
    let logical_counts = result_value["authoritative_logical_counts"]
        .as_array()
        .ok_or_else(|| fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: result has no logical counts"))?;
    let mut counts_by_hash = BTreeMap::new();
    for count in unique_counts {
        let hash = as_str(count, "request_body_sha256", "unique count")?.to_owned();
        if counts_by_hash.insert(hash, count).is_some() {
            return Err(
                fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: duplicate V1 unique count hash").into(),
            );
        }
    }
    let logical_by_id = logical_counts
        .iter()
        .map(|count| {
            Ok((
                as_str(count, "logical_request_id", "logical count")?.to_owned(),
                count,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?;

    let mut groups: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for request in selected {
        groups
            .entry(as_str(request, "request_body_sha256", "V1 request")?.to_owned())
            .or_default()
            .push(request);
    }
    if groups.len() != 14 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: expected 14 retained request hashes, found {}",
            groups.len()
        ))
        .into());
    }

    let mut inherited_request_hashes = Vec::with_capacity(14);
    let mut logical_identity_count = 0usize;
    for (hash, members) in groups {
        let count = counts_by_hash.get(&hash).ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: V1 count missing for {hash}"
            ))
        })?;
        let input_tokens = count["input_tokens"].as_u64().ok_or_else(|| {
            fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: unusable V1 count for {hash}"
            ))
        })?;
        let measured_ids = count["logical_request_ids"]
            .as_array()
            .ok_or_else(|| {
                fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: count has no mapping for {hash}"
                ))
            })?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail("invalid V1 logical request ID"))
            })
            .collect::<Result<BTreeSet<_>, _>>()?;
        let mut group_ids = BTreeSet::new();
        let mut body_length = None;
        let mut messages_hashes = BTreeSet::new();
        for request in &members {
            let id = as_str(request, "logical_request_id", "V1 request")?;
            group_ids.insert(id.to_owned());
            body_length = Some(request["request_body_utf8_byte_length"].as_u64().unwrap());
            messages_hashes.insert(as_str(request, "messages_sha256", "V1 request")?);
            let logical = logical_by_id.get(id).ok_or_else(|| {
                fail(format!("V1_TOKEN_EVIDENCE_NOT_REUSABLE: result omits {id}"))
            })?;
            if logical["request_body_sha256"] != hash || logical["input_tokens"] != input_tokens {
                return Err(fail(format!(
                    "V1_TOKEN_EVIDENCE_NOT_REUSABLE: result mapping/count differs for {id}"
                ))
                .into());
            }
        }
        if measured_ids != group_ids || messages_hashes.is_empty() {
            return Err(fail(format!(
                "V1_TOKEN_EVIDENCE_NOT_REUSABLE: sealed logical mapping differs for {hash}"
            ))
            .into());
        }
        logical_identity_count += group_ids.len();
        inherited_request_hashes.push(json!({
            "request_body_sha256": hash,
            "request_body_utf8_byte_length": body_length,
            "message_sequence_sha256_values": messages_hashes,
            "input_tokens": input_tokens,
            "measurement_label": "V1_MEASUREMENT_REUSED_IN_V2",
            "v1_logical_request_ids": group_ids,
            "v2_logical_request_ids": group_ids,
        }));
    }
    if logical_identity_count != 36 {
        return Err(fail(format!(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained logical identity coverage is {logical_identity_count}, expected 36"
        ))
        .into());
    }

    let v1_fixture_identities = ledger["fixture_identities"]["cases"]
        .as_array()
        .ok_or_else(|| fail("V1_TOKEN_EVIDENCE_NOT_REUSABLE: fixture identity list missing"))?;
    let retained_fixture_identities = v1_fixture_identities
        .iter()
        .filter(|identity| {
            identity["case_id"]
                .as_str()
                .is_some_and(|case_id| RETAINED_CASES.contains(&case_id))
        })
        .cloned()
        .collect::<Vec<_>>();
    if retained_fixture_identities.len() != RETAINED_CASES.len() {
        return Err(fail(
            "V1_TOKEN_EVIDENCE_NOT_REUSABLE: retained fixture identity coverage is incomplete",
        )
        .into());
    }

    let evidence = json(&raw_evidence, "V1 raw tokenization evidence")?;
    let map = json!({
        "schema_id": "prefixity.phase1c.claim2-v2-token-evidence-inheritance-map",
        "schema_version": 2,
        "status": "RETAINED_V1_REQUEST_IDENTITIES_VERIFIED_FOR_V2",
        "scope": {
            "cohort_order": ["CP02", "CP03", "CP07", "CP08", "CP05", "CP06"],
            "retained_cases_covered": RETAINED_CASES,
            "new_cases_not_yet_materialized": ["CP07", "CP08"],
            "logical_requests_covered": logical_identity_count,
            "unique_exact_request_hashes_covered": inherited_request_hashes.len(),
            "complete_v2_request_ledger_present": false,
            "tokenization_performed_by_this_generator": false
        },
        "v1_bindings": {
            "tokenization_identity": V1_IDENTITY_ID,
            "identity_path": V1_IDENTITY_PATH,
            "identity_canonical_sha256": V1_IDENTITY_SHA256,
            "identity_seal_sidecar_sha256": digest(&identity_seal_sidecar),
            "raw_evidence_path": V1_RAW_EVIDENCE_PATH,
            "raw_evidence_sha256": digest(&raw_evidence),
            "interpreted_result_path": V1_RESULT_PATH,
            "interpreted_result_canonical_sha256": V1_RESULT_SEAL,
            "interpreted_result_seal_sidecar_sha256": digest(&result_seal_sidecar),
            "request_ledger_path": V1_LEDGER_PATH,
            "request_ledger_sha256": digest(regenerated_v1_ledger),
            "contact_plan_path": V1_PLAN_PATH,
            "contact_plan_sha256": digest(&contact_plan),
            "terminal_classification": result_value["terminal_classification"],
            "identity_file_sha256": digest(&identity_bytes),
            "token_result_file_sha256": digest(&result_bytes),
            "raw_evidence_json_schema_id": evidence["schema_id"],
            "instrument_identity": {
                "accepted_gguf": identity["bindings"]["accepted_gguf"],
                "accepted_llama_executable": identity["bindings"]["accepted_llama_executable"],
                "runtime_contract": identity["bindings"]["runtime_contract"],
                "endpoint_allowlist": identity["bindings"]["endpoint_allowlist"],
                "frozen_client_executable": identity["bindings"]["frozen_client_executable"]
            },
            "retained_fixture_identities": retained_fixture_identities
        },
        "reuse_conditions": [
            "exact request-body UTF-8 bytes and SHA-256 match a V1 measured body",
            "same GGUF/tokenizer, model, llama build, reasoning-off setting, template behavior, endpoint, and other relevant runtime conditions",
            "all bound V1 identity, evidence, and result seals verify"
        ],
        "inherited_unique_request_hashes": inherited_request_hashes
    });
    let mut bytes = serde_json::to_vec_pretty(&map)?;
    bytes.push(b'\n');
    Ok(bytes)
}
