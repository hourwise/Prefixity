use prefixity_controlled_benchmark::{
    normalize_llama_cpp_response, project_llama_cpp_request, CaseRelationship,
    ConformanceExperiment, FakeLlamaCppTransport, LlamaCppConformanceRunner, LlamaCppResponse,
    LlamaCppResponseFormat, MutationClass, LLAMA_CPP_PROTOCOL_ID,
};
use prefixity_core::observation::{
    CapabilityEvidence, CapabilitySupport, Observed, RuntimeCacheCapabilities, RuntimeIdentity,
    CACHE_OBSERVATION_SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn response_fixture(name: &str) -> LlamaCppResponse {
    let path = workspace_root().join("fixtures/llama-cpp").join(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    let value: Value = serde_json::from_slice(&bytes).expect("fixture JSON should parse");
    LlamaCppResponse::from_json(value).expect("fixture response should normalize to protocol data")
}

fn documented_capability_fixture() -> RuntimeCacheCapabilities {
    let path = workspace_root().join("fixtures/capabilities/llama-cpp-documented-v1.json");
    let bytes = std::fs::read(path).expect("documented capability fixture should exist");
    serde_json::from_slice(&bytes).expect("documented capability fixture should parse")
}

fn experiment() -> ConformanceExperiment {
    let path = workspace_root().join("fixtures/conformance/coding-agent-cache-conformance-v1.json");
    let bytes = std::fs::read(path).expect("conformance fixture should exist");
    serde_json::from_slice(&bytes).expect("conformance fixture should parse")
}

fn runtime() -> RuntimeIdentity {
    RuntimeIdentity {
        backend: "llama.cpp".to_string(),
        provider: Observed::Known("local".to_string()),
        model: Observed::Known("example-local-model".to_string()),
        protocol: Observed::Known(LLAMA_CPP_PROTOCOL_ID.to_string()),
        runtime: Observed::Known("llama.cpp".to_string()),
        runtime_version: Observed::Unknown,
        ..RuntimeIdentity::default()
    }
}

fn llama_profiled_experiment() -> ConformanceExperiment {
    let mut experiment = experiment();
    experiment.runtime_profile.profile_id = "synthetic-llama-cpp-profile-v1".to_string();
    experiment.runtime_profile.identity = runtime();
    experiment
}

fn supported_experiment() -> ConformanceExperiment {
    let mut experiment = llama_profiled_experiment();
    for case in &mut experiment.cases {
        case.request.envelope.reasoning = None;
    }
    experiment.baseline_request.envelope.reasoning = None;
    experiment
}

#[test]
fn projection_is_deterministic_and_preserves_context_order() {
    let mut experiment = experiment();
    experiment.baseline_request.envelope.reasoning = None;
    for case in &mut experiment.cases {
        case.request.envelope.reasoning = None;
    }
    let baseline = project_llama_cpp_request(&experiment.cases[0].request).unwrap();
    let repeated = project_llama_cpp_request(&experiment.cases[1].request).unwrap();
    assert_eq!(
        serde_json::to_vec(&baseline).unwrap(),
        serde_json::to_vec(&repeated).unwrap()
    );
    assert_eq!(baseline.model, "example-local-model");
    assert_eq!(baseline.messages[0].role, "system");
    assert_eq!(
        baseline.messages[1].content,
        "src/lib.rs\nsrc/main.rs\ntests/\nCargo.toml"
    );
    assert_eq!(baseline.messages[2].role, "user");
    assert_eq!(baseline.tools[0].function.name, "read_file");
    assert_eq!(baseline.tools[1].function.name, "list_files");
    assert!(matches!(
        baseline.response_format,
        Some(LlamaCppResponseFormat::Text)
    ));
}

#[test]
fn projection_preserves_whitespace_tool_and_envelope_mutations() {
    let experiment = supported_experiment();
    let baseline = project_llama_cpp_request(&experiment.cases[0].request).unwrap();
    let whitespace = project_llama_cpp_request(&experiment.cases[4].request).unwrap();
    let tool_order = project_llama_cpp_request(&experiment.cases[6].request).unwrap();
    let tool_change = project_llama_cpp_request(&experiment.cases[8].request).unwrap();
    let model_change = project_llama_cpp_request(&experiment.cases[9].request).unwrap();
    let response_change = project_llama_cpp_request(&experiment.cases[11].request).unwrap();
    assert_ne!(baseline.messages, whitespace.messages);
    assert_ne!(
        baseline
            .tools
            .iter()
            .map(|tool| &tool.function.name)
            .collect::<Vec<_>>(),
        tool_order
            .tools
            .iter()
            .map(|tool| &tool.function.name)
            .collect::<Vec<_>>()
    );
    assert_ne!(baseline.tools[0], tool_change.tools[0]);
    assert_ne!(baseline.model, model_change.model);
    assert_ne!(baseline.response_format, response_change.response_format);
}

#[test]
fn reasoning_setting_is_rejected_as_not_representable() {
    let experiment = experiment();
    let error = project_llama_cpp_request(&experiment.cases[10].request).unwrap_err();
    let text = error.to_string();
    assert!(text.contains("reasoning-setting") || text.contains("reasoning"));
}

#[test]
fn capability_fixture_is_documented_not_experimentally_observed() {
    let capabilities = documented_capability_fixture();
    capabilities.validate().unwrap();
    assert_eq!(
        capabilities.prefix_cache.prefix_reuse.support,
        CapabilitySupport::Supported
    );
    assert_eq!(
        capabilities.prefix_cache.prefix_reuse.evidence,
        CapabilityEvidence::Documented
    );
    assert_ne!(
        capabilities.prefix_cache.prefix_reuse.evidence,
        CapabilityEvidence::ExperimentallyObserved
    );
}

#[test]
fn response_a_maps_native_timings_and_usage_without_collapsing_fields() {
    let experiment = experiment();
    let response = response_fixture("response-cache-reuse.json");
    let observation = normalize_llama_cpp_response(
        &response,
        &experiment.cases[0].request,
        "request-fingerprint".to_string(),
        "observation-a".to_string(),
        "2026-08-12T12:00:00Z".to_string(),
        runtime(),
    )
    .unwrap();
    observation.validate().unwrap();
    assert_eq!(observation.schema_version, CACHE_OBSERVATION_SCHEMA_VERSION);
    assert_eq!(
        observation.accounting.transmitted_input_tokens,
        Observed::Known(token_count(100))
    );
    assert_eq!(
        observation.accounting.provider_cached_tokens,
        Observed::Known(token_count(80))
    );
    assert_eq!(
        observation.accounting.fresh_prefill_tokens,
        Observed::Known(token_count(20))
    );
    assert_eq!(
        observation.accounting.output_tokens,
        Observed::Known(token_count(12))
    );
    assert_eq!(observation.timing.prefill_duration_ms, Observed::Known(11));
    assert_eq!(
        observation.timing.generation_duration_ms,
        Observed::Known(20)
    );
    assert!(matches!(
        observation.accounting.reconstructed_context_tokens,
        Observed::NotObserved
    ));
    assert_eq!(
        observation.raw_telemetry["adapter"],
        json!(LLAMA_CPP_PROTOCOL_ID)
    );
    assert!(observation.raw_telemetry.contains_key("llama_cpp_timings"));
    assert!(observation.raw_telemetry.contains_key("llama_cpp_usage"));
}

#[test]
fn absent_cache_telemetry_is_not_observed_and_explicit_zero_is_known() {
    let experiment = experiment();
    let absent = response_fixture("response-no-cache-telemetry.json");
    let observation = normalize_llama_cpp_response(
        &absent,
        &experiment.cases[0].request,
        "request-fingerprint".to_string(),
        "observation-b".to_string(),
        "2026-08-12T12:00:00Z".to_string(),
        runtime(),
    )
    .unwrap();
    assert!(matches!(
        observation.accounting.provider_cached_tokens,
        Observed::NotObserved
    ));
    assert!(matches!(
        observation.accounting.fresh_prefill_tokens,
        Observed::NotObserved
    ));

    let zero = LlamaCppResponse::from_json(json!({
        "timings": {"cache_n": 0, "prompt_n": 100, "predicted_n": 0},
        "usage": {"prompt_tokens": 100, "completion_tokens": 0, "total_tokens": 100,
                   "prompt_tokens_details": {"cached_tokens": 0}}
    }))
    .unwrap();
    let observation = normalize_llama_cpp_response(
        &zero,
        &experiment.cases[0].request,
        "request-fingerprint".to_string(),
        "observation-zero".to_string(),
        "2026-08-12T12:00:00Z".to_string(),
        runtime(),
    )
    .unwrap();
    assert_eq!(
        observation.accounting.provider_cached_tokens,
        Observed::Known(token_count(0))
    );
    assert_eq!(
        observation.accounting.fresh_prefill_tokens,
        Observed::Known(token_count(100))
    );
}

#[test]
fn malformed_and_conflicting_telemetry_fail_cleanly() {
    let malformed = workspace_root().join("fixtures/llama-cpp/response-malformed-timings.json");
    let value: Value = serde_json::from_slice(&std::fs::read(malformed).unwrap()).unwrap();
    assert!(LlamaCppResponse::from_json(value).is_err());

    let experiment = experiment();
    let conflict = response_fixture("response-conflicting-cache.json");
    let error = normalize_llama_cpp_response(
        &conflict,
        &experiment.cases[0].request,
        "request-fingerprint".to_string(),
        "observation-d".to_string(),
        "2026-08-12T12:00:00Z".to_string(),
        runtime(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("conflicting"));
}

#[test]
fn fake_transport_runs_supported_cases_in_order_and_preserves_result_provenance() {
    let experiment = supported_experiment();
    let responses = (0..12)
        .map(|_| Ok(response_fixture("response-cache-reuse.json")))
        .collect();
    let transport = FakeLlamaCppTransport::new(responses);
    let mut runner = LlamaCppConformanceRunner::new(transport, "2026-08-12T12:00:00Z", runtime());
    let mut supported = experiment.clone();
    supported
        .cases
        .retain(|case| case.mutation != MutationClass::ReasoningSetting);
    let result = supported.run(&mut runner).unwrap();
    result.validate().unwrap();
    assert_eq!(result.provenance["runner"], "llama.cpp-adapter");
    assert_eq!(
        result.provenance["evidence"],
        "synthetic-protocol-validation-only"
    );
    assert_eq!(result.cases[0].case_id, "baseline");
    assert_eq!(
        result.cases[1].relationship,
        CaseRelationship::ExactRepeatOf("baseline".to_string())
    );
    assert!(result
        .cases
        .iter()
        .all(|case| case.observation.raw_telemetry["adapter"] == json!(LLAMA_CPP_PROTOCOL_ID)));
    assert_eq!(runner.transport().requests().len(), 11);
}

#[test]
fn unsupported_case_and_transport_failure_are_bounded_case_errors() {
    let unsupported = llama_profiled_experiment();
    let transport = FakeLlamaCppTransport::new(Vec::new());
    let mut runner = LlamaCppConformanceRunner::new(transport, "2026-08-12T12:00:00Z", runtime());
    let error = unsupported.run(&mut runner).unwrap_err().to_string();
    assert!(error.contains("case=baseline"));
    assert!(error.contains("not representable"));

    let experiment = supported_experiment();
    let transport =
        FakeLlamaCppTransport::new(vec![Err("synthetic transport unavailable".to_string())]);
    let mut runner = LlamaCppConformanceRunner::new(transport, "2026-08-12T12:00:00Z", runtime());
    let error = experiment.run(&mut runner).unwrap_err().to_string();
    assert!(error.contains("experiment=coding-agent-cache-conformance-v1"));
    assert!(error.contains("case=baseline"));
    assert!(error.len() < 1200);

    let transport = FakeLlamaCppTransport::new(Vec::new());
    let mut runner = LlamaCppConformanceRunner::new(transport, "2026-08-12T12:00:00Z", runtime());
    let error = experiment.run(&mut runner).unwrap_err().to_string();
    assert!(error.contains("case=baseline"));
    assert!(error.contains("transport"));
}

fn token_count(count: u64) -> prefixity_core::observation::TokenCount {
    prefixity_core::observation::TokenCount {
        count,
        provider: Observed::Known("local".to_string()),
        model: Observed::Known("example-local-model".to_string()),
        tokenizer: Observed::Unknown,
    }
}
