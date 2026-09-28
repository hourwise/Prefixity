use prefixity_controlled_benchmark::phase1c_claim2_tokenization::{
    contact_plan_bytes, endpoint_path_is_allowed, execute_plan_with_transport,
    validate_frozen_bytes, FrozenRequest, RawHttpResponse, TokenizationTransport,
    ACCEPTED_LEDGER_SHA256, LEDGER_RELATIVE_PATH, PLAN_RELATIVE_PATH,
};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn accepted_inputs() -> prefixity_controlled_benchmark::phase1c_claim2_tokenization::ValidatedInputs
{
    let root = repository_root();
    let ledger = fs::read(root.join(LEDGER_RELATIVE_PATH)).unwrap();
    let plan = fs::read(root.join(PLAN_RELATIVE_PATH)).unwrap();
    validate_frozen_bytes(&ledger, &plan).unwrap()
}

#[derive(Default)]
struct FakeTransport {
    readiness_calls: usize,
    token_calls: usize,
    sent_bodies: Vec<Vec<u8>>,
    readiness_result: Option<Result<RawHttpResponse, String>>,
    token_results: VecDeque<Result<RawHttpResponse, String>>,
}

impl TokenizationTransport for FakeTransport {
    fn readiness(&mut self) -> Result<RawHttpResponse, String> {
        self.readiness_calls += 1;
        self.readiness_result
            .take()
            .unwrap_or_else(|| Ok(response(200, b"{\"status\":\"ok\"}")))
    }

    fn input_tokens(&mut self, exact_body: &[u8]) -> Result<RawHttpResponse, String> {
        self.token_calls += 1;
        self.sent_bodies.push(exact_body.to_vec());
        self.token_results.pop_front().unwrap_or_else(|| {
            Ok(response(
                200,
                b"{\"input_tokens\":100,\"object\":\"response.input_tokens\"}",
            ))
        })
    }
}

fn response(status: u16, body: &[u8]) -> RawHttpResponse {
    RawHttpResponse {
        status,
        body: body.to_vec(),
        truncated: false,
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn endpoint_allowlist_is_exact_and_generation_paths_are_unselectable() {
    assert!(endpoint_path_is_allowed("/health"));
    assert!(endpoint_path_is_allowed(
        "/v1/chat/completions/input_tokens"
    ));
    for forbidden in [
        "/v1/chat/completions",
        "/v1/completions",
        "/v1/chat/completions/",
        "/v1/chat/completions/input_tokens/extra",
        "/v1/chat/completions/input_tokens?redirect=/v1/chat/completions",
        "https://example.invalid/v1/chat/completions/input_tokens",
        "//127.0.0.1:8080/v1/chat/completions",
    ] {
        assert!(!endpoint_path_is_allowed(forbidden), "accepted {forbidden}");
    }
}

#[test]
fn contact_plan_is_the_sorted_22_hash_frozen_projection_of_all_exact_54_bodies() {
    let inputs = accepted_inputs();
    assert_eq!(inputs.ledger_sha256, ACCEPTED_LEDGER_SHA256);
    assert_eq!(inputs.requests.len(), 22);
    assert_eq!(inputs.plan.entries.len(), 22);
    assert_eq!(inputs.plan.request_counts.logical_requests, 54);
    assert_eq!(inputs.plan.request_counts.unique_request_bodies, 22);
    assert_eq!(
        inputs
            .plan
            .request_counts
            .duplicate_logical_requests_avoided,
        32
    );
    assert_eq!(inputs.plan.request_counts.maximum_readiness_contacts, 1);
    assert_eq!(inputs.plan.request_counts.maximum_token_count_contacts, 22);
    assert_eq!(inputs.plan.request_counts.inference_allowance, 0);
    assert!(inputs
        .plan
        .entries
        .windows(2)
        .all(|pair| pair[0].request_body_sha256 < pair[1].request_body_sha256));
    for entry in &inputs.plan.entries {
        assert_eq!(
            entry.representative_logical_request_id,
            entry.logical_request_ids[0]
        );
        let request = inputs
            .requests
            .iter()
            .find(|request| request.request_body_sha256 == entry.request_body_sha256)
            .unwrap();
        assert_eq!(
            request.exact_body.len(),
            entry.request_body_utf8_byte_length
        );
        assert_eq!(digest(&request.exact_body), entry.request_body_sha256);
        assert_eq!(request.logical_request_ids, entry.logical_request_ids);
    }
}

#[test]
fn ledger_and_plan_tampering_or_substitution_fail_before_a_transport_exists() {
    let root = repository_root();
    let ledger = fs::read(root.join(LEDGER_RELATIVE_PATH)).unwrap();
    let plan = fs::read(root.join(PLAN_RELATIVE_PATH)).unwrap();
    let mut ledger_tampered = ledger.clone();
    let body_marker = b"future_token_counter_body";
    let marker_at = ledger_tampered
        .windows(body_marker.len())
        .position(|window| window == body_marker)
        .unwrap();
    ledger_tampered[marker_at] ^= 1;
    assert!(validate_frozen_bytes(&ledger_tampered, &plan).is_err());

    let mut plan_tampered = plan.clone();
    let endpoint_at = plan_tampered
        .windows(b"/health".len())
        .position(|window| window == b"/health")
        .unwrap();
    plan_tampered[endpoint_at] = b'X';
    assert!(validate_frozen_bytes(&ledger, &plan_tampered).is_err());

    let substituted_ledger =
        fs::read(root.join("fixtures/claim2/materialization-report-v2.json")).unwrap();
    assert!(validate_frozen_bytes(&substituted_ledger, &plan).is_err());
    let substituted_plan =
        fs::read(root.join("fixtures/claim2/advancing-output-domain-v1.json")).unwrap();
    assert!(validate_frozen_bytes(&ledger, &substituted_plan).is_err());
    let regenerated_plan = contact_plan_bytes(&ledger).unwrap();
    assert_eq!(plan, regenerated_plan);
}

#[test]
fn plan_generation_does_not_expand_after_counts_and_keeps_the_frozen_endpoint_map() {
    let root = repository_root();
    let ledger = fs::read(root.join(LEDGER_RELATIVE_PATH)).unwrap();
    let plan = contact_plan_bytes(&ledger).unwrap();
    assert_eq!(plan, fs::read(root.join(PLAN_RELATIVE_PATH)).unwrap());
    let inputs = validate_frozen_bytes(&ledger, &plan).unwrap();
    assert_eq!(inputs.requests.len(), 22);
    assert_eq!(inputs.plan.endpoint_allowlist.readiness.method, "GET");
    assert_eq!(inputs.plan.endpoint_allowlist.readiness.path, "/health");
    assert_eq!(inputs.plan.endpoint_allowlist.token_count.method, "POST");
    assert_eq!(
        inputs.plan.endpoint_allowlist.token_count.path,
        "/v1/chat/completions/input_tokens"
    );
    assert_eq!(
        inputs
            .plan
            .entries
            .iter()
            .map(|entry| entry.request_body_sha256.as_str())
            .collect::<Vec<_>>(),
        inputs
            .requests
            .iter()
            .map(|request| request.request_body_sha256.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn readiness_failure_stops_with_one_contact_zero_token_counts_and_partial_evidence() {
    let inputs = accepted_inputs();
    let mut fake = FakeTransport {
        readiness_result: Some(Err("offline test readiness failure".into())),
        ..FakeTransport::default()
    };
    let mut saved = Vec::new();
    let evidence = execute_plan_with_transport(&inputs, &mut fake, |record| {
        saved.push(record.clone());
        Ok(())
    })
    .unwrap();
    assert_eq!(fake.readiness_calls, 1);
    assert_eq!(fake.token_calls, 0);
    assert_eq!(
        evidence.terminal_classification.as_deref(),
        Some("TOKENIZATION_PASS_INCONCLUSIVE")
    );
    assert_eq!(evidence.contact_accounting.readiness_contacts, 1);
    assert_eq!(evidence.contact_accounting.unique_token_count_contacts, 0);
    assert_eq!(evidence.unique_request_results[0].status, "NOT_CONTACTED");
    assert_eq!(saved.last().unwrap(), &evidence);
}

#[test]
fn malformed_token_response_stops_without_retry_and_preserves_partial_counts() {
    let inputs = accepted_inputs();
    let mut fake = FakeTransport {
        token_results: VecDeque::from([
            Ok(response(
                200,
                b"{\"input_tokens\":\"100\",\"object\":\"response.input_tokens\"}",
            )),
            Ok(response(
                200,
                b"{\"input_tokens\":101,\"object\":\"response.input_tokens\"}",
            )),
        ]),
        ..FakeTransport::default()
    };
    let mut saved = Vec::new();
    let evidence = execute_plan_with_transport(&inputs, &mut fake, |record| {
        saved.push(record.clone());
        Ok(())
    })
    .unwrap();
    assert_eq!(fake.readiness_calls, 1);
    assert_eq!(fake.token_calls, 1);
    assert_eq!(
        evidence.terminal_classification.as_deref(),
        Some("TOKENIZATION_PASS_INCONCLUSIVE")
    );
    assert_eq!(evidence.contact_accounting.unique_token_count_contacts, 1);
    assert_eq!(evidence.unique_request_results[0].status, "FAILED");
    assert_eq!(evidence.unique_request_results[0].input_tokens, None);
    assert_eq!(evidence.unique_request_results[1].status, "NOT_CONTACTED");
    assert_eq!(saved.last().unwrap(), &evidence);
}

#[test]
fn token_response_requires_the_accepted_llama_object_and_input_tokens_field() {
    let inputs = accepted_inputs();
    for body in [
        br#"{"input_tokens":100}"#.as_slice(),
        br#"{"tokens":100,"object":"response.input_tokens"}"#.as_slice(),
        br#"{"input_tokens":100,"object":"another.response"}"#.as_slice(),
    ] {
        let mut fake = FakeTransport {
            token_results: VecDeque::from([Ok(response(200, body))]),
            ..FakeTransport::default()
        };
        let evidence = execute_plan_with_transport(&inputs, &mut fake, |_| Ok(())).unwrap();
        assert_eq!(fake.readiness_calls, 1);
        assert_eq!(fake.token_calls, 1);
        assert_eq!(
            evidence.terminal_classification.as_deref(),
            Some("TOKENIZATION_PASS_INCONCLUSIVE")
        );
        assert_eq!(evidence.unique_request_results[0].status, "FAILED");
        assert_eq!(evidence.unique_request_results[1].status, "NOT_CONTACTED");
    }
}

#[test]
fn fake_transport_receives_each_exact_frozen_body_once_and_returns_counts_only() {
    let inputs = accepted_inputs();
    let expected_bodies = inputs
        .requests
        .iter()
        .map(|request: &FrozenRequest| request.exact_body.clone())
        .collect::<Vec<_>>();
    let mut fake = FakeTransport::default();
    let evidence = execute_plan_with_transport(&inputs, &mut fake, |_| Ok(())).unwrap();
    assert_eq!(fake.readiness_calls, 1);
    assert_eq!(fake.token_calls, 22);
    assert_eq!(fake.sent_bodies, expected_bodies);
    assert_eq!(
        evidence.status,
        "TOKENIZATION_COUNTS_COMPLETE_PENDING_REVIEW"
    );
    assert_eq!(evidence.contact_accounting.readiness_contacts, 1);
    assert_eq!(evidence.contact_accounting.unique_token_count_contacts, 22);
    assert_eq!(evidence.contact_accounting.inference_requests, 0);
    assert!(evidence
        .unique_request_results
        .iter()
        .all(|result| result.status == "COUNTED" && result.input_tokens == Some(100)));
    assert_eq!(
        evidence.admission_outcomes["context_fit"],
        serde_json::Value::Null
    );
    assert_eq!(
        evidence.admission_outcomes["cpu_practicality"],
        serde_json::Value::Null
    );
}
