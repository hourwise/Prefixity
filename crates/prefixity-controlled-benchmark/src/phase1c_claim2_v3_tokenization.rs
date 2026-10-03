//! Frozen Claim-2 V3 plans. Validation is offline and precedes any transport.

use crate::phase1c_claim2_tokenization::{
    ContactPlan, ContactPlanEntry, EndpointAllowlist, FrozenCounts, FrozenRequest, SourceIdentity,
    StaticEndpoint, TokenizationError, ValidatedInputs,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const LEDGER: &str = "fixtures/claim2/workload-request-ledger-v3.json";
pub const MAP: &str = "fixtures/claim2/token-evidence-inheritance-map-v3.json";
pub const HYBRID: &str = "fixtures/claim2/tokenization-contact-plan-v3-hybrid.json";
pub const FULL_FRESH: &str = "fixtures/claim2/tokenization-contact-plan-v3-full-fresh.json";
pub const IDENTITY: &str = "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V3.json";
pub const MATERIALIZATION_COMMIT: &str = "64d732dad2448e768ce12bc13a15bf3081f63205";
const LEDGER_SHA: &str = "406586d71839d91bc565b9db5da69a3d4152193c4109f7876e6a1dc56d89dce2";
const MAP_SHA: &str = "6a53ea43c74d20d8c5bf99483ffdf79e6706656076014fd4eb7728198bcb7aa2";
const HYBRID_SHA: &str = "7897b44d1c6e3c43d387f760fe4994ff5ee94ca1d258e86a3bc2c91b6033ce7e";
const FULL_SHA: &str = "acb927d97df1a00f2e9a33d4fb7569ba1fc3eca89a0bd720042a28337b06fe5e";
const V1_LEDGER_SHA: &str = "739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45";
const V1_PLAN_SHA: &str = "6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6";
const V1_RAW_SHA: &str = "caf62ccaba1130a0e75d55316a1828cdcb17a8df4cc73f839f96f31a6110c264";
const V1_IDENTITY_SEAL: &str = "4b7265e8a509829b3109947302ec82c2efec4f05cedd3aeac926324fa2a11f16";
const V1_RESULT_SEAL: &str = "03263b532a13619f9e6e38a447bf984651a712944d7c9aa83df9a86764821040";
const MODEL_SHA: &str = "cd76ec205963b3b33350093e6904d9de16c4e666fd104e1f632d25c7f15f2a13";
const LLAMA_SHA: &str = "cbe0655558e73168b3bc73f61aa70ec224475152b44c022f6e837616704d0617";

type BodyGroups = BTreeMap<String, (Vec<u8>, Vec<String>)>;
type InheritedCounts = BTreeMap<String, (u64, Vec<String>)>;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fail(message: &str) -> TokenizationError {
    TokenizationError(message.into())
}

fn read_pinned(root: &Path, path: &str, sha: &str) -> Result<Vec<u8>, TokenizationError> {
    let bytes = fs::read(root.join(path)).map_err(|e| TokenizationError(format!("{path}: {e}")))?;
    if digest(&bytes) != sha {
        return Err(fail(&format!("source pin mismatch: {path}")));
    }
    Ok(bytes)
}

fn parse(bytes: &[u8]) -> Result<Value, TokenizationError> {
    serde_json::from_slice(bytes).map_err(|e| TokenizationError(e.to_string()))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, TokenizationError> {
    value[key]
        .as_str()
        .ok_or_else(|| fail(&format!("missing {key}")))
}

fn groups_from_ledger(ledger: &Value) -> Result<BodyGroups, TokenizationError> {
    let requests = ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("missing V3 requests"))?;
    if requests.len() != 54
        || ledger["offline_boundary"]["server_contacts"] != 0
        || ledger["offline_boundary"]["inference_requests"] != 0
    {
        return Err(fail("V3 ledger offline/count boundary changed"));
    }
    let mut groups = BTreeMap::<String, (Vec<u8>, Vec<String>)>::new();
    let mut ids = BTreeSet::new();
    for request in requests {
        let id = string(request, "logical_request_id")?.to_owned();
        let case = string(request, "case_id")?;
        let arm = string(request, "arm")?;
        let slot = request["request_slot"]
            .as_u64()
            .ok_or_else(|| fail("invalid slot"))?;
        if !["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"].contains(&case)
            || !["BASELINE", "NO_OP", "INTERVENTION"].contains(&arm)
            || !(1..=3).contains(&slot)
            || id != format!("{case}/{arm}/slot-{slot}")
            || !ids.insert(id.clone())
        {
            return Err(fail("V3 logical roster mismatch"));
        }
        let body = string(request, "future_token_counter_body")?
            .as_bytes()
            .to_vec();
        let hash = digest(&body);
        let body_json = parse(&body)?;
        let fields = body_json
            .as_object()
            .ok_or_else(|| fail("body is not an object"))?;
        let expected_fields = BTreeSet::from([
            "model",
            "messages",
            "max_tokens",
            "temperature",
            "top_p",
            "seed",
            "stream",
        ]);
        if fields.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_fields
            || body_json["messages"] != request["messages"]
            || body_json["max_tokens"] != 1024
            || body_json["temperature"] != 0
            || body_json["top_p"] != 1
            || body_json["seed"] != 1
            || body_json["stream"] != false
            || string(request, "request_body_sha256")? != hash
            || request["request_body_utf8_byte_length"] != body.len()
            || request["tokenization_status"] != "NOT_PERFORMED"
            || request["live_dispatchable"] != false
        {
            return Err(fail("V3 body or request semantics mismatch"));
        }
        let expected_class = if ["CP09", "CP10"].contains(&case) {
            "NEW_TOKEN_COUNT_REQUIRED"
        } else {
            "INHERITED_V1_TOKEN_COUNT"
        };
        if request["evidence_classification"] != expected_class {
            return Err(fail("V3 inheritance classification changed"));
        }
        let group = groups
            .entry(hash)
            .or_insert_with(|| (body.clone(), Vec::new()));
        if group.0 != body {
            return Err(fail("request hash collision"));
        }
        group.1.push(id);
    }
    for case in ["CP02", "CP03", "CP09", "CP10", "CP05", "CP06"] {
        for slot in 1..=3 {
            for arm in ["BASELINE", "NO_OP", "INTERVENTION"] {
                if !ids.contains(&format!("{case}/{arm}/slot-{slot}")) {
                    return Err(fail("incomplete V3 roster"));
                }
            }
        }
    }
    if groups.len() != 22 {
        return Err(fail("V3 unique request count changed"));
    }
    for group in groups.values_mut() {
        group.1.sort();
    }
    Ok(groups)
}

fn inherited_map(map: &Value) -> Result<InheritedCounts, TokenizationError> {
    let entries = map["inherited_unique_request_hashes"]
        .as_array()
        .ok_or_else(|| fail("missing inheritance map"))?;
    if entries.len() != 14 {
        return Err(fail("inheritance map must contain 14 hashes"));
    }
    let mut result = BTreeMap::new();
    for entry in entries {
        let hash = string(entry, "request_body_sha256")?.to_owned();
        let count = entry["input_tokens"]
            .as_u64()
            .ok_or_else(|| fail("missing V1 count"))?;
        let mut ids = entry["v3_logical_request_ids"]
            .as_array()
            .ok_or_else(|| fail("missing inherited IDs"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail("invalid inherited ID"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        ids.sort();
        if result.insert(hash, (count, ids)).is_some() {
            return Err(fail("duplicate inherited hash"));
        }
    }
    Ok(result)
}

fn validate_plan(
    root: &Path,
    mode: &str,
    groups: &BodyGroups,
    inherited: &InheritedCounts,
) -> Result<ValidatedInputs, TokenizationError> {
    let (path, sha, expected_count) = match mode {
        "HYBRID_8_NEW" => (HYBRID, HYBRID_SHA, 8),
        "FULL_FRESH_22" => (FULL_FRESH, FULL_SHA, 22),
        _ => return Err(fail("unknown frozen V3 plan mode")),
    };
    let plan = parse(&read_pinned(root, path, sha)?)?;
    if plan["schema_id"] != "prefixity.phase1c.claim2-tokenization-v3-contact-plan"
        || plan["schema_version"] != 3
        || plan["mode"] != mode
        || plan["source_ledger"]["sha256"] != LEDGER_SHA
        || plan["source_inheritance_map"]["sha256"] != MAP_SHA
        || plan["logical_request_count"] != 54
        || plan["total_unique_body_count"] != 22
        || plan["fresh_unique_body_count"] != expected_count
        || plan["maximum_readiness_contacts"] != 1
        || plan["maximum_token_count_contacts"] != expected_count
        || plan["inference_allowance"] != 0
        || plan["endpoint_allowlist"]["readiness"] != json!({"method":"GET","path":"/health"})
        || plan["endpoint_allowlist"]["token_count"]
            != json!({"method":"POST","path":"/v1/chat/completions/input_tokens"})
    {
        return Err(fail("frozen V3 plan header or endpoint boundary changed"));
    }
    let entries = plan["entries"]
        .as_array()
        .ok_or_else(|| fail("missing frozen plan entries"))?;
    let selected = groups
        .iter()
        .filter(|(h, _)| mode == "FULL_FRESH_22" || !inherited.contains_key(*h))
        .collect::<Vec<_>>();
    if entries.len() != expected_count || selected.len() != expected_count {
        return Err(fail("V3 plan count mismatch"));
    }
    let mut requests = Vec::new();
    let mut contact_entries = Vec::new();
    for (entry, (hash, (body, ids))) in entries.iter().zip(selected) {
        let case = ids[0].split('/').next().unwrap_or_default();
        let slot = ids[0]
            .rsplit('-')
            .next()
            .unwrap_or_default()
            .parse::<u64>()
            .map_err(|_| fail("invalid slot"))?;
        let arms = ids
            .iter()
            .filter_map(|id| id.split('/').nth(1))
            .collect::<BTreeSet<_>>();
        let equality = if arms.len() == 3 {
            "ALL_ARMS"
        } else if arms == BTreeSet::from(["BASELINE", "NO_OP"]) {
            "BASELINE_EQUALS_NO_OP"
        } else {
            "INTERVENTION_ONLY"
        };
        let entry_ids = entry["logical_request_ids"]
            .as_array()
            .ok_or_else(|| fail("entry IDs missing"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| fail("entry ID invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if string(entry, "request_body_sha256")? != hash
            || entry["request_body_utf8_byte_length"] != body.len()
            || entry_ids != *ids
            || string(entry, "representative_logical_request_id")? != ids[0]
            || string(entry, "case_id")? != case
            || entry["request_slot"] != slot
            || string(entry, "treatment_equality_class")? != equality
            || string(entry, "exact_request_body")?.as_bytes() != body
        {
            return Err(fail("frozen V3 entry differs from exact ledger body"));
        }
        contact_entries.push(ContactPlanEntry {
            request_body_sha256: hash.clone(),
            request_body_utf8_byte_length: body.len(),
            representative_logical_request_id: ids[0].clone(),
            logical_request_ids: ids.clone(),
        });
        requests.push(FrozenRequest {
            request_body_sha256: hash.clone(),
            request_body_utf8_byte_length: body.len(),
            representative_logical_request_id: ids[0].clone(),
            logical_request_ids: ids.clone(),
            exact_body: body.clone(),
        });
    }
    let contact_plan = ContactPlan {
        schema_id: "prefixity.phase1c.claim2-tokenization-v3-contact-plan".into(),
        schema_version: 3,
        status: "FROZEN_OFFLINE_NO_CONTACTS".into(),
        source_ledger: SourceIdentity {
            path: LEDGER.into(),
            sha256: LEDGER_SHA.into(),
        },
        request_counts: FrozenCounts {
            logical_requests: 54,
            unique_request_bodies: 22,
            duplicate_logical_requests_avoided: 32,
            maximum_readiness_contacts: 1,
            maximum_token_count_contacts: expected_count,
            inference_allowance: 0,
        },
        ordering: "request_body_sha256 ascending, lowercase ASCII hex".into(),
        endpoint_allowlist: EndpointAllowlist {
            readiness: StaticEndpoint {
                method: "GET".into(),
                path: "/health".into(),
            },
            token_count: StaticEndpoint {
                method: "POST".into(),
                path: "/v1/chat/completions/input_tokens".into(),
            },
        },
        entries: contact_entries,
    };
    Ok(ValidatedInputs {
        ledger_sha256: LEDGER_SHA.into(),
        plan_sha256: sha.into(),
        plan: contact_plan,
        requests,
    })
}

fn verify_v1_seal(root: &Path, path: &str, seal: &str) -> Result<Value, TokenizationError> {
    let bytes = fs::read(root.join(path)).map_err(|e| TokenizationError(format!("{path}: {e}")))?;
    let (physical_crlf, physical_lf, sidecar_crlf, sidecar_lf) = if seal == V1_IDENTITY_SEAL {
        (
            "4696629b5e0f2ed87f80bd3c193bc457d5e374f570df258b35892e54372be902",
            "437d803feebe6b10db4b5615c6a7ff731e8ecb4ff66d8e0fc694e42257989d84",
            "a0b247048f4d58f7dd5c1af8cd9a1a472e875156444273e2ea1724b3bb609102",
            "e4384ba69b87d14e39af34b803346ac7f03309542a7953d52aa56d5b50a7109d",
        )
    } else {
        (
            "727b577ce044d11c9b69cca33f7e3d3b23eab9785875d7445dda028b9a2b1377",
            "13a6cb66266ee64165650ca501b255b5d2643aee9b09c7166dd9e8031a8497be",
            "f92b8620cfb61612f281f98940fe7c15aa11444fb079edcec571d25a8573d881",
            "e0860c5157bf2962e5520233ae53d6374625dbe7e368405fcb2bc4a3b7099f8d",
        )
    };
    if digest(&bytes) != physical_crlf && digest(&bytes) != physical_lf {
        return Err(fail(
            "V1 Git text physical form differs from both pinned forms",
        ));
    }
    let value = parse(&bytes)?;
    let canonical = serde_json::to_vec(&value).map_err(|e| TokenizationError(e.to_string()))?;
    if digest(&canonical) != seal {
        return Err(fail("V1 canonical evidence seal failed"));
    }
    let sidecar_bytes = fs::read(root.join(path.replace(".json", ".sha256")))
        .map_err(|e| TokenizationError(e.to_string()))?;
    if digest(&sidecar_bytes) != sidecar_crlf && digest(&sidecar_bytes) != sidecar_lf {
        return Err(fail(
            "V1 seal sidecar physical form differs from both pinned forms",
        ));
    }
    let sidecar = String::from_utf8(sidecar_bytes).map_err(|e| TokenizationError(e.to_string()))?;
    if sidecar.split_whitespace().next() != Some(seal) {
        return Err(fail("V1 evidence sidecar failed"));
    }
    Ok(value)
}

pub fn verify_inheritance(
    root: &Path,
    ledger: &Value,
    map: &Value,
    inherited: &InheritedCounts,
) -> Result<(), TokenizationError> {
    let bindings = &map["historical_v1_bindings"];
    if bindings["identity_canonical_sha256"] != V1_IDENTITY_SEAL
        || bindings["interpreted_result_canonical_sha256"] != V1_RESULT_SEAL
        || bindings["raw_evidence_sha256"] != V1_RAW_SHA
        || bindings["request_ledger_sha256"] != V1_LEDGER_SHA
        || bindings["contact_plan_sha256"] != V1_PLAN_SHA
        || bindings["instrument_identity"]["accepted_gguf"]["sha256"] != MODEL_SHA
        || bindings["instrument_identity"]["accepted_llama_executable"]["sha256"] != LLAMA_SHA
        || bindings["instrument_identity"]["accepted_llama_executable"]["build"]
            != "b10217-ddd4ec142"
        || bindings["instrument_identity"]["runtime_contract"]
            != json!({
                "chat_template_kwargs": "ABSENT", "context_tokens": 8192,
                "host": "127.0.0.1", "max_tokens": 1024,
                "offline": true, "port": 8080, "reasoning": "off",
                "seed": 1, "slots": 1, "stream": false,
                "temperature": 0, "text_only": true, "top_p": 1
            })
        || bindings["instrument_identity"]["endpoint_allowlist"]["token_count"]["path"]
            != "/v1/chat/completions/input_tokens"
    {
        return Err(fail("V1 inheritance instrument binding changed"));
    }
    read_pinned(
        root,
        "fixtures/claim2/tokenization-request-ledger-v1.json",
        V1_LEDGER_SHA,
    )?;
    read_pinned(
        root,
        "fixtures/claim2/tokenization-contact-plan-v1.json",
        V1_PLAN_SHA,
    )?;
    read_pinned(
        root,
        "claim2-tokenization-pass-evidence-v1.json",
        V1_RAW_SHA,
    )?;
    let identity = verify_v1_seal(
        root,
        "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_IDENTITY_V1.json",
        V1_IDENTITY_SEAL,
    )?;
    if identity["experiment_id"] != bindings["tokenization_identity"] {
        return Err(fail("V1 experiment identity changed"));
    }
    let result = verify_v1_seal(
        root,
        "docs/phase-1/PHASE_1C_CLAIM_2_TOKENIZATION_RESULT_V1.json",
        V1_RESULT_SEAL,
    )?;
    if result["terminal_classification"] != "WORKLOAD_TOKEN_ADMISSION_FAILED" {
        return Err(fail("V1 result classification changed"));
    }
    let unique = result["authoritative_unique_counts"]
        .as_array()
        .ok_or_else(|| fail("V1 unique counts missing"))?;
    let counts = unique
        .iter()
        .map(|item| {
            Ok((
                string(item, "request_body_sha256")?.to_owned(),
                item["input_tokens"]
                    .as_u64()
                    .ok_or_else(|| fail("V1 token count missing"))?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>, TokenizationError>>()?;
    if counts.len() != 22 {
        return Err(fail("V1 measured hash roster changed"));
    }
    let v1_ledger = parse(&read_pinned(
        root,
        "fixtures/claim2/tokenization-request-ledger-v1.json",
        V1_LEDGER_SHA,
    )?)?;
    let v1_by_id = v1_ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("V1 request ledger missing"))?
        .iter()
        .map(|r| Ok((string(r, "logical_request_id")?.to_owned(), r)))
        .collect::<Result<BTreeMap<_, _>, TokenizationError>>()?;
    let mut covered = 0;
    for request in ledger["requests"]
        .as_array()
        .ok_or_else(|| fail("V3 request ledger missing"))?
    {
        if request["evidence_classification"] == "INHERITED_V1_TOKEN_COUNT" {
            let id = string(request, "logical_request_id")?;
            let prior = v1_by_id
                .get(id)
                .ok_or_else(|| fail("V1 retained logical ID missing"))?;
            let hash = string(request, "request_body_sha256")?;
            let (count, ids) = inherited
                .get(hash)
                .ok_or_else(|| fail("V1 inherited hash missing"))?;
            if prior["future_token_counter_body"] != request["future_token_counter_body"]
                || prior["request_body_sha256"] != hash
                || !ids.iter().any(|candidate| candidate == id)
                || counts.get(hash) != Some(count)
                || request["inherited_v1_input_tokens"] != *count
            {
                return Err(fail("V1 exact-byte/count inheritance mismatch"));
            }
            covered += 1;
        }
    }
    if covered != 36 || inherited.len() != 14 {
        return Err(fail("V1 inheritance coverage changed"));
    }
    Ok(())
}

pub struct OfflinePreflight {
    pub hybrid: ValidatedInputs,
    pub full_fresh: ValidatedInputs,
    pub inheritance_eligible: bool,
    pub inheritance_reason: Option<String>,
}

pub fn preflight(root: &Path) -> Result<OfflinePreflight, TokenizationError> {
    for (path, sha) in [
        (
            "fixtures/claim2/workload-contract-v3.md",
            "28d48448719433a9baa28c0b668cafade63c7858655be00a1fd9effbb638ac5e",
        ),
        (
            "fixtures/claim2/advancing-output-domain-v3.json",
            "d9f2de3056734026e364f0b2b89af8af8bf3cfbba78ded8e06533d9fc20c4186",
        ),
        (
            "fixtures/claim2/materialization-report-v4.json",
            "dbf40e192c60b6f27b3e0112eb639be29a91e71b59d369b4f43ad682337c1e03",
        ),
    ] {
        read_pinned(root, path, sha)?;
    }
    let ledger = parse(&read_pinned(root, LEDGER, LEDGER_SHA)?)?;
    let map = parse(&read_pinned(root, MAP, MAP_SHA)?)?;
    let groups = groups_from_ledger(&ledger)?;
    let inherited = inherited_map(&map)?;
    for (hash, (_, ids)) in &inherited {
        if groups.get(hash).map(|(_, actual)| actual) != Some(ids) {
            return Err(fail("V3 inherited mapping differs from ledger"));
        }
    }
    let hybrid = validate_plan(root, "HYBRID_8_NEW", &groups, &inherited)?;
    let full_fresh = validate_plan(root, "FULL_FRESH_22", &groups, &inherited)?;
    let inheritance = verify_inheritance(root, &ledger, &map, &inherited);
    Ok(OfflinePreflight {
        hybrid,
        full_fresh,
        inheritance_eligible: inheritance.is_ok(),
        inheritance_reason: inheritance.err().map(|e| e.to_string()),
    })
}

pub fn repository_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn endpoint_path_is_allowed(path: &str) -> bool {
    ["/health", "/v1/chat/completions/input_tokens"].contains(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_preflight_reproduces_both_modes_and_inheritance() {
        let result = preflight(&repository_root()).unwrap();
        assert!(
            result.inheritance_eligible,
            "{:?}",
            result.inheritance_reason
        );
        assert_eq!(result.hybrid.requests.len(), 8);
        assert_eq!(result.full_fresh.requests.len(), 22);
        assert_eq!(result.hybrid.plan.request_counts.inference_allowance, 0);
        assert_eq!(
            result
                .full_fresh
                .plan
                .request_counts
                .maximum_readiness_contacts,
            1
        );
    }

    #[test]
    fn body_tamper_and_plan_substitution_are_rejected_before_contact() {
        let root = repository_root();
        let mut ledger = parse(&read_pinned(&root, LEDGER, LEDGER_SHA).unwrap()).unwrap();
        ledger["requests"][0]["future_token_counter_body"] = json!("{}");
        assert!(groups_from_ledger(&ledger).is_err());

        let original = parse(&read_pinned(&root, LEDGER, LEDGER_SHA).unwrap()).unwrap();
        let groups = groups_from_ledger(&original).unwrap();
        let map = parse(&read_pinned(&root, MAP, MAP_SHA).unwrap()).unwrap();
        let inherited = inherited_map(&map).unwrap();
        let temp = std::env::temp_dir().join(format!(
            "prefixity-v3-plan-substitution-{}",
            std::process::id()
        ));
        let fixtures = temp.join("fixtures/claim2");
        fs::create_dir_all(&fixtures).unwrap();
        fs::write(
            fixtures.join("tokenization-contact-plan-v3-hybrid.json"),
            fs::read(root.join(FULL_FRESH)).unwrap(),
        )
        .unwrap();
        assert!(validate_plan(&temp, "HYBRID_8_NEW", &groups, &inherited).is_err());
        fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn endpoint_allowlist_rejects_generation_and_arbitrary_paths() {
        assert!(endpoint_path_is_allowed("/health"));
        assert!(endpoint_path_is_allowed(
            "/v1/chat/completions/input_tokens"
        ));
        for path in [
            "/v1/chat/completions",
            "/completion",
            "/v1/completions",
            "http://example.invalid/",
        ] {
            assert!(!endpoint_path_is_allowed(path));
        }
    }
}
