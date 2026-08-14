use prefixity_controlled_benchmark::{
    build_paired_mutation_conformance_experiment, build_synthetic_paired_mutation_seed,
    preflight_paired_mutation_experiment, prepare_paired_mutation_experiment,
    project_llama_cpp_request, project_llama_cpp_request_with_generation_limit,
    validate_llama_cpp_generation_limit, BenchmarkError, CaseRelationship, LiveEvidenceState,
    LivePreparationErrorCode, LiveRawEvidenceSource, LlamaCppLiveConfig, LlamaCppRequest,
    LlamaCppResponse, LlamaCppTransport, LoopbackEndpoint, MutationClass, PairedMutationDefinition,
    PairedMutationRunRecord, RuntimeProfileReference,
};
use prefixity_core::observation::{Observed, RuntimeIdentity};
use std::collections::BTreeMap;

fn profile() -> RuntimeProfileReference {
    RuntimeProfileReference {
        profile_id: "fixture-paired-loopback".to_string(),
        identity: RuntimeIdentity {
            backend: "llama.cpp".to_string(),
            provider: Observed::Known("local".to_string()),
            model: Observed::Known("fixture-model".to_string()),
            protocol: Observed::Known("llama.cpp-openai-chat-v1".to_string()),
            runtime: Observed::Known("llama-server".to_string()),
            runtime_version: Observed::Known("caller-supplied".to_string()),
            ..RuntimeIdentity::default()
        },
    }
}

fn config(execute_live: bool, fresh_server_for_run: bool) -> LlamaCppLiveConfig {
    LlamaCppLiveConfig {
        schema_id: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_VERSION,
        endpoint: LoopbackEndpoint::parse("http://127.0.0.1:8080/v1/chat/completions").unwrap(),
        llama_build: "caller-supplied".to_string(),
        model_identity: "ggml-org/Qwen3.5-0.8B-GGUF".to_string(),
        quantization: Some("Q4_0".to_string()),
        context_size: 8192,
        threads: None,
        gpu_offload: None,
        kv_cache: None,
        batch_size: None,
        generation_limit: 1,
        parallel_slots: 1,
        metrics_enabled: true,
        temperature: Some(0.0),
        top_p: Some(1.0),
        seed: Some(1),
        connect_timeout_ms: 100,
        request_timeout_ms: 1000,
        max_response_bytes: 1024 * 1024,
        max_context_bytes: 1024 * 1024,
        evidence_location: "evidence/p0-l6b".to_string(),
        execute_live,
        fresh_server_for_run,
        runtime_profile: profile(),
        provenance: BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    }
}

fn prepared() -> PairedMutationDefinition {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    prepare_paired_mutation_experiment(
        seed,
        profile(),
        true,
        BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    )
    .unwrap()
}

#[test]
fn paired_experiment_has_exactly_five_named_steps() {
    let definition = prepared();
    assert_eq!(definition.sequence.len(), 5);
    assert_eq!(
        definition
            .sequence
            .iter()
            .map(|step| step.step_id.as_str())
            .collect::<Vec<_>>(),
        ["A0", "A1", "B1", "C0", "C1"]
    );
}

#[test]
fn control_mutation_changes_only_volatile_artifact() {
    let definition = prepared();
    let diff = &definition.control_mutation.request_diff;
    assert_eq!(diff.prefix_diff.changes.len(), 1);
    assert_eq!(
        diff.prefix_diff.changes[0].category,
        prefixity_controlled_benchmark::ChangeCategory::ArtifactContentChanged
    );
    assert_ne!(
        definition.control_initial.context.artifacts[1].content,
        definition.control_mutated.context.artifacts[1].content
    );
    assert!(diff.envelope_diff.changes.is_empty());
}

#[test]
fn c0_is_certified_from_a0() {
    let definition = prepared();
    assert_eq!(
        definition.treatment_initial.source_request_fingerprint,
        definition.control_initial.request_fingerprint().unwrap()
    );
    assert!(definition
        .treatment_initial
        .safety_certificate
        .validate()
        .is_ok());
}

#[test]
fn c1_is_independently_certified_from_a1() {
    let definition = prepared();
    assert_eq!(
        definition.treatment_mutated.source_request_fingerprint,
        definition.control_mutated.request_fingerprint().unwrap()
    );
    assert!(definition
        .treatment_mutated
        .safety_certificate
        .validate()
        .is_ok());
    assert_ne!(
        definition.treatment_initial.safety_certificate,
        definition.treatment_mutated.safety_certificate
    );
}

#[test]
fn information_is_conserved_for_both_layouts() {
    let definition = prepared();
    for (source, candidate) in [
        (
            &definition.control_initial,
            &definition.treatment_initial.materialized_request,
        ),
        (
            &definition.control_mutated,
            &definition.treatment_mutated.materialized_request,
        ),
    ] {
        assert_eq!(
            source.context.system_instruction,
            candidate.context.system_instruction
        );
        assert_eq!(source.context.user_content, candidate.context.user_content);
        assert_eq!(source.context.tools, candidate.context.tools);
        assert_eq!(
            source
                .context
                .artifacts
                .iter()
                .map(|artifact| artifact.artifact_id.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            candidate
                .context
                .artifacts
                .iter()
                .map(|artifact| artifact.artifact_id.as_str())
                .collect::<std::collections::BTreeSet<_>>()
        );
    }
}

#[test]
fn control_and_treatment_structures_are_explicit() {
    let definition = prepared();
    assert_eq!(
        definition
            .control_initial
            .context
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>(),
        ["stable-a", "volatile-v", "stable-b"]
    );
    assert_eq!(
        definition
            .treatment_initial
            .materialized_request
            .context
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>(),
        ["stable-a", "stable-b", "volatile-v"]
    );
}

#[test]
fn p0_l10_inversion_reduction_is_recorded_without_a_claim() {
    let definition = prepared();
    assert!(
        definition.treatment_initial_inversion_count <= definition.control_initial_inversion_count
    );
    assert!(
        definition.treatment_initial_leading_segments
            >= definition.control_initial_leading_segments
    );
    assert_eq!(
        definition.primary_comparison.cache_outcome,
        "not_observed; no required direction"
    );
}

#[test]
fn p0_l7_layout_diffs_are_reorder_only() {
    let definition = prepared();
    for comparison in [&definition.control_layout, &definition.treatment_layout] {
        assert_eq!(comparison.request_diff.prefix_diff.changes.len(), 1);
        assert_eq!(
            comparison.request_diff.prefix_diff.changes[0].category,
            prefixity_controlled_benchmark::ChangeCategory::ArtifactOrderChanged
        );
        assert!(comparison.request_diff.envelope_diff.changes.is_empty());
    }
}

#[test]
fn p0_l7_treatment_mutation_is_the_same_volatile_change() {
    let definition = prepared();
    let control_change = &definition.control_mutation.request_diff.prefix_diff.changes[0];
    let treatment_change = &definition
        .treatment_mutation
        .request_diff
        .prefix_diff
        .changes[0];
    assert_eq!(control_change.category, treatment_change.category);
    assert_ne!(
        definition
            .treatment_initial
            .materialized_request
            .context
            .artifacts[2]
            .content,
        definition
            .treatment_mutated
            .materialized_request
            .context
            .artifacts[2]
            .content
    );
    assert!(!treatment_change.order_changed);
}

#[test]
fn identity_and_sequence_are_deterministic() {
    let first = prepared();
    let second = prepared();
    assert_eq!(first.experiment_id, second.experiment_id);
    assert_eq!(first.sequence, second.sequence);
}

#[test]
fn preflight_is_zero_network_and_records_comparisons() {
    let record = preflight_paired_mutation_experiment(&prepared(), &config(false, true)).unwrap();
    assert_eq!(record.state, LiveEvidenceState::Prepared);
    assert_eq!(record.network_calls, 0);
    assert_eq!(record.comparisons, ["A0-to-A1", "C0-to-C1", "A1-vs-C1"]);
    assert!(record.primary_outcome.contains("no required direction"));
}

#[test]
fn paired_generic_conformance_has_one_a0_baseline_and_traceable_c0_layout() {
    let experiment = build_paired_mutation_conformance_experiment(&prepared()).unwrap();
    experiment.validate().unwrap();
    assert_eq!(
        experiment
            .cases
            .iter()
            .filter(|case| case.relationship == CaseRelationship::Baseline)
            .count(),
        1
    );
    assert_eq!(
        experiment
            .cases
            .iter()
            .filter(|case| case.mutation == MutationClass::Baseline)
            .count(),
        1
    );
    let a0 = &experiment.cases[0];
    let a1 = &experiment.cases[1];
    let b1 = &experiment.cases[2];
    let c0 = &experiment.cases[3];
    let c1 = &experiment.cases[4];
    assert_eq!(a0.case_id, "A0");
    assert_eq!(a0.relationship, CaseRelationship::Baseline);
    assert_eq!(a0.mutation, MutationClass::Baseline);
    assert_eq!(a1.mutation, MutationClass::VolatileArtifactContent);
    assert_eq!(
        a1.relationship,
        CaseRelationship::MutationOf("A0".to_string())
    );
    assert_eq!(
        b1.relationship,
        CaseRelationship::MutationOf("A0".to_string())
    );
    assert_eq!(c0.mutation, MutationClass::ArtifactOrder);
    assert_eq!(
        c0.relationship,
        CaseRelationship::MutationOf("A0".to_string())
    );
    assert_eq!(c1.mutation, MutationClass::VolatileArtifactContent);
    assert_eq!(
        c1.relationship,
        CaseRelationship::MutationOf("C0".to_string())
    );
}

#[test]
fn duplicated_baseline_fixture_fails_at_shared_preflight_boundary() {
    let definition = prepared();
    let mut experiment = build_paired_mutation_conformance_experiment(&definition).unwrap();
    let baseline_request = experiment.baseline_request.clone();
    let c0 = experiment
        .cases
        .iter_mut()
        .find(|case| case.case_id == "C0")
        .unwrap();
    c0.mutation = MutationClass::Baseline;
    c0.relationship = CaseRelationship::Baseline;
    c0.request = baseline_request;
    let error = experiment.validate().unwrap_err();
    assert!(error.to_string().contains("exactly one baseline"));
    assert!(
        preflight_paired_mutation_experiment(&definition, &config(false, true)).is_ok(),
        "the valid preflight must use the same validated construction path"
    );
}

#[test]
fn fresh_server_is_a_caller_assertion() {
    let error =
        preflight_paired_mutation_experiment(&prepared(), &config(false, false)).unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::FreshServerAssertionRequired,
            ..
        }
    ));
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
        Err(BenchmarkError::validation("transport must not be called"))
    }
}

impl LiveRawEvidenceSource for CountingTransport {
    fn raw_evidence(&self) -> Vec<prefixity_controlled_benchmark::RawLlamaCppEvidence> {
        Vec::new()
    }
}

#[derive(Default)]
struct OfflineTransport {
    calls: usize,
    bodies: Vec<Vec<u8>>,
}

impl LlamaCppTransport for OfflineTransport {
    fn chat_completion(
        &mut self,
        request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        self.calls += 1;
        self.bodies.push(
            serde_json::to_vec(request)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?,
        );
        Ok(LlamaCppResponse {
            timings: None,
            usage: None,
            raw_telemetry: BTreeMap::new(),
        })
    }
}

impl LiveRawEvidenceSource for OfflineTransport {
    fn raw_evidence(&self) -> Vec<prefixity_controlled_benchmark::RawLlamaCppEvidence> {
        Vec::new()
    }
}

#[test]
fn paired_execution_requires_explicit_opt_in_without_network() {
    let mut transport = CountingTransport::default();
    let error = prefixity_controlled_benchmark::execute_paired_mutation_experiment(
        &prepared(),
        &config(false, true),
        &mut transport,
    )
    .unwrap_err();
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
fn offline_execution_passes_generic_validation_without_a_real_socket() {
    let definition = prepared();
    let experiment = build_paired_mutation_conformance_experiment(&definition).unwrap();
    let expected_bodies = experiment
        .cases
        .iter()
        .map(|case| {
            serde_json::to_vec(
                &project_llama_cpp_request_with_generation_limit(&case.request, 1).unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let mut transport = OfflineTransport::default();
    let record = prefixity_controlled_benchmark::execute_paired_mutation_experiment(
        &definition,
        &config(true, true),
        &mut transport,
    )
    .unwrap();
    assert_eq!(transport.calls, 5);
    assert_eq!(record.completed_steps, 5);
    assert_eq!(record.state, LiveEvidenceState::Normalized);
    assert_eq!(transport.bodies, expected_bodies);
    for body in &transport.bodies {
        let value: serde_json::Value = serde_json::from_slice(body).unwrap();
        assert_eq!(
            value.get("max_tokens").and_then(serde_json::Value::as_u64),
            Some(1)
        );
    }
}

#[test]
fn live_readiness_generation_validator_rejects_an_omitted_bound() {
    let definition = prepared();
    let experiment = build_paired_mutation_conformance_experiment(&definition).unwrap();
    let unbounded = project_llama_cpp_request(&experiment.cases[0].request).unwrap();
    assert!(validate_llama_cpp_generation_limit(&unbounded, 1).is_err());

    let bounded =
        project_llama_cpp_request_with_generation_limit(&experiment.cases[0].request, 1).unwrap();
    validate_llama_cpp_generation_limit(&bounded, 1).unwrap();
    preflight_paired_mutation_experiment(&definition, &config(false, true)).unwrap();
}

#[test]
fn workload_is_bounded_and_uses_small_generation_limit() {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    assert!(seed.workload.control_initial_bytes < 256 * 1024);
    assert_eq!(seed.workload.context_limit, 8192);
    assert_eq!(config(false, true).generation_limit, 1);
}

#[test]
fn outcome_is_falsifiable_and_not_positive_by_construction() {
    let definition = prepared();
    assert_eq!(
        definition.expected_outcome,
        prefixity_controlled_benchmark::PairedOutcomeExpectation::NoRequiredDirection
    );
    assert!(definition
        .primary_comparison
        .interpretation
        .contains("not causal"));
}

#[test]
fn runtime_unknowns_remain_unknown_or_caller_supplied() {
    let value = config(false, true);
    assert!(value.threads.is_none());
    assert!(value.batch_size.is_none());
    assert!(value.kv_cache.is_none());
    assert!(value.gpu_offload.is_none());
    assert_eq!(value.model_identity, "ggml-org/Qwen3.5-0.8B-GGUF");
    assert_eq!(value.quantization.as_deref(), Some("Q4_0"));
}

#[test]
fn serialized_definition_contains_no_result_or_performance_claim() {
    let serialized = serde_json::to_string(&prepared()).unwrap();
    assert!(!serialized.contains("speedup"));
    assert!(!serialized.contains("statistical_significance"));
    assert!(!serialized.contains("performance_improvement"));
    assert!(serialized.contains("not_observed"));
}

#[test]
fn normal_workspace_inputs_are_synthetic_only() {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    let encoded = serde_json::to_string(&seed).unwrap();
    assert!(encoded.contains("synthetic"));
    assert!(!encoded.contains("secret"));
    assert!(!encoded.contains("private source"));
}

#[test]
fn definition_round_trips_deterministically() {
    let definition = prepared();
    let first = definition.canonical_json().unwrap();
    let second = definition.canonical_json().unwrap();
    assert_eq!(first, second);
}

#[test]
fn incomplete_run_cannot_be_complete_or_admitted() {
    let record = PairedMutationRunRecord {
        schema_id: prefixity_controlled_benchmark::PAIRED_MUTATION_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::PAIRED_MUTATION_SCHEMA_VERSION,
        experiment_id: "fixture-paired".to_string(),
        state: LiveEvidenceState::Partial,
        expected_steps: 5,
        completed_steps: 2,
        raw_evidence: Vec::new(),
        normalized_result: None,
        failure: Some("incomplete_sequence".to_string()),
        provenance: BTreeMap::new(),
    };
    assert!(record.validate().is_ok());
    let mut invalid = record;
    invalid.state = LiveEvidenceState::Normalized;
    assert!(invalid.validate().is_err());
}
