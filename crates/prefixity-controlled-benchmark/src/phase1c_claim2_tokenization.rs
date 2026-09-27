//! Frozen, non-inference Claim-2 tokenization contact plan and client core.
//!
//! This module validates the exact offline request ledger and frozen contact
//! plan. Its transport interface exposes only readiness and input-token
//! counting. It has no completion-generation operation.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const LEDGER_RELATIVE_PATH: &str = "fixtures/claim2/tokenization-request-ledger-v1.json";
pub const PLAN_RELATIVE_PATH: &str = "fixtures/claim2/tokenization-contact-plan-v1.json";
pub const ACCEPTED_LEDGER_SHA256: &str =
    "739205fb56e4f40bd55245f37d0768b8ca73c891b2f8d28bdb8f284e9f811d45";
// Set to the independently generated contact-plan hash before the Phase-3
// implementation is returned for review.
pub const ACCEPTED_PLAN_SHA256: &str =
    "6b7634a3fc7a3d4b76289aea1772c1df686a6641309acc9e307e5f330dfefdf6";

pub const READINESS_PATH: &str = "/health";
pub const INPUT_TOKEN_PATH: &str = "/v1/chat/completions/input_tokens";
pub const ALLOWED_ENDPOINT_PATHS: [&str; 2] = [READINESS_PATH, INPUT_TOKEN_PATH];

const PLAN_SCHEMA_ID: &str = "prefixity.phase1c.claim2-tokenization-contact-plan";
const PLAN_SCHEMA_VERSION: u64 = 1;
const EXPECTED_LOGICAL_REQUESTS: usize = 54;
const EXPECTED_UNIQUE_BODIES: usize = 22;
const EXPECTED_DUPLICATES_AVOIDED: usize = 32;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPlan {
    pub schema_id: String,
    pub schema_version: u64,
    pub status: String,
    pub source_ledger: SourceIdentity,
    pub request_counts: FrozenCounts,
    pub ordering: String,
    pub endpoint_allowlist: EndpointAllowlist,
    pub entries: Vec<ContactPlanEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenCounts {
    pub logical_requests: usize,
    pub unique_request_bodies: usize,
    pub duplicate_logical_requests_avoided: usize,
    pub maximum_readiness_contacts: usize,
    pub maximum_token_count_contacts: usize,
    pub inference_allowance: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointAllowlist {
    pub readiness: StaticEndpoint,
    pub token_count: StaticEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaticEndpoint {
    pub method: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactPlanEntry {
    pub request_body_sha256: String,
    pub request_body_utf8_byte_length: usize,
    pub representative_logical_request_id: String,
    pub logical_request_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenRequest {
    pub request_body_sha256: String,
    pub request_body_utf8_byte_length: usize,
    pub representative_logical_request_id: String,
    pub logical_request_ids: Vec<String>,
    pub exact_body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInputs {
    pub ledger_sha256: String,
    pub plan_sha256: String,
    pub plan: ContactPlan,
    /// The frozen plan order is retained exactly; no contacts are discovered
    /// from the ledger after this point.
    pub requests: Vec<FrozenRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawHttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub truncated: bool,
}

pub trait TokenizationTransport {
    fn readiness(&mut self) -> Result<RawHttpResponse, String>;
    fn input_tokens(&mut self, exact_body: &[u8]) -> Result<RawHttpResponse, String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PassEvidence {
    pub schema_id: String,
    pub schema_version: u64,
    pub status: String,
    pub terminal_classification: Option<String>,
    pub source_identities: EvidenceSourceIdentities,
    pub planned_accounting: FrozenCounts,
    pub operator_runtime_confirmation: Value,
    pub contact_accounting: ContactAccounting,
    pub readiness: ReadinessEvidence,
    pub unique_request_results: Vec<TokenCountEvidence>,
    pub admission_outcomes: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSourceIdentities {
    pub ledger_sha256: String,
    pub contact_plan_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactAccounting {
    pub readiness_contacts: usize,
    pub unique_token_count_contacts: usize,
    pub inference_requests: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadinessEvidence {
    pub status: String,
    pub http_status: Option<u16>,
    pub response_body_sha256: Option<String>,
    pub response_body_utf8_byte_length: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenCountEvidence {
    pub request_body_sha256: String,
    pub representative_logical_request_id: String,
    pub logical_request_ids: Vec<String>,
    pub status: String,
    pub http_status: Option<u16>,
    pub input_tokens: Option<u64>,
    pub response_count_field: Option<String>,
    pub response_body_sha256: Option<String>,
    pub response_body_utf8_byte_length: Option<usize>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct TokenizationError(pub String);

impl fmt::Display for TokenizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for TokenizationError {}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str, TokenizationError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| TokenizationError(format!("missing string field {key}")))
}

fn required_usize(value: &Value, key: &str) -> Result<usize, TokenizationError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| TokenizationError(format!("missing integer field {key}")))
}

fn expected_identity_groups() -> BTreeSet<BTreeSet<String>> {
    let mut groups = BTreeSet::new();
    for case_id in ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"] {
        for slot in [1, 2] {
            groups.insert(
                ["BASELINE", "NO_OP", "INTERVENTION"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-{slot}"))
                    .collect(),
            );
        }
        if matches!(case_id, "CP01" | "CP02" | "CP03" | "CP04") {
            groups.insert(
                ["BASELINE", "NO_OP"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-3"))
                    .collect(),
            );
            groups.insert(BTreeSet::from([format!("{case_id}/INTERVENTION/slot-3")]));
        } else {
            groups.insert(
                ["BASELINE", "NO_OP", "INTERVENTION"]
                    .into_iter()
                    .map(|arm| format!("{case_id}/{arm}/slot-3"))
                    .collect(),
            );
        }
    }
    groups
}

fn validate_exact_body(
    request: &Value,
    expected_semantics: &Value,
) -> Result<Vec<u8>, TokenizationError> {
    let body = required_str(request, "future_token_counter_body")?
        .as_bytes()
        .to_vec();
    let hash = digest(&body);
    if required_str(request, "request_body_sha256")? != hash
        || required_usize(request, "request_body_utf8_byte_length")? != body.len()
    {
        return Err(TokenizationError(
            "request-body hash or byte length mismatch".into(),
        ));
    }
    let body_value: Value = serde_json::from_slice(&body)
        .map_err(|error| TokenizationError(format!("request body is not JSON: {error}")))?;
    let body_object = body_value
        .as_object()
        .ok_or_else(|| TokenizationError("request body is not an object".into()))?;
    let expected_fields = BTreeSet::from([
        "model",
        "messages",
        "max_tokens",
        "temperature",
        "top_p",
        "seed",
        "stream",
    ]);
    let actual_fields = body_object
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual_fields != expected_fields
        || body_value["model"] != expected_semantics["model"]
        || body_value["max_tokens"] != 1024
        || body_value["temperature"] != 0
        || body_value["top_p"] != 1
        || body_value["seed"] != 1
        || body_value["stream"] != false
        || body_value["messages"] != request["messages"]
        || request["chat_template_kwargs"] != Value::Null
        || request["tokenizable"] != true
        || request["live_dispatchable"] != false
        || request["renderer_dispatchable"] != false
        || request["tokenization_status"] != "NOT_PERFORMED"
        || request["token_count_status"] != "EXACT_TOKENIZATION_REQUIRED"
    {
        return Err(TokenizationError(
            "request semantics or offline dispatch boundary changed".into(),
        ));
    }
    Ok(body)
}

fn build_plan_and_requests(
    ledger_bytes: &[u8],
) -> Result<(ContactPlan, Vec<FrozenRequest>), TokenizationError> {
    let ledger_sha256 = digest(ledger_bytes);
    if ledger_sha256 != ACCEPTED_LEDGER_SHA256 {
        return Err(TokenizationError(format!(
            "accepted tokenization ledger hash mismatch: {ledger_sha256}"
        )));
    }
    let ledger: Value = serde_json::from_slice(ledger_bytes).map_err(|error| {
        TokenizationError(format!("tokenization ledger is invalid JSON: {error}"))
    })?;
    if required_str(&ledger, "schema_id")? != "prefixity.phase1c.claim2-tokenization-request-ledger"
        || required_usize(&ledger, "schema_version")? != 1
        || required_str(&ledger, "classification")?
            != "OFFLINE_NON_INFERENCE_TOKENIZATION_PREPARATION"
        || ledger["status"] != "REQUEST_IDENTITY_STRUCTURE_MATCHES_REVIEW_EXPECTATION"
        || ledger["offline_boundary"]["tokenization_performed"] != false
        || required_usize(&ledger["offline_boundary"], "server_contacts")? != 0
        || required_usize(&ledger["offline_boundary"], "inference_requests")? != 0
        || ledger["offline_boundary"]["no_live_dispatch_interface"] != true
    {
        return Err(TokenizationError(
            "ledger schema or offline status changed".into(),
        ));
    }

    let semantics = &ledger["request_semantics"];
    if required_str(semantics, "token_counter_endpoint")? != INPUT_TOKEN_PATH
        || required_usize(semantics, "max_tokens")? != 1024
        || required_usize(semantics, "temperature")? != 0
        || required_usize(semantics, "top_p")? != 1
        || required_usize(semantics, "seed")? != 1
        || semantics["stream"] != false
        || semantics["chat_template_kwargs"] != "ABSENT"
        || semantics["content_type"] != "TEXT_ONLY"
        || required_usize(semantics, "context_tokens")? != 8192
        || required_usize(semantics, "input_preflight_limit_tokens")? != 6000
    {
        return Err(TokenizationError(
            "accepted request semantics changed".into(),
        ));
    }

    let requests = ledger["requests"]
        .as_array()
        .ok_or_else(|| TokenizationError("ledger has no request array".into()))?;
    if requests.len() != EXPECTED_LOGICAL_REQUESTS
        || required_usize(&ledger["deduplication"], "logical_request_count")?
            != EXPECTED_LOGICAL_REQUESTS
    {
        return Err(TokenizationError(
            "ledger logical request count is not 54".into(),
        ));
    }

    let mut logical_ids = BTreeSet::new();
    let mut body_by_hash = BTreeMap::<String, Vec<u8>>::new();
    let mut ids_by_hash = BTreeMap::<String, Vec<String>>::new();
    let mut slot_hashes = BTreeMap::<u8, BTreeSet<String>>::new();
    for request in requests {
        let logical_id = required_str(request, "logical_request_id")?.to_owned();
        let case_id = required_str(request, "case_id")?;
        let arm = required_str(request, "arm")?;
        let slot = request["request_slot"]
            .as_u64()
            .and_then(|value| u8::try_from(value).ok())
            .ok_or_else(|| TokenizationError("request slot is not an integer".into()))?;
        if !matches!(case_id, "CP01" | "CP02" | "CP03" | "CP04" | "CP05" | "CP06")
            || !matches!(arm, "BASELINE" | "NO_OP" | "INTERVENTION")
            || !(1..=3).contains(&slot)
            || logical_id != format!("{case_id}/{arm}/slot-{slot}")
            || !logical_ids.insert(logical_id.clone())
        {
            return Err(TokenizationError(
                "logical request identity is invalid or repeated".into(),
            ));
        }
        let body = validate_exact_body(request, semantics)?;
        let hash = digest(&body);
        if let Some(existing) = body_by_hash.get(&hash) {
            if existing != &body {
                return Err(TokenizationError(
                    "SHA-256 collision across different request bodies".into(),
                ));
            }
        } else {
            body_by_hash.insert(hash.clone(), body);
        }
        ids_by_hash
            .entry(hash.clone())
            .or_default()
            .push(logical_id);
        slot_hashes.entry(slot).or_default().insert(hash);
    }
    if logical_ids.len() != EXPECTED_LOGICAL_REQUESTS
        || (1..=3).any(|slot| {
            (1..=3).any(|arm| {
                let arm_name = ["BASELINE", "NO_OP", "INTERVENTION"][arm - 1];
                (1..=6).any(|case_index| {
                    let case_id = ["CP01", "CP02", "CP03", "CP04", "CP05", "CP06"][case_index - 1];
                    !logical_ids.contains(&format!("{case_id}/{arm_name}/slot-{slot}"))
                })
            })
        })
    {
        return Err(TokenizationError(
            "ledger does not contain the complete 6x3x3 roster".into(),
        ));
    }

    let observed_groups = ids_by_hash
        .values()
        .map(|ids| ids.iter().cloned().collect::<BTreeSet<_>>())
        .collect::<BTreeSet<_>>();
    if body_by_hash.len() != EXPECTED_UNIQUE_BODIES
        || observed_groups != expected_identity_groups()
        || slot_hashes.get(&1).map(BTreeSet::len) != Some(6)
        || slot_hashes.get(&2).map(BTreeSet::len) != Some(6)
        || slot_hashes.get(&3).map(BTreeSet::len) != Some(10)
        || required_usize(&ledger["deduplication"], "unique_request_body_count")?
            != EXPECTED_UNIQUE_BODIES
        || required_usize(
            &ledger["deduplication"],
            "duplicate_logical_requests_avoided_if_contacted_later",
        )? != EXPECTED_DUPLICATES_AVOIDED
        || ledger["deduplication"]["equality_structure_matches_review_expectation"] != true
    {
        return Err(TokenizationError(
            "TOKENIZATION_REQUEST_IDENTITY_MISMATCH".into(),
        ));
    }

    let entries = body_by_hash
        .iter()
        .map(|(hash, body)| {
            let mut ids = ids_by_hash.get(hash).cloned().unwrap_or_default();
            ids.sort();
            let representative = ids
                .first()
                .cloned()
                .ok_or_else(|| TokenizationError("empty deduplication group".into()))?;
            Ok((
                ContactPlanEntry {
                    request_body_sha256: hash.clone(),
                    request_body_utf8_byte_length: body.len(),
                    representative_logical_request_id: representative,
                    logical_request_ids: ids,
                },
                body.clone(),
            ))
        })
        .collect::<Result<Vec<_>, TokenizationError>>()?;
    let plan = ContactPlan {
        schema_id: PLAN_SCHEMA_ID.into(),
        schema_version: PLAN_SCHEMA_VERSION,
        status: "FROZEN_OFFLINE_CONTACT_PLAN".into(),
        source_ledger: SourceIdentity {
            path: LEDGER_RELATIVE_PATH.into(),
            sha256: ledger_sha256,
        },
        request_counts: FrozenCounts {
            logical_requests: EXPECTED_LOGICAL_REQUESTS,
            unique_request_bodies: EXPECTED_UNIQUE_BODIES,
            duplicate_logical_requests_avoided: EXPECTED_DUPLICATES_AVOIDED,
            maximum_readiness_contacts: 1,
            maximum_token_count_contacts: EXPECTED_UNIQUE_BODIES,
            inference_allowance: 0,
        },
        ordering: "request_body_sha256 ascending, lowercase ASCII hex".into(),
        endpoint_allowlist: EndpointAllowlist {
            readiness: StaticEndpoint {
                method: "GET".into(),
                path: READINESS_PATH.into(),
            },
            token_count: StaticEndpoint {
                method: "POST".into(),
                path: INPUT_TOKEN_PATH.into(),
            },
        },
        entries: entries.iter().map(|(entry, _)| entry.clone()).collect(),
    };
    if plan.entries.len() != EXPECTED_UNIQUE_BODIES {
        return Err(TokenizationError("contact plan count is not 22".into()));
    }
    let frozen_requests = entries
        .into_iter()
        .map(|(entry, exact_body)| FrozenRequest {
            request_body_sha256: entry.request_body_sha256,
            request_body_utf8_byte_length: entry.request_body_utf8_byte_length,
            representative_logical_request_id: entry.representative_logical_request_id,
            logical_request_ids: entry.logical_request_ids,
            exact_body,
        })
        .collect();
    Ok((plan, frozen_requests))
}

pub fn contact_plan_bytes(ledger_bytes: &[u8]) -> Result<Vec<u8>, TokenizationError> {
    let (plan, _) = build_plan_and_requests(ledger_bytes)?;
    let mut bytes = serde_json::to_vec_pretty(&plan)
        .map_err(|error| TokenizationError(format!("could not encode contact plan: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn validate_frozen_bytes(
    ledger_bytes: &[u8],
    plan_bytes: &[u8],
) -> Result<ValidatedInputs, TokenizationError> {
    let plan_sha256 = digest(plan_bytes);
    if plan_sha256 != ACCEPTED_PLAN_SHA256 {
        return Err(TokenizationError(format!(
            "frozen contact-plan hash mismatch: {plan_sha256}"
        )));
    }
    let (recomputed, requests) = build_plan_and_requests(ledger_bytes)?;
    let observed: ContactPlan = serde_json::from_slice(plan_bytes)
        .map_err(|error| TokenizationError(format!("contact plan is invalid JSON: {error}")))?;
    if observed != recomputed {
        return Err(TokenizationError(
            "frozen contact plan differs from the accepted ledger".into(),
        ));
    }
    Ok(ValidatedInputs {
        ledger_sha256: digest(ledger_bytes),
        plan_sha256,
        plan: observed,
        requests,
    })
}

pub fn validate_repository_files(root: &Path) -> Result<ValidatedInputs, TokenizationError> {
    let ledger = fs::read(root.join(LEDGER_RELATIVE_PATH)).map_err(|error| {
        TokenizationError(format!("cannot read accepted request ledger: {error}"))
    })?;
    let plan = fs::read(root.join(PLAN_RELATIVE_PATH))
        .map_err(|error| TokenizationError(format!("cannot read frozen contact plan: {error}")))?;
    validate_frozen_bytes(&ledger, &plan)
}

pub fn validate_repository_files_at_manifest(
) -> Result<(PathBuf, ValidatedInputs), TokenizationError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let validated = validate_repository_files(&root)?;
    Ok((root, validated))
}

pub fn endpoint_path_is_allowed(path: &str) -> bool {
    ALLOWED_ENDPOINT_PATHS.contains(&path)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTokenCount {
    count: u64,
    field: String,
}

fn parse_token_count(response: &RawHttpResponse) -> Result<ParsedTokenCount, String> {
    if response.status != 200 {
        return Err(format!(
            "input-token endpoint returned HTTP {}",
            response.status
        ));
    }
    if response.truncated {
        return Err("input-token response exceeded the 65536-byte evidence limit".into());
    }
    let value: Value = serde_json::from_slice(&response.body)
        .map_err(|error| format!("input-token response is not valid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "input-token response is not a JSON object".to_owned())?;
    let fields = ["input_tokens", "tokens"]
        .into_iter()
        .filter(|key| object.contains_key(*key))
        .collect::<Vec<_>>();
    if fields.len() != 1 {
        return Err("input-token response must contain exactly one supported count field".into());
    }
    let field = fields[0];
    let count = object[field].as_u64().ok_or_else(|| {
        format!("input-token response field {field} is not a nonnegative integer")
    })?;
    Ok(ParsedTokenCount {
        count,
        field: field.to_owned(),
    })
}

fn response_identity(response: &RawHttpResponse) -> (String, usize) {
    (digest(&response.body), response.body.len())
}

pub fn new_pass_evidence(inputs: &ValidatedInputs) -> PassEvidence {
    PassEvidence {
        schema_id: "prefixity.phase1c.claim2-tokenization-pass-evidence".into(),
        schema_version: 1,
        status: "CONTACTS_NOT_STARTED_OFFLINE_PREFLIGHT_PASSED".into(),
        terminal_classification: None,
        source_identities: EvidenceSourceIdentities {
            ledger_sha256: inputs.ledger_sha256.clone(),
            contact_plan_sha256: inputs.plan_sha256.clone(),
        },
        planned_accounting: inputs.plan.request_counts.clone(),
        operator_runtime_confirmation: json!({
            "operator_started_server": false,
            "model_path": null,
            "model_sha256": null,
            "llama_executable_path": null,
            "llama_build": null,
            "reasoning": null,
            "context_tokens": null,
            "slots": null,
            "offline": null,
            "host": null,
            "port": null
        }),
        contact_accounting: ContactAccounting {
            readiness_contacts: 0,
            unique_token_count_contacts: 0,
            inference_requests: 0,
        },
        readiness: ReadinessEvidence {
            status: "NOT_CONTACTED".into(),
            http_status: None,
            response_body_sha256: None,
            response_body_utf8_byte_length: None,
            error: None,
        },
        unique_request_results: inputs
            .requests
            .iter()
            .map(|request| TokenCountEvidence {
                request_body_sha256: request.request_body_sha256.clone(),
                representative_logical_request_id: request
                    .representative_logical_request_id
                    .clone(),
                logical_request_ids: request.logical_request_ids.clone(),
                status: "NOT_CONTACTED".into(),
                http_status: None,
                input_tokens: None,
                response_count_field: None,
                response_body_sha256: None,
                response_body_utf8_byte_length: None,
                error: None,
            })
            .collect(),
        admission_outcomes: json!({
            "context_fit": null,
            "positive_reduction": null,
            "materiality_800_tokens": null,
            "request3_reduction_20_percent": null,
            "cumulative_reduction_8_percent": null,
            "control_zero_mutation": null,
            "cpu_practicality": null
        }),
    }
}

fn save_evidence<F>(evidence: &PassEvidence, persist: &mut F) -> Result<(), TokenizationError>
where
    F: FnMut(&PassEvidence) -> Result<(), String>,
{
    persist(evidence).map_err(TokenizationError)
}

/// Executes exactly the frozen contact plan using an injected transport.
/// Production transport has only the two static methods in the trait.
pub fn execute_plan_with_transport<T, F>(
    inputs: &ValidatedInputs,
    transport: &mut T,
    mut persist: F,
) -> Result<PassEvidence, TokenizationError>
where
    T: TokenizationTransport,
    F: FnMut(&PassEvidence) -> Result<(), String>,
{
    let mut evidence = new_pass_evidence(inputs);
    save_evidence(&evidence, &mut persist)?;
    evidence.status = "READINESS_CONTACT_IN_PROGRESS".into();
    evidence.contact_accounting.readiness_contacts = 1;
    save_evidence(&evidence, &mut persist)?;
    match transport.readiness() {
        Ok(response) => {
            let (body_hash, body_len) = response_identity(&response);
            evidence.readiness.http_status = Some(response.status);
            evidence.readiness.response_body_sha256 = Some(body_hash);
            evidence.readiness.response_body_utf8_byte_length = Some(body_len);
            if response.status != 200 || response.truncated {
                evidence.readiness.status = "FAILED".into();
                evidence.readiness.error = Some(if response.truncated {
                    "readiness response exceeded the 65536-byte evidence limit".into()
                } else {
                    format!("readiness endpoint returned HTTP {}", response.status)
                });
                evidence.status = "TOKENIZATION_PASS_INCONCLUSIVE".into();
                evidence.terminal_classification = Some("TOKENIZATION_PASS_INCONCLUSIVE".into());
                save_evidence(&evidence, &mut persist)?;
                return Ok(evidence);
            }
            evidence.readiness.status = "READY".into();
            evidence.status = "READINESS_PASSED".into();
            save_evidence(&evidence, &mut persist)?;
        }
        Err(error) => {
            evidence.readiness.status = "FAILED".into();
            evidence.readiness.error = Some(error);
            evidence.status = "TOKENIZATION_PASS_INCONCLUSIVE".into();
            evidence.terminal_classification = Some("TOKENIZATION_PASS_INCONCLUSIVE".into());
            save_evidence(&evidence, &mut persist)?;
            return Ok(evidence);
        }
    }

    for (index, request) in inputs.requests.iter().enumerate() {
        evidence.status = "TOKEN_COUNT_CONTACT_IN_PROGRESS".into();
        evidence.contact_accounting.unique_token_count_contacts += 1;
        evidence.unique_request_results[index].status = "CONTACTED".into();
        save_evidence(&evidence, &mut persist)?;
        let response = match transport.input_tokens(&request.exact_body) {
            Ok(response) => response,
            Err(error) => {
                let result = &mut evidence.unique_request_results[index];
                result.status = "FAILED".into();
                result.error = Some(error);
                evidence.status = "TOKENIZATION_PASS_INCONCLUSIVE".into();
                evidence.terminal_classification = Some("TOKENIZATION_PASS_INCONCLUSIVE".into());
                save_evidence(&evidence, &mut persist)?;
                return Ok(evidence);
            }
        };
        let (body_hash, body_len) = response_identity(&response);
        let result = &mut evidence.unique_request_results[index];
        result.http_status = Some(response.status);
        result.response_body_sha256 = Some(body_hash);
        result.response_body_utf8_byte_length = Some(body_len);
        match parse_token_count(&response) {
            Ok(parsed) => {
                result.status = "COUNTED".into();
                result.input_tokens = Some(parsed.count);
                result.response_count_field = Some(parsed.field);
            }
            Err(error) => {
                result.status = "FAILED".into();
                result.error = Some(error);
                evidence.status = "TOKENIZATION_PASS_INCONCLUSIVE".into();
                evidence.terminal_classification = Some("TOKENIZATION_PASS_INCONCLUSIVE".into());
                save_evidence(&evidence, &mut persist)?;
                return Ok(evidence);
            }
        }
        save_evidence(&evidence, &mut persist)?;
    }
    evidence.status = "TOKENIZATION_COUNTS_COMPLETE_PENDING_REVIEW".into();
    save_evidence(&evidence, &mut persist)?;
    Ok(evidence)
}

pub fn plan_summary(inputs: &ValidatedInputs) -> Value {
    json!({
        "status": "OFFLINE_PREFLIGHT_PASSED_ZERO_CONTACTS",
        "ledger_sha256": inputs.ledger_sha256,
        "contact_plan_sha256": inputs.plan_sha256,
        "logical_requests": inputs.plan.request_counts.logical_requests,
        "unique_request_bodies": inputs.plan.request_counts.unique_request_bodies,
        "duplicate_logical_requests_avoided": inputs.plan.request_counts.duplicate_logical_requests_avoided,
        "maximum_readiness_contacts": inputs.plan.request_counts.maximum_readiness_contacts,
        "maximum_token_count_contacts": inputs.plan.request_counts.maximum_token_count_contacts,
        "inference_allowance": inputs.plan.request_counts.inference_allowance,
        "ordered_hashes": inputs.plan.entries.iter().map(|entry| &entry.request_body_sha256).collect::<Vec<_>>(),
        "server_contacts": 0,
        "inference_requests": 0
    })
}

pub fn repository_root_from_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
