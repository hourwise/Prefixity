use prefixity_controlled_benchmark::{
    build_live_experiment_definition, evaluate_candidate, execute_live_experiment,
    materialize_candidate, preflight_live_experiment, BenchmarkError, CandidateEvaluation,
    CandidateEvaluationInput, ConformanceRequest, ContextArtifactInput, ContextStabilityInputs,
    EnvironmentState, LiveEnvironmentManifest, LiveEvidenceState, LivePreparationErrorCode,
    LiveRawEvidenceSource, LiveRunRecord, LlamaCppLiveConfig, LlamaCppRequest, LlamaCppResponse,
    LlamaCppTransport, LoopbackEndpoint, OrderedJsonObject, OrderingConstraint, RequestContext,
    RequestEnvelope, RuntimeProfileReference, ToolDefinition,
};
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, ContextArtifact, Observed,
    RuntimeIdentity, TrustLevel, CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
use std::collections::BTreeMap;

fn request() -> ConformanceRequest {
    ConformanceRequest {
        context: RequestContext {
            system_instruction: "system instruction".to_string(),
            artifacts: vec![
                ContextArtifactInput {
                    artifact_id: "a".to_string(),
                    content: "artifact-a".to_string(),
                },
                ContextArtifactInput {
                    artifact_id: "b".to_string(),
                    content: "artifact-b".to_string(),
                },
                ContextArtifactInput {
                    artifact_id: "c".to_string(),
                    content: "artifact-c".to_string(),
                },
            ],
            user_content: "current user task".to_string(),
            tools: vec![
                ToolDefinition {
                    name: "read_file".to_string(),
                    description: "read a bounded file".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
                ToolDefinition {
                    name: "list_files".to_string(),
                    description: "list bounded files".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
            ],
        },
        envelope: RequestEnvelope {
            model: "fixture-model".to_string(),
            reasoning: None,
            response_format: None,
        },
    }
}

fn artifact(id: &str, stability: ArtifactStability) -> ContextArtifact {
    ContextArtifact {
        schema_version: CONTEXT_ARTIFACT_SCHEMA_VERSION,
        artifact_id: id.to_string(),
        origin_id: format!("origin-{id}"),
        content_source_id: Observed::Known(format!("source-{id}")),
        content_hash: Observed::Unknown,
        revision: Observed::Known("v1".to_string()),
        artifact_type: ArtifactType::SourceFile,
        stability,
        lifecycle: ArtifactLifecycle::PersistentVersioned,
        sizes: ArtifactSizes {
            byte_size: Observed::Known(10),
            ..ArtifactSizes::default()
        },
        cache: Default::default(),
        trust: Observed::Known(TrustLevel::Trusted),
        provenance: BTreeMap::new(),
        metadata: BTreeMap::new(),
    }
}

fn materialized() -> (
    ConformanceRequest,
    prefixity_controlled_benchmark::MaterializedCandidate,
) {
    let source = request();
    let inputs = ContextStabilityInputs {
        artifacts: BTreeMap::from([
            ("a".to_string(), artifact("a", ArtifactStability::Stable)),
            ("b".to_string(), artifact("b", ArtifactStability::Volatile)),
            ("c".to_string(), artifact("c", ArtifactStability::Stable)),
        ]),
        ..ContextStabilityInputs::default()
    };
    let constraints = prefixity_controlled_benchmark::LayoutPlanningConstraints {
        constraints: ["a", "b", "c"]
            .into_iter()
            .map(|id| OrderingConstraint::MovableWithinCompatibleRegion {
                segment: format!("context.artifacts[{id}]"),
                region: "artifact-sequence".to_string(),
            })
            .collect(),
        ..Default::default()
    };
    let plan = prefixity_controlled_benchmark::plan_request_layout(&source, &inputs, &constraints)
        .unwrap();
    let candidate = plan.candidates[0].clone();
    let evaluation: CandidateEvaluation = evaluate_candidate(CandidateEvaluationInput {
        candidate: &candidate,
        capability_profile: None,
        observations: &[],
        environment: &EnvironmentState::available(),
    })
    .unwrap();
    let materialized = materialize_candidate(
        &source,
        &plan,
        &candidate,
        &evaluation,
        &inputs,
        BTreeMap::from([("case".to_string(), "offline-test".to_string())]),
    )
    .unwrap();
    (source, materialized)
}

fn profile() -> RuntimeProfileReference {
    RuntimeProfileReference {
        profile_id: "fixture-loopback".to_string(),
        identity: RuntimeIdentity {
            backend: "llama.cpp".to_string(),
            provider: Observed::Known("local".to_string()),
            model: Observed::Known("fixture-model".to_string()),
            protocol: Observed::Known("llama.cpp-openai-chat-v1".to_string()),
            runtime: Observed::Known("llama-server".to_string()),
            runtime_version: Observed::Known("fixture-build".to_string()),
            ..RuntimeIdentity::default()
        },
    }
}

fn config(execute_live: bool) -> LlamaCppLiveConfig {
    LlamaCppLiveConfig {
        schema_id: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_VERSION,
        endpoint: LoopbackEndpoint::parse("http://127.0.0.1:8080/v1/chat/completions").unwrap(),
        llama_build: "fixture-build".to_string(),
        model_identity: "fixture-model".to_string(),
        quantization: Some("fixture-quant".to_string()),
        context_size: 4096,
        threads: Some(4),
        gpu_offload: Some("none".to_string()),
        kv_cache: Some("default".to_string()),
        batch_size: Some(128),
        generation_limit: 64,
        parallel_slots: 1,
        metrics_enabled: true,
        temperature: Some(0.0),
        top_p: Some(1.0),
        seed: Some(7),
        connect_timeout_ms: 100,
        request_timeout_ms: 1000,
        max_response_bytes: 1024 * 1024,
        max_context_bytes: 1024 * 1024,
        evidence_location: "evidence/p0-l6".to_string(),
        execute_live,
        fresh_server_for_run: true,
        runtime_profile: profile(),
        provenance: BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    }
}

fn definition() -> prefixity_controlled_benchmark::LiveExperimentDefinition {
    let (source, candidate) = materialized();
    build_live_experiment_definition(
        &source,
        candidate,
        "control-source",
        "candidate-treatment",
        BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    )
    .unwrap()
}

#[test]
fn endpoint_policy_accepts_only_http_loopback() {
    for value in [
        "http://127.0.0.1:8080",
        "http://localhost:8080",
        "http://[::1]:8080",
    ] {
        assert!(LoopbackEndpoint::parse(value).is_ok(), "{value}");
    }
    for value in [
        "https://127.0.0.1:8080",
        "http://192.168.1.10:8080",
        "http://example.com:8080",
        "http://localhost.evil:8080",
    ] {
        assert!(LoopbackEndpoint::parse(value).is_err(), "{value}");
    }
}

#[test]
fn configuration_is_bounded_and_live_execution_is_not_defaulted() {
    let mut value = serde_json::to_value(config(false)).unwrap();
    assert_eq!(value["execute_live"], false);
    value.as_object_mut().unwrap().remove("execute_live");
    let decoded: LlamaCppLiveConfig = serde_json::from_value(value).unwrap();
    assert!(!decoded.execute_live);

    let mut invalid = config(false);
    invalid.evidence_location = "C:\\private\\evidence".to_string();
    assert!(invalid.validate().is_err());
    invalid = config(false);
    invalid.request_timeout_ms = 0;
    assert!(invalid.validate().is_err());
}

#[test]
fn environment_manifest_preserves_unknowns_without_machine_scanning() {
    let unknown = LiveEnvironmentManifest {
        schema_id: prefixity_controlled_benchmark::ENVIRONMENT_MANIFEST_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::ENVIRONMENT_MANIFEST_SCHEMA_VERSION,
        os: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        llama_build: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        model_identity: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        quantization: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        model_file_size_bytes: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        context_size: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        cpu: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        logical_threads: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        gpu: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        ram_bytes: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        vram_bytes: prefixity_controlled_benchmark::EnvironmentObservation::unknown(),
        launch_configuration: BTreeMap::new(),
        provenance: BTreeMap::from([("source".to_string(), "caller-supplied".to_string())]),
    };
    assert!(unknown.validate().is_ok());
    let encoded = serde_json::to_string(&unknown).unwrap();
    assert!(encoded.contains("unknown"));
}

#[test]
fn fixed_sequence_and_interference_are_deterministic() {
    let first = definition();
    let second = definition();
    assert_eq!(first.experiment_id, second.experiment_id);
    assert_eq!(
        first
            .sequence
            .iter()
            .map(|step| step.step_id.as_str())
            .collect::<Vec<_>>(),
        ["A1", "A2", "C1", "C2", "B1", "A3", "C3"]
    );
    assert_eq!(
        first.sequence[0].request_fingerprint,
        first.sequence[1].request_fingerprint
    );
    assert_ne!(
        first.sequence[0].request_fingerprint,
        first.sequence[4].request_fingerprint
    );
    assert_ne!(
        first.control_request.context.system_instruction,
        first.interference_request.context.system_instruction
    );
    let serialized = serde_json::to_string(&first).unwrap();
    assert!(!serialized.contains("optimized"));
    assert!(!serialized.contains("tuning"));
}

#[test]
fn preflight_is_machine_readable_and_makes_zero_network_calls() {
    let record = preflight_live_experiment(&definition(), &config(false)).unwrap();
    assert_eq!(record.state, LiveEvidenceState::Prepared);
    assert_eq!(record.network_calls, 0);
    assert_eq!(record.sequence_step_ids.len(), 7);
    assert!(serde_json::to_string(&record).unwrap().contains("prepared"));
}

#[derive(Default)]
struct CountingTransport {
    calls: usize,
}

impl LlamaCppTransport for CountingTransport {
    fn chat_completion(
        &mut self,
        _request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        self.calls += 1;
        Err(BenchmarkError::validation(
            "test transport must not be called",
        ))
    }
}

impl LiveRawEvidenceSource for CountingTransport {
    fn raw_evidence(&self) -> Vec<prefixity_controlled_benchmark::RawLlamaCppEvidence> {
        Vec::new()
    }
}

#[test]
fn explicit_gate_blocks_transport_even_with_valid_preflight() {
    let mut transport = CountingTransport::default();
    let error = execute_live_experiment(&definition(), &config(false), &mut transport).unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::LiveOptInRequired,
            ..
        }
    ));
    assert_eq!(transport.calls, 0);
}

#[test]
fn partial_and_failed_states_cannot_contain_a_final_result() {
    let partial = LiveRunRecord {
        schema_id: prefixity_controlled_benchmark::LIVE_HARNESS_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::LIVE_HARNESS_SCHEMA_VERSION,
        experiment_id: "fixture-experiment".to_string(),
        state: LiveEvidenceState::Partial,
        expected_steps: 7,
        completed_steps: 2,
        raw_evidence: Vec::new(),
        normalized_result: None,
        failure: None,
        provenance: BTreeMap::new(),
    };
    assert!(partial.validate().is_ok());
    let mut invalid = partial.clone();
    invalid.state = LiveEvidenceState::Normalized;
    assert!(invalid.validate().is_err());
}
